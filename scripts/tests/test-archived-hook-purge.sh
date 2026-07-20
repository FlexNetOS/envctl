#!/usr/bin/env bash
set -euo pipefail

root="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
cd "$root"

fail() {
  printf 'archived-hook purge: FAIL: %s\n' "$*" >&2
  exit 1
}

for path in \
  .codex/archive/lifecycle-hooks-20260703T024950Z \
  .codex/config.toml.old \
  .codex/hooks.json.old \
  .codex/hooks/pre-cleanroom-hooks.zip \
  .claude.tar.xz \
  home/.codex/hooks.json \
  home/.codex/mined-live/rules/default.rules
do
  [ ! -e "$path" ] && [ ! -L "$path" ] \
    || fail "retired hook payload remains: $path"
done

for mirror in \
  .agents/skills/agent-env-codex/references/source-prompt.md \
  .codex/skills/agent-env-codex/references/source-prompt.md
do
  [ -f "$mirror" ] || fail "missing generated-mirror tombstone: $mirror"
  grep -Fq 'Generated mirror blocked pending owner regeneration' "$mirror" \
    || fail "generated mirror is not fail-closed: $mirror"
  ! grep -Eq 'Wire hooks to|Required hooks:|SessionStart:' "$mirror" \
    || fail "retired lifecycle instructions remain in $mirror"
done

printf 'archived-hook purge: PASS\n'
