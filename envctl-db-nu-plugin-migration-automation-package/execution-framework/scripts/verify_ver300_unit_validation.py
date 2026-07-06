#!/usr/bin/env python3
"""Aggregate and prove the VER-300 unit/integration validation gate."""

from __future__ import annotations

import fnmatch
import re
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, read_json, root, sha256_file, write_json


TASK_ID = "VER-300_UNIT_VALIDATION"
HELPER_ID = "helper-validate-01"
MODEL_TAG = "gpt-5.3-spark"
ACTOR = "validation-agent"
PACKET_PATH = "generated/execution_packets/VER-300_UNIT_VALIDATION.json"
REPORT_PATH = "generated/ver300_unit_validation_report.json"
HEARTBEAT_PATH = "state/VER-300_UNIT_VALIDATION.heartbeat.json"
LOG_PATH = "logs/VER-300_UNIT_VALIDATION.log"
PROOF_PATH = "proof_records/VER-300_UNIT_VALIDATION.proof.json"
APPROVAL_PATH = "approvals/VER-300_UNIT_VALIDATION.agent_approval.json"
STATUS_PATH = "generated/status_report.json"
STATUS_FROM_PROOFS_PATH = "generated/status_from_proofs.json"
FINAL_VERIFICATION_PATH = "generated/final_verification_report.json"
TEST_COVERAGE_REPORT_PATH = "generated/art124_test_coverage_report.json"

DEPENDENCY_PROOFS = {
    "REQ-041_TWO_REPO_INTEGRATION": "proof_records/REQ-041_TWO_REPO_INTEGRATION.proof.json",
    "REQ-202_FLEXNETOS_ADAPTER_RECIPE": "proof_records/REQ-202_FLEXNETOS_ADAPTER_RECIPE.proof.json",
    "ART-123_VALIDATION_RECONCILIATION": "proof_records/ART-123_VALIDATION_RECONCILIATION.proof.json",
}

VALIDATION_INPUTS = {
    "envctl_db": "generated/envctl_migration_db_validation_report.json",
    "artifact_registry": "generated/envctl_artifact_registry_report.json",
    "validation_evidence": "generated/envctl_validation_evidence_report.json",
    "shared_protocol": "generated/shared_protocol_validation_report.json",
    "two_repo_integration": "generated/req041_two_repo_integration_report.json",
    "filesystem_boundaries": "generated/filesystem_boundary_validation_report.json",
    "security_redaction": "generated/security_redaction_validation_report.json",
    "adapter_recipe": "generated/flexnetos_adapter_recipe_validation_report.json",
    "validation_reconciliation": "generated/art123_validation_reconciliation_report.json",
    "test_coverage": TEST_COVERAGE_REPORT_PATH,
}

