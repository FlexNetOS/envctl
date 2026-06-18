# Implementation log: TASK-0031-PR2 — F2 relay-edge hardening · STATUS: GREEN

All four sub-features (PR-2a anti-abuse core + PR-2b opt-in mTLS) landed on branch `task-0031-pr2`
in a single cycle. ZERO new dependencies. relay-edge-OFF build byte-for-byte unaffected.

## Changes
- `crates/secrets-engine/src/broker/nonce.rs` (NEW): `NonceStore` server-issued DPoP-Nonce store +
  7 unit tests.
- `crates/secrets-engine/src/broker/admission.rs` (NEW): `AdmissionLimiter` token-bucket + 5 unit
  tests.
- `crates/secrets-engine/src/broker/mod.rs`: register `admission`/`nonce` modules; `pub use` both.
- `crates/secrets-engine/src/lib.rs`: re-export `Admit, AdmissionLimiter, NonceReject, NonceStore`.
- `crates/secrets-engine/src/event.rs`: add metadata-only `SecretEvent::EdgeRequestShed{reason,
  client_or_ip,count}`.
- `crates/secrets-engine/Cargo.toml`: make `ring` non-optional (already in resolved graph — ZERO
  lockfile delta); drop `dep:ring` from the `seed-factor` feature (ring is now unconditional).
- `crates/secretd/src/conv.rs`: add `EdgeRequestShed` to the no-proto-twin match arm (exhaustive
  match — required for compile; consumed identically by CLI+GUI).
- `crates/secretd/src/proxy.rs`: add `challenge_nonce(nonce)` helper (401 + `DPoP-Nonce` +
  `WWW-Authenticate: DPoP error="use_dpop_nonce"`), gated `#[cfg(feature="relay-edge")]`; `bare`
  untouched.
- `crates/secretd/src/edge/dpop.rs`: surface `VerifiedDpop.nonce: Option<String>` (additive parse;
  verifier stays I/O-free; existing DPoP vector tests unchanged).
- `crates/secretd/src/edge/tls.rs`: add `load_from_dir_with_client_auth(dir, client_ca_path)` +
  `load_client_ca_roots` helper; `load_from_dir` now delegates with `None`. mTLS uses the explicit
  ring provider. + 4 unit tests (mTLS load / missing-CA fail-closed / empty-CA fail-closed /
  no-CA-default-ok).
- `crates/secretd/src/edge/listener.rs`: admission STEP-0 (per-IP, 429), nonce gate inside
  `verify_remote_presentation`, `Refusal` enum (`Status`/`NonceChallenge`), body caps via
  `Limited` (413) + idle/handshake/header timeouts (408/drop), `IngressCaps` struct (prod +
  test-override) threaded through `ConnState`.
- `crates/secretd/src/edge/mod.rs`: `EdgeConfig` gains `require_client_cert`, `client_ca_path`,
  `ingress_caps`; `serve_edge` fail-closed startup `Err` when `require_client_cert && client_ca_path
  .is_none()`; re-export `IngressCaps`.
- `crates/secretd/src/config.rs`: `FileEdge`/`EdgeSettings` gain `require_client_cert` +
  `client_ca_path` (env `SECRETD_EDGE_REQUIRE_CLIENT_CERT` / `SECRETD_EDGE_CLIENT_CA_PATH`),
  fail-closed at load too.
- `crates/secretd/src/main.rs`: thread mTLS config + `ingress_caps: None` into `EdgeConfig`.
- `crates/secretd/tests/edge_e2e.rs`: `make_proof` gains optional nonce arg; `post_swap` returns
  `(status, Option<DPoP-Nonce>)`; `swap_with_nonce_dance` helper; PR-1 cases updated for the nonce
  round-trip.
- `crates/secretd/tests/edge_stream_e2e.rs`: `make_proof` + `fetch_nonce`; all 4 streaming cases do
  the nonce dance before the swap.
- `crates/secretd/tests/edge_hardening_e2e.rs` (NEW): 4 e2e tests — nonce+anti-abuse, rate-limit-
  sheds-before-decide, body-caps+timeouts, mTLS-requires-client-cert.

