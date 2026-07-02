# Codemap — handoff (cycle 2, union with rusty-idd)

- **Target:** `handoff` — the continuity kernel: the `hf` CLI + the witnessed `.handoff` ledger, planned as the **union with rusty-idd** (owner north-star: `handoff + rusty-idd` union @ `$META_ROOT + handoff`).
- **Worktree (read-only):** `/home/drdave/Desktop/meta/.worktrees/plan-handoff-cycle2/handoff` @ `f6abf962413bafe164d56fa26b70b0a5fdacb8a2`.
- **Graph:** 2974 symbols (2128 own + 846 vendored), 141 files, 7265 resolved call edges. See `graph/handoff.{symbols,callgraph,metrics,graph}.json/md`.
- All claims below cite a path / symbol / graph row. Project docs (NORTH-STAR.md, FLEET_GUIDE.md, PRD/ADR refs in crate docs) state **intent** and are recorded as claims for the verifier, not as facts.

## 1. What this target is

A pure-Rust (`edition 2024`, `rust-version 1.96`, `unsafe_code = "deny"`) continuity kernel. `hf` is the hub binary; it claims/works/checkpoints/hands-off ONE witnessed task per cycle against a redb-authoritative `.handoff` ledger, with a tamper-evident witness chain. The work unit is the `handoff.task.v1` "work order" (`work-order` crate, schemars source-of-truth). The repo has been progressively de-monolithed: shared modules peeled out of the `hf` monolith into 13 `handoff-*` library crates (HFTASK-0081/0083, ADR-0019), toward a PRD 12+-crate target.

## 2. Crate roles + boundaries (21 Cargo workspace members)

### KERNEL group (16 crates)

| Crate | pkg name | sym | Role (from crate doc — claim to verify) |
|---|---|---|---|
| `hf/` | `hf` | 604 | The `.handoff` continuity CLI hub. 27 `cmd_*` verbs. Depends on all 15 below + RuVector. Two bins: `hf`, `hf-mcp`. |
| `ledger/` | `ledger` | 164 | The `.handoff` operational-truth tier — redb authoritative event store + RVF semantic-recall overlay + witness chain (`v1.rs`/`v2.rs`/`migrate.rs`). |
| `work-order/` | `work-order` | 74 | The `handoff.task.v1` envelope (`WorkOrder`), schemars single-source-of-truth for `schemas/task.schema.json`; `intake.rs` carries `Intent::classify` + `synthesize_spec`. |
| `handoff-core/` | `handoff-core` | 27 | Shared continuity primitives extracted from the `hf` monolith (`ledger_path`, `now_ns`, `tasks_dir`, `current_statuses`, `current_northstar_revision`). Deps: ledger, work-order, handoff-schema. |
| `handoff-schema/` | `handoff-schema` | 10 | `handoff.task.v1` JSON-Schema runtime validation (`validate_card`) + the `hf schema` verb (PRD §7.3/§23). Dep: work-order. |
| `handoff-policy/` | `handoff-policy` | 55 | Branch/remote/merge/loop policy engine (ADR-0001 §3). Leaf. |
| `handoff-fleet/` | `handoff-fleet` | 43 | `hf fleet status` aggregation (ADR-0004 §4). Deps: core, ledger, work-order. |
| `handoff-drift/` | `handoff-drift` | 33 | Drift-audit + policy-check engine (ADR-0019 D5). Deps: core, ledger, work-order. |
| `handoff-lease/` | `handoff-lease` | 28 | Weave lease bridge (WL-024): `hf claim` → mesh-coordinated claim. Leaf. |
| `handoff-gatekeeper/` | `handoff-gatekeeper` | 26 | AI gatekeeper foundation + GhPrView (HFTASK-0014). Deps: core, policy, route, ledger, secrets, test-support. |
| `handoff-hooks/` | `handoff-hooks` | 24 | Typed hook contract (HFTASK-0052, PRD §18). Dep: core. |
| `handoff-index/` | `handoff-index` | 22 | `hf index` repo nav maps + `hf plan` task DAG (PRD §8/§9). Deps: core, work-order. |
| `handoff-intake/` | `handoff-intake` | 17 | Front-door intake/dispatch verbs (HFTASK-0003). Deps: core, work-order. |
| `handoff-route/` | `handoff-route` | 13 | Ledger/tasks routing — two-ledger residency (ADR-0004 §3). Deps: core, fleet, test-support. |
| `handoff-secrets/` | `handoff-secrets` | 12 | envctl secrets-engine seam (HFTASK-0013, experimental, optional). Leaf. |
| `handoff-test-support/` | `handoff-test-support` | 1 | Shared test-only helpers (HFTASK-0029). Leaf. |

