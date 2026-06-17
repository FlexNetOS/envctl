# TASK-0032 (F5, P0) — streaming-revocation tear-down (FS-S5) · VERDICT: GO

Adds an engine-side `relay_stream_authorized(...)` re-check seam (re-runs the SAME `decide()`
authorization with fresh clock/USB/revocation reads, NO key fetch, NO counter bump) and a streaming
tear-down driver in the existing `relay-edge` listener that wraps the upstream stream with a periodic
re-check + revocation observation, aborting the in-flight `StreamBody` the instant authorization lapses.
ZERO new dependencies. Single-repo (envctl), sequential single-crew.

## Target repos
- **envctl** (single). 2 modified (`crates/secrets-engine/src/lib.rs`, `crates/secretd/src/edge/listener.rs`)
  + 1 new (`crates/secretd/src/edge/stream.rs`) + 1 test (`crates/secretd/tests/edge_stream_e2e.rs`).
  ≤3 engine-first modules → sequential single-crew (no A2 / no grit).

## Engine API delta (additive, non-mutating)
The existing seam can't be reused: `relay_swap`/`relay_swap_prepare` FETCH the key and `broker.bump()`
the usage counters every call — re-running per tick would consume budget/rate and re-materialize the key.
New method answers only "is this stream still authorized?" with no key fetch, no counter mutation,
`bytes_out = 0`, routed through the SAME `decide()`:

```rust
// crates/secrets-engine/src/lib.rs
pub fn relay_stream_authorized(&self, bearer: &str, req: &EgressReq, sink: &EventSink) -> StreamAuthz;
pub enum StreamAuthz { Authorized, TearDown(DenyReason) }  // DenyReason re-exported from broker::decide
```
- Factor the bearer-verify + policy-load + gate-snapshot prelude of `relay_swap_prepare` (lib.rs:1409–1546)
  into private `authorize_relay(&self, bearer, req, bump: bool) -> {Deny(DenyReason)|Allow(AllowMeta)}`
  WITHOUT the key fetch. `relay_swap_prepare` calls it `bump=true` then fetches key on Allow (behavior
  byte-for-byte unchanged). `relay_stream_authorized` calls it `bump=false`, ignores key fetch.
- Add `Broker::peek(&self, token_id, now_ms) -> (u64,u64,u32)` (read-only counterpart to `bump` at
  broker/mod.rs:231): recompute rate_in_window for the current window WITHOUT incrementing, so the
  re-check still enforces ceilings against live tallies but never consumes them. `bytes_out=0`.
- Pass the SAME `RemotePeer` captured at open → decide() clause 11a (decide.rs:192) re-asserts
  `dpop_verified` + client_id/jkt binding each tick.
- OPTIONAL metadata-only `SecretEvent::RelayStreamTornDown { relay, token_id, reason }` for GUI/CLI
  audit granularity (recommended; consumed identically by both front-ends — no divergence). No front-end
  behavior change either way (neither drives the edge).

## Edge changes
- NEW `crates/secretd/src/edge/stream.rs` (`#[cfg(feature="relay-edge")]`): `relay_stream_response(...)`
  wraps the engine's per-request upstream chunk `mpsc::Receiver` (proxy.rs:513) in a supervised middle
  task forwarding chunks downstream while `tokio::select!` races: (a) next upstream chunk, (b)
  `tokio::time::interval` tick, (c) revocation observation, (d) hard max-stream-duration deadline. On a
  tick → `engine.relay_stream_authorized(...)`; on `TearDown`/deadline → drop downstream sender (StreamBody
  ends cleanly, client sees HTTP/2 close) + metadata-only audit. Body stays the existing `ProxyBody`
  (`Either::Left(StreamBody::new(ReceiverStream::new(rx)))`, proxy.rs:47–53) — same shape, no new framework.
- MODIFY `crates/secretd/src/edge/listener.rs::handle_edge_request` (:159): after the verified `RemotePeer`
  (:330) and an `Allowed` swap, route the returned `body_rx` through `stream::relay_stream_response`
  instead of returning it bare; thread the captured-at-open `EgressReq` + bearer + `RemotePeer` (all in scope).
- Reuse `swap_and_respond` → `relay_swap` for the swap itself (no policy duplicated in the edge).

## Re-check cadence & cancellation
- `tokio::time::interval(RECHECK_INTERVAL=2s)` (named const) runs `relay_stream_authorized` each tick;
  hard `MAX_STREAM_SECS` (~300s) deadline tears down unconditionally.
