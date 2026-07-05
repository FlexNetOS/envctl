from __future__ import annotations

import json
import re
from pathlib import Path

from _common import (
    append_proof,
    file_checksums,
    load_ledger,
    make_proof,
    now,
    package_root,
    read_json,
    read_task_graph,
    sha256_file,
    split_list,
    write_json,
)


CONTRACT_NAME = "full-migration-artifact-contract"
CONTRACT_VERSION = "1.0.0"
SOURCE_PACKAGE_NAME = "codex-flexnetos-migration-prompt-package"

TREE_PATH = Path(
    "source/codex-flexnetos-migration-prompt-package/expected-output/migration-artifacts-tree.md"
)
SOURCE_CONTEXT = Path("source/previous-migration-artifact-context.md")
AUTOMATION_ARTIFACTS = Path("expected-output/migration-automation-artifacts.md")
UTILIZATION_PROMPT = Path("prompts/UTILIZE_FLEXNETOS_PACKAGE.md")
STRATEGY_PROMPT = Path("prompts/STRATEGY_DECISION.md")
MASTER_PROMPT = Path("prompts/MASTER_PROMPT_ENVCTL_DB_NU_PLUGIN.md")

REQUIRED_SOURCE_FILES = [
    "README.md",
    str(SOURCE_CONTEXT),
    str(TREE_PATH),
    str(AUTOMATION_ARTIFACTS),
    str(UTILIZATION_PROMPT),
    str(STRATEGY_PROMPT),
    str(MASTER_PROMPT),
    "sql/003_seed_artifact_contract.sql",
]

KEY_REQUIRED_PATHS = {
    "migration-artifacts/MIGRATION_MEMORY.md",
    "migration-artifacts/index.md",
    "migration-artifacts/wiki-home.md",
    "migration-artifacts/artifact-manifest.json",
    "migration-artifacts/evidence-register.md",
    "migration-artifacts/link-graph.md",
    "migration-artifacts/01-current-state/system-inventory.md",
    "migration-artifacts/01-current-state/dependency-graph.md",
    "migration-artifacts/01-current-state/data-flow-current.md",
    "migration-artifacts/03-code-analysis/code-map-for-debugging.md",
    "migration-artifacts/03-code-analysis/toolchain-dependency-tree.md",
    "migration-artifacts/04-data-migration/database-schema-map.md",
    "migration-artifacts/04-data-migration/data-lineage-map.md",
    "migration-artifacts/05-integrations/api-contract-catalog.md",
    "migration-artifacts/05-integrations/event-message-contract-map.md",
    "migration-artifacts/06-testing-validation/validation-evidence.md",
    "migration-artifacts/06-testing-validation/validation-reconciliation-reports.md",
    "migration-artifacts/07-cutover/cutover-checklist.md",
    "migration-artifacts/07-cutover/rollback-plan.md",
    "migration-artifacts/09-governance/risk-register.md",
    "migration-artifacts/09-governance/decision-log.md",
}

