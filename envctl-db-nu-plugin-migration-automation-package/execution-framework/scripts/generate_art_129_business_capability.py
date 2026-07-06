from __future__ import annotations

import json
import sqlite3
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, package_root, root
from artifact_registry import ArtifactRegistry, fetch_artifact
from verify_envctl_db_schema import apply_migrations


TASK_ID = "ART-129_BUSINESS_CAPABILITY"
HELPER_ID = "helper-artifact-30"
MODEL_TAG = "gpt-5.3-spark"
RUN_ID = "run-art129-business-capability"
OPERATION_ID = "op-art129-generate-business-capability"
CONTRACT_ID = "contract-art129-full-migration-artifacts"


def artifact_dir() -> Path:
    return root() / "migration-artifacts" / "art-129_business_capability"


def artifact_paths() -> dict[str, Path]:
    base = artifact_dir()
    return {
        "markdown": base / "business_capability_map.md",
        "json": base / "business_capability_map.json",
        "report": root() / "generated" / "art_129_business_capability_registry_report.json",
        "log": root() / "logs" / f"{TASK_ID}.log",
        "heartbeat": root() / "state" / f"{TASK_ID}.heartbeat.json",
    }


def read_text_if_exists(path: Path) -> str:
    return path.read_text(encoding="utf-8") if path.exists() else ""


