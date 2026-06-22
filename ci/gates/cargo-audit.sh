#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

fail() {
  printf 'CARGO-AUDIT GATE FAIL: %s\n' "$*" >&2
  exit 1
}

command -v cargo-audit >/dev/null 2>&1 || fail "cargo-audit is not installed"

# RUSTSEC-2023-0071 (rsa Marvin timing side channel) has no fixed stable release in the
# RustCrypto rsa 0.9 line.
#
# Every other vulnerability, including CVE-2024-47609 / GHSA-4jwc-w2hc-78qv in tonic, must fail.
cargo audit \
  --ignore RUSTSEC-2023-0071

echo "CARGO-AUDIT GATE PASS"
