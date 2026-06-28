# Planning backlog — grit instance (parallel run via plan-loop-parallel-run.md)
#
# This is an isolated per-target instance (plan/loop-grit, off origin/master). It plans ONE target:
# grit. Shared-state files here are grit-scoped (per-target write discipline §7). The full fleet
# backlog lives on the union loop branch (plan/fleet-convergence-first-run).
#
# Legend: [ ] todo  [~] in-flight / planned-with-gaps  [x] planned+verified  [!] blocked  [!!] SUPERVISED

## Cycle 5 (this instance)
- [~] grit: symbol-level merge/lock substrate — enables union step 2 (dedup ~95% shared handoff/rusty-idd crates)
