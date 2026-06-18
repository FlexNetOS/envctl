#!/usr/bin/env bash
# loop-state.sh — forge-loop counter-integrity gate. (TASK-0041, Epic G)
#
# The forge-loop's batch-boundary / hand-off / WRAP-UP-OWED logic all key off hand-edited
# integers in `.handoff/loop/loop_state.md` (cycles_total, last_wrapup_total, wrap_every,
# cycle_budget, cycles_this_session). `cycles_total` is reconstructed by free-text narration in a
# trailing comment, with no check that it stays a valid, monotonic integer. A non-integer or a
# silently-decreased counter would change whether a boundary or a hand-off fires — drift the hook
# (.claude/hooks/hf-checkpoint.sh) parses past silently.
#
# This gate makes the counters trustworthy. Fail-closed on what it can PROVE; never false-block:
#   1. cycle_budget, wrap_every, last_wrapup_total, cycles_total, cycles_this_session each parse as
#      a non-negative integer.
#   2. wrap_every >= 1 and cycle_budget >= 1 (a 0 would trigger a boundary/hand-off every turn).
#   3. cycles_total >= last_wrapup_total (the boundary delta can never be negative).
#   4. cycles_total is MONOTONIC (non-decreasing) vs the previous committed loop_state.md — checked
#      only when the prior version is readable (HEAD~1); if it is not (shallow clone / first commit /
#      file absent before), the check is SKIPPED with a note, never failed. Subset-of-truth, like
#      preflight: never stricter than what we can demonstrate.
#
# Read-only, zero-network, no toolchain. Sibling to ci/gates/{no-c,shape,enable,p7,agent-env}.sh.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

LS=".handoff/loop/loop_state.md"
if [ ! -f "$LS" ]; then
  echo "LOOP-STATE GATE SKIP — no $LS (not a forge-loop worktree)"
  exit 0
fi

# field <key> [file]  -> prints the value token after "<key>:" (strips trailing "# comment")
field() {
  awk -v k="$1:" '$1==k{print $2; exit}' "${2:-$LS}"
}

fail() { echo "LOOP-STATE GATE FAIL — $1" >&2; exit 1; }

is_uint() { case "$1" in ''|*[!0-9]*) return 1 ;; *) return 0 ;; esac; }

cycle_budget="$(field cycle_budget)"
wrap_every="$(field wrap_every)"
last_wrapup_total="$(field last_wrapup_total)"
cycles_total="$(field cycles_total)"
cycles_this_session="$(field cycles_this_session)"

# 1. every counter is a non-negative integer
for kv in \
  "cycle_budget=$cycle_budget" \
  "wrap_every=$wrap_every" \
  "last_wrapup_total=$last_wrapup_total" \
  "cycles_total=$cycles_total" \
  "cycles_this_session=$cycles_this_session"; do
  k="${kv%%=*}"; v="${kv#*=}"
  is_uint "$v" || fail "$k is not a non-negative integer (got: '${v:-<missing>}')"
done

# 2. cadence knobs must be >= 1 (a 0 fires a boundary/hand-off every turn)
[ "$wrap_every"   -ge 1 ] || fail "wrap_every must be >= 1 (got $wrap_every)"
[ "$cycle_budget" -ge 1 ] || fail "cycle_budget must be >= 1 (got $cycle_budget)"

# 3. boundary delta can never be negative
[ "$cycles_total" -ge "$last_wrapup_total" ] \
  || fail "cycles_total ($cycles_total) < last_wrapup_total ($last_wrapup_total) — negative boundary delta"

# 4. monotonic cycles_total vs the prior committed version (skip if unreadable — never false-block)
prev=""
if prev_file="$(git show HEAD~1:"$LS" 2>/dev/null)"; then
  prev="$(printf '%s\n' "$prev_file" | awk -v k="cycles_total:" '$1==k{print $2; exit}')"
fi
if [ -n "$prev" ] && is_uint "$prev"; then
  [ "$cycles_total" -ge "$prev" ] \
    || fail "cycles_total regressed: $prev (HEAD~1) -> $cycles_total (now) — counters must be monotonic"
  mono="monotonic ok ($prev -> $cycles_total)"
else
  mono="monotonic SKIPPED (no readable prior cycles_total)"
fi

echo "LOOP-STATE GATE PASS — budget=$cycle_budget wrap_every=$wrap_every last_wrapup=$last_wrapup_total cycles_total=$cycles_total session=$cycles_this_session; $mono"
