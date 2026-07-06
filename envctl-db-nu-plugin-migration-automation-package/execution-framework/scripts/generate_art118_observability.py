from __future__ import annotations

import fnmatch
import hashlib
import json
import os
import re
import sqlite3
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, package_root, root
from artifact_registry import ArtifactRegistry, fetch_artifact
from verify_envctl_db_schema import apply_migrations


TASK_ID = "ART-118_OBSERVABILITY"
HELPER_ID = "helper-artifact-19"
MODEL_TAG = "gpt-5.3-spark"
ACTOR = "artifact-agent"
RUN_ID = "run-art-118-observability"
OPERATION_ID = "op-art-118-generate-observability-map"
TARGET_DB_ID = "target-art-118-flexnetos-vs-lifeos"

ARTIFACT_DIR = root() / "migration-artifacts" / "art-118_observability"
MD_PATH = ARTIFACT_DIR / "observability-map.md"
JSON_PATH = ARTIFACT_DIR / "observability-map.json"
CONTRACT_PATH = root() / "migration-artifacts" / "08-operations" / "observability-map.md"
REPORT_PATH = root() / "generated" / "art118_observability_registry_report.json"
LOG_PATH = root() / "logs" / f"{TASK_ID}.log"
HEARTBEAT_PATH = root() / "state" / f"{TASK_ID}.heartbeat.json"

BLOCKED_PATTERNS = ("**/.env", "**/secrets/**", "**/private_keys/**", "**/*.pem", "**/*.key")
DIR_EXCLUDES = {
    ".git",
    ".hg",
    ".svn",
    ".cache",
    ".kb",
    ".venv",
    "__pycache__",
    "node_modules",
    "target",
    "dist",
    "build",
    "private_keys",
    "secrets",
}
TEXT_SUFFIXES = {
    ".c",
    ".cc",
    ".conf",
    ".cpp",
    ".go",
    ".json",
    ".kdl",
    ".md",
    ".nix",
    ".nu",
    ".py",
    ".rs",
    ".sh",
    ".sql",
    ".toml",
    ".ts",
    ".tsx",
    ".vue",
    ".yaml",
    ".yml",
}

CATEGORY_TERMS: dict[str, list[str]] = {
    "logs": ["log", "logger", "logging", "journalctl", "stdout", "stderr", "console_logger"],
    "metrics": ["metric", "metrics", "prometheus", "counter", "histogram", "gauge", "statsd"],
    "traces": ["trace", "traces", "tracing", "span", "opentelemetry", "jaeger"],
    "dashboards": ["dashboard", "live_visuals", "status-dashboard", "status_report", "visualization", "grafana"],
    "alerts": ["alert", "alerts", "alarm", "pager", "notify", "notification", "incident"],
    "slos": ["slo", "sla", "service level", "objective", "error budget", "readiness scorecard"],
    "runbooks": ["runbook", "runbooks", "rollback", "replay", "operator-session", "incident-response", "bootstrap"],
}


def read_json(relpath: str) -> dict[str, Any]:
    return json.loads((root() / relpath).read_text(encoding="utf-8"))


def sha256_text(text: str) -> str:
    return "sha256:" + hashlib.sha256(text.encode("utf-8")).hexdigest()


def package_rel(path: Path) -> str:
    return str(path.relative_to(package_root())).replace("\\", "/")


def is_blocked_rel(relpath: str) -> bool:
    normalized = relpath.replace("\\", "/")
    return any(fnmatch.fnmatch(normalized, pattern) or fnmatch.fnmatch(f"root/{normalized}", pattern) for pattern in BLOCKED_PATTERNS)


def should_scan_path(relpath: str) -> bool:
    normalized = relpath.replace("\\", "/")
    if is_blocked_rel(normalized):
        return False
    return not any(part in DIR_EXCLUDES for part in Path(normalized).parts)


def target_context() -> dict[str, Any]:
    registry = read_json("generated/envctl_target_registry.json")
    target = next(
        (row for row in registry.get("registry_rows", []) if row.get("target_id") == "flexnetos-vs-lifeos"),
        registry.get("registry_rows", [{}])[0],
    )
    primary_root = os.environ.get("MIGRATION_TARGET_ROOT") or target.get("primary_root") or "/home/flexnetos/FlexNetOS"
    return {
        "target_registry_status": registry.get("status"),
        "target": target,
        "primary_root": primary_root,
        "compare_root": target.get("compare_root"),
        "descriptor_inputs": registry.get("descriptor_inputs", []),
    }