### RIDD-TOOLKIT group (5 crates — rusty-idd-* lineage; see §4 union verdict)

| Crate | pkg name | sym | Role |
|---|---|---|---|
| `crates/tui/` | `rusty-idd-tui` | 270 | OpenSpec TUI (`App`, `ui::draw`); vendors `vendor/syntect` (846 sym highlighter). |
| `crates/runner/` | `rusty-idd-runner` | 240 | Batch implementation runner + TUI config/data model. |
| `crates/spec/` | `rusty-idd-spec` | 205 | OpenSpec spec/delta model + parser + validate + merge (`SpecDoc`, `parse_spec`, `apply_delta`). |
| `crates/core/` | `rusty-idd-core` | 153 | IDD core: scanner, planner, manifest, templates, validation. |
| `crates/cli/` | `rusty-idd-cli` | 97 | `rusty-idd` CLI (`run`/`dispatch`, `spec status` etc.). 3rd binary. |

## 3. Cross-crate edge list (Cargo dependency DAG — ground truth)

```
work-order            → (none, leaf)
handoff-policy        → (none, leaf)        handoff-lease → (none)   handoff-secrets → (none)   handoff-test-support → (none)
handoff-schema        → work-order
ledger                → work-order
handoff-core          → ledger, work-order, handoff-schema
handoff-hooks         → handoff-core
handoff-index         → handoff-core, work-order
handoff-intake        → handoff-core, work-order
handoff-fleet         → handoff-core, ledger, work-order
handoff-drift         → handoff-core, ledger, work-order
handoff-route         → handoff-core, handoff-fleet, handoff-test-support
handoff-gatekeeper    → handoff-core, handoff-policy, handoff-route, ledger, handoff-secrets, handoff-test-support
hf                    → ALL 15 above  +  ruvector-verified, ruvector-domain-expansion, cognitum-gate-tilezero(opt)
ledger (features)     → rvf-crypto (redb-store) + rvf-runtime/rvf-index/rvf-types (v2, DEFAULT)
crates/cli            → crates/core, crates/runner, crates/spec
crates/tui            → crates/runner, crates/spec, vendor/syntect
```
Strict DAG — **no Cargo cycle**. The call-graph "cycles" in `graph/handoff.graph.md §4` are same-name resolver artifacts, not dep edges.

## 4. UNION VERDICT — `crates/{cli,core,runner,spec,tui}`: shared-lineage divergent forks

**Verdict: SHARED-LINEAGE, DIVERGENT FORKS (not distinct, not in-sync).**

Evidence:
- **Identical package names.** `crates/{cli,core,runner,spec,tui}/Cargo.toml` declare exactly `rusty-idd-{cli,core,runner,spec,tui}` — the same names the rusty-idd repo (`/home/drdave/Desktop/meta/rusty-idd/crates/*`) declares.
- **Identical module trees, divergent contents.** `diff -rq` across `src/` (handoff vs rusty-idd), divergence by line count:
  | crate | diff lines | handoff src lines | divergence |
  |---|---|---|---|
  | `crates/tui` | 7 | 8440 | ~0.1% (near-identical) |
  | `crates/spec` | 27 | 2274 | ~1.2% |
  | `crates/runner` | 483 | 4419 | ~11% |
  | `crates/core` | 942 | 2815 | ~33% |
  | `crates/cli` | 548 | 1366 | ~40% |
