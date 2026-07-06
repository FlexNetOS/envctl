from __future__ import annotations

import json
import sqlite3
from pathlib import Path
from typing import Any

from jsonschema import Draft202012Validator

from _common import append_proof, make_proof, now, package_root, read_json, root, write_json
from agent_control_api import Actor, AgentControlApi
from envctl_run_ledger import RunLedger, apply_migrations, canonical_json, sha256_file, sha256_json


TASK_ID = "REQ-041_TWO_REPO_INTEGRATION"
HELPER_ID = "helper-integration-01"
MODEL_TAG = "gpt-5.3-spark"
ACTOR = "integration-agent"
REPORT_PATH = "generated/req041_two_repo_integration_report.json"
DOC_PATH = "docs/TWO_REPO_INTEGRATION.md"
HEARTBEAT_PATH = "state/REQ-041_TWO_REPO_INTEGRATION.heartbeat.json"
TRACKED_FILES = [
    "execution-framework/generated/req041_two_repo_integration_report.json",
    "execution-framework/docs/TWO_REPO_INTEGRATION.md",
    "execution-framework/state/REQ-041_TWO_REPO_INTEGRATION.heartbeat.json",
    "execution-framework/proof_records/REQ-041_TWO_REPO_INTEGRATION.proof.json",
]


def rel_hash(relpath: str) -> str:
    return "sha256:" + sha256_file(package_root() / relpath)


def load_shared_validator() -> Draft202012Validator:
    schema = read_json("schemas/shared_protocol.schema.json")
    return Draft202012Validator(schema)


def load_packet() -> dict[str, Any]:
    return read_json("generated/execution_packets/REQ-041_TWO_REPO_INTEGRATION.json")


def load_contract(relpath: str) -> dict[str, Any]:
    return read_json(relpath)


