#!/usr/bin/env bash
# Validate a command exported by the sole Yazelix/Nix profile.
set -euo pipefail
export PATH=/usr/bin:/bin

die() { printf 'profile-command: %s\n' "$*" >&2; exit 1; }

command_name="${1:-}"
phase="${2:-verify}"
case "$command_name" in
  gemini|kimi|devin) ;;
  *) die 'usage: envctl-profile-command-lifecycle.sh gemini|kimi|devin detect|verify|install|fix|remove' ;;
esac

real_home="${ENVCTL_REAL_HOME:-${HOME:-}}"
[ -n "$real_home" ] || die 'ENVCTL_REAL_HOME or HOME is required'
profile="$real_home/.nix-profile"
store_root="${ENVCTL_NIX_STORE_ROOT:-/nix/store}"

validate_command() {
  local bin toolbin resolved_bin resolved_toolbin
  bin="$profile/bin/$command_name"
  toolbin="$profile/toolbin/$command_name"
  [ -x "$bin" ] && [ -x "$toolbin" ] \
    || die "$command_name is missing from profile bin/toolbin"
  resolved_bin="$(/usr/bin/readlink -f -- "$bin")"
  resolved_toolbin="$(/usr/bin/readlink -f -- "$toolbin")"
  [ "$resolved_bin" = "$resolved_toolbin" ] \
    || die "$command_name profile frontdoors diverge"
  case "$resolved_bin" in
    "$store_root"/*) ;;
    *) die "$command_name target is outside the immutable store" ;;
  esac
  HOME="$real_home" PATH="$profile/toolbin:$profile/bin" \
    /usr/bin/timeout --kill-after=2s 20s "$bin" --version >/dev/null
}

case "$phase" in
  detect|verify) validate_command ;;
  install|fix)
    validate_command || die "$command_name installation belongs to merged FlexNetOS/yazelix origin/main"
    printf 'profile-command: %s already satisfies the profile contract\n' "$command_name"
    ;;
  remove)
    printf 'profile-command: nothing removed; envctl does not own profile %s\n' "$command_name"
    ;;
  *) die 'usage: envctl-profile-command-lifecycle.sh gemini|kimi|devin detect|verify|install|fix|remove' ;;
esac
