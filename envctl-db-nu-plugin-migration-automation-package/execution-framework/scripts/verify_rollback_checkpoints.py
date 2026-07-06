from __future__ import annotations

import json
import sqlite3

from _common import append_proof, make_proof, now, package_root, root
from envctl_run_ledger import OperationRecord, RunLedger
from rollback_checkpoints import RollbackCheckpointError, RollbackCheckpointStore
from verify_envctl_db_schema import apply_migrations


TASK_ID = "REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS"
HELPER_ID = "helper-envctl-rollback-01"
MODEL_TAG = "gpt-5.3-spark"


def insert_req026_fixture(conn: sqlite3.Connection) -> dict:
    ledger = RunLedger(conn)
    catalog = ledger.seed_base_catalog()
    ledger.create_run(
        run_id="run-req026",
        target_id=catalog["target_id"],
        recipe_id=catalog["recipe_id"],
        artifact_contract_id=catalog["contract_id"],
        human_mode="approval-gated",
        initiated_by="envctl-db-agent",
        sandbox_policy="workspace-write",
        approval_policy="never",
        tool_versions={"python": "stdlib", "sqlite": "stdlib"},
    )
    safe_op = OperationRecord(
        operation_id="op-req026-safe",
        run_id="run-req026",
        operation_type="generate_artifact_boundary",
        status="succeeded",
        risk="R2",
        idempotency_key="REQ-026/safe-boundary",
        phase="02-envctl-db",
        command_redacted="python3 scripts/verify_rollback_checkpoints.py --safe",
        input={"task_id": TASK_ID, "risk": "R2"},
        output_ref="execution-framework/generated/rollback_checkpoints/safe-artifact.json",
    )
    risky_op = OperationRecord(
        operation_id="op-req026-risky",
        run_id="run-req026",
        operation_type="mutate_target_boundary",
        status="succeeded",
        risk="R4",
        idempotency_key="REQ-026/risky-boundary",
        phase="02-envctl-db",
        command_redacted="envctl rollback apply --requires-approval",
        input={"task_id": TASK_ID, "risk": "R4"},
        output_ref="execution-framework/generated/rollback_checkpoints/risky-artifact.json",
    )
    ledger.record_operation(safe_op)
    ledger.record_operation(risky_op)
    ledger.append_event(
        run_id="run-req026",
        event_type="rollback_substrate_fixture_started",
        actor_type="agent",
        actor_id="envctl-db-agent",
        payload={"task_id": TASK_ID},
    )
    return {
        "run_id": "run-req026",
        "safe_operation_id": safe_op.operation_id,
        "risky_operation_id": risky_op.operation_id,
    }


def expect_rejection(label: str, action) -> dict:
    try:
        action()
    except (RollbackCheckpointError, sqlite3.IntegrityError) as exc:
        return {"case": label, "status": "rejected", "message": str(exc)}
    return {"case": label, "status": "accepted", "message": "unsafe rollback/checkpoint record was accepted"}


