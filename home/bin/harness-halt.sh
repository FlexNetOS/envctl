#!/usr/bin/env bash
# harness-halt.sh — the kill switch. Stops teams (tmux sweep), dispatched
# background sessions (supervisor jobs), background bash tasks, and prunes
# stale .claude/worktrees. Idempotent; prints what it did; ledgers the halt.
set -u
HARNESS_VAR="${HARNESS_VAR:-/home/flexnetos/lifeos/var}"
LEDGER="$HARNESS_VAR/log/claude-harness/ledger.jsonl"
TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
echo "== harness-halt $TS =="

# 1. Agent-team teammates: tmux panes/sessions created by Claude teams.
if command -v tmux >/dev/null 2>&1 && tmux ls >/dev/null 2>&1; then
  tmux ls -F '#{session_name}' 2>/dev/null | while read -r s; do
    case "$s" in
      *claude*|*team*|*teammate*)
        tmux kill-session -t "$s" && echo "killed tmux session: $s" ;;
      *) echo "left tmux session alone (not team-shaped): $s" ;;
    esac
  done
else
  echo "no tmux server running"
fi

# 2. Orphaned TEAM state only. LAW 3: never touch per-session task lists —
#    ~/.claude/tasks/<uuid> are live sessions' task lists (destroying them harms
#    parallel sessions). Only sweep ~/.claude/teams/<name> and the matching
#    ~/.claude/tasks/<name> (a team name is NOT a session UUID).
HTS=$(date -u +%Y%m%dT%H%M%SZ)
is_uuid() { printf '%s' "$1" | grep -Eq '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'; }
sweep() {
  local d="$1"
  local dest="$HOME/.claude/archive/$HTS/halt-sweep/$(basename "$d")"
  mkdir -p "$(dirname "$dest")"
  mv "$d" "$dest" 2>/dev/null && echo "archived orphaned team state: $d -> $dest"
}
for d in "$HOME/.claude/teams"/*; do
  [ -e "$d" ] || continue
  name=$(basename "$d")
  sweep "$d"
  # archive the team's task list too, but ONLY if its name is a team name (not a session uuid)
  if [ -e "$HOME/.claude/tasks/$name" ] && ! is_uuid "$name"; then
    sweep "$HOME/.claude/tasks/$name"
  fi
done
[ -n "$(ls -A "$HOME/.claude/teams" 2>/dev/null)" ] || echo "no team state to sweep (session task lists left untouched)"

# 3. Dispatched background sessions (claude agents supervisor jobs).
if [ -d "$HOME/.claude/jobs" ]; then
  for st in "$HOME/.claude/jobs"/*/state.json; do
    [ -f "$st" ] || continue
    PID=$(command -v jq >/dev/null 2>&1 && jq -r '.pid // empty' "$st" 2>/dev/null)
    if [ -n "${PID:-}" ] && kill -0 "$PID" 2>/dev/null; then
      kill "$PID" && echo "stopped dispatched session pid $PID ($(dirname "$st"))"
    fi
  done
else
  echo "no dispatched-session jobs dir"
fi

# 4. Background bash tasks (descendant shells of CC sessions). SCOPED: multiple
#    sanctioned parallel sessions run on this box (observed 2026-07-07), so
#    cross-session process kills require the explicit --all flag.
if [ "${1:-}" = "--all" ]; then
  pgrep -f 'claude' 2>/dev/null | while read -r cp; do
    for child in $(pgrep -P "$cp" -f 'bash|sh|cargo|npm' 2>/dev/null); do
      kill "$child" 2>/dev/null && echo "killed background task pid $child (parent CC $cp)"
    done
  done
else
  echo "process sweep skipped (pass --all to kill background tasks of EVERY CC session on this box)"
fi

# 5. Worktree hygiene: prune stale .claude/worktrees in known repos.
for repo in /home/flexnetos/lifeos/src/*/; do
  [ -d "$repo/.claude/worktrees" ] || continue
  git -C "$repo" worktree prune 2>/dev/null && echo "pruned worktrees in $repo"
done

printf '{"ts":"%s","event":"halt","by":"harness-halt.sh"}\n' "$TS" >>"$LEDGER" 2>/dev/null || true
echo "== halt complete =="
