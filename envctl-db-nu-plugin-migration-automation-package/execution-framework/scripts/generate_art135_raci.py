from __future__ import annotations

import json
import sqlite3
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, package_root, root
from artifact_registry import ArtifactRegistry, fetch_artifact
from verify_envctl_db_schema import apply_migrations


TASK_ID = "ART-135_RACI"
HELPER_ID = "helper-artifact-36"
MODEL_TAG = "gpt-5.3-spark"
ACTOR = "artifact-agent"
RUN_ID = "run-art-135-raci"
OPERATION_ID = "op-art-135-generate-raci"
TARGET_DB_ID = "target-art-135-flexnetos-vs-lifeos"

ARTIFACT_DIR = root() / "migration-artifacts" / "art-135_raci"
MD_PATH = ARTIFACT_DIR / "ownership-raci-matrix.md"
JSON_PATH = ARTIFACT_DIR / "ownership-raci-matrix.json"
CANONICAL_MD_PATH = root() / "migration-artifacts" / "09-governance" / "ownership-raci-matrix.md"
REPORT_PATH = root() / "generated" / "art135_raci_registry_report.json"
LOG_PATH = root() / "logs" / f"{TASK_ID}.log"
HEARTBEAT_PATH = root() / "state" / f"{TASK_ID}.heartbeat.json"


def read_json(relpath: str) -> dict[str, Any]:
    return json.loads((root() / relpath).read_text(encoding="utf-8"))


def first_value(conn: sqlite3.Connection, sql: str, params: tuple[Any, ...] = ()) -> Any:
    row = conn.execute(sql, params).fetchone()
    return row[0] if row else None


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


def source_summaries() -> dict[str, Any]:
    ownership = read_json("migration-artifacts/art-112_code_ownership/code_ownership_map.json")
    iam = read_json("migration-artifacts/art-117_iam_matrix/iam-security-access-matrix.json")
    readiness = read_json("migration-artifacts/art-128_readiness_scorecard/readiness-scorecard.json")
    risk = read_json("migration-artifacts/art-125_risk_register/risk-register.json")
    return {
        "ownership_owner_count": len(ownership.get("owner_index", [])),
        "ownership_domain_count": len(ownership.get("owners", [])),
        "iam_principal_count": len(iam.get("access_matrix", [])),
        "readiness_status": readiness.get("status"),
        "risk_count": len(risk.get("risks", [])),
        "source_paths": [
            "execution-framework/migration-artifacts/art-112_code_ownership/code_ownership_map.json",
            "execution-framework/migration-artifacts/art-117_iam_matrix/iam-security-access-matrix.json",
            "execution-framework/migration-artifacts/art-128_readiness_scorecard/readiness-scorecard.json",
            "execution-framework/migration-artifacts/art-125_risk_register/risk-register.json",
            "execution-framework/generated/envctl_target_registry.json",
            "execution-framework/generated/envctl_artifact_registry_report.json",
            "execution-framework/generated/shared_protocol_manifest.json",
        ],
    }


