#!/usr/bin/env bash
# canonical: scripts/tests/blueprint/t6_musl_static.sh
# T6 — static musl build smoke.
#   RED now:  the x86_64-unknown-linux-musl target is not installed and no static
#             musl build of envctl exists.
#   GREEN:    after R9 adds the musl static build, file(1) on the target binary
#             reports "statically linked".
#   flip-on:  once GREEN, wire beside ci/gates/no-c.sh (supply-chain / build).
#
# Read-only: inspects a build artifact with file(1); builds nothing.
set -uo pipefail

SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(git -C "$SELF_DIR" rev-parse --show-toplevel 2>/dev/null || echo "$SELF_DIR/../../..")"
BIN="${MUSL_BIN:-$REPO_ROOT/target/x86_64-unknown-linux-musl/release/envctl}"

echo "== T6: static musl binary smoke =="
echo "expected binary: $BIN"

if [ ! -x "$BIN" ]; then
  echo "FAIL: musl target binary absent (add target x86_64-unknown-linux-musl and build --release after R9)"
  echo "T6 RED"
  exit 1
fi

info="$(file "$BIN")"
echo "file: $info"
if printf '%s' "$info" | grep -qE 'statically linked'; then
  echo "PASS: binary is statically linked (no dynamic loader / no libc.so dependency)"
  echo "T6 GREEN"
  exit 0
fi

echo "FAIL: binary is not statically linked"
echo "T6 RED"
exit 1
