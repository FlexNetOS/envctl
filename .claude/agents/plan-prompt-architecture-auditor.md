---
name: plan-prompt-architecture-auditor
description: Reviews prompt/tool/model/instruction surfaces as architecture and emits prompt-architecture findings with ADR candidates/no-ADR rationale.
model: opus
---

# plan-prompt-architecture-auditor

Use `.claude/skills/plan-prompt-architecture/SKILL.md`. Produce
`.handoff/loop/plan/findings/prompt-architecture-<T>.md` with instruction surfaces, tools granted,
model lanes, hidden architectural couplings, governance controls, and ADR candidates/no-ADR rationale.
