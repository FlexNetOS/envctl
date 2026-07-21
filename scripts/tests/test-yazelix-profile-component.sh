#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
LIFECYCLE="$ROOT/assets/scripts/envctl-yazelix-profile-lifecycle.sh"
tmp="$(mktemp -d -t envctl-yazelix-profile.XXXXXXXX)"
trap '/usr/bin/rm -rf --one-file-system -- "$tmp"' EXIT

fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
expect_failure() {
  local name="$1"
  shift
  if "$@" >"$tmp/$name.out" 2>"$tmp/$name.err"; then
    fail "expected failure: $name"
  fi
}

home="$tmp/home"
store="$tmp/nix/store"
package="$store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-yazelix"
profile_store="$store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-profile"
install -d -m 755 "$home" "$package/bin" "$profile_store/bin" "$profile_store/toolbin"
printf '#!/bin/sh\n[ "${1:-}" = "--version" ] || exit 2\nprintf "%s\\n" "yzx fixture"\n' \
  >"$package/bin/yzx"
chmod 755 "$package/bin/yzx"
ln -s "$package/bin/yzx" "$profile_store/bin/yzx"
ln -s "$package/bin/yzx" "$profile_store/toolbin/yzx"
ln -s "$profile_store" "$home/.nix-profile-1-link"
ln -s '.nix-profile-1-link' "$home/.nix-profile"

run_lifecycle() {
  env -i \
    HOME="$home" \
    ENVCTL_REAL_HOME="$home" \
    ENVCTL_NIX_STORE_ROOT="$store" \
    PATH=/usr/bin:/bin \
    "$LIFECYCLE" "$@"
}

run_lifecycle detect
run_lifecycle verify
run_lifecycle install | grep -Fq 'already satisfies'
run_lifecycle fix | grep -Fq 'already satisfies'
run_lifecycle remove | grep -Fq 'nothing removed'
[ -L "$home/.nix-profile" ] || fail 'remove changed profile selector'

rm "$home/.nix-profile"
ln -s "$profile_store" "$home/.nix-profile"
expect_failure indirect-selector run_lifecycle verify

python3 - "$ROOT/manifest/nix-yazelix.toml" <<'PY'
import pathlib
import sys
import tomllib

components = {c["id"]: c for c in tomllib.loads(pathlib.Path(sys.argv[1]).read_text())["component"]}
yzx = components["yazelix"]
for phase in ("install", "fix", "remove"):
    assert yzx[phase] == {
        "kind": "shipped_script",
        "path": "$ENVCTL_SOURCE_ROOT/assets/scripts/envctl-yazelix-profile-lifecycle.sh",
        "args": [phase],
    }
for phase in ("detect", "verify"):
    assert yzx[phase]["kind"] == "command"
    assert "envctl-yazelix-profile-lifecycle.sh" in yzx[phase]["args"][1]
assert "read-only" in yzx["description"].lower()
PY

bash -n "$LIFECYCLE"
printf '%s\n' 'PASS: envctl validates Yazelix and never performs the source-repository cutover'
