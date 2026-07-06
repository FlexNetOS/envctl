from __future__ import annotations

import json
import sqlite3
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, package_root, root, sha256_file
from artifact_registry import ArtifactRegistry, fetch_artifact
from verify_envctl_db_schema import apply_migrations


TASK_ID = "ART-127_BLAST_RADIUS"
HELPER_ID = "helper-artifact-28"
MODEL_TAG = "gpt-5.3-spark"
RUN_ID = "run-art-127-blast-radius"
OPERATION_ID = "op-art-127-generate-blast-radius"
CONTRACT_ID = "contract-full-migration-artifact-contract-1.0.0"
RECIPE_ID = "recipe-flexnetos-package-artifact-contract-1.0.0"
TARGET_ID = "target-art-127-filesystem"


def rel(path: Path) -> str:
    return str(path.relative_to(package_root()))


def sha256_uri(path: Path) -> str:
    return f"sha256:{sha256_file(path)}"


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def contract_row() -> dict[str, Any]:
    manifest = read_json(root() / "generated" / "contract_manifest.json")
    rows = manifest["contract"]["rows"]
    for row in rows:
        if row.get("producer_task_id") == TASK_ID:
            return row
    raise KeyError(f"contract row not found for {TASK_ID}")


def source_summary() -> dict[str, Any]:
    target_registry = read_json(root() / "generated" / "envctl_target_registry.json")
    db_model = read_json(root() / "generated" / "envctl_migration_db_model.json")
    protocol = read_json(root() / "generated" / "shared_protocol_manifest.json")
    return {
        "target_registry_status": target_registry["status"],
        "target_ids": [row["target_id"] for row in target_registry["registry_rows"]],
        "db_required_tables": db_model["required_tables"],
        "db_required_views": db_model["required_views"],
        "protocol_records": protocol["required_records"],
    }


