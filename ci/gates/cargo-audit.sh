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
# RUSTSEC-2026-0009 (time stack-exhaustion DoS) is fixed in time >=0.3.47, but that line's
# manifest requires Cargo's Edition-2024 support and fails the workspace's Rust/Cargo 1.80 MSRV
# gate. Keep this exception explicit until the workspace floor is raised or the cert stack can move
# to a fixed, 1.80-compatible dependency path.
#
# Every other vulnerability, including CVE-2024-47609 / GHSA-4jwc-w2hc-78qv in tonic, must fail.
cargo audit \
  --ignore RUSTSEC-2023-0071 \
  --ignore RUSTSEC-2026-0009

echo "CARGO-AUDIT GATE PASS"
