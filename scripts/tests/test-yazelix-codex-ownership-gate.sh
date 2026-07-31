#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
YAZELIX="$ROOT/assets/scripts/envctl-yazelix-profile-lifecycle.sh"
CODEX="$ROOT/assets/scripts/envctl-codex-profile-lifecycle.sh"
CLAUDE="$ROOT/assets/scripts/envctl-claude-cleanup.sh"

fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }

grep -Fq 'clean, merged FlexNetOS/yazelix' "$YAZELIX" \
  || fail 'Yazelix validator does not name the canonical cutover owner'
grep -Fq 'envctl is deliberately a read-only bridge' "$YAZELIX" \
  || fail 'envctl read-only boundary is missing'

for lifecycle in "$YAZELIX" "$CODEX" "$CLAUDE"; do
  if grep -Eq 'nix (build|profile)|git (checkout|switch|pull)|profile (install|remove)' "$lifecycle"; then
    fail "validator can mutate or switch the installed profile: $lifecycle"
  fi
done

grep -Fq '/home/flexnetos/meta/var/lib/codex' "$CODEX" \
  || fail 'Codex Yazelix-owned state is not explicit'
grep -Fq '/home/flexnetos/meta/var/lib/claude' "$CLAUDE" \
  || fail 'Claude Yazelix-owned state is not explicit'

printf '%s\n' 'PASS: envctl cannot perform the Yazelix source-repository cutover'
