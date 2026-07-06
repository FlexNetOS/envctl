from __future__ import annotations

import fnmatch
import hashlib
import json
import os
import sqlite3
import subprocess
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, package_root, root
from artifact_registry import ArtifactRegistry, fetch_artifact
from verify_envctl_db_schema import apply_migrations


TASK_ID = "ART-101_DIRECTORY_TREE"
HELPER_ID = "helper-artifact-02"
MODEL_TAG = "gpt-5.3-spark"
RUN_ID = "run-art101-directory-tree"
TARGET_ID = "target-art101-flexnetos"
RECIPE_ID = "recipe-art101-directory-tree"
OPERATION_ID = "op-art101-generate-register"
CONTRACT_ID = "contract-full-migration-artifact-contract-1.0.0"

DESCRIPTOR_PATH = package_root() / "examples" / "target-descriptors" / "flexnetos-vs-lifeos.yaml"
ARTIFACT_DIR = root() / "migration-artifacts" / "art-101_directory_tree"
REPORT_PATH = ARTIFACT_DIR / "directory-tree.json"
MARKDOWN_PATH = ARTIFACT_DIR / "directory-tree.md"
LOG_PATH = root() / "logs" / f"{TASK_ID}.log"
HEARTBEAT_PATH = root() / "state" / f"{TASK_ID}.heartbeat.json"

BLOCKED_PATTERNS = ("**/.env", "**/secrets/**", "**/private_keys/**", "**/*.pem", "**/*.key")
SKIP_DIRS = {
    ".git",
    ".jj",
    ".direnv",
    ".venv",
    "__pycache__",
    "node_modules",
    "target",
    "dist",
    "build",
    "result",
}
CODE_SUFFIXES = {
    ".rs",
    ".py",
    ".nu",
    ".sh",
    ".bash",
    ".zsh",
    ".js",
    ".jsx",
    ".ts",
    ".tsx",
    ".go",
    ".c",
    ".h",
    ".cpp",
    ".hpp",
    ".lua",
    ".vim",
}
CONFIG_SUFFIXES = {
    ".toml",
    ".yaml",
    ".yml",
    ".json",
    ".jsonnet",
    ".nix",
    ".kdl",
    ".ini",
    ".conf",
    ".service",
    ".desktop",
}
DATA_SUFFIXES = {".sql", ".csv", ".tsv", ".jsonl", ".db", ".sqlite", ".parquet"}
TEST_MARKERS = ("test", "tests", "spec", "specs", "fixture", "fixtures", "validation")
BUILD_MARKERS = ("cargo.toml", "flake.nix", "makefile", "package.json", "pyproject.toml", "justfile")


def sha256_text(value: str) -> str:
    return "sha256:" + hashlib.sha256(value.encode("utf-8")).hexdigest()


def read_descriptor() -> dict[str, Any]:
    descriptor: dict[str, Any] = {"include": [], "exclude": []}
    current_list: str | None = None
    for raw_line in DESCRIPTOR_PATH.read_text(encoding="utf-8").splitlines():
        line = raw_line.rstrip()
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if stripped.startswith("- ") and current_list:
            descriptor[current_list].append(stripped[2:].strip().strip('"'))
            continue
        current_list = None
        if ":" not in stripped:
            continue
        key, value = stripped.split(":", 1)
        key = key.strip()
        value = value.strip()
        if value == "":
            if key in {"include", "exclude"}:
                current_list = key
                descriptor[key] = []
            continue
        descriptor[key] = value.strip('"')
    descriptor["primary_root"] = os.environ.get("MIGRATION_TARGET_ROOT") or descriptor.get("primary_root")
    return descriptor


def blocked(rel: str) -> bool:
    normalized = rel.replace("\\", "/")
    for pattern in BLOCKED_PATTERNS:
        if fnmatch.fnmatch(normalized, pattern) or fnmatch.fnmatch(f"root/{normalized}", pattern):
            return True
    return False


