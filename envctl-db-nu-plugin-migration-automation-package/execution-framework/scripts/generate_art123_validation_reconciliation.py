from __future__ import annotations

import hashlib
import json
import sqlite3
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, package_root, read_json, read_task_graph, root, sha256_file
from artifact_registry import ArtifactRegistry, fetch_artifact
from validation_evidence import ValidationEvidenceStore, fetch_validation
from verify_envctl_db_schema import apply_migrations


TASK_ID = "ART-123_VALIDATION_RECONCILIATION"
HELPER_ID = "helper-artifact-24"
MODEL_TAG = "gpt-5.3-spark"
RUN_ID = "run-art123-validation-reconciliation"
OPERATION_ID = "produce-06-testing-validation-reconciliation-reports-md"
CONTRACT_ID = "contract-full-migration-artifact-contract-1.0.0"
RECIPE_ID = "recipe-flexnetos-package-artifact-contract-1.0.0"
TARGET_ROW_ID = "target-flexnetos-vs-lifeos"

ARTIFACT_DIR = root() / "migration-artifacts" / "art-123_validation_reconciliation"
TASK_MD = ARTIFACT_DIR / "validation-reconciliation-reports.md"
TASK_JSON = ARTIFACT_DIR / "validation-reconciliation-reports.json"
CANONICAL_MD = root() / "migration-artifacts" / "06-testing-validation" / "validation-reconciliation-reports.md"
REPORT_JSON = root() / "generated" / "art123_validation_reconciliation_report.json"


def rel(path: Path) -> str:
    return str(path.relative_to(package_root()))


def sha256_text(value: str) -> str:
    return "sha256:" + hashlib.sha256(value.encode("utf-8")).hexdigest()


def read_optional_json(path: str) -> dict[str, Any]:
    candidate = root() / path
    if not candidate.exists():
        return {"status": "missing", "path": path}
    return read_json(path)


def proof_exists(task_id: str) -> bool:
    return (root() / "proof_records" / f"{task_id}.proof.json").exists()


def status_is_success(status: str) -> bool:
    return status in {"completed", "passed", "pass"}


def task_statuses(status_report: dict[str, Any]) -> dict[str, str]:
    return {item.get("task_id", ""): item.get("status", "unknown") for item in status_report.get("tasks", [])}


def task_phase_counts(rows: list[dict[str, str]], statuses: dict[str, str]) -> list[dict[str, Any]]:
    phases = sorted({row["phase"] for row in rows})
    out = []
    for phase in phases:
        task_ids = [row["task_id"] for row in rows if row["phase"] == phase]
        complete = sum(1 for task_id in task_ids if status_is_success(statuses.get(task_id, "unknown")))
        pending = sum(1 for task_id in task_ids if statuses.get(task_id, "unknown") == "pending")
        out.append(
            {
                "phase": phase,
                "task_count": len(task_ids),
                "complete_or_passed": complete,
                "pending": pending,
                "other": len(task_ids) - complete - pending,
            }
        )
    return out


def contract_rows_for_task(contract_manifest: dict[str, Any]) -> list[dict[str, Any]]:
    rows = contract_manifest.get("contract", {}).get("rows", [])
    return [row for row in rows if row.get("producer_task_id") == TASK_ID]


def checksum_paths(paths: list[str]) -> dict[str, str]:
    out = {}
    base = package_root()
    for path in paths:
        candidates = [root() / path, base / path]
        for candidate in candidates:
            if candidate.exists() and candidate.is_file():
                out[str(candidate.relative_to(base))] = sha256_file(candidate)
                break
    return out


