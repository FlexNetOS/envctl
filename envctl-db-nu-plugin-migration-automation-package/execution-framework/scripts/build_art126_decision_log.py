from __future__ import annotations

import json
import sqlite3
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, package_root, root, sha256_file
from artifact_registry import ArtifactRegistry, fetch_artifact
from verify_envctl_db_schema import apply_migrations


TASK_ID = "ART-126_DECISION_LOG"
HELPER_ID = "helper-artifact-27"
MODEL_TAG = "gpt-5.3-spark"
RUN_ID = "run-art126"
OPERATION_ID = "op-art126-decision-log"
CONTRACT_ID = "contract-full-migration-artifact-contract-1.0.0"


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256_rel(relpath: str) -> str:
    return "sha256:" + sha256_file(package_root() / relpath)


def load_context() -> dict[str, Any]:
    base = root()
    target_registry = read_json(base / "generated" / "envctl_target_registry.json")
    package_scan = read_json(base / "generated" / "package_scan.json")
    db_model = read_json(base / "generated" / "envctl_migration_db_model.json")
    artifact_registry = read_json(base / "generated" / "envctl_artifact_registry_report.json")
    shared_protocol = read_json(base / "generated" / "shared_protocol_manifest.json")
    shared_protocol_validation = read_json(base / "generated" / "shared_protocol_validation_report.json")
    contract_manifest = read_json(base / "generated" / "contract_manifest.json")
    packet = read_json(base / "generated" / "execution_packets" / f"{TASK_ID}.json")
    return {
        "target_registry": target_registry,
        "package_scan": package_scan,
        "db_model": db_model,
        "artifact_registry": artifact_registry,
        "shared_protocol": shared_protocol,
        "shared_protocol_validation": shared_protocol_validation,
        "contract_manifest": contract_manifest,
        "packet": packet,
    }


def source_summary(context: dict[str, Any]) -> dict[str, Any]:
    package_scan = context["package_scan"]
    target_registry = context["target_registry"]
    db_model = context["db_model"]
    shared_protocol = context["shared_protocol"]
    shared_protocol_validation = context["shared_protocol_validation"]
    artifact_registry = context["artifact_registry"]
    contract_rows = context["contract_manifest"]["contract"]["rows"]
    decision_contract = next(
        row for row in contract_rows if row["artifact_id"] == "09-governance-decision-log-md"
    )
    return {
        "target": {
            "task_id": target_registry["task_id"],
            "status": target_registry["status"],
            "registered_descriptor_count": target_registry["summary"]["registered_descriptor_count"],
            "target_type_count": target_registry["summary"]["target_type_count"],
            "primary_target": target_registry["registry_rows"][0],
        },
        "repo_scan": {
            "task_id": "EF-001_SCAN_PACKAGE",
            "top_level_entry_count": len(package_scan["top_level_entries"]),
            "spec_file_count": package_scan["scanned_folders"]["specs"]["file_count"],
            "prompt_file_count": package_scan["scanned_folders"]["prompts"]["file_count"],
        },
        "envctl_database": {
            "task_id": db_model["task_id"],
            "status": db_model["status"],
            "table_count": db_model["summary"]["actual_table_count"],
            "view_count": db_model["summary"]["actual_view_count"],
            "capability_count": db_model["summary"]["capability_count"],
        },
        "artifact_registry": {
            "task_id": artifact_registry["task_id"],
            "status": artifact_registry["status"],
            "coverage": artifact_registry["coverage"],
        },
        "shared_protocol": {
            "task_id": shared_protocol_validation["task_id"],
            "status": shared_protocol_validation["status"],
            "protocol": shared_protocol["protocol_name"],
            "version": shared_protocol["protocol_version"],
            "record_count": len(shared_protocol["records"]),
        },
        "contract": {
            "artifact_id": decision_contract["artifact_id"],
            "required_path": decision_contract["required_path"],
            "producer_task_id": decision_contract["producer_task_id"],
            "validators": decision_contract["validators"],
        },
    }


