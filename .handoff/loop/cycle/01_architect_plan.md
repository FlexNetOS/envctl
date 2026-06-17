# TASK-0031 — F2 remote relay-edge listener (in-process TLS + DPoP/EKM) · VERDICT: GO (3-PR split)

PR-1 (THIS cycle) = minimal-coherent edge. PR-2 (nonce + rate-limit + hardened mTLS) and PR-3
(streaming + revocation tear-down → folds into TASK-0032) stacked after. Engine remote core already
built (relay_mint_remote, register_remote_client, decide() clause 11a, JtiReplayStore) — do NOT rebuild.

## Target repos
1 — envctl. Branch task-0031-edge (stacked on task-0030-jti; F6 JtiReplayStore present, consumed read-only).

## Engine API delta (ONE additive seam — decide() UNTOUCHED)
1. Add `pub remote: Option<broker::decide::RemotePeer>` to `EgressReq` (lib.rs:198). Set `None` at all
   existing constructors (proxy.rs + tests — enumerate via grep `EgressReq {`).
2. In `relay_swap_prepare` (lib.rs:1519-1533) replace hardcoded `remote: None` with `remote: req.remote.clone()`.
   That is the ENTIRE engine wiring. decide() clause 11a (decide.rs:187-207) already fails closed
   RemoteNoDPoP if !dpop_verified, denies CrossKindPresentation on plane mismatch, RemoteBindingMismatch
   on client_id/jkt divergence. Do NOT modify decide().
3. Edge calls EXISTING `Engine::relay_swap(bearer, &EgressReq{remote: Some(rp), ..}, &sink)` (lib.rs:1267);
   map SwapOutcome::{Allowed,Denied,InternalRefused} → HTTP 200/403/503.
4. Add `Engine::load_remote_client(&str) -> Result<Option<RemoteClient>>` read accessor (additive,
   non-mutating; internal use exists at lib.rs:1114). Edge raises RemoteClientUnknown/Revoked → 401
   BEFORE decide() (mirrors UnknownBearer pre-decide raise).

## Module layout (NEW crates/secretd/src/edge/)
- mod.rs    — `EdgeConfig`, `pub fn serve_edge(engine, paths, cfg, shutdown) -> Result<(SocketAddr, JoinHandle)>`
- listener.rs — rustls ServerConfig from relay-tls/ ONLY; TlsAcceptor; accept loop; per-conn handler;
  EKM read; map SwapOutcome → hyper Response.
- dpop.rs    — pure sync `verify_dpop_proof(...) -> Result<VerifiedDpop, DpopReject>` (RFC 9449) + types.
- tls.rs     — `RelayTlsConfig` newtype that can ONLY load relay-tls/{cert.pem,key.pem} (FS-S25 structural;
  never imports MITM-CA types).
- `Paths::relay_tls_dir()` NEW helper in paths.rs (mirrors config_file()): ~/.config/env-ctl/relay-tls/.
- Gate behind a `relay-edge` cargo feature (default-off, mirrors mitm-ca) so --no-default-features drops it.

## DPoP verification (edge/dpop.rs — pure, vector-testable; SERVER-MODE §4.2)
1. Parse DPoP header JWT (3 base64url segs). Malformed → 401.
2. Header: typ=="dpop+jwt", alg=="EdDSA" (Ed25519), embedded OKP jwk (x = 32-byte pubkey). Else 401.
3. Verify sig over b64url(header).b64url(payload) via ring ED25519 UnparsedPublicKey::verify. Bad → 401.
4. jkt = b64url(SHA-256 of RFC 7638 canonical JWK) (sha2). Must match registered client dpop_jkt.
5. Claims: htm==method, htu==canonical URI (scheme+host+path, no query), iat in F6 window. Mismatch → 401.
6. EKM (FS-S20, RFC 9449 §5): rustls 0.23 `export_keying_material` off the terminated tokio_rustls server
   stream (get_ref().1); proof must bind it; uncomputable binding → 403 fail-closed (edge MUST terminate
   TLS in-process; no external TLS-terminating proxy — L4 passthrough only). IMPLEMENTER: confirm the
   exact accessor against rustls 0.23 (context7) before wiring — flagged risk, don't assume.
7. jti: `JtiReplayStore::check_and_record(client_id, jti, iat_ms, now_ms)` under edge-owned
   `Mutex<JtiReplayStore>`; any Err → 401; POISONED mutex → 401 (never .unwrap()).
8. All pass → RemotePeer{client_id, dpop_jkt, dpop_verified:true} → relay_swap. decide() re-asserts.

