#!/usr/bin/env bash
# Validate the installed Yazelix foundation without building or switching it.
# The only authorized cutover owner is a clean, merged FlexNetOS/yazelix
# origin/main checkout; envctl is deliberately a read-only bridge here.
set -euo pipefail
export PATH=/usr/bin:/bin

die() {
  printf 'yazelix-profile: %s\n' "$*" >&2
  exit 1
}

phase="${1:-verify}"
real_home="${ENVCTL_REAL_HOME:-${HOME:-}}"
[ -n "$real_home" ] || die "ENVCTL_REAL_HOME or HOME is required"
profile="$real_home/.nix-profile"
store_root="${ENVCTL_NIX_STORE_ROOT:-/nix/store}"

validate_profile() {
  local selector generation resolved bin toolbin resolved_bin resolved_toolbin
  [ -L "$profile" ] || die "profile frontdoor is not a symlink: $profile"
  selector="$(/usr/bin/readlink -- "$profile")"
  printf '%s\n' "$selector" | /usr/bin/grep -Eq '^\.nix-profile-[1-9][0-9]*-link$' \
    || die "profile must select one direct generation: $selector"
  generation="$real_home/$selector"
  [ -L "$generation" ] || die "profile generation is missing: $generation"
  resolved="$(/usr/bin/readlink -f -- "$profile")"
  case "$resolved" in "$store_root"/*-profile) ;; *) die "profile target is not an immutable profile output: $resolved" ;; esac

  bin="$profile/bin/yzx"
  toolbin="$profile/toolbin/yzx"
  [ -x "$bin" ] && [ -x "$toolbin" ] || die "yzx is missing from profile bin/toolbin"
  resolved_bin="$(/usr/bin/readlink -f -- "$bin")"
  resolved_toolbin="$(/usr/bin/readlink -f -- "$toolbin")"
  [ "$resolved_bin" = "$resolved_toolbin" ] || die "profile yzx frontdoors diverge"
  case "$resolved_bin" in "$store_root"/*) ;; *) die "yzx target is outside the immutable store" ;; esac

  HOME="$real_home" PATH="$profile/toolbin:$profile/bin" \
    /usr/bin/timeout --kill-after=2s 15s "$bin" --version >/dev/null
}

case "$phase" in
  detect|verify)
    validate_profile
    ;;
  install|fix)
    validate_profile || die "install/fix belongs to merged FlexNetOS/yazelix origin/main"
    printf 'yazelix-profile: profile already satisfies the read-only envctl contract\n'
    ;;
  remove)
    printf 'yazelix-profile: nothing removed; envctl does not own the installed profile\n'
    ;;
  *)
    die "usage: envctl-yazelix-profile-lifecycle.sh detect|verify|install|fix|remove"
    ;;
esac
