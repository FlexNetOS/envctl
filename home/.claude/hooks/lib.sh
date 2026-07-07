#!/usr/bin/env bash
# lib.sh — shared helpers for FlexNetOS Claude harness hooks.
# Contract: fast, idempotent, fail-closed for guards / fail-open for observers.
# Every hook appends exactly one JSON line to the append-only ledger.

HARNESS_VAR="${HARNESS_VAR:-/home/flexnetos/FlexNetOS/var}"
LEDGER_DIR="$HARNESS_VAR/log/claude-harness"
LEDGER="$LEDGER_DIR/ledger.jsonl"
STATE_DIR="$HARNESS_VAR/lib/claude-harness"
DECISIONS_DIR="$STATE_DIR/decisions"
RATE_CACHE="$STATE_DIR/rate-limits.json"
COUNTER="$STATE_DIR/active-agents.count"
BLOCK_FLAG="$STATE_DIR/budget-block.flag"
ARCHIVE_ROOT="$HOME/.claude/archive"

MAX_ACTIVE_AGENTS="${HARNESS_MAX_ACTIVE_AGENTS:-6}"
RATE_BLOCK_PCT="${HARNESS_RATE_BLOCK_PCT:-80}"

mkdir -p "$LEDGER_DIR" "$STATE_DIR" "$DECISIONS_DIR" 2>/dev/null || true

have_jq() { command -v jq >/dev/null 2>&1; }

# ledger <event> <json-fragment-without-braces>
ledger() {
  local ev="$1" extra="${2:-}"
  local ts
  ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  local line="{\"ts\":\"$ts\",\"event\":\"$ev\",\"session\":\"${CLAUDE_SESSION_ID:-${SESSION_ID:-unknown}}\""
  [ -n "$extra" ] && line="$line,$extra"
  line="$line}"
  ( flock -w 2 9 || true; printf '%s\n' "$line" >>"$LEDGER" ) 9>>"$LEDGER.lock" 2>/dev/null || \
    printf '%s\n' "$line" >>"$LEDGER" 2>/dev/null || true
}

# json_escape <string>  — minimal escaper for ledger fragments
json_escape() { printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g' | tr '\n\t' '  '; }

# deny <reason>  — PreToolUse permissionDecision deny, then exit 0
deny() {
  local reason="$1"
  local esc; esc=$(json_escape "$reason")
  printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"%s"}}\n' "$esc"
  exit 0
}

# ask <reason>  — escalate to the operator instead of hard deny
ask() {
  local reason="$1"
  local esc; esc=$(json_escape "$reason")
  printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"ask","permissionDecisionReason":"%s"}}\n' "$esc"
  exit 0
}

counter_get() { cat "$COUNTER" 2>/dev/null | tr -dc '0-9'; }

counter_bump() { # counter_bump +1|-1
  local d="$1" cur
  ( flock -w 2 8 || true
    cur=$(cat "$COUNTER" 2>/dev/null | tr -dc '0-9'); cur=${cur:-0}
    if [ "$d" = "+1" ]; then cur=$((cur+1)); else cur=$((cur>0 ? cur-1 : 0)); fi
    printf '%s' "$cur" >"$COUNTER"
  ) 8>>"$COUNTER.lock" 2>/dev/null || true
}

rate_pct_max() { # highest of five_hour/seven_day used_percentage from cache; empty if none
  [ -f "$RATE_CACHE" ] || { echo ""; return; }
  if have_jq; then
    jq -r '[.five_hour.used_percentage // 0, .seven_day.used_percentage // 0] | max | floor' "$RATE_CACHE" 2>/dev/null
  else
    grep -o '"used_percentage":[0-9.]*' "$RATE_CACHE" | cut -d: -f2 | sort -n | tail -1 | cut -d. -f1
  fi
}