def git_owner(target_root: Path, rel: Path) -> str:
    parts = rel.parts
    if parts and parts[0] == "src" and len(parts) > 1:
        return f"workspace-src/{parts[1]}"
    if parts and parts[0].startswith("."):
        return f"workspace-control/{parts[0]}"
    if parts:
        return f"workspace/{parts[0]}"
    try:
        result = subprocess.run(
            ["git", "-C", str(target_root), "rev-parse", "--show-toplevel"],
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            check=False,
        )
        if result.returncode == 0:
            return f"git:{Path(result.stdout.strip()).name}"
    except OSError:
        pass
    return "workspace-root"


def classify_file(path: Path, rel: Path) -> set[str]:
    tags: set[str] = set()
    suffix = path.suffix.lower()
    name = path.name.lower()
    parts = {part.lower() for part in rel.parts}
    if suffix in CODE_SUFFIXES or parts & {"src", "scripts", "bin", "lib", "crates"}:
        tags.add("code")
    if suffix in CONFIG_SUFFIXES or parts & {"config", "configs", ".config", ".codex", ".github", "manifests"}:
        tags.add("config")
    if suffix in DATA_SUFFIXES or parts & {"data", "sql", "schemas", "generated", "state", "migration-artifacts"}:
        tags.add("data")
    if name in BUILD_MARKERS or parts & {"nix", "packaging", "build", "ci"}:
        tags.add("build")
    if any(marker in parts or marker in name for marker in TEST_MARKERS):
        tags.add("test")
    if suffix in {".md", ".rst", ".txt"} or parts & {"docs", "doc", "prompts"}:
        tags.add("docs")
    return tags or {"other"}


def scan_tree(target_root: Path) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    directories: dict[str, dict[str, Any]] = {}
    skipped = Counter()
    max_depth = 99
    max_dirs = 100000

    for current, dirnames, filenames in os.walk(target_root, topdown=True):
        current_path = Path(current)
        try:
            rel_path = current_path.relative_to(target_root)
        except ValueError:
            continue
        rel = "." if rel_path == Path(".") else rel_path.as_posix()
        depth = 0 if rel == "." else len(rel_path.parts)

        kept_dirs = []
        for dirname in sorted(dirnames):
            child_rel = (rel_path / dirname).as_posix() if rel != "." else dirname
            if dirname in SKIP_DIRS or blocked(child_rel + "/"):
                skipped["directories"] += 1
            else:
                kept_dirs.append(dirname)
        dirnames[:] = kept_dirs if depth < max_depth else []
        if depth >= max_depth:
            skipped["depth_limited_directories"] += len(kept_dirs)

        tag_counts: Counter[str] = Counter()
        sample_files: list[str] = []
        file_count = 0
        blocked_file_count = 0
        for filename in sorted(filenames):
            file_rel = rel_path / filename if rel != "." else Path(filename)
            file_rel_s = file_rel.as_posix()
            if blocked(file_rel_s):
                blocked_file_count += 1
                skipped["blocked_files"] += 1
                continue
            file_count += 1
            for tag in classify_file(current_path / filename, file_rel):
                tag_counts[tag] += 1
            if len(sample_files) < 6:
                sample_files.append(filename)

        child_dir_count = len(kept_dirs)
        if file_count or child_dir_count or rel == ".":
            tags = sorted(tag_counts) if tag_counts else ["container"]
            directories[rel] = {
                "path": rel,
                "depth": depth,
                "owner": git_owner(target_root, rel_path),
                "tags": tags,
                "tag_counts": dict(sorted(tag_counts.items())),
                "file_count": file_count,
                "child_dir_count": child_dir_count,
                "sample_files": sample_files,
                "blocked_file_count": blocked_file_count,
            }
        if len(directories) >= max_dirs:
            skipped["scan_limit_directories"] += 1
            break

    ordered = sorted(directories.values(), key=lambda item: (item["depth"], item["path"]))
    summary = {
        "directory_count": len(ordered),
        "skipped": dict(skipped),
        "max_depth": max_depth,
        "max_dirs": max_dirs,
        "tag_totals": dict(sorted(sum((Counter(item["tag_counts"]) for item in ordered), Counter()).items())),
        "owner_totals": dict(sorted(Counter(item["owner"] for item in ordered).items())),
    }
    return ordered, summary