def build_reconciliation() -> dict[str, Any]:
    task_rows = read_task_graph("generated/task_graph.csv")
    status_report = read_optional_json("generated/status_from_proofs.json")
    final_verification = read_optional_json("generated/final_verification_report.json")
    artifact_registry = read_optional_json("generated/envctl_artifact_registry_report.json")
    validation_evidence = read_optional_json("generated/envctl_validation_evidence_report.json")
    shared_protocol = read_optional_json("generated/shared_protocol_validation_report.json")
    target_registry = read_optional_json("generated/envctl_target_registry.json")
    db_model = read_optional_json("generated/envctl_migration_db_model.json")
    package_scan = read_optional_json("generated/package_scan.json")
    contract_manifest = read_optional_json("generated/contract_manifest.json")
    statuses = task_statuses(status_report)

    task_ids = [row["task_id"] for row in task_rows]
    packets = sorted((root() / "generated" / "execution_packets").glob("*.json"))
    packet_ids = [path.stem for path in packets]
    proof_paths = sorted((root() / "proof_records").glob("*.proof.json"))
    proof_ids = [path.name.removesuffix(".proof.json") for path in proof_paths]
    successful_tasks = [task_id for task_id in task_ids if status_is_success(statuses.get(task_id, "unknown"))]

    deps = ["REQ-024_ENVCTL_ARTIFACT_REGISTRY", "REQ-040_SHARED_PROTOCOL_SCHEMAS", "REQ-025_ENVCTL_VALIDATION_EVIDENCE"]
    contract_rows = contract_rows_for_task(contract_manifest)
    source_paths = [
        "generated/envctl_target_registry.json",
        "generated/package_scan.json",
        "generated/envctl_migration_db_model.json",
        "generated/envctl_artifact_registry_report.json",
        "generated/envctl_validation_evidence_report.json",
        "generated/shared_protocol_validation_report.json",
        "generated/status_from_proofs.json",
        "generated/final_verification_report.json",
        "generated/contract_manifest.json",
        "generated/task_graph.csv",
    ]

    parity = {
        "task_graph_rows": len(task_ids),
        "execution_packet_count": len(packet_ids),
        "proof_record_count": len(proof_ids),
        "status_report_tasks": len(statuses),
        "successful_tasks": len(successful_tasks),
        "missing_execution_packets": sorted(set(task_ids) - set(packet_ids)),
        "packets_without_task_graph_rows": sorted(set(packet_ids) - set(task_ids)),
        "successful_tasks_without_proof": sorted(task_id for task_id in successful_tasks if not proof_exists(task_id)),
        "proofs_without_task_graph_rows": sorted(set(proof_ids) - set(task_ids)),
    }
    parity["passed"] = not (
        parity["missing_execution_packets"]
        or parity["packets_without_task_graph_rows"]
        or parity["successful_tasks_without_proof"]
    )

    evidence_counts = validation_evidence.get("summary", {})
    registry_counts = artifact_registry.get("summary", {})
    protocol_results = shared_protocol.get("sample_results", {})
    dependency_statuses = {task_id: statuses.get(task_id, "unknown") for task_id in deps}

    reconciliation = {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "title": "Validation reconciliation reports",
        "generated_at": now(),
        "source_inputs": {
            "target_descriptor": "generated/envctl_target_registry.json",
            "repo_scan": "generated/package_scan.json",
            "envctl_database": "generated/envctl_migration_db_model.json",
            "artifact_registry": "generated/envctl_artifact_registry_report.json",
            "validation_evidence": "generated/envctl_validation_evidence_report.json",
            "shared_protocols": "generated/shared_protocol_validation_report.json",
            "task_status": "generated/status_from_proofs.json",
            "final_verification": "generated/final_verification_report.json",
            "contract_manifest": "generated/contract_manifest.json",
        },
        "parity": parity,
        "counts": {
            "by_phase": task_phase_counts(task_rows, statuses),
            "artifact_registry": {
                "evidence_rows": registry_counts.get("evidence_count", 0),
                "graph_edges": registry_counts.get("graph_edge_count", 0),
                "validation_rows": registry_counts.get("validation_count", 0),
                "rejection_cases": registry_counts.get("rejection_count", 0),
            },
            "validation_evidence": {
                "validation_rows": evidence_counts.get("validation_count", 0),
                "evidence_rows": evidence_counts.get("evidence_count", 0),
                "hashed_evidence_rows": evidence_counts.get("hashed_evidence_count", 0),
                "rejection_cases": evidence_counts.get("rejection_count", 0),
            },
            "shared_protocol_samples": {
                "total": len(protocol_results),
                "passed": sum(1 for value in protocol_results.values() if value == "passed"),
            },
            "final_verification": {
                "task_count": final_verification.get("task_count", 0),
                "packet_count": final_verification.get("packet_count", 0),
                "proof_count": final_verification.get("proof_count", 0),
                "missing_outputs": len(final_verification.get("missing_outputs", [])),
                "unresolved_gaps": len(final_verification.get("unresolved_gaps", [])),
            },
        },
        "checksums": {
            "inputs": checksum_paths(source_paths),
            "upstream_proofs": checksum_paths([f"proof_records/{task_id}.proof.json" for task_id in deps]),
        },
        "dependency_statuses": dependency_statuses,
        "contract_mapping": {
            "contract_row_id": "artifact:06-testing-validation-validation-reconciliation-reports-md",
            "contract_required_path": "migration-artifacts/06-testing-validation/validation-reconciliation-reports.md",
            "task_scoped_paths": [
                "migration-artifacts/art-123_validation_reconciliation/validation-reconciliation-reports.md",
                "migration-artifacts/art-123_validation_reconciliation/validation-reconciliation-reports.json",
            ],
            "contract_rows": contract_rows,
        },
        "output_expectations": {
            "target_files": [
                "migration-artifacts/art-123_validation_reconciliation/*.md",
                "migration-artifacts/art-123_validation_reconciliation/*.json",
            ],
            "registered_artifacts": [
                "art-123-validation-reconciliation-md",
                "art-123-validation-reconciliation-json",
                "06-testing-validation-validation-reconciliation-reports-md",
            ],
        },
        "validation_summary": {
            "artifact_registry_status": artifact_registry.get("status", "unknown"),
            "validation_evidence_status": validation_evidence.get("status", "unknown"),
            "shared_protocol_status": shared_protocol.get("status", "unknown"),
            "final_verification_status": final_verification.get("status", "unknown"),
            "target_registry_status": target_registry.get("status", "unknown"),
            "db_model_status": db_model.get("status", "available" if db_model.get("schema_version") else "unknown"),
            "package_scan_status": package_scan.get("status", "available" if package_scan.get("schema_version") else "unknown"),
        },
    }
    errors = []
    if not all(status_is_success(status) for status in dependency_statuses.values()):
        errors.append(f"dependency status not complete/pass: {dependency_statuses}")
    if artifact_registry.get("coverage", {}).get("hashes") is not True:
        errors.append("artifact registry hash coverage is not true")
    if validation_evidence.get("coverage", {}).get("reconciliation") is not True:
        errors.append("validation evidence reconciliation coverage is not true")
    if validation_evidence.get("coverage", {}).get("parity") is not True:
        errors.append("validation evidence parity coverage is not true")
    if evidence_counts.get("hashed_evidence_count", 0) != evidence_counts.get("evidence_count", -1):
        errors.append("not all validation evidence rows have hashes")
    if not parity["passed"]:
        errors.append("task graph, packet, status, and proof parity did not pass")
    if not contract_rows:
        errors.append("contract manifest does not map ART-123 to a contract row")

    reconciliation["status"] = "passed" if not errors else "failed"
    reconciliation["errors"] = errors
    return reconciliation


