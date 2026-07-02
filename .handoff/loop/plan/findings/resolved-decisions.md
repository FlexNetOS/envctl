# Resolved decision-findings — owner verdicts (2026-06-26)

These are OWNER-CONFIRMED answers to the three cycle-1 decision-findings. They are now binding
north-star **data** for every subsequent cycle (bind-to-north-star-as-data, not hardcode-as-prose).

## D1 — Where the shared fleet north-star artifact lives → RESOLVED
**Verdict:** it lives at **`$META_ROOT` + handoff** (the meta-root + the handoff kernel, not buried in
one member's skill prose). **The goal is the `handoff` + `rusty-idd` UNION** — a unified
continuity+intent control plane. The north-star artifact is therefore co-located with the kernel that
every repo can already reach (`hf` / `.handoff`) and the meta root.
**Implication for the loop:** cycle 2 retargets to **`handoff`**, planned explicitly as the union with
rusty-idd. A DRAFT north-star artifact is produced this cycle for `$META_ROOT` + handoff (PROPOSE tier —
owner canon; not written into fleet canon without approval).
Supersedes the cycle-1 "two competing NORTH-STAR docs, neither propagated" gap: the resolution is to
converge them into the `$META_ROOT`+handoff location as the single source.

## D2 — Run-from / residency / transport → RESOLVED
**Verdict:** **run from `envctl`** (confirmed as the standing model). Validated in cycle 1: plan-state
landed correctly under the envctl loop worktree; weave resolved via `META_ROOT` (transport, not
run-from). No change required.

## D3 — harness_hub audience / role → RESOLVED
**Verdict:** harness_hub is the **connection / interpreter at the Front Door** — it **transforms user
intent into the language models need to deliver the user's desired output**. It is the intent→model
translation/interpreter layer that fronts the control plane (rusty-idd = intent; harness_hub =
interpret intent into model-ready language; models execute; output returns to the user).
**Implication:** the north-star binds in a **meta-level layer the skill reads** (so harness_hub can
interpret against it), consistent with D1's `$META_ROOT`+handoff location — NOT hardcoded inside one
skill. harness_hub is a shareable interpreter/connector, not an internal-only catalog.

## Convergence picture (owner-confirmed)
```
user intent ──> harness_hub (Front Door: interpret intent -> model language)
                     │
                     v
            rusty-idd (intent control plane: why/what)  ⇄  handoff (continuity kernel: witnessed state)
                     │   the UNION = unified continuity+intent control plane (north-star @ $META_ROOT + handoff)
                     v
              weave (A2A transport)  ──>  models / distributed compute  ──> output ──> user
```
