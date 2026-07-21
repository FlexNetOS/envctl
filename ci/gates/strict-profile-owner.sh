#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-$(git rev-parse --show-toplevel)}"
ROOT="$(cd "$ROOT" && pwd -P)"

dot_local=".$(printf '%s' local)"
dot_codex=".$(printf '%s' codex)"
dot_claude=".$(printf '%s' claude)"

active_paths=(
  .agents
  .claude
  .codex
  AGENTS.md
  CLAUDE.md
  README.md
  agent-env.yaml
  agent-skills
  assets
  ci
  crates
  docs
  envctl-db-nu-plugin-migration-automation-package/execution-framework/docs
  home
  manifest
  packaging
  scripts
  systemd
)

is_projection_or_history() {
  case "$1" in
    .beads/*|.handoff/*|.kb/*|third_party/*|target/*|docs/generated/*) return 0 ;;
    */execution-framework/logs/*|*/migration-artifacts/*) return 0 ;;
    *) return 1 ;;
  esac
}

declare -a failures=()
while IFS= read -r -d '' path; do
  is_projection_or_history "$path" && continue
  [[ -e "$ROOT/$path" || -L "$ROOT/$path" ]] || continue
  case "$path" in
    home/"$dot_codex"/*|home/"$dot_claude"/*)
      failures+=("forbidden maintained path: $path")
      continue
      ;;
  esac

  [[ -f "$ROOT/$path" ]] || continue
  grep -Iq . "$ROOT/$path" 2>/dev/null || continue
  for pattern in \
    "$dot_local/" \
    "/home/flexnetos/$dot_codex" \
    "/home/flexnetos/$dot_claude" \
    "~/$dot_codex" \
    "~/$dot_claude" \
    "\$HOME/$dot_codex" \
    "\$HOME/$dot_claude" \
    "home/$dot_codex" \
    "home/$dot_claude"
  do
    if grep -Fq -- "$pattern" "$ROOT/$path"; then
      failures+=("forbidden maintained reference: $path :: $pattern")
    fi
  done
done < <(git -C "$ROOT" ls-files -z -- "${active_paths[@]}")

if ((${#failures[@]} > 0)); then
  printf '%s\n' "${failures[@]}" >&2
  printf 'strict profile owner gate: FAIL (%d findings)\n' "${#failures[@]}" >&2
  exit 1
fi

printf 'strict profile owner gate: PASS\n'
