#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# Overridable for the hermetic contract test (scripts/tests/test-runner-routing.sh) only.
CI_WORKFLOW="${RUNNER_ROUTING_CI:-$ROOT/.github/workflows/ci.yml}"
SYNC_WORKFLOW="${RUNNER_ROUTING_SYNC:-$ROOT/.github/workflows/sync-master.yml}"

python3 - "$CI_WORKFLOW" "$SYNC_WORKFLOW" <<'PY'
from pathlib import Path
import re
import sys

ci_path = Path(sys.argv[1])
sync_path = Path(sys.argv[2])
ci = ci_path.read_text()
sync = sync_path.read_text()
errors: list[str] = []


def require(condition: bool, message: str) -> None:
    if not condition:
        errors.append(message)


def job_block(job_id: str) -> str:
    match = re.search(
        rf"(?ms)^  {re.escape(job_id)}:\n(?P<body>.*?)(?=^  [A-Za-z0-9_-]+:|\Z)",
        ci,
    )
    if not match:
        errors.append(f"missing job: {job_id}")
        return ""
    return match.group("body")

# Operator directive 2026-07-09: LOCAL-FIRST CI IS MANDATORY. Required contexts
# (compile + test + gates) run on the FlexNetOS self-hosted runner fleet for every
# trusted ref (pushes and same-repo PRs). GitHub-hosted ubuntu-latest is FALLBACK
# ONLY: fork PRs (these repos are PUBLIC — untrusted code never executes on the
# local runners) and the CI_FORCE_HOSTED=1 repo-variable outage escape hatch.
LOCAL_LABELS = "fromJSON('[\"self-hosted\",\"linux\",\"x64\",\"local\",\"flexnetos\"]')"
FORK_GUARD = "github.event.pull_request.head.repo.full_name == github.repository"
ESCAPE_HATCH = "vars.CI_FORCE_HOSTED"
TARGET_ISOLATION = (
    'echo "CARGO_TARGET_DIR=$RUNNER_TEMP/envctl-target-'
    '$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT-$GITHUB_JOB" >> "$GITHUB_ENV"'
)

# CI checkouts live below META_ROOT on the self-hosted fleet. Cargo walks parent
# directories and therefore sees META_ROOT/.cargo/config.toml, whose optional
# Wild/Kache acceleration is host state rather than part of this repository's CI
# contract. These workflow-level overrides keep every job reproducible on both
# self-hosted and GitHub-hosted runners.
for setting in (
    "  CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER: cc",
    '  CARGO_BUILD_RUSTC_WRAPPER: ""',
    '  RUSTFLAGS: ""',
):
    require(setting in ci, f"ci.yml missing hermetic Cargo override: {setting.strip()}")

# Derive the job list from the workflow itself — a hardcoded literal let any NEW job
# escape the local-first/fork-guard/escape-hatch invariants (audit finding 2026-07-12).
# The floor set guards renames/removals: shrinking below it fails, never silently.
jobs_section = ci[ci.index("\njobs:") :]
derived_jobs = re.findall(r"(?m)^  ([A-Za-z0-9_-]+):\s*(?:#.*)?$", jobs_section)
REQUIRED_FLOOR = {"rustfmt", "clippy", "msrv", "cargo-audit", "test", "gates"}
missing_floor = REQUIRED_FLOOR - set(derived_jobs)
require(
    not missing_floor,
    f"required jobs missing from ci.yml (rename/removal must update the gate floor): {sorted(missing_floor)}",
)

for job_id in derived_jobs:
    body = job_block(job_id)
    runs_on = re.search(r"(?m)^    runs-on:\s*(?P<expr>.+)$", body)
    require(runs_on is not None, f"{job_id}: missing runs-on")
    if not runs_on:
        continue
    expr = runs_on.group("expr")
    require(
        LOCAL_LABELS in expr,
        f"{job_id} must target the local FlexNetOS runner labels first (local-first is mandatory)",
    )
    require(
        ESCAPE_HATCH in expr,
        f"{job_id} must keep the CI_FORCE_HOSTED hosted-fallback escape hatch",
    )
    require(
        FORK_GUARD in expr,
        f"{job_id} must route fork PRs to hosted runners (untrusted code never runs locally)",
    )
    require(
        "'ubuntu-latest'" in expr,
        f"{job_id} must declare the GitHub-hosted fallback lane",
    )
    require(
        TARGET_ISOLATION in body,
        f"{job_id} must isolate CARGO_TARGET_DIR by run, attempt, and job",
    )

# Stale PR runs should not keep either queue busy, but protected develop pushes must
# be allowed to finish because sync-master depends on their completed status.
require("concurrency:" in ci, "ci workflow must define concurrency")
require(
    "cancel-in-progress: ${{ github.ref != 'refs/heads/develop' }}" in ci,
    "ci workflow must cancel stale non-develop runs only",
)

# sync-master is a trusted maintenance workflow and should stay on the org runner.
require(
    "runs-on: [self-hosted, linux, x64, local, flexnetos]" in sync,
    "sync-master must stay on the FlexNetOS org self-hosted runner",
)

if errors:
    print("runner-routing gate failed:", file=sys.stderr)
    for error in errors:
        print(f"  - {error}", file=sys.stderr)
    sys.exit(1)
print("runner-routing gate passed")
PY
