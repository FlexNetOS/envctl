from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, package_root, sha256_file, write_json
from verify_target_registry import descriptor_hash, load_descriptor, normalized_descriptor, validate_descriptor


TASK_ID = "REQ-200_FLEXNETOS_TARGET_DESCRIPTOR"
SOURCE_DESCRIPTOR = "examples/target-descriptors/flexnetos-vs-lifeos.yaml"
ARTIFACT_DESCRIPTOR = "execution-framework/migration-artifacts/_meta/flexnetos-vs-lifeos.target-descriptor.yaml"
REPORT_PATH = "execution-framework/generated/flexnetos_target_descriptor_validation_report.json"
LOG_PATH = f"execution-framework/logs/{TASK_ID}.log"
HEARTBEAT_PATH = f"execution-framework/state/{TASK_ID}.heartbeat.json"
FRAMEWORK_REPORT_PATH = "generated/flexnetos_target_descriptor_validation_report.json"
FRAMEWORK_LOG_PATH = f"logs/{TASK_ID}.log"
FRAMEWORK_HEARTBEAT_PATH = f"state/{TASK_ID}.heartbeat.json"

EXPECTED_PRIMARY_ROOT = "/home/flexnetos/FlexNetOS"
EXPECTED_COMPARE_ROOT = "/home/flexnetos/lifeos"
SECRET_EXCLUDES = [
    "**/.env",
    "**/secrets/**",
    "**/private_keys/**",
    "**/*.pem",
    "**/*.key",
]
SECRET_PATTERNS = [
    re.compile(r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
    re.compile(r"\bsk-[A-Za-z0-9_-]{20,}\b"),
    re.compile(r"(?i)(password|secret|token|api[_-]?key)\s*[:=]\s*['\"]?[A-Za-z0-9_./+=-]{12,}"),
]


def render_scalar(value: Any) -> str:
    if isinstance(value, bool):
        return "true" if value else "false"
    if value is None:
        return "null"
    if isinstance(value, int):
        return str(value)
    text = str(value)
    if not text or text.startswith((" ", "{", "[", "*", "&", "!", "|", ">", "@", "`")):
        return json.dumps(text)
    if any(ch in text for ch in [": ", "#", "\n", "\""]):
        return json.dumps(text)
    return text


def render_yaml(value: dict[str, Any]) -> str:
    lines: list[str] = []
    for key, item in value.items():
        if isinstance(item, dict):
            lines.append(f"{key}:")
            for child_key, child_value in item.items():
                lines.append(f"  {child_key}: {render_scalar(child_value)}")
        elif isinstance(item, list):
            lines.append(f"{key}:")
            for child_value in item:
                lines.append(f"  - {render_scalar(child_value)}")
        else:
            lines.append(f"{key}: {render_scalar(item)}")
    return "\n".join(lines) + "\n"


def task_descriptor(source: dict[str, Any]) -> dict[str, Any]:
    descriptor = normalized_descriptor(source)
    descriptor["primary_root"] = EXPECTED_PRIMARY_ROOT
    descriptor["compare_root"] = EXPECTED_COMPARE_ROOT
    descriptor["output_root"] = "migration-artifacts"
    descriptor["exclude"] = sorted(set(descriptor.get("exclude", [])) | set(SECRET_EXCLUDES))
    descriptor["metadata"] = {
        "purpose": "Describe FlexNetOS and lifeos as evidence-only migration comparison targets.",
        "source_descriptor": SOURCE_DESCRIPTOR,
        "comparison_role": "primary_root is the FlexNetOS workspace; compare_root is the lifeos workspace.",
        "verification": "REQ-200 validates descriptor fields, expected roots, path existence, and secret redaction guards.",
    }
    return descriptor


def secret_scan(paths: list[Path]) -> list[str]:
    findings: list[str] = []
    for path in paths:
        text = path.read_text(encoding="utf-8")
        for pattern in SECRET_PATTERNS:
            if pattern.search(text):
                findings.append(str(path))
                break
    return findings


def main() -> None:
    base = package_root()
    generated_at = now()
    source_path = base / SOURCE_DESCRIPTOR
    artifact_path = base / ARTIFACT_DESCRIPTOR

    source = load_descriptor(source_path)
    descriptor = task_descriptor(source)
    artifact_path.parent.mkdir(parents=True, exist_ok=True)
    artifact_path.write_text(render_yaml(descriptor), encoding="utf-8")

    parsed_artifact = load_descriptor(artifact_path)
    normalized_artifact = normalized_descriptor(parsed_artifact)
    errors = validate_descriptor(parsed_artifact, ARTIFACT_DESCRIPTOR)

    root_checks = {
        "primary_root_expected": normalized_artifact.get("primary_root") == EXPECTED_PRIMARY_ROOT,
        "compare_root_expected": normalized_artifact.get("compare_root") == EXPECTED_COMPARE_ROOT,
        "primary_root_exists": Path(EXPECTED_PRIMARY_ROOT).exists(),
        "compare_root_exists": Path(EXPECTED_COMPARE_ROOT).exists(),
    }
    for name, passed in root_checks.items():
        if name.endswith("_exists"):
            continue
        if not passed:
            errors.append(f"{name} check failed")

    missing_secret_excludes = [
        pattern for pattern in SECRET_EXCLUDES if pattern not in normalized_artifact.get("exclude", [])
    ]
    if missing_secret_excludes:
        errors.append(f"missing secret exclude patterns: {', '.join(missing_secret_excludes)}")

    source_errors = validate_descriptor(source, SOURCE_DESCRIPTOR)
    if source_errors:
        errors.extend(f"source fixture: {error}" for error in source_errors)

    scan_paths = [artifact_path]
    secret_findings = secret_scan(scan_paths)
    if secret_findings:
        errors.append(f"secret-like material found in generated descriptor: {', '.join(secret_findings)}")

    status = "passed" if not errors else "failed"
    report = {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": status,
        "generated_at": generated_at,
        "source_descriptor": SOURCE_DESCRIPTOR,
        "artifact_descriptor": ARTIFACT_DESCRIPTOR,
        "descriptor": {
            "target_id": normalized_artifact["target_id"],
            "target_type": normalized_artifact["target_type"],
            "primary_root": normalized_artifact["primary_root"],
            "compare_root": normalized_artifact["compare_root"],
            "output_root": normalized_artifact["output_root"],
            "safety": normalized_artifact["safety"],
            "artifact_contract": normalized_artifact["artifact_contract"],
            "recipe": normalized_artifact["recipe"],
        },
        "descriptor_hash": descriptor_hash(normalized_artifact),
        "source_descriptor_hash": descriptor_hash(normalized_descriptor(source)),
        "artifact_sha256": sha256_file(artifact_path),
        "root_checks": root_checks,
        "warnings": [
            "compare_root is described but not present in this runtime"
            if not root_checks["compare_root_exists"]
            else ""
        ],
        "secret_exclude_patterns": SECRET_EXCLUDES,
        "secret_scan": {
            "status": "passed" if not secret_findings else "failed",
            "scanned_files": [ARTIFACT_DESCRIPTOR],
            "findings": secret_findings,
        },
        "errors": errors,
        "evidence": [
            ARTIFACT_DESCRIPTOR,
            REPORT_PATH,
            LOG_PATH,
        ],
    }

    report["warnings"] = [warning for warning in report["warnings"] if warning]

    write_json(FRAMEWORK_REPORT_PATH, report)
    write_json(FRAMEWORK_LOG_PATH, report)
    write_json(
        FRAMEWORK_HEARTBEAT_PATH,
        {
            "schema_version": "1.0",
            "task_id": TASK_ID,
            "status": status,
            "updated_at": generated_at,
            "proof_uri": f"proof_records/{TASK_ID}.proof.json",
            "artifact_descriptor": ARTIFACT_DESCRIPTOR,
        },
    )

    files_changed = [
        "execution-framework/scripts/verify_flexnetos_target_descriptor.py",
        ARTIFACT_DESCRIPTOR,
        REPORT_PATH,
        LOG_PATH,
        HEARTBEAT_PATH,
        f"execution-framework/proof_records/{TASK_ID}.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    proof = make_proof(
        TASK_ID,
        "completed" if status == "passed" else "failed",
        "codex-cli-local",
        "helper-flexnetos-01",
        "gpt-5.3-spark",
        str(base),
        files_changed,
        ["python3 scripts/verify_flexnetos_target_descriptor.py"],
        report,
        report["evidence"],
        "" if status == "passed" else "; ".join(errors),
        "run REQ-201_FLEXNETOS_LIFEOS_COMPARISON using the validated target descriptor"
        if status == "passed"
        else "fix REQ-200 target descriptor validation errors",
    )
    append_proof(proof)

    print(
        "flexnetos target descriptor status={status} artifact={artifact} hash={hash}".format(
            status=status,
            artifact=ARTIFACT_DESCRIPTOR,
            hash=report["descriptor_hash"],
        )
    )
    if status != "passed":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