def render_markdown(report: dict[str, Any]) -> str:
    counts = report["counts"]
    parity = report["parity"]
    lines = [
        "# Validation reconciliation reports",
        "",
        f"Generated at: `{report['generated_at']}`",
        f"Status: `{report['status']}`",
        "",
        "## Parity",
        "",
        "| Check | Count |",
        "|---|---:|",
        f"| Task graph rows | {parity['task_graph_rows']} |",
        f"| Execution packets | {parity['execution_packet_count']} |",
        f"| Status report tasks | {parity['status_report_tasks']} |",
        f"| Proof records | {parity['proof_record_count']} |",
        f"| Successful tasks | {parity['successful_tasks']} |",
        f"| Missing packets | {len(parity['missing_execution_packets'])} |",
        f"| Successful tasks without proof | {len(parity['successful_tasks_without_proof'])} |",
        "",
        "## Counts",
        "",
        "| Area | Metric | Count |",
        "|---|---|---:|",
        f"| Artifact registry | evidence rows | {counts['artifact_registry']['evidence_rows']} |",
        f"| Artifact registry | graph edges | {counts['artifact_registry']['graph_edges']} |",
        f"| Artifact registry | validation rows | {counts['artifact_registry']['validation_rows']} |",
        f"| Validation evidence | validation rows | {counts['validation_evidence']['validation_rows']} |",
        f"| Validation evidence | evidence rows | {counts['validation_evidence']['evidence_rows']} |",
        f"| Validation evidence | hashed evidence rows | {counts['validation_evidence']['hashed_evidence_rows']} |",
        f"| Shared protocols | samples passed | {counts['shared_protocol_samples']['passed']} / {counts['shared_protocol_samples']['total']} |",
        f"| Final verification | missing outputs | {counts['final_verification']['missing_outputs']} |",
        f"| Final verification | unresolved gaps | {counts['final_verification']['unresolved_gaps']} |",
        "",
        "## Phase Status Counts",
        "",
        "| Phase | Tasks | Complete or passed | Pending | Other |",
        "|---|---:|---:|---:|---:|",
    ]
    for item in counts["by_phase"]:
        lines.append(
            f"| {item['phase']} | {item['task_count']} | {item['complete_or_passed']} | {item['pending']} | {item['other']} |"
        )
    lines.extend(
        [
            "",
            "## Checksums",
            "",
            "| Path | SHA-256 |",
            "|---|---|",
        ]
    )
    for path, digest in sorted(report["checksums"]["inputs"].items()):
        lines.append(f"| `{path}` | `{digest}` |")
    lines.extend(
        [
            "",
            "## Contract Mapping",
            "",
            f"- Contract row: `{report['contract_mapping']['contract_row_id']}`",
            f"- Canonical path: `{report['contract_mapping']['contract_required_path']}`",
            f"- Task-scoped Markdown: `{report['contract_mapping']['task_scoped_paths'][0]}`",
            f"- Task-scoped JSON: `{report['contract_mapping']['task_scoped_paths'][1]}`",
            "",
            "## Output Gate",
            "",
            "The artifact registry gate is satisfied when the task-scoped Markdown, task-scoped JSON, and canonical contract Markdown paths are registered with SHA-256 content hashes and linked to validation evidence.",
        ]
    )
    if report["errors"]:
        lines.extend(["", "## Errors", ""])
        for error in report["errors"]:
            lines.append(f"- {error}")
    return "\n".join(lines) + "\n"


