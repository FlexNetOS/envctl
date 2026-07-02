# Target dependency graph — fleet-convergence loop (prompt-hub instance, cycle 6)

This is the global target dependency graph for the fleet-convergence plan-loop, rendered from
`graph/target-dag.json`. It uses **Task-Decoupled Planning (TDP)**: the supervisor decomposes the
fleet into nodes, schedules the **ready-set** in topological order, gives each node a node-scoped
context (exact artifact paths it may read), and revises **only** the affected downstream subgraph
when a verifier changes an upstream assumption — never a global reset.

- Run: `plan-prompt-hub-20260627` · cycle 6 · picked node: **prompt-hub**
- Fleet index: `/home/drdave/Desktop/meta/.meta.yaml`

## Id convention (kebab-case only)

Node ids are **kebab-case only**. The fleet (`.meta.yaml`) uses snake_case / mixed repo ids;
these are **standardized** to kebab-case here:

| fleet id (.meta.yaml) | DAG node id |
|---|---|
| `prompt_hub` | `prompt-hub` |
| `harness_hub` | `harness-hub` |
| `meta-ruvector` | `ruvector` |
| `rusty-idd`, `handoff`, `weave`, `grit`, `icm`, `envctl`, `lane`, `shimmy` | unchanged (already kebab/flat) |

## ASCII overview

```
  substrate roots (ready-set)                      converging organs
  ---------------------------                      ------------------

  prompt-hub  ──(ADR-0007: emits goal artifacts)──▶ rusty-idd ──┐
   ▲  ▲   (intent STORE; in-flight; PICKED)                     │ (union D1)
   ┊  ┊                                            grit ──┬─────▶ rusty-idd
   ┊  └┄┄ shimmy        (soft model-lane seam)            └─────▶ handoff ──▶ harness-hub
   └┄┄┄┄┄ harness-hub   (soft interpreter seam, D3)                ▲  ▲  ▲       ▲
                                                          weave ───┘  │  │       │
  lane ──▶ weave ───────────────────────────────────────────────────┘  │       │
  ruvector ──▶ icm ──────────────────────────────────────────────────────┘      │
  weave ─────────────────────────────────────────────────────────────────────────┘
  envctl   (substrate root; no fleet deps)

  ──▶ hard dependency edge (drives topological order)
  ┄┄▶ soft seam (annotated; does NOT block scheduling)
```

Hard chain to the top of the stack: `prompt-hub → rusty-idd → handoff → harness-hub`, with
`grit/weave/icm/lane/ruvector` feeding in. The graph is acyclic.

## ready-set (topological)

A node is **ready** only when all its hard `deps` are `done` or explicitly `blocked` with a
qualified gap that does not invalidate the node. Roots (empty `deps`) are always ready.

**ready-set = `[prompt-hub, grit, envctl, lane, shimmy, ruvector]`** → **picked: `prompt-hub`**.

`prompt-hub` is in the ready-set because it is modeled as a **substrate-ish root** for this cycle:
its `deps` are empty. The only upstream relationship is the **harness_hub interpreter seam** (the
two-layer front door, owner decision **D3**: `harness_hub` interprets intent → model-ready language
ahead of the `prompt_hub` store). That seam is held **soft** — per D3 the store is planned
**independently** — so it does not block `prompt-hub`. (If a verifier later proves the interpreter is
a hard prerequisite, `sr-003` promotes it to a hard dep and re-topo's, dropping prompt-hub from the
ready-set.)

Non-ready nodes wait on their deps:

| node | status | hard deps | ready? |
|---|---|---|---|
| prompt-hub | in-flight | — | yes (PICKED) |
| grit | planned | — | yes |
| envctl | planned | — | yes |
| lane | planned | — | yes |
| shimmy | planned | — | yes |
| ruvector | planned | — | yes |
| icm | planned | ruvector | no |
| weave | planned | lane | no |
| rusty-idd | planned | prompt-hub, grit | no |
| handoff | planned | grit, weave, icm, rusty-idd | no |
| harness-hub | planned | handoff, weave | no |

