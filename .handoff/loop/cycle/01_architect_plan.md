# TASK-0030 — F6 bounded DPoP `jti` replay store  ·  VERDICT: GO

Engine-first, single-repo (envctl). Spec written first as `docs/secrets/OI-SM-1-jti-replay-store.md`
(persisted alongside this plan). The F2 edge listener that CALLS the store is TASK-0031 — OUT OF SCOPE.

## Target repos
1 repo: envctl. All changes in `crates/secrets-engine` (new module + 2 one-line wiring edits + tests)
+ the OI-SM-1 design doc. No cli/gui/secretd/proto/libsql changes this cycle. → sequential single-crew.

## Placement
- Crate: `crates/secrets-engine` (the security-policy authority — beside `broker/decide.rs`/`gate.rs`,
  one shared authority, pure unit-testable, no socket/TLS/tokio). NOT secretd (would couple policy to I/O).
- New module: `crates/secrets-engine/src/broker/jti.rs`; register `pub mod jti;` in `broker/mod.rs:5`.
- Re-export `JtiReplayStore`/`JtiReject` from `lib.rs` (~line 40 broker re-export block) so TASK-0031
  can `use envctl_secrets_engine::{JtiReplayStore, JtiReject};`.
- No CLI/GUI delta (daemon-internal; parity vacuous — `decide`/`gate` set the no-verb precedent).

## Engine API delta (new pure type in broker/jti.rs)
```rust
pub struct JtiReplayStore {
    accept_past_ms: i64,   // default 300_000 (ACCEPT_PAST_MS)
    accept_future_ms: i64, // default 30_000  (ACCEPT_FUTURE_MS)
    sweep_slack_ms: i64,   // default 30_000  (SWEEP_SLACK_MS)
    max_entries: usize,    // default 16_384  (MAX_ENTRIES)
    seen: std::collections::HashMap<String, i64>, // key = "client_id\u{0}jti", val = expiry_ms
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JtiReject { Replayed, ClockDriftPast, ClockDriftFuture, StoreFull }
impl JtiReplayStore {
    pub fn new() -> Self;
    pub fn with_params(accept_past_ms: i64, accept_future_ms: i64, max_entries: usize) -> Self;
    /// Atomic check-and-insert; wall-ms i64 (engine convention, like gate.rs/decide). &mut self =>
    /// caller owns the single lock (§7 atomicity). Sweeps expired entries every call.
    pub fn check_and_record(&mut self, client_id: &str, jti: &str, iat_ms: i64, now_ms: i64)
        -> Result<(), JtiReject>;
    #[cfg(test)] fn len(&self) -> usize;
}
impl Default for JtiReplayStore { fn default() -> Self { Self::new() } }
```
- Wall-ms i64 throughout (consistent with gate.rs:17-22, decide's now_ms). Store takes `now_ms` from
  the caller — pure function of inputs (deterministic, no `Clock` dep).
- `&mut self`: caller (edge) holds the single `Mutex<JtiReplayStore>` (TASK-0031). Keeps the engine
  type interior-mutability-free; matches broker `RwLock` precedent at the daemon layer.
- Non-printing: returns `Err(JtiReject)`; caller maps all four to a 401 at the edge (proof failures
  401 at the edge per decide.rs:39-42 RemoteNoDPoP note). No println!.
- `check_and_record` order: drift gate → sweep (`seen.retain(|_,&mut exp| exp > now_ms)`) → dedup
  (`contains_key` → Err(Replayed)) → cap (`len() >= MAX_ENTRIES` → Err(StoreFull)) → insert → Ok(()).
- Key encoding: `format!("{client_id}\u{0}{jti}")` (NUL separator — can't appear in either field).

## Wiring to the broker decision path (decide() UNCHANGED)
The jti check is a separate MUTATING pre-`decide` step (decide() is pure/non-mutating, decide.rs:122-144).
Edge call order (TASK-0031): TLS terminate + verify proof sig/htm/htu → `check_and_record(...)` (Err→401,
stop, never reaches decide) → on Ok build `RemotePeer{dpop_verified:true}` (decide.rs:60-67) → decide()
(lib.rs:1533). The store sits immediately before `dpop_verified=true` — the seam decide.rs:39-42 anticipates.
Readiness: type is pub/sync/dep-free, takes only client_id/jti/iat/now (all available at the edge). No
further engine change when the edge lands. TASK-0031 NOT built here.

## Invariants (each checkable)
- no-C: only std::collections::HashMap; ZERO new deps; nothing pulls SQLite/OpenSSL/aws-lc. Run no-c.sh.
- one rustls ring-only: no TLS/crypto crate touched.
- engine = single sync pure-Rust non-printing lib: sync, no println!/clap/UI; typed Result; in secrets-engine.
- CLI/GUI parity: vacuous (no user surface; decide/gate precedent).
- fail-closed: every uncertain outcome (Replayed/drift/StoreFull) REJECTS; no accept-on-error path.
  StoreFull rejects the NEW proof (never evicts a live entry → no replay hole). Unit-tested.
- no secret bytes: stores jti + expiry int only; caller logs DenyReason + client_id only.
- bounded memory / DoS cap: MAX_ENTRIES=16384 ≈1 MiB; sweep + fail-closed cap; documented + tested.

## Lock/manifest sync
None. No new dep (Cargo.lock unchanged → no-c unaffected), no manifest component, no lock change.

## Sequencing (leaf-first, single-crew sequential)
Write jti.rs + tests fully → `pub mod jti;` in broker/mod.rs:5 → re-export in lib.rs → verify:
`cargo test -p envctl-secrets-engine jti`, `cargo fmt --all`, `cargo clippy --workspace -- -D warnings`,
`bash ci/gates/no-c.sh` (must stay green), `bash ci/gates/shape.sh`.

## Tests (inline #[cfg(test)] in jti.rs; sync, fixed now/iat — no real clock)
1. first_use_accepted (Ok, len==1). 2. replay_rejected (2nd → Err(Replayed)). 3. different_clients_
same_jti_both_accepted (per-client scoping). 4. expired_then_fresh_same_value_accepted (sweep evicted;
+ stale iat → ClockDriftPast). 5. clock_drift_past_rejected (+ inclusive boundary Ok). 6. clock_drift_
future_rejected (+ boundary Ok). 7. capacity_cap_fail_closed (N+1 → StoreFull; a prior live jti still
Err(Replayed) after — proves no live-eviction hole). 8. sweep_reclaims_then_admits (cap not a permanent
wall). 9. concurrent_check_and_insert_single_winner (Arc<Mutex>, N threads same jti → exactly one Ok).

## Risks
Lock-poisoning at daemon (TASK-0031): edge must map poisoned lock → reject; documented in spec §5 +
re-export doc-comment, not enforceable in this pure type. Param tuning (300s/16384 evidence-based;
with_params keeps retune a 1-liner). `&mut self` vs `&self` (chose &mut; concurrency test proves the
Mutex-ownership contract). rtk corrupts fmt/clippy — implementer uses dedicated tools / `rtk proxy` raw;
cwd resets between agent bash calls → absolute paths.

## Open questions
None — OI-SM-1 jti-store items resolved with concrete defaults; nonce lifecycle + remote_clients schema
are separate OI-SM-1 sub-items (TASK-0031/F15); store designed to plug in.