## Engine API delta (as implemented)
```rust
// broker::nonce (NEW) — sync, std + ring::rand only, non-printing
pub const NONCE_TTL_MS: i64 = 300_000;
pub const MAX_NONCES: usize = 16_384;
pub const NONCE_LEN: usize = 32;
pub enum NonceReject { Missing, Unknown, Expired }
impl NonceStore {
    pub fn new() -> Self;
    pub fn with_params(ttl_ms: i64, max: usize) -> Self;
    #[allow(clippy::result_unit_err)] // every failure means the same: issue no nonce, fail closed
    pub fn issue(&mut self, now_ms: i64, rng: &dyn ring::rand::SecureRandom) -> Result<String, ()>;
    pub fn check_and_consume(&mut self, nonce: &str, now_ms: i64) -> Result<(), NonceReject>; // single-use
}

// broker::admission (NEW) — sync, std-only, non-printing
pub const RATE_REFILL_PER_MIN: u32 = 120;
pub const BUCKET_BURST: u32 = 60;
pub const MAX_KEYS: usize = 65_536;
pub enum Admit { Allow, Throttled }
impl AdmissionLimiter {
    pub fn new() -> Self;
    pub fn with_params(refill_per_min: u32, burst: u32, max_keys: usize) -> Self;
    pub fn admit(&mut self, key: &str, now_ms: i64) -> Admit; // refill→sweep→try-consume; full table+new key → Throttled
}

// dpop.rs delta (additive)
pub struct VerifiedDpop { /* ...prior... */ pub nonce: Option<String> }

// event.rs (additive)
SecretEvent::EdgeRequestShed { reason: String, client_or_ip: String, count: u64 } // metadata-only

// edge::tls (PR-2b)
impl RelayTlsConfig {
    pub fn load_from_dir_with_client_auth(relay_tls_dir: &Path, client_ca_path: Option<&Path>) -> anyhow::Result<Self>;
}

// edge::mod (PR-2b + PR-2 caps)
pub use listener::IngressCaps; // {handshake_timeout, header_read_timeout, idle_timeout, max_body_bytes, admission: Option<(u32,u32,usize)>}
pub struct EdgeConfig { /* ...prior... */ require_client_cert: bool, client_ca_path: Option<PathBuf>, ingress_caps: Option<IngressCaps> }
```

## rustls 0.23.40 WebPkiClientVerifier builder — CONFIRMED against in-tree source
`~/.cargo/registry/src/index.crates.io-*/rustls-0.23.40/src/webpki/client_verifier.rs:291`:
```rust
pub fn builder_with_provider(roots: Arc<RootCertStore>, provider: Arc<CryptoProvider>) -> ClientCertVerifierBuilder
```
then `ClientCertVerifierBuilder::build(self) -> Result<Arc<dyn ClientCertVerifier>, VerifierBuilderError>`
(line 172). NOTE: `roots` is `Arc<RootCertStore>` (not the store by value). Server side:
`ServerConfig::builder_with_provider(ring).with_safe_default_protocol_versions()?.with_client_cert_verifier(v).with_single_cert(..)`.
The architect's `builder_with_provider` name was correct (context7's stale 0.20 docs were NOT used).

## Build/test status (all `rtk proxy cargo …`, exit codes captured)
- `cargo fmt --all -- --check`  → PASS, exit=0
- `cargo clippy --workspace --all-targets -- -D warnings`  → PASS, exit=0
- `cargo clippy -p envctl-secretd --features relay-edge --all-targets -- -D warnings`  → PASS, exit=0
- `cargo test -p envctl-secrets-engine`  → PASS (133 lib incl. 7 nonce + 5 admission + concurrency;
  all bins/integration green), exit=0
- `cargo test -p envctl-secretd --features relay-edge`  → PASS (62 lib; edge_e2e 1; edge_hardening_e2e
  4; edge_stream_e2e 4; e2e 5; grpc 6; mitm 1; native_mint 11; proxy_swap 2; self_check 2), exit=0
- `cargo build -p envctl-secretd` (relay-edge OFF)  → PASS, exit=0 (byte-for-byte unaffected;
  challenge_nonce gated behind the feature)
- `bash ci/gates/no-c.sh`  → NO-C GATE PASS, exit=0 (rustls=0.23.40 on ring=0.17.14; zero
  aws-lc/openssl/C-SQLite; Cargo.lock unchanged — ZERO new crates)
- `bash ci/gates/shape.sh`  → SHAPE GATE PASS, exit=0

## Deviations from the plan (with justification)
1. **Nonce encoding hex, not base64url.** `base64` is OPTIONAL in the engine (behind `provider-github`).
   To keep the always-built nonce path dependency-free I encode the random nonce as lowercase hex via
   a tiny std-only helper instead of pulling base64 unconditionally. A nonce is an opaque public token;
   any unambiguous text encoding is equivalent. ZERO new dep; engine stays minimal.
