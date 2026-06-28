# Verdicts — plan-verifier (the GATE)

Adversarial verification of analyst CLAIMs + feasibility-gating of UPGRADEs.
Verdict grammar: CONFIRMED | REFUTED (<counter>) | QUALIFIED (<cond>) | INCONCLUSIVE (<why>).
Only CONFIRMED/QUALIFIED + feasible rows flow to the architect.

## grit

Dated: 2026-06-27. TARGET = grit (cycle 5). Source verified at `/home/drdave/Desktop/meta/grit`;
build/test probes in the RED worktree `/home/drdave/Desktop/meta/.worktrees/plan-grit-red/grit`.

### Material claims (adversarial)

- VERDICT claim-1a (merge is line-level git, NOT symbol-level) -> CONFIRMED. `merge_worktree`
  shells `git merge --no-ff <branch>` after `git rebase` — src/git/mod.rs:221-224 (rebase),
  :243-253 (merge). No symbol-level reconciliation; "no conflicts" comes from partitioning, not
  reconciliation.
- VERDICT claim-1b (per-symbol content hash computed but NEVER read for merge/dedup) -> CONFIRMED.
  Hash computed at src/parser/mod.rs:329 via `hash_str` (:420-424, `DefaultHasher`/SipHash) and
  stored on `Symbol` (:357,363). In db it is ONLY written: INSERT/`ON CONFLICT DO UPDATE SET hash`
  at src/db/mod.rs:156-161,171; schema col :89. `grep` of src/db for `hash` shows zero `SELECT`/
  comparison of the column outside `#[cfg(test)]` fixtures (db/mod.rs:476,520,528 tests; :717,729 a
  fixture symbol literally named "hash"). No production read path. **HEADLINE CONFIRMED: grit is an
  advisory symbol-LOCK + git-worktree coordinator, not a symbol-level reconciliation engine.**
- VERDICT claim-2 (RED suite is RED-for-the-right-reason: capability absence, not compile error)
  -> CONFIRMED. Ran `cargo test --test union_dedup_contract` in the RED worktree (after a transient
  empty `[workspace]` table appended to grit's Cargo.toml to clear the phantom-workspace wall;
  reverted via `git checkout -- Cargo.toml Cargo.lock`, worktree clean after). Result:
  `test result: FAILED. 0 passed; 3 failed; 0 ignored; finished in 0.01s`. All three fail with
  `error: unrecognized subcommand 'reconcile'` (clap rejects the absent command). The tests
  COMPILE and RUN — failure is capability-absence. Binary-only confirmed: no src/lib.rs; src/main.rs
  declares private `mod cli/config/db/git/parser/room`, so a `tests/` file can only drive the binary.
  Primitives exist: `Symbol.hash` deterministic; `LockStore::try_lock` at db/lock_store.rs:29.
  Composition (a `Reconcile{a,b}` command) absent.
