#!/usr/bin/env bash
# test-agent-env-hooks.sh — retired lifecycle-hook eradication contract.
set -euo pipefail

root="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }

for path in \
  "$root/home/.claude/hooks" \
  "$root/home/.codex/hooks.json" \
  "$root/.codex/hooks" \
  "$root/.codex/hooks.json"; do
  [ ! -e "$path" ] && [ ! -L "$path" ] || fail "retired lifecycle path remains: $path"
done

for path in "$root/home/.claude/settings.json" "$root/home/.claude/settings.json.tmpl"; do
  python3 - "$path" <<'PY'
import json
import sys

data = json.load(open(sys.argv[1]))
for key in ("hooks", "statusLine"):
    if key in data:
        raise SystemExit(f"retired lifecycle setting remains: {key}")
PY
done

cmp -s "$root/home/.claude/settings.json" "$root/home/.claude/settings.json.tmpl" \
  || fail "settings template and rendered source diverged"

if rg -n --glob '!envctl-db-nu-plugin-migration-automation-package/**' \
  --glob '!docs/generated/**' \
  --glob '!scripts/tests/test-archived-hook-purge.sh' \
  --glob '!scripts/tests/test-agent-env-hooks.sh' \
  --glob '!scripts/tests/test-flexnetos-codex-runtime-gate.sh' \
  --glob '!home/agent-env/codex-harness/src/bin/codex-harness-final-verify.rs' \
  'rtk hook claude|\.claude/hooks|\.codex/hooks|hooks\.json' \
  "$root/home" "$root/manifest" "$root/assets/scripts"; then
  fail "a tracked lifecycle owner can still execute or regenerate retired hooks"
fi

printf 'AGENT-ENV HOOK ERADICATION PASS\n'
