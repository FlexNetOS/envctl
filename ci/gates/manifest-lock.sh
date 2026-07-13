#!/usr/bin/env bash
# manifest-lock.sh — fail-closed, non-mutating envctl.lock drift gate.
set -euo pipefail

root="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
cd "$root"

fail() {
  echo "MANIFEST-LOCK GATE FAIL: $*" >&2
  exit 1
}

hash_inputs() {
  while IFS= read -r -d '' path; do
    printf '%s\0' "$path"
    git hash-object -- "$path"
  done < <(git ls-files -z -- manifest)
}

before="$(hash_inputs | git hash-object --stdin)"
set +e
cargo run --locked -p envctl -- --color never lock --check
rc=$?
set -e
after="$(hash_inputs | git hash-object --stdin)"

[[ "$before" == "$after" ]] || fail "lock check mutated tracked manifest inputs"
[[ "$rc" -eq 0 ]] || fail "envctl lock --check exited $rc"

echo "MANIFEST-LOCK GATE PASS"
