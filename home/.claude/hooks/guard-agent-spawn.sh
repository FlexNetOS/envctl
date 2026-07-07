#!/usr/bin/env bash
# guard-agent-spawn.sh — PreToolUse[Agent]. Depth, fan-out and budget gate.
# Platform allows 5-level nesting since v2.1.172; this harness allows depth 1 only.
set -u
. "$(dirname "$0")/lib.sh"

INPUT=$(cat)
TRANSCRIPT=""; SESS=""
if have_jq; then
  TRANSCRIPT=$(printf '%s' "$INPUT" | jq -r '.transcript_path // empty' 2>/dev/null)
  SESS=$(printf '%s' "$INPUT" | jq -r '.session_id // empty' 2>/dev/null)
fi
export CLAUDE_SESSION_ID="${SESS:-${CLAUDE_SESSION_ID:-}}"

# Optional debug capture for hardening (enable: touch $STATE_DIR/guard-debug)
if [ -f "$STATE_DIR/guard-debug" ]; then
  printf '%s\n' "$INPUT" >>"$STATE_DIR/guard-agent-spawn.debug.jsonl" 2>/dev/null || true
fi

# --- depth gate: an Agent call issued from inside a subagent/teammate is denied.
# Primary discriminator (empirical 2026-07-07): subagent transcripts live under
# .../subagents/agent-<id>.jsonl. Env markers are NOT usable (AI_AGENT and
# CLAUDE_CODE_CHILD_SESSION are set in the main session too on this box).
if printf '%s' "$TRANSCRIPT" | grep -q '/subagents/'; then
  ledger "guard.deny" "\"rule\":\"agent-depth\",\"transcript\":\"$(json_escape "$TRANSCRIPT")\""
  deny "Recursion containment: subagents and teammates may not spawn further agents (depth-1 policy). Return your findings to the lead instead."
fi
# Harness-set fallback marker (exported by team/loop wrappers if ever used)
if [ "${HARNESS_AGENT_DEPTH:-0}" -ge 1 ] 2>/dev/null; then
  ledger "guard.deny" "\"rule\":\"agent-depth-env\",\"depth\":\"${HARNESS_AGENT_DEPTH}\""
  deny "Recursion containment: subagents and teammates may not spawn further agents (depth-1 policy). Return your findings to the lead instead."
fi

# --- fan-out gate: hard cap on simultaneously active agents (machine-global:
# the counter is shared across every session on this box — deliberate, this
# machine has runaway-fan-out history).
ACTIVE=$(counter_get); ACTIVE=${ACTIVE:-0}
if [ "$ACTIVE" -ge "$MAX_ACTIVE_AGENTS" ]; then
  ledger "guard.deny" "\"rule\":\"agent-cap\",\"active\":\"$ACTIVE\""
  deny "Runaway containment: $ACTIVE agents already active machine-wide (cap $MAX_ACTIVE_AGENTS). Wait for completions or run the kill switch (harness-halt.sh)."
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