- **rusty-idd is the superset / more-developed side.** rusty-idd's workspace adds `config`, `knowledge`, `merge-tools`, `external/{codegraph-core,codegraph-parser,repomix-shared}` — none of which exist in handoff's `crates/`. handoff's `crates/` is an **older partial fork**.
- **Provenance.** These crates entered handoff via commit `ac5385f` ("IntentLock 5-field extension + typed hooks…"). ICM `decisions-rusty-idd` records this as a **"prior-poor-merge"** requiring per-crate reconciliation (slice 3.4), NOT absorption.

**Implication for the union:** the union does NOT need to merge these forks blindly. `tui`/`spec` are effectively the same code (cheap reconcile); `runner` moderate; `core`/`cli` substantially diverged and need a real per-crate diff/merge decision. handoff's copies are stale and should converge toward rusty-idd's (the more-developed side) or be deleted in favor of a single canonical set.

## 5. The LIVE union seams (where rusty-idd binds to handoff)

Today rusty-idd consumes handoff **BY-FILE only** — there is no live library/IPC dependency. The seams:

1. **`handoff.task.v1` schema (the work-order contract).** `work-order::WorkOrder` (handoff side) is the schemars source-of-truth that generates `schemas/task.schema.json` (`title: "WorkOrder"`). rusty-idd carries its **own copy** of `work-order` whose doc literally says *"mirrors `~/Downloads/tmp/handoff/handoff/schemas/task.schema.json`"* (`rusty-idd/crates/work-order/src/lib.rs:35`). → rusty-idd attaches via a **mirrored file copy**, not a dependency. **Live-seam upgrade:** rusty-idd depends on handoff's `work-order` + `handoff-schema` (`validate_card`) instead of mirroring.
2. **The `.handoff` ledger contract.** `schemas/{task,packet,session}.schema.json` + the `ledger` crate (redb store + witness chain). rusty-idd writes/reads `.handoff` artifacts as files; the live seam is calling `ledger`/`hf` for **witnessed** persistence instead of plain file IO.
3. **The `hf` CLI / `hf-mcp` MCP surface.** 27 verbs + the MCP server are the kernel's runtime interface. A live union has rusty-idd's loop drive `hf` (or link `handoff-intake`/`handoff-core`) rather than reimplement claim/checkpoint/handoff.
4. **`handoff-intake`/`work-order::intake`** (`Intent::classify`, `synthesize_spec`) — the front-door that turns raw text into a provable `WorkOrder`. This is the natural attach point for rusty-idd's spec/IDD flow.

**Standalone-ization blocker (must be planned):** `hf` + `ledger` pull RuVector via `../../RuVector/*` path deps (witness chain `rvf-crypto`, `v2` overlay `rvf-runtime/index/types`, `ruvector-verified`/`-domain-expansion`/`cognitum-gate-tilezero`). A union @ `$META_ROOT + handoff` is **not standalone** without a RuVector strategy (vendor / path-dep / publish) — see ICM `decisions-rusty-idd`.

## 6. Entrypoints + build/run surface

- **Binaries (3):** `hf` (`hf/src/main.rs::main`), `hf-mcp` (`hf/src/bin/hf-mcp.rs`), `rusty-idd-cli` (`crates/cli/src/main.rs::main`).
- **`hf` verbs (27 `cmd_*`):** init, resume, claim, checkpoint, handoff, drift, policy, status, fleet, schema, migrate, route, gate, gatekeeper, intake, lease, hook, index, session, sync, … (clap subcommands in `hf/src/main.rs`).
- **Build:** root `Cargo.toml` workspace, `resolver = 3`. `Makefile`, `scripts/`, `qodana.yaml`, `release-please`, `.github/workflows/`. Lint policy: `unsafe_code = "deny"` (one audited FFI exception: `ledger::v2::pid_is_alive`).
- **Contract files:** `schemas/{task,packet,session}.schema.json`. Live ledger: `.handoff/ledger.db` (redb).

## 7. Map gaps / honesty notes

- `git-kb code flows` returned empty — no traced execution flows to map (recorded, not fabricated).
- Dead-code (1258 own candidates) is heavily false-positive (clap string-dispatch of `hf cmd_*`); not a removal list — analyst triages per-symbol.
- The two-subsystem decoupling is inferred from Cargo deps (ground truth) + the near-zero RIDD→KERNEL call edge; the 41 KERNEL→RIDD call edges are same-name resolver collisions, not real coupling.