TASK_LINK_HINTS = {
    "system-inventory": "ART-100_SYSTEM_INVENTORY",
    "directory-tree": "ART-101_DIRECTORY_TREE",
    "repository-map": "ART-102_REPOSITORY_MAP",
    "repo-map": "ART-102_REPOSITORY_MAP",
    "dependency-graph": "ART-103_SERVICE_DEP_GRAPH",
    "application-service-dependency-graph": "ART-103_SERVICE_DEP_GRAPH",
    "toolchain-dependency-tree": "ART-104_TOOLCHAIN_TREE",
    "package-library-dependency-graph": "ART-105_PACKAGE_LIB_GRAPH",
    "package-dependencies": "ART-105_PACKAGE_LIB_GRAPH",
    "runtime-dependency-map": "ART-106_RUNTIME_DEP_MAP",
    "data-flow": "ART-107_DATA_FLOW_GRAPH",
    "database-schema-map": "ART-108_DB_SCHEMA_MAP",
    "schema-map": "ART-108_DB_SCHEMA_MAP",
    "data-lineage-map": "ART-109_DATA_LINEAGE",
    "api-contract": "ART-110_API_CATALOG",
    "api-catalog": "ART-110_API_CATALOG",
    "event-message-contract-map": "ART-111_EVENT_MAP",
    "event-catalog": "ART-111_EVENT_MAP",
    "code-ownership-map": "ART-112_CODE_OWNERSHIP",
    "ownership-matrix": "ART-112_CODE_OWNERSHIP",
    "code-map-for-debugging": "ART-113_DEBUG_CODE_MAP",
    "environment-matrix": "ART-114_ENV_CONFIG_MATRIX",
    "configuration-inventory": "ART-115_CONFIG_INVENTORY",
    "infrastructure-topology-map": "ART-116_INFRA_TOPOLOGY",
    "iam-security-access-matrix": "ART-117_IAM_MATRIX",
    "observability-map": "ART-118_OBSERVABILITY",
    "business-process-map": "ART-119_BUSINESS_PROCESS",
    "migration-wave-plan": "ART-120_WAVE_PLAN",
    "wave-plan": "ART-120_WAVE_PLAN",
    "cutover-checklist": "ART-121_CUTOVER",
    "rollback-plan": "ART-122_ROLLBACK",
    "validation-reconciliation": "ART-123_VALIDATION_RECONCILIATION",
    "test-coverage-matrix": "ART-124_TEST_COVERAGE",
    "risk-register": "ART-125_RISK_REGISTER",
    "decision-log": "ART-126_DECISION_LOG",
    "blast-radius": "ART-127_BLAST_RADIUS",
    "readiness-scorecard": "ART-128_READINESS_SCORECARD",
    "business-capability": "ART-129_BUSINESS_CAPABILITY",
    "shadow-traffic": "ART-130_SHADOW_TRAFFIC",
    "golden-dataset": "ART-131_GOLDEN_DATASET",
    "parity-dashboard": "ART-132_PARITY_DASHBOARD",
    "deprecation-map": "ART-133_DEPRECATION_MAP",
    "exception-inventory": "ART-134_EXCEPTION_INVENTORY",
    "raci": "ART-135_RACI",
    "technical-debt-ledger": "ART-136_TECH_DEBT_LEDGER",
}


def slug(value: str) -> str:
    return re.sub(r"[^a-z0-9]+", "-", value.lower()).strip("-")


def title_from_path(path: str) -> str:
    stem = Path(path).stem
    return stem.replace("-", " ").replace("_", " ").title()


def extract_tree_block(text: str) -> list[str]:
    lines = text.splitlines()
    in_block = False
    block: list[str] = []
    for line in lines:
        if line.strip() == "```text":
            in_block = True
            continue
        if in_block and line.strip() == "```":
            break
        if in_block:
            block.append(line.rstrip())
    return block


