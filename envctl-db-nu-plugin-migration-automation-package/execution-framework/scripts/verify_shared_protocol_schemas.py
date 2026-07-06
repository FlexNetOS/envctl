from __future__ import annotations

import copy
import json
from pathlib import Path

from jsonschema import Draft202012Validator

from _common import append_proof, file_checksums, now, package_root, root, write_json


TASK_ID = "REQ-040_SHARED_PROTOCOL_SCHEMAS"
HELPER_ID = "helper-shared-schema-01"
MODEL_TAG = "gpt-5.3-spark"
PROTOCOL_VERSION = "1.0.0"

SOURCE_SCHEMAS = {
    "Operation": "schemas/operation.schema.json",
    "RunEvent": "schemas/run_event.schema.json",
    "ProofRecord": "execution-framework/schemas/proof_record.schema.json",
    "ArtifactRecord": "schemas/artifact_record.schema.json",
    "ApprovalRequest": "schemas/approval_request.schema.json",
    "TargetDescriptor": "schemas/target_descriptor.schema.json",
    "MigrationRecipe": "schemas/migration_recipe.schema.json",
    "ValidationResult": "schemas/validation_result.schema.json",
}

REQUIRED_RECORDS = [
    "TargetDescriptor",
    "MigrationRecipe",
    "MigrationRun",
    "RunEvent",
    "Operation",
    "ApprovalRequest",
    "ApprovalDecision",
    "ArtifactRecord",
    "EvidenceRecord",
    "GraphEdge",
    "ValidationResult",
    "ReplayRequest",
    "ReplayResult",
    "ProofRecord",
]


def read_json(base: Path, relpath: str) -> dict:
    return json.loads((base / relpath).read_text(encoding="utf-8"))


def rewrite_internal_refs(schema: object, def_name: str) -> object:
    if isinstance(schema, dict):
        out = {}
        for key, value in schema.items():
            if key == "$ref" and isinstance(value, str) and value.startswith("#/$defs/"):
                out[key] = value.replace("#/$defs/", f"#/$defs/{def_name}/$defs/", 1)
            else:
                out[key] = rewrite_internal_refs(value, def_name)
        return out
    if isinstance(schema, list):
        return [rewrite_internal_refs(value, def_name) for value in schema]
    return schema


def imported_schema(base: Path, def_name: str, relpath: str) -> dict:
    schema = read_json(base, relpath)
    schema = rewrite_internal_refs(schema, def_name)
    schema.pop("$schema", None)
    schema["x-shared-protocol-source"] = relpath
    return schema


def string_or_null() -> dict:
    return {"type": ["string", "null"]}