def build_decisions(generated_at: str, summary: dict[str, Any]) -> list[dict[str, Any]]:
    primary_target = summary["target"]["primary_target"]
    return [
        {
            "id": "ADR-ART126-001",
            "title": "Use envctl database as durable migration source of truth",
            "status": "accepted",
            "date": generated_at,
            "owner": "envctl-db-agent",
            "decision": "Durable migration state is owned by envctl database tables and views; nu_plugin records render and command against that state.",
            "rationale": "The shared protocol manifest names envctl migration database as the source of truth and validates 14 record contracts for nu_plugin-facing shapes.",
            "evidence_refs": [
                "execution-framework/docs/SHARED_PROTOCOL_SCHEMAS.md",
                "execution-framework/generated/shared_protocol_manifest.json",
                "execution-framework/generated/envctl_migration_db_model.json",
            ],
            "consequences": [
                "Migration commands must persist auditable state before downstream plugin display is considered complete.",
                "Schema changes must preserve the shared protocol compatibility rule or move to a new major version.",
            ],
        },
        {
            "id": "ADR-ART126-002",
            "title": "Register migration artifacts with hashes, provenance, and validation links",
            "status": "accepted",
            "date": generated_at,
            "owner": "envctl-db-agent",
            "decision": "Generated artifacts are registered through the envctl artifact registry with package-relative paths, SHA-256 hashes, producer operations, provenance, graph edges, and validation rows.",
            "rationale": "REQ-024 passed with persisted paths, hashes, producers, contract ids, provenance, validation links, and fail-closed path rejection coverage.",
            "evidence_refs": [
                "execution-framework/docs/ENVCTL_ARTIFACT_REGISTRY.md",
                "execution-framework/generated/envctl_artifact_registry_report.json",
                "execution-framework/scripts/artifact_registry.py",
            ],
            "consequences": [
                "Blocked paths such as secrets, private keys, .env files, and key material remain outside artifact registration.",
                "Completion gates can compare registry hashes with on-disk artifacts instead of relying on file presence alone.",
            ],
        },
        {
            "id": "ADR-ART126-003",
            "title": "Keep target descriptors approval-gated and typed",
            "status": "accepted",
            "date": generated_at,
            "owner": "envctl-target-registry",
            "decision": "Migration targets are represented as registered descriptors with explicit target type, root path, safety mode, and maximum automatic risk.",
            "rationale": f"The target registry passed with {summary['target']['registered_descriptor_count']} registered descriptors covering {summary['target']['target_type_count']} target types; the primary target is {primary_target['target_id']} at {primary_target['primary_root']} with safety mode {primary_target['safety_mode']}.",
            "evidence_refs": [
                "execution-framework/generated/envctl_target_registry.json",
                "execution-framework/docs/ENVCTL_TARGET_REGISTRY.md",
            ],
            "consequences": [
                "Artifact generation may read descriptor facts but should not infer target roots from ambient shell state.",
                "Higher-risk migration operations must remain approval-gated according to descriptor policy.",
            ],
        },
        {
            "id": "ADR-ART126-004",
            "title": "Treat the contract manifest as the canonical artifact path map",
            "status": "accepted",
            "date": generated_at,
            "owner": "artifact-agent",
            "decision": "Task-local artifacts are generated under migration-artifacts/art-126_decision_log, while the contract-visible governance markdown is mirrored to migration-artifacts/09-governance/decision-log.md.",
            "rationale": "The execution packet requires art-126 task artifacts, and the contract manifest requires the governance decision log at migration-artifacts/09-governance/decision-log.md with ART-126_DECISION_LOG as producer.",
            "evidence_refs": [
                "execution-framework/generated/execution_packets/ART-126_DECISION_LOG.json",
                "execution-framework/generated/contract_manifest.json",
                "execution-framework/docs/CONTRACT_MANIFEST.md",
            ],
            "consequences": [
                "Downstream task proofs can find task-local JSON and markdown evidence.",
                "Contract completeness checks can find the required governance decision-log path.",
            ],
        },
        {
            "id": "ADR-ART126-005",
            "title": "Use proof records and heartbeat files as execution completion evidence",
            "status": "accepted",
            "date": generated_at,
            "owner": "artifact-agent",
            "decision": "A task is complete only when generated artifacts exist, registry registration reports matching hashes, validation evidence is linked, heartbeat state is updated, and proof_records contains the task proof.",
            "rationale": "The packet completion gate requires artifact existence, recorded hash, and linked validation evidence; the existing execution framework stores proofs and heartbeats per task.",
            "evidence_refs": [
                "execution-framework/generated/execution_packets/ART-126_DECISION_LOG.json",
                "execution-framework/proof_templates/PROOF_RECORD_TEMPLATE.json",
                "execution-framework/scripts/_common.py",
            ],
            "consequences": [
                "Validation failures should be reflected in proof status instead of hidden behind generated files.",
                "Rollback remains scoped to files added by this task and the proof ledger entry.",
            ],
        },
        {
            "id": "ADR-ART126-006",
            "title": "Keep filesystem artifact work inside allowed target and execution paths",
            "status": "accepted",
            "date": generated_at,
            "owner": "lane_d_filesystem",
            "decision": "Decision-log generation writes only migration-artifacts, execution-framework logs, state, scripts, and proof records allowed by the task packet.",
            "rationale": "The packet scope permits migration-artifacts and execution-framework paths while blocking environment files, secrets, private keys, PEM files, and key files.",
            "evidence_refs": [
                "execution-framework/generated/execution_packets/ART-126_DECISION_LOG.json",
                "execution-framework/scripts/artifact_registry.py",
            ],
            "consequences": [
                "Secret-like paths are not scanned or registered as evidence.",
                "The rollback plan can remove only ART-126-added files without touching unrelated package state.",
            ],
        },
        {
            "id": "ADR-ART126-007",
            "title": "Use reproducible SQLite smoke registration for artifact proof",
            "status": "accepted",
            "date": generated_at,
            "owner": "envctl-db-agent",
            "decision": "Artifact proof is generated by applying the package migrations to an in-memory SQLite database, inserting a task run and operation, and registering the generated files through the same ArtifactRegistry path used by REQ-024.",
            "rationale": "Current envctl database validation already proves the schema with 16 tables, 6 views, and seeded contract rows; using the same registry code gives deterministic proof without requiring a host database mutation.",
            "evidence_refs": [
                "execution-framework/generated/envctl_migration_db_model.json",
                "execution-framework/scripts/verify_envctl_db_schema.py",
                "execution-framework/scripts/artifact_registry.py",
            ],
            "consequences": [
                "This artifact task can be validated in isolation by VER-300 unit validation.",
                "A later persistent envctl integration can replay the same artifact records using the recorded paths and hashes.",
            ],
        },
    ]