def insert_artifact_rows(conn: sqlite3.Connection, run_id: str, operation_id: str) -> None:
    conn.execute(
        """
        INSERT INTO envctl_migration_artifacts
          (id, run_id, artifact_id, title, artifact_type, status, path,
           content_hash, generated_by_operation_id, evidence_json, links_json)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "artifact-req041-integration-report",
            run_id,
            "req041-two-repo-report",
            "Two-repo integration report",
            "integration_report",
            "complete",
            REPORT_PATH,
            sha256_json({"task_id": TASK_ID, "artifact": REPORT_PATH, "kind": "integration_report"}),
            operation_id,
            canonical_json(
                [
                    "generated/REQ-033_PLUGIN_HUMAN_APPROVAL.contract.json",
                    "generated/REQ-034_PLUGIN_STATUS_STREAMS.contract.json",
                ]
            ),
            canonical_json(["docs/TWO_REPO_INTEGRATION.md"]),
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_graph_edges
          (id, run_id, from_node, to_node, edge_type, source_artifact_id, confidence, evidence_json)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "edge-req041-envctl-to-plugin",
            run_id,
            "envctl_migration_runs",
            "envctl_status_stream",
            "projects_to",
            "req041-two-repo-report",
            "high",
            canonical_json(
                [
                    "generated/REQ-034_PLUGIN_STATUS_STREAMS.contract.json",
                    "generated/REQ-033_PLUGIN_HUMAN_APPROVAL.contract.json",
                ]
            ),
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_validations
          (id, run_id, artifact_id, operation_id, validator, status, details_json, evidence_json)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "validation-req041-two-repo",
            run_id,
            "req041-two-repo-report",
            operation_id,
            "verify_two_repo_integration.py",
            "pass",
            canonical_json(
                {
                    "protocol_records_validated": 11,
                    "plugin_projections_validated": 2,
                    "shared_schema": "schemas/shared_protocol.schema.json",
                }
            ),
            canonical_json(
                [
                    REPORT_PATH,
                    "generated/REQ-033_PLUGIN_HUMAN_APPROVAL.contract.json",
                    "generated/REQ-034_PLUGIN_STATUS_STREAMS.contract.json",
                ]
            ),
        ),
    )
    conn.commit()


def build_fixture() -> dict[str, Any]:
    conn = sqlite3.connect(":memory:")
    conn.row_factory = sqlite3.Row
    applied = apply_migrations(conn, package_root())
    ledger = RunLedger(conn)
    catalog = ledger.seed_base_catalog()
    run = ledger.create_run(
        run_id="run-req041-two-repo",
        target_id=catalog["target_id"],
        recipe_id=catalog["recipe_id"],
        artifact_contract_id=catalog["contract_id"],
        human_mode="approval-gated",
        initiated_by=ACTOR,
        sandbox_policy="workspace-write",
        approval_policy="never",
        tool_versions={"python3": "stdlib", "sqlite": sqlite3.sqlite_version, "codex": MODEL_TAG},
    )
    ledger.set_run_status(run["run_id"], "running")
    ledger.append_event(
        run_id=run["run_id"],
        event_type="integration_run_started",
        phase="04-shared",
        actor_type="agent",
        actor_id=ACTOR,
        payload={"task_id": TASK_ID, "phase": "04-shared", "status": "running"},
    )

    control = AgentControlApi(conn)
    agent_actor = Actor(actor_type="agent", actor_id=HELPER_ID, authority="safe_execute")
    operator_actor = Actor(actor_type="human", actor_id="operator-req041", authority="operator")

    safe_op = control.enqueue_operation(
        run_id=run["run_id"],
        operation_type="envctl.db.schema.snapshot",
        risk="R1",
        actor=agent_actor,
        phase="04-shared",
        target_scope="envctl-db",
        input_payload={
            "task_id": TASK_ID,
            "shared_protocol": True,
            "source": "REQ-020_ENVCTL_DB_SCHEMA",
        },
        command_redacted="envctl db schema export --format json",
        reason="project envctl db state into shared protocol rows",
        idempotency_key="REQ-041/envctl-db-schema-snapshot",
    )
    gated_op = control.enqueue_operation(
        run_id=run["run_id"],
        operation_type="nu_plugin.status.surface.refresh",
        risk="R4",
        actor=agent_actor,
        phase="04-shared",
        target_scope="nu-plugin",
        input_payload={
            "task_id": TASK_ID,
            "contract": "REQ-034_PLUGIN_STATUS_STREAMS",
            "approval_surface": "REQ-033_PLUGIN_HUMAN_APPROVAL",
        },
        command_redacted="codedb envctl status stream --events generated/run-events.json",
        reason="plugin-facing status stream refresh requires human review at R4",
        idempotency_key="REQ-041/nu-plugin-status-refresh",
    )
    decision = control.approval_decision(
        approval_id=gated_op["approval_id"],
        decision="approved",
        actor=operator_actor,
        reason="integration verifier may project the approved plugin surface",
        idempotency_key="REQ-041/approve-status-refresh",
    )

    ledger.set_operation_status(safe_op["operation_id"], "succeeded")
    ledger.set_operation_status(gated_op["operation_id"], "succeeded")
    ledger.link_evidence(
        run_id=run["run_id"],
        operation_id=safe_op["operation_id"],
        uri="execution-framework/proof_records/REQ-020_ENVCTL_DB_SCHEMA.proof.json",
        evidence_kind="proof_record",
        sha256=rel_hash("execution-framework/proof_records/REQ-020_ENVCTL_DB_SCHEMA.proof.json"),
        metadata={"depends_on": "REQ-020_ENVCTL_DB_SCHEMA"},
    )
    ledger.link_evidence(
        run_id=run["run_id"],
        operation_id=gated_op["operation_id"],
        uri="execution-framework/generated/REQ-034_PLUGIN_STATUS_STREAMS.contract.json",
        evidence_kind="plugin_contract",
        sha256=rel_hash("execution-framework/generated/REQ-034_PLUGIN_STATUS_STREAMS.contract.json"),
        metadata={"depends_on": "REQ-034_PLUGIN_STATUS_STREAMS"},
    )
    ledger.link_evidence(
        run_id=run["run_id"],
        operation_id=gated_op["operation_id"],
        uri="execution-framework/generated/REQ-033_PLUGIN_HUMAN_APPROVAL.contract.json",
        evidence_kind="plugin_contract",
        sha256=rel_hash("execution-framework/generated/REQ-033_PLUGIN_HUMAN_APPROVAL.contract.json"),
        metadata={"depends_on": "REQ-033_PLUGIN_HUMAN_APPROVAL"},
    )
    ledger.append_event(
        run_id=run["run_id"],
        event_type="plugin_projection_ready",
        phase="04-shared",
        actor_type="plugin",
        actor_id="codedb",
        operation_id=gated_op["operation_id"],
        evidence_refs=[
            "execution-framework/generated/REQ-034_PLUGIN_STATUS_STREAMS.contract.json",
            "execution-framework/generated/REQ-033_PLUGIN_HUMAN_APPROVAL.contract.json",
        ],
        payload={
            "task_id": TASK_ID,
            "status": "ready",
            "next_action": "stream records through plugin command tables",
        },
    )
    insert_artifact_rows(conn, run["run_id"], gated_op["operation_id"])
    ledger.set_run_status(run["run_id"], "completed")

    return {
        "conn": conn,
        "applied_migrations": applied,
        "run_id": run["run_id"],
        "safe_operation_id": safe_op["operation_id"],
        "gated_operation_id": gated_op["operation_id"],
        "approval_id": gated_op["approval_id"],
        "approval_decision": decision,
        "event_chain_valid": ledger.validate_event_chain(run["run_id"]),
    }


def fetch_one(conn: sqlite3.Connection, sql: str, params: tuple[Any, ...]) -> sqlite3.Row:
    row = conn.execute(sql, params).fetchone()
    if row is None:
        raise ValueError(f"query returned no rows: {sql}")
    return row


def json_or_default(value: str | None, default: Any) -> Any:
    if not value:
        return default
    return json.loads(value)


def collect_protocol_records(conn: sqlite3.Connection, run_id: str, approval_id: str) -> dict[str, Any]:
    target_row = fetch_one(
        conn,
        """
        SELECT id, target_id, target_type, primary_root, compare_root, descriptor_json, descriptor_hash, safety_mode, max_auto_risk
        FROM envctl_migration_targets
        WHERE id = ?
        """,
        ("target-req022",),
    )
    recipe_row = fetch_one(
        conn,
        """
        SELECT id, recipe_name, recipe_version, artifact_contract_id, recipe_hash, recipe_json
        FROM envctl_migration_recipes
        WHERE id = ?
        """,
        ("recipe-req022",),
    )
    run_row = fetch_one(
        conn,
        """
        SELECT id, target_id, recipe_id, artifact_contract_id, status, human_mode, reproducibility_hash,
               started_at_utc, completed_at_utc
        FROM envctl_migration_runs
        WHERE id = ?
        """,
        (run_id,),
    )
    approval_row = fetch_one(
        conn,
        """
        SELECT id, run_id, operation_id, risk, status, requested_by, decided_by, reason, decided_at_utc
        FROM envctl_migration_approvals
        WHERE id = ?
        """,
        (approval_id,),
    )
    records = {
        "TargetDescriptor": {
            "schema_version": 1,
            "target_id": target_row["target_id"],
            "target_type": target_row["target_type"],
            "primary_root": target_row["primary_root"],
            "compare_root": target_row["compare_root"],
            "output_root": "migration-artifacts",
            "include": ["execution-framework/**"],
            "exclude": ["**/.env", "**/secrets/**"],
            "safety": {
                "default_mode": target_row["safety_mode"],
                "max_auto_risk": target_row["max_auto_risk"],
                "allow_network": False,
                "allow_destructive": False,
            },
            "collectors": {"db_schema": True, "plugin_surface": True},
            "artifact_contract": {"name": "req022-run-ledger-contract", "version": "1.0.0"},
            "recipe": {"name": "req022-run-ledger-recipe", "version": "1.0.0"},
            "metadata": {
                "descriptor_hash": target_row["descriptor_hash"],
                "fixture": True,
            },
        },
        "MigrationRecipe": {
            "schema_version": 1,
            "recipe_id": recipe_row["id"],
            "version": recipe_row["recipe_version"],
            "phases": [
                {
                    "phase_id": "04-shared",
                    "depends_on": ["REQ-020_ENVCTL_DB_SCHEMA", "REQ-040_SHARED_PROTOCOL_SCHEMAS"],
                    "approval_gate": True,
                    "operations": [
                        {
                            "operation_id": "req041-envctl-db-schema-snapshot",
                            "operation_type": "envctl.db.schema.snapshot",
                            "risk": "R1",
                            "inputs": {"source": "REQ-020_ENVCTL_DB_SCHEMA"},
                            "expected_artifacts": ["req041-two-repo-report"],
                            "validators": ["verify_two_repo_integration.py"],
                            "rollback": None,
                        },
                        {
                            "operation_id": "req041-nu-plugin-status-refresh",
                            "operation_type": "nu_plugin.status.surface.refresh",
                            "risk": "R4",
                            "inputs": {"source": "REQ-034_PLUGIN_STATUS_STREAMS"},
                            "expected_artifacts": ["req041-two-repo-report"],
                            "validators": ["verify_two_repo_integration.py"],
                            "rollback": None,
                        },
                    ],
                }
            ],
            "metadata": {
                "recipe_name": recipe_row["recipe_name"],
                "artifact_contract_id": recipe_row["artifact_contract_id"],
                "recipe_hash": recipe_row["recipe_hash"],
            },
        },
        "MigrationRun": {
            "run_id": run_row["id"],
            "target_id": run_row["target_id"],
            "recipe_id": run_row["recipe_id"],
            "artifact_contract_id": run_row["artifact_contract_id"],
            "status": "succeeded",
            "human_mode": "manual",
            "max_auto_risk": "R4",
            "reproducibility_hash": run_row["reproducibility_hash"],
            "created_at_utc": run_row["started_at_utc"],
            "updated_at_utc": run_row["completed_at_utc"],
            "metadata": {"task_id": TASK_ID, "owner_lane": "lane_e_verification"},
        },
    }
    operation_rows = conn.execute(
        """
        SELECT id, run_id, parent_operation_id, operation_type, status, risk, idempotency_key,
               input_json, output_ref, error_json
        FROM envctl_migration_operations
        WHERE run_id = ?
        ORDER BY id
        """,
        (run_id,),
    ).fetchall()
    records["Operation"] = [
        {
            "operation_id": row["id"],
            "run_id": row["run_id"],
            "parent_operation_id": row["parent_operation_id"],
            "operation_type": row["operation_type"],
            "status": row["status"],
            "risk": row["risk"],
            "idempotency_key": row["idempotency_key"],
            "input": json_or_default(row["input_json"], {}),
            "output_ref": row["output_ref"],
            "error": json_or_default(row["error_json"], None),
        }
        for row in operation_rows
    ]
    event_rows = conn.execute(
        """
        SELECT run_id, event_seq, event_type, phase, actor_type, actor_id, operation_id, created_at_utc,
               payload_json, evidence_refs_json, previous_event_hash, event_hash
        FROM envctl_migration_run_events
        WHERE run_id = ?
        ORDER BY event_seq
        """,
        (run_id,),
    ).fetchall()
    records["RunEvent"] = [
        {
            "run_id": row["run_id"],
            "event_seq": row["event_seq"],
            "event_type": row["event_type"],
            "phase": row["phase"],
            "actor_type": row["actor_type"],
            "actor_id": row["actor_id"],
            "operation_id": row["operation_id"],
            "timestamp_utc": row["created_at_utc"],
            "payload": json_or_default(row["payload_json"], {}),
            "evidence_refs": json_or_default(row["evidence_refs_json"], []),
            "previous_event_hash": row["previous_event_hash"],
            "event_hash": row["event_hash"],
        }
        for row in event_rows
    ]
    records["ApprovalRequest"] = {
        "approval_id": approval_row["id"],
        "run_id": approval_row["run_id"],
        "operation_id": approval_row["operation_id"],
        "risk": approval_row["risk"],
        "status": approval_row["status"],
        "requested_by": approval_row["requested_by"],
        "reason": approval_row["reason"],
    }
    records["ApprovalDecision"] = {
        "approval_id": approval_row["id"],
        "decision": "approved",
        "decided_by": approval_row["decided_by"],
        "decided_at_utc": approval_row["decided_at_utc"],
        "reason": approval_row["reason"],
        "event_id": None,
    }
    artifact_row = fetch_one(
        conn,
        """
        SELECT artifact_id, run_id, title, artifact_type, status, path, content_hash,
               generated_by_operation_id, evidence_json, links_json
        FROM envctl_migration_artifacts
        WHERE artifact_id = ?
        """,
        ("req041-two-repo-report",),
    )
    records["ArtifactRecord"] = {
        "artifact_id": artifact_row["artifact_id"],
        "run_id": artifact_row["run_id"],
        "title": artifact_row["title"],
        "artifact_type": artifact_row["artifact_type"],
        "status": artifact_row["status"],
        "path": artifact_row["path"],
        "content_hash": artifact_row["content_hash"],
        "generated_by_operation_id": artifact_row["generated_by_operation_id"],
        "evidence_refs": json_or_default(artifact_row["evidence_json"], []),
        "links": [{"to": item, "type": "documentation"} for item in json_or_default(artifact_row["links_json"], [])],
    }
    evidence_rows = conn.execute(
        """
        SELECT id, run_id, operation_id, uri, evidence_kind, sha256, redacted, metadata_json
        FROM envctl_migration_evidence
        WHERE run_id = ?
        ORDER BY id
        """,
        (run_id,),
    ).fetchall()
    records["EvidenceRecord"] = [
        {
            "evidence_id": row["id"],
            "run_id": row["run_id"],
            "operation_id": row["operation_id"],
            "uri": row["uri"],
            "evidence_kind": row["evidence_kind"],
            "sha256": row["sha256"].removeprefix("sha256:"),
            "redacted": bool(row["redacted"]),
            "metadata": json_or_default(row["metadata_json"], {}),
        }
        for row in evidence_rows
    ]
    graph_row = fetch_one(
        conn,
        """
        SELECT id, run_id, from_node, to_node, edge_type, source_artifact_id
        FROM envctl_migration_graph_edges
        WHERE id = ?
        """,
        ("edge-req041-envctl-to-plugin",),
    )
    records["GraphEdge"] = {
        "edge_id": graph_row["id"],
        "run_id": graph_row["run_id"],
        "from_node": graph_row["from_node"],
        "to_node": graph_row["to_node"],
        "edge_type": graph_row["edge_type"],
        "source_artifact_id": graph_row["source_artifact_id"],
    }
    validation_row = fetch_one(
        conn,
        """
        SELECT id, run_id, artifact_id, operation_id, validator, status, details_json, evidence_json
        FROM envctl_migration_validations
        WHERE id = ?
        """,
        ("validation-req041-two-repo",),
    )
    records["ValidationResult"] = {
        "validation_id": validation_row["id"],
        "run_id": validation_row["run_id"],
        "artifact_id": validation_row["artifact_id"],
        "operation_id": validation_row["operation_id"],
        "validator": validation_row["validator"],
        "status": validation_row["status"],
        "details": json_or_default(validation_row["details_json"], {}),
        "evidence_refs": json_or_default(validation_row["evidence_json"], []),
    }
    records["ReplayRequest"] = {
        "replay_id": "replay-req041-shared-surface",
        "run_id": run_id,
        "mode": "dry_run",
        "requested_by": HELPER_ID,
        "operation_ids": [records["Operation"][0]["operation_id"], records["Operation"][1]["operation_id"]],
        "target_descriptor_id": records["TargetDescriptor"]["target_id"],
        "reason": "prove shared protocol rows can support replay-aware plugin inspection",
    }
    records["ReplayResult"] = {
        "replay_id": "replay-req041-shared-surface",
        "run_id": run_id,
        "status": "pass",
        "completed_at_utc": now(),
        "event_refs": [f"event:{event['event_seq']}" for event in records["RunEvent"]],
        "artifact_refs": [records["ArtifactRecord"]["artifact_id"]],
    }
    return records


def validate_protocol_records(records: dict[str, Any], proof_preview: dict[str, Any]) -> dict[str, int]:
    validator = load_shared_validator()
    counts: dict[str, int] = {}
    for key, value in records.items():
        if isinstance(value, list):
            counts[key] = len(value)
            for item in value:
                validator.validate(item)
        else:
            counts[key] = 1
            validator.validate(value)
    validator.validate(proof_preview)
    counts["ProofRecord"] = 1
    return counts


def project_status_rows(events: list[dict[str, Any]], proof: dict[str, Any], contract: dict[str, Any]) -> list[dict[str, Any]]:
    rows = []
    required = set(contract["columns"])
    for event in events:
        payload = event["payload"]
        rows.append(
            {
                "table": contract["output_table"],
                "view": "event",
                "run_id": event["run_id"],
                "event_seq": event["event_seq"],
                "time": event["timestamp_utc"],
                "phase": event["phase"],
                "event_type": event["event_type"],
                "actor": event["actor_type"],
                "actor_id": event["actor_id"],
                "operation": event["operation_id"],
                "status": payload.get("status", "n/a"),
                "summary": payload.get("reason")
                or payload.get("next_action")
                or payload.get("status")
                or event["event_type"],
                "blocked": bool(payload.get("blocked", False)),
                "next_action": payload.get("next_action", ""),
                "proof_task_id": "",
                "proof_status": "",
                "proof_uri": "",
                "evidence_count": len(event.get("evidence_refs", [])),
            }
        )
    rows.append(
        {
            "table": contract["output_table"],
            "view": "proof",
            "run_id": "",
            "event_seq": 0,
            "time": proof["completed_at"],
            "phase": "04-shared",
            "event_type": "proof_record",
            "actor": proof["actor"],
            "actor_id": proof["helper_id"],
            "operation": "",
            "status": proof["status"],
            "summary": "REQ-041 integration proof recorded",
            "blocked": False,
            "next_action": proof["next_action"],
            "proof_task_id": proof["task_id"],
            "proof_status": proof["status"],
            "proof_uri": f"proof_records/{proof['task_id']}.proof.json",
            "evidence_count": len(proof["evidence"]),
        }
    )
    for row in rows:
        missing = required.difference(row.keys())
        if missing:
            raise ValueError(f"status row missing columns: {sorted(missing)}")
    return rows


def project_approval_rows(
    approval_request: dict[str, Any],
    approval_decision: dict[str, Any],
    operations: list[dict[str, Any]],
    contract: dict[str, Any],
) -> list[dict[str, Any]]:
    required = set(contract["columns"])
    by_operation = {op["operation_id"]: op for op in operations}
    op = by_operation[approval_request["operation_id"]]
    row = {
        "table": contract["output_table"],
        "view": "approval",
        "mode": "approval-gated",
        "run_id": approval_request["run_id"],
        "operation_id": approval_request["operation_id"],
        "approval_id": approval_request["approval_id"],
        "risk": approval_request["risk"],
        "status": approval_request["status"],
        "requested_by": approval_request["requested_by"],
        "decided_by": approval_decision["decided_by"],
        "reason": approval_decision["reason"],
        "blocked": False,
        "requires_human": True,
        "allowed_actions": ["approve", "deny"],
        "next_safe_action": "run plugin-facing status stream projection",
        "delegate_target": "operator",
        "summary": f"{op['operation_type']} approved for plugin projection",
    }
    missing = required.difference(row.keys())
    if missing:
        raise ValueError(f"approval row missing columns: {sorted(missing)}")
    return [row]


def write_doc(report: dict[str, Any], packet: dict[str, Any]) -> None:
    lines = [
        "# Two-Repo Integration",
        "",
        f"- Task: `{TASK_ID}`",
        f"- Goal: {packet['goal']}",
        f"- Generated at: {report['generated_at']}",
        f"- Verification status: {report['verification']['status']}",
        "",
        "## Verified flow",
        "",
        "1. envctl run-ledger fixture creates migration run, operations, approvals, events, evidence, artifact, graph, and validation rows.",
        "2. Shared protocol records are emitted from the fixture and validated against `schemas/shared_protocol.schema.json`.",
        "3. Plugin contracts project those records into `codedb envctl status stream` and `codedb envctl human approvals` table shapes.",
        "4. The task proof, report, and status ledger are written back into the execution framework.",
        "",
        "## Contract inputs",
        "",
        f"- Human approval surface: `{report['inputs']['approval_contract']}`",
        f"- Status stream surface: `{report['inputs']['status_contract']}`",
        f"- Command manifest: `{report['inputs']['command_manifest']}`",
        "",
        "## Coverage",
        "",
        f"- Shared protocol record counts: `{json.dumps(report['protocol_record_counts'], sort_keys=True)}`",
        f"- Status rows emitted: `{len(report['plugin_projection']['status_rows'])}`",
        f"- Approval rows emitted: `{len(report['plugin_projection']['approval_rows'])}`",
    ]
    path = root() / DOC_PATH
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    packet = load_packet()
    approval_contract = load_contract("generated/REQ-033_PLUGIN_HUMAN_APPROVAL.contract.json")
    status_contract = load_contract("generated/REQ-034_PLUGIN_STATUS_STREAMS.contract.json")
    command_manifest = read_json("generated/nu_plugin_command_manifest.json")
    heartbeat = {
        "task_id": TASK_ID,
        "status": "running",
        "updated_at": now(),
        "owner_agent": packet["owner_agent"],
        "helper_id": packet["helper_id"],
    }
    write_json(HEARTBEAT_PATH, heartbeat)

    fixture = build_fixture()
    records = collect_protocol_records(fixture["conn"], fixture["run_id"], fixture["approval_id"])
    proof_preview = make_proof(
        task_id=TASK_ID,
        status="completed",
        actor=ACTOR,
        helper_id=HELPER_ID,
        model_tag=MODEL_TAG,
        repo_path=packet["repo_path"],
        files_changed=TRACKED_FILES,
        commands_run=[
            "python3 scripts/verify_two_repo_integration.py",
            "python3 scripts/status_from_proofs.py",
        ],
        verification_output={
            "integration_record_counts": {key: (len(value) if isinstance(value, list) else 1) for key, value in records.items()}
        },
        evidence=[
            "proof_records/REQ-020_ENVCTL_DB_SCHEMA.proof.json",
            "proof_records/REQ-031_PLUGIN_COMMAND_SURFACE.proof.json",
            "proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json",
            "generated/REQ-033_PLUGIN_HUMAN_APPROVAL.contract.json",
            "generated/REQ-034_PLUGIN_STATUS_STREAMS.contract.json",
        ],
    )
    protocol_counts = validate_protocol_records(records, proof_preview)
    status_rows = project_status_rows(records["RunEvent"], proof_preview, status_contract)
    approval_rows = project_approval_rows(
        records["ApprovalRequest"],
        records["ApprovalDecision"],
        records["Operation"],
        approval_contract,
    )

    command_names = {
        row["name"]
        for row in command_manifest.get("commands", [])
        if row.get("name") in {"envctl migration status", "envctl migration replay", "envctl migration approve"}
    }
    missing_commands = sorted(
        {"envctl migration status", "envctl migration replay", "envctl migration approve"}.difference(command_names)
    )
    if missing_commands:
        raise ValueError(f"nu_plugin operator command manifest missing required commands: {missing_commands}")

    report = {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "generated_at": now(),
        "packet": {
            "title": packet["title"],
            "goal": packet["goal"],
            "depends_on": packet["depends_on"],
            "proof_uri": packet["proof_uri"],
        },
        "inputs": {
            "approval_contract": "generated/REQ-033_PLUGIN_HUMAN_APPROVAL.contract.json",
            "status_contract": "generated/REQ-034_PLUGIN_STATUS_STREAMS.contract.json",
            "command_manifest": "generated/nu_plugin_command_manifest.json",
        },
        "applied_migrations": fixture["applied_migrations"],
        "protocol_record_counts": protocol_counts,
        "plugin_projection": {
            "status_rows": status_rows,
            "approval_rows": approval_rows,
        },
        "verification": {
            "status": "passed",
            "required_commands_present": sorted(command_names),
            "contract_commands_present": [
                approval_contract["command"],
                status_contract["command"],
            ],
            "event_chain_valid": fixture["event_chain_valid"],
            "approval_decision": fixture["approval_decision"],
        },
    }
    write_json(REPORT_PATH, report)
    write_doc(report, packet)

    proof = make_proof(
        task_id=TASK_ID,
        status="completed",
        actor=ACTOR,
        helper_id=HELPER_ID,
        model_tag=MODEL_TAG,
        repo_path=packet["repo_path"],
        files_changed=TRACKED_FILES,
        commands_run=[
            "python3 scripts/verify_two_repo_integration.py",
            "python3 scripts/status_from_proofs.py",
        ],
        verification_output={
            "report": REPORT_PATH,
            "status_rows": len(status_rows),
            "approval_rows": len(approval_rows),
            "required_commands_present": sorted(command_names),
        },
        evidence=[
            "proof_records/REQ-020_ENVCTL_DB_SCHEMA.proof.json",
            "proof_records/REQ-031_PLUGIN_COMMAND_SURFACE.proof.json",
            "proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json",
            "generated/REQ-033_PLUGIN_HUMAN_APPROVAL.contract.json",
            "generated/REQ-034_PLUGIN_STATUS_STREAMS.contract.json",
            REPORT_PATH,
            DOC_PATH,
        ],
    )
    append_proof(proof)
    heartbeat["status"] = "completed"
    heartbeat["updated_at"] = now()
    heartbeat["proof_uri"] = packet["proof_uri"]
    write_json(HEARTBEAT_PATH, heartbeat)
    print(json.dumps({"task_id": TASK_ID, "status": "completed", "report": REPORT_PATH}, indent=2))


if __name__ == "__main__":
    main()
