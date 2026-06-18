//! The F2 remote relay-edge accept loop (TASK-0031 PR-1, hardened by PR-2). Binds a TCP listener,
//! terminates inbound TLS IN-PROCESS with [`RelayTlsConfig`] (FS-S20 — no external TLS-terminating
//! front), computes the RFC 5705 exported keying material (EKM) off the terminated rustls server
//! stream, and serves hyper HTTP/1.1+2. The ONLY route is `POST /v1/relay/swap`.
//!
//! Per request the order is fail-closed (SERVER-MODE §6.4):
//! 0. **Admission (PR-2)** — a per-source-IP token-bucket SHED, BEFORE any crypto, so a flood cannot
//!    burn signature-verification or vault work (CVE-2024-47609). `Throttled ⇒ 429`. This can only
//!    reject early; it NEVER substitutes for the verify ladder / `decide()`.
//! 1. route/method gate → htu → [`verify_remote_presentation`] which runs EKM (FS-S20) → DPoP
//!    ([`verify_dpop_proof`]) → **DPoP-Nonce (PR-2)**: a missing/unknown/expired nonce ⇒ a fresh
//!    server-issued nonce challenge (`401 + DPoP-Nonce` — RFC 9449 §8–9); a present+valid nonce is
//!    consumed single-use → jti `check_and_record` → registry/jkt bind → `RemotePeer`.
//! 2. **Body caps + timeouts (PR-2)** — handshake/header/idle/body timeouts (drop / 408) and a
//!    `MAX_BODY_BYTES` cap (`413`) before the swap consumes the body.
//! 3. `proxy::swap_and_respond_streaming(.., remote: Some(rp))` (the SAME swap core as the local
//!    proxy, so MINT + DECIDE stay in the engine). `decide()` remains the SOLE Allow authority.
//!
//! Any verification failure short-circuits to 401/403/408/413/429 and the request NEVER reaches a
//! mint. NO secret bytes are logged: tracing carries only `client_id` / a decision label / a status,
//! never the bearer / proof / EKM / key / nonce / body. The poisoned-mutex path is a reject.

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use envctl_secrets::broker::admission::{AdmissionLimiter, Admit};
use envctl_secrets::broker::decide::RemotePeer;
use envctl_secrets::broker::jti::JtiReplayStore;
use envctl_secrets::broker::nonce::NonceStore;
use envctl_secrets::Engine;
use http_body_util::{BodyExt, Full, Limited};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::{TokioExecutor, TokioIo};

use crate::edge::dpop::{verify_dpop_proof, DpopReject, HttpMethod};
use crate::edge::tls::RelayTlsConfig;
use crate::proxy::{
    bare, challenge_nonce, extract_bearer, method_from_hyper, request_host,
    swap_and_respond_streaming, ProxyCtx,
};

/// The only route the edge serves.
const SWAP_PATH: &str = "/v1/relay/swap";

/// The RFC 5705 exporter label the edge binds the DPoP proof against. A fixed, application-specific
/// label (RFC 5705 §4 disambiguation) so the value is unique to the envctl relay-DPoP binding.
const EKM_LABEL: &[u8] = b"EXPORTER-envctl-relay-dpop-v1";

/// Exported-keying-material length (bytes). 32 bytes = 256 bits, ample to bind a channel.
const EKM_LEN: usize = 32;

/// PR-2 ingress hardening caps. A connection whose TLS handshake does not complete within
/// `HANDSHAKE_TIMEOUT` is DROPPED (no plaintext read). After the handshake, hyper enforces a
/// header-read timeout (`HEADER_READ_TIMEOUT`); a request whose BODY does not arrive within
/// `IDLE_TIMEOUT` is a `408`; a body exceeding `MAX_BODY_BYTES` is a `413`. All are slow-loris /
/// resource-exhaustion backstops (SERVER-MODE §6.2). Overridable per-edge for the e2e (small values
/// so a test exercises the 408/413 path without sleeping the production interval).
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);
const HEADER_READ_TIMEOUT: Duration = Duration::from_secs(15);
const IDLE_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_BODY_BYTES: usize = 1024 * 1024;

