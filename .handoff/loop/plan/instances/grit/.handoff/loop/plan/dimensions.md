# Dimension ledger (verifier-gated)

Legend: [ ] todo  [~] analysed-not-verified  [x] verified  [!] blocked

## grit

<!-- Seeded by plan-cartographer (cycle 5) from graph/grit.* — dependency-ordered. Do not reset existing [x]/[~]/[!] rows. -->
- [x] architecture — VERIFIED (verdicts.md#grit): line-level git merge (git/mod.rs:243), write-only hash (db/mod.rs:156-171), room dies w/ init (room/mod.rs:66-96), partial-claim leak (cli/mod.rs:612-616), 0 layering violations + 0 true cycles (open↔open = resolver artifact)
- [x] public-api-contracts — VERIFIED: `LockStore::try_lock` (lock_store.rs:29) is a LOCK contract, not a merge engine; union-fitness answered = UNFIT as-is (claim-1a/1b CONFIRMED)
- [x] data-flow — VERIFIED: claim→grant→bail leak path (cli/mod.rs:563-616), queue promotion 600s TTL (cli/mod.rs:693-700), AST scan→upsert→hash spine (parser:329→db:156); deps-precision over-link carried as accuracy UPGRADE (not gate-blocking)
- [~] correctness-concurrency — PARTIAL: sqlite `BEGIN IMMEDIATE` race + regression test CONFIRMED (sqlite_store.rs:68,451), partial-claim non-atomicity CONFIRMED; S3/Azure conditional-write atomicity NOT run (no cloud creds) — fail-closed
- [x] hotspots-coupling — VERIFIED: parser/ hottest (SymbolIndex.new in-deg 39), LockStore trait change spans all 3 backends (graph/grit.graph.md confirmed by spot-reads)
- [~] external-backends-network — PARTIAL: cloud read-skew + no-resync CONFIRMED (db:179-227 LEFT JOIN local locks; s3/azure keyspace .grit/locks/, zero local writes), SDK currency azure 0.21 CONFIRMED; retry/timeout config (s3_store.rs:52-60) + S3Config secret field NOT line-verified — fail-closed
- [x] dead-code — VERIFIED: `NameExtractor::ChildKind` declared (parser:90) + matched (:386) but NEVER constructed; `get_deps`/`count_deps` `#[allow(dead_code)]`; default_mode = serde false-positive (graph)
- [x] tooling-dependencies — VERIFIED: deps confirmed (azure_* 0.21 pre-GA, rusqlite 0.31 bundled=C, 14 tree-sitter C grammars, no MSRV); grit substrate is NOT no-C (trust-boundary feasibility note in verdicts)
- [~] governance-config — PARTIAL: gov-008 catch-all `_=>SQLite` (cli:407), gov-009 Azure plaintext key (azure_store.rs:29), gov-005 no-MSRV CONFIRMED; remaining gov-001..004/006/007/010/011 analysed-not-gate-verified — fail-closed
- [x] test-coverage — VERIFIED: RED suite ran 3, all RED for `unrecognized subcommand 'reconcile'` (capability absence, compiled+ran 0.01s); binary-only (no lib.rs) + 77 inline tests confirmed
