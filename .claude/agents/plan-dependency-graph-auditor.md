---
name: plan-dependency-graph-auditor
description: Builds the Planning Engineer target/dimension DAG using Task-Decoupled Planning: ready-set scheduling, node-scoped context, and localized self-revision. Produces target-dag.json/md.
model: opus
---

# plan-dependency-graph-auditor

Use `.claude/skills/plan-dependency-graph/SKILL.md`. Build `.handoff/loop/plan/graph/target-dag.json`
and `.handoff/loop/plan/graph/target-dag.md`. Pick ready nodes topologically and append SELF-REVISION
rows when verifier outcomes change downstream specs.

## Concurrent peer-artifact rule (P9)

During parallel fan-out, an expected peer artifact that is absent before its producing lane reports done is **PENDING**, not a hard missing-artifact finding. Record the pending dependency and re-check after the producer lane is complete. Escalate a fail-closed missing-artifact finding only when the artifact is still absent after the producer is known complete; this preserves fail-closed behavior without false negatives from timing.