/// Tunable ingress caps (PR-2). The daemon always uses [`IngressCaps::production`]; the e2e injects
/// small values so the 408/413/429 paths are exercised quickly. Cloned into every connection's state.
#[derive(Clone, Copy, Debug)]
pub struct IngressCaps {
    pub handshake_timeout: Duration,
    pub header_read_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_body_bytes: usize,
    /// Optional per-IP admission token-bucket override `(refill_per_min, burst, max_keys)`. `None` ⇒
    /// the audited [`AdmissionLimiter::new`] defaults. A test sets `burst = 1` so a single extra
    /// request is shed (429) without sending the production `BUCKET_BURST`.
    pub admission: Option<(u32, u32, usize)>,
}

impl IngressCaps {
    /// The production caps (the audited PR-2 defaults).
    pub fn production() -> Self {
        Self {
            handshake_timeout: HANDSHAKE_TIMEOUT,
            header_read_timeout: HEADER_READ_TIMEOUT,
            idle_timeout: IDLE_TIMEOUT,
            max_body_bytes: MAX_BODY_BYTES,
            admission: None,
        }
    }
}

impl Default for IngressCaps {
    fn default() -> Self {
        Self::production()
    }
}

/// The fail-closed reason `verify_remote_presentation` returns: either a plain status to emit, or a
/// fresh DPoP-Nonce challenge (the caller renders it as `401 + DPoP-Nonce` via
/// [`crate::proxy::challenge_nonce`]). Both are rejections — neither reaches a mint.
enum Refusal {
    /// Emit this status as a bare response (the PR-1 401/403/400 behavior).
    Status(StatusCode),
    /// Issue this fresh nonce as a `401 + DPoP-Nonce` challenge (RFC 9449 §8–9).
    NonceChallenge(String),
}

/// Per-connection edge state shared by every request on one TLS connection.
#[derive(Clone)]
struct ConnState {
    ctx: ProxyCtx,
    /// The edge-owned replay store (one `Mutex<JtiReplayStore>` for the whole edge — atomicity §7).
    jti: Arc<Mutex<JtiReplayStore>>,
    /// PR-2: the edge-owned per-IP admission limiter (one for the whole edge).
    admit: Arc<Mutex<AdmissionLimiter>>,
    /// PR-2: the edge-owned server-issued DPoP-Nonce store (one for the whole edge).
    nonce: Arc<Mutex<NonceStore>>,
    /// PR-2: a shared system RNG used to mint fresh nonces (cheap to clone the `Arc`).
    rng: Arc<ring::rand::SystemRandom>,
    /// `Some(ekm)` once the handshake EKM was exported; `None` if uncomputable (FS-S20 → 403).
    ekm: Option<Arc<Vec<u8>>>,
    /// TASK-0032 / FS-S5: the streaming re-check cadence + lifetime cap for every stream served on
    /// this connection (production default unless a test override was configured).
    timing: crate::edge::stream::Timing,
    /// PR-2: the connection's source address — keyed by IP for admission. Per-IP only (client_id is
    /// unauthenticated pre-verify; the per-client quota stays in engine `decide()` on the accept path).
    peer: SocketAddr,
    /// PR-2: ingress caps (body size + idle/body timeouts) for this connection.
    caps: IngressCaps,
}

