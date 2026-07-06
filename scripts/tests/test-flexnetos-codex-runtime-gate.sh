#!/usr/bin/env bash
# Guard that archived FlexNetOS Codex lifecycle hooks stay archive-only.
set -euo pipefail

ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
ACTIVE_HOOK="$ROOT/.codex/hooks/flexnetos-runtime-gate.sh"
ACTIVE_HOOKS_JSON="$ROOT/.codex/hooks.json"
ARCHIVE_DIR="$ROOT/.codex/archive/lifecycle-hooks-20260703T024950Z"
ARCHIVED_HOOK="$ARCHIVE_DIR/hooks/flexnetos-runtime-gate.sh.md"
ARCHIVED_HOOKS_JSON="$ARCHIVE_DIR/hooks.json.md"
ARCHIVED_ZIP="$ROOT/.codex/hooks/pre-cleanroom-hooks.zip"

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

echo "PASS: FlexNetOS Codex runtime gate is archived and inactive"