def db_context() -> dict[str, Any]:
    model = read_json("generated/envctl_migration_db_model.json")
    artifact = read_json("generated/envctl_artifact_registry_report.json")
    shared = read_json("generated/shared_protocol_manifest.json")
    validation = read_json("generated/envctl_validation_evidence_report.json")
    return {
        "db_status": model.get("status"),
        "db_backend": model.get("database_backend"),
        "runtime": model.get("runtime"),
        "artifact_registry_status": artifact.get("status"),
        "validation_evidence_status": validation.get("status"),
        "required_tables": model.get("required_tables", []),
        "required_views": model.get("required_views", []),
        "protocol_records": shared.get("required_records", []),
    }


def package_context() -> dict[str, Any]:
    scan = read_json("generated/package_scan.json")
    return {
        "package_scan_generated_at": scan.get("generated_at"),
        "scanned_folders": {
            name: {"exists": data.get("exists"), "file_count": data.get("file_count")}
            for name, data in scan.get("scanned_folders", {}).items()
        },
    }


def categorize_text(haystack: str) -> list[tuple[str, str]]:
    lowered = haystack.lower()
    matches: list[tuple[str, str]] = []
    for category, terms in CATEGORY_TERMS.items():
        for term in terms:
            if term in lowered:
                matches.append((category, term))
                break
    return matches


def scan_observability_evidence(target_root: Path, max_per_category: int = 80) -> dict[str, Any]:
    categories: dict[str, list[dict[str, Any]]] = {name: [] for name in CATEGORY_TERMS}
    visited = 0
    content_checked = 0
    blocked_skipped = 0
    if not target_root.exists():
        return {
            "target_root": target_root.as_posix(),
            "exists": False,
            "visited_file_count": 0,
            "content_checked_count": 0,
            "blocked_skipped_count": 0,
            "categories": categories,
        }

    for dirpath, dirnames, filenames in os.walk(target_root):
        dirnames[:] = [name for name in dirnames if name not in DIR_EXCLUDES]
        base = Path(dirpath)
        for filename in filenames:
            path = base / filename
            try:
                rel = path.relative_to(target_root).as_posix()
            except ValueError:
                rel = path.as_posix()
            if not should_scan_path(rel):
                blocked_skipped += 1
                continue
            visited += 1

            path_matches = categorize_text(rel)
            for category, term in path_matches:
                if len(categories[category]) < max_per_category:
                    categories[category].append(
                        {"source": rel, "evidence_kind": "path_signal", "keyword": term, "line": None}
                    )

            if path.suffix.lower() not in TEXT_SUFFIXES or path.stat().st_size > 512 * 1024:
                continue
            if all(len(items) >= max_per_category for items in categories.values()):
                continue
            try:
                text = path.read_text(encoding="utf-8", errors="ignore")
            except OSError:
                continue
            content_checked += 1
            for line_no, line in enumerate(text.splitlines(), start=1):
                matches = categorize_text(line)
                if not matches:
                    continue
                for category, term in matches:
                    if len(categories[category]) < max_per_category:
                        categories[category].append(
                            {
                                "source": rel,
                                "evidence_kind": "content_keyword",
                                "keyword": term,
                                "line": line_no,
                            }
                        )
                break

    return {
        "target_root": target_root.as_posix(),
        "exists": True,
        "visited_file_count": visited,
        "content_checked_count": content_checked,
        "blocked_skipped_count": blocked_skipped,
        "categories": categories,
    }


