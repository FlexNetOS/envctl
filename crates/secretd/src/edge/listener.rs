//! The F2 remote relay-edge accept loop (TASK-0031 PR-1). Binds a TCP listener, terminates inbound
//! TLS IN-PROCESS with [`RelayTlsConfig`] (FS-S20 — no external TLS-terminating front), computes the
//! RFC 5705 exported keying material (EKM) off the terminated rustls server stream, and serves hyper
//! HTTP/1.1+2. The ONLY route is `POST /v1/relay/swap`.
//!
//! Per request the order is fail-closed (SERVER-MODE §6.4): extract the bearer + `DPoP` header →
//! [`verify_dpop_proof`] (passing the connection EKM — FS-S20) → `JtiReplayStore::check_and_record`
//! under an edge-owned `Mutex` (poisoned lock ⇒ reject, NEVER unwrap) → `Engine::load_remote_client`
//! (unknown/revoked ⇒ 401 BEFORE decide) → build `RemotePeer{dpop_verified:true}` →
//! `proxy::swap_and_respond(.., remote: Some(rp))` (the SAME swap core as the local proxy, so MINT +
//! DECIDE stay in the engine). `SwapOutcome::{Allowed→200(passthrough),Denied→403,InternalRefused→503}`.
//!
//! Any verification failure short-circuits to 401/403 and the request NEVER reaches a mint. NO secret
//! bytes are logged: tracing carries only `client_id` / a decision label, never the bearer / proof /
//! EKM / key. EKM uncomputable ⇒ 403 (fail-closed binding). The poisoned-mutex path is a reject.

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use envctl_secrets::broker::decide::RemotePeer;
use envctl_secrets::broker::jti::JtiReplayStore;
use envctl_secrets::Engine;
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::{TokioExecutor, TokioIo};

use crate::edge::dpop::{verify_dpop_proof, DpopReject, HttpMethod};
use crate::edge::tls::RelayTlsConfig;
use crate::proxy::{
    bare, extract_bearer, method_from_hyper, request_host, swap_and_respond_streaming, ProxyCtx,
};

/// The only route the PR-1 edge serves.
const SWAP_PATH: &str = "/v1/relay/swap";

/// The RFC 5705 exporter label the edge binds the DPoP proof against. A fixed, application-specific
/// label (RFC 5705 §4 disambiguation) so the value is unique to the envctl relay-DPoP binding.
const EKM_LABEL: &[u8] = b"EXPORTER-envctl-relay-dpop-v1";

/// Exported-keying-material length (bytes). 32 bytes = 256 bits, ample to bind a channel.
const EKM_LEN: usize = 32;

/// Per-connection edge state shared by every request on one TLS connection: the engine handle, the
/// edge-owned replay store (one `Mutex<JtiReplayStore>` for the whole edge — atomicity per §7), and
/// THIS connection's EKM (computed once after the handshake; `None` if it could not be computed).
#[derive(Clone)]
struct ConnState {
    ctx: ProxyCtx,
    jti: Arc<Mutex<JtiReplayStore>>,
    /// `Some(ekm)` once the handshake EKM was exported; `None` if uncomputable (FS-S20 → 403).
    ekm: Option<Arc<Vec<u8>>>,
    /// TASK-0032 / FS-S5: the streaming re-check cadence + lifetime cap for every stream served on
    /// this connection (production default unless a test override was configured).
    timing: crate::edge::stream::Timing,
}

/// Bind the edge TCP listener on `bind_addr`, build the in-process TLS acceptor from the relay-tls
/// cert, and serve `POST /v1/relay/swap` until `shutdown` resolves. Returns the bound address + the
/// serving task handle. A cert-load or bind failure is an `Err` (the caller makes it FATAL when the
/// edge is explicitly enabled — fail-closed).
pub async fn serve_edge_listener(
    engine: Engine,
    relay_tls_dir: &std::path::Path,
    bind_addr: SocketAddr,
    timing: crate::edge::stream::Timing,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) -> anyhow::Result<(SocketAddr, tokio::task::JoinHandle<()>)> {
    // Load the relay-tls ServerConfig FIRST (fail-closed: no cert ⇒ no edge). This is the ONLY cert
    // source — never the MITM CA (FS-S25, structural in `tls.rs`).
    let tls = RelayTlsConfig::load_from_dir(relay_tls_dir)?;
    let acceptor = tokio_rustls::TlsAcceptor::from(tls.server_config());

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    let local_addr = listener.local_addr()?;

    // One replay store for the whole edge (per-process; bounded — OI-SM-1 §4/§5).
    let jti = Arc::new(Mutex::new(JtiReplayStore::new()));
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
                    tokio::spawn(async move {
                        serve_connection(acceptor, ctx, jti, timing, tcp, peer).await;
                    });
                }
            }
        }
    });

    Ok((local_addr, handle))
}

