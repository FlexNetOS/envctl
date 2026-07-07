#!/usr/bin/env bash
# notify-cosmic.sh — Notification hook → COSMIC desktop notification.
set -u
. "$(dirname "$0")/lib.sh"

INPUT=$(cat)
MSG="Claude Code needs attention"; KIND=""
if have_jq; then
  MSG=$(printf '%s' "$INPUT" | jq -r '.message // .notification // "Claude Code needs attention"' 2>/dev/null)
  KIND=$(printf '%s' "$INPUT" | jq -r '.notification_type // .matcher // empty' 2>/dev/null)
fi
ledger "notify" "\"kind\":\"$(json_escape "${KIND:-?}")\""
command -v notify-send >/dev/null 2>&1 && notify-send -a "Claude Code" "Claude Code" "$MSG" 2>/dev/null
exit 0
