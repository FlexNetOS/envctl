#!/usr/bin/env bash
# stop-decision-gate.sh — Stop hook. Blocks completion while an operator decision
# is pending (marker files under $DECISIONS_DIR/*.pending). Respects the platform
# block cap via stop_hook_active. Emits exactly one clean question, no scaffold.
set -u
. "$(dirname "$0")/lib.sh"

INPUT=$(cat)
ACTIVE="false"
if have_jq; then
  ACTIVE=$(printf '%s' "$INPUT" | jq -r '.stop_hook_active // false' 2>/dev/null)
fi

PENDING=$(ls "$DECISIONS_DIR"/*.pending 2>/dev/null | head -1)
if [ -z "$PENDING" ]; then
  ledger "stop.pass" ""
  exit 0
fi

if [ "$ACTIVE" = "true" ]; then
  # Cap-respecting release: never loop. Surface loudly instead.
  ledger "stop.cap-release" "\"marker\":\"$(json_escape "$PENDING")\""
  command -v notify-send >/dev/null 2>&1 && \
    notify-send -u critical "Claude harness" "Session stopped with an UNANSWERED decision: $(basename "$PENDING")" 2>/dev/null
  exit 0
fi

# One clean question, scrubbed of anything that could be a scaffold marker.
Q=$(head -c 500 "$PENDING" | tr -d '\r' | sed 's/<[^>]*>//g' | tr '\n' ' ')
ledger "stop.block" "\"marker\":\"$(json_escape "$PENDING")\""
echo "An operator decision is pending and must be answered before stopping. Ask the operator exactly this one question via AskUserQuestion, then mark it answered by renaming the marker from .pending to .answered: ${Q}" >&2
exit 2