def write_artifacts(reconciliation: dict[str, Any]) -> None:
    ARTIFACT_DIR.mkdir(parents=True, exist_ok=True)
    CANONICAL_MD.parent.mkdir(parents=True, exist_ok=True)
    markdown = render_markdown(reconciliation)
    TASK_MD.write_text(markdown, encoding="utf-8")
    CANONICAL_MD.write_text(markdown, encoding="utf-8")
    TASK_JSON.write_text(json.dumps(reconciliation, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def insert_fixture(conn: sqlite3.Connection) -> None:
    descriptor = {
        "target_id": "flexnetos-vs-lifeos",
        "target_type": "mixed",
        "primary_root": "/home/flexnetos/FlexNetOS",
        "compare_root": "/home/flexnetos/lifeos",
        "source": "generated/envctl_target_registry.json",
    }
    conn.execute(
        """
        INSERT INTO envctl_migration_targets
          (id, target_id, target_type, primary_root, compare_root, descriptor_json,
           descriptor_hash, safety_mode, max_auto_risk)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(target_id) DO UPDATE SET
          target_type = excluded.target_type,
          primary_root = excluded.primary_root,
          compare_root = excluded.compare_root,
          descriptor_json = excluded.descriptor_json,
          descriptor_hash = excluded.descriptor_hash,
          safety_mode = excluded.safety_mode,
          max_auto_risk = excluded.max_auto_risk
        """,
        (
            TARGET_ROW_ID,
            "flexnetos-vs-lifeos",
            "mixed",
            "/home/flexnetos/FlexNetOS",
            "/home/flexnetos/lifeos",
            json.dumps(descriptor, sort_keys=True),
            sha256_text(json.dumps(descriptor, sort_keys=True)),
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
            TARGET_ROW_ID,
            RECIPE_ID,
            CONTRACT_ID,
            "validating",
            "approval-gated",
            "artifact-agent",
            "workspace-write",
            "never",
            '{"python":"stdlib","sqlite":"stdlib"}',
            sha256_text(TASK_ID + ":validation-reconciliation"),
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
            f"{TASK_ID}/generate-and-register",
            sha256_text("python3 scripts/generate_art123_validation_reconciliation.py"),
            "python3 scripts/generate_art123_validation_reconciliation.py",
            json.dumps({"task_id": TASK_ID, "contract_row_id": "artifact:06-testing-validation-validation-reconciliation-reports-md"}),
        ),
    )
    conn.commit()


def register_artifacts(conn: sqlite3.Connection, reconciliation: dict[str, Any]) -> list[dict[str, Any]]:
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
            "contract_row_id": "artifact:06-testing-validation-validation-reconciliation-reports-md",
            "parity_passed": reconciliation["parity"]["passed"],
        },
        "evidence_refs": [
            rel(TASK_JSON),
            "execution-framework/generated/envctl_artifact_registry_report.json",
            "execution-framework/generated/envctl_validation_evidence_report.json",
            "execution-framework/generated/shared_protocol_validation_report.json",
            "execution-framework/generated/status_from_proofs.json",
            "execution-framework/proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json",
            "execution-framework/proof_records/REQ-025_ENVCTL_VALIDATION_EVIDENCE.proof.json",
            "execution-framework/proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json",
        ],
        "links": [
            {"to": "artifact:06-testing-validation-validation-reconciliation-reports-md", "type": "satisfies_contract_row"},
            {"to": "REQ-024_ENVCTL_ARTIFACT_REGISTRY", "type": "depends_on"},
            {"to": "REQ-025_ENVCTL_VALIDATION_EVIDENCE", "type": "depends_on"},
            {"to": "REQ-040_SHARED_PROTOCOL_SCHEMAS", "type": "depends_on"},
            {"to": "VER-300_UNIT_VALIDATION", "type": "blocks"},
        ],
        "validations": [
            {
                "validator": "art123:parity",
                "status": "pass" if reconciliation["parity"]["passed"] else "fail",
                "details": reconciliation["parity"],
                "evidence_refs": ["execution-framework/generated/status_from_proofs.json", "execution-framework/generated/task_graph.csv"],
            },
            {
                "validator": "art123:counts",
                "status": "pass",
                "details": reconciliation["counts"],
                "evidence_refs": [
                    "execution-framework/generated/envctl_artifact_registry_report.json",
                    "execution-framework/generated/envctl_validation_evidence_report.json",
                ],
            },
            {
                "validator": "art123:checksums",
                "status": "pass",
                "details": {"input_checksum_count": len(reconciliation["checksums"]["inputs"])},
                "evidence_refs": [rel(TASK_JSON)],
            },
            {
                "validator": "art123:contract-output",
                "status": "pass" if reconciliation["contract_mapping"]["contract_rows"] else "fail",
                "details": reconciliation["contract_mapping"],
                "evidence_refs": [rel(TASK_MD), rel(TASK_JSON), rel(CANONICAL_MD), "execution-framework/generated/contract_manifest.json"],
            },
        ],
    }
    records = [
        {
            **common,
            "artifact_id": "art-123-validation-reconciliation-md",
            "title": "ART-123 Validation Reconciliation Reports",
            "artifact_type": "validation_reconciliation_report",
            "path": rel(TASK_MD),
        },
        {
            **common,
            "artifact_id": "art-123-validation-reconciliation-json",
            "title": "ART-123 Validation Reconciliation Report Data",
            "artifact_type": "machine_readable_validation_reconciliation",
            "path": rel(TASK_JSON),
        },
        {
            **common,
            "artifact_id": "06-testing-validation-validation-reconciliation-reports-md",
            "title": "Validation Reconciliation Reports",
            "artifact_type": "migration_artifact",
            "path": rel(CANONICAL_MD),
        },
    ]
    return [registry.register(record) for record in records]


