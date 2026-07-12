#!/usr/bin/env bash
# test-loop-state-gate.sh — proves ci/gates/loop-state.sh enforces forge-loop counter integrity:
#   * PASS on a well-formed loop_state.md (integers, cadence>=1, cycles_total>=last_wrapup_total)
#   * PASS on a planning-engineer markdown-table loop_state.md with the same required counters
#   * FAIL on a non-integer cycles_total
#   * FAIL on cycles_total < last_wrapup_total (negative boundary delta)
#   * FAIL on wrap_every: 0 (would fire a boundary every turn)
#   * FAIL on a cycles_total that REGRESSED vs the prior commit (monotonic)
#   * SKIP cleanly when no loop_state.md exists
#
# Hermetic: builds a throwaway git repo containing only ci/gates/loop-state.sh + a synthetic
# .handoff/loop/loop_state.md, runs the REAL gate against each scenario, asserts exit status. No
# network, no real workspace touched.
set -euo pipefail

REPO_ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
GATE="$REPO_ROOT/ci/gates/loop-state.sh"
[ -f "$GATE" ] || { echo "FAIL: $GATE missing" >&2; exit 1; }
fail() { echo "FAIL: $*" >&2; exit 1; }

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
gitc() { git -C "$tmp" -c user.email=t@example.com -c user.name=test -c commit.gpgsign=false "$@"; }

# throwaway repo carrying the real gate
git init -q "$tmp"
mkdir -p "$tmp/ci/gates" "$tmp/.handoff/loop"
cp "$GATE" "$tmp/ci/gates/loop-state.sh"

LS="$tmp/.handoff/loop/loop_state.md"
write_state() { # <cycle_budget> <wrap_every> <last_wrapup_total> <cycles_total> <cycles_this_session>
  cat > "$LS" <<EOF
# Loop state (synthetic test)
cycle_budget: $1   # comment survives
wrap_every: $2   # comment
last_wrapup_total: $3   # comment
cycles_this_session: $5
cycles_total: $4   # narration $4
EOF
}

run_gate() { ( cd "$tmp" && bash ci/gates/loop-state.sh >/dev/null 2>&1 ); }

# 1. well-formed -> PASS, committed so HEAD~1 exists for the monotonic scenario later
write_state 1 5 18 18 1
gitc add -A >/dev/null; gitc commit -q -m "seed: cycles_total=18"
run_gate || fail "well-formed loop_state.md should PASS"

# 2. non-integer cycles_total -> FAIL
write_state 1 5 18 "eighteen" 1
run_gate && fail "non-integer cycles_total should FAIL" || true

# 3. cycles_total < last_wrapup_total -> FAIL
write_state 1 5 18 17 1
run_gate && fail "cycles_total < last_wrapup_total should FAIL" || true

# 4. wrap_every: 0 -> FAIL
write_state 1 0 0 18 1
run_gate && fail "wrap_every=0 should FAIL" || true

# 5. monotonic: regress cycles_total 18 -> 12 vs the committed HEAD (=18) -> FAIL
write_state 1 5 5 12 1
gitc add -A >/dev/null; gitc commit -q -m "regress cycles_total to 12"
run_gate && fail "regressed cycles_total (18->12) should FAIL the monotonic check" || true

# 6. advance cycles_total 18 -> 20 -> PASS (monotonic forward)
write_state 1 5 18 20 1
gitc add -A >/dev/null; gitc commit -q -m "advance cycles_total to 20" || true
run_gate || fail "monotonic-forward cycles_total (18->20) should PASS"

# 7. planning-engineer table syntax with the same required counters -> PASS
PLAN_LS="$tmp/.handoff/loop/plan/loop_state.md"
mkdir -p "$(dirname "$PLAN_LS")"
cat > "$PLAN_LS" <<'EOF'
# Planning Engineer Loop — state

| key | value |
|---|---|
| run | synthetic-plan |
| cycles_this_session | 1 |
| cycle_budget | 1 |
| wrap_every | 1 |
| last_wrapup_total | 20 |
| cycles_total | 20 |
EOF
rm -f "$LS"
gitc add -A >/dev/null; gitc commit -q -m "add planning loop table state"
run_gate || fail "planning-engineer table loop_state.md with required counters should PASS"

# 8. planning table still fail-closes on missing required counters
python3 - <<PY
from pathlib import Path
p=Path('$PLAN_LS')
s=p.read_text().replace('| wrap_every | 1 |\n', '')
p.write_text(s)
PY
run_gate && fail "planning table missing wrap_every should FAIL" || true

# 9. no loop_state.md -> SKIP (exit 0)
rm -f "$PLAN_LS"
run_gate || fail "missing loop_state.md should SKIP (exit 0), not fail"

# 10. HANDOFF parity: handoff_cycles_total matching loop_state cycles_total -> PASS
write_state 1 5 18 20 1
cat > "$tmp/.handoff/loop/HANDOFF.md" <<'EOF'
# HANDOFF (synthetic)
handoff_cycles_total: 20   # comment survives
EOF
run_gate || fail "matching handoff_cycles_total should PASS"

# 11. HANDOFF parity: stale handoff_cycles_total != cycles_total -> FAIL
#     (the 2026-07-12 incident class: HANDOFF two boundaries behind loop_state, dead worktree path)
cat > "$tmp/.handoff/loop/HANDOFF.md" <<'EOF'
# HANDOFF (synthetic, stale)
handoff_cycles_total: 12
EOF
run_gate && fail "stale handoff_cycles_total (12 != 20) should FAIL" || true

# 12. legacy HANDOFF without the machine field -> PASS (skip note, never false-block)
cat > "$tmp/.handoff/loop/HANDOFF.md" <<'EOF'
# HANDOFF (legacy prose, no machine field)
Resume: whatever.
EOF
run_gate || fail "legacy HANDOFF without handoff_cycles_total should PASS (skipped)"

echo "test-loop-state-gate: PASS"
