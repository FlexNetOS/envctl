from __future__ import annotations

import json
import sqlite3
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from envctl_run_ledger import RunLedger, canonical_json, sha256_json, stable_id


CHECKPOINT_KINDS = {
    "pre_operation",
    "post_operation",
    "artifact_boundary",
    "evidence_boundary",
    "manual",
    "rollback_boundary",
}
ROLLBACK_TYPES = {
    "restore_checkpoint",
    "compensating_operation",
    "remove_added_files",
    "rerun_from_checkpoint",
    "manual_operator",
}
ROLLBACK_STATUSES = {
    "planned",
    "awaiting_approval",
    "running",
    "succeeded",
    "failed",
    "blocked",
    "cancelled",
}
RISK_ORDER = {"R0": 0, "R1": 1, "R2": 2, "R3": 3, "R4": 4, "R5": 5}
BLOCKED_REF_PARTS = {
    ".env",
    "secrets",
    "private_keys",
}
BLOCKED_REF_SUFFIXES = {
    ".pem",
    ".key",
}


class RollbackCheckpointError(ValueError):
    """Raised when checkpoint or rollback data would violate the migration contract."""


@dataclass(frozen=True)
class CheckpointRecord:
    checkpoint_id: str
    run_id: str
    operation_id: str | None
    checkpoint_kind: str
    checkpoint_ref: str
    checkpoint_hash: str
    metadata: dict[str, Any]
    inserted: bool


@dataclass(frozen=True)
class RollbackHandle:
    rollback_id: str
    run_id: str
    operation_id: str | None
    rollback_type: str
    status: str
    plan: dict[str, Any]
    result: dict[str, Any] | None


def checkpoint_hash(checkpoint_kind: str, checkpoint_ref: str, metadata: dict[str, Any] | None = None) -> str:
    return sha256_json(
        {
            "checkpoint_kind": checkpoint_kind,
            "checkpoint_ref": checkpoint_ref,
            "metadata": metadata or {},
        }
    )


def validate_checkpoint_ref(checkpoint_ref: str) -> None:
    if not isinstance(checkpoint_ref, str) or not checkpoint_ref.strip():
        raise RollbackCheckpointError("checkpoint_ref must be a non-empty string")
    normalized = checkpoint_ref.replace("\\", "/")
    parts = {part for part in normalized.split("/") if part}
    lower_parts = {part.lower() for part in parts}
    if ".." in parts:
        raise RollbackCheckpointError("checkpoint_ref must not contain parent-directory traversal")
    if lower_parts & BLOCKED_REF_PARTS:
        raise RollbackCheckpointError("checkpoint_ref points at a blocked secret/private-key path")
    if any(normalized.lower().endswith(suffix) for suffix in BLOCKED_REF_SUFFIXES):
        raise RollbackCheckpointError("checkpoint_ref points at a blocked key material path")


def _decode_json(value: str | None, fallback: Any) -> Any:
    if not value:
        return fallback
    return json.loads(value)


