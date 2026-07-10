#!/usr/bin/env bash
# canonical: scripts/tests/blueprint/t2_envctl_db_fresh.sh
# T2 — envctl `db` subcommand smoke (fresh-box surface check).
#   RED now:  `envctl db --help` returns "unrecognized subcommand 'db'" (exit 2)
#             — the db verb does not exist yet.
#   GREEN:    after R2 wires the `db` verb into the CLI, `envctl db --help`
#             exits 0 and prints the db surface.
#   flip-on:  once GREEN, wire this beside the other script gates in ci/gates/*.
#
# Read-only smoke: only runs `--help`; touches no production data.
set -uo pipefail

ENVCTL="${ENVCTL_BIN:-/home/flexnetos/lifeos/usr/bin/envctl}"

echo "== T2: envctl db --help =="
echo "binary: $ENVCTL"
if [ ! -x "$ENVCTL" ] && ! command -v "$ENVCTL" >/dev/null 2>&1; then
  echo "FAIL: envctl binary not found/executable: $ENVCTL"
  echo "T2 RED"
  exit 1
fi

out="$("$ENVCTL" db --help 2>&1)"
rc=$?
echo "--- output (exit=$rc) ---"
printf '%s\n' "$out"
echo "-------------------------"

if [ "$rc" -ne 0 ]; then
  echo "FAIL: 'envctl db --help' exit=$rc (unrecognized subcommand until R2 wires the db verb)"
  echo "T2 RED"
  exit 1
fi

# Pin to the REAL shipped db surface (R2 landed: roots/query/refactor verbs).
# The original RED-authored pattern guessed 'blob|capture|store' and used
# 'usage[^\n]*db', where POSIX ERE reads [^\n] as "any char except \ or n" —
# "Usage: envctl db" contains an 'n', so it could never match the live output.
if ! printf '%s' "$out" | grep -qiE 'usage: envctl db|db (roots|query|refactor)'; then
  echo "FAIL: 'envctl db --help' exited 0 but printed no db surface text"
  echo "T2 RED"
  exit 1
fi

echo "PASS: 'envctl db --help' exits 0 and prints the db surface"
echo "T2 GREEN"
exit 0