def missing_record_defs() -> dict:
    risk_enum = ["R0", "R1", "R2", "R3", "R4", "R5"]
    return {
        "MigrationRun": {
            "title": "envctl Migration Run",
            "type": "object",
            "required": [
                "run_id",
                "target_id",
                "recipe_id",
                "artifact_contract_id",
                "status",
                "created_at_utc",
            ],
            "properties": {
                "schema_version": {"type": "integer", "minimum": 1, "default": 1},
                "run_id": {"type": "string", "minLength": 1},
                "target_id": {"type": "string", "minLength": 1},
                "recipe_id": {"type": "string", "minLength": 1},
                "artifact_contract_id": {"type": "string", "minLength": 1},
                "status": {
                    "enum": [
                        "queued",
                        "running",
                        "succeeded",
                        "failed",
                        "blocked",
                        "cancelled",
                    ]
                },
                "human_mode": {
                    "enum": ["unattended", "supervised", "manual"],
                    "default": "supervised",
                },
                "max_auto_risk": {"enum": risk_enum},
                "reproducibility_hash": string_or_null(),
                "created_at_utc": {"type": "string"},
                "updated_at_utc": string_or_null(),
                "metadata": {"type": "object"},
            },
            "additionalProperties": True,
        },
        "ApprovalDecision": {
            "title": "envctl Migration Approval Decision",
            "type": "object",
            "required": ["approval_id", "decision", "decided_by", "decided_at_utc"],
            "properties": {
                "approval_id": {"type": "string", "minLength": 1},
                "decision": {"enum": ["approved", "denied", "cancelled"]},
                "decided_by": {"type": "string", "minLength": 1},
                "decided_at_utc": {"type": "string"},
                "reason": string_or_null(),
                "event_id": string_or_null(),
            },
            "additionalProperties": True,
        },
        "EvidenceRecord": {
            "title": "envctl Migration Evidence Record",
            "type": "object",
            "required": ["evidence_id", "run_id", "uri", "evidence_kind", "sha256"],
            "properties": {
                "evidence_id": {"type": "string", "minLength": 1},
                "run_id": {"type": "string", "minLength": 1},
                "operation_id": string_or_null(),
                "uri": {"type": "string", "minLength": 1},
                "evidence_kind": {"type": "string", "minLength": 1},
                "sha256": {"type": "string", "pattern": "^[0-9a-f]{64}$"},
                "redacted": {"type": "boolean", "default": True},
                "metadata": {"type": "object"},
            },
            "additionalProperties": True,
        },
        "GraphEdge": {
            "title": "envctl Migration Graph Edge",
            "type": "object",
            "required": ["edge_id", "run_id", "from_node", "to_node", "edge_type"],
            "properties": {
                "edge_id": {"type": "string", "minLength": 1},
                "run_id": {"type": "string", "minLength": 1},
                "from_node": {"type": "string", "minLength": 1},
                "to_node": {"type": "string", "minLength": 1},
                "edge_type": {"type": "string", "minLength": 1},
                "source_artifact_id": string_or_null(),
            },
            "additionalProperties": True,
        },
        "ReplayRequest": {
            "title": "envctl Migration Replay Request",
            "type": "object",
            "required": ["replay_id", "run_id", "mode", "requested_by"],
            "properties": {
                "replay_id": {"type": "string", "minLength": 1},
                "run_id": {"type": "string", "minLength": 1},
                "mode": {"enum": ["dry_run", "apply"]},
                "requested_by": {"type": "string", "minLength": 1},
                "operation_ids": {"type": "array", "items": {"type": "string"}},
                "target_descriptor_id": string_or_null(),
                "reason": string_or_null(),
            },
            "additionalProperties": True,
        },
        "ReplayResult": {
            "title": "envctl Migration Replay Result",
            "type": "object",
            "required": ["replay_id", "run_id", "status", "completed_at_utc"],
            "properties": {
                "replay_id": {"type": "string", "minLength": 1},
                "run_id": {"type": "string", "minLength": 1},
                "status": {"enum": ["pass", "fail", "blocked", "partial"]},
                "completed_at_utc": {"type": "string"},
                "event_refs": {"type": "array", "items": {"type": "string"}},
                "artifact_refs": {"type": "array", "items": {"type": "string"}},
                "error": {"$ref": "#/$defs/StructuredError"},
            },
            "additionalProperties": True,
        },
        "StructuredError": {
            "title": "envctl Migration Structured Error",
            "type": "object",
            "required": ["code", "message"],
            "properties": {
                "code": {"type": "string", "minLength": 1},
                "message": {"type": "string", "minLength": 1},
                "retryable": {"type": "boolean"},
                "details": {"type": "object"},
            },
            "additionalProperties": True,
        },
    }


def build_shared_schema(base: Path) -> dict:
    defs = {
        name: imported_schema(base, name, relpath)
        for name, relpath in SOURCE_SCHEMAS.items()
    }
    defs.update(missing_record_defs())
    return {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://flexnetos.local/envctl/shared-protocol.schema.json",
        "title": "envctl / nu_plugin Shared Migration Protocol",
        "description": (
            "Versioned record contracts shared by envctl's migration database "
            "and the nu_plugin presentation/control surface."
        ),
        "oneOf": [{"$ref": f"#/$defs/{name}"} for name in REQUIRED_RECORDS],
        "$defs": defs,
    }


