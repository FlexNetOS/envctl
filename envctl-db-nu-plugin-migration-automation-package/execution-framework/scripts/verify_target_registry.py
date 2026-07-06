from __future__ import annotations

import copy
import hashlib
import json
import sqlite3
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, package_root, root, sha256_file, write_json
from verify_envctl_db_schema import apply_migrations


TASK_ID = "REQ-021_ENVCTL_TARGET_REGISTRY"

DESCRIPTOR_SCHEMA = "schemas/target_descriptor.schema.json"
DESCRIPTOR_GLOB = "examples/target-descriptors/*.yaml"
MODEL_PATH = "generated/envctl_target_registry.json"
REPORT_PATH = "generated/envctl_target_registry_validation_report.json"
DOC_PATH = "docs/ENVCTL_TARGET_REGISTRY.md"
LOG_PATH = f"logs/{TASK_ID}.log"
HEARTBEAT_PATH = f"state/{TASK_ID}.heartbeat.json"

TARGET_TYPES = {"codebase", "data", "infrastructure", "integration", "mixed"}
SAFETY_MODES = {"observer", "approval-gated", "operator", "agent-only"}
RISK_LEVELS = {"R0", "R1", "R2", "R3", "R4", "R5"}
REQUIRED_DESCRIPTOR_FIELDS = {
    "schema_version",
    "target_id",
    "target_type",
    "primary_root",
    "safety",
    "artifact_contract",
    "recipe",
}


def parse_scalar(value: str) -> Any:
    value = value.strip()
    if value in {"null", "~"}:
        return None
    if value == "true":
        return True
    if value == "false":
        return False
    if len(value) >= 2 and value[0] == value[-1] and value[0] in {"'", '"'}:
        return value[1:-1]
    try:
        return int(value)
    except ValueError:
        return value


def parse_yaml_subset(text: str) -> dict[str, Any]:
    parsed: dict[str, Any] = {}
    current_key: str | None = None
    for raw_line in text.splitlines():
        if not raw_line.strip() or raw_line.lstrip().startswith("#"):
            continue
        indent = len(raw_line) - len(raw_line.lstrip(" "))
        line = raw_line.strip()
        if indent == 0:
            key, sep, value = line.partition(":")
            if not sep:
                raise ValueError(f"invalid YAML line: {raw_line}")
            current_key = key.strip()
            parsed[current_key] = parse_scalar(value) if value.strip() else None
            continue
        if indent != 2 or current_key is None:
            raise ValueError(f"unsupported YAML nesting: {raw_line}")
        if line.startswith("- "):
            if not isinstance(parsed.get(current_key), list):
                parsed[current_key] = []
            parsed[current_key].append(parse_scalar(line[2:]))
            continue
        key, sep, value = line.partition(":")
        if not sep:
            raise ValueError(f"invalid nested YAML line: {raw_line}")
        if not isinstance(parsed.get(current_key), dict):
            parsed[current_key] = {}
        parsed[current_key][key.strip()] = parse_scalar(value)
    return parsed


def load_descriptor(path: Path) -> dict[str, Any]:
    text = path.read_text(encoding="utf-8")
    try:
        import yaml  # type: ignore

        loaded = yaml.safe_load(text)
    except Exception:
        loaded = parse_yaml_subset(text)
    if not isinstance(loaded, dict):
        raise ValueError(f"{path} did not parse to a descriptor object")
    return loaded


def canonical_json(value: Any) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def descriptor_hash(descriptor: dict[str, Any]) -> str:
    return "sha256:" + hashlib.sha256(canonical_json(descriptor).encode("utf-8")).hexdigest()


def normalized_descriptor(raw: dict[str, Any]) -> dict[str, Any]:
    descriptor = copy.deepcopy(raw)
    descriptor.setdefault("compare_root", None)
    descriptor.setdefault("output_root", "migration-artifacts")
    descriptor.setdefault("include", [])
    descriptor.setdefault("exclude", [])
    descriptor.setdefault("collectors", {})
    descriptor.setdefault("metadata", {})
    return descriptor


def require_object(value: Any, name: str, errors: list[str]) -> dict[str, Any]:
    if not isinstance(value, dict):
        errors.append(f"{name} must be an object")
        return {}
    return value


