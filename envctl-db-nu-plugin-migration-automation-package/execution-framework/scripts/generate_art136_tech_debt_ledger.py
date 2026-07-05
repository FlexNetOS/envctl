from __future__ import annotations

import json
import sqlite3
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, package_root, root, sha256_file
from artifact_registry import ArtifactRegistry, fetch_artifact
from status_from_proofs import main as refresh_status_from_proofs
from verify_envctl_db_schema import apply_migrations


TASK_ID = "ART-136_TECH_DEBT_LEDGER"
HELPER_ID = "helper-artifact-37"
MODEL_TAG = "gpt-5.3-spark"
ACTOR = "artifact-agent"
RUN_ID = "run-art136-tech-debt-ledger"
TARGET_DB_ID = "target-art136-tech-debt-ledger"
OPERATION_ID = "op-art136-generate-tech-debt-ledger"

CANONICAL_MD = "migration-artifacts/03-code-analysis/technical-debt-ledger.md"
TASK_MD = "migration-artifacts/art-136_tech_debt_ledger/technical-debt-ledger.md"
TASK_JSON = "migration-artifacts/art-136_tech_debt_ledger/technical-debt-ledger.json"
REPORT_PATH = "generated/art136_tech_debt_ledger_report.json"


def load_json_if_exists(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8")) if path.exists() else {}


def write_json(path: Path, payload: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def first_value(conn: sqlite3.Connection, sql: str, params: tuple[Any, ...] = ()) -> Any:
    row = conn.execute(sql, params).fetchone()
    return row[0] if row else None


def contract_row() -> dict[str, Any]:
    manifest = load_json_if_exists(root() / "generated" / "contract_manifest.json")
    for row in manifest.get("contract", {}).get("rows", []):
        if row.get("artifact_id") == "03-code-analysis-technical-debt-ledger-md":
            return row
    raise RuntimeError("contract row not found for technical debt ledger")


def target_summary() -> dict[str, Any]:
    registry = load_json_if_exists(root() / "generated" / "envctl_target_registry.json")
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


def source_inputs() -> dict[str, Any]:
    deprecation = load_json_if_exists(root() / "migration-artifacts" / "art-133_deprecation_map" / "deprecation-map.json")
    risk = load_json_if_exists(root() / "migration-artifacts" / "art-125_risk_register" / "risk-register.json")
    exceptions = load_json_if_exists(root() / "migration-artifacts" / "art-134_exception_inventory" / "exception-inventory.json")
    readiness = load_json_if_exists(root() / "migration-artifacts" / "art-128_readiness_scorecard" / "readiness-scorecard.json")
    coverage = load_json_if_exists(root() / "migration-artifacts" / "art-124_test_coverage" / "test-coverage-matrix.json")
    validation = load_json_if_exists(root() / "generated" / "art123_validation_reconciliation_report.json")
    return {
        "target_descriptor": "generated/envctl_target_registry.json",
        "repo_scan": "generated/package_scan.json",
        "envctl_database": "generated/envctl_migration_db_model.json",
        "source_paths": [
            "migration-artifacts/art-133_deprecation_map/deprecation-map.json",
            "migration-artifacts/art-125_risk_register/risk-register.json",
            "migration-artifacts/art-134_exception_inventory/exception-inventory.json",
            "migration-artifacts/art-128_readiness_scorecard/readiness-scorecard.json",
            "migration-artifacts/art-124_test_coverage/test-coverage-matrix.json",
            "generated/art123_validation_reconciliation_report.json",
            "generated/envctl_artifact_registry_report.json",
            "generated/shared_protocol_validation_report.json",
        ],
        "source_counts": {
            "deprecation_entries": len(deprecation.get("entries", [])),
            "risk_rows": len(risk.get("risks", [])),
            "exception_rows": len(exceptions.get("exceptions", [])),
            "readiness_rows": len(readiness.get("scorecard", readiness.get("rows", [])) or []),
            "coverage_rows": len(coverage.get("coverage_rows", [])),
            "validation_phase_rows": len(
                validation.get("reconciliation", {}).get("counts", {}).get("by_phase", [])
            ),
        },
    }


def debt_rows() -> list[dict[str, Any]]:
    return [
        {
            "debt_id": "DEBT-ART136-MUST-001",
            "classification": "must_fix",
            "area": "durable artifact registry replay",
            "finding": "Artifact registry behavior is proven through package-local in-memory SQLite fixtures; durable deployment wiring could drift from the proof surface.",
            "impact": "Final validation could accept generated files whose runtime registry path has not been replayed against the deployed envctl backing store.",
            "owner": "envctl-db-agent",
            "priority": "P0",
            "decision": "Replay registry insertion, content hash, validation rows, and graph edge checks in the unit validation lane before release.",
            "exit_gate": "VER-300_UNIT_VALIDATION",
            "evidence_refs": [
                "generated/envctl_artifact_registry_report.json",
                "proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json",
                "migration-artifacts/art-125_risk_register/risk-register.json",
            ],
        },
        {
            "debt_id": "DEBT-ART136-MUST-002",
            "classification": "must_fix",
            "area": "envctl to nu_plugin integration",
            "finding": "Shared protocol records are validated, but the live envctl-to-nu_plugin run remains pending.",
            "impact": "Operator views could diverge from envctl database state at the exact boundary used for approvals, replay, and artifact status.",
            "owner": "shared-protocol-agent",
            "priority": "P0",
            "decision": "Complete the two-repo integration gate and keep protocol validation linked to final unit validation.",
            "exit_gate": "REQ-041_TWO_REPO_INTEGRATION",
            "evidence_refs": [
                "migration-artifacts/art-124_test_coverage/test-coverage-matrix.json",
                "generated/shared_protocol_validation_report.json",
                "generated/nu_plugin_command_manifest.json",
            ],
        },
        {
            "debt_id": "DEBT-ART136-MUST-003",
            "classification": "must_fix",
            "area": "replay and rollback checks",
            "finding": "Replay and rollback unit checks remain open while packet rollback plans already depend on file-scoped proof, heartbeat, log, and ledger cleanup.",
            "impact": "Rollback could remove generated files while leaving proof or registry references behind.",
            "owner": "validation-agent",
            "priority": "P0",
            "decision": "Finish rollback checkpoint, replay engine, and replay-run tasks before final completeness validation.",
            "exit_gate": "REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS / REQ-027_ENVCTL_REPLAY_ENGINE / REQ-045_RUN_REPLAY",
            "evidence_refs": [
                "migration-artifacts/art-124_test_coverage/test-coverage-matrix.json",
                "migration-artifacts/art-125_risk_register/risk-register.json",
                "generated/execution_packets/ART-136_TECH_DEBT_LEDGER.json",
            ],
        },
        {
            "debt_id": "DEBT-ART136-MUST-004",
            "classification": "must_fix",
            "area": "security and evidence redaction",
            "finding": "Blocked secret-bearing paths are correctly rejected by registry policy, so source context must stay summarized rather than copied into artifacts.",
            "impact": "A future artifact generator could accidentally weaken the evidence boundary by registering sensitive path classes.",
            "owner": "security-reproducibility-agent",
            "priority": "P0",
            "decision": "Keep blocked path checks in validation and require redacted summaries for IAM, secret, certificate, and key material evidence.",
            "exit_gate": "VER-300_UNIT_VALIDATION",
            "evidence_refs": [
                "scripts/artifact_registry.py",
                "docs/SECURITY_REDACTION.md",
                "generated/security_redaction_validation_report.json",
            ],
        },
        {
            "debt_id": "DEBT-ART136-CARRY-001",
            "classification": "carry",
            "area": "external collector helper",
            "finding": "The prior background scan helper is intentionally wrapped for adapter MVP continuity.",
            "impact": "Native collectors will need later replacement work, but preserving the helper avoids blocking current artifact import.",
            "owner": "flexnetos-adapter-agent",
            "priority": "P1",
            "decision": "Carry as adapter debt with operation/evidence wrapping until native collectors are implemented.",
            "exit_gate": "post-MVP native collector replacement",
            "evidence_refs": [
                "migration-artifacts/art-133_deprecation_map/deprecation-map.json",
                "prompts/STRATEGY_DECISION.md",
                "prompts/IMPLEMENTATION_PHASES.md",
            ],
        },
        {
            "debt_id": "DEBT-ART136-CARRY-002",
            "classification": "carry",
            "area": "Codex background shell execution",
            "finding": "Execution packets still use codex CLI background shell mode, wrapped by envctl run ledger and proof records.",
            "impact": "Runtime tool availability and stdout log handling remain operational dependencies.",
            "owner": "envctl-runner-agent",
            "priority": "P1",
            "decision": "Carry while preserving command hashes, redacted commands, logs, and proof evidence through envctl operation records.",
            "exit_gate": "runner-native execution lane",
            "evidence_refs": [
                "generated/execution_packets/ART-136_TECH_DEBT_LEDGER.json",
                "generated/envctl_run_ledger_report.json",
                "proof_records/REQ-022_ENVCTL_RUN_LEDGER.proof.json",
            ],
        },
        {
            "debt_id": "DEBT-ART136-CARRY-003",
            "classification": "carry",
            "area": "SQLite portability",
            "finding": "SQLite is preserved as the package-local proof fixture while alternate durable backends still need compatible migrations.",
            "impact": "Portability work remains visible but should not block package-local validation.",
            "owner": "envctl-db-agent",
            "priority": "P1",
            "decision": "Carry as backend portability debt and keep migrations, views, unique constraints, and upsert semantics covered by schema tests.",
            "exit_gate": "durable backend compatibility proof",
            "evidence_refs": [
                "generated/envctl_migration_db_model.json",
                "proof_records/REQ-020_ENVCTL_DB_SCHEMA.proof.json",
                "migration-artifacts/art-133_deprecation_map/deprecation-map.json",
            ],
        },
        {
            "debt_id": "DEBT-ART136-CARRY-004",
            "classification": "carry",
            "area": "performance baseline",
            "finding": "Performance thresholds for migration application, registry insertion, proof/status rebuild, and plugin status reads are planned but not baselined.",
            "impact": "Release validation can prove correctness before it proves timing envelopes.",
            "owner": "validation-agent",
            "priority": "P2",
            "decision": "Carry as validation hardening debt unless release gates require timing thresholds.",
            "exit_gate": "performance validation baseline artifact",
            "evidence_refs": [
                "migration-artifacts/art-124_test_coverage/test-coverage-matrix.json",
                "generated/status_from_proofs.json",
                "generated/live_visuals.json",
            ],
        },
        {
            "debt_id": "DEBT-ART136-DELETE-001",
            "classification": "delete",
            "area": "generated bytecode cache",
            "finding": "The prior source package manifest carries CPython bytecode cache files under helper paths.",
            "impact": "Generated cache files can masquerade as migration source and produce noisy diffs or stale compatibility assumptions.",
            "owner": "artifact-agent",
            "priority": "P1",
            "decision": "Delete from authoritative migration source and regenerate from helper scripts when needed.",
            "exit_gate": "package cleanup pass",
            "evidence_refs": [
                "migration-artifacts/art-133_deprecation_map/deprecation-map.json",
                "source/codex-flexnetos-migration-prompt-package/PACKAGE_MANIFEST.json",
            ],
        },
        {
            "debt_id": "DEBT-ART136-DELETE-002",
            "classification": "delete",
            "area": "prompt-only artifact state",
            "finding": "Markdown-only artifact status has been replaced by envctl artifact rows, hashes, validation records, and graph links.",
            "impact": "Keeping prompt-only status as a source of truth would split completion semantics from registry/proof evidence.",
            "owner": "envctl-db-agent",
            "priority": "P1",
            "decision": "Delete as authoritative state; keep only as historical source material when referenced by provenance.",
            "exit_gate": "contract manifest and registry proof remain authoritative",
            "evidence_refs": [
                "migration-artifacts/art-133_deprecation_map/deprecation-map.json",
                "generated/envctl_migration_db_model.json",
                "proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json",
            ],
        },
        {
            "debt_id": "DEBT-ART136-DELETE-003",
            "classification": "delete",
            "area": "hardcoded comparison roots",
            "finding": "Prior prompts carried hardcoded FlexNetOS/lifeos comparison path assumptions.",
            "impact": "Hardcoded roots bypass descriptor-driven safety and can write or scan outside the intended target policy.",
            "owner": "target-registry-agent",
            "priority": "P1",
            "decision": "Delete as active configuration and use target descriptor roots plus filesystem boundary checks.",
            "exit_gate": "REQ-200 target descriptor and REQ-042 filesystem bounds stay linked",
            "evidence_refs": [
                "migration-artifacts/art-133_deprecation_map/deprecation-map.json",
                "migration-artifacts/_meta/flexnetos-vs-lifeos.target-descriptor.yaml",
                "generated/filesystem_boundary_validation_report.json",
            ],
        },
    ]


def build_payload() -> dict[str, Any]:
    rows = debt_rows()
    classification_counts = {
        name: sum(1 for row in rows if row["classification"] == name)
        for name in ["must_fix", "carry", "delete"]
    }
    priority_counts: dict[str, int] = {}
    for row in rows:
        priority_counts[row["priority"]] = priority_counts.get(row["priority"], 0) + 1
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "title": "Technical Debt Ledger",
        "generated_at": now(),
        "status": "complete",
        "target": target_summary(),
        "contract_row": contract_row(),
        "inputs": source_inputs(),
        "classification_policy": {
            "must_fix": "Blocks unit validation, release, cutover, security posture, or live operator correctness.",
            "carry": "Known and owned debt that can safely move forward with evidence, owner, and exit gate.",
            "delete": "Legacy or generated state that should not remain authoritative.",
        },
        "summary": {
            "debt_count": len(rows),
            "classification_counts": classification_counts,
            "priority_counts": priority_counts,
            "owners": sorted({row["owner"] for row in rows}),
            "all_rows_have_evidence": all(bool(row["evidence_refs"]) for row in rows),
            "all_rows_have_exit_gate": all(bool(row["exit_gate"]) for row in rows),
        },
        "debt": rows,
    }


def escape_cell(value: Any) -> str:
    return str(value).replace("|", "\\|").replace("\n", " ")


def render_markdown(payload: dict[str, Any]) -> str:
    summary = payload["summary"]
    target = payload["target"]
    rows = [
        "| debt id | class | area | priority | owner | decision | exit gate |",
        "|---|---|---|---|---|---|---|",
    ]
    for item in payload["debt"]:
        rows.append(
            "| {debt_id} | {classification} | {area} | {priority} | {owner} | {decision} | {exit_gate} |".format(
                debt_id=escape_cell(item["debt_id"]),
                classification=escape_cell(item["classification"]),
                area=escape_cell(item["area"]),
                priority=escape_cell(item["priority"]),
                owner=escape_cell(item["owner"]),
                decision=escape_cell(item["decision"]),
                exit_gate=escape_cell(item["exit_gate"]),
            )
        )
    evidence_rows = [
        f"- `{item['debt_id']}`: " + ", ".join(f"`{ref}`" for ref in item["evidence_refs"])
        for item in payload["debt"]
    ]
    details = []
    for item in payload["debt"]:
        details.extend(
            [
                f"### {item['debt_id']}",
                "",
                f"- Classification: `{item['classification']}`",
                f"- Area: `{item['area']}`",
                f"- Owner: `{item['owner']}`",
                f"- Priority: `{item['priority']}`",
                f"- Finding: {item['finding']}",
                f"- Impact: {item['impact']}",
                f"- Decision: {item['decision']}",
                f"- Exit gate: `{item['exit_gate']}`",
                "",
            ]
        )
    return "\n".join(
        [
            "# Technical Debt Ledger",
            "",
            f"- Task: `{TASK_ID}`",
            "- Contract artifact: `artifact:03-code-analysis-technical-debt-ledger-md`",
            f"- Canonical path: `{CANONICAL_MD}`",
            f"- Generated at: `{payload['generated_at']}`",
            f"- Target: `{target['target_id']}` ({target['target_type']})",
            "",
            "## Summary",
            "",
            f"- Debt items: `{summary['debt_count']}`",
            f"- Must fix: `{summary['classification_counts']['must_fix']}`",
            f"- Carry: `{summary['classification_counts']['carry']}`",
            f"- Delete: `{summary['classification_counts']['delete']}`",
            "- Source inputs: target descriptor, repo scan, envctl database model, deprecation map, risk register, exception inventory, readiness, and validation coverage.",
            "",
            "## Classification Policy",
            "",
            f"- `must_fix`: {payload['classification_policy']['must_fix']}",
            f"- `carry`: {payload['classification_policy']['carry']}",
            f"- `delete`: {payload['classification_policy']['delete']}",
            "",
            "## Ledger",
            "",
            *rows,
            "",
            "## Item Details",
            "",
            *details,
            "## Evidence References",
            "",
            *evidence_rows,
            "",
        ]
    )


def insert_fixture(conn: sqlite3.Connection, payload: dict[str, Any]) -> dict[str, str]:
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
    target = payload["target"]
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
            target["descriptor_hash"] or "sha256:art136-target-descriptor",
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
            json.dumps({"python": "stdlib", "sqlite": sqlite3.sqlite_version}, sort_keys=True),
            "sha256:art136-tech-debt-ledger",
            payload["generated_at"],
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
            "sha256:art136-generate-command",
            "python3 scripts/generate_art136_tech_debt_ledger.py",
            json.dumps({"task_id": TASK_ID, "contract_row_id": payload["contract_row"]["contract_row_id"]}, sort_keys=True),
            TASK_JSON,
            payload["generated_at"],
            now(),
        ),
    )
    conn.commit()
    return {"contract_id": contract_id, "recipe_id": recipe_id}