def build_manifest(schema: dict) -> dict:
    table_map = {
        "TargetDescriptor": "envctl_migration_targets",
        "MigrationRecipe": "envctl_migration_recipes",
        "MigrationRun": "envctl_migration_runs",
        "RunEvent": "envctl_migration_run_events",
        "Operation": "envctl_migration_operations",
        "ApprovalRequest": "envctl_migration_approvals",
        "ApprovalDecision": "envctl_migration_approvals",
        "ArtifactRecord": "envctl_migration_artifacts",
        "EvidenceRecord": "envctl_migration_evidence",
        "GraphEdge": "envctl_migration_graph_edges",
        "ValidationResult": "envctl_migration_validations",
        "ReplayRequest": "envctl_migration_runs",
        "ReplayResult": "envctl_migration_run_events",
        "ProofRecord": "execution_framework_proof_records",
    }
    plugin_shapes = {
        "RunEvent": "Nushell table row stream",
        "Operation": "Nushell operation table row and detail record",
        "ApprovalRequest": "Nushell approval table row",
        "ApprovalDecision": "Nushell decision record",
        "ArtifactRecord": "Nushell artifact table row and detail record",
        "ValidationResult": "Nushell validation table row",
        "TargetDescriptor": "Nushell target descriptor record",
        "MigrationRecipe": "Nushell recipe record",
        "MigrationRun": "Nushell run status record",
        "EvidenceRecord": "Nushell evidence table row",
        "GraphEdge": "Nushell graph edge table row",
        "ReplayRequest": "Nushell replay command input record",
        "ReplayResult": "Nushell replay result record",
        "ProofRecord": "Execution proof file record",
    }
    records = []
    for name in REQUIRED_RECORDS:
        schema_def = schema["$defs"][name]
        records.append(
            {
                "name": name,
                "schema_ref": f"schemas/shared_protocol.schema.json#/$defs/{name}",
                "source_schema": schema_def.get("x-shared-protocol-source", "generated"),
                "source_of_truth": table_map[name],
                "producer": "envctl",
                "consumer": "nu_plugin",
                "plugin_shape": plugin_shapes[name],
                "required": True,
            }
        )
    return {
        "schema_version": "1.0",
        "protocol_name": "envctl_nu_plugin_migration_protocol",
        "protocol_version": PROTOCOL_VERSION,
        "generated_at": now(),
        "source_of_truth_rule": "envctl migration database owns durable state; nu_plugin renders records and submits auditable commands.",
        "compatibility_rule": "Minor versions may add optional fields; removals, type narrowing, and enum removal require a new major protocol version.",
        "consumers": ["envctl", "nu_plugin"],
        "required_records": REQUIRED_RECORDS,
        "records": records,
        "generated_files": [
            "execution-framework/schemas/shared_protocol.schema.json",
            "execution-framework/generated/shared_protocol_manifest.json",
            "execution-framework/docs/SHARED_PROTOCOL_SCHEMAS.md",
        ],
    }


