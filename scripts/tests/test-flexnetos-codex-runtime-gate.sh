#!/usr/bin/env bash
# Guard that archived FlexNetOS Codex lifecycle hooks stay archive-only and
# envctl does not regenerate the pre-cleanroom baseline.
set -euo pipefail

ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
ACTIVE_HOOK="$ROOT/.codex/hooks/flexnetos-runtime-gate.sh"
ACTIVE_HOOKS_JSON="$ROOT/.codex/hooks.json"
ARCHIVE_DIR="$ROOT/.codex/archive/lifecycle-hooks-20260703T024950Z"
ARCHIVED_HOOK="$ARCHIVE_DIR/hooks/flexnetos-runtime-gate.sh.md"
ARCHIVED_HOOKS_JSON="$ARCHIVE_DIR/hooks.json.md"
ARCHIVED_ZIP="$ROOT/.codex/hooks/pre-cleanroom-hooks.zip"
CODEX_BASELINE="$ROOT/manifest/components.d/codex-global-baseline.toml"
WORKSPACE_CODEX_CONFIG="/home/flexnetos/FlexNetOS/.codex/config.toml"
HOME_CODEX_CONFIG="/home/flexnetos/.codex/config.toml"
HOME_BASHRC="/home/flexnetos/.bashrc"
HOME_PROFILE="/home/flexnetos/.profile"
HOME_LOCAL_BIN="/home/flexnetos/.local/bin"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

assert_not_contains() {
  local path="$1"
  local pattern="$2"
  local message="$3"

  if [ -e "$path" ] && grep -HEnI "$pattern" "$path" >/tmp/flexnetos-codex-runtime-gate.$$ 2>/dev/null; then
    cat /tmp/flexnetos-codex-runtime-gate.$$ >&2
    rm -f /tmp/flexnetos-codex-runtime-gate.$$
    fail "$message"
  fi
  rm -f /tmp/flexnetos-codex-runtime-gate.$$
}

assert_absent() {
  local path="$1"
  local message="$2"

  [ ! -e "$path" ] || fail "$message: $path"
}

[ ! -e "$ACTIVE_HOOK" ] || fail "repo-local runtime gate is active: $ACTIVE_HOOK"
[ ! -e "$ACTIVE_HOOKS_JSON" ] || fail "repo-local hooks.json is active: $ACTIVE_HOOKS_JSON"
[ -s "$ARCHIVED_HOOK" ] || fail "archived runtime gate missing: $ARCHIVED_HOOK"
[ -s "$ARCHIVED_HOOKS_JSON" ] || fail "archived hooks.json missing: $ARCHIVED_HOOKS_JSON"
[ -s "$ARCHIVED_ZIP" ] || fail "compressed pre-cleanroom hook archive missing: $ARCHIVED_ZIP"

grep -q "FlexNetOS Codex runtime gate" "$ARCHIVED_HOOK" \
  || fail "archived runtime gate does not contain expected gate body"
grep -q "PreToolUse" "$ARCHIVED_HOOKS_JSON" \
  || fail "archived hooks.json does not preserve lifecycle hook wiring"
grep -q "Clean-room hooks are mandatory but deferred" "$CODEX_BASELINE" \
  || fail "codex baseline does not record deferred clean-room hook policy"
grep -q "'hooks = false'" "$CODEX_BASELINE" \
  || fail "codex baseline does not pin hooks disabled while clean-room hooks are deferred"
grep -q "stale_hooks.unlink" "$CODEX_BASELINE" \
  || fail "codex baseline does not purge stale pre-cleanroom hooks.json"
! grep -q "hooks.write_text" "$CODEX_BASELINE" \
  || fail "codex baseline still writes hooks.json"
! grep -q 'with-meta-env.sh' "$CODEX_BASELINE" \
  || fail "codex baseline still depends on pre-cleanroom hook helper"

for path in "$ROOT/agent-env.yaml" "$ROOT/agent-env.lock" "$ROOT/.mcp.json" "$ROOT"/agent-skills/mcps/*.json; do
  assert_not_contains "$path" "n8n-mcp" \
    "retired n8n-mcp must not remain in envctl agent-env sources or lock"
  assert_not_contains "$path" "/home/drdave/Desktop/meta" \
    "envctl MCP sources must not point at the retired workspace root"
  assert_not_contains "$path" '\$ROOT/\.local/bin' \
    "envctl MCP launch PATH must not reintroduce user-bin shadows"
done

assert_absent "$ROOT/agent-skills/mcps/n8n-mcp.json" \
  "retired n8n-mcp source asset must be removed"

for path in "$WORKSPACE_CODEX_CONFIG" "$HOME_CODEX_CONFIG"; do
  [ -e "$path" ] || continue
  assert_not_contains "$path" "n8n-mcp|codex-security|openai-api-key-local-confirmation|mcp_servers\.gitkb" \
    "active Codex MCP config must remain at the six-server baseline"
  assert_not_contains "$path" "marketplaces\.|plugins\.\"" \
    "active Codex config must not publish plugin marketplace inventory as a runtime authority"
  assert_not_contains "$path" '\$ROOT/\.local/bin' \
    "active Codex MCP launch PATH must not reintroduce user-bin shadows"
done

assert_not_contains "$CODEX_BASELINE" 'ln -sfn .*codex.*\.local/bin|write_text\(.*codex' \
  "Codex baseline must not create real-home user-bin Codex shadows"

assert_not_contains "$HOME_BASHRC" 'export PATH="/home/flexnetos/\.local/bin:\$PATH"' \
  "home bashrc must not prepend real-home user-bin ahead of profile/runtime frontdoors"
assert_not_contains "$HOME_PROFILE" 'PATH="\$HOME/\.local/bin:\$PATH"' \
  "home profile must not prepend real-home user-bin ahead of profile/runtime frontdoors"

for name in yzx codex rtk git-kb agent bun bunx loop meta meta-git meta-mcp meta-project meta-release meta-rust kache-rustc-wrapper; do
  assert_absent "$HOME_LOCAL_BIN/$name" \
    "real-home user-bin must not contain active binary shadows"
done

echo "PASS: FlexNetOS Codex runtime gate is archived, inactive, generator-disabled, and Yazelix-owned"
