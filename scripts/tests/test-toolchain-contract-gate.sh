#!/usr/bin/env bash
set -euo pipefail

fail() { echo "FAIL: $*" >&2; exit 1; }
root="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
gate="$root/ci/gates/toolchain-contract.sh"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/manifest" "$tmp/.github/workflows"
cp "$root/Cargo.toml" "$tmp/Cargo.toml"
cp "$root/rust-toolchain.toml" "$tmp/rust-toolchain.toml"
cp "$root/manifest/base.toml" "$tmp/manifest/base.toml"
cp "$root/.github/workflows/ci.yml" "$tmp/.github/workflows/ci.yml"

ENVCTL_GATE_ROOT="$tmp" bash "$gate" >/dev/null

sed -i 's/cargo +1\.89\.0 check/cargo check/' "$tmp/.github/workflows/ci.yml"
if ENVCTL_GATE_ROOT="$tmp" bash "$gate" >"$tmp/out" 2>"$tmp/err"; then
  fail "gate accepted a newer-runner floor check in place of exact MSRV compilation"
fi
grep -q 'exact Rust 1.89.0' "$tmp/err" || fail "exact-MSRV failure was unclear"

cp "$root/.github/workflows/ci.yml" "$tmp/.github/workflows/ci.yml"
sed -i '/rustup toolchain install 1\.89\.0/d' "$tmp/manifest/base.toml"
if ENVCTL_GATE_ROOT="$tmp" bash "$gate" >"$tmp/out" 2>"$tmp/err"; then
  fail "gate accepted a rustup component without the exact MSRV toolchain"
fi
grep -q 'provision exact MSRV' "$tmp/err" || fail "manifest failure was unclear"

cp "$root/manifest/base.toml" "$tmp/manifest/base.toml"
sed -i '0,/ln -sfn rustup/{s|ln -sfn rustup|ln -sfn "$CARGO_HOME/bin/rustup"|}' "$tmp/manifest/base.toml"
if ENVCTL_GATE_ROOT="$tmp" bash "$gate" >"$tmp/out" 2>"$tmp/err"; then
  fail "gate accepted an absolute, checkout-specific rustup proxy target"
fi
grep -q 'relocation-safe relative tool proxies' "$tmp/err" \
  || fail "rustup proxy failure was unclear"

echo "PASS: toolchain contract requires exact Rust 1.89.0 while keeping nightly default"