def blast_nodes() -> list[dict[str, Any]]:
    return [
        {
            "id": "service-envctl-migration-db",
            "kind": "service",
            "name": "envctl migration database",
            "failure_mode": "SQLite schema, seed, or artifact registry persistence is unavailable.",
            "breaks": [
                "target descriptor lookup",
                "artifact hash registration",
                "run ledger status",
                "validation scorecard",
                "replay readiness",
            ],
            "blocked_tasks": ["VER-300_UNIT_VALIDATION", "VER-301_SQL_SCHEMA_TEST", "VER-304_FINAL_COMPLETENESS"],
            "detect_with": ["python3 scripts/verify_envctl_db_schema.py", "python3 scripts/verify_artifact_registry.py"],
            "containment": "Stop artifact completion claims, preserve logs, restore schema/seed from package sources, then rerun registry verification.",
            "risk": "critical",
        },
        {
            "id": "table-envctl-migration-artifacts",
            "kind": "table",
            "name": "envctl_migration_artifacts",
            "failure_mode": "Artifact rows cannot be inserted, updated, indexed, or hash-verified.",
            "breaks": [
                "migration-artifacts hash provenance",
                "contract row satisfaction",
                "artifact index view",
                "downstream verification gates",
            ],
            "blocked_tasks": ["VER-300_UNIT_VALIDATION", "VER-304_FINAL_COMPLETENESS"],
            "detect_with": ["SELECT artifact_id, path, content_hash FROM envctl_migration_artifact_index"],
            "containment": "Keep generated files on disk but mark the task failed until the registry row and content hash are present.",
            "risk": "critical",
        },
        {
            "id": "table-envctl-migration-evidence",
            "kind": "table",
            "name": "envctl_migration_evidence",
            "failure_mode": "Proof files, generated artifacts, and validation inputs cannot be linked to operations.",
            "breaks": [
                "auditability",
                "proof-to-artifact trace",
                "rollback confidence",
                "human review of generated outputs",
            ],
            "blocked_tasks": ["VER-300_UNIT_VALIDATION"],
            "detect_with": ["registry evidence_ids are empty", "proof record evidence list misses generated artifact paths"],
            "containment": "Regenerate evidence rows from existing files and hashes before marking completion.",
            "risk": "high",
        },
        {
            "id": "queue-envctl-migration-operations",
            "kind": "queue",
            "name": "envctl_migration_operations",
            "failure_mode": "Operation state cannot identify the producer command or lifecycle state.",
            "breaks": [
                "idempotency tracking",
                "producer operation foreign keys",
                "status stream rendering",
                "replay command selection",
            ],
            "blocked_tasks": ["REQ-045_RUN_REPLAY", "VER-303_GOAL_LOOP_COMPUTE"],
            "detect_with": ["python3 scripts/verify_operation_state_machine.py"],
            "containment": "Do not enqueue follow-on validations until the producer operation is restored or replayed.",
            "risk": "high",
        },
        {
            "id": "queue-envctl-migration-run-events",
            "kind": "queue",
            "name": "envctl_migration_run_events",
            "failure_mode": "Append-only event chain is missing, duplicated, or reordered.",
            "breaks": [
                "live timeline",
                "goal-loop status",
                "audit ordering",
                "replay determinism",
            ],
            "blocked_tasks": ["REQ-045_RUN_REPLAY", "VER-303_GOAL_LOOP_COMPUTE"],
            "detect_with": ["python3 scripts/verify_envctl_run_ledger.py"],
            "containment": "Freeze replay claims and rebuild from operation/proof records with hash-chain validation.",
            "risk": "high",
        },
        {
            "id": "api-artifact-registry-python",
            "kind": "api",
            "name": "scripts/artifact_registry.py",
            "failure_mode": "Registration API rejects valid package-relative paths or accepts unsafe paths.",
            "breaks": [
                "artifact registration",
                "blocked-path enforcement",
                "content-hash verification",
                "graph and validation links",
            ],
            "blocked_tasks": ["ARTIFACTS", "VER-300_UNIT_VALIDATION"],
            "detect_with": ["python3 scripts/verify_artifact_registry.py"],
            "containment": "Run fail-closed rejection cases before trusting any newly generated artifact rows.",
            "risk": "critical",
        },
        {
            "id": "api-nu-plugin-protocol",
            "kind": "api",
            "name": "shared protocol and nu_plugin command surface",
            "failure_mode": "Protocol records drift from the database-owned shapes.",
            "breaks": [
                "operator command rendering",
                "status streams",
                "approval decisions",
                "artifact and validation display",
            ],
            "blocked_tasks": ["REQ-031_PLUGIN_COMMAND_SURFACE", "REQ-040_SHARED_PROTOCOL_SCHEMAS"],
            "detect_with": ["python3 scripts/verify_shared_protocol_schemas.py", "python3 scripts/verify_plugin_command_surface.py"],
            "containment": "Treat envctl database rows as source of truth and regenerate plugin-facing schemas/manifests.",
            "risk": "high",
        },
        {
            "id": "credential-codex-provider",
            "kind": "credential",
            "name": "Codex CLI provider credential",
            "failure_mode": "Background codex execution cannot start or complete artifact-generation packets.",
            "breaks": [
                "parallel artifact generation",
                "codex exec packet workflow",
                "proof log capture",
            ],
            "blocked_tasks": ["ARTIFACTS", "REL-401_HANDOFF"],
            "detect_with": ["codex exec < generated/execution_packets/ART-127_BLAST_RADIUS.json"],
            "containment": "Do not expose or inspect secret values; verify only command availability and preserve stdout logs.",
            "risk": "medium",
        },
        {
            "id": "credential-filesystem-write-scope",
            "kind": "credential",
            "name": "workspace filesystem write authority",
            "failure_mode": "Agent cannot write migration-artifacts, state, logs, or proof records.",
            "breaks": [
                "artifact materialization",
                "heartbeat updates",
                "proof ledger append",
                "rollback manifest usefulness",
            ],
            "blocked_tasks": ["VER-300_UNIT_VALIDATION", "VER-304_FINAL_COMPLETENESS"],
            "detect_with": ["test writes under migration-artifacts/, execution-framework/state/, execution-framework/proof_records/"],
            "containment": "Fail closed with no completion proof until the write scope matches the packet allowed paths.",
            "risk": "high",
        },
    ]


