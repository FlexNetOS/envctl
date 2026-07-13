#!/usr/bin/env bash
set -euo pipefail

source_root="$(git rev-parse --show-toplevel)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

set_root="$tmp/set"
envctl="$set_root/envctl"
mkdir -p "$envctl/ci" "$envctl/crates/engine" "$envctl/crates/cli"
cp "$source_root/ci/setup-meta-deps.sh" "$envctl/ci/setup-meta-deps.sh"

git -C "$envctl" init -q -b main
git -C "$envctl" config user.name test
git -C "$envctl" config user.email test@example.invalid
{
  printf '[[package]]\n'
  printf 'name = "loop_lib"\n'
  printf 'version = "0.2.25"\n\n'
  printf '[[package]]\n'
  printf 'name = "meta_plugin_protocol"\n'
  printf 'version = "0.2.25"\n'
} >"$envctl/Cargo.lock"
git -C "$envctl" add Cargo.lock ci/setup-meta-deps.sh
git -C "$envctl" commit -qm seed

seed_linked_worktree() {
  local name="$1"
  local source="$tmp/sources/$name"
  mkdir -p "$source"
  git -C "$source" init -q -b main
  git -C "$source" config user.name test
  git -C "$source" config user.email test@example.invalid
  {
    printf '[package]\n'
    printf 'name = "%s"\n' "$name"
    printf 'version.workspace = true\n'
    printf 'edition.workspace = true\n'
    printf 'license.workspace = true\n'
    printf 'repository.workspace = true\n'
  } >"$source/Cargo.toml"
  git -C "$source" add Cargo.toml
  git -C "$source" commit -qm seed
  git -C "$source" worktree add -q "$set_root/$name" HEAD
}

seed_linked_worktree loop_lib
seed_linked_worktree meta_plugin_protocol

loop_head="$(git -C "$set_root/loop_lib" rev-parse HEAD)"
protocol_head="$(git -C "$set_root/meta_plugin_protocol" rev-parse HEAD)"

if ! (
  cd "$envctl"
  bash ci/setup-meta-deps.sh >"$tmp/setup.out" 2>"$tmp/setup.err"
); then
  sed -n '1,200p' "$tmp/setup.out" >&2
  sed -n '1,200p' "$tmp/setup.err" >&2
  echo "initial linked-worktree setup failed" >&2
  exit 1
fi

test -f "$set_root/loop_lib/.git"
test ! -d "$set_root/loop_lib/.git"
test -f "$set_root/meta_plugin_protocol/.git"
test ! -d "$set_root/meta_plugin_protocol/.git"
test "$(git -C "$set_root/loop_lib" rev-parse HEAD)" = "$loop_head"
test "$(git -C "$set_root/meta_plugin_protocol" rev-parse HEAD)" = "$protocol_head"
grep -Fq 'preserving linked worktree loop_lib' "$tmp/setup.out"
grep -Fq 'preserving linked worktree meta_plugin_protocol' "$tmp/setup.out"
grep -Fq 'members = ["loop_lib", "meta_plugin_protocol"]' "$set_root/Cargo.toml"
grep -Fq 'version = "0.2.25"' "$set_root/Cargo.toml"

sed -i 's/version = "0.2.25"/version = "0.2.22"/' "$set_root/Cargo.toml"
if ! (
  cd "$envctl"
  bash ci/setup-meta-deps.sh >"$tmp/refresh.out" 2>"$tmp/refresh.err"
); then
  sed -n '1,200p' "$tmp/refresh.out" >&2
  sed -n '1,200p' "$tmp/refresh.err" >&2
  echo "stale generated parent-workspace refresh failed" >&2
  exit 1
fi
grep -Fq 'refreshing stale generated parent workspace' "$tmp/refresh.out"
grep -Fq 'version = "0.2.25"' "$set_root/Cargo.toml"

{
  printf '[workspace]\n'
  printf 'members = ["unrelated"]\n\n'
  printf '[workspace.package]\n'
  printf 'version = "9.9.9"\n'
} >"$set_root/Cargo.toml"
before="$(sha256sum "$set_root/Cargo.toml" | cut -d' ' -f1)"
if (
  cd "$envctl"
  bash ci/setup-meta-deps.sh >"$tmp/refuse.out" 2>"$tmp/refuse.err"
); then
  echo "expected setup-meta-deps to refuse an unrelated parent workspace" >&2
  exit 1
fi
after="$(sha256sum "$set_root/Cargo.toml" | cut -d' ' -f1)"
test "$before" = "$after"
grep -Fq 'refusing to overwrite unrelated or incompatible parent workspace' "$tmp/refuse.err"

echo "test-setup-meta-deps: PASS"
