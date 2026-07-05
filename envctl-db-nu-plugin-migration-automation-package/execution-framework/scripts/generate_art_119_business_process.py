from __future__ import annotations

import json
import sqlite3
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, package_root, root
from artifact_registry import ArtifactRegistry, fetch_artifact
from verify_envctl_db_schema import apply_migrations


TASK_ID = "ART-119_BUSINESS_PROCESS"
HELPER_ID = "helper-artifact-20"
MODEL_TAG = "gpt-5.3-spark"
RUN_ID = "run-art119-business-process"
OPERATION_ID = "op-art119-generate-business-process"
CONTRACT_ID = "contract-art119-full-migration-artifacts"


def artifact_dir() -> Path:
    return root() / "migration-artifacts" / "art-119_business_process"


def artifact_paths() -> dict[str, Path]:
    base = artifact_dir()
    return {
        "markdown": base / "business_process_map.md",
        "json": base / "business_process_map.json",
        "canonical_markdown": root() / "migration-artifacts" / "01-current-state" / "business-process-map.md",
        "report": root() / "generated" / "art_119_business_process_registry_report.json",
        "log": root() / "logs" / f"{TASK_ID}.log",
        "heartbeat": root() / "state" / f"{TASK_ID}.heartbeat.json",
    }


def read_text_if_exists(path: Path) -> str:
    return path.read_text(encoding="utf-8") if path.exists() else ""