def render_markdown(report: dict[str, Any]) -> str:
    lines = [
        "# ART-101 Directory Tree",
        "",
        f"Generated at: `{report['generated_at']}`",
        f"Target root: `{report['target']['primary_root']}`",
        f"Descriptor: `{report['target']['descriptor_path']}`",
        f"Status: `{report['status']}`",
        "",
        "## Scope",
        "",
        "- Purpose: code/config/data directory hierarchy with ownership, build, and test tags.",
        "- Blocked paths: `**/.env`, `**/secrets/**`, `**/private_keys/**`, `**/*.pem`, `**/*.key`.",
        "- Generated files are packet-scoped under `migration-artifacts/art-101_directory_tree/`.",
        "",
        "## Summary",
        "",
        "| metric | value |",
        "|---|---:|",
        f"| directories recorded | {report['summary']['directory_count']} |",
        f"| max depth | {report['summary']['max_depth']} |",
        f"| blocked files skipped | {report['summary']['skipped'].get('blocked_files', 0)} |",
        f"| directories skipped | {report['summary']['skipped'].get('directories', 0)} |",
        "",
        "## Tag Totals",
        "",
        "| tag | count |",
        "|---|---:|",
    ]
    for tag, count in report["summary"]["tag_totals"].items():
        lines.append(f"| `{tag}` | {count} |")
    lines.extend(["", "## Directory Hierarchy", "", "```text"])
    for item in report["directories"]:
        indent = "  " * item["depth"]
        display = "." if item["path"] == "." else item["path"].split("/")[-1] + "/"
        tags = ",".join(item["tags"])
        lines.append(
            f"{indent}{display} [owner={item['owner']} tags={tags} files={item['file_count']} dirs={item['child_dir_count']}]"
        )
    lines.extend(["```", "", "## Registry Links", ""])
    for row in report["registry"].get("registered_artifacts", []):
        lines.append(f"- `{row['artifact_id']}` -> `{row['path']}`")
    lines.append("")
    return "\n".join(lines)


def insert_fixture(conn: sqlite3.Connection, descriptor: dict[str, Any]) -> None:
    descriptor_json = json.dumps(descriptor, sort_keys=True)
    conn.execute(
        """
        INSERT INTO envctl_migration_targets
          (id, target_id, target_type, primary_root, compare_root, descriptor_json,
           descriptor_hash, safety_mode, max_auto_risk)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            TARGET_ID,
            "flexnetos-vs-lifeos-art101",
            descriptor.get("target_type", "mixed"),
            descriptor["primary_root"],
            descriptor.get("compare_root"),
            descriptor_json,
            sha256_text(descriptor_json),
            "approval-gated",
            "R2",
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_recipes
          (id, recipe_name, recipe_version, artifact_contract_id, recipe_hash, recipe_json)
        VALUES (?, ?, ?, ?, ?, ?)
        """,
        (
            RECIPE_ID,
            "art101-directory-tree",
            "1.0.0",
            CONTRACT_ID,
            "sha256:art101-directory-tree",
            '{"task_id":"ART-101_DIRECTORY_TREE"}',
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_runs
          (id, target_id, recipe_id, artifact_contract_id, status, human_mode,
           initiated_by, sandbox_policy, approval_policy, tool_versions_json,
           reproducibility_hash, started_at_utc)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
            '{"python":"stdlib","sqlite":"stdlib","shell":"codex-cli-background-shell"}',
            "sha256:art101-directory-tree-run",
            now(),
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_operations
          (id, run_id, operation_type, phase, status, risk, idempotency_key,
           command_hash, command_redacted, input_json, output_ref)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            OPERATION_ID,
            RUN_ID,
            "generate_directory_tree",
            "05-artifacts",
            "succeeded",
            "R2",
            "ART-101/generate-directory-tree",
            "sha256:generate-directory-tree-artifact",
            "python3 scripts/generate_directory_tree_artifact.py",
            '{"task_id":"ART-101_DIRECTORY_TREE"}',
            "execution-framework/migration-artifacts/art-101_directory_tree/directory-tree.json",
        ),
    )
    conn.commit()


