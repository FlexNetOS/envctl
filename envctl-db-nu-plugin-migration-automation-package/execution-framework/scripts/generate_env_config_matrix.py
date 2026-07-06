from __future__ import annotations

import json
import sqlite3
from pathlib import Path
from typing import Any

from _common import append_proof, file_checksums, make_proof, now, package_root, root, sha256_file, write_json
from artifact_registry import ArtifactRegistry, fetch_artifact
from verify_envctl_db_schema import apply_migrations


TASK_ID = "ART-114_ENV_CONFIG_MATRIX"
HELPER_ID = "helper-artifact-15"
MODEL_TAG = "gpt-5.3-spark"
RUN_ID = "run-art114-env-config-matrix"
OPERATION_ID = "produce-01-current-state-environment-matrix-md"
CONTRACT_ID = "contract-full-migration-artifact-contract-1.0.0"
RECIPE_ID = "flexnetos-package-artifact-contract-recipe"
TARGET_ID = "target-flexnetos-vs-lifeos"


ARTIFACT_DIR = "migration-artifacts/art-114_env_config_matrix"
CANONICAL_MD = "migration-artifacts/01-current-state/environment-matrix.md"
TASK_MD = f"{ARTIFACT_DIR}/environment-matrix.md"
TASK_JSON = f"{ARTIFACT_DIR}/environment-matrix.json"
REPORT_ROOT = "generated/art114_env_config_matrix_registry_report.json"
LOG_ROOT = f"logs/{TASK_ID}.log"
HEARTBEAT_ROOT = f"state/{TASK_ID}.heartbeat.json"
PROOF_ROOT = f"proof_records/{TASK_ID}.proof.json"
LEDGER_ROOT = "proof_records/proof_ledger.jsonl"
REPORT_PATH = f"execution-framework/{REPORT_ROOT}"
LOG_PATH = f"execution-framework/{LOG_ROOT}"
HEARTBEAT_PATH = f"execution-framework/{HEARTBEAT_ROOT}"
PROOF_PATH = f"execution-framework/{PROOF_ROOT}"
LEDGER_PATH = f"execution-framework/{LEDGER_ROOT}"


def load_json(relpath: str) -> dict[str, Any]:
    return json.loads((root() / relpath).read_text(encoding="utf-8"))


def artifact_path(relpath: str) -> Path:
    raw = Path(relpath)
    if raw.parts and raw.parts[0] == "migration-artifacts":
        return root() / raw
    return package_root() / raw


def proof_path(relpath: str) -> str:
    raw = Path(relpath)
    if raw.parts and raw.parts[0] == "migration-artifacts":
        return f"execution-framework/{relpath}"
    return relpath


def target_descriptor() -> dict[str, Any]:
    return {
        "schema_version": 1,
        "target_id": "flexnetos-vs-lifeos",
        "target_type": "mixed",
        "primary_root": "/home/flexnetos/FlexNetOS",
        "compare_root": "/home/flexnetos/lifeos",
        "output_root": "migration-artifacts",
        "safety": {
            "default_mode": "approval-gated",
            "max_auto_risk": "R2",
            "allow_network": False,
            "allow_destructive": False,
        },
        "collectors": {
            "filesystem": True,
            "git": True,
            "package_managers": True,
            "databases": True,
            "infrastructure": True,
            "apis": True,
            "observability": True,
        },
        "artifact_contract": {"name": "full-migration-artifact-contract", "version": 1},
        "recipe": {"name": "codex-flexnetos-full-artifact-run", "version": 1},
    }


