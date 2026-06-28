# Graph diff — prompt-hub

**Baseline snapshot — no prior committed graph snapshot for target `prompt-hub`.**

- This is the first `git-kb code` graph captured for this target (cycle 6).
- Snapshot of record: `graph/prompt-hub.symbols.json@f826ea33`
  (branch `plan/fleet-arch-integration-cycle1`, captured 2026-06-27).
- No `new/removed symbols`, `edge churn`, or `metric movement` deltas can be computed
  this cycle. The next cycle diffs against this baseline.

## Baseline counts (for next cycle's delta)
| Metric | Value |
|---|---|
| member symbols | 3,589 |
| intra-repo call edges (non-test) | 4,006 |
| public src symbols | 1,405 |
| server routes | 111 |
| CLI top-level verbs | ~41 |
| SCC cycles (size>1) | 4 (3 artifacts + 1 plausible) |
| real layering violations | 0 |
| dead-code candidates (src) | 416 |
| top fan-in (architectural) | PromptHub::lock (76) |

## Caveat carried forward
Edge resolution required scoping `git-kb code index` to each member's `src/` (the
vendor-inclusive full index resolved 0 edges). The next cycle must reproduce that scoping
or edge deltas will be spurious. Record in `loop_state.md`:
`graph_snapshot: graph/prompt-hub.symbols.json@f826ea33`.
