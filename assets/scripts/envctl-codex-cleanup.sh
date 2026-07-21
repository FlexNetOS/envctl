#!/usr/bin/env bash
# Compatibility entrypoint retained for manifests/tests. The single-owner model
# has no alternate Codex tree to mutate; verification delegates to the profile
# lifecycle.
set -euo pipefail
export PATH=/usr/bin:/bin

source_root="${ENVCTL_SOURCE_ROOT:-${META_ROOT:-}/src/envctl}"
lifecycle="$source_root/assets/scripts/envctl-codex-profile-lifecycle.sh"
[ -x "$lifecycle" ] || {
  printf 'codex-cleanup: missing profile lifecycle: %s\n' "$lifecycle" >&2
  exit 1
}

case "${1:-verify}" in
  verify) exec "$lifecycle" verify ;;
  clean)
    "$lifecycle" verify
    printf 'codex-cleanup: no alternate Codex state exists\n'
    ;;
  *) printf 'usage: envctl-codex-cleanup.sh verify|clean\n' >&2; exit 2 ;;
esac
