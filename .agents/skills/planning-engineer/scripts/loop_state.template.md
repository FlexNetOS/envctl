# Loop state — planning-engineer (plan-loop)
session_started: <UTC e.g. 2026-06-25T16:00:00Z>   # you supply it; the runtime can't read the clock
loop: planning-engineer
branch: <branch>
worktree: <abs path>
planning_target: <current T (crate/subsystem slug), or "(sweeping targets.md)">
target_root: <abs path of the subsystem/crate under plan, e.g. ~/Desktop/meta/envctl/crates/secrets-proto>
recency_window_days: 90                              # the rolling web-research window (R3a)
graph_snapshot: graph/<T>.symbols.json@<git-sha>     # which snapshot the metrics derive from
cycle_budget: 3                                      # completed planning cycles per session before HAND OFF
wrap_every: 5                                        # in-session batch-boundary cadence (reaper + retro + reconcile)
last_wrapup_total: 0
cycles_this_session: 0                               # reset to 0 on RESUME
cycles_total: 0                                      # carried across sessions
ledger: dimensions 0/<n> verified ; targets 0/<b> planned
last_item: (none — mapping only)
status: MAP pending — targets not yet derived
last_update: <UTC>
