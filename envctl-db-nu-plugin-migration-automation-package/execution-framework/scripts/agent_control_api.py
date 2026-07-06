from __future__ import annotations

import argparse
import json
import sqlite3
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from _common import now, package_root
from envctl_run_ledger import (
    ACTOR_TYPES,
    RISKS,
    OperationRecord as LedgerOperationRecord,
    RunLedger,
    apply_migrations,
    canonical_json,
    sha256_json,
    stable_id,
)
from operation_state_machine import (
    OperationState,
    OperationRecord as StateOperationRecord,
    OperationTransitionError,
    approval_required,
    normalize_state,
    transition,
)


AUTHORITY_LEVELS = {"read_only", "safe_execute", "approval_request", "operator", "admin"}
WRITE_AUTHORITIES = {"safe_execute", "approval_request", "operator", "admin"}
APPROVAL_AUTHORITIES = {"operator", "admin"}
LEASE_AUTHORITIES = {"safe_execute", "operator", "admin"}
CONTROL_EVENT_PREFIX = "agent_control_"


class AgentControlError(ValueError):
    """Raised when a control request violates the agent control protocol."""


@dataclass(frozen=True)
class Actor:
    actor_type: str
    actor_id: str
    authority: str


class AgentControlApi:
    """Database-backed control API for migration agents, helpers, and operators."""

    def __init__(self, conn: sqlite3.Connection):
        conn.row_factory = sqlite3.Row
        self.conn = conn
        self.ledger = RunLedger(conn)

    def run_status(self, run_id: str, *, recent_events: int = 10) -> dict[str, Any]:
        snapshot = self.ledger.run_snapshot(run_id)
        snapshot["approvals"] = self._approval_summary(run_id)
        snapshot["operation_status_counts"] = self._operation_counts(run_id)
        snapshot["visible_locks"] = self._visible_locks(run_id)
        snapshot["recent_events"] = self.run_events(run_id, limit=recent_events)["events"]
        return snapshot

    def run_events(self, run_id: str, *, limit: int | None = None) -> dict[str, Any]:
        self._require_run(run_id)
        params: list[Any] = [run_id]
        sql = """
            SELECT event_seq, event_type, phase, actor_type, actor_id, operation_id,
                   payload_json, evidence_refs_json, previous_event_hash, event_hash,
                   created_at_utc
            FROM envctl_migration_run_events
            WHERE run_id = ?
            ORDER BY event_seq
        """
        if limit is not None:
            if limit < 1:
                raise AgentControlError("limit must be greater than zero")
            sql += " LIMIT ?"
            params.append(limit)
        rows = self.conn.execute(sql, params).fetchall()
        return {
            "run_id": run_id,
            "events": [self._event_row(row) for row in rows],
            "event_chain": self.ledger.validate_event_chain(run_id),
        }

    def enqueue_operation(
        self,
        *,
        run_id: str,
        operation_type: str,
        risk: str,
        actor: Actor,
        phase: str | None = None,
        parent_operation_id: str | None = None,
        recipe_step_id: str | None = None,
        target_scope: str | None = None,
        input_payload: dict[str, Any] | None = None,
        command_redacted: str | None = None,
        reason: str | None = None,
        idempotency_key: str | None = None,
    ) -> dict[str, Any]:
        self._require_run(run_id)
        self._require_actor(actor, WRITE_AUTHORITIES)
        self._require_value("risk", risk, RISKS)
        if not operation_type:
            raise AgentControlError("operation_type is required")

        input_payload = dict(input_payload or {})
        target_descriptor_hash = sha256_json(
            {
                "target_scope": target_scope or "*",
                "operation_type": operation_type,
                "phase": phase,
            }
        )
        input_hash = sha256_json(input_payload)
        idem = idempotency_key or sha256_json(
            {
                "run_id": run_id,
                "operation_type": operation_type,
                "target_descriptor_hash": target_descriptor_hash,
                "recipe_step_id": recipe_step_id or "",
                "input_hash": input_hash,
            }
        )
        operation_id = f"op-{stable_id(run_id, idem)}"
        existing = self.conn.execute(
            """
            SELECT id, status, risk, operation_type
            FROM envctl_migration_operations
            WHERE run_id = ? AND idempotency_key = ?
            """,
            (run_id, idem),
        ).fetchone()
        if existing is not None:
            return {
                "status": "existing",
                "operation_id": existing["id"],
                "operation_status": existing["status"],
                "risk": existing["risk"],
                "operation_type": existing["operation_type"],
                "idempotency_key": idem,
            }

        operation_status = "awaiting_approval" if approval_required(risk) else "ready"
        operation_input = {
            "actor": actor.__dict__,
            "input": input_payload,
            "recipe_step_id": recipe_step_id,
            "target_scope": target_scope,
            "target_descriptor_hash": target_descriptor_hash,
            "input_hash": input_hash,
        }
        self.ledger.record_operation(
            LedgerOperationRecord(
                operation_id=operation_id,
                run_id=run_id,
                parent_operation_id=parent_operation_id,
                operation_type=operation_type,
                phase=phase,
                status=operation_status,
                risk=risk,
                idempotency_key=idem,
                command_redacted=command_redacted,
                input=operation_input,
            )
        )
        approval_id = None
        if approval_required(risk):
            approval_id = f"approval-{stable_id(run_id, operation_id, idem)}"
            self.conn.execute(
                """
                INSERT INTO envctl_migration_approvals
                  (id, run_id, operation_id, risk, status, requested_by, reason)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    approval_id,
                    run_id,
                    operation_id,
                    risk,
                    "open",
                    actor.actor_id,
                    reason or f"{operation_type} requires approval for {risk}",
                ),
            )
            self.ledger.set_run_status(run_id, "awaiting_approval")

        event = self.ledger.append_event(
            run_id=run_id,
            event_type=f"{CONTROL_EVENT_PREFIX}operation_enqueued",
            phase=phase,
            actor_type=actor.actor_type,
            actor_id=actor.actor_id,
            operation_id=operation_id,
            payload={
                "operation_id": operation_id,
                "operation_type": operation_type,
                "risk": risk,
                "operation_status": operation_status,
                "approval_id": approval_id,
                "authority": actor.authority,
                "idempotency_key": idem,
                "target_scope": target_scope,
                "recipe_step_id": recipe_step_id,
            },
        )
        self.conn.commit()
        return {
            "status": "created",
            "operation_id": operation_id,
            "operation_status": operation_status,
            "approval_id": approval_id,
            "idempotency_key": idem,
            "event_seq": event.event_seq,
            "event_hash": event.event_hash,
        }

    def approval_decision(
        self,
        *,
        approval_id: str,
        decision: str,
        actor: Actor,
        reason: str | None = None,
        idempotency_key: str | None = None,
    ) -> dict[str, Any]:
        self._require_actor(actor, APPROVAL_AUTHORITIES)
        if decision not in {"approved", "denied"}:
            raise AgentControlError("decision must be approved or denied")
        approval = self._approval_row(approval_id)
        run_id = approval["run_id"]
        operation_id = approval["operation_id"]

        if approval["status"] in {"approved", "denied"}:
            if approval["status"] != decision:
                raise AgentControlError(
                    f"approval {approval_id} is already {approval['status']}"
                )
            return {
                "status": "existing",
                "approval_id": approval_id,
                "decision": decision,
                "operation_id": operation_id,
                "idempotency_key": idempotency_key,
            }
        if approval["status"] != "open":
            raise AgentControlError(f"approval {approval_id} is {approval['status']}")

        self.conn.execute(
            """
            UPDATE envctl_migration_approvals
            SET status = ?, decided_by = ?, reason = COALESCE(?, reason), decided_at_utc = ?
            WHERE id = ? AND status = 'open'
            """,
            (decision, actor.actor_id, reason, now(), approval_id),
        )
        operation_status = "ready" if decision == "approved" else "denied"
        self.ledger.set_operation_status(
            operation_id,
            operation_status,
            error={"reason": reason or "approval denied"} if decision == "denied" else None,
        )
        self._refresh_run_status(run_id)
        event = self.ledger.append_event(
            run_id=run_id,
            event_type=f"{CONTROL_EVENT_PREFIX}approval_{decision}",
            actor_type=actor.actor_type,
            actor_id=actor.actor_id,
            operation_id=operation_id,
            payload={
                "approval_id": approval_id,
                "decision": decision,
                "authority": actor.authority,
                "reason": reason,
                "idempotency_key": idempotency_key,
            },
        )
        self.conn.commit()
        return {
            "status": "decided",
            "approval_id": approval_id,
            "decision": decision,
            "operation_id": operation_id,
            "operation_status": operation_status,
            "event_seq": event.event_seq,
            "event_hash": event.event_hash,
        }

    def transition_operation(
        self,
        *,
        operation_id: str,
        trigger: str,
        actor: Actor,
        reason: str | None = None,
    ) -> dict[str, Any]:
        self._require_actor(actor, WRITE_AUTHORITIES)
        row = self._operation_row(operation_id)
        state = normalize_state(row["status"])
        if (
            row["status"] == "ready"
            and approval_required(row["risk"])
            and self._operation_has_approval(operation_id)
        ):
            state = OperationState.APPROVED
        state_record = StateOperationRecord(
            operation_id=row["id"],
            run_id=row["run_id"],
            operation_type=row["operation_type"],
            state=state,
            risk=row["risk"],
        )
        try:
            _, result = transition(state_record, trigger, reason=reason)
        except OperationTransitionError as exc:
            self.ledger.append_event(
                run_id=row["run_id"],
                event_type=f"{CONTROL_EVENT_PREFIX}operation_transition_refused",
                phase=row["phase"],
                actor_type=actor.actor_type,
                actor_id=actor.actor_id,
                operation_id=operation_id,
                payload={
                    "operation_id": operation_id,
                    "trigger": trigger,
                    "authority": actor.authority,
                    "reason": str(exc),
                },
            )
            self.conn.commit()
            raise AgentControlError(str(exc)) from exc

        self.ledger.set_operation_status(
            operation_id,
            result.db_status,
            error={"reason": reason} if result.db_status in {"blocked", "failed"} and reason else None,
        )
        self._refresh_run_status(row["run_id"])
        event = self.ledger.append_event(
            run_id=row["run_id"],
            event_type=f"{CONTROL_EVENT_PREFIX}{result.event_type}",
            phase=row["phase"],
            actor_type=actor.actor_type,
            actor_id=actor.actor_id,
            operation_id=operation_id,
            payload={**result.event_payload(), "authority": actor.authority},
        )
        return {
            "status": "transitioned",
            "operation_id": operation_id,
            "trigger": trigger,
            "operation_status": result.db_status,
            "event_seq": event.event_seq,
            "event_hash": event.event_hash,
        }

    def acquire_lease(
        self,
        *,
        run_id: str,
        target_scope: str,
        actor: Actor,
        operation_id: str | None = None,
        lease_id: str | None = None,
        reason: str | None = None,
    ) -> dict[str, Any]:
        self._require_run(run_id)
        self._require_actor(actor, LEASE_AUTHORITIES)
        if not target_scope:
            raise AgentControlError("target_scope is required")
        active = [
            lock
            for lock in self._visible_locks(run_id)
            if lock["target_scope"] == target_scope and lock["status"] == "held"
        ]
        if active:
            event = self.ledger.append_event(
                run_id=run_id,
                event_type=f"{CONTROL_EVENT_PREFIX}lease_blocked",
                actor_type=actor.actor_type,
                actor_id=actor.actor_id,
                operation_id=operation_id,
                payload={
                    "target_scope": target_scope,
                    "authority": actor.authority,
                    "blocking_lease_id": active[0]["lease_id"],
                    "reason": reason,
                },
            )
            return {
                "status": "blocked",
                "target_scope": target_scope,
                "blocking_lease_id": active[0]["lease_id"],
                "event_seq": event.event_seq,
            }
        resolved_lease_id = lease_id or f"lease-{stable_id(run_id, target_scope, actor.actor_id)}"
        event = self.ledger.append_event(
            run_id=run_id,
            event_type=f"{CONTROL_EVENT_PREFIX}lease_acquired",
            actor_type=actor.actor_type,
            actor_id=actor.actor_id,
            operation_id=operation_id,
            payload={
                "lease_id": resolved_lease_id,
                "target_scope": target_scope,
                "authority": actor.authority,
                "reason": reason,
            },
        )
        return {
            "status": "acquired",
            "lease_id": resolved_lease_id,
            "target_scope": target_scope,
            "event_seq": event.event_seq,
            "event_hash": event.event_hash,
        }

    def release_lease(
        self,
        *,
        run_id: str,
        lease_id: str,
        actor: Actor,
        reason: str | None = None,
    ) -> dict[str, Any]:
        self._require_run(run_id)
        self._require_actor(actor, LEASE_AUTHORITIES)
        visible = {lock["lease_id"]: lock for lock in self._visible_locks(run_id)}
        if lease_id not in visible:
            raise AgentControlError(f"unknown active lease: {lease_id}")
        event = self.ledger.append_event(
            run_id=run_id,
            event_type=f"{CONTROL_EVENT_PREFIX}lease_released",
            actor_type=actor.actor_type,
            actor_id=actor.actor_id,
            payload={
                "lease_id": lease_id,
                "target_scope": visible[lease_id]["target_scope"],
                "authority": actor.authority,
                "reason": reason,
            },
        )
        return {
            "status": "released",
            "lease_id": lease_id,
            "target_scope": visible[lease_id]["target_scope"],
            "event_seq": event.event_seq,
            "event_hash": event.event_hash,
        }

    def _approval_summary(self, run_id: str) -> dict[str, Any]:
        rows = self.conn.execute(
            """
            SELECT id, operation_id, risk, status, requested_by, decided_by, reason,
                   requested_at_utc, decided_at_utc
            FROM envctl_migration_approvals
            WHERE run_id = ?
            ORDER BY requested_at_utc, id
            """,
            (run_id,),
        ).fetchall()
        counts: dict[str, int] = {}
        approvals = []
        for row in rows:
            counts[row["status"]] = counts.get(row["status"], 0) + 1
            approvals.append(dict(row))
        return {"counts": counts, "items": approvals}

    def _operation_counts(self, run_id: str) -> dict[str, int]:
        rows = self.conn.execute(
            """
            SELECT status, COUNT(*) AS count
            FROM envctl_migration_operations
            WHERE run_id = ?
            GROUP BY status
            ORDER BY status
            """,
            (run_id,),
        ).fetchall()
        return {row["status"]: int(row["count"]) for row in rows}

    def _visible_locks(self, run_id: str) -> list[dict[str, Any]]:
        rows = self.conn.execute(
            """
            SELECT event_seq, event_type, actor_type, actor_id, operation_id,
                   payload_json, created_at_utc
            FROM envctl_migration_run_events
            WHERE run_id = ?
              AND event_type IN (
                'agent_control_lease_acquired',
                'agent_control_lease_released'
              )
            ORDER BY event_seq
            """,
            (run_id,),
        ).fetchall()
        locks: dict[str, dict[str, Any]] = {}
        for row in rows:
            payload = json.loads(row["payload_json"])
            lease_id = payload.get("lease_id")
            if not lease_id:
                continue
            if row["event_type"] == f"{CONTROL_EVENT_PREFIX}lease_released":
                locks.pop(lease_id, None)
                continue
            locks[lease_id] = {
                "lease_id": lease_id,
                "target_scope": payload.get("target_scope"),
                "status": "held",
                "actor_type": row["actor_type"],
                "actor_id": row["actor_id"],
                "operation_id": row["operation_id"],
                "acquired_event_seq": row["event_seq"],
                "acquired_at_utc": row["created_at_utc"],
            }
        return sorted(locks.values(), key=lambda item: item["acquired_event_seq"])

    def _refresh_run_status(self, run_id: str) -> None:
        open_approvals = self.conn.execute(
            """
            SELECT COUNT(*)
            FROM envctl_migration_approvals
            WHERE run_id = ? AND status = 'open'
            """,
            (run_id,),
        ).fetchone()[0]
        if open_approvals:
            self.ledger.set_run_status(run_id, "awaiting_approval")
            return
        denied = self.conn.execute(
            """
            SELECT COUNT(*)
            FROM envctl_migration_operations
            WHERE run_id = ? AND status = 'denied'
            """,
            (run_id,),
        ).fetchone()[0]
        if denied:
            self.ledger.set_run_status(run_id, "blocked")
            return
        running = self.conn.execute(
            """
            SELECT COUNT(*)
            FROM envctl_migration_operations
            WHERE run_id = ? AND status = 'running'
            """,
            (run_id,),
        ).fetchone()[0]
        if running:
            self.ledger.set_run_status(run_id, "running")
            return
        self.ledger.set_run_status(run_id, "planning")

    def _approval_row(self, approval_id: str) -> sqlite3.Row:
        row = self.conn.execute(
            "SELECT * FROM envctl_migration_approvals WHERE id = ?",
            (approval_id,),
        ).fetchone()
        if row is None:
            raise AgentControlError(f"unknown approval: {approval_id}")
        return row

    def _operation_row(self, operation_id: str) -> sqlite3.Row:
        row = self.conn.execute(
            "SELECT * FROM envctl_migration_operations WHERE id = ?",
            (operation_id,),
        ).fetchone()
        if row is None:
            raise AgentControlError(f"unknown operation: {operation_id}")
        return row

    def _operation_has_approval(self, operation_id: str) -> bool:
        row = self.conn.execute(
            """
            SELECT COUNT(*)
            FROM envctl_migration_approvals
            WHERE operation_id = ? AND status = 'approved'
            """,
            (operation_id,),
        ).fetchone()
        return int(row[0]) > 0

    def _require_run(self, run_id: str) -> None:
        row = self.conn.execute("SELECT id FROM envctl_migration_runs WHERE id = ?", (run_id,)).fetchone()
        if row is None:
            raise AgentControlError(f"unknown run: {run_id}")

    def _require_actor(self, actor: Actor, allowed_authorities: set[str]) -> None:
        self._require_value("actor_type", actor.actor_type, ACTOR_TYPES)
        self._require_value("authority", actor.authority, AUTHORITY_LEVELS)
        if actor.authority not in allowed_authorities:
            raise AgentControlError(
                f"authority {actor.authority} is not permitted for this action"
            )
        if not actor.actor_id:
            raise AgentControlError("actor_id is required")

    def _require_value(self, name: str, value: str, allowed: set[str]) -> None:
        if value not in allowed:
            raise AgentControlError(f"invalid {name}: {value}")

    def _event_row(self, row: sqlite3.Row) -> dict[str, Any]:
        return {
            "event_seq": row["event_seq"],
            "event_type": row["event_type"],
            "phase": row["phase"],
            "actor_type": row["actor_type"],
            "actor_id": row["actor_id"],
            "operation_id": row["operation_id"],
            "payload": json.loads(row["payload_json"]),
            "evidence_refs": json.loads(row["evidence_refs_json"] or "[]"),
            "previous_event_hash": row["previous_event_hash"],
            "event_hash": row["event_hash"],
            "created_at_utc": row["created_at_utc"],
        }


def connect(path: Path, *, migrate: bool = True) -> sqlite3.Connection:
    conn = sqlite3.connect(path)
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA foreign_keys = ON")
    if migrate:
        apply_migrations(conn, package_root())
    return conn


def parse_json_value(value: str | None) -> dict[str, Any]:
    if not value:
        return {}
    parsed = json.loads(value)
    if not isinstance(parsed, dict):
        raise AgentControlError("JSON input must be an object")
    return parsed


def actor_from_args(args: argparse.Namespace) -> Actor:
    return Actor(
        actor_type=args.actor_type,
        actor_id=args.actor_id,
        authority=args.authority,
    )


def emit(value: dict[str, Any]) -> None:
    print(json.dumps(value, indent=2, sort_keys=True))


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Database-backed envctl agent control API CLI",
    )
    parser.add_argument("--db", required=True, help="SQLite database path")
    sub = parser.add_subparsers(dest="command", required=True)

    status = sub.add_parser("status", help="GET /migration/runs/{run_id}/status")
    status.add_argument("run_id")
    status.add_argument("--recent-events", type=int, default=10)

    events = sub.add_parser("events", help="GET /migration/runs/{run_id}/events")
    events.add_argument("run_id")
    events.add_argument("--limit", type=int)

    enqueue = sub.add_parser("enqueue", help="Create a database-controlled operation")
    enqueue.add_argument("run_id")
    enqueue.add_argument("operation_type")
    enqueue.add_argument("--risk", required=True, choices=sorted(RISKS))
    enqueue.add_argument("--actor-type", required=True, choices=sorted(ACTOR_TYPES))
    enqueue.add_argument("--actor-id", required=True)
    enqueue.add_argument("--authority", required=True, choices=sorted(AUTHORITY_LEVELS))
    enqueue.add_argument("--phase")
    enqueue.add_argument("--parent-operation-id")
    enqueue.add_argument("--recipe-step-id")
    enqueue.add_argument("--target-scope")
    enqueue.add_argument("--input-json")
    enqueue.add_argument("--command-redacted")
    enqueue.add_argument("--reason")
    enqueue.add_argument("--idempotency-key")

    decision = sub.add_parser("decision", help="POST /migration/approvals/{approval_id}/decision")
    decision.add_argument("approval_id")
    decision.add_argument("--decision", required=True, choices=["approved", "denied"])
    decision.add_argument("--actor-type", required=True, choices=sorted(ACTOR_TYPES))
    decision.add_argument("--actor-id", required=True)
    decision.add_argument("--authority", required=True, choices=sorted(AUTHORITY_LEVELS))
    decision.add_argument("--reason")
    decision.add_argument("--idempotency-key")

    trans = sub.add_parser("transition", help="Apply a control transition to an operation")
    trans.add_argument("operation_id")
    trans.add_argument("--trigger", required=True)
    trans.add_argument("--actor-type", required=True, choices=sorted(ACTOR_TYPES))
    trans.add_argument("--actor-id", required=True)
    trans.add_argument("--authority", required=True, choices=sorted(AUTHORITY_LEVELS))
    trans.add_argument("--reason")

    lease = sub.add_parser("lease", help="Acquire a visible run-scoped lock")
    lease.add_argument("run_id")
    lease.add_argument("--target-scope", required=True)
    lease.add_argument("--actor-type", required=True, choices=sorted(ACTOR_TYPES))
    lease.add_argument("--actor-id", required=True)
    lease.add_argument("--authority", required=True, choices=sorted(AUTHORITY_LEVELS))
    lease.add_argument("--operation-id")
    lease.add_argument("--lease-id")
    lease.add_argument("--reason")

    release = sub.add_parser("release-lease", help="Release a visible run-scoped lock")
    release.add_argument("run_id")
    release.add_argument("lease_id")
    release.add_argument("--actor-type", required=True, choices=sorted(ACTOR_TYPES))
    release.add_argument("--actor-id", required=True)
    release.add_argument("--authority", required=True, choices=sorted(AUTHORITY_LEVELS))
    release.add_argument("--reason")
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        with connect(Path(args.db)) as conn:
            api = AgentControlApi(conn)
            if args.command == "status":
                emit(api.run_status(args.run_id, recent_events=args.recent_events))
            elif args.command == "events":
                emit(api.run_events(args.run_id, limit=args.limit))
            elif args.command == "enqueue":
                emit(
                    api.enqueue_operation(
                        run_id=args.run_id,
                        operation_type=args.operation_type,
                        risk=args.risk,
                        actor=actor_from_args(args),
                        phase=args.phase,
                        parent_operation_id=args.parent_operation_id,
                        recipe_step_id=args.recipe_step_id,
                        target_scope=args.target_scope,
                        input_payload=parse_json_value(args.input_json),
                        command_redacted=args.command_redacted,
                        reason=args.reason,
                        idempotency_key=args.idempotency_key,
                    )
                )
            elif args.command == "decision":
                emit(
                    api.approval_decision(
                        approval_id=args.approval_id,
                        decision=args.decision,
                        actor=actor_from_args(args),
                        reason=args.reason,
                        idempotency_key=args.idempotency_key,
                    )
                )
            elif args.command == "transition":
                emit(
                    api.transition_operation(
                        operation_id=args.operation_id,
                        trigger=args.trigger,
                        actor=actor_from_args(args),
                        reason=args.reason,
                    )
                )
            elif args.command == "lease":
                emit(
                    api.acquire_lease(
                        run_id=args.run_id,
                        target_scope=args.target_scope,
                        actor=actor_from_args(args),
                        operation_id=args.operation_id,
                        lease_id=args.lease_id,
                        reason=args.reason,
                    )
                )
            elif args.command == "release-lease":
                emit(
                    api.release_lease(
                        run_id=args.run_id,
                        lease_id=args.lease_id,
                        actor=actor_from_args(args),
                        reason=args.reason,
                    )
                )
        return 0
    except (AgentControlError, sqlite3.Error, json.JSONDecodeError) as exc:
        emit({"status": "error", "error": str(exc)})
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
