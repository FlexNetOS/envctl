# Codemap — rusty-idd

The intent-driven control-plane organ of the FlexNetOS `meta` fleet: the *why/what* plane
(OpenSpec lifecycle + IDD + merge-goal workflows). Pure-Rust Cargo workspace, single binary
`rusty-idd`. Read-only map; every row cites a real path. Graph metrics in
`graph/rusty-idd.md`; machine form in `graph/rusty-idd.json`.

## Entrypoints

| entrypoint | path | role |
|---|---|---|
| `rusty-idd` binary | `crates/cli/src/main.rs::main` (line 4) | the one product entrypoint; delegates to `crates/cli/src/lib.rs` |
| TUI launch | `crates/cli/src/commands/tui.rs` → `crates/tui` (`App::new` @ `app.rs:114`) | ratatui OpenSpec TUI |
| (no daemon/server) | — | no HTTP routes inside crates/ (`cross-service-impact` empty) |

`code entrypoints` confirms exactly one `main_function` in crates/ (`crates/cli/src/main.rs`);
the other 5 are `#[test]` binaries in `crates/cli/tests/`.

## The 11 workspace members (`Cargo.toml` `members`)

| crate | manifest name | role | public surface | internal deps |
|---|---|---|---|---|
| **cli** | `rusty-idd` (bin) + `rusty-idd-cli` (lib) | clap wiring; the front door. `commands/` = 17 modules | 74 public syms; blast 2 | core, knowledge, merge-tools, spec, runner, tui |
| **core** | `rusty_idd_core` | std-only zero-dep IDD primitives: planner, scanner, manifest, env_contract, validation, templates | 71 public syms; blast 0 (facade) | — |
| **runner** | `rusty_idd_runner` | non-UI execution layer: `runner.rs` (72 KB), `data.rs` (60 KB), `config.rs` (23 KB) | blast **803** (highest) | — |
| **tui** | `rusty_idd_tui` | ratatui app: `app.rs` (190 KB), `ui.rs` (83 KB) | blast 248 | runner |
| **spec** | `rusty_idd_spec` | OpenSpec engine: parse / model / merge / validate / archive / scaffold / adr / schema | blast 0 (facade) | — |
| **merge-tools** | `rusty-idd-merge-tools` | reusable merge-goal workflow package (`MergeToolPackage`, `verify_workspace`) | blast 11 | — |
| **knowledge** | `rusty_idd_knowledge` | `.idd/knowledge` engine + architecture-diagram render; **241 KB single lib.rs** | 18 dead; blast 105 | external/codegraph-core, external/codegraph-parser, core |
| **work-order** | `work-order` | `handoff.task.v1` envelope (S1 spike): `lib.rs`, `intake.rs` | 24 dead (unconsumed) | — |
| **external/codegraph-core** | `codegraph-core` | vendored codegraph-rust core (node/watch/config/perf) | 355 public, 120 dead | — |
| **external/codegraph-parser** | `codegraph-parser` | vendored tree-sitter parser/visitor/semantic/diff | 62 dead | external/codegraph-core |
| **external/repomix-shared** | `repomix-shared` | tiny shared types (`patch.crates-io` override) | trivial | — |

Module boundaries are clean (the crate graph is a DAG, no cycles — `graph/rusty-idd.md`).
`cli` is the only node that depends on the user-facing crates; `external/codegraph-core` is the
deepest shared leaf (via `knowledge`).

### Cross-crate edges (the backbone)

```
cli ──▶ core, spec, merge-tools, runner, tui, knowledge
tui ──▶ runner
knowledge ──▶ codegraph-core, codegraph-parser, core
codegraph-parser ──▶ codegraph-core
(work-order, repomix-shared: isolated)
```

## CLI command surface (`crates/cli/src/commands/`)

17 command modules: `spec.rs`, `spec_status.rs`, `spec_archive.rs`, `spec_adr.rs`,
`spec_scaffold.rs`, `spec_plan_integration.rs` (the OpenSpec lifecycle); `merge_tools.rs`
(`rusty-idd merge-tools show`); `knowledge.rs`; `codex.rs` (**60 KB** — Codex/agent harness
integration, scaffolds `.idd` + `openspec`); `harness.rs`; `deploy.rs`; `render.rs`; `run.rs`;
`next.rs`; `core.rs`; `tui.rs`. The control-plane verbs the CLAUDE bridge names — `spec status`,
`spec archive`, `merge-tools show` — map to `spec_status.rs`, `spec_archive.rs`, `merge_tools.rs`.

## Convergence surfaces (rusty-idd → the one fabric)

How rusty-idd attaches to the fleet's organs — and, critically, where it **does not**:

| fleet organ | rusty-idd integration | evidence |
|---|---|---|
| **handoff / continuity (`.handoff`)** | **filesystem + schema contract**, not a lib dep. Reads `.handoff/tasks` (`commands/codex.rs:593`); `merge-tools` declares `_workspace/{backlog,loop_state,HANDOFF}.md` (`lib.rs:110`); **`work-order` IS the `handoff.task.v1` envelope** (`work-order/src/lib.rs:35-45`). Also VENDORED 3× (`third_party/upstream/handoff`, `imports/handoff`). | grep, manifests |
| **OpenSpec lifecycle** | first-class, in-crate (`crates/spec` engine + 6 `spec_*` CLI cmds). `SpecDoc.contains` is the #1 hotspot (842 callers). | `crates/spec/`, `graph/rusty-idd.md` |
| **merge / grit** | rusty-idd has its **own** `merge-tools` workflow package; **NO `grit` reference in product code**. | `crates/merge-tools/`, grep |
| **memory / icm** | **NONE in product code** — `icm` appears only in `crates/cli/tests/harness_cli.rs` as a harness-checker contract concept. | grep |
| **comms / weave (A2A)** | **NONE** — zero `weave`/`a2a` references in product code. | grep |
| **`hf` kernel** | **NONE as a library/IPC dep** — coupling is purely the `.handoff/`/`handoff.task.v1` *file+schema* contract above. | grep, manifests |
| **`.idd` / `openspec` dirs** | scaffolded/managed by `commands/codex.rs` (lines 191-213) and the spec engine. | grep |

**Headline for the architect:** rusty-idd converges with the fleet through **filesystem
contracts and JSON schemas** (`.handoff/`, `.idd/`, `openspec/`, `_workspace/`,
`handoff.task.v1`), **not** through live library/IPC bindings. weave (comms), icm (memory),
grit (merge), and the `hf` kernel are **absent** from product code — the path *into* the one
fabric is the gap to plan. `work-order` is the one seam already shaped to the handoff schema but
not yet consumed (24 dead symbols).

## Build/run surface

- Workspace: `resolver = "3"` (members mix editions: core 2021, tui/runner 2024), rust 1.88,
  `[patch.crates-io] repomix-shared` path override.
- Build/run: `cargo build` / `cargo run -p rusty-idd-cli -- <cmd>` (binary `rusty-idd`).
- Tests live in `crates/{cli,core}/tests/` (`codex_cli.rs`, `harness_cli.rs`, `adr_check_cli.rs`,
  `smoke.rs`).

## Anomalies / non-crate dirs (findings)

- `crates/config/` is **not a workspace member** — only `crates/config/example.toml`, no
  `Cargo.toml`. Per the workspace comment, `runner` absorbed config. Stray dir (F3).
- `third_party/upstream/` + `imports/` hold full duplicate crate trees (handoff, codegraph-rust,
  prompt_hub, prompts.chat) — vendored, not workspace members; they inflate the raw index but
  are excluded from all crates/-scoped metrics (F1).

## Pre-DONE completeness sweep — expected public surface per crate

A later sweep must confirm each is examined; none were silently capped:

- [ ] **cli** — `main` + 17 `commands/*` modules + `lib.rs` (74 public syms)
- [ ] **core** — planner, scanner, manifest, env_contract, validation, templates, model, cli, fs_utils (71 public syms)
- [ ] **runner** — runner.rs, data.rs, config.rs (blast 803 — highest-risk)
- [ ] **tui** — app.rs (190 KB), ui.rs, lib.rs
- [ ] **spec** — parse/model/merge/validate/archive/scaffold/adr/schema (30 dead to reconcile)
- [ ] **merge-tools** — MergeToolPackage, package(), render_markdown, verify_workspace
- [ ] **knowledge** — lib.rs (241 KB), architecture-diagram engine
- [ ] **work-order** — handoff.task.v1 envelope + intake (24 dead — integration status)
- [ ] **external/codegraph-core** — vendored; 355 public / 120 dead (slim candidate)
- [ ] **external/codegraph-parser** — vendored; 62 dead (slim candidate)
- [ ] **external/repomix-shared** — trivial shared types
- [ ] **Convergence axis** — weave/icm/grit/hf integration (currently ABSENT)

**Sweep verdict basis:** re-derivation from the graph was non-zero for all 11 crates (symbols,
entrypoints, public-api all populated) → sweep can proceed. `public-api` (≥500) and `dead`
(≥278) results were truncated at the tool's 500 cap — re-run with explicit per-crate `symbols`
queries before any DONE that depends on an *exhaustive* public-surface count.
