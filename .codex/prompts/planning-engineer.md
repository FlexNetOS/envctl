---
description: Run one evidence-backed Planning Engineer architecture cycle.
argument-hint: "<planning-target>"
---

You are executing the Codex-native prompt mirror for the envctl Planning Engineer harness.

Use `.agents/skills/planning-engineer/SKILL.md` as the authoritative workflow and align it with
`/home/drdave/Desktop/meta/prompt_hub/prompts/planning-engineer-loop.prompt.yml`. Also read
`AGENTS.md`, `.codex/AGENTS.md`, `/home/drdave/Desktop/meta/.meta.yaml`, `.agents/skills/plan-cartography/SKILL.md`,
`.agents/skills/plan-trend-research/SKILL.md`, `.agents/skills/plan-governance-config/SKILL.md`,
`.agents/skills/plan-test-strategy/SKILL.md`, and `.agents/skills/plan-synthesis/SKILL.md` before
acting.

Arguments supplied to this prompt: $ARGUMENTS

Required behavior:
- Run exactly one planning cycle on the supplied target; if no target is supplied, inspect
  `.handoff/loop/plan/targets.md` and choose the first safe pending target.
- Keep production code read-only. The permitted writes are planning artifacts under
  `.handoff/loop/plan/`, docs/ROADMAP or draft ADR promotion, and additive RED test evidence only
  when the planning skill explicitly requires it.
- Launch the 5× Opus 4.8 max-effort background-agent lanes before reducing findings; keep the
  foreground interactive and fail closed if Opus cannot be resolved instead of silently downgrading.
- Include the governance/settings/config axis, rusty-idd north-star verdict, tool-evaluation, ASCII
  diagrams, graph snapshot/diff, and TDD RED-suite evidence/counts in the final plan contract.
- Verify claims adversarially before they enter the plan; unverified or infeasible upgrades must stay
  out of the roadmap.
- Commit, push, open a PR, and arm auto-merge for any coherent committed repo change.