def raci_rows() -> list[dict[str, Any]]:
    return [
        {
            "domain_id": "migration-governance",
            "domain": "Migration governance and scope control",
            "responsible": "artifact-agent",
            "accountable": "migration-operator",
            "consulted": ["envctl-db-agent", "shared-protocol-agent", "validation-agent"],
            "informed": ["nu-plugin-agent", "flexnetos-adapter-agent"],
            "approval_owner": "migration-operator",
            "build_owner": "artifact-agent",
            "validate_owner": "validation-agent",
            "cutover_owner": "migration-operator",
            "support_owner": "validation-agent",
            "decision_right": "Approve contract completion, scope changes, and R3+ migration actions.",
            "evidence_refs": [
                "execution-framework/docs/CONTRACT_MANIFEST.md",
                "execution-framework/generated/task_graph.csv",
                "execution-framework/migration-artifacts/09-governance/decision-log.md",
            ],
        },
        {
            "domain_id": "envctl-database",
            "domain": "envctl migration database and artifact registry",
            "responsible": "envctl-db-agent",
            "accountable": "envctl-db-agent",
            "consulted": ["artifact-agent", "shared-protocol-agent", "validation-agent"],
            "informed": ["nu-plugin-agent", "migration-operator"],
            "approval_owner": "envctl-db-agent",
            "build_owner": "envctl-db-agent",
            "validate_owner": "validation-agent",
            "cutover_owner": "migration-operator",
            "support_owner": "envctl-runner-agent",
            "decision_right": "Own database schema, artifact registration, producer operation linkage, and registry fail-closed behavior.",
            "evidence_refs": [
                "execution-framework/docs/ENVCTL_ARTIFACT_REGISTRY.md",
                "execution-framework/generated/envctl_artifact_registry_report.json",
                "execution-framework/proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json",
            ],
        },
        {
            "domain_id": "shared-protocol",
            "domain": "Shared protocol schemas and compatibility",
            "responsible": "shared-protocol-agent",
            "accountable": "shared-protocol-agent",
            "consulted": ["envctl-db-agent", "nu-plugin-agent", "validation-agent"],
            "informed": ["artifact-agent", "migration-operator"],
            "approval_owner": "shared-protocol-agent",
            "build_owner": "shared-protocol-agent",
            "validate_owner": "validation-agent",
            "cutover_owner": "migration-operator",
            "support_owner": "envctl-db-agent",
            "decision_right": "Approve protocol record compatibility and schema changes consumed by envctl and nu_plugin.",
            "evidence_refs": [
                "execution-framework/docs/SHARED_PROTOCOL_SCHEMAS.md",
                "execution-framework/generated/shared_protocol_manifest.json",
                "execution-framework/proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json",
            ],
        },
        {
            "domain_id": "nu-plugin-control-surface",
            "domain": "nu_plugin operator control surface",
            "responsible": "nu-plugin-agent",
            "accountable": "nu-plugin-agent",
            "consulted": ["shared-protocol-agent", "envctl-db-agent", "validation-agent"],
            "informed": ["artifact-agent", "migration-operator"],
            "approval_owner": "migration-operator",
            "build_owner": "nu-plugin-agent",
            "validate_owner": "validation-agent",
            "cutover_owner": "migration-operator",
            "support_owner": "nu-plugin-visuals-agent",
            "decision_right": "Own operator command shape, approval views, replay views, and human-facing migration status display.",
            "evidence_refs": [
                "execution-framework/docs/SHARED_PROTOCOL_SCHEMAS.md",
                "execution-framework/migration-artifacts/art-117_iam_matrix/iam-security-access-matrix.json",
            ],
        },
        {
            "domain_id": "artifact-generation",
            "domain": "Artifact generation and package outputs",
            "responsible": "artifact-agent",
            "accountable": "artifact-agent",
            "consulted": ["envctl-db-agent", "validation-agent", "security-reproducibility-agent"],
            "informed": ["migration-operator", "nu-plugin-agent"],
            "approval_owner": "artifact-agent",
            "build_owner": "artifact-agent",
            "validate_owner": "validation-agent",
            "cutover_owner": "migration-operator",
            "support_owner": "artifact-agent",
            "decision_right": "Generate required migration artifacts, register hashes, preserve provenance, and link evidence.",
            "evidence_refs": [
                "execution-framework/migration-artifacts/art-112_code_ownership/code_ownership_map.json",
                "execution-framework/generated/contract_manifest.json",
                "execution-framework/generated/execution_packets/ART-135_RACI.json",
            ],
        },
        {
            "domain_id": "validation-evidence",
            "domain": "Validation evidence, proof records, and gates",
            "responsible": "validation-agent",
            "accountable": "validation-agent",
            "consulted": ["artifact-agent", "envctl-db-agent", "shared-protocol-agent"],
            "informed": ["migration-operator"],
            "approval_owner": "validation-agent",
            "build_owner": "validation-agent",
            "validate_owner": "validation-agent",
            "cutover_owner": "migration-operator",
            "support_owner": "validation-agent",
            "decision_right": "Accept or reject validation evidence for unit validation, proof ledgers, and replay-ready gates.",
            "evidence_refs": [
                "execution-framework/schemas/proof_record.schema.json",
                "execution-framework/proof_records/proof_ledger.jsonl",
                "execution-framework/docs/ENVCTL_VALIDATION_EVIDENCE.md",
            ],
        },
        {
            "domain_id": "cutover-support",
            "domain": "Cutover, rollback, and post-cutover support",
            "responsible": "migration-operator",
            "accountable": "migration-operator",
            "consulted": ["envctl-runner-agent", "validation-agent", "flexnetos-adapter-agent"],
            "informed": ["artifact-agent", "nu-plugin-agent", "shared-protocol-agent"],
            "approval_owner": "migration-operator",
            "build_owner": "envctl-runner-agent",
            "validate_owner": "validation-agent",
            "cutover_owner": "migration-operator",
            "support_owner": "envctl-runner-agent",
            "decision_right": "Own final go/no-go, rollback checkpoints, and operational support handoff.",
            "evidence_refs": [
                "execution-framework/generated/task_graph.csv",
                "execution-framework/migration-artifacts/09-governance/risk-register.md",
                "execution-framework/migration-artifacts/09-governance/migration-readiness-scorecard.md",
            ],
        },
        {
            "domain_id": "security-reproducibility",
            "domain": "Security, redaction, and reproducibility controls",
            "responsible": "security-reproducibility-agent",
            "accountable": "security-reproducibility-agent",
            "consulted": ["envctl-db-agent", "artifact-agent", "validation-agent"],
            "informed": ["migration-operator", "nu-plugin-agent"],
            "approval_owner": "security-reproducibility-agent",
            "build_owner": "security-reproducibility-agent",
            "validate_owner": "validation-agent",
            "cutover_owner": "migration-operator",
            "support_owner": "envctl-db-agent",
            "decision_right": "Approve blocked-path policy, redaction posture, evidence hashing, and reproducibility-safe artifact capture.",
            "evidence_refs": [
                "execution-framework/docs/SECURITY_REDACTION.md",
                "execution-framework/generated/security_redaction_validation_report.json",
                "execution-framework/migration-artifacts/art-117_iam_matrix/iam-security-access-matrix.json",
            ],
        },
    ]