## Startup (config-gated, OFF by default)
In secretd main::serve, after the proxy task: load `[edge]` from secretd.toml (bind addr, enabled). Absent/
disabled → do not bind (stock secretd serves no public edge). Enabled → serve_edge under the SAME broadcast
shutdown as the proxy. Cert-load/bind failure is FATAL when edge explicitly enabled (fail-closed). Serve
hyper HTTP/1.1+2 over tokio_rustls server stream — reuse already-linked hyper/hyper-util/http-body-util
(NO axum). PR-1 route = exactly `POST /v1/relay/swap` → relay_swap.

## Zero new deps (no-C proof)
All present in resolved graph: tokio-rustls 0.26 + rustls 0.23 (ring-only, default-features=false
features=["ring","tls12"], Cargo.toml:91-97), ring 0.17 (Ed25519), base64 0.22, sha2 0.10, serde_json,
hyper/hyper-util/http-body-util (secretd). Banned list (no-c.sh:74) — none pulled. Edge doesn't touch the
store layer. Run no-c.sh after the EgressReq engine edit.

## Invariants (each checkable)
- no-C / one rustls ring-only: PASS (zero new deps; reuse pinned ring rustls). Run ci/gates/no-c.sh.
- relay-tls ONLY never MITM CA (FS-S25): PASS by construction (RelayTlsConfig newtype; edge never imports
  MITM-CA types). Add CI grep: edge/ never references the MITM-CA path/symbol (guardian-checkable).
- EKM binding (FS-S20): PASS — dpop_verified true ONLY after export_keying_material succeeds AND proof binds
  it; uncomputable → 403. EKM-mismatch reject vector tested.
- fail-closed: bad TLS / bad-expired-replayed DPoP (401) / jti reject (401) / unknown-revoked client (401
  pre-decide) / poisoned mutex (401) / locked vault-internal (503). Never reaches a mint on failure.
- engine single non-printing authority: PASS — edge does I/O + proof verify only; MINT/DECIDE stay in
  relay_swap/decide; edge uses tracing metadata-only, never println!.
- no secret bytes in logs/audit: PASS — log client_id/source_ip/jkt-hash/decision only; engine emits the
  secret-free durable audit row.
- destructive guards / dry-run: N/A (network listener gated by config presence = the --apply analogue).

## Sequencing (leaf-first)
1. Engine seam: EgressReq.remote field + all-constructor None + relay_swap_prepare wire + load_remote_client
   accessor; build engine + decide.rs table tests pass. 2. Paths::relay_tls_dir() + test. 3. edge/dpop.rs
   pure verifier + vectors (TDD). 4. edge/tls.rs RelayTlsConfig (proxy.rs:686 shape, relay-tls/ cert, no
   leaf-mint). 5. edge/listener.rs accept→TLS→EKM→bearer+DPoP→verify→jti→load/verify client→EgressReq{remote}
   →relay_swap→map outcome. 6. edge/mod.rs EdgeConfig+serve_edge; [edge] parse (mirror StoreConfig). 7. main.rs
   start task under shared shutdown when enabled; relay-edge feature. 8. CI grep FS-S25. 9. tests. Then
   fmt/clippy --workspace -D warnings + 4 ci/gates.

## Tests
Unit (dpop.rs): ACCEPT (valid Ed25519, correct htm/htu/iat, matching EKM); REJECT vectors — bad sig, wrong
alg/typ, htm/htu mismatch, iat out of window, EKM mismatch, malformed JWT; jkt matches RFC 7638 vector; jti
replay → 401; poisoned mutex → reject. tls.rs: loads relay-tls/, missing relay-tls/ fails closed, no MITM-CA
path. Engine: relay_swap with remote=Some{dpop_verified:false} → RemoteNoDPoP; remote bearer + remote=None →
CrossKindPresentation. Integration (crates/secretd/tests/edge_e2e.rs, reuse mitm_e2e.rs harness): test
relay-tls cert in tempdir, serve_edge w/ faked seams (fake USB so register/mint pass gate, fake Upstream),
tokio-rustls client real handshake → register+mint remote bearer → valid DPoP bound to EKM → POST
/v1/relay/swap → 200 + faked upstream. Negatives: replayed jti → 401, revoked → 401, tampered → 401, no DPoP
header → 401. CI gates: no-c.sh (engine edit + edge deps), shape.sh (new module).

## Risks
EKM accessor through tokio_rustls 0.26 server stream — confirm against rustls 0.23 (context7) before wiring
(the one API to verify, not assume). EgressReq additive field blast radius low (compile error catches a miss).
Edge runs ON the reactor → await relay_swap normally (no spawn_blocking; proxy.rs does the same).

## Out of scope (record as follow-ups in wrap-up)
PR-2: server-issued nonce challenge (OI-SM-1 nonce half), per-IP/per-client rate-limit + body caps + timeouts
+ admission shedding (CVE-2024-47609), hardened-mode mTLS ClientCertVerifier (OI-SM-4). PR-3: streaming +
in-stream decide() re-check + tear-down on revoke/USB-pull (→ TASK-0032).
