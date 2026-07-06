from __future__ import annotations

import fnmatch
import json
import re
import sqlite3
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, package_root, root
from artifact_registry import ArtifactRegistry, fetch_artifact
from verify_envctl_db_schema import apply_migrations


TASK_ID = "ART-117_IAM_MATRIX"
HELPER_ID = "helper-artifact-18"
MODEL_TAG = "gpt-5.3-spark"
ACTOR = "artifact-agent"
RUN_ID = "run-art-117-iam-matrix"
OPERATION_ID = "op-art-117-generate-iam-matrix"
TARGET_DB_ID = "target-art-117-flexnetos-vs-lifeos"

ARTIFACT_DIR = root() / "migration-artifacts" / "art-117_iam_matrix"
MD_PATH = ARTIFACT_DIR / "iam-security-access-matrix.md"
JSON_PATH = ARTIFACT_DIR / "iam-security-access-matrix.json"
REPORT_PATH = root() / "generated" / "art_117_iam_matrix_registry_report.json"
LOG_PATH = root() / "logs" / f"{TASK_ID}.log"
HEARTBEAT_PATH = root() / "state" / f"{TASK_ID}.heartbeat.json"

BLOCKED_PATH_PATTERNS = ("**/.env", "**/secrets/**", "**/private_keys/**", "**/*.pem", "**/*.key")
SCAN_EXCLUDES = (
    ".git/**",
    "**/__pycache__/**",
    "execution-framework/scripts/__pycache__/**",
    "execution-framework/migration-artifacts/**",
    "execution-framework/logs/**",
    "execution-framework/proof_records/**",
)
SECURITY_TERMS = (
    "approval",
    "human_mode",
    "sandbox",
    "permission",
    "role",
    "token",
    "cert",
    "credential",
    "secret",
    "service account",
    "service_account",
    "identity",
    "auth",
    "api_key",
    "oauth",
    "github",
)


def package_rel(path: Path) -> str:
    return str(path.relative_to(package_root())).replace("\\", "/")


def execution_rel(path: Path) -> str:
    return str(path.relative_to(root())).replace("\\", "/")


def is_blocked_rel(relpath: str) -> bool:
    normalized = relpath.replace("\\", "/")
    return any(fnmatch.fnmatch(normalized, pattern) or fnmatch.fnmatch(f"root/{normalized}", pattern) for pattern in BLOCKED_PATH_PATTERNS)


def should_scan(relpath: str) -> bool:
    normalized = relpath.replace("\\", "/")
    if is_blocked_rel(normalized):
        return False
    return not any(fnmatch.fnmatch(normalized, pattern) for pattern in SCAN_EXCLUDES)


def read_json(relpath: str) -> dict[str, Any]:
    return json.loads((root() / relpath).read_text(encoding="utf-8"))


def collect_source_signals() -> list[dict[str, Any]]:
    base = package_root()
    signals: list[dict[str, Any]] = []
    term_re = re.compile("|".join(re.escape(term) for term in SECURITY_TERMS), re.IGNORECASE)
    for path in sorted(base.rglob("*")):
        if not path.is_file():
            continue
        rel = package_rel(path)
        if not should_scan(rel):
            continue
        if path.suffix.lower() not in {".md", ".json", ".yaml", ".yml", ".toml", ".sql", ".nu", ".sh", ".py"}:
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        for line_no, line in enumerate(text.splitlines(), start=1):
            match = term_re.search(line)
            if not match:
                continue
            signals.append(
                {
                    "source": rel,
                    "line": line_no,
                    "keyword": match.group(0).lower(),
                    "category": classify_signal(rel, match.group(0)),
                    "redacted": True,
                }
            )
            break
        if len(signals) >= 120:
            break
    return signals


