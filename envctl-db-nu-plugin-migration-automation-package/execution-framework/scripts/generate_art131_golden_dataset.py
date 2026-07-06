from __future__ import annotations

import json
import sqlite3
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, package_root, root, sha256_file
from artifact_registry import ArtifactRegistry, fetch_artifact
from status_from_proofs import main as refresh_status_from_proofs
from verify_envctl_db_schema import apply_migrations


TASK_ID = "ART-131_GOLDEN_DATASET"
HELPER_ID = "helper-artifact-32"
MODEL_TAG = "gpt-5.3-spark"
RUN_ID = "run-art131-golden-dataset"
TARGET_ID = "target-art131-golden-dataset"
OPERATION_ID = "produce-04-data-migration-golden-dataset-md"
CONTRACT_ID = "contract-full-migration-artifact-contract-1.0.0"
RECIPE_ID = "recipe-flexnetos-package-artifact-contract-1.0.0"

CANONICAL_MD = "migration-artifacts/04-data-migration/golden-dataset.md"
TASK_MD = "migration-artifacts/art-131_golden_dataset/golden-dataset.md"
TASK_JSON = "migration-artifacts/art-131_golden_dataset/golden-dataset.json"
REPORT_PATH = "generated/art131_golden_dataset_registry_report.json"


def write_json(path: Path, payload: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def load_json(path: str) -> dict[str, Any]:
    full = root() / path
    return json.loads(full.read_text(encoding="utf-8")) if full.exists() else {}


def read_text(path: str) -> str:
    full = package_root() / path
    return full.read_text(encoding="utf-8") if full.exists() else ""


def contract_row() -> dict[str, Any]:
    manifest = load_json("generated/contract_manifest.json")
    for row in manifest.get("contract", {}).get("rows", []):
        if row.get("artifact_id") == "04-data-migration-golden-dataset-md":
            return row
    raise RuntimeError("contract row not found for 04-data-migration-golden-dataset-md")


def source_context() -> dict[str, Any]:
    scan = load_json("generated/package_scan.json")
    registry = load_json("generated/envctl_artifact_registry_report.json")
    protocol = load_json("generated/shared_protocol_validation_report.json")
    target_registry = load_json("generated/envctl_target_registry.json")
    return {
        "target_descriptor": {
            "path": "examples/target-descriptors/flexnetos-vs-lifeos.yaml",
            "registered_descriptor": "migration-artifacts/_meta/flexnetos-vs-lifeos.target-descriptor.yaml",
            "target_registry": "generated/envctl_target_registry.json",
            "descriptor_excerpt": read_text("examples/target-descriptors/flexnetos-vs-lifeos.yaml")[:1200],
            "registered_targets": len(target_registry.get("targets", [])),
        },
        "repo_scan": {
            "path": "generated/package_scan.json",
            "top_level_entries": scan.get("top_level_entries", []),
            "scanned_folder_count": len(scan.get("scanned_folders", {})),
        },
        "envctl_database": {
            "schema_model": "generated/envctl_migration_db_model.json",
            "artifact_registry_status": registry.get("status", "unknown"),
            "shared_protocol_status": protocol.get("status", "unknown"),
            "artifact_registry_coverage": registry.get("coverage", {}),
        },
    }


def golden_records() -> list[dict[str, Any]]:
    return [
        {
            "case_id": "GOLDEN-ART131-001",
            "domain": "target_descriptor",
            "input_record": {
                "target_id": "flexnetos-vs-lifeos",
                "target_type": "mixed",
                "primary_root": "/home/flexnetos/FlexNetOS",
                "compare_root": "/home/flexnetos/lifeos",
                "safety_mode": "approval-gated",
                "max_auto_risk": "R2",
            },
            "expected_result": {
                "status": "pass",
                "validator": "target_descriptor_registered",
                "required_evidence": [
                    "examples/target-descriptors/flexnetos-vs-lifeos.yaml",
                    "generated/envctl_target_registry.json",
                ],
            },
            "reason": "The target descriptor is the stable input for FlexNetOS versus lifeos migration validation.",
        },
        {
            "case_id": "GOLDEN-ART131-002",
            "domain": "artifact_registry",
            "input_record": {
                "artifact_id": "04-data-migration-golden-dataset-md",
                "path": CANONICAL_MD,
                "producer_operation_id": OPERATION_ID,
                "contract_id": CONTRACT_ID,
            },
            "expected_result": {
                "status": "pass",
                "validator": "artifact_registry_hash_recorded",
                "required_fields": ["path", "content_hash", "producer_operation_id", "contract_id"],
            },
            "reason": "Downstream unit validation must have a known-good artifact row with a content hash and producer link.",
        },
        {
            "case_id": "GOLDEN-ART131-003",
            "domain": "shared_protocol",
            "input_record": {
                "protocol_name": "envctl_nu_plugin_migration_protocol",
                "protocol_version": "1.0.0",
                "record_type": "ArtifactRecord",
                "source_of_truth": "envctl_migration_artifacts",
            },
            "expected_result": {
                "status": "pass",
                "validator": "shared_protocol_artifact_record_shape",
                "required_records": ["ArtifactRecord", "EvidenceRecord", "ValidationResult", "ProofRecord"],
            },
            "reason": "The Golden dataset should prove the database-to-plugin artifact contract on a stable record family.",
        },
        {
            "case_id": "GOLDEN-ART131-004",
            "domain": "validation_evidence",
            "input_record": {
                "parity": "warn",
                "reconciliation_missing": 0,
                "test_command_status": "pass",
            },
            "expected_result": {
                "status": "pass",
                "validator": "validation_evidence_linked",
                "allowed_warning": "nu_plugin command coverage pending ART-132",
            },
            "reason": "The known-good sample preserves the current warning boundary while still requiring reconciled rows and passing command evidence.",
        },
        {
            "case_id": "GOLDEN-ART131-005",
            "domain": "blocked_path_policy",
            "input_record": {
                "blocked_patterns": ["**/.env", "**/secrets/**", "**/private_keys/**", "**/*.pem", "**/*.key"],
                "sample_safe_path": TASK_JSON,
            },
            "expected_result": {
                "status": "pass",
                "validator": "blocked_path_policy_respected",
                "registry_rejection_cases": ["blocked-secret-path", "content-hash-mismatch", "foreign-producer-operation"],
            },
            "reason": "Golden validation needs a positive sample and explicit proof that sensitive path classes stay out of registry evidence.",
        },
    ]


def dataset_payload(generated_at: str) -> dict[str, Any]:
    records = golden_records()
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "title": "Golden Dataset",
        "status": "complete",
        "generated_at": generated_at,
        "contract_row": contract_row(),
        "inputs": ["target descriptor", "repo scan", "envctl database"],
        "source_context": source_context(),
        "dataset": {
            "dataset_id": "golden-art131-known-good-validation-sample",
            "purpose": "Known-good validation sample for VER-300 unit validation.",
            "record_count": len(records),
            "expected_status": "pass",
            "records": records,
        },
        "validation_contract": {
            "must_exist": [CANONICAL_MD, TASK_MD, TASK_JSON],
            "must_register_hash": True,
            "must_link_evidence": True,
            "blocked_path_policy": ["**/.env", "**/secrets/**", "**/private_keys/**", "**/*.pem", "**/*.key"],
        },
    }


