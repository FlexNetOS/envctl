#!/usr/bin/env bash
# eject.sh — install the packaged `planning-engineer` harness into a target repo's .claude/.
# SAFE: copy + scaffold only (and the harness is read-only on the target's code). Prints the
# .gitignore / CLAUDE.md snippets to apply. Usage: bash eject.sh <target-repo-dir>
#
# No-downgrade semantics:
#   OWN files (the harness itself + its specific reuse) are ALWAYS (re)copied so a re-eject updates them.
#   SHARED infra (session-relay-*, harness-loop-init, harness-evolution, icm-memory, the two stewards)
#   is copied ONLY IF ABSENT in the target — never clobber a target's own canonical version.
set -euo pipefail
TARGET="${1:-}"
[ -n "$TARGET" ] || { echo "usage: bash eject.sh <target-repo-dir>" >&2; exit 1; }
[ -d "$TARGET" ] || { echo "error: target dir not found: $TARGET" >&2; exit 1; }
TARGET="$(cd "$TARGET" && pwd)"
PLUGIN="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"  # harness/

# OWN: the harness's own skills + its specific reuse (code-research-verify) → always refresh.
OWN_SKILLS=(planning-engineer plan-loop plan-cartography plan-trend-research plan-test-strategy plan-synthesis code-research-verify)
OWN_AGENTS=(plan-cartographer plan-trend-researcher plan-analyst plan-test-strategist plan-verifier plan-architect)
# SHARED: copy only if the target lacks them (don't downgrade a hand-authored canonical version).
SHARED_SKILLS=(session-relay-wrap-up session-relay-resume harness-loop-init harness-evolution icm-memory)
SHARED_AGENTS=(continuity-steward evolution-steward)

mkdir -p "$TARGET/.claude/skills" "$TARGET/.claude/agents" \
         "$TARGET/.handoff/loop/plan/graph" "$TARGET/.handoff/loop/plan/research" \
         "$TARGET/.handoff/loop/plan/findings" "$TARGET/.handoff/loop/plan/reports"
# .gitkeep so the (initially empty) contract dirs persist in the target's git — the first cycle fills
# them, but without this an `git add` in a fresh target would drop the empty dirs and lose the layout.
for d in graph research findings reports; do
  [ -e "$TARGET/.handoff/loop/plan/$d/.gitkeep" ] || : > "$TARGET/.handoff/loop/plan/$d/.gitkeep"
done

for s in "${OWN_SKILLS[@]}"; do
  if [ -d "$PLUGIN/skills/$s" ]; then rm -rf "$TARGET/.claude/skills/$s"; cp -r "$PLUGIN/skills/$s" "$TARGET/.claude/skills/$s"; echo "  skill  (own)    -> .claude/skills/$s";
  else echo "  skill  -- MISSING in plugin: $s" >&2; fi
done
for s in "${SHARED_SKILLS[@]}"; do
  if [ -d "$TARGET/.claude/skills/$s" ]; then echo "  skill  (shared) == keep target's existing .claude/skills/$s";
  elif [ -d "$PLUGIN/skills/$s" ]; then cp -r "$PLUGIN/skills/$s" "$TARGET/.claude/skills/$s"; echo "  skill  (shared) -> .claude/skills/$s (was absent)";
  else echo "  skill  -- MISSING in plugin & target: $s" >&2; fi
done
for a in "${OWN_AGENTS[@]}"; do
  if [ -f "$PLUGIN/agents/$a.md" ]; then cp "$PLUGIN/agents/$a.md" "$TARGET/.claude/agents/$a.md"; echo "  agent  (own)    -> .claude/agents/$a.md";
  else echo "  agent  -- MISSING in plugin: $a" >&2; fi
done
for a in "${SHARED_AGENTS[@]}"; do
  if [ -f "$TARGET/.claude/agents/$a.md" ]; then echo "  agent  (shared) == keep target's existing .claude/agents/$a.md";
  elif [ -f "$PLUGIN/agents/$a.md" ]; then cp "$PLUGIN/agents/$a.md" "$TARGET/.claude/agents/$a.md"; echo "  agent  (shared) -> .claude/agents/$a.md (was absent)";
  else echo "  agent  -- MISSING in plugin & target: $a" >&2; fi
done
echo "  state  -> .handoff/loop/plan/ scaffolded (seed loop_state.md: planning_target + target_root; targets.md auto-derives on first cycle)"

cat <<'SNIP'

── Apply to the target repo yourself (repo-specific; not edited for you) ──

# .gitignore:
.claude/*
!.claude/agents/
!.claude/skills/
.handoff/loop/plan/*.log
.handoff/loop/plan/ralph-run-*.log

# CLAUDE.md pointer:
## Harness: Planning Engineer
**Goal:** continuous, evidence-backed planning/architecture — code graph + 90-day research ->
adversarially-verified gaps -> a plan with ASCII diagrams, quality/speed/accuracy upgrades, tool-eval.
**Trigger:** for "plan <subsystem> / architecture plan / deep planning / loop on the architecture /
resume the planning loop", use the `plan-loop` (continuous) / `planning-engineer` (single-cycle) skill.
Read-only on production code; writes plans/graph under .handoff/loop/plan/ + docs.

# Harness self-tests (optional CI gate): the package ships hermetic, network-free tests at
#   .claude/skills/planning-engineer/scripts/tests/test-plan-{eject,loop-state,contract}.sh
# They resolve their scripts-under-test from the repo root; wire them into CI (run each with `bash`).
# The loop-state test needs ci/gates/loop-state.sh present (copy it if your repo runs the plan loop).

Done. Invoke as: /planning-engineer  (single cycle)  ·  /plan-loop  (continuous)
SNIP
