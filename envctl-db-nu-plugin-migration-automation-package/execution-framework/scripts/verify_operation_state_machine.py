from __future__ import annotations

import json
import sqlite3
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, package_root, root
from operation_state_machine import (
    DB_STATUS_BY_STATE,
    OperationRecord,
    OperationState,
    OperationTransitionError,
    db_status_for_state,
    state_machine_model,
    transition,
)
from verify_envctl_db_schema import apply_migrations, insert_lifecycle_fixture


TASK_ID = "REQ-023_ENVCTL_OPERATION_STATE"

REQUIRED_CANONICAL_STATES = {
    "planned",
    "runnable",
    "running",
    "blocked",
    "failed",
    "approved",
    "completed",
    "rolled_back",
}


def load_operation_schema_statuses(base: Path) -> set[str]:
    schema = json.loads((base / "schemas" / "operation.schema.json").read_text(encoding="utf-8"))
    return set(schema["properties"]["status"]["enum"])


def assert_raises(fn, expected: str) -> str:
    try:
        fn()
    except OperationTransitionError as exc:
        message = str(exc)
        if expected not in message:
            raise AssertionError(f"expected {expected!r} in {message!r}") from exc
        return message
    raise AssertionError("expected OperationTransitionError")


def run_transition_paths() -> dict[str, Any]:
    happy = OperationRecord.planned("op-state-happy", "run-req020", "scan", "R1")
    happy_events = []
    for trigger in ["mark_runnable", "start", "complete"]:
        happy, result = transition(happy, trigger)
        happy_events.append(result.as_dict())

    risky = OperationRecord.planned("op-state-risky", "run-req020", "apply_patch", "R3")
    risky_events = []
    for trigger in ["mark_runnable", "approve", "start", "fail", "rollback"]:
        kwargs = {"reason": "verification fixture"} if trigger in {"fail", "rollback"} else {}
        risky, result = transition(risky, trigger, **kwargs)
        risky_events.append(result.as_dict())

    blocked = OperationRecord.planned("op-state-blocked", "run-req020", "wait_for_lock", "R0")
    blocked_events = []
    for trigger in ["block", "mark_runnable", "start", "complete"]:
        kwargs = {"reason": "dependency not complete"} if trigger == "block" else {}
        blocked, result = transition(blocked, trigger, **kwargs)
        blocked_events.append(result.as_dict())

    illegal = {
        "completed_to_running": assert_raises(
            lambda: transition(happy, "start"),
            "illegal operation transition",
        ),
        "rolled_back_to_runnable": assert_raises(
            lambda: transition(risky, "mark_runnable"),
            "illegal operation transition",
        ),
        "r3_start_without_approval": assert_raises(
            lambda: transition(
                OperationRecord(
                    operation_id="op-state-unapproved",
                    run_id="run-req020",
                    operation_type="apply_patch",
                    state=OperationState.RUNNABLE,
                    risk="R3",
                ),
                "start",
            ),
            "requires approved state",
        ),
    }

    return {
        "happy_path": happy_events,
        "risky_approval_and_rollback_path": risky_events,
        "blocked_then_runnable_path": blocked_events,
        "illegal_transition_checks": illegal,
    }


def run_sqlite_bridge(base: Path) -> dict[str, Any]:
    conn = sqlite3.connect(":memory:")
    conn.execute("PRAGMA foreign_keys = ON")
    apply_migrations(conn, base)
    insert_lifecycle_fixture(conn)
    inserted = []
    for index, state in enumerate(OperationState, start=1):
        operation_id = f"op-state-{state.value}"
        conn.execute(
            """
            INSERT INTO envctl_migration_operations
              (id, run_id, operation_type, status, risk, idempotency_key, input_json)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            """,
            (
                operation_id,
                "run-req020",
                "state_machine_fixture",
                db_status_for_state(state),
                "R1",
                f"REQ-023/{index}/{state.value}",
                json.dumps({"canonical_state": state.value}),
            ),
        )
        inserted.append({"state": state.value, "db_status": db_status_for_state(state)})
    conn.commit()
    rows = conn.execute(
        """
        SELECT status, COUNT(*)
        FROM envctl_migration_operations
        WHERE operation_type = 'state_machine_fixture'
        GROUP BY status
        ORDER BY status
        """
    ).fetchall()
    return {
        "inserted_operation_count": len(inserted),
        "inserted_states": inserted,
        "status_counts": [{"status": status, "count": count} for status, count in rows],
        "foreign_key_errors": [dict(row) for row in conn.execute("PRAGMA foreign_key_check")],
    }


def write_docs(model: dict[str, Any], report: dict[str, Any]) -> None:
    lines = [
        "# envctl operation state machine",
        "",
        f"Generated at: `{report['generated_at']}`",
        f"Status: `{report['status']}`",
        "",
        "## Canonical states",
        "",
        "| state | persisted operation status | terminal |",
        "|---|---|---|",
    ]
    terminal_states = set(model["terminal_states"])
    for state in model["canonical_states"]:
        lines.append(
            f"| `{state}` | `{model['db_status_by_state'][state]}` | "
            f"{'yes' if state in terminal_states else 'no'} |"
        )
    lines.extend(
        [
            "",
            "## Transitions",
            "",
            "| from | allowed targets |",
            "|---|---|",
        ]
    )
    for state, targets in model["transitions"].items():
        rendered_targets = ", ".join(f"`{target}`" for target in targets) or "_none_"
        lines.append(f"| `{state}` | {rendered_targets} |")
    lines.extend(
        [
            "",
            "## Guards",
            "",
            "- `R3` through `R5` operations must enter `approved` before `running`.",
            "- `completed` and `rolled_back` are terminal for dispatch; `completed` can still be compensated by a rollback transition.",
            "- The persisted SQL enum remains the existing operation schema enum; canonical state is bridged through the state machine mapping.",
            "",
            "## Verification",
            "",
            f"- Required canonical states covered: `{report['summary']['required_state_count']}`",
            f"- Transition path events exercised: `{report['summary']['transition_event_count']}`",
            f"- Illegal transitions rejected: `{report['summary']['illegal_transition_rejection_count']}`",
            f"- SQLite bridge rows inserted: `{report['summary']['sqlite_inserted_operation_count']}`",
        ]
    )
    (root() / "docs" / "OPERATION_STATE_MACHINE.md").write_text(
        "\n".join(lines) + "\n",
        encoding="utf-8",
    )


