from __future__ import annotations

import json
import sqlite3
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, package_root, root, write_json
from agent_control_api import Actor, AgentControlApi, AgentControlError
from envctl_run_ledger import OperationRecord, RunLedger, apply_migrations, canonical_json


TASK_ID = "REQ-028_ENVCTL_AGENT_CONTROL_API"
HELPER_ID = "helper-envctl-agent-api-01"
MODEL_TAG = "gpt-5.3-spark"
RUN_ID = "run-req028-agent-control"
PHASE = "02-envctl-db"


def json_run(args: list[str]) -> dict[str, Any]:
    proc = subprocess.run(
        [sys.executable, "scripts/agent_control_api.py", *args],
        cwd=root(),
        text=True,
        capture_output=True,
        check=False,
    )
    try:
        payload = json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        raise AssertionError(
            f"CLI did not emit JSON for {args}: stdout={proc.stdout!r} stderr={proc.stderr!r}"
        ) from exc
    if proc.returncode != 0:
        raise AssertionError(f"CLI failed for {args}: {payload}")
    return payload


def seed_fixture(conn: sqlite3.Connection) -> dict[str, Any]:
    apply_migrations(conn, package_root())
    ledger = RunLedger(conn)
    catalog = ledger.seed_base_catalog()
    run = ledger.create_run(
        run_id=RUN_ID,
        target_id=catalog["target_id"],
        recipe_id=catalog["recipe_id"],
        artifact_contract_id=catalog["contract_id"],
        human_mode="approval-gated",
        initiated_by=HELPER_ID,
        sandbox_policy="workspace-write",
        approval_policy="R3+ explicit approval",
        tool_versions={"python": sys.version.split()[0], "sqlite": sqlite3.sqlite_version},
    )
    ledger.append_event(
        run_id=RUN_ID,
        event_type="agent_control_fixture_ready",
        phase=PHASE,
        actor_type="system",
        actor_id="verify-agent-control-api",
        payload={"task_id": TASK_ID, "run": run},
    )
    conn.commit()
    return {"catalog": catalog, "run": run}


def assert_error(fn, message_fragment: str) -> str:
    try:
        fn()
    except AgentControlError as exc:
        message = str(exc)
        if message_fragment not in message:
            raise AssertionError(f"expected {message_fragment!r} in {message!r}") from exc
        return message
    raise AssertionError(f"expected AgentControlError containing {message_fragment!r}")


