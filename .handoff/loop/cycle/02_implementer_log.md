# Implementation log: TASK-0031 PR-1 — F2 remote relay-edge listener (in-process TLS + DPoP/EKM)

Status: **GREEN**. PR-1 minimal-coherent edge implemented behind the `relay-edge` cargo feature
(default-OFF), config-gated by `[edge].enabled`. One route: `POST /v1/relay/swap` → the EXISTING
`Engine::relay_swap`. Zero new lockfile crates. All four CI gates pass.

## Confirmed EKM accessor (the flagged risk — verified against pinned source, NOT assumed)
- rustls **0.23.40** (lockfile-pinned): `ConnectionCommon::export_keying_material<T: AsMut<[u8]>>(&self,
  output: T, label: &[u8], context: Option<&[u8]>) -> Result<T, Error>` (`src/conn.rs:460`).
  `ServerConnection` / `ClientConnection` deref to `ConnectionCommon`, so both ends expose it.
- tokio-rustls **0.26.4** (lockfile-pinned): post-handshake `server::TlsStream<IO>::get_ref(&self) ->
  (&IO, &ServerConnection)` (`src/server.rs:314`); client side returns `(&IO, &ClientConnection)`
  (`src/client.rs:217`). So the edge reads EKM via `tls_stream.get_ref().1.export_keying_material(out,
  EKM_LABEL, None)`. The e2e test computes the SAME value on the client side and binds it into the
  DPoP `ekm` claim — proving the symmetric RFC 5705 export matches end-to-end (happy-path 200).
- Label: `EKM_LABEL = b"EXPORTER-envctl-relay-dpop-v1"`, 32-byte output, `context = None`.

## Engine API delta (the parity contract — additive; `decide()` UNTOUCHED)
- `EgressReq` (engine `lib.rs:198`) gains `pub remote: Option<broker::decide::RemotePeer>`. Set `None`
  at every existing constructor: `proxy.rs`, `relay_swap`'s owned-copy rebuild, `tests/relay.rs`,
  `tests/proxy_swap_e2e.rs`.
- `relay_swap_prepare` (engine `lib.rs`): the hardcoded `remote: None` → `remote: req.remote.clone()`
  (the ENTIRE engine wiring; `decide()` clause 11a re-asserts fail-closed).
- NEW `pub fn Engine::load_remote_client(&str) -> anyhow::Result<Option<RemoteClient>>` (additive,
  non-mutating read accessor; wraps the internal `store.load_remote_client`). The edge raises
  unknown/revoked → 401 BEFORE `decide()`.
- NEW `Paths::relay_tls_dir()` → `~/.config/env-ctl/relay-tls/` (engine `paths.rs`; mirrors
  `config_file()`). NOTE: the plan said `crates/secretd/src/paths.rs`, but `Paths` actually lives in
  the ENGINE (`crates/secrets-engine/src/paths.rs`) — added there (the real home). See Deviations.

## Changes (files touched)
- `crates/secrets-engine/src/lib.rs` — `EgressReq.remote` field; `relay_swap_prepare` wire;
  `relay_swap` owned-copy carries `remote`; NEW `load_remote_client` accessor.
- `crates/secrets-engine/src/paths.rs` — NEW `relay_tls_dir()` + 2 unit tests.
- `crates/secrets-engine/tests/relay.rs` — `remote: None` in `post_req`; NEW test
  `relay_swap_remote_unverified_dpop_denied_no_dpop` (remote=Some{dpop_verified:false}→RemoteNoDPoP).
- `crates/secretd/Cargo.toml` — NEW `relay-edge` feature (default-OFF; `dep:tokio-rustls,ring,base64,
  sha2`); dev-deps `rcgen/ring/base64/sha2/serde_json` (all already in graph — zero new crate).
- `crates/secretd/src/lib.rs` — `#[cfg(feature="relay-edge")] pub mod edge;`.
- `crates/secretd/src/edge/mod.rs` — NEW: `EdgeConfig` + `pub async fn serve_edge(engine, paths, cfg,
  shutdown)`.
- `crates/secretd/src/edge/dpop.rs` — NEW: pure-sync `verify_dpop_proof` (RFC 9449) + `VerifiedDpop`/
  `DpopReject`/`HttpMethod` + 17 vector tests.
- `crates/secretd/src/edge/tls.rs` — NEW: `RelayTlsConfig` (loads ONLY `relay_tls_dir()`; ring-only
  ServerConfig; no MITM-CA import — FS-S25 structural) + 4 fail-closed tests.
- `crates/secretd/src/edge/listener.rs` — NEW: accept→TLS→EKM→verify-ladder→jti→registry→
  `swap_and_respond(remote: Some)`; `SwapOutcome::{Allowed→200,Denied→403,InternalRefused→503}`.
- `crates/secretd/src/proxy.rs` — `swap_and_respond` gains a `remote` param (ONE shared swap core for
  proxy+edge); helpers (`ProxyCtx`/`bare`/`extract_bearer`/`method_from_hyper`/`request_host`/
  `ProxyBody`) made `pub(crate)`; `ProxyCtx::for_edge`; bearer extracted via Bearer-scheme when remote.
