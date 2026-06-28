# handoff — code graph + graph intelligence (cycle 2)

- **Target:** `handoff` (the continuity kernel — `hf` + the witnessed `.handoff` ledger), planned as the **UNION with rusty-idd**.
- **Snapshot:** `graph/handoff.symbols.json@f6abf962413bafe164d56fa26b70b0a5fdacb8a2` (worktree `plan/handoff-union-cycle2`, content on `origin/master` lineage).
- **Source:** `git-kb code` live store `.kb/.cache/gitkb.db` (branch `feat/hftask-0072-full-kb-adoption`), re-indexed `--force` this run. doctor: **symbol_count 2974, file_count 141, call_edge_count 7265**.
- **Caution applied:** the stale `.git/gitkb/code.db` (pre-peel `develop`, missing every `handoff-*` crate) was detected and **NOT used**. 846 of 2974 symbols are vendored `crates/tui/vendor/syntect`; **own code = 2128 symbols**.

## 1. Subsystem map — TWO programs in one workspace

```
                         handoff worktree (21 Cargo members)
   ┌───────────────────────────────── KERNEL (16 crates) ──────────────────────────────────┐
   │   hf  (604 sym, the hub binary)  ── depends-on ──▶ ALL 15 below + RuVector path deps    │
   │     ├─ ledger (164)  ── work-order                                                      │
   │     ├─ work-order (74)  [handoff.task.v1 envelope, schemars source-of-truth]            │
   │     ├─ handoff-core (27)  ── ledger, work-order, handoff-schema                         │
   │     ├─ handoff-schema (10) ── work-order        handoff-drift (33) ── core,ledger,wo    │
   │     ├─ handoff-fleet (43)  ── core,ledger,wo    handoff-route (13) ── core,fleet,ts     │
   │     ├─ handoff-gatekeeper (26) ── core,policy,route,ledger,secrets,ts                   │
   │     ├─ handoff-intake (17) ── core,wo           handoff-index (22) ── core,wo           │
   │     ├─ handoff-hooks (24) ── core               handoff-policy (55)  handoff-lease (28) │
   │     └─ handoff-secrets (12)  handoff-test-support (1)                                    │
   └────────────────────────────────────────────────────────────────────────────────────────┘
                          ▲  call edges between the two groups:
                          │   RIDD-TOOLKIT ─▶ KERNEL : 1     (essentially zero)
                          │   KERNEL ─▶ RIDD-TOOLKIT : 41    (same-name collisions, not real deps)
   ┌────────────────── RIDD-TOOLKIT (5 crates = rusty-idd-* lineage) ──────────────────────┐
   │   crates/cli=rusty-idd-cli (97)  ── core,runner,spec                                    │
   │   crates/core=rusty-idd-core (153)   crates/runner=rusty-idd-runner (240)               │
   │   crates/spec=rusty-idd-spec (205)   crates/tui=rusty-idd-tui (270) ── vendor/syntect   │
   │        └── vendor/syntect (846 sym, vendored highlighter — NOT a workspace member)      │
   └────────────────────────────────────────────────────────────────────────────────────────┘
```

**Finding:** there is **NO Cargo dependency** between any KERNEL crate and any `rusty-idd-*` crate. The two subsystems are effectively **separate programs sharing one workspace + `Cargo.lock`**. The `rusty-idd-*` crates here are a divergent fork co-resident from a prior poor merge (see codemap §Union verdict).

## 2. Hotspots / centrality (in-degree over resolved call edges, own code)

```
in   out  symbol                                   location                          group
157   1   SpecDoc.contains                         crates/spec/src/model/spec.rs:30   RIDD
143   4   TuiConfig.default                        crates/runner/src/config.rs:64     RIDD
136   3   App.new                                  crates/tui/src/app.rs:114          RIDD
112   0   write (test helper)                      crates/cli/tests/spec_status_cli.rs:34  RIDD(test)
109   1   McpServer.new                            hf/src/bin/hf-mcp.rs:114           KERNEL  ◀ top kernel hub
 78   0   BatchImplState.new                       crates/runner/src/runner.rs:54     RIDD
 74   3   Ledger.open                              ledger/src/v1.rs:415               KERNEL  ◀ kernel foundation
 66   1   temp_dir (test helper)                   crates/core/tests/smoke.rs:11      RIDD(test)
 27   1   ledger_path                              handoff-core/src/lib.rs:58         KERNEL
 20   2   apply_delta                              crates/spec/src/model/merge.rs:62  RIDD
 18   1   compute_intent_lock                      work-order/src/lib.rs:175          KERNEL  ◀ IntentLock
 17   3   current_statuses                         handoff-core/src/lib.rs:72         KERNEL
```