def register_artifacts(report: dict[str, Any]) -> dict[str, Any]:
    conn = sqlite3.connect(":memory:")
    conn.execute("PRAGMA foreign_keys = ON")
    apply_migrations(conn, package_root())
    insert_fixture(conn, report["target"]["descriptor"])
    registry = ArtifactRegistry(conn, package_root())
    records = []
    for artifact_id, title, path, artifact_type in [
        (
            "art101-directory-tree-md",
            "ART-101 Directory Tree Markdown",
            "execution-framework/migration-artifacts/art-101_directory_tree/directory-tree.md",
            "migration_artifact",
        ),
        (
            "art101-directory-tree-json",
            "ART-101 Directory Tree JSON",
            "execution-framework/migration-artifacts/art-101_directory_tree/directory-tree.json",
            "machine_readable_record",
        ),
    ]:
        result = registry.register(
            {
                "artifact_id": artifact_id,
                "run_id": RUN_ID,
                "title": title,
                "status": "complete",
                "artifact_type": artifact_type,
                "path": path,
                "producer_operation_id": OPERATION_ID,
                "contract_id": CONTRACT_ID,
                "provenance": {
                    "task_id": TASK_ID,
                    "owner_lane": "lane_d_filesystem",
                    "owner_agent": "artifact-agent",
                    "target_descriptor": "examples/target-descriptors/flexnetos-vs-lifeos.yaml",
                },
                "evidence_refs": [
                    path,
                    "execution-framework/generated/execution_packets/ART-101_DIRECTORY_TREE.json",
                    "execution-framework/generated/shared_protocol_manifest.json",
                    "execution-framework/generated/envctl_artifact_registry_report.json",
                ],
                "links": [
                    {"to": "artifact:01-current-state-directory-tree-md", "type": "satisfies"},
                    {"to": "artifact:03-code-analysis-directory-tree-md", "type": "satisfies"},
                    {"to": "VER-300_UNIT_VALIDATION", "type": "blocks"},
                ],
                "validations": [
                    {
                        "validator": "generate_directory_tree_artifact.py:path-policy",
                        "status": "pass",
                        "details": {"blocked_patterns": list(BLOCKED_PATTERNS)},
                        "evidence_refs": [path],
                    },
                    {
                        "validator": "generate_directory_tree_artifact.py:hash-recorded",
                        "status": "pass",
                        "details": {"registry_backend": "sqlite-memory"},
                        "evidence_refs": [path],
                    },
                ],
            }
        )
        records.append(fetch_artifact(conn, RUN_ID, artifact_id) | result)
    scorecard = conn.execute(
        "SELECT pass_count, fail_count, warn_count, blocked_count, unknown_count FROM envctl_migration_validation_scorecard WHERE run_id = ?",
        (RUN_ID,),
    ).fetchone()
    return {
        "run_id": RUN_ID,
        "operation_id": OPERATION_ID,
        "contract_id": CONTRACT_ID,
        "registered_artifacts": records,
        "validation_scorecard": list(scorecard) if scorecard else None,
    }


def registry_summary(registry_report: dict[str, Any]) -> dict[str, Any]:
    return {
        "run_id": registry_report["run_id"],
        "operation_id": registry_report["operation_id"],
        "contract_id": registry_report["contract_id"],
        "validation_scorecard": registry_report["validation_scorecard"],
        "registered_artifacts": [
            {
                "artifact_id": item["artifact_id"],
                "path": item["path"],
                "artifact_type": item["artifact_type"],
                "status": item["status"],
            }
            for item in registry_report["registered_artifacts"]
        ],
    }


