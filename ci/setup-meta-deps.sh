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

workspace_version() {
  local loop_version protocol_version
  loop_version="$(locked_package_version loop_lib)"
  protocol_version="$(locked_package_version meta_plugin_protocol)"
  if [ -z "$loop_version" ] || [ -z "$protocol_version" ]; then
    echo "FAIL: unable to determine shared substrate versions from Cargo.lock" >&2
    exit 1
  fi
  if [ "$loop_version" != "$protocol_version" ]; then
    echo "FAIL: shared substrate lock versions diverge: loop_lib=$loop_version meta_plugin_protocol=$protocol_version" >&2
    exit 1
  fi
  printf '%s\n' "$loop_version"
}

locked_package_version() {
  local package="$1"
  awk -v package="$package" '
    $0 == "name = \"" package "\"" { in_package=1; next }
    in_package && /^version = "/ {
      gsub(/^version = "/, "")
      gsub(/"$/, "")
      print
      exit
    }
    in_package && /^\[\[/ { in_package=0 }
  ' "$root/Cargo.lock"
}

parent_workspace_is_compatible() {
  local manifest="$1"
  local expected_version="$2"
  local actual_version
  actual_version="$(awk '
    /^\[workspace\.package\]$/ { in_package=1; next }
    /^\[/ { in_package=0 }
    in_package && /^version[[:space:]]*=/ {
      value=$0
      sub(/^[^=]*=[[:space:]]*"/, "", value)
      sub(/".*$/, "", value)
      print value
      exit
    }
  ' "$manifest")"
  [ "$actual_version" = "$expected_version" ] || return 1

  awk '
    /^\[workspace\]$/ { in_workspace=1; next }
    /^\[/ { in_workspace=0 }
    in_workspace && /"loop_lib"/ { loop_lib=1 }
    in_workspace && /"meta_plugin_protocol"/ { protocol=1 }
    END { exit !(loop_lib && protocol) }
  ' "$manifest"
}

ensure_parent_workspace() {
  local manifest="$meta_root/Cargo.toml"
  local version
  version="$(workspace_version)"
  if [ -f "$manifest" ]; then
    if parent_workspace_is_compatible "$manifest" "$version"; then
      echo "meta-dep: validated parent workspace manifest at $manifest (version $version)"
      return
    fi
    echo "FAIL: refusing to overwrite unrelated or incompatible parent workspace at $manifest; require loop_lib + meta_plugin_protocol members and workspace version $version" >&2
    exit 1
  fi

  echo "meta-dep: writing minimal parent workspace manifest at $manifest"
  cat > "$manifest" <<EOF_MANIFEST
[workspace]
members = ["loop_lib", "meta_plugin_protocol"]
resolver = "2"

[workspace.package]
version = "$version"
edition = "2021"
license = "MIT"
repository = "https://github.com/FlexNetOS/meta"
EOF_MANIFEST
}

ensure_repo() {
  local repo="$1"
  local target="$meta_root/$repo"
  local url
  local ref="HEAD"
  url="$(repo_url "$repo")"
  if [ "$repo" = "loop_lib" ]; then
    ref="${LOOP_LIB_REF:-HEAD}"
  fi

  if git -C "$target" rev-parse --is-inside-work-tree >/dev/null 2>&1 &&
     [ "$(git -C "$target" rev-parse --show-toplevel)" = "$(cd "$target" && pwd -P)" ]; then
    if [ -f "$target/.git" ]; then
      echo "meta-dep: preserving linked worktree $repo at $target"
      test -f "$target/Cargo.toml" || { echo "FAIL: $repo missing Cargo.toml at $target" >&2; exit 1; }
      return
    fi
    echo "meta-dep: updating standalone checkout $repo at $target"
  elif [ -e "$target" ]; then
    echo "FAIL: refusing to replace non-repository sibling path $target" >&2
    exit 1
  else
    echo "meta-dep: cloning $repo to $target"
    GIT_TERMINAL_PROMPT=0 git clone --depth=1 "$url" "$target"
  fi
  # A pinned upgrade-branch ref can vanish once its PR merges and origin reaps the
  # branch (delete_branch_on_merge) — that must degrade to the default branch, not
  # hard-fail every CI job in the repo. The pin still wins whenever it exists.
  if git -C "$target" fetch --depth=1 origin "$ref"; then
    git -C "$target" checkout --detach FETCH_HEAD
  elif [ "$ref" != "HEAD" ]; then
    echo "WARN: pinned ref $ref for $repo not found on origin (merged + reaped?); falling back to HEAD" >&2
    git -C "$target" fetch --depth=1 origin HEAD
    git -C "$target" checkout --detach FETCH_HEAD
  else
    echo "FAIL: unable to fetch $repo from origin" >&2
    exit 1
  fi

  test -f "$target/Cargo.toml" || { echo "FAIL: $repo missing Cargo.toml at $target" >&2; exit 1; }
}

ensure_repo loop_lib
ensure_repo meta_plugin_protocol
ensure_parent_workspace

# Prove the exact paths Cargo resolves from envctl's path dependencies exist.
test -f "$root/crates/engine/../../../loop_lib/Cargo.toml" \
  || { echo "FAIL: loop_lib path dependency is not materialized" >&2; exit 1; }
test -f "$root/crates/cli/../../../meta_plugin_protocol/Cargo.toml" \
  || { echo "FAIL: meta_plugin_protocol path dependency is not materialized" >&2; exit 1; }

echo "meta-deps ready under $meta_root"