/// Bind the edge TCP listener on `bind_addr`, build the in-process TLS acceptor from the relay-tls
/// cert (PR-2b: optionally requiring a client cert against a configured client-CA), and serve
/// `POST /v1/relay/swap` until `shutdown` resolves. Returns the bound address + the serving task
/// handle. A cert-load / mTLS-misconfig / bind failure is an `Err` (the caller makes it FATAL when
/// the edge is explicitly enabled — fail-closed).
#[allow(clippy::too_many_arguments)]
pub async fn serve_edge_listener(
    engine: Engine,
    relay_tls_dir: &std::path::Path,
    bind_addr: SocketAddr,
    timing: crate::edge::stream::Timing,
    caps: IngressCaps,
    client_ca_path: Option<&std::path::Path>,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) -> anyhow::Result<(SocketAddr, tokio::task::JoinHandle<()>)> {
    // Load the relay-tls ServerConfig FIRST (fail-closed: no cert ⇒ no edge). This is the ONLY cert
    // source — never the MITM CA (FS-S25, structural in `tls.rs`). PR-2b: when a client-CA bundle is
    // configured, the same relay-tls ServerConfig additionally requires a verified client cert (mTLS).
    let tls = RelayTlsConfig::load_from_dir_with_client_auth(relay_tls_dir, client_ca_path)?;
    let acceptor = tokio_rustls::TlsAcceptor::from(tls.server_config());

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    let local_addr = listener.local_addr()?;

    // One replay store, one admission limiter, one nonce store, and one RNG for the whole edge
    // (per-process; bounded — OI-SM-1 §4/§5, admission MAX_KEYS, nonce MAX_NONCES). The admission
    // limiter honors a test override (small burst) when configured; the daemon always uses defaults.
    let jti = Arc::new(Mutex::new(JtiReplayStore::new()));
    let admit = Arc::new(Mutex::new(match caps.admission {
        Some((refill, burst, max_keys)) => AdmissionLimiter::with_params(refill, burst, max_keys),
        None => AdmissionLimiter::new(),
    }));
    let nonce = Arc::new(Mutex::new(NonceStore::new()));
    let rng = Arc::new(ring::rand::SystemRandom::new());
    let ctx = ProxyCtx::for_edge(engine);

    let handle = tokio::spawn(async move {
        tokio::pin!(shutdown);
        loop {
            tokio::select! {
                _ = &mut shutdown => {
                    tracing::info!("relay edge shutting down");
                    break;
                }
                accept = listener.accept() => {
                    let (tcp, peer) = match accept {
                        Ok(p) => p,
                        Err(e) => {
                            tracing::warn!(error = %e, "relay edge accept failed");
                            continue;
                        }
                    };
                    let acceptor = acceptor.clone();
                    let ctx = ctx.clone();
                    let jti = Arc::clone(&jti);
                    let admit = Arc::clone(&admit);
                    let nonce = Arc::clone(&nonce);
                    let rng = Arc::clone(&rng);
                    tokio::spawn(async move {
                        serve_connection(
                            acceptor, ctx, jti, admit, nonce, rng, timing, caps, tcp, peer,
                        )
                        .await;
                    });
                }
            }
        }
    });

    Ok((local_addr, handle))
}

/// Terminate TLS on one accepted TCP stream, export the connection EKM, and serve its requests.
#[allow(clippy::too_many_arguments)]
async fn serve_connection(
    acceptor: tokio_rustls::TlsAcceptor,
    ctx: ProxyCtx,
    jti: Arc<Mutex<JtiReplayStore>>,
    admit: Arc<Mutex<AdmissionLimiter>>,
    nonce: Arc<Mutex<NonceStore>>,
    rng: Arc<ring::rand::SystemRandom>,
    timing: crate::edge::stream::Timing,
    caps: IngressCaps,
    tcp: tokio::net::TcpStream,
    peer: SocketAddr,
) {
    // PR-2: bound the TLS handshake — a peer that opens a TCP connection and never (or slowly)
    // completes the handshake is DROPPED on elapse (no plaintext is ever read; no key material is in
    // scope). A slow-loris backstop at the very front of the pipeline.
    let tls_stream = match tokio::time::timeout(caps.handshake_timeout, acceptor.accept(tcp)).await
    {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => {
            // A handshake failure (no client trust — PR-2b mTLS, bad ALPN, etc.) ends the connection
            // with no plaintext ever read. No key material is in scope.
            tracing::debug!(peer = %peer, error = %e, "relay edge TLS handshake failed");
            return;
        }
        Err(_) => {
            tracing::debug!(peer = %peer, "relay edge TLS handshake timed out (dropped)");
            return;
        }
    };

    // Compute the RFC 5705 EKM off the TERMINATED server stream (FS-S20). The accessor is, on
    // tokio-rustls 0.26's post-handshake `server::TlsStream`, `get_ref().1` → `&ServerConnection`,
    // which derefs to `ConnectionCommon` exposing
    // `export_keying_material::<T: AsMut<[u8]>>(output, label, context) -> Result<T, Error>`
    // (rustls 0.23.40 conn.rs:460 — confirmed against the pinned source). `None` ⇒ uncomputable ⇒
    // every request on this connection is refused 403 (binding fail-closed).
    let ekm: Option<Arc<Vec<u8>>> = {
        let (_, server_conn) = tls_stream.get_ref();
        let out = [0u8; EKM_LEN];
        match server_conn.export_keying_material(out, EKM_LABEL, None) {
            Ok(buf) => Some(Arc::new(buf.to_vec())),
            Err(e) => {
                tracing::warn!(peer = %peer, error = %e, "relay edge could not export keying material (binding fail-closed)");
                None
            }
        }
    };

    let conn = ConnState {
        ctx,
        jti,
        admit,
        nonce,
        rng,
        ekm,
        timing,
        peer,
        caps,
    };
    let service = service_fn(move |req| {
        let conn = conn.clone();
        async move { Ok::<_, std::convert::Infallible>(handle_edge_request(conn, req).await) }
    });

    // PR-2: a header-read timeout (slow-loris on the request HEAD) via the hyper HTTP/1 auto Builder.
    // `header_read_timeout` REQUIRES a `Timer` on the http1 sub-builder (hyper panics otherwise), so a
    // `TokioTimer` is installed alongside it. `caps` is `Copy`, so reading it here after `conn` (which
    // holds its own copy) moved into the service is fine. The header-read timeout is the only cap
    // consumed at the hyper-Builder layer; the body-size + body-read (idle) caps run per-request
    // inside `handle_edge_request`.
    let mut builder = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new());
    builder
        .http1()
        .timer(hyper_util::rt::TokioTimer::new())
        .header_read_timeout(caps.header_read_timeout);
    if let Err(e) = builder
        .serve_connection(TokioIo::new(tls_stream), service)
        .await
    {
        tracing::debug!(peer = %peer, error = %e, "relay edge connection ended");
    }
}

