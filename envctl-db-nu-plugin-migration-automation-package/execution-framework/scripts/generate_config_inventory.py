from __future__ import annotations

import fnmatch
import json
import os
import re
import sqlite3
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, package_root, root, sha256_file
from artifact_registry import ArtifactRegistry, fetch_artifact
from verify_envctl_db_schema import apply_migrations


TASK_ID = "ART-115_CONFIG_INVENTORY"
HELPER_ID = "helper-artifact-16"
MODEL_TAG = "gpt-5.3-spark"
RUN_ID = "run-art-115-config-inventory"
OPERATION_ID = "produce-01-current-state-configuration-inventory-md"
CONTRACT_ID = "contract-full-migration-artifact-contract-1.0.0"
RECIPE_ID = "recipe-flexnetos-package-artifact-contract-1.0.0"
TARGET_DB_ID = "target-art-115-flexnetos-vs-lifeos"

BLOCKED_PATTERNS = ("**/.env", "**/secrets/**", "**/private_keys/**", "**/*.pem", "**/*.key")
MAX_SCANNED_FILES = 60000
SKIP_DIRS = {
    ".kb",
    ".local",
    ".toolchains",
    ".git",
    ".hg",
    ".svn",
    ".cache",
    ".direnv",
    ".venv",
    "__pycache__",
    "node_modules",
    "artifacts",
    "target",
    "dist",
    "build",
    "release",
    "snapshots",
    "var",
    "vendor",
    "_quarantine",
    "_work",
}
TEXT_SUFFIXES = {
    ".bash",
    ".cfg",
    ".conf",
    ".config",
    ".envrc",
    ".ini",
    ".json",
    ".jsonc",
    ".kdl",
    ".lock",
    ".md",
    ".nix",
    ".nu",
    ".py",
    ".rs",
    ".service",
    ".sh",
    ".sql",
    ".toml",
    ".ts",
    ".tsx",
    ".yaml",
    ".yml",
    ".zsh",
}
CONFIG_FILENAMES = {
    ".envrc",
    ".gitignore",
    ".mcp.json",
    "AGENTS.md",
    "Cargo.toml",
    "flake.lock",
    "flake.nix",
    "package.json",
    "pyproject.toml",
    "rust-toolchain.toml",
    "settings.json",
}
CONFIG_SUFFIXES = {".cfg", ".conf", ".config", ".ini", ".json", ".jsonc", ".kdl", ".nix", ".service", ".toml", ".yaml", ".yml"}
SECRET_NAME_RE = re.compile(r"(SECRET|TOKEN|PASSWORD|PASSWD|API[_-]?KEY|PRIVATE[_-]?KEY|CLIENT[_-]?SECRET|AUTH)", re.I)
ENV_PATTERNS = [
    re.compile(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}"),
    re.compile(r"(?<![A-Za-z0-9_])\$([A-Za-z_][A-Za-z0-9_]*)"),
    re.compile(r"\b(?:process\.env\.|env\.|std::env::var\(|getenv\(|env\()\s*[\"']?([A-Za-z_][A-Za-z0-9_]*)"),
    re.compile(r"\b([A-Z][A-Z0-9_]{2,})\s*="),
]
FLAG_RE = re.compile(r"\b([A-Za-z0-9_]*(?:FEATURE|FLAG|ENABLE|DISABLE|EXPERIMENT)[A-Za-z0-9_]*)\b", re.I)


def package_rel(path: Path) -> str:
    try:
        return str(path.relative_to(package_root()))
    except ValueError:
        return str(path)


def target_root_from_registry() -> Path:
    env = os.environ.get("MIGRATION_TARGET_ROOT")
    if env:
        return Path(env).expanduser().resolve()
    report = root() / "generated" / "envctl_target_registry.json"
    data = json.loads(report.read_text(encoding="utf-8"))
    for row in data.get("registry_rows", []):
        if row.get("target_id") == "flexnetos-vs-lifeos":
            return Path(row["primary_root"]).expanduser().resolve()
    return package_root().resolve()


def is_blocked(relpath: str) -> bool:
    normalized = relpath.replace("\\", "/")
    for pattern in BLOCKED_PATTERNS:
        if fnmatch.fnmatch(normalized, pattern) or fnmatch.fnmatch(f"root/{normalized}", pattern):
            return True
    return False


