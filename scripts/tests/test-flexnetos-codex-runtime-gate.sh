#!/usr/bin/env bash
# Guard that envctl generates only the profile-owned RTK Bash hook for Codex.
set -euo pipefail

ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
ACTIVE_HOOK="$ROOT/.codex/hooks/flexnetos-runtime-gate.sh"
CODEX_BASELINE="$ROOT/manifest/components.d/codex-global-baseline.toml"
CODEX_LIFECYCLE="$ROOT/assets/scripts/envctl-codex-global-baseline-lifecycle.sh"
AI_CLIS="$ROOT/manifest/ai-clis.toml"
WORKSPACE_CODEX_CONFIG="/home/flexnetos/FlexNetOS/.codex/config.toml"
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
python3 - "$CODEX_BASELINE" <<'PY'
import sys
import tomllib

with open(sys.argv[1], "rb") as source:
    data = tomllib.load(source)
component = data["component"][0]
for phase in ("detect", "install", "verify", "fix", "remove"):
    step = component[phase]
    if step != {
        "kind": "shipped_script",
        "path": "$ENVCTL_SOURCE_ROOT/assets/scripts/envctl-codex-global-baseline-lifecycle.sh",
        "args": [phase],
    }:
        raise SystemExit(f"Codex baseline {phase} must delegate to the canonical lifecycle owner")
PY
bash -n "$CODEX_LIFECYCLE" \
  || fail "Codex baseline lifecycle has invalid shell syntax"
grep -Fq 'codex_global_sync_hook_dispatcher' "$CODEX_LIFECYCLE" \
  || fail "Codex baseline lifecycle does not own hook generation"
grep -Fq '/home/flexnetos/.nix-profile/bin/rtk hook claude' "$CODEX_LIFECYCLE" \
  || fail "Codex baseline lifecycle does not use the RTK hook processor"
! grep -Eq 'hooks\.json\.(old|bak|backup|disabled|saved|archive|orig|rej)' "$CODEX_LIFECYCLE" \
  || fail "Codex baseline lifecycle scans archived hook payloads"
[ "$(grep -Fc '"command": "/home/flexnetos/.nix-profile/bin/rtk hook claude"' "$AI_CLIS")" -eq 0 ] \
  || fail "Codex CLI manifest must not retain a duplicate lifecycle-hook writer"
! grep -q 'if hooks.exists' "$AI_CLIS" \
  || fail "Codex CLI migration paths preserve legacy hook payloads"

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

echo "PASS: Codex hook generation is RTK-only, active, and Yazelix-owned"
