#!/usr/bin/env bash
# Guard that archived FlexNetOS Codex lifecycle hooks stay archive-only and
# envctl validates the editable active-home config without regenerating a
# parallel runtime.
set -euo pipefail

ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
ACTIVE_HOOK="$ROOT/.codex/hooks/flexnetos-runtime-gate.sh"
ACTIVE_HOOKS_JSON="$ROOT/.codex/hooks.json"
ARCHIVE_DIR="$ROOT/.codex/archive/lifecycle-hooks-20260703T024950Z"
ARCHIVED_HOOK="$ARCHIVE_DIR/hooks/flexnetos-runtime-gate.sh.md"
ARCHIVED_HOOKS_JSON="$ARCHIVE_DIR/hooks.json.md"
ARCHIVED_ZIP="$ROOT/.codex/hooks/pre-cleanroom-hooks.zip"
CODEX_BASELINE="$ROOT/manifest/components.d/codex-global-baseline.toml"
RETIRED_FLEXNETOS_CODEX_ROOT="/home/flexnetos/FlexNetOS/.codex"
RETIRED_LIFEOS_CODEX_ROOT="/home/flexnetos/lifeos/.codex"
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

  [ ! -e "$path" ] && [ ! -L "$path" ] || fail "$message: $path"
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
grep -q 'envctl-codex-global-baseline-lifecycle.sh' "$CODEX_BASELINE" \
  || fail "codex baseline does not delegate the audited active-home lifecycle"
assert_not_contains "$CODEX_BASELINE" \
  'hooks\.json|with-meta-env\.sh|model-catalog|home/\.codex/agents|marketplaces\.|plugins\."|CODEX_HOME|CODEX_SQLITE_HOME' \
  "codex baseline must not regenerate hooks, catalogs, agents, plugins, or state roots"

for path in "$ROOT/agent-env.yaml" "$ROOT/agent-env.lock" "$ROOT/.mcp.json" "$ROOT"/agent-skills/mcps/*.json; do
  assert_not_contains "$path" "n8n-mcp" \
    "retired n8n-mcp must not remain in envctl agent-env sources or lock"
  assert_not_contains "$path" "/home/drdave/Desktop/meta" \
    "envctl MCP sources must not point at the retired workspace root"
  # shellcheck disable=SC2016 # Match the literal runtime token; do not expand this test pattern.
  assert_not_contains "$path" '\$ROOT/\.local/bin' \
    "envctl MCP launch PATH must not reintroduce user-bin shadows"
done

assert_absent "$ROOT/agent-skills/mcps/n8n-mcp.json" \
  "retired n8n-mcp source asset must be removed"

assert_absent "$RETIRED_FLEXNETOS_CODEX_ROOT" \
  "retired FlexNetOS Codex mirror must not reappear through a workspace alias"
assert_absent "$RETIRED_LIFEOS_CODEX_ROOT" \
  "retired lifeos Codex mirror must not reappear"

if [ -e "$HOME_CODEX_CONFIG" ]; then
  python3 - "$HOME_CODEX_CONFIG" <<'PY'
import sys
import tomllib
from pathlib import Path

path = Path(sys.argv[1])
data = tomllib.loads(path.read_text())
if "marketplaces" in data or "plugins" in data:
    raise SystemExit("active Codex config publishes plugin/marketplace runtime authority")
expected = {
    "exa": "https://mcp.exa.ai/mcp",
    "openaiDeveloperDocs": "https://developers.openai.com/mcp",
}
for name, value in data.get("mcp_servers", {}).items():
    if name not in expected:
        raise SystemExit(f"forbidden active-home MCP server: {name}")
    if value != {"url": expected[name]}:
        raise SystemExit(f"active-home MCP `{name}` is not canonical remote-URL-only config")
PY
fi

assert_not_contains "$CODEX_BASELINE" 'ln -sfn .*codex.*\.local/bin|write_text\(.*codex' \
  "Codex baseline must not create real-home user-bin Codex shadows"

# shellcheck disable=SC2016 # Match the literal shell source; do not expand this test pattern.
assert_not_contains "$HOME_BASHRC" 'export PATH="/home/flexnetos/\.local/bin:\$PATH"' \
  "home bashrc must not prepend real-home user-bin ahead of profile/runtime frontdoors"
# shellcheck disable=SC2016 # Match the literal shell source; do not expand this test pattern.
assert_not_contains "$HOME_PROFILE" 'PATH="\$HOME/\.local/bin:\$PATH"' \
  "home profile must not prepend real-home user-bin ahead of profile/runtime frontdoors"

for name in yzx codex rtk git-kb agent bun bunx loop meta meta-git meta-mcp meta-project meta-release meta-rust kache-rustc-wrapper; do
  assert_absent "$HOME_LOCAL_BIN/$name" \
    "real-home user-bin must not contain active binary shadows"
done

echo "PASS: FlexNetOS Codex runtime gate is archived, active-home policy-only, and Yazelix-owned"
