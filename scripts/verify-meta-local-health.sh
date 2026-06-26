#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
META_ROOT="${META_ROOT:-$(cd "$ROOT/.." && pwd -P)}"
REAL_HOME="${ENVCTL_REAL_HOME:-$HOME}"
META_LOCAL="$META_ROOT/.local"
HOME_LOCAL="$REAL_HOME/.local"

fail() {
  echo "meta-local-health: FAIL: $*" >&2
  exit 1
}

[ -d "$META_LOCAL" ] || fail "$META_LOCAL missing"
[ -L "$HOME_LOCAL" ] || fail "$HOME_LOCAL must be the single bridge symlink"
[ "$(readlink -f "$HOME_LOCAL")" = "$(readlink -f "$META_LOCAL")" ] || fail "$HOME_LOCAL does not resolve to $META_LOCAL"

if find "$META_LOCAL" -xdev -type l -print -quit | grep -q .; then
  echo "meta-local-health: symlinks under $META_LOCAL are forbidden (single bridge only):" >&2
  find "$META_LOCAL" -xdev -type l -print >&2
  exit 1
fi

[ -d "$META_LOCAL/bin" ] || fail "$META_LOCAL/bin missing"
export PATH="$META_LOCAL/bin:$PATH"

while IFS= read -r -d '' p; do
  [ -f "$p" ] || fail "bin entry is not a regular file: $p"
  [ -x "$p" ] || fail "bin entry is not executable: $p"
  rp="$(realpath -e "$p")"
  case "$rp" in
    "$META_ROOT"/*) ;;
    *) fail "bin entry resolves outside META_ROOT: $p -> $rp" ;;
  esac
  if head -c 2 "$p" 2>/dev/null | grep -q '#!'; then
    legacy_home_local="$REAL_HOME/.local"
    if grep -nF "$legacy_home_local" "$p" >&2; then
      fail "script frontdoor embeds the legacy real-home .local path: $p"
    fi
    if grep -nE '(~|\$HOME|\$\{HOME\}|%h)/\.local' "$p" >&2; then
      fail "script frontdoor embeds a real-home .local path: $p"
    fi
  fi
done < <(find "$META_LOCAL/bin" -mindepth 1 -maxdepth 1 -print0)

for tool in envctl meta git gh uv uvx bun node nvidia-smi nvcc python3 python3.14 cargo-nextest just zellij yazi hx fd bat fzf jj zizmor claude ruby gem bundle bundler; do
  [ -e "$META_LOCAL/bin/$tool" ] || continue
  cmd="$(command -v "$tool" || true)"
  [ -n "$cmd" ] || fail "$tool exists in META_ROOT/.local/bin but is not on PATH"
  rp="$(realpath -e "$cmd")"
  case "$rp" in
    "$META_ROOT"/*) ;;
    *) fail "$tool resolves outside META_ROOT: $cmd -> $rp" ;;
  esac
done

bin_count="$(find "$META_LOCAL/bin" -mindepth 1 -maxdepth 1 | wc -l)"
echo "meta-local-health: PASS META_ROOT=$META_ROOT bin_count=$bin_count"
