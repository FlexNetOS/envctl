#!/usr/bin/env python3
"""Run and prove the VER-302 packet and schema validation gate."""

from __future__ import annotations

import fnmatch
import json
import re
import sys
from pathlib import Path
from typing import Any

from jsonschema import Draft7Validator, Draft202012Validator
from jsonschema.exceptions import ValidationError
from jsonschema.validators import validator_for

from _common import append_proof, file_checksums, make_proof, now, read_json, read_task_graph, root, split_list, write_json


TASK_ID = "VER-302_PACKET_SCHEMA_VALIDATION"
PACKET_PATH = "generated/execution_packets/VER-302_PACKET_SCHEMA_VALIDATION.json"
REPORT_PATH = "generated/ver302_packet_schema_validation_report.json"
HEARTBEAT_PATH = "state/VER-302_PACKET_SCHEMA_VALIDATION.heartbeat.json"
LOG_PATH = "logs/VER-302_PACKET_SCHEMA_VALIDATION.log"
PROOF_PATH = "proof_records/VER-302_PACKET_SCHEMA_VALIDATION.proof.json"

DEPENDENCY_PROOFS = [
    "proof_records/VER-301_SQL_SCHEMA_TEST.proof.json",
    "proof_records/REQ-042_FILESYSTEM_BOUNDS.proof.json",
    "proof_records/REQ-043_SECURITY_REDACTION.proof.json",
    "proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json",
]

PACKET_DIR = "generated/execution_packets"
SCHEMA_DIR = "schemas"
PROOF_DIR = "proof_records"
PROOF_TEMPLATE_PATH = "proof_templates/PROOF_RECORD_TEMPLATE.json"
NORMALIZED_GRAPH_PATH = "generated/task_graph.normalized.json"
LANE_TEMPLATE_PATH = "templates/AGENT_LANE_TEMPLATE.json"
FILESYSTEM_BOUNDARIES_PATH = "generated/filesystem_boundaries.json"
GAP_REPORT_PATH = "generated/gap_report.json"
UPGRADE_REPORT_PATH = "generated/upgrade_report.json"
SHARED_PROTOCOL_MANIFEST_PATH = "generated/shared_protocol_manifest.json"
SHARED_PROTOCOL_REPORT_PATH = "generated/shared_protocol_validation_report.json"

ARRAY_FIELDS = {
    "input_files",
    "target_files",
    "target_artifacts",
    "allowed_paths",
    "blocked_paths",
    "depends_on",
    "blocks",
    "required_tools",
}
BOOL_FIELDS = {"can_run_parallel", "proof_required", "human_approval_required"}
INT_FIELDS = {"max_parallel", "priority"}

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


def secret_findings(relpaths: list[str]) -> list[str]:
    findings: list[str] = []
    base = root()
    for relpath in relpaths:
        path = base / relpath
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8")
        for pattern in SECRET_PATTERNS:
            if pattern.search(text):
                findings.append(relpath)
                break
    return sorted(set(findings))


def convert_row_to_packet(row: dict[str, str], generated_at: str) -> dict[str, Any]:
    packet: dict[str, Any] = {"packet_schema_version": "1.0"}
    for key, value in row.items():
        if key in ARRAY_FIELDS:
            packet[key] = split_list(value)
        elif key in BOOL_FIELDS:
            packet[key] = str(value).lower() == "true"
        elif key in INT_FIELDS:
            packet[key] = int(value)
        elif key == "status":
            continue
        else:
            packet[key] = value
    packet["source_graph_uri"] = "generated/task_graph.csv"
    packet["generated_at"] = generated_at
    return packet


def packet_differences(expected: dict[str, Any], actual: dict[str, Any]) -> list[str]:
    diffs: list[str] = []
    for key in sorted(set(expected) | set(actual)):
        if key == "generated_at":
            continue
        if expected.get(key) != actual.get(key):
            diffs.append(key)
    return diffs