def exercise_library(conn: sqlite3.Connection) -> dict[str, Any]:
    api = AgentControlApi(conn)
    agent = Actor("agent", HELPER_ID, "safe_execute")
    request_agent = Actor("agent", HELPER_ID, "approval_request")
    operator = Actor("human", "operator-req028", "operator")

    safe = api.enqueue_operation(
        run_id=RUN_ID,
        operation_type="collect_status",
        risk="R1",
        actor=agent,
        phase=PHASE,
        recipe_step_id="status-r1",
        target_scope="execution-framework/status",
        input_payload={"include_events": True},
        command_redacted="envctl agent status --run run-req028-agent-control",
        reason="safe read-oriented status collection",
    )
    if safe["operation_status"] != "ready" or safe["approval_id"] is not None:
        raise AssertionError(f"safe operation should be ready without approval: {safe}")
    duplicate = api.enqueue_operation(
        run_id=RUN_ID,
        operation_type="collect_status",
        risk="R1",
        actor=agent,
        phase=PHASE,
        recipe_step_id="status-r1",
        target_scope="execution-framework/status",
        input_payload={"include_events": True},
    )
    if duplicate["status"] != "existing" or duplicate["operation_id"] != safe["operation_id"]:
        raise AssertionError(f"idempotency did not return existing operation: {duplicate}")

    gated = api.enqueue_operation(
        run_id=RUN_ID,
        operation_type="apply_schema_patch",
        risk="R3",
        actor=request_agent,
        phase=PHASE,
        recipe_step_id="schema-r3",
        target_scope="envctl/sql",
        input_payload={"change": "additive-control-api-smoke"},
        command_redacted="envctl agent apply-schema --redacted",
        reason="prove R3 operation is approval gated",
    )
    if gated["operation_status"] != "awaiting_approval" or not gated["approval_id"]:
        raise AssertionError(f"R3 operation should create open approval: {gated}")

    refusal = assert_error(
        lambda: api.transition_operation(
            operation_id=gated["operation_id"],
            trigger="start",
            actor=agent,
            reason="prove start is blocked before approval",
        ),
        "illegal operation transition",
    )
    approved = api.approval_decision(
        approval_id=gated["approval_id"],
        decision="approved",
        actor=operator,
        reason="operator approves controlled additive smoke operation",
    )
    if approved["operation_status"] != "ready":
        raise AssertionError(f"approved operation should become ready: {approved}")

    started = api.transition_operation(
        operation_id=gated["operation_id"],
        trigger="start",
        actor=agent,
        reason="run approved smoke operation",
    )
    completed = api.transition_operation(
        operation_id=gated["operation_id"],
        trigger="complete",
        actor=agent,
        reason="smoke operation completed",
    )
    if started["operation_status"] != "running" or completed["operation_status"] != "succeeded":
        raise AssertionError(f"unexpected transition results: {started} {completed}")

    denied_op = api.enqueue_operation(
        run_id=RUN_ID,
        operation_type="destructive_cutover_attempt",
        risk="R4",
        actor=request_agent,
        phase=PHASE,
        recipe_step_id="cutover-r4",
        target_scope="envctl/cutover",
        input_payload={"change": "blocked"},
        command_redacted="envctl cutover --redacted",
        reason="prove operator denial path",
    )
    denied = api.approval_decision(
        approval_id=denied_op["approval_id"],
        decision="denied",
        actor=operator,
        reason="R4 destructive operation is not approved in verification",
    )
    if denied["operation_status"] != "denied":
        raise AssertionError(f"denied operation should be marked denied: {denied}")

    first_lease = api.acquire_lease(
        run_id=RUN_ID,
        target_scope="envctl/sql",
        actor=agent,
        operation_id=gated["operation_id"],
        reason="serialize sql target",
    )
    blocked_lease = api.acquire_lease(
        run_id=RUN_ID,
        target_scope="envctl/sql",
        actor=Actor("agent", "helper-conflict", "safe_execute"),
        reason="prove conflicting lock is visible",
    )
    if first_lease["status"] != "acquired" or blocked_lease["status"] != "blocked":
        raise AssertionError(f"lease conflict was not enforced: {first_lease} {blocked_lease}")
    status_with_lock = api.run_status(RUN_ID)
    if not status_with_lock["visible_locks"]:
        raise AssertionError("run status did not expose held lease")
    released = api.release_lease(
        run_id=RUN_ID,
        lease_id=first_lease["lease_id"],
        actor=agent,
        reason="release sql target",
    )
    status_after_release = api.run_status(RUN_ID)
    if status_after_release["visible_locks"]:
        raise AssertionError(f"lease still visible after release: {status_after_release['visible_locks']}")

    return {
        "safe_operation": safe,
        "duplicate_operation": duplicate,
        "gated_operation": gated,
        "preapproval_refusal": refusal,
        "approval_result": approved,
        "start_result": started,
        "complete_result": completed,
        "denied_operation": denied,
        "lease": first_lease,
        "blocked_lease": blocked_lease,
        "released_lease": released,
        "status_with_lock": status_with_lock,
        "status_after_release": status_after_release,
    }


def exercise_cli(db_path: Path) -> dict[str, Any]:
    cli_status = json_run(["--db", str(db_path), "status", RUN_ID, "--recent-events", "5"])
    if cli_status["run_id"] != RUN_ID or not cli_status["event_chain"]["chain_valid"]:
        raise AssertionError(f"CLI status did not return valid run status: {cli_status}")
    cli_events = json_run(["--db", str(db_path), "events", RUN_ID, "--limit", "3"])
    if len(cli_events["events"]) != 3:
        raise AssertionError(f"CLI events limit failed: {cli_events}")
    cli_enqueue = json_run(
        [
            "--db",
            str(db_path),
            "enqueue",
            RUN_ID,
            "cli_collect_evidence",
            "--risk",
            "R2",
            "--actor-type",
            "plugin",
            "--actor-id",
            "nu-plugin-envctl",
            "--authority",
            "safe_execute",
            "--phase",
            PHASE,
            "--target-scope",
            "execution-framework/proof_records",
            "--input-json",
            '{"proof_required":true}',
        ]
    )
    if cli_enqueue["operation_status"] != "ready":
        raise AssertionError(f"CLI enqueue did not create ready R2 operation: {cli_enqueue}")
    return {
        "status": {
            "run_id": cli_status["run_id"],
            "event_count": cli_status["event_chain"]["event_count"],
            "chain_valid": cli_status["event_chain"]["chain_valid"],
            "visible_locks": cli_status["visible_locks"],
        },
        "events": {
            "returned": len(cli_events["events"]),
            "chain_valid": cli_events["event_chain"]["chain_valid"],
        },
        "enqueue": cli_enqueue,
    }


