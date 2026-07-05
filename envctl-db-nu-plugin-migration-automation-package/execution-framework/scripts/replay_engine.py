from __future__ import annotations

import json
import sqlite3
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from _common import now, package_root, root, sha256_file
from envctl_run_ledger import RunLedger, canonical_json, sha256_json, stable_id


REPLAY_MODES = {"dry_run", "apply"}
BLOCKED_REF_PARTS = {".env", "secrets", "private_keys"}
BLOCKED_REF_SUFFIXES = {".pem", ".key"}
NON_DETERMINISTIC_OPERATION_TYPES = {
    "external.apply",
    "manual_operator",
    "shell.exec",
    "target.mutate",
}


class ReplayError(ValueError):
    """Raised when a replay request or stored run cannot be reproduced safely."""


@dataclass(frozen=True)
class ReplayRequest:
    replay_id: str
    run_id: str
    mode: str
    requested_by: str
    operation_ids: list[str]
    target_descriptor_id: str | None = None
    reason: str | None = None


def is_blocked_ref(uri: str) -> bool:
    normalized = uri.replace("\\", "/")
    parts = {part.lower() for part in normalized.split("/") if part}
    return bool(parts & BLOCKED_REF_PARTS) or any(
        normalized.lower().endswith(suffix) for suffix in BLOCKED_REF_SUFFIXES
    )


def resolve_package_file(uri: str, base: Path | None = None) -> Path | None:
    if is_blocked_ref(uri):
        return None
    candidate = Path(uri)
    base = base or package_root()
    if not candidate.is_absolute():
        if uri.startswith("execution-framework/"):
            candidate = base / uri
        else:
            package_candidate = base / uri
            framework_candidate = root() / uri
            candidate = package_candidate if package_candidate.exists() else framework_candidate
    try:
        candidate.resolve().relative_to(base.resolve())
    except ValueError:
        return None
    return candidate


def parse_request(raw: dict[str, Any]) -> ReplayRequest:
    missing = [key for key in ("replay_id", "run_id", "mode", "requested_by") if not raw.get(key)]
    if missing:
        raise ReplayError(f"replay request missing required fields: {', '.join(missing)}")
    mode = str(raw["mode"])
    if mode not in REPLAY_MODES:
        raise ReplayError(f"invalid replay mode: {mode}")
    operation_ids = [str(item) for item in raw.get("operation_ids", [])]
    return ReplayRequest(
        replay_id=str(raw["replay_id"]),
        run_id=str(raw["run_id"]),
        mode=mode,
        requested_by=str(raw["requested_by"]),
        operation_ids=operation_ids,
        target_descriptor_id=raw.get("target_descriptor_id"),
        reason=raw.get("reason"),
    )


def load_recipe_operations(recipe_json: str) -> list[dict[str, Any]]:
    recipe = json.loads(recipe_json)
    operations: list[dict[str, Any]] = []
    for phase in recipe.get("phases", []):
        phase_id = str(phase.get("phase_id", ""))
        approval_gate = bool(phase.get("approval_gate", False))
        for op in phase.get("operations", []):
            operations.append(
                {
                    "phase_id": phase_id,
                    "approval_gate": approval_gate,
                    "operation_id": str(op.get("operation_id", "")),
                    "operation_type": str(op.get("operation_type", "")),
                    "risk": str(op.get("risk", "")),
                    "expected_artifacts": list(op.get("expected_artifacts", [])),
                    "validators": list(op.get("validators", [])),
                }
            )
    return operations


def file_hash_check(uri: str, expected_sha256: str | None, base: Path | None = None) -> dict[str, Any]:
    check = {
        "uri": uri,
        "expected_sha256": expected_sha256,
        "actual_sha256": None,
        "status": "missing_hash" if not expected_sha256 else "missing",
        "blocked": is_blocked_ref(uri),
    }
    if check["blocked"]:
        check["status"] = "blocked_ref"
        return check
    if not expected_sha256:
        return check
    path = resolve_package_file(uri, base)
    if path is None:
        check["status"] = "out_of_scope"
        return check
    if not path.exists() or not path.is_file():
        return check
    actual = "sha256:" + sha256_file(path)
    check["actual_sha256"] = actual
    check["status"] = "match" if actual == expected_sha256 else "mismatch"
    return check