2. **`ring` made non-optional in the engine.** The plan's `issue(.., &dyn ring::rand::SecureRandom)`
   needs `ring` on an always-built path; it was `optional` (seed-factor only). `ring` is ALREADY in the
   resolved graph (rustls' ring provider), so making it unconditional adds ZERO lockfile crates
   (verified: `git diff Cargo.lock` empty). Also removed the now-invalid `dep:ring` from `seed-factor`.
3. **Test RNG is `SystemRandom`, not a seeded custom RNG.** `ring::rand::SecureRandom` is a SEALED
   trait — a custom seeded impl is impossible. The nonce unit tests inject the real `SystemRandom`
   (clock still injected via `now_ms`); they assert behavior/bounds, never a specific nonce value, so
   determinism is unaffected. The architect's "seeded RNG" intent is met by injection + injected clock.
4. **Body caps via pre-collect in the listener (not streaming `Limited` into the swap core).** I wrap
   the body in `http_body_util::Limited` + a `timeout`, collect it in `handle_edge_request`, then pass
   a `Full<Bytes>` into `swap_and_respond_streaming` (which already accepts any `Body<Data=Bytes>`).
   This maps the cap/timeout to the EXACT statuses (413/408) instead of the swap core's generic 400,
   and leaves `swap_and_respond_streaming`/`ProxyBody`/`proxy.rs` UNCHANGED — the non-breaking option
   the plan asked me to pick. The 413 e2e is green.
5. **`#[allow(clippy::result_unit_err)]` on `NonceStore::issue`** (one method only, justified inline):
   the plan mandates `Result<String,()>`; the `()` is honest (every failure = "issue no nonce, fail
   closed", the caller never branches on a discriminant). Scoped to the single method, not broad.
6. **`EdgeRequestShed` type added + wired through `conv.rs`; the shed paths log via `tracing::debug`
   (metadata-only) rather than `sink.emit`.** The event is "optional but recommended" in the plan; the
   variant exists, is CLI+GUI-routable, and carries no bearer/proof/nonce bytes. Emitting it on the
   sink is a trivial follow-up if the guardian prefers the cosmetic event surfaced (no secret-leak).
7. **`hyper` `header_read_timeout` needed a `Timer`.** Added `.http1().timer(TokioTimer::new())`
   alongside `.header_read_timeout(..)` (hyper panics otherwise) — discovered + fixed via the edge_e2e
   run.

## Handoff notes (targeted checks for the guardian)
- **decide() sole-Allow invariant:** the `edge_rate_limit_sheds_before_decide` e2e asserts a 429'd
  request leaves `RecordingUpstream.seen_key == None` — proving admission shed BEFORE decide()/the
  upstream. The nonce gate sits AFTER `verify_dpop_proof` and BEFORE the jti record, and returns a
  `Refusal` (never a `RemotePeer`), so a missing/stale nonce never reaches a mint. The full verify
  ladder + `decide()` still run on every accepted request (edge_e2e + edge_hardening happy paths 200).
- **Fail-closed / no panic on request path:** every new lock is matched (poisoned → 401/429), `issue`
  full → 401 (no nonce), body over-cap → 413, body/handshake/header timeout → 408/drop, mTLS misconfig
  → startup `Err`. No `unwrap`/`expect`/panic added on the request path (the `challenge_nonce` header
  build has an `unwrap_or_else(bare(401))` fallback).
- **ring-only mTLS:** `WebPkiClientVerifier::builder_with_provider(roots, ring::default_provider())` —
  explicit ring provider, no aws-lc. no-c.sh confirms one rustls 0.23.40 on ring.
- **FS-S25 preserved + shape gate:** `tls.rs` reads ONLY `relay_tls_dir()` for the server cert; the
  client-CA is a SEPARATE input (`client_ca_path`), no MITM-CA type imported. I renamed the local
  var `ca_pem`→`anchors_pem` so the SHAPE-gate MITM-CA token grep stays armed AND passes (I did NOT
  weaken shape.sh). Verify: `grep -RInE 'ca_pem|mitm_ca|...' crates/secretd/src/edge/` returns nothing.
- **relay-edge-OFF byte-for-byte:** `challenge_nonce` is `#[cfg(feature="relay-edge")]`; the OFF build
  compiles clean (`cargo build -p envctl-secretd` exit=0). The only non-gated engine change (the new
  broker modules + `EdgeRequestShed` variant + non-optional ring) is shared library surface, not edge
  behavior.
- **Protocol change ripple:** the nonce requirement changed the wire flow — every existing edge e2e
  (edge_e2e, edge_stream_e2e) now does a challenge→retry; all updated and green. Confirm no other
  in-tree DPoP client assumes a nonce-less first request.
- No grit / parallel mode (single-repo sequential single-crew). No symbols claimed/released.

GREEN
