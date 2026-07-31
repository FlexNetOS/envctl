#!/usr/bin/env bash
# Validate profile-owned Claude and its Yazelix-owned configuration. No home-state
# cleanup or alternate binary installation is permitted.
set -euo pipefail
export PATH=/usr/bin:/bin

die() {
  printf 'claude-profile: %s\n' "$*" >&2
  exit 1
}

phase="${1:-verify}"
real_home="${ENVCTL_REAL_HOME:-${HOME:-}}"
[ -n "$real_home" ] || die "ENVCTL_REAL_HOME or HOME is required"
profile="$real_home/.nix-profile"
runtime_root="${CLAUDE_CONFIG_DIR:-/home/flexnetos/meta/var/lib/claude}"
xdg_runtime="${XDG_RUNTIME_DIR:-/home/flexnetos/meta/var/lib/yazelix/runtime/xdg}"
store_root="${ENVCTL_NIX_STORE_ROOT:-/nix/store}"

validate_claude() {
  local bin toolbin resolved_bin resolved_toolbin
  bin="$profile/bin/claude"
  toolbin="$profile/toolbin/claude"
  [ -x "$bin" ] && [ -x "$toolbin" ] || die "Claude is missing from profile bin/toolbin"
  resolved_bin="$(/usr/bin/readlink -f -- "$bin")"
  resolved_toolbin="$(/usr/bin/readlink -f -- "$toolbin")"
  [ "$resolved_bin" = "$resolved_toolbin" ] || die "Claude profile frontdoors diverge"
  case "$resolved_bin" in "$store_root"/*) ;; *) die "Claude target is outside the immutable store" ;; esac

  for source in settings.json.src CLAUDE.md.src RTK.md.src; do
    [ -f "$profile/share/yazelix/agent_configs/claude/$source" ] \
      || die "missing profile-owned Claude config input: $source"
  done

  HOME="$real_home" CLAUDE_CONFIG_DIR="$runtime_root" \
    XDG_RUNTIME_DIR="$xdg_runtime" \
    PATH="$profile/toolbin:$profile/bin" \
    /usr/bin/timeout --kill-after=2s 20s "$bin" --version >/dev/null

  for materialized in settings.json CLAUDE.md RTK.md; do
    [ -f "$runtime_root/$materialized" ] \
      || die "Claude config was not materialized: $materialized"
  done
}

case "$phase" in
  detect|verify) validate_claude ;;
  install|fix)
    validate_claude || die "Claude installation belongs to merged FlexNetOS/yazelix origin/main"
    printf 'claude-profile: Yazelix-owned state already satisfies the envctl contract\n'
    ;;
  clean)
    validate_claude
    printf 'claude-profile: no alternate state exists to clean\n'
    ;;
  remove)
    printf 'claude-profile: nothing removed; envctl does not own profile Claude\n'
    ;;
  *) die "usage: envctl-claude-cleanup.sh detect|verify|install|fix|clean|remove" ;;
esac
