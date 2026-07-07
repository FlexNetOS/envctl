#!/usr/bin/env bash
# guard-bash.sh — PreToolUse[Bash]. Anti-recursion + anti-destruction gate.
# LAW 7: containment before capability. Fails closed on the patterns below.
set -u
. "$(dirname "$0")/lib.sh"

INPUT=$(cat)
if have_jq; then
  CMD=$(printf '%s' "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null)
  SESS=$(printf '%s' "$INPUT" | jq -r '.session_id // empty' 2>/dev/null)
else
  CMD=$(printf '%s' "$INPUT" | sed -n 's/.*"command"[[:space:]]*:[[:space:]]*"\(.*\)".*/\1/p' | head -1)
  SESS=""
fi
[ -n "${CMD:-}" ] || exit 0
export CLAUDE_SESSION_ID="${SESS:-${CLAUDE_SESSION_ID:-}}"

lc() { printf '%s' "$1" | tr '[:upper:]' '[:lower:]'; }
LCMD=$(lc "$CMD")

# ---- 1. bypassPermissions: never, unconditionally -------------------------
case "$LCMD" in
  *--dangerously-skip-permissions*)
    ledger "guard.deny" "\"rule\":\"dangerously-skip-permissions\",\"cmd\":\"$(json_escape "$CMD")\""
    deny "LAW: --dangerously-skip-permissions is forbidden on this machine, no exceptions." ;;
esac

# ---- 2. anti-recursion: no nested Claude sessions --------------------------
# Command-position match only: the claude binary as the first word of any
# pipeline segment (after stripping wrappers/env assignments). String args
# that merely contain the word do not trip it.
first_words() {
  printf '%s\n' "$1" | tr ';|&' '\n\n\n' | while read -r seg; do
    set -f; # shellcheck disable=SC2086
    set -- $seg; set +f
    while [ $# -gt 0 ]; do
      case "$1" in
        sudo|env|exec|command|nohup|setsid|nice|time|timeout|xargs) shift ;;
        [A-Za-z_][A-Za-z_0-9]*=*) shift ;;         # env assignments
        -*) shift ;;                                # wrapper flags (e.g. timeout 5)
        [0-9]*) shift ;;                            # timeout durations
        *) printf '%s\n' "$1"; break ;;
      esac
    done
  done
}
# Strip quoted spans first: content inside "..." or '...' can never be a command,
# and its separators (| ; &) must not be read as pipeline breaks. This kills the
# grep -E "a|claude|b" false-positive class.
CMDQ=$(printf '%s' "$CMD" | sed "s/\"[^\"]*\"/ /g; s/'[^']*'/ /g")
NESTED=""
while IFS= read -r w; do
  case "$w" in
    claude|*/claude|rtk) NESTED="$w" ;;
  esac
done <<EOF_FW
$(first_words "$CMDQ")
EOF_FW
# rtk is only nested-relevant when invoking the CC binary through it
if [ "$NESTED" = "rtk" ]; then
  printf '%s' "$CMDQ" | grep -Eq 'rtk[[:space:]]+claude([[:space:]]|$)' || NESTED=""
fi
if [ -n "$NESTED" ]; then
  if ! printf '%s' "$CMDQ" | grep -Eq '^[[:space:]]*(rtk[[:space:]]+)?claude[[:space:]]+(--version|update|doctor)[[:space:]]*$'; then
    ledger "guard.deny" "\"rule\":\"nested-claude\",\"cmd\":\"$(json_escape "$CMD")\""
    deny "Nested Claude sessions are denied (recursion containment; weave is not built yet, so there is no sanctioned wrapper). Allowed: 'claude --version', 'claude update', 'claude doctor'. Use teams/subagents/background bash instead."
  fi
fi

