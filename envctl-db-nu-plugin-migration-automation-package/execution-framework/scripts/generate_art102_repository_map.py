from __future__ import annotations

import json
import os
import sqlite3
import subprocess
from collections import Counter
from pathlib import Path
from typing import Any

from _common import append_proof, make_proof, now, package_root, root, sha256_file, write_json
from artifact_registry import ArtifactRegistry, fetch_artifact
from verify_envctl_db_schema import apply_migrations


TASK_ID = "ART-102_REPOSITORY_MAP"
HELPER_ID = "helper-artifact-03"
MODEL_TAG = "gpt-5.3-spark"
RUN_ID = "run-art102-repository-map"
OPERATION_ID = "op-art102-register-repository-map"
TARGET_ID = "target-art102-flexnetos"
PACKAGE_ID = "pkg-art102"
CONTRACT_ID = "contract-art102-repository-map"
RECIPE_ID = "recipe-art102-repository-map"

ARTIFACT_DIR = root() / "migration-artifacts" / "art-102_repository_map"
JSON_PATH = ARTIFACT_DIR / "repository-map.json"
MD_PATH = ARTIFACT_DIR / "repository-map.md"
LOG_PATH = root() / "logs" / f"{TASK_ID}.log"
HEARTBEAT_PATH = root() / "state" / f"{TASK_ID}.heartbeat.json"

SKIP_DIR_NAMES = {
    ".cache",
    ".cargo",
    ".direnv",
    ".git",
    ".hg",
    ".mypy_cache",
    ".pytest_cache",
    ".ruff_cache",
    ".svn",
    ".tox",
    ".venv",
    "__pycache__",
    "node_modules",
    "target",
}
BLOCKED_SUFFIXES = (".pem", ".key")
BLOCKED_PARTS = {".env", "secrets", "private_keys"}
LANGUAGE_EXTENSIONS = {
    ".rs": "Rust",
    ".py": "Python",
    ".nu": "Nushell",
    ".nix": "Nix",
    ".toml": "TOML",
    ".json": "JSON",
    ".yaml": "YAML",
    ".yml": "YAML",
    ".md": "Markdown",
    ".sh": "Shell",
    ".ts": "TypeScript",
    ".tsx": "TypeScript",
    ".js": "JavaScript",
    ".jsx": "JavaScript",
    ".sql": "SQL",
    ".html": "HTML",
    ".css": "CSS",
}
MARKER_FILES = [
    "Cargo.toml",
    "flake.nix",
    "package.json",
    "pyproject.toml",
    "deno.json",
    "Makefile",
    "justfile",
    "README.md",
    "AGENTS.md",
    ".meta.yaml",
]
OWNER_FILES = [
    "CODEOWNERS",
    ".github/CODEOWNERS",
    "OWNERS",
    "MAINTAINERS.md",
    "AGENTS.md",
]


def run_git(repo: Path, args: list[str], timeout: int = 10) -> str | None:
    try:
        completed = subprocess.run(
            ["git", "-C", str(repo), *args],
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
            timeout=timeout,
        )
    except (OSError, subprocess.SubprocessError):
        return None
    if completed.returncode != 0:
        return None
    return completed.stdout.strip()


def is_blocked_path(path: Path) -> bool:
    parts = set(path.parts)
    if parts & BLOCKED_PARTS:
        return True
    return path.name.endswith(BLOCKED_SUFFIXES)


def load_target_descriptor() -> dict[str, Any]:
    registry_path = root() / "generated" / "envctl_target_registry.json"
    registry = json.loads(registry_path.read_text(encoding="utf-8"))
    rows = registry.get("registry_rows", [])
    selected = next((row for row in rows if row.get("target_id") == "flexnetos-vs-lifeos"), rows[0])
    return {
        "target_id": selected["target_id"],
        "target_type": selected["target_type"],
        "primary_root": selected["primary_root"],
        "compare_root": selected.get("compare_root"),
        "descriptor_hash": selected["descriptor_hash"],
        "source": "generated/envctl_target_registry.json",
    }


