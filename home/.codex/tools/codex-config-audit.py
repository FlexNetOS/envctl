#!/usr/bin/env python3
"""Audit Codex configuration, hooks, rules, skills, and agent docs."""

from __future__ import annotations

import argparse
import csv
import json
import os
import re
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Any


COLUMNS = [
    "path",
    "file_type",
    "layer_scope",
    "repo_root",
    "git_remote",
    "git_branch",
    "is_tracked",
    "source_kind",
    "key_path",
    "value_summary",
    "risk_level",
    "risk_reason",
    "secret_like",
    "unsafe_sandbox_or_approval",
    "project_ignored_keys",
    "legacy_profile_format",
    "deprecated_keys",
    "hook_events",
    "hook_representation",
    "hook_targets",
    "mcp_servers",
    "relative_paths",
    "patch_file",
    "recommended_action",
]

EXCLUDED_DIRS = {
    ".beads",
    ".cache",
    ".git",
    ".hg",
    ".next",
    ".npm",
    ".pnpm-store",
    ".svn",
    ".tmp",
    ".venv",
    "__pycache__",
    "artifacts",
    "build",
    "coverage",
    "dist",
    "execution-reports",
    "node_modules",
    "result",
    "result-bin",
    "result-dev",
    "result-doc",
    "sessions",
    "target",
    "tmp",
    "var",
}

CONFIG_FILENAMES = {"config.toml", "hooks.json", "AGENTS.md", "AGENTS.override.md"}
PROJECT_IGNORED_KEYS = {
    "approval_policy",
    "auth",
    "disable_response_storage",
    "history",
    "model",
    "model_context_window",
    "model_provider",
    "model_reasoning_effort",
    "model_reasoning_summary",
    "mcp_servers",
    "providers",
    "sandbox_mode",
    "shell_environment_policy",
    "telemetry",
}
DEPRECATED_KEYS = {
    "features.codex_hooks",
}
UNSAFE_SANDBOX_VALUES = {"danger-full-access"}
UNSAFE_APPROVAL_VALUES = {"never"}
SECRET_RE = re.compile(
    r"(?i)(sk-[a-z0-9_-]{20,}|gh[pousr]_[a-z0-9_]{20,}|"
    r"xox[baprs]-[a-z0-9-]{20,}|api[_-]?key|access[_-]?token|"
    r"secret|password|passwd|private[_-]?key)"
)
PATH_KEY_RE = re.compile(r"(?i)(path|file|dir|cwd|command|script|hook|bin|source|target)")


def run_git(repo: Path, args: list[str]) -> str:
    try:
        completed = subprocess.run(
            ["git", "-C", str(repo), *args],
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError):
        return ""
    return completed.stdout.strip()


def find_repo(path: Path, cache: dict[Path, str]) -> str:
    cur = path if path.is_dir() else path.parent
    for parent in [cur, *cur.parents]:
        if parent in cache:
            return cache[parent]
        if (parent / ".git").exists():
            cache[parent] = str(parent)
            return str(parent)
    cache[cur] = ""
    return ""


def git_meta(path: Path, repo_cache: dict[Path, str], tracked_cache: dict[tuple[str, str], str]) -> dict[str, str]:
    repo = find_repo(path, repo_cache)
    if not repo:
        return {"repo_root": "", "git_remote": "", "git_branch": "", "is_tracked": ""}

    repo_path = Path(repo)
    rel = os.path.relpath(path, repo)
    tracked_key = (repo, rel)
    if tracked_key not in tracked_cache:
        tracked_cache[tracked_key] = "true" if run_git(repo_path, ["ls-files", "--error-unmatch", rel]) else "false"

    return {
        "repo_root": repo,
        "git_remote": run_git(repo_path, ["config", "--get", "remote.origin.url"]),
        "git_branch": run_git(repo_path, ["branch", "--show-current"]) or run_git(repo_path, ["rev-parse", "--short", "HEAD"]),
        "is_tracked": tracked_cache[tracked_key],
    }


