from __future__ import annotations

import json
import sqlite3
from pathlib import Path

from _common import append_proof, make_proof, now, package_root, root, sha256_file
from envctl_run_ledger import (
    LedgerError,
    OperationRecord,
    RunLedger,
    apply_migrations,
)


TASK_ID = "REQ-022_ENVCTL_RUN_LEDGER"


def proof_sha(relpath: str) -> str | None:
    path = root() / relpath
    if not path.exists():
        return None
    return "sha256:" + sha256_file(path)


def exercise_ledger(conn: sqlite3.Connection) -> dict:
    ledger = RunLedger(conn)
    catalog = ledger.seed_base_catalog()
    run = ledger.create_run(
        run_id="run-req022",
        target_id=catalog["target_id"],
        recipe_id=catalog["recipe_id"],
        artifact_contract_id=catalog["contract_id"],
        human_mode="approval-gated",
        initiated_by="envctl-db-agent",
        sandbox_policy="workspace-write",
        approval_policy="never",
        tool_versions={"python": "3", "sqlite": sqlite3.sqlite_version},
    )
    ledger.append_event(
        run_id=run["run_id"],
        event_type="run_created",
        phase="02-envctl-db",
        actor_type="agent",
        actor_id="envctl-db-agent",
        payload={"status": "created", "task_id": TASK_ID},
    )
    ledger.record_operation(
        OperationRecord(
            operation_id="op-req022-run-ledger",
            run_id=run["run_id"],
            operation_type="persist_run_ledger",
            phase="02-envctl-db",
            status="ready",
            risk="R1",
            idempotency_key="REQ-022/persist-run-ledger",
            command_redacted="python3 scripts/verify_envctl_run_ledger.py",
            input={
                "schemas": [
                    "schemas/run_event.schema.json",
                    "schemas/operation.schema.json",
                ],
                "tables": [
                    "envctl_migration_runs",
                    "envctl_migration_operations",
                    "envctl_migration_run_events",
                    "envctl_migration_evidence",
                ],
            },
        )
    )
    ledger.set_run_status(run["run_id"], "running")
    ledger.set_operation_status("op-req022-run-ledger", "running")
    ledger.append_event(
        run_id=run["run_id"],
        event_type="operation_started",
        phase="02-envctl-db",
        actor_type="agent",
        actor_id="helper-envctl-ledger-01",
        operation_id="op-req022-run-ledger",
        payload={"status": "running", "operation_type": "persist_run_ledger"},
    )
    proof_link = ledger.link_evidence(
        run_id=run["run_id"],
        operation_id="op-req022-run-ledger",
        uri="proof_records/REQ-020_ENVCTL_DB_SCHEMA.proof.json",
        evidence_kind="proof_record",
        sha256=proof_sha("proof_records/REQ-020_ENVCTL_DB_SCHEMA.proof.json"),
        metadata={"depends_on": "REQ-020_ENVCTL_DB_SCHEMA"},
    )
    ledger.append_event(
        run_id=run["run_id"],
        event_type="proof_linked",
        phase="02-envctl-db",
        actor_type="system",
        actor_id="run-ledger",
        operation_id="op-req022-run-ledger",
        evidence_refs=[proof_link["uri"]],
        payload={"evidence_id": proof_link["evidence_id"], "kind": "proof_record"},
    )
    report_link = ledger.link_evidence(
        run_id=run["run_id"],
        operation_id="op-req022-run-ledger",
        uri="generated/envctl_run_ledger_report.json",
        evidence_kind="run_ledger_report",
        metadata={"generated_by": "verify_envctl_run_ledger.py"},
    )
    ledger.set_operation_status(
        "op-req022-run-ledger",
        "succeeded",
        output_ref="generated/envctl_run_ledger_report.json",
    )
    ledger.append_event(
        run_id=run["run_id"],
        event_type="operation_succeeded",
        phase="02-envctl-db",
        actor_type="agent",
        actor_id="helper-envctl-ledger-01",
        operation_id="op-req022-run-ledger",
        evidence_refs=[report_link["uri"]],
        payload={"status": "succeeded", "output_ref": report_link["uri"]},
    )
    ledger.set_run_status(run["run_id"], "completed")
    ledger.append_event(
        run_id=run["run_id"],
        event_type="run_completed",
        phase="02-envctl-db",
        actor_type="system",
        actor_id="run-ledger",
        payload={"status": "completed", "next_unblocked": "REQ-027_ENVCTL_REPLAY_ENGINE"},
    )

    rejected_duplicate = False
    rejected_invalid_status = False
    try:
        ledger.append_event(
            run_id=run["run_id"],
            event_seq=2,
            event_type="out_of_order",
            actor_type="system",
            payload={"should": "fail"},
        )
    except LedgerError:
        rejected_duplicate = True
    try:
        ledger.set_operation_status("op-req022-run-ledger", "done")
    except LedgerError:
        rejected_invalid_status = True

    snapshot = ledger.run_snapshot(run["run_id"])
    checks = {
        "run_completed": snapshot["status"] == "completed",
        "operation_succeeded": snapshot["operations"][0]["status"] == "succeeded",
        "proof_link_recorded": any(item["evidence_kind"] == "proof_record" for item in snapshot["evidence"]),
        "event_chain_valid": snapshot["event_chain"]["chain_valid"],
        "event_count": snapshot["event_chain"]["event_count"],
        "duplicate_seq_rejected": rejected_duplicate,
        "invalid_status_rejected": rejected_invalid_status,
    }
    errors = []
    if checks["event_count"] != 5:
        errors.append(f"expected 5 events, got {checks['event_count']}")
    for name, ok in checks.items():
        if name != "event_count" and not ok:
            errors.append(f"{name} failed")
    return {
        "run": run,
        "snapshot": snapshot,
        "checks": checks,
        "errors": errors,
    }