def exercise_store(conn: sqlite3.Connection, fixture: dict) -> dict:
    store = RollbackCheckpointStore(conn, package_root())
    safe_checkpoint = store.record_checkpoint(
        run_id=fixture["run_id"],
        operation_id=fixture["safe_operation_id"],
        checkpoint_kind="artifact_boundary",
        checkpoint_ref="execution-framework/generated/rollback_checkpoints/safe-artifact.json",
        metadata={
            "task_id": TASK_ID,
            "boundary": "before repeatable artifact regeneration",
            "repeat_safe": True,
        },
    )
    duplicate_checkpoint = store.record_checkpoint(
        run_id=fixture["run_id"],
        operation_id=fixture["safe_operation_id"],
        checkpoint_kind="artifact_boundary",
        checkpoint_ref="execution-framework/generated/rollback_checkpoints/safe-artifact.json",
        metadata={
            "task_id": TASK_ID,
            "boundary": "before repeatable artifact regeneration",
            "repeat_safe": True,
        },
    )
    risky_checkpoint = store.record_checkpoint(
        run_id=fixture["run_id"],
        operation_id=fixture["risky_operation_id"],
        checkpoint_kind="pre_operation",
        checkpoint_ref="history/pre_execution_framework_manifest.json",
        metadata={
            "task_id": TASK_ID,
            "boundary": "before approval-gated target mutation",
            "repeat_safe": False,
        },
    )
    safe_rollback = store.plan_rollback(
        run_id=fixture["run_id"],
        operation_id=fixture["safe_operation_id"],
        rollback_type="rerun_from_checkpoint",
        checkpoint_id=safe_checkpoint.checkpoint_id,
        reason="prove repeat/revert boundary for generated artifact operation",
        instructions={"mode": "verify-only", "remove_added_files": []},
    )
    safe_running = store.set_rollback_status(safe_rollback.rollback_id, "running")
    safe_done = store.set_rollback_status(
        safe_running.rollback_id,
        "succeeded",
        result={
            "applied": False,
            "mode": "verify-only",
            "verified_checkpoint_hash": safe_checkpoint.checkpoint_hash,
        },
    )
    risky_rollback = store.plan_rollback(
        run_id=fixture["run_id"],
        operation_id=fixture["risky_operation_id"],
        rollback_type="restore_checkpoint",
        checkpoint_id=risky_checkpoint.checkpoint_id,
        reason="approval-gated rollback handle for high-risk operation",
        instructions={"mode": "manual_operator", "restore_ref": risky_checkpoint.checkpoint_ref},
    )
    approved_risky = store.approve_rollback(
        risky_rollback.rollback_id,
        decided_by="approved-human-gate",
        reason="REQ-026 smoke approval for rollback handle",
    )
    rejections = [
        expect_rejection(
            "blocked-secret-checkpoint-ref",
            lambda: store.record_checkpoint(
                run_id=fixture["run_id"],
                operation_id=fixture["safe_operation_id"],
                checkpoint_kind="manual",
                checkpoint_ref="execution-framework/secrets/token.txt",
            ),
        ),
        expect_rejection(
            "operation-run-mismatch",
            lambda: store.record_checkpoint(
                run_id=fixture["run_id"],
                operation_id="missing-operation",
                checkpoint_kind="manual",
                checkpoint_ref="execution-framework/generated/rollback_checkpoints/missing.json",
            ),
        ),
        expect_rejection(
            "illegal-rollback-transition",
            lambda: store.set_rollback_status(safe_done.rollback_id, "running"),
        ),
    ]
    return {
        "safe_checkpoint": safe_checkpoint,
        "duplicate_checkpoint": duplicate_checkpoint,
        "risky_checkpoint": risky_checkpoint,
        "safe_rollback": safe_done,
        "risky_rollback": approved_risky,
        "checkpoints": store.list_checkpoints(fixture["run_id"]),
        "rejections": rejections,
    }


def build_report(conn: sqlite3.Connection, fixture: dict, result: dict) -> dict:
    event_chain = RunLedger(conn).validate_event_chain(fixture["run_id"])
    counts = {
        "checkpoint_rows": conn.execute("SELECT COUNT(*) FROM envctl_migration_checkpoints").fetchone()[0],
        "rollback_rows": conn.execute("SELECT COUNT(*) FROM envctl_migration_rollbacks").fetchone()[0],
        "approval_rows": conn.execute("SELECT COUNT(*) FROM envctl_migration_approvals").fetchone()[0],
        "approved_rollback_approvals": conn.execute(
            "SELECT COUNT(*) FROM envctl_migration_approvals WHERE status = 'approved'"
        ).fetchone()[0],
        "event_rows": conn.execute("SELECT COUNT(*) FROM envctl_migration_run_events").fetchone()[0],
    }
    accepted_rejections = [item["case"] for item in result["rejections"] if item["status"] != "rejected"]
    errors = []
    if not result["safe_checkpoint"].inserted:
        errors.append("initial safe checkpoint was not inserted")
    if result["duplicate_checkpoint"].inserted:
        errors.append("duplicate checkpoint insert was not idempotent")
    if result["safe_rollback"].status != "succeeded":
        errors.append("safe rollback did not reach succeeded state")
    if result["risky_rollback"].status != "planned":
        errors.append("approved risky rollback did not return to planned status")
    if not result["risky_rollback"].plan.get("approval_required"):
        errors.append("risky rollback plan did not require approval")
    if counts["approved_rollback_approvals"] < 1:
        errors.append("risky rollback approval row was not approved")
    if accepted_rejections:
        errors.append(f"unsafe cases accepted: {', '.join(accepted_rejections)}")
    if not event_chain["chain_valid"]:
        errors.extend(event_chain["errors"])
    coverage = {
        "checkpoint_rows": counts["checkpoint_rows"] >= 2,
        "idempotent_repeat": not result["duplicate_checkpoint"].inserted,
        "rollback_handles": counts["rollback_rows"] >= 2,
        "safe_status_transitions": result["safe_rollback"].status == "succeeded",
        "approval_gated_risky_rollback": result["risky_rollback"].plan.get("approval_required")
        and result["risky_rollback"].status == "planned",
        "event_chain": event_chain["chain_valid"],
        "fail_closed_rejections": bool(result["rejections"]) and not accepted_rejections,
    }
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "passed" if not errors else "failed",
        "generated_at": now(),
        "fixture": fixture,
        "counts": counts,
        "coverage": coverage,
        "safe_checkpoint": result["safe_checkpoint"].__dict__,
        "risky_checkpoint": result["risky_checkpoint"].__dict__,
        "safe_rollback": result["safe_rollback"].__dict__,
        "risky_rollback": result["risky_rollback"].__dict__,
        "checkpoint_rows": result["checkpoints"],
        "rejection_cases": result["rejections"],
        "event_chain": event_chain,
        "errors": errors,
        "evidence": [
            "scripts/rollback_checkpoints.py",
            "scripts/verify_rollback_checkpoints.py",
            "generated/envctl_rollback_checkpoints_report.json",
            "docs/ENVCTL_ROLLBACK_CHECKPOINTS.md",
            "docs/ENVCTL_DB_SCHEMA.md",
            "prompts/SECURITY_REPRODUCIBILITY_MODEL.md",
        ],
    }


