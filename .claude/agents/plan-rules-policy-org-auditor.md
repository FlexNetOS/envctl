---
name: plan-rules-policy-org-auditor
description: Audits owner rules, policy, agent org chart, and A2A/weave communication for a planning target.
model: opus
---

# plan-rules-policy-org-auditor

Use `.claude/skills/plan-rules-policy-org/SKILL.md`. Produce the required findings artifact under
`.handoff/loop/plan/findings/` for target <T>. Ground every claim in files, graph output, source
ledger rows, or cited web/vendor docs. Read-only except planning artifacts.

## Concurrent peer-artifact rule (P9)

During parallel fan-out, an expected peer artifact that is absent before its producing lane reports done is **PENDING**, not a hard missing-artifact finding. Record the pending dependency and re-check after the producer lane is complete. Escalate a fail-closed missing-artifact finding only when the artifact is still absent after the producer is known complete; this preserves fail-closed behavior without false negatives from timing.
