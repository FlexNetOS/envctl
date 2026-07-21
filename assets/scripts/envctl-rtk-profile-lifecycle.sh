#!/usr/bin/env bash
# Validate RTK through the sole profile frontdoor. Envctl never installs or
# archives an RTK binary.
set -euo pipefail
export PATH=/usr/bin:/bin

die() {
  printf 'rtk-profile: %s\n' "$*" >&2
  exit 1
}

phase="${1:-verify}"
real_home="${ENVCTL_REAL_HOME:-${HOME:-}}"
[ -n "$real_home" ] || die "ENVCTL_REAL_HOME or HOME is required"
profile="$real_home/.nix-profile"
store_root="${ENVCTL_NIX_STORE_ROOT:-/nix/store}"

validate_rtk() {
  local bin toolbin resolved_bin resolved_toolbin
  bin="$profile/bin/rtk"
  toolbin="$profile/toolbin/rtk"
  [ -x "$bin" ] && [ -x "$toolbin" ] || die "RTK is missing from profile bin/toolbin"
  resolved_bin="$(/usr/bin/readlink -f -- "$bin")"
  resolved_toolbin="$(/usr/bin/readlink -f -- "$toolbin")"
  [ "$resolved_bin" = "$resolved_toolbin" ] || die "RTK profile frontdoors diverge"
  case "$resolved_bin" in "$store_root"/*) ;; *) die "RTK target is outside the immutable store" ;; esac
  HOME="$real_home" PATH="$profile/toolbin:$profile/bin" "$bin" --version >/dev/null
}

case "$phase" in
  detect|verify) validate_rtk ;;
  install|fix)
    validate_rtk || die "RTK installation belongs to merged FlexNetOS/yazelix origin/main"
    printf 'rtk-profile: profile RTK already satisfies the envctl contract\n'
    ;;
  remove)
    printf 'rtk-profile: nothing removed; envctl does not own profile RTK\n'
    ;;
  *) die "usage: envctl-rtk-profile-lifecycle.sh detect|verify|install|fix|remove" ;;
esac
