# TASK-0031-PR2 — Hardening the F2 relay edge · VERDICT: GO

All four sub-features in one cycle, single repo (envctl, ~6 modules → sequential single-crew), ZERO new
deps. Sequenced as two coherent slices on the SAME branch: **PR-2a anti-abuse core** (nonce + admission/
rate-limit + body-caps/timeouts) then **PR-2b opt-in mTLS**. Nonce store + admission limiter are
engine-side security policy (siblings to `broker::jti::JtiReplayStore`); the edge does pure I/O.

## Scope for this cycle
IN: (1) server-issued DPoP-Nonce challenge (OI-SM-1 nonce half, RFC 9449 §8–9); (2) per-IP admission +
token-bucket rate-limit, shed BEFORE verify/decide (CVE-2024-47609, SERVER-MODE §6.2); (3) body caps +
handshake/header/idle/body timeouts; (4) opt-in hardened-mode mTLS `ClientCertVerifier` (OI-SM-4),
default-OFF. 2a lands first (request-pipeline + 2 engine stores), 2b second (tls.rs + EdgeConfig toggle).
Named deferrals (NOT silent drops): PROXY-protocol source-IP parsing → TASK-0031-PR2c; remote-clients-CA
lifecycle (mint/renew/revoke) → TASK-0033; group-commit audit batching + watch-push stream re-check → pre-existing.

## Target repos
envctl (single). engine: `broker/{nonce.rs(NEW),admission.rs(NEW),mod.rs}`; secretd:
`edge/{listener.rs,tls.rs,mod.rs,dpop.rs}` + new test files. ~6 modules → sequential single-crew.

## Engine API delta (all new security policy in engine, sync/non-printing/std+ring::rand-only)
### broker::nonce::NonceStore (NEW)
consts NONCE_TTL_MS=300_000 (coherent w/ ACCEPT_PAST_MS), MAX_NONCES=16_384 (~1MiB), NONCE_LEN=32.
`enum NonceReject{Missing,Unknown,Expired}`. `new()/with_params(ttl,max)`;
`issue(now_ms,&dyn ring::rand::SecureRandom)->Result<String,()>` (sweep-first; full-after-sweep→Err, caller
fails closed = 401 with no nonce); `check_and_consume(nonce,now_ms)->Result<(),NonceReject>` (single-use:
remove on accept). RNG injected as trait obj so engine stays pure/testable. **Single-use** recommended
(strongest replay posture; genuine retry re-challenges → fresh nonce → fresh jti, OI-SM-1 §6); windowed is
the one-line fallback (TTL without removal) if HTTP/2 coalescing flakes.
### broker::admission::AdmissionLimiter (NEW)
consts RATE_REFILL_PER_MIN=120, BUCKET_BURST=60, MAX_KEYS=65_536. `enum Admit{Allow,Throttled}`.
`admit(key,now_ms)->Admit` token-bucket: refill-by-elapsed → sweep idle → try-consume; full key table +
new key → Throttled (never grow, never evict-to-admit). Poisoned lock → caller 429.
Reexport both in broker/mod.rs + lib.rs (`pub use`). Optional metadata-only
`SecretEvent::EdgeRequestShed{reason,client_or_ip,count}` (recommended, like RelayStreamTornDown).
### dpop.rs delta
`VerifiedDpop.nonce: Option<String>` surfaced at parse (additive; verifier stays I/O-free, validation in
caller next to jti check so existing vector tests are unchanged).

