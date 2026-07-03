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

fail() {
  echo "FAIL: $*" >&2
  exit 1
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

echo "PASS: FlexNetOS Codex runtime gate is archived, inactive, and generator-disabled"