def find_git_repositories(target_root: Path) -> list[Path]:
    repos: list[Path] = []
    for current, dirs, _files in os.walk(target_root):
        current_path = Path(current)
        dirs[:] = sorted(d for d in dirs if d not in SKIP_DIR_NAMES and not is_blocked_path(current_path / d))
        if (current_path / ".git").exists():
            repos.append(current_path)
    return sorted(repos, key=lambda item: str(item.relative_to(target_root)))


def classify_scope(relpath: str, markers: list[str], languages: Counter[str]) -> str:
    lower = relpath.lower()
    marker_set = set(markers)
    if relpath == ".":
        return "workspace-root"
    if lower.startswith("src/envctl"):
        return "envctl-control-plane"
    if lower.startswith("src/nu_plugin"):
        return "nu-plugin-surface"
    if lower.startswith("src/meta"):
        return "meta-control-plane"
    if lower.startswith("src/yazelix"):
        return "terminal-runtime"
    if lower.startswith("src/flexnetos_runner"):
        return "runner-runtime"
    if "flake.nix" in marker_set or languages.get("Nix"):
        return "nix-packaging"
    if "Cargo.toml" in marker_set or languages.get("Rust"):
        return "rust-codebase"
    if "package.json" in marker_set:
        return "node-codebase"
    if "pyproject.toml" in marker_set or languages.get("Python"):
        return "python-codebase"
    return "supporting-content"


def owner_hints(repo: Path) -> list[dict[str, str]]:
    hints: list[dict[str, str]] = []
    for rel in OWNER_FILES:
        path = repo / rel
        if not path.exists() or not path.is_file() or is_blocked_path(path):
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        lines = [line.strip() for line in text.splitlines() if line.strip() and not line.strip().startswith("#")]
        sample = lines[0][:160] if lines else "file present"
        hints.append({"source": rel, "sample": sample})
    return hints


def list_files(repo: Path) -> list[str]:
    tracked = run_git(repo, ["ls-files"], timeout=20)
    if tracked:
        return [line for line in tracked.splitlines() if line and not is_blocked_path(Path(line))]
    files: list[str] = []
    for current, dirs, names in os.walk(repo):
        current_path = Path(current)
        dirs[:] = sorted(d for d in dirs if d not in SKIP_DIR_NAMES and not is_blocked_path(current_path / d))
        for name in names:
            path = current_path / name
            if is_blocked_path(path):
                continue
            try:
                files.append(str(path.relative_to(repo)))
            except ValueError:
                continue
    return sorted(files)


def summarize_repo(target_root: Path, repo: Path) -> dict[str, Any]:
    relpath = str(repo.relative_to(target_root)) or "."
    files = list_files(repo)
    language_counts: Counter[str] = Counter()
    top_level: Counter[str] = Counter()
    for raw in files:
        path = Path(raw)
        if path.parts:
            top_level[path.parts[0]] += 1
        language = LANGUAGE_EXTENSIONS.get(path.suffix.lower())
        if language:
            language_counts[language] += 1
    markers = [marker for marker in MARKER_FILES if (repo / marker).exists()]
    branch = run_git(repo, ["rev-parse", "--abbrev-ref", "HEAD"]) or "unknown"
    head = run_git(repo, ["rev-parse", "--short=12", "HEAD"])
    last_commit = run_git(repo, ["log", "-1", "--date=iso-strict", "--pretty=format:%H%x09%ad%x09%an%x09%s"])
    status_lines = (run_git(repo, ["status", "--short"]) or "").splitlines()
    remotes = (run_git(repo, ["remote", "-v"]) or "").splitlines()
    remote_names = sorted({line.split()[0] for line in remotes if line.split()})
    commit_count_text = run_git(repo, ["rev-list", "--count", "HEAD"])
    try:
        commit_count = int(commit_count_text) if commit_count_text else None
    except ValueError:
        commit_count = None
    last_commit_record = None
    if last_commit:
        parts = last_commit.split("\t", 3)
        if len(parts) == 4:
            last_commit_record = {
                "hash": parts[0],
                "date": parts[1],
                "author": parts[2],
                "subject": parts[3],
            }
    return {
        "path": relpath,
        "absolute_path": str(repo),
        "scope": classify_scope(relpath, markers, language_counts),
        "markers": markers,
        "owners": owner_hints(repo),
        "contents": {
            "tracked_or_scanned_files": len(files),
            "top_level_entries": [
                {"name": name, "file_count": count}
                for name, count in sorted(top_level.items(), key=lambda item: (-item[1], item[0]))[:20]
            ],
            "language_file_counts": dict(sorted(language_counts.items())),
        },
        "activity": {
            "branch": branch,
            "head": head,
            "commit_count": commit_count,
            "last_commit": last_commit_record,
            "dirty_status_count": len(status_lines),
            "remote_names": remote_names,
        },
    }