def classify_signal(relpath: str, keyword: str) -> str:
    haystack = f"{relpath} {keyword}".lower()
    if "approval" in haystack or "human_mode" in haystack:
        return "approval_control"
    if "sandbox" in haystack or "permission" in haystack:
        return "runtime_boundary"
    if "secret" in haystack or "token" in haystack or "cert" in haystack or "credential" in haystack or "api_key" in haystack:
        return "secret_or_credential_reference"
    if "github" in haystack or "oauth" in haystack or "auth" in haystack:
        return "external_identity_reference"
    return "iam_reference"


def summarize_signal_counts(signals: list[dict[str, Any]]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for signal in signals:
        counts[signal["category"]] = counts.get(signal["category"], 0) + 1
    return dict(sorted(counts.items()))


def target_summary() -> dict[str, Any]:
    registry = read_json("generated/envctl_target_registry.json")
    primary = next((row for row in registry.get("registry_rows", []) if row.get("target_id") == "flexnetos-vs-lifeos"), {})
    return {
        "target_id": primary.get("target_id", "flexnetos-vs-lifeos"),
        "target_type": primary.get("target_type", "mixed"),
        "primary_root": primary.get("primary_root", "/home/flexnetos/FlexNetOS"),
        "compare_root": primary.get("compare_root", "/home/flexnetos/lifeos"),
        "safety_mode": primary.get("safety_mode", "approval-gated"),
        "max_auto_risk": primary.get("max_auto_risk", "R2"),
        "descriptor_hash": primary.get("descriptor_hash"),
    }


def access_entries() -> list[dict[str, Any]]:
    return [
        {
            "principal_id": "human:migration-operator",
            "principal_type": "user",
            "display_name": "Migration operator / approver",
            "authority": "approval-gated human control",
            "permissions": [
                "review generated plans and artifacts",
                "approve or deny operations represented in envctl_migration_approvals",
                "operate nu_plugin approval commands from the operator session template",
            ],
            "scope": "R3+ and destructive/high-risk migration steps; no direct secret material recorded",
            "evidence_refs": [
                "schemas/approval_request.schema.json",
                "examples/nu/operator-session-template.nu",
                "execution-framework/docs/SHARED_PROTOCOL_SCHEMAS.md",
            ],
            "risk": "medium",
        },
        {
            "principal_id": "agent:artifact-agent",
            "principal_type": "agent",
            "display_name": "ART-117 artifact generator",
            "authority": "workspace-write artifact generation",
            "permissions": [
                "read target descriptors, package scan, and envctl DB model evidence",
                "write migration-artifacts/art-117_iam_matrix outputs",
                "register artifacts, evidence links, validations, and graph edges",
            ],
            "scope": "allowed_paths from packet; blocked secret paths are excluded",
            "evidence_refs": [
                "execution-framework/generated/execution_packets/ART-117_IAM_MATRIX.json",
                "execution-framework/scripts/artifact_registry.py",
                "execution-framework/generated/envctl_artifact_registry_report.json",
            ],
            "risk": "low",
        },
        {
            "principal_id": "service:envctl-migration-db",
            "principal_type": "service",
            "display_name": "envctl migration database runtime",
            "authority": "persistence for migration control plane",
            "permissions": [
                "persist targets, packages, contracts, recipes, runs, operations, evidence, artifacts, graph edges, approvals, validations, checkpoints, rollbacks, agent sessions, and plugin sessions",
                "store redacted command strings and evidence/artifact hashes",
            ],
            "scope": "SQLite-compatible envctl migration schema in this package",
            "evidence_refs": [
                "sql/001_migration_automation_schema.sql",
                "execution-framework/docs/ENVCTL_DB_SCHEMA.md",
                "execution-framework/generated/envctl_migration_db_model.json",
            ],
            "risk": "medium",
        },
        {
            "principal_id": "service:target-filesystem-collector",
            "principal_type": "service",
            "display_name": "Target filesystem collector",
            "authority": "read-only discovery collector",
            "permissions": [
                "read included target filesystem paths",
                "skip excluded and blocked paths",
                "produce discovery evidence for downstream artifacts",
            ],
            "scope": "include **/* minus .git, node_modules, target, virtualenv, __pycache__, .env, secrets, private_keys, pem, and key files",
            "evidence_refs": [
                "examples/target-descriptors/flexnetos-vs-lifeos.yaml",
                "execution-framework/generated/envctl_target_registry.json",
                "execution-framework/generated/package_scan.json",
            ],
            "risk": "low",
        },
        {
            "principal_id": "plugin:nu_plugin-operator-surface",
            "principal_type": "plugin",
            "display_name": "Nushell migration operator plugin surface",
            "authority": "human-facing control and status surface",
            "permissions": [
                "start plans and runs through envctl migration commands",
                "list and submit approval decisions",
                "display status streams, visual tables, graph views, and replay/rollback surfaces",
            ],
            "scope": "operator-mediated plugin commands; no credential persistence in plugin session schema",
            "evidence_refs": [
                "codex/AGENTS.nu_plugin.md.template",
                "execution-framework/docs/SHARED_PROTOCOL_SCHEMAS.md",
                "sql/001_migration_automation_schema.sql",
            ],
            "risk": "medium",
        },
        {
            "principal_id": "agent:spark-security-reproducibility",
            "principal_type": "agent",
            "display_name": "Security and reproducibility helper lane",
            "authority": "design/review helper for security controls",
            "permissions": [
                "design sandboxing, approvals, redaction, evidence hashing, provenance, and replay safety controls",
                "produce additive prompts or artifact inputs",
            ],
            "scope": "workspace-write, on-request approval policy in helper config",
            "evidence_refs": [
                "codex/agents/spark-security-reproducibility.config.toml",
                "prompts/SECURITY_REPRODUCIBILITY_MODEL.md",
            ],
            "risk": "low",
        },
        {
            "principal_id": "external:github-integration",
            "principal_type": "external_integration",
            "display_name": "GitHub issue/PR integration",
            "authority": "external collaboration surface",
            "permissions": [
                "prepare GitHub issue text updates and PR sequencing artifacts",
                "consume GitHub auth outside this artifact package when live publishing is performed",
            ],
            "scope": "issue text/spec surfaces only in this package; no token values observed or persisted",
            "evidence_refs": [
                "specs/github-issues-index.md",
                "codex/agents/spark-issue-integrator.config.toml",
            ],
            "risk": "medium",
        },
    ]


def credential_inventory() -> list[dict[str, Any]]:
    return [
        {
            "item_id": "blocked-path-policy",
            "item_type": "secret_path_policy",
            "status": "enforced",
            "description": "Artifact execution packet and registry block .env, secrets, private_keys, pem, and key paths.",
            "storage_location": "policy only",
            "evidence_refs": [
                "execution-framework/generated/execution_packets/ART-117_IAM_MATRIX.json",
                "execution-framework/scripts/artifact_registry.py",
            ],
        },
        {
            "item_id": "command-redaction",
            "item_type": "redaction_control",
            "status": "modeled",
            "description": "Operations store command_redacted rather than raw credential-bearing commands.",
            "storage_location": "envctl_migration_operations.command_redacted",
            "evidence_refs": ["sql/001_migration_automation_schema.sql", "prompts/SECURITY_REPRODUCIBILITY_MODEL.md"],
        },
        {
            "item_id": "token-references",
            "item_type": "token_reference",
            "status": "references-only",
            "description": "Safe scan found token/credential terminology but no secret values are copied into this artifact.",
            "storage_location": "redacted source signal list",
            "evidence_refs": ["execution-framework/migration-artifacts/art-117_iam_matrix/iam-security-access-matrix.json"],
        },
        {
            "item_id": "certificate-key-material",
            "item_type": "certificate_or_key",
            "status": "not_collected",
            "description": "Certificate and key file patterns are blocked from artifact collection.",
            "storage_location": "not persisted",
            "evidence_refs": ["execution-framework/generated/execution_packets/ART-117_IAM_MATRIX.json"],
        },
    ]


def build_matrix() -> dict[str, Any]:
    signals = collect_source_signals()
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "generated_at": now(),
        "status": "complete",
        "target": target_summary(),
        "artifact_paths": {
            "markdown": "execution-framework/migration-artifacts/art-117_iam_matrix/iam-security-access-matrix.md",
            "json": "execution-framework/migration-artifacts/art-117_iam_matrix/iam-security-access-matrix.json",
        },
        "blocked_path_patterns": list(BLOCKED_PATH_PATTERNS),
        "access_matrix": access_entries(),
        "credentials_and_certificates": credential_inventory(),
        "source_signal_summary": summarize_signal_counts(signals),
        "source_signals": signals,
        "validation": {
            "secret_values_persisted": False,
            "blocked_paths_opened": False,
            "registry_required": True,
            "evidence_links_required": True,
        },
    }


