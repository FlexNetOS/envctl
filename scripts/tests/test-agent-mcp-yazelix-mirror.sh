#!/usr/bin/env bash
# Verify generated agent MCP config follows the Yazelix mirror rule.
set -euo pipefail

root="$(git -C "$(dirname "${BASH_SOURCE[0]}")/../.." rev-parse --show-toplevel)"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

require_file() {
  [ -f "$1" ] || fail "expected file $1"
}

require_absent() {
  [ ! -e "$1" ] || fail "stale MCP source must not exist: $1"
}

stale_mcps=(context7 github memory playwright sequential-thinking)

require_file "$root/agent-skills/mcps/exa.json"
grep -Fq '"url": "https://mcp.exa.ai/mcp"' "$root/agent-skills/mcps/exa.json" \
  || fail "exa MCP source must remain URL-only"

for name in "${stale_mcps[@]}"; do
  require_absent "$root/agent-skills/mcps/$name.json"
  ! grep -Eq "^[[:space:]]*-[[:space:]]*$name[[:space:]]*$" "$root/agent-env.yaml" \
    || fail "agent-env.yaml still selects stale MCP $name"
  ! grep -Fq "mcp::./agent-skills::$name.json" "$root/agent-env.lock" \
    || fail "agent-env.lock still tracks stale MCP $name"
done

config_files=("$root/.mcp.json")
if [ -n "${ENVCTL_RENDERED_CODEX_CONFIG:-}" ]; then
  config_files+=("$ENVCTL_RENDERED_CODEX_CONFIG")
elif [ -f "$root/.codex/config.toml" ]; then
  config_files+=("$root/.codex/config.toml")
else
  fail "set ENVCTL_RENDERED_CODEX_CONFIG or provide .codex/config.toml"
fi

for file in "${config_files[@]}"; do
  require_file "$file"
  grep -Fq "https://mcp.exa.ai/mcp" "$file" \
    || fail "$file does not include exa URL MCP"
  ! grep -Eq "context7|github|memory|playwright|sequential-thinking" "$file" \
    || fail "$file still includes a stale MCP entry"
  ! grep -Eq 'bunx|envctl-mcp-memory-server|command[[:space:]]*=|"command"[[:space:]]*:' "$file" \
    || fail "$file still launches a local MCP binary/script"
done

echo "AGENT-MCP-YAZELIX-MIRROR TEST PASS"