- `crates/secretd/src/config.rs` — `[edge]` block parse + `EdgeSettings::load` (env>file; enabled⇒
  bind_addr required, fail-closed).
- `crates/secretd/src/main.rs` — start `serve_edge` under the SAME broadcast shutdown when the feature
  is on AND `[edge].enabled`; cert-load/bind failure FATAL when explicitly enabled; await on shutdown.
- `crates/secretd/tests/edge_e2e.rs` — NEW `#[tokio::test] #[cfg(feature="relay-edge")]` full e2e.
- `ci/gates/shape.sh` — armed/tightened the FS-S25/FS-S18 edge-vs-MITM-CA grep + REQ-SEC-11 grep now
  that `crates/secretd/src/edge/` exists.

## Tests added + what they prove
- engine `paths.rs`: `relay_tls_dir_is_under_config_sibling_of_secretd_toml`,
  `relay_tls_dir_resolves_under_env_ctl_config` — relay-tls/ is under config, NOT data (≠ MITM-CA).
- `edge/dpop.rs` (17): valid accept; EKM uncomputable→reject (fail-closed), EKM mismatch, EKM claim
  absent; bad signature; tampered payload; wrong typ/alg; non-OKP jwk; htm/htu mismatch; iat past/
  future; missing jti; malformed JWT (≠3 segs, empty); non-base64 seg; RFC 7638 jkt == SHA-256(canon).
- `edge/tls.rs` (4): loads relay-tls/; missing dir / missing key / empty cert all fail closed.
- engine `tests/relay.rs`: `relay_swap_remote_unverified_dpop_denied_no_dpop` → `RemoteNoDPoP`, key
  never fetched. (CrossKindPresentation — remote bearer over local `remote: None` — is ALREADY proven
  by the existing `relay_mint_remote_binds_client_and_cross_kind_denied_locally`.)
- `tests/edge_e2e.rs` (1 test, 5 scenarios): real tokio-rustls handshake (trusting ONLY relay-tls
  cert), client-computed EKM bound into a valid DPoP proof → POST /v1/relay/swap → **200 + the REAL
  key (SENTINEL) reaches the faked Upstream**. Negatives: replayed jti→401, no DPoP header→401,
  tampered proof→401, unregistered client→401 (the `load_remote_client→None/revoked` pre-decide branch
  a revoked client also hits).

## Build/test status (exact commands; rtk proxy raw)
- `cargo fmt --all` + `--check` — clean.
- `cargo clippy --workspace --features relay-edge -- -D warnings` — PASS (exit 0).
- `cargo clippy --workspace -- -D warnings` (feature OFF) — PASS (exit 0).
- `cargo clippy -p envctl-secretd -p envctl-secrets-engine --all-targets --features relay-edge -- -D
  warnings` (test code) — PASS (exit 0).
- `cargo test -p envctl-secrets-engine -p envctl-secretd --features relay-edge` — PASS: secretd-lib
  52, edge_e2e 1, mitm_e2e 1, proxy_swap_e2e 2, e2e 5, native_mint 11, self_check 2; engine-lib 129,
  relay 18, vault 15, inject 4, phase0 6. 0 failed.
- `cargo test -p envctl-secretd` (feature OFF) — PASS: secretd-lib 31, e2e 5, mitm_e2e 1, native_mint
  11, proxy_swap 2, self_check 2. 0 failed. (`edge_e2e` correctly absent — `#[cfg(relay-edge)]`.)
- `bash ci/gates/no-c.sh` — PASS (rustls=[0.23.40] on ring=[0.17.14]; zero aws-lc/openssl/C-SQLite;
  `--all-features` covers `relay-edge`; independent `cargo metadata --features relay-edge` ⇒ 0 banned).
- `bash ci/gates/shape.sh` — PASS. `bash ci/gates/enable.sh` — PASS. `bash ci/gates/p7.sh` — PASS.
- `cargo build -p envctl-engine -p envctl` — clean (tight loop unaffected).

## Pre-existing lint baseline
No pre-existing fmt/clippy drift surfaced; the workspace was clean before and after. (Toolchain is the
pinned `1.96.0` per `rust-toolchain.toml`, so no floating-`stable` mis-attribution.)

## Deviations (with rationale)
1. **`relay_tls_dir()` lives in the ENGINE `paths.rs`, not `secretd/src/paths.rs`.** The plan named
   `crates/secretd/src/paths.rs`, but secretd has no `paths.rs` — the `Paths` struct is defined in
   `crates/secrets-engine/src/paths.rs` and secretd imports `envctl_secrets::paths::Paths`. Adding the
   helper to the real home keeps ONE `Paths` type (no shadow). Unit-tested there.