SECRET_PATTERNS = [
    re.compile(r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
    re.compile(r"\bsk-[A-Za-z0-9_-]{20,}\b"),
    re.compile(r"(?i)(password|secret|token|api[_-]?key)\s*[:=]\s*['\"]?[A-Za-z0-9_./+=-]{12,}"),
]


def is_pass_like(status: str | None) -> bool:
    return str(status).lower() in {"pass", "passed", "complete", "completed"}


def path_allowed(relpath: str, allowed_patterns: list[str]) -> bool:
    normalized = relpath.replace("\\", "/")
    return any(fnmatch.fnmatch(normalized, pattern) for pattern in allowed_patterns)


def read_required_json(relpath: str) -> dict[str, Any]:
    return read_json(relpath)


def secret_findings(paths: list[Path]) -> list[str]:
    findings: list[str] = []
    for path in paths:
        text = path.read_text(encoding="utf-8")
        for pattern in SECRET_PATTERNS:
            if pattern.search(text):
                findings.append(str(path.relative_to(root())))
                break
    return findings


def derive_validation_status(
    name: str,
    payload: dict[str, Any],
    dependency_results: dict[str, Any],
) -> str | None:
    status = payload.get("status")
    if status is not None:
        return status

    verification = payload.get("verification")
    if isinstance(verification, dict):
        verification_status = verification.get("status")
        if verification_status is not None:
            return verification_status

    if name == "two_repo_integration":
        return dependency_results["REQ-041_TWO_REPO_INTEGRATION"]["status"]

    return None


def main() -> None:
    base = root()
    generated_at = now()
    packet = read_required_json(PACKET_PATH)
    approval = read_required_json(APPROVAL_PATH)
    status_report = read_required_json(STATUS_PATH)
    final_verification = read_required_json(FINAL_VERIFICATION_PATH)
    test_coverage = read_required_json(TEST_COVERAGE_REPORT_PATH)

    errors: list[str] = []
    warnings: list[str] = []

    dependency_results: dict[str, Any] = {}
    for task_id, proof_path in DEPENDENCY_PROOFS.items():
        proof = read_required_json(proof_path)
        ok = is_pass_like(proof.get("status"))
        dependency_results[task_id] = {
            "proof_path": proof_path,
            "status": proof.get("status"),
            "ok": ok,
        }
        if not ok:
            errors.append(f"dependency proof not completed: {task_id}")

    validation_results: dict[str, Any] = {}
    for name, relpath in VALIDATION_INPUTS.items():
        payload = read_required_json(relpath)
        status = derive_validation_status(name, payload, dependency_results)
        ok = is_pass_like(status)
        validation_results[name] = {
            "path": relpath,
            "status": status,
            "ok": ok,
        }
        if not ok:
            errors.append(f"validation input not passing: {name} ({status})")

    runnable_task_ids = {item["task_id"] for item in status_report.get("runnable_tasks", [])}
    dispatch_task_ids = {item["task_id"] for item in status_report.get("dispatch_packets", [])}
    status_map = status_report.get("statuses", {})
    coverage_summary = test_coverage["matrix"]["coverage_summary"]
    dependency_status = test_coverage["matrix"]["dependency_status"]
    completed_dependencies = [
        "REQ-024_ENVCTL_ARTIFACT_REGISTRY",
        "REQ-025_ENVCTL_VALIDATION_EVIDENCE",
        "REQ-033_PLUGIN_HUMAN_APPROVAL",
        "REQ-040_SHARED_PROTOCOL_SCHEMAS",
        "REQ-041_TWO_REPO_INTEGRATION",
        "REQ-045_RUN_REPLAY",
    ]

    checks = {
        "approval_decision_approved": approval.get("decision") == "approved",
        "packet_proof_required": packet.get("proof_required") is True,
        "packet_single_threaded": packet.get("can_run_parallel") is False and packet.get("max_parallel") == 1,
        "status_report_dispatchable": TASK_ID in runnable_task_ids and TASK_ID in dispatch_task_ids,
        "status_report_pending_before_execution": status_map.get(TASK_ID) == "pending",
        "final_verification_local_package_complete": final_verification.get("local_package_complete") is True,
        "final_verification_no_failed_tasks": final_verification.get("goal_loop_summary", {}).get("failed_count") == 0,
        "coverage_classes_complete": coverage_summary.get("covered_class_count") == len(coverage_summary.get("required_classes", [])),
        "coverage_dependencies_completed": all(
            status_map.get(task_id) == "complete" for task_id in completed_dependencies
        ),
        "dependency_proofs_completed": all(item["ok"] for item in dependency_results.values()),
        "validation_inputs_passed": all(item["ok"] for item in validation_results.values()),
    }

    if coverage_summary.get("ver300_entry_status") != "ready_with_open_runtime_gates":
        warnings.append("test coverage entry status drifted from ready_with_open_runtime_gates")
    if final_verification.get("status") != "pass_with_external_blocker":
        warnings.append("final verification status drifted from pass_with_external_blocker")
    stale_coverage_dependencies = [
        task_id
        for task_id in completed_dependencies
        if status_map.get(task_id) == "complete" and dependency_status.get(task_id) != "completed"
    ]
    if stale_coverage_dependencies:
        warnings.append(
            "test coverage dependency_status is stale for: " + ", ".join(stale_coverage_dependencies)
        )

    for name, passed in checks.items():
        if not passed:
            errors.append(f"check failed: {name}")

    output_paths = [
        "execution-framework/scripts/verify_ver300_unit_validation.py",
        "execution-framework/" + REPORT_PATH,
        "execution-framework/" + HEARTBEAT_PATH,
        "execution-framework/" + LOG_PATH,
        "execution-framework/" + PROOF_PATH,
    ]
    allowed_paths = packet.get("allowed_paths", [])
    path_violations = [path for path in output_paths if not path_allowed(path, allowed_paths)]
    if path_violations:
        errors.append("generated outputs outside allowed paths: " + ", ".join(path_violations))

    report = {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "pass" if not errors else "fail",
        "generated_at": generated_at,
        "packet_summary": {
            "goal": packet.get("goal"),
            "repo_target": packet.get("repo_target"),
            "repo_path": packet.get("repo_path"),
            "filesystem_scope": packet.get("filesystem_scope"),
            "verification_command": "python3 scripts/verify_ver300_unit_validation.py",
        },
        "approval": {
            "path": APPROVAL_PATH,
            "decision": approval.get("decision"),
            "reviewed_at": approval.get("reviewed_at"),
            "reviewer": approval.get("reviewer"),
            "model": approval.get("model"),
        },
        "checks": checks,
        "dependency_proofs": dependency_results,
        "validation_inputs": validation_results,
        "coverage_summary": coverage_summary,
        "coverage_dependency_status": dependency_status,
        "status_surfaces": {
            "status_report_generated_at": status_report.get("generated_at"),
            "status_report_task_status": status_map.get(TASK_ID),
            "final_verification_status": final_verification.get("status"),
            "final_verification_failed_count": final_verification.get("goal_loop_summary", {}).get("failed_count"),
        },
        "warnings": warnings,
        "errors": errors,
        "allowed_paths": packet.get("allowed_paths", []),
        "output_paths": output_paths,
        "sha256": {
            relpath: "sha256:" + sha256_file(base / relpath)
            for relpath in [APPROVAL_PATH, *DEPENDENCY_PROOFS.values(), *VALIDATION_INPUTS.values()]
        },
    }

    write_json(REPORT_PATH, report)
    write_json(LOG_PATH, report)
    write_json(
        HEARTBEAT_PATH,
        {
            "schema_version": "1.0",
            "task_id": TASK_ID,
            "status": report["status"],
            "updated_at": generated_at,
            "proof_uri": PROOF_PATH,
            "validation_report": REPORT_PATH,
        },
    )

    secret_scan_paths = [base / REPORT_PATH, base / LOG_PATH, Path(__file__)]
    secret_hits = secret_findings(secret_scan_paths)
    report["secret_scan"] = {
        "paths": [str(path.relative_to(base)) for path in secret_scan_paths],
        "findings": secret_hits,
    }
    if secret_hits:
        report["status"] = "fail"
        report["errors"].append("secret-like content detected in generated outputs: " + ", ".join(secret_hits))
        write_json(REPORT_PATH, report)
        write_json(LOG_PATH, report)
        write_json(
            HEARTBEAT_PATH,
            {
                "schema_version": "1.0",
                "task_id": TASK_ID,
                "status": report["status"],
                "updated_at": generated_at,
                "proof_uri": PROOF_PATH,
                "validation_report": REPORT_PATH,
            },
        )

    files_changed = [
        "execution-framework/scripts/verify_ver300_unit_validation.py",
        "execution-framework/generated/ver300_unit_validation_report.json",
        "execution-framework/state/VER-300_UNIT_VALIDATION.heartbeat.json",
        "execution-framework/logs/VER-300_UNIT_VALIDATION.log",
        "execution-framework/proof_records/VER-300_UNIT_VALIDATION.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    proof = make_proof(
        TASK_ID,
        "completed" if report["status"] == "pass" else "failed",
        ACTOR,
        packet["helper_id"],
        packet["model_tag"],
        packet["repo_path"],
        files_changed,
        [
            "python3 scripts/verify_ver300_unit_validation.py",
            "python3 -m py_compile scripts/verify_ver300_unit_validation.py",
        ],
        report,
        [
            APPROVAL_PATH,
            *DEPENDENCY_PROOFS.values(),
            *VALIDATION_INPUTS.values(),
            STATUS_PATH,
            STATUS_FROM_PROOFS_PATH,
            FINAL_VERIFICATION_PATH,
        ],
        "" if report["status"] == "pass" else "; ".join(report["errors"]),
        "unblock VER-301_SQL_SCHEMA_TEST" if report["status"] == "pass" else "fix VER-300 validation errors",
    )
    append_proof(proof)

    if report["status"] != "pass":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
