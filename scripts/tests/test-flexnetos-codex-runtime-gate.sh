#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
STRICT_GATE="$ROOT/ci/gates/strict-profile-owner.sh"

fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }

for script in \
  envctl-yazelix-profile-lifecycle.sh \
  envctl-codex-profile-lifecycle.sh \
  envctl-rtk-profile-lifecycle.sh \
  envctl-claude-cleanup.sh \
  envctl-codex-global-baseline-lifecycle.sh; do
  path="$ROOT/assets/scripts/$script"
  [ -x "$path" ] || fail "missing executable validator: $path"
  bash -n "$path"
done

dot_codex=".$(printf '%s' codex)"
dot_claude=".$(printf '%s' claude)"
if find "$ROOT/home/$dot_codex" -type f -print -quit 2>/dev/null | grep -q .; then
  fail 'retired Codex home projection contains files'
fi
if find "$ROOT/home/$dot_claude" -type f -print -quit 2>/dev/null | grep -q .; then
  fail 'retired Claude home projection contains files'
fi

for lifecycle in \
  "$ROOT/assets/scripts/envctl-yazelix-profile-lifecycle.sh" \
  "$ROOT/assets/scripts/envctl-codex-profile-lifecycle.sh" \
  "$ROOT/assets/scripts/envctl-rtk-profile-lifecycle.sh" \
  "$ROOT/assets/scripts/envctl-claude-cleanup.sh"; do
  grep -Fq '.nix-profile' "$lifecycle" \
    || fail "validator does not use the profile selector: $lifecycle"
done

"$STRICT_GATE" "$ROOT"
printf '%s\n' 'PASS: installed agent runtimes have one profile owner'
