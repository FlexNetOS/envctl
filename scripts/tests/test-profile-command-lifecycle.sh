#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
LIFECYCLE="$ROOT/assets/scripts/envctl-profile-command-lifecycle.sh"
tmp="$(mktemp -d -t envctl-profile-commands.XXXXXXXX)"
trap '/usr/bin/rm -rf --one-file-system -- "$tmp"' EXIT

fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }

home="$tmp/home"
store="$tmp/nix/store"
package="$store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-ai-clis"
profile_store="$store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-profile"
install -d -m 755 "$home" "$package/bin" "$profile_store/bin" "$profile_store/toolbin"

for command_name in gemini kimi devin; do
  printf '#!/bin/sh\n[ "${1:-}" = "--version" ] || exit 2\nprintf "%%s\\n" "%s fixture"\n' \
    "$command_name" >"$package/bin/$command_name"
  chmod 755 "$package/bin/$command_name"
  ln -s "$package/bin/$command_name" "$profile_store/bin/$command_name"
  ln -s "$package/bin/$command_name" "$profile_store/toolbin/$command_name"
done
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

for command_name in gemini kimi devin; do
  run_lifecycle "$command_name" detect
  run_lifecycle "$command_name" verify
  run_lifecycle "$command_name" install | grep -Fq 'already satisfies'
  run_lifecycle "$command_name" fix | grep -Fq 'already satisfies'
  run_lifecycle "$command_name" remove | grep -Fq 'nothing removed'
  [ -x "$home/.nix-profile/bin/$command_name" ] \
    || fail "remove changed profile command: $command_name"
done

python3 - "$ROOT/manifest/ai-clis.toml" <<'PY'
import pathlib
import sys
import tomllib

components = {c["id"]: c for c in tomllib.loads(pathlib.Path(sys.argv[1]).read_text())["component"]}
for component_id, command_name in (("gemini-cli", "gemini"), ("kimi-cli", "kimi"), ("devin-cli", "devin")):
    component = components[component_id]
    for phase in ("detect", "install", "verify", "fix", "remove"):
        assert component[phase] == {
            "kind": "shipped_script",
            "path": "$ENVCTL_SOURCE_ROOT/assets/scripts/envctl-profile-command-lifecycle.sh",
            "args": [command_name, phase],
        }
PY

bash -n "$LIFECYCLE"
printf '%s\n' 'PASS: Gemini, Kimi, and Devin are profile-validation-only components'
