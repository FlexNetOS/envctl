#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
LIFECYCLE="$ROOT/assets/scripts/envctl-codex-global-baseline-lifecycle.sh"
PROFILE_LIFECYCLE="$ROOT/assets/scripts/envctl-codex-profile-lifecycle.sh"
MANIFEST="$ROOT/manifest/components.d/codex-global-baseline.toml"

fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }

bash -n "$LIFECYCLE"
grep -Fq 'envctl-codex-profile-lifecycle.sh' "$LIFECYCLE" \
  || fail 'global baseline does not delegate to the profile validator'
if grep -Eq 'nix (build|profile)|profile (install|remove)|git (checkout|switch)|rm -rf|ln -s|cp -a' \
    "$LIFECYCLE"; then
  fail 'global baseline contains an installation, cutover, or mutation primitive'
fi

python3 - "$MANIFEST" <<'PY'
import pathlib
import sys
import tomllib

component = tomllib.loads(pathlib.Path(sys.argv[1]).read_text())["component"][0]
assert component["id"] == "codex-global-baseline"
for phase in ("detect", "install", "verify", "fix", "remove"):
    assert component[phase] == {
        "kind": "shipped_script",
        "path": "$ENVCTL_SOURCE_ROOT/assets/scripts/envctl-codex-global-baseline-lifecycle.sh",
        "args": [phase],
    }
PY

[ -x "$PROFILE_LIFECYCLE" ] || fail 'profile validator is not executable'
printf '%s\n' 'PASS: Codex baseline is a read-only profile compatibility component'
