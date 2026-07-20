#!/usr/bin/env bash
# Proves that active Codex runtime ownership has no retired lifecycle dispatcher.
set -euo pipefail

ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
CODEX_BASELINE="$ROOT/manifest/components.d/codex-global-baseline.toml"
AI_CLIS="$ROOT/manifest/ai-clis.toml"
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }

python3 - "$CODEX_BASELINE" <<'PY'
import sys
import tomllib

component = tomllib.loads(open(sys.argv[1], encoding='utf-8').read())['component'][0]
script = '$ENVCTL_SOURCE_ROOT/assets/scripts/envctl-codex-global-baseline-lifecycle.sh'
for phase in ('detect', 'install', 'verify', 'fix', 'remove'):
    actual = component[phase]
    if actual != {'kind': 'shipped_script', 'path': script, 'args': [phase]}:
        raise SystemExit(f'wrong Codex baseline lifecycle for {phase}: {actual!r}')
PY

for path in "$CODEX_BASELINE" "$AI_CLIS"; do
  ! rg -n 'hooks\.write_text|hooks\.json|rtk hook claude|flexnetos-runtime-gate' "$path" \
    || fail "retired lifecycle dispatcher remains in $path"
done

for path in \
  "$ROOT/.codex/hooks/flexnetos-runtime-gate.sh" \
  "$ROOT/.codex/hooks.json" \
  "$ROOT/home/.codex/hooks.json"; do
  [ ! -e "$path" ] && [ ! -L "$path" ] || fail "retired hook path remains: $path"
done

printf 'CODEX RUNTIME LIFECYCLE ERADICATION PASS\n'
