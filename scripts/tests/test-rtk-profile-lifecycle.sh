#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
LIFECYCLE="$ROOT/assets/scripts/envctl-rtk-profile-lifecycle.sh"
tmp="$(mktemp -d -t envctl-rtk-profile.XXXXXXXX)"
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
package="$store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-rtk"
profile_store="$store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-profile"
install -d -m 755 "$home" "$package/bin" "$profile_store/bin" "$profile_store/toolbin"
printf '#!/bin/sh\n[ "${1:-}" = "--version" ] || exit 2\nprintf "%s\\n" "rtk fixture"\n' \
  >"$package/bin/rtk"
chmod 755 "$package/bin/rtk"
ln -s "$package/bin/rtk" "$profile_store/bin/rtk"
ln -s "$package/bin/rtk" "$profile_store/toolbin/rtk"
ln -s "$profile_store" "$home/profile-generation"
ln -s "$home/profile-generation" "$home/.nix-profile"

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
[ -x "$home/.nix-profile/bin/rtk" ] || fail 'remove changed profile RTK'

foreign="$store/cccccccccccccccccccccccccccccccc-foreign"
install -d -m 755 "$foreign"
printf '#!/bin/sh\nexit 0\n' >"$foreign/rtk"
chmod 755 "$foreign/rtk"
rm "$profile_store/toolbin/rtk"
ln -s "$foreign/rtk" "$profile_store/toolbin/rtk"
expect_failure divergent-frontdoors run_lifecycle verify

python3 - "$ROOT/manifest/base.toml" <<'PY'
import pathlib
import sys
import tomllib

components = {c["id"]: c for c in tomllib.loads(pathlib.Path(sys.argv[1]).read_text())["component"]}
rtk = components["rtk"]
for phase in ("detect", "install", "verify", "fix", "remove"):
    assert rtk[phase] == {
        "kind": "shipped_script",
        "path": "$ENVCTL_SOURCE_ROOT/assets/scripts/envctl-rtk-profile-lifecycle.sh",
        "args": [phase],
    }
PY

bash -n "$LIFECYCLE"
printf '%s\n' 'PASS: RTK is validated only through one profile payload'
