#!/usr/bin/env bash
# Verify the MCP memory launcher roots all writable state in $META_ROOT/.local.
set -euo pipefail

root="$(git -C "$(dirname "${BASH_SOURCE[0]}")/../.." rev-parse --show-toplevel)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

meta="$tmp/meta"
mkdir -p "$meta/usr/bin"
cat > "$meta/usr/bin/bunx" <<'BUNX'
#!/usr/bin/env bash
set -euo pipefail
{
  printf 'HOME=%s\n' "$HOME"
  printf 'XDG_DATA_HOME=%s\n' "$XDG_DATA_HOME"
  printf 'XDG_STATE_HOME=%s\n' "$XDG_STATE_HOME"
  printf 'XDG_CACHE_HOME=%s\n' "$XDG_CACHE_HOME"
  printf 'npm_config_cache=%s\n' "$npm_config_cache"
  printf 'npm_config_prefix=%s\n' "$npm_config_prefix"
  printf 'MEMORY_FILE_PATH=%s\n' "$MEMORY_FILE_PATH"
  printf 'argv=%s\n' "$*"
} > "$META_ROOT/env.out"
BUNX
chmod +x "$meta/usr/bin/bunx"

META_ROOT="$meta" "$root/assets/scripts/envctl-mcp-memory-server"

want() {
  local key="$1" value="$2"
  if ! grep -Fxq "$key=$value" "$meta/env.out"; then
    echo "FAIL: expected $key=$value" >&2
    echo "--- captured ---" >&2
    cat "$meta/env.out" >&2
    exit 1
  fi
}

want HOME "$meta/.local"
want XDG_DATA_HOME "$meta/.local/share"
want XDG_STATE_HOME "$meta/.local/state"
want XDG_CACHE_HOME "$meta/.local/cache"
want npm_config_cache "$meta/.local/cache/npm"
want npm_config_prefix "$meta/.local"
want MEMORY_FILE_PATH "$meta/.local/share/mcp-memory/memory.jsonl"
want argv "@modelcontextprotocol/server-memory"

test -d "$meta/.local/share/mcp-memory" || { echo "FAIL: memory data dir was not created" >&2; exit 1; }
test -d "$meta/.local/cache/npm" || { echo "FAIL: npm cache dir was not created" >&2; exit 1; }

# Source and rendered MCP configs must launch the same meta-rooted wrapper.
for file in "$root/agent-skills/mcps/memory.json" "$root/.mcp.json" "$root/.codex/config.toml"; do
  grep -q 'envctl-mcp-memory-server' "$file" || { echo "FAIL: $file does not use envctl-mcp-memory-server" >&2; exit 1; }
  grep -q 'META_ROOT' "$file" || { echo "FAIL: $file does not preserve META_ROOT routing" >&2; exit 1; }
done

echo "MCP-MEMORY-META-ROOT TEST PASS"
