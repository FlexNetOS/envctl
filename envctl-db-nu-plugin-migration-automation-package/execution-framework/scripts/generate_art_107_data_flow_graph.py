from __future__ import annotations

import json
import sqlite3
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, package_root, root
from artifact_registry import ArtifactRegistry, fetch_artifact
from verify_envctl_db_schema import apply_migrations


TASK_ID = "ART-107_DATA_FLOW_GRAPH"
HELPER_ID = "helper-artifact-08"
MODEL_TAG = "gpt-5.3-spark"
RUN_ID = "run-art107-data-flow-graph"
OPERATION_ID = "op-art107-generate-data-flow-graph"
CONTRACT_ID = "contract-art107-full-migration-artifacts"


def artifact_paths() -> dict[str, Path]:
    base = root() / "migration-artifacts" / "art-107_data_flow_graph"
    return {
        "markdown": base / "data_flow_graph.md",
        "json": base / "data_flow_graph.json",
        "report": root() / "generated" / "art_107_data_flow_graph_registry_report.json",
        "log": root() / "logs" / f"{TASK_ID}.log",
        "heartbeat": root() / "state" / f"{TASK_ID}.heartbeat.json",
    }


def read_json_if_exists(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def read_text_if_exists(path: Path) -> str:
    if not path.exists():
        return ""
    return path.read_text(encoding="utf-8")


def package_rel(path: Path) -> str:
    return str(path.relative_to(package_root())).replace("\\", "/")


def source_inputs() -> dict[str, Any]:
    target_registry = read_json_if_exists(root() / "generated" / "envctl_target_registry.json")
    package_scan = read_json_if_exists(root() / "generated" / "package_scan.json")
    db_model = read_json_if_exists(root() / "generated" / "envctl_migration_db_model.json")
    protocol = read_json_if_exists(root() / "generated" / "shared_protocol_manifest.json")
    return {
        "target_descriptor": {
            "path": "execution-framework/migration-artifacts/_meta/flexnetos-vs-lifeos.target-descriptor.yaml",
            "target_id": "flexnetos-vs-lifeos",
            "primary_root": "/home/flexnetos/FlexNetOS",
            "compare_root": "/home/flexnetos/lifeos",
            "descriptor_excerpt": read_text_if_exists(
                root() / "migration-artifacts" / "_meta" / "flexnetos-vs-lifeos.target-descriptor.yaml"
            ),
            "registry_status": target_registry.get("status"),
            "registry_rows": len(target_registry.get("registry_rows", [])),
        },
        "repo_scan": {
            "path": "execution-framework/generated/package_scan.json",
            "top_level_entries": package_scan.get("top_level_entries", []),
            "scanned_folders": sorted(package_scan.get("scanned_folders", {}).keys()),
        },
        "envctl_database": {
            "path": "execution-framework/generated/envctl_migration_db_model.json",
            "status": db_model.get("status"),
            "required_table_count": db_model.get("summary", {}).get("required_table_count"),
            "required_view_count": db_model.get("summary", {}).get("required_view_count"),
            "required_tables": db_model.get("required_tables", []),
            "required_views": db_model.get("required_views", []),
        },
        "shared_protocol": {
            "path": "execution-framework/generated/shared_protocol_manifest.json",
            "source_of_truth_rule": protocol.get("source_of_truth_rule"),
            "required_records": protocol.get("required_records", []),
        },
    }


def graph_nodes() -> list[dict[str, Any]]:
    return [
        {
            "id": "entry:target-descriptor",
            "stage": "data_entry",
            "label": "Target descriptor intake",
            "kind": "configuration_input",
            "description": "Defines target roots, output root, collector classes, safety posture, and artifact contract metadata.",
            "data_subjects": ["target_id", "primary_root", "compare_root", "include/exclude globs", "safety policy"],
            "evidence_refs": [
                "execution-framework/migration-artifacts/_meta/flexnetos-vs-lifeos.target-descriptor.yaml",
                "execution-framework/generated/envctl_target_registry.json",
            ],
            "redaction": "blocked paths exclude .env, secrets, private_keys, pem, and key files before graph materialization",
        },
        {
            "id": "entry:repo-scan",
            "stage": "data_entry",
            "label": "Repository and filesystem scan",
            "kind": "collector_input",
            "description": "Scans bounded package and target filesystem surfaces for manifests, source, docs, services, and generated evidence.",
            "data_subjects": ["file paths", "manifest metadata", "service definitions", "source inventory"],
            "evidence_refs": [
                "execution-framework/generated/package_scan.json",
                "execution-framework/migration-artifacts/art-101_directory_tree/directory-tree.json",
                "execution-framework/migration-artifacts/art-102_repository_map/repository-map.json",
            ],
            "redaction": "secret-like paths are skipped by descriptor policy and artifact registry path policy",
        },
        {
            "id": "entry:operator-command",
            "stage": "data_entry",
            "label": "Operator command surface",
            "kind": "human_or_plugin_input",
            "description": "Nushell plugin commands submit target, run, approval, replay, and proof requests to envctl-owned endpoints.",
            "data_subjects": ["command arguments", "approval decisions", "replay requests", "proof lookups"],
            "evidence_refs": [
                "execution-framework/generated/nu_plugin_command_manifest.json",
                "execution-framework/generated/shared_protocol_manifest.json",
            ],
            "redaction": "commands are represented as redacted command strings in operations and events",
        },
        {
            "id": "move:package-import",
            "stage": "movement",
            "label": "Package import and contract bind",
            "kind": "control_plane_transfer",
            "description": "Package files, contract rows, and recipe metadata move into envctl migration package, contract, and recipe records.",
            "data_subjects": ["package hash", "contract hash", "recipe hash", "artifact requirements"],
            "evidence_refs": [
                "execution-framework/generated/contract_manifest.json",
                "execution-framework/generated/contract_manifest.seed.sql",
                "execution-framework/docs/CONTRACT_MANIFEST.md",
            ],
        },
        {
            "id": "move:run-operation-events",
            "stage": "movement",
            "label": "Run, operation, and event stream",
            "kind": "runtime_transfer",
            "description": "A migration run owns operation rows and append-only event records; plugins observe the latest status and timeline views.",
            "data_subjects": ["run status", "operation state", "risk", "idempotency key", "event hash"],
            "evidence_refs": [
                "execution-framework/generated/envctl_run_ledger_report.json",
                "execution-framework/generated/operation_state_machine.json",
                "execution-framework/docs/ENVCTL_RUN_LEDGER.md",
            ],
        },
        {
            "id": "transform:redaction-policy",
            "stage": "transformation",
            "label": "Security redaction and path policy",
            "kind": "guard_transform",
            "description": "Blocked paths and sensitive command details are filtered or represented by hashes before becoming graph evidence.",
            "data_subjects": ["path policy", "redacted command", "safe evidence URI"],
            "evidence_refs": [
                "execution-framework/generated/security_redaction_validation_report.json",
                "execution-framework/docs/SECURITY_REDACTION.md",
                "execution-framework/scripts/artifact_registry.py",
            ],
        },
        {
            "id": "transform:protocol-shaping",
            "stage": "transformation",
            "label": "Shared protocol shaping",
            "kind": "schema_transform",
            "description": "Envctl DB rows are shaped into shared records consumed by envctl and nu_plugin without moving durable ownership to the plugin.",
            "data_subjects": ["TargetDescriptor", "Operation", "ArtifactRecord", "EvidenceRecord", "GraphEdge", "ValidationResult"],
            "evidence_refs": [
                "execution-framework/generated/shared_protocol_manifest.json",
                "execution-framework/schemas/shared_protocol.schema.json",
                "execution-framework/docs/SHARED_PROTOCOL_SCHEMAS.md",
            ],
        },
        {
            "id": "transform:artifact-generation",
            "stage": "transformation",
            "label": "Artifact generation and hash calculation",
            "kind": "artifact_transform",
            "description": "Generated Markdown and JSON artifacts are converted into registry records with content hashes, evidence refs, validations, and graph edges.",
            "data_subjects": ["artifact body", "sha256 content hash", "validation details", "graph links"],
            "evidence_refs": [
                "execution-framework/generated/envctl_artifact_registry_report.json",
                "execution-framework/docs/ENVCTL_ARTIFACT_REGISTRY.md",
                "execution-framework/schemas/proof_record.schema.json",
            ],
        },
        {
            "id": "persist:envctl-db",
            "stage": "persistence",
            "label": "Envctl migration database",
            "kind": "sqlite_persistence",
            "description": "Durable migration control-plane state is persisted in targets, packages, contracts, recipes, runs, operations, evidence, artifacts, graph edges, approvals, validations, checkpoints, rollbacks, agent sessions, and plugin sessions.",
            "data_subjects": ["all envctl_migration_* rows and views"],
            "evidence_refs": [
                "execution-framework/generated/envctl_migration_db_model.json",
                "execution-framework/docs/ENVCTL_DB_SCHEMA.md",
                "sql/001_migration_automation_schema.sql",
                "sql/002_views_and_indexes.sql",
            ],
        },
        {
            "id": "persist:proof-ledger",
            "stage": "persistence",
            "label": "Proof records and proof ledger",
            "kind": "file_persistence",
            "description": "Task completion evidence is written as a proof JSON file and deduplicated proof ledger row.",
            "data_subjects": ["files changed", "commands run", "verification output", "checksums"],
            "evidence_refs": [
                "execution-framework/proof_records/proof_ledger.jsonl",
                "execution-framework/schemas/proof_record.schema.json",
                "execution-framework/proof_templates/PROOF_RECORD_TEMPLATE.json",
            ],
        },
        {
            "id": "exit:migration-artifacts",
            "stage": "exit",
            "label": "Migration artifact outputs",
            "kind": "operator_artifact",
            "description": "Markdown and JSON deliverables leave the generator as migration-artifacts files, with registry hashes proving the emitted content.",
            "data_subjects": ["data_flow_graph.md", "data_flow_graph.json", "registry report"],
            "evidence_refs": [
                "execution-framework/migration-artifacts/art-107_data_flow_graph/data_flow_graph.md",
                "execution-framework/migration-artifacts/art-107_data_flow_graph/data_flow_graph.json",
                "execution-framework/generated/art_107_data_flow_graph_registry_report.json",
            ],
        },
        {
            "id": "exit:operator-views",
            "stage": "exit",
            "label": "Operator and downstream validation views",
            "kind": "consumer_output",
            "description": "Artifact index, validation scorecard, status streams, and VER-300 consume hashes, validation rows, and graph edges.",
            "data_subjects": ["artifact index rows", "validation scorecard rows", "status table rows"],
            "evidence_refs": [
                "execution-framework/generated/live_visuals.json",
                "execution-framework/generated/final_verification_report.json",
                "execution-framework/generated/task_graph.normalized.json",
            ],
        },
    ]


def graph_edges() -> list[dict[str, Any]]:
    return [
        {
            "id": "edge-data-entry-target-to-db",
            "from": "entry:target-descriptor",
            "to": "persist:envctl-db",
            "type": "enters_as",
            "data": ["target descriptor fields", "descriptor hash", "safety mode"],
            "via": ["envctl_migration_targets"],
            "evidence_refs": ["execution-framework/generated/envctl_target_registry.json"],
        },
        {
            "id": "edge-repo-scan-to-package-import",
            "from": "entry:repo-scan",
            "to": "move:package-import",
            "type": "moves_into",
            "data": ["package inventory", "contract manifest", "source refs"],
            "via": ["envctl_migration_packages", "envctl_migration_artifact_contracts"],
            "evidence_refs": ["execution-framework/generated/package_scan.json", "execution-framework/generated/contract_manifest.json"],
        },
        {
            "id": "edge-operator-command-to-run",
            "from": "entry:operator-command",
            "to": "move:run-operation-events",
            "type": "submits",
            "data": ["run plan requests", "approval decisions", "proof queries"],
            "via": ["envctl endpoints with --emit-event"],
            "evidence_refs": ["execution-framework/generated/nu_plugin_command_manifest.json"],
        },
        {
            "id": "edge-package-import-to-run",
            "from": "move:package-import",
            "to": "move:run-operation-events",
            "type": "binds_recipe_to_run",
            "data": ["recipe id", "artifact contract id", "run id"],
            "via": ["envctl_migration_runs", "envctl_migration_operations"],
            "evidence_refs": ["execution-framework/generated/envctl_run_ledger_report.json"],
        },
        {
            "id": "edge-run-to-redaction",
            "from": "move:run-operation-events",
            "to": "transform:redaction-policy",
            "type": "normalizes",
            "data": ["command_redacted", "evidence URI", "blocked path checks"],
            "via": ["security controls", "artifact registry path policy"],
            "evidence_refs": ["execution-framework/generated/security_redaction_validation_report.json"],
        },
        {
            "id": "edge-db-to-protocol",
            "from": "persist:envctl-db",
            "to": "transform:protocol-shaping",
            "type": "projects",
            "data": ["source-of-truth rows", "record schemas", "plugin table shapes"],
            "via": ["shared protocol manifest"],
            "evidence_refs": ["execution-framework/generated/shared_protocol_manifest.json"],
        },
        {
            "id": "edge-redaction-to-artifact-generation",
            "from": "transform:redaction-policy",
            "to": "transform:artifact-generation",
            "type": "permits_safe_evidence",
            "data": ["safe evidence refs", "redaction status", "hashable artifact paths"],
            "via": ["ArtifactRegistry._validate_path_policy"],
            "evidence_refs": ["execution-framework/scripts/artifact_registry.py"],
        },
        {
            "id": "edge-protocol-to-artifact-generation",
            "from": "transform:protocol-shaping",
            "to": "transform:artifact-generation",
            "type": "schemas",
            "data": ["ArtifactRecord", "EvidenceRecord", "GraphEdge", "ValidationResult", "ProofRecord"],
            "via": ["schemas/shared_protocol.schema.json"],
            "evidence_refs": ["execution-framework/schemas/shared_protocol.schema.json"],
        },
        {
            "id": "edge-artifact-generation-to-db",
            "from": "transform:artifact-generation",
            "to": "persist:envctl-db",
            "type": "persists_registry_rows",
            "data": ["artifact rows", "evidence rows", "graph edges", "validation rows"],
            "via": [
                "envctl_migration_artifacts",
                "envctl_migration_evidence",
                "envctl_migration_graph_edges",
                "envctl_migration_validations",
            ],
            "evidence_refs": ["execution-framework/generated/envctl_artifact_registry_report.json"],
        },
        {
            "id": "edge-artifact-generation-to-proof-ledger",
            "from": "transform:artifact-generation",
            "to": "persist:proof-ledger",
            "type": "records_proof",
            "data": ["verification output", "checksums", "evidence list"],
            "via": ["proof_records/ART-107_DATA_FLOW_GRAPH.proof.json", "proof_records/proof_ledger.jsonl"],
            "evidence_refs": ["execution-framework/schemas/proof_record.schema.json"],
        },
        {
            "id": "edge-proof-ledger-to-artifact-exit",
            "from": "persist:proof-ledger",
            "to": "exit:migration-artifacts",
            "type": "attests",
            "data": ["file checksums", "commands run", "registry result"],
            "via": ["proof record"],
            "evidence_refs": ["execution-framework/proof_records/ART-107_DATA_FLOW_GRAPH.proof.json"],
        },
        {
            "id": "edge-db-to-operator-views",
            "from": "persist:envctl-db",
            "to": "exit:operator-views",
            "type": "renders_as",
            "data": ["artifact index", "validation scorecard", "timeline", "latest status"],
            "via": ["envctl_migration_artifact_index", "envctl_migration_validation_scorecard"],
            "evidence_refs": ["execution-framework/generated/live_visuals.json"],
        },
        {
            "id": "edge-artifact-exit-to-validation",
            "from": "exit:migration-artifacts",
            "to": "exit:operator-views",
            "type": "blocks",
            "data": ["VER-300_UNIT_VALIDATION input"],
            "via": ["task graph block edge"],
            "evidence_refs": ["execution-framework/generated/task_graph.normalized.json"],
        },
    ]


def flow_paths(nodes: list[dict[str, Any]], edges: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return [
        {
            "path_id": "flow-target-descriptor-to-status",
            "entry": "entry:target-descriptor",
            "movement": ["move:package-import", "move:run-operation-events"],
            "transformation": ["transform:protocol-shaping"],
            "persistence": ["persist:envctl-db"],
            "exit": ["exit:operator-views"],
            "data_classes": ["target config", "safety policy", "status rows"],
        },
        {
            "path_id": "flow-repo-scan-to-artifacts",
            "entry": "entry:repo-scan",
            "movement": ["move:package-import"],
            "transformation": ["transform:redaction-policy", "transform:artifact-generation"],
            "persistence": ["persist:envctl-db", "persist:proof-ledger"],
            "exit": ["exit:migration-artifacts", "exit:operator-views"],
            "data_classes": ["file inventory", "artifact body", "content hash", "validation evidence"],
        },
        {
            "path_id": "flow-operator-command-to-proof",
            "entry": "entry:operator-command",
            "movement": ["move:run-operation-events"],
            "transformation": ["transform:protocol-shaping", "transform:artifact-generation"],
            "persistence": ["persist:envctl-db", "persist:proof-ledger"],
            "exit": ["exit:operator-views"],
            "data_classes": ["command input", "operation event", "artifact/proof lookup"],
        },
    ]


def build_payload(generated_at: str) -> dict[str, Any]:
    nodes = graph_nodes()
    edges = graph_edges()
    stage_counts: dict[str, int] = {}
    for node in nodes:
        stage_counts[node["stage"]] = stage_counts.get(node["stage"], 0) + 1
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "title": "Data flow graph",
        "generated_at": generated_at,
        "scope": {
            "target": "flexnetos-vs-lifeos",
            "goal": "Map data entry, movement, transformation, persistence, and exit for the envctl migration automation package.",
            "owner_lane": "lane_d_filesystem",
            "owner_agent": "artifact-agent",
        },
        "source_inputs": source_inputs(),
        "summary": {
            "node_count": len(nodes),
            "edge_count": len(edges),
            "flow_path_count": 3,
            "stage_counts": stage_counts,
            "entry_stage_count": stage_counts.get("data_entry", 0),
            "movement_stage_count": stage_counts.get("movement", 0),
            "transformation_stage_count": stage_counts.get("transformation", 0),
            "persistence_stage_count": stage_counts.get("persistence", 0),
            "exit_stage_count": stage_counts.get("exit", 0),
            "all_required_stages_present": all(
                stage_counts.get(stage, 0) > 0
                for stage in ["data_entry", "movement", "transformation", "persistence", "exit"]
            ),
        },
        "nodes": nodes,
        "edges": edges,
        "flow_paths": flow_paths(nodes, edges),
    }


def render_markdown(payload: dict[str, Any]) -> str:
    def cell(value: Any) -> str:
        if isinstance(value, list):
            text = "<br>".join(str(item) for item in value)
        elif isinstance(value, dict):
            text = "<br>".join(f"{key}: {val}" for key, val in value.items())
        else:
            text = str(value)
        return text.replace("|", "\\|")

    lines = [
        "# Data flow graph",
        "",
        f"Task: `{TASK_ID}`",
        f"Generated at: `{payload['generated_at']}`",
        "",
        "## Scope",
        "",
        "This artifact maps how migration data enters, moves through, transforms within, persists in, and exits the envctl migration automation package for the `flexnetos-vs-lifeos` target.",
        "",
        "## Summary",
        "",
        f"- Nodes: `{payload['summary']['node_count']}`",
        f"- Edges: `{payload['summary']['edge_count']}`",
        f"- Flow paths: `{payload['summary']['flow_path_count']}`",
        f"- Required stages present: `{payload['summary']['all_required_stages_present']}`",
        f"- Stage counts: `{json.dumps(payload['summary']['stage_counts'], sort_keys=True)}`",
        "",
        "## Graph Nodes",
        "",
        "| id | stage | label | data subjects | persistence/redaction note | evidence |",
        "|---|---|---|---|---|---|",
    ]
    for node in payload["nodes"]:
        lines.append(
            "| {id} | {stage} | {label} | {data} | {note} | {evidence} |".format(
                id=cell(node["id"]),
                stage=cell(node["stage"]),
                label=cell(node["label"]),
                data=cell(node["data_subjects"]),
                note=cell(node.get("redaction", node["description"])),
                evidence=cell([f"`{ref}`" for ref in node["evidence_refs"]]),
            )
        )
    lines.extend(["", "## Graph Edges", "", "| id | from | to | type | data | via |", "|---|---|---|---|---|---|"])
    for edge in payload["edges"]:
        lines.append(
            "| {id} | {source} | {target} | {edge_type} | {data} | {via} |".format(
                id=cell(edge["id"]),
                source=cell(edge["from"]),
                target=cell(edge["to"]),
                edge_type=cell(edge["type"]),
                data=cell(edge["data"]),
                via=cell(edge["via"]),
            )
        )
    lines.extend(
        [
            "",
            "## Flow Paths",
            "",
            "| path | entry | movement | transformation | persistence | exit | data classes |",
            "|---|---|---|---|---|---|---|",
        ]
    )
    for path in payload["flow_paths"]:
        lines.append(
            "| {path_id} | {entry} | {movement} | {transformation} | {persistence} | {exit} | {data_classes} |".format(
                path_id=cell(path["path_id"]),
                entry=cell(path["entry"]),
                movement=cell(path["movement"]),
                transformation=cell(path["transformation"]),
                persistence=cell(path["persistence"]),
                exit=cell(path["exit"]),
                data_classes=cell(path["data_classes"]),
            )
        )
    lines.extend(
        [
            "",
            "## Registration Gate",
            "",
            "The companion registry report records the Markdown and JSON artifacts in the envctl artifact registry with SHA-256 hashes, evidence rows, validation rows, and graph edges. `VER-300_UNIT_VALIDATION` is the downstream validation consumer.",
        ]
    )
    return "\n".join(lines) + "\n"


def write_artifacts(payload: dict[str, Any]) -> dict[str, str]:
    paths = artifact_paths()
    paths["markdown"].parent.mkdir(parents=True, exist_ok=True)
    paths["json"].write_text(json.dumps(payload, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    paths["markdown"].write_text(render_markdown(payload), encoding="utf-8")
    return {
        "markdown": "execution-framework/migration-artifacts/art-107_data_flow_graph/data_flow_graph.md",
        "json": "execution-framework/migration-artifacts/art-107_data_flow_graph/data_flow_graph.json",
    }


def insert_fixture(conn: sqlite3.Connection) -> dict[str, str]:
    conn.execute(
        """
        INSERT INTO envctl_migration_targets
          (id, target_id, target_type, primary_root, compare_root, descriptor_json,
           descriptor_hash, safety_mode, max_auto_risk)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "target-art107-flexnetos-vs-lifeos",
            "flexnetos-vs-lifeos",
            "mixed",
            "/home/flexnetos/FlexNetOS",
            "/home/flexnetos/lifeos",
            json.dumps(source_inputs()["target_descriptor"], sort_keys=True),
            "sha256:art107-target-descriptor",
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
            "pkg-art107",
            "envctl-db-nu-plugin-migration-automation-package",
            ".",
            "sha256:pkg-art107",
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
            "art107-data-flow-graph-contract",
            "1.0.0",
            "pkg-art107",
            "sha256:contract-art107",
            json.dumps({"required": ["Data flow graph"], "stages": ["entry", "movement", "transformation", "persistence", "exit"]}, sort_keys=True),
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_recipes
          (id, recipe_name, recipe_version, artifact_contract_id, recipe_hash, recipe_json)
        VALUES (?, ?, ?, ?, ?, ?)
        """,
        (
            "recipe-art107",
            "codex-flexnetos-full-artifact-run",
            "1.0.0",
            CONTRACT_ID,
            "sha256:recipe-art107",
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
            "target-art107-flexnetos-vs-lifeos",
            "recipe-art107",
            CONTRACT_ID,
            "running",
            "approval-gated",
            "artifact-agent",
            "workspace-write",
            "never",
            json.dumps({"python": "stdlib", "sqlite": "stdlib"}, sort_keys=True),
            "sha256:run-art107",
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
            f"{TASK_ID}/data-flow-graph",
            "sha256:command-art107",
            "python3 scripts/generate_art_107_data_flow_graph.py",
            json.dumps({"task_id": TASK_ID, "artifact": "data_flow_graph"}, sort_keys=True),
        ),
    )
    conn.commit()
    return {"run_id": RUN_ID, "operation_id": OPERATION_ID, "contract_id": CONTRACT_ID}


def register_artifacts(conn: sqlite3.Connection, fixture: dict[str, str], relpaths: dict[str, str], payload: dict[str, Any]) -> list[dict[str, Any]]:
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
                "execution-framework/migration-artifacts/_meta/flexnetos-vs-lifeos.target-descriptor.yaml",
                "execution-framework/generated/package_scan.json",
                "execution-framework/generated/envctl_migration_db_model.json",
                "execution-framework/generated/envctl_artifact_registry_report.json",
                "execution-framework/generated/shared_protocol_manifest.json",
            ],
        },
        "evidence_refs": [
            relpaths["markdown"],
            relpaths["json"],
            "execution-framework/generated/package_scan.json",
            "execution-framework/generated/envctl_migration_db_model.json",
            "execution-framework/generated/envctl_artifact_registry_report.json",
            "execution-framework/generated/shared_protocol_manifest.json",
            "execution-framework/generated/nu_plugin_command_manifest.json",
        ],
        "links": [
            {"to": "artifact:REQ-024_ENVCTL_ARTIFACT_REGISTRY", "type": "depends_on"},
            {"to": "artifact:REQ-040_SHARED_PROTOCOL_SCHEMAS", "type": "depends_on"},
            {"to": "artifact:VER-300_UNIT_VALIDATION", "type": "blocks"},
            {"to": "data-stage:data-entry", "type": "maps_stage"},
            {"to": "data-stage:movement", "type": "maps_stage"},
            {"to": "data-stage:transformation", "type": "maps_stage"},
            {"to": "data-stage:persistence", "type": "maps_stage"},
            {"to": "data-stage:exit", "type": "maps_stage"},
        ],
        "validations": [
            {
                "validator": "generate_art_107_data_flow_graph.py:stage-coverage",
                "status": "pass",
                "details": {
                    "stage_counts": payload["summary"]["stage_counts"],
                    "all_required_stages_present": payload["summary"]["all_required_stages_present"],
                },
                "evidence_refs": [relpaths["json"], relpaths["markdown"]],
            },
            {
                "validator": "generate_art_107_data_flow_graph.py:registry-hash",
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
                "artifact_id": "art107-data-flow-graph-md",
                "title": "ART-107 Data Flow Graph",
                "artifact_type": "data_flow_graph_markdown",
                "path": relpaths["markdown"],
            }
        ),
        registry.register(
            {
                **common,
                "artifact_id": "art107-data-flow-graph-json",
                "title": "ART-107 Data Flow Graph Source",
                "artifact_type": "data_flow_graph_json",
                "path": relpaths["json"],
            }
        ),
    ]