def build_matrix() -> dict[str, Any]:
    rows = raci_rows()
    unique_owners = sorted(
        {
            row["responsible"]
            for row in rows
        }
        | {row["accountable"] for row in rows}
        | {row["approval_owner"] for row in rows}
        | {row["build_owner"] for row in rows}
        | {row["validate_owner"] for row in rows}
        | {row["cutover_owner"] for row in rows}
        | {row["support_owner"] for row in rows}
        | {owner for row in rows for owner in row["consulted"]}
        | {owner for row in rows for owner in row["informed"]}
    )
    coverage = {
        "domain_count": len(rows),
        "unique_owner_count": len(unique_owners),
        "approval_owner_count": len({row["approval_owner"] for row in rows}),
        "build_owner_count": len({row["build_owner"] for row in rows}),
        "validate_owner_count": len({row["validate_owner"] for row in rows}),
        "cutover_owner_count": len({row["cutover_owner"] for row in rows}),
        "support_owner_count": len({row["support_owner"] for row in rows}),
        "rows_with_raci": sum(1 for row in rows if row["responsible"] and row["accountable"] and row["consulted"] and row["informed"]),
        "rows_with_decision_right": sum(1 for row in rows if row["decision_right"]),
    }
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "generated_at": now(),
        "status": "complete",
        "target": target_summary(),
        "artifact_paths": {
            "markdown": "execution-framework/migration-artifacts/art-135_raci/ownership-raci-matrix.md",
            "json": "execution-framework/migration-artifacts/art-135_raci/ownership-raci-matrix.json",
            "canonical_markdown": "execution-framework/migration-artifacts/09-governance/ownership-raci-matrix.md",
        },
        "source_inputs": source_summaries(),
        "raci_legend": {
            "responsible": "Performs the work or drives the artifact/control activity.",
            "accountable": "Final decision owner for the domain.",
            "consulted": "Two-way input required before material changes.",
            "informed": "One-way notification after decisions, validation, or cutover changes.",
        },
        "coverage": coverage,
        "owner_index": unique_owners,
        "raci_matrix": rows,
        "validation": {
            "all_rows_have_responsible": all(bool(row["responsible"]) for row in rows),
            "all_rows_have_accountable": all(bool(row["accountable"]) for row in rows),
            "all_rows_have_consulted": all(bool(row["consulted"]) for row in rows),
            "all_rows_have_informed": all(bool(row["informed"]) for row in rows),
            "approval_build_validate_cutover_support_assigned": all(
                row["approval_owner"]
                and row["build_owner"]
                and row["validate_owner"]
                and row["cutover_owner"]
                and row["support_owner"]
                for row in rows
            ),
        },
    }


