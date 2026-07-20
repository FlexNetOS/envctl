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
  home/.claude/hooks \
  .codex/hooks \
  .codex/hooks.json \
  home/.codex/mined-live/rules/default.rules
do
  [ ! -e "$path" ] && [ ! -L "$path" ] \
    || fail "retired hook payload remains: $path"
done

source=agent-skills/agent-env-codex/references/source-prompt.md
[ -f "$source" ] || fail "missing lifecycle owner prompt: $source"
! grep -Eq 'codex-harness-hook|\.claude/hooks|\.codex/hooks' "$source" \
  || fail "retired lifecycle instructions remain in owner prompt"

if rg -n --glob '!envctl-db-nu-plugin-migration-automation-package/**' \
  --glob '!docs/generated/**' \
  --glob '!*.lock' \
  --glob '!scripts/tests/test-archived-hook-purge.sh' \
  --glob '!scripts/tests/test-agent-env-hooks.sh' \
  --glob '!scripts/tests/test-flexnetos-codex-runtime-gate.sh' \
  --glob '!home/agent-env/codex-harness/src/bin/codex-harness-final-verify.rs' \
  'codex-harness-hook|hooks\.write_text|/home/flexnetos/FlexNetOS/\.codex/hooks' \
  agent-skills home manifest assets scripts ci; then
  fail "retired lifecycle regeneration path remains"
fi

printf 'archived-hook purge: PASS\n'