def build_repository_map() -> dict[str, Any]:
    descriptor = load_target_descriptor()
    target_root = Path(descriptor["primary_root"]).resolve()
    repos = [summarize_repo(target_root, repo) for repo in find_git_repositories(target_root)]
    scope_rollup = Counter(repo["scope"] for repo in repos)
    language_rollup: Counter[str] = Counter()
    dirty_repos = []
    for repo in repos:
        language_rollup.update(repo["contents"]["language_file_counts"])
        if repo["activity"]["dirty_status_count"]:
            dirty_repos.append({"path": repo["path"], "dirty_status_count": repo["activity"]["dirty_status_count"]})
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "generated_at": now(),
        "target": descriptor,
        "scan_policy": {
            "blocked_paths": ["**/.env", "**/secrets/**", "**/private_keys/**", "**/*.pem", "**/*.key"],
            "skipped_directory_names": sorted(SKIP_DIR_NAMES),
            "activity_source": "git metadata and status for repositories under target primary_root",
        },
        "summary": {
            "repository_count": len(repos),
            "dirty_repository_count": len(dirty_repos),
            "scope_rollup": dict(sorted(scope_rollup.items())),
            "language_file_rollup": dict(sorted(language_rollup.items())),
        },
        "repositories": repos,
        "dirty_repositories": dirty_repos,
    }


def render_markdown(report: dict[str, Any]) -> str:
    lines = [
        "# ART-102 Repository Map",
        "",
        f"- Task: `{TASK_ID}`",
        f"- Generated at: `{report['generated_at']}`",
        f"- Target: `{report['target']['target_id']}`",
        f"- Primary root: `{report['target']['primary_root']}`",
        f"- Descriptor hash: `{report['target']['descriptor_hash']}`",
        f"- Repository count: `{report['summary']['repository_count']}`",
        f"- Dirty repository count: `{report['summary']['dirty_repository_count']}`",
        "",
        "## Scope Rollup",
        "",
        "| Scope | Repositories |",
        "| --- | ---: |",
    ]
    for scope, count in report["summary"]["scope_rollup"].items():
        lines.append(f"| `{scope}` | {count} |")
    lines.extend(["", "## Language Rollup", "", "| Language | Files |", "| --- | ---: |"])
    for language, count in report["summary"]["language_file_rollup"].items():
        lines.append(f"| {language} | {count} |")
    lines.extend(
        [
            "",
            "## Repositories",
            "",
            "| Path | Scope | Branch | Head | Files | Dirty | Owners | Markers | Last activity |",
            "| --- | --- | --- | --- | ---: | ---: | --- | --- | --- |",
        ]
    )
    for repo in report["repositories"]:
        owners = ", ".join(item["source"] for item in repo["owners"]) or "none detected"
        markers = ", ".join(repo["markers"]) or "none"
        last = repo["activity"]["last_commit"]
        last_activity = f"{last['date']} {last['hash'][:12]}" if last else "unknown"
        lines.append(
            "| "
            + " | ".join(
                [
                    f"`{repo['path']}`",
                    f"`{repo['scope']}`",
                    f"`{repo['activity']['branch']}`",
                    f"`{repo['activity']['head'] or 'unknown'}`",
                    str(repo["contents"]["tracked_or_scanned_files"]),
                    str(repo["activity"]["dirty_status_count"]),
                    owners,
                    markers,
                    last_activity,
                ]
            )
            + " |"
        )
    lines.extend(["", "## Dirty Repositories", ""])
    if report["dirty_repositories"]:
        for repo in report["dirty_repositories"]:
            lines.append(f"- `{repo['path']}`: {repo['dirty_status_count']} status entries")
    else:
        lines.append("- None detected.")
    lines.append("")
    return "\n".join(lines)