def classify(path: Path, home: Path) -> tuple[str, str, str]:
    name = path.name
    text = str(path)
    if name == "config.toml" or (path.suffix == ".toml" and ".codex" in path.parts):
        file_type = "toml"
        source_kind = "config"
    elif name == "hooks.json":
        file_type = "json"
        source_kind = "hooks"
    elif name in {"AGENTS.md", "AGENTS.override.md"}:
        file_type = "markdown"
        source_kind = "agents"
    elif "/.codex/rules/" in text:
        file_type = path.suffix.lstrip(".") or "text"
        source_kind = "rules"
    elif "/.codex/skills/" in text:
        file_type = path.suffix.lstrip(".") or "text"
        source_kind = "skills"
    else:
        file_type = path.suffix.lstrip(".") or "text"
        source_kind = "other"

    if str(path).startswith(str(home / ".codex")):
        layer_scope = "user"
    elif str(path).startswith("/etc/codex"):
        layer_scope = "system"
    elif ".codex" in path.parts:
        layer_scope = "project"
    else:
        layer_scope = "docs"
    return file_type, layer_scope, source_kind


def interesting(path: Path) -> bool:
    if path.name in CONFIG_FILENAMES:
        return True
    text = str(path)
    if "/.codex/rules/" in text or "/.codex/skills/" in text:
        return path.is_file()
    if path.parent.name == ".codex" and path.suffix == ".toml":
        return True
    return False


def walk_files(roots: list[Path]) -> list[Path]:
    seen_roots: set[Path] = set()
    found: dict[Path, None] = {}
    for root in roots:
        if not root.exists():
            continue
        real = root.resolve()
        if real in seen_roots:
            continue
        seen_roots.add(real)
        if root.is_file():
            if interesting(root):
                found[root.resolve()] = None
            continue
        for dirpath, dirnames, filenames in os.walk(root):
            here = Path(dirpath)
            dirnames[:] = [
                d
                for d in dirnames
                if d not in EXCLUDED_DIRS and not (here / d).is_symlink()
            ]
            for filename in filenames:
                path = here / filename
                if interesting(path):
                    found[path.resolve()] = None
    return sorted(found)


def flatten(value: Any, prefix: str = "") -> list[tuple[str, Any]]:
    if isinstance(value, dict):
        rows: list[tuple[str, Any]] = []
        for key, child in value.items():
            key_path = f"{prefix}.{key}" if prefix else str(key)
            rows.extend(flatten(child, key_path))
        return rows
    if isinstance(value, list):
        if not value:
            return [(prefix, value)]
        rows = []
        for index, child in enumerate(value):
            rows.extend(flatten(child, f"{prefix}[{index}]"))
        return rows
    return [(prefix, value)]


def summarize(value: Any) -> str:
    if isinstance(value, str):
        if SECRET_RE.search(value):
            return "<redacted>"
        value = value.replace("\n", "\\n")
        return value[:120]
    if isinstance(value, (bool, int, float)) or value is None:
        return str(value).lower() if isinstance(value, bool) else str(value)
    return f"<{type(value).__name__}>"


def relative_path_for(key_path: str, value: Any) -> str:
    if not isinstance(value, str):
        return ""
    if not PATH_KEY_RE.search(key_path):
        return ""
    if value.startswith(("http://", "https://", "$", "~", "/")):
        return ""
    if value in {"bash", "sh", "zsh", "python", "python3", "node", "bun", "bunx", "nix", "git"}:
        return ""
    if "/" in value or value.startswith("."):
        return value
    return ""


def hook_info(data: Any) -> tuple[str, str, str]:
    if not isinstance(data, dict):
        return "", "none", ""
    hooks = data.get("hooks") if "hooks" in data else data
    if not isinstance(hooks, dict):
        return "", "none", ""
    events = sorted(str(key) for key in hooks.keys())
    targets: list[str] = []
    for event_value in hooks.values():
        for key_path, value in flatten(event_value):
            if isinstance(value, str) and ("command" in key_path.lower() or "/" in value):
                targets.append(value[:120])
    return ",".join(events), "inline" if "hooks" in data else "hooks_json", "|".join(sorted(set(targets))[:25])


def mcp_servers(data: Any) -> str:
    if not isinstance(data, dict):
        return ""
    servers = data.get("mcp_servers")
    if not isinstance(servers, dict):
        return ""
    return ",".join(sorted(str(key) for key in servers.keys()))


def base_row(path: Path, home: Path, repo_cache: dict[Path, str], tracked_cache: dict[tuple[str, str], str]) -> dict[str, str]:
    file_type, layer_scope, source_kind = classify(path, home)
    row = {column: "" for column in COLUMNS}
    row.update(
        {
            "path": str(path),
            "file_type": file_type,
            "layer_scope": layer_scope,
            "source_kind": source_kind,
            "risk_level": "info",
            "recommended_action": "review context",
        }
    )
    row.update(git_meta(path, repo_cache, tracked_cache))
    return row