def write_docs(report: dict) -> None:
    lines = [
        "# envctl rollback checkpoints",
        "",
        f"Generated at: `{report['generated_at']}`",
        f"Status: `{report['status']}`",
        "",
        "## Coverage",
        "",
        "| capability | covered |",
        "|---|---|",
    ]
    for key, covered in report["coverage"].items():
        lines.append(f"| {key.replace('_', ' ')} | {'yes' if covered else 'no'} |")
    lines.extend(
        [
            "",
            "## Runtime smoke",
            "",
            f"- Run: `{report['fixture']['run_id']}`",
            f"- Checkpoint rows: `{report['counts']['checkpoint_rows']}`",
            f"- Rollback handles: `{report['counts']['rollback_rows']}`",
            f"- Approval rows: `{report['counts']['approval_rows']}`",
            f"- Event rows: `{report['counts']['event_rows']}`",
            f"- Safe rollback status: `{report['safe_rollback']['status']}`",
            f"- Risky rollback status after approval: `{report['risky_rollback']['status']}`",
            f"- Rejection cases: `{len(report['rejection_cases'])}`",
            "",
            "The smoke persists checkpoint boundaries into `envctl_migration_checkpoints`, creates rollback handles in `envctl_migration_rollbacks`, gates an R4 rollback through `envctl_migration_approvals`, records append-only run events for each mutation, and confirms blocked secret references, run/operation mismatches, and illegal rollback transitions fail closed.",
        ]
    )
    (root() / "docs" / "ENVCTL_ROLLBACK_CHECKPOINTS.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    base = package_root()
    conn = sqlite3.connect(":memory:")
    conn.execute("PRAGMA foreign_keys = ON")
    apply_migrations(conn, base)
    fixture = insert_req026_fixture(conn)
    result = exercise_store(conn, fixture)
    report = build_report(conn, fixture, result)

    report_path = root() / "generated" / "envctl_rollback_checkpoints_report.json"
    log_path = root() / "logs" / f"{TASK_ID}.log"
    heartbeat_path = root() / "state" / f"{TASK_ID}.heartbeat.json"
    report_path.write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    write_docs(report)
    log_path.write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    heartbeat_path.parent.mkdir(parents=True, exist_ok=True)
    heartbeat_path.write_text(
        json.dumps(
            {
                "task_id": TASK_ID,
                "status": "completed" if report["status"] == "passed" else "failed",
                "updated_at": report["generated_at"],
                "proof_uri": f"proof_records/{TASK_ID}.proof.json",
            },
            indent=2,
            sort_keys=False,
        )
        + "\n",
        encoding="utf-8",
    )

    files_changed = [
        "execution-framework/scripts/rollback_checkpoints.py",
        "execution-framework/scripts/verify_rollback_checkpoints.py",
        "execution-framework/generated/envctl_rollback_checkpoints_report.json",
        "execution-framework/docs/ENVCTL_ROLLBACK_CHECKPOINTS.md",
        "execution-framework/state/REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS.heartbeat.json",
        "execution-framework/logs/REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS.log",
        "execution-framework/proof_records/REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    commands_run = [
        "python3 scripts/verify_rollback_checkpoints.py",
        "python3 -m py_compile scripts/rollback_checkpoints.py scripts/verify_rollback_checkpoints.py",
    ]
    proof = make_proof(
        TASK_ID,
        "completed" if report["status"] == "passed" else "failed",
        "codex-cli-local",
        HELPER_ID,
        MODEL_TAG,
        str(base),
        files_changed,
        commands_run,
        report,
        report["evidence"],
        "" if report["status"] == "passed" else "; ".join(report["errors"]),
        "run REQ-027 replay engine against rollback checkpoint handles"
        if report["status"] == "passed"
        else "fix rollback checkpoint verification errors",
    )
    append_proof(proof)
    print(
        "rollback checkpoint status={status} checkpoints={checkpoints} rollbacks={rollbacks} approvals={approvals} events={events}".format(
            status=report["status"],
            checkpoints=report["counts"]["checkpoint_rows"],
            rollbacks=report["counts"]["rollback_rows"],
            approvals=report["counts"]["approval_rows"],
            events=report["counts"]["event_rows"],
        )
    )
    if report["status"] != "passed":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
