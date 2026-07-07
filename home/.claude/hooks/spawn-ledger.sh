#!/usr/bin/env bash
# spawn-ledger.sh — SubagentStart/SubagentStop/TaskCreated/TaskCompleted/TeammateIdle.
# Ledgers every event; maintains the active-agent counter; enforces two gates:
#   TaskCreated  → block tasks without an owner while a team is active
#   TeammateIdle → block idle while the teammate still owns in_progress tasks
set -u
. "$(dirname "$0")/lib.sh"

INPUT=$(cat)
EV=""
if have_jq; then
  EV=$(printf '%s' "$INPUT" | jq -r '.hook_event_name // empty' 2>/dev/null)
  SESS=$(printf '%s' "$INPUT" | jq -r '.session_id // empty' 2>/dev/null)
  export CLAUDE_SESSION_ID="${SESS:-${CLAUDE_SESSION_ID:-}}"
fi
[ -n "$EV" ] || EV="${1:-unknown}"

AGENT=$(printf '%s' "$INPUT" | { have_jq && jq -r '.agent_type // .agent_name // .teammate_name // empty' 2>/dev/null || true; } )

case "$EV" in
  SubagentStart)
    counter_bump +1
    ledger "subagent.start" "\"agent\":\"$(json_escape "${AGENT:-?}")\",\"active\":\"$(counter_get)\""
    ;;
  SubagentStop)
    counter_bump -1
    ledger "subagent.stop" "\"agent\":\"$(json_escape "${AGENT:-?}")\",\"active\":\"$(counter_get)\""
    ;;
  TaskCreated)
    OWNER=$(printf '%s' "$INPUT" | { have_jq && jq -r '.task.owner // .owner // empty' 2>/dev/null || true; })
    SUBJ=$(printf '%s' "$INPUT" | { have_jq && jq -r '.task.subject // .subject // empty' 2>/dev/null || true; })
    ledger "task.created" "\"owner\":\"$(json_escape "${OWNER:-}")\",\"subject\":\"$(json_escape "${SUBJ:-}")\""
    # Ownership gate only while a team is live (team config dir present)
    if [ -d "$HOME/.claude/teams" ] && [ -n "$(ls -A "$HOME/.claude/teams" 2>/dev/null)" ] && [ -z "${OWNER:-}" ]; then
      echo "Task rejected: team is active and this task has no owner. Assign an owner (file-ownership partitioning is mandatory)." >&2
      exit 2
    fi
    ;;
  TaskCompleted)
    ledger "task.completed" "\"agent\":\"$(json_escape "${AGENT:-}")\""
    ;;
  TeammateIdle)
    TM=$(printf '%s' "$INPUT" | { have_jq && jq -r '.teammate_name // .agent_name // empty' 2>/dev/null || true; })
    ledger "teammate.idle" "\"teammate\":\"$(json_escape "${TM:-?}")\""
    # Quality gate: no idling while owning in_progress tasks
    if [ -n "${TM:-}" ] && have_jq && [ -d "$HOME/.claude/tasks" ]; then
      OPEN=$(grep -rls "\"owner\"[[:space:]]*:[[:space:]]*\"$TM\"" "$HOME/.claude/tasks" 2>/dev/null \
        | xargs -r grep -l '"status"[[:space:]]*:[[:space:]]*"in_progress"' 2>/dev/null | head -1)
      if [ -n "$OPEN" ]; then
        echo "Idle rejected: you still own in_progress tasks. Finish them, update their status, and show real test/lint output before idling." >&2
        exit 2
      fi
    fi
    ;;
  *)
    ledger "hook.event" "\"name\":\"$(json_escape "$EV")\""
    ;;
esac
exit 0
