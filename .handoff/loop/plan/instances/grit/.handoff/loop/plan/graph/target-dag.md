# Target dependency graph — fleet-convergence loop (grit instance, cycle 5)

This is the global **target dependency graph** for the fleet-convergence plan loop. It treats meta as
ONE converging system and schedules the convergence organs + substrates with **Task-Decoupled
Planning (TDP)**: a supervisor decomposes the fleet into nodes, schedules the **ready-set** in
**topological** order, and revises only the affected downstream subgraph when a verifier changes an
upstream assumption.

Companion machine artifact: `graph/target-dag.json`.

## Task-Decoupled Planning (how this loop schedules)

- **Ready-set scheduling.** A node is *ready* only when all of its `deps` are `done` (or `blocked`
  with a qualified gap that does not invalidate this node). The loop never walks the
  first-unchecked line of `targets.md`; it picks from the topological ready-set.
- **Node-scoped context.** Each node may read only the artifact paths in its `context_paths`
  (its own `graph/<id>.json`, `targets.md`, `dimensions.md`, `loop_state.md`). Planning one node
  does not pull in the whole fleet's context — that is what keeps a cycle decoupled.
- **Localized self-revision.** When a verifier REFUTES or QUALIFIES an upstream claim, we append a
  `SELF-REVISION` row and mark only the impacted **downstream** nodes `pending`. Verified, unrelated
  nodes are preserved — never a global reset.

## ASCII overview

```
            SUBSTRATE ROOTS (deps satisfied -> ready-set)
   grit*      envctl      lane        shimmy        ruvector
    |  \                   |            |              |
    |   \                  v            v              v
    |    \               weave       prompt-hub       icm
    |     \                |            |              |
    |      \               |            |              |
    |       +------> handoff <----------+--------------+
    |               (continuity kernel / union target)
    |                 ^      |
    +----> rusty-idd -+      v
       (union target)     harness-hub
                       (Front-Door interpreter)

   * grit = picked node this cycle (substrate root, deps satisfied -> in ready-set)
   union step 2: grit dedups rusty-idd's cli/core/runner/spec/tui into handoff (~95% shared)
```

## Ready-set (chosen by topological order)

Nodes whose `deps` are all satisfied — the only nodes the loop may auto-pick:

| node | status | deps | why ready |
|---|---|---|---|
| **grit** (picked) | in-flight | — | substrate root, no upstream deps; powers union dedup |
| envctl | planned | — | substrate root (env manager, loop run-from host) |
| lane | planned | — | substrate root (network spine) |
| shimmy | planned | — | substrate root (local model server) |
| ruvector | planned | — | substrate root (vector compute / code intelligence) |

**Picked this cycle: `grit`** — a substrate root with no upstream dependencies, so its deps are
trivially satisfied and it enters the ready-set immediately. Its downstream is the union dedup set
`{handoff, rusty-idd}`: grit's symbol-level merge/lock is what lets union step 2 dedup the ~95%
shared handoff<->rusty-idd crates.

Not yet ready (waiting on upstream): `weave` (needs lane), `prompt-hub` (needs shimmy),
`icm` (needs ruvector), `handoff` (needs grit+weave+icm), `rusty-idd` (needs grit),
`harness-hub` (needs handoff+weave+prompt-hub).

## Node-scoped context (exact paths each node may read)

| node | artifact_prefix | context_paths |
|---|---|---|
| grit | grit | targets.md, dimensions.md, loop_state.md, graph/grit.json |
| rusty-idd | rusty-idd | targets.md, dimensions.md, graph/rusty-idd.json |
| handoff | handoff | targets.md, dimensions.md, graph/handoff.json |
| weave | weave | targets.md, dimensions.md, graph/weave.json |
| icm | icm | targets.md, dimensions.md, graph/icm.json |
| harness-hub | harness-hub | targets.md, dimensions.md, graph/harness-hub.json |
| envctl | envctl | targets.md, dimensions.md, graph/envctl.json |
| lane | lane | targets.md, dimensions.md, graph/lane.json |
| prompt-hub | prompt-hub | targets.md, dimensions.md, graph/prompt-hub.json |
| shimmy | shimmy | targets.md, dimensions.md, graph/shimmy.json |
| ruvector | ruvector | targets.md, dimensions.md, graph/ruvector.json |

Each node writes under its prefix: `graph/<prefix>.*`, `findings/*-<prefix>.md`,
`reports/<prefix>-plan.md` (e.g. grit -> `graph/grit.*`, `findings/*-grit.md`,
`reports/grit-plan.md`).

## Naming standardization (kebab-case slug law)

The gate slug regex is `^[a-z0-9][a-z0-9-]*$`, so two fleet members carry snake_case ids that are
**standardized to kebab-case** as node ids here:

| fleet id (.meta.yaml) | node id (this DAG) |
|---|---|
| `harness_hub` | `harness-hub` |
| `prompt_hub` | `prompt-hub` |

`ruvector` maps to the fleet member at path `meta-ruvector`; the node id stays `ruvector` (already
kebab-safe). All other organ/substrate ids were already kebab-case.

## SELF-REVISION ledger

Localized recovery rule: when a verifier REFUTES/QUALIFIES an upstream node's claim, replan the
**smallest affected downstream set** only; never reset unrelated, already-verified nodes.

| id | trigger | scope | affected (downstream only) | action | status |
|---|---|---|---|---|---|
| sr-001 | grit verifier outcome on symbol-level merge feasibility / dedup ratio | downstream-only | handoff, rusty-idd | If grit's verifier QUALIFIES/REFUTES the dedup spec (e.g. the ~95% shared-crate ratio is lower in practice), revise only the union dedup spec on `handoff` and `rusty-idd`; preserve weave/icm/lane/shimmy/ruvector/envctl/harness-hub. | armed-not-triggered |

No verifier refutation has changed a downstream spec this cycle, so sr-001 stands armed but not
triggered. New rows are appended (never edited in place) each time a verifier outcome propagates a
spec change to a downstream node.