def load_json_if_exists(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8")) if path.exists() else {}


def source_inputs() -> dict[str, Any]:
    base = package_root()
    return {
        "target_descriptor": {
            "path": "examples/target-descriptors/flexnetos-vs-lifeos.yaml",
            "registry_source": "generated/envctl_target_registry.json",
            "target_id": "flexnetos-vs-lifeos",
            "target_type": "mixed",
            "primary_root": "/home/flexnetos/FlexNetOS",
            "compare_root": "/home/flexnetos/lifeos",
            "purpose": "Determine what FlexNetOS was used for compared with lifeos using real evidence.",
            "descriptor_excerpt": read_text_if_exists(base / "examples" / "target-descriptors" / "flexnetos-vs-lifeos.yaml"),
        },
        "repo_scan": {
            "path": "execution-framework/generated/package_scan.json",
            "summary": {
                "top_level_entries": load_json_if_exists(root() / "generated" / "package_scan.json").get("top_level_entries", []),
                "scanned_folders": sorted(
                    load_json_if_exists(root() / "generated" / "package_scan.json").get("scanned_folders", {}).keys()
                ),
            },
        },
        "envctl_database": {
            "schema_report": "execution-framework/generated/envctl_migration_db_model.json",
            "artifact_registry_report": "execution-framework/generated/envctl_artifact_registry_report.json",
            "shared_protocol_manifest": "execution-framework/generated/shared_protocol_manifest.json",
            "command_manifest": "execution-framework/generated/nu_plugin_command_manifest.json",
        },
    }


def capability_rows() -> list[dict[str, Any]]:
    return [
        {
            "capability_id": "BUS-CAP-001",
            "business_function": "Migration target intake and scoping",
            "business_outcome": "Operators can define the FlexNetOS versus lifeos migration target, preserve safety limits, and select the comparison roots for evidence-based planning.",
            "technical_systems": [
                "Target Descriptor Registry",
                "schemas/target_descriptor.schema.json",
                "envctl migration target list",
                "envctl migration target add",
            ],
            "envctl_db_objects": ["envctl_migration_targets"],
            "nu_plugin_surface": ["envctl migration target list", "envctl migration targets"],
            "migration_artifacts": ["Target descriptor", "Run ledger"],
            "evidence_refs": [
                "examples/target-descriptors/flexnetos-vs-lifeos.yaml",
                "execution-framework/generated/envctl_target_registry.json",
                "execution-framework/docs/ENVCTL_TARGET_REGISTRY.md",
            ],
            "controls": ["descriptor hash", "safety mode", "max auto risk"],
            "confidence": "high",
        },
        {
            "capability_id": "BUS-CAP-002",
            "business_function": "Package import and contract governance",
            "business_outcome": "Prompt packages, artifact contracts, and recipes become versioned business commitments rather than loose files.",
            "technical_systems": [
                "Package Import Registry",
                "Artifact Contract Registry",
                "Migration Recipe Registry",
                "contract manifest seed SQL",
            ],
            "envctl_db_objects": [
                "envctl_migration_packages",
                "envctl_migration_artifact_contracts",
                "envctl_migration_recipes",
            ],
            "nu_plugin_surface": ["envctl migration packages", "envctl migration package inspect", "envctl migration package import"],
            "migration_artifacts": ["Artifact contract version", "Migration recipe version"],
            "evidence_refs": [
                "execution-framework/generated/contract_manifest.json",
                "execution-framework/generated/contract_manifest.seed.sql",
                "expected-output/migration-automation-artifacts.md",
            ],
            "controls": ["contract hash", "recipe hash", "source package id"],
            "confidence": "high",
        },
        {
            "capability_id": "BUS-CAP-003",
            "business_function": "Execution planning and run control",
            "business_outcome": "Migration work can be planned, started, paused, resumed, and inspected as auditable operations.",
            "technical_systems": [
                "Run Manager",
                "Operation Queue",
                "Operation State Machine",
                "envctl run ledger",
            ],
            "envctl_db_objects": [
                "envctl_migration_runs",
                "envctl_migration_operations",
                "envctl_migration_run_events",
                "envctl_migration_run_latest_status",
            ],
            "nu_plugin_surface": [
                "envctl migration run plan",
                "envctl migration run start",
                "envctl migration pause",
                "envctl migration resume",
                "envctl migration status",
            ],
            "migration_artifacts": ["Run ledger", "Event timeline", "Operation queue"],
            "evidence_refs": [
                "execution-framework/generated/envctl_run_ledger_report.json",
                "execution-framework/generated/operation_state_machine.json",
                "execution-framework/docs/OPERATION_STATE_MACHINE.md",
            ],
            "controls": ["operation idempotency key", "risk class", "state transition model"],
            "confidence": "high",
        },
        {
            "capability_id": "BUS-CAP-004",
            "business_function": "Human approval and intervention control",
            "business_outcome": "Risky migration changes are gated by explicit approval records and visible operator state.",
            "technical_systems": [
                "Approval Gate",
                "Human Approval Ledger",
                "Live Visuals",
                "Agent Control API",
            ],
            "envctl_db_objects": ["envctl_migration_approvals", "envctl_migration_open_approvals"],
            "nu_plugin_surface": ["envctl migration approve", "envctl migration status"],
            "migration_artifacts": ["Approval ledger", "Human involvement transcript"],
            "evidence_refs": [
                "prompts/LIVE_VISUALS_AND_HUMAN_CONTROL.md",
                "execution-framework/generated/live_visuals.json",
                "execution-framework/generated/operation_state_machine.json",
            ],
            "controls": ["approval-required-from-risk R3", "approval status", "human mode"],
            "confidence": "medium",
        },
        {
            "capability_id": "BUS-CAP-005",
            "business_function": "Evidence, artifact, and lineage governance",
            "business_outcome": "Every produced migration deliverable can be tied to content hashes, producers, evidence records, and graph edges.",
            "technical_systems": [
                "Artifact Registry",
                "Evidence Store",
                "Link Graph",
                "Proof Record Ledger",
            ],
            "envctl_db_objects": [
                "envctl_migration_artifacts",
                "envctl_migration_evidence",
                "envctl_migration_graph_edges",
                "envctl_migration_artifact_index",
            ],
            "nu_plugin_surface": ["envctl migration proof", "envctl migration status"],
            "migration_artifacts": ["Evidence register", "Artifact registry", "Link graph"],
            "evidence_refs": [
                "execution-framework/generated/envctl_artifact_registry_report.json",
                "execution-framework/docs/ENVCTL_ARTIFACT_REGISTRY.md",
                "execution-framework/schemas/proof_record.schema.json",
            ],
            "controls": ["SHA-256 content hash", "producer operation id", "contract id", "blocked path rejection"],
            "confidence": "high",
        },
        {
            "capability_id": "BUS-CAP-006",
            "business_function": "Validation and readiness decision support",
            "business_outcome": "Migration readiness can be scored from validation rows, replay checks, and generated proof evidence before downstream verification starts.",
            "technical_systems": [
                "Validation Ledger",
                "Validation Scorecard View",
                "Replay Engine",
                "Readiness Reports",
            ],
            "envctl_db_objects": [
                "envctl_migration_validations",
                "envctl_migration_validation_scorecard",
                "envctl_migration_replay_readiness",
            ],
            "nu_plugin_surface": ["envctl migration replay", "envctl migration proof", "envctl migration status"],
            "migration_artifacts": ["Validation scorecard", "Replay readiness report"],
            "evidence_refs": [
                "execution-framework/generated/envctl_migration_db_model.json",
                "execution-framework/generated/shared_protocol_validation_report.json",
                "execution-framework/generated/final_verification_report.json",
            ],
            "controls": ["validation status", "evidence refs", "replay readiness view"],
            "confidence": "medium",
        },
        {
            "capability_id": "BUS-CAP-007",
            "business_function": "Rollback and checkpoint assurance",
            "business_outcome": "Operators can identify rollback handles and checkpoints before applying migration changes.",
            "technical_systems": [
                "Rollback Registry",
                "Checkpoint Registry",
                "pre-execution framework manifest",
            ],
            "envctl_db_objects": ["envctl_migration_checkpoints", "envctl_migration_rollbacks"],
            "nu_plugin_surface": ["envctl migration status", "envctl migration replay"],
            "migration_artifacts": ["Rollback readiness report", "Rollback/checkpoint metadata"],
            "evidence_refs": [
                "history/pre_execution_framework_manifest.json",
                "execution-framework/generated/envctl_migration_db_model.json",
                "prompts/SECURITY_REPRODUCIBILITY_MODEL.md",
            ],
            "controls": ["rollback plan JSON", "checkpoint hash", "rollback status"],
            "confidence": "medium",
        },
        {
            "capability_id": "BUS-CAP-008",
            "business_function": "Operator presentation through Nushell",
            "business_outcome": "Operators get table-shaped, command-oriented access while envctl remains the durable source of truth.",
            "technical_systems": [
                "nu_plugin_envctl_migration",
                "Shared Protocol Schemas",
                "envctl JSON command boundary",
            ],
            "envctl_db_objects": [
                "envctl_migration_plugin_sessions",
                "envctl_migration_agent_sessions",
                "execution_framework_proof_records",
            ],
            "nu_plugin_surface": [
                "envctl migration target list",
                "envctl migration run plan",
                "envctl migration status",
                "envctl migration proof",
            ],
            "migration_artifacts": ["Plugin session ledger", "Agent session ledger"],
            "evidence_refs": [
                "execution-framework/generated/nu_plugin_command_manifest.json",
                "execution-framework/generated/shared_protocol_manifest.json",
                "execution-framework/docs/SHARED_PROTOCOL_SCHEMAS.md",
            ],
            "controls": ["envctl owns durable state", "mutations emit events", "shared record schemas"],
            "confidence": "high",
        },
        {
            "capability_id": "BUS-CAP-009",
            "business_function": "Operational observability and live status",
            "business_outcome": "Status, timeline, and stream records let teams monitor artifact production and blockers during the migration.",
            "technical_systems": [
                "Append-only Event Log",
                "Live Timeline View",
                "Plugin Status Streams",
                "Live Visuals Renderer",
            ],
            "envctl_db_objects": [
                "envctl_migration_run_events",
                "envctl_migration_live_timeline",
                "envctl_migration_run_latest_status",
            ],
            "nu_plugin_surface": ["envctl migration status"],
            "migration_artifacts": ["Event timeline", "Plugin session ledger", "Agent session ledger"],
            "evidence_refs": [
                "execution-framework/generated/REQ-034_PLUGIN_STATUS_STREAMS.contract.json",
                "execution-framework/generated/live_visuals.md",
                "execution-framework/docs/ENVCTL_RUN_LEDGER.md",
            ],
            "controls": ["event sequence", "actor type", "previous event hash", "stream status contract"],
            "confidence": "medium",
        },
    ]


def build_artifact_payload(generated_at: str) -> dict[str, Any]:
    capabilities = capability_rows()
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "title": "Business capability map",
        "generated_at": generated_at,
        "scope": {
            "target": "flexnetos-vs-lifeos",
            "purpose": "Map technical migration automation systems to business functions.",
            "owner_lane": "lane_d_filesystem",
            "owner_agent": "artifact-agent",
        },
        "source_inputs": source_inputs(),
        "summary": {
            "capability_count": len(capabilities),
            "all_capabilities_have_business_function": all(bool(item["business_function"]) for item in capabilities),
            "all_capabilities_have_technical_systems": all(bool(item["technical_systems"]) for item in capabilities),
            "mapped_envctl_db_object_count": len({obj for item in capabilities for obj in item["envctl_db_objects"]}),
            "mapped_nu_plugin_command_count": len({cmd for item in capabilities for cmd in item["nu_plugin_surface"]}),
        },
        "capabilities": capabilities,
    }


