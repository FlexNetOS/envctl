#!/usr/bin/env bash
# grit-advise.sh — PreToolUse[Edit|Write]. Parallel-code collision ADVISORY.
#
# grit locks AST `file::symbol` scopes. When ANOTHER agent holds a lock overlapping
# the code file about to be edited, surface it to the operator ('ask') so two agents
# don't clobber the same code in a shared checkout — the class of collision that
# worktree isolation + grit coordination exist to prevent.
#
# Contract: FAIL-OPEN + FAST. Any uncertainty (grit absent, repo not grit-enabled,
# not a git repo, not a code file, parse miss) -> allow (exit 0). Only engages in
# repos that opted in by having a `.grit/` dir at the repo root. Config files are
# never grit symbols, so JSON/TOML/etc. always pass through.
set -u
. "$(dirname "$0")/lib.sh"

INPUT=$(cat)
have_jq || exit 0
FP=$(printf '%s' "$INPUT" | jq -r '.tool_input.file_path // .tool_input.notebook_path // empty' 2>/dev/null)
[ -n "${FP:-}" ] || exit 0

DIR=$(dirname "$FP" 2>/dev/null); [ -d "$DIR" ] || exit 0
ROOT=$(git -C "$DIR" rev-parse --show-toplevel 2>/dev/null) || exit 0
[ -d "$ROOT/.grit" ] || exit 0                       # repo not opted into grit -> allow
command -v grit >/dev/null 2>&1 || exit 0            # coordinator absent -> allow

SELF="${CLAUDE_SESSION_ID:-claude-$$}"
REL=${FP#"$ROOT"/}

# grit status groups locks by agent:
#   * <agent> -- <intent>
#     | <file::symbol> (ts) [STATUS]
# Find a lock on REL::* held by an agent other than SELF (fail-open on any error).
HIT=$( (cd "$ROOT" && grit status 2>/dev/null) | awk -v self="$SELF" -v rel="$REL" '
  /^[*] / { agent=$2; next }
  index($0, rel "::") > 0 && agent != "" && agent != self {
    line=$0; sub(/^[[:space:]]*[|][[:space:]]*/, "", line)
    print agent " holds " line; exit
  }
' 2>/dev/null )

if [ -n "$HIT" ]; then
  ledger "grit.conflict" "\"path\":\"$(json_escape "$REL")\",\"hit\":\"$(json_escape "$HIT")\""
  ask "grit: another agent overlaps this file — $HIT. Run 'grit-claim.sh status' to coordinate; proceed only if you own this scope (a [EXPIRED] lock is stale and safe to override)."
fi
exit 0
