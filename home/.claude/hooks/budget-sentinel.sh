#!/usr/bin/env bash
# budget-sentinel.sh — async PostToolUse observer. Tallies usage, warns at each
# 25% step of the rate-limit windows, sets the block flag at the ceiling.
# Enforcement (deny) happens in guard-agent-spawn.sh; this is the watchtower.
set -u
INPUT=$(cat)   # async observer; capture stdin for session_id
if command -v jq >/dev/null 2>&1; then
  SESS=$(printf '%s' "$INPUT" | jq -r '.session_id // empty' 2>/dev/null)
  export CLAUDE_SESSION_ID="${SESS:-${CLAUDE_SESSION_ID:-}}"
fi
. "$(dirname "$0")/lib.sh"

PCT=$(rate_pct_max)
[ -n "$PCT" ] || exit 0

# Session-scoped step tracking so parallel sessions don't clobber each other.
STEP_FILE="$STATE_DIR/budget-warned-${CLAUDE_SESSION_ID:-generic}.step"
LAST=$(cat "$STEP_FILE" 2>/dev/null | tr -dc '0-9'); LAST=${LAST:-0}
STEP=$(( PCT / 25 * 25 ))

if [ "$STEP" -gt "$LAST" ] && [ "$STEP" -ge 25 ]; then
  printf '%s' "$STEP" >"$STEP_FILE"
  ledger "budget.step" "\"pct\":\"$PCT\",\"step\":\"$STEP\""
  command -v notify-send >/dev/null 2>&1 && \
    notify-send -u normal "Claude harness budget" "Rate-limit usage crossed ${STEP}% (now ${PCT}%)" 2>/dev/null
fi

# At/above ceiling: notify once per step crossing (the hard block is enforced
# live in guard-agent-spawn from the same session-scoped cache — no shared flag).
if [ "$PCT" -ge "$RATE_BLOCK_PCT" ] 2>/dev/null && [ "$STEP" -gt "$LAST" ]; then
  ledger "budget.block" "\"pct\":\"$PCT\""
  command -v notify-send >/dev/null 2>&1 && \
    notify-send -u critical "Claude harness budget" "CEILING: ${PCT}% ≥ ${RATE_BLOCK_PCT}% — new agent spawns are blocked" 2>/dev/null
fi
exit 0
