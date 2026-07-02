# Code graph + intelligence — `grit` (cycle 5)

- Snapshot: `graph/grit.symbols.json@57b60842d71145c271b994bb7a8c33c3bca42dfe` (branch `master`).
- Source: `git-kb code` v0.2.10 (`symbols` + per-symbol `callees`), scoped to `src/`.
- Nodes (symbols): **305** · Intra-src call edges: **548** · `pub` symbols: **74** · files: **11**.
- Built from call data (AST call graph), not grep. Cross-repo edges: see `## Cross-repo` (deferred — grit is a self-contained crate this cycle).

## Module call graph (ASCII) — layered DAG, no back-edges

```
                         ┌──────────────┐
                         │  main.rs     │  main()  → cli::run()
                         └──────┬───────┘
                                │ 1
                         ┌──────▼───────────────────────────────┐
                         │  cli/mod.rs  (dispatch hub)           │
                         │  run() ─match Command→ cmd_* handlers │
                         └──┬─────┬──────┬───────┬───────┬───────┘
              db x58 │     │     │ x23  │ x12   │ x10   │ x6
                     ▼     │     ▼      ▼       ▼       ▼
          ┌────────────┐  │ ┌────────┐┌──────┐┌────────┐┌─────────┐
          │  db/       │  │ │ git/   ││room/ ││parser/ ││ config  │
          │  Database  │  │ │GitRepo ││Room  ││Symbol  ││GritConfig│
          │  +LockStore│  │ │worktree││Notif ││Index   ││         │
          └─────┬──────┘  │ └────────┘└──────┘└────────┘└────▲────┘
                │ x1      └──────────────────────────────────┘ x6
                └────────────── db → config x1 ────────────────┘

  Module edge weights (caller→callee, intra-src):
    cli → db      58      cli → parser  10
    cli → git     23      cli → config   6
    cli → room    12      db  → config   1
                          main→ cli      1
```

Edge direction is strictly downward (`main → cli → {db, git, room, parser, config}`, plus `db → config`).
No edge points back up (no `db→cli`, `parser→cli`, `git→db`, …). **Layering violations: 0.**

## LockStore polymorphism (the convergence-relevant core)

```
            trait LockStore (db/lock_store.rs:28)
            try_lock · release · release_all · all_locks
            locks_for_agent · gc_expired_locks · refresh_ttl
                 ▲              ▲              ▲
       impl ─────┘     impl ────┘     impl ───┘
   SqliteLockStore   S3LockStore     AzureLockStore
   (sqlite_store.rs  (s3_store.rs    (azure_store.rs
    :47, local DB)    :520, cond.     :383, Blob +
                      PUT If-None-     Event Grid)
                      Match:*)
   selected at runtime by GritConfig → cli::resolve_lock_store (cli/mod.rs, in-deg 12)
```

## Centrality / hotspots

**Top in-degree (most depended-on within src) — the load-bearing symbols:**

| in-deg | symbol | file | role |
|---|---|---|---|
| 39 | `SymbolIndex.new` | parser/mod.rs:94 | AST index constructor — used everywhere symbols are scanned |
| 25 | `write_file` (helper) | parser/mod.rs | shared file I/O helper |
| 23 | `scan_all` | parser/mod.rs:250 | full-repo symbol scan |
| 19 | `ensure_initialized` | cli/mod.rs:369 | guard every verb calls before touching `.grit/` |
| 19 | `SqliteLockStore.try_lock` | sqlite_store.rs:48 | the lock primitive |
| 18 | `Database.open` | db/mod.rs:44 | SQLite handle |
| 18 | `find_sym` | parser/mod.rs:605 | symbol lookup helper |
| 17 | `setup_db` (test helper) | db/mod.rs | test fixture |
| 14 | `SqliteLockStore.setup` | sqlite_store.rs | schema setup |
| 13 | `all_locks` | sqlite_store.rs | lock enumeration |
| 12 | `resolve_lock_store` | cli/mod.rs | backend factory (chooses sqlite/s3/azure) |
| 11 | `Database.upsert_symbols` | db/mod.rs | persist scanned symbols |

**Top out-degree (the orchestrators / fan-out):** `run` (23) → `cmd_claim` (14) → `cmd_done` (13) → `cmd_assign` (10) → `cmd_release`/`cmd_init`/`cmd_session_pr` (9). All in `cli/mod.rs` — confirming `cli` is the single coordination layer; the lower modules are leaves of the call tree.

