#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
LIFECYCLE="$ROOT/assets/scripts/envctl-codex-profile-lifecycle.sh"
tmp="$(mktemp -d -t envctl-codex-profile.XXXXXXXX)"
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
package="$store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-codex-cli"
profile_store="$store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-profile"
runtime="$tmp/runtime"
install -d -m 755 \
  "$home" "$package/bin" "$profile_store/bin" "$profile_store/toolbin" \
  "$profile_store/share/yazelix/agent_configs/codex" "$runtime"

cat >"$package/bin/codex" <<'CODEX'
#!/bin/sh
set -eu
[ "${1:-}" = "--version" ] || exit 2
/usr/bin/mkdir -p "${CODEX_HOME:?}"
printf 'model = "fixture"\n' >"$CODEX_HOME/config.toml"
printf '# fixture rules\n' >"$CODEX_HOME/RULES.md"
printf '%s\n' 'codex-cli fixture'
CODEX
chmod 755 "$package/bin/codex"
ln -s "$package/bin/codex" "$profile_store/bin/codex"
ln -s "$package/bin/codex" "$profile_store/toolbin/codex"
printf 'model = "fixture"\n' \
  >"$profile_store/share/yazelix/agent_configs/codex/config.toml.src"
printf '# fixture rules\n' \
  >"$profile_store/share/yazelix/agent_configs/codex/RULES.md.src"
ln -s "$profile_store" "$home/profile-generation"
ln -s "$home/profile-generation" "$home/.nix-profile"

run_lifecycle() {
  env -i \
    HOME="$home" \
    ENVCTL_REAL_HOME="$home" \
    ENVCTL_NIX_STORE_ROOT="$store" \
    XDG_RUNTIME_DIR="$runtime" \
    PATH=/usr/bin:/bin \
    "$LIFECYCLE" "$@"
}

run_lifecycle detect
run_lifecycle verify
run_lifecycle install | grep -Fq 'already satisfies'
run_lifecycle fix | grep -Fq 'already satisfies'
run_lifecycle remove | grep -Fq 'nothing removed'
[ -x "$home/.nix-profile/bin/codex" ] || fail 'remove changed profile Codex'
[ -f "$runtime/yazelix/profile-runtime/codex/config.toml" ] \
  || fail 'Codex config did not materialize into volatile runtime'
[ -f "$runtime/yazelix/profile-runtime/codex/RULES.md" ] \
  || fail 'Codex rules did not materialize into volatile runtime'

foreign="$store/cccccccccccccccccccccccccccccccc-foreign"
install -d -m 755 "$foreign"
printf '#!/bin/sh\nexit 0\n' >"$foreign/codex"
chmod 755 "$foreign/codex"
rm "$profile_store/toolbin/codex"
ln -s "$foreign/codex" "$profile_store/toolbin/codex"
expect_failure divergent-frontdoors run_lifecycle verify

python3 - "$ROOT/manifest/ai-clis.toml" <<'PY'
import pathlib
import sys
import tomllib

components = {c["id"]: c for c in tomllib.loads(pathlib.Path(sys.argv[1]).read_text())["component"]}
codex = components["codex-cli"]
for phase in ("detect", "install", "verify", "fix", "remove"):
    assert codex[phase] == {
        "kind": "shipped_script",
        "path": "$ENVCTL_SOURCE_ROOT/assets/scripts/envctl-codex-profile-lifecycle.sh",
        "args": [phase],
    }
PY

bash -n "$LIFECYCLE"
printf '%s\n' 'PASS: Codex is validated only through one profile and volatile runtime'