Centrality is dominated by the RIDD-TOOLKIT fork. The KERNEL's own hubs are `McpServer.new` (MCP surface), `Ledger.open` (ledger foundation), `ledger_path`/`current_statuses` (handoff-core primitives) and `compute_intent_lock` (no-downgrade IntentLock).

## 3. Blast-radius (transitive callers / file impact)

```
symbol / file                              transitive_callers   direct   note
ledger/src/v1.rs::Ledger.open                     120             74     ledger open = kernel foundation; widest blast
handoff-core/src/lib.rs::ledger_path               54             27
handoff-schema/src/lib.rs::validate_card           40              5     handoff.task.v1 FAIL-CLOSED validation gate
handoff-core/src/lib.rs::now_ns                    13              7
work-order/src/lib.rs::compute_intent_lock          —             18     high direct fan-in
file impact (git-kb impact --depth 3, total_count): handoff-core/lib.rs=116  work-order/lib.rs=111  ledger/v1.rs=153
```

Any change to `Ledger.open`, `ledger_path`, or `validate_card` touches the widest surface — these are the kernel's load-bearing contracts.

## 4. Cycles — Tarjan SCC (in-process, over the resolved edge list)

14 SCCs (all size 2–5). **No genuine architectural cycle found.**

```
~9  VENDOR  vendor/syntect parser recursion (parse_line_inner / exec_pattern / parse_* ) — expected for a parser
 5  OWN     same-name resolver COLLISIONS across parallel modules (NOT mutual recursion):
            • verify_witness_chain   ledger/v1.rs <-> ledger/v2.rs
            • acquire_store/open     ledger/v2.rs
            • file_is_legacy_sqlite/open + deserialize   ledger/v1.rs
            • strip_prefix           crates/spec/.../delta_parser.rs <-> spec_parser.rs
```

**Crate-level call cycles** reported (`hf<->handoff-core/-index/-lease/-fleet`, `crates/cli<->crates/core`, `crates/core<->ledger`) are the **same artifact**: identical symbol names (`new`/`open`/`default`/`contains`) the name-resolver attributes across crates.

> **GROUND TRUTH = the Cargo manifest dependency graph, which is a strict DAG with zero cycles.** `hf` depends on everything; nothing depends on `hf`. The call-graph "cycles" are a property of duplicate symbol names + parallel `v1`/`v2` ledger modules, not of the architecture.

## 5. Layering / boundaries

- **Cargo dep DAG:** clean. Leaves: `handoff-policy`, `handoff-lease`, `handoff-secrets`, `handoff-test-support`, `work-order`. Hub: `hf`.
- **Two-subsystem split** (see §1): KERNEL ⟂ RIDD-TOOLKIT (1 real-direction call edge).
- **RuVector coupling (standalone-ization blocker):** `hf` → `ruvector-verified`, `ruvector-domain-expansion`, `cognitum-gate-tilezero(opt)`; `ledger` → `rvf-crypto` (witness chain, `redb-store` feat) + `rvf-runtime`/`rvf-index`/`rvf-types` (`v2` default feat). All `../../RuVector/*` path deps — the kernel is **not standalone** without a RuVector strategy.

## 6. Dead-code (HIGH false-positive — analyst must triage)

1258 own symbols have caller_count 0 AND in-degree 0. This is **not** a confirmed-dead list:
```
hf 314   crates/tui 214   crates/runner 193   crates/spec 110   ledger 101   crates/core 74
work-order 44   handoff-policy 41   crates/cli 23   handoff-lease 21   ... (all handoff-* < 25)
```
`hf`'s 314 are dominated by `cmd_*` verb handlers **dispatched via clap string-match** (no resolved call edge) — false positives. The rest are public API, trait impls, serde derives, and test helpers. Triage with `git-kb code dead` + `query dead-code-explain` per symbol before any removal.

## 7. Entrypoints

- **Binaries (3):** `hf` (`hf/src/main.rs::main`, 27 `cmd_*` verbs incl. init/resume/claim/checkpoint/handoff/drift/policy/schema/fleet/route/intake/gatekeeper/lease/hook/index/session/migrate/gate), `hf-mcp` (`hf/src/bin/hf-mcp.rs`, MCP server), `rusty-idd-cli` (`crates/cli/src/main.rs`).
- git-kb inferred 100 entrypoints total (13 `main_function`, 87 `test_symbol_or_file`).
- **Flows:** `git-kb code flows` returned empty for this snapshot (none traced).
