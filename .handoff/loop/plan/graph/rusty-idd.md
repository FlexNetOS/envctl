# rusty-idd — Code Graph Intelligence

Source: `git-kb code` (tree-sitter AST call-graph), index run on branch
`plan/lifeos-meta-front-door`. **All metrics below are scoped to `crates/`** (the 11 real
workspace members). The index also swept `third_party/upstream/*` and `imports/*`, which are
**vendored duplicate copies** of `handoff`, `prompt_hub`, `codegraph-rust`, `prompts.chat`
(finding F1) and are excluded from every metric here.

## Index health (whole repo, `code doctor`)

| metric | value |
|---|---|
| symbol_count | **19429** (PASS, >0) |
| file_count | 1234 |
| call_edge_count | 35647 (resolved) |
| unresolved_call_count | 94967 (no_match 39998 / skip_list 38757 / ambiguous 12675 / stdlib 3537) |
| language_breakdown | rust 17214, **typescript 2089** (all vendored), js 82, py 43, php 1 |

The 2089 TS / 82 JS / 43 PY / 1 PHP symbols are **100% vendored** (`third_party/`, `imports/`):
rusty-idd's own product code is pure Rust. The high unresolved ratio (~73%) is dominated by
the vendored trees and stdlib/external crate calls — not a defect in the product crates.

## Crate dependency graph (from Cargo manifests)

```
                          ┌─────────────────────┐
                          │  crates/cli          │  bin: `rusty-idd`  (entrypoint)
                          │  src/main.rs::main   │  blast=2 (thin wiring facade)
                          └──────────┬──────────-┘
        ┌──────────┬──────────┬──────┼─────────┬───────────────┐
        ▼          ▼          ▼      ▼         ▼               ▼
   ┌────────┐ ┌────────┐ ┌────────┐ ┌──────┐ ┌──────────┐ ┌──────────────┐
   │ core   │ │ spec   │ │ merge- │ │runner│ │   tui    │ │  knowledge   │
   │blast=0 │ │blast=0 │ │ tools  │ │ ★803 │ │  ★248    │ │   ★105       │
   │facade  │ │facade  │ │blast=11│ │      │ │          │ │              │
   └────────┘ └────────┘ └────────┘ └──▲───┘ └────┬─────┘ └──┬────────┬──┘
                                       │          │          │        │
                                       └──────────┘          │        │
                                       tui→runner            ▼        ▼
                                              ┌──────────────────┐ ┌────────┐
                                              │external/codegraph│ │  core  │
                                              │     -parser      │ └────────┘
                                              └────────┬─────────┘
                                                       ▼
                                              ┌──────────────────┐
                                              │external/codegraph│
                                              │      -core       │
                                              └──────────────────┘

   Unlinked leaves (no internal edges): work-order, external/repomix-shared
   ★ = blast radius (transitive callers, `code impact --depth 3`)
```

Edges (10): cli→{core,spec,merge-tools,runner,tui,knowledge}; tui→runner;
knowledge→{external/codegraph-core, external/codegraph-parser, core};
codegraph-parser→codegraph-core.

## Cycles — Tarjan SCC (in-process, over the crate edge list)

**Verdict: clean DAG. Zero cross-crate cycles.** Every SCC is a singleton. `cli` is the
unique sink-of-control (depends on everything; nothing depends on it). `external/codegraph-core`
is the deepest shared leaf. (Symbol-level SCC not computed: `git-kb code` exposes no full-edge
dump and `code flows` returned `[]` — recorded as a method limit, not a clean pass. The crate
DAG is the authoritative coupling view.)

## Hotspots / centrality (top in-degree, `query hotspots`, crates/ only)

