#!/usr/bin/env bash
set -euo pipefail

fail() { echo "FAIL: $*" >&2; exit 1; }
root="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
gate="$root/ci/gates/manifest-lock.sh"
bin="${ENVCTL_BIN:-${CARGO_TARGET_DIR:-$root/target}/debug/envctl}"
[ -x "$bin" ] || CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$root/target}" cargo build --locked -p envctl

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/repo"
cp -a "$root/manifest" "$tmp/repo/manifest"
ENVCTL_MANIFEST_DIR="$tmp/repo/manifest" "$bin" lock --json --color never >/dev/null
ENVCTL_GATE_ROOT="$tmp/repo" ENVCTL_BIN="$bin" bash "$gate" >/dev/null

sed -i '0,/^name = /s//name = "changed fixture" # /' "$tmp/repo/manifest/base.toml"
if ENVCTL_GATE_ROOT="$tmp/repo" ENVCTL_BIN="$bin" bash "$gate" >"$tmp/out" 2>"$tmp/err"; then
  fail "gate accepted a changed component"
fi
grep -q 'MANIFEST LOCK GATE FAIL' "$tmp/err" || fail "changed failure was unclear"

cp "$root/manifest/base.toml" "$tmp/repo/manifest/base.toml"
cat >"$tmp/repo/manifest/components.d/fixture-added.toml" <<'TOML'
[[component]]
id = "fixture-added"
name = "fixture-added"
[component.detect]
kind = "command"
command = "true"
TOML
if ENVCTL_GATE_ROOT="$tmp/repo" ENVCTL_BIN="$bin" bash "$gate" >"$tmp/out" 2>"$tmp/err"; then
  fail "gate accepted an added component"
fi

rm "$tmp/repo/manifest/components.d/fixture-added.toml"
rm "$tmp/repo/manifest/components.d/envctl-cli.toml"
if ENVCTL_GATE_ROOT="$tmp/repo" ENVCTL_BIN="$bin" bash "$gate" >"$tmp/out" 2>"$tmp/err"; then
  fail "gate accepted a removed component"
fi

echo "PASS: manifest lock gate rejects changed, added, and removed components"
