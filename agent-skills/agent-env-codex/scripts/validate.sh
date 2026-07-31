#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-$(git -C "$(dirname "${BASH_SOURCE[0]}")/../../.." rev-parse --show-toplevel)}"
SKILL_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DURABLE="$ROOT/agent-skills/agent-env-codex"
ORIG="$ROOT/.codex/prompts/prompt:codex-gpt-harness.prompt.md"
FULL="$ROOT/.codex/prompts/prompt:codex-gpt-harness-v3-full-access-no-sandbox.prompt.md"
SNAPSHOT="$SKILL_ROOT/references/source-prompt.md"
HARNESS="$ROOT/home/agent-env/codex-harness/Cargo.toml"
SUBSTRATE_PROMPT="$ROOT/.codex/prompts/prompt:substrate-init.inherit.md"
VALIDATOR=/home/flexnetos/meta/var/lib/codex/skills/.system/skill-creator/scripts/quick_validate.py

for path in "$ORIG" "$FULL" "$SNAPSHOT" "$HARNESS" "$SUBSTRATE_PROMPT" \
  "$SKILL_ROOT/SKILL.md" \
  "$SKILL_ROOT/references/coverage-map.md" \
  "$SKILL_ROOT/references/bunx-and-github-ssh.md" \
  "$SKILL_ROOT/references/github-execution-policy.md" \
  "$SKILL_ROOT/references/github-org-and-ccboard.md" \
  "$SKILL_ROOT/references/ownership-map.md" \
  "$SKILL_ROOT/references/runbook-cli-contract.md" \
  "$SKILL_ROOT/references/yazelix-cli-plugin-policy.md" \
  "$SKILL_ROOT/scripts/check-bun-command-policy.py" \
  "$SKILL_ROOT/scripts/check-yazelix-contract.py" \
  "$SKILL_ROOT/agents/openai.yaml" "$DURABLE"; do
  [[ -e "$path" ]] || { echo "missing required path: $path" >&2; exit 2; }
done

printf '== skill validation ==\n'
python3 "$VALIDATOR" "$SKILL_ROOT"
python3 "$VALIDATOR" "$DURABLE"
if grep -RInE 'TODO|PLACEHOLDER' "$SKILL_ROOT" --exclude=source-prompt.md --exclude=validate.sh; then
  echo 'unfinished skill marker found' >&2
  exit 3
fi
grep -Fq 'name: agent-env-codex' "$SKILL_ROOT/SKILL.md"
grep -Fq '/agent-env-codex' "$SKILL_ROOT/SKILL.md"
grep -Fq 'references/source-prompt.md' "$SKILL_ROOT/SKILL.md"
grep -Fq 'references/github-execution-policy.md' "$SKILL_ROOT/SKILL.md"
grep -Fq 'references/github-org-and-ccboard.md' "$SKILL_ROOT/SKILL.md"
grep -Fq 'references/bunx-and-github-ssh.md' "$SKILL_ROOT/SKILL.md"
grep -Fq 'Use $agent-env-codex' "$SKILL_ROOT/agents/openai.yaml"
grep -Fq 'Never cherry-pick.' "$SKILL_ROOT/references/github-execution-policy.md"
grep -Fq 'Permission integrity' "$SKILL_ROOT/references/github-execution-policy.md"
grep -Fq 'Unfinished-work closure' "$SKILL_ROOT/references/github-execution-policy.md"
grep -Fq 'Meta worktree authority' "$SKILL_ROOT/references/github-execution-policy.md"
grep -Fq 'Linux-only automation' "$SKILL_ROOT/references/github-execution-policy.md"
grep -Fq 'Non-destructive fork sync' "$SKILL_ROOT/references/github-execution-policy.md"
grep -Fq 'Personal and organization SSH proof' "$SKILL_ROOT/references/github-execution-policy.md"
grep -Fq 'Codex is partially wired, not absent' "$SKILL_ROOT/references/github-org-and-ccboard.md"
grep -Fq 'Do not claim SSH can configure organization settings.' "$SKILL_ROOT/references/github-org-and-ccboard.md"
grep -Fq 'bunx ruv-swarm/claude-flow@alpha' "$SKILL_ROOT/references/bunx-and-github-ssh.md"
grep -Fq 'drdave-flexnetos' "$SKILL_ROOT/references/bunx-and-github-ssh.md"
grep -Fq 'Do not invent a `yzx sync` command' "$SKILL_ROOT/references/yazelix-cli-plugin-policy.md"
grep -Fq '/home/flexnetos/meta/src/yazelix-yazi-assets' "$SKILL_ROOT/references/yazelix-cli-plugin-policy.md"
grep -Fq 'yzx update local_source' "$SKILL_ROOT/references/yazelix-cli-plugin-policy.md"
grep -Fq 'yzx update upstream' "$SKILL_ROOT/references/yazelix-cli-plugin-policy.md"
grep -Fq 'yzx update home_manager' "$SKILL_ROOT/references/yazelix-cli-plugin-policy.md"
if grep -Fq 'raw `git` is only' "$FULL"; then
  echo 'legacy raw-git exception remains in prompt' >&2
  exit 6