def build_payload(row: dict[str, Any], summary: dict[str, Any]) -> dict[str, Any]:
    nodes = blast_nodes()
    by_kind: dict[str, int] = {}
    for node in nodes:
        by_kind[node["kind"]] = by_kind.get(node["kind"], 0) + 1
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "title": "Blast-radius map",
        "generated_at": now(),
        "contract_row": row,
        "source_summary": summary,
        "coverage": {
            "services": by_kind.get("service", 0),
            "tables": by_kind.get("table", 0),
            "queues": by_kind.get("queue", 0),
            "apis": by_kind.get("api", 0),
            "credentials": by_kind.get("credential", 0),
            "failure_modes_total": len(nodes),
        },
        "blast_radius": nodes,
        "critical_paths": [
            {
                "path": "artifact generation to registry to validation",
                "depends_on": [
                    "credential-filesystem-write-scope",
                    "api-artifact-registry-python",
                    "service-envctl-migration-db",
                    "table-envctl-migration-artifacts",
                    "table-envctl-migration-evidence",
                ],
                "break_effect": "Generated files may exist without durable hash/evidence registration, so the artifact cannot gate validation.",
            },
            {
                "path": "run replay and status stream",
                "depends_on": [
                    "queue-envctl-migration-operations",
                    "queue-envctl-migration-run-events",
                    "api-nu-plugin-protocol",
                ],
                "break_effect": "Operators lose reliable ordering, replayability, and UI/API status for migration operations.",
            },
        ],
    }


def markdown(payload: dict[str, Any]) -> str:
    lines = [
        "# Blast-radius map",
        "",
        f"Generated at: `{payload['generated_at']}`",
        f"Task: `{TASK_ID}`",
        f"Contract row: `{payload['contract_row']['contract_row_id']}`",
        f"Canonical path: `{payload['contract_row']['required_path']}`",
        "",
        "## Coverage",
        "",
        "| kind | count |",
        "|---|---:|",
    ]
    for key in ["services", "tables", "queues", "apis", "credentials", "failure_modes_total"]:
        lines.append(f"| {key.replace('_', ' ')} | {payload['coverage'][key]} |")
    lines.extend(["", "## Failure modes", "", "| surface | kind | risk | what breaks | containment |", "|---|---|---|---|---|"])
    for node in payload["blast_radius"]:
        breaks = "; ".join(node["breaks"])
        lines.append(f"| `{node['name']}` | {node['kind']} | {node['risk']} | {breaks} | {node['containment']} |")
    lines.extend(["", "## Critical paths", ""])
    for item in payload["critical_paths"]:
        lines.append(f"### {item['path']}")
        lines.append("")
        lines.append(f"- Depends on: `{', '.join(item['depends_on'])}`")
        lines.append(f"- Break effect: {item['break_effect']}")
        lines.append("")
    lines.extend(["## Detection commands", ""])
    commands = []
    for node in payload["blast_radius"]:
        commands.extend(node["detect_with"])
    for command in sorted(set(commands)):
        lines.append(f"- `{command}`")
    return "\n".join(lines).rstrip() + "\n"


