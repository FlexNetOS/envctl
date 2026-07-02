#!/usr/bin/env bash
# Hermetic tests for the FlexNetOS Codex runtime gate.
set -euo pipefail

ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
GATE="$ROOT/.codex/hooks/flexnetos-runtime-gate.sh"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

export FLEXNETOS_ROOT="$tmp/FlexNetOS"
export FLEXNETOS_GATE_STATE="$tmp/state"
export FLEXNETOS_GATE_LOG_DIR="$tmp/log"

mkdir -p "$FLEXNETOS_ROOT/src/yazelix"
git -C "$FLEXNETOS_ROOT/src/yazelix" init -q
printf 'seed\n' >"$FLEXNETOS_ROOT/src/yazelix/README.md"
git -C "$FLEXNETOS_ROOT/src/yazelix" add README.md
git -C "$FLEXNETOS_ROOT/src/yazelix" -c user.email=test@example.invalid -c user.name='Test User' commit -q -m seed

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

expect_fail() {
  if "$@"; then
    fail "expected failure: $*"
  fi
}

expect_ok() {
  "$@" || fail "expected success: $*"
}

profile_payload='{"cmd":"nix profile add --refresh --accept-flake-config github:luccahuguet/yazelix#yazelix"}'
expect_fail bash -c "printf '%s' '$profile_payload' | '$GATE' pre-tool-use"
[ -d "$FLEXNETOS_GATE_STATE/violations/open" ] || fail "violation directory missing"

status_payload='{"cmd":"/home/flexnetos/.nix-profile/bin/yzx status --versions"}'
expect_ok bash -c "printf '%s' '$status_payload' | '$GATE' pre-tool-use"

install_check_payload='{"cmd":"curl -fsSL https://raw.githubusercontent.com/luccahuguet/yazelix/main/shells/posix/install_check.sh | sh"}'
expect_ok bash -c "printf '%s' '$install_check_payload' | '$GATE' pre-tool-use"
expect_ok bash -c "printf '%s' '{\"cmd\":\"curl -fsSL https://raw.githubusercontent.com/luccahuguet/yazelix/main/shells/posix/install_check.sh | sh\",\"exit_code\":0}' | '$GATE' post-tool-use"
[ -f "$FLEXNETOS_GATE_STATE/proofs/yazelix_install_check.ok" ] || fail "install_check proof not recorded"

expect_fail bash -c "printf '%s' '$profile_payload' | '$GATE' pre-tool-use"

patch_payload="{\"tool\":\"apply_patch\",\"path\":\"$FLEXNETOS_ROOT/src/yazelix/README.md\"}"
expect_fail bash -c "printf '%s' '$patch_payload' | '$GATE' pre-tool-use"
expect_ok "$GATE" clear-violations test-reset

snapshot_path="$("$GATE" snapshot "$FLEXNETOS_ROOT/src/yazelix")"
[ -s "$snapshot_path" ] || fail "snapshot file not written"
expect_ok bash -c "printf '%s' '$patch_payload' | '$GATE' pre-tool-use"

proof_log="$tmp/yazelix-proof.log"
printf 'cargo nextest run --profile ci\nPASS\n' >"$proof_log"
expect_ok "$GATE" record-yazelix-source-proof "$proof_log"
expect_ok "$GATE" allow-installed-surface-mutation "test unlock"
expect_ok bash -c "printf '%s' '$profile_payload' | '$GATE' pre-tool-use"

expect_fail bash -c "printf '%s' '$patch_payload' | FLEXNETOS_GATE_STATE='$tmp/state2' FLEXNETOS_GATE_LOG_DIR='$tmp/log2' FLEXNETOS_ROOT='$FLEXNETOS_ROOT' '$GATE' pre-tool-use"
expect_fail bash -c "FLEXNETOS_GATE_STATE='$tmp/state2' FLEXNETOS_GATE_LOG_DIR='$tmp/log2' FLEXNETOS_ROOT='$FLEXNETOS_ROOT' '$GATE' stop"

expect_ok "$GATE" self-test
echo "PASS: FlexNetOS Codex runtime gate"