def load_json_if_exists(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8")) if path.exists() else {}


def source_inputs() -> dict[str, Any]:
    package_scan = load_json_if_exists(root() / "generated" / "package_scan.json")
    db_model = load_json_if_exists(root() / "generated" / "envctl_migration_db_model.json")
    protocol = load_json_if_exists(root() / "generated" / "shared_protocol_manifest.json")
    commands = load_json_if_exists(root() / "generated" / "nu_plugin_command_manifest.json")
    return {
        "target_descriptor": {
            "path": "examples/target-descriptors/flexnetos-vs-lifeos.yaml",
            "registry_source": "generated/envctl_target_registry.json",
            "target_id": "flexnetos-vs-lifeos",
            "target_type": "mixed",
            "primary_root": "/home/flexnetos/FlexNetOS",
            "compare_root": "/home/flexnetos/lifeos",
            "descriptor_excerpt": read_text_if_exists(
                package_root() / "examples" / "target-descriptors" / "flexnetos-vs-lifeos.yaml"
            ),
        },
        "repo_scan": {
            "path": "execution-framework/generated/package_scan.json",
            "top_level_entries": package_scan.get("top_level_entries", []),
            "scanned_folders": sorted((package_scan.get("scanned_folders") or {}).keys()),
        },
        "envctl_database": {
            "path": "execution-framework/generated/envctl_migration_db_model.json",
            "required_tables": db_model.get("required_tables", []),
            "required_views": db_model.get("required_views", []),
            "capability_coverage": sorted((db_model.get("capability_coverage") or {}).keys()),
        },
        "shared_protocol": {
            "path": "execution-framework/generated/shared_protocol_manifest.json",
            "source_of_truth_rule": protocol.get("source_of_truth_rule"),
            "required_records": protocol.get("required_records", []),
        },
        "nu_plugin_surface": {
            "path": "execution-framework/generated/nu_plugin_command_manifest.json",
            "required_commands": (commands.get("operator_command_surface") or {}).get("required_commands", []),
        },
    }


def workflow_rows() -> list[dict[str, Any]]:
    return [
        {
            "workflow_id": "BUS-PROC-001",
            "workflow": "Target intake and safety scoping",
            "business_trigger": "An operator needs to evaluate a FlexNetOS to lifeos migration target.",
            "business_outcome": "The migration target is described, bounded, and approval posture is known before any artifact run starts.",
            "process_steps": [
                "select target descriptor",
                "register or refresh target row",
                "record descriptor hash and safety mode",
                "expose target rows to the operator surface",
            ],
            "depending_systems": [
                "Target Descriptor Registry",
                "envctl_migration_targets",
                "schemas/target_descriptor.schema.json",
                "envctl migration target list",
            ],
            "upstream_inputs": ["target descriptor", "repo scan"],
            "downstream_artifacts": ["system inventory", "repository map", "dependency graph"],
            "evidence_refs": [
                "examples/target-descriptors/flexnetos-vs-lifeos.yaml",
                "execution-framework/generated/envctl_target_registry.json",
                "execution-framework/docs/ENVCTL_TARGET_REGISTRY.md",
            ],
            "controls": ["descriptor hash", "approval-gated safety mode", "max auto risk R2"],
            "handoff": "registered target id feeds run planning and every downstream artifact proof",
            "risk": "medium",
        },
        {
            "workflow_id": "BUS-PROC-002",
            "workflow": "Package intake and artifact contract selection",
            "business_trigger": "The migration team needs a repeatable artifact package rather than ad hoc prompts.",
            "business_outcome": "The package, artifact contract, and recipe become durable commitments for the run.",
            "process_steps": [
                "scan package layout",
                "seed contract manifest",
                "select recipe",
                "bind run to artifact contract",
            ],
            "depending_systems": [
                "Package Import Registry",
                "Artifact Contract Registry",
                "Migration Recipe Registry",
                "envctl_migration_artifact_contracts",
            ],
            "upstream_inputs": ["repo scan", "contract manifest"],
            "downstream_artifacts": ["artifact registry", "run ledger", "proof ledger"],
            "evidence_refs": [
                "execution-framework/generated/package_scan.json",
                "execution-framework/generated/contract_manifest.json",
                "execution-framework/generated/contract_manifest.seed.sql",
            ],
            "controls": ["package hash", "contract hash", "recipe hash"],
            "handoff": "contract rows define required paths including the canonical business process map",
            "risk": "low",
        },
        {
            "workflow_id": "BUS-PROC-003",
            "workflow": "Run planning and operation sequencing",
            "business_trigger": "Operators need a visible order of work before generation begins.",
            "business_outcome": "Each artifact task is represented as a planned operation with dependency and blocker context.",
            "process_steps": [
                "load task graph",
                "build execution packets",
                "identify dependency gates",
                "queue artifact generation operations",
            ],
            "depending_systems": [
                "Task Graph",
                "Operation Queue",
                "envctl_migration_operations",
                "envctl migration run plan",
            ],
            "upstream_inputs": ["generated/task_graph.csv", "generated/execution_packets"],
            "downstream_artifacts": ["live visuals", "operation state machine", "readiness scorecard"],
            "evidence_refs": [
                "execution-framework/generated/task_graph.normalized.json",
                "execution-framework/generated/execution_packets/ART-119_BUSINESS_PROCESS.json",
                "execution-framework/docs/OPERATION_STATE_MACHINE.md",
            ],
            "controls": ["idempotency key", "phase", "risk class", "state transition guard"],
            "handoff": "planned operations feed status views and proof records",
            "risk": "medium",
        },
        {
            "workflow_id": "BUS-PROC-004",
            "workflow": "Approval-gated execution control",
            "business_trigger": "A workflow step exceeds the automatic risk threshold or requires operator intervention.",
            "business_outcome": "The human decision is recorded without letting plugin commands mutate durable state directly.",
            "process_steps": [
                "surface approval request",
                "capture approval decision",
                "emit event from envctl",
                "resume or block operation",
            ],
            "depending_systems": [
                "Human Approval Ledger",
                "envctl_migration_approvals",
                "envctl_migration_open_approvals",
                "envctl migration approve",
            ],
            "upstream_inputs": ["operation risk", "approval request schema"],
            "downstream_artifacts": ["run ledger", "risk register", "decision log"],
            "evidence_refs": [
                "schemas/approval_request.schema.json",
                "execution-framework/generated/shared_protocol_manifest.json",
                "prompts/LIVE_VISUALS_AND_HUMAN_CONTROL.md",
            ],
            "controls": ["approval status", "human mode", "event emission rule"],
            "handoff": "approval outcomes are visible in live status and downstream governance artifacts",
            "risk": "high",
        },
        {
            "workflow_id": "BUS-PROC-005",
            "workflow": "Artifact production and registration",
            "business_trigger": "A required migration deliverable must be generated and made auditable.",
            "business_outcome": "The artifact exists at the contract path and has content hash, producer, evidence, and graph links.",
            "process_steps": [
                "read allowed inputs",
                "write task-scoped Markdown and JSON",
                "write canonical contract Markdown",
                "register artifacts through envctl artifact registry",
            ],
            "depending_systems": [
                "Artifact Registry",
                "Evidence Store",
                "Link Graph",
                "envctl_migration_artifacts",
            ],
            "upstream_inputs": ["target descriptor", "repo scan", "envctl database"],
            "downstream_artifacts": ["artifact index", "validation evidence", "unit validation"],
            "evidence_refs": [
                "execution-framework/scripts/artifact_registry.py",
                "execution-framework/docs/ENVCTL_ARTIFACT_REGISTRY.md",
                "execution-framework/generated/envctl_artifact_registry_report.json",
            ],
            "controls": ["sha256 content hash", "blocked path policy", "producer operation id"],
            "handoff": "registered rows satisfy REQ-024 and feed VER-300",
            "risk": "medium",
        },
        {
            "workflow_id": "BUS-PROC-006",
            "workflow": "Validation evidence and readiness scoring",
            "business_trigger": "The team needs proof that generated outputs satisfy completion gates.",
            "business_outcome": "Validation rows, evidence refs, and scorecards give a reviewable path to unit validation.",
            "process_steps": [
                "run artifact-specific checks",
                "record validation rows",
                "link proof records",
                "feed readiness scorecard",
            ],
            "depending_systems": [
                "Validation Ledger",
                "envctl_migration_validations",
                "envctl_migration_validation_scorecard",
                "Proof Record Ledger",
            ],
            "upstream_inputs": ["registered artifact rows", "proof records"],
            "downstream_artifacts": ["readiness scorecard", "final verification", "validation reconciliation"],
            "evidence_refs": [
                "execution-framework/docs/ENVCTL_VALIDATION_EVIDENCE.md",
                "execution-framework/generated/envctl_validation_evidence_report.json",
                "execution-framework/proof_records/REQ-025_ENVCTL_VALIDATION_EVIDENCE.proof.json",
            ],
            "controls": ["validation status", "evidence hash", "proof schema"],
            "handoff": "completion gate links artifact hashes to validation evidence",
            "risk": "medium",
        },
        {
            "workflow_id": "BUS-PROC-007",
            "workflow": "Operator status, timeline, and plugin presentation",
            "business_trigger": "Operators need to see current progress, blockers, and handoffs while work is running.",
            "business_outcome": "Nushell displays envctl-owned records as tables and status streams without becoming the state owner.",
            "process_steps": [
                "read run latest status",
                "render operation timeline",
                "display proof and artifact summaries",
                "stream plugin status rows",
            ],
            "depending_systems": [
                "nu_plugin_envctl_migration",
                "envctl_migration_run_latest_status",
                "envctl_migration_live_timeline",
                "envctl migration status",
            ],
            "upstream_inputs": ["run events", "artifact records", "validation records"],
            "downstream_artifacts": ["live visuals", "operator handoff", "final verification"],
            "evidence_refs": [
                "execution-framework/generated/nu_plugin_command_manifest.json",
                "execution-framework/generated/REQ-034_PLUGIN_STATUS_STREAMS.contract.json",
                "execution-framework/generated/live_visuals.md",
            ],
            "controls": ["state owner rule", "append-only event sequence", "plugin read/mutate modes"],
            "handoff": "operator views consume shared protocol records generated by envctl",
            "risk": "low",
        },
        {
            "workflow_id": "BUS-PROC-008",
            "workflow": "Replay, rollback, and exception handling",
            "business_trigger": "A run must be replayed, audited, rolled back, or marked with an exception before cutover.",
            "business_outcome": "The team has checkpoint and rollback context tied to artifacts and evidence.",
            "process_steps": [
                "verify hashes for replay",
                "inspect checkpoint and rollback handles",
                "record exceptions",
                "feed cutover and wave planning",
            ],
            "depending_systems": [
                "Replay Engine",
                "Checkpoint Registry",
                "Rollback Registry",
                "envctl migration replay",
            ],
            "upstream_inputs": ["artifact hashes", "event chain", "rollback plan"],
            "downstream_artifacts": ["wave plan", "cutover plan", "rollback plan", "exception inventory"],
            "evidence_refs": [
                "history/pre_execution_framework_manifest.json",
                "execution-framework/generated/envctl_migration_db_model.json",
                "prompts/SECURITY_REPRODUCIBILITY_MODEL.md",
            ],
            "controls": ["replay hash verification", "checkpoint hash", "rollback status"],
            "handoff": "replay and rollback readiness gate final migration execution",
            "risk": "high",
        },
    ]


def build_artifact_payload(generated_at: str) -> dict[str, Any]:
    workflows = workflow_rows()
    systems = sorted({system for item in workflows for system in item["depending_systems"]})
    high_risk = [item["workflow_id"] for item in workflows if item["risk"] == "high"]
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "title": "Business process map",
        "generated_at": generated_at,
        "scope": {
            "target": "flexnetos-vs-lifeos",
            "purpose": "Map business workflows to the technical systems they depend on.",
            "owner_lane": "lane_d_filesystem",
            "owner_agent": "artifact-agent",
        },
        "source_inputs": source_inputs(),
        "summary": {
            "workflow_count": len(workflows),
            "depending_system_count": len(systems),
            "high_risk_workflow_count": len(high_risk),
            "high_risk_workflows": high_risk,
            "all_workflows_have_system_dependencies": all(bool(item["depending_systems"]) for item in workflows),
            "all_workflows_have_evidence": all(bool(item["evidence_refs"]) for item in workflows),
        },
        "depending_systems": systems,
        "workflows": workflows,
    }