def write_docs(report: dict) -> None:
    lines = [
        "# envctl migration run ledger",
        "",
        f"Generated at: `{report['generated_at']}`",
        f"Status: `{report['status']}`",
        "",
        "## Ledger surfaces",
        "",
        "- Run lineage: `envctl_migration_runs` stores target, recipe, artifact contract, status, tool versions, and reproducibility hash.",
        "- Operation state: `envctl_migration_operations` stores idempotent operation rows with status, risk, command hash, input, output, and error fields.",
        "- Event stream: `envctl_migration_run_events` is append-only per run and hash-chained by `(run_id, event_seq)`.",
        "- Proof links: `envctl_migration_evidence` links proof records and generated reports back to a run and operation.",
        "",
        "## Runtime smoke",
        "",
        f"- Run: `{report['ledger']['run']['run_id']}`",
        f"- Status: `{report['ledger']['snapshot']['status']}`",
        f"- Operations: `{report['ledger']['snapshot']['operation_count']}`",
        f"- Events: `{report['ledger']['snapshot']['event_chain']['event_count']}`",
        f"- Hash chain valid: `{report['ledger']['snapshot']['event_chain']['chain_valid']}`",
        f"- Duplicate event sequence rejected: `{report['ledger']['checks']['duplicate_seq_rejected']}`",
        f"- Invalid operation status rejected: `{report['ledger']['checks']['invalid_status_rejected']}`",
        "",
        "## Timeline",
        "",
        "| seq | event | actor | operation | operation status |",
        "|---:|---|---|---|---|",
    ]
    for event in report["ledger"]["snapshot"]["timeline"]:
        lines.append(
            "| {event_seq} | `{event_type}` | `{actor_type}` | `{operation_id}` | `{operation_status}` |".format(
                **{key: "" if value is None else value for key, value in event.items()}
            )
        )
    lines.extend(
        [
            "",
            "## Evidence",
            "",
            "| kind | uri | sha256 |",
            "|---|---|---|",
        ]
    )
    for evidence in report["ledger"]["snapshot"]["evidence"]:
        lines.append(
            f"| `{evidence['evidence_kind']}` | `{evidence['uri']}` | `{evidence['sha256'] or ''}` |"
        )
    (root() / "docs" / "ENVCTL_RUN_LEDGER.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    base = package_root()
    conn = sqlite3.connect(":memory:")
    conn.execute("PRAGMA foreign_keys = ON")
    applied = apply_migrations(conn, base)
    ledger_result = exercise_ledger(conn)
    status = "passed" if not ledger_result["errors"] else "failed"
    report = {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "generated_at": now(),
        "status": status,
        "database_backend": "sqlite",
        "runtime": "python sqlite3 in-memory",
        "applied_migrations": applied,
        "ledger": ledger_result,
        "evidence": [
            "generated/envctl_run_ledger_report.json",
            "docs/ENVCTL_RUN_LEDGER.md",
            "proof_records/REQ-020_ENVCTL_DB_SCHEMA.proof.json",
        ],
        "errors": ledger_result["errors"],
    }
    report_path = root() / "generated" / "envctl_run_ledger_report.json"
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    write_docs(report)
    log_path = root() / "logs" / f"{TASK_ID}.log"
    log_path.parent.mkdir(parents=True, exist_ok=True)
    log_path.write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")

    files_changed = [
        "execution-framework/scripts/envctl_run_ledger.py",
        "execution-framework/scripts/verify_envctl_run_ledger.py",
        "execution-framework/generated/envctl_run_ledger_report.json",
        "execution-framework/docs/ENVCTL_RUN_LEDGER.md",
        "execution-framework/logs/REQ-022_ENVCTL_RUN_LEDGER.log",
        "execution-framework/proof_records/REQ-022_ENVCTL_RUN_LEDGER.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    commands_run = [
        "python3 scripts/verify_envctl_run_ledger.py",
    ]
    proof = make_proof(
        TASK_ID,
        "completed" if status == "passed" else "failed",
        "codex-cli-local",
        "helper-envctl-ledger-01",
        "gpt-5.3-spark",
        str(base),
        files_changed,
        commands_run,
        {
            "status": status,
            "event_count": ledger_result["checks"]["event_count"],
            "event_chain_valid": ledger_result["checks"]["event_chain_valid"],
            "proof_link_recorded": ledger_result["checks"]["proof_link_recorded"],
            "duplicate_seq_rejected": ledger_result["checks"]["duplicate_seq_rejected"],
            "invalid_status_rejected": ledger_result["checks"]["invalid_status_rejected"],
            "errors": ledger_result["errors"],
        },
        report["evidence"],
        "" if status == "passed" else "; ".join(ledger_result["errors"]),
        "run REQ-027 replay engine against the hash-chained run ledger" if status == "passed" else "fix run ledger verification errors",
    )
    append_proof(proof)
    print(
        "envctl run ledger status={status} events={events} chain={chain} proof_link={proof_link}".format(
            status=status,
            events=ledger_result["checks"]["event_count"],
            chain=ledger_result["checks"]["event_chain_valid"],
            proof_link=ledger_result["checks"]["proof_link_recorded"],
        )
    )
    if status != "passed":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