## prompt-hub — DAG position and edges

- **Position:** substrate-ish **root** of the intent pipeline; sits at the head of the hard chain
  `prompt-hub → rusty-idd → handoff → harness-hub`. status **in-flight**, **picked** this cycle.
- **Outgoing hard edge:** `prompt-hub → rusty-idd` — *ADR-0007: the intent STORE emits
  provenance-stamped goal artifacts that rusty-idd consumes; prompt_hub never owns rusty-idd's
  lifecycle.*
- **Soft seams (annotated, not topological):**
  - `harness-hub ⇢ prompt-hub` — interpreter seam (D3): intent → model-ready language ahead of the
    store; soft because prompt-hub is planned independently.
  - `shimmy ⇢ prompt-hub` — model lanes served by shimmy; soft, non-blocking.
- **Downstream impact:** changes to prompt-hub's emit contract propagate to `rusty-idd` (consumer)
  and transitively to `handoff` (union target) and `harness-hub` (interpreter), but the loop revises
  only the smallest affected downstream set (see SELF-REVISION).

## Node-scoped context (what each node may read)

Each node reads ONLY its `context_paths` from `target-dag.json`. Every node may read
`targets.md` + `dimensions.md`; the picked node (`prompt-hub`) additionally reads `loop_state.md`
and its own `graph/prompt-hub.json`. Nodes never read sibling nodes' graph artifacts — this is the
TDP node-scoping that keeps context small and prevents cross-node contamination.

## SELF-REVISION (localized downstream-only replanning)

When a verifier refutes/qualifies an upstream assumption, append a SELF-REVISION row and mark only
the **impacted downstream nodes** `pending`; preserve verified, unrelated nodes. Never reset the
whole DAG.

| id | trigger | scope | affected | status |
|---|---|---|---|---|
| sr-001 | prompt-hub role corrected (cycle 6 vs prior) | downstream-only | rusty-idd, harness-hub | **applied** |
| sr-002 | prompt-hub verifier on the ADR-0007 emit contract | downstream-only | rusty-idd | armed-not-triggered |
| sr-003 | harness_hub interpreter seam proven a HARD prerequisite | topology | prompt-hub | armed-not-triggered |

- **sr-001 (applied):** Prior cycles (e.g. the plan-grit DAG) modeled prompt-hub as a *prompt/model
  registry* with edge `prompt-hub → harness-hub`. The verified ADR-0007 + owner decision D3 / reply
  #178 reframe it as the **intent STORE feeding rusty-idd**. Localized revision: (1) ADDED hard edge
  `prompt-hub → rusty-idd` and added prompt-hub as an upstream dep of rusty-idd; (2) RETIRED the old
  `prompt-hub → harness-hub` edge, replaced by the soft interpreter seam `harness-hub ⇢ prompt-hub`
  (direction flipped). Only `rusty-idd` and `harness-hub` specs were touched; all substrate roots
  and `handoff/weave/icm/lane/shimmy/ruvector/envctl` were preserved.
- **sr-002 (armed):** if prompt-hub's verifier QUALIFIES/REFUTES the goal-artifact emit contract
  (schema / provenance stamping), revise ONLY rusty-idd's consume spec; preserve everything else.
- **sr-003 (armed):** if the harness_hub interpreter seam is proven a hard prerequisite, promote it
  to a hard dep (`prompt-hub.deps += harness-hub`), which removes prompt-hub from the ready-set and
  re-runs the topological order. Currently held SOFT per D3.

## Localized recovery rule

On any verifier outcome that changes a node's spec: replan the **smallest affected downstream set**
only (follow `downstream` / `edges`), append a SELF-REVISION row, and leave verified unrelated
nodes untouched. A `supervised` node writes `NEEDS-HUMAN` and is never auto-picked. If a graph query
(e.g. `git-kb code query entrypoints`) returns empty, record it as an `INCONCLUSIVE` node finding and
self-revise the nodes that depended on it.