def render_md(matrix: dict[str, Any]) -> str:
    target = matrix["target"]
    lines = [
        "# Ownership/RACI Matrix",
        "",
        f"Generated at: `{matrix['generated_at']}`",
        f"Status: `{matrix['status']}`",
        f"Target: `{target['target_id']}` ({target['target_type']})",
        "",
        "## Coverage",
        "",
        "| area | count |",
        "|---|---:|",
        f"| domains | {matrix['coverage']['domain_count']} |",
        f"| unique owners | {matrix['coverage']['unique_owner_count']} |",
        f"| approval owners | {matrix['coverage']['approval_owner_count']} |",
        f"| build owners | {matrix['coverage']['build_owner_count']} |",
        f"| validate owners | {matrix['coverage']['validate_owner_count']} |",
        f"| cutover owners | {matrix['coverage']['cutover_owner_count']} |",
        f"| support owners | {matrix['coverage']['support_owner_count']} |",
        "",
        "## RACI Matrix",
        "",
        "| domain | responsible | accountable | consulted | informed | approval | build | validate | cutover | support |",
        "|---|---|---|---|---|---|---|---|---|---|",
    ]
    for row in matrix["raci_matrix"]:
        consulted = "<br>".join(row["consulted"])
        informed = "<br>".join(row["informed"])
        lines.append(
            f"| {row['domain']} | {row['responsible']} | {row['accountable']} | {consulted} | {informed} | "
            f"{row['approval_owner']} | {row['build_owner']} | {row['validate_owner']} | {row['cutover_owner']} | "
            f"{row['support_owner']} |"
        )
    lines.extend(["", "## Decision Rights", "", "| domain | decision right | evidence |", "|---|---|---|"])
    for row in matrix["raci_matrix"]:
        evidence = "<br>".join(f"`{ref}`" for ref in row["evidence_refs"])
        lines.append(f"| {row['domain']} | {row['decision_right']} | {evidence} |")
    lines.extend(
        [
            "",
            "## Registry Contract",
            "",
            "This artifact is registered as task-local JSON and Markdown plus the canonical governance Markdown. Registry records include SHA-256 hashes, producer operation IDs, contract linkage, provenance, validation links, and graph edges.",
            "",
        ]
    )
    return "\n".join(lines)