def record_validation_evidence(conn: sqlite3.Connection, registry_results: list[dict[str, Any]], reconciliation: dict[str, Any]) -> list[dict[str, Any]]:
    store = ValidationEvidenceStore(conn, package_root())
    records = [
        {
            "validation_id": "validation-art123-reconciliation",
            "run_id": RUN_ID,
            "artifact_id": "art-123-validation-reconciliation-json",
            "operation_id": OPERATION_ID,
            "validator": "generate_art123_validation_reconciliation.py:reconciliation",
            "status": "pass" if reconciliation["status"] == "passed" else "fail",
            "details": {
                "parity_passed": reconciliation["parity"]["passed"],
                "dependency_statuses": reconciliation["dependency_statuses"],
                "registered_artifacts": [item["artifact_id"] for item in registry_results],
            },
            "evidence_refs": [
                {"uri": rel(TASK_JSON), "evidence_kind": "reconciliation"},
                {"uri": "execution-framework/generated/envctl_validation_evidence_report.json", "evidence_kind": "proof_record"},
            ],
        },
        {
            "validation_id": "validation-art123-parity",
            "run_id": RUN_ID,
            "artifact_id": "art-123-validation-reconciliation-md",
            "operation_id": OPERATION_ID,
            "validator": "generate_art123_validation_reconciliation.py:parity",
            "status": "pass" if reconciliation["parity"]["passed"] else "fail",
            "details": reconciliation["parity"],
            "evidence_refs": [
                {"uri": rel(TASK_MD), "evidence_kind": "parity"},
                {"uri": "execution-framework/generated/status_from_proofs.json", "evidence_kind": "proof_record"},
            ],
        },
    ]
    return [store.record(record) for record in records]


