#!/usr/bin/env bash
# compact-reinject.sh — PreCompact/PostCompact. Ledger + re-inject LAWS so they
# survive context compaction.
set -u
. "$(dirname "$0")/lib.sh"

INPUT=$(cat)
EV="Compact"
if have_jq; then EV=$(printf '%s' "$INPUT" | jq -r '.hook_event_name // "Compact"' 2>/dev/null); fi
ledger "compact" "\"phase\":\"$(json_escape "$EV")\""

LAWS="FlexNetOS LAWS still bind after compaction: never delete (archive via harness-archive.sh); upgrade-only; real execution only; terminal-only reporting; no nested Claude sessions; agents never spawn agents; max 6 active agents; budget ceiling 80%; decisions block via AskUserQuestion; only main/develop are long-lived branches."
ESC=$(json_escape "$LAWS")
printf '{"hookSpecificOutput":{"hookEventName":"%s","additionalContext":"%s"}}\n' "$EV" "$ESC"
exit 0