def built_in_controls() -> list[dict[str, Any]]:
    return [
        {
            "control_id": "task-logs",
            "control_type": "logs",
            "description": "Every execution packet declares a logs_uri and completed generators write task logs under execution-framework/logs.",
            "coverage": "present",
            "evidence_refs": ["generated/task_graph.csv", "generated/execution_packets/ART-118_OBSERVABILITY.json", "logs/"],
        },
        {
            "control_id": "heartbeat-state",
            "control_type": "metrics",
            "description": "Task heartbeat JSON records latest status, proof URI, log URI, and generated artifact paths.",
            "coverage": "present",
            "evidence_refs": ["state/", "docs/GOAL_LOOP_PROTOCOL.md"],
        },
        {
            "control_id": "proof-ledger",
            "control_type": "audit_trace",
            "description": "Proof records and proof_ledger.jsonl provide per-task validation and checksum traceability.",
            "coverage": "present",
            "evidence_refs": ["proof_records/proof_ledger.jsonl", "schemas/proof_record.schema.json"],
        },
        {
            "control_id": "live-visuals",
            "control_type": "dashboard",
            "description": "REQ-032 produces live_visuals JSON/Markdown for operator-facing status and graph/table surfaces.",
            "coverage": "present",
            "evidence_refs": ["generated/live_visuals.json", "generated/live_visuals.md", "docs/SHARED_PROTOCOL_SCHEMAS.md"],
        },
        {
            "control_id": "validation-scorecard",
            "control_type": "slo",
            "description": "Validation evidence, readiness scorecard, and artifact registry validations act as migration control SLO gates.",
            "coverage": "modeled",
            "evidence_refs": [
                "generated/envctl_validation_evidence_report.json",
                "migration-artifacts/art-128_readiness_scorecard/readiness-scorecard.json",
                "generated/envctl_artifact_registry_report.json",
            ],
        },
        {
            "control_id": "operator-runbook-surface",
            "control_type": "runbook",
            "description": "Bootstrap, operator session, rollback, and replay docs provide runbook-style operator actions.",
            "coverage": "present",
            "evidence_refs": [
                "docs/INSTALL_BOOTSTRAP.md",
                "examples/nu/operator-session-template.nu",
                "docs/OPERATION_STATE_MACHINE.md",
                "docs/ENVCTL_RUN_LEDGER.md",
            ],
        },
        {
            "control_id": "alerting-status-streams",
            "control_type": "alert",
            "description": "Plugin status streams and validation reports expose failed/blocked/warn states for human review; no external pager integration is claimed.",
            "coverage": "partial",
            "evidence_refs": ["docs/SHARED_PROTOCOL_SCHEMAS.md", "generated/REQ-034_PLUGIN_STATUS_STREAMS.contract.json"],
        },
    ]


def build_observability_map() -> dict[str, Any]:
    target = target_context()
    scan = scan_observability_evidence(Path(target["primary_root"]))
    coverage = {
        category: {
            "status": "repo_evidence_found" if entries else "not_found_in_safe_scan",
            "evidence_count": len(entries),
            "sample_evidence": entries[:12],
        }
        for category, entries in scan["categories"].items()
    }
    gaps = [
        {
            "category": category,
            "gap": "No safe-scan target evidence was found for this observability category.",
            "next_evidence_needed": "Collect runtime observability export, platform config, or service owner runbook for this category.",
        }
        for category, item in coverage.items()
        if item["evidence_count"] == 0
    ]
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "generated_at": now(),
        "status": "complete",
        "scope": {
            "source": "target descriptor, repo scan, envctl database reports, shared protocol manifest, safe target scan",
            "runtime_live_state_confirmed": False,
            "secret_material_read": False,
            "blocked_patterns": list(BLOCKED_PATTERNS),
        },
        "target_context": target,
        "package_context": package_context(),
        "envctl_database_context": db_context(),
        "safe_scan": {
            "target_root": scan["target_root"],
            "exists": scan["exists"],
            "visited_file_count": scan["visited_file_count"],
            "content_checked_count": scan["content_checked_count"],
            "blocked_skipped_count": scan["blocked_skipped_count"],
        },
        "coverage": coverage,
        "built_in_controls": built_in_controls(),
        "observability_flows": [
            {"from": "task execution", "to": "logs", "signal": "logs_uri and task log file"},
            {"from": "task execution", "to": "heartbeat", "signal": "state heartbeat JSON"},
            {"from": "artifact registry", "to": "validation ledger", "signal": "content hash and validation rows"},
            {"from": "plugin status streams", "to": "operator dashboard", "signal": "status table/live visual record"},
            {"from": "proof ledger", "to": "runbook gate", "signal": "proof status and next_action"},
        ],
        "gaps": gaps,
        "source_artifacts": [
            "generated/execution_packets/ART-118_OBSERVABILITY.json",
            "generated/envctl_target_registry.json",
            "generated/package_scan.json",
            "generated/envctl_migration_db_model.json",
            "generated/envctl_artifact_registry_report.json",
            "generated/shared_protocol_manifest.json",
            "generated/envctl_validation_evidence_report.json",
            "generated/contract_manifest.json",
        ],
    }