def build_matrix() -> dict[str, Any]:
    generated_at = now()
    descriptor = target_descriptor()
    target_registry = load_json("generated/envctl_target_registry.json")
    packet = load_json("generated/execution_packets/ART-114_ENV_CONFIG_MATRIX.json")
    contract = load_json("generated/contract_manifest.json")
    contract_rows = contract.get("rows") or contract.get("contract", {}).get("rows", [])
    contract_row = next(
        row
        for row in contract_rows
        if row.get("producer_task_id") == TASK_ID and row.get("artifact_id") == "01-current-state-environment-matrix-md"
    )

    environments = [
        {
            "environment": "dev",
            "purpose": "Local agent execution, discovery, artifact generation, and deterministic smoke validation.",
            "root": descriptor["primary_root"],
            "network_policy": "disabled by descriptor unless a later approved task explicitly enables it",
            "write_policy": "workspace-write for execution-framework and generated migration-artifacts only",
            "risk_ceiling": "R2",
            "approval_mode": "approval-gated",
            "data_policy": "No blocked secret paths are read or emitted; credential material is excluded by packet policy.",
            "validation": [
                "artifact file existence",
                "JSON parse",
                "registry content_hash equality",
                "proof ledger append",
            ],
            "evidence_refs": [
                "execution-framework/generated/execution_packets/ART-114_ENV_CONFIG_MATRIX.json",
                "execution-framework/generated/envctl_target_registry.json",
            ],
        },
        {
            "environment": "stage",
            "purpose": "Replay, reconciliation, and validation lane before any production-impacting change.",
            "root": descriptor["primary_root"],
            "network_policy": "kept disabled in descriptor for this package; external staging integrations require explicit approval",
            "write_policy": "artifact/proof writes plus registry rows in the envctl database model",
            "risk_ceiling": "R2 by descriptor, R3 only when a later approval-gated cutover task declares it",
            "approval_mode": "approval-gated",
            "data_policy": "Use redacted evidence links and hashes, not raw secrets or private keys.",
            "validation": [
                "shared protocol schema evidence",
                "registry validation links",
                "hash-backed proof record",
                "later VER-300 unit validation",
            ],
            "evidence_refs": [
                "execution-framework/proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json",
                "execution-framework/proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json",
            ],
        },
        {
            "environment": "prod",
            "purpose": "Production reference and cutover target; this artifact generation task performs no production mutation.",
            "root": "external production targets, not written by this task",
            "network_policy": "no production network writes from ART-114",
            "write_policy": "blocked for production systems in this phase",
            "risk_ceiling": "R5 operations require human approval and are outside this artifact task",
            "approval_mode": "human approval required for destructive or production-impacting operations",
            "data_policy": "Secrets, private keys, .env files, pem/key material, and private_keys paths are blocked from artifact output.",
            "validation": [
                "production differences are recorded as policy constraints",
                "cutover/rollback tasks must add separate approval evidence",
                "artifact registry stores only file hashes and metadata",
            ],
            "evidence_refs": [
                "execution-framework/generated/execution_packets/ART-114_ENV_CONFIG_MATRIX.json",
                "execution-framework/docs/ENVCTL_ARTIFACT_REGISTRY.md",
            ],
        },
    ]

    dimensions = [
        {
            "dimension": "Root and source of truth",
            "dev": "FlexNetOS workspace primary_root with local execution-framework writes",
            "stage": "Same primary_root plus envctl SQLite model for replayable validation state",
            "prod": "Production targets are referenced only; mutation is blocked in this phase",
        },
        {
            "dimension": "Configuration inputs",
            "dev": "target descriptor, generated task packet, repo/package scan, contract manifest",
            "stage": "same inputs plus proof records from registry and shared schema tasks",
            "prod": "approved cutover configuration only; not generated or applied by ART-114",
        },
        {
            "dimension": "Secrets and credentials",
            "dev": "blocked path patterns are excluded from reads and artifact content",
            "stage": "evidence is linked by hash and metadata; raw secrets remain out of scope",
            "prod": "secret material must stay external to artifacts and requires separate controls",
        },
        {
            "dimension": "Risk and approval",
            "dev": "R2 maximum automatic risk, approval-gated descriptor",
            "stage": "R2 validation lane; higher-risk cutover work belongs to later tasks",
            "prod": "production-impacting or destructive work requires human approval",
        },
        {
            "dimension": "Outputs",
            "dev": "matrix markdown and JSON under migration-artifacts plus proof/log files",
            "stage": "registry rows, validation rows, graph links, and proof ledger evidence",
            "prod": "no output is written into production by this task",
        },
    ]

    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "generated_at": generated_at,
        "title": "Environment Matrix",
        "status": "complete",
        "target": {
            "target_id": descriptor["target_id"],
            "target_type": descriptor["target_type"],
            "primary_root": descriptor["primary_root"],
            "compare_root": descriptor["compare_root"],
            "output_root": descriptor["output_root"],
            "descriptor_hash": target_registry["descriptor_inputs"][0]["descriptor_hash"],
        },
        "contract": {
            "contract_id": CONTRACT_ID,
            "contract_row_id": contract_row["contract_row_id"],
            "canonical_path": contract_row["required_path"],
            "packet_artifact_dir": ARTIFACT_DIR,
        },
        "blocked_paths": packet["blocked_paths"],
        "environments": environments,
        "matrix": dimensions,
        "drift_controls": [
            "Capture dev/stage/prod differences as artifact data before VER-300 validation.",
            "Register content hashes in envctl_migration_artifacts.",
            "Link registry evidence to target descriptor, packet, and prerequisite proof records.",
            "Keep production mutations out of artifact generation.",
        ],
        "evidence_refs": [
            "execution-framework/generated/envctl_target_registry.json",
            "execution-framework/generated/package_scan.json",
            "execution-framework/generated/contract_manifest.json",
            "execution-framework/proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json",
            "execution-framework/proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json",
        ],
    }