def risk_for(layer_scope: str, key_path: str, value: Any, text: str) -> tuple[str, str, str, str, str, str]:
    secret_like = "true" if SECRET_RE.search(key_path) or (isinstance(value, str) and SECRET_RE.search(value)) else "false"
    unsafe = "false"
    risk = "info"
    reason = ""
    project_ignored = ""
    deprecated = ""
    rel = relative_path_for(key_path, value)

    if key_path.endswith("sandbox_mode") and str(value) in UNSAFE_SANDBOX_VALUES:
        unsafe = "true"
        risk = "high"
        reason = "unsafe sandbox default"
    if key_path.endswith("approval_policy") and str(value) in UNSAFE_APPROVAL_VALUES:
        unsafe = "true"
        risk = "high"
        reason = "unsafe approval policy"
    if secret_like == "true":
        risk = "high" if layer_scope == "project" else max_risk(risk, "medium")
        reason = join_reason(reason, "secret-like content")
    if layer_scope == "project":
        root_key = key_path.split(".", 1)[0].split("[", 1)[0]
        if root_key in PROJECT_IGNORED_KEYS:
            project_ignored = root_key
            risk = max_risk(risk, "medium")
            reason = join_reason(reason, "project config key is ignored or user-only")
    if key_path in DEPRECATED_KEYS:
        deprecated = key_path
        risk = max_risk(risk, "medium")
        reason = join_reason(reason, "deprecated key")
    if rel:
        risk = max_risk(risk, "low")
        reason = join_reason(reason, "relative path")
    if "danger-full-access" in text or "approval_policy = \"never\"" in text:
        if risk == "info":
            risk = "medium"
        reason = join_reason(reason, "unsafe policy text present")
    return risk, reason, secret_like, unsafe, project_ignored, deprecated


def join_reason(existing: str, extra: str) -> str:
    if not existing:
        return extra
    if extra in existing.split("; "):
        return existing
    return f"{existing}; {extra}"


def max_risk(left: str, right: str) -> str:
    order = {"info": 0, "low": 1, "medium": 2, "high": 3}
    return left if order[left] >= order[right] else right


def audit_toml(path: Path, home: Path, repo_cache: dict[Path, str], tracked_cache: dict[tuple[str, str], str]) -> list[dict[str, str]]:
    row0 = base_row(path, home, repo_cache, tracked_cache)
    text = path.read_text(errors="replace")
    try:
        data = tomllib.loads(text)
    except tomllib.TOMLDecodeError as exc:
        row = dict(row0)
        row.update(
            {
                "key_path": "<parse>",
                "risk_level": "high",
                "risk_reason": f"TOML parse error: {exc}",
                "recommended_action": "fix TOML syntax before relying on this config",
            }
        )
        return [row]

    hook_events, hook_representation, hook_targets = hook_info(data)
    servers = mcp_servers(data)
    legacy = "true" if isinstance(data.get("profiles"), dict) else "false"
    rows = []
    for key_path, value in flatten(data):
        row = dict(row0)
        risk, reason, secret, unsafe, project_ignored, deprecated = risk_for(row0["layer_scope"], key_path, value, text)
        row.update(
            {
                "key_path": key_path,
                "value_summary": summarize(value),
                "risk_level": risk,
                "risk_reason": reason,
                "secret_like": secret,
                "unsafe_sandbox_or_approval": unsafe,
                "project_ignored_keys": project_ignored,
                "legacy_profile_format": legacy,
                "deprecated_keys": deprecated,
                "hook_events": hook_events,
                "hook_representation": hook_representation,
                "hook_targets": hook_targets,
                "mcp_servers": servers,
                "relative_paths": relative_path_for(key_path, value),
                "recommended_action": recommendation(risk, reason),
            }
        )
        rows.append(row)
    return rows or [row0]


