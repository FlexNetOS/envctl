#!/usr/bin/env python3
"""Safely classify and sync a meta peer-repo fleet.

The script is intentionally conservative:
  * never commits, stashes, resets, rebases, merges, or force-pushes;
  * skips every dirty, diverged, no-upstream, or gone-upstream checkout;
  * only runs `git pull --ff-only` for clean behind-only repos;
  * only runs `git push` for clean ahead-only repos.
It also recognizes linked git worktrees during the fetch phase so shared repos
in the meta workspace are not silently skipped.

Use this after `scripts/reap-worktrees.sh --apply` when the workspace has lots of
intentional upgrade dirt and raw `meta exec -- git pull/push` would be too broad.
"""

from __future__ import annotations

import argparse
import json
import os
import pathlib
import subprocess
import sys
from collections import Counter
from dataclasses import asdict, dataclass, field
from datetime import datetime
from typing import Any


@dataclass
class GitResult:
    rc: int
    stdout: str = ""
    stderr: str = ""


@dataclass
class RepoState:
    name: str
    path: str
    repo: str | None = None
    exists: bool = False
    is_git: bool = False
    branch: str | None = None
    upstream: str | None = None
    upstream_track: str | None = None
    origin: str | None = None
    dirty: bool = False
    dirty_count: int = 0
    tracked_dirty_count: int = 0
    untracked_count: int = 0
    ahead: int | None = None
    behind: int | None = None
    bucket: str = "unknown"
    commands: list[str] = field(default_factory=list)
    errors: list[str] = field(default_factory=list)


def run(cmd: list[str], cwd: pathlib.Path, timeout: int = 90) -> GitResult:
    try:
        cp = subprocess.run(
            cmd,
            cwd=cwd,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=timeout,
        )
        return GitResult(cp.returncode, cp.stdout.strip(), cp.stderr.strip())
    except Exception as exc:  # pragma: no cover - defensive operational guard
        return GitResult(255, "", repr(exc))


def git(repo: pathlib.Path, *args: str, timeout: int = 90) -> GitResult:
    return run(["git", *args], repo, timeout=timeout)


def result_stdout(result: GitResult) -> str | None:
    if result.rc != 0:
        return None
    return result.stdout or None


def is_git_checkout(repo_path: pathlib.Path) -> bool:
    if not repo_path.exists():
        return False
    result = git(repo_path, "rev-parse", "--is-inside-work-tree")
    return result.rc == 0 and result.stdout == "true"


def load_project_list(meta_root: pathlib.Path, project_list_json: pathlib.Path | None) -> dict[str, Any]:
    if project_list_json:
        return json.loads(project_list_json.read_text())
    result = run(["meta", "project", "list", "--json"], meta_root)
    if result.rc != 0:
        raise SystemExit(f"meta project list --json failed:\n{result.stderr or result.stdout}")
    return json.loads(result.stdout)


def repos_from_project_list(project_list: dict[str, Any]) -> list[dict[str, str | None]]:
    repos: list[dict[str, str | None]] = [
        {"name": "meta", "path": ".", "repo": project_list.get("repo")}
    ]
    for project in project_list.get("projects", []):
        repos.append(
            {
                "name": project.get("name") or project.get("path"),
                "path": project["path"],
                "repo": project.get("repo"),
            }
        )
    return repos


def current_branch_track(repo_path: pathlib.Path, branch: str | None) -> str | None:
    if not branch:
        return None
    result = git(repo_path, "for-each-ref", "--format=%(upstream:track)", f"refs/heads/{branch}")
    return result_stdout(result)


def classify_repo(meta_root: pathlib.Path, repo_def: dict[str, str | None]) -> RepoState:
    rel = repo_def["path"] or "."
    state = RepoState(name=repo_def["name"] or rel, path=rel, repo=repo_def.get("repo"))
    repo_path = meta_root / rel
    state.exists = repo_path.exists()
    if not state.exists:
        state.bucket = "missing_skip"
        return state

    inside = git(repo_path, "rev-parse", "--is-inside-work-tree")
    if inside.rc != 0 or inside.stdout != "true":
        state.bucket = "not_git_skip"
        state.errors.append(inside.stderr or inside.stdout)
        return state

    state.is_git = True
    branch = git(repo_path, "branch", "--show-current")
    state.branch = result_stdout(branch)
    origin = git(repo_path, "remote", "get-url", "origin")
    state.origin = result_stdout(origin)
    upstream = git(repo_path, "rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}")
    state.upstream = result_stdout(upstream)
    state.upstream_track = current_branch_track(repo_path, state.branch)

    porcelain = git(repo_path, "status", "--porcelain=v1")
    if porcelain.rc != 0:
        state.bucket = "status_error_skip"
        state.errors.append(porcelain.stderr or porcelain.stdout)
        return state
    dirty_lines = [line for line in porcelain.stdout.splitlines() if line]
    state.dirty_count = len(dirty_lines)
    state.tracked_dirty_count = len([line for line in dirty_lines if not line.startswith("??")])
    state.untracked_count = len([line for line in dirty_lines if line.startswith("??")])
    state.dirty = state.dirty_count > 0

    if state.upstream:
        ab = git(repo_path, "rev-list", "--left-right", "--count", "HEAD...@{u}")
        if ab.rc == 0 and ab.stdout:
            parts = ab.stdout.split()
            if len(parts) == 2:
                state.ahead = int(parts[0])
                state.behind = int(parts[1])
        else:
            state.errors.append(ab.stderr or ab.stdout)

    if state.dirty:
        state.bucket = "dirty_skip"
    elif not state.upstream:
        state.bucket = "no_upstream_skip"
    elif state.upstream_track == "[gone]":
        state.bucket = "gone_upstream_skip"
    else:
        ahead = state.ahead or 0
        behind = state.behind or 0
        if ahead == 0 and behind == 0:
            state.bucket = "clean_synced"
        elif ahead == 0 and behind > 0:
            state.bucket = "safe_pull_ff"
            state.commands.append("git pull --ff-only")
        elif ahead > 0 and behind == 0:
            state.bucket = "safe_push"
            state.commands.append("git push")
        else:
            state.bucket = "diverged_skip"
    return state