def main() -> None:
    base = package_root()
    model = state_machine_model()
    errors = []

    schema_statuses = load_operation_schema_statuses(base)
    mapped_statuses = set(DB_STATUS_BY_STATE.values())
    canonical_states = set(model["canonical_states"])
    if canonical_states != REQUIRED_CANONICAL_STATES:
        errors.append(
            f"canonical state mismatch: expected {sorted(REQUIRED_CANONICAL_STATES)}, got {sorted(canonical_states)}"
        )
    missing_schema_statuses = sorted(mapped_statuses - schema_statuses)
    if missing_schema_statuses:
        errors.append(f"state mapping uses statuses missing from operation schema: {missing_schema_statuses}")

    transition_paths = run_transition_paths()
    sqlite_bridge = run_sqlite_bridge(base)
    if sqlite_bridge["foreign_key_errors"]:
        errors.append(f"sqlite foreign key errors: {sqlite_bridge['foreign_key_errors']}")

    transition_event_count = sum(
        len(transition_paths[key])
        for key in ["happy_path", "risky_approval_and_rollback_path", "blocked_then_runnable_path"]
    )
    report = {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "passed" if not errors else "failed",
        "generated_at": now(),
        "summary": {
            "required_state_count": len(REQUIRED_CANONICAL_STATES),
            "canonical_state_count": len(canonical_states),
            "transition_event_count": transition_event_count,
            "illegal_transition_rejection_count": len(transition_paths["illegal_transition_checks"]),
            "sqlite_inserted_operation_count": sqlite_bridge["inserted_operation_count"],
            "operation_schema_status_count": len(schema_statuses),
        },
        "errors": errors,
        "evidence": [
            "scripts/operation_state_machine.py",
            "generated/operation_state_machine.json",
            "generated/operation_state_machine_validation_report.json",
            "docs/OPERATION_STATE_MACHINE.md",
        ],
    }
    output = {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "model": model,
        "operation_schema_statuses": sorted(schema_statuses),
        "transition_paths": transition_paths,
        "sqlite_bridge": sqlite_bridge,
        "report": report,
    }

    (root() / "generated" / "operation_state_machine.json").write_text(
        json.dumps(output, indent=2, sort_keys=False) + "\n",
        encoding="utf-8",
    )
    (root() / "generated" / "operation_state_machine_validation_report.json").write_text(
        json.dumps(report, indent=2, sort_keys=False) + "\n",
        encoding="utf-8",
    )
    write_docs(model, report)
    (root() / "logs" / f"{TASK_ID}.log").write_text(
        json.dumps(report, indent=2, sort_keys=False) + "\n",
        encoding="utf-8",
    )
    (root() / "state" / f"{TASK_ID}.heartbeat.json").write_text(
        json.dumps(
            {
                "task_id": TASK_ID,
                "status": report["status"],
                "updated_at": report["generated_at"],
                "evidence": report["evidence"],
            },
            indent=2,
            sort_keys=False,
        )
        + "\n",
        encoding="utf-8",
    )

    files_changed = [
        "execution-framework/scripts/operation_state_machine.py",
        "execution-framework/scripts/verify_operation_state_machine.py",
        "execution-framework/generated/operation_state_machine.json",
        "execution-framework/generated/operation_state_machine_validation_report.json",
        "execution-framework/docs/OPERATION_STATE_MACHINE.md",
        "execution-framework/state/REQ-023_ENVCTL_OPERATION_STATE.heartbeat.json",
        "execution-framework/logs/REQ-023_ENVCTL_OPERATION_STATE.log",
        "execution-framework/proof_records/REQ-023_ENVCTL_OPERATION_STATE.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    commands_run = ["python3 scripts/verify_operation_state_machine.py"]
    proof = make_proof(
        TASK_ID,
        "completed" if report["status"] == "passed" else "failed",
        "codex-cli-local",
        "helper-envctl-state-01",
        "gpt-5.3-spark",
        str(base),
        files_changed,
        commands_run,
        report,
        report["evidence"],
        "" if report["status"] == "passed" else "; ".join(errors),
        "run REQ-028_ENVCTL_AGENT_CONTROL_API after REQ-025 validation evidence is complete"
        if report["status"] == "passed"
        else "fix operation state machine validation errors",
    )
    append_proof(proof)
    print(
        "operation state machine status={status} states={states} transitions={events} sqlite_rows={rows}".format(
            status=report["status"],
            states=report["summary"]["canonical_state_count"],
            events=report["summary"]["transition_event_count"],
            rows=report["summary"]["sqlite_inserted_operation_count"],
        )
    )
    if report["status"] != "passed":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