fi
if grep -Eq '^- git -C ' "$FULL"; then
  echo 'legacy executable raw-git probe remains in prompt' >&2
  exit 6
fi
if grep -Fq 'Subagents are optional' "$FULL"; then
  echo 'legacy optional-subagent downgrade remains in prompt' >&2
  exit 6
fi
grep -Fq 'empty harness-owned roster' "$FULL"
grep -Fq 'Never leave completed or idle subagents running.' "$SKILL_ROOT/SKILL.md"

printf '\n== Bun/Bunx skill command policy ==\n'
python3 "$SKILL_ROOT/scripts/check-bun-command-policy.py" "$ROOT"
POLICY_FIXTURE="$(mktemp -d)"
trap 'rm -rf "$POLICY_FIXTURE"' EXIT
mkdir -p "$POLICY_FIXTURE/agent-skills/policy-fixture/scripts"
printf '%s\n' '#!/usr/bin/env bash' 'npx forbidden-package' \
  > "$POLICY_FIXTURE/agent-skills/policy-fixture/scripts/fail.sh"
if python3 "$SKILL_ROOT/scripts/check-bun-command-policy.py" "$POLICY_FIXTURE" \
  >/dev/null 2>&1; then
  echo 'Bun/Bunx validator failed to reject an executable non-Markdown npx recipe' >&2
  exit 8
fi
printf '%s\n' '#!/usr/bin/env bash' 'bunx allowed-package' \
  > "$POLICY_FIXTURE/agent-skills/policy-fixture/scripts/pass.sh"
rm "$POLICY_FIXTURE/agent-skills/policy-fixture/scripts/fail.sh"
python3 "$SKILL_ROOT/scripts/check-bun-command-policy.py" "$POLICY_FIXTURE"
rm -rf "$POLICY_FIXTURE"
trap - EXIT
echo 'non-Markdown executable fixture enforcement: yes'

printf '\n== Yazelix durable policy ==\n'
python3 "$SKILL_ROOT/scripts/check-yazelix-contract.py" --root "$ROOT"

printf '\n== canonical owner/projection contract ==\n'
# The catalog owns activation state.  This skill can be inactive while core-only
# discovery is selected, so neither a home nor a project mirror is an owner or
# a validator prerequisite.  The retired Claude root mirror must remain gone.
[[ ! -e "$ROOT/.claude/skills/agent-env-codex" ]] \
  || { echo 'retired Claude agent-env-codex projection returned' >&2; exit 2; }
grep -Fq 'agent-env-codex:' "$ROOT/agent-skills/skill-catalog/catalog.yaml"
echo 'durable source is catalogued; projections are owner-generated only: yes'

printf '\n== complete prompt identity ==\n'
sha256sum "$ORIG" "$FULL" "$SNAPSHOT"
cmp -s "$ORIG" "$FULL" || {
  echo 'canonical and full-access prompts are not byte-identical' >&2
  exit 4
}
cmp -s "$FULL" "$SNAPSHOT" || {
  echo 'bundled source-prompt.md is not byte-identical to the full prompt' >&2
  exit 5
}
echo 'canonical = full-access = bundled snapshot: yes'

printf '\n== one-skill target contract ==\n'
grep -Fq 'one compact `/agent-env-codex`' "$FULL"
grep -Fq 'agent-skills/agent-env-codex/' "$FULL"
grep -Fq 'This is one skill.' "$FULL"
for path in \
  'references/source-prompt.md' \
  'references/ownership-map.md' \
  'references/runbook-cli-contract.md' \
  'references/coverage-map.md' \
  'references/bunx-and-github-ssh.md' \
  'references/github-execution-policy.md' \
  'references/github-org-and-ccboard.md' \
  'references/yazelix-cli-plugin-policy.md' \
  'scripts/check-bun-command-policy.py' \
  'scripts/check-yazelix-contract.py' \
  'scripts/validate.sh'; do
  grep -Fq "$path" "$FULL" || {
    echo "prompt target shape omits $path" >&2
    exit 6
  }