def fetch_all(repo_path: pathlib.Path) -> GitResult:
    return git(repo_path, "fetch", "--all", "--prune", timeout=240)


def apply_bucket(repo_path: pathlib.Path, state: RepoState) -> None:
    if state.bucket == "safe_pull_ff":
        result = git(repo_path, "pull", "--ff-only", timeout=240)
    elif state.bucket == "safe_push":
        result = git(repo_path, "push", timeout=240)
    else:
        return
    if result.rc != 0:
        state.errors.append(result.stderr or result.stdout)
        state.bucket = f"{state.bucket}_failed"


def build_report(meta_root: pathlib.Path, states: list[RepoState], apply: bool, fetched: bool) -> dict[str, Any]:
    summary = Counter(state.bucket for state in states)
    return {
        "generated_at": datetime.now().isoformat(timespec="seconds"),
        "meta_root": str(meta_root),
        "apply": apply,
        "fetched": fetched,
        "total": len(states),
        "summary": dict(sorted(summary.items())),
        "repos": [asdict(state) for state in states],
    }


def print_text(report: dict[str, Any]) -> None:
    print(f"meta fleet sync report ({report['total']} repos)")
    print("summary:")
    for bucket, count in report["summary"].items():
        print(f"  {bucket}: {count}")
    for bucket in ("safe_pull_ff", "safe_push", "dirty_skip", "diverged_skip", "gone_upstream_skip", "no_upstream_skip", "missing_skip"):
        rows = [repo for repo in report["repos"] if repo["bucket"] == bucket]
        if not rows:
            continue
        print(f"\n{bucket}:")
        for repo in rows:
            detail = f"{repo['path']} branch={repo.get('branch')} upstream={repo.get('upstream')}"
            if repo.get("ahead") is not None or repo.get("behind") is not None:
                detail += f" ahead={repo.get('ahead')} behind={repo.get('behind')}"
            if repo.get("dirty_count"):
                detail += f" dirty={repo.get('dirty_count')} tracked={repo.get('tracked_dirty_count')} untracked={repo.get('untracked_count')}"
            if repo.get("commands"):
                detail += " :: " + " && ".join(repo["commands"])
            print(f"  {detail}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--meta-root", type=pathlib.Path, default=pathlib.Path.cwd())
    parser.add_argument("--project-list-json", type=pathlib.Path, help="Use a saved `meta project list --json` payload")
    parser.add_argument("--fetch", action="store_true", help="Run `git fetch --all --prune` in each repo before classification")
    parser.add_argument("--no-fetch", action="store_true", help="Do not fetch, even with --apply")
    parser.add_argument("--apply", action="store_true", help="Run safe pull/push actions; default is report-only")
    parser.add_argument("--json", action="store_true", help="Emit JSON only")
    parser.add_argument("--output", type=pathlib.Path, help="Write the report JSON to this path")
    args = parser.parse_args(argv)

    meta_root = args.meta_root.resolve()
    project_list = load_project_list(meta_root, args.project_list_json)
    repo_defs = repos_from_project_list(project_list)

    should_fetch = (args.fetch or args.apply) and not args.no_fetch
    if should_fetch:
        for repo_def in repo_defs:
            repo_path = meta_root / (repo_def["path"] or ".")
            if is_git_checkout(repo_path):
                fetch_all(repo_path)

    states = [classify_repo(meta_root, repo_def) for repo_def in repo_defs]

    if args.apply:
        for state in states:
            apply_bucket(meta_root / state.path, state)
        states = [classify_repo(meta_root, repo_def) for repo_def in repo_defs]

    report = build_report(meta_root, states, args.apply, should_fetch)
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, indent=2) + "\n")
    if args.json:
        print(json.dumps(report, indent=2))
    else:
        print_text(report)
        if args.output:
            print(f"\njson: {args.output}")
    failed = [state for state in states if state.bucket.endswith("_failed")]
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