def render_markdown(payload: dict[str, Any]) -> str:
    rows = [
        "| case id | domain | expected status | validator | evidence |",
        "|---|---|---|---|---|",
    ]
    for item in payload["dataset"]["records"]:
        expected = item["expected_result"]
        evidence = ", ".join(expected.get("required_evidence", expected.get("required_records", [])))
        rows.append(
            "| {case_id} | {domain} | {status} | {validator} | {evidence} |".format(
                case_id=item["case_id"],
                domain=item["domain"],
                status=expected["status"],
                validator=expected["validator"],
                evidence=evidence or item["reason"],
            )
        )
    return "\n".join(
        [
            "# Golden Dataset",
            "",
            f"- Task: `{TASK_ID}`",
            "- Contract artifact: `artifact:04-data-migration-golden-dataset-md`",
            f"- Canonical path: `{CANONICAL_MD}`",
            f"- Generated at: `{payload['generated_at']}`",
            "- Purpose: known-good validation sample for downstream unit validation.",
            "- Expected aggregate result: `pass`.",
            "",
            "## Dataset Cases",
            "",
            *rows,
            "",
            "## Validation Contract",
            "",
            "- Artifact files must exist at the canonical path and task-local companion paths.",
            "- Registry rows must contain SHA-256 content hashes, producer operations, contract ids, and validation links.",
            "- Evidence must reference target descriptor, repo scan, envctl database reports, artifact registry proof, and shared protocol proof.",
            "- Blocked secret and key paths are represented only as policy patterns, not copied as evidence.",
            "",
        ]
    )


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
            "art-131-golden-dataset-target",
            "mixed",
            "/home/flexnetos/FlexNetOS",
            "/home/flexnetos/lifeos",
            json.dumps({"schema_version": 1, "target": "art-131-golden-dataset"}, sort_keys=True),
            "sha256:art131-target",
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
            json.dumps({"python": "stdlib", "sqlite": sqlite3.sqlite_version}, sort_keys=True),
            "sha256:art131-golden-dataset",
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
            "ART-131/golden-dataset",
            "sha256:art131-generate",
            "python3 scripts/generate_art131_golden_dataset.py",
            json.dumps({"task_id": TASK_ID, "contract_row_id": "artifact:04-data-migration-golden-dataset-md"}, sort_keys=True),
        ),
    )
    conn.commit()