def sample_records() -> dict:
    hash_value = "0" * 64
    return {
        "TargetDescriptor": {
            "schema_version": 1,
            "target_id": "target-flexnetos",
            "target_type": "codebase",
            "primary_root": "/workspace/flexnetos",
            "safety": {
                "default_mode": "approval-gated",
                "max_auto_risk": "R2",
                "allow_network": True,
                "allow_destructive": False,
            },
            "artifact_contract": {"name": "full-migration-artifact-contract", "version": "1.0.0"},
            "recipe": {"name": "database-migration", "version": "1.0.0"},
        },
        "MigrationRecipe": {
            "schema_version": 1,
            "recipe_id": "recipe-shared-protocol",
            "version": "1.0.0",
            "phases": [
                {
                    "phase_id": "shared",
                    "operations": [
                        {
                            "operation_id": "op-shared-schema",
                            "operation_type": "generate_shared_protocol",
                            "risk": "R1",
                        }
                    ],
                }
            ],
        },
        "MigrationRun": {
            "schema_version": 1,
            "run_id": "run-001",
            "target_id": "target-flexnetos",
            "recipe_id": "recipe-shared-protocol",
            "artifact_contract_id": "full-migration-artifact-contract",
            "status": "running",
            "created_at_utc": "2026-07-04T23:00:00+00:00",
        },
        "RunEvent": {
            "run_id": "run-001",
            "event_seq": 1,
            "event_type": "operation.started",
            "actor_type": "agent",
            "actor_id": "shared-protocol-agent",
            "operation_id": "op-shared-schema",
            "timestamp_utc": "2026-07-04T23:00:00+00:00",
            "payload": {"status": "running"},
        },
        "Operation": {
            "operation_id": "op-shared-schema",
            "run_id": "run-001",
            "operation_type": "generate_shared_protocol",
            "status": "running",
            "risk": "R1",
        },
        "ApprovalRequest": {
            "approval_id": "approval-001",
            "run_id": "run-001",
            "operation_id": "op-shared-schema",
            "risk": "R3",
            "status": "open",
        },
        "ApprovalDecision": {
            "approval_id": "approval-001",
            "decision": "approved",
            "decided_by": "human",
            "decided_at_utc": "2026-07-04T23:01:00+00:00",
        },
        "ArtifactRecord": {
            "artifact_id": "artifact-shared-schema",
            "run_id": "run-001",
            "title": "Shared protocol schema",
            "status": "complete",
        },
        "EvidenceRecord": {
            "evidence_id": "evidence-001",
            "run_id": "run-001",
            "uri": "logs/REQ-040_SHARED_PROTOCOL_SCHEMAS.log",
            "evidence_kind": "verification-log",
            "sha256": hash_value,
            "redacted": True,
        },
        "GraphEdge": {
            "edge_id": "edge-001",
            "run_id": "run-001",
            "from_node": "Operation:op-shared-schema",
            "to_node": "ArtifactRecord:artifact-shared-schema",
            "edge_type": "produces",
        },
        "ValidationResult": {
            "validation_id": "validation-001",
            "run_id": "run-001",
            "validator": "shared-protocol-schema",
            "status": "pass",
        },
        "ReplayRequest": {
            "replay_id": "replay-001",
            "run_id": "run-001",
            "mode": "dry_run",
            "requested_by": "nu_plugin",
        },
        "ReplayResult": {
            "replay_id": "replay-001",
            "run_id": "run-001",
            "status": "pass",
            "completed_at_utc": "2026-07-04T23:02:00+00:00",
        },
        "ProofRecord": {
            "proof_schema_version": "1.0",
            "task_id": TASK_ID,
            "status": "completed",
            "started_at": "2026-07-04T23:00:00+00:00",
            "completed_at": "2026-07-04T23:02:00+00:00",
            "actor": "codex-cli-local",
            "helper_id": HELPER_ID,
            "model_tag": MODEL_TAG,
            "repo_path": "/workspace/package",
            "files_changed": [],
            "commands_run": [],
            "verification_output": {"status": "passed"},
            "checksums": {},
            "logs_uri": "logs/REQ-040_SHARED_PROTOCOL_SCHEMAS.log",
            "rollback_point": "history/pre_execution_framework_manifest.json",
            "evidence": [],
            "failure_reason": "",
            "next_action": "",
        },
    }