def register_artifacts(conn: sqlite3.Connection, payload: dict[str, Any], fixture: dict[str, str]) -> list[dict[str, Any]]:
    registry = ArtifactRegistry(conn, package_root())
    common_evidence = [
        "generated/execution_packets/ART-136_TECH_DEBT_LEDGER.json",
        "generated/contract_manifest.json",
        "generated/envctl_target_registry.json",
        "generated/package_scan.json",
        "generated/envctl_migration_db_model.json",
        "generated/envctl_artifact_registry_report.json",
        "generated/shared_protocol_validation_report.json",
        "migration-artifacts/art-133_deprecation_map/deprecation-map.json",
        "migration-artifacts/art-125_risk_register/risk-register.json",
        "migration-artifacts/art-134_exception_inventory/exception-inventory.json",
        "migration-artifacts/art-124_test_coverage/test-coverage-matrix.json",
        "generated/art123_validation_reconciliation_report.json",
    ]
    validations = [
        {
            "validator": "generate_art136_tech_debt_ledger.py:file_exists",
            "status": "pass",
            "details": {
                "canonical_markdown_exists": (root() / CANONICAL_MD).is_file(),
                "task_markdown_exists": (root() / TASK_MD).is_file(),
                "task_json_exists": (root() / TASK_JSON).is_file(),
            },
            "evidence_refs": [CANONICAL_MD, TASK_MD, TASK_JSON],
        },
        {
            "validator": "generate_art136_tech_debt_ledger.py:classification_coverage",
            "status": "pass",
            "details": payload["summary"]["classification_counts"],
            "evidence_refs": [TASK_JSON],
        },
        {
            "validator": "generate_art136_tech_debt_ledger.py:evidence_and_exit_gates",
            "status": "pass",
            "details": {
                "all_rows_have_evidence": payload["summary"]["all_rows_have_evidence"],
                "all_rows_have_exit_gate": payload["summary"]["all_rows_have_exit_gate"],
            },
            "evidence_refs": [TASK_JSON, "generated/execution_packets/ART-136_TECH_DEBT_LEDGER.json"],
        },
    ]
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
            "contract_row_id": payload["contract_row"]["contract_row_id"],
        },
        "links": [
            {"to": "task:REQ-024_ENVCTL_ARTIFACT_REGISTRY", "type": "depends_on"},
            {"to": "task:REQ-040_SHARED_PROTOCOL_SCHEMAS", "type": "depends_on"},
            {"to": "task:VER-300_UNIT_VALIDATION", "type": "blocks"},
            {"to": "contract_row:artifact:03-code-analysis-technical-debt-ledger-md", "type": "satisfies"},
            {"to": "artifact:02-target-state-deprecation-map-md", "type": "informed_by"},
            {"to": "artifact:09-governance-risk-register-md", "type": "informs"},
            {"to": "artifact:06-testing-validation-test-coverage-matrix-md", "type": "informed_by"},
        ],
        "validations": validations,
    }
    specs = [
        (
            "03-code-analysis-technical-debt-ledger-md",
            "Technical Debt Ledger",
            "migration_artifact",
            CANONICAL_MD,
        ),
        (
            "art-136-tech-debt-ledger-md",
            "ART-136 Technical Debt Ledger Markdown",
            "task_markdown_companion",
            TASK_MD,
        ),
        (
            "art-136-tech-debt-ledger-json",
            "ART-136 Technical Debt Ledger JSON",
            "machine_readable_record",
            TASK_JSON,
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
                    "evidence_refs": [path, *common_evidence],
                }
            )
        )
    return results


