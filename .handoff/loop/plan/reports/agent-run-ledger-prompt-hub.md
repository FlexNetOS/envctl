# Agent run ledger — prompt-hub (cycle 6)

Run: plan-prompt-hub-20260627 (cycle 6; parallel instance). One row per cycle-6 agent.
lane/model = opus; artifact = the file each produced. Source: loop_state.md; findings/*; graph/*.

| agent | lane | model | artifact |
|---|---|---|---|
| plan-cartographer | plan-prompt-hub-20260627 | opus | graph/prompt-hub.{graph.md,metrics.json,symbols.json,callgraph.json,diff.md}, reports/codemap-prompt-hub.md |
| plan-dependency-graph-auditor | plan-prompt-hub-20260627 | opus | graph/target-dag.{json,md} |
| plan-trend-researcher | plan-prompt-hub-20260627 | opus | research/prompt-hub.trends.md |
| plan-analyst (architecture/correctness/convergence) | plan-prompt-hub-20260627 | opus | findings/architecture-prompt-hub.md |
| plan-prompt-architecture-auditor | plan-prompt-hub-20260627 | opus | findings/prompt-architecture-prompt-hub.md |
| plan-governance-config-auditor | plan-prompt-hub-20260627 | opus | findings/governance-config-prompt-hub.md |
| plan-filesystem-layout-auditor | plan-prompt-hub-20260627 | opus | findings/filesystem-layout-prompt-hub.md |
| plan-memory-vector-intelligence-auditor | plan-prompt-hub-20260627 | opus | findings/memory-vector-intelligence-prompt-hub.md |
| plan-autoresearch-loop-auditor | plan-prompt-hub-20260627 | opus | findings/autoresearch-prompt-hub.md |
| plan-rules-policy-org-auditor | plan-prompt-hub-20260627 | opus | findings/rules-policy-org-prompt-hub.md |
| plan-distributed-compute-auditor | plan-prompt-hub-20260627 | opus | findings/distributed-compute-prompt-hub.md |
| plan-test-strategist | plan-prompt-hub-20260627 | opus | findings/test-strategy-prompt-hub.md + RED suite prompt-hub/tests/goal_artifact_contract.rs (commit 6fa3462b) |
| plan-verifier | plan-prompt-hub-20260627 | opus | findings/verdicts.md (## prompt-hub) |
| plan-architect | plan-prompt-hub-20260627 | opus | reports/prompt-hub-plan.md, risk-policy.md, agent-backend-matrix.md, agent-interop.md, reports/ROADMAP-prompt-hub.md, reports/adr-draft-prompt-hub-goal-artifact.md, this ledger |
| evolution-steward | plan-prompt-hub-20260627 | opus | (post-cycle self-eval; harness-evolution retro) |

agent run ledger notes:
- lane is the run id (`run` in loop_state.md); model is opus for every cycle-6 agent.
- artifact is the durable output under `.handoff/loop/plan/` (or the RED-test commit for the strategist).
- read-only agents mutate nothing in the target; only the strategist authored additive tests (isolated worktree).
