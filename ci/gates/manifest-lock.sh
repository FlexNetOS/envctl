#!/usr/bin/env bash
set -euo pipefail

ROOT="${ENVCTL_GATE_ROOT:-$(git rev-parse --show-toplevel)}"
MANIFEST_DIR="$ROOT/manifest"
BIN="${ENVCTL_BIN:-${CARGO_TARGET_DIR:-$ROOT/target}/debug/envctl}"

fail() { echo "MANIFEST LOCK GATE FAIL: $*" >&2; exit 1; }

if [ ! -x "$BIN" ]; then
  CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}" cargo build --locked -p envctl
fi
[ -x "$BIN" ] || fail "envctl binary unavailable at $BIN"

if ENVCTL_MANIFEST_DIR="$MANIFEST_DIR" "$BIN" lock --check --json --color never; then
  echo "MANIFEST LOCK GATE PASS"
else
  fail "manifest/envctl.lock does not match the declarative component roster"
fi
