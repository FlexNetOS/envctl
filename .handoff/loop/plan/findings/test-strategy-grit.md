# test-strategy — TARGET grit (cycle 5)

Dimension: `test-coverage` (always-on). Read-only analysis + an additive RED suite
authored in the RED worktree. grit is single crate `grit` v0.4.0, a binary-only
crate (no `src/lib.rs`, no `[[bin]]` override ⇒ bin name `grit`).

tests-ran: 3

(3 = the real count from `cargo test --test union_dedup_contract` in the RED
worktree; all 3 RED. Total pre-existing inline unit tests in the crate: 77.)

---

## Existing coverage map (by file, inline `#[cfg(test)]`)

Coverage here is reachability via the inline test modules; grit has **zero Rust
integration tests** (`tests/` held only shell scripts: benchmark.sh, harness.sh,
claude_agents_test.sh, gen_graph.py) before this suite.

- CLAIM: parser symbol/dep extraction is well-covered | evidence: 25 `#[test]` in src/parser/mod.rs:588-1470 exercise `scan_all` (src/parser/mod.rs:250) and `scan_with_deps` (src/parser/mod.rs:101) across 13 languages | confidence: high
- CLAIM: `Database` CRUD/queue/deps covered | evidence: 19 `#[test]` in src/db/mod.rs:462-826 cover upsert/list/search/queue/transitive-deps | confidence: high
- CLAIM: `SqliteLockStore` lock semantics covered incl. cross-process race | evidence: 15 `#[test]` in src/db/sqlite_store.rs:197-612, esp. `test_concurrent_access_separate_connections` (src/db/sqlite_store.rs:451) guarding the `BEGIN IMMEDIATE` atomicity (src/db/sqlite_store.rs:68) | confidence: high
- CLAIM: CLI helpers covered, command bodies almost entirely uncovered | evidence: 11 `#[test]` in src/cli/mod.rs:1548-1702 cover only `validate_identifier` (src/cli/mod.rs:262) + `is_entry_expired_local` (src/cli/mod.rs:252) + one `cmd_claim` path (src/cli/mod.rs:1631) | confidence: high
- CLAIM: config + git partially covered | evidence: 5 `#[test]` in src/config.rs, 2 `#[test]` in src/git/mod.rs | confidence: high

## Coverage gaps (contract-bearing symbols with NO test caller)

- CLAIM: cloud lock backends are entirely untested | evidence: 0 `#[test]` in src/db/s3_store.rs and src/db/azure_store.rs, yet both implement the `LockStore` contract (src/db/lock_store.rs:28) selected at runtime by `resolve_lock_store` (src/cli/mod.rs:388-412) | confidence: high
- CLAIM: `promote_queued` (FIFO queue→lock promotion) has no direct test | evidence: src/cli/mod.rs:681-728; it is a hotspot (called by both `cmd_release` src/cli/mod.rs:664 and `cmd_done` src/cli/mod.rs:963) and carries a deferred-promotion limitation note at src/cli/mod.rs:694 (hardcoded 600s TTL, no worktree for the promoted agent) | confidence: high
- CLAIM: the room notification server is untested | evidence: 0 `#[test]` in src/room/mod.rs; `NotificationServer::start` is wired into `cmd_init` (src/cli/mod.rs:448) and `Room::notify` fires on every claim/release/done | confidence: high
- CLAIM: end-to-end command flows are unverified | evidence: no integration test drives the `grit` binary through init→claim→release→done; only `cmd_claim`'s wait/backoff is unit-tested (src/cli/mod.rs:1631) | confidence: high
- CLAIM: error/fail-closed paths thin | evidence: `unknown_symbol_error` FK translation (src/db/sqlite_store.rs:12) and `ensure_initialized` bail (src/cli/mod.rs:369) have no asserting test | confidence: medium

## UNION-STEP-2 convergence gap (the cycle's contract)

- CLAIM: grit cannot reconcile two near-identical crates at symbol level | evidence: the `Command` enum (src/cli/mod.rs:31-175) has no reconcile/diff/converge variant; `cmd_init` indexes exactly ONE repo via `SymbolIndex::new(repo)` (src/cli/mod.rs:423) | confidence: high
- CLAIM: the two primitives union-step-2 needs already exist but are not composed across two sources | evidence: per-symbol source hash `Symbol.hash` (src/parser/mod.rs:15) computed at src/parser/mod.rs:329 via `hash_str` (src/parser/mod.rs:420) — identical source ⇒ identical hash, proven by `test_symbol_hash_deterministic` (src/parser/mod.rs:908); symbol-level locking `LockStore::try_lock` (src/db/lock_store.rs:29). The missing piece is a capability that points at TWO roots, partitions symbols by hash into identical(auto-merge)/divergent(conflict), and routes conflicts to the locker | confidence: high
- CLAIM: no library target ⇒ binary is the only public surface a `tests/` file can drive | evidence: no src/lib.rs; modules are private `mod` declarations in src/main.rs:1-6, so `use grit::…` is impossible from integration tests | confidence: high

## UPGRADE rows (designed tests)