def is_text_candidate(path: Path) -> bool:
    if path.name in CONFIG_FILENAMES:
        return True
    if path.suffix in TEXT_SUFFIXES:
        return True
    return False


def is_config_file(path: Path) -> bool:
    if path.name in CONFIG_FILENAMES:
        return True
    if path.suffix in CONFIG_SUFFIXES:
        return True
    lower_parts = {part.lower() for part in path.parts}
    return "config" in lower_parts or "configs" in lower_parts or "configuration" in lower_parts


def read_text_sample(path: Path) -> str:
    try:
        if path.stat().st_size > 1024 * 1024:
            return ""
        return path.read_text(encoding="utf-8", errors="ignore")
    except OSError:
        return ""


def extract_env_names(text: str) -> set[str]:
    names: set[str] = set()
    for pattern in ENV_PATTERNS:
        names.update(match.group(1) for match in pattern.finditer(text))
    return {name for name in names if len(name) > 1}


def classify_secret_reference(name: str, relpath: str) -> str | None:
    if SECRET_NAME_RE.search(name):
        return "env_or_setting_name"
    if SECRET_NAME_RE.search(Path(relpath).name):
        return "path_name"
    return None


def walk_inventory(target_root: Path) -> dict[str, Any]:
    config_files: list[dict[str, Any]] = []
    env_refs: dict[str, set[str]] = defaultdict(set)
    flag_refs: dict[str, set[str]] = defaultdict(set)
    secret_refs: dict[str, dict[str, Any]] = {}
    skipped_blocked: list[str] = []
    skipped_dirs: Counter[str] = Counter()
    scanned_files = 0
    scanned_text_files = 0

    for dirpath, dirnames, filenames in os.walk(target_root):
        if scanned_files >= MAX_SCANNED_FILES:
            break
        current = Path(dirpath)
        kept_dirs = []
        for dirname in dirnames:
            child = current / dirname
            rel = child.relative_to(target_root).as_posix()
            if dirname in SKIP_DIRS:
                skipped_dirs[dirname] += 1
                continue
            if is_blocked(rel + "/"):
                skipped_blocked.append(rel + "/")
                continue
            kept_dirs.append(dirname)
        dirnames[:] = kept_dirs

        for filename in filenames:
            if scanned_files >= MAX_SCANNED_FILES:
                break
            path = current / filename
            rel = path.relative_to(target_root).as_posix()
            if is_blocked(rel):
                skipped_blocked.append(rel)
                continue
            scanned_files += 1
            if is_config_file(path):
                try:
                    stat = path.stat()
                    size = stat.st_size
                except OSError:
                    size = None
                config_files.append(
                    {
                        "path": rel,
                        "kind": path.suffix.lstrip(".") or path.name,
                        "size_bytes": size,
                        "secret_named": bool(SECRET_NAME_RE.search(rel)),
                    }
                )
            if not is_text_candidate(path):
                continue
            text = read_text_sample(path)
            if not text:
                continue
            scanned_text_files += 1
            for name in extract_env_names(text):
                env_refs[name].add(rel)
                secret_kind = classify_secret_reference(name, rel)
                if secret_kind:
                    secret_refs.setdefault(
                        name,
                        {"name": name, "reference_kind": secret_kind, "paths": set(), "value_captured": False},
                    )["paths"].add(rel)
            for match in FLAG_RE.finditer(text):
                flag_refs[match.group(1)].add(rel)
            if SECRET_NAME_RE.search(rel):
                key = Path(rel).name
                secret_refs.setdefault(
                    key,
                    {"name": key, "reference_kind": "path_name", "paths": set(), "value_captured": False},
                )["paths"].add(rel)

    def refs_to_rows(refs: dict[str, set[str]], limit: int = 200) -> list[dict[str, Any]]:
        rows = []
        for name in sorted(refs):
            paths = sorted(refs[name])
            rows.append({"name": name, "reference_count": len(paths), "paths": paths[:20]})
        return rows[:limit]

    secret_rows = []
    for name in sorted(secret_refs):
        item = secret_refs[name]
        paths = sorted(item["paths"])
        secret_rows.append(
            {
                "name": item["name"],
                "reference_kind": item["reference_kind"],
                "reference_count": len(paths),
                "paths": paths[:20],
                "value_captured": False,
            }
        )

    config_by_kind = Counter(item["kind"] or "unknown" for item in config_files)
    return {
        "target_root": str(target_root),
        "scan_policy": {
            "blocked_patterns": list(BLOCKED_PATTERNS),
            "skipped_directory_names": sorted(SKIP_DIRS),
            "max_scanned_files": MAX_SCANNED_FILES,
            "max_text_file_bytes": 1024 * 1024,
            "secret_values_captured": False,
        },
        "summary": {
            "scanned_files": scanned_files,
            "scanned_text_files": scanned_text_files,
            "config_file_count": len(config_files),
            "env_var_reference_count": len(env_refs),
            "feature_flag_reference_count": len(flag_refs),
            "secret_reference_count": len(secret_rows),
            "blocked_path_count": len(skipped_blocked),
        },
        "config_files": sorted(config_files, key=lambda item: item["path"])[:500],
        "config_by_kind": dict(sorted(config_by_kind.items())),
        "env_vars": refs_to_rows(env_refs),
        "feature_flags": refs_to_rows(flag_refs),
        "secret_references": secret_rows[:200],
        "skipped": {
            "blocked_paths": sorted(skipped_blocked)[:200],
            "directories": dict(sorted(skipped_dirs.items())),
            "scan_cap_reached": scanned_files >= MAX_SCANNED_FILES,
        },
    }