**`git-kb query hotspots` (caller_count, full index incl. fixtures & multiple call-sites):** `SymbolIndex.new` 81 · `find_sym` 68 · `Symbol.new` 53 · `try_lock` 36 · `write_file` 29 · `scan_all` 24 · `make_symbol` 22 · `Database.open` 19. → **`parser/` is the hottest module**; any change to `SymbolIndex`/`find_sym`/`Symbol` has the widest blast radius.

## Blast-radius (impact)

- `parser/mod.rs` — highest. `SymbolIndex.new` (in-deg 39) + `find_sym` + `scan_all` are reached from `init`, `symbols`, `assign`, `plan`, `claim --with-deps`. A signature change here ripples across most CLI verbs.
- `db/lock_store.rs` (the `LockStore` trait, 7 methods) — changing the trait forces edits to **all 3** backend impls (sqlite/s3/azure) simultaneously. High, fan-wide.
- `db/mod.rs::Database` — 23 methods over the SQLite schema; the metadata backbone for symbols/deps/sessions/queue.
- `config.rs::GritConfig` — small in-degree but it gates backend selection; a schema change touches `resolve_lock_store` + all 4 `config` verbs.

## Cycles (Tarjan SCC over the 548-edge list)

- Multi-node SCCs found: **1**, but it is a **RESOLVER ARTIFACT, not a real cycle**: `Database.open ↔ SqliteLockStore.open`. Both methods call `rusqlite::Connection::open(path)` (`db/mod.rs:45`, `sqlite_store.rs:32`); git-kb's ambiguous-name resolver (101 `ambiguous` calls reported) mis-linked the bare name `open` to each other.
- **True architectural cycles: 0.** Self-recursive symbols: 0.
- Verdict: the call graph is a clean DAG. (Verified by reading both `open` bodies — neither calls the other.)

## Dead code

- Symbols with **zero internal graph callers: 167 / 305**. This is NOT 167 pieces of literal dead code — it is dominated by:
  - `LockStore`/trait methods invoked via `dyn` dispatch (no static call edge),
  - CLI command-handler arms reached through clap `match` in `run` (dispatch, not a call edge),
  - the 74-symbol `pub` API surface,
  - serde helper `default_mode` (db/lock_store.rs:16) — used via `#[serde(default = "default_mode")]` (db/lock_store.rs:12), a known false positive.
- `git-kb query dead-code-explain` top src hit: `default_mode` (`reason: NoCallers`) — confirmed false positive above. Remaining entries are mostly `test-projects/` fixtures.
- **Action for analyst**: triage the 167 against dyn-dispatch + clap-dispatch + pub-API to isolate any *genuinely* unreferenced private helper.

## Public API surface (74 `pub`)

Headline contract symbols (full list in `grit.symbols.json`):
- `LockStore` trait + `LockEntry` + `LockResult` (db/lock_store.rs) — the merge/lock contract.
- `SqliteLockStore` / `S3LockStore` / `S3Config` / `AzureLockStore` / `AzureConfig` — backends + their config.
- `Symbol`, `Dep`, `SymbolIndex` (parser) — the AST symbol model.
- `Database` + 23 methods (db/mod.rs) — metadata persistence.
- `GitRepo` + 10 methods (git) — worktree/branch/PR plumbing.
- `Room`, `NotificationServer`, `RoomEvent`, `EventType` (room) — eventing.
- `GritConfig` (config), `Cli`/`Command`/`*Action` enums (cli).

## Index-health caveats (fail-closed findings)

- `git-kb code entrypoints` AND `query entrypoints` → **`[]`** (zero inferred). Real entrypoint `main → cli::run` asserted from source.
- **1 file skipped on index** (error, unnamed by the tool) — gap; analyst should confirm which `src/` file (if any) is under-indexed.
- 3,094 unresolved calls (63.9% `no_match`, 32.6% `skip_list`) — overwhelmingly external-crate calls (rusqlite, aws-sdk, azure, tree-sitter, std); the intra-src graph (548 edges) resolved with 0 unresolved endpoints after dropping externals.

## Cross-repo

Deferred this cycle: `grit` is a single self-contained binary crate with no intra-meta source dependencies (its `Cargo.toml` deps are all crates.io). Cross-repo edges become relevant only when grit is *embedded* as the merge engine into the handoff/rusty-idd union — that mapping belongs to the union-step plan, not to grit's own graph.
