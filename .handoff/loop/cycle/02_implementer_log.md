# Implementation log: TASK-0032 (F5, P0) — streaming-revocation tear-down (FS-S5)

Status: **GREEN**

## Changes
- `crates/secrets-engine/src/broker/mod.rs`: added `Broker::peek(&self, token_id, now_ms) -> (u64,u64,u32)` — read-only counterpart to `bump` (no mutation, no row insert); + 3 unit tests (counters unchanged across N peeks; window-roll reports 0 without resetting the row; missing row reads all-zero).
- `crates/secrets-engine/src/lib.rs`: factored the bearer-verify + policy-load + gate-snapshot + decide prelude of `relay_swap_prepare` into private `authorize_relay(&self, bearer, req, bump: bool) -> anyhow::Result<Authz>` (key fetched only when `bump`). `relay_swap_prepare` is now a thin wrapper over it (audits the deny + builds `AllowPrepared` exactly as before — byte-for-byte). Added `pub fn relay_stream_authorized(&self, bearer, req, sink) -> StreamAuthz` (calls `authorize_relay(bump=false)`, `bytes_out=0`, same captured `RemotePeer`), the private `enum Authz`, and `pub enum StreamAuthz { Authorized, TearDown(DenyReason) }`. `DenyReason` was already re-exported at the crate root.
- `crates/secrets-engine/src/event.rs`: added metadata-only `SecretEvent::RelayStreamTornDown { relay, token_id, reason }`.
- `crates/secretd/src/edge/stream.rs` (NEW, under the crate-level `#[cfg(feature="relay-edge")]`): `relay_stream_response(...)` wraps the upstream chunk `BodyRx` in a supervised forwarding task (`tokio::select!` over {next upstream chunk, `interval(RECHECK_INTERVAL=2s)` tick, `MAX_STREAM_SECS=300s` deadline}); on tick → `engine.relay_stream_authorized`; on `TearDown`/deadline → drop the downstream sender (StreamBody ends cleanly) + emit `RelayStreamTornDown`. `Timing` (production + test override), `StreamAudit`, `recheck_egress_req` helpers. Keeps the bounded `BODY_CHANNEL_CAP` downstream channel (backpressure preserved).
- `crates/secretd/src/edge/mod.rs`: `pub mod stream;`; added `recheck_timing: Option<stream::Timing>` to `EdgeConfig` and threaded it through `serve_edge`.
- `crates/secretd/src/edge/listener.rs`: `handle_edge_request` now routes the Allowed body through `stream::relay_stream_response` via the new `swap_and_respond_streaming` (threads the captured `EgressReq`/bearer/`RemotePeer`); `ConnState`/`serve_connection`/`serve_edge_listener` carry the `Timing`.
- `crates/secretd/src/proxy.rs`: refactored `swap_and_respond` to delegate to generic `swap_and_respond_streaming<B,W: FnOnce(BodyRx)->BodyRx>` (local proxy = identity wrap; edge = the supervisor). Exposed `BodyChunk`/`BodyRx` type aliases, made `BODY_CHANNEL_CAP` `pub(crate)`, added test-only `__test_take_body_tx()`.
- `crates/secretd/src/conv.rs`: `RelayStreamTornDown` joins the no-proto-twin drop set (like `RelayRevoked`) — CLI+GUI consume it identically, zero proto churn.
- `crates/secretd/src/main.rs`: daemon `EdgeConfig` passes `recheck_timing: None` (production default).
- `crates/secrets-engine/tests/relay.rs`: 4 engine unit tests for `relay_stream_authorized`.
- `crates/secretd/tests/edge_e2e.rs`: `EdgeConfig` updated with `recheck_timing: None`.
- `crates/secretd/tests/edge_stream_e2e.rs` (NEW): the 4-case streaming e2e.

## Engine API delta (as implemented)
```rust
// crates/secrets-engine/src/broker/mod.rs
pub fn peek(&self, token_id: &str, now_ms: i64) -> (u64, u64, u32);   // read-only; no mutation
// crates/secrets-engine/src/lib.rs (crate root)
pub enum StreamAuthz { Authorized, TearDown(DenyReason) }             // DenyReason re-exported already
pub fn relay_stream_authorized(&self, bearer: &str, req: &EgressReq, sink: &EventSink) -> StreamAuthz;
fn authorize_relay(&self, bearer: &str, req: &EgressReq, bump: bool) -> anyhow::Result<Authz>; // private
// crates/secrets-engine/src/event.rs
SecretEvent::RelayStreamTornDown { relay: String, token_id: String, reason: String }
```

## Tests added
- `peek_does_not_mutate_counters_across_n_calls` — 50 peeks leave the row byte-identical; the next `bump` advances from the real prior state (peek consumed no rate/budget). Guards the plan's #1 risk (peek mis-impl).
- `peek_reports_zero_window_after_roll_without_resetting_row`, `peek_missing_row_reads_all_zero`.
- `relay_stream_authorized_allows_live_remote_and_consumes_no_budget` — 200 re-checks of a live remote bearer stay `Authorized` under a rate-limited policy (proves peek-not-bump) and never fetch/emit the key.
- `relay_stream_authorized_tears_down_on_bearer_revoke` — `TearDown(BearerRevoked)` after `relay_revoke_bearer(apply)`.
- `relay_stream_authorized_tears_down_on_usb_pull` — `TearDown(GateAbsent)` when a togglable USB probe flips absent (USB-gated vault).
- `relay_stream_authorized_tears_down_on_locked_vault` — locked vault → `TearDown` (no panic, no key).
- e2e (`edge_stream_e2e.rs`, real `serve_edge` + tokio-rustls + EKM-bound DPoP + slow-pump upstream, small `Timing` override): `revoke_mid_stream_tears_down_within_bound`, `usb_pull_mid_stream_tears_down_within_bound`, `still_authorized_stream_survives_a_recheck_tick` (rate_per_min=2 proves peek-not-bump: a bumping re-check would `RateLimited` the 2nd tick), `max_duration_cap_tears_down`. Each truncation case asserts the stream FLOWED then was cut (`1..PUMP_CHUNKS`), not "never started".