def parse_artifact_tree(text: str) -> list[dict]:
    stack: list[str] = []
    rows: list[dict] = []
    for raw in extract_tree_block(text):
        if not raw.strip():
            continue
        if raw.strip() == "migration-artifacts/":
            stack = ["migration-artifacts"]
            continue
        match = re.match(r"^(?P<prefix>[ \u2502]*)(?:\u251c\u2500\u2500|\u2514\u2500\u2500) (?P<name>.+)$", raw)
        if not match or not stack:
            continue
        prefix = match.group("prefix")
        name = match.group("name").strip()
        depth = (len(prefix) // 4) + 1
        if name.endswith("/"):
            stack = stack[:depth] + [name.rstrip("/")]
            continue
        path_parts = stack[:depth] + [name]
        artifact_path = "/".join(path_parts)
        artifact_id = slug(artifact_path.removeprefix("migration-artifacts/"))
        row = {
            "contract_row_id": f"artifact:{artifact_id}",
            "artifact_id": artifact_id,
            "title": title_from_path(artifact_path),
            "required_path": artifact_path,
            "section": path_parts[1] if len(path_parts) > 2 else "_root",
            "artifact_type": infer_artifact_type(artifact_path),
            "format": infer_format(artifact_path),
            "status": "required",
            "source_refs": [str(TREE_PATH), str(SOURCE_CONTEXT)],
            "producer_task_id": infer_task_id(artifact_path),
            "validators": ["path_registered", "evidence_or_status_recorded", "linked_from_index"],
        }
        rows.append(row)
    return rows


def infer_format(path: str) -> str:
    suffix = Path(path).suffix.lower().lstrip(".")
    return suffix or "directory"


def infer_artifact_type(path: str) -> str:
    if "/_spark/" in path:
        return "spark_helper_output"
    if "/_meta/" in path:
        return "run_metadata"
    if path.endswith(".json") or path.endswith(".jsonl") or path.endswith(".tsv"):
        return "machine_readable_record"
    if "/07-cutover/" in path:
        return "migration_control"
    if "/06-testing-validation/" in path:
        return "validation_evidence"
    if "/09-governance/" in path:
        return "governance_record"
    return "migration_artifact"


def infer_task_id(path: str) -> str | None:
    normalized = slug(Path(path).stem)
    for hint, task_id in TASK_LINK_HINTS.items():
        if hint in normalized:
            return task_id
    return None


def parse_automation_artifacts(text: str) -> list[dict]:
    rows: list[dict] = []
    for line in text.splitlines():
        stripped = line.strip()
        if not stripped.startswith("- "):
            continue
        title = stripped[2:].rstrip(".")
        artifact_id = f"automation-{slug(title)}"
        rows.append(
            {
                "contract_row_id": f"automation:{artifact_id}",
                "artifact_id": artifact_id,
                "title": title,
                "required_path": None,
                "section": "automation",
                "artifact_type": "automation_record",
                "format": "database",
                "status": "required",
                "source_refs": [str(AUTOMATION_ARTIFACTS)],
                "producer_task_id": None,
                "validators": ["database_record_created", "event_or_evidence_linked"],
            }
        )
    return rows


def source_hashes(base: Path) -> dict:
    hashes = {}
    for rel in REQUIRED_SOURCE_FILES:
        path = base / rel
        hashes[rel] = sha256_file(path) if path.exists() and path.is_file() else None
    return hashes


def contract_hash(payload: dict) -> str:
    encoded = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    import hashlib

    return hashlib.sha256(encoded).hexdigest()


def load_package_manifest(base: Path) -> dict:
    manifest_path = base / "source" / SOURCE_PACKAGE_NAME / "PACKAGE_MANIFEST.json"
    return json.loads(manifest_path.read_text(encoding="utf-8"))


def build_recipe(rows: list[dict]) -> dict:
    phase_order = [
        "_root",
        "_meta",
        "_spark",
        "00-executive-summary",
        "01-current-state",
        "02-target-state",
        "03-code-analysis",
        "04-data-migration",
        "05-integrations",
        "06-testing-validation",
        "07-cutover",
        "08-operations",
        "09-governance",
        "automation",
    ]
    phases = []
    by_phase = {phase: [] for phase in phase_order}
    for row in rows:
        by_phase.setdefault(row["section"], []).append(row)
    for phase in phase_order:
        phase_rows = by_phase.get(phase, [])
        if not phase_rows:
            continue
        phases.append(
            {
                "phase_id": phase,
                "depends_on": [] if phase in {"_root", "_meta"} else ["_root"],
                "approval_gate": phase in {"07-cutover"},
                "operations": [
                    {
                        "operation_id": f"produce-{row['artifact_id']}",
                        "operation_type": "produce_artifact_record",
                        "risk": "R1" if phase != "07-cutover" else "R3",
                        "inputs": {"contract_row_id": row["contract_row_id"]},
                        "expected_artifacts": [row["artifact_id"]],
                        "validators": row["validators"],
                        "rollback": {
                            "mode": "remove_generated_artifact_only",
                            "requires_human_approval": False,
                        },
                    }
                    for row in phase_rows
                ],
            }
        )
    return {
        "schema_version": 1,
        "recipe_id": "flexnetos-package-artifact-contract-recipe",
        "version": CONTRACT_VERSION,
        "metadata": {
            "source_package": SOURCE_PACKAGE_NAME,
            "contract_name": CONTRACT_NAME,
            "upgrade_only": True,
        },
        "phases": phases,
    }


def write_markdown(manifest: dict, validation: dict) -> None:
    base = package_root()
    docs = base / "execution-framework" / "docs"
    docs.mkdir(parents=True, exist_ok=True)
    lines = [
        "# Contract Manifest",
        "",
        f"Contract: `{manifest['contract']['name']}`",
        f"Version: `{manifest['contract']['version']}`",
        f"Hash: `{manifest['contract']['contract_hash']}`",
        f"Source package: `{manifest['source_package']['name']}`",
        "",
        "## Summary",
        "",
        f"- Required artifact rows: {manifest['summary']['artifact_row_count']}",
        f"- Automation rows: {manifest['summary']['automation_row_count']}",
        f"- Recipe phases: {manifest['summary']['recipe_phase_count']}",
        f"- Validation status: {validation['status']}",
        "",
        "## Required Sources",
        "",
    ]
    for rel, digest in manifest["source_hashes"].items():
        lines.append(f"- `{rel}`: `{digest or 'missing'}`")
    lines += [
        "",
        "## Contract Rows",
        "",
        "| row | artifact | path | task |",
        "|---|---|---|---|",
    ]
    for row in manifest["contract"]["rows"]:
        path = row["required_path"] or "(database record)"
        task = row["producer_task_id"] or ""
        lines.append(f"| `{row['contract_row_id']}` | {row['title']} | `{path}` | `{task}` |")
    (docs / "CONTRACT_MANIFEST.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


def write_sql_seed(manifest: dict) -> None:
    base = package_root()
    out = base / "execution-framework" / "generated" / "contract_manifest.seed.sql"
    contract_json = json.dumps(manifest["contract"], sort_keys=True, separators=(",", ":")).replace("'", "''")
    recipe_json = json.dumps(manifest["recipe"], sort_keys=True, separators=(",", ":")).replace("'", "''")
    package_hash = manifest["source_package"]["package_manifest_hash"]
    contract_hash_value = manifest["contract"]["contract_hash"]
    recipe_hash_value = manifest["recipe_hash"]
    sql = f"""-- Generated by REQ-010_CONTRACT_LOCK.
-- Concrete seed rows for the first package fixture; adapt IDs to repo-native migrations if needed.

INSERT INTO envctl_migration_packages (id, package_name, package_path, package_hash, manifest_json)
VALUES ('pkg-{SOURCE_PACKAGE_NAME}', '{SOURCE_PACKAGE_NAME}', 'source/{SOURCE_PACKAGE_NAME}', '{package_hash}', '{{}}')
ON CONFLICT(package_name, package_hash) DO NOTHING;

INSERT INTO envctl_migration_artifact_contracts
  (id, contract_name, contract_version, source_package_id, contract_hash, contract_json)
VALUES
  ('contract-{slug(CONTRACT_NAME)}-{CONTRACT_VERSION}', '{CONTRACT_NAME}', '{CONTRACT_VERSION}', 'pkg-{SOURCE_PACKAGE_NAME}', '{contract_hash_value}', '{contract_json}')
ON CONFLICT(contract_name, contract_version) DO NOTHING;

INSERT INTO envctl_migration_recipes
  (id, recipe_name, recipe_version, artifact_contract_id, recipe_hash, recipe_json)
VALUES
  ('recipe-flexnetos-package-artifact-contract-{CONTRACT_VERSION}', 'flexnetos-package-artifact-contract-recipe', '{CONTRACT_VERSION}', 'contract-{slug(CONTRACT_NAME)}-{CONTRACT_VERSION}', '{recipe_hash_value}', '{recipe_json}')
ON CONFLICT(recipe_name, recipe_version) DO NOTHING;
"""
    out.write_text(sql, encoding="utf-8")


def validate_manifest(manifest: dict) -> dict:
    paths = {row["required_path"] for row in manifest["contract"]["rows"] if row["required_path"]}
    missing_sources = [rel for rel, digest in manifest["source_hashes"].items() if digest is None]
    missing_key_paths = sorted(KEY_REQUIRED_PATHS - paths)
    duplicate_rows = sorted(
        row_id
        for row_id in {row["contract_row_id"] for row in manifest["contract"]["rows"]}
        if sum(1 for row in manifest["contract"]["rows"] if row["contract_row_id"] == row_id) > 1
    )
    errors = []
    if missing_sources:
        errors.append({"kind": "missing_source_files", "items": missing_sources})
    if missing_key_paths:
        errors.append({"kind": "missing_key_contract_paths", "items": missing_key_paths})
    if duplicate_rows:
        errors.append({"kind": "duplicate_contract_rows", "items": duplicate_rows})
    if manifest["summary"]["artifact_row_count"] < 90:
        errors.append({"kind": "artifact_row_count_too_low", "count": manifest["summary"]["artifact_row_count"]})
    if manifest["summary"]["automation_row_count"] < 10:
        errors.append({"kind": "automation_row_count_too_low", "count": manifest["summary"]["automation_row_count"]})
    return {
        "schema_version": "1.0",
        "generated_at": now(),
        "status": "passed" if not errors else "failed",
        "errors": errors,
        "checks": {
            "source_file_count": len(manifest["source_hashes"]),
            "artifact_row_count": manifest["summary"]["artifact_row_count"],
            "automation_row_count": manifest["summary"]["automation_row_count"],
            "key_path_count": len(KEY_REQUIRED_PATHS),
            "recipe_phase_count": manifest["summary"]["recipe_phase_count"],
        },
    }


def update_task_index(manifest: dict, validation: dict) -> None:
    index_path = "generated/task_graph.index.json"
    index = read_json(index_path)
    index["contract_lock"] = {
        "task_id": "REQ-010_CONTRACT_LOCK",
        "status": validation["status"],
        "contract_name": manifest["contract"]["name"],
        "contract_version": manifest["contract"]["version"],
        "contract_hash": manifest["contract"]["contract_hash"],
        "manifest_uri": "generated/contract_manifest.json",
        "validation_uri": "generated/contract_manifest.validation_report.json",
        "docs_uri": "docs/CONTRACT_MANIFEST.md",
        "seed_sql_uri": "generated/contract_manifest.seed.sql",
        "artifact_row_count": manifest["summary"]["artifact_row_count"],
        "automation_row_count": manifest["summary"]["automation_row_count"],
        "unblocks": [
            "REQ-020_ENVCTL_DB_SCHEMA",
            "REQ-030_PLUGIN_PROTOCOL_MANIFEST",
            "REQ-040_SHARED_PROTOCOL_SCHEMAS",
            "ART-100_SYSTEM_INVENTORY",
        ],
    }
    write_json(index_path, index)


def main() -> None:
    base = package_root()
    tree_text = (base / TREE_PATH).read_text(encoding="utf-8")
    automation_text = (base / AUTOMATION_ARTIFACTS).read_text(encoding="utf-8")
    rows = parse_artifact_tree(tree_text)
    automation_rows = parse_automation_artifacts(automation_text)
    all_rows = rows + automation_rows
    recipe = build_recipe(all_rows)
    source_package_manifest = load_package_manifest(base)
    source_package_manifest_hash = sha256_file(base / "source" / SOURCE_PACKAGE_NAME / "PACKAGE_MANIFEST.json")
    source_file_hashes = source_hashes(base)
    payload = {
        "name": CONTRACT_NAME,
        "version": CONTRACT_VERSION,
        "source_package": SOURCE_PACKAGE_NAME,
        "upgrade_only": True,
        "rows": all_rows,
    }
    digest = contract_hash(
        {
            "contract": payload,
            "source_hashes": source_file_hashes,
            "source_package_manifest_hash": source_package_manifest_hash,
        }
    )
    payload["contract_hash"] = digest
    recipe_hash = contract_hash(recipe)
    manifest = {
        "schema_version": "1.0",
        "generated_at": now(),
        "contract": payload,
        "source_package": {
            "name": SOURCE_PACKAGE_NAME,
            "package_manifest_hash": source_package_manifest_hash,
            "file_count": source_package_manifest.get("file_count"),
            "package_root": f"source/{SOURCE_PACKAGE_NAME}",
        },
        "source_hashes": source_file_hashes,
        "recipe": recipe,
        "recipe_hash": recipe_hash,
        "summary": {
            "artifact_row_count": len(rows),
            "automation_row_count": len(automation_rows),
            "total_contract_row_count": len(all_rows),
            "recipe_phase_count": len(recipe["phases"]),
            "linked_task_count": len({row["producer_task_id"] for row in all_rows if row["producer_task_id"]}),
        },
    }
    validation = validate_manifest(manifest)
    write_json("generated/contract_manifest.json", manifest)
    write_json("generated/contract_manifest.validation_report.json", validation)
    write_markdown(manifest, validation)
    write_sql_seed(manifest)
    update_task_index(manifest, validation)
    changed = [
        "execution-framework/scripts/lock_contract.py",
        "execution-framework/generated/contract_manifest.json",
        "execution-framework/generated/contract_manifest.validation_report.json",
        "execution-framework/generated/contract_manifest.seed.sql",
        "execution-framework/generated/task_graph.index.json",
        "execution-framework/docs/CONTRACT_MANIFEST.md",
        "execution-framework/logs/REQ-010_CONTRACT_LOCK.log",
        "execution-framework/proof_records/REQ-010_CONTRACT_LOCK.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    proof = make_proof(
        "REQ-010_CONTRACT_LOCK",
        "completed" if validation["status"] == "passed" else "failed",
        "codex-cli-local",
        "helper-plan-01",
        "gpt-5.3-spark",
        str(base),
        changed,
        [
            "python3 scripts/validate_task_graph.py generated/task_graph.csv",
            "python3 scripts/task_graph_to_packets.py generated/task_graph.csv",
            "python3 scripts/goal_loop.py generated/task_graph.csv",
            "python3 scripts/verify_history_and_completeness.py",
            "python3 scripts/lock_contract.py",
        ],
        {
            "contract_manifest": "generated/contract_manifest.json",
            "validation": validation,
            "summary": manifest["summary"],
        },
        [
            "generated/contract_manifest.json",
            "generated/contract_manifest.validation_report.json",
            "generated/contract_manifest.seed.sql",
            "docs/CONTRACT_MANIFEST.md",
            "generated/task_graph.index.json#contract_lock",
        ],
        "" if validation["status"] == "passed" else "see contract_manifest.validation_report.json",
        "run REQ-020_ENVCTL_DB_SCHEMA, REQ-030_PLUGIN_PROTOCOL_MANIFEST, and REQ-040_SHARED_PROTOCOL_SCHEMAS",
    )
    append_proof(proof)
    log_path = base / "execution-framework" / "logs" / "REQ-010_CONTRACT_LOCK.log"
    log_path.write_text(
        "\n".join(
            [
                f"generated_at={now()}",
                f"contract_hash={digest}",
                f"artifact_rows={len(rows)}",
                f"automation_rows={len(automation_rows)}",
                f"validation_status={validation['status']}",
            ]
        )
        + "\n",
        encoding="utf-8",
    )
    print(
        "contract lock status="
        f"{validation['status']} artifact_rows={len(rows)} automation_rows={len(automation_rows)} hash={digest}"
    )
    if validation["status"] != "passed":
        raise SystemExit(1)


if __name__ == "__main__":
    main()
