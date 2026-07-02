# Fleet target dependency graph (Planning Engineer loop)

Cycle 7 · generated 2026-06-27 · this is the **fleet view** (every known organ),
not just the active target. Built with Task-Decoupled Planning: node-scoped
context, **topological** ready-set scheduling, and localized SELF-REVISION when a
downstream verification changes an upstream assumption.

> **North-star:** `meta` is ONE system converging on **handoff + rusty-idd UNION**
> (one continuity + intent control plane) @ `$META_ROOT`. Each member repo is an
> organ heading into that fabric. icm is the persistent-memory organ.

## Edge semantics

`A --deps--> B` means **A binds-into / depends-on B**: B must be at least
*planned-with-gaps* before A is "ready". Edges are refined every cycle.

## DAG (deps point downward to foundations)

```
              ┌───────────┐        ┌────────────┐
              │  handoff   │        │ rusty-idd  │     ┌────────┐   ┌──────┐
              │ continuity │        │  intent    │     │ weave  │   │ grit │
              │  kernel    │        │  control   │     │  A2A   │   │merge/│
              │ (root)     │        │ (root)     │     │transp. │   │lock  │
              └─────┬──────┘        └─────┬──────┘     └───┬────┘   └──────┘
                    │     \              /  \              /
                    │      \            /    \            /
                    v       \          v      \          v
              ┌───────────┐  \   ┌──────────┐  \  ┌──────────────┐
              │harness-hub│   `--│   icm    │---'  │   (icm also  │
              │ Front-Door│      │ MEMORY   │      │   uses weave │
              │INTERPRETER│      │  PLANE   │      │   heartbeat) │
              └─────┬─────┘      │(in-flight│      └──────────────┘
                    │  \         │ cycle 7) │
                    v   \        └────┬─────┘
              ┌────────┐ \            │
              │  lane  │  \           v
              │ exec / │   \    ┌────────────┐
              │ model  │    `---│ prompt-hub │
              │ lanes  │        │ Front-Door │
              └────────┘        │   STORE    │
                                └────────────┘
```

## Nodes

| id | status | deps | spec (1-line) | artifact_prefix |
|---|---|---|---|---|
| `handoff` | planned-with-gaps | — | continuity kernel (witnessed ledger) | `graph/handoff` |
| `rusty-idd` | planned-with-gaps | — | intent control plane (OpenSpec + merge-tools) | `graph/rusty-idd` |
| `weave` | planned | — | A2A transport / heartbeat | `graph/weave` |
| `grit` | planned-with-gaps | — | merge/lock substrate | `graph/grit` |
| **`icm`** | **in-flight** | handoff, rusty-idd, weave | **persistent-memory organ (this cycle)** | `graph/icm` |
| `harness-hub` | unplanned | handoff, rusty-idd | Front-Door INTERPRETER | `graph/harness-hub` |
| `prompt-hub` | planned-with-gaps | icm, harness-hub | Front-Door STORE | `graph/prompt-hub` |
| `lane` | unplanned | harness-hub | execution / model-lane organ | `graph/lane` |

## Scheduling

- **topological order:** `handoff → rusty-idd → weave → grit → harness-hub → icm → lane → prompt-hub`
- **ready-set (deps ≥ planned-with-gaps):** `{ handoff, rusty-idd, weave, grit, icm, harness-hub }`
  - **icm is in the ready-set** (deps handoff + rusty-idd planned-with-gaps, weave planned) → correctly selected as the cycle-7 active target.
- **blocked:** `prompt-hub` (waits on `icm` reaching planned-with-gaps — advancing this cycle), `lane` (waits on `harness-hub`).

## SELF-REVISION log

Localized replans triggered when verification changes a downstream assumption:

- **2026-06-27 (cycle 7, baseline):** No prior icm graph; `icm` node materialized
  with deps `{handoff, rusty-idd, weave}`. Open assumption to verify downstream:
  whether icm is the **canonical** memory plane or a **peer** of git-kb's
  code-graph memory and `.handoff` witnessed-ledger context. If the analyst/architect
  resolve icm as a *peer* (not canonical), revise `prompt-hub --deps--> icm` and
  re-evaluate the `icm` edge into the handoff+rusty-idd union (SELF-REVISION will
  re-scope, not reset, the affected nodes).
- icm's fleet bindings are **runtime CLI/MCP calls** (`icm store`/`icm recall`),
  not source call-edges — so they are modeled here in the DAG, not as icm-internal
  cross-repo edges (those resolved empty; single self-contained workspace).