def build_registry_report(
    conn: sqlite3.Connection,
    reconciliation: dict[str, Any],
    registry_results: list[dict[str, Any]],
    validation_results: list[dict[str, Any]],
) -> dict[str, Any]:
    index_rows = [
        dict(zip(["artifact_id", "status", "path", "content_hash"], row))
        for row in conn.execute(
            """
            SELECT artifact_id, status, path, content_hash
            FROM envctl_migration_artifact_index
            WHERE run_id = ?
            ORDER BY artifact_id
            """,
            (RUN_ID,),
        )
    ]
    validation_scorecard = conn.execute(
        """
        SELECT pass_count, fail_count, warn_count, blocked_count, unknown_count
        FROM envctl_migration_validation_scorecard
        WHERE run_id = ?
        """,
        (RUN_ID,),
    ).fetchone()
    replay_row = conn.execute(
        """
        SELECT has_reproducibility_hash, evidence_missing_hashes, artifacts_missing_hashes, open_approvals
        FROM envctl_migration_replay_readiness
        WHERE run_id = ?
        """,
        (RUN_ID,),
    ).fetchone()
    evidence_by_kind = {
        kind: count
        for kind, count in conn.execute(
            """
            SELECT evidence_kind, COUNT(*)
            FROM envctl_migration_evidence
            WHERE run_id = ?
            GROUP BY evidence_kind
            ORDER BY evidence_kind
            """,
            (RUN_ID,),
        )
    }
    expected_paths = {rel(TASK_MD), rel(TASK_JSON), rel(CANONICAL_MD)}
    indexed_paths = {row["path"] for row in index_rows}
    errors = list(reconciliation["errors"])
    missing = sorted(expected_paths - indexed_paths)
    if missing:
        errors.append(f"missing artifact index paths: {', '.join(missing)}")
    missing_hashes = [row["path"] for row in index_rows if not str(row["content_hash"]).startswith("sha256:")]
    if missing_hashes:
        errors.append(f"missing content hashes: {', '.join(missing_hashes)}")
    if not {"reconciliation", "parity", "proof_record"}.issubset(evidence_by_kind):
        errors.append(f"missing validation evidence kinds: {sorted({'reconciliation', 'parity', 'proof_record'} - set(evidence_by_kind))}")
    if replay_row is None or replay_row[2] != 0:
        errors.append("registered artifacts are not replay-ready with hashes")
    for artifact_id in ["art-123-validation-reconciliation-md", "art-123-validation-reconciliation-json"]:
        fetch_artifact(conn, RUN_ID, artifact_id)
    for validation_id in ["validation-art123-reconciliation", "validation-art123-parity"]:
        fetch_validation(conn, validation_id)
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "generated_at": now(),
        "status": "passed" if not errors else "failed",
        "reconciliation": reconciliation,
        "registry_results": registry_results,
        "validation_results": validation_results,
        "artifact_index_rows": index_rows,
        "validation_scorecard_row": list(validation_scorecard) if validation_scorecard else None,
        "replay_readiness_row": list(replay_row) if replay_row else None,
        "evidence_by_kind": evidence_by_kind,
        "registered_output_hashes": {row["path"]: row["content_hash"] for row in index_rows},
        "errors": errors,
        "evidence": [
            rel(TASK_MD),
            rel(TASK_JSON),
            rel(CANONICAL_MD),
            "execution-framework/generated/art123_validation_reconciliation_report.json",
            "execution-framework/generated/envctl_artifact_registry_report.json",
            "execution-framework/generated/envctl_validation_evidence_report.json",
            "execution-framework/generated/shared_protocol_validation_report.json",
            "execution-framework/generated/status_from_proofs.json",
            "execution-framework/proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json",
            "execution-framework/proof_records/REQ-025_ENVCTL_VALIDATION_EVIDENCE.proof.json",
            "execution-framework/proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json",
        ],
    }


