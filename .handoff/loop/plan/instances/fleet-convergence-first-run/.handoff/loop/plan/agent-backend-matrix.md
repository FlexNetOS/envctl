# Agent backend matrix — execution surfaces for the Planning Engineer Loop

How each agent lane in this loop is (or can be) executed, with the trust/isolation properties that
gate which work may run where. This run executed entirely on the first two rows; the rest are the
convergence target (the loop should be able to dispatch onto all of them via the fabric).

| Backend | Isolation | Mutation rights | Used this run | Convergence note |
|---|---|---|---|---|
| **read-only-local** sub-agent | shares session FS, no writes to product code | reads only; writes plan artifacts under `.handoff/loop/plan/` | YES — cartographer, trend-researcher, fleet-mapper, all 8 axis/analyst auditors | the default planning lane; cheapest, fail-closed read-only law applies |
| **isolated-worktree** agent | dedicated `git worktree` off `origin/<base>`; cannot touch the owner's dirty checkout | additive only (RED tests) on its own branch | YES — test-strategist in `meta/.worktrees/plan-rusty-idd-red/rusty-idd` (branch `plan/rusty-idd-red-tests`); the loop itself runs from `meta/.worktrees/plan-fleet-convergence/envctl` | the mutation lane; one worktree per concurrent writer keeps the owner's WIP untouched |
| **container** agent | OS-level FS+net isolation | scoped to mounted volume | NO — N/A this run | target lane for untrusted toolchains / network-fetching research workers |
| **remote-vm** agent | full host isolation, remote | scoped to VM | NO — N/A this run | target lane for heavy/long compute the workstation should not host |
| **cloud-agent** (e.g. GitHub-hosted) | provider-isolated; PR-mediated | PR proposals only | NO — N/A this run | target lane for `/code-review ultra`-class cloud review; PR is the trust boundary |

## Backend selection rule (fail-closed)
- Read-only analysis → **read-only-local** (default).
- Any mutation (RED tests, doc PRs) → **isolated-worktree** off a fresh base; never the owner's working checkout.
- Untrusted network/tooling → **container**/**remote-vm** (not exercised this run).
- Cross-provider/cloud work → **cloud-agent**, mediated by PR + the **ACP/A2A** interop boundary (see `agent-interop.md`).

## Dual-model accuracy strategy
Foreground here is Claude, so heavy lanes ran as **direct Opus sub-agents** (`run_in_background`),
not weave→Opus (that route is for a Codex foreground). If an Opus worker could not be obtained where
the contract requires one, the lane fails closed with a provider/transport gap — it does NOT silently
drop to a weaker model. See `risk-policy.md` (provider/model row) and `agent-interop.md`.
