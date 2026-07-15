#!/usr/bin/env bash
# Guard that envctl generates only the profile-owned RTK Bash hook for Codex.
set -euo pipefail

ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
ACTIVE_HOOK="$ROOT/.codex/hooks/flexnetos-runtime-gate.sh"
CODEX_BASELINE="$ROOT/manifest/components.d/codex-global-baseline.toml"
AI_CLIS="$ROOT/manifest/ai-clis.toml"
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

assert_rtk_hook_contract() {
  python3 - "$1" <<'PY'
import json
import sys

expected = {
    'hooks': {
        'PreToolUse': [{
            'matcher': 'Bash',
            'hooks': [{
                'type': 'command',
                'command': '/home/flexnetos/.nix-profile/bin/rtk hook claude',
            }],
        }],
    },
}
with open(sys.argv[1]) as source:
    actual = json.load(source)
if actual != expected:
    raise SystemExit(f'RTK-only Codex hook contract mismatch: {sys.argv[1]}')
PY
}

[ ! -e "$ACTIVE_HOOK" ] || fail "repo-local runtime gate is active: $ACTIVE_HOOK"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
mkdir -p "$TMP_DIR/.local/share/codex"
awk '
  /^python3 - <<'\''PY'\''$/ { capture = 1; next }
  capture && /^PY$/ { exit }
  capture { print }
' "$CODEX_BASELINE" > "$TMP_DIR/generate_codex_baseline.py"
META_ROOT="$TMP_DIR" ENVCTL_ROOT="$ROOT" python3 "$TMP_DIR/generate_codex_baseline.py"
assert_rtk_hook_contract "$TMP_DIR/.local/share/codex/hooks.json"
grep -q "'hooks'" "$CODEX_BASELINE" \
  || fail "codex baseline does not enable the hooks feature"
grep -q "hooks.write_text" "$CODEX_BASELINE" \
  || fail "codex baseline does not generate hooks.json"
grep -q 'rtk hook claude' "$CODEX_BASELINE" \
  || fail "codex baseline does not use the RTK hook processor"
! grep -q "stale_hooks.unlink" "$CODEX_BASELINE" \
  || fail "codex baseline still purges hooks.json"
! grep -q 'with-meta-env.sh' "$CODEX_BASELINE" \
  || fail "codex baseline still depends on pre-cleanroom hook helper"
[ "$(grep -Fc '"command": "/home/flexnetos/.nix-profile/bin/rtk hook claude"' "$AI_CLIS")" -eq 2 ] \
  || fail "Codex CLI migration paths do not both write the RTK hook contract"
! grep -q 'if hooks.exists' "$AI_CLIS" \
  || fail "Codex CLI migration paths preserve legacy hook payloads"

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

echo "PASS: Codex hook generation is RTK-only, active, and Yazelix-owned"
