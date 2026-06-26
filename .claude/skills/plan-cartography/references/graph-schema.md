# plan-cartography — code-graph store schema + per-metric recipes

The exact on-disk shape of every `.handoff/loop/plan/graph/<T>.*` artifact and the precise recipe for
every derived metric. The cartographer writes to this contract so the **diff** (cycle N vs N−1) and
the **metrics** are stable and re-checkable across cycles, and so the verifier/architect can re-derive
any number. Built **only** from `git-kb code` JSON — plain JSON + markdown, **no C dependency, no
graph database**.

## Table of contents
- [0. Conventions](#0-conventions)
- [1. The `git-kb code` surface](#1-the-git-kb-code-surface)
- [2. `graph/<T>.symbols.json` — symbol snapshot](#2-graphtsymbolsjson--symbol-snapshot)
- [3. `graph/<T>.callgraph.json` — edges + entrypoints + flows](#3-graphtcallgraphjson--edges--entrypoints--flows)
- [4. `graph/<T>.metrics.json` — derived intelligence](#4-graphtmetricsjson--derived-intelligence)
- [5. Per-metric recipes](#5-per-metric-recipes)
- [6. `graph/<T>.diff.md` — the update](#6-graphtdiffmd--the-update)
- [7. `graph/<T>.graph.md` — ASCII view](#7-graphtgraphmd--ascii-view)
- [8. `dimensions.md` + claim/upgrade row formats](#8-dimensionsmd--claimupgrade-row-formats)
- [9. `targets.md` auto-derivation](#9-targetsmd-auto-derivation)
- [10. rtk gotcha](#10-rtk-gotcha)

---

## 0. Conventions

- `<T>` = target slug (crate/subsystem, e.g. `secrets-proto`). `<root>` = abs path of `target_root`.
- All JSON written via `git-kb code … --json` (the stable, complete output) **redirected to a file**
  or run through `rtk proxy` (see §10). Never hand-edit the snapshot JSON — it is machine-derived.
- All paths inside artifacts are repo-relative so a snapshot is comparable across worktrees.
- Every metric entry carries the `path`/`symbol`/`edge` it came from. A bare number with no source is
  invalid — drop it rather than guess.
- The snapshot files are committed each cycle; the **previous committed** version is the diff baseline.

## 1. The `git-kb code` surface

All subcommands accept `--json` (prefer it) and the relevant ones accept `--refresh` (rebuild derived
tables first). The cartographer uses exactly this surface:

| Command | Purpose |
|---------|---------|
| `git-kb code index <root> [--force]` | Index source symbols (idempotent; `--force` refreshes changed/unchanged files) |
| `git-kb code symbols [--json]` | List indexed symbols (filter by file/kind/lang) |
| `git-kb code callers <sym> [--json]` | Callers of a symbol (real call sites, AST) |
| `git-kb code callees <sym> [--json]` | Callees of a symbol |
| `git-kb code impact <file> --depth N [--json]` | Transitive blast radius for a file via the call graph |
| `git-kb code dead [--json]` | Symbols with zero callers (candidate dead code) |
| `git-kb code refs <sym> [--json]` | KB documents referencing a code symbol |
| `git-kb code entrypoints [--json]` | Inferred entry points (mains, servers, CLIs, exported APIs) |
| `git-kb code flows [--json]` / `flow <id> [--json]` | Traced execution flows / one flow's path |
| `git-kb code stats --json` | Index statistics (use to confirm a warm index) |
| `git-kb code query <template> [--json] [--depth N] [--target S] [--limit N] [--refresh]` | Typed graph queries (§ below) |

`query` templates (exact `[possible values]`):
`hotspots`, `public-api`, `entrypoints`, `unresolved-by-reason`, `cross-service-impact`,
`dead-code-explain`, `routes`, `route-clients`, `handler-routes`.

## 2. `graph/<T>.symbols.json` — symbol snapshot

Store the raw `git-kb code symbols --json` payload for the in-scope files, wrapped with provenance so
the diff has a stable anchor:

```json
{
  "target": "secrets-proto",
  "target_root": "crates/secrets-proto",
  "git_sha": "a97c96a",
  "captured_at": "2026-06-25T00:00:00Z",
  "source_cmd": "git-kb code symbols --json",
  "symbols": [
    {
      "name": "MintReq",
      "kind": "struct",
      "language": "rust",
      "path": "crates/secrets-proto/src/lib.rs",
      "line": 142,
      "signature": "pub struct MintReq { ... }",
      "visibility": "pub"
    }
  ]
}
```

Keep the `symbols[]` objects as `git-kb code` returns them (do not drop fields); add only the top-level
provenance block. `visibility`/`pub` is what feeds the public-API metric.

## 3. `graph/<T>.callgraph.json` — edges + entrypoints + flows

The call graph as **nodes + directed edges**, plus the entrypoint and flow arrays. Edges are the
substrate for centrality, blast-radius and Tarjan SCC.

```json
{
  "target": "secrets-proto",
  "git_sha": "a97c96a",
  "captured_at": "2026-06-25T00:00:00Z",
  "source_cmds": [
    "git-kb code callers <sym> --json",
    "git-kb code callees <sym> --json",
    "git-kb code entrypoints --json",
    "git-kb code flows --json", "git-kb code flow <id> --json"
  ],
  "nodes": [
    { "id": "crates/secrets-proto/src/lib.rs::MintReq", "name": "MintReq", "kind": "struct" }
  ],
  "edges": [
    { "from": "crates/secretd/src/grpc.rs::mint_github", "to": "crates/secrets-proto/src/lib.rs::MintReq", "kind": "uses" }
  ],
  "entrypoints": [
    { "id": "crates/secretd/src/grpc.rs::mint_github", "kind": "grpc-handler", "path": "crates/secretd/src/grpc.rs", "line": 88 }
  ],
  "flows": [
    { "id": "flow-3", "name": "mint-github", "path": ["...::mint_github", "...::MintReq", "...::vault_open"] }
  ]
}
```

- `id` = stable `path::symbol` so the same node is recognizable across cycles (basis of the diff).
- `edges[].kind` ∈ {`calls`, `uses`, `impl`, …} as `git-kb code` reports it.
- Build `edges` by unioning `callees` (from→to) and `callers` (to←from, normalized to from→to) over the
  in-scope symbols; de-duplicate identical edges.

## 4. `graph/<T>.metrics.json` — derived intelligence

All six metrics in one object; every entry cites its source. The analyst/verifier/architect read this.

```json
{
  "target": "secrets-proto",
  "git_sha": "a97c96a",
  "metrics": {
    "hotspots": [
      { "symbol": "MintReq", "id": "crates/secrets-proto/src/lib.rs::MintReq",
        "in_degree": 7, "out_degree": 1, "centrality": 8,
        "source": "edges + query hotspots" }
    ],
    "blast_radius": [
      { "file": "crates/secrets-proto/src/lib.rs", "depth": 3, "dependents": 14,
        "dependent_files": ["crates/secretd/src/grpc.rs", "..."],
        "source": "impact crates/secrets-proto/src/lib.rs --depth 3 --json" }
    ],
    "dead_code": [
      { "symbol": "legacy_encode", "id": "crates/secrets-proto/src/legacy.rs::legacy_encode",
        "reason": "no callers; not an entrypoint", "source": "dead + query dead-code-explain" }
    ],
    "cycles": [
      { "scc": ["...::a", "...::b"], "size": 2, "source": "tarjan(edges)" }
    ],
    "layering_violations": [
      { "from": "...", "to": "...", "violation": "proto -> daemon (upward dep)",
        "source": "query cross-service-impact vs codemap boundaries" }
    ],
    "public_api": [
      { "symbol": "SecretService", "id": "crates/secrets-proto/src/lib.rs::SecretService",
        "kind": "tonic-service", "source": "query public-api" }
    ]
  }
}
```

An empty metric is recorded as `[]` with a note (e.g. `"dead_code": []  // none found`), never omitted.

## 5. Per-metric recipes

Run each recipe, then write the result into `metrics.json` (§4). Cite the exact command in `source`.

| # | Metric | Inputs | Exact recipe | Output field |
|---|--------|--------|--------------|--------------|
| 1 | **centrality / hotspots** | `<T>.callgraph.json` edges; `query hotspots` | For each node, `in_degree` = #edges with `to==id`, `out_degree` = #edges with `from==id`; `centrality = in_degree + out_degree`. Cross-check the top-N against `git-kb code query hotspots --json` (the resolver's own central-code view); reconcile and note any divergence. | `metrics.hotspots[]` |
| 2 | **blast-radius** | high-centrality files | For each hotspot file: `git-kb code impact <file> --depth N --json` (start `N=2`, raise to `3–4` for spine files). Record `dependents` count + `dependent_files`. This is the transitive set that breaks if the file's contract changes — the analyst uses it to scope each upgrade's risk. | `metrics.blast_radius[]` |
| 3 | **dead code** | whole target | `git-kb code dead --json` for zero-caller symbols; `git-kb code query dead-code-explain --json` for *why* each is unreferenced (and to filter false positives — an entrypoint or `pub` API is not dead). | `metrics.dead_code[]` |
| 4 | **cycles (SCC)** | `<T>.callgraph.json` edges | **Tarjan's strongly-connected-components, computed in-reasoning over the edge list — NO library, NO graph DB.** Treat `nodes` as vertices and `edges` (from→to) as directed arcs; run Tarjan (DFS assigning `index`/`lowlink`, an on-stack set, pop an SCC when `lowlink==index`). Any SCC with **size > 1** is a dependency cycle (mutual recursion / circular module coupling) — a planning red flag. Record each non-trivial SCC's member ids. | `metrics.cycles[]` |
| 5 | **layering-violations** | `query cross-service-impact`; codemap module boundaries | `git-kb code query cross-service-impact --json`, then compare each cross-boundary edge against the **declared layering** captured in `reports/codemap-<T>.md` (e.g. proto must not depend on daemon; engine must not depend on CLI/GUI). An edge that crosses a boundary the wrong way is a violation. | `metrics.layering_violations[]` |
| 6 | **public-API surface** | `query public-api`; `pub` symbols | `git-kb code query public-api --json` (the exported contract consumers depend on); cross-check with `visibility=="pub"` symbols from `<T>.symbols.json`. This set defines what the completeness sweep must have examined and what an upgrade must not silently break. | `metrics.public_api[]` |

### Tarjan SCC — the algorithm to run (no library)

```
index = 0; stack = []; onstack = {}; idx = {}; low = {}; sccs = []
for v in nodes: if v not in idx: strongconnect(v)

strongconnect(v):
  idx[v] = low[v] = index; index += 1
  stack.push(v); onstack[v] = true
  for (v -> w) in edges:
      if w not in idx:        strongconnect(w); low[v] = min(low[v], low[w])
      elif onstack[w]:        low[v] = min(low[v], idx[w])
  if low[v] == idx[v]:                      # v is an SCC root
      comp = []
      repeat: w = stack.pop(); onstack[w] = false; comp.push(w) until w == v
      if len(comp) > 1: sccs.push(comp)      # size>1 ⇒ a real cycle
```

Report `sccs` (each member list) into `metrics.cycles[]`. Self-loops (a node calling itself) count as a
cycle too — note them explicitly.

## 6. `graph/<T>.diff.md` — the update

The delta vs the **previous committed** `<T>.symbols.json` / `<T>.callgraph.json`
(`git show HEAD:.handoff/loop/plan/graph/<T>.symbols.json`). This file is *how the graph updates*.

```markdown
# graph diff — <T> @ <new-sha> vs <prev-sha>

## Symbols
+ added:   <path::symbol> (<kind>)
- removed: <path::symbol>
~ changed: <path::symbol>  (signature: <old> -> <new>)

## Edges
+ new:     <from> -> <to>  (<kind>)
- severed: <from> -> <to>

## Entry points
+ / - / re-routed: <id>

## Flows
+ / - / re-routed: <flow-id> (<name>)

## Metric movement (optional, high-signal only)
- hotspot <symbol>: centrality <old> -> <new>
- new cycle: [<a>, <b>]
- new dead-code: <symbol>
```

First cycle for a target → `diff: baseline (first snapshot)` and skip the deltas.

## 7. `graph/<T>.graph.md` — ASCII view

A human, diagram-ready view of the graph + metrics for the architect to lift into the plan. Use
box-drawing characters and cite the source snapshot. (The full diagram legend — `[A]/[A*]/[P]/[H]/[!!]`
— belongs to `plan-synthesis`; here just render structure + annotate hotspots/cycles.)

```
Source: graph/<T>.callgraph.json @ <sha>

  entrypoints                core (hotspots *)              leaves
  ┌────────────────┐   ┌──────────────────────┐   ┌──────────────────┐
  │ grpc:mint_github│──▶│ MintReq *  (c=8)      │──▶│ vault_open        │
  └────────────────┘   │ SecretService (pub)   │   └──────────────────┘
                       └──────────┬────────────┘
                                  │  cycle: [A,B]  ⚠ SCC size 2
                                  ▼
                       ┌──────────────────────┐
                       │ legacy_encode  (dead) │
                       └──────────────────────┘
```

## 8. `dimensions.md` + claim/upgrade row formats

Adapt the `code-research` research-ledger schema; **extend it with the UPGRADE row** (the planning
addition). The cartographer seeds the dimension rows; the analyst writes CLAIM/UPGRADE rows into
`findings/<dim>.md`; the verifier writes VERDICT rows into `findings/verdicts.md`.

**Dimension row** (in `dimensions.md`):
```
- [ ] <id> · <area> · <the specific question this dimension answers> · deps: <ids|none>
```
Status legend: `- [ ]` not analyzed · `- [~]` analyzed, claims unverified · `- [x]` verified ·
`- [!]` blocked: `<reason>`.

Dimension catalog (pick what the target needs): `architecture`, `data-flow`, `hotspots/coupling`,
`dead-code`, `public-API/contracts`, `perf`, `correctness/accuracy`, `tooling`.

**CLAIM row** (analyst → `findings/<dim>.md`):
```
- CLAIM: <falsifiable statement> | evidence: <path:line / symbol / call-path / test> | confidence: high|medium|low
```

**UPGRADE row** (analyst → `findings/<dim>.md` — the planning extension):
```
- UPGRADE: <change> | axis: quality|speed|accuracy | rationale: <why> | evidence: <path:line> | blast: <impact-scope> | risk: low|med|high
```
`blast` is taken from `metrics.blast_radius` for the touched file; `axis` is mandatory and exactly one
of quality/speed/accuracy.

**VERDICT row** (verifier → `findings/verdicts.md`):
```
- <ref> -> CONFIRMED | REFUTED (<counter>) | QUALIFIED (<cond>) | INCONCLUSIVE (<why>)
```
Only `CONFIRMED`/`QUALIFIED` reach the plan; notable `REFUTED` overclaims are still reported as
findings (and infeasible upgrades are excluded and listed under gaps with the reason).

## 9. `targets.md` auto-derivation

When the invocation supplies no explicit target list, the cartographer enumerates the planning backlog:

1. **Cargo workspace members** — `cargo metadata --no-deps --format-version 1` → `packages[].name`
   (or parse the root `Cargo.toml` `[workspace] members`). One target per member.
2. **Major modules** of large crates — split a big crate into its top-level `mod`s where they are
   independently plannable.

Write one row each:
```
- [ ] <T>: <one-line what-it-is>
```
Status legend: `- [x]` planned · `- [!]` blocked/skip · `- [~]` in-flight. An owner-supplied target
list in the invocation **OVERRIDES** this auto-derived list for that run.

## 10. rtk gotcha

The `rtk` hook rewrites `cargo` / `git` / `git-kb` invocations and can corrupt exit codes and JSON
diagnostics — which silently breaks snapshot capture. Get clean, parseable output two ways:

- **Redirect to file:** `git-kb code symbols --json > .handoff/loop/plan/graph/<T>.symbols.json`
- **Bypass the filter:** `rtk proxy git-kb code query hotspots --json`

Never pipe `git-kb code … --json` *through* the rtk filter expecting valid JSON back.