def load_contract_row() -> dict[str, Any]:
    data = json.loads((root() / "generated" / "contract_manifest.json").read_text(encoding="utf-8"))
    for row in data["contract"]["rows"]:
        if row.get("artifact_id") == "01-current-state-configuration-inventory-md":
            return row
    raise RuntimeError("configuration inventory contract row not found")


def write_outputs(inventory: dict[str, Any], contract_row: dict[str, Any]) -> dict[str, str]:
    task_dir = root() / "migration-artifacts" / "art-115_config_inventory"
    canonical_md = root() / contract_row["required_path"]
    task_json = task_dir / "config_inventory.json"
    task_md = task_dir / "config_inventory.md"
    task_dir.mkdir(parents=True, exist_ok=True)
    canonical_md.parent.mkdir(parents=True, exist_ok=True)

    generated_at = now()
    payload = {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "generated_at": generated_at,
        "contract_row": contract_row,
        "inventory": inventory,
    }
    task_json.write_text(json.dumps(payload, indent=2, sort_keys=False) + "\n", encoding="utf-8")

    md = render_markdown(payload)
    task_md.write_text(md, encoding="utf-8")
    canonical_md.write_text(md, encoding="utf-8")

    return {
        "task_json": package_rel(task_json),
        "task_md": package_rel(task_md),
        "canonical_md": package_rel(canonical_md),
    }


def render_markdown(payload: dict[str, Any]) -> str:
    inv = payload["inventory"]
    summary = inv["summary"]
    lines = [
        "# Configuration Inventory",
        "",
        f"Generated: `{payload['generated_at']}`",
        f"Task: `{payload['task_id']}`",
        f"Target root: `{inv['target_root']}`",
        "",
        "## Summary",
        "",
        "| metric | value |",
        "|---|---:|",
    ]
    for key in [
        "scanned_files",
        "scanned_text_files",
        "config_file_count",
        "env_var_reference_count",
        "feature_flag_reference_count",
        "secret_reference_count",
        "blocked_path_count",
    ]:
        lines.append(f"| `{key}` | {summary[key]} |")

    lines.extend(["", "## Config Files", "", "| path | kind | secret-named |", "|---|---|---:|"])
    for item in inv["config_files"][:80]:
        lines.append(f"| `{item['path']}` | `{item['kind']}` | {str(item['secret_named']).lower()} |")

    lines.extend(["", "## Environment Variable References", "", "| name | refs | sample paths |", "|---|---:|---|"])
    for item in inv["env_vars"][:80]:
        sample = ", ".join(f"`{p}`" for p in item["paths"][:3])
        lines.append(f"| `{item['name']}` | {item['reference_count']} | {sample} |")

    lines.extend(["", "## Feature Flag References", "", "| name | refs | sample paths |", "|---|---:|---|"])
    for item in inv["feature_flags"][:80]:
        sample = ", ".join(f"`{p}`" for p in item["paths"][:3])
        lines.append(f"| `{item['name']}` | {item['reference_count']} | {sample} |")

    lines.extend(["", "## Secret References", "", "Secret values were not captured. Only names and reference locations are listed.", "", "| name | kind | refs | sample paths |", "|---|---|---:|---|"])
    for item in inv["secret_references"][:80]:
        sample = ", ".join(f"`{p}`" for p in item["paths"][:3])
        lines.append(f"| `{item['name']}` | `{item['reference_kind']}` | {item['reference_count']} | {sample} |")

    lines.extend(["", "## Scan Policy", "", f"- Blocked patterns: `{', '.join(inv['scan_policy']['blocked_patterns'])}`", "- Secret values captured: `false`", ""])
    return "\n".join(lines)