def validate_instance(validator: Any, instance: Any) -> list[str]:
    return [error.message for error in validator.iter_errors(instance)]


def schema_validator(schema: dict[str, Any]) -> Any:
    validator_cls = validator_for(schema)
    validator_cls.check_schema(schema)
    return validator_cls(schema)


def main() -> None:
    base = root()
    generated_at = now()
    packet = read_json(PACKET_PATH)

    errors: list[str] = []
    warnings: list[str] = []

    dependency_statuses: list[dict[str, Any]] = []
    for relpath in DEPENDENCY_PROOFS:
        proof = read_json(relpath)
        ok = is_pass_like(proof.get("status"))
        dependency_statuses.append({"path": relpath, "status": proof.get("status"), "ok": ok})
        if not ok:
            errors.append(f"dependency proof not completed: {relpath}")

    packet_schema = read_json("schemas/execution_packet.schema.json")
    proof_schema = read_json("schemas/proof_record.schema.json")
    task_graph_schema = read_json("schemas/task_graph.schema.json")
    gap_report_schema = read_json("schemas/gap_report.schema.json")
    upgrade_report_schema = read_json("schemas/upgrade_report.schema.json")
    filesystem_boundaries_schema = read_json("schemas/filesystem_boundaries.schema.json")
    agent_lane_schema = read_json("schemas/agent_lane.schema.json")
    shared_protocol_schema = read_json("schemas/shared_protocol.schema.json")

    schema_results: list[dict[str, Any]] = []

    # Self-validate every schema against its declared draft.
    for schema_path in sorted((base / SCHEMA_DIR).glob("*.json")):
        relpath = str(schema_path.relative_to(base))
        schema = json.loads(schema_path.read_text(encoding="utf-8"))
        draft = schema.get("$schema")
        result: dict[str, Any] = {
            "schema_path": relpath,
            "draft": draft,
            "self_valid": False,
            "schema_title": schema.get("title"),
            "fixture_checks": [],
        }
        try:
            validator = schema_validator(schema)
            result["self_valid"] = True
            result["validator"] = validator.__class__.__name__
        except Exception as exc:  # pragma: no cover - defensive reporting
            result["error"] = str(exc)
            errors.append(f"schema failed self-validation: {relpath}: {exc}")
            schema_results.append(result)
            continue

        try:
            if relpath == "schemas/agent_lane.schema.json":
                lane_template = read_json(LANE_TEMPLATE_PATH)
                fixture_errors = validate_instance(validator, lane_template)
                result["fixture_checks"].append(
                    {
                        "artifact": LANE_TEMPLATE_PATH,
                        "status": "pass" if not fixture_errors else "fail",
                        "error_count": len(fixture_errors),
                        "errors": fixture_errors[:20],
                    }
                )
                if fixture_errors:
                    errors.append(f"agent lane template failed schema validation: {LANE_TEMPLATE_PATH}")
            elif relpath == "schemas/execution_packet.schema.json":
                pass
            elif relpath == "schemas/filesystem_boundaries.schema.json":
                fs_config = read_json(FILESYSTEM_BOUNDARIES_PATH)
                fixture_errors = validate_instance(validator, fs_config)
                result["fixture_checks"].append(
                    {
                        "artifact": FILESYSTEM_BOUNDARIES_PATH,
                        "status": "pass" if not fixture_errors else "fail",
                        "error_count": len(fixture_errors),
                        "errors": fixture_errors[:20],
                    }
                )
                if fixture_errors:
                    errors.append(f"filesystem boundaries artifact failed schema validation: {FILESYSTEM_BOUNDARIES_PATH}")
            elif relpath == "schemas/gap_report.schema.json":
                gap_report = read_json(GAP_REPORT_PATH)
                fixture_errors = validate_instance(validator, gap_report)
                result["fixture_checks"].append(
                    {
                        "artifact": GAP_REPORT_PATH,
                        "status": "pass" if not fixture_errors else "fail",
                        "error_count": len(fixture_errors),
                        "errors": fixture_errors[:20],
                    }
                )
                if fixture_errors:
                    errors.append(f"gap report failed schema validation: {GAP_REPORT_PATH}")
            elif relpath == "schemas/proof_record.schema.json":
                proof_template = read_json(PROOF_TEMPLATE_PATH)
                template_errors = validate_instance(validator, proof_template)
                result["fixture_checks"].append(
                    {
                        "artifact": PROOF_TEMPLATE_PATH,
                        "status": "pass" if not template_errors else "fail",
                        "error_count": len(template_errors),
                        "errors": template_errors[:20],
                    }
                )
                if template_errors:
                    errors.append(f"proof template failed schema validation: {PROOF_TEMPLATE_PATH}")

                representative_proof = read_json("proof_records/REQ-031_PLUGIN_COMMAND_SURFACE.proof.json")
                proof_errors = validate_instance(validator, representative_proof)
                result["fixture_checks"].append(
                    {
                        "artifact": "proof_records/REQ-031_PLUGIN_COMMAND_SURFACE.proof.json",
                        "status": "pass" if not proof_errors else "fail",
                        "error_count": len(proof_errors),
                        "errors": proof_errors[:20],
                    }
                )
                if proof_errors:
                    errors.append("representative proof record failed schema validation")
            elif relpath == "schemas/shared_protocol.schema.json":
                manifest = read_json(SHARED_PROTOCOL_MANIFEST_PATH)
                report = read_json(SHARED_PROTOCOL_REPORT_PATH)
                shared_protocol_sample = read_json("proof_records/VER-301_SQL_SCHEMA_TEST.proof.json")
                manifest_checks = {
                    "required_record_count_matches_manifest": len(manifest.get("required_records", [])) == len(
                        manifest.get("records", [])
                    ),
                    "shared_protocol_validation_report_passed": report.get("status") == "passed",
                }
                sample_errors = validate_instance(validator, shared_protocol_sample)
                result["fixture_checks"].append(
                    {
                        "artifact": SHARED_PROTOCOL_MANIFEST_PATH,
                        "status": "pass" if all(manifest_checks.values()) else "fail",
                        "checks": manifest_checks,
                    }
                )
                result["fixture_checks"].append(
                    {
                        "artifact": "proof_records/VER-301_SQL_SCHEMA_TEST.proof.json",
                        "status": "pass" if not sample_errors else "fail",
                        "error_count": len(sample_errors),
                        "errors": sample_errors[:20],
                    }
                )
                if not all(manifest_checks.values()):
                    errors.append("shared protocol manifest/report checks failed")
                if sample_errors:
                    errors.append("shared protocol schema rejected proof-record branch sample")
            elif relpath == "schemas/task_graph.schema.json":
                normalized = read_json(NORMALIZED_GRAPH_PATH)
                invalid_rows: list[str] = []
                for row in normalized.get("tasks", []):
                    row_errors = validate_instance(validator, row)
                    if row_errors:
                        invalid_rows.append(str(row.get("task_id", "<unknown>")))
                result["fixture_checks"].append(
                    {
                        "artifact": NORMALIZED_GRAPH_PATH,
                        "checked_count": len(normalized.get("tasks", [])),
                        "status": "pass" if not invalid_rows else "fail",
                        "invalid_task_ids": invalid_rows[:20],
                    }
                )
                if invalid_rows:
                    errors.append("task graph rows failed schema validation: " + ", ".join(invalid_rows[:10]))
            elif relpath == "schemas/upgrade_report.schema.json":
                upgrade_report = read_json(UPGRADE_REPORT_PATH)
                fixture_errors = validate_instance(validator, upgrade_report)
                result["fixture_checks"].append(
                    {
                        "artifact": UPGRADE_REPORT_PATH,
                        "status": "pass" if not fixture_errors else "fail",
                        "error_count": len(fixture_errors),
                        "errors": fixture_errors[:20],
                    }
                )
                if fixture_errors:
                    errors.append(f"upgrade report failed schema validation: {UPGRADE_REPORT_PATH}")
        except FileNotFoundError as exc:
            warnings.append(f"schema fixture unavailable for {relpath}: {exc.filename}")

        schema_results.append(result)

    packet_validator = Draft7Validator(packet_schema)
    graph_rows = read_task_graph("generated/task_graph.csv")
    graph_lookup = {row["task_id"]: row for row in graph_rows}
    expected_task_ids = sorted(graph_lookup)
    packet_paths = sorted((base / PACKET_DIR).glob("*.json"))
    packet_task_ids = sorted(path.stem for path in packet_paths)

    missing_packets = sorted(set(expected_task_ids) - set(packet_task_ids))
    unexpected_packets = sorted(set(packet_task_ids) - set(expected_task_ids))
    if missing_packets:
        errors.append("missing execution packets: " + ", ".join(missing_packets[:10]))
    if unexpected_packets:
        errors.append("unexpected execution packets: " + ", ".join(unexpected_packets[:10]))

    packet_validation_results: list[dict[str, Any]] = []
    blocked_paths = packet.get("blocked_paths", [])
    packet_secret_hits: list[str] = []

    for packet_path in packet_paths:
        relpath = str(packet_path.relative_to(base))
        packet_data = json.loads(packet_path.read_text(encoding="utf-8"))
        task_id = packet_data.get("task_id", packet_path.stem)
        filename_matches_task_id = packet_path.stem == task_id
        schema_errors = [error.message for error in packet_validator.iter_errors(packet_data)]
        expected_row = graph_lookup.get(packet_path.stem)
        graph_match = expected_row is not None
        mismatch_fields: list[str] = []
        if expected_row is not None:
            expected_packet = convert_row_to_packet(expected_row, str(packet_data.get("generated_at", "")))
            mismatch_fields = packet_differences(expected_packet, packet_data)

        blocked_field_refs: list[str] = []
        for field in ["proof_uri", "heartbeat_file", "logs_uri"]:
            value = packet_data.get(field)
            if isinstance(value, str) and any(fnmatch.fnmatch(value, pattern) for pattern in blocked_paths):
                blocked_field_refs.append(f"{field}:{value}")

        if secret_findings([relpath]):
            packet_secret_hits.append(relpath)

        status = "pass"
        if schema_errors or not filename_matches_task_id or not graph_match or mismatch_fields or blocked_field_refs:
            status = "fail"
            errors.append(f"packet validation failed: {relpath}")

        packet_validation_results.append(
            {
                "packet_path": relpath,
                "task_id": task_id,
                "status": status,
                "filename_matches_task_id": filename_matches_task_id,
                "task_id_present_in_graph": graph_match,
                "schema_error_count": len(schema_errors),
                "schema_errors": schema_errors[:20],
                "graph_mismatch_fields": mismatch_fields[:20],
                "blocked_field_refs": blocked_field_refs,
            }
        )

    if packet_secret_hits:
        errors.append("secret-like content detected in execution packets: " + ", ".join(packet_secret_hits[:10]))

    output_paths = [
        "execution-framework/scripts/verify_ver302_packet_schema_validation.py",
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
            "verification_command": "python3 scripts/verify_ver302_packet_schema_validation.py",
        },
        "dependencies": dependency_statuses,
        "counts": {
            "execution_packets": len(packet_paths),
            "task_graph_rows": len(graph_rows),
            "schemas": len(schema_results),
            "proof_records": len(list((base / PROOF_DIR).glob("*.proof.json"))),
        },
        "packet_set_parity": {
            "missing_packets": missing_packets,
            "unexpected_packets": unexpected_packets,
            "expected_task_count": len(expected_task_ids),
            "packet_task_count": len(packet_task_ids),
        },
        "packet_validations": packet_validation_results,
        "schema_validations": schema_results,
        "warnings": warnings,
        "errors": errors,
        "allowed_paths": allowed_paths,
        "blocked_paths": blocked_paths,
        "output_paths": output_paths,
        "sha256": file_checksums(
            [
                "execution-framework/generated/execution_packets",
                "execution-framework/generated/task_graph.csv",
                "execution-framework/generated/task_graph.normalized.json",
                "execution-framework/generated/filesystem_boundaries.json",
                "execution-framework/generated/gap_report.json",
                "execution-framework/generated/upgrade_report.json",
                "execution-framework/generated/shared_protocol_manifest.json",
                "execution-framework/generated/shared_protocol_validation_report.json",
                "execution-framework/templates/AGENT_LANE_TEMPLATE.json",
                "execution-framework/proof_templates/PROOF_RECORD_TEMPLATE.json",
                "execution-framework/schemas/execution_packet.schema.json",
                "execution-framework/schemas/proof_record.schema.json",
                "execution-framework/schemas/task_graph.schema.json",
                "execution-framework/schemas/agent_lane.schema.json",
                "execution-framework/schemas/gap_report.schema.json",
                "execution-framework/schemas/upgrade_report.schema.json",
                "execution-framework/schemas/filesystem_boundaries.schema.json",
                "execution-framework/schemas/shared_protocol.schema.json",
            ]
        ),
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

    report_secret_scan_paths = [
        REPORT_PATH,
        LOG_PATH,
        "scripts/verify_ver302_packet_schema_validation.py",
    ]
    secret_hits = secret_findings(report_secret_scan_paths)
    report["secret_scan"] = {"paths": report_secret_scan_paths, "findings": secret_hits}
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
        "execution-framework/scripts/verify_ver302_packet_schema_validation.py",
        "execution-framework/generated/ver302_packet_schema_validation_report.json",
        "execution-framework/state/VER-302_PACKET_SCHEMA_VALIDATION.heartbeat.json",
        "execution-framework/logs/VER-302_PACKET_SCHEMA_VALIDATION.log",
        "execution-framework/proof_records/VER-302_PACKET_SCHEMA_VALIDATION.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    proof = make_proof(
        TASK_ID,
        "completed" if report["status"] == "pass" else "failed",
        packet["owner_agent"],
        packet["helper_id"],
        packet["model_tag"],
        packet["repo_path"],
        files_changed,
        [
            "python3 scripts/verify_ver302_packet_schema_validation.py",
            "python3 -m py_compile scripts/verify_ver302_packet_schema_validation.py",
        ],
        report,
        DEPENDENCY_PROOFS
        + [
            "generated/task_graph.csv",
            NORMALIZED_GRAPH_PATH,
            FILESYSTEM_BOUNDARIES_PATH,
            GAP_REPORT_PATH,
            UPGRADE_REPORT_PATH,
            SHARED_PROTOCOL_MANIFEST_PATH,
            SHARED_PROTOCOL_REPORT_PATH,
            LANE_TEMPLATE_PATH,
            PROOF_TEMPLATE_PATH,
            "schemas/execution_packet.schema.json",
            "schemas/proof_record.schema.json",
            "schemas/task_graph.schema.json",
            "schemas/agent_lane.schema.json",
            "schemas/gap_report.schema.json",
            "schemas/upgrade_report.schema.json",
            "schemas/filesystem_boundaries.schema.json",
            "schemas/shared_protocol.schema.json",
        ],
        "" if report["status"] == "pass" else "; ".join(report["errors"][:20]),
        "unblock VER-303_GOAL_LOOP_COMPUTE" if report["status"] == "pass" else "fix packet/schema validation failures",
    )
    append_proof(proof)

    if report["status"] != "pass":
        raise SystemExit(1)


if __name__ == "__main__":
    try:
        main()
    except ValidationError as exc:
        print("validation error", file=sys.stderr)
        raise SystemExit(1) from exc
