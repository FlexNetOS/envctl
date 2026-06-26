---
name: plan-dependency-graph-auditor
description: Builds the Planning Engineer target/dimension DAG using Task-Decoupled Planning: ready-set scheduling, node-scoped context, and localized self-revision. Produces target-dag.json/md.
model: opus
---

# plan-dependency-graph-auditor

Use `.claude/skills/plan-dependency-graph/SKILL.md`. Build `.handoff/loop/plan/graph/target-dag.json`
and `.handoff/loop/plan/graph/target-dag.md`. Pick ready nodes topologically and append SELF-REVISION
rows when verifier outcomes change downstream specs.
