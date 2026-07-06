from __future__ import annotations

import json
import hashlib
import sqlite3
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from _common import now, package_root, sha256_file


MIGRATION_FILES = [
    "sql/001_migration_automation_schema.sql",
    "sql/002_views_and_indexes.sql",
    "execution-framework/generated/contract_manifest.seed.sql",
]

RUN_STATUSES = {
    "created",
    "planning",
    "awaiting_approval",
    "running",
    "paused",
    "validating",
    "completed",
    "failed",
    "blocked",
    "cancelled",
    "denied",
}
OPERATION_STATUSES = {
    "queued",
    "ready",
    "awaiting_approval",
    "running",
    "succeeded",
    "failed",
    "blocked",
    "denied",
    "cancelled",
}
RISKS = {"R0", "R1", "R2", "R3", "R4", "R5"}
ACTOR_TYPES = {"system", "agent", "human", "plugin", "external"}
HUMAN_MODES = {"observer", "approval-gated", "operator", "agent-only"}


class LedgerError(ValueError):
    """Raised when a run ledger mutation would violate the contract."""


def apply_migrations(conn: sqlite3.Connection, base: Path | None = None) -> list[dict[str, Any]]:
    base = base or package_root()
    applied = []
    for relpath in MIGRATION_FILES:
        path = base / relpath
        sql = path.read_text(encoding="utf-8")
        conn.executescript(sql)
        applied.append(
            {
                "path": relpath,
                "sha256": sha256_file(path),
                "bytes": len(sql.encode("utf-8")),
            }
        )
    return applied


def canonical_json(value: Any) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def sha256_text(value: str) -> str:
    return "sha256:" + hashlib.sha256(value.encode("utf-8")).hexdigest()


def sha256_json(value: Any) -> str:
    return sha256_text(canonical_json(value))


def stable_id(*parts: str) -> str:
    body = "|".join(parts)
    return hashlib.sha256(body.encode("utf-8")).hexdigest()[:24]


@dataclass(frozen=True)
class OperationRecord:
    operation_id: str
    run_id: str
    operation_type: str
    status: str
    risk: str
    idempotency_key: str
    parent_operation_id: str | None = None
    phase: str | None = None
    command_redacted: str | None = None
    input: dict[str, Any] | None = None
    output_ref: str | None = None
    error: dict[str, Any] | None = None


@dataclass(frozen=True)
class EventRecord:
    event_id: str
    run_id: str
    event_seq: int
    event_type: str
    timestamp_utc: str
    phase: str | None
    actor_type: str
    actor_id: str | None
    operation_id: str | None
    payload: dict[str, Any]
    evidence_refs: list[str]
    previous_event_hash: str | None
    event_hash: str