- VERDICT claim-3a (backend catch-all `_ => SQLite` silently downgrades a typo'd backend) ->
  CONFIRMED. src/cli/mod.rs:392 `match config.backend.as_str()` with :407-410 `_ => SqliteLockStore`.
  `backend: String` (config.rs:11) — `"azur"`/`"s33"` route to local locking with no error.
- VERDICT claim-3b (cloud access keys plaintext in .grit/config.json) -> CONFIRMED (Azure half).
  `pub access_key: String` with serde Serialize, no skip, at src/db/azure_store.rs:29; used at :51.
  S3 half not line-verified for an embedded secret field (s3_store.rs S3Config :636) -> that sub-part
  INCONCLUSIVE; Azure exposure stands.
- VERDICT claim-4a (cloud read/write skew: reads JOIN local `locks` table cloud never populates)
  -> CONFIRMED. `available_symbols_in_files` (db/mod.rs:179-205) and `list_symbols`
  (db/mod.rs:214-227) both `LEFT JOIN locks` (the local SQLite table). S3/Azure stores keep ALL lock
  state in cloud objects under prefix `.grit/locks/` (s3_store.rs:24; azure_store.rs:36) — `grep` of
  s3_store.rs + azure_store.rs shows NO `INSERT`/`Database`/`registry.db`/local-`locks` write and NO
  resync path back to the local table. The analyst's QUALIFIED-downgrade condition (a resync exists)
  is FALSE, so the claim is NOT downgraded: under s3/azure, `symbols`/`plan`/`assign` views report
  stale/empty lock state.
- VERDICT claim-4b (non-atomic multi-symbol claim leaks partial locks) -> CONFIRMED. Terminal path:
  granted locks are committed + notified (cli/mod.rs:563-575) then the command `bail!`s "Some symbols
  are blocked" (:612-616) WITHOUT releasing the granted subset; contrast the retry/wait path which
  DOES release before sleeping (:626-628). A 2-symbol claim with 1 blocked (no `--wait`/`--queue`)
  leaves the granted lock held by the "failed" agent.
- VERDICT claim-4c (room socket server dies with `grit init`; real-time watch never works) ->
  CONFIRMED. `NotificationServer::start` only `thread::spawn`s and returns ("runs until the process
  exits", room/mod.rs:66-96). Sole `.start()` caller is `cmd_init` (cli/mod.rs:448-449), which then
  prints and returns Ok (:451-456) -> main exits -> the background thread is killed; the socket file
  is left with no Drop cleanup. The other `Room::new` sites (cli/mod.rs:569,666,687,966,1261) are
  connect-send `notify` producers, not server starts. Functional eventing path is `--poll`.
- VERDICT claim-5 (0 true architectural cycles — the open↔open SCC is a resolver artifact; 0 layering
  violations) -> CONFIRMED. Spot-checked both bodies: `Database::open` (db/mod.rs:44) and
  `SqliteLockStore::open` (sqlite_store.rs:31) each call `rusqlite::Connection::open(path)`; neither
  calls the other — git-kb's ambiguous-name resolver mis-linked the bare `open`. graph/grit.graph.md
  records strictly-downward module edges (main→cli→{db,git,room,parser,config}, plus db→config), no
  back-edges -> layering violations 0.
- VERDICT claim-6a (deprecated pre-GA azure_* 0.21; rusqlite 0.31 stale; no MSRV) -> CONFIRMED
  (in-repo facts). Cargo.toml:58-60 `azure_core/azure_storage/azure_storage_blobs = "0.21"`;
  Cargo.toml:15 `rusqlite = "0.31"` (bundled); no `rust-version` key (MSRV absent).
- VERDICT claim-6b (azure_storage_blob 1.0.0 GA 2026-05-14; Rust >= 1.96.0 for Cargo
  CVE-2026-5223/5222) -> QUALIFIED (web-sourced; not independently re-verified by the GATE here).
  The actionable in-repo half (deps are old/pre-GA) is CONFIRMED above; the exact GA date and CVE IDs
  are trend-researcher web claims — treat as advisory currency input, verify the GA/CVE specifics at
  apply-time. (`research/grit.trends.md` carries the citations.)
- VERDICT claim-7 (stray /home/drdave/Desktop/meta/.worktrees/Cargo.toml phantom workspace hijacks
  grit's standalone build) -> CONFIRMED. File exists (196B): `[workspace] members =
  ["loop_lib","meta_plugin_protocol"]`. From grit's worktree dir, `cargo build` fails:
  "current package believes it's in a workspace when it's not ... workspace:
  /home/drdave/Desktop/meta/.worktrees/Cargo.toml". (The two members ARE present as symlinks in
  .worktrees/, so the manifest is a real but mis-scoped root that captures any nested crate; building
  grit requires an empty `[workspace]` in grit's Cargo.toml or removing/relocating the stray
  manifest.)

Counter-checks that did NOT refute (defensive): no cloud→local lock resync exists (claim-4a holds);
the cross-process try_lock race IS correctly closed by `BEGIN IMMEDIATE` (sqlite_store.rs:68) with a
separate-connections regression test — that analyst sub-claim CONFIRMED, not a defect.

### UPGRADE feasibility verdicts (buildable within grit's invariants?)

Invariant note (load-bearing): grit's own substrate is NOT no-C — `rusqlite` uses `bundled` SQLite (C)
and all 14 tree-sitter grammars are C. So:

- FEASIBILITY union-engine direction (build `grit reconcile` / symbol dedup over two roots; the
  test-strategy + architecture UPGRADE) -> **feasible WITHIN grit as additive Rust** (reuses
  `scan_all` + `Symbol.hash` + `LockStore::try_lock`; additive `Reconcile` clap variant + dispatch +
  `cmd_reconcile`; the 3 RED tests pin the contract). BUT **infeasible as the engine "inside the
  envctl no-C trust boundary"** — grit's C substrate (rusqlite-bundled + tree-sitter) violates the
  NON-NEGOTIABLE no-C trust boundary. Architect routing: build reconcile in grit as the
  coordination/dedup substrate OUTSIDE the trust boundary, or lift the pure-Rust dedup logic into a
  no-C component that uses grit only for coordination. The "make grit the in-boundary union engine
  as-is" framing is REFUTED on feasibility.
- FEASIBILITY stable version-independent symbol hash (replace `DefaultHasher`) -> feasible. Pure-Rust,
  additive, single function (parser/mod.rs:420-424); APPLY-tier. Serves axis:accuracy (DefaultHasher
  is not guaranteed stable across toolchains; any persisted dedup key needs a fixed algo).
- FEASIBILITY route lock-availability reads through the active LockStore / unify cloud locks ->
  feasible (additive read path); serves axis:accuracy (fixes the confirmed cloud read-skew). PROPOSE
  (touches central read helpers; blast medium-high).
- FEASIBILITY atomic multi-symbol claim (release granted on terminal bail) -> feasible, additive,
  single command; APPLY-tier; serves correctness. Mirrors the existing release at :626-628.
- FEASIBILITY disambiguate symbol ids (add kind/positional discriminator) -> feasible WITH a schema/
  contract migration (id is the system-wide PK across locks/deps/queue/CLI args); PROPOSE.
- FEASIBILITY parse-each-file-once + single tree glob -> feasible, additive, init-internal; serves
  axis:speed; differential-output acceptance is sound.
- FEASIBILITY call-edge scope resolution -> feasible (accuracy); reduces `--with-deps` over-locking.
- FEASIBILITY retire socket Room or run a real `grit serve` daemon -> feasible; quality.
- FEASIBILITY remove dead code (`NameExtractor::ChildKind` declared :90 + matched :386 but NEVER
  constructed; `get_deps`/`count_deps` `#[allow(dead_code)]`) -> feasible, APPLY; confirmed dead.
- FEASIBILITY governance upgrades (AGENTS.md hard-rules + `.claude/rules` destructive guard; trim RTK
  noise in CLAUDE.md; MSRV + rust-toolchain pin; clippy `--all-targets` + `cargo audit`/`deny`;
  `enum Backend` deny-unknown replacing the catch-all; 0600 config perms / keyref for secrets;
  parameterize release `rtk-ai`→`${{ github.repository }}`) -> all feasible (docs/CI/config-additive,
  no trust-boundary or no-C conflict). Each STRENGTHENS a gate; none weakens one.
- FEASIBILITY phantom-workspace remediation (remove/relocate stray .worktrees/Cargo.toml, or add an
  empty `[workspace]` to grit's manifest) -> feasible; prerequisite for any grit standalone build/CI.

### Tally (grit)

- Material claims: CONFIRMED 12 (1a,1b,2,3a,4a,4b,4c,5,6a,7 + the two defensive sub-claims) ·
  QUALIFIED 1 (6b web GA/CVE specifics) · partial INCONCLUSIVE 1 (3b S3-secret-field sub-part;
  Azure half CONFIRMED) · REFUTED 0.
- UPGRADES: feasible — all upgrade rows buildable additively within grit. INFEASIBLE 1 framing:
  "grit as-is = the in-boundary (no-C) union engine" (REFUTED on the no-C trust-boundary invariant);
  the union/dedup capability itself is feasible in grit as an out-of-boundary coordination substrate.
- Union-fitness verdict: **UNFIT as-is** for union step 2 — confirmed advisory symbol-LOCK +
  git-worktree (line-level) coordinator; the dedup hash exists but is never read; reconcile is absent
  (RED-proven). Usable as the parallel-agent coordination substrate AROUND a dedup engine that must
  be built (pure-Rust if it is to sit inside the no-C trust boundary).
