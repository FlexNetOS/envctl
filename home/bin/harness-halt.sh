#!/usr/bin/env bash
# Stop Claude team jobs without treating a home directory as runtime authority.
set -euo pipefail

runtime_base="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/yazelix/profile-runtime/claude"
archive_root="$runtime_base/archive/$(date -u +%Y%m%dT%H%M%SZ)/halt-sweep"
ledger="${HARNESS_VAR:-/home/flexnetos/meta/var}/log/claude-harness/ledger.jsonl"
timestamp="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

is_uuid() {
  printf '%s' "$1" | grep -Eq '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
}

archive_team_state() {
  local source="$1" destination
  destination="$archive_root/$(basename "$source")"
  mkdir -p "$(dirname "$destination")"
  mv "$source" "$destination"
  printf 'archived team state: %s -> %s\n' "$source" "$destination"
}

if command -v tmux >/dev/null 2>&1 && tmux list-sessions >/dev/null 2>&1; then
  while IFS= read -r session; do
    case "$session" in
      *claude*|*team*|*teammate*) tmux kill-session -t "$session" ;;
    esac
  done < <(tmux list-sessions -F '#{session_name}')
fi

for team_dir in "$runtime_base/teams"/*; do
  [ -e "$team_dir" ] || continue
  team="$(basename "$team_dir")"
  archive_team_state "$team_dir"
  task_dir="$runtime_base/tasks/$team"
  if [ -e "$task_dir" ] && ! is_uuid "$team"; then
    archive_team_state "$task_dir"
  fi
done

for state in "$runtime_base/jobs"/*/state.json; do
  [ -f "$state" ] || continue
  pid="$(jq -r '.pid // empty' "$state" 2>/dev/null || true)"
  if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
    kill "$pid"
  fi
done

if [ "${1:-}" = "--all" ]; then
  while IFS= read -r parent; do
    while IFS= read -r child; do
      kill "$child" 2>/dev/null || true
    done < <(pgrep -P "$parent" -f 'bash|sh|cargo|bun' 2>/dev/null || true)
  done < <(pgrep -f 'claude' 2>/dev/null || true)
fi

mkdir -p "$(dirname "$ledger")"
printf '{"ts":"%s","event":"halt","by":"harness-halt.sh"}\n' "$timestamp" >>"$ledger"