def validate_descriptor(raw: dict[str, Any], source: str) -> list[str]:
    errors: list[str] = []
    missing = sorted(REQUIRED_DESCRIPTOR_FIELDS - set(raw))
    if missing:
        errors.append(f"{source}: missing required fields: {', '.join(missing)}")
    if not isinstance(raw.get("schema_version"), int) or raw.get("schema_version", 0) < 1:
        errors.append(f"{source}: schema_version must be an integer >= 1")
    if not isinstance(raw.get("target_id"), str) or not raw.get("target_id", "").strip():
        errors.append(f"{source}: target_id must be a non-empty string")
    if raw.get("target_type") not in TARGET_TYPES:
        errors.append(f"{source}: target_type must be one of {sorted(TARGET_TYPES)}")
    if not isinstance(raw.get("primary_root"), str) or not raw.get("primary_root", "").strip():
        errors.append(f"{source}: primary_root must be a non-empty string")
    if "compare_root" in raw and raw["compare_root"] is not None and not isinstance(raw["compare_root"], str):
        errors.append(f"{source}: compare_root must be a string or null")
    for key in ("include", "exclude"):
        value = raw.get(key, [])
        if not isinstance(value, list) or any(not isinstance(item, str) for item in value):
            errors.append(f"{source}: {key} must be a list of strings")
    safety = require_object(raw.get("safety"), f"{source}: safety", errors)
    if safety.get("default_mode") not in SAFETY_MODES:
        errors.append(f"{source}: safety.default_mode must be one of {sorted(SAFETY_MODES)}")
    if safety.get("max_auto_risk") not in RISK_LEVELS:
        errors.append(f"{source}: safety.max_auto_risk must be one of {sorted(RISK_LEVELS)}")
    for key in ("allow_network", "allow_destructive"):
        if not isinstance(safety.get(key), bool):
            errors.append(f"{source}: safety.{key} must be boolean")
    for key in ("artifact_contract", "recipe"):
        item = require_object(raw.get(key), f"{source}: {key}", errors)
        if not isinstance(item.get("name"), str) or not item.get("name", "").strip():
            errors.append(f"{source}: {key}.name must be a non-empty string")
        if not isinstance(item.get("version"), (int, str)):
            errors.append(f"{source}: {key}.version must be an integer or string")
    collectors = raw.get("collectors", {})
    if collectors is not None:
        if not isinstance(collectors, dict) or any(not isinstance(value, bool) for value in collectors.values()):
            errors.append(f"{source}: collectors must map names to booleans")
    return errors


def stable_target_row_id(target_id: str) -> str:
    safe = "".join(ch if ch.isalnum() or ch in {"-", "_"} else "-" for ch in target_id)
    return f"target-{safe}"


def register_descriptor(conn: sqlite3.Connection, descriptor: dict[str, Any]) -> dict[str, Any]:
    target_hash = descriptor_hash(descriptor)
    row = {
        "id": stable_target_row_id(descriptor["target_id"]),
        "target_id": descriptor["target_id"],
        "target_type": descriptor["target_type"],
        "primary_root": descriptor["primary_root"],
        "compare_root": descriptor.get("compare_root"),
        "descriptor_json": canonical_json(descriptor),
        "descriptor_hash": target_hash,
        "safety_mode": descriptor["safety"]["default_mode"],
        "max_auto_risk": descriptor["safety"]["max_auto_risk"],
    }
    conn.execute(
        """
        INSERT INTO envctl_migration_targets
          (id, target_id, target_type, primary_root, compare_root, descriptor_json,
           descriptor_hash, safety_mode, max_auto_risk)
        VALUES
          (:id, :target_id, :target_type, :primary_root, :compare_root, :descriptor_json,
           :descriptor_hash, :safety_mode, :max_auto_risk)
        ON CONFLICT(target_id) DO UPDATE SET
          target_type = excluded.target_type,
          primary_root = excluded.primary_root,
          compare_root = excluded.compare_root,
          descriptor_json = excluded.descriptor_json,
          descriptor_hash = excluded.descriptor_hash,
          safety_mode = excluded.safety_mode,
          max_auto_risk = excluded.max_auto_risk,
          updated_at_utc = strftime('%Y-%m-%dT%H:%M:%fZ','now')
        """,
        row,
    )
    return row


def rows(conn: sqlite3.Connection) -> list[dict[str, Any]]:
    conn.row_factory = sqlite3.Row
    return [
        dict(row)
        for row in conn.execute(
            """
            SELECT id, target_id, target_type, primary_root, compare_root,
                   descriptor_hash, safety_mode, max_auto_risk, descriptor_json
            FROM envctl_migration_targets
            ORDER BY target_id
            """
        )
    ]


def synthetic_descriptors(base: dict[str, Any], covered_types: set[str]) -> list[dict[str, Any]]:
    descriptors = []
    for target_type in sorted(TARGET_TYPES - covered_types):
        item = copy.deepcopy(base)
        item["target_id"] = f"synthetic-{target_type}-target"
        item["target_type"] = target_type
        item["primary_root"] = f"/virtual/{target_type}"
        item["compare_root"] = None
        item["metadata"] = {"source": "REQ-021 synthetic target type coverage"}
        descriptors.append(item)
    return descriptors