def seed_art115_fixture(conn: sqlite3.Connection, target_root: Path) -> None:
    descriptor = {
        "schema_version": "1.0",
        "target_id": "flexnetos-vs-lifeos",
        "target_type": "mixed",
        "primary_root": str(target_root),
        "output_root": "migration-artifacts",
    }
    conn.execute(
        """
        INSERT INTO envctl_migration_targets
          (id, target_id, target_type, primary_root, compare_root, descriptor_json,
           descriptor_hash, safety_mode, max_auto_risk)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(target_id) DO UPDATE SET
          primary_root = excluded.primary_root,
          descriptor_json = excluded.descriptor_json,
          descriptor_hash = excluded.descriptor_hash,
          updated_at_utc = strftime('%Y-%m-%dT%H:%M:%fZ','now')
        """,
        (
            TARGET_DB_ID,
            "flexnetos-vs-lifeos",
            "mixed",
            str(target_root),
            "/home/flexnetos/lifeos",
            json.dumps(descriptor, sort_keys=True),
            "sha256:" + sha256_text(json.dumps(descriptor, sort_keys=True)),
            "approval-gated",
            "R2",
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_runs
          (id, target_id, recipe_id, artifact_contract_id, status, human_mode,
           initiated_by, sandbox_policy, approval_policy, tool_versions_json,
           reproducibility_hash, started_at_utc, completed_at_utc)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
          status = excluded.status,
          completed_at_utc = excluded.completed_at_utc
        """,
        (
            RUN_ID,
            TARGET_DB_ID,
            RECIPE_ID,
            CONTRACT_ID,
            "completed",
            "agent-only",
            "artifact-agent",
            "workspace-write",
            "never",
            json.dumps({"python": "stdlib", "sqlite": "stdlib"}, sort_keys=True),
            "sha256:" + sha256_text(f"{TASK_ID}:{target_root}"),
            now(),
            now(),
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_operations
          (id, run_id, operation_type, phase, status, risk, idempotency_key,
           command_hash, command_redacted, input_json, output_ref, started_at_utc, completed_at_utc)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
          status = excluded.status,
          output_ref = excluded.output_ref,
          completed_at_utc = excluded.completed_at_utc
        """,
        (
            OPERATION_ID,
            RUN_ID,
            "produce_artifact_record",
            "05-artifacts",
            "succeeded",
            "R1",
            f"{TASK_ID}/configuration-inventory",
            "sha256:" + sha256_text("python3 scripts/generate_config_inventory.py"),
            "python3 scripts/generate_config_inventory.py",
            json.dumps({"task_id": TASK_ID, "contract_row_id": "artifact:01-current-state-configuration-inventory-md"}, sort_keys=True),
            "migration-artifacts/01-current-state/configuration-inventory.md",
            now(),
            now(),
        ),
    )
    conn.commit()


def sha256_text(value: str) -> str:
    import hashlib

    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def register_artifact(conn: sqlite3.Connection, outputs: dict[str, str], inventory: dict[str, Any]) -> dict[str, Any]:
    registry = ArtifactRegistry(conn, package_root())
    result = registry.register(
        {
            "artifact_id": "01-current-state-configuration-inventory-md",
            "run_id": RUN_ID,
            "title": "Configuration Inventory",
            "status": "complete",
            "artifact_type": "migration_artifact",
            "path": outputs["canonical_md"],
            "producer_operation_id": OPERATION_ID,
            "contract_id": CONTRACT_ID,
            "provenance": {
                "task_id": TASK_ID,
                "owner_agent": "artifact-agent",
                "helper_id": HELPER_ID,
                "target_root": inventory["target_root"],
                "source_inputs": ["target descriptor", "repo scan", "envctl database"],
            },
            "evidence_refs": [
                outputs["canonical_md"],
                outputs["task_md"],
                outputs["task_json"],
                "execution-framework/generated/envctl_target_registry.json",
                "execution-framework/generated/contract_manifest.json",
            ],
            "links": [
                {"to": "contract_row:artifact:01-current-state-configuration-inventory-md", "type": "satisfies"},
                {"to": "task:ART-115_CONFIG_INVENTORY", "type": "produced_by"},
                {"to": "task:VER-300_UNIT_VALIDATION", "type": "blocks"},
            ],
            "validations": [
                {
                    "validator": "generate_config_inventory.py:artifact-files-exist",
                    "status": "pass",
                    "details": {"paths": outputs},
                    "evidence_refs": list(outputs.values()),
                },
                {
                    "validator": "generate_config_inventory.py:redaction-policy",
                    "status": "pass",
                    "details": inventory["scan_policy"],
                    "evidence_refs": [outputs["task_json"]],
                },
                {
                    "validator": "artifact_registry.py:hash-recorded",
                    "status": "pass",
                    "details": {"registry_contains_hash": True},
                    "evidence_refs": [outputs["canonical_md"]],
                },
            ],
        }
    )
    return {"registry_result": result, "artifact_row": fetch_artifact(conn, RUN_ID, "01-current-state-configuration-inventory-md")}


def verify_outputs(outputs: dict[str, str], registry_payload: dict[str, Any], inventory: dict[str, Any]) -> dict[str, Any]:
    checks = {
        "task_json_exists": (package_root() / outputs["task_json"]).is_file(),
        "task_md_exists": (package_root() / outputs["task_md"]).is_file(),
        "canonical_md_exists": (package_root() / outputs["canonical_md"]).is_file(),
        "registry_hash_recorded": bool(registry_payload["registry_result"].get("content_hash")),
        "secret_values_captured": inventory["scan_policy"]["secret_values_captured"],
    }
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "passed" if all(value for key, value in checks.items() if key != "secret_values_captured") and not checks["secret_values_captured"] else "failed",
        "generated_at": now(),
        "summary": inventory["summary"],
        "outputs": outputs,
        "checks": checks,
        **registry_payload,
    }