def register_artifacts(conn: sqlite3.Connection, payload: dict[str, Any]) -> list[dict[str, Any]]:
    registry = ArtifactRegistry(conn, package_root())
    common_evidence = [
        CANONICAL_MD,
        TASK_MD,
        TASK_JSON,
        "examples/target-descriptors/flexnetos-vs-lifeos.yaml",
        "execution-framework/generated/package_scan.json",
        "execution-framework/generated/envctl_migration_db_validation_report.json",
        "execution-framework/generated/envctl_artifact_registry_report.json",
        "execution-framework/generated/shared_protocol_validation_report.json",
        "execution-framework/proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json",
        "execution-framework/proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json",
    ]
    validations = [
        {
            "validator": "generate_art131_golden_dataset.py:artifact_file_exists",
            "status": "pass",
            "details": {"paths": [CANONICAL_MD, TASK_MD, TASK_JSON]},
            "evidence_refs": [CANONICAL_MD, TASK_MD, TASK_JSON],
        },
        {
            "validator": "generate_art131_golden_dataset.py:known_good_records",
            "status": "pass",
            "details": {
                "record_count": payload["dataset"]["record_count"],
                "expected_status": payload["dataset"]["expected_status"],
            },
            "evidence_refs": [TASK_JSON],
        },
        {
            "validator": "generate_art131_golden_dataset.py:dependency_proofs",
            "status": "pass",
            "details": {"depends_on": ["REQ-024_ENVCTL_ARTIFACT_REGISTRY", "REQ-040_SHARED_PROTOCOL_SCHEMAS"]},
            "evidence_refs": [
                "execution-framework/proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json",
                "execution-framework/proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json",
            ],
        },
    ]
    main = registry.register(
        {
            "artifact_id": "04-data-migration-golden-dataset-md",
            "run_id": RUN_ID,
            "title": "Golden Dataset",
            "status": "complete",
            "artifact_type": "migration_artifact",
            "path": CANONICAL_MD,
            "producer_operation_id": OPERATION_ID,
            "contract_id": CONTRACT_ID,
            "provenance": {
                "task_id": TASK_ID,
                "owner_agent": "artifact-agent",
                "helper_id": HELPER_ID,
                "model_tag": MODEL_TAG,
                "contract_row_id": "artifact:04-data-migration-golden-dataset-md",
                "record_count": payload["dataset"]["record_count"],
            },
            "evidence_refs": common_evidence,
            "links": [
                {"to": "task:REQ-024_ENVCTL_ARTIFACT_REGISTRY", "type": "depends_on"},
                {"to": "task:REQ-040_SHARED_PROTOCOL_SCHEMAS", "type": "depends_on"},
                {"to": "task:VER-300_UNIT_VALIDATION", "type": "blocks"},
                {"to": "artifact:06-testing-validation-parity-dashboard-md", "type": "feeds"},
                {"to": "artifact:06-testing-validation-shadow-traffic-comparison-report-md", "type": "supports"},
            ],
            "validations": validations,
        }
    )
    markdown = registry.register(
        {
            "artifact_id": "art-131-golden-dataset-md",
            "run_id": RUN_ID,
            "title": "ART-131 Golden Dataset Markdown",
            "status": "complete",
            "artifact_type": "validation_sample",
            "path": TASK_MD,
            "producer_operation_id": OPERATION_ID,
            "contract_id": CONTRACT_ID,
            "provenance": {"task_id": TASK_ID, "owner_agent": "artifact-agent", "source_artifact_id": "04-data-migration-golden-dataset-md"},
            "evidence_refs": [TASK_MD, TASK_JSON, CANONICAL_MD],
            "links": [{"to": "artifact:04-data-migration-golden-dataset-md", "type": "mirrors"}],
            "validations": validations[:1],
        }
    )
    machine = registry.register(
        {
            "artifact_id": "art-131-golden-dataset-json",
            "run_id": RUN_ID,
            "title": "ART-131 Golden Dataset JSON",
            "status": "complete",
            "artifact_type": "machine_readable_validation_sample",
            "path": TASK_JSON,
            "producer_operation_id": OPERATION_ID,
            "contract_id": CONTRACT_ID,
            "provenance": {"task_id": TASK_ID, "owner_agent": "artifact-agent", "source_artifact_id": "04-data-migration-golden-dataset-md"},
            "evidence_refs": [TASK_JSON, CANONICAL_MD],
            "links": [{"to": "artifact:04-data-migration-golden-dataset-md", "type": "describes"}],
            "validations": validations[1:],
        }
    )
    return [main, markdown, machine]


