#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CI_WORKFLOW="$ROOT/.github/workflows/ci.yml"
SYNC_WORKFLOW="$ROOT/.github/workflows/sync-master.yml"

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

# Preserve the branch-protection contexts while keeping required CI proof on
# GitHub-hosted clean runners. These jobs must remain GitHub-hosted fan-out.
for job_id in ["rustfmt", "clippy", "msrv", "cargo-audit", "test", "gates"]:
    body = job_block(job_id)
    require(
        re.search(r"(?m)^    runs-on:\s*ubuntu-latest\s*$", body) is not None,
        f"{job_id} must run on ubuntu-latest for parallel fan-out",
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
