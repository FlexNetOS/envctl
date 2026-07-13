#!/usr/bin/env bash
# test-runner-routing.sh — hermetic contract test for ci/gates/runner-routing.sh:
#   * PASS on the repo's real ci.yml + sync-master.yml (derivation covers every live job)
#   * FAIL when a NEW job lacks the local-first labels/fork-guard/escape-hatch (the
#     derivation must catch jobs a hardcoded list never knew about)
#   * FAIL when a required-floor job disappears (rename/removal must update the gate)
# Uses the gate's RUNNER_ROUTING_CI/RUNNER_ROUTING_SYNC overrides; no network.
set -euo pipefail
root="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
GATE="$root/ci/gates/runner-routing.sh"
fail() { echo "FAIL: $*" >&2; exit 1; }
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT

# 1. real workflow files -> PASS
bash "$GATE" >/dev/null || fail "gate should PASS on the repo's real workflows"

# 2. a NEW job with bare runs-on escapes a literal list — derivation must FAIL it
cp "$root/.github/workflows/ci.yml" "$tmp/ci.yml"
printf '\n  sneaky-probe:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo probe\n' >> "$tmp/ci.yml"
if RUNNER_ROUTING_CI="$tmp/ci.yml" bash "$GATE" >/dev/null 2>&1; then
  fail "uncovered new job (sneaky-probe, bare runs-on) must FAIL the derived gate"
fi

# 3. removing a floor job (test) must FAIL, not silently shrink coverage
python3 - "$root/.github/workflows/ci.yml" "$tmp/ci-nofloor.yml" <<'PY'
import re, sys
t = open(sys.argv[1]).read()
t2 = re.sub(r"(?ms)^  test:\n.*?(?=^  [A-Za-z0-9_-]+:|\Z)", "", t, count=1)
assert t2 != t, "fixture: failed to remove the test job"
open(sys.argv[2], "w").write(t2)
PY
if RUNNER_ROUTING_CI="$tmp/ci-nofloor.yml" bash "$GATE" >/dev/null 2>&1; then
  fail "missing floor job (test) must FAIL the gate"
fi

echo "test-runner-routing: PASS"
