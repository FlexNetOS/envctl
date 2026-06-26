#!/usr/bin/env bash
# setup-meta-deps.sh — materialize envctl's meta sibling path dependencies for CI.
#
# envctl intentionally depends on meta-owned Rust crates via sibling path deps
# (for example ../../../loop_lib from crates/engine). A GitHub checkout of only
# FlexNetOS/envctl is therefore not a valid build topology. This script recreates
# the minimal meta sibling layout without vendoring, deleting, or downgrading the
# dependencies.
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
meta_root="$(cd "$root/.." && pwd)"

token="${META_DEPS_TOKEN:-${GITHUB_TOKEN:-${GH_TOKEN:-}}}"
repo_url() {
  local repo="$1"
  if [ -n "$token" ]; then
    printf 'https://x-access-token:%s@github.com/FlexNetOS/%s.git\n' "$token" "$repo"
  else
    printf 'https://github.com/FlexNetOS/%s.git\n' "$repo"
  fi
}

ensure_repo() {
  local repo="$1"
  local target="$meta_root/$repo"
  local url
  url="$(repo_url "$repo")"

  if [ -d "$target/.git" ]; then
    echo "meta-dep: updating $repo at $target"
    git -C "$target" fetch --depth=1 origin HEAD
    git -C "$target" checkout --detach FETCH_HEAD
  else
    echo "meta-dep: cloning $repo to $target"
    rm -rf "$target"
    GIT_TERMINAL_PROMPT=0 git clone --depth=1 "$url" "$target"
  fi

  test -f "$target/Cargo.toml" || { echo "FAIL: $repo missing Cargo.toml at $target" >&2; exit 1; }
}

ensure_repo loop_lib
ensure_repo meta_plugin_protocol

# Prove the exact paths Cargo resolves from envctl's path dependencies exist.
test -f "$root/crates/engine/../../../loop_lib/Cargo.toml" \
  || { echo "FAIL: loop_lib path dependency is not materialized" >&2; exit 1; }
test -f "$root/crates/cli/../../../meta_plugin_protocol/Cargo.toml" \
  || { echo "FAIL: meta_plugin_protocol path dependency is not materialized" >&2; exit 1; }

echo "meta-deps ready under $meta_root"