## Build/test status (commands run; rtk proxy used so exit codes/diagnostics are intact)
- `cargo fmt --all -- --check` → exit=0
- `cargo clippy --workspace --all-targets -- -D warnings` → exit=0
- `cargo clippy -p envctl-secretd --features relay-edge --all-targets -- -D warnings` → exit=0
- `cargo test -p envctl-secrets-engine` → exit=0 (110 unit + 22 relay [incl. 4 new] + 6 inject + 15 vault + 4 phase0, 0 failed)
- `cargo test -p envctl-secretd --features relay-edge` → exit=0 (52 unit; e2e 5; edge_e2e 1; **edge_stream_e2e 4**; mitm 1; native_mint 11; **proxy_swap_e2e 2 — swap path unchanged**; self_check 2; 0 failed)
- `bash ci/gates/no-c.sh` → exit=0 (PASS; rustls=0.23.40 on ring; zero aws-lc/openssl/C-SQLite — ZERO new deps)
- `bash ci/gates/shape.sh` → exit=0 (PASS)

## Deviations
- The max-duration tear-down reports `DenyReason::PolicyExpired` (the closest decision reason for "stream outlived its allowed lifetime") in the metadata-only audit event — `decide()` is not consulted on the deadline branch (it is an unconditional hard cap). All other tear-downs carry the real `decide()` reason.
- The `RelayStreamTornDown` event has NO proto twin (joins `RelayRevoked` in the conv.rs drop set) — zero proto-schema churn, no new dep, parity-safe (CLI+GUI drain the same funnel). The event is best-effort cosmetic; the open swap is already durably audited by the engine. No engine-side durable audit ROW is appended for the tear-down (plan said "metadata-only audit"; implemented as the metadata-only event, consistent with how the existing relay revoke surfaces).
- The wiring intercepts the body via a generic `wrap_body` closure on the shared `swap_and_respond_streaming` rather than literally returning `body_rx` from `handle_edge_request` (the `body_rx` is created inside the shared core). The local proxy passes identity (unchanged); only the edge wraps. Net effect is exactly what the plan asked for, with zero change to the local proxy plane.

## Handoff notes (for the invariant-guardian — targeted checks)
- **decide() is the only Allow authority**: all re-check policy is in `Engine::relay_stream_authorized` → `authorize_relay(bump=false)` → `decide()`. `edge/stream.rs` is select/forward/drop I/O only — it can never keep a stream alive past a `decide()` deny (verify there is no independent "still allow" branch in `relay_stream_response`).
- **Fail-closed matrix**: every `Err` from `authorize_relay` (locked vault / poisoned lock / store err / vanished bearer / MAC fail / USB absent) maps to `StreamAuthz::TearDown` in `relay_stream_authorized` (the `Err(_) =>` arm); the deadline branch tears down unconditionally. No `unwrap`/`expect`/panic on the periodic path (verify `stream.rs` + `relay_stream_authorized` + `authorize_relay(bump=false)` lock acquisitions use `map_err`). Covered by `relay_stream_authorized_tears_down_on_locked_vault`.
- **peek-not-bump**: `Broker::peek` takes `&self`; `authorize_relay(bump=false)` takes the broker READ lock and calls `peek`. A regression to `bump` would false-tear; enforced by `peek_does_not_mutate_counters_across_n_calls` + the e2e `still_authorized_stream_survives_a_recheck_tick` (rate_per_min=2).
- **No secret in logs/audit**: the tear-down event carries `{relay (=upstream host), token_id (public, via broker::parse_bearer), reason}` only — never the bearer, real key, or body. The bearer in `stream.rs` is `Zeroizing<String>`. `relay_stream_authorized`'s `Err`/`Deny` carry no key; `emit_teardown` carries only the three metadata fields.
- **Byte-for-byte swap path**: `proxy_swap_e2e` (2) + the engine `relay.rs` `relay_swap_*`/`decide` tests all pass unchanged — the `authorize_relay` factoring preserved `relay_swap_prepare`'s observable behavior (deny audit shape + `AllowPrepared` key fetch under the same vault read-lock lifetime).
- **Default-OFF**: `edge/stream.rs`, the `swap_and_respond_streaming` edge wiring, and `EdgeConfig.recheck_timing` are reachable only under `relay-edge` (the whole `edge` module is `#[cfg(feature="relay-edge")]`). The engine method is inert unless the edge calls it. Default workspace build + clippy are clean without the feature.
- **Zero new deps / no-C / one-rustls**: `no-c.sh` PASS; nothing new in the lockfile (tokio time/select/mpsc, http-body-util StreamBody/Either, hyper, tokio-stream ReceiverStream, the engine — all already resolved). TLS/cert/EKM code untouched.