- UPGRADE: add integration suite `tests/union_dedup_contract.rs` driving the binary for the union-step-2 reconcile contract | axis: accuracy | rationale: encodes the convergence capability (identical-vs-conflicting partition + conflict→lock handoff) that does not yet exist | evidence: src/cli/mod.rs:31-175 (no reconcile variant), src/parser/mod.rs:329 (hash primitive) | blast: guards the merge/lock substrate used by rust-port-merge | risk: low (authored, RED)
- UPGRADE: add integration tests for `promote_queued` FIFO promotion | axis: accuracy | rationale: hotspot with zero direct coverage | evidence: src/cli/mod.rs:681-728 | blast: queue fairness on release/done | risk: low
- UPGRADE: add fail-closed test for claiming an unindexed symbol | axis: accuracy | rationale: assert the actionable FK-translation message | evidence: src/db/sqlite_store.rs:12 | blast: error UX on stale registry | risk: low
- UPGRADE: add binary e2e for init→claim→status→release→done | axis: quality | rationale: no flow-level coverage of the command layer | evidence: src/cli/mod.rs:286-362 dispatch | blast: the whole CLI contract | risk: low
- UPGRADE: add backend-contract tests for S3/Azure `LockStore` (feature-gated / mocked) | axis: accuracy | rationale: two contract impls with zero tests | evidence: src/db/s3_store.rs, src/db/azure_store.rs | blast: cloud coordination correctness | risk: low

## traceability

| plan item / contract | acceptance criterion | test (path::name) | RED/GREEN |
|---|---|---|---|
| union-step-2: reconcile entry point exists | binary exposes a `reconcile` subcommand | tests/union_dedup_contract.rs::union_step2_reconcile_subcommand_is_supported | RED |
| union-step-2: partition identical vs conflicting | reconcile over 2 near-identical crates marks the 9 byte-identical symbols auto-mergeable and `checksum` a conflict | tests/union_dedup_contract.rs::union_step2_partitions_identical_and_conflicting_symbols | RED |
| union-step-2: conflict→lock handoff | reconcile emits divergent id `src/core.rs::checksum` as a lockable conflict target | tests/union_dedup_contract.rs::union_step2_divergent_symbol_is_flagged_for_lock | RED |

Each RED failure reason is identical and correct: `error: unrecognized subcommand
'reconcile'` (clap rejects the absent command). The tests COMPILE and RUN — the
failure is capability-absence, not a compile error. Command + counts:

- command: `cargo test --test union_dedup_contract` (run in /home/drdave/Desktop/meta/.worktrees/plan-grit-red/grit)
- expected RED failure reason: unrecognized subcommand `reconcile` ⇒ each test's `assert!(ok, …)` fires
- tests-ran count: 3 (`test result: FAILED. 0 passed; 3 failed`) — non-zero, satisfies the parse_tests_ran / no-fail-open rule
- owner wall: building grit standalone required a transient `[workspace]` table in Cargo.toml because a stray `/home/drdave/Desktop/meta/.worktrees/Cargo.toml` (members `loop_lib`, `meta_plugin_protocol`, neither present) hijacks it into a phantom workspace; the edit was reverted (`git checkout Cargo.toml`) so only the test file ships. Feature Forge must apply the same transient root, OR the stray manifest should be removed.

## FF test-build spec

Hand-off to Feature Forge — this is the "creation + implementation" path; the
strategist specified, Feature Forge writes + runs. GREEN means flipping the 3
RED tests by implementing the union-step-2 reconcile capability.

- Test surface (already authored, additive): tests/union_dedup_contract.rs (binary-driven via `CARGO_BIN_EXE_grit`). Keep additive; do not weaken assertions.
- Production work that makes them GREEN (engine-first, then thin CLI wiring):
  1. Engine: add a reconcile function that takes two repo roots, runs `SymbolIndex::scan_all` (src/parser/mod.rs:250) on each, joins symbols by `id`, and partitions by `Symbol.hash` (src/parser/mod.rs:15) into `identical` (equal hash ⇒ auto-mergeable) and `conflicting` (same id, differing hash) plus `only_in_a`/`only_in_b`.
  2. CLI: add a `Reconcile { a: String, b: String, lock_conflicts: bool }` variant to `Command` (src/cli/mod.rs:31-175) + dispatch arm in `run` (src/cli/mod.rs:303-361) + a `cmd_reconcile` that prints the partition (each symbol name; conflicts labelled with the word "conflict") and the divergent symbol ids in `<file>::<name>` form (e.g. `src/core.rs::checksum`).
  3. `--lock-conflicts`: for each conflicting symbol, route through `LockStore::try_lock` (src/db/lock_store.rs:29) so the divergent subset is claimed for resolution under symbol-level mutual exclusion.
- GREEN acceptance (maps 1:1 to the 3 RED tests):
  - `grit reconcile --help` exits 0.
  - `grit reconcile <A> <B>` exits 0; stdout contains `parse` and `dedupe` (auto-merge set) AND `checksum` together with the word `conflict`.
  - `grit reconcile --lock-conflicts <A> <B>` exits 0; stdout contains `core.rs::checksum`.
- Differential/golden fixtures to capture: the two near-identical crate fixtures are generated in-test (`write_crate`, 9 byte-identical helpers + 1 divergent `checksum`); promote them to a golden `tests/fixtures/union/{crate_a,crate_b}/src/core.rs` and snapshot the reconcile stdout for regression.
- Coverage target: the 3 union-step-2 cases plus the additional UPGRADE rows above (promote_queued, fail-closed FK message, init→done e2e, S3/Azure contract) — bring command-layer reachability from ~1 path to full dispatch coverage.
- CI gate(s) touched: `cargo test` (the new integration target compiles + runs under the normal test gate); fmt/clippy gates unaffected (test-only addition). Note the workspace owner wall above so CI builds grit standalone.

## verdict (1–3 lines for the verifier)

grit is unit-test-rich (77 inline tests on parser/db/lock-store) but had zero Rust
integration tests and no binary-level e2e; the union-step-2 reconcile capability is
entirely absent though both required primitives (`Symbol.hash`, `LockStore::try_lock`)
exist. Authored an additive 3-test RED suite (tests-ran: 3, all RED for the right
reason: unrecognized `reconcile` subcommand) that pins the convergence contract;
top remaining gaps are cloud backends, `promote_queued`, and command-flow e2e.
