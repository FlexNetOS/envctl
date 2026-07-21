#!/usr/bin/env bash
# Validate the profile-owned Codex runtime and its volatile configuration.
# This script never installs, wraps, copies, or switches Codex.
set -euo pipefail
export PATH=/usr/bin:/bin

die() {
  printf 'codex-profile: %s\n' "$*" >&2
  exit 1
}

phase="${1:-verify}"
real_home="${ENVCTL_REAL_HOME:-${HOME:-}}"
[ -n "$real_home" ] || die "ENVCTL_REAL_HOME or HOME is required"
profile="$real_home/.nix-profile"
runtime_root="${XDG_RUNTIME_DIR:-/run/user/$(/usr/bin/id -u)}/yazelix/profile-runtime/codex"
store_root="${ENVCTL_NIX_STORE_ROOT:-/nix/store}"

validate_codex() {
  local bin toolbin resolved_bin resolved_toolbin
  [ -L "$profile" ] || die "profile frontdoor is missing: $profile"
  bin="$profile/bin/codex"
  toolbin="$profile/toolbin/codex"
  [ -x "$bin" ] && [ -x "$toolbin" ] || die "Codex is missing from profile bin/toolbin"
  resolved_bin="$(/usr/bin/readlink -f -- "$bin")"
  resolved_toolbin="$(/usr/bin/readlink -f -- "$toolbin")"
  [ "$resolved_bin" = "$resolved_toolbin" ] || die "Codex profile frontdoors diverge"
  case "$resolved_bin" in "$store_root"/*) ;; *) die "Codex target is outside the immutable store" ;; esac

  for source in config.toml.src RULES.md.src; do
    [ -f "$profile/share/yazelix/agent_configs/codex/$source" ] \
      || die "missing profile-owned Codex config input: $source"
  done

  HOME="$real_home" CODEX_HOME="$runtime_root" \
    XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(/usr/bin/id -u)}" \
    PATH="$profile/toolbin:$profile/bin" \
    /usr/bin/timeout --kill-after=2s 20s "$bin" --version >/dev/null

  [ -f "$runtime_root/config.toml" ] || die "Codex config was not materialized"
  [ -f "$runtime_root/RULES.md" ] || die "Codex rules were not materialized"
}

case "$phase" in
  detect|verify)
    validate_codex
    ;;
  install|fix)
    validate_codex || die "Codex installation belongs to merged FlexNetOS/yazelix origin/main"
    printf 'codex-profile: profile runtime already satisfies the envctl contract\n'
    ;;
  remove)
    printf 'codex-profile: nothing removed; envctl does not own profile Codex\n'
    ;;
  *)
    die "usage: envctl-codex-profile-lifecycle.sh detect|verify|install|fix|remove"
    ;;
esac
