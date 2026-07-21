#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

bash "$ROOT/scripts/tests/test-yazelix-codex-ownership-gate.sh"
bash "$ROOT/scripts/tests/test-nushell-rtk-ownership.sh"
bash "$ROOT/scripts/tests/test-rtk-command-policy.sh"

if matches="$(rg -n '/home/flexnetos/lifeos|/home/flexnetos/meta/lifeos' \
  "$ROOT/assets/scripts/envctl-claude-cleanup.sh" \
  "$ROOT/assets/scripts/envctl-codex-profile-lifecycle.sh" \
  "$ROOT/home/bin/harness-halt.sh")"; then
  printf '%s\n' "$matches" >&2
  printf '%s\n' "active Claude harness owners must not recreate retired LifeOS authority paths" >&2
  exit 1
fi

printf '%s\n' "ok - active Claude harness owners use the Meta root"
