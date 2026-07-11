#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-/home/flexnetos/meta/src/envctl}"
SKILL_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DURABLE="$ROOT/agent-skills/agent-env-codex"
ACTIVE="${CODEX_HOME:-/home/flexnetos/.codex}/skills/agent-env-codex"
PROJECT_CODEX="$ROOT/.codex/skills/agent-env-codex"
PROJECT_CLAUDE="$ROOT/.claude/skills/agent-env-codex"
ORIG="$ROOT/.codex/prompts/prompt:codex-gpt-harness.prompt.md"
FULL="$ROOT/.codex/prompts/prompt:codex-gpt-harness-v3-full-access-no-sandbox.prompt.md"
SNAPSHOT="$SKILL_ROOT/references/source-prompt.md"
HARNESS="$ROOT/home/agent-env/codex-harness/Cargo.toml"
VALIDATOR=/home/flexnetos/.codex/skills/.system/skill-creator/scripts/quick_validate.py

for path in "$ORIG" "$FULL" "$SNAPSHOT" "$HARNESS" "$SKILL_ROOT/SKILL.md" \
  "$SKILL_ROOT/references/coverage-map.md" \
  "$SKILL_ROOT/references/ownership-map.md" \
  "$SKILL_ROOT/references/runbook-cli-contract.md" \
  "$SKILL_ROOT/agents/openai.yaml" "$DURABLE" "$ACTIVE" "$PROJECT_CODEX" "$PROJECT_CLAUDE"; do
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
grep -Fq 'Use $agent-env-codex' "$SKILL_ROOT/agents/openai.yaml"

printf '\n== skill source/projection identity ==\n'
diff -qr "$DURABLE" "$PROJECT_CODEX"
diff -qr "$DURABLE" "$PROJECT_CLAUDE"
diff -qr "$DURABLE" "$ACTIVE"
echo 'durable source = project Codex = project Claude = active materialization: yes'

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

if git -C "$ROOT" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  printf '\n== diff check ==\n'
  git -C "$ROOT" diff --check
  echo 'diff-check OK'
  printf '\n== repo status ==\n'
  git -C "$ROOT" status --short --branch
fi
