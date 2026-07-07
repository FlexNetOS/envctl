#!/usr/bin/env bash
# guard-agent-spawn.sh — PreToolUse[Agent]. Depth, fan-out and budget gate.
# Platform allows 5-level nesting since v2.1.172; this harness allows depth 1 only.
set -u
. "$(dirname "$0")/lib.sh"

INPUT=$(cat)

# --- depth gate: any Agent call issued from inside a subagent/teammate is denied.
# Discriminators (empirically verified on this box, see P1.6 drill):
#   - teammates/subagents run with CLAUDE_CODE_AGENT_* / teammate env markers, and
#   - the harness marks depth via HARNESS_AGENT_DEPTH exported in agent hook contexts.
DEPTH="${HARNESS_AGENT_DEPTH:-0}"
if [ -n "${CLAUDE_CODE_AGENT_NAME:-}${CLAUDE_AGENT_NAME:-}${CLAUDE_CODE_TEAMMATE:-}" ] || [ "$DEPTH" -ge 1 ]; then
  ledger "guard.deny" "\"rule\":\"agent-depth\",\"depth\":\"$DEPTH\""
  deny "Recursion containment: subagents and teammates may not spawn further agents (depth-1 policy). Return your findings to the lead instead."
fi

# --- fan-out gate: hard cap on simultaneously active agents.
ACTIVE=$(counter_get); ACTIVE=${ACTIVE:-0}
if [ "$ACTIVE" -ge "$MAX_ACTIVE_AGENTS" ]; then
  ledger "guard.deny" "\"rule\":\"agent-cap\",\"active\":\"$ACTIVE\""
  deny "Runaway containment: $ACTIVE agents already active (cap $MAX_ACTIVE_AGENTS). Wait for completions or run the kill switch (harness-halt.sh)."
fi

# --- budget gate: block at ${RATE_BLOCK_PCT}% of either rate-limit window.
if [ -f "$BLOCK_FLAG" ]; then
  ledger "guard.deny" "\"rule\":\"budget-flag\""
  deny "Budget sentinel: rate-limit ceiling reached (flag set). Operator must clear $BLOCK_FLAG after deciding how to proceed."
fi
PCT=$(rate_pct_max)
if [ -n "$PCT" ] && [ "$PCT" -ge "$RATE_BLOCK_PCT" ] 2>/dev/null; then
  touch "$BLOCK_FLAG"
  command -v notify-send >/dev/null 2>&1 && notify-send -u critical "Claude harness" "Rate limit ${PCT}% ≥ ${RATE_BLOCK_PCT}% — agent spawns blocked" 2>/dev/null
  ledger "guard.deny" "\"rule\":\"budget-rate\",\"pct\":\"$PCT\""
  deny "Budget sentinel: usage at ${PCT}% of a rate-limit window (ceiling ${RATE_BLOCK_PCT}%). Spawn blocked; ask the operator."
fi

ledger "guard.pass" "\"tool\":\"Agent\",\"active\":\"$ACTIVE\""
exit 0