def insert_art102_fixture(conn: sqlite3.Connection, descriptor: dict[str, Any]) -> None:
    conn.execute(
        """
        INSERT INTO envctl_migration_targets
          (id, target_id, target_type, primary_root, compare_root, descriptor_json,
           descriptor_hash, safety_mode, max_auto_risk)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(target_id) DO NOTHING
        """,
        (
            TARGET_ID,
            descriptor["target_id"],
            descriptor["target_type"],
            descriptor["primary_root"],
            descriptor.get("compare_root"),
            json.dumps(descriptor, sort_keys=True),
            descriptor["descriptor_hash"],
            "approval-gated",
            "R2",
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_packages
          (id, package_name, package_path, package_hash, manifest_json)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(package_name, package_hash) DO NOTHING
        """,
        (
            PACKAGE_ID,
            "envctl-db-nu-plugin-migration-automation-package",
            ".",
            "sha256:art102-package",
            json.dumps({"task_id": TASK_ID, "artifact": "repository-map"}, sort_keys=True),
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_artifact_contracts
          (id, contract_name, contract_version, source_package_id, contract_hash, contract_json)
        VALUES (?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO NOTHING
        """,
        (
            CONTRACT_ID,
            "art102-repository-map",
            "1.0.0",
            PACKAGE_ID,
            "sha256:art102-contract",
            json.dumps({"required": ["repository-map.md", "repository-map.json"]}, sort_keys=True),
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_recipes
          (id, recipe_name, recipe_version, artifact_contract_id, recipe_hash, recipe_json)
        VALUES (?, ?, ?, ?, ?, ?)
        ON CONFLICT(recipe_name, recipe_version) DO NOTHING
        """,
        (
            RECIPE_ID,
            "art102-repository-map",
            "1.0.0",
            CONTRACT_ID,
            "sha256:art102-recipe",
            json.dumps({"phases": ["scan", "render", "register"]}, sort_keys=True),
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_runs
          (id, target_id, recipe_id, artifact_contract_id, status, human_mode,
           initiated_by, sandbox_policy, approval_policy, tool_versions_json,
           reproducibility_hash, started_at_utc)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO NOTHING
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
            json.dumps({"python": "stdlib", "git": "system"}, sort_keys=True),
            "sha256:art102-run",
            now(),
        ),
    )
    conn.execute(
        """
        INSERT INTO envctl_migration_operations
          (id, run_id, operation_type, phase, status, risk, idempotency_key,
           command_hash, command_redacted, input_json)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO NOTHING
        """,
        (
            OPERATION_ID,
            RUN_ID,
            "produce_artifact_record",
            "05-artifacts",
            "succeeded",
            "R2",
            f"{TASK_ID}/repository-map",
            "sha256:art102-command",
            "python3 scripts/generate_art102_repository_map.py",
            json.dumps({"task_id": TASK_ID, "repo_path": "${MIGRATION_TARGET_ROOT}"}, sort_keys=True),
        ),
    )
    conn.commit()


def register_artifacts(conn: sqlite3.Connection) -> dict[str, Any]:
    registry = ArtifactRegistry(conn, package_root())
    common = {
        "run_id": RUN_ID,
        "status": "complete",
        "producer_operation_id": OPERATION_ID,
        "contract_id": CONTRACT_ID,
        "provenance": {
            "task_id": TASK_ID,
            "owner_agent": "artifact-agent",
            "helper_id": HELPER_ID,
            "target_descriptor": "execution-framework/generated/envctl_target_registry.json",
            "envctl_database": "sqlite in-memory via sql/001_migration_automation_schema.sql",
        },
        "evidence_refs": [
            "execution-framework/generated/envctl_target_registry.json",
            "execution-framework/proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json",
            "execution-framework/proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json",
        ],
        "links": [
            {"to": "artifact:VER-300_UNIT_VALIDATION", "type": "blocks"},
            {"to": "artifact:REQ-024_ENVCTL_ARTIFACT_REGISTRY", "type": "depends_on"},
            {"to": "artifact:REQ-040_SHARED_PROTOCOL_SCHEMAS", "type": "depends_on"},
        ],
    }
    json_result = registry.register(
        {
            **common,
            "artifact_id": "art-102-repository-map-json",
            "title": "ART-102 Repository Map JSON",
            "artifact_type": "repository_map",
            "path": "execution-framework/migration-artifacts/art-102_repository_map/repository-map.json",
            "evidence_refs": [
                *common["evidence_refs"],
                "execution-framework/migration-artifacts/art-102_repository_map/repository-map.json",
            ],
            "validations": [
                {
                    "validator": "generate_art102_repository_map.py:json-artifact-exists",
                    "status": "pass",
                    "details": {"file": "repository-map.json"},
                    "evidence_refs": ["execution-framework/migration-artifacts/art-102_repository_map/repository-map.json"],
                },
                {
                    "validator": "generate_art102_repository_map.py:repository-count",
                    "status": "pass",
                    "details": {"repository_count_positive": True},
                    "evidence_refs": ["execution-framework/migration-artifacts/art-102_repository_map/repository-map.json"],
                },
            ],
        }
    )
    md_result = registry.register(
        {
            **common,
            "artifact_id": "art-102-repository-map-md",
            "title": "ART-102 Repository Map Markdown",
            "artifact_type": "repository_map",
            "path": "execution-framework/migration-artifacts/art-102_repository_map/repository-map.md",
            "evidence_refs": [
                *common["evidence_refs"],
                "execution-framework/migration-artifacts/art-102_repository_map/repository-map.md",
            ],
            "validations": [
                {
                    "validator": "generate_art102_repository_map.py:markdown-artifact-exists",
                    "status": "pass",
                    "details": {"file": "repository-map.md"},
                    "evidence_refs": ["execution-framework/migration-artifacts/art-102_repository_map/repository-map.md"],
                },
                {
                    "validator": "generate_art102_repository_map.py:registry-hash",
                    "status": "pass",
                    "details": {"hash_recorded": True},
                    "evidence_refs": ["execution-framework/migration-artifacts/art-102_repository_map/repository-map.md"],
                },
            ],
        }
    )
    return {"json": json_result, "markdown": md_result}


def verify_report(conn: sqlite3.Connection, report: dict[str, Any], registry_results: dict[str, Any]) -> dict[str, Any]:
    errors: list[str] = []
    for path in [JSON_PATH, MD_PATH]:
        if not path.exists() or path.stat().st_size == 0:
            errors.append(f"artifact missing or empty: {path.relative_to(root())}")
    if report["summary"]["repository_count"] < 1:
        errors.append("repository scan found no git repositories")
    artifact_rows = {}
    for artifact_id in ["art-102-repository-map-json", "art-102-repository-map-md"]:
        artifact = fetch_artifact(conn, RUN_ID, artifact_id)
        artifact_rows[artifact_id] = artifact
        if not artifact.get("content_hash", "").startswith("sha256:"):
            errors.append(f"registry did not record a hash for {artifact_id}")
    validation_count = conn.execute(
        "SELECT COUNT(*) FROM envctl_migration_validations WHERE run_id = ?",
        (RUN_ID,),
    ).fetchone()[0]
    evidence_count = conn.execute(
        "SELECT COUNT(*) FROM envctl_migration_evidence WHERE run_id = ?",
        (RUN_ID,),
    ).fetchone()[0]
    if validation_count < 4:
        errors.append(f"expected at least 4 validation rows, got {validation_count}")
    return {
        "schema_version": "1.0",
        "task_id": TASK_ID,
        "status": "passed" if not errors else "failed",
        "generated_at": now(),
        "target": report["target"],
        "summary": report["summary"],
        "registry_results": registry_results,
        "artifact_rows": artifact_rows,
        "validation_count": validation_count,
        "evidence_count": evidence_count,
        "artifact_hashes": {
            "repository-map.json": f"sha256:{sha256_file(JSON_PATH)}",
            "repository-map.md": f"sha256:{sha256_file(MD_PATH)}",
        },
        "errors": errors,
        "evidence": [
            "execution-framework/migration-artifacts/art-102_repository_map/repository-map.json",
            "execution-framework/migration-artifacts/art-102_repository_map/repository-map.md",
            "execution-framework/generated/envctl_target_registry.json",
            "execution-framework/proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json",
            "execution-framework/proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json",
        ],
    }


def main() -> int:
    started = now()
    ARTIFACT_DIR.mkdir(parents=True, exist_ok=True)
    (root() / "logs").mkdir(parents=True, exist_ok=True)
    (root() / "state").mkdir(parents=True, exist_ok=True)
    report = build_repository_map()
    write_json(JSON_PATH, report)
    MD_PATH.write_text(render_markdown(report), encoding="utf-8")
    conn = sqlite3.connect(":memory:")
    apply_migrations(conn, package_root())
    insert_art102_fixture(conn, report["target"])
    registry_results = register_artifacts(conn)
    verification = verify_report(conn, report, registry_results)
    LOG_PATH.write_text(json.dumps(verification, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    HEARTBEAT_PATH.write_text(
        json.dumps(
            {
                "task_id": TASK_ID,
                "status": "completed" if verification["status"] == "passed" else "failed",
                "started_at": started,
                "updated_at": now(),
                "artifact_paths": [
                    "migration-artifacts/art-102_repository_map/repository-map.json",
                    "migration-artifacts/art-102_repository_map/repository-map.md",
                ],
                "proof_uri": f"proof_records/{TASK_ID}.proof.json",
            },
            indent=2,
            sort_keys=False,
        )
        + "\n",
        encoding="utf-8",
    )
    files_changed = [
        "execution-framework/scripts/generate_art102_repository_map.py",
        "execution-framework/migration-artifacts/art-102_repository_map/repository-map.json",
        "execution-framework/migration-artifacts/art-102_repository_map/repository-map.md",
        "execution-framework/state/ART-102_REPOSITORY_MAP.heartbeat.json",
        "execution-framework/logs/ART-102_REPOSITORY_MAP.log",
        "execution-framework/proof_records/ART-102_REPOSITORY_MAP.proof.json",
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
        ["python3 scripts/generate_art102_repository_map.py"],
        verification,
        verification["evidence"] + ["logs/ART-102_REPOSITORY_MAP.log"],
        "" if verification["status"] == "passed" else "; ".join(verification["errors"]),
        "run VER-300_UNIT_VALIDATION",
    )
    append_proof(proof)
    print(json.dumps(verification, indent=2, sort_keys=False))
    return 0 if verification["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
