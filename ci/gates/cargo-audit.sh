#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"
META_ROOT="${META_ROOT:-$(cd "$ROOT/.." && pwd)}"

META_CARGO_HOME="$META_ROOT/.toolchains/cargo"
export PATH="${CARGO_HOME:-$META_ROOT/.toolchains/cargo}/bin:$META_CARGO_HOME/bin:$PATH"

fail() {
  printf 'CARGO-AUDIT GATE FAIL: %s\n' "$*" >&2
  exit 1
}

command -v cargo-audit >/dev/null 2>&1 || fail "cargo-audit is not installed in ${CARGO_HOME:-$META_ROOT/.toolchains/cargo}/bin or $META_CARGO_HOME/bin"

# RUSTSEC-2023-0071 (rsa Marvin timing side channel) has no fixed stable release in the
# RustCrypto rsa 0.9 line.
#
# RUSTSEC-2025-0134 (rustls-pemfile unmaintained) has no replacement release in
# crates.io as of the PR 409 audit repair: cargo resolves 2.2.0 and `cargo info
# rustls-pemfile` reports 2.2.0 as latest. envctl keeps it narrowly for TLS PEM
# parsing in the secrets/edge crates until a replacement parser is selected.
#
# RUSTSEC-2026-0192 (ttf-parser unmaintained) has no patched release in crates.io:
# 0.25.1 is the latest visible version. The dependency is transitive through
# egui/epaint. eframe/egui 0.34+ removes this path but raises rust-version to
# 1.92, above envctl's current 1.88 MSRV; keep the exception narrow until the
# GUI stack and MSRV can move together.
#
# RUSTSEC-2026-0194 and RUSTSEC-2026-0195 are quick-xml parser advisories fixed
# in quick-xml >=0.41. The only remaining path is wayland-scanner 0.31.10, which
# is the latest visible crate and pins quick-xml = 0.39; eframe 0.33 hard-enables
# egui-winit clipboard/link support that reaches this scanner path. Keep these
# IDs explicit until the upstream Wayland/egui stack can resolve quick-xml >=0.41
# without raising envctl's MSRV beyond 1.88.
#
# Every other vulnerability, including CVE-2024-47609 / GHSA-4jwc-w2hc-78qv in tonic, must fail.
cargo audit \
  --ignore RUSTSEC-2023-0071 \
  --ignore RUSTSEC-2025-0134 \
  --ignore RUSTSEC-2026-0192 \
  --ignore RUSTSEC-2026-0194 \
  --ignore RUSTSEC-2026-0195

echo "CARGO-AUDIT GATE PASS"