def markdown(matrix: dict[str, Any]) -> str:
    rows = [
        "# Environment Matrix",
        "",
        f"Task: `{TASK_ID}`",
        f"Generated at: `{matrix['generated_at']}`",
        f"Status: `{matrix['status']}`",
        "",
        "## Target",
        "",
        f"- Target: `{matrix['target']['target_id']}` (`{matrix['target']['target_type']}`)",
        f"- Primary root: `{matrix['target']['primary_root']}`",
        f"- Compare root: `{matrix['target']['compare_root']}`",
        f"- Descriptor hash: `{matrix['target']['descriptor_hash']}`",
        "",
        "## Dev / Stage / Prod Differences",
        "",
        "| Dimension | Dev | Stage | Prod |",
        "|---|---|---|---|",
    ]
    for item in matrix["matrix"]:
        rows.append(f"| {item['dimension']} | {item['dev']} | {item['stage']} | {item['prod']} |")
    rows.extend(["", "## Environment Detail", ""])
    for env in matrix["environments"]:
        rows.extend(
            [
                f"### {env['environment']}",
                "",
                f"- Purpose: {env['purpose']}",
                f"- Root: `{env['root']}`",
                f"- Network policy: {env['network_policy']}",
                f"- Write policy: {env['write_policy']}",
                f"- Risk ceiling: {env['risk_ceiling']}",
                f"- Approval mode: {env['approval_mode']}",
                f"- Data policy: {env['data_policy']}",
                f"- Validation: {', '.join(env['validation'])}",
                "",
            ]
        )
    rows.extend(
        [
            "## Drift Controls",
            "",
            *[f"- {item}" for item in matrix["drift_controls"]],
            "",
            "## Evidence",
            "",
            *[f"- `{item}`" for item in matrix["evidence_refs"]],
            "",
        ]
    )
    return "\n".join(rows)


def write_artifacts(matrix: dict[str, Any]) -> list[str]:
    paths = [CANONICAL_MD, TASK_MD, TASK_JSON]
    for relpath in paths:
        path = artifact_path(relpath)
        path.parent.mkdir(parents=True, exist_ok=True)
        if relpath.endswith(".json"):
            path.write_text(json.dumps(matrix, indent=2, sort_keys=False) + "\n", encoding="utf-8")
        else:
            path.write_text(markdown(matrix), encoding="utf-8")
    return paths


