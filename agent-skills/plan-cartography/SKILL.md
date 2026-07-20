---
name: plan-cartography
description: >-
  Turn a target subsystem into a persistent CODE GRAPH and graph-intelligence — built ONLY from
  `git-kb code` JSON (no C dep, no graph DB), snapshotted + diffed each cycle so the graph "updates".
  ALWAYS use to map a planning target, on "map the code", "build the code graph", "what are the
  hotspots / dead code / blast radius", "symbol & data-flow map", "what should we plan", "derive
  targets", the pre-DONE completeness sweep, AND follow-ups — "re-run the map", "update the graph",
  "redo only the graph for <target>", "dig deeper on <subsystem>". Graph + metrics, not grep; read-only
  on target code. Used by `plan-cartographer`.
---

# plan-cartography — code graph + graph-intelligence (R3b + R8)

Orient a planning target by **materializing its code into a graph and deriving intelligence from it**:
symbols, call edges, entry points, data flows, then metrics (centrality/hotspots, blast-radius, dead
code, cycles, layering-violations, public-API surface). The graph is the substrate every downstream
agent queries — the analyst scopes risk by blast-radius and prioritizes by centrality, the verifier
refutes "safe/unused" claims with callers/impact, the architect renders the ASCII graph and grounds
the roadmap. Built **only** from `git-kb code` JSON — plain JSON + markdown, **no C dependency, no
graph database** (the repo's NON-NEGOTIABLE no-C-in-trust-boundary invariant applies to *us* too).
**Read-only** on the target's production code.

The graph is **persistent and versioned**: each cycle snapshots it under `.handoff/loop/plan/graph/`
and **diffs against the previous committed snapshot** — the diff IS the update (added/removed symbols,
new/severed edges, entrypoint/flow changes). Used by `plan-cartographer`.

> **Schemas live in `references/graph-schema.md`** — the exact JSON store shape for every artifact and
> the full per-metric recipe table. This file is the imperative method; that file is the contract.

## Method — 4 steps per cycle

Let `<T>` = the target slug (crate/subsystem, e.g. `secrets-proto`); `<root>` = its abs `target_root`.

### 1. MATERIALIZE — index, then snapshot the graph as JSON

1. **Index** the target (idempotent; the daemon also re-indexes on save):
   `git-kb code index <root> --force`, then confirm with `git-kb code stats --json`.
   - **Branch-scoping (the #1 empty-result cause — verified live).** `git-kb code` symbols and queries
     are scoped to the **current git branch** (every result carries a `"branch"` field; the index is
     built per-branch). On a fresh loop worktree branch — and **every cycle runs in a fresh
     `meta/.worktrees/<slug>` branch** — symbols indexed on `master`/`develop` are INVISIBLE:
     `symbols --json` returns `{"symbols":[],"count":0}` even while `stats` reports thousands of symbols
     (on another branch). So ALWAYS `git-kb code index <root>` on the current branch FIRST to populate
     the graph for the branch you are planning on. To deliberately read another branch's graph, pass
     `--branch <name>`. Empty results *after* a fresh index of a non-empty target is a real finding
     (record it), not a tooling hiccup.
2. **Snapshot** the graph to JSON under `.handoff/loop/plan/graph/`. Capture, each `--json --refresh`:
   - `git-kb code symbols` (scoped to `<root>`) → `graph/<T>.symbols.json`
   - `git-kb code callers <sym>` + `callees <sym>` for the in-scope symbols, `entrypoints`, `flows`
     (+ `flow <id>` for each traced flow) → `graph/<T>.callgraph.json` (nodes = symbols, edges =
     caller→callee, plus the `entrypoints` and `flows` arrays). See `references/graph-schema.md` for
     the exact object shape — conform to it so diff/metrics are stable across cycles.
3. Record `graph_snapshot: graph/<T>.symbols.json@<git-sha>` in `loop_state.md` so the next cycle
   knows what it is diffing against.

Prefer `git-kb code` (AST / real call graph) over grep for anything about callers, callees, usage,
definitions, or impact — grep sees text, the graph sees edges. Grep stays appropriate for strings,
config, and docs only.

### 2. DIFF — delta vs the previous committed snapshot (this is how the graph "updates")

Compare the new `<T>.symbols.json`/`<T>.callgraph.json` against the **previous committed** snapshot of
the same files (`git show HEAD:...`); write the delta to `graph/<T>.diff.md`:

- **Symbols** added / removed / signature-changed.
- **Edges** new / severed (a call path that appeared or disappeared).
- **Entry points** and **flows** added / removed / re-routed.

On the first cycle for a target there is no prior snapshot → record `diff: baseline (first snapshot)`.
The diff is what makes the graph a *living* artifact: a re-run on the same target after code changed
surfaces exactly what moved, so the plan tracks reality.

### 3. DERIVE — graph-intelligence metrics → `graph/<T>.metrics.json`

Compute these from the snapshot + targeted queries; full recipes (inputs, exact command, output field)
are in `references/graph-schema.md`. Summary:

| Metric | How (recipe in references/graph-schema.md) |
|--------|--------------------------------------------|
| **centrality / hotspots** | in-degree + out-degree from the edge list, cross-checked with `git-kb code query hotspots --json` |
| **blast-radius** | `git-kb code impact <file> --depth N --json` per high-centrality file (transitive dependents) |
| **dead code** | `git-kb code dead --json` + `query dead-code-explain --json` (zero-caller symbols + *why*) |
| **cycles (SCC)** | **Tarjan's strongly-connected-components, run in-reasoning over the edge list — NO new library.** Any SCC with >1 node is a dependency cycle. Algorithm in references/graph-schema.md |
| **layering-violations** | `query cross-service-impact --json` cross-checked against the codemap's module boundaries (an edge that crosses a declared layer the wrong way) |
| **public-API surface** | `query public-api --json` (the exported contract — what consumers depend on) |

Each metric entry cites the symbol/file/edge it came from, so the verifier and architect can re-check
it. Do **not** invent numbers — if a query returns nothing, record the empty result honestly.

### 4. RENDER — `graph/<T>.graph.md` + the codemap, and tell downstream how to QUERY

1. **`graph/<T>.graph.md`** — a human ASCII view of the graph + metrics, diagram-ready for the
   architect: entry points → core symbols → leaf calls, with hotspots and cycles annotated. Use the
   box-drawing convention (`plan-synthesis` owns the full legend) and cite `Source: <T>.callgraph.json`.
2. **`reports/codemap-<T>.md`** — the structural map: modules/files, entry points, public surface,
   internal + the handful of shaping external deps, build/run surface. Ground every entry in a
   `path`/`symbol` so analysts cite real code. Map *behavior*, not just names.
3. **State how agents query the graph** at the top of the codemap: analyst → blast-radius (scope each
   upgrade's risk) + centrality (prioritize); verifier → `callers`/`impact` (refute "unused/safe");
   architect → render + ground the roadmap ordering.

## Seed the ledgers

- **`dimensions.md`** — from the graph + codemap, seed one `- [ ]` row per dimension the target needs:
  `architecture`, `data-flow`, `hotspots/coupling`, `dead-code`, `public-API/contracts`, `perf`,
  `correctness/accuracy`, `tooling` — pick what the target actually has (a pure proto crate won't need
  `perf`) — **plus always seed `test-coverage`** (owned by `plan-test-strategist`; every target gets a
  test-coverage analysis). Row format and status legend are in `references/graph-schema.md` (adapted
  from the `code-research` research-ledger). The analyst/strategist fill these; the verifier gates them.
- **`targets.md` auto-derivation** — when no explicit target list is supplied, enumerate the repo's
  **Cargo workspace members** (`cargo metadata --no-deps --format-version 1` → `packages[].name`, or
  parse the root `Cargo.toml` `[workspace] members`) **+ the major modules** of large crates, writing
  one `- [ ] <T>: <one-line>` each. An owner-supplied target list in the invocation **OVERRIDES** the
  auto-derived list for that run.

## Pre-DONE completeness sweep (the anti-silent-cap gate)

Before the loop may write `DONE`, **re-derive the target's expected surface from the graph** —
modules, entry points, public-API — and diff it against what `dimensions.md` actually examined. Any
major module/interface/dimension unexamined → it is NOT done: emit an explicit `- [ ]` row and the
loop stays open. A partial or zero re-derivation → **INCONCLUSIVE → NEEDS-HUMAN**, never DONE. No
silent caps: deferred areas are explicit rows, not omissions. Record the sweep result so the
orchestrator can stamp it into `DONE`.

## Discipline

- **Graph, not grep** — callers/callees/impact/hotspots come from `git-kb code`; text search is for
  strings/config/docs only.
- **No C, no graph DB** — JSON + markdown artifacts only. Tarjan SCC runs in-reasoning over the edge
  list; do **not** add a graph-library dependency. (This is the same invariant we're helping plan.)
- **Read-only on target code** — cartography never edits the subsystem; the only writes are under
  `.handoff/loop/plan/`.
- **rtk gotcha** — the rtk hook rewrites `cargo`/`git`/`git-kb` and can corrupt exit codes and JSON
  diagnostics. Get clean output with **`rtk proxy git-kb code … --json`** or by redirecting to a file
  (`git-kb code symbols --json > graph/<T>.symbols.json`), never by piping JSON through the filter.
- **Cite the source of every metric** — a number with no symbol/file/edge behind it is a guess the
  verifier will drop.
- **Diff or it didn't update** — a re-run that doesn't write `<T>.diff.md` hasn't proven the graph
  reflects current code.

## References
- `references/graph-schema.md` — exact JSON store schema for every `graph/<T>.*` artifact + the full
  per-metric recipe table (centrality, blast-radius, dead, SCC/Tarjan, layering, public-API) + the
  dimension/claim/upgrade row formats.
