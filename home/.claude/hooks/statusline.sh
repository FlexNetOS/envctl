#!/usr/bin/env bash
# statusline.sh — statusline command. Shows live model / effort / context / cost /
# rate-limit windows / active agent counts. Side effects: caches rate_limits for
# the budget sentinel; one-shot desktop alert if the model was rerouted off Fable.
set -u
HARNESS_VAR="${HARNESS_VAR:-/home/flexnetos/meta/var}"
STATE_DIR="$HARNESS_VAR/lib/claude-harness"
mkdir -p "$STATE_DIR" 2>/dev/null || true

INPUT=$(cat)
command -v jq >/dev/null 2>&1 || { echo "harness"; exit 0; }

SESSION_ID=$(printf '%s' "$INPUT" | jq -r '.session_id // empty')
MODEL_ID=$(printf '%s' "$INPUT" | jq -r '.model.id // "?"')
MODEL=$(printf '%s' "$INPUT" | jq -r '.model.display_name // .model.id // "?"')
EFFORT=$(printf '%s' "$INPUT" | jq -r '.effort.level // "-"')
CTX=$(printf '%s' "$INPUT" | jq -r '.context_window.used_percentage // empty' | cut -d. -f1)
COST=$(printf '%s' "$INPUT" | jq -r '.cost.total_cost_usd // empty')
R5=$(printf '%s' "$INPUT" | jq -r '.rate_limits.five_hour.used_percentage // empty' | cut -d. -f1)
R7=$(printf '%s' "$INPUT" | jq -r '.rate_limits.seven_day.used_percentage // empty' | cut -d. -f1)

# cache rate limits for the sentinel/guards — SESSION-SCOPED so parallel
# sessions never read each other's (or a stale) rate value.
if [ -n "$SESSION_ID" ]; then RCACHE="$STATE_DIR/rate-limits-$SESSION_ID.json"; else RCACHE="$STATE_DIR/rate-limits.json"; fi
printf '%s' "$INPUT" | jq -c '.rate_limits // {}' >"$RCACHE" 2>/dev/null || true

# counts
AG=$(cat "$STATE_DIR/active-agents.count" 2>/dev/null | tr -dc '0-9'); AG=${AG:-0}
TM=0
[ -d "$HOME/.claude/teams" ] && TM=$(find "$HOME/.claude/teams" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | wc -l)
BG=0
command -v tmux >/dev/null 2>&1 && BG=$(tmux ls 2>/dev/null | wc -l)

# reroute alarm (one-shot per session). SESSION-SCOPED like the rate cache
# above: a global flag let a Fable-context render (rm below) clear the latch
# that an Opus-context render set, so notify-send re-fired on every repaint
# across parallel sessions. Per-session flag = at most one alert per genuinely
# rerouted session, no cross-session churn.
if [ -n "$SESSION_ID" ]; then FLAG="$STATE_DIR/reroute-alerted-$SESSION_ID"; else FLAG="$STATE_DIR/reroute-alerted"; fi
case "$MODEL_ID" in
  claude-fable-5*) rm -f "$FLAG" 2>/dev/null; BADGE="" ;;
  *)
    BADGE="⚠ REROUTED→${MODEL} "
    if [ ! -f "$FLAG" ]; then
      touch "$FLAG"
      command -v notify-send >/dev/null 2>&1 && \
        notify-send -u critical "Claude harness" "Model rerouted off Fable → $MODEL. Statusline flagged; decide before continuing (/model fable to return)." 2>/dev/null
    fi ;;
esac

OUT="${BADGE}${MODEL} e:${EFFORT}"
[ -n "$CTX" ] && OUT="$OUT ctx:${CTX}%"
[ -n "$COST" ] && OUT="$OUT \$$(printf '%.2f' "$COST" 2>/dev/null || echo "$COST")"
[ -n "$R5" ] && OUT="$OUT 5h:${R5}%"
[ -n "$R7" ] && OUT="$OUT 7d:${R7}%"
OUT="$OUT ag:${AG} tm:${TM} tx:${BG}"
[ -f "$STATE_DIR/budget-block.flag" ] && OUT="$OUT [BUDGET-BLOCKED]"
printf '%s\n' "$OUT"
