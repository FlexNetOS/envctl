#!/usr/bin/env python3
"""Generate and verify filesystem work boundaries for background helpers."""

from __future__ import annotations

import copy
import fnmatch
import json
import posixpath

from jsonschema import Draft202012Validator

from _common import append_proof, file_checksums, now, package_root, root, write_json


TASK_ID = "REQ-042_FILESYSTEM_BOUNDS"
HELPER_ID = "helper-fs-boundary-01"
MODEL_TAG = "gpt-5.3-spark"
DEPENDENCY_TASK_ID = "REQ-040_SHARED_PROTOCOL_SCHEMAS"


CONFIG_RELPATH = "generated/filesystem_boundaries.json"
SCHEMA_RELPATH = "schemas/filesystem_boundaries.schema.json"
REPORT_RELPATH = "generated/filesystem_boundary_validation_report.json"
DOC_RELPATH = "docs/FILESYSTEM_BOUNDARIES.md"


def read_json_from_root(relpath: str) -> dict:
    return json.loads((root() / relpath).read_text(encoding="utf-8"))


def read_package_text(relpath: str) -> str:
    return (package_root() / relpath).read_text(encoding="utf-8")


def normalize_logical_path(path: str) -> str:
    if path.startswith("${"):
        return path.rstrip("/")
    normalized = posixpath.normpath(path)
    if normalized == ".":
        return ""
    return normalized


def pattern_prefix(pattern: str) -> str:
    if pattern.endswith("/**"):
        return pattern[:-3]
    return pattern


def matches_allow(path: str, pattern: str) -> bool:
    prefix = pattern_prefix(pattern)
    return path == prefix or path.startswith(f"{prefix}/")


def evaluate_path(path: str, allowed_patterns: list[str], blocked_patterns: list[str]) -> dict:
    logical_path = normalize_logical_path(path)
    blocked_by = [
        pattern for pattern in blocked_patterns if fnmatch.fnmatch(logical_path, pattern)
    ]
    allowed_by = [
        pattern for pattern in allowed_patterns if matches_allow(logical_path, pattern)
    ]
    if blocked_by:
        decision = "blocked"
        reason = "blocked_pattern"
    elif allowed_by:
        decision = "allowed"
        reason = "allowed_pattern"
    else:
        decision = "blocked"
        reason = "outside_allowed_paths"
    return {
        "path": path,
        "normalized_path": logical_path,
        "decision": decision,
        "reason": reason,
        "allowed_by": allowed_by,
        "blocked_by": blocked_by,
    }


def boundary_schema() -> dict:
    return {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://flexnetos.local/envctl/filesystem-boundaries.schema.json",
        "title": "envctl Background Helper Filesystem Boundaries",
        "type": "object",
        "required": [
            "schema_version",
            "task_id",
            "scope",
            "allowed_paths",
            "blocked_paths",
            "safe_workspaces",
            "rules",
        ],
        "properties": {
            "schema_version": {"type": "string"},
            "task_id": {"type": "string"},
            "scope": {"type": "string"},
            "repo_path": {"type": "string"},
            "allowed_paths": {
                "type": "array",
                "items": {"$ref": "#/$defs/pathRule"},
                "minItems": 1,
            },
            "blocked_paths": {
                "type": "array",
                "items": {"$ref": "#/$defs/pathRule"},
                "minItems": 1,
            },
            "safe_workspaces": {
                "type": "array",
                "items": {"$ref": "#/$defs/workspace"},
                "minItems": 1,
            },
            "rules": {
                "type": "array",
                "items": {"type": "string", "minLength": 1},
                "minItems": 1,
            },
            "redaction": {"type": "object"},
            "metadata": {"type": "object"},
        },
        "$defs": {
            "pathRule": {
                "type": "object",
                "required": ["id", "pattern", "mode", "description"],
                "properties": {
                    "id": {"type": "string", "minLength": 1},
                    "pattern": {"type": "string", "minLength": 1},
                    "mode": {"enum": ["allow", "block"]},
                    "env_var": {"type": ["string", "null"]},
                    "description": {"type": "string", "minLength": 1},
                },
                "additionalProperties": True,
            },
            "workspace": {
                "type": "object",
                "required": ["id", "path", "purpose", "write_policy"],
                "properties": {
                    "id": {"type": "string", "minLength": 1},
                    "path": {"type": "string", "minLength": 1},
                    "purpose": {"type": "string", "minLength": 1},
                    "write_policy": {"type": "string", "minLength": 1},
                    "retention": {"type": "string"},
                },
                "additionalProperties": True,
            },
        },
        "additionalProperties": True,
    }