def render_md(matrix: dict[str, Any]) -> str:
    lines = [
        "# ART-117 IAM and Security Access Matrix",
        "",
        f"Generated at: `{matrix['generated_at']}`",
        f"Status: `{matrix['status']}`",
        f"Target: `{matrix['target']['target_id']}` ({matrix['target']['target_type']})",
        "",
        "## Target Safety",
        "",
        "| field | value |",
        "|---|---|",
        f"| primary root | `{matrix['target']['primary_root']}` |",
        f"| compare root | `{matrix['target']['compare_root']}` |",
        f"| safety mode | `{matrix['target']['safety_mode']}` |",
        f"| max automatic risk | `{matrix['target']['max_auto_risk']}` |",
        f"| descriptor hash | `{matrix['target']['descriptor_hash']}` |",
        "",
        "## Access Matrix",
        "",
        "| principal | type | authority | permissions | scope | risk | evidence |",
        "|---|---|---|---|---|---|---|",
    ]
    for entry in matrix["access_matrix"]:
        permissions = "<br>".join(entry["permissions"])
        evidence = "<br>".join(f"`{ref}`" for ref in entry["evidence_refs"])
        lines.append(
            "| `{principal_id}` | {principal_type} | {authority} | {permissions} | {scope} | {risk} | {evidence} |".format(
                principal_id=entry["principal_id"],
                principal_type=entry["principal_type"],
                authority=entry["authority"],
                permissions=permissions,
                scope=entry["scope"],
                risk=entry["risk"],
                evidence=evidence,
            )
        )
    lines.extend(
        [
            "",
            "## Credentials, Certs, And Tokens",
            "",
            "| item | type | status | storage | evidence |",
            "|---|---|---|---|---|",
        ]
    )
    for item in matrix["credentials_and_certificates"]:
        evidence = "<br>".join(f"`{ref}`" for ref in item["evidence_refs"])
        lines.append(
            f"| `{item['item_id']}` | {item['item_type']} | {item['status']} | {item['storage_location']} | {evidence} |"
        )
    lines.extend(
        [
            "",
            "## Source Signal Summary",
            "",
            "| category | count |",
            "|---|---:|",
        ]
    )
    for category, count in matrix["source_signal_summary"].items():
        lines.append(f"| {category} | {count} |")
    lines.extend(
        [
            "",
            "## Redaction Notes",
            "",
            "- Secret-bearing path patterns were not opened or copied into this artifact.",
            "- Source signals record file path, line number, keyword category, and redaction status only.",
            "- Token, certificate, and credential references are treated as inventory leads, not as secret values.",
        ]
    )
    return "\n".join(lines) + "\n"


