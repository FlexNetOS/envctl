#!/usr/bin/env bash
# test-plan-loop-state.sh — proves the (extended) ci/gates/loop-state.sh enforces counter integrity
# for the planning-engineer plan-loop's state file `.handoff/loop/plan/loop_state.md` (same schema as
# the forge-loop's `.handoff/loop/loop_state.md`):
#   * PASS on a well-formed plan loop_state.md
#   * FAIL on a non-integer cycles_total
#   * FAIL on cycles_total < last_wrapup_total (negative boundary delta)
#   * FAIL on wrap_every: 0 (would fire a boundary every turn)
#   * FAIL on a cycles_total that REGRESSED vs the prior commit (monotonic, HEAD~1)
#   * SKIP cleanly when NEITHER forge nor plan loop_state.md exists (no false-block)
#   * (bonus) a well-formed FORGE loop_state.md still PASSes (the refactor kept the original path)
#
# Hermetic: a throwaway git repo carrying the REAL gate + a synthetic plan loop_state.md, run against
# each scenario, asserting exit status. No network, no real workspace touched.
set -euo pipefail

fail() { echo "FAIL: $*" >&2; exit 1; }
# Locate envctl's loop-state gate regardless of which of the two byte-identical copies is running
# (this file is mirrored into both envctl/scripts/tests/ and the harness_hub plugin). Walk up from
# this script to the meta-worktree root (the dir holding both envctl/ and harness_hub/) and descend.
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; root="$here"
while [ "$root" != "/" ] && [ ! -f "$root/envctl/ci/gates/loop-state.sh" ]; do root="$(dirname "$root")"; done
GATE="$root/envctl/ci/gates/loop-state.sh"
[ -f "$GATE" ] || GATE="$(git -C "$here" rev-parse --show-toplevel)/ci/gates/loop-state.sh"
[ -f "$GATE" ] || { echo "FAIL: ci/gates/loop-state.sh not found from $here" >&2; exit 1; }

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
gitc() { git -C "$tmp" -c user.email=t@example.com -c user.name=test -c commit.gpgsign=false "$@"; }

# throwaway repo carrying the real gate; the plan loop_state lives under .handoff/loop/plan/
git init -q "$tmp"
mkdir -p "$tmp/ci/gates" "$tmp/.handoff/loop/plan"
cp "$GATE" "$tmp/ci/gates/loop-state.sh"

PLAN="$tmp/.handoff/loop/plan/loop_state.md"
FORGE="$tmp/.handoff/loop/loop_state.md"
# write_plan <cycle_budget> <wrap_every> <last_wrapup_total> <cycles_total> <cycles_this_session>
# uses the planning-engineer schema (loop_state.template.md): extra plan-only keys are ignored by the gate.
write_plan() {
  cat > "$PLAN" <<EOF
# Loop state — planning-engineer (plan-loop) [synthetic test]
loop: planning-engineer
planning_target: secrets-proto
target_root: /tmp/fake/crates/secrets-proto
recency_window_days: 90
cycle_budget: $1   # comment survives the awk field-split
wrap_every: $2   # comment
last_wrapup_total: $3
cycles_this_session: $5
cycles_total: $4   # narration token $4
status: MAP pending
EOF
}

run_gate() { ( cd "$tmp" && bash ci/gates/loop-state.sh >/dev/null 2>&1 ); }

# 1. well-formed plan loop_state -> PASS; commit so HEAD~1 exists for the monotonic scenario
write_plan 3 5 18 18 1
gitc add -A >/dev/null; gitc commit -q -m "seed: plan cycles_total=18"
run_gate || fail "well-formed plan loop_state.md should PASS"

# 2. non-integer cycles_total -> FAIL
write_plan 3 5 18 "eighteen" 1
run_gate && fail "non-integer plan cycles_total should FAIL" || true

# 3. cycles_total < last_wrapup_total -> FAIL
write_plan 3 5 18 17 1
run_gate && fail "plan cycles_total < last_wrapup_total should FAIL" || true

# 4. wrap_every: 0 -> FAIL
write_plan 3 0 0 18 1
run_gate && fail "plan wrap_every=0 should FAIL" || true

# 5. monotonic regression: commit cycles_total=18 (done above), then a lower 12 -> FAIL (gate reads HEAD~1)
write_plan 3 5 5 12 1
gitc add -A >/dev/null; gitc commit -q -m "regress plan cycles_total to 12"
run_gate && fail "regressed plan cycles_total (18->12) should FAIL the monotonic check" || true

# 6. monotonic forward 18 -> 20 -> PASS
write_plan 3 5 18 20 1
gitc add -A >/dev/null; gitc commit -q -m "advance plan cycles_total to 20" || true
run_gate || fail "monotonic-forward plan cycles_total (18->20) should PASS"

# 7. neither forge nor plan loop_state -> SKIP (exit 0, no false-block)
rm -f "$PLAN" "$FORGE"
run_gate || fail "no loop_state.md at all should SKIP (exit 0), not fail"

# 8. (bonus) a well-formed FORGE loop_state.md still PASSes (original path intact post-refactor)
cat > "$FORGE" <<EOF
# Loop state (forge-loop) [synthetic test]
cycle_budget: 1
wrap_every: 5
last_wrapup_total: 0
cycles_this_session: 1
cycles_total: 4
EOF
run_gate || fail "well-formed FORGE loop_state.md should still PASS (refactor broke the original path)"

echo "test-plan-loop-state: PASS"