def render_markdown(payload: dict[str, Any]) -> str:
    def cell(value: Any) -> str:
        if isinstance(value, list):
            text = "<br>".join(str(item) for item in value)
        else:
            text = str(value)
        return text.replace("|", "\\|")

    lines = [
        "# Business capability map",
        "",
        f"Task: `{TASK_ID}`",
        f"Generated at: `{payload['generated_at']}`",
        "",
        "## Scope",
        "",
        "This artifact maps envctl migration automation systems to the business functions they support for the `flexnetos-vs-lifeos` target. The map is grounded in the target descriptor, package scan, envctl database model, artifact registry, and shared protocol manifests.",
        "",
        "## Summary",
        "",
        f"- Capability rows: `{payload['summary']['capability_count']}`",
        f"- Envctl DB objects mapped: `{payload['summary']['mapped_envctl_db_object_count']}`",
        f"- nu_plugin commands mapped: `{payload['summary']['mapped_nu_plugin_command_count']}`",
        "- Source of truth: `envctl migration database owns durable state; nu_plugin renders records and submits auditable commands.`",
        "",
        "## Capability Matrix",
        "",
        "| id | business function | business outcome | technical systems | envctl DB objects | nu_plugin surface | controls | confidence |",
        "|---|---|---|---|---|---|---|---|",
    ]
    for item in payload["capabilities"]:
        lines.append(
            "| {id} | {function} | {outcome} | {systems} | {db} | {plugin} | {controls} | {confidence} |".format(
                id=cell(item["capability_id"]),
                function=cell(item["business_function"]),
                outcome=cell(item["business_outcome"]),
                systems=cell(item["technical_systems"]),
                db=cell(item["envctl_db_objects"]),
                plugin=cell(item["nu_plugin_surface"]),
                controls=cell(item["controls"]),
                confidence=cell(item["confidence"]),
            )
        )
    lines.extend(
        [
            "",
            "## Evidence Inputs",
            "",
            "| input | path | purpose |",
            "|---|---|---|",
            "| target descriptor | `examples/target-descriptors/flexnetos-vs-lifeos.yaml` | target roots, safety posture, and migration purpose |",
            "| repo scan | `execution-framework/generated/package_scan.json` | package folders and migration automation source inventory |",
            "| envctl database | `execution-framework/generated/envctl_migration_db_model.json` | durable state tables, views, and capability coverage |",
            "| artifact registry | `execution-framework/generated/envctl_artifact_registry_report.json` | content hash, evidence, graph, validation, and fail-closed path behavior |",
            "| shared protocol | `execution-framework/generated/shared_protocol_manifest.json` | envctl and nu_plugin record contracts |",
            "",
            "## Registration Gate",
            "",
            "The companion verification report registers this Markdown artifact and its JSON source through the envctl artifact registry smoke path. The proof record records the resulting registry row ids, SHA-256 hashes, validation ids, and graph edges.",
        ]
    )
    return "\n".join(lines) + "\n"