def build_config(packet: dict, redaction_patterns: list[str]) -> dict:
    allowed_patterns = packet["allowed_paths"]
    blocked_patterns = packet["blocked_paths"]
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "generated_at": now(),
        "scope": packet["filesystem_scope"],
        "repo_path": packet["repo_path"],
        "evaluation_order": [
            "normalize logical path",
            "apply blocked_paths first",
            "apply allowed_paths second",
            "deny paths that match neither list",
        ],
        "allowed_paths": [
            {
                "id": "envctl_repo",
                "pattern": "${ENVCTL_REPO}/**",
                "mode": "allow",
                "env_var": "ENVCTL_REPO",
                "description": "envctl repository root selected by the operator.",
            },
            {
                "id": "nu_plugin_repo",
                "pattern": "${NU_PLUGIN_REPO}/**",
                "mode": "allow",
                "env_var": "NU_PLUGIN_REPO",
                "description": "nu_plugin repository root selected by the operator.",
            },
            {
                "id": "execution_framework",
                "pattern": "execution-framework/**",
                "mode": "allow",
                "env_var": None,
                "description": "package-local execution framework state, generated artifacts, logs, and proofs.",
            },
            {
                "id": "execution_templates",
                "pattern": "execution-templates/**",
                "mode": "allow",
                "env_var": None,
                "description": "package-local reusable execution templates.",
            },
        ],
        "blocked_paths": [
            {
                "id": "dot_env_files",
                "pattern": "**/.env",
                "mode": "block",
                "env_var": None,
                "description": "environment files are never read into logs, prompts, proofs, or generated artifacts.",
            },
            {
                "id": "secrets_directories",
                "pattern": "**/secrets/**",
                "mode": "block",
                "env_var": None,
                "description": "secret directories are outside helper read and write scope.",
            },
            {
                "id": "private_key_directories",
                "pattern": "**/private_keys/**",
                "mode": "block",
                "env_var": None,
                "description": "private key directories are outside helper read and write scope.",
            },
            {
                "id": "pem_files",
                "pattern": "**/*.pem",
                "mode": "block",
                "env_var": None,
                "description": "PEM material is excluded even inside otherwise allowed roots.",
            },
            {
                "id": "key_files",
                "pattern": "**/*.key",
                "mode": "block",
                "env_var": None,
                "description": "key material is excluded even inside otherwise allowed roots.",
            },
        ],
        "safe_workspaces": [
            {
                "id": "helper_state",
                "path": "execution-framework/state/helpers/${HELPER_ID}/",
                "purpose": "helper-local scratch state and resumable progress markers",
                "write_policy": "append-or-replace files owned by the active helper_id only",
                "retention": "task lifetime unless retained as evidence",
            },
            {
                "id": "generated_artifacts",
                "path": "execution-framework/generated/",
                "purpose": "machine-readable task artifacts and validation reports",
                "write_policy": "additive task-specific files; do not overwrite unrelated task output",
                "retention": "package artifact",
            },
            {
                "id": "logs",
                "path": "execution-framework/logs/",
                "purpose": "redacted command and verification logs",
                "write_policy": "task-specific log files named by task_id",
                "retention": "package artifact",
            },
            {
                "id": "proof_records",
                "path": "execution-framework/proof_records/",
                "purpose": "task proof JSON and proof ledger entries",
                "write_policy": "task-specific proof file plus proof_ledger.jsonl append/update",
                "retention": "package artifact",
            },
        ],
        "rules": [
            "Blocked patterns take precedence over allowed paths.",
            "Background helpers must not read or write paths that match blocked_paths.",
            "Background helpers may read and write only paths that match allowed_paths and do not match blocked_paths.",
            "ENVCTL_REPO and NU_PLUGIN_REPO are operator-provided roots; if unset, helpers must treat those lanes as unavailable.",
            "Helper scratch writes must use execution-framework/state/helpers/${HELPER_ID}/ unless the task packet names a narrower output path.",
            "Logs and proof evidence must be redacted using helpers/redaction_patterns.txt before capture.",
            "Symlink traversal must be evaluated against the final resolved path and denied when it escapes the matched allowed root.",
        ],
        "redaction": {
            "source": "helpers/redaction_patterns.txt",
            "pattern_count": len(redaction_patterns),
            "capture_policy": "logs, proof evidence, and generated reports must not contain matched secret values",
        },
        "metadata": {
            "source_packet": "execution-framework/generated/execution_packets/REQ-042_FILESYSTEM_BOUNDS.json",
            "source_graph": packet["source_graph_uri"],
            "dependency": DEPENDENCY_TASK_ID,
        },
    }


