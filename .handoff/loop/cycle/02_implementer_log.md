# Implementation log: TASK-0030 — F6 bounded DPoP `jti` replay store

STATUS: GREEN

## Changes
- `crates/secrets-engine/src/broker/jti.rs` (NEW): the bounded, in-memory, per-process DPoP `jti`
  replay-dedup store — `JtiReplayStore` + `JtiReject`, consts, `check_and_record`, 9 inline tests.
- `crates/secrets-engine/src/broker/mod.rs`: `pub mod jti;` (next to `pub mod gate;`) +
  `pub use jti::{JtiReject, JtiReplayStore};` (broker re-export block, matching the `pub use decide::…` style).
- `crates/secrets-engine/src/lib.rs`: added `JtiReject, JtiReplayStore` to the `pub use broker::{…}`
  re-export block so TASK-0031 can `use envctl_secrets_engine::{JtiReplayStore, JtiReject};`.
- (`.handoff/loop/cycle/01_architect_plan.md` + `docs/secrets/OI-SM-1-jti-replay-store.md` are the
  architect's artifacts, untouched by me.)

## Engine API (new pure type — the parity contract; no CLI/GUI surface, parity vacuous like decide/gate)
- `pub enum JtiReject { Replayed, ClockDriftPast, ClockDriftFuture, StoreFull }` — `#[derive(Clone, Copy, Debug, PartialEq, Eq)]`.
- `pub struct JtiReplayStore { accept_past_ms: i64, accept_future_ms: i64, sweep_slack_ms: i64, max_entries: usize, seen: HashMap<String, i64> }`.
- `pub fn new()`, `pub fn with_params(accept_past_ms, accept_future_ms, max_entries)` (keeps default `SWEEP_SLACK_MS`), `impl Default`.
- `pub fn check_and_record(&mut self, client_id: &str, jti: &str, iat_ms: i64, now_ms: i64) -> Result<(), JtiReject>`
  — exact order: drift gate (inclusive boundaries) → sweep `retain(exp > now)` → dedup → cap `len() >= max_entries` → insert `iat + accept_past + sweep_slack`.
- Key = `format!("{client_id}\u{0}{jti}")` (NUL separator). `#[cfg(test)] fn len()`.
- Consts: `ACCEPT_PAST_MS=300_000`, `ACCEPT_FUTURE_MS=30_000`, `SWEEP_SLACK_MS=30_000`, `MAX_ENTRIES=16_384`.
- Sync, non-printing, `std::collections` only — zero new deps.

## Tests added (9, inline `#[cfg(test)] mod tests` in jti.rs; fixed now/iat ints, no real clock)
1. `first_use_accepted` — fresh proof Ok, len==1.
2. `replay_rejected` — 2nd identical → Err(Replayed); len stays 1.
3. `different_clients_same_jti_both_accepted` — per-client scoping; both Ok, len==2.
4. `expired_then_fresh_same_value_accepted` — sweep evicts stale, same jti re-admitted with fresh iat;
   AND a proof reusing the OLD iat → Err(ClockDriftPast) (proves the stale value can't actually replay).
5. `clock_drift_past_rejected` — too-old → Err(ClockDriftPast), not recorded; inclusive boundary `iat == now - ACCEPT_PAST` → Ok.
6. `clock_drift_future_rejected` — too-new → Err(ClockDriftFuture), not recorded; inclusive boundary `iat == now + ACCEPT_FUTURE` → Ok.
7. `capacity_cap_fail_closed` — fill to cap via `with_params`, N+1 unexpired → Err(StoreFull) (no growth/eviction),
   AND a prior live jti still → Err(Replayed) afterward (proves NO live-eviction replay hole).
8. `sweep_reclaims_then_admits` — full at NOW, then post-expiry sweep frees room → fresh proof Ok (cap not a permanent wall).
9. `concurrent_check_and_insert_single_winner` — `Arc<Mutex<JtiReplayStore>>`, 32 threads same (client,jti,iat):
   asserts EXACTLY one Ok and 31 Err(Replayed), final len==1.

## Build/test status (commands run from worktree root, raw `rtk proxy` to preserve exit codes/diagnostics)
- `rtk proxy cargo fmt --all` → exit 0 (reformatted only the new file's test asserts).
- `rtk proxy cargo clippy --workspace -- -D warnings` → exit 0 (clean).
- `rtk proxy cargo test -p envctl-secrets-engine` → exit 0; all 9 jti tests PASS; whole crate
  105 unit + 4 + 6 + 17 + 15 + 0 integration tests all PASS, 0 failed / 0 ignored.
- `bash ci/gates/no-c.sh` → exit 0: "NO-C GATE PASS" (rustls=0.23.40 on ring=0.17.14; zero aws-lc/openssl/C-SQLite).
- `bash ci/gates/shape.sh` → exit 0: "SHAPE GATE PASS".

## Deviations
None. Implemented exactly per the plan and OI-SM-1 spec.

## Follow-ups
- TASK-0031 (F2 edge listener) is the caller — OUT OF SCOPE here. It must:
  (a) own the single `Mutex<JtiReplayStore>` (the type is `&mut self`, not interior-mutable);
  (b) map a POISONED lock → reject (fail-closed) — not enforceable in this pure type (spec §5, noted in module doc);
  (c) call `check_and_record` once per proof immediately before setting `dpop_verified=true`, mapping every `JtiReject` to a 401.
- `with_params` keeps a retune to a one-liner if the audited defaults need tuning.

## Handoff notes (for the invariant-guardian)
- Fail-closed cap is the load-bearing security property: verify `capacity_cap_fail_closed` proves the
  N+1 proof is REFUSED (StoreFull) AND a prior live jti is still Replayed afterward — i.e. NO live entry
  was evicted to admit the new one (no replay hole). This is the explicit OI-SM-1 §4 fail-closed choice.
- Drift boundaries are INCLUSIVE (`iat == now - ACCEPT_PAST` and `iat == now + ACCEPT_FUTURE` are Ok) —
  covered by the boundary asserts in tests 5 & 6.
- No-C / zero-new-deps: `jti.rs` imports only `std::collections::HashMap` (+ `std::sync`/`std::thread` in
  `#[cfg(test)]`). Cargo.lock unchanged; no-c gate re-run green. secrets-engine still never links libSQL.
- No secret bytes stored: the map holds only `(client_id\u{0}jti)` keys + an `i64` expiry. No proof body,
  signature, bearer, or key material.
- Non-printing/sync: no `println!`/`eprintln!`, no async, no clap, no UI — typed `Result<(), JtiReject>`.
- Re-export reachability: `envctl_secrets_engine::{JtiReplayStore, JtiReject}` resolves (added to both
  `broker/mod.rs` and `lib.rs` re-export blocks).
- Clippy was run with default `--workspace` (no `--all-targets`), per the plan, and is clean. My change
  adds no `--all-targets`-only surface. NOTE the known PRE-EXISTING baseline lint (unrelated to this
  cycle): `cargo clippy -p envctl-gui --all-targets` trips `doc_list_item_without_indentation` at
  `crates/gui/src/main.rs` under the floating `stable` toolchain (documented in the prior TASK-0020 log) —
  a gui-doc-comment lint in an untouched crate, not a TASK-0030 regression.