def write_doc(report: dict[str, Any]) -> None:
    lines = [
        "# Envctl Agent Control API",
        "",
        "Status: implemented and locally verified.",
        "",
        "This control surface exposes database-backed operations for agents, helpers, plugins, and humans. It uses the existing envctl migration tables for runs, operations, approvals, and append-only events.",
        "",
        "## CLI Commands",
        "",
        "- `status RUN_ID`: returns run status, approval summary, operation counts, visible locks, recent events, and event-chain validation.",
        "- `events RUN_ID`: returns the hash-chained event timeline.",
        "- `enqueue RUN_ID OPERATION_TYPE`: creates an idempotent controlled operation. R0-R2 operations become ready; R3-R5 operations open an approval and enter `awaiting_approval`.",
        "- `decision APPROVAL_ID`: approves or denies an open approval. Only `operator` and `admin` authority can decide.",
        "- `transition OPERATION_ID`: applies the existing operation state machine. R3+ operations cannot start until approved.",
        "- `lease RUN_ID` and `release-lease RUN_ID LEASE_ID`: record visible run-scoped target locks as append-only control events.",
        "",
        "## Verification",
        "",
        f"- Library smoke status: `{report['status']}`",
        f"- Event chain valid: `{report['library']['status_after_release']['event_chain']['chain_valid']}`",
        f"- CLI event count observed: `{report['cli']['status']['event_count']}`",
        f"- Proof record: `proof_records/{TASK_ID}.proof.json`",
    ]
    (root() / "docs" / "ENVCTL_AGENT_CONTROL_API.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    started = now()
    commands = [
        "python3 scripts/verify_agent_control_api.py",
        "python3 -m py_compile scripts/agent_control_api.py scripts/verify_agent_control_api.py",
    ]
    with tempfile.TemporaryDirectory(prefix="req028-agent-control-") as td:
        db_path = Path(td) / "agent_control.sqlite3"
        conn = sqlite3.connect(db_path)
        conn.row_factory = sqlite3.Row
        fixture = seed_fixture(conn)
        library = exercise_library(conn)
        cli = exercise_cli(db_path)
        final_status = AgentControlApi(conn).run_status(RUN_ID, recent_events=20)
        report = {
            "schema_version": "1.0",
            "task_id": TASK_ID,
            "status": "pass",
            "started_at": started,
            "completed_at": now(),
            "fixture": fixture,
            "library": library,
            "cli": cli,
            "final_status": final_status,
            "api_contract": {
                "status_endpoint": "GET /migration/runs/{run_id}/status",
                "events_endpoint": "GET /migration/runs/{run_id}/events",
                "approval_endpoint": "POST /migration/approvals/{approval_id}/decision",
                "authority_levels": [
                    "read_only",
                    "safe_execute",
                    "approval_request",
                    "operator",
                    "admin",
                ],
            },
        }
        report_path = root() / "generated" / "envctl_agent_control_report.json"
        write_json(report_path, report)
        write_doc(report)

    log_path = root() / "logs" / f"{TASK_ID}.log"
    log_path.write_text(
        "\n".join(
            [
                f"{TASK_ID} verification status=pass",
                f"event_chain_valid={report['final_status']['event_chain']['chain_valid']}",
                f"operations={report['final_status']['operation_count']}",
                f"open_approvals={report['final_status']['open_approval_count']}",
                f"cli_event_count={report['cli']['status']['event_count']}",
            ]
        )
        + "\n",
        encoding="utf-8",
    )
    heartbeat = {
        "task_id": TASK_ID,
        "status": "completed",
        "updated_at": now(),
        "helper_id": HELPER_ID,
        "proof_uri": f"proof_records/{TASK_ID}.proof.json",
        "logs_uri": f"logs/{TASK_ID}.log",
    }
    write_json(root() / "state" / f"{TASK_ID}.heartbeat.json", heartbeat)

    files_changed = [
        "execution-framework/scripts/agent_control_api.py",
        "execution-framework/scripts/verify_agent_control_api.py",
        "execution-framework/generated/envctl_agent_control_report.json",
        "execution-framework/docs/ENVCTL_AGENT_CONTROL_API.md",
        f"execution-framework/logs/{TASK_ID}.log",
        f"execution-framework/state/{TASK_ID}.heartbeat.json",
    ]
    proof = make_proof(
        TASK_ID,
        "completed",
        "envctl-runner-agent",
        HELPER_ID,
        MODEL_TAG,
        "execution-framework",
        files_changed,
        commands,
        {
            "agent_control_report": "generated/envctl_agent_control_report.json",
            "status": report["status"],
            "event_chain_valid": report["final_status"]["event_chain"]["chain_valid"],
            "cli_smoke_passed": True,
            "open_approvals": report["final_status"]["open_approval_count"],
            "operation_count": report["final_status"]["operation_count"],
        },
        [
            "generated/envctl_agent_control_report.json",
            "docs/ENVCTL_AGENT_CONTROL_API.md",
            f"logs/{TASK_ID}.log",
            f"state/{TASK_ID}.heartbeat.json",
        ],
    )
    append_proof(proof)
    print(
        "agent control api verification status=pass "
        f"events={report['final_status']['event_chain']['event_count']} "
        f"operations={report['final_status']['operation_count']}"
    )


if __name__ == "__main__":
    main()