| callers | symbol |
|---|---|
| 842 | `crates/spec/src/model/spec.rs::method::SpecDoc.contains` |
| 355 | `crates/tui/src/app.rs::method::App.new` |
| 343 | `crates/runner/src/config.rs::method::TuiConfig.default` |
| 221 | `crates/runner/src/runner.rs::method::BatchImplState.new` |
| 159 | `crates/external/codegraph-core/src/node.rs::method::CodeNode.new` |
| 137 | `crates/core/src/model.rs::method::RepoInventory.new` |
| 126 | `crates/tui/src/app.rs::method::App.handle_config_input` |
| 98  | `crates/knowledge/src/lib.rs::method::ArchitectureDiagramOptions.new` |
| 80  | `crates/external/codegraph-core/src/watch/mod.rs::method::IntelligentFileWatcher.new` |

`SpecDoc.contains` (the OpenSpec model query primitive) is the single most-called symbol in the
product — the OpenSpec model is the gravitational center of the control plane.

## Blast radius (transitive callers, `code impact --depth 3`)

| file | total_count | reading |
|---|---|---|
| `crates/runner/src/runner.rs` | **803** | execution core — highest-risk surface to change |
| `crates/tui/src/app.rs` | 248 | the 190 KB ratatui app (god-file candidate, F4) |
| `crates/knowledge/src/lib.rs` | 105 | 241 KB single-file knowledge engine (god-file candidate, F4) |
| `crates/merge-tools/src/lib.rs` | 11 | small, contained merge-workflow package |
| `crates/cli/src/lib.rs` | 2 | thin facade |
| `crates/core/src/lib.rs` | 0 | pure re-export facade |
| `crates/spec/src/lib.rs` | 0 | pure re-export facade |

## Dead code (`code dead`, crates/ only)

≥**278** symbols with zero resolved callers (result truncated at the 500 cap, so a lower bound):

| count | crate | note |
|---|---|---|
| 120 | external/codegraph-core | vendored lib — most surface unused by rusty-idd |
| 62  | external/codegraph-parser | vendored lib — same |
| 30  | spec | OpenSpec engine API not yet fully wired to CLI |
| 24  | work-order | S1 spike envelope — not yet consumed in product flow |
| 20  | core | |
| 18  | knowledge | |
| 4   | merge-tools | |

182/278 (65%) of dead code is in the two vendored `codegraph` crates → candidate to slim to a
feature-gated subset (F2). `work-order`'s 24 dead symbols confirm it is a **spike not yet
integrated** (F5).

## Layering / public-API

- `query cross-service-impact` inside `crates/` → **empty**: rusty-idd defines **no internal
  HTTP service routes** (the 114 routes the index found are all vendored prompt_hub). It is a
  **CLI + TUI + library**, not a service. No layering violations of a service boundary exist
  because no service boundary exists.
- `query public-api` → ≥**500** public symbols in crates/ (truncated at 500). Visible
  concentration: external/codegraph-core 355, cli 74, core 71 — i.e. the vendored library
  exposes the widest public surface, larger than the product's own.

## Findings index (for analysts/architect)

- **F1** vendored duplication: `third_party/upstream/handoff`, `imports/handoff`,
  `third_party/upstream/codegraph-rust`, `imports/prompt_hub`, `third_party/upstream/prompts.chat`
  are full duplicate crate trees inflating the index (handoff appears 3×).
- **F2** vendored `codegraph-{core,parser}` carry 182 dead symbols — slim/feature-gate.
- **F3** `crates/config/` is **not a crate** (only `example.toml`, no Cargo.toml, not a workspace
  member) — stray dir; runner absorbed config per the workspace comment.
- **F4** god-files: `tui/src/app.rs` 190 KB (blast 248), `knowledge/src/lib.rs` 241 KB single
  file (blast 105), `runner/src/runner.rs` 72 KB (blast 803).
- **F5** `work-order` (handoff.task.v1 envelope) is a wired-but-unconsumed S1 spike (24 dead).
- **Convergence** (see codemap §Convergence): integration with the fleet is via **filesystem
  contracts + schema**, not live libs. `weave`/`icm`/`grit`/`hf`-kernel have **no product-code
  dependency** — absence is the headline finding for the architect.