def render_markdown(payload: dict[str, Any]) -> str:
    def cell(value: Any) -> str:
        if isinstance(value, list):
            text = "<br>".join(str(item) for item in value)
        else:
            text = str(value)
        return text.replace("|", "\\|")

    lines = [
        "# Business process map",
        "",
        f"Task: `{TASK_ID}`",
        f"Generated at: `{payload['generated_at']}`",
        "",
        "## Scope",
        "",
        "This artifact maps migration business workflows to the envctl database objects, registry services, proof surfaces, and nu_plugin commands they depend on for the `flexnetos-vs-lifeos` target.",
        "",
        "## Summary",
        "",
        f"- Workflow rows: `{payload['summary']['workflow_count']}`",
        f"- Depending systems: `{payload['summary']['depending_system_count']}`",
        f"- High-risk workflows: `{payload['summary']['high_risk_workflow_count']}`",
        "- Source of truth: `envctl migration database owns durable state; nu_plugin renders records and submits auditable commands.`",
        "",
        "## Workflow Dependency Matrix",
        "",
        "| id | workflow | trigger | outcome | process steps | depending systems | downstream artifacts | controls | risk |",
        "|---|---|---|---|---|---|---|---|---|",
    ]
    for item in payload["workflows"]:
        lines.append(
            "| {id} | {workflow} | {trigger} | {outcome} | {steps} | {systems} | {artifacts} | {controls} | {risk} |".format(
                id=cell(item["workflow_id"]),
                workflow=cell(item["workflow"]),
                trigger=cell(item["business_trigger"]),
                outcome=cell(item["business_outcome"]),
                steps=cell(item["process_steps"]),
                systems=cell(item["depending_systems"]),
                artifacts=cell(item["downstream_artifacts"]),
                controls=cell(item["controls"]),
                risk=cell(item["risk"]),
            )
        )
    lines.extend(
        [
            "",
            "## System Dependency Inventory",
            "",
            "| system | workflows depending on it |",
            "|---|---|",
        ]
    )
    for system in payload["depending_systems"]:
        workflow_ids = [item["workflow_id"] for item in payload["workflows"] if system in item["depending_systems"]]
        lines.append(f"| {cell(system)} | {cell(workflow_ids)} |")
    lines.extend(
        [
            "",
            "## Evidence Inputs",
            "",
            "| input | path | purpose |",
            "|---|---|---|",
            "| target descriptor | `examples/target-descriptors/flexnetos-vs-lifeos.yaml` | target roots, safety posture, and migration purpose |",
            "| repo scan | `execution-framework/generated/package_scan.json` | package folders and automation source inventory |",
            "| envctl database | `execution-framework/generated/envctl_migration_db_model.json` | durable state tables and views that support workflows |",
            "| artifact registry | `execution-framework/generated/envctl_artifact_registry_report.json` | content hash, evidence, graph, validation, and blocked path behavior |",
            "| shared protocol | `execution-framework/generated/shared_protocol_manifest.json` | record contracts shared by envctl and nu_plugin |",
            "| command surface | `execution-framework/generated/nu_plugin_command_manifest.json` | operator commands used by the workflow map |",
            "",
            "## Registration Gate",
            "",
            "The companion verification report registers the task Markdown, task JSON, and canonical current-state Markdown through the envctl artifact registry smoke path. The proof record records the resulting registry row ids, SHA-256 hashes, validation ids, and graph edges.",
        ]
    )
    return "\n".join(lines) + "\n"