def validate_config(config: dict, schema: dict, packet: dict) -> dict:
    errors = []
    Draft202012Validator.check_schema(schema)
    schema_errors = sorted(
        Draft202012Validator(schema).iter_errors(config), key=lambda err: err.path
    )
    errors.extend(f"schema: {error.message}" for error in schema_errors)

    declared_allowed = [entry["pattern"] for entry in config["allowed_paths"]]
    declared_blocked = [entry["pattern"] for entry in config["blocked_paths"]]
    if declared_allowed != packet["allowed_paths"]:
        errors.append("allowed_paths do not match packet declaration")
    if declared_blocked != packet["blocked_paths"]:
        errors.append("blocked_paths do not match packet declaration")

    dependency = read_json_from_root(f"proof_records/{DEPENDENCY_TASK_ID}.proof.json")
    dependency_status = dependency.get("status")
    if dependency_status != "completed":
        errors.append(f"{DEPENDENCY_TASK_ID} proof status is {dependency_status!r}")

    samples = [
        ("execution-framework/generated/task_graph.csv", "allowed"),
        ("execution-framework/logs/REQ-042_FILESYSTEM_BOUNDS.log", "allowed"),
        ("execution-templates/TASK_GRAPH_TEMPLATE.csv", "allowed"),
        ("${ENVCTL_REPO}/src/lib.rs", "allowed"),
        ("${NU_PLUGIN_REPO}/src/lib.rs", "allowed"),
        ("execution-framework/.env", "blocked"),
        ("execution-framework/secrets/token.txt", "blocked"),
        ("${ENVCTL_REPO}/private_keys/id_rsa", "blocked"),
        ("${NU_PLUGIN_REPO}/certs/local.pem", "blocked"),
        ("${ENVCTL_REPO}/config/local.key", "blocked"),
        ("history/pre_execution_framework_manifest.json", "blocked"),
        ("../outside-package.txt", "blocked"),
        ("../execution-framework/generated/task_graph.csv", "blocked"),
    ]
    sample_results = []
    for path, expected in samples:
        result = evaluate_path(path, declared_allowed, declared_blocked)
        result["expected"] = expected
        result["passed"] = result["decision"] == expected
        if not result["passed"]:
            errors.append(
                f"{path} resolved to {result['decision']}, expected {expected}"
            )
        sample_results.append(result)

    workspace_paths = [workspace["path"] for workspace in config["safe_workspaces"]]
    safe_workspace_results = [
        evaluate_path(path.replace("${HELPER_ID}", HELPER_ID), declared_allowed, declared_blocked)
        for path in workspace_paths
    ]
    for result in safe_workspace_results:
        if result["decision"] != "allowed":
            errors.append(f"safe workspace is not allowed: {result['path']}")

    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "passed" if not errors else "failed",
        "generated_at": config["generated_at"],
        "summary": {
            "allowed_path_count": len(config["allowed_paths"]),
            "blocked_path_count": len(config["blocked_paths"]),
            "safe_workspace_count": len(config["safe_workspaces"]),
            "redaction_pattern_count": config["redaction"]["pattern_count"],
            "path_sample_count": len(sample_results),
            "dependency_status": dependency_status,
        },
        "sample_results": sample_results,
        "safe_workspace_results": safe_workspace_results,
        "errors": errors,
        "evidence": [
            f"execution-framework/{SCHEMA_RELPATH}",
            f"execution-framework/{CONFIG_RELPATH}",
            f"execution-framework/{REPORT_RELPATH}",
            f"execution-framework/{DOC_RELPATH}",
            "helpers/redaction_patterns.txt",
            f"execution-framework/proof_records/{DEPENDENCY_TASK_ID}.proof.json",
        ],
    }