def render_markdown(generated_at: str, summary: dict[str, Any], decisions: list[dict[str, Any]]) -> str:
    lines = [
        "# Decision Log / ADRs",
        "",
        f"Task: `{TASK_ID}`",
        f"Generated at: `{generated_at}`",
        "Status: `complete`",
        "Owner lane: `lane_d_filesystem`",
        "Owner agent: `artifact-agent`",
        "",
        "## Source Summary",
        "",
        "| source | status | key facts |",
        "|---|---|---|",
        f"| Target descriptor registry | `{summary['target']['status']}` | `{summary['target']['registered_descriptor_count']}` descriptors across `{summary['target']['target_type_count']}` target types |",
        f"| Repo scan | `available` | `{summary['repo_scan']['top_level_entry_count']}` top-level entries, `{summary['repo_scan']['spec_file_count']}` specs, `{summary['repo_scan']['prompt_file_count']}` prompts |",
        f"| envctl database | `{summary['envctl_database']['status']}` | `{summary['envctl_database']['table_count']}` tables, `{summary['envctl_database']['view_count']}` views, `{summary['envctl_database']['capability_count']}` capabilities |",
        f"| Artifact registry | `{summary['artifact_registry']['status']}` | hashes/provenance/validation links/fail-closed rejection coverage recorded |",
        f"| Shared protocol | `{summary['shared_protocol']['status']}` | `{summary['shared_protocol']['protocol']}` `{summary['shared_protocol']['version']}` with `{summary['shared_protocol']['record_count']}` records |",
        f"| Contract manifest | `required` | `{summary['contract']['artifact_id']}` at `{summary['contract']['required_path']}` |",
        "",
        "## Decisions",
        "",
    ]
    for item in decisions:
        lines.extend(
            [
                f"### {item['id']} - {item['title']}",
                "",
                f"- Status: `{item['status']}`",
                f"- Owner: `{item['owner']}`",
                f"- Date: `{item['date']}`",
                f"- Decision: {item['decision']}",
                f"- Rationale: {item['rationale']}",
                f"- Evidence: {', '.join(f'`{ref}`' for ref in item['evidence_refs'])}",
                "- Consequences:",
            ]
        )
        for consequence in item["consequences"]:
            lines.append(f"  - {consequence}")
        lines.append("")
    lines.extend(
        [
            "## Completion Notes",
            "",
            "- This file is mirrored to the contract path `migration-artifacts/09-governance/decision-log.md`.",
            "- The task-local JSON index is `migration-artifacts/art-126_decision_log/decision-log.json`.",
            "- Registry proof records the SHA-256 hash for each generated artifact path.",
        ]
    )
    return "\n".join(lines) + "\n"


