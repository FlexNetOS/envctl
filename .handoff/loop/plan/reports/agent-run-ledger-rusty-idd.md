# Agent run ledger — rusty-idd (cycle 1, fleet-convergence-first-run)

Every agent lane dispatched this cycle: its backend lane, model, and the artifact(s) it produced.
Foreground = Claude, so every heavy lane ran as a **direct Opus sub-agent** (`run_in_background`),
per the dual-model accuracy strategy (see `agent-backend-matrix.md` / `agent-interop.md`). No weaker
model was substituted; no lane failed closed for want of an Opus worker.

| # | agent id | role | lane (backend) | model | artifact(s) |
|---|---|---|---|---|---|
| 1 | a9c3e7e6… | plan-cartographer | read-only-local | opus | graph/rusty-idd.{json,symbols.json,callgraph.json,metrics.json,graph.md,diff.md}, reports/codemap-rusty-idd.md |
| 2 | a5a88967… | plan-trend-researcher | read-only-local | opus | research/rusty-idd.trends.md, research/sources-rusty-idd.jsonl (27 rows) |
| 3 | a79bd528… | fleet-mapper (general-purpose) | read-only-local | opus | findings/fleet-north-star-map.md |
| 4 | ae57848d… | plan-governance-config-auditor | read-only-local | opus | findings/governance-config-rusty-idd.md |
| 5 | a77a5773… | plan-filesystem-layout-auditor | read-only-local | opus | findings/filesystem-layout-rusty-idd.md |
| 6 | a2c8b3cc… | plan-test-strategist | isolated-worktree | opus | findings/test-strategy-rusty-idd.md + RED suite crates/work-order/tests/handoff_card_consumer.rs (branch plan/rusty-idd-red-tests @ 2f8a42f; tests-ran 4 = 3 RED + 1 GREEN) |
| 7 | a06b7d22… | plan-memory-vector-intelligence-auditor | read-only-local | opus | findings/memory-vector-intelligence-rusty-idd.md |
| 8 | a00dd09a… | plan-autoresearch-loop-auditor | read-only-local | opus | findings/autoresearch-rusty-idd.md |
| 9 | ac7371ba… | plan-rules-policy-org-auditor | read-only-local | opus | findings/rules-policy-org-rusty-idd.md |
| 10 | a1acd72a… | plan-distributed-compute-auditor | read-only-local | opus | findings/distributed-compute-rusty-idd.md |
| 11 | a3d3abd3… | plan-prompt-architecture-auditor | read-only-local | opus | findings/prompt-architecture-rusty-idd.md |
| 12 | a4e83f23… | plan-analyst (core dims) | read-only-local | opus | findings/architecture-rusty-idd.md (15 CLAIM / 10 UPGRADE) |
| 13 | a036ea7b… | plan-dependency-graph-auditor | read-only-local | opus | graph/target-dag.{json,md} (63 nodes) |
| 14 | a6f2f780… | plan-verifier (gate) | read-only-local | opus | findings/verdicts.md |
| 15 | (pending) | plan-architect (synthesis) | read-only-local | opus | reports/rusty-idd-plan.md, risk-policy.md, docs/ROADMAP row + DRAFT ADR |
| 16 | (pending) | evolution-steward (self-eval) | read-only-local | opus | evaluation.md, LESSONS.md, proposed-upgrades.md |

## Lane summary
- read-only-local: 14 lanes (all analysis + synthesis + verify).
- isolated-worktree: 1 lane (test-strategist — the one permitted additive mutation, on its own branch).
- container / remote-vm / cloud-agent: 0 this run (N/A — see agent-backend-matrix.md).

## Reconciliation events
- Agents 1 + 2 were resumed via SendMessage to emit gate-named split artifacts (graph JSON splits;
  sources ledger + "Tool-currency & advisories"/"Sources" headers). No re-index; existing graph reused.