done
if grep -Fq '`rtk git ...`' "$FULL" || grep -Fq '/bin/rtk git status' "$FULL"; then
  echo 'direct RTK Git route remains in prompt; use RTK/Meta' >&2
  exit 6
fi
for path in "$FULL" "$SUBSTRATE_PROMPT"; do
  if grep -Eq '`rtk git (status|worktree|fetch|pull|push|commit|branch|merge|rebase)' "$path" \
    || grep -Fq '`meta --' "$path" \
    || grep -Fq '`meta git status`' "$path" \
    || grep -Fq '`meta exec -- git' "$path" \
    || grep -Fq '`rtk meta exec -- git' "$path"; then
    echo "non-Meta-only Git recipe remains in $path" >&2
    exit 6
  fi
done
grep -Fq 'rtk meta exec --include <repo> -- git <command>' "$FULL"
if grep -Fq 'harness-session/SKILL.md' "$FULL"; then
  echo 'stale split-skill target remains in prompt' >&2
  exit 6
fi
echo 'single durable /agent-env-codex target: yes'

printf '\n== complete controller/phase anchor coverage ==\n'
python3 - "$FULL" <<'PY'
from pathlib import Path
import sys
text = Path(sys.argv[1]).read_text()
anchors = [
    '2026-07-11 PROMPT-POLISH AND SKILL-BUILD CONTROLLER',
    'Current source anchors',
    'Runbook capture requirement',
    'Research proof ledger captured for skill build',
    'Yazelix/Nix/Nushell ownership controller',
    'Mandatory-task, latest-toolchain, and Yazelix convergence controller',
    'Non-mutating harness init and command-routing controller',
    'Professional CLI probe matrix for prompt and skill validation',
    'Automations and hardware optimization contracts',
    'Permission and capability toggles',
    'Model-lane controller',
    'Subagent and context-preservation controller',
    'Skill-building target shape',
    '2026-07-09 FULL-ACCESS INCIDENT CONTROLLER',
    'ABSOLUTE LAWS',
    'PHASE 0 - HISTORICAL RESEARCH GATE',
    'PHASE 1 — CONTAINMENT BEFORE AGENTIC POWER',
    'PHASE 2 — CONFIG, MODEL CATALOG, AND PROVIDER TOGGLES',
    'PHASE 3 — SUBAGENT-MANDATORY TEAM FABRIC',
    'PHASE 4 — ADVANCED TUI, TIMERS, AND BAD-BEHAVIOR COUNTERS',
    'PHASE 5 — BROWSER USE AND COMPUTER USE',
    'PHASE 6 — MEMORY AND DATABASE',
    'PHASE 7 — PROVIDERS, NETWORKING, AND MODEL FABRIC',
    'PHASE 8 — GITHUB CONTROL, POLICY, AND WORKTREES',
    'PHASE 9 — SKILLS, PLUGINS, AND MCP',
    'PHASE 10 — PARALLEL EXECUTION FABRIC',
    'PHASE 11 — FINAL VERIFICATION',
    'Additive Secret/Vault/Envctl Harness Rules',
    'Full-Access Variant Provenance',
]
missing = [anchor for anchor in anchors if anchor not in text]
if missing:
    print('missing anchors:')
    for anchor in missing:
        print(f'- {anchor}')
    raise SystemExit(7)
print(f'anchors present: {len(anchors)}/{len(anchors)}')
PY

printf '\n== no-downgrade prompt reviews ==\n'
cargo run --quiet --manifest-path "$HARNESS" --bin codex-harness-prompt-review -- "$ORIG"
cargo run --quiet --manifest-path "$HARNESS" --bin codex-harness-prompt-review -- "$FULL"

printf '\n== complete Codex harness tests ==\n'
cargo test --quiet --manifest-path "$HARNESS"

RTK=/home/flexnetos/.nix-profile/bin/rtk
META_ROOT=/home/flexnetos/meta
if [[ -x "$RTK" && -d "$META_ROOT" ]]; then
  printf '\n== diff check ==\n'
  (cd "$META_ROOT" && "$RTK" meta exec --include envctl -- git -C "$ROOT" diff --check)
  echo 'diff-check OK'
  printf '\n== repo status ==\n'
  (cd "$META_ROOT" && "$RTK" meta exec --include envctl -- git -C "$ROOT" status --short --branch)
else
  echo 'RTK/Meta unavailable: repository status proof skipped, not replaced by raw git' >&2
fi
