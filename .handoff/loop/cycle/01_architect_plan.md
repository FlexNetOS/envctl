<<<<<<< HEAD
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
=======
# TASK-0035 — secretd gRPC surface gaps (Vault/Relay/Audit/read)  ·  VERDICT: GO

Single-repo (envctl), engine-first, cohesive cycle. Every in-scope RPC already has its proto
message and `secretctl` CLI client wired — the gap is purely **engine read/mutation methods +
secretd handler wiring**. Zero new dependencies. **No proto change.**

## Target repos
1 repo: envctl. Crates: secrets-engine (new public methods + one Store method), secretd (handler
wiring + conv.rs converters), secretctl (confirm `audit query` verb; rest already wired). GUI
out of scope (secretctl is the front-end for secrets-engine). → sequential single-crew.

## In scope (replace `Status::unimplemented` in crates/secretd/src/grpc.rs)
- Vault.List — metadata-only list (no values, no ct_tag/nonce).
- Vault.Rm — DESTRUCTIVE, dry-run-by-default (`apply && confirm`), locked-refusal.
- Vault.Rotate — append new sealed version via existing secret_put (carry-forward meta); apply-gated.
- Relay.Create — named-policy create via existing save_relay_policy (additive, unlocked).
- Relay.List — read list_relay_policies; filter revoked unless include_revoked.
- Audit.Query — pass-through to store.query_audit; daemon post-filters actor/relay/since/until; clamp limit ≤1000.
- GetSecret.meta — populate the currently-`None` meta from secret_meta (metadata only).

## Out of scope — DEFER to a NEW backlog item (record only, do not build)
Certs.* service (CaInit/Rotate/Issue/Renew/Revoke/TrustApply/List), non-mitm ca_issue
(secrets-engine/src/lib.rs ~2290-2330), `secretctl ca`; empty features provider-openai + libsql `embedded`.
→ Phase 4+.

## Engine API delta (all sync, non-printing, return metadata not secrets; audit via audit_ok/refuse)
1. secret_list(provider: Option<Provider>, sink) -> Vec<SecretListItem> — store.list_secret_names + get_secret_latest per name; new `SecretListItem` struct (non-secret SecretRow fields + version + created_ts). Gate on unlocked (Locked when dek().is_none(), matches secret_get).
2. secret_meta(name) -> Option<SecretMeta> — non-secret metadata for GetSecret.meta. No value.
3. secret_rm(name, apply, sink) -> u32 — DESTRUCTIVE template = relay_revoke. Locked-refusal. apply=false counts would-remove (list_secret_versions), mutates nothing. apply=true removes all versions via new Store::delete_secret, audits, emits SecretEvent. No secret bytes.
4. secret_rotate(name, new_value: Zeroizing<Vec<u8>>, apply, sink) — PREFERRED engine method: meta-read (carry provider/note/broker_only) + secret_put under write lock (secret_put already appends version=max+1 monotonically). apply-gated dry-run.
5. relay_list(include_revoked, sink) -> Vec<RelayPolicy> — store.list_relay_policies; filter revoked.
6. relay_create(policy: RelayPolicy, sink) -> i64 — store.save_relay_policy(RelayPolicyRow{id:0,policy}); unlocked; audit relay_created.
7. audit_query(since_seq, limit, sink) -> Vec<AuditRecord> — store.query_audit; clamp limit. Already metadata-only.

### New Store-trait surface (one method)
- Store::delete_secret(name) -> Result<u32> with DEFAULT `Ok(0)` (non-breaking to existing impls) + real InMemStore impl (retain-filter, return count). libSQL backend: real `DELETE FROM secrets WHERE name = ?` if straightforward, else inherit default + tracked follow-up so no-C/compile stay green. Update any hand-rolled mock (lib.rs ~2964).

## Proto delta
NONE. All messages/fields exist in control.proto: Vault.List/Rm/Rotate, Relay.Create/List,
Audit.Query, GetSecretResp.meta, and the apply/confirm fail-closed fields. CI proto round-trip stays green.

## Invariants (each checkable)
- no-C: zero new deps; secrets-engine still never links libSQL. Run ci/gates/no-c.sh.
- one rustls ring-only: no TLS/CA crate touched.
- engine = single sync non-printing lib: all methods sync, emit SecretEvents + audit, no println!/clap/UI.
- destructive fail-closed + dry-run default: Rm/Rotate gate on apply (proto3 default false), refuse when locked; unit-test-enforced.
- no secret bytes in logs/audit/List: SecretListItem/SecretMeta carry only non-secret SecretRow fields; audit_query returns engine-written AuditRecords; rotate value in Zeroizing.
- broker-only never revealable: untouched; List/meta expose broker_only as bool flag only; reveal gate not modified.

## Daemon wiring (grpc.rs)
Replace 6 unimplemented bodies (~205-234, 360-369, 414-422, 719-729) with spawn_blocking engine
calls mapped via conv.rs. Add converters secret_list_item_to_proto (→ v1::SecretMeta),
policy_to_proto (engine RelayPolicy → v1::RelayPolicy); reuse provider_to_proto/audit_to_entry.
Populate GetSecretResp.meta in the get handler (~193) via engine.secret_meta. Fold
apply = req.apply && req.confirm for Rm (mirrors Relay.Revoke ~377). Map Locked→failed_precondition,
empty-arg→invalid_argument. Update module-doc unimplemented list (~13-15); leave Certs.* as Phase 4+.

## Sequencing (leaf-first)
Store::delete_secret + InMemStore impl → engine reads (secret_list/SecretListItem, secret_meta,
relay_list, audit_query) + inline tests → engine mutations (relay_create, secret_rm, secret_rotate)
+ tests → conv.rs converters + tests → secretd handlers + tokio round-trip tests → confirm/add
secretctl `audit query` verb → module-doc updates → append deferred Certs.* backlog item →
fmt/clippy --workspace -D warnings + cargo test -p envctl-secrets-engine -p envctl-secretd →
no-c.sh + shape.sh.

## Resolved defaults (open questions — none block)
1. Reads gate on unlocked (fail-closed, consistent with secret_get). 2. Rotate = engine method
(single authority). 3. Audit filters daemon-side this cycle (keep engine signature minimal).
4. libSQL delete_secret: real impl if straightforward, else default-stub + follow-up.

## Risks
Store-trait addition blast radius (mitigate: default body + check libsql impl + mock compiles).
Double-audit on Get.meta (keep secret_meta un-audited to avoid duplicate rows on a Get).
rtk corrupts fmt/clippy — implementer uses `rtk proxy` / file redirect.
>>>>>>> 727f7ba (secretd: implement Vault/Relay/Audit gRPC surface gaps (TASK-0035))
