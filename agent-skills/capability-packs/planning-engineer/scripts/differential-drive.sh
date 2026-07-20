#!/usr/bin/env bash
# differential-drive.sh — drive the REAL binary/CLI for each declared case and diff its output against a
# golden/expected. The planning-engineer harness's P8 "differential-drive live case": the owner's
# HFTASK-0078 doctrine that "green unit tests are not proof" — a unit test can pass while the shipped
# binary misbehaves, so the plan's acceptance MUST include driving the real CLI and diffing its output.
#
# FAIL-CLOSED: zero cases run is a FAIL (the tests-ran > 0 / parse_tests_ran gate), never a silent pass.
#
# Usage: differential-drive.sh [CASES_FILE]
#   default CASES_FILE: ./scripts/differential-drive.cases.sh or ./differential-drive.cases.sh
# The cases file (authored per-target by plan-test-strategist) declares cases by calling:
#   dd_expect "<expected output>"          # optional: inline expected for the next case with golden "-"
#   dd_case "<name>" "<command...>" "<golden-file | ->"
#     - <command> is run through the shell; its combined output (add `2>&1` in the command for stderr)
#       is the ACTUAL output of the REAL binary/CLI.
#     - <golden-file> holds the EXPECTED output; "-" uses the value set by the preceding dd_expect.
# Exit: 0 = all cases ran AND matched · 1 = a case mismatched · 2 = zero cases (fail-closed) · 3 = usage.
set -uo pipefail

DD_RAN=0
DD_FAIL=0
_dd_expected=""

dd_expect() { _dd_expected="$1"; }

dd_case() {
  local name="$1" cmd="$2" golden="${3:-}"
  DD_RAN=$((DD_RAN + 1))
  local actual expected
  actual="$(eval "$cmd" 2>&1)"
  if [ "$golden" = "-" ]; then
    expected="$_dd_expected"; _dd_expected=""
  elif [ -n "$golden" ] && [ -f "$golden" ]; then
    expected="$(cat "$golden")"
  else
    DD_FAIL=$((DD_FAIL + 1)); echo "FAIL[$name] (case $DD_RAN): golden not found: '$golden'" >&2; return
  fi
  if [ "$actual" = "$expected" ]; then
    echo "PASS[$name] (case $DD_RAN)"
  else
    DD_FAIL=$((DD_FAIL + 1))
    echo "FAIL[$name] (case $DD_RAN): real-CLI output differs from golden" >&2
    diff <(printf '%s\n' "$expected") <(printf '%s\n' "$actual") 2>/dev/null | sed 's/^/    /' >&2 || true
  fi
}

CASES="${1:-}"
if [ -z "$CASES" ]; then
  for c in ./scripts/differential-drive.cases.sh ./differential-drive.cases.sh; do
    [ -f "$c" ] && CASES="$c" && break
  done
fi
[ -n "$CASES" ] && [ -f "$CASES" ] || {
  echo "differential-drive: no cases file (looked for scripts/differential-drive.cases.sh)" >&2; exit 3
}

# shellcheck source=/dev/null
source "$CASES"

# Fail-closed: a cases file that declared zero cases is NOT a pass — the parse_tests_ran / tests-ran>0 gate.
if [ "$DD_RAN" -eq 0 ]; then
  echo "differential-drive: FAIL — 0 cases ran (tests-ran must be > 0)" >&2; exit 2
fi
echo "differential-drive: tests-ran=$DD_RAN failed=$DD_FAIL"
[ "$DD_FAIL" -eq 0 ] || exit 1
exit 0