# ---- 3. rm of user data → archive flow ------------------------------------
# Scratch prefixes where real deletion is acceptable.
is_scratch() {
  case "$1" in
    /tmp/*|/var/tmp/*) return 0 ;;
    "$HOME"/.cache/*) return 0 ;;
    */target/*|*/target) return 0 ;;
    */node_modules/*|*/node_modules) return 0 ;;
    */.claude/worktrees/*) return 0 ;;
    "$HARNESS_VAR"/tmp/*|"$HARNESS_VAR"/cache/*) return 0 ;;
    *) return 1 ;;
  esac
}
if printf '%s' "$CMD" | grep -Eq '(^|[[:space:];&|])rm([[:space:]]|$)'; then
  bad=""
  # Scan ONLY the segment(s) that actually start with rm — split on separators
  # first so an unrelated segment (e.g. an echo string) can't be mis-parsed.
  while IFS= read -r seg; do
    # is this segment an rm invocation (optionally sudo-prefixed)?
    printf '%s' "$seg" | grep -Eq '^[[:space:]]*(sudo[[:space:]]+)?rm([[:space:]]|$)' || continue
    seen_rm=0
    # shellcheck disable=SC2086
    set -f; set -- $seg; set +f
    for tok in "$@"; do
      if [ "$seen_rm" = 0 ]; then
        case "$tok" in rm) seen_rm=1 ;; esac
        continue
      fi
      # skip flags, sudo, and redirections (2>/dev/null, >f, <f, 2>&1, &>f)
      case "$tok" in
        -*|sudo) continue ;;
        *'>'*|*'<'*|'&'*) continue ;;
        [0-9]) continue ;;                 # bare fd number preceding a redirect
      esac
      case "$tok" in
        /*|~*|"$HOME"*|./*|../*|[A-Za-z0-9_.]*)
          p="$tok"
          case "$p" in "~"*) p="$HOME${p#\~}";; esac
          case "$p" in /*) :;; *) p="$PWD/$p";; esac
          is_scratch "$p" || bad="$p" ;;
      esac
    done
  done <<EOF_RM
$(printf '%s' "$CMD" | tr ';|&' '\n\n\n')
EOF_RM
  if [ -n "$bad" ]; then
    ledger "guard.deny" "\"rule\":\"rm-user-data\",\"path\":\"$(json_escape "$bad")\",\"cmd\":\"$(json_escape "$CMD")\""
    deny "LAW 1: never delete, always archive. '$bad' is not a scratch path. Use: $HOME/.claude/hooks/harness-archive.sh <path> (moves it to ~/.claude/archive/<UTC>/rm-redirect/). rm stays allowed for /tmp, target/, node_modules/, .claude/worktrees, caches."
  fi
fi

# ---- 4. git topology protection --------------------------------------------
if printf '%s' "$LCMD" | grep -Eq '(^|[;&|[:space:]])git[[:space:]]'; then
  # branch -D: deny on long-lived, ask otherwise
  if printf '%s' "$CMD" | grep -Eq 'branch[[:space:]]+(-[a-zA-Z]*D[a-zA-Z]*)[[:space:]]'; then
    if printf '%s' "$CMD" | grep -Eq 'branch[[:space:]]+-[a-zA-Z]*D[a-zA-Z]*[[:space:]]+(main|master|develop)([[:space:]]|$)'; then
      ledger "guard.deny" "\"rule\":\"branch-D-longlived\",\"cmd\":\"$(json_escape "$CMD")\""
      deny "Topology invariant: main/master/develop are long-lived and never force-deleted."
    fi
    ledger "guard.ask" "\"rule\":\"branch-D\",\"cmd\":\"$(json_escape "$CMD")\""
    ask "git branch -D can destroy unmerged work (LAW 1). Prefer 'git branch -d' or archive the branch first. Approve only if you know the branch is disposable."
  fi
  # force push to long-lived branches: deny. Any other force push: ask.
  if printf '%s' "$CMD" | grep -Eq 'push[^;&|]*([[:space:]](--force|-f)([[:space:]]|$)|--force-with-lease|[[:space:]]\+[^[:space:]]+)'; then
    if printf '%s' "$CMD" | grep -Eq '(main|master|develop)'; then
      ledger "guard.deny" "\"rule\":\"force-push-longlived\",\"cmd\":\"$(json_escape "$CMD")\""
      deny "Force-push touching main/master/develop is forbidden (upgrade-only history)."
    fi
    ledger "guard.ask" "\"rule\":\"force-push\",\"cmd\":\"$(json_escape "$CMD")\""
    ask "Force-push rewrites remote history. Approve only for a private feature branch."
  fi
  # history rewrites and reflog destruction: deny
  if printf '%s' "$LCMD" | grep -Eq 'filter-branch|filter-repo|reflog[[:space:]]+expire.*--expire[=[:space:]]|gc[[:space:]].*--prune=now'; then
    ledger "guard.deny" "\"rule\":\"history-rewrite\",\"cmd\":\"$(json_escape "$CMD")\""
    deny "History rewrites / reflog destruction are forbidden (LAW 1 + upgrade-only). Archive instead."
  fi
  # hard reset: operator sign-off
  if printf '%s' "$LCMD" | grep -Eq 'reset[[:space:]]+--hard'; then
    ledger "guard.ask" "\"rule\":\"reset-hard\",\"cmd\":\"$(json_escape "$CMD")\""
    ask "git reset --hard discards local changes. Approve only if the working tree is disposable or archived."
  fi
fi

ledger "guard.pass" "\"tool\":\"Bash\""
exit 0
