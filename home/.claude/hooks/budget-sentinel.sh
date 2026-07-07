#!/usr/bin/env bash
# budget-sentinel.sh — async PostToolUse observer. Tallies usage, warns at each
# 25% step of the rate-limit windows, sets the block flag at the ceiling.
# Enforcement (deny) happens in guard-agent-spawn.sh; this is the watchtower.
set -u
. "$(dirname "$0")/lib.sh"

cat >/dev/null  # drain stdin; async observer

PCT=$(rate_pct_max)
[ -n "$PCT" ] || exit 0

STEP_FILE="$STATE_DIR/budget-warned.step"
LAST=$(cat "$STEP_FILE" 2>/dev/null | tr -dc '0-9'); LAST=${LAST:-0}
STEP=$(( PCT / 25 * 25 ))

if [ "$STEP" -gt "$LAST" ] && [ "$STEP" -ge 25 ]; then
  printf '%s' "$STEP" >"$STEP_FILE"
  ledger "budget.step" "\"pct\":\"$PCT\",\"step\":\"$STEP\""
  command -v notify-send >/dev/null 2>&1 && \
    notify-send -u normal "Claude harness budget" "Rate-limit usage crossed ${STEP}% (now ${PCT}%)" 2>/dev/null
fi

if [ "$PCT" -ge "$RATE_BLOCK_PCT" ] 2>/dev/null && [ ! -f "$BLOCK_FLAG" ]; then
  touch "$BLOCK_FLAG"
  ledger "budget.block" "\"pct\":\"$PCT\""
  command -v notify-send >/dev/null 2>&1 && \
    notify-send -u critical "Claude harness budget" "CEILING: ${PCT}% ≥ ${RATE_BLOCK_PCT}% — new agent spawns are blocked" 2>/dev/null
fi
exit 0