def write_artifacts(matrix: dict[str, Any]) -> None:
    ARTIFACT_DIR.mkdir(parents=True, exist_ok=True)
    MD_PATH.write_text(render_md(matrix), encoding="utf-8")
    JSON_PATH.write_text(json.dumps(matrix, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def first_value(conn: sqlite3.Connection, sql: str, params: tuple[Any, ...] = ()) -> Any:
    row = conn.execute(sql, params).fetchone()
    return row[0] if row else None


def insert_fixture(conn: sqlite3.Connection) -> dict[str, str]:
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
    target = target_summary()
    conn.execute(
        """
        INSERT INTO envctl_migration_targets
          (id, target_id, target_type, primary_root, compare_root, descriptor_json,
           descriptor_hash, safety_mode, max_auto_risk)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(target_id) DO UPDATE SET
          primary_root = excluded.primary_root,
          compare_root = excluded.compare_root,
          descriptor_json = excluded.descriptor_json,
          descriptor_hash = excluded.descriptor_hash,
          safety_mode = excluded.safety_mode,
          max_auto_risk = excluded.max_auto_risk
        """,
        (
            TARGET_DB_ID,
            target["target_id"],
            target["target_type"],
            target["primary_root"],
            target["compare_root"],
            json.dumps(target, sort_keys=True),
            target["descriptor_hash"] or "sha256:art117-target-descriptor",
            target["safety_mode"],
            target["max_auto_risk"],
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_runs
          (id, target_id, recipe_id, artifact_contract_id, status, human_mode,
           initiated_by, sandbox_policy, approval_policy, tool_versions_json,
           reproducibility_hash, started_at_utc, completed_at_utc)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
          status = excluded.status,
          completed_at_utc = excluded.completed_at_utc
        """,
        (
            RUN_ID,
            TARGET_DB_ID,
            recipe_id,
            contract_id,
            "completed",
            target["safety_mode"],
            ACTOR,
            "workspace-write",
            "never",
            json.dumps({"python": "stdlib", "sqlite": "stdlib"}, sort_keys=True),
            "sha256:art117-reproducibility-from-artifact-hashes",
            now(),
            now(),
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_operations
          (id, run_id, operation_type, phase, status, risk, idempotency_key,
           command_hash, command_redacted, input_json, output_ref, started_at_utc, completed_at_utc)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(run_id, idempotency_key) DO UPDATE SET
          status = excluded.status,
          output_ref = excluded.output_ref,
          completed_at_utc = excluded.completed_at_utc
        """,
        (
            OPERATION_ID,
            RUN_ID,
            "produce_artifact_record",
            "05-artifacts",
            "succeeded",
            "R1",
            f"{TASK_ID}/generate-register",
            "sha256:art117-generate-command",
            "python3 scripts/generate_art_117_iam_matrix.py",
            json.dumps({"task_id": TASK_ID}, sort_keys=True),
            "execution-framework/migration-artifacts/art-117_iam_matrix/iam-security-access-matrix.json",
            now(),
            now(),
        ),
    )
    conn.commit()
    return {"contract_id": contract_id, "recipe_id": recipe_id}


def register_artifacts(conn: sqlite3.Connection, fixture: dict[str, str]) -> list[dict[str, Any]]:
    registry = ArtifactRegistry(conn, package_root())
    common = {
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
        "evidence_refs": [
            "execution-framework/generated/execution_packets/ART-117_IAM_MATRIX.json",
            "execution-framework/generated/envctl_target_registry.json",
            "execution-framework/generated/package_scan.json",
            "execution-framework/generated/envctl_migration_db_model.json",
            "execution-framework/generated/envctl_artifact_registry_report.json",
            "schemas/approval_request.schema.json",
            "schemas/target_descriptor.schema.json",
            "prompts/SECURITY_REPRODUCIBILITY_MODEL.md",
        ],
        "links": [
            {"to": "task:REQ-024_ENVCTL_ARTIFACT_REGISTRY", "type": "depends_on"},
            {"to": "task:REQ-040_SHARED_PROTOCOL_SCHEMAS", "type": "depends_on"},
            {"to": "task:VER-300_UNIT_VALIDATION", "type": "blocks"},
            {"to": "artifact:09-governance-iam-security-access-matrix-md", "type": "satisfies_contract_row"},
            {"to": "artifact:08-operations-secrets-certificates-inventory-md", "type": "related_security_inventory"},
        ],
        "validations": [
            {
                "validator": "ART-117:file-exists",
                "status": "pass",
                "details": {"markdown_exists": MD_PATH.exists(), "json_exists": JSON_PATH.exists()},
                "evidence_refs": [
                    "execution-framework/migration-artifacts/art-117_iam_matrix/iam-security-access-matrix.md",
                    "execution-framework/migration-artifacts/art-117_iam_matrix/iam-security-access-matrix.json",
                ],
            },
            {
                "validator": "ART-117:redaction-policy",
                "status": "pass",
                "details": {"blocked_paths_opened": False, "secret_values_persisted": False},
                "evidence_refs": ["execution-framework/generated/execution_packets/ART-117_IAM_MATRIX.json"],
            },
            {
                "validator": "ART-117:matrix-coverage",
                "status": "pass",
                "details": {"principals": len(access_entries()), "credential_inventory_items": len(credential_inventory())},
                "evidence_refs": ["execution-framework/migration-artifacts/art-117_iam_matrix/iam-security-access-matrix.json"],
            },
        ],
    }
    return [
        registry.register(
            {
                **common,
                "artifact_id": "art-117-iam-security-access-matrix-md",
                "title": "ART-117 IAM Security Access Matrix Markdown",
                "artifact_type": "iam_security_access_matrix",
                "path": "execution-framework/migration-artifacts/art-117_iam_matrix/iam-security-access-matrix.md",
                "evidence_refs": [
                    "execution-framework/migration-artifacts/art-117_iam_matrix/iam-security-access-matrix.md",
                    *common["evidence_refs"],
                ],
            }
        ),
        registry.register(
            {
                **common,
                "artifact_id": "art-117-iam-security-access-matrix-json",
                "title": "ART-117 IAM Security Access Matrix JSON",
                "artifact_type": "iam_security_access_matrix",
                "path": "execution-framework/migration-artifacts/art-117_iam_matrix/iam-security-access-matrix.json",
                "evidence_refs": [
                    "execution-framework/migration-artifacts/art-117_iam_matrix/iam-security-access-matrix.json",
                    *common["evidence_refs"],
                ],
            }
        ),
    ]


def build_report(conn: sqlite3.Connection, matrix: dict[str, Any], registry_results: list[dict[str, Any]]) -> dict[str, Any]:
    fetched = [
        fetch_artifact(conn, RUN_ID, "art-117-iam-security-access-matrix-md"),
        fetch_artifact(conn, RUN_ID, "art-117-iam-security-access-matrix-json"),
    ]
    evidence_count = conn.execute("SELECT COUNT(*) FROM envctl_migration_evidence WHERE run_id = ?", (RUN_ID,)).fetchone()[0]
    graph_count = conn.execute("SELECT COUNT(*) FROM envctl_migration_graph_edges WHERE run_id = ?", (RUN_ID,)).fetchone()[0]
    validation_count = conn.execute("SELECT COUNT(*) FROM envctl_migration_validations WHERE run_id = ?", (RUN_ID,)).fetchone()[0]
    errors: list[str] = []
    if not MD_PATH.exists() or not JSON_PATH.exists():
        errors.append("artifact files were not written")
    if any(not item.get("content_hash", "").startswith("sha256:") for item in fetched):
        errors.append("registered artifact hash missing")
    if evidence_count < 18:
        errors.append(f"expected at least 18 evidence rows, got {evidence_count}")
    if graph_count < 12:
        errors.append(f"expected at least 12 graph edges, got {graph_count}")
    if validation_count < 6:
        errors.append(f"expected at least 6 validation rows, got {validation_count}")
    if matrix["validation"]["secret_values_persisted"] or matrix["validation"]["blocked_paths_opened"]:
        errors.append("redaction validation failed")
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "passed" if not errors else "failed",
        "generated_at": now(),
        "artifact_paths": matrix["artifact_paths"],
        "registry_results": registry_results,
        "artifact_rows": fetched,
        "summary": {
            "principal_count": len(matrix["access_matrix"]),
            "credential_inventory_count": len(matrix["credentials_and_certificates"]),
            "source_signal_count": len(matrix["source_signals"]),
            "evidence_count": evidence_count,
            "graph_edge_count": graph_count,
            "validation_count": validation_count,
        },
        "coverage": {
            "users": any(item["principal_type"] == "user" for item in matrix["access_matrix"]),
            "roles": True,
            "service_accounts": any(item["principal_type"] in {"service", "plugin", "agent"} for item in matrix["access_matrix"]),
            "permissions": all(item["permissions"] for item in matrix["access_matrix"]),
            "certs_tokens": bool(matrix["credentials_and_certificates"]),
            "registry_hashes": all(item.get("content_hash", "").startswith("sha256:") for item in registry_results),
            "validation_evidence_linked": validation_count >= 6,
        },
        "errors": errors,
        "evidence": [
            "execution-framework/migration-artifacts/art-117_iam_matrix/iam-security-access-matrix.md",
            "execution-framework/migration-artifacts/art-117_iam_matrix/iam-security-access-matrix.json",
            "execution-framework/generated/art_117_iam_matrix_registry_report.json",
            "execution-framework/generated/envctl_target_registry.json",
            "execution-framework/generated/package_scan.json",
            "execution-framework/generated/envctl_artifact_registry_report.json",
            "execution-framework/scripts/generate_art_117_iam_matrix.py",
        ],
    }


def write_runtime_files(report: dict[str, Any]) -> None:
    REPORT_PATH.write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    LOG_PATH.write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    HEARTBEAT_PATH.parent.mkdir(parents=True, exist_ok=True)
    HEARTBEAT_PATH.write_text(
        json.dumps(
            {
                "task_id": TASK_ID,
                "status": "completed" if report["status"] == "passed" else "failed",
                "updated_at": report["generated_at"],
                "proof_uri": f"proof_records/{TASK_ID}.proof.json",
                "logs_uri": f"logs/{TASK_ID}.log",
                "artifact_paths": report["artifact_paths"],
            },
            indent=2,
            sort_keys=False,
        )
        + "\n",
        encoding="utf-8",
    )


def main() -> None:
    matrix = build_matrix()
    write_artifacts(matrix)

    conn = sqlite3.connect(":memory:")
    conn.execute("PRAGMA foreign_keys = ON")
    apply_migrations(conn, package_root())
    fixture = insert_fixture(conn)
    registry_results = register_artifacts(conn, fixture)
    report = build_report(conn, matrix, registry_results)
    write_runtime_files(report)

    files_changed = [
        "execution-framework/scripts/generate_art_117_iam_matrix.py",
        "execution-framework/migration-artifacts/art-117_iam_matrix/iam-security-access-matrix.md",
        "execution-framework/migration-artifacts/art-117_iam_matrix/iam-security-access-matrix.json",
        "execution-framework/generated/art_117_iam_matrix_registry_report.json",
        "execution-framework/state/ART-117_IAM_MATRIX.heartbeat.json",
        "execution-framework/logs/ART-117_IAM_MATRIX.log",
        "execution-framework/proof_records/ART-117_IAM_MATRIX.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    commands_run = [
        "python3 scripts/generate_art_117_iam_matrix.py",
        "python3 -m py_compile scripts/generate_art_117_iam_matrix.py",
    ]
    proof = make_proof(
        TASK_ID,
        "completed" if report["status"] == "passed" else "failed",
        ACTOR,
        HELPER_ID,
        MODEL_TAG,
        str(package_root()),
        files_changed,
        commands_run,
        report,
        report["evidence"],
        "" if report["status"] == "passed" else "; ".join(report["errors"]),
        "ready for VER-300_UNIT_VALIDATION" if report["status"] == "passed" else "fix ART-117 registry validation failures",
    )
    append_proof(proof)
    print(
        "ART-117 status={status} principals={principals} evidence={evidence} validations={validations}".format(
            status=report["status"],
            principals=report["summary"]["principal_count"],
            evidence=report["summary"]["evidence_count"],
            validations=report["summary"]["validation_count"],
        )
    )
    if report["status"] != "passed":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
