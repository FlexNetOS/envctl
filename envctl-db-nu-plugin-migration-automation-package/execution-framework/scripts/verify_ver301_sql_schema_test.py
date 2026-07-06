#!/usr/bin/env python3
"""Run and prove the VER-301 SQL schema verification gate."""

from __future__ import annotations

import fnmatch
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, read_json, root, sha256_file, write_json


TASK_ID = "VER-301_SQL_SCHEMA_TEST"
HELPER_ID = "helper-sql-test-01"
MODEL_TAG = "gpt-5.3-spark"
ACTOR = "sql-test-agent"
PACKET_PATH = "generated/execution_packets/VER-301_SQL_SCHEMA_TEST.json"
REPORT_PATH = "generated/ver301_sql_schema_test_report.json"
HEARTBEAT_PATH = "state/VER-301_SQL_SCHEMA_TEST.heartbeat.json"
LOG_PATH = "logs/VER-301_SQL_SCHEMA_TEST.log"
PROOF_PATH = "proof_records/VER-301_SQL_SCHEMA_TEST.proof.json"
DEPENDENCY_PROOF_PATH = "proof_records/VER-300_UNIT_VALIDATION.proof.json"
REQ020_PROOF_PATH = "proof_records/REQ-020_ENVCTL_DB_SCHEMA.proof.json"
REQ020_REPORT_PATH = "generated/envctl_migration_db_validation_report.json"
REQ020_MODEL_PATH = "generated/envctl_migration_db_model.json"
REQ020_DOC_PATH = "docs/ENVCTL_DB_SCHEMA.md"

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


def secret_findings(paths: list[Path]) -> list[str]:
    findings: list[str] = []
    for path in paths:
        text = path.read_text(encoding="utf-8")
        for pattern in SECRET_PATTERNS:
            if pattern.search(text):
                findings.append(str(path.relative_to(root())))
                break
    return findings


def main() -> None:
    base = root()
    generated_at = now()
    packet = read_json(PACKET_PATH)
    dependency_proof = read_json(DEPENDENCY_PROOF_PATH)

    errors: list[str] = []

    dependency_ok = is_pass_like(dependency_proof.get("status"))
    if not dependency_ok:
        errors.append(f"dependency proof not completed: {DEPENDENCY_PROOF_PATH}")

    command = [sys.executable, "scripts/verify_envctl_db_schema.py"]
    result = subprocess.run(command, cwd=base, capture_output=True, text=True, check=False)

    stdout_lines = [line for line in result.stdout.splitlines() if line.strip()]
    stderr_lines = [line for line in result.stderr.splitlines() if line.strip()]
    if result.returncode != 0:
        errors.append(f"schema verifier exited non-zero: {result.returncode}")

    req020_report = read_json(REQ020_REPORT_PATH)
    req020_proof = read_json(REQ020_PROOF_PATH)
    req020_model = read_json(REQ020_MODEL_PATH)

    req020_status_ok = req020_report.get("status") == "passed"
    req020_proof_ok = is_pass_like(req020_proof.get("status"))
    if not req020_status_ok:
        errors.append(f"REQ-020 validation report not passed: {req020_report.get('status')}")
    if not req020_proof_ok:
        errors.append(f"REQ-020 proof not completed: {req020_proof.get('status')}")

    summary = req020_report.get("summary", {})
    checks = {
        "dependency_ver300_completed": dependency_ok,
        "schema_verifier_exit_zero": result.returncode == 0,
        "req020_report_passed": req020_status_ok,
        "req020_proof_completed": req020_proof_ok,
        "table_count_matches_required": summary.get("actual_table_count") == summary.get("required_table_count"),
        "view_count_matches_required": summary.get("actual_view_count") == summary.get("required_view_count"),
        "constraint_check_rejected_invalid_risk": req020_model.get("constraint_check", {}).get(
            "invalid_operation_risk_rejected"
        )
        is True,
        "foreign_key_check_clean": len(req020_model.get("foreign_key_check", [])) == 0,
        "required_view_queries_passed": all(
            item.get("query_ok") is True for item in req020_model.get("view_runtime_queries", {}).values()
        ),
    }

    for name, passed in checks.items():
        if not passed:
            errors.append(f"check failed: {name}")

    output_paths = [
        "execution-framework/scripts/verify_ver301_sql_schema_test.py",
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
            "verification_command": "python3 scripts/verify_ver301_sql_schema_test.py",
        },
        "dependency_proof": {
            "path": DEPENDENCY_PROOF_PATH,
            "status": dependency_proof.get("status"),
            "ok": dependency_ok,
        },
        "schema_execution": {
            "command": "python3 scripts/verify_envctl_db_schema.py",
            "exit_code": result.returncode,
            "stdout": stdout_lines,
            "stderr": stderr_lines,
        },
        "req020_artifacts": {
            "report_path": REQ020_REPORT_PATH,
            "proof_path": REQ020_PROOF_PATH,
            "model_path": REQ020_MODEL_PATH,
            "doc_path": REQ020_DOC_PATH,
            "status": req020_report.get("status"),
            "proof_status": req020_proof.get("status"),
        },
        "checks": checks,
        "schema_summary": summary,
        "required_views": req020_model.get("required_views", []),
        "view_runtime_queries": req020_model.get("view_runtime_queries", {}),
        "constraint_check": req020_model.get("constraint_check", {}),
        "foreign_key_check": req020_model.get("foreign_key_check", []),
        "applied_migrations": req020_model.get("applied_migrations", []),
        "errors": errors,
        "allowed_paths": allowed_paths,
        "output_paths": output_paths,
        "sha256": {
            relpath: "sha256:" + sha256_file(base / relpath)
            for relpath in [
                DEPENDENCY_PROOF_PATH,
                REQ020_REPORT_PATH,
                REQ020_PROOF_PATH,
                REQ020_MODEL_PATH,
                REQ020_DOC_PATH,
            ]
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
        "execution-framework/scripts/verify_ver301_sql_schema_test.py",
        "execution-framework/generated/ver301_sql_schema_test_report.json",
        "execution-framework/state/VER-301_SQL_SCHEMA_TEST.heartbeat.json",
        "execution-framework/logs/VER-301_SQL_SCHEMA_TEST.log",
        "execution-framework/proof_records/VER-301_SQL_SCHEMA_TEST.proof.json",
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
            "python3 scripts/verify_ver301_sql_schema_test.py",
            "python3 scripts/verify_envctl_db_schema.py",
            "python3 -m py_compile scripts/verify_ver301_sql_schema_test.py",
        ],
        report,
        [
            DEPENDENCY_PROOF_PATH,
            REQ020_REPORT_PATH,
            REQ020_PROOF_PATH,
            REQ020_MODEL_PATH,
            REQ020_DOC_PATH,
        ],
        "" if report["status"] == "pass" else "; ".join(report["errors"]),
        "unblock VER-302_PACKET_SCHEMA_VALIDATION" if report["status"] == "pass" else "fix SQL schema verification",
    )
    append_proof(proof)

    if report["status"] != "pass":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
