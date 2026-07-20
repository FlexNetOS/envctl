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
    if [[ -L "$path" ]]; then
      mode=120000
      object="$(printf '%s' "$(readlink -- "$path")" | git hash-object --stdin)"
    elif [[ -f "$path" ]]; then
      if [[ -x "$path" ]]; then
        mode=100755
      else
        mode=100644
      fi
      object="$(git hash-object --no-filters -- "$path")"
    else
      # A path removed before the snapshot is no longer a live manifest input.
      continue
    fi
    printf '%s\0%s\0%s\0' "$path" "$mode" "$object"
  done < <(find manifest \( -type f -o -type l \) -print0 | sort -z)
}

before="$(hash_inputs | git hash-object --stdin)"
set +e
cargo run --locked -p envctl -- --color never lock --check
rc=$?
set -e
after="$(hash_inputs | git hash-object --stdin)"

[[ "$before" == "$after" ]] || fail "lock check mutated live manifest inputs"
[[ "$rc" -eq 0 ]] || fail "envctl lock --check exited $rc"

echo "MANIFEST-LOCK GATE PASS"
