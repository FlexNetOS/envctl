#!/usr/bin/env bash
# meta-substrates.sh — fail-closed guard for envctl's shared meta substrate wiring.
#
# envctl is the meta environment manager, not an island. It may wrap or extend shared
# substrates, and it may upgrade those substrates, but it must not silently downgrade
# itself by removing/bypassing them.
set -euo pipefail

fail() {
  echo "META-SUBSTRATES GATE FAIL: $*" >&2
  exit 1
}

test -f crates/engine/Cargo.toml || fail "run from repo root"
test -f crates/cli/Cargo.toml || fail "run from repo root"

bash scripts/tests/test-setup-meta-deps.sh \
  || fail "linked-worktree/meta-parent setup regression"

grep -Eq '^[[:space:]]*loop_lib[[:space:]]*=[[:space:]]*\{.*path[[:space:]]*=[[:space:]]*"../../../loop_lib"' crates/engine/Cargo.toml \
  || fail "envctl-engine must keep loop_lib as a ../../../loop_lib path dependency; upgrade loop_lib instead of bypassing it"
grep -Eq '^[[:space:]]*meta_plugin_protocol[[:space:]]*=[[:space:]]*\{.*path[[:space:]]*=[[:space:]]*"../../../meta_plugin_protocol"' crates/cli/Cargo.toml \
  || fail "envctl CLI must keep meta_plugin_protocol as a ../../../meta_plugin_protocol path dependency"

grep -q 'use loop_lib::' crates/engine/src/runner.rs \
  || fail "runner must import loop_lib; envctl may wrap supervision but must not own shared Command construction"
grep -q 'build_command as loop_build_command' crates/engine/src/runner.rs \
  || fail "runner must call loop_lib::build_command (aliased loop_build_command)"
grep -q 'SpawnSpec' crates/engine/src/runner.rs \
  || fail "runner must construct loop_lib::SpawnSpec"
grep -q 'loop_build_command(&SpawnSpec' crates/engine/src/runner.rs \
  || fail "runner must delegate Command construction to loop_lib"

grep -q 'ensure_repo loop_lib' ci/setup-meta-deps.sh \
  || fail "CI must materialize loop_lib sibling dependency"
grep -q 'ensure_repo meta_plugin_protocol' ci/setup-meta-deps.sh \
  || fail "CI must materialize meta_plugin_protocol sibling dependency"
grep -q 'loop_lib path dependency is not materialized' ci/setup-meta-deps.sh \
  || fail "CI must prove loop_lib path dependency exists"
grep -q 'meta_plugin_protocol path dependency is not materialized' ci/setup-meta-deps.sh \
  || fail "CI must prove meta_plugin_protocol path dependency exists"
grep -q 'preserving linked worktree' ci/setup-meta-deps.sh \
  || fail "CI setup must preserve linked sibling worktrees whose .git is a file"
grep -q 'refusing to overwrite unrelated or incompatible parent workspace' ci/setup-meta-deps.sh \
  || fail "CI setup must refuse to overwrite unrelated parent workspaces"

metadata="${TMPDIR:-/tmp}/envctl-meta-substrates-metadata.json"
cargo metadata --format-version 1 --locked >"$metadata"
python3 - "$metadata" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
data = json.loads(path.read_text())
names = {pkg["name"] for pkg in data["packages"]}
missing = [name for name in ("loop_lib", "meta_plugin_protocol") if name not in names]
if missing:
    print(
        "META-SUBSTRATES GATE FAIL: missing resolved package(s): " + ", ".join(missing),
        file=sys.stderr,
    )
    sys.exit(1)
PY

echo "META-SUBSTRATES GATE PASS"
