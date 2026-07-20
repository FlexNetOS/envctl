---
name: plan-dependency-graph
description: >-
  Build and maintain the Planning Engineer target/dimension dependency DAG using Task-Decoupled
  Planning: node-scoped context, topological ready-set scheduling, and localized self-revision when
  verification changes downstream assumptions. Use for plan-loop target scheduling, partial replans,
  and DONE/completeness gates.
---

# plan-dependency-graph — TDP target DAG + localized self-revision

This skill turns `targets.md` and `dimensions.md` into an explicit dependency graph so the planning
loop is not a monolithic first-unchecked-line walker. It adopts the Task-Decoupled Planning pattern:
a supervisor decomposes work into nodes, schedules the ready set topologically, and revises only the
affected downstream subgraph when a verifier refutes or qualifies an assumption.

## Required artifacts

- `.handoff/loop/plan/graph/target-dag.json`
- `.handoff/loop/plan/graph/target-dag.md`

`target-dag.json` schema:

```json
{
  "nodes": [
    {
      "id": "engine",
      "spec": "shared sync library",
      "status": "pending|ready|in_progress|done|blocked|supervised",
      "deps": [],
      "artifact_prefix": "engine",
      "context_paths": ["targets.md", "dimensions.md"],
      "downstream": []
    }
  ],
  "edges": [{"from": "engine", "to": "cli", "reason": "CLI drives Engine API"}],
  "ready_set": ["engine"],
  "self_revision": [
    {"id": "sr-001", "trigger": "verifier-refuted", "affected": ["cli"], "action": "revise downstream spec only"}
  ]
}
```

`target-dag.md` must include:

1. `# Target dependency graph` with an ASCII overview.
2. `ready-set` chosen by topological order.
3. `SELF-REVISION` rows for every verifier refutation/qualification that changes downstream specs.
4. Node-scoped context: exact artifact paths each node may read.
5. Localized recovery rule: replan the smallest affected downstream set; never reset unrelated nodes.

## Scheduling rules

- A node is ready only when all `deps` are `done` or explicitly `blocked` with a qualified gap that
  does not invalidate this node.
- A `supervised` node writes `NEEDS-HUMAN`; it is never auto-picked.
- If `git-kb code query entrypoints` or another graph query returns empty, record the empty query as an
  `INCONCLUSIVE` node finding and self-revise downstream nodes that depended on it.
- If a verifier changes an upstream claim, append a `SELF-REVISION` row and mark only impacted
  downstream nodes `pending`; preserve verified unrelated nodes.

## Gate

`scripts/plan-artifact-gate.sh` requires `target-dag.json` and `target-dag.md` for every completed
planning run and rejects DONE without a self-revision-ready DAG.
