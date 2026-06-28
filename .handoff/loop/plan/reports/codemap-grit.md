# Codemap — `grit` (cycle 5)

- **Crate**: `grit` v0.4.0 (single Cargo binary crate), Apache-2.0. `Cargo.toml:1-7`.
- **One-liner (own claim, to verify)**: "Coordination layer for parallel AI agents on top of git". `Cargo.toml:5`, `src/cli/mod.rs:20`.
- **Snapshot**: source @ git `57b60842d71145c271b994bb7a8c33c3bca42dfe` (branch `master`, last commit 2026-06-21).
- **Scale**: 11 `src/*.rs` files, 305 indexed symbols, 548 intra-src call edges, 74 `pub` symbols. (`test-projects/` are fixtures, excluded.)
- **Convergence role**: this is the **symbol-level merge/lock substrate**. Its `LockStore` trait + `Symbol`/`Dep` AST index + git-worktree integration are the primitives that would power UNION STEP 2 — symbol-level dedup of the ~95%-shared `handoff`↔`rusty-idd` `crates/{cli,core,runner,spec,tui}` instead of a hand-merge.

## Module map (what each module IS)

```
src/main.rs       thin entrypoint: parse Cli (clap) -> cli::run         (7 symbols)
src/cli/mod.rs    command surface + ALL verb dispatch/handlers          (50 symbols)
src/config.rs     GritConfig: backend selection (local/s3/azure), .grit/config.json (12 symbols)
src/parser/mod.rs tree-sitter AST symbol index: Symbol, Dep, SymbolIndex (52 symbols)
src/db/mod.rs     Database (SQLite schema: symbols, deps, sessions, queue) (51 symbols)
src/db/lock_store.rs  LockStore TRAIT + LockEntry + LockResult (the abstraction) (4 symbols)
src/db/sqlite_store.rs SqliteLockStore  — local backend                  (30 symbols)
src/db/s3_store.rs     S3LockStore — S3/R2/GCS conditional-PUT backend   (32 symbols)
src/db/azure_store.rs  AzureLockStore — Azure Blob + Event Grid backend  (31 symbols)
src/git/mod.rs    GitRepo: worktree/branch/PR/merge git plumbing         (23 symbols)
src/room/mod.rs   Room + NotificationServer: Unix-socket event pub/sub   (13 symbols)
```

## Entry points

- **Process entry**: `src/main.rs::main` (`src/main.rs:12`) → calls only `cli::run` (`src/main.rs:14`). It loads config and dispatches.
- **CLI dispatch**: `src/cli/mod.rs::run` (`src/cli/mod.rs:286`) — the single fan-out hub. clap `Command` enum (`src/cli/mod.rs:32-175`) is matched here.
- **NOTE — fail-closed finding**: `git-kb code entrypoints` and `query entrypoints` both returned `[]` (zero inferred entrypoints). The real entrypoint (`main` → `cli::run`) is asserted from source, not from the tool. Recorded in metrics `index_health` + `notes`.

### CLI verbs (the external command surface) — `src/cli/mod.rs:32-249`
| Verb | Purpose | Source |
|---|---|---|
| `init` | initialize grit in repo (creates `.grit/`, scans symbols) | `:34` |
| `claim` | claim symbols before editing (TTL, mode read/write, `--wait`, `--queue`, `--with-deps`) | `:36-68` |
| `release` | release symbols held by agent | `:71-78` |
| `status` | show current lock status | `:81` |
| `symbols` | list indexed symbols (`--file` filter) | `:84-88` |
| `plan` | declare intent, get smart suggestions | `:91-99` |
| `done` | mark agent done, merge worktree, release all locks | `:102-106` |
| `watch` | stream room events (Unix socket) or poll S3 | `:109-113` |
| `worktree list` | list git worktrees | `:116-119`, `:178-181` |
| `queue list/cancel` | manage the FIFO lock queue | `:122-125`, `:206-217` |
| `gc` | garbage-collect expired locks | `:128` |
| `session start/status/pr/end` | feature-branch sessions for multi-agent work | `:131-134`, `:184-203` |
| `config set-s3/set-azure/set-local/show` | choose backend | `:137-140`, `:220-249` |
| `assign` | auto-pick + claim a free symbol from a file | `:143-163` |
| `heartbeat` | refresh an agent's lock TTL | `:166-174` |

## Public surface (74 `pub` symbols) — the merge/lock primitives