class RollbackCheckpointStore:
    def __init__(self, conn: sqlite3.Connection, base: Path | None = None):
        self.conn = conn
        self.base = base
        self.conn.execute("PRAGMA foreign_keys = ON")
        self.conn.row_factory = sqlite3.Row

    def record_checkpoint(
        self,
        *,
        run_id: str,
        checkpoint_kind: str,
        checkpoint_ref: str,
        operation_id: str | None = None,
        metadata: dict[str, Any] | None = None,
        checkpoint_id: str | None = None,
        actor_id: str = "envctl-db-agent",
    ) -> CheckpointRecord:
        self._require_value("checkpoint_kind", checkpoint_kind, CHECKPOINT_KINDS)
        validate_checkpoint_ref(checkpoint_ref)
        metadata = metadata or {}
        self._require_json_object("metadata", metadata)
        self._require_run(run_id)
        if operation_id:
            self._require_operation_for_run(run_id, operation_id)
        digest = checkpoint_hash(checkpoint_kind, checkpoint_ref, metadata)
        checkpoint_id = checkpoint_id or f"checkpoint-{stable_id(run_id, operation_id or '-', checkpoint_kind, checkpoint_ref, digest)}"
        row = {
            "id": checkpoint_id,
            "run_id": run_id,
            "operation_id": operation_id,
            "checkpoint_kind": checkpoint_kind,
            "checkpoint_ref": checkpoint_ref,
            "checkpoint_hash": digest,
            "metadata_json": canonical_json(metadata),
        }
        try:
            self.conn.execute(
                """
                INSERT INTO envctl_migration_checkpoints
                  (id, run_id, operation_id, checkpoint_kind, checkpoint_ref,
                   checkpoint_hash, metadata_json)
                VALUES
                  (:id, :run_id, :operation_id, :checkpoint_kind, :checkpoint_ref,
                   :checkpoint_hash, :metadata_json)
                """,
                row,
            )
            self.conn.commit()
            inserted = True
            RunLedger(self.conn).append_event(
                run_id=run_id,
                event_type="checkpoint_recorded",
                actor_type="agent",
                actor_id=actor_id,
                operation_id=operation_id,
                payload={
                    "checkpoint_id": checkpoint_id,
                    "checkpoint_kind": checkpoint_kind,
                    "checkpoint_ref": checkpoint_ref,
                    "checkpoint_hash": digest,
                },
                evidence_refs=[checkpoint_ref],
            )
        except sqlite3.IntegrityError:
            existing = self._checkpoint_row(checkpoint_id)
            if not existing:
                raise
            if (
                existing["run_id"] != run_id
                or existing["operation_id"] != operation_id
                or existing["checkpoint_kind"] != checkpoint_kind
                or existing["checkpoint_ref"] != checkpoint_ref
                or existing["checkpoint_hash"] != digest
            ):
                raise RollbackCheckpointError(f"checkpoint id collision with different content: {checkpoint_id}") from None
            inserted = False
        return CheckpointRecord(
            checkpoint_id=checkpoint_id,
            run_id=run_id,
            operation_id=operation_id,
            checkpoint_kind=checkpoint_kind,
            checkpoint_ref=checkpoint_ref,
            checkpoint_hash=digest,
            metadata=metadata,
            inserted=inserted,
        )

    def plan_rollback(
        self,
        *,
        run_id: str,
        rollback_type: str,
        checkpoint_id: str,
        operation_id: str | None = None,
        reason: str,
        requested_by: str = "envctl-db-agent",
        rollback_id: str | None = None,
        instructions: dict[str, Any] | None = None,
    ) -> RollbackHandle:
        self._require_value("rollback_type", rollback_type, ROLLBACK_TYPES)
        if not reason.strip():
            raise RollbackCheckpointError("rollback reason must be non-empty")
        checkpoint = self._require_checkpoint_for_run(run_id, checkpoint_id)
        if operation_id:
            self._require_operation_for_run(run_id, operation_id)
        elif checkpoint["operation_id"]:
            operation_id = checkpoint["operation_id"]
        operation = self._operation_row(operation_id) if operation_id else None
        approval_required = bool(operation and RISK_ORDER[operation["risk"]] >= RISK_ORDER["R3"])
        status = "awaiting_approval" if approval_required else "planned"
        plan = {
            "schema_version": "1.0",
            "rollback_type": rollback_type,
            "reason": reason,
            "requested_by": requested_by,
            "approval_required": approval_required,
            "boundary": {
                "run_id": run_id,
                "operation_id": operation_id,
                "operation_risk": operation["risk"] if operation else None,
            },
            "checkpoint": {
                "id": checkpoint["id"],
                "kind": checkpoint["checkpoint_kind"],
                "ref": checkpoint["checkpoint_ref"],
                "hash": checkpoint["checkpoint_hash"],
                "metadata": _decode_json(checkpoint["metadata_json"], {}),
            },
            "instructions": instructions or {},
            "non_destructive": True,
        }
        rollback_id = rollback_id or f"rollback-{stable_id(run_id, operation_id or '-', rollback_type, checkpoint_id, sha256_json(plan))}"
        self.conn.execute(
            """
            INSERT INTO envctl_migration_rollbacks
              (id, run_id, operation_id, rollback_type, status, plan_json, result_json)
            VALUES (?, ?, ?, ?, ?, ?, NULL)
            """,
            (rollback_id, run_id, operation_id, rollback_type, status, canonical_json(plan)),
        )
        if approval_required:
            self.conn.execute(
                """
                INSERT INTO envctl_migration_approvals
                  (id, run_id, operation_id, risk, status, requested_by, reason)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    f"approval-{stable_id(rollback_id, 'rollback')}",
                    run_id,
                    operation_id,
                    operation["risk"],
                    "open",
                    requested_by,
                    f"rollback:{rollback_id}:{reason}",
                ),
            )
        self.conn.commit()
        RunLedger(self.conn).append_event(
            run_id=run_id,
            event_type="rollback_planned",
            actor_type="agent",
            actor_id=requested_by,
            operation_id=operation_id,
            payload={"rollback_id": rollback_id, "status": status, "checkpoint_id": checkpoint_id},
            evidence_refs=[checkpoint["checkpoint_ref"]],
        )
        return self.fetch_rollback(rollback_id)

    def approve_rollback(self, rollback_id: str, *, decided_by: str, reason: str = "") -> RollbackHandle:
        handle = self.fetch_rollback(rollback_id)
        if handle.status != "awaiting_approval":
            raise RollbackCheckpointError(f"rollback {rollback_id} is not awaiting approval")
        self.conn.execute(
            """
            UPDATE envctl_migration_approvals
            SET status = 'approved', decided_by = ?, reason = COALESCE(NULLIF(?, ''), reason),
                decided_at_utc = strftime('%Y-%m-%dT%H:%M:%fZ','now')
            WHERE status = 'open' AND reason LIKE ?
            """,
            (decided_by, reason, f"rollback:{rollback_id}:%"),
        )
        self.conn.execute(
            "UPDATE envctl_migration_rollbacks SET status = 'planned' WHERE id = ?",
            (rollback_id,),
        )
        self.conn.commit()
        RunLedger(self.conn).append_event(
            run_id=handle.run_id,
            event_type="rollback_approved",
            actor_type="human",
            actor_id=decided_by,
            operation_id=handle.operation_id,
            payload={"rollback_id": rollback_id, "reason": reason},
        )
        return self.fetch_rollback(rollback_id)

    def set_rollback_status(
        self,
        rollback_id: str,
        status: str,
        *,
        actor_id: str = "envctl-db-agent",
        result: dict[str, Any] | None = None,
    ) -> RollbackHandle:
        self._require_value("rollback_status", status, ROLLBACK_STATUSES)
        handle = self.fetch_rollback(rollback_id)
        allowed = {
            "planned": {"running", "blocked", "cancelled"},
            "awaiting_approval": {"blocked", "cancelled"},
            "running": {"succeeded", "failed", "blocked", "cancelled"},
            "blocked": {"planned", "cancelled"},
            "succeeded": set(),
            "failed": set(),
            "cancelled": set(),
        }
        if status not in allowed[handle.status]:
            raise RollbackCheckpointError(f"illegal rollback transition: {handle.status} -> {status}")
        if result is not None:
            self._require_json_object("result", result)
        self.conn.execute(
            """
            UPDATE envctl_migration_rollbacks
            SET status = ?, result_json = COALESCE(?, result_json)
            WHERE id = ?
            """,
            (status, canonical_json(result) if result is not None else None, rollback_id),
        )
        self.conn.commit()
        RunLedger(self.conn).append_event(
            run_id=handle.run_id,
            event_type=f"rollback_{status}",
            actor_type="agent",
            actor_id=actor_id,
            operation_id=handle.operation_id,
            payload={"rollback_id": rollback_id, "status": status, "result": result or {}},
        )
        return self.fetch_rollback(rollback_id)

    def fetch_rollback(self, rollback_id: str) -> RollbackHandle:
        row = self.conn.execute(
            """
            SELECT id, run_id, operation_id, rollback_type, status, plan_json, result_json
            FROM envctl_migration_rollbacks
            WHERE id = ?
            """,
            (rollback_id,),
        ).fetchone()
        if row is None:
            raise RollbackCheckpointError(f"unknown rollback handle: {rollback_id}")
        return RollbackHandle(
            rollback_id=row["id"],
            run_id=row["run_id"],
            operation_id=row["operation_id"],
            rollback_type=row["rollback_type"],
            status=row["status"],
            plan=_decode_json(row["plan_json"], {}),
            result=_decode_json(row["result_json"], None),
        )

    def list_checkpoints(self, run_id: str, operation_id: str | None = None) -> list[dict[str, Any]]:
        params: list[Any] = [run_id]
        where = "run_id = ?"
        if operation_id is not None:
            where += " AND operation_id = ?"
            params.append(operation_id)
        return [
            {
                **dict(row),
                "metadata": _decode_json(row["metadata_json"], {}),
            }
            for row in self.conn.execute(
                f"""
                SELECT id, run_id, operation_id, checkpoint_kind, checkpoint_ref,
                       checkpoint_hash, metadata_json, created_at_utc
                FROM envctl_migration_checkpoints
                WHERE {where}
                ORDER BY created_at_utc, id
                """,
                params,
            )
        ]

    def _checkpoint_row(self, checkpoint_id: str) -> sqlite3.Row | None:
        return self.conn.execute(
            """
            SELECT id, run_id, operation_id, checkpoint_kind, checkpoint_ref,
                   checkpoint_hash, metadata_json
            FROM envctl_migration_checkpoints
            WHERE id = ?
            """,
            (checkpoint_id,),
        ).fetchone()

    def _require_checkpoint_for_run(self, run_id: str, checkpoint_id: str) -> sqlite3.Row:
        row = self._checkpoint_row(checkpoint_id)
        if row is None:
            raise RollbackCheckpointError(f"unknown checkpoint: {checkpoint_id}")
        if row["run_id"] != run_id:
            raise RollbackCheckpointError(f"checkpoint {checkpoint_id} does not belong to run {run_id}")
        return row

    def _operation_row(self, operation_id: str | None) -> sqlite3.Row | None:
        if operation_id is None:
            return None
        return self.conn.execute(
            "SELECT id, run_id, risk, status FROM envctl_migration_operations WHERE id = ?",
            (operation_id,),
        ).fetchone()

    def _require_operation_for_run(self, run_id: str, operation_id: str) -> sqlite3.Row:
        row = self._operation_row(operation_id)
        if row is None:
            raise RollbackCheckpointError(f"unknown operation: {operation_id}")
        if row["run_id"] != run_id:
            raise RollbackCheckpointError(f"operation {operation_id} does not belong to run {run_id}")
        return row

    def _require_run(self, run_id: str) -> None:
        row = self.conn.execute("SELECT id FROM envctl_migration_runs WHERE id = ?", (run_id,)).fetchone()
        if row is None:
            raise RollbackCheckpointError(f"unknown run: {run_id}")

    @staticmethod
    def _require_value(name: str, value: str, allowed: set[str]) -> None:
        if value not in allowed:
            raise RollbackCheckpointError(f"{name} must be one of {sorted(allowed)}")

    @staticmethod
    def _require_json_object(name: str, value: dict[str, Any]) -> None:
        if not isinstance(value, dict):
            raise RollbackCheckpointError(f"{name} must be a JSON object")
        canonical_json(value)