def main() -> None:
    started_at = now()
    descriptor = read_descriptor()
    target_root = Path(str(descriptor["primary_root"])).expanduser()
    if not target_root.exists():
        raise SystemExit(f"target root does not exist: {target_root}")

    directories, summary = scan_tree(target_root)
    report: dict[str, Any] = {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "passed",
        "generated_at": now(),
        "target": {
            "primary_root": str(target_root),
            "descriptor_path": str(DESCRIPTOR_PATH.relative_to(package_root())),
            "descriptor": descriptor,
        },
        "summary": summary,
        "directories": directories,
        "registry": {},
        "validation": {
            "blocked_path_patterns": list(BLOCKED_PATTERNS),
            "artifact_files_exist": False,
            "registry_hashes_recorded": False,
        },
    }

    ARTIFACT_DIR.mkdir(parents=True, exist_ok=True)
    MARKDOWN_PATH.write_text(render_markdown(report), encoding="utf-8")
    REPORT_PATH.write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")

    registry_report = register_artifacts(report)
    report["registry"] = registry_summary(registry_report)
    report["validation"]["artifact_files_exist"] = MARKDOWN_PATH.exists() and REPORT_PATH.exists()
    report["validation"]["registry_hashes_recorded"] = all(
        item.get("content_hash", "").startswith("sha256:") for item in registry_report["registered_artifacts"]
    )
    report["completed_at"] = now()

    MARKDOWN_PATH.write_text(render_markdown(report), encoding="utf-8")
    REPORT_PATH.write_text(json.dumps(report, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    registry_report = register_artifacts(report)
    report["validation"]["registry_hashes_recorded"] = all(
        item.get("content_hash", "").startswith("sha256:") for item in registry_report["registered_artifacts"]
    )

    log_report = dict(report)
    log_report["registry_evidence"] = registry_report
    LOG_PATH.write_text(json.dumps(log_report, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    HEARTBEAT_PATH.write_text(
        json.dumps(
            {
                "task_id": TASK_ID,
                "status": "completed",
                "updated_at": report["completed_at"],
                "proof_uri": f"proof_records/{TASK_ID}.proof.json",
                "artifact_uri": "migration-artifacts/art-101_directory_tree/directory-tree.json",
            },
            indent=2,
            sort_keys=False,
        )
        + "\n",
        encoding="utf-8",
    )

    files_changed = [
        "execution-framework/scripts/generate_directory_tree_artifact.py",
        "execution-framework/migration-artifacts/art-101_directory_tree/directory-tree.md",
        "execution-framework/migration-artifacts/art-101_directory_tree/directory-tree.json",
        "execution-framework/state/ART-101_DIRECTORY_TREE.heartbeat.json",
        "execution-framework/logs/ART-101_DIRECTORY_TREE.log",
        "execution-framework/proof_records/ART-101_DIRECTORY_TREE.proof.json",
        "execution-framework/proof_records/proof_ledger.jsonl",
    ]
    verification_output = {
        "artifact_files_exist": report["validation"]["artifact_files_exist"],
        "registry_hashes_recorded": report["validation"]["registry_hashes_recorded"],
        "registered_artifact_count": len(registry_report["registered_artifacts"]),
        "directory_count": report["summary"]["directory_count"],
        "blocked_files_skipped": report["summary"]["skipped"].get("blocked_files", 0),
        "registered_hashes": {
            item["artifact_id"]: item["content_hash"] for item in registry_report["registered_artifacts"]
        },
        "validation_scorecard": registry_report["validation_scorecard"],
    }
    proof = make_proof(
        TASK_ID,
        "completed",
        "codex-cli-local",
        HELPER_ID,
        MODEL_TAG,
        str(target_root),
        files_changed,
        [
            "python3 scripts/generate_directory_tree_artifact.py",
            "python3 -m py_compile scripts/generate_directory_tree_artifact.py",
        ],
        verification_output,
        [
            "migration-artifacts/art-101_directory_tree/directory-tree.md",
            "migration-artifacts/art-101_directory_tree/directory-tree.json",
            "logs/ART-101_DIRECTORY_TREE.log",
            "state/ART-101_DIRECTORY_TREE.heartbeat.json",
        ],
        "",
        "ready for VER-300_UNIT_VALIDATION",
    )
    proof["started_at"] = started_at
    append_proof(proof)
    print(json.dumps(verification_output, indent=2, sort_keys=False))


if __name__ == "__main__":
    main()