class ReplayEngine:
    def __init__(self, conn: sqlite3.Connection, base: Path | None = None):
        self.conn = conn
        self.conn.row_factory = sqlite3.Row
        self.base = base or package_root()
        self.ledger = RunLedger(conn)

    def replay(self, request: ReplayRequest) -> dict[str, Any]:
        run = self._require_run(request.run_id)
        target = self._require_row(
            "SELECT * FROM envctl_migration_targets WHERE id = ?",
            (run["target_id"],),
            f"unknown target for run: {run['target_id']}",
        )
        recipe = self._require_row(
            "SELECT * FROM envctl_migration_recipes WHERE id = ?",
            (run["recipe_id"],),
            f"unknown recipe for run: {run['recipe_id']}",
        )
        contract = self._require_row(
            "SELECT * FROM envctl_migration_artifact_contracts WHERE id = ?",
            (run["artifact_contract_id"],),
            f"unknown artifact contract for run: {run['artifact_contract_id']}",
        )
        operations = self._operations(request.run_id, request.operation_ids)
        evidence_checks = self._evidence_checks(request.run_id, request.operation_ids)
        artifact_checks = self._artifact_checks(request.run_id, request.operation_ids)
        event_chain = self.ledger.validate_event_chain(request.run_id)
        open_approvals = self._open_approvals(request.run_id, request.operation_ids)
        checkpoints = self._checkpoints(request.run_id, request.operation_ids)
        recipe_operations = load_recipe_operations(recipe["recipe_json"])
        replay_input = {
            "target_descriptor": {
                "id": target["id"],
                "target_id": target["target_id"],
                "descriptor_hash": target["descriptor_hash"],
                "safety_mode": target["safety_mode"],
                "max_auto_risk": target["max_auto_risk"],
            },
            "artifact_contract": {
                "id": contract["id"],
                "contract_hash": contract["contract_hash"],
            },
            "recipe": {
                "id": recipe["id"],
                "recipe_hash": recipe["recipe_hash"],
                "operation_count": len(recipe_operations),
            },
            "run": {
                "id": run["id"],
                "status": run["status"],
                "human_mode": run["human_mode"],
                "sandbox_policy": run["sandbox_policy"],
                "approval_policy": run["approval_policy"],
                "tool_versions": json.loads(run["tool_versions_json"] or "{}"),
                "reproducibility_hash": run["reproducibility_hash"],
            },
            "operation_ids": [item["id"] for item in operations],
            "event_chain_head": event_chain["head_event_hash"],
        }
        recomputed_reproducibility_hash = sha256_json(
            {
                "run_id": run["id"],
                "target_id": run["target_id"],
                "recipe_id": run["recipe_id"],
                "artifact_contract_id": run["artifact_contract_id"],
                "tool_versions": json.loads(run["tool_versions_json"] or "{}"),
            }
        )
        hash_status = {
            "reproducibility_hash_matches": recomputed_reproducibility_hash == run["reproducibility_hash"],
            "stored_reproducibility_hash": run["reproducibility_hash"],
            "recomputed_reproducibility_hash": recomputed_reproducibility_hash,
            "evidence_matches": sum(1 for item in evidence_checks if item["status"] == "match"),
            "evidence_mismatches": [item for item in evidence_checks if item["status"] == "mismatch"],
            "evidence_missing": [item for item in evidence_checks if item["status"] in {"missing", "missing_hash"}],
            "artifact_matches": sum(1 for item in artifact_checks if item["status"] == "match"),
            "artifact_mismatches": [item for item in artifact_checks if item["status"] == "mismatch"],
            "artifact_missing": [item for item in artifact_checks if item["status"] in {"missing", "missing_hash"}],
            "blocked_refs": [
                item
                for item in [*evidence_checks, *artifact_checks]
                if item["status"] in {"blocked_ref", "out_of_scope"}
            ],
        }
        non_deterministic = self._non_deterministic_operations(operations)
        readiness = self._readiness_row(request.run_id)
        required_approvals = [
            {
                "approval_id": row["approval_id"],
                "operation_id": row["operation_id"],
                "risk": row["risk"],
                "reason": row["reason"],
            }
            for row in open_approvals
        ]
        errors = list(event_chain["errors"])
        if not hash_status["reproducibility_hash_matches"]:
            errors.append("run reproducibility hash mismatch")
        if hash_status["evidence_mismatches"]:
            errors.append("evidence hash mismatch")
        if hash_status["artifact_mismatches"]:
            errors.append("artifact hash mismatch")
        if hash_status["blocked_refs"]:
            errors.append("blocked or out-of-scope replay reference")
        if request.mode == "apply" and required_approvals:
            errors.append("apply replay requires closed approvals")
        if request.mode == "apply" and non_deterministic:
            errors.append("apply replay requires manual handling for non-deterministic operations")
        status = self._status(request, errors, hash_status, event_chain, required_approvals, non_deterministic)
        safe_next_action = self._safe_next_action(request, status, required_approvals, non_deterministic, hash_status)
        return {
            "schema_version": "1.0",
            "replay_id": request.replay_id,
            "run_id": request.run_id,
            "mode": request.mode,
            "requested_by": request.requested_by,
            "completed_at_utc": now(),
            "status": status,
            "replay_input_hash": sha256_json(replay_input),
            "replay_input": replay_input,
            "readiness": readiness,
            "event_chain": event_chain,
            "recipe_operations": recipe_operations,
            "operation_replay_plan": self._operation_plan(operations, recipe_operations, checkpoints),
            "hash_status": hash_status,
            "missing_evidence": hash_status["evidence_missing"],
            "non_deterministic_operations": non_deterministic,
            "required_approvals": required_approvals,
            "safe_next_action": safe_next_action,
            "errors": errors,
        }

    def _require_run(self, run_id: str) -> sqlite3.Row:
        return self._require_row("SELECT * FROM envctl_migration_runs WHERE id = ?", (run_id,), f"unknown run: {run_id}")

    def _require_row(self, query: str, params: tuple[Any, ...], message: str) -> sqlite3.Row:
        row = self.conn.execute(query, params).fetchone()
        if row is None:
            raise ReplayError(message)
        return row

    def _operations(self, run_id: str, operation_ids: list[str]) -> list[sqlite3.Row]:
        params: list[Any] = [run_id]
        where = "run_id = ?"
        if operation_ids:
            where += " AND id IN ({})".format(",".join("?" for _ in operation_ids))
            params.extend(operation_ids)
        rows = self.conn.execute(
            f"""
            SELECT *
            FROM envctl_migration_operations
            WHERE {where}
            ORDER BY created_at_utc, id
            """,
            params,
        ).fetchall()
        found = {row["id"] for row in rows}
        missing = sorted(set(operation_ids) - found)
        if missing:
            raise ReplayError(f"unknown operation ids for replay: {', '.join(missing)}")
        return rows

    def _evidence_checks(self, run_id: str, operation_ids: list[str]) -> list[dict[str, Any]]:
        params: list[Any] = [run_id]
        where = "run_id = ?"
        if operation_ids:
            where += " AND operation_id IN ({})".format(",".join("?" for _ in operation_ids))
            params.extend(operation_ids)
        rows = self.conn.execute(
            f"""
            SELECT uri, sha256, evidence_kind, operation_id
            FROM envctl_migration_evidence
            WHERE {where}
            ORDER BY created_at_utc, id
            """,
            params,
        ).fetchall()
        checks = []
        for row in rows:
            check = file_hash_check(row["uri"], row["sha256"], self.base)
            check.update({"kind": row["evidence_kind"], "operation_id": row["operation_id"]})
            checks.append(check)
        return checks

    def _artifact_checks(self, run_id: str, operation_ids: list[str]) -> list[dict[str, Any]]:
        params: list[Any] = [run_id]
        where = "run_id = ?"
        if operation_ids:
            where += " AND generated_by_operation_id IN ({})".format(",".join("?" for _ in operation_ids))
            params.extend(operation_ids)
        rows = self.conn.execute(
            f"""
            SELECT artifact_id, path, content_hash, generated_by_operation_id
            FROM envctl_migration_artifacts
            WHERE {where}
            ORDER BY created_at_utc, id
            """,
            params,
        ).fetchall()
        checks = []
        for row in rows:
            if not row["path"]:
                checks.append(
                    {
                        "uri": None,
                        "artifact_id": row["artifact_id"],
                        "operation_id": row["generated_by_operation_id"],
                        "expected_sha256": row["content_hash"],
                        "actual_sha256": None,
                        "status": "missing_path",
                        "blocked": False,
                    }
                )
                continue
            check = file_hash_check(row["path"], row["content_hash"], self.base)
            check.update({"artifact_id": row["artifact_id"], "operation_id": row["generated_by_operation_id"]})
            checks.append(check)
        return checks

    def _open_approvals(self, run_id: str, operation_ids: list[str]) -> list[sqlite3.Row]:
        params: list[Any] = [run_id]
        where = "run_id = ?"
        if operation_ids:
            where += " AND operation_id IN ({})".format(",".join("?" for _ in operation_ids))
            params.extend(operation_ids)
        return self.conn.execute(
            f"""
            SELECT *
            FROM envctl_migration_open_approvals
            WHERE {where}
            ORDER BY requested_at_utc, approval_id
            """,
            params,
        ).fetchall()

    def _checkpoints(self, run_id: str, operation_ids: list[str]) -> list[dict[str, Any]]:
        params: list[Any] = [run_id]
        where = "run_id = ?"
        if operation_ids:
            where += " AND operation_id IN ({})".format(",".join("?" for _ in operation_ids))
            params.extend(operation_ids)
        return [
            {
                "checkpoint_id": row["id"],
                "operation_id": row["operation_id"],
                "checkpoint_kind": row["checkpoint_kind"],
                "checkpoint_ref": row["checkpoint_ref"],
                "checkpoint_hash": row["checkpoint_hash"],
                "metadata": json.loads(row["metadata_json"] or "{}"),
            }
            for row in self.conn.execute(
                f"""
                SELECT id, operation_id, checkpoint_kind, checkpoint_ref, checkpoint_hash, metadata_json
                FROM envctl_migration_checkpoints
                WHERE {where}
                ORDER BY created_at_utc, id
                """,
                params,
            )
        ]

    def _readiness_row(self, run_id: str) -> dict[str, Any]:
        row = self._require_row(
            "SELECT * FROM envctl_migration_replay_readiness WHERE run_id = ?",
            (run_id,),
            f"missing replay readiness row for run: {run_id}",
        )
        return dict(row)

    def _non_deterministic_operations(self, operations: list[sqlite3.Row]) -> list[dict[str, Any]]:
        out = []
        for row in operations:
            input_json = json.loads(row["input_json"] or "{}")
            if row["operation_type"] in NON_DETERMINISTIC_OPERATION_TYPES or input_json.get("non_deterministic"):
                out.append(
                    {
                        "operation_id": row["id"],
                        "operation_type": row["operation_type"],
                        "risk": row["risk"],
                        "reason": input_json.get("replay_note", "operation requires external or manual state"),
                    }
                )
        return out

    def _operation_plan(
        self,
        operations: list[sqlite3.Row],
        recipe_operations: list[dict[str, Any]],
        checkpoints: list[dict[str, Any]],
    ) -> list[dict[str, Any]]:
        recipe_by_type = {item["operation_type"]: item for item in recipe_operations}
        checkpoints_by_op: dict[str, list[dict[str, Any]]] = {}
        for checkpoint in checkpoints:
            checkpoints_by_op.setdefault(checkpoint["operation_id"], []).append(checkpoint)
        plan = []
        for row in operations:
            recipe_op = recipe_by_type.get(row["operation_type"], {})
            plan.append(
                {
                    "operation_id": row["id"],
                    "operation_type": row["operation_type"],
                    "phase": row["phase"],
                    "status": row["status"],
                    "risk": row["risk"],
                    "idempotency_key": row["idempotency_key"],
                    "command_hash": row["command_hash"],
                    "output_ref": row["output_ref"],
                    "recipe_phase": recipe_op.get("phase_id"),
                    "approval_gate": bool(recipe_op.get("approval_gate")),
                    "checkpoint_refs": checkpoints_by_op.get(row["id"], []),
                    "replay_action": "verify_only" if row["status"] == "succeeded" else "resume_from_checkpoint",
                }
            )
        return plan

    @staticmethod
    def _status(
        request: ReplayRequest,
        errors: list[str],
        hash_status: dict[str, Any],
        event_chain: dict[str, Any],
        required_approvals: list[dict[str, Any]],
        non_deterministic: list[dict[str, Any]],
    ) -> str:
        if errors and request.mode == "apply":
            return "blocked"
        if hash_status["evidence_missing"] or hash_status["artifact_missing"] or required_approvals or non_deterministic:
            return "partial" if event_chain["chain_valid"] and not errors else "blocked"
        return "pass" if not errors else "fail"

    @staticmethod
    def _safe_next_action(
        request: ReplayRequest,
        status: str,
        required_approvals: list[dict[str, Any]],
        non_deterministic: list[dict[str, Any]],
        hash_status: dict[str, Any],
    ) -> str:
        if hash_status["evidence_mismatches"] or hash_status["artifact_mismatches"]:
            return "stop: inspect hash mismatches before replay"
        if hash_status["blocked_refs"]:
            return "stop: remove blocked or out-of-scope replay references"
        if required_approvals:
            return "request human approval before apply replay"
        if non_deterministic:
            return "dry-run only: manual/non-deterministic operations require operator handling"
        if request.mode == "dry_run" and status in {"pass", "partial"}:
            return "safe to produce replay command plan; apply remains approval-gated"
        if request.mode == "apply" and status == "pass":
            return "safe to re-run deterministic operations from recorded descriptors and hashes"
        return "stop: replay result is not clean"


def build_replay_command_summary(request: ReplayRequest) -> list[str]:
    command = [
        "envctl",
        "replay",
        request.mode.replace("_", "-"),
        "--run-id",
        request.run_id,
        "--replay-id",
        request.replay_id,
        "--requested-by",
        request.requested_by,
    ]
    if request.operation_ids:
        command.extend(["--operation-ids", ",".join(request.operation_ids)])
    if request.reason:
        command.extend(["--reason", request.reason])
    return [" ".join(command), "python3 scripts/verify_replay_engine.py"]