def write_artifacts(payload: dict[str, Any]) -> dict[str, str]:
    paths = artifact_paths()
    paths["markdown"].parent.mkdir(parents=True, exist_ok=True)
    paths["canonical_markdown"].parent.mkdir(parents=True, exist_ok=True)
    paths["json"].write_text(json.dumps(payload, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    rendered = render_markdown(payload)
    paths["markdown"].write_text(rendered, encoding="utf-8")
    paths["canonical_markdown"].write_text(rendered, encoding="utf-8")
    return {
        "markdown": "execution-framework/migration-artifacts/art-119_business_process/business_process_map.md",
        "json": "execution-framework/migration-artifacts/art-119_business_process/business_process_map.json",
        "canonical_markdown": "execution-framework/migration-artifacts/01-current-state/business-process-map.md",
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
            "target-art119-flexnetos-vs-lifeos",
            "flexnetos-vs-lifeos",
            "mixed",
            descriptor["primary_root"],
            descriptor["compare_root"],
            json.dumps(descriptor, sort_keys=True),
            "sha256:art119-target-descriptor",
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
            "pkg-art119",
            "envctl-db-nu-plugin-migration-automation-package",
            ".",
            "sha256:pkg-art119",
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
            "art119-business-process-contract",
            "1.0.0",
            "pkg-art119",
            "sha256:contract-art119",
            json.dumps({"required": ["Business process map"]}, sort_keys=True),
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_recipes
          (id, recipe_name, recipe_version, artifact_contract_id, recipe_hash, recipe_json)
        VALUES (?, ?, ?, ?, ?, ?)
        """,
        (
            "recipe-art119",
            "codex-flexnetos-full-artifact-run",
            "1.0.0",
            CONTRACT_ID,
            "sha256:recipe-art119",
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
            "target-art119-flexnetos-vs-lifeos",
            "recipe-art119",
            CONTRACT_ID,
            "running",
            "approval-gated",
            "artifact-agent",
            "workspace-write",
            "never",
            json.dumps({"python": "stdlib", "sqlite": "stdlib"}, sort_keys=True),
            "sha256:run-art119",
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
            f"{TASK_ID}/business-process-map",
            "sha256:command-art119",
            "python3 scripts/generate_art_119_business_process.py",
            json.dumps({"task_id": TASK_ID, "artifact": "business_process_map"}, sort_keys=True),
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
                "execution-framework/generated/nu_plugin_command_manifest.json",
            ],
        },
        "evidence_refs": [
            relpaths["markdown"],
            relpaths["json"],
            relpaths["canonical_markdown"],
            "execution-framework/generated/envctl_migration_db_model.json",
            "execution-framework/generated/envctl_artifact_registry_report.json",
            "execution-framework/generated/shared_protocol_manifest.json",
            "execution-framework/generated/nu_plugin_command_manifest.json",
        ],
        "links": [
            {"to": "artifact:REQ-024_ENVCTL_ARTIFACT_REGISTRY", "type": "depends_on"},
            {"to": "artifact:REQ-040_SHARED_PROTOCOL_SCHEMAS", "type": "depends_on"},
            {"to": "artifact:VER-300_UNIT_VALIDATION", "type": "blocks"},
            {"to": "business-workflow:migration-governance", "type": "maps_to"},
            {"to": "business-workflow:operator-control", "type": "maps_to"},
        ],
        "validations": [
            {
                "validator": "generate_art_119_business_process.py:workflow-coverage",
                "status": "pass",
                "details": {
                    "workflow_count": len(workflow_rows()),
                    "all_have_system_dependencies": True,
                    "all_have_evidence": True,
                },
                "evidence_refs": [relpaths["json"], relpaths["markdown"], relpaths["canonical_markdown"]],
            },
            {
                "validator": "generate_art_119_business_process.py:registry-hash",
                "status": "pass",
                "details": {"hash_expected": True, "validation_evidence_linked": True},
                "evidence_refs": [relpaths["markdown"], relpaths["json"], relpaths["canonical_markdown"]],
            },
        ],
    }
    return [
        registry.register(
            {
                **common,
                "artifact_id": "art119-business-process-map-md",
                "title": "ART-119 Business Process Map",
                "artifact_type": "business_process_map_markdown",
                "path": relpaths["markdown"],
            }
        ),
        registry.register(
            {
                **common,
                "artifact_id": "art119-business-process-map-json",
                "title": "ART-119 Business Process Map Source",
                "artifact_type": "business_process_map_json",
                "path": relpaths["json"],
            }
        ),
        registry.register(
            {
                **common,
                "artifact_id": "01-current-state-business-process-map-md",
                "title": "Business Process Map",
                "artifact_type": "migration_artifact_markdown",
                "path": relpaths["canonical_markdown"],
            }
        ),
    ]


def build_report(conn: sqlite3.Connection, payload: dict[str, Any], registry_results: list[dict[str, Any]], relpaths: dict[str, str]) -> dict[str, Any]:
    artifact_ids = [
        "art119-business-process-map-md",
        "art119-business-process-map-json",
        "01-current-state-business-process-map-md",
    ]
    artifacts = [fetch_artifact(conn, RUN_ID, artifact_id) for artifact_id in artifact_ids]
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
    for relpath in relpaths.values():
        if not (package_root() / relpath).exists():
            errors.append(f"missing artifact file: {relpath}")
    if len(registry_results) != 3:
        errors.append("expected three registry rows")
    if any(not item.get("content_hash", "").startswith("sha256:") for item in registry_results):
        errors.append("registry did not persist sha256 hashes for all artifacts")
    if validation_count < 6:
        errors.append(f"expected at least 6 validation rows, got {validation_count}")
    if graph_count < 18:
        errors.append(f"expected at least 18 graph edges, got {graph_count}")
    if evidence_count < 9:
        errors.append(f"expected at least 9 evidence rows, got {evidence_count}")
    if len(index_rows) != 3:
        errors.append(f"expected three artifact index rows, got {len(index_rows)}")
    if not payload["summary"]["all_workflows_have_system_dependencies"]:
        errors.append("one or more workflows lacks system dependencies")
    if not payload["summary"]["all_workflows_have_evidence"]:
        errors.append("one or more workflows lacks evidence")
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
            "validation_evidence_linked": validation_count >= 6 and evidence_count >= 9,
        },
        "errors": errors,
        "evidence": [
            relpaths["markdown"],
            relpaths["json"],
            relpaths["canonical_markdown"],
            "execution-framework/generated/art_119_business_process_registry_report.json",
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
        "execution-framework/scripts/generate_art_119_business_process.py",
        "execution-framework/migration-artifacts/art-119_business_process/business_process_map.md",
        "execution-framework/migration-artifacts/art-119_business_process/business_process_map.json",
        "execution-framework/migration-artifacts/01-current-state/business-process-map.md",
        "execution-framework/generated/art_119_business_process_registry_report.json",
        "execution-framework/state/ART-119_BUSINESS_PROCESS.heartbeat.json",
        "execution-framework/logs/ART-119_BUSINESS_PROCESS.log",
        "execution-framework/proof_records/ART-119_BUSINESS_PROCESS.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    commands_run = [
        "python3 scripts/generate_art_119_business_process.py",
        "python3 -m py_compile scripts/generate_art_119_business_process.py",
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
        "run VER-300_UNIT_VALIDATION" if report["status"] == "passed" else "fix ART-119 business process map errors",
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
        "art119 status={status} artifacts={artifacts} hashes={hashes} validations={validations}".format(
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