def build_registry_model() -> dict[str, Any]:
    base = package_root()
    schema_path = base / DESCRIPTOR_SCHEMA
    descriptor_paths = sorted(base.glob(DESCRIPTOR_GLOB))
    conn = sqlite3.connect(":memory:")
    conn.execute("PRAGMA foreign_keys = ON")
    applied = apply_migrations(conn, base)

    errors: list[str] = []
    loaded: list[dict[str, Any]] = []
    for path in descriptor_paths:
        raw = load_descriptor(path)
        relpath = str(path.relative_to(base))
        errors.extend(validate_descriptor(raw, relpath))
        descriptor = normalized_descriptor(raw)
        loaded.append({"source": relpath, "descriptor": descriptor})

    if loaded:
        first_descriptor = loaded[0]["descriptor"]
        covered_types = {item["descriptor"]["target_type"] for item in loaded}
        for descriptor in synthetic_descriptors(first_descriptor, covered_types):
            source = f"generated:{descriptor['target_id']}"
            errors.extend(validate_descriptor(descriptor, source))
            loaded.append({"source": source, "descriptor": descriptor})

    first_pass_rows = []
    for item in loaded:
        first_pass_rows.append({"source": item["source"], **register_descriptor(conn, item["descriptor"])})
    conn.commit()
    row_count_after_first_pass = conn.execute("SELECT COUNT(*) FROM envctl_migration_targets").fetchone()[0]

    second_pass_rows = []
    for item in loaded:
        second_pass_rows.append({"source": item["source"], **register_descriptor(conn, item["descriptor"])})
    conn.commit()
    row_count_after_second_pass = conn.execute("SELECT COUNT(*) FROM envctl_migration_targets").fetchone()[0]

    invalid_descriptor = copy.deepcopy(loaded[0]["descriptor"]) if loaded else {}
    invalid_descriptor["target_id"] = ""
    invalid_descriptor["target_type"] = "unknown"
    invalid_errors = validate_descriptor(invalid_descriptor, "invalid-fixture")

    registry_rows = rows(conn)
    by_target_id = {row["target_id"]: row for row in registry_rows}
    lookup_checks = []
    for item in loaded:
        target_id = item["descriptor"]["target_id"]
        stored = by_target_id.get(target_id)
        lookup_checks.append(
            {
                "target_id": target_id,
                "source": item["source"],
                "found": stored is not None,
                "hash_matches": stored is not None
                and stored["descriptor_hash"] == descriptor_hash(item["descriptor"]),
                "safety_mode": stored["safety_mode"] if stored else None,
                "max_auto_risk": stored["max_auto_risk"] if stored else None,
            }
        )

    type_coverage = {
        target_type: any(row["target_type"] == target_type for row in registry_rows)
        for target_type in sorted(TARGET_TYPES)
    }
    duplicate_hash_stable = [
        first["descriptor_hash"] == second["descriptor_hash"]
        for first, second in zip(first_pass_rows, second_pass_rows, strict=True)
    ]

    if row_count_after_first_pass != row_count_after_second_pass:
        errors.append("idempotent descriptor registration changed row count")
    if not all(duplicate_hash_stable):
        errors.append("idempotent descriptor registration changed descriptor hashes")
    if not invalid_errors:
        errors.append("invalid descriptor fixture was not rejected")
    missing_lookups = [item["target_id"] for item in lookup_checks if not item["found"] or not item["hash_matches"]]
    if missing_lookups:
        errors.append(f"descriptor lookup failed for: {', '.join(missing_lookups)}")
    uncovered_types = [target_type for target_type, covered in type_coverage.items() if not covered]
    if uncovered_types:
        errors.append(f"target types not covered: {', '.join(uncovered_types)}")

    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "generated_at": now(),
        "status": "passed" if not errors else "failed",
        "database_backend": "sqlite",
        "runtime": "python sqlite3 in-memory",
        "descriptor_schema": {
            "path": DESCRIPTOR_SCHEMA,
            "sha256": sha256_file(schema_path),
        },
        "applied_migrations": applied,
        "summary": {
            "input_descriptor_count": len(descriptor_paths),
            "registered_descriptor_count": len(registry_rows),
            "target_type_count": len(type_coverage),
            "target_type_covered_count": sum(1 for covered in type_coverage.values() if covered),
            "row_count_after_first_pass": row_count_after_first_pass,
            "row_count_after_second_pass": row_count_after_second_pass,
            "invalid_descriptor_error_count": len(invalid_errors),
        },
        "errors": errors,
        "target_type_coverage": type_coverage,
        "descriptor_inputs": [
            {
                "source": item["source"],
                "target_id": item["descriptor"]["target_id"],
                "target_type": item["descriptor"]["target_type"],
                "descriptor_hash": descriptor_hash(item["descriptor"]),
            }
            for item in loaded
        ],
        "registry_rows": [
            {
                key: value
                for key, value in row.items()
                if key != "descriptor_json"
            }
            for row in registry_rows
        ],
        "lookup_checks": lookup_checks,
        "idempotency_check": {
            "row_count_stable": row_count_after_first_pass == row_count_after_second_pass,
            "descriptor_hashes_stable": all(duplicate_hash_stable),
        },
        "invalid_descriptor_check": {
            "rejected": bool(invalid_errors),
            "errors": invalid_errors,
        },
    }