def render_docs(config: dict, validation: dict) -> str:
    lines = [
        "# Filesystem Boundaries",
        "",
        f"Task: `{TASK_ID}`",
        f"Scope: `{config['scope']}`",
        f"Generated at: `{config['generated_at']}`",
        "",
        "## Decision Order",
        "",
    ]
    lines.extend(f"- {item}" for item in config["evaluation_order"])
    lines.extend(["", "## Allowed Paths", "", "| id | pattern | source | purpose |", "|---|---|---|---|"])
    for entry in config["allowed_paths"]:
        source = entry["env_var"] or "package"
        lines.append(
            f"| `{entry['id']}` | `{entry['pattern']}` | `{source}` | {entry['description']} |"
        )
    lines.extend(["", "## Blocked Paths", "", "| id | pattern | reason |", "|---|---|---|"])
    for entry in config["blocked_paths"]:
        lines.append(f"| `{entry['id']}` | `{entry['pattern']}` | {entry['description']} |")
    lines.extend(["", "## Safe Workspaces", "", "| id | path | write policy |", "|---|---|---|"])
    for workspace in config["safe_workspaces"]:
        lines.append(
            f"| `{workspace['id']}` | `{workspace['path']}` | {workspace['write_policy']} |"
        )
    lines.extend(["", "## Helper Rules", ""])
    lines.extend(f"- {rule}" for rule in config["rules"])
    lines.extend(
        [
            "",
            "## Verification",
            "",
            f"Status: `{validation['status']}`",
            "",
            "| check | value |",
            "|---|---|",
        ]
    )
    for key, value in validation["summary"].items():
        lines.append(f"| `{key}` | `{value}` |")
    lines.extend(["", "Path samples:", ""])
    for result in validation["sample_results"]:
        lines.append(
            "- `{path}`: `{decision}` via `{reason}`".format(**result)
        )
    if validation["errors"]:
        lines.extend(["", "Errors:", ""])
        lines.extend(f"- {error}" for error in validation["errors"])
    else:
        lines.extend(["", "No filesystem boundary errors were found."])
    lines.append("")
    return "\n".join(lines)


def write_text(relpath: str, text: str) -> None:
    path = root() / relpath
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def main() -> int:
    started_at = now()
    packet = read_json_from_root("generated/execution_packets/REQ-042_FILESYSTEM_BOUNDS.json")
    redaction_patterns = [
        line.strip()
        for line in read_package_text("helpers/redaction_patterns.txt").splitlines()
        if line.strip()
    ]
    schema = boundary_schema()
    config = build_config(packet, redaction_patterns)
    validation = validate_config(config, schema, packet)

    write_json(SCHEMA_RELPATH, schema)
    write_json(CONFIG_RELPATH, config)
    write_json(REPORT_RELPATH, validation)
    write_text(DOC_RELPATH, render_docs(config, validation))

    heartbeat = {
        "task_id": TASK_ID,
        "status": validation["status"],
        "updated_at": now(),
        "helper_id": HELPER_ID,
        "evidence": validation["evidence"],
    }
    write_json(f"state/{TASK_ID}.heartbeat.json", heartbeat)
    write_json(f"logs/{TASK_ID}.log", validation)

    files_changed = [
        "execution-framework/scripts/verify_filesystem_boundaries.py",
        f"execution-framework/{SCHEMA_RELPATH}",
        f"execution-framework/{CONFIG_RELPATH}",
        f"execution-framework/{REPORT_RELPATH}",
        f"execution-framework/{DOC_RELPATH}",
        f"execution-framework/state/{TASK_ID}.heartbeat.json",
        f"execution-framework/logs/{TASK_ID}.log",
        f"execution-framework/proof_records/{TASK_ID}.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    proof = {
        "proof_schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "completed" if validation["status"] == "passed" else "failed",
        "started_at": started_at,
        "completed_at": now(),
        "actor": "codex-cli-local",
        "helper_id": HELPER_ID,
        "model_tag": MODEL_TAG,
        "repo_path": str(package_root()),
        "files_changed": files_changed,
        "commands_run": [
            "python3 scripts/verify_filesystem_boundaries.py",
        ],
        "verification_output": copy.deepcopy(validation),
        "checksums": file_checksums(files_changed[:-2]),
        "logs_uri": f"logs/{TASK_ID}.log",
        "rollback_point": "history/pre_execution_framework_manifest.json",
        "evidence": validation["evidence"] + [f"logs/{TASK_ID}.log"],
        "failure_reason": "" if validation["status"] == "passed" else "; ".join(validation["errors"]),
        "next_action": "run VER-302_PACKET_SCHEMA_VALIDATION with generated/filesystem_boundaries.json available",
    }
    append_proof(proof)
    return 0 if validation["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