/// Terminate TLS on one accepted TCP stream, export the connection EKM, and serve its requests.
async fn serve_connection(
    acceptor: tokio_rustls::TlsAcceptor,
    ctx: ProxyCtx,
    jti: Arc<Mutex<JtiReplayStore>>,
    timing: crate::edge::stream::Timing,
    tcp: tokio::net::TcpStream,
    peer: SocketAddr,
) {
    let tls_stream = match acceptor.accept(tcp).await {
        Ok(s) => s,
        Err(e) => {
            // A handshake failure (no client trust, bad ALPN, etc.) ends the connection with no
            // plaintext ever read. No key material is in scope.
            tracing::debug!(peer = %peer, error = %e, "relay edge TLS handshake failed");
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
        ekm,
        timing,
    };
    let service = service_fn(move |req| {
        let conn = conn.clone();
        async move { Ok::<_, std::convert::Infallible>(handle_edge_request(conn, req).await) }
    });

    if let Err(e) = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
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

    // The canonical htu the DPoP proof must bind: scheme+host+path (NO query). The edge host comes
    // from the request's Host header (the remote client addressed the edge). A missing Host ⇒ 400.
    let edge_host = match request_host(&req) {
        Some(h) => h,
        None => return bare(StatusCode::BAD_REQUEST),
    };
    let htu = format!("https://{edge_host}{SWAP_PATH}");

    // The verified remote presentation context, or a status to return — built BEFORE the swap so a
    // failure NEVER reaches a mint.
    let rp = match verify_remote_presentation(&conn, &req, &htu) {
        Ok(rp) => rp,
        Err(status) => {
            // Metadata-only: the status code, never the bearer/proof/EKM. A refusal here NEVER reached
            // a mint (it short-circuits before the swap core).
            tracing::debug!(status = %status.as_u16(), "relay edge presentation refused");
            return bare(status);
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

    // Hand to the SHARED swap core with the verified remote context. The provider is derived from the
    // upstream host INSIDE the core (same as the proxy), and the bearer is extracted there from the
    // forwarded headers exactly as the local plane does — so the edge and proxy never diverge in how
    // they drive `relay_swap`. The remote edge additionally wraps the response body in the streaming-
    // revocation supervisor; the local proxy passes the identity wrap (no in-stream re-check).
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

/// The fail-closed verification ladder. Returns the verified [`RemotePeer`] or the HTTP status to
/// return. NEVER reaches a mint on any error. The bearer/proof/EKM never enter a log line.
fn verify_remote_presentation(
    conn: &ConnState,
    req: &Request<Incoming>,
    htu: &str,
) -> Result<RemotePeer, StatusCode> {
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
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    // (2) The relay bearer must be PRESENT before we do any DPoP crypto (else `relay_swap` would
    // refuse `UnknownBearer` anyway — short-circuit to 401). The bearer rides `Authorization: Bearer`
    // (relay bearers are Bearer-scheme regardless of upstream provider). `swap_and_respond` re-extracts
    // it from the SAME headers, so we only confirm presence here (no need to thread the value through).
    if extract_bearer(req.headers(), envctl_secrets::Provider::Generic).is_none() {
        return Err(StatusCode::UNAUTHORIZED);
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
            return Err(StatusCode::FORBIDDEN);
        }
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // (4) Replay store: `check_and_record` under the edge-owned Mutex. A poisoned lock ⇒ reject
    // (NEVER unwrap), an Err (replay / drift / full) ⇒ 401.
    {
        let mut guard = match conn.jti.lock() {
            Ok(g) => g,
            // Poisoned mutex ⇒ fail-closed reject (never bypass).
            Err(_) => return Err(StatusCode::UNAUTHORIZED),
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
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    // (5) The bearer carries the bound `client_id`; but PR-1 derives identity from the DPoP `jkt`
    // matched against the registry. The remote client is identified by the `client_id` the proof
    // asserts (cross-checked against the registered jkt). Resolve it from the proof's client_id claim
    // (the bearer's own client_id is authenticated by decide()'s clause 11a binding). A proof with no
    // client_id claim cannot be edge-registry-checked ⇒ 401.
    let client_id = match &verified.client_id {
        Some(c) if !c.is_empty() => c.clone(),
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    // (6) Registry lookup BEFORE decide() (mirrors UnknownBearer pre-decide raise). Unknown/revoked ⇒
    // 401. A store error ⇒ fail-closed 401 (treat as a refusal, never an accept). Also assert the
    // proven jkt matches the registered jkt (RemoteBindingMismatch defense at the edge).
    match conn.ctx.engine.load_remote_client(&client_id) {
        Ok(Some(c)) if c.enabled && c.revoked_at_ms.is_none() => {
            if c.dpop_jkt != verified.jkt {
                // The proven key does not match the registered binding ⇒ 401.
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
        // Known but disabled/revoked ⇒ 401; unknown ⇒ 401; store error ⇒ fail-closed 401.
        Ok(Some(_)) | Ok(None) | Err(_) => return Err(StatusCode::UNAUTHORIZED),
    }

    // All checks passed: the proof is verified and bound, the jti is fresh, the client is registered +
    // enabled, and the proven jkt matches the registration. decide()'s clause 11a re-asserts everything
    // fail-closed once more inside the engine.
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