def audit_json(path: Path, home: Path, repo_cache: dict[Path, str], tracked_cache: dict[tuple[str, str], str]) -> list[dict[str, str]]:
    row0 = base_row(path, home, repo_cache, tracked_cache)
    text = path.read_text(errors="replace")
    try:
        data = json.loads(text)
    except json.JSONDecodeError as exc:
        row = dict(row0)
        row.update(
            {
                "key_path": "<parse>",
                "risk_level": "high",
                "risk_reason": f"JSON parse error: {exc}",
                "recommended_action": "fix JSON syntax before relying on this hooks file",
            }
        )
        return [row]

    hook_events, hook_representation, hook_targets = hook_info(data)
    rows = []
    for key_path, value in flatten(data):
        row = dict(row0)
        risk, reason, secret, unsafe, project_ignored, deprecated = risk_for(row0["layer_scope"], key_path, value, text)
        row.update(
            {
                "key_path": key_path,
                "value_summary": summarize(value),
                "risk_level": risk,
                "risk_reason": reason,
                "secret_like": secret,
                "unsafe_sandbox_or_approval": unsafe,
                "project_ignored_keys": project_ignored,
                "deprecated_keys": deprecated,
                "hook_events": hook_events,
                "hook_representation": hook_representation,
                "hook_targets": hook_targets,
                "relative_paths": relative_path_for(key_path, value),
                "recommended_action": recommendation(risk, reason),
            }
        )
        rows.append(row)
    return rows or [row0]


def audit_text(path: Path, home: Path, repo_cache: dict[Path, str], tracked_cache: dict[tuple[str, str], str]) -> list[dict[str, str]]:
    row = base_row(path, home, repo_cache, tracked_cache)
    text = path.read_text(errors="replace")
    secret = "true" if SECRET_RE.search(text) else "false"
    unsafe = "true" if "danger-full-access" in text or "approval_policy = \"never\"" in text else "false"
    risk = "info"
    reason = ""
    if secret == "true":
        risk = "medium"
        reason = "secret-like content"
    if unsafe == "true":
        risk = max_risk(risk, "medium")
        reason = join_reason(reason, "unsafe policy text present")
    row.update(
        {
            "key_path": "<document>",
            "value_summary": f"{len(text)} bytes",
            "risk_level": risk,
            "risk_reason": reason,
            "secret_like": secret,
            "unsafe_sandbox_or_approval": unsafe,
            "recommended_action": recommendation(risk, reason),
        }
    )
    return [row]


def recommendation(risk: str, reason: str) -> str:
    if risk == "high":
        return "repair or remove before trusting this Codex surface"
    if risk == "medium":
        return "review and patch if this surface is active"
    if risk == "low":
        return "normalize if the path is active or relocatable"
    return "no immediate action"


def audit(paths: list[Path], home: Path) -> list[dict[str, str]]:
    repo_cache: dict[Path, str] = {}
    tracked_cache: dict[tuple[str, str], str] = {}
    rows: list[dict[str, str]] = []
    for path in paths:
        try:
            if path.name == "hooks.json":
                rows.extend(audit_json(path, home, repo_cache, tracked_cache))
            elif path.suffix == ".toml":
                rows.extend(audit_toml(path, home, repo_cache, tracked_cache))
            else:
                rows.extend(audit_text(path, home, repo_cache, tracked_cache))
        except OSError as exc:
            row = base_row(path, home, repo_cache, tracked_cache)
            row.update(
                {
                    "key_path": "<read>",
                    "risk_level": "medium",
                    "risk_reason": f"read error: {exc}",
                    "recommended_action": "inspect filesystem permissions",
                }
            )
            rows.append(row)
    return rows


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", action="append", default=[], help="root to scan; may be repeated")
    parser.add_argument("--csv", required=True, help="CSV output path")
    parser.add_argument("--json", required=True, help="JSON output path")
    args = parser.parse_args()

    home = Path.home().resolve()
    roots = [Path(root).expanduser() for root in args.root] or [home, Path("/home/flexnetos")]
    paths = walk_files(roots)
    rows = audit(paths, home)

    csv_path = Path(args.csv).expanduser()
    json_path = Path(args.json).expanduser()
    csv_path.parent.mkdir(parents=True, exist_ok=True)
    json_path.parent.mkdir(parents=True, exist_ok=True)

    with csv_path.open("w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=COLUMNS, extrasaction="ignore")
        writer.writeheader()
        writer.writerows(rows)
    with json_path.open("w") as f:
        json.dump(rows, f, indent=2, sort_keys=True)
        f.write("\n")

    counts: dict[str, int] = {}
    for row in rows:
        counts[row["risk_level"]] = counts.get(row["risk_level"], 0) + 1
    print(json.dumps({"files": len(paths), "rows": len(rows), "risk_counts": counts}, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
