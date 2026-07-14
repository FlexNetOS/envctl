#!/usr/bin/env bash
set -euo pipefail

ROOT="${ENVCTL_GATE_ROOT:-$(git rev-parse --show-toplevel)}"
fail() { echo "TOOLCHAIN CONTRACT GATE FAIL: $*" >&2; exit 1; }

grep -Fq 'rust-version = "1.89"' "$ROOT/Cargo.toml" \
  || fail "Cargo.toml must declare rust-version 1.89"
grep -Fq 'channel = "nightly"' "$ROOT/rust-toolchain.toml" \
  || fail "developer/default repo toolchain must remain nightly"
grep -Fq 'rustup toolchain install 1.89.0' "$ROOT/manifest/base.toml" \
  || fail "rustup component must provision exact MSRV toolchain 1.89.0"
grep -Fq 'rustup default nightly' "$ROOT/manifest/base.toml" \
  || fail "rustup component must retain nightly as the default"
[ "$(grep -Fc 'ln -sfn rustup "$CARGO_HOME/bin/$tool"' "$ROOT/manifest/base.toml")" -eq 2 ] \
  || fail "rustup install/fix must create relocation-safe relative tool proxies"
! grep -Fq 'rustfmt rustup; do' "$ROOT/manifest/base.toml" \
  || fail "rustup proxy loop must not replace the rustup executable with a self-link"
grep -Fq 'cargo +1.89.0 check --workspace --locked' "$ROOT/.github/workflows/ci.yml" \
  || fail "CI must compile with exact Rust 1.89.0, not only test a newer compiler floor"

echo "TOOLCHAIN CONTRACT GATE PASS"