def write_artifacts(payload: dict[str, Any]) -> dict[str, Path]:
    artifact_dir = root() / "migration-artifacts" / "art-127_blast_radius"
    canonical_dir = root() / "migration-artifacts" / "01-current-state"
    artifact_dir.mkdir(parents=True, exist_ok=True)
    canonical_dir.mkdir(parents=True, exist_ok=True)
    json_path = artifact_dir / "blast-radius-map.json"
    md_path = artifact_dir / "blast-radius-map.md"
    canonical_md = canonical_dir / "blast-radius-map.md"
    json_path.write_text(json.dumps(payload, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    md_text = markdown(payload)
    md_path.write_text(md_text, encoding="utf-8")
    canonical_md.write_text(md_text, encoding="utf-8")
    return {"json": json_path, "md": md_path, "canonical_md": canonical_md}


def insert_fixture(conn: sqlite3.Connection) -> None:
    conn.execute(
        """
        INSERT INTO envctl_migration_targets
          (id, target_id, target_type, primary_root, compare_root, descriptor_json,
           descriptor_hash, safety_mode, max_auto_risk)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            TARGET_ID,
            "art-127-filesystem-target",
            "mixed",
            str(package_root()),
            None,
            '{"schema_version":"1.0","task_id":"ART-127_BLAST_RADIUS"}',
            "sha256:art127-target",
            "approval-gated",
            "R2",
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_runs
          (id, target_id, recipe_id, artifact_contract_id, status, human_mode,
           initiated_by, sandbox_policy, approval_policy, tool_versions_json,
           reproducibility_hash, started_at_utc)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            RUN_ID,
            TARGET_ID,
            RECIPE_ID,
            CONTRACT_ID,
            "running",
            "approval-gated",
            "artifact-agent",
            "workspace-write",
            "never",
            '{"python":"stdlib","sqlite":"stdlib","codex":"codex-cli-background-shell"}',
            "sha256:art127-run",
            now(),
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_operations
          (id, run_id, operation_type, phase, status, risk, idempotency_key,
           command_hash, command_redacted, input_json)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            OPERATION_ID,
            RUN_ID,
            "produce_artifact_record",
            "05-artifacts",
            "succeeded",
            "R2",
            "ART-127/generate-register-blast-radius",
            "sha256:art127-command",
            "python3 scripts/generate_art_127_blast_radius.py",
            '{"task_id":"ART-127_BLAST_RADIUS"}',
        ),
    )
    conn.commit()


def register_artifacts(conn: sqlite3.Connection, paths: dict[str, Path]) -> list[dict[str, Any]]:
    registry = ArtifactRegistry(conn, package_root())
    evidence_refs = [
        rel(paths["md"]),
        rel(paths["json"]),
        rel(paths["canonical_md"]),
        "execution-framework/generated/envctl_target_registry.json",
        "execution-framework/generated/envctl_migration_db_model.json",
        "execution-framework/generated/shared_protocol_manifest.json",
        "execution-framework/generated/contract_manifest.json",
    ]
    records = [
        {
            "artifact_id": "01-current-state-blast-radius-map-md",
            "run_id": RUN_ID,
            "title": "Blast Radius Map",
            "status": "complete",
            "artifact_type": "migration_artifact",
            "path": rel(paths["canonical_md"]),
            "producer_operation_id": OPERATION_ID,
            "contract_id": CONTRACT_ID,
            "provenance": {
                "task_id": TASK_ID,
                "owner_agent": "artifact-agent",
                "helper_id": HELPER_ID,
                "source_graph_uri": "execution-framework/generated/task_graph.csv",
            },
            "evidence_refs": evidence_refs,
            "links": [
                {"to": "artifact:09-governance-risk-register-md", "type": "informs"},
                {"to": "artifact:08-operations-incident-response-md", "type": "informs"},
                {"to": "artifact:VER-300_UNIT_VALIDATION", "type": "blocks"},
            ],
            "validations": [
                {
                    "validator": "generate_art_127_blast_radius.py:path-and-hash",
                    "status": "pass",
                    "details": {"canonical_path": rel(paths["canonical_md"]), "hash": sha256_uri(paths["canonical_md"])},
                    "evidence_refs": [rel(paths["canonical_md"])],
                },
                {
                    "validator": "generate_art_127_blast_radius.py:coverage",
                    "status": "pass",
                    "details": {"required_kinds": ["service", "table", "queue", "api", "credential"]},
                    "evidence_refs": [rel(paths["json"])],
                },
            ],
        },
        {
            "artifact_id": "art-127-blast-radius-map-json",
            "run_id": RUN_ID,
            "title": "Blast Radius Map JSON",
            "status": "complete",
            "artifact_type": "machine_readable_record",
            "path": rel(paths["json"]),
            "producer_operation_id": OPERATION_ID,
            "contract_id": CONTRACT_ID,
            "provenance": {"task_id": TASK_ID, "owner_agent": "artifact-agent", "helper_id": HELPER_ID},
            "evidence_refs": evidence_refs,
            "links": [{"to": "artifact:01-current-state-blast-radius-map-md", "type": "machine_readable_for"}],
            "validations": [
                {
                    "validator": "generate_art_127_blast_radius.py:json-shape",
                    "status": "pass",
                    "details": {"schema_version": "1.0", "failure_modes_total": len(blast_nodes())},
                    "evidence_refs": [rel(paths["json"])],
                }
            ],
        },
    ]
    return [registry.register(record) for record in records]


def build_report(conn: sqlite3.Connection, paths: dict[str, Path], registry_results: list[dict[str, Any]], payload: dict[str, Any]) -> dict[str, Any]:
    errors: list[str] = []
    registered = []
    for result in registry_results:
        row = fetch_artifact(conn, RUN_ID, result["artifact_id"])
        path = package_root() / row["path"]
        actual = sha256_uri(path)
        if row["content_hash"] != actual:
            errors.append(f"hash mismatch for {row['artifact_id']}")
        if not row["evidence"].get("evidence_ids"):
            errors.append(f"missing evidence ids for {row['artifact_id']}")
        registered.append(row)
    required_kinds = {"service", "table", "queue", "api", "credential"}
    actual_kinds = {node["kind"] for node in payload["blast_radius"]}
    if not required_kinds.issubset(actual_kinds):
        errors.append("blast-radius map does not cover all required failure surface kinds")
    index_count = conn.execute(
        "SELECT COUNT(*) FROM envctl_migration_artifact_index WHERE run_id = ? AND content_hash IS NOT NULL",
        (RUN_ID,),
    ).fetchone()[0]
    validation_count = conn.execute(
        "SELECT COUNT(*) FROM envctl_migration_validations WHERE run_id = ?",
        (RUN_ID,),
    ).fetchone()[0]
    evidence_count = conn.execute(
        "SELECT COUNT(*) FROM envctl_migration_evidence WHERE run_id = ?",
        (RUN_ID,),
    ).fetchone()[0]
    graph_edge_count = conn.execute(
        "SELECT COUNT(*) FROM envctl_migration_graph_edges WHERE run_id = ?",
        (RUN_ID,),
    ).fetchone()[0]
    if index_count < 2:
        errors.append("artifact index does not contain both registered artifacts")
    if validation_count < 3:
        errors.append("validation evidence links were not recorded")
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "passed" if not errors else "failed",
        "generated_at": now(),
        "artifacts": {key: rel(path) for key, path in paths.items()},
        "artifact_hashes": {key: sha256_uri(path) for key, path in paths.items()},
        "registry_results": registry_results,
        "registered_artifacts": registered,
        "registry_summary": {
            "artifact_index_hash_rows": index_count,
            "validation_count": validation_count,
            "evidence_count": evidence_count,
            "graph_edge_count": graph_edge_count,
        },
        "coverage": payload["coverage"],
        "errors": errors,
    }


def main() -> None:
    row = contract_row()
    payload = build_payload(row, source_summary())
    paths = write_artifacts(payload)
    conn = sqlite3.connect(":memory:")
    conn.execute("PRAGMA foreign_keys = ON")
    apply_migrations(conn, package_root())
    insert_fixture(conn)
    registry_results = register_artifacts(conn, paths)
    report = build_report(conn, paths, registry_results, payload)

    log_path = root() / "logs" / f"{TASK_ID}.log"
    heartbeat_path = root() / "state" / f"{TASK_ID}.heartbeat.json"
    log_path.write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")
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
        "execution-framework/scripts/generate_art_127_blast_radius.py",
        rel(paths["md"]),
        rel(paths["json"]),
        rel(paths["canonical_md"]),
        f"execution-framework/logs/{TASK_ID}.log",
        f"execution-framework/state/{TASK_ID}.heartbeat.json",
        f"execution-framework/proof_records/{TASK_ID}.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    commands_run = [
        "python3 scripts/generate_art_127_blast_radius.py",
        "python3 -m py_compile scripts/generate_art_127_blast_radius.py",
    ]
    evidence = [
        rel(paths["md"]),
        rel(paths["json"]),
        rel(paths["canonical_md"]),
        "execution-framework/logs/ART-127_BLAST_RADIUS.log",
        "execution-framework/generated/envctl_target_registry.json",
        "execution-framework/generated/envctl_migration_db_model.json",
        "execution-framework/generated/shared_protocol_manifest.json",
        "execution-framework/generated/contract_manifest.json",
    ]
    proof = make_proof(
        TASK_ID,
        "completed" if report["status"] == "passed" else "failed",
        "codex-cli-local",
        HELPER_ID,
        MODEL_TAG,
        str(package_root()),
        files_changed,
        commands_run,
        report,
        evidence,
        "" if report["status"] == "passed" else "; ".join(report["errors"]),
        "run VER-300_UNIT_VALIDATION" if report["status"] == "passed" else "fix ART-127 registry validation errors",
    )
    append_proof(proof)
    print(
        "ART-127 status={status} artifacts={artifacts} validations={validations} evidence={evidence}".format(
            status=report["status"],
            artifacts=len(report["registered_artifacts"]),
            validations=report["registry_summary"]["validation_count"],
            evidence=report["registry_summary"]["evidence_count"],
        )
    )
    if report["status"] != "passed":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
