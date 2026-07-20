# Loop state — planning-engineer (plan-loop)
session_started: <UTC e.g. 2026-06-25T16:00:00Z>   # you supply it; the runtime can't read the clock
loop: planning-engineer
branch: <branch>
worktree: <abs path>
planning_target: <current T (crate/subsystem slug), or "(sweeping targets.md)">
target_root: <abs path of the subsystem/crate under plan, e.g. ~/Desktop/meta/envctl/crates/secrets-proto>
recency_window_days: 90                              # the rolling web-research window (R3a)
graph_snapshot: graph/<T>.symbols.json@<git-sha>     # which snapshot the metrics derive from
target_dag: graph/target-dag.json                       # TDP scheduler state: nodes/edges/ready_set/self_revision
artifact_gate: scripts/plan-artifact-gate.sh             # runtime DONE/completeness validator
source_ledger: research/sources-<T>.jsonl                # machine-readable cited source ledger
agent_run_ledger: reports/agent-run-ledger-<T>.md         # background-lane spans + artifacts + verdicts
risk_policy: risk-policy.md                              # HITL/SUPERVISED routing table
agent_backend_matrix: agent-backend-matrix.md            # local/worktree/container/remote/cloud/ACP/A2A isolation decision
agent_interop: agent-interop.md                          # weave/MCP/ACP/A2A/GitHub cloud agent routing registry
prompt_architecture: findings/prompt-architecture-<T>.md # prompt/tool/model couplings + ADR/no-ADR review
memory_vector_intelligence: findings/memory-vector-intelligence-<T>.md # persistent memory + vector/code intelligence
autoresearch: findings/autoresearch-<T>.md            # constant code+web research cadence
rules_policy_org: findings/rules-policy-org-<T>.md    # Upgrade Only/No Downgrades + agent org + A2A
distributed_compute: findings/distributed-compute-<T>.md # Rust/Lua hardware/vendor compute fabric
weave_dispatch: weave-dispatch/<run-id>.jsonl        # five Opus lane transport rows (peer/session/message/job ids)
weave_orchestrator: envctl-plan-orchestrator-<run-id> # unique foreground registration; avoid ambiguous aliases
cycle_budget: 3                                      # completed planning cycles per session before HAND OFF
wrap_every: 5                                        # in-session batch-boundary cadence (reaper + retro + reconcile)
last_wrapup_total: 0
cycles_this_session: 0                               # reset to 0 on RESUME
cycles_total: 0                                      # carried across sessions
ledger: dimensions 0/<n> verified ; targets 0/<b> planned
last_item: (none — mapping only)
status: MAP pending — targets not yet derived
last_update: <UTC>
