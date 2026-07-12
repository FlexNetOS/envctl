#!/usr/bin/env bash
# actionlint.sh — workflow-syntax/label gate. The repo ships .github/actionlint.yaml
# (custom runner labels) but nothing ran actionlint (audit 2026-07-12). Fail-closed when
# the linter is available; SKIP with a note when it is not (never false-block — the
# binary ships via the yazelix foundation profile; CodeQL actions analysis still covers
# workflow security in CI regardless).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
if ! command -v actionlint >/dev/null 2>&1; then
  echo "ACTIONLINT GATE SKIP — actionlint not on PATH (ships via yazelix foundation; CodeQL actions analysis still active)"
  exit 0
fi
actionlint -config-file "$ROOT/.github/actionlint.yaml" "$ROOT"/.github/workflows/*.yml
echo "ACTIONLINT GATE PASS"