/// Handle one decrypted edge request. Route is fixed to `POST /v1/relay/swap`; everything else is a
/// bare 404/405. The fail-closed verification ladder runs before the swap core is reached.
async fn handle_edge_request(
    conn: ConnState,
    req: Request<Incoming>,
) -> Response<crate::proxy::ProxyBody> {
    // Route + method gate.
    let edge_path = req.uri().path().to_string();
    if edge_path != SWAP_PATH {
        return bare(StatusCode::NOT_FOUND);
    }
    if req.method() != hyper::Method::POST {
        return bare(StatusCode::METHOD_NOT_ALLOWED);
    }

    // ---- PR-2 STEP 0: admission (BEFORE any crypto/verify/decide) -----------------------------
    // Per-source-IP token bucket. A poisoned lock ⇒ 429 (fail-closed, never bypass). `Throttled` ⇒
    // 429 and the request is SHED here — it NEVER reaches the verify ladder / a mint / `decide()`.
    {
        let now_ms = chrono::Utc::now().timestamp_millis();
        let admit = {
            let mut guard = match conn.admit.lock() {
                Ok(g) => g,
                Err(_) => return bare(StatusCode::TOO_MANY_REQUESTS),
            };
            guard.admit(&conn.peer.ip().to_string(), now_ms)
        };
        if admit == Admit::Throttled {
            // Metadata-only: never the bearer/proof/IP-as-secret; the source IP is operational, not
            // secret. A shed here proves the verify ladder / recording upstream was NEVER reached.
            tracing::debug!(peer = %conn.peer, "relay edge request shed (rate limited)");
            return bare(StatusCode::TOO_MANY_REQUESTS);
        }
    }

    // The canonical htu the DPoP proof must bind: scheme+host+path (NO query). The edge host comes
    // from the request's Host header (the remote client addressed the edge). A missing Host ⇒ 400.
    let edge_host = match request_host(&req) {
        Some(h) => h,
        None => return bare(StatusCode::BAD_REQUEST),
    };
    let htu = format!("https://{edge_host}{SWAP_PATH}");

    // The verified remote presentation context, or a refusal — built BEFORE the swap so a failure
    // NEVER reaches a mint. PR-2: a missing/stale nonce is rendered as a 401 + DPoP-Nonce challenge.
    let rp = match verify_remote_presentation(&conn, &req, &htu) {
        Ok(rp) => rp,
        Err(Refusal::Status(status)) => {
            // Metadata-only: the status code, never the bearer/proof/EKM. A refusal here NEVER reached
            // a mint (it short-circuits before the swap core).
            tracing::debug!(status = %status.as_u16(), "relay edge presentation refused");
            return bare(status);
        }
        Err(Refusal::NonceChallenge(nonce)) => {
            // RFC 9449 §8–9: hand the client a fresh server-issued nonce so it can retry with the
            // nonce echoed in its next proof. The nonce is public; never log the bearer/proof.
            tracing::debug!("relay edge issued a fresh DPoP-Nonce challenge");
            return challenge_nonce(&nonce);
        }
    };

    // The UPSTREAM target the swap fences against. For PR-1 the remote client conveys it as an
    // `X-Relay-Upstream-Host` header (the only routing knob); the upstream method/path/body are the
    // request's own. `relay_swap`'s `decide()` re-fences host/path/method against the policy allowlist,
    // so a forged/unallowed target is denied there — the edge does NOT enforce policy (engine owns it).
    let upstream_host = match req
        .headers()
        .get("x-relay-upstream-host")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
    {
        Some(h) if !h.is_empty() => h,
        _ => return bare(StatusCode::BAD_REQUEST),
    };
    let method = match method_from_hyper(req.method()) {
        Some(m) => m,
        None => return bare(StatusCode::METHOD_NOT_ALLOWED),
    };
    // The edge route is fixed (`/v1/relay/swap`); the UPSTREAM path rides a header so the engine's
    // `decide()` can fence it against the policy's `path_allow`. Default `/` when unset.
    let upstream_path = req
        .headers()
        .get("x-relay-upstream-path")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "/".to_string());

    let (parts, body) = req.into_parts();

    // ---- PR-2 STEP 2: body caps + body-read timeout (BEFORE the swap consumes the body) -------
    // Cap the inbound body with `http_body_util::Limited` (→ 413 on exceed) and bound the body read
    // with the idle timeout (→ 408 on a stalled body). We collect HERE rather than streaming the raw
    // `Incoming` into the swap core so the cap/timeout map to the exact statuses (413/408) instead of
    // the swap core's generic 400; the collected bytes are then handed on as a `Full<Bytes>` body
    // (same `Body<Data = Bytes>` contract `swap_and_respond_streaming` already accepts — no body-type
    // break to `swap_and_respond_streaming`/`ProxyBody`).
    let limited = Limited::new(body, conn.caps.max_body_bytes);
    let collected = match tokio::time::timeout(conn.caps.idle_timeout, limited.collect()).await {
        Ok(Ok(c)) => c.to_bytes(),
        Ok(Err(_)) => {
            // The only error `Limited` raises on a swap-sized POST is the length cap (`LengthLimitError`)
            // — anything else is a truncated/aborted body; both are over-budget / malformed ingress.
            // Fail-closed to 413 (request entity too large).
            tracing::debug!("relay edge request body exceeded the cap (413)");
            return bare(StatusCode::PAYLOAD_TOO_LARGE);
        }
        Err(_) => {
            tracing::debug!("relay edge request body read timed out (408)");
            return bare(StatusCode::REQUEST_TIMEOUT);
        }
    };
    let body = Full::new(collected);

    // TASK-0032 / FS-S5 — build the in-stream re-check plan from THIS request's verified context so a
    // long-lived response stream is actively torn down the instant authorization lapses (relay/bearer
    // revoke, vault lock, USB-key pull, or the max-duration cap). The plan captures:
    //   * the relay BEARER (extracted from the SAME headers `swap_and_respond` reads — Bearer-scheme
    //     on the remote plane), as a `Zeroizing<String>` so it is wiped on drop;
    //   * a ZERO-byte re-check `EgressReq` carrying the SAME `RemotePeer` (`rp`) captured at open, so
    //     `decide()` clause 11a re-asserts dpop_verified + the client_id/jkt binding each tick;
    //   * the metadata-only audit labels (the upstream host as the relay label + the PUBLIC token_id).
    // The engine remains the SOLE policy authority — the supervisor only forwards/re-checks/drops.
    let bearer_for_recheck = extract_bearer(&parts.headers, envctl_secrets::Provider::Generic)
        .map(zeroize::Zeroizing::new);
    let token_id = bearer_for_recheck
        .as_deref()
        .and_then(|b| envctl_secrets::broker::parse_bearer(b).map(|(tid, _)| tid.to_string()))
        .unwrap_or_default();
    let recheck_req = crate::edge::stream::recheck_egress_req(
        method,
        upstream_host.clone(),
        upstream_path.clone(),
        Some(rp.clone()),
    );
    let audit = crate::edge::stream::StreamAudit {
        relay: upstream_host.clone(),
        token_id,
    };
    let engine = conn.ctx.engine.clone();
    let timing = conn.timing;

    // Hand to the SHARED swap core with the verified remote context. `decide()` is the SOLE Allow
    // authority — admission + nonce only rejected early; the full verify ladder ran above and the swap
    // core still runs MINT + DECIDE in the engine. The provider is derived from the upstream host
    // INSIDE the core (same as the proxy), and the bearer is extracted there from the forwarded
    // headers exactly as the local plane does — so the edge and proxy never diverge in how they drive
    // `relay_swap`. The remote edge additionally wraps the response body in the streaming-revocation
    // supervisor; the local proxy passes the identity wrap (no in-stream re-check).
    swap_and_respond_streaming(
        &conn.ctx,
        method,
        upstream_host,
        upstream_path,
        &parts.headers,
        body,
        // No MITM SNI on the remote plane.
        None,
        Some(rp),
        move |upstream_rx| {
            // Only reached on an `Allowed` swap. If the bearer somehow could not be re-extracted (it
            // was present — the swap allowed — but be fail-safe), forward verbatim rather than panic;
            // a missing bearer would itself tear down on the first tick anyway.
            let Some(bearer) = bearer_for_recheck else {
                return upstream_rx;
            };
            crate::edge::stream::relay_stream_response(
                upstream_rx,
                engine,
                bearer,
                recheck_req,
                audit,
                envctl_secrets::EventSink::null(),
                timing,
            )
        },
    )
    .await
}