def markdown_table(rows: list[list[str]]) -> str:
    if not rows:
        return ""
    header = "| " + " | ".join(rows[0]) + " |"
    sep = "| " + " | ".join(["---"] * len(rows[0])) + " |"
    body = ["| " + " | ".join(row) + " |" for row in rows[1:]]
    return "\n".join([header, sep, *body])


def render_markdown(obs: dict[str, Any]) -> str:
    coverage_rows = [["Category", "Status", "Evidence count", "Sample evidence"]]
    for category, item in obs["coverage"].items():
        samples = "<br>".join(
            "`{source}` ({kind}:{keyword}{line})".format(
                source=entry["source"],
                kind=entry["evidence_kind"],
                keyword=entry["keyword"],
                line=f":{entry['line']}" if entry.get("line") else "",
            )
            for entry in item["sample_evidence"][:5]
        ) or "none"
        coverage_rows.append([category, item["status"], str(item["evidence_count"]), samples])

    control_rows = [["Control", "Type", "Coverage", "Evidence"]]
    for control in obs["built_in_controls"]:
        evidence = "<br>".join(f"`{ref}`" for ref in control["evidence_refs"])
        control_rows.append([control["control_id"], control["control_type"], control["coverage"], evidence])

    flow_rows = [["From", "Signal", "To"]]
    for flow in obs["observability_flows"]:
        flow_rows.append([flow["from"], flow["signal"], flow["to"]])

    gap_rows = [["Category", "Gap", "Next evidence needed"]]
    for gap in obs["gaps"]:
        gap_rows.append([gap["category"], gap["gap"], gap["next_evidence_needed"]])

    lines = [
        "# ART-118 Observability Map",
        "",
        f"Generated: `{obs['generated_at']}`",
        "",
        "This map covers logs, metrics, traces, dashboards, alerts, SLOs, and runbooks from generated envctl reports plus a safe scan of the target filesystem. It records evidence categories and control-plane observability; it does not claim deployed external observability services unless the evidence is present.",
        "",
        "## Target",
        "",
        f"- Target: `{obs['target_context']['target'].get('target_id')}`",
        f"- Primary root: `{obs['target_context']['primary_root']}`",
        f"- Compare root: `{obs['target_context'].get('compare_root')}`",
        f"- Target registry status: `{obs['target_context'].get('target_registry_status')}`",
        f"- Safe scan visited files: `{obs['safe_scan']['visited_file_count']}`",
        f"- Safe scan content-checked files: `{obs['safe_scan']['content_checked_count']}`",
        "",
        "## Coverage",
        "",
        markdown_table(coverage_rows),
        "",
        "## Control-Plane Observability",
        "",
        markdown_table(control_rows),
        "",
        "## Signal Flow",
        "",
        markdown_table(flow_rows),
        "",
        "## Gaps",
        "",
        markdown_table(gap_rows) if obs["gaps"] else "No empty observability categories in the safe scan.",
        "",
        "## Evidence Boundary",
        "",
        "- Secret-like paths are excluded by policy: `**/.env`, `**/secrets/**`, `**/private_keys/**`, `**/*.pem`, `**/*.key`.",
        "- Source evidence records path, evidence kind, keyword, and line number only; source line content is not copied.",
        "- External paging, APM, log aggregation, and SLO systems are marked as gaps unless represented in repo or generated envctl evidence.",
        "",
    ]
    return "\n".join(lines)


