---
description: Run or resume the continuous Planning Engineer Ralph loop.
argument-hint: "[resume|budget=N|target=<slug>]"
allowed-tools: "*"
---

# /plan-loop — continuous Planning Engineer Ralph loop

Use `.codex/prompts/plan-loop.md` as the canonical prompt body for this repository. Read it first,
then execute it with the arguments below.

Arguments supplied to this command: $ARGUMENTS

Claude Code execution requirements:
- Treat this as the continuous `plan-loop` skill, not a one-off answer. Use `.claude/skills/plan-loop/SKILL.md` and `.claude/skills/planning-engineer/SKILL.md` as the authoritative Claude-side workflow mirrors.
- Preserve the PromptHub source alignment named by the canonical Codex prompt: `/home/drdave/Desktop/meta/prompt_hub/prompts/planning-engineer-loop.prompt.yml`.
- On `resume`, reconstruct durable state from `.handoff/loop/plan/loop_state.md`, `.handoff/loop/plan/targets.md`, latest plan reports, ICM recall, and any committed handoff packet before choosing work.
- Launch the five required Opus background lanes through weave while keeping the foreground chat interactive: code graph, web/trends, governance/control plane, settings/config/filesystem layout, and rusty-idd north-star.
- Keep production code read-only except permitted planning artifacts and explicitly additive RED test evidence required by the planning contract.
- Produce and gate the plan artifacts required by `scripts/plan-artifact-gate.sh .handoff/loop/plan`, including the target DAG, prompt architecture findings, source ledger, agent run ledger, risk policy, backend matrix, and interop registry.
- Preserve the owner contract: strict upgrade only, no downgrades/destructive resets, and commit/push/PR/auto-merge for every coherent repo artifact change.