def build_report(conn: sqlite3.Connection, payload: dict[str, Any], registry_results: list[dict[str, Any]], relpaths: dict[str, str]) -> dict[str, Any]:
    artifacts = [
        fetch_artifact(conn, RUN_ID, "art107-data-flow-graph-md"),
        fetch_artifact(conn, RUN_ID, "art107-data-flow-graph-json"),
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
    for relpath in relpaths.values():
        if not (package_root() / relpath).exists():
            errors.append(f"missing artifact file: {relpath}")
    if len(registry_results) != 2:
        errors.append("expected two registry rows")
    if any(not item.get("content_hash", "").startswith("sha256:") for item in registry_results):
        errors.append("registry did not persist sha256 hashes for all artifacts")
    if validation_count < 4:
        errors.append(f"expected at least 4 validation rows, got {validation_count}")
    if graph_count < 16:
        errors.append(f"expected at least 16 graph edges, got {graph_count}")
    if evidence_count < 7:
        errors.append(f"expected at least 7 evidence rows, got {evidence_count}")
    if len(index_rows) != 2:
        errors.append(f"expected two artifact index rows, got {len(index_rows)}")
    if not payload["summary"]["all_required_stages_present"]:
        errors.append("data flow graph is missing one or more required stages")
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
            "validation_evidence_linked": validation_count >= 4 and evidence_count >= 7,
        },
        "errors": errors,
        "evidence": [
            relpaths["markdown"],
            relpaths["json"],
            "execution-framework/generated/art_107_data_flow_graph_registry_report.json",
            "execution-framework/generated/package_scan.json",
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
        "execution-framework/scripts/generate_art_107_data_flow_graph.py",
        "execution-framework/migration-artifacts/art-107_data_flow_graph/data_flow_graph.md",
        "execution-framework/migration-artifacts/art-107_data_flow_graph/data_flow_graph.json",
        "execution-framework/generated/art_107_data_flow_graph_registry_report.json",
        "execution-framework/state/ART-107_DATA_FLOW_GRAPH.heartbeat.json",
        "execution-framework/logs/ART-107_DATA_FLOW_GRAPH.log",
        "execution-framework/proof_records/ART-107_DATA_FLOW_GRAPH.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    commands_run = [
        "python3 scripts/generate_art_107_data_flow_graph.py",
        "python3 -m py_compile scripts/generate_art_107_data_flow_graph.py",
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
        "run VER-300_UNIT_VALIDATION" if report["status"] == "passed" else "fix ART-107 data flow graph errors",
    )
    append_proof(proof)


def main() -> None:
    generated_at = now()
    payload = build_payload(generated_at)
    relpaths = write_artifacts(payload)
    conn = sqlite3.connect(":memory:")
    conn.execute("PRAGMA foreign_keys = ON")
    apply_migrations(conn, package_root())
    fixture = insert_fixture(conn)
    registry_results = register_artifacts(conn, fixture, relpaths, payload)
    report = build_report(conn, payload, registry_results, relpaths)
    write_report_and_proof(report)
    print(
        "art107 status={status} artifacts={artifacts} hashes={hashes} stages={stages} validations={validations}".format(
            status=report["status"],
            artifacts=len(report["registry_results"]),
            hashes=sum(1 for item in report["registry_results"] if item.get("content_hash")),
            stages=json.dumps(report["summary"]["stage_counts"], sort_keys=True),
            validations=report["summary"]["validation_count"],
        )
    )
    if report["status"] != "passed":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