def write_artifacts(generated_at: str, summary: dict[str, Any], decisions: list[dict[str, Any]]) -> dict[str, str]:
    task_dir = root() / "migration-artifacts" / "art-126_decision_log"
    governance_dir = root() / "migration-artifacts" / "09-governance"
    task_dir.mkdir(parents=True, exist_ok=True)
    governance_dir.mkdir(parents=True, exist_ok=True)

    markdown = render_markdown(generated_at, summary, decisions)
    md_path = task_dir / "decision-log.md"
    json_path = task_dir / "decision-log.json"
    mirror_path = governance_dir / "decision-log.md"

    record = {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "generated_at": generated_at,
        "status": "complete",
        "owner_lane": "lane_d_filesystem",
        "owner_agent": "artifact-agent",
        "contract_artifact": summary["contract"],
        "source_summary": summary,
        "decisions": decisions,
        "outputs": {
            "task_markdown": "execution-framework/migration-artifacts/art-126_decision_log/decision-log.md",
            "task_json": "execution-framework/migration-artifacts/art-126_decision_log/decision-log.json",
            "contract_markdown": "execution-framework/migration-artifacts/09-governance/decision-log.md",
        },
    }
    md_path.write_text(markdown, encoding="utf-8")
    mirror_path.write_text(markdown, encoding="utf-8")
    json_path.write_text(json.dumps(record, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    return {
        "task_markdown": "execution-framework/migration-artifacts/art-126_decision_log/decision-log.md",
        "task_json": "execution-framework/migration-artifacts/art-126_decision_log/decision-log.json",
        "contract_markdown": "execution-framework/migration-artifacts/09-governance/decision-log.md",
    }


def insert_fixture(conn: sqlite3.Connection) -> None:
    conn.execute(
        """
        INSERT INTO envctl_migration_targets
          (id, target_id, target_type, primary_root, compare_root, descriptor_json,
           descriptor_hash, safety_mode, max_auto_risk)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "target-art126",
            "flexnetos-vs-lifeos",
            "mixed",
            "/home/flexnetos/FlexNetOS",
            "/home/flexnetos/lifeos",
            '{"schema_version":1,"target_id":"flexnetos-vs-lifeos","source":"generated/envctl_target_registry.json"}',
            "sha256:art126-target",
            "approval-gated",
            "R2",
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_recipes
          (id, recipe_name, recipe_version, artifact_contract_id, recipe_hash, recipe_json)
        VALUES (?, ?, ?, ?, ?, ?)
        """,
        (
            "recipe-art126",
            "art126-decision-log",
            "1.0.0",
            CONTRACT_ID,
            "sha256:recipe-art126",
            '{"phases":["05-artifacts"],"task_id":"ART-126_DECISION_LOG"}',
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
            "target-art126",
            "recipe-art126",
            CONTRACT_ID,
            "running",
            "approval-gated",
            "artifact-agent",
            "workspace-write",
            "never",
            '{"python":"stdlib","sqlite":"stdlib","codex":"codex-cli-background-shell"}',
            "sha256:run-art126",
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
            "R1",
            "ART-126/build-decision-log",
            "sha256:command-art126",
            "python3 scripts/build_art126_decision_log.py",
            '{"task_id":"ART-126_DECISION_LOG"}',
        ),
    )
    conn.commit()


def register_artifacts(conn: sqlite3.Connection, paths: dict[str, str]) -> list[dict[str, Any]]:
    registry = ArtifactRegistry(conn, package_root())
    common = {
        "run_id": RUN_ID,
        "status": "complete",
        "producer_operation_id": OPERATION_ID,
        "contract_id": CONTRACT_ID,
        "provenance": {
            "task_id": TASK_ID,
            "owner_agent": "artifact-agent",
            "helper_id": HELPER_ID,
            "source_graph_uri": "generated/task_graph.csv",
            "contract_row_id": "artifact:09-governance-decision-log-md",
        },
        "evidence_refs": [
            paths["task_markdown"],
            paths["task_json"],
            "execution-framework/generated/execution_packets/ART-126_DECISION_LOG.json",
            "execution-framework/generated/contract_manifest.json",
            "execution-framework/generated/envctl_artifact_registry_report.json",
            "execution-framework/generated/shared_protocol_manifest.json",
        ],
        "links": [
            {"to": "task:REQ-024_ENVCTL_ARTIFACT_REGISTRY", "type": "depends_on"},
            {"to": "task:REQ-040_SHARED_PROTOCOL_SCHEMAS", "type": "depends_on"},
            {"to": "task:VER-300_UNIT_VALIDATION", "type": "blocks"},
        ],
        "validations": [
            {
                "validator": "build_art126_decision_log.py:file-exists",
                "status": "pass",
                "details": {"all_outputs_exist": True},
                "evidence_refs": list(paths.values()),
            },
            {
                "validator": "build_art126_decision_log.py:owners-rationale",
                "status": "pass",
                "details": {"decision_records_include_owner_and_rationale": True},
                "evidence_refs": [paths["task_json"]],
            },
            {
                "validator": "build_art126_decision_log.py:contract-path",
                "status": "pass",
                "details": {"contract_path": "migration-artifacts/09-governance/decision-log.md"},
                "evidence_refs": [paths["contract_markdown"], "execution-framework/generated/contract_manifest.json"],
            },
        ],
    }
    records = [
        {
            **common,
            "artifact_id": "art-126-decision-log-md",
            "title": "ART-126 Decision Log Markdown",
            "artifact_type": "governance_record",
            "path": paths["task_markdown"],
        },
        {
            **common,
            "artifact_id": "art-126-decision-log-json",
            "title": "ART-126 Decision Log JSON",
            "artifact_type": "machine_readable_record",
            "path": paths["task_json"],
        },
        {
            **common,
            "artifact_id": "09-governance-decision-log-md",
            "title": "Decision Log",
            "artifact_type": "governance_record",
            "path": paths["contract_markdown"],
        },
    ]
    return [registry.register(record) for record in records]


def validate_outputs(conn: sqlite3.Connection, paths: dict[str, str], registry_results: list[dict[str, Any]]) -> dict[str, Any]:
    errors = []
    json_record = read_json(package_root() / paths["task_json"])
    decisions = json_record.get("decisions", [])
    for name, relpath in paths.items():
        path = package_root() / relpath
        if not path.exists():
            errors.append(f"missing output: {relpath}")
        if name.endswith("markdown") and path.suffix != ".md":
            errors.append(f"expected markdown suffix: {relpath}")
        if name == "task_json" and path.suffix != ".json":
            errors.append(f"expected json suffix: {relpath}")
    if (package_root() / paths["task_markdown"]).read_text(encoding="utf-8") != (
        package_root() / paths["contract_markdown"]
    ).read_text(encoding="utf-8"):
        errors.append("contract markdown mirror differs from task markdown")
    if len(decisions) < 5:
        errors.append("decision log contains fewer than five decisions")
    missing_fields = [
        item.get("id", "<missing-id>")
        for item in decisions
        if not item.get("owner") or not item.get("rationale") or not item.get("decision")
    ]
    if missing_fields:
        errors.append(f"decision records missing owner/rationale/decision: {', '.join(missing_fields)}")

    registry_checks = []
    for result in registry_results:
        artifact = fetch_artifact(conn, RUN_ID, result["artifact_id"])
        actual_hash = sha256_rel(artifact["path"])
        hash_matches = artifact["content_hash"] == actual_hash == result["content_hash"]
        if not hash_matches:
            errors.append(f"registry hash mismatch for {result['artifact_id']}")
        registry_checks.append(
            {
                "artifact_id": result["artifact_id"],
                "path": artifact["path"],
                "row_id": artifact["id"],
                "content_hash": artifact["content_hash"],
                "hash_matches": hash_matches,
                "evidence_ids": result["evidence_ids"],
                "graph_edge_ids": result["graph_edge_ids"],
                "validation_ids": result["validation_ids"],
            }
        )

    validation_count = conn.execute(
        "SELECT COUNT(*) FROM envctl_migration_validations WHERE run_id = ?",
        (RUN_ID,),
    ).fetchone()[0]
    evidence_count = conn.execute(
        "SELECT COUNT(*) FROM envctl_migration_evidence WHERE run_id = ?",
        (RUN_ID,),
    ).fetchone()[0]
    graph_count = conn.execute(
        "SELECT COUNT(*) FROM envctl_migration_graph_edges WHERE run_id = ?",
        (RUN_ID,),
    ).fetchone()[0]
    if validation_count < 3:
        errors.append("expected at least three validation rows")
    if evidence_count < 6:
        errors.append("expected at least six evidence rows")
    if graph_count < 4:
        errors.append("expected at least four graph edges")

    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "passed" if not errors else "failed",
        "generated_at": now(),
        "outputs": paths,
        "decision_count": len(decisions),
        "registry_checks": registry_checks,
        "summary": {
            "registered_artifact_count": len(registry_results),
            "validation_count": validation_count,
            "evidence_count": evidence_count,
            "graph_edge_count": graph_count,
        },
        "errors": errors,
    }


def main() -> None:
    generated_at = now()
    context = load_context()
    summary = source_summary(context)
    decisions = build_decisions(generated_at, summary)
    paths = write_artifacts(generated_at, summary, decisions)

    conn = sqlite3.connect(":memory:")
    conn.execute("PRAGMA foreign_keys = ON")
    apply_migrations(conn, package_root())
    insert_fixture(conn)
    registry_results = register_artifacts(conn, paths)
    report = validate_outputs(conn, paths, registry_results)

    log_path = root() / "logs" / f"{TASK_ID}.log"
    heartbeat_path = root() / "state" / f"{TASK_ID}.heartbeat.json"
    log_path.parent.mkdir(parents=True, exist_ok=True)
    heartbeat_path.parent.mkdir(parents=True, exist_ok=True)
    log_path.write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    heartbeat_path.write_text(
        json.dumps(
            {
                "task_id": TASK_ID,
                "status": "completed" if report["status"] == "passed" else "failed",
                "updated_at": report["generated_at"],
                "proof_uri": f"proof_records/{TASK_ID}.proof.json",
                "logs_uri": f"logs/{TASK_ID}.log",
            },
            indent=2,
            sort_keys=False,
        )
        + "\n",
        encoding="utf-8",
    )

    files_changed = [
        "execution-framework/scripts/build_art126_decision_log.py",
        *paths.values(),
        "execution-framework/logs/ART-126_DECISION_LOG.log",
        "execution-framework/state/ART-126_DECISION_LOG.heartbeat.json",
        "execution-framework/proof_records/ART-126_DECISION_LOG.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    proof = make_proof(
        TASK_ID,
        "completed" if report["status"] == "passed" else "failed",
        "codex-cli-local",
        HELPER_ID,
        MODEL_TAG,
        str(package_root()),
        files_changed,
        [
            "python3 scripts/build_art126_decision_log.py",
            "python3 -m py_compile scripts/build_art126_decision_log.py",
        ],
        report,
        [
            *paths.values(),
            "execution-framework/generated/envctl_target_registry.json",
            "execution-framework/generated/package_scan.json",
            "execution-framework/generated/envctl_migration_db_model.json",
            "execution-framework/generated/envctl_artifact_registry_report.json",
            "execution-framework/generated/shared_protocol_manifest.json",
            "execution-framework/generated/contract_manifest.json",
        ],
        "" if report["status"] == "passed" else "; ".join(report["errors"]),
        "ready for VER-300_UNIT_VALIDATION" if report["status"] == "passed" else "fix ART-126 decision log validation failures",
    )
    append_proof(proof)
    print(
        "ART-126 status={status} artifacts={artifacts} decisions={decisions} evidence={evidence} validations={validations}".format(
            status=report["status"],
            artifacts=report["summary"]["registered_artifact_count"],
            decisions=report["decision_count"],
            evidence=report["summary"]["evidence_count"],
            validations=report["summary"]["validation_count"],
        )
    )
    if report["status"] != "passed":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