## Edge changes (exact insertion points)
Current ladder (mod.rs:11 / listener.rs): route/method gate → htu → verify_remote_presentation{EKM,DPoP,
verify_dpop_proof, jti check_and_record, client_id, registry} → upstream → swap_and_respond_streaming.
**A. Admission (step 0, BEFORE any crypto):** add `admit: Arc<Mutex<AdmissionLimiter>>` + connection
`peer: SocketAddr` to ConnState (built in serve_edge_listener next to jti, listener.rs:78). In
handle_edge_request (listener.rs:169) right after route/method gate (after :180), before host/htu/verify:
`admit.lock()` (poisoned→bare(429)) + `admit(peer.ip(), now)`; Throttled→`bare(TOO_MANY_REQUESTS)`. Per-IP
only (client_id unauthenticated pre-verify); per-client quota stays in engine decide() clause 15
(rate_per_min) on accept. **B. Nonce (within verify_remote_presentation, listener.rs:297):** add
`nonce: Arc<Mutex<NonceStore>>` + `rng: Arc<ring::rand::SystemRandom>` to ConnState. After verify_dpop_proof
succeeds (:329) BEFORE jti check_and_record (:342): if verified.nonce None/unknown/expired → issue fresh +
return typed `Refusal::NonceChallenge(b64)`; present+valid → check_and_consume (single-use), Err→401. Change
verify_remote_presentation error type StatusCode → `enum Refusal{Status(StatusCode),NonceChallenge(String)}`.
New helper `challenge_nonce(nonce)->Response<ProxyBody>` beside `bare` (proxy.rs:625; bare emits no headers —
don't modify, add sibling) building 401 + `DPoP-Nonce: <b64>` + `WWW-Authenticate: DPoP error="use_dpop_nonce"`.
**C. Body caps + timeouts:** handshake timeout wraps acceptor.accept (listener.rs:120) in
tokio::time::timeout(HANDSHAKE_TIMEOUT)→drop on elapse; hyper auto Builder (listener.rs:159)
.http1().header_read_timeout(...); body cap via http_body_util::Limited::new(body,MAX_BODY_BYTES) before
swap consumes (listener.rs:229/:264)→413 on exceed; body-read timeout→408. consts HANDSHAKE_TIMEOUT=10s,
HEADER_READ_TIMEOUT=15s, IDLE_TIMEOUT=30s, MAX_BODY_BYTES=1MiB.

## PR-2b mTLS (tls.rs)
`RelayTlsConfig::load_from_dir_with_client_auth(relay_tls_dir, client_ca_path:Option<&Path>)`: when client-CA
configured, replace `.with_no_client_auth()` (tls.rs:86) with `.with_client_cert_verifier(v)` where
`v = WebPkiClientVerifier::builder_with_provider(roots, Arc::new(rustls::crypto::ring::default_provider())).build()?`
(rustls-webpki + rustls-pki-types already in Cargo.lock → zero new deps; explicit ring provider → ring-only).
roots = operator-provisioned remote-clients-CA PEM (SERVER-MODE §6.1 line 196) — NEVER the MITM CA, never the
server cert; distinct config input preserves FS-S25. EdgeConfig (mod.rs:31) gains
`require_client_cert: bool`(default false) + `client_ca_path: Option<PathBuf>`; serve_edge (mod.rs:54) threads
it. `require_client_cert && client_ca_path.is_none()` → fail-closed startup Err.

## Fail-closed matrix
missing nonce→401+DPoP-Nonce+WWW-Authenticate use_dpop_nonce · unknown/expired nonce→401 re-challenge ·
NonceStore full on issue→401 no nonce · poisoned NonceStore lock→401 · per-IP rate breach→429 · admission
key table full(new)→429 · poisoned admission lock→429 · body>MAX_BODY_BYTES→413 · header/idle/body timeout→408
· TLS handshake timeout→drop · mTLS required no client cert→handshake fail/drop · mTLS required no client_ca
path→startup Err · existing PR-1 bad DPoP/EKM/jti→401/403 unchanged. No unwrap/panic on request path; every
lock match→reject; no default-open.

## Dep decision (no-C proof)
rustls 0.23.40(ring), rustls-webpki, rustls-pki-types, ring, http-body-util(Limited), std/tokio::time — ALL
already in Cargo.lock. ZERO new crates. mTLS verifier built _with_provider(ring) → single ring-only rustls.
no-c.sh stays green.

## Tests
engine units (inject clock + seeded RNG, jti.rs pattern): nonce issue→consume; second consume→Unknown;
expired→Expired; missing→Missing; full→Err; sweep-then-issue; concurrent single-winner. admission: burst→
Throttled; refill; idle sweep; MAX_KEYS full→Throttled; concurrent. e2e (extend edge_e2e.rs; new
edge_hardening_e2e.rs; cfg relay-edge, reuse connect_and_ekm/make_proof, make_proof gains optional nonce arg):
nonce challenge→retry 200; stale-nonce→401; rate-limit→429 (assert shed BEFORE upstream recorder saw a key);
oversized-body→413; stalled-body→408 (small injected timeout override); mTLS require_client_cert=true: no
cert→handshake fail, valid cert(rcgen client-CA)→200. CI-tolerant (pure in-process, small injected params).
gates: no-c.sh, shape.sh (confirm mTLS verifier adds no banned import), fmt, clippy --workspace -Dwarnings,
test -p secrets-engine, test -p secretd --features relay-edge.

## Sequencing (leaf-first)
1 nonce.rs+tests+reexport. 2 admission.rs+tests+reexport. 3 dpop.rs surface nonce. 4 listener.rs PR-2a
(ConnState admit/nonce/rng/peer; admission step-0; nonce challenge; Refusal enum + challenge_nonce helper).
5 listener.rs body-caps/timeouts. 6 e2e PR-2a. 7 tls.rs+mod.rs PR-2b mTLS + fail-closed startup + e2e.
8 all gates + fmt/clippy; verify relay-edge-OFF build byte-for-byte unaffected.

## Invariants (each checkable)
1 no-C/one ring-only: zero new deps, mTLS _with_provider(ring), no-c.sh green. 2 engine single non-printing:
NonceStore/AdmissionLimiter sync std+ring::rand, typed rejects, no println!; edge does I/O. 3 decide() sole
Allow authority: admission/nonce only reject early; full verify+decide() run on every accepted req (test:
429'd req never reaches recording upstream); mTLS additive not replacement. 4 fail-closed/no panic: every
matrix row rejects; poisoned locks reject; no unwrap on req path; mTLS-misconfig→startup Err. 5 no secret in
logs: rejections metadata-only; nonces (non-secret) not logged needlessly. 6 relay-tls only/FS-S25/EKM:
client-CA separate input on SAME ServerConfig, no MITM-CA import, EKM untouched. 7 default-OFF relay-edge +
mTLS additionally opt-in (require_client_cert default false; with_no_client_auth byte-for-byte when off).

## Risks
body-cap wiring vs swap_and_respond_streaming body type (Incoming/ProxyBody, proxy.rs:47) — implementer
confirms non-breaking Limited wrap vs Content-Length guard; 413 e2e is the gate. verify_remote_presentation
return-type change (StatusCode→Refusal) touches one caller (listener.rs:192), low blast radius. HTTP/2
coalescing + single-use nonce: each req carries own proof+nonce → fine; windowed fallback is one-line.
context7 returned stale rustls 0.20 docs → WebPkiClientVerifier::builder_with_provider asserted from in-tree
rustls 0.23.40 usage (tls.rs:83) + lockfile; implementer MUST confirm exact builder method name vs rustls
0.23.40 rustdoc before finalizing PR-2b.

## Out of scope (named follow-ups)
TASK-0031-PR2c proxy-protocol-source-ip · TASK-0033 remote-clients-CA lifecycle (mint/renew/revoke) ·
group-commit audit batching + watch-push stream re-check (pre-existing deferrals).