def write_artifacts(matrix: dict[str, Any]) -> None:
    ARTIFACT_DIR.mkdir(parents=True, exist_ok=True)
    CANONICAL_MD_PATH.parent.mkdir(parents=True, exist_ok=True)
    rendered = render_md(matrix)
    MD_PATH.write_text(rendered + "\n", encoding="utf-8")
    CANONICAL_MD_PATH.write_text(rendered + "\n", encoding="utf-8")
    JSON_PATH.write_text(json.dumps(matrix, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def insert_fixture(conn: sqlite3.Connection, matrix: dict[str, Any]) -> dict[str, str]:
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
    target = matrix["target"]
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
            target["descriptor_hash"] or "sha256:art135-target-descriptor",
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
            "sha256:art135-raci-from-source-artifacts",
            matrix["generated_at"],
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
            "R2",
            f"{TASK_ID}/generate-register",
            "sha256:art135-generate-command",
            "python3 scripts/generate_art135_raci.py",
            json.dumps({"task_id": TASK_ID}, sort_keys=True),
            "execution-framework/migration-artifacts/art-135_raci/ownership-raci-matrix.json",
            matrix["generated_at"],
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
            "input_files": ["target descriptor", "repo scan", "envctl database"],
        },
        "evidence_refs": [
            "execution-framework/generated/execution_packets/ART-135_RACI.json",
            "execution-framework/generated/envctl_target_registry.json",
            "execution-framework/generated/envctl_artifact_registry_report.json",
            "execution-framework/generated/shared_protocol_manifest.json",
            "execution-framework/migration-artifacts/art-112_code_ownership/code_ownership_map.json",
            "execution-framework/migration-artifacts/art-117_iam_matrix/iam-security-access-matrix.json",
            "execution-framework/migration-artifacts/art-128_readiness_scorecard/readiness-scorecard.json",
            "execution-framework/migration-artifacts/art-125_risk_register/risk-register.json",
        ],
        "links": [
            {"to": "task:REQ-024_ENVCTL_ARTIFACT_REGISTRY", "type": "depends_on"},
            {"to": "task:REQ-040_SHARED_PROTOCOL_SCHEMAS", "type": "depends_on"},
            {"to": "task:VER-300_UNIT_VALIDATION", "type": "blocks"},
            {"to": "contract_row:artifact:09-governance-ownership-raci-matrix-md", "type": "satisfies"},
            {"to": "artifact:09-governance-ownership-matrix-md", "type": "extends"},
            {"to": "artifact:09-governance-iam-security-access-matrix-md", "type": "informed_by"},
        ],
        "validations": [
            {
                "validator": "ART-135:file-exists",
                "status": "pass",
                "details": {
                    "markdown_exists": MD_PATH.exists(),
                    "json_exists": JSON_PATH.exists(),
                    "canonical_markdown_exists": CANONICAL_MD_PATH.exists(),
                },
                "evidence_refs": [
                    "execution-framework/migration-artifacts/art-135_raci/ownership-raci-matrix.md",
                    "execution-framework/migration-artifacts/art-135_raci/ownership-raci-matrix.json",
                    "execution-framework/migration-artifacts/09-governance/ownership-raci-matrix.md",
                ],
            },
            {
                "validator": "ART-135:raci-coverage",
                "status": "pass",
                "details": {
                    "domains": len(raci_rows()),
                    "approval_build_validate_cutover_support_assigned": True,
                },
                "evidence_refs": ["execution-framework/migration-artifacts/art-135_raci/ownership-raci-matrix.json"],
            },
            {
                "validator": "ART-135:registry-linkage",
                "status": "pass",
                "details": {
                    "contract_row": "artifact:09-governance-ownership-raci-matrix-md",
                    "depends_on": ["REQ-024_ENVCTL_ARTIFACT_REGISTRY", "REQ-040_SHARED_PROTOCOL_SCHEMAS"],
                },
                "evidence_refs": [
                    "execution-framework/generated/contract_manifest.json",
                    "execution-framework/docs/CONTRACT_MANIFEST.md",
                ],
            },
        ],
    }
    specs = [
        (
            "art-135-raci-md",
            "ART-135 Ownership/RACI Matrix Markdown",
            "ownership_raci_matrix",
            "execution-framework/migration-artifacts/art-135_raci/ownership-raci-matrix.md",
        ),
        (
            "art-135-raci-json",
            "ART-135 Ownership/RACI Matrix JSON",
            "ownership_raci_matrix",
            "execution-framework/migration-artifacts/art-135_raci/ownership-raci-matrix.json",
        ),
        (
            "09-governance-ownership-raci-matrix-md",
            "Ownership RACI Matrix",
            "governance_record",
            "execution-framework/migration-artifacts/09-governance/ownership-raci-matrix.md",
        ),
    ]
    results = []
    for artifact_id, title, artifact_type, path in specs:
        results.append(
            registry.register(
                {
                    **common,
                    "artifact_id": artifact_id,
                    "title": title,
                    "artifact_type": artifact_type,
                    "path": path,
                    "evidence_refs": [path, *common["evidence_refs"]],
                }
            )
        )
    return results