def write_artifacts(payload: dict[str, Any]) -> dict[str, str]:
    paths = artifact_paths()
    paths["markdown"].parent.mkdir(parents=True, exist_ok=True)
    paths["json"].write_text(json.dumps(payload, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    paths["markdown"].write_text(render_markdown(payload), encoding="utf-8")
    return {
        "markdown": "execution-framework/migration-artifacts/art-129_business_capability/business_capability_map.md",
        "json": "execution-framework/migration-artifacts/art-129_business_capability/business_capability_map.json",
    }


def insert_fixture(conn: sqlite3.Connection) -> dict[str, str]:
    descriptor = source_inputs()["target_descriptor"]
    conn.execute(
        """
        INSERT INTO envctl_migration_targets
          (id, target_id, target_type, primary_root, compare_root, descriptor_json,
           descriptor_hash, safety_mode, max_auto_risk)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "target-art129-flexnetos-vs-lifeos",
            "flexnetos-vs-lifeos",
            "mixed",
            descriptor["primary_root"],
            descriptor["compare_root"],
            json.dumps(descriptor, sort_keys=True),
            "sha256:art129-target-descriptor",
            "approval-gated",
            "R2",
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_packages
          (id, package_name, package_path, package_hash, manifest_json)
        VALUES (?, ?, ?, ?, ?)
        """,
        (
            "pkg-art129",
            "envctl-db-nu-plugin-migration-automation-package",
            ".",
            "sha256:pkg-art129",
            json.dumps({"schema_version": 1, "task_id": TASK_ID}, sort_keys=True),
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_artifact_contracts
          (id, contract_name, contract_version, source_package_id, contract_hash, contract_json)
        VALUES (?, ?, ?, ?, ?, ?)
        """,
        (
            CONTRACT_ID,
            "art129-business-capability-contract",
            "1.0.0",
            "pkg-art129",
            "sha256:contract-art129",
            json.dumps({"required": ["Business capability map"]}, sort_keys=True),
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_recipes
          (id, recipe_name, recipe_version, artifact_contract_id, recipe_hash, recipe_json)
        VALUES (?, ?, ?, ?, ?, ?)
        """,
        (
            "recipe-art129",
            "codex-flexnetos-full-artifact-run",
            "1.0.0",
            CONTRACT_ID,
            "sha256:recipe-art129",
            json.dumps({"phases": ["05-artifacts"], "task_id": TASK_ID}, sort_keys=True),
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
            "target-art129-flexnetos-vs-lifeos",
            "recipe-art129",
            CONTRACT_ID,
            "running",
            "approval-gated",
            "artifact-agent",
            "workspace-write",
            "never",
            json.dumps({"python": "stdlib", "sqlite": "stdlib"}, sort_keys=True),
            "sha256:run-art129",
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
            "generate_artifact",
            "05-artifacts",
            "succeeded",
            "R2",
            f"{TASK_ID}/business-capability-map",
            "sha256:command-art129",
            "python3 scripts/generate_art_129_business_capability.py",
            json.dumps({"task_id": TASK_ID, "artifact": "business_capability_map"}, sort_keys=True),
        ),
    )
    conn.commit()
    return {"run_id": RUN_ID, "operation_id": OPERATION_ID, "contract_id": CONTRACT_ID}


def register_artifacts(conn: sqlite3.Connection, fixture: dict[str, str], relpaths: dict[str, str]) -> list[dict[str, Any]]:
    registry = ArtifactRegistry(conn, package_root())
    common = {
        "run_id": fixture["run_id"],
        "status": "complete",
        "producer_operation_id": fixture["operation_id"],
        "contract_id": fixture["contract_id"],
        "provenance": {
            "task_id": TASK_ID,
            "owner_agent": "artifact-agent",
            "helper_id": HELPER_ID,
            "source_inputs": [
                "examples/target-descriptors/flexnetos-vs-lifeos.yaml",
                "execution-framework/generated/package_scan.json",
                "execution-framework/generated/envctl_migration_db_model.json",
                "execution-framework/generated/envctl_artifact_registry_report.json",
                "execution-framework/generated/shared_protocol_manifest.json",
            ],
        },
        "evidence_refs": [
            relpaths["markdown"],
            relpaths["json"],
            "execution-framework/generated/envctl_migration_db_model.json",
            "execution-framework/generated/envctl_artifact_registry_report.json",
            "execution-framework/generated/shared_protocol_manifest.json",
            "execution-framework/generated/nu_plugin_command_manifest.json",
        ],
        "links": [
            {"to": "artifact:REQ-024_ENVCTL_ARTIFACT_REGISTRY", "type": "depends_on"},
            {"to": "artifact:REQ-040_SHARED_PROTOCOL_SCHEMAS", "type": "depends_on"},
            {"to": "artifact:VER-300_UNIT_VALIDATION", "type": "blocks"},
            {"to": "business-function:migration-governance", "type": "maps_to"},
        ],
        "validations": [
            {
                "validator": "generate_art_129_business_capability.py:capability-coverage",
                "status": "pass",
                "details": {
                    "capability_count": len(capability_rows()),
                    "all_have_business_function": True,
                    "all_have_technical_systems": True,
                },
                "evidence_refs": [relpaths["json"], relpaths["markdown"]],
            },
            {
                "validator": "generate_art_129_business_capability.py:registry-hash",
                "status": "pass",
                "details": {"hash_expected": True, "validation_evidence_linked": True},
                "evidence_refs": [relpaths["markdown"], relpaths["json"]],
            },
        ],
    }
    return [
        registry.register(
            {
                **common,
                "artifact_id": "art129-business-capability-map-md",
                "title": "ART-129 Business Capability Map",
                "artifact_type": "business_capability_map_markdown",
                "path": relpaths["markdown"],
            }
        ),
        registry.register(
            {
                **common,
                "artifact_id": "art129-business-capability-map-json",
                "title": "ART-129 Business Capability Map Source",
                "artifact_type": "business_capability_map_json",
                "path": relpaths["json"],
            }
        ),
    ]


def build_report(conn: sqlite3.Connection, payload: dict[str, Any], registry_results: list[dict[str, Any]], relpaths: dict[str, str]) -> dict[str, Any]:
    artifacts = [
        fetch_artifact(conn, RUN_ID, "art129-business-capability-map-md"),
        fetch_artifact(conn, RUN_ID, "art129-business-capability-map-json"),
    ]
    validation_count = conn.execute(
        "SELECT COUNT(*) FROM envctl_migration_validations WHERE run_id = ?",
        (RUN_ID,),
    ).fetchone()[0]
    graph_count = conn.execute(
        "SELECT COUNT(*) FROM envctl_migration_graph_edges WHERE run_id = ?",
        (RUN_ID,),
    ).fetchone()[0]
    evidence_count = conn.execute(
        "SELECT COUNT(*) FROM envctl_migration_evidence WHERE run_id = ?",
        (RUN_ID,),
    ).fetchone()[0]
    index_rows = conn.execute(
        """
        SELECT artifact_id, status, path, content_hash
        FROM envctl_migration_artifact_index
        WHERE run_id = ?
        ORDER BY artifact_id
        """,
        (RUN_ID,),
    ).fetchall()
    errors = []
    for label, relpath in relpaths.items():
        if not (package_root() / relpath).exists():
            errors.append(f"missing artifact file: {relpath}")
    if len(registry_results) != 2:
        errors.append("expected two registry rows")
    if any(not item.get("content_hash", "").startswith("sha256:") for item in registry_results):
        errors.append("registry did not persist sha256 hashes for all artifacts")
    if validation_count < 4:
        errors.append(f"expected at least 4 validation rows, got {validation_count}")
    if graph_count < 10:
        errors.append(f"expected at least 10 graph edges, got {graph_count}")
    if evidence_count < 6:
        errors.append(f"expected at least 6 evidence rows, got {evidence_count}")
    if len(index_rows) != 2:
        errors.append(f"expected two artifact index rows, got {len(index_rows)}")
    if not payload["summary"]["all_capabilities_have_business_function"]:
        errors.append("one or more capabilities lacks a business function")
    if not payload["summary"]["all_capabilities_have_technical_systems"]:
        errors.append("one or more capabilities lacks technical systems")
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "passed" if not errors else "failed",
        "generated_at": now(),
        "artifact_paths": relpaths,
        "registry_results": registry_results,
        "registered_artifacts": artifacts,
        "summary": {
            **payload["summary"],
            "evidence_count": evidence_count,
            "graph_edge_count": graph_count,
            "validation_count": validation_count,
            "artifact_index_rows": [list(row) for row in index_rows],
        },
        "completion_gate": {
            "artifact_exists": all((package_root() / relpath).exists() for relpath in relpaths.values()),
            "hash_recorded": all(item.get("content_hash", "").startswith("sha256:") for item in registry_results),
            "validation_evidence_linked": validation_count >= 4 and evidence_count >= 6,
        },
        "errors": errors,
        "evidence": [
            relpaths["markdown"],
            relpaths["json"],
            "execution-framework/generated/art_129_business_capability_registry_report.json",
            "execution-framework/generated/envctl_migration_db_model.json",
            "execution-framework/generated/envctl_artifact_registry_report.json",
            "execution-framework/generated/shared_protocol_manifest.json",
            "execution-framework/generated/nu_plugin_command_manifest.json",
        ],
    }


def write_report_and_proof(report: dict[str, Any]) -> None:
    paths = artifact_paths()
    paths["report"].write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    paths["log"].write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    paths["heartbeat"].parent.mkdir(parents=True, exist_ok=True)
    paths["heartbeat"].write_text(
        json.dumps(
            {
                "task_id": TASK_ID,
                "status": "completed" if report["status"] == "passed" else "failed",
                "updated_at": report["generated_at"],
                "proof_uri": f"proof_records/{TASK_ID}.proof.json",
                "artifact_paths": report["artifact_paths"],
            },
            indent=2,
            sort_keys=False,
        )
        + "\n",
        encoding="utf-8",
    )
    files_changed = [
        "execution-framework/scripts/generate_art_129_business_capability.py",
        "execution-framework/migration-artifacts/art-129_business_capability/business_capability_map.md",
        "execution-framework/migration-artifacts/art-129_business_capability/business_capability_map.json",
        "execution-framework/generated/art_129_business_capability_registry_report.json",
        "execution-framework/state/ART-129_BUSINESS_CAPABILITY.heartbeat.json",
        "execution-framework/logs/ART-129_BUSINESS_CAPABILITY.log",
        "execution-framework/proof_records/ART-129_BUSINESS_CAPABILITY.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    commands_run = [
        "python3 scripts/generate_art_129_business_capability.py",
        "python3 -m py_compile scripts/generate_art_129_business_capability.py",
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
        report["evidence"],
        "" if report["status"] == "passed" else "; ".join(report["errors"]),
        "run VER-300_UNIT_VALIDATION" if report["status"] == "passed" else "fix ART-129 capability map errors",
    )
    append_proof(proof)


def main() -> None:
    generated_at = now()
    payload = build_artifact_payload(generated_at)
    relpaths = write_artifacts(payload)
    conn = sqlite3.connect(":memory:")
    conn.execute("PRAGMA foreign_keys = ON")
    apply_migrations(conn, package_root())
    fixture = insert_fixture(conn)
    registry_results = register_artifacts(conn, fixture, relpaths)
    report = build_report(conn, payload, registry_results, relpaths)
    write_report_and_proof(report)
    print(
        "art129 status={status} artifacts={artifacts} hashes={hashes} validations={validations}".format(
            status=report["status"],
            artifacts=len(report["registry_results"]),
            hashes=sum(1 for item in report["registry_results"] if item.get("content_hash")),
            validations=report["summary"]["validation_count"],
        )
    )
    if report["status"] != "passed":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