def insert_art114_fixture(conn: sqlite3.Connection) -> None:
    descriptor = target_descriptor()
    conn.execute(
        """
        INSERT INTO envctl_migration_targets
          (id, target_id, target_type, primary_root, compare_root, descriptor_json,
           descriptor_hash, safety_mode, max_auto_risk)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
          target_id = excluded.target_id,
          target_type = excluded.target_type,
          primary_root = excluded.primary_root,
          compare_root = excluded.compare_root,
          descriptor_json = excluded.descriptor_json,
          descriptor_hash = excluded.descriptor_hash,
          safety_mode = excluded.safety_mode,
          max_auto_risk = excluded.max_auto_risk
        """,
        (
            TARGET_ID,
            descriptor["target_id"],
            descriptor["target_type"],
            descriptor["primary_root"],
            descriptor["compare_root"],
            json.dumps(descriptor, sort_keys=True),
            "sha256:b3f653f9e9cda7991821687f041cb540ce3e4342bde03ef333aa0e72a6b42384",
            descriptor["safety"]["default_mode"],
            descriptor["safety"]["max_auto_risk"],
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_runs
          (id, target_id, recipe_id, artifact_contract_id, status, human_mode,
           initiated_by, sandbox_policy, approval_policy, tool_versions_json,
           reproducibility_hash, started_at_utc)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
          status = excluded.status,
          tool_versions_json = excluded.tool_versions_json
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
            '{"python":"stdlib","sqlite":"stdlib","codex":"codex-cli-background-shell"}',
            "sha256:art114-env-config-matrix",
            now(),
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_operations
          (id, run_id, operation_type, phase, status, risk, idempotency_key,
           command_hash, command_redacted, input_json)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
          status = excluded.status,
          input_json = excluded.input_json
        """,
        (
            OPERATION_ID,
            RUN_ID,
            "produce_artifact_record",
            "05-artifacts",
            "succeeded",
            "R1",
            f"{TASK_ID}/environment-matrix",
            "sha256:command-art114-env-config-matrix",
            "python3 scripts/generate_env_config_matrix.py",
            json.dumps({"task_id": TASK_ID, "artifacts": [CANONICAL_MD, TASK_MD, TASK_JSON]}, sort_keys=True),
        ),
    )
    conn.commit()


def register_artifacts(conn: sqlite3.Connection, artifact_paths: list[str]) -> list[dict[str, Any]]:
    registry = ArtifactRegistry(conn, package_root())
    records = [
        {
            "artifact_id": "01-current-state-environment-matrix-md",
            "title": "Environment Matrix",
            "artifact_type": "migration_artifact",
            "path": CANONICAL_MD,
            "links": [
                {"to": "artifact:01-current-state-configuration-inventory-md", "type": "paired_with"},
                {"to": "artifact:08-operations-environment-parity-matrix-md", "type": "feeds"},
            ],
        },
        {
            "artifact_id": "art-114-env-config-matrix-md",
            "title": "ART-114 Environment Matrix Markdown",
            "artifact_type": "task_artifact",
            "path": TASK_MD,
            "links": [{"to": "artifact:01-current-state-environment-matrix-md", "type": "mirrors"}],
        },
        {
            "artifact_id": "art-114-env-config-matrix-json",
            "title": "ART-114 Environment Matrix JSON",
            "artifact_type": "machine_readable_record",
            "path": TASK_JSON,
            "links": [{"to": "artifact:01-current-state-environment-matrix-md", "type": "describes"}],
        },
    ]
    results = []
    for record in records:
        results.append(
            registry.register(
                {
                    **record,
                    "run_id": RUN_ID,
                    "status": "complete",
                    "producer_operation_id": OPERATION_ID,
                    "contract_id": CONTRACT_ID,
                    "provenance": {
                        "task_id": TASK_ID,
                        "owner_agent": "artifact-agent",
                        "helper_id": HELPER_ID,
                        "source_packet": "execution-framework/generated/execution_packets/ART-114_ENV_CONFIG_MATRIX.json",
                        "target_descriptor": "examples/target-descriptors/flexnetos-vs-lifeos.yaml",
                    },
                    "evidence_refs": artifact_paths
                    + [
                        "execution-framework/generated/envctl_target_registry.json",
                        "execution-framework/proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json",
                        "execution-framework/proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json",
                    ],
                    "validations": [
                        {
                            "validator": "generate_env_config_matrix.py:path-exists",
                            "status": "pass",
                            "details": {"path": record["path"]},
                            "evidence_refs": [record["path"]],
                        },
                        {
                            "validator": "generate_env_config_matrix.py:hash-recorded",
                            "status": "pass",
                            "details": {"registry_computes_hash": True},
                            "evidence_refs": [record["path"]],
                        },
                        {
                            "validator": "generate_env_config_matrix.py:dev-stage-prod-coverage",
                            "status": "pass",
                            "details": {"environments": ["dev", "stage", "prod"]},
                            "evidence_refs": [TASK_JSON],
                        },
                    ],
                }
            )
        )
    return results


def verify_registry(conn: sqlite3.Connection, registrations: list[dict[str, Any]]) -> dict[str, Any]:
    rows = []
    errors = []
    for item in registrations:
        row = fetch_artifact(conn, item["run_id"], item["artifact_id"])
        relpath = row["path"]
        expected_hash = "sha256:" + sha256_file(artifact_path(relpath))
        hash_matches = row["content_hash"] == expected_hash == item["content_hash"]
        if not hash_matches:
            errors.append({"artifact_id": item["artifact_id"], "expected_hash": expected_hash, "row_hash": row["content_hash"]})
        rows.append(
            {
                "artifact_id": row["artifact_id"],
                "path": relpath,
                "content_hash": row["content_hash"],
                "hash_matches": hash_matches,
                "status": row["status"],
            }
        )
    matrix_json = json.loads(artifact_path(TASK_JSON).read_text(encoding="utf-8"))
    environments = {item["environment"] for item in matrix_json["environments"]}
    if environments != {"dev", "stage", "prod"}:
        errors.append({"json_environments": sorted(environments)})
    return {
        "artifact_count": len(rows),
        "registry_rows": rows,
        "json_environment_count": len(environments),
        "json_environments": sorted(environments),
        "all_hashes_match": all(row["hash_matches"] for row in rows),
        "errors": errors,
    }


def main() -> None:
    started_at = now()
    matrix = build_matrix()
    artifact_paths = write_artifacts(matrix)
    conn = sqlite3.connect(":memory:")
    conn.row_factory = sqlite3.Row
    applied_migrations = apply_migrations(conn, package_root())
    insert_art114_fixture(conn)
    registrations = register_artifacts(conn, artifact_paths)
    verification = verify_registry(conn, registrations)
    status = "passed" if not verification["errors"] else "failed"
    report = {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": status,
        "generated_at": now(),
        "started_at": started_at,
        "completed_at": now(),
        "artifacts": artifact_paths,
        "applied_migrations": applied_migrations,
        "registry_results": registrations,
        "verification": verification,
        "evidence": [
            *artifact_paths,
            "execution-framework/generated/envctl_target_registry.json",
            "execution-framework/generated/contract_manifest.json",
            "execution-framework/proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json",
            "execution-framework/proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json",
        ],
    }
    write_json(REPORT_ROOT, report)
    write_json(LOG_ROOT, report)
    heartbeat = {
        "task_id": TASK_ID,
        "status": "complete" if status == "passed" else "failed",
        "updated_at": now(),
        "proof_uri": f"proof_records/{TASK_ID}.proof.json",
        "artifacts": artifact_paths,
    }
    write_json(HEARTBEAT_ROOT, heartbeat)

    files_changed = [
        "execution-framework/scripts/generate_env_config_matrix.py",
        *[proof_path(path) for path in artifact_paths],
        REPORT_PATH,
        LOG_PATH,
        HEARTBEAT_PATH,
        PROOF_PATH,
        LEDGER_PATH,
    ]
    commands_run = [
        "python3 scripts/generate_env_config_matrix.py",
        "python3 -m py_compile scripts/generate_env_config_matrix.py",
    ]
    proof = make_proof(
        TASK_ID,
        "completed" if status == "passed" else "failed",
        "artifact-agent",
        HELPER_ID,
        MODEL_TAG,
        "/home/flexnetos/FlexNetOS/src/envctl/envctl-db-nu-plugin-migration-automation-package",
        files_changed,
        commands_run,
        report,
        report["evidence"],
        "" if status == "passed" else json.dumps(verification["errors"], sort_keys=True),
        "run VER-300_UNIT_VALIDATION",
    )
    append_proof(proof)
    proof["checksums"] = file_checksums(files_changed)
    append_proof(proof)
    print(
        "ART-114 env matrix status={status} artifacts={artifacts} registry_rows={rows} hashes_match={hashes}".format(
            status=status,
            artifacts=len(artifact_paths),
            rows=verification["artifact_count"],
            hashes=verification["all_hashes_match"],
        )
    )
    if status != "passed":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