/// The fail-closed verification ladder. Returns the verified [`RemotePeer`] or a [`Refusal`] (a
/// status to emit, or a fresh DPoP-Nonce challenge). NEVER reaches a mint on any error. The
/// bearer/proof/EKM/nonce never enter a log line.
fn verify_remote_presentation(
    conn: &ConnState,
    req: &Request<Incoming>,
    htu: &str,
) -> Result<RemotePeer, Refusal> {
    // (0) EKM must have been computed (FS-S20). Uncomputable ⇒ 403.
    let ekm = conn.ekm.as_deref().map(|v| v.as_slice());

    // (1) The DPoP proof header.
    let dpop = match req
        .headers()
        .get("dpop")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
    {
        Some(d) if !d.is_empty() => d,
        // No proof ⇒ 401 (RFC 9449: a sender-constrained bearer requires a proof).
        _ => return Err(Refusal::Status(StatusCode::UNAUTHORIZED)),
    };

    // (2) The relay bearer must be PRESENT before we do any DPoP crypto (else `relay_swap` would
    // refuse `UnknownBearer` anyway — short-circuit to 401). The bearer rides `Authorization: Bearer`
    // (relay bearers are Bearer-scheme regardless of upstream provider). `swap_and_respond` re-extracts
    // it from the SAME headers, so we only confirm presence here (no need to thread the value through).
    if extract_bearer(req.headers(), envctl_secrets::Provider::Generic).is_none() {
        return Err(Refusal::Status(StatusCode::UNAUTHORIZED));
    }

    // (3) Verify the DPoP proof (RFC 9449) bound to THIS connection's EKM (FS-S20). `htu` binds the
    // EDGE URL the client addressed. Method is POST (already gated). `now_ms` is the wall clock — the
    // ONE clock read in the verify path, supplied here (the verifier itself does no I/O).
    let now_ms = chrono::Utc::now().timestamp_millis();
    let verified = match verify_dpop_proof(&dpop, HttpMethod::Post, htu, ekm, now_ms) {
        Ok(v) => v,
        Err(DpopReject::EkmUncomputable)
        | Err(DpopReject::EkmMismatch)
        | Err(DpopReject::EkmMissing) => {
            // Channel-binding failures are 403 (FS-S20 fail-closed), distinct from a proof-format 401.
            return Err(Refusal::Status(StatusCode::FORBIDDEN));
        }
        Err(_) => return Err(Refusal::Status(StatusCode::UNAUTHORIZED)),
    };

    // (3b) PR-2: server-issued DPoP-Nonce (RFC 9449 §8–9). AFTER the proof verifies (so we never mint
    // a nonce for an unauthenticated proof) and BEFORE the jti record (so a re-challenge does not burn
    // a jti). A missing/unknown/expired nonce ⇒ issue a FRESH one and return a 401 challenge; a
    // present+valid nonce is consumed SINGLE-USE (a poisoned lock or a full store ⇒ a 401, fail-closed
    // — never an accept). The nonce store is edge-owned (`Mutex`), atomic with issue/consume.
    {
        let mut guard = match conn.nonce.lock() {
            Ok(g) => g,
            // Poisoned mutex ⇒ fail-closed reject (never bypass). No nonce to offer ⇒ bare 401.
            Err(_) => return Err(Refusal::Status(StatusCode::UNAUTHORIZED)),
        };
        match verified.nonce.as_deref() {
            // Present: must be a still-live nonce we issued; consume it single-use.
            Some(n) if !n.is_empty() => {
                if guard.check_and_consume(n, now_ms).is_err() {
                    // Unknown/expired/missing ⇒ re-challenge with a fresh nonce (so a genuine retry
                    // succeeds, while a replayed/stale nonce never accepts).
                    match guard.issue(now_ms, conn.rng.as_ref()) {
                        Ok(fresh) => return Err(Refusal::NonceChallenge(fresh)),
                        // Store full ⇒ fail-closed 401 with NO nonce (never accept-on-error).
                        Err(()) => return Err(Refusal::Status(StatusCode::UNAUTHORIZED)),
                    }
                }
            }
            // Absent: issue the first challenge so the client retries with a server-issued nonce.
            _ => match guard.issue(now_ms, conn.rng.as_ref()) {
                Ok(fresh) => return Err(Refusal::NonceChallenge(fresh)),
                Err(()) => return Err(Refusal::Status(StatusCode::UNAUTHORIZED)),
            },
        }
    }

    // (4) Replay store: `check_and_record` under the edge-owned Mutex. A poisoned lock ⇒ reject
    // (NEVER unwrap), an Err (replay / drift / full) ⇒ 401.
    {
        let mut guard = match conn.jti.lock() {
            Ok(g) => g,
            // Poisoned mutex ⇒ fail-closed reject (never bypass).
            Err(_) => return Err(Refusal::Status(StatusCode::UNAUTHORIZED)),
        };
        if guard
            .check_and_record(
                identity_for_jti(&verified),
                &verified.jti,
                verified.iat_ms,
                now_ms,
            )
            .is_err()
        {
            return Err(Refusal::Status(StatusCode::UNAUTHORIZED));
        }
    }

    // (5) The bearer carries the bound `client_id`; but PR-1 derives identity from the DPoP `jkt`
    // matched against the registry. The remote client is identified by the `client_id` the proof
    // asserts (cross-checked against the registered jkt). Resolve it from the proof's client_id claim
    // (the bearer's own client_id is authenticated by decide()'s clause 11a binding). A proof with no
    // client_id claim cannot be edge-registry-checked ⇒ 401.
    let client_id = match &verified.client_id {
        Some(c) if !c.is_empty() => c.clone(),
        _ => return Err(Refusal::Status(StatusCode::UNAUTHORIZED)),
    };

    // (6) Registry lookup BEFORE decide() (mirrors UnknownBearer pre-decide raise). Unknown/revoked ⇒
    // 401. A store error ⇒ fail-closed 401 (treat as a refusal, never an accept). Also assert the
    // proven jkt matches the registered jkt (RemoteBindingMismatch defense at the edge).
    match conn.ctx.engine.load_remote_client(&client_id) {
        Ok(Some(c)) if c.enabled && c.revoked_at_ms.is_none() => {
            if c.dpop_jkt != verified.jkt {
                // The proven key does not match the registered binding ⇒ 401.
                return Err(Refusal::Status(StatusCode::UNAUTHORIZED));
            }
        }
        // Known but disabled/revoked ⇒ 401; unknown ⇒ 401; store error ⇒ fail-closed 401.
        Ok(Some(_)) | Ok(None) | Err(_) => return Err(Refusal::Status(StatusCode::UNAUTHORIZED)),
    }

    // All checks passed: the proof is verified and bound, the nonce was fresh + consumed, the jti is
    // fresh, the client is registered + enabled, and the proven jkt matches the registration.
    // decide()'s clause 11a re-asserts everything fail-closed once more inside the engine.
    Ok(RemotePeer {
        client_id,
        dpop_jkt: verified.jkt,
        dpop_verified: true,
    })
}

/// The replay-store identity key: the proof's client_id (already required non-empty by the caller).
/// Kept as a tiny helper so the `&str` borrow is explicit at the call site.
fn identity_for_jti(v: &crate::edge::dpop::VerifiedDpop) -> &str {
    v.client_id.as_deref().unwrap_or("")
}