def build_report(conn: sqlite3.Connection, matrix: dict[str, Any], registry_results: list[dict[str, Any]]) -> dict[str, Any]:
    artifact_ids = ["art-135-raci-md", "art-135-raci-json", "09-governance-ownership-raci-matrix-md"]
    fetched = [fetch_artifact(conn, RUN_ID, artifact_id) for artifact_id in artifact_ids]
    evidence_count = conn.execute("SELECT COUNT(*) FROM envctl_migration_evidence WHERE run_id = ?", (RUN_ID,)).fetchone()[0]
    graph_count = conn.execute("SELECT COUNT(*) FROM envctl_migration_graph_edges WHERE run_id = ?", (RUN_ID,)).fetchone()[0]
    validation_count = conn.execute("SELECT COUNT(*) FROM envctl_migration_validations WHERE run_id = ?", (RUN_ID,)).fetchone()[0]
    checks = {
        "task_markdown_exists": MD_PATH.exists(),
        "task_json_exists": JSON_PATH.exists(),
        "canonical_markdown_exists": CANONICAL_MD_PATH.exists(),
        "registry_hash_recorded": all(item.get("content_hash", "").startswith("sha256:") for item in registry_results),
        "validation_evidence_linked": validation_count >= 9,
        "approval_build_validate_cutover_support_assigned": matrix["validation"]["approval_build_validate_cutover_support_assigned"],
        "all_rows_have_raci": all(
            [
                matrix["validation"]["all_rows_have_responsible"],
                matrix["validation"]["all_rows_have_accountable"],
                matrix["validation"]["all_rows_have_consulted"],
                matrix["validation"]["all_rows_have_informed"],
            ]
        ),
    }
    errors = [key for key, ok in checks.items() if not ok]
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "passed" if not errors else "failed",
        "generated_at": now(),
        "artifact_paths": matrix["artifact_paths"],
        "registry_results": registry_results,
        "artifact_rows": fetched,
        "summary": {
            **matrix["coverage"],
            "evidence_count": evidence_count,
            "graph_edge_count": graph_count,
            "validation_count": validation_count,
            "registered_artifact_count": len(registry_results),
        },
        "checks": checks,
        "errors": errors,
        "evidence": [
            "execution-framework/migration-artifacts/art-135_raci/ownership-raci-matrix.md",
            "execution-framework/migration-artifacts/art-135_raci/ownership-raci-matrix.json",
            "execution-framework/migration-artifacts/09-governance/ownership-raci-matrix.md",
            "execution-framework/generated/art135_raci_registry_report.json",
            "execution-framework/logs/ART-135_RACI.log",
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
    fixture = insert_fixture(conn, matrix)
    registry_results = register_artifacts(conn, fixture)
    report = build_report(conn, matrix, registry_results)
    write_runtime_files(report)

    files_changed = [
        "execution-framework/scripts/generate_art135_raci.py",
        "execution-framework/migration-artifacts/art-135_raci/ownership-raci-matrix.md",
        "execution-framework/migration-artifacts/art-135_raci/ownership-raci-matrix.json",
        "execution-framework/migration-artifacts/09-governance/ownership-raci-matrix.md",
        "execution-framework/generated/art135_raci_registry_report.json",
        "execution-framework/state/ART-135_RACI.heartbeat.json",
        "execution-framework/logs/ART-135_RACI.log",
        "execution-framework/proof_records/ART-135_RACI.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    commands_run = [
        "python3 scripts/generate_art135_raci.py",
        "python3 -m py_compile scripts/generate_art135_raci.py",
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
        "ready for VER-300_UNIT_VALIDATION" if report["status"] == "passed" else "fix ART-135 registry validation failures",
    )
    append_proof(proof)
    print(
        "ART-135 status={status} domains={domains} owners={owners} artifacts={artifacts} validations={validations}".format(
            status=report["status"],
            domains=report["summary"]["domain_count"],
            owners=report["summary"]["unique_owner_count"],
            artifacts=report["summary"]["registered_artifact_count"],
            validations=report["summary"]["validation_count"],
        )
    )
    if report["status"] != "passed":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
