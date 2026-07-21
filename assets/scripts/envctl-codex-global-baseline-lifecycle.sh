#!/usr/bin/env bash
# Compatibility component for the former envctl-owned Codex baseline. Reviewed
# configuration now ships in the Yazelix profile and materializes only into the
# volatile profile runtime.
set -euo pipefail
export PATH=/usr/bin:/bin

die() {
  printf 'codex-baseline: %s\n' "$*" >&2
  exit 1
}

phase="${1:-verify}"
source_root="${ENVCTL_SOURCE_ROOT:-${META_ROOT:-}/src/envctl}"
lifecycle="$source_root/assets/scripts/envctl-codex-profile-lifecycle.sh"
[ -x "$lifecycle" ] || die "missing profile lifecycle: $lifecycle"

case "$phase" in
  detect|verify)
    exec "$lifecycle" verify
    ;;
  install|fix)
    "$lifecycle" "$phase"
    printf 'codex-baseline: configuration is profile-owned and current\n'
    ;;
  remove)
    printf 'codex-baseline: nothing removed; configuration is profile-owned\n'
    ;;
  *)
    die "usage: envctl-codex-global-baseline-lifecycle.sh detect|verify|install|fix|remove"
    ;;
esac