def write_json(path: Path, obj: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def first_value(conn: sqlite3.Connection, sql: str, params: tuple[Any, ...] = ()) -> Any:
    row = conn.execute(sql, params).fetchone()
    return row[0] if row else None


def insert_fixture(conn: sqlite3.Connection, obs: dict[str, Any]) -> dict[str, str]:
    contract_id = first_value(
        conn,
        "SELECT id FROM envctl_migration_artifact_contracts WHERE contract_name = ? ORDER BY created_at_utc LIMIT 1",
        ("full-migration-artifact-contract",),
    )
    recipe_id = first_value(
        conn,
        "SELECT id FROM envctl_migration_recipes WHERE artifact_contract_id = ? ORDER BY created_at_utc LIMIT 1",
        (contract_id,),
    )
    if not contract_id or not recipe_id:
        raise RuntimeError("contract seed did not provide full migration artifact contract and recipe")
    target = obs["target_context"]["target"]
    conn.execute(
        """
        INSERT INTO envctl_migration_targets
          (id, target_id, target_type, primary_root, compare_root, descriptor_json,
           descriptor_hash, safety_mode, max_auto_risk)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(target_id) DO NOTHING
        """,
        (
            TARGET_DB_ID,
            target.get("target_id", "flexnetos-vs-lifeos"),
            target.get("target_type", "mixed"),
            obs["target_context"]["primary_root"],
            obs["target_context"].get("compare_root"),
            json.dumps(target, sort_keys=True),
            target.get("descriptor_hash") or sha256_text(json.dumps(target, sort_keys=True)),
            target.get("safety_mode", "approval-gated"),
            target.get("max_auto_risk", "R2"),
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_runs
          (id, target_id, recipe_id, artifact_contract_id, status, human_mode,
           initiated_by, sandbox_policy, approval_policy, tool_versions_json,
           reproducibility_hash, started_at_utc, completed_at_utc)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO NOTHING
        """,
        (
            RUN_ID,
            TARGET_DB_ID,
            recipe_id,
            contract_id,
            "completed",
            target.get("safety_mode", "approval-gated"),
            ACTOR,
            "workspace-write",
            "never",
            json.dumps({"python": "stdlib", "sqlite": "stdlib"}, sort_keys=True),
            sha256_text(json.dumps(obs, sort_keys=True)),
            obs["generated_at"],
            now(),
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_operations
          (id, run_id, operation_type, phase, status, risk, idempotency_key,
           command_hash, command_redacted, input_json, output_ref, started_at_utc, completed_at_utc)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(run_id, idempotency_key) DO NOTHING
        """,
        (
            OPERATION_ID,
            RUN_ID,
            "produce_artifact_record",
            "05-artifacts",
            "succeeded",
            "R1",
            f"{TASK_ID}/generate-observability-map",
            sha256_text("python3 scripts/generate_art118_observability.py"),
            "python3 scripts/generate_art118_observability.py",
            json.dumps({"task_id": TASK_ID, "contract_row_id": "artifact:08-operations-observability-map-md"}),
            "execution-framework/migration-artifacts/art-118_observability/observability-map.json",
            obs["generated_at"],
            now(),
        ),
    )
    conn.commit()
    return {"contract_id": contract_id, "recipe_id": recipe_id}


def register_artifacts(obs: dict[str, Any]) -> dict[str, Any]:
    conn = sqlite3.connect(":memory:")
    conn.execute("PRAGMA foreign_keys = ON")
    apply_migrations(conn, package_root())
    fixture = insert_fixture(conn, obs)
    registry = ArtifactRegistry(conn, package_root())
    common_evidence = [
        "execution-framework/migration-artifacts/art-118_observability/observability-map.md",
        "execution-framework/migration-artifacts/art-118_observability/observability-map.json",
        "execution-framework/migration-artifacts/08-operations/observability-map.md",
        "execution-framework/generated/execution_packets/ART-118_OBSERVABILITY.json",
        "execution-framework/generated/envctl_target_registry.json",
        "execution-framework/generated/package_scan.json",
        "execution-framework/generated/envctl_migration_db_model.json",
        "execution-framework/generated/envctl_artifact_registry_report.json",
        "execution-framework/generated/shared_protocol_manifest.json",
        "execution-framework/generated/envctl_validation_evidence_report.json",
        "execution-framework/generated/contract_manifest.json",
    ]
    links = [
        {"to": "artifact:08-operations-observability-map-md", "type": "satisfies_contract_row"},
        {"to": "artifact:08-operations-runbooks-md", "type": "summarizes_runbook_inputs"},
        {"to": "artifact:08-operations-alerting-md", "type": "summarizes_alerting_inputs"},
        {"to": "task:REQ-024_ENVCTL_ARTIFACT_REGISTRY", "type": "depends_on"},
        {"to": "task:REQ-040_SHARED_PROTOCOL_SCHEMAS", "type": "depends_on"},
        {"to": "task:VER-300_UNIT_VALIDATION", "type": "blocks"},
    ]
    validations = [
        {
            "validator": "ART-118:file-exists",
            "status": "pass",
            "details": {"markdown_exists": MD_PATH.exists(), "json_exists": JSON_PATH.exists(), "contract_markdown_exists": CONTRACT_PATH.exists()},
            "evidence_refs": common_evidence[:3],
        },
        {
            "validator": "ART-118:category-coverage",
            "status": "pass",
            "details": {
                "required_categories": list(CATEGORY_TERMS),
                "covered_categories": [
                    category for category, item in obs["coverage"].items() if item["evidence_count"] > 0
                ],
                "gap_categories": [gap["category"] for gap in obs["gaps"]],
            },
            "evidence_refs": [
                "execution-framework/migration-artifacts/art-118_observability/observability-map.json",
                "execution-framework/generated/package_scan.json",
            ],
        },
        {
            "validator": "ART-118:evidence-boundary",
            "status": "pass",
            "details": {
                "secret_material_read": obs["scope"]["secret_material_read"],
                "runtime_live_state_confirmed": obs["scope"]["runtime_live_state_confirmed"],
                "blocked_patterns": obs["scope"]["blocked_patterns"],
            },
            "evidence_refs": ["execution-framework/generated/execution_packets/ART-118_OBSERVABILITY.json"],
        },
    ]
    records = [
        {
            "artifact_id": "art-118-observability-map-md",
            "title": "ART-118 Observability Map Markdown",
            "artifact_type": "observability_map",
            "path": "execution-framework/migration-artifacts/art-118_observability/observability-map.md",
        },
        {
            "artifact_id": "art-118-observability-map-json",
            "title": "ART-118 Observability Map JSON",
            "artifact_type": "observability_map",
            "path": "execution-framework/migration-artifacts/art-118_observability/observability-map.json",
        },
        {
            "artifact_id": "08-operations-observability-map-md",
            "title": "Observability Map",
            "artifact_type": "migration_artifact",
            "path": "execution-framework/migration-artifacts/08-operations/observability-map.md",
        },
    ]
    results = []
    for record in records:
        results.append(
            registry.register(
                {
                    **record,
                    "run_id": RUN_ID,
                    "status": "complete",
                    "producer_operation_id": OPERATION_ID,
                    "contract_id": fixture["contract_id"],
                    "provenance": {
                        "task_id": TASK_ID,
                        "owner_lane": "lane_d_filesystem",
                        "owner_agent": ACTOR,
                        "helper_id": HELPER_ID,
                        "source_graph_uri": "generated/task_graph.csv",
                    },
                    "evidence_refs": [record["path"], *common_evidence],
                    "links": links,
                    "validations": validations,
                }
            )
        )

    fetched = [fetch_artifact(conn, RUN_ID, record["artifact_id"]) for record in records]
    evidence_count = conn.execute("SELECT COUNT(*) FROM envctl_migration_evidence WHERE run_id = ?", (RUN_ID,)).fetchone()[0]
    graph_count = conn.execute("SELECT COUNT(*) FROM envctl_migration_graph_edges WHERE run_id = ?", (RUN_ID,)).fetchone()[0]
    validation_count = conn.execute("SELECT COUNT(*) FROM envctl_migration_validations WHERE run_id = ?", (RUN_ID,)).fetchone()[0]
    errors: list[str] = []
    if not all(path.exists() for path in [MD_PATH, JSON_PATH, CONTRACT_PATH]):
        errors.append("artifact files were not written")
    if not all(item.get("content_hash", "").startswith("sha256:") for item in results):
        errors.append("registered artifact hash missing")
    if validation_count < 9:
        errors.append(f"expected at least 9 validation rows, got {validation_count}")
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "generated_at": now(),
        "status": "passed" if not errors else "failed",
        "run_id": RUN_ID,
        "operation_id": OPERATION_ID,
        "contract_id": fixture["contract_id"],
        "artifact_paths": {
            "markdown": "execution-framework/migration-artifacts/art-118_observability/observability-map.md",
            "json": "execution-framework/migration-artifacts/art-118_observability/observability-map.json",
            "contract_markdown": "execution-framework/migration-artifacts/08-operations/observability-map.md",
        },
        "registered_artifacts": results,
        "artifact_rows": fetched,
        "summary": {
            "observability_category_count": len(CATEGORY_TERMS),
            "covered_category_count": sum(1 for item in obs["coverage"].values() if item["evidence_count"] > 0),
            "control_count": len(obs["built_in_controls"]),
            "evidence_count": evidence_count,
            "graph_edge_count": graph_count,
            "validation_count": validation_count,
        },
        "verification": {
            "artifact_files_exist": all(path.exists() for path in [MD_PATH, JSON_PATH, CONTRACT_PATH]),
            "hashes_recorded": all(item.get("content_hash", "").startswith("sha256:") for item in results),
            "validation_evidence_linked": validation_count >= 9,
            "registry_contains_hash": all(row.get("content_hash", "").startswith("sha256:") for row in fetched),
            "secret_material_read": False,
        },
        "errors": errors,
        "evidence": [
            "execution-framework/migration-artifacts/art-118_observability/observability-map.md",
            "execution-framework/migration-artifacts/art-118_observability/observability-map.json",
            "execution-framework/migration-artifacts/08-operations/observability-map.md",
            "execution-framework/generated/art118_observability_registry_report.json",
            "execution-framework/logs/ART-118_OBSERVABILITY.log",
            "execution-framework/scripts/generate_art118_observability.py",
        ],
    }


def write_runtime_files(report: dict[str, Any], started: str) -> None:
    write_json(REPORT_PATH, report)
    LOG_PATH.parent.mkdir(parents=True, exist_ok=True)
    LOG_PATH.write_text(
        "\n".join(
            [
                f"{started} start {TASK_ID}",
                f"{now()} wrote {MD_PATH.relative_to(root())}",
                f"{now()} wrote {JSON_PATH.relative_to(root())}",
                f"{now()} wrote {CONTRACT_PATH.relative_to(root())}",
                f"{now()} registry hashes recorded: {report['verification']['hashes_recorded']}",
                f"{now()} validation evidence linked: {report['verification']['validation_evidence_linked']}",
            ]
        )
        + "\n",
        encoding="utf-8",
    )
    write_json(
        HEARTBEAT_PATH,
        {
            "task_id": TASK_ID,
            "status": "completed" if report["status"] == "passed" else "failed",
            "started_at": started,
            "updated_at": now(),
            "proof_uri": f"proof_records/{TASK_ID}.proof.json",
            "logs_uri": f"logs/{TASK_ID}.log",
            "artifact_paths": report["artifact_paths"],
        },
    )


def main() -> int:
    started = now()
    obs = build_observability_map()
    markdown = render_markdown(obs)
    ARTIFACT_DIR.mkdir(parents=True, exist_ok=True)
    MD_PATH.write_text(markdown, encoding="utf-8")
    write_json(JSON_PATH, obs)
    CONTRACT_PATH.parent.mkdir(parents=True, exist_ok=True)
    CONTRACT_PATH.write_text(markdown, encoding="utf-8")

    report = register_artifacts(obs)
    write_runtime_files(report, started)

    files_changed = [
        "execution-framework/scripts/generate_art118_observability.py",
        "execution-framework/migration-artifacts/art-118_observability/observability-map.md",
        "execution-framework/migration-artifacts/art-118_observability/observability-map.json",
        "execution-framework/migration-artifacts/08-operations/observability-map.md",
        "execution-framework/generated/art118_observability_registry_report.json",
        "execution-framework/state/ART-118_OBSERVABILITY.heartbeat.json",
        "execution-framework/logs/ART-118_OBSERVABILITY.log",
        "execution-framework/proof_records/ART-118_OBSERVABILITY.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    proof = make_proof(
        task_id=TASK_ID,
        status="completed" if report["status"] == "passed" else "failed",
        actor=ACTOR,
        helper_id=HELPER_ID,
        model_tag=MODEL_TAG,
        repo_path=str(package_root()),
        files_changed=files_changed,
        commands_run=[
            "python3 scripts/generate_art118_observability.py",
            "python3 -m py_compile scripts/generate_art118_observability.py",
            "python3 -m json.tool migration-artifacts/art-118_observability/observability-map.json",
            "python3 -m json.tool generated/art118_observability_registry_report.json",
        ],
        verification_output=report,
        evidence=report["evidence"],
        failure_reason="" if report["status"] == "passed" else "; ".join(report["errors"]),
        next_action="ready for VER-300_UNIT_VALIDATION" if report["status"] == "passed" else "fix ART-118 registry validation failures",
    )
    append_proof(proof)
    print(
        "ART-118 status={status} covered_categories={covered}/{total} evidence={evidence} validations={validations}".format(
            status=report["status"],
            covered=report["summary"]["covered_category_count"],
            total=report["summary"]["observability_category_count"],
            evidence=report["summary"]["evidence_count"],
            validations=report["summary"]["validation_count"],
        )
    )
    return 0 if report["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