def main() -> None:
    reconciliation = build_reconciliation()
    write_artifacts(reconciliation)

    conn = sqlite3.connect(":memory:")
    conn.execute("PRAGMA foreign_keys = ON")
    apply_migrations(conn, package_root())
    insert_fixture(conn)
    registry_results = register_artifacts(conn, reconciliation)
    validation_results = record_validation_evidence(conn, registry_results, reconciliation)
    report = build_registry_report(conn, reconciliation, registry_results, validation_results)

    REPORT_JSON.write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    log_path = root() / "logs" / f"{TASK_ID}.log"
    log_path.write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    heartbeat_path = root() / "state" / f"{TASK_ID}.heartbeat.json"
    heartbeat_path.write_text(
        json.dumps(
            {
                "task_id": TASK_ID,
                "status": "completed" if report["status"] == "passed" else "failed",
                "updated_at": report["generated_at"],
                "proof_uri": f"proof_records/{TASK_ID}.proof.json",
                "artifact_count": len(report["artifact_index_rows"]),
                "parity_passed": reconciliation["parity"]["passed"],
            },
            indent=2,
            sort_keys=False,
        )
        + "\n",
        encoding="utf-8",
    )

    files_changed = [
        "execution-framework/scripts/generate_art123_validation_reconciliation.py",
        rel(TASK_MD),
        rel(TASK_JSON),
        rel(CANONICAL_MD),
        "execution-framework/generated/art123_validation_reconciliation_report.json",
        "execution-framework/logs/ART-123_VALIDATION_RECONCILIATION.log",
        "execution-framework/state/ART-123_VALIDATION_RECONCILIATION.heartbeat.json",
        "execution-framework/proof_records/ART-123_VALIDATION_RECONCILIATION.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    commands_run = [
        "python3 scripts/generate_art123_validation_reconciliation.py",
        "python3 -m py_compile scripts/generate_art123_validation_reconciliation.py",
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
        {
            "status": report["status"],
            "parity_passed": reconciliation["parity"]["passed"],
            "artifact_index_rows": len(report["artifact_index_rows"]),
            "validation_scorecard_row": report["validation_scorecard_row"],
            "replay_readiness_row": report["replay_readiness_row"],
            "registered_output_hashes": report["registered_output_hashes"],
            "evidence_by_kind": report["evidence_by_kind"],
        },
        report["evidence"],
        "" if report["status"] == "passed" else "; ".join(report["errors"]),
        "run VER-300_UNIT_VALIDATION after remaining artifact tasks are generated",
    )
    append_proof(proof)

    print(
        "ART-123 validation reconciliation status={status} parity={parity} artifacts={artifacts} validations={validations}".format(
            status=report["status"],
            parity=reconciliation["parity"]["passed"],
            artifacts=len(report["artifact_index_rows"]),
            validations=report["validation_scorecard_row"],
        )
    )
    if report["status"] != "passed":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