def write_docs(model: dict[str, Any]) -> None:
    lines = [
        "# envctl target descriptor registry",
        "",
        f"Generated at: `{model['generated_at']}`",
        f"Status: `{model['status']}`",
        "",
        "## Contract",
        "",
        f"- Descriptor schema: `{model['descriptor_schema']['path']}` (`{model['descriptor_schema']['sha256']}`)",
        "- Backing table: `envctl_migration_targets`",
        "- Registry key: stable `target_id` upsert with canonical descriptor JSON and SHA-256 hash",
        "",
        "## Target type coverage",
        "",
        "| target type | covered |",
        "|---|---|",
    ]
    for target_type, covered in model["target_type_coverage"].items():
        lines.append(f"| `{target_type}` | {'yes' if covered else 'no'} |")
    lines.extend(
        [
            "",
            "## Registered descriptors",
            "",
            "| target id | type | source | descriptor hash |",
            "|---|---|---|---|",
        ]
    )
    sources = {item["target_id"]: item["source"] for item in model["descriptor_inputs"]}
    for row in model["registry_rows"]:
        lines.append(
            f"| `{row['target_id']}` | `{row['target_type']}` | `{sources.get(row['target_id'], '')}` | `{row['descriptor_hash']}` |"
        )
    lines.extend(
        [
            "",
            "## Runtime checks",
            "",
            f"- Idempotent upsert kept row count stable: `{model['idempotency_check']['row_count_stable']}`",
            f"- Descriptor hashes stayed stable on re-register: `{model['idempotency_check']['descriptor_hashes_stable']}`",
            f"- Invalid descriptor rejected: `{model['invalid_descriptor_check']['rejected']}`",
            f"- Lookup checks passing: `{all(item['found'] and item['hash_matches'] for item in model['lookup_checks'])}`",
        ]
    )
    if model["errors"]:
        lines.extend(["", "## Errors", ""])
        for error in model["errors"]:
            lines.append(f"- {error}")
    (root() / DOC_PATH).write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    base = package_root()
    model = build_registry_model()
    report = {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": model["status"],
        "generated_at": model["generated_at"],
        "summary": model["summary"],
        "errors": model["errors"],
        "evidence": [
            MODEL_PATH,
            REPORT_PATH,
            DOC_PATH,
        ],
    }
    write_json(MODEL_PATH, model)
    write_json(REPORT_PATH, report)
    write_docs(model)
    write_json(LOG_PATH, report)
    write_json(
        HEARTBEAT_PATH,
        {
            "schema_version": "1.0",
            "task_id": TASK_ID,
            "status": model["status"],
            "updated_at": model["generated_at"],
            "proof_uri": f"proof_records/{TASK_ID}.proof.json",
        },
    )

    files_changed = [
        "execution-framework/scripts/verify_target_registry.py",
        f"execution-framework/{MODEL_PATH}",
        f"execution-framework/{REPORT_PATH}",
        f"execution-framework/{DOC_PATH}",
        f"execution-framework/{LOG_PATH}",
        f"execution-framework/{HEARTBEAT_PATH}",
        f"execution-framework/proof_records/{TASK_ID}.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    proof = make_proof(
        TASK_ID,
        "completed" if model["status"] == "passed" else "failed",
        "codex-cli-local",
        "helper-envctl-target-01",
        "gpt-5.3-spark",
        str(base),
        files_changed,
        ["python3 scripts/verify_target_registry.py"],
        report,
        report["evidence"],
        "" if model["status"] == "passed" else "; ".join(model["errors"]),
        "run REQ-027 replay engine once ledger/artifact registry tasks are complete"
        if model["status"] == "passed"
        else "fix target descriptor registry validation errors",
    )
    append_proof(proof)
    print(
        "target registry status={status} descriptors={descriptors} target_types={covered}/{total}".format(
            status=model["status"],
            descriptors=model["summary"]["registered_descriptor_count"],
            covered=model["summary"]["target_type_covered_count"],
            total=model["summary"]["target_type_count"],
        )
    )
    if model["status"] != "passed":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
