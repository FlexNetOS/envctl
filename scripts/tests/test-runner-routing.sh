#!/usr/bin/env bash
# test-runner-routing.sh — hermetic contract test for ci/gates/runner-routing.sh:
#   * PASS on the tracked disabled migration-source workflows (derivation covers every job)
#   * FAIL when a NEW job lacks the local-first labels/fork-guard/escape-hatch (the
#     derivation must catch jobs a hardcoded list never knew about)
#   * FAIL when a required-floor job disappears (rename/removal must update the gate)
#   * FAIL when ambient Cargo isolation is removed from the workflow or any job
# Uses the gate's RUNNER_ROUTING_CI/RUNNER_ROUTING_SYNC overrides; no network.
set -euo pipefail
root="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
GATE="$root/ci/gates/runner-routing.sh"
fail() { echo "FAIL: $*" >&2; exit 1; }
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT

# 1. real workflow files -> PASS
bash "$GATE" >/dev/null || fail "gate should PASS on the repo's real workflows"

# 2. a NEW job with bare runs-on escapes a literal list — derivation must FAIL it
cp "$root/.github/workflows_disabled/ci.yml" "$tmp/ci.yml"
printf '\n  sneaky-probe:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo probe\n' >> "$tmp/ci.yml"
if RUNNER_ROUTING_CI="$tmp/ci.yml" bash "$GATE" >/dev/null 2>&1; then
  fail "uncovered new job (sneaky-probe, bare runs-on) must FAIL the derived gate"
fi

# 3. removing a floor job (test) must FAIL, not silently shrink coverage
python3 - "$root/.github/workflows_disabled/ci.yml" "$tmp/ci-nofloor.yml" <<'PY'
import re, sys
t = open(sys.argv[1]).read()
t2 = re.sub(r"(?ms)^  test:\n.*?(?=^  [A-Za-z0-9_-]+:|\Z)", "", t, count=1)
assert t2 != t, "fixture: failed to remove the test job"
open(sys.argv[2], "w").write(t2)
PY
if RUNNER_ROUTING_CI="$tmp/ci-nofloor.yml" bash "$GATE" >/dev/null 2>&1; then
  fail "missing floor job (test) must FAIL the gate"
fi

# 4. Every workflow-level Cargo override is required: CI runs below META_ROOT and
# must not inherit optional host-local linker/compiler-wrapper settings from it.
for key in \
  CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER \
  CARGO_BUILD_RUSTC_WRAPPER \
  RUSTFLAGS
do
  python3 - "$root/.github/workflows_disabled/ci.yml" "$tmp/ci-no-$key.yml" "$key" <<'PY'
from pathlib import Path
import sys

source = Path(sys.argv[1])
destination = Path(sys.argv[2])
key = sys.argv[3]
lines = source.read_text().splitlines(keepends=True)
filtered = [line for line in lines if not line.startswith(f"  {key}:")]
assert len(filtered) == len(lines) - 1, f"fixture: expected one {key} line"
destination.write_text("".join(filtered))
PY
  if RUNNER_ROUTING_CI="$tmp/ci-no-$key.yml" bash "$GATE" >/dev/null 2>&1; then
    fail "missing hermetic Cargo override ($key) must FAIL the gate"
  fi
done

# 5. A new target directory per run/attempt/job prevents persistent self-hosted
# runners from reusing compiler artifacts across jobs or toolchain upgrades.
python3 - "$root/.github/workflows_disabled/ci.yml" "$tmp/ci-shared-target.yml" <<'PY'
from pathlib import Path
import sys

source = Path(sys.argv[1]).read_text()
needle = 'echo "CARGO_TARGET_DIR=$RUNNER_TEMP/envctl-target-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT-$GITHUB_JOB" >> "$GITHUB_ENV"'
assert source.count(needle) >= 1, "fixture: target-isolation step missing"
Path(sys.argv[2]).write_text(source.replace(needle, 'echo "CARGO_TARGET_DIR=$RUNNER_TEMP/envctl-target" >> "$GITHUB_ENV"', 1))
PY
if RUNNER_ROUTING_CI="$tmp/ci-shared-target.yml" bash "$GATE" >/dev/null 2>&1; then
  fail "shared Cargo target directory must FAIL the gate"
fi

echo "test-runner-routing: PASS"