- **Lock abstraction** (`src/db/lock_store.rs`): `trait LockStore: Send + Sync` (`:28`) with `try_lock / release / release_all / all_locks / locks_for_agent / gc_expired_locks / refresh_ttl`; `struct LockEntry` (`:6`, fields: symbol_id, agent_id, intent, locked_at, ttl_seconds, mode); `enum LockResult { Granted, Blocked{by_agent,by_intent} }` (`:22`).
- **3 backend impls of `LockStore`**: `SqliteLockStore` (`src/db/sqlite_store.rs:47`), `S3LockStore` (`src/db/s3_store.rs:520`), `AzureLockStore` (`src/db/azure_store.rs:383`). Pluggable via `GritConfig`.
- **AST index** (`src/parser/mod.rs`): `struct Symbol` (`:8`), `struct Dep` (`:19`), `struct SymbolIndex` (`:25`) with `new` (`:94`), `scan_with_deps` (`:101`), `scan_all` (`:250`). Backed by 14 tree-sitter grammars (Rust/TS/JS/Py/Go/Java/C/C++/C#/Ruby/PHP/Swift/Kotlin — `Cargo.toml`).
- **Metadata store** (`src/db/mod.rs`): `struct Database` (`:39`) — SQLite over `symbols`, `deps`, `sessions`, `queue` tables; 23 query/mutation methods incl. `get_transitive_deps` (`:364`) — the dependency-aware-locking primitive.
- **Git plumbing** (`src/git/mod.rs`): `struct GitRepo` (`:6`) — `create_worktree` (`:21`), `merge_worktree` (`:133`), `create_session_branch` (`:365`), `push_and_create_pr` (`:410`), `list_worktrees` (`:476`).
- **Eventing** (`src/room/mod.rs`): `struct Room` + `notify` (`:35`), `struct NotificationServer` + `start` (`:68`), `RoomEvent`/`EventType`.
- **Config** (`src/config.rs`): `struct GritConfig` (`:9`), `load` (`:31`), `save` (`:51`).

## External interfaces (boundaries to the outside world)

- **Local SQLite** — `rusqlite` (bundled), `.grit/grit.db`. `src/db/mod.rs`, `src/db/sqlite_store.rs`.
- **S3 / R2 / GCS** — `aws-sdk-s3`/`aws-config`; **atomic lock acquisition via conditional PUT `If-None-Match: *`** (`src/db/s3_store.rs:15`, retry/timeout config `:52-60`). Network egress.
- **Azure Blob Storage** — `azure_storage_blobs` native API with conditional writes; real-time only with an **Event Grid** subscription configured (`src/db/azure_store.rs:14`, `BlobServiceClient` `:52`). Network egress.
- **Git** — shells out / libgit-style plumbing via `GitRepo` (worktrees, branches, PRs). `src/git/mod.rs`.
- **Unix domain socket** — local real-time event bus: `UnixListener::bind` / `UnixStream::connect` (`src/room/mod.rs:3,39,74`). `watch` verb consumes it.
- **tree-sitter** — 14 language grammars for the AST symbol scan. `src/parser/mod.rs`, `Cargo.toml`.
- **Async runtime** — `tokio` multi-thread, used only to drive the S3/Azure SDK calls from the sync CLI.

## Build / run surface

- Build: `cargo build --release` → single binary `grit`. Deps: clap, rusqlite(bundled), tree-sitter ×14, serde/serde_json, anyhow, chrono, tokio(rt-multi-thread), glob, colored, aws-config/aws-sdk-s3, azure_core/azure_storage/azure_storage_blobs, urlencoding. `Cargo.toml`.
- Run: `grit --repo <path> <verb> …` (default repo `.`, `src/cli/mod.rs:27-28`).
- State: `.grit/` per repo — `grit.db` (SQLite) + `config.json`; locks live in the configured backend (sqlite/s3/azure).
- Index health (this run): 511 total symbols / 93 files / 995 call edges / **3,094 unresolved calls** (63.9% `no_match`, 32.6% `skip_list` — almost all external-crate calls: rusqlite, aws-sdk, tree-sitter, std). **1 file skipped on index** (error) — recorded as a gap to investigate. Languages indexed: rust 385, typescript 106, python 20 (TS/Py are from `test-projects/` fixtures).

## Claims to verify (the project's own statements, not yet facts)
- "Atomic acquisition via conditional PUT (If-None-Match: *)" guarantees mutual exclusion across distributed agents on S3-compatible stores. `src/db/s3_store.rs:15`.
- Azure backend gives real-time notification "only if an Event Grid subscription is configured". `src/db/azure_store.rs:13-15`.
- `--with-deps` performs dependency-aware locking (locks all callees as read). `src/cli/mod.rs:62-64` + `Database.get_transitive_deps` `src/db/mod.rs:364`.
- Identifier validation prevents path traversal / argument injection. `src/cli/mod.rs:262-284` (called early in `run`, `:288-300`).
