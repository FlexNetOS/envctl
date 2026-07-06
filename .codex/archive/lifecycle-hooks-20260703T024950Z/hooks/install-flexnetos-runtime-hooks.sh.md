#!/usr/bin/env bash
# Install project-local Codex hooks for the active FlexNetOS work roots.
set -euo pipefail

repo_root_hook_template="/home/flexnetos/FlexNetOS/.codex/hooks/repo-root-lifecycle-hook.sh"
[ -x "$repo_root_hook_template" ] || chmod +x "$repo_root_hook_template"

install_one() {
  local project_root="$1"
  local codex_dir="$project_root/.codex"
  local hooks_json="$codex_dir/hooks.json"
  mkdir -p "$codex_dir/hooks"
  if [ -f "$hooks_json" ] && ! grep -Fq "repo-root-lifecycle-hook.sh" "$hooks_json"; then
    cp "$hooks_json" "$hooks_json.bak.$(date -u +%Y%m%dT%H%M%SZ)"
  fi
  install -D -m 755 "$repo_root_hook_template" "$codex_dir/hooks/repo-root-lifecycle-hook.sh"
  cp /home/flexnetos/FlexNetOS/.codex/hooks.json "$hooks_json"
  cat >"$codex_dir/FLEXNETOS_RUNTIME_GATES.md" <<EOF
# FlexNetOS Runtime Gates

This project-local Codex hook file now points at the repo-root lifecycle entrypoint:

  $codex_dir/hooks/repo-root-lifecycle-hook.sh

Reinstall with:

  bash /home/flexnetos/FlexNetOS/src/envctl/.codex/hooks/install-flexnetos-runtime-hooks.sh

Validate with:

  bash /home/flexnetos/FlexNetOS/.codex/hooks/flexnetos-codex-doctor.sh
EOF
  printf 'installed %s\n' "$hooks_json"
}

install_one /home/flexnetos/workspace
install_one /home/flexnetos/FlexNetOS
