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
bash "$root/scripts/tests/test-agent-env-hooks.sh"
bash "$root/scripts/tests/test-loop-state-gate.sh"
bash "$root/scripts/tests/test-runner-routing.sh"
bash "$root/scripts/tests/test-meta-local-path-audit.sh"
bash "$root/scripts/tests/test-skill-contract.sh"
bash "$root/scripts/tests/test-plan-eject.sh"
bash "$root/scripts/tests/test-plan-contract.sh"
bash "$root/scripts/tests/test-plan-loop-state.sh"
bash "$root/scripts/tests/test-plan-weave-dispatch.sh"
bash "$root/scripts/tests/test-plan-artifact-gate.sh"
bash "$root/scripts/tests/test-plan-evals.sh"
bash "$root/scripts/tests/test-codex-cli-release-lanes.sh"
bash "$root/scripts/tests/test-codex-profile-lifecycle.sh"
bash "$root/scripts/tests/test-codex-global-baseline-lifecycle.sh"
bash "$root/scripts/tests/test-flexnetos-codex-runtime-gate.sh"
bash "$root/scripts/tests/test-manifest-lock-gate.sh"

# Guard the owner-critical publish contract. The harness must never regress to
# committed-but-unpushed local work; every committed chunk is pushed, PR-backed,
# and auto-merge armed immediately.
if grep -RIn "push unless asked" "$root/.agents" "$root/.claude" "$root/.codex" >/tmp/envctl-publish-contract-grep.txt 2>/dev/null; then
  cat /tmp/envctl-publish-contract-grep.txt >&2
  echo "FAIL: forbidden publish-contract wording found" >&2
  exit 1
fi

for skill in \
  "$root/agent-skills/feature-forge/SKILL.md" \
  "$root/.claude/skills/feature-forge/SKILL.md"; do
  grep -q "gh pr create --fill" "$skill" || { echo "FAIL: $skill missing PR-create publish contract" >&2; exit 1; }
  grep -q "gh pr merge <PR> --auto --squash" "$skill" || { echo "FAIL: $skill missing auto-merge publish contract" >&2; exit 1; }
done

for skill in \
  "$root/agent-skills/forge-loop/SKILL.md" \
  "$root/.claude/skills/forge-loop/SKILL.md"; do
  grep -q "Write-side (no post-arm push)" "$skill" || { echo "FAIL: $skill missing no-post-arm-push safeguard" >&2; exit 1; }
done

echo "HARNESS-SCRIPTS GATE PASS"
