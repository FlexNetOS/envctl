# rusty-idd — Graph Diff

**No prior snapshot — baseline established this run.** This is the FIRST cartography run for
`rusty-idd`; there is no previous committed `graph/rusty-idd.symbols.json@<sha>` to diff against.
Per fail-closed honesty: no delta is computed (none can be). Future runs diff against the
baseline counts below.

## Baseline (snapshot reference)

- snapshot: `graph/rusty-idd.symbols.json@<git-sha at commit>` (branch `plan/lifeos-meta-front-door`)
- source: `git-kb code` AST index, scoped to `crates/` (11 workspace members)

| metric | baseline value | notes |
|---|---|---|
| symbol_count (whole repo) | 19429 | includes vendored third_party/imports trees |
| file_count (whole repo) | 1234 | |
| resolved_call_edges | 35647 | |
| unresolved_calls | 94967 | mostly vendored + stdlib |
| symbols returned in crates/ | 500 | truncated at git-kb 500-row cap (lower bound) |
| entrypoints (crates/) | 6 | 1 product `main` + 5 test bins |
| hotspots (crates/, with callers) | 43 | top: `SpecDoc.contains` (842) |
| public-api (crates/) | ≥500 | truncated at 500-cap |
| dead-code (crates/) | ≥278 | truncated at 500-cap; 182 in vendored codegraph |
| cross-crate cycles (Tarjan SCC) | 0 | clean DAG |
| internal HTTP routes | 0 | CLI+TUI+lib, not a service |
| max blast radius | 803 | `crates/runner/src/runner.rs` |

## What future diffs will compare

On the next cycle, diff against this baseline and report:
- **new/removed symbols** (by `symbol_id` set difference over `rusty-idd.symbols.json`)
- **edge churn** (crate_dependency_edges added/removed; symbol in-degree movement in hotspots)
- **metric movement** (Δ symbol_count, Δ dead-code, Δ hotspot caller_counts, Δ max blast radius)
- **structural events** (any new cross-crate SCC cycle = regression; any new HTTP route = service
  surface appearing; convergence-organ refs appearing for weave/icm/grit/hf = integration progress)

## Convergence watch (baseline = ABSENT)

Future diffs should flag the FIRST appearance of product-code references to: `weave` (comms),
`icm` (memory), `grit` (merge), `hf`-kernel (continuity lib/IPC). Baseline: all absent;
`work-order` already shaped to `handoff.task.v1` but unconsumed (24 dead symbols).