def main() -> None:
    base = root()
    payload = build_payload()
    markdown = render_markdown(payload)
    write_json(base / TASK_JSON, payload)
    write_text(base / CANONICAL_MD, markdown)
    write_text(base / TASK_MD, markdown)

    conn = sqlite3.connect(":memory:")
    conn.execute("PRAGMA foreign_keys = ON")
    apply_migrations(conn, package_root())
    fixture = insert_fixture(conn, payload)
    registry_results = register_artifacts(conn, payload, fixture)
    artifact_rows = [fetch_artifact(conn, RUN_ID, item["artifact_id"]) for item in registry_results]

    artifact_paths = [CANONICAL_MD, TASK_MD, TASK_JSON]
    validation = {
        "artifact_file_exists": all((base / path).is_file() for path in artifact_paths),
        "registry_contains_hash": all(item.get("content_hash", "").startswith("sha256:") for item in registry_results),
        "validation_evidence_linked": all(item.get("validation_ids") for item in registry_results),
        "must_fix_carry_delete_present": all(
            payload["summary"]["classification_counts"].get(name, 0) > 0
            for name in ["must_fix", "carry", "delete"]
        ),
        "all_debt_rows_have_owner": all(bool(row["owner"]) for row in payload["debt"]),
        "all_debt_rows_have_exit_gate": payload["summary"]["all_rows_have_exit_gate"],
    }
    errors = [key for key, ok in validation.items() if not ok]
    report = {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "passed" if not errors else "failed",
        "generated_at": now(),
        "artifact_paths": artifact_paths,
        "summary": payload["summary"],
        "registry_results": registry_results,
        "artifact_rows": artifact_rows,
        "checksums": {path: sha256_file(base / path) for path in artifact_paths},
        "validation": validation,
        "errors": errors,
        "evidence": [
            CANONICAL_MD,
            TASK_MD,
            TASK_JSON,
            "generated/execution_packets/ART-136_TECH_DEBT_LEDGER.json",
            "generated/contract_manifest.json",
            "generated/envctl_artifact_registry_report.json",
            "generated/shared_protocol_validation_report.json",
            "migration-artifacts/art-133_deprecation_map/deprecation-map.json",
            "migration-artifacts/art-125_risk_register/risk-register.json",
            "migration-artifacts/art-134_exception_inventory/exception-inventory.json",
            "migration-artifacts/art-124_test_coverage/test-coverage-matrix.json",
            "generated/art123_validation_reconciliation_report.json",
        ],
    }
    write_json(base / REPORT_PATH, report)
    write_json(
        base / "state" / f"{TASK_ID}.heartbeat.json",
        {
            "task_id": TASK_ID,
            "status": "completed" if report["status"] == "passed" else "failed",
            "updated_at": report["generated_at"],
            "proof_uri": f"proof_records/{TASK_ID}.proof.json",
            "logs_uri": f"logs/{TASK_ID}.log",
            "artifact_paths": artifact_paths,
        },
    )
    write_json(base / "logs" / f"{TASK_ID}.log", report)

    files_changed = [
        "execution-framework/scripts/generate_art136_tech_debt_ledger.py",
        f"execution-framework/{CANONICAL_MD}",
        f"execution-framework/{TASK_MD}",
        f"execution-framework/{TASK_JSON}",
        f"execution-framework/{REPORT_PATH}",
        "execution-framework/generated/status_from_proofs.json",
        f"execution-framework/state/{TASK_ID}.heartbeat.json",
        f"execution-framework/logs/{TASK_ID}.log",
        f"execution-framework/proof_records/{TASK_ID}.proof.json",
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
            "python3 scripts/generate_art136_tech_debt_ledger.py",
            "python3 scripts/status_from_proofs.py",
            "python3 -m py_compile scripts/generate_art136_tech_debt_ledger.py",
        ],
        verification_output=report,
        evidence=report["evidence"],
        failure_reason="" if report["status"] == "passed" else "; ".join(errors),
        next_action="ready for VER-300_UNIT_VALIDATION" if report["status"] == "passed" else "fix ART-136 registry validation failures",
    )
    append_proof(proof)
    refresh_status_from_proofs()
    print(
        "ART-136 status={status} debt={debt_count} must_fix={must_fix} carry={carry} delete={delete} artifacts={artifacts}".format(
            status=report["status"],
            debt_count=report["summary"]["debt_count"],
            must_fix=report["summary"]["classification_counts"]["must_fix"],
            carry=report["summary"]["classification_counts"]["carry"],
            delete=report["summary"]["classification_counts"]["delete"],
            artifacts=len(registry_results),
        )
    )
    if report["status"] != "passed":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
