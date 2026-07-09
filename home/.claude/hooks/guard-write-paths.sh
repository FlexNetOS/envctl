#!/usr/bin/env bash
# guard-write-paths.sh — PreToolUse[Edit|Write|NotebookEdit]. Protected-path gate.
# Protects: ledger + harness runtime state, ~/.claude/archive, and the LIVE harness
# source (envctl main checkout). Sanctioned edit flow = the envctl WORKTREE.
# Exemption (operator decision 2026-07-09): $STATE_DIR/memory/*.md is the harness
# file-based memory contract — Write/Edit allowed there so durable operator
# directives can be saved. Ledger and the rest of STATE_DIR stay protected.
set -u
. "$(dirname "$0")/lib.sh"

INPUT=$(cat)
if have_jq; then
  FP=$(printf '%s' "$INPUT" | jq -r '.tool_input.file_path // .tool_input.notebook_path // empty' 2>/dev/null)
else
  FP=$(printf '%s' "$INPUT" | sed -n 's/.*"file_path"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)
fi
[ -n "${FP:-}" ] || exit 0

RES=$(readlink -f "$FP" 2>/dev/null || printf '%s' "$FP")

case "$RES" in
  "$STATE_DIR"/memory/*.md)
    # Memory contract carve-out: one fact per markdown file, index in MEMORY.md.
    exit 0 ;;
  "$LEDGER_DIR"/*|"$STATE_DIR"/*)
    ledger "guard.deny" "\"rule\":\"protected-runtime\",\"path\":\"$(json_escape "$RES")\""
    deny "Protected harness runtime state ($RES). The ledger is append-only via hooks; never edited." ;;
  "$ARCHIVE_ROOT"/*)
    ledger "guard.deny" "\"rule\":\"protected-archive\",\"path\":\"$(json_escape "$RES")\""
    deny "Archives are immutable (LAW 1). Never modify ~/.claude/archive contents." ;;
  /home/flexnetos/lifeos/src/envctl/home/.claude/*)
    ledger "guard.deny" "\"rule\":\"protected-live-harness\",\"path\":\"$(json_escape "$RES")\""
    deny "This is the LIVE harness source (envctl main checkout). Sanctioned flow: edit in the envctl worktree on develop, commit, then fast-forward the checkout." ;;
esac

exit 0