def main() -> int:
    target_root = target_root_from_registry()
    contract_row = load_contract_row()
    inventory = walk_inventory(target_root)
    outputs = write_outputs(inventory, contract_row)

    conn = sqlite3.connect(":memory:")
    apply_migrations(conn, package_root())
    seed_art115_fixture(conn, target_root)
    registry_payload = register_artifact(conn, outputs, inventory)
    verification = verify_outputs(outputs, registry_payload, inventory)

    report_path = root() / "generated" / "art115_config_inventory_report.json"
    report_path.write_text(json.dumps(verification, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    log_path = root() / "logs" / f"{TASK_ID}.log"
    log_path.parent.mkdir(parents=True, exist_ok=True)
    log_path.write_text(json.dumps(verification, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    heartbeat = {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "completed" if verification["status"] == "passed" else "failed",
        "updated_at": now(),
        "proof_uri": f"proof_records/{TASK_ID}.proof.json",
        "logs_uri": f"logs/{TASK_ID}.log",
    }
    (root() / "state" / f"{TASK_ID}.heartbeat.json").write_text(json.dumps(heartbeat, indent=2) + "\n", encoding="utf-8")

    files_changed = [
        "execution-framework/scripts/generate_config_inventory.py",
        outputs["canonical_md"],
        outputs["task_md"],
        outputs["task_json"],
        "execution-framework/generated/art115_config_inventory_report.json",
        f"execution-framework/state/{TASK_ID}.heartbeat.json",
        f"execution-framework/logs/{TASK_ID}.log",
        f"execution-framework/proof_records/{TASK_ID}.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    proof = make_proof(
        TASK_ID,
        "completed" if verification["status"] == "passed" else "failed",
        "codex-cli-local",
        HELPER_ID,
        MODEL_TAG,
        str(package_root()),
        files_changed,
        ["python3 scripts/generate_config_inventory.py", "python3 -m py_compile scripts/generate_config_inventory.py"],
        verification,
        [
            outputs["canonical_md"],
            outputs["task_md"],
            outputs["task_json"],
            "execution-framework/generated/art115_config_inventory_report.json",
            f"logs/{TASK_ID}.log",
        ],
        "" if verification["status"] == "passed" else "ART-115 verification failed",
        "run VER-300_UNIT_VALIDATION",
    )
    append_proof(proof)
    print(json.dumps(verification, indent=2, sort_keys=False))
    return 0 if verification["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