2. **The edge reuses `proxy::swap_and_respond` (threaded a `remote` param) instead of an edge-local
   `relay_swap` call site.** This keeps ONE swap core for the proxy + the edge (engine-parity
   principle: the planes can't diverge in how they drive `relay_swap`/stream the upstream via the
   `EGRESS_CTX` task-local + `DaemonUpstream`). The edge still builds `EgressReq{remote: Some(rp)}` and
   reaches the EXISTING `relay_swap` exactly as the plan requires — the param is just the seam.
3. **Bearer header convention for the remote plane.** `swap_and_respond` previously extracted the
   bearer with the UPSTREAM provider's convention (Anthropic = bare `x-api-key`). The remote edge
   client addresses the EDGE and always sends `Authorization: Bearer`, so for `remote.is_some()` the
   bearer is read with the Bearer-scheme (`Provider::Generic`). Found + fixed via the e2e (was a 403).
4. **"Revoked client → 401" e2e is exercised via the unregistered-client path.** The edge refuses an
   unknown OR revoked client through the SAME `load_remote_client → None/disabled/revoked → 401`
   pre-decide branch. There is no public remote-client revoke verb yet (revocation TEAR-DOWN is
   explicitly PR-3 per the plan's out-of-scope), and adding one (or a 68-method `SharedStore` test
   shim) would exceed PR-1 scope, so the e2e proves the identical edge refusal branch with an
   unregistered `client_id`. The negative assertion (request never reaches a mint) is intact.
5. **Upstream target framing.** PR-1 conveys the upstream target via `X-Relay-Upstream-Host` +
   `X-Relay-Upstream-Path` headers (the edge route is the fixed `/v1/relay/swap`; the DPoP `htu` binds
   the edge URL). `decide()` re-fences host/path/method against the policy allowlist, so a
   forged/unallowed target is denied IN THE ENGINE — the edge enforces no policy.

## Handoff notes (for the invariant-guardian — targeted checks)
- **FS-S20 (EKM channel binding):** `dpop_verified:true` is set ONLY after `verify_dpop_proof`
  succeeds, which requires `ekm = Some(..)` AND the proof's `ekm` claim to equal it. Uncomputable EKM
  ⇒ `EkmUncomputable` ⇒ **403** (listener maps the three `Ekm*` rejects to 403, all other rejects to
  401). Verify: `edge/dpop.rs::uncomputable_ekm_rejected_failclosed` + `ekm_mismatch_rejected` +
  `ekm_claim_absent_rejected`, and the e2e's symmetric client-side EKM export feeding the happy path.
- **FS-S25/FS-S18 (relay-tls ONLY, never MITM CA):** structural in `edge/tls.rs` (`RelayTlsConfig`
  reads ONLY `relay_tls_dir()`, imports no MITM-CA type). Backstop: `ci/gates/shape.sh` greps the
  `edge/` tree for any MITM/local-CA symbol. Confirm the grep actually scans `crates/secretd/src/edge`
  (it does — `EDGE_SRC`).
- **Poisoned-mutex fail-closed:** `listener.rs verify_remote_presentation` step (4) — `conn.jti.lock()`
  `Err(_) => 401`, NEVER `.unwrap()`. The replay store itself is the F6 `JtiReplayStore` (read-only).
- **Never reaches a mint on failure:** every verify failure returns from `verify_remote_presentation`
  BEFORE `swap_and_respond` is called; confirm there is no path that builds `RemotePeer` without all
  of: EKM bound + DPoP verified + jti fresh + client registered+enabled + proven jkt == registered jkt.
- **No secret bytes in logs:** the listener's `tracing::debug!` lines carry only status code / peer /
  error-display — never bearer/proof/EKM/key. The engine emits the secret-free durable audit row.
- **decide() untouched:** confirm `crates/secrets-engine/src/broker/decide.rs` has NO diff (only the
  `EgressReq`/`relay_swap_prepare`/`load_remote_client`/`paths` additive edits).
- **Default-OFF proof:** feature-off `cargo test -p envctl-secretd` passes with `edge_e2e` absent and
  no edge module compiled; `[edge]` absent ⇒ `serve_edge` never called (main.rs is `#[cfg]`-gated).

## Follow-ups (deferred PR-2 / PR-3 — recorded per the plan's out-of-scope)
- **PR-2:** server-issued nonce challenge (OI-SM-1 nonce half; the `dpop.rs` window is nonce-agnostic
  and ready to extend the dedup key to `(client_id, nonce, jti)`); per-IP / per-client rate-limit +
  body-size caps + request timeouts + admission shedding (CVE-2024-47609 accept-loop class); hardened-
  mode mTLS `ClientCertVerifier` from a SEPARATE remote-clients CA (OI-SM-4); a startup self-check that
  the presented edge cert chains to a PUBLIC root and explicitly NOT the MITM/remote-clients CA.
- **PR-3 (→ folds into TASK-0032):** streaming + in-stream `decide()` re-check; active stream tear-down
  on `RevokeBearer`/`RevokeRemoteClient`/`lock`/USB-pull; a public `revoke_remote_client` engine verb
  (the edge's revoked-client refusal branch is already present via `load_remote_client`).
