#!/usr/bin/env bash
# harness-scripts.sh — CI gate for the hand-authored harness tooling (Feature Forge continuity).
# These scripts are git-tracked, outside the agent-env pipeline, and guard against two real failure
# modes that bit the forge-loop: silent loop-state concatenation (cycle 5) and worktree/branch pileup
# (46-worktree mess). They are destructive/merge-affecting, so their safety invariants get a test.
#
# Runs the hermetic, network-free harness-script tests. Fail-closed: any test failing fails the gate.
set -euo pipefail
root="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"

bash "$root/scripts/tests/test-merge-driver.sh"
bash "$root/scripts/tests/test-reaper.sh"

echo "HARNESS-SCRIPTS GATE PASS"