class RunLedger:
    def __init__(self, conn: sqlite3.Connection):
        self.conn = conn
        self.conn.execute("PRAGMA foreign_keys = ON")

    def seed_base_catalog(self) -> dict[str, str]:
        target_id = "target-req022"
        package_id = "pkg-req022"
        contract_id = "contract-req022"
        recipe_id = "recipe-req022"
        self.conn.execute(
            """
            INSERT INTO envctl_migration_targets
              (id, target_id, target_type, primary_root, compare_root, descriptor_json,
               descriptor_hash, safety_mode, max_auto_risk)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                target_id,
                "flexnetos-run-ledger",
                "mixed",
                "/workspace/flexnetos",
                "/workspace/lifeos",
                canonical_json({"schema_version": 1, "target": "flexnetos-run-ledger"}),
                sha256_text("target-req022"),
                "approval-gated",
                "R3",
            ),
        )
        self.conn.execute(
            """
            INSERT INTO envctl_migration_packages
              (id, package_name, package_path, package_hash, manifest_json)
            VALUES (?, ?, ?, ?, ?)
            """,
            (
                package_id,
                "req022-run-ledger-fixture",
                "source/req022-run-ledger-fixture",
                sha256_text("pkg-req022"),
                canonical_json({"schema_version": 1, "task_id": "REQ-022_ENVCTL_RUN_LEDGER"}),
            ),
        )
        self.conn.execute(
            """
            INSERT INTO envctl_migration_artifact_contracts
              (id, contract_name, contract_version, source_package_id, contract_hash, contract_json)
            VALUES (?, ?, ?, ?, ?, ?)
            """,
            (
                contract_id,
                "req022-run-ledger-contract",
                "1.0.0",
                package_id,
                sha256_text("contract-req022"),
                canonical_json({"automation": ["run-ledger", "event-timeline", "proof-links"]}),
            ),
        )
        self.conn.execute(
            """
            INSERT INTO envctl_migration_recipes
              (id, recipe_name, recipe_version, artifact_contract_id, recipe_hash, recipe_json)
            VALUES (?, ?, ?, ?, ?, ?)
            """,
            (
                recipe_id,
                "req022-run-ledger-recipe",
                "1.0.0",
                contract_id,
                sha256_text("recipe-req022"),
                canonical_json({"phases": ["02-envctl-db"], "run_ledger_required": True}),
            ),
        )
        self.conn.commit()
        return {
            "target_id": target_id,
            "package_id": package_id,
            "contract_id": contract_id,
            "recipe_id": recipe_id,
        }

    def create_run(
        self,
        *,
        run_id: str,
        target_id: str,
        recipe_id: str,
        artifact_contract_id: str,
        human_mode: str,
        initiated_by: str,
        sandbox_policy: str,
        approval_policy: str,
        tool_versions: dict[str, Any],
    ) -> dict[str, Any]:
        self._require("human_mode", human_mode, HUMAN_MODES)
        reproducibility_input = {
            "run_id": run_id,
            "target_id": target_id,
            "recipe_id": recipe_id,
            "artifact_contract_id": artifact_contract_id,
            "tool_versions": tool_versions,
        }
        reproducibility_hash = sha256_json(reproducibility_input)
        ts = now()
        self.conn.execute(
            """
            INSERT INTO envctl_migration_runs
              (id, target_id, recipe_id, artifact_contract_id, status, human_mode,
               initiated_by, sandbox_policy, approval_policy, tool_versions_json,
               reproducibility_hash, started_at_utc)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                run_id,
                target_id,
                recipe_id,
                artifact_contract_id,
                "created",
                human_mode,
                initiated_by,
                sandbox_policy,
                approval_policy,
                canonical_json(tool_versions),
                reproducibility_hash,
                ts,
            ),
        )
        self.conn.commit()
        return {
            "run_id": run_id,
            "status": "created",
            "started_at_utc": ts,
            "reproducibility_hash": reproducibility_hash,
        }

    def set_run_status(self, run_id: str, status: str) -> None:
        self._require("run_status", status, RUN_STATUSES)
        completed_at = now() if status in {"completed", "failed", "blocked", "cancelled", "denied"} else None
        cursor = self.conn.execute(
            """
            UPDATE envctl_migration_runs
            SET status = ?, completed_at_utc = COALESCE(?, completed_at_utc)
            WHERE id = ?
            """,
            (status, completed_at, run_id),
        )
        if cursor.rowcount != 1:
            raise LedgerError(f"unknown run: {run_id}")
        self.conn.commit()

    def record_operation(self, op: OperationRecord) -> dict[str, Any]:
        self._require("operation_status", op.status, OPERATION_STATUSES)
        self._require("risk", op.risk, RISKS)
        command_hash = sha256_text(op.command_redacted) if op.command_redacted else None
        self.conn.execute(
            """
            INSERT INTO envctl_migration_operations
              (id, run_id, parent_operation_id, operation_type, phase, status, risk,
               idempotency_key, command_hash, command_redacted, input_json, output_ref, error_json)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                op.operation_id,
                op.run_id,
                op.parent_operation_id,
                op.operation_type,
                op.phase,
                op.status,
                op.risk,
                op.idempotency_key,
                command_hash,
                op.command_redacted,
                canonical_json(op.input or {}),
                op.output_ref,
                canonical_json(op.error) if op.error else None,
            ),
        )
        self.conn.commit()
        return {
            "operation_id": op.operation_id,
            "run_id": op.run_id,
            "status": op.status,
            "risk": op.risk,
            "command_hash": command_hash,
        }

    def set_operation_status(
        self,
        operation_id: str,
        status: str,
        *,
        output_ref: str | None = None,
        error: dict[str, Any] | None = None,
    ) -> None:
        self._require("operation_status", status, OPERATION_STATUSES)
        started_at = now() if status == "running" else None
        completed_at = now() if status in {"succeeded", "failed", "blocked", "denied", "cancelled"} else None
        cursor = self.conn.execute(
            """
            UPDATE envctl_migration_operations
            SET status = ?,
                output_ref = COALESCE(?, output_ref),
                error_json = COALESCE(?, error_json),
                started_at_utc = COALESCE(started_at_utc, ?),
                completed_at_utc = COALESCE(?, completed_at_utc)
            WHERE id = ?
            """,
            (
                status,
                output_ref,
                canonical_json(error) if error else None,
                started_at,
                completed_at,
                operation_id,
            ),
        )
        if cursor.rowcount != 1:
            raise LedgerError(f"unknown operation: {operation_id}")
        self.conn.commit()

    def append_event(
        self,
        *,
        run_id: str,
        event_type: str,
        actor_type: str,
        payload: dict[str, Any],
        phase: str | None = None,
        actor_id: str | None = None,
        operation_id: str | None = None,
        evidence_refs: list[str] | None = None,
        event_seq: int | None = None,
    ) -> EventRecord:
        self._require("actor_type", actor_type, ACTOR_TYPES)
        next_seq, previous_hash = self.next_event_position(run_id)
        if event_seq is not None and event_seq != next_seq:
            raise LedgerError(f"event_seq must append at {next_seq}, got {event_seq}")
        seq = next_seq
        timestamp = now()
        evidence = list(evidence_refs or [])
        hash_input = {
            "run_id": run_id,
            "event_seq": seq,
            "event_type": event_type,
            "phase": phase,
            "actor_type": actor_type,
            "actor_id": actor_id,
            "operation_id": operation_id,
            "timestamp_utc": timestamp,
            "payload": payload,
            "evidence_refs": evidence,
            "previous_event_hash": previous_hash,
        }
        event_hash = sha256_json(hash_input)
        event_id = f"event-{stable_id(run_id, str(seq), event_hash)}"
        self.conn.execute(
            """
            INSERT INTO envctl_migration_run_events
              (id, run_id, event_seq, event_type, phase, actor_type, actor_id,
               operation_id, payload_json, evidence_refs_json, previous_event_hash,
               event_hash, created_at_utc)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                event_id,
                run_id,
                seq,
                event_type,
                phase,
                actor_type,
                actor_id,
                operation_id,
                canonical_json(payload),
                canonical_json(evidence),
                previous_hash,
                event_hash,
                timestamp,
            ),
        )
        self.conn.commit()
        return EventRecord(
            event_id=event_id,
            run_id=run_id,
            event_seq=seq,
            event_type=event_type,
            timestamp_utc=timestamp,
            phase=phase,
            actor_type=actor_type,
            actor_id=actor_id,
            operation_id=operation_id,
            payload=payload,
            evidence_refs=evidence,
            previous_event_hash=previous_hash,
            event_hash=event_hash,
        )

    def link_evidence(
        self,
        *,
        run_id: str,
        uri: str,
        evidence_kind: str,
        operation_id: str | None = None,
        sha256: str | None = None,
        redacted: bool = False,
        metadata: dict[str, Any] | None = None,
    ) -> dict[str, Any]:
        evidence_id = f"evidence-{stable_id(run_id, uri, evidence_kind)}"
        self.conn.execute(
            """
            INSERT INTO envctl_migration_evidence
              (id, run_id, operation_id, uri, evidence_kind, sha256, redacted, metadata_json)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                evidence_id,
                run_id,
                operation_id,
                uri,
                evidence_kind,
                sha256,
                1 if redacted else 0,
                canonical_json(metadata or {}),
            ),
        )
        self.conn.commit()
        return {
            "evidence_id": evidence_id,
            "run_id": run_id,
            "operation_id": operation_id,
            "uri": uri,
            "evidence_kind": evidence_kind,
            "sha256": sha256,
        }

    def next_event_position(self, run_id: str) -> tuple[int, str | None]:
        row = self.conn.execute(
            """
            SELECT event_seq, event_hash
            FROM envctl_migration_run_events
            WHERE run_id = ?
            ORDER BY event_seq DESC
            LIMIT 1
            """,
            (run_id,),
        ).fetchone()
        if row is None:
            return 1, None
        return int(row[0]) + 1, row[1]

    def validate_event_chain(self, run_id: str) -> dict[str, Any]:
        rows = self.conn.execute(
            """
            SELECT event_seq, event_type, phase, actor_type, actor_id, operation_id,
                   payload_json, evidence_refs_json, previous_event_hash, event_hash,
                   created_at_utc
            FROM envctl_migration_run_events
            WHERE run_id = ?
            ORDER BY event_seq
            """,
            (run_id,),
        ).fetchall()
        expected_previous = None
        errors = []
        for expected_seq, row in enumerate(rows, start=1):
            (
                event_seq,
                event_type,
                phase,
                actor_type,
                actor_id,
                operation_id,
                payload_json,
                evidence_refs_json,
                previous_event_hash,
                event_hash,
                created_at_utc,
            ) = row
            payload = json.loads(payload_json)
            evidence_refs = json.loads(evidence_refs_json or "[]")
            if event_seq != expected_seq:
                errors.append(f"event sequence gap at {event_seq}, expected {expected_seq}")
            if previous_event_hash != expected_previous:
                errors.append(f"previous hash mismatch at event {event_seq}")
            recomputed = sha256_json(
                {
                    "run_id": run_id,
                    "event_seq": event_seq,
                    "event_type": event_type,
                    "phase": phase,
                    "actor_type": actor_type,
                    "actor_id": actor_id,
                    "operation_id": operation_id,
                    "timestamp_utc": created_at_utc,
                    "payload": payload,
                    "evidence_refs": evidence_refs,
                    "previous_event_hash": previous_event_hash,
                }
            )
            if recomputed != event_hash:
                errors.append(f"event hash mismatch at event {event_seq}")
            expected_previous = event_hash
        return {
            "run_id": run_id,
            "event_count": len(rows),
            "chain_valid": not errors,
            "errors": errors,
            "head_event_hash": expected_previous,
        }

    def run_snapshot(self, run_id: str) -> dict[str, Any]:
        status_row = self.conn.execute(
            "SELECT * FROM envctl_migration_run_latest_status WHERE run_id = ?",
            (run_id,),
        ).fetchone()
        if status_row is None:
            raise LedgerError(f"unknown run: {run_id}")
        timeline = [
            {
                "event_seq": row[0],
                "event_type": row[1],
                "actor_type": row[2],
                "operation_id": row[3],
                "operation_status": row[4],
            }
            for row in self.conn.execute(
                """
                SELECT event_seq, event_type, actor_type, operation_id, operation_status
                FROM envctl_migration_live_timeline
                WHERE run_id = ?
                ORDER BY event_seq
                """,
                (run_id,),
            )
        ]
        operations = [
            {
                "operation_id": row[0],
                "operation_type": row[1],
                "phase": row[2],
                "status": row[3],
                "risk": row[4],
                "output_ref": row[5],
            }
            for row in self.conn.execute(
                """
                SELECT id, operation_type, phase, status, risk, output_ref
                FROM envctl_migration_operations
                WHERE run_id = ?
                ORDER BY created_at_utc, id
                """,
                (run_id,),
            )
        ]
        evidence = [
            {
                "evidence_id": row[0],
                "operation_id": row[1],
                "uri": row[2],
                "evidence_kind": row[3],
                "sha256": row[4],
            }
            for row in self.conn.execute(
                """
                SELECT id, operation_id, uri, evidence_kind, sha256
                FROM envctl_migration_evidence
                WHERE run_id = ?
                ORDER BY created_at_utc, id
                """,
                (run_id,),
            )
        ]
        return {
            "run_id": run_id,
            "status": status_row[3],
            "operation_count": status_row[7],
            "failed_operation_count": status_row[8],
            "open_approval_count": status_row[9],
            "artifact_count": status_row[10],
            "last_event_at_utc": status_row[11],
            "timeline": timeline,
            "operations": operations,
            "evidence": evidence,
            "event_chain": self.validate_event_chain(run_id),
        }

    def _require(self, name: str, value: str, allowed: set[str]) -> None:
        if value not in allowed:
            raise LedgerError(f"invalid {name}: {value}")