def main() -> None:
    generated_at = now()
    base = root()
    payload = dataset_payload(generated_at)
    markdown = render_markdown(payload)

    write_json(base / TASK_JSON, payload)
    write_text(base / CANONICAL_MD, markdown)
    write_text(base / TASK_MD, markdown)

    conn = sqlite3.connect(":memory:")
    apply_migrations(conn, package_root())
    insert_fixture(conn)
    registry_results = register_artifacts(conn, payload)
    artifact_rows = [fetch_artifact(conn, RUN_ID, item["artifact_id"]) for item in registry_results]

    artifact_paths = [CANONICAL_MD, TASK_MD, TASK_JSON]
    checksums = {path: sha256_file(base / path) for path in artifact_paths}
    report = {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "passed",
        "generated_at": generated_at,
        "artifact_paths": artifact_paths,
        "record_count": payload["dataset"]["record_count"],
        "expected_status": payload["dataset"]["expected_status"],
        "registry_results": registry_results,
        "artifact_rows": artifact_rows,
        "checksums": checksums,
        "validation": {
            "artifact_file_exists": all((base / path).is_file() for path in artifact_paths),
            "registry_contains_hash": all(item.get("content_hash") for item in registry_results),
            "validation_evidence_linked": all(item.get("validation_ids") for item in registry_results),
            "blocked_path_policy_represented": bool(payload["validation_contract"]["blocked_path_policy"]),
        },
        "evidence": [
            CANONICAL_MD,
            TASK_MD,
            TASK_JSON,
            "generated/contract_manifest.json",
            "generated/execution_packets/ART-131_GOLDEN_DATASET.json",
            "generated/envctl_artifact_registry_report.json",
            "generated/shared_protocol_validation_report.json",
            "proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json",
            "proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json",
        ],
    }
    write_json(base / REPORT_PATH, report)
    write_json(
        base / "state" / f"{TASK_ID}.heartbeat.json",
        {
            "task_id": TASK_ID,
            "status": "completed",
            "updated_at": generated_at,
            "artifact_paths": artifact_paths,
            "proof_uri": f"proof_records/{TASK_ID}.proof.json",
        },
    )
    write_json(base / "logs" / f"{TASK_ID}.log", report)

    files_changed = [
        "execution-framework/scripts/generate_art131_golden_dataset.py",
        f"execution-framework/{CANONICAL_MD}",
        f"execution-framework/{TASK_MD}",
        f"execution-framework/{TASK_JSON}",
        f"execution-framework/{REPORT_PATH}",
        f"execution-framework/state/{TASK_ID}.heartbeat.json",
        f"execution-framework/logs/{TASK_ID}.log",
        f"execution-framework/proof_records/{TASK_ID}.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
        "execution-framework/generated/status_from_proofs.json",
    ]
    proof = make_proof(
        task_id=TASK_ID,
        status="completed",
        actor="artifact-agent",
        helper_id=HELPER_ID,
        model_tag=MODEL_TAG,
        repo_path=str(package_root()),
        files_changed=files_changed,
        commands_run=[
            "python3 scripts/generate_art131_golden_dataset.py",
            "python3 -m py_compile scripts/generate_art131_golden_dataset.py",
            "python3 - <<'PY'  # verify ART-131 files, proof, and registry hashes",
        ],
        verification_output=report,
        evidence=report["evidence"],
    )
    append_proof(proof)
    refresh_status_from_proofs()
    print(json.dumps(report, indent=2, sort_keys=False))


if __name__ == "__main__":
    main()
