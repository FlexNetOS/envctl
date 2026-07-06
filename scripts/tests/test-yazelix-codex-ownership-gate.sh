#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

fail() {
  printf 'FAIL: %s\n' "$*" >&2
  exit 1
}

assert_not_contains() {
  local path="$1"
  local pattern="$2"
  local why="$3"

  [[ -e "$path" ]] || return 0
  if grep -Fq -- "$pattern" "$path"; then
    fail "$path contains forbidden pattern '$pattern' ($why)"
  fi
}

assert_absent() {
  local path="$1"
  local why="$2"

  if [[ -e "$path" ]]; then
    fail "$path exists ($why)"
  fi
}

assert_contains() {
  local path="$1"
  local pattern="$2"
  local why="$3"

  [[ -e "$path" ]] || fail "$path missing ($why)"
  if ! grep -Fq -- "$pattern" "$path"; then
    fail "$path does not contain required pattern '$pattern' ($why)"
  fi
}

active_mcp_sources=(
  "$ROOT/agent-env.yaml"
  "$ROOT/agent-env.lock"
  "$ROOT/.mcp.json"
  "$ROOT/.codex/AGENTS.md"
)

for path in "${active_mcp_sources[@]}"; do
  assert_not_contains "$path" "n8n-mcp" "envctl MCP baseline is exactly github/context7/exa/memory/playwright/sequential-thinking"
  assert_not_contains "$path" "/home/drdave/Desktop/meta" "active envctl MCP sources must not point at the retired workspace root"
done

assert_absent "$ROOT/agent-skills/mcps/n8n-mcp.json" "retired MCP must not remain as a source-pack asset"

if [[ -d "$ROOT/agent-skills/mcps" ]]; then
  while IFS= read -r -d '' path; do
    assert_not_contains "$path" "/home/drdave/Desktop/meta" "active MCP asset must use the current FlexNetOS workspace root"
  done < <(find "$ROOT/agent-skills/mcps" -type f -name '*.json' -print0)
fi

assert_contains "$ROOT/AGENTS.md" "Codex binary/runtime ownership must mirror the Yazelix binary/runtime model" \
  "repo instructions must preserve the Yazelix-as-authority contract"
assert_contains "$ROOT/AGENTS.md" "User-bin shadows, repo-cache materializations, temp plugin bundles, marketplace caches, and generated-output files are never alternate active locations." \
  "repo instructions must forbid parallel non-Yazelix ownership paths"

assert_not_contains "$ROOT/manifest/ai-clis.toml" 'ln -sfn "$LINK" "$real_link"' \
  "Codex manifest must not create real-home user-bin Codex shadows"
assert_not_contains "$ROOT/manifest/ai-clis.toml" 'ln -sfn "$link" "$real_link"' \
  "Codex manifest must not preserve fallback user-bin Codex wrappers"
assert_not_contains "$ROOT/manifest/ai-clis.toml" 'tar -C "$ENVCTL_REAL_HOME/.codex"' \
  "Codex manifest must not copy legacy real-home generated config as source"

assert_not_contains "$ROOT/manifest/components.d/codex-global-baseline.toml" 'ln -s "$CODEX_HOME_DIR" "$R/.codex"' \
  "Codex global baseline must not turn active ~/.codex into a generated symlink shadow"
assert_not_contains "$ROOT/manifest/components.d/codex-global-baseline.toml" 'ln -sfn "$M/usr/bin/codex" "$R/.local/bin/codex"' \
  "Codex global baseline must not create real-home user-bin Codex shadows"

if grep -R -n -F "/home/flexnetos/.local/share/yazelix" \
  "$ROOT/agent-env.yaml" \
  "$ROOT/manifest" \
  "$ROOT/agent-skills" \
  "$ROOT/.codex/AGENTS.md" 2>/dev/null |
  grep -E "edit|write|source|input|owner" >/dev/null; then
  fail "source/config text appears to treat /home/flexnetos/.local/share/yazelix as an editable input"
fi

printf 'PASS: Yazelix/Codex ownership source gate\n'
