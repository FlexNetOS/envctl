#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

find_meta_root() {
  local start dir
  for start in "$ROOT" "$(git rev-parse --git-common-dir 2>/dev/null || true)"; do
    [ -n "$start" ] || continue
    if [[ "$start" != /* ]]; then
      start="$ROOT/$start"
    fi
    dir="$(cd "$start" 2>/dev/null && pwd -P)" || continue
    while [ "$dir" != / ]; do
      if [ -f "$dir/.meta.yaml" ]; then
        printf '%s\n' "$dir"
        return 0
      fi
      dir="${dir%/*}"
      [ -n "$dir" ] || dir=/
    done
  done
  return 1
}

# Main checkouts live at META_ROOT/src/envctl and managed worktrees may live elsewhere. The git
# common directory still anchors them to the owning meta tree, whose `.meta.yaml` is authoritative.
# Standalone/CI clones have no marker and keep the historical parent-directory fallback.
META_ROOT="${META_ROOT:-$(find_meta_root || { cd "$ROOT/.." && pwd; })}"

META_CARGO_HOME="$META_ROOT/.toolchains/cargo"
export CARGO_HOME="${CARGO_HOME:-$META_CARGO_HOME}"
export PATH="$META_ROOT/usr/bin:${CARGO_HOME:-$META_ROOT/.toolchains/cargo}/bin:$META_CARGO_HOME/bin:$PATH"

fail() {
  printf 'CARGO-AUDIT GATE FAIL: %s\n' "$*" >&2
  exit 1
}

if [ -f "$META_ROOT/.meta.yaml" ]; then
  # Inside the real meta workspace, accepting a later ambient PATH entry would let a stale user or
  # system cargo-audit shadow the declared component. Require the exact regular envctl frontdoor and
  # its private payload. Standalone CI clones have no meta marker and use the freshly installed
  # Cargo-home binary below.
  AUDIT_BIN="$META_ROOT/usr/bin/cargo-audit"
  PRIVATE="$META_ROOT/usr/libexec/envctl/cargo-audit/bin/cargo-audit"
  [ -x "$PRIVATE" ] && [ -f "$PRIVATE" ] && [ ! -L "$PRIVATE" ] \
    || fail "managed cargo-audit payload is missing at $PRIVATE"
  [ -x "$AUDIT_BIN" ] && [ -f "$AUDIT_BIN" ] && [ ! -L "$AUDIT_BIN" ] \
    || fail "managed cargo-audit frontdoor is missing or non-regular at $AUDIT_BIN"
  grep -Fqx '# managed-by: envctl component cargo-audit' "$AUDIT_BIN" \
    || fail "cargo-audit frontdoor lacks the envctl ownership marker"
  grep -Fqx "exec \"$PRIVATE\" \"\$@\"" "$AUDIT_BIN" \
    || fail "cargo-audit frontdoor does not target the managed payload"
else
  AUDIT_BIN="$(command -v cargo-audit || true)"
  [ -n "$AUDIT_BIN" ] \
    || fail "cargo-audit is not installed in ${CARGO_HOME:-$META_ROOT/.toolchains/cargo}/bin"
fi

"$AUDIT_BIN" --version | grep -Eq '(^| )0\.22\.2($| )' \
  || fail "cargo-audit must be exact version 0.22.2"

# Vulnerabilities and advisory warnings (unmaintained, unsound, or yanked) all fail. Keeping this
# fail-closed prevents a future dependency refresh from silently restoring the retired RuVector
# codec/macro crates after their exact-source patches have made the graph warning-free.
"$AUDIT_BIN" audit --deny warnings

echo "CARGO-AUDIT GATE PASS"