def validate_shared_protocol(schema: dict, manifest: dict) -> dict:
    errors = []
    defs = schema.get("$defs", {})
    for record in REQUIRED_RECORDS:
        if record not in defs:
            errors.append(f"missing $defs/{record}")
    manifest_records = {record["name"] for record in manifest.get("records", [])}
    for record in REQUIRED_RECORDS:
        if record not in manifest_records:
            errors.append(f"manifest missing {record}")

    Draft202012Validator.check_schema(schema)
    samples = sample_records()
    sample_results = {}
    for name in REQUIRED_RECORDS:
        validator = Draft202012Validator({"$ref": f"#/$defs/{name}", "$defs": defs})
        record_errors = sorted(validator.iter_errors(samples[name]), key=lambda err: err.path)
        if record_errors:
            errors.extend(f"{name}: {err.message}" for err in record_errors)
            sample_results[name] = "failed"
        else:
            sample_results[name] = "passed"

    source_schema_count = sum(
        1
        for name in REQUIRED_RECORDS
        if defs[name].get("x-shared-protocol-source", "generated") != "generated"
    )
    generated_schema_count = len(REQUIRED_RECORDS) - source_schema_count
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "passed" if not errors else "failed",
        "generated_at": manifest["generated_at"],
        "summary": {
            "required_record_count": len(REQUIRED_RECORDS),
            "manifest_record_count": len(manifest.get("records", [])),
            "source_schema_count": source_schema_count,
            "generated_schema_count": generated_schema_count,
            "sample_validation_count": len(sample_results),
        },
        "sample_results": sample_results,
        "errors": errors,
        "evidence": manifest["generated_files"],
    }


def render_docs(manifest: dict, validation: dict) -> str:
    lines = [
        "# Shared Protocol Schemas",
        "",
        f"Protocol: `{manifest['protocol_name']}`",
        f"Version: `{manifest['protocol_version']}`",
        f"Generated at: `{manifest['generated_at']}`",
        "",
        "## Source Of Truth",
        "",
        manifest["source_of_truth_rule"],
        "",
        "## Compatibility Rule",
        "",
        manifest["compatibility_rule"],
        "",
        "## Record Contracts",
        "",
        "| record | schema ref | source of truth | source schema | nu_plugin shape |",
        "|---|---|---|---|---|",
    ]
    for record in manifest["records"]:
        lines.append(
            "| `{name}` | `{schema_ref}` | `{source_of_truth}` | `{source_schema}` | {plugin_shape} |".format(
                **record
            )
        )
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
    lines.extend(["", "Sample record validation:", ""])
    for name, status in validation["sample_results"].items():
        lines.append(f"- `{name}`: `{status}`")
    if validation["errors"]:
        lines.extend(["", "Errors:", ""])
        lines.extend(f"- {error}" for error in validation["errors"])
    else:
        lines.extend(["", "No schema or sample-validation errors were found."])
    lines.append("")
    return "\n".join(lines)


def write_text(relpath: str, text: str) -> None:
    path = root() / relpath
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def main() -> int:
    base = package_root()
    started_at = now()
    schema = build_shared_schema(base)
    manifest = build_manifest(schema)
    validation = validate_shared_protocol(schema, manifest)

    write_json("schemas/shared_protocol.schema.json", schema)
    write_json("generated/shared_protocol_manifest.json", manifest)
    write_text("docs/SHARED_PROTOCOL_SCHEMAS.md", render_docs(manifest, validation))
    write_json("generated/shared_protocol_validation_report.json", validation)

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
        "execution-framework/scripts/verify_shared_protocol_schemas.py",
        "execution-framework/schemas/shared_protocol.schema.json",
        "execution-framework/generated/shared_protocol_manifest.json",
        "execution-framework/generated/shared_protocol_validation_report.json",
        "execution-framework/docs/SHARED_PROTOCOL_SCHEMAS.md",
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
            "python3 scripts/verify_shared_protocol_schemas.py",
        ],
        "verification_output": copy.deepcopy(validation),
        "checksums": file_checksums(files_changed[:-2]),
        "logs_uri": f"logs/{TASK_ID}.log",
        "rollback_point": "history/pre_execution_framework_manifest.json",
        "evidence": validation["evidence"] + [f"logs/{TASK_ID}.log"],
        "failure_reason": "" if validation["status"] == "passed" else "; ".join(validation["errors"]),
        "next_action": "run REQ-041_TWO_REPO_INTEGRATION against schemas/shared_protocol.schema.json",
    }
    append_proof(proof)
    return 0 if validation["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