- Worst-case revoke/lock/USB-pull detection latency = one interval (≤2s): decide() reads the USB gate +
  bearer `revoked` + policy fresh each call. Select wakes immediately on upstream EOF/error.
- FORK (resolved, non-blocking): interval-poll vs watch-push. Interval-poll ships now (≤2s bound, zero new
  wiring). `tokio::sync::watch` push (~0 latency) needs an engine broadcast seam keyed by client/token →
  larger cross-cutting change → documented PR-4 follow-up.

## Dep decision (no-C proof)
tokio (interval/select/mpsc/watch), http-body-util (StreamBody/Either), hyper/hyper-util, tokio-stream
(ReceiverStream), envctl-secrets-engine — ALL already resolved (secretd Cargo.toml:44–49; proxy.rs:47–53).
ZERO new lockfile crates. No SQLite/OpenSSL/aws-lc/mimalloc; no new rustls backend; ring untouched (re-check
does no crypto). `no-c.sh` green by construction (still run it).

## Fail-closed matrix (every uncertainty → tear down)
decide() Deny → TearDown(reason) · vault locked → TearDown · RwLock poisoned → map_err→TearDown (no unwrap)
· store err re-loading bearer → TearDown · bearer row vanished/MAC fails → TearDown · USB pulled (gate
absent) → TearDown(≤2s) · Engine handle dropped → sender dropped, stream closes · client vanished →
downstream send err, stop+drop · max-duration → TearDown · re-check panic FORBIDDEN (no unwrap/expect/index
on hot path). Default = always tear-down on uncertainty.

## Tests
Engine unit (lib.rs near relay_swap + broker/mod.rs for peek): Authorized for valid remote bearer;
TearDown(BearerRevoked) after relay_revoke_bearer(apply); TearDown(GateAbsent) on absent USB gate;
TearDown on locked/poisoned (no panic); peek leaves counters unchanged across N re-checks.
E2E new `crates/secretd/tests/edge_stream_e2e.rs` (`#![cfg(feature="relay-edge")]`, reuse edge_e2e harness:
fake PresentUsb, RecordingUpstream slow-pumping multiple chunks, with_seams, real serve_edge + tokio-rustls
client + EKM-bound DPoP): (1) revoke mid-stream → client stream closes within ~2× RECHECK_INTERVAL, body
truncated; (2) USB pull mid-stream → close within bound; (3) survives a tick when still authorized (no
false-tear, counters didn't deny); (4) max-duration cap tears down. Generous CI timeouts; fakes only.

## Sequencing (leaf-first)
1. `Broker::peek` + unit test. 2. factor `authorize_relay(bump)` + `relay_stream_authorized` + `StreamAuthz`
(re-export DenyReason); confirm relay_swap byte-for-byte unchanged (existing proxy_swap_e2e/decide tests
pass); engine unit tests; (optional RelayStreamTornDown event). 3. add `edge/stream.rs` + `pub mod stream;`
(cfg relay-edge). 4. wire body_rx through relay_stream_response in listener. 5. edge_stream_e2e.rs.
6. fmt + clippy --workspace -Dwarnings + test -p secrets-engine + test -p secretd --features relay-edge +
no-c.sh + shape.sh (via `rtk proxy cargo ...`).

## Invariants (each checkable)
1 no-C: zero new crates, no-c.sh from cargo metadata. 2 engine single non-printing: policy in
relay_stream_authorized→decide(), edge stream.rs is select/forward/drop I/O only, no println!. 3 decide()
only Allow authority: re-check calls decide() with SAME captured inputs incl. open-time RemotePeer.
4 fail-closed: matrix maps every error to tear-down, no unwrap on periodic path. 5 no secret in logs/audit:
tear-down events {reason,client_id,token_id} only; key confined to Upstream::send. 6 relay-tls/EKM unchanged:
no TLS/cert/EKM code touched. 7 default-OFF: new module+wiring cfg relay-edge; engine method inert unless
edge calls it. 8 dry-run/fail-closed destructive: N/A (tear-down is internal fail-safe, not an --apply op).

## Risks
peek mis-impl (reusing bump) → false deny — guarded by "counters unchanged" unit test. Backpressure: middle
forwarding task must keep the bounded BODY_CHANNEL_CAP (proxy.rs:42), no unbounded buffer. 2s poll is
"prompt" not instant (watch-push fork if sub-second needed). CI flake → generous timeouts + fakes.

## Out of scope (follow-up)
PR-4 watch-channel push (~0 latency) · per-client fan-out tear-down of all concurrent streams on one revoke
· Profile B presence-token re-check (blocked OI-SM-2/3) · N-byte cadence refinement.
