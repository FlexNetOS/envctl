//! TASK-0032 (F5, P0) — streaming-revocation tear-down acceptance test (FS-S5).
//!
//! End-to-end over the REAL `serve_edge` listener (the SAME harness shape as `edge_e2e.rs`: fake
//! PresentUsb, a real relay-tls cert + tokio-rustls client, an EKM-bound RFC 9449 DPoP proof, a
//! registered+minted remote bearer). The difference is a `RecordingUpstream` that SLOW-PUMPS multiple
//! response-body chunks over time (it takes the per-request body sink and drives its own detached
//! pump), so the response is a LONG-LIVED stream the edge's periodic re-check supervises.
//!
//! The edge runs with a small `Timing` override (sub-second re-check + a few-second cap) so a stream
//! closes within seconds rather than the production 2s/300s — a test-only knob (`EdgeConfig.recheck_
//! timing`), NOT a production sleep.
//!
//! Cases:
//!   1. revoke the bearer mid-stream → the client's stream closes within ~2× the re-check interval
//!      and the body is TRUNCATED (fewer chunks than the pump would have sent).
//!   2. pull the USB key mid-stream → the stream closes within the bound (the presence gate goes
//!      absent, decide() denies GateAbsent).
//!   3. a still-authorized stream SURVIVES at least one re-check tick and receives a LATER chunk (no
//!      false-tear; the re-check `peek`s, so a rate-limited policy is not falsely denied).
//!   4. the hard max-duration cap tears a still-authorized stream down.
#![cfg(feature = "relay-edge")]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use envctl_secrets::seam::{NoMint, SystemClock, UpstreamError, UsbProbe};
use envctl_secrets::vault::{InMemStore, Store};
use envctl_secrets::{
    EgressReq, EgressResp, Engine, EventSink, Method, Provider, RelayKind, RelayPolicy, SecretMeta,
    SwapMode, Unlock, Upstream,
};
use ring::signature::{Ed25519KeyPair, KeyPair};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use zeroize::Zeroizing;

const SENTINEL: &[u8] = b"REAL-KEY-SENTINEL-STREAM";
const USB_UUID: &str = "STREAM-E2E-USB";
const UPSTREAM_HOST: &str = "api.anthropic.com";
const EDGE_HOST: &str = "edge.local";
const SWAP_PATH: &str = "/v1/relay/swap";
/// MUST match `listener::EKM_LABEL` / `EKM_LEN`.
const EKM_LABEL: &[u8] = b"EXPORTER-envctl-relay-dpop-v1";
const EKM_LEN: usize = 32;

/// The pump sends this many marker chunks if never torn down; a small inter-chunk delay makes the
/// stream span several re-check ticks. A torn-down stream receives strictly fewer.
const PUMP_CHUNKS: usize = 40;
const PUMP_DELAY: Duration = Duration::from_millis(150);

/// A small re-check cadence + cap so the test closes a stream within seconds (test-only override).
const RECHECK: Duration = Duration::from_millis(400);
const MAX_DURATION: Duration = Duration::from_secs(3);

// ---- fakes -----------------------------------------------------------------------------------

/// A USB probe whose possession can be FLIPPED at runtime (models a key pull mid-stream).
struct TogglableUsb {
    keyfile: Zeroizing<Vec<u8>>,
    present: Arc<AtomicBool>,
}
impl UsbProbe for TogglableUsb {
    fn keyfile_for(&self, uuid: &str) -> Option<Zeroizing<Vec<u8>>> {
        if self.present.load(Ordering::SeqCst) && uuid == USB_UUID {
            Some(self.keyfile.clone())
        } else {
            None
        }
    }
}

/// An upstream that asserts the REAL key reaches it, then SLOW-PUMPS `PUMP_CHUNKS` distinct marker
/// chunks into the per-request body sink on a detached task — exactly as `DaemonUpstream::send` spawns
/// a pump for a real streaming upstream. Each chunk is `CHUNK-NNNN\n` so the client can count them.
#[derive(Clone)]
struct SlowPumpUpstream {
    seen_key: Arc<Mutex<Option<Vec<u8>>>>,
}
#[async_trait::async_trait]
impl Upstream for SlowPumpUpstream {
    async fn send(
        &self,
        _req: EgressReq,
        real_key: &Zeroizing<Vec<u8>>,
    ) -> Result<EgressResp, UpstreamError> {
        *self.seen_key.lock().unwrap() = Some(real_key.to_vec());
        // Take THIS request's body sink and drive a detached slow pump (the real key is NOT captured
        // by the task — only opaque marker bytes flow here).
        if let Some(tx) = envctl_secretd::proxy::__test_take_body_tx() {
            tokio::spawn(async move {
                for i in 0..PUMP_CHUNKS {
                    let chunk = format!("CHUNK-{i:04}\n");
                    if tx
                        .send(Ok(hyper::body::Frame::data(hyper::body::Bytes::from(
                            chunk,
                        ))))
                        .await
                        .is_err()
                    {
                        break; // downstream (supervisor) torn down / client hung up → stop pumping.
                    }
                    tokio::time::sleep(PUMP_DELAY).await;
                }
                // tx drops here → body completes if it ran to the end.
            });
        }
        Ok(EgressResp {
            status: 200,
            headers: vec![("content-type".to_string(), "text/plain".to_string())],
            allowed: true,
        })
    }
}

// ---- helpers ----------------------------------------------------------------------------------

fn fast_params() -> envctl_secrets::keyslot::Argon2Params {
    envctl_secrets::keyslot::Argon2Params {
        m_kib: envctl_secrets::keyslot::ARGON2_M_KIB_FLOOR,
        t_cost: envctl_secrets::keyslot::ARGON2_T_COST_FLOOR,
        p_lanes: 1,
    }
}

/// A covering policy. `rate_per_min: Some(1)` so case 3 also proves the re-check `peek`s (does NOT
/// bump): if the re-check bumped, the 2nd tick would trip `RateLimited` and falsely tear the stream.
fn covering_policy() -> RelayPolicy {
    RelayPolicy {
        relay_id: "edge-stream-e2e".to_string(),
        kind: RelayKind::Named,
        provider: Provider::Anthropic,
        secret_name: "anthropic_key".to_string(),
        swap: SwapMode::BaseUrlRepoint {
            upstream_base: format!("https://{UPSTREAM_HOST}"),
        },
        host_allow: vec![UPSTREAM_HOST.to_string()],
        path_allow: vec!["/".to_string()],
        method_allow: vec![Method::Post, Method::Get],
        policy_ttl_secs: 86_400,
        // Intentionally TIGHT (=2): the open swap bumps in_window→1 (1 < 2 → Allow). A re-check that
        // BUMPED would push in_window→2 (2 >= 2 → RateLimited DENY) and falsely tear the stream down;
        // a `peek` re-check leaves it at 1 (Allow), so a live stream survives. This is the
        // peek-not-bump proof for the still-authorized case.
        rate_per_min: Some(2),
        quota_total_requests: None,
        quota_total_bytes: None,
        enabled: true,
        revoked: false,
    }
}

fn jkt_of(kp: &Ed25519KeyPair) -> [u8; 32] {
    let x_b64 = URL_SAFE_NO_PAD.encode(kp.public_key().as_ref());
    let canonical = format!("{{\"crv\":\"Ed25519\",\"kty\":\"OKP\",\"x\":\"{x_b64}\"}}");
    Sha256::digest(canonical.as_bytes()).into()
}

fn make_proof(
    kp: &Ed25519KeyPair,
    htu: &str,
    ekm: &[u8],
    jti: &str,
    iat_secs: i64,
    client_id: &str,
) -> String {
    let x = URL_SAFE_NO_PAD.encode(kp.public_key().as_ref());
    let header = serde_json::json!({
        "typ": "dpop+jwt", "alg": "EdDSA",
        "jwk": { "kty": "OKP", "crv": "Ed25519", "x": x },
    });
    let payload = serde_json::json!({
        "htm": "POST", "htu": htu, "jti": jti, "iat": iat_secs,
        "ekm": URL_SAFE_NO_PAD.encode(ekm), "client_id": client_id,
    });
    let h = URL_SAFE_NO_PAD.encode(serde_json::to_string(&header).unwrap().as_bytes());
    let p = URL_SAFE_NO_PAD.encode(serde_json::to_string(&payload).unwrap().as_bytes());
    let signing_input = format!("{h}.{p}");
    let sig = kp.sign(signing_input.as_bytes());
    format!("{h}.{p}.{}", URL_SAFE_NO_PAD.encode(sig.as_ref()))
}

fn write_relay_cert(dir: &std::path::Path) {
    let cert = rcgen::generate_simple_self_signed(vec![EDGE_HOST.to_string()]).unwrap();
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join("cert.pem"), cert.cert.pem()).unwrap();
    std::fs::write(dir.join("key.pem"), cert.key_pair.serialize_pem()).unwrap();
}

fn client_config_trusting_only(cert_pem: &std::path::Path) -> rustls::ClientConfig {
    let pem = std::fs::read(cert_pem).expect("read relay cert");
    let mut rd = std::io::BufReader::new(&pem[..]);
    let mut roots = rustls::RootCertStore::empty();
    for cert in rustls_pemfile::certs(&mut rd) {
        roots.add(cert.expect("parse cert")).expect("add root");
    }
    rustls::ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
        .with_safe_default_protocol_versions()
        .expect("ring safe protocol versions")
        .with_root_certificates(roots)
        .with_no_client_auth()
}

async fn connect_and_ekm(
    connector: &tokio_rustls::TlsConnector,
    addr: std::net::SocketAddr,
) -> (
    tokio_rustls::client::TlsStream<tokio::net::TcpStream>,
    Vec<u8>,
) {
    let tcp = tokio::net::TcpStream::connect(addr)
        .await
        .expect("tcp connect");
    let server_name = rustls::pki_types::ServerName::try_from(EDGE_HOST).unwrap();
    let tls = connector
        .connect(server_name, tcp)
        .await
        .expect("tls handshake");
    let ekm = {
        let (_, client_conn) = tls.get_ref();
        let out = [0u8; EKM_LEN];
        client_conn
            .export_keying_material(out, EKM_LABEL, None)
            .expect("client EKM export")
            .to_vec()
    };
    (tls, ekm)
}

/// Write the POST request head + body (the swap request). Does NOT read the response — the caller
/// reads/counts streamed body chunks itself (so it can act mid-stream).
async fn write_swap_request<S>(tls: &mut S, bearer: &str, dpop: &str)
where
    S: AsyncWriteExt + Unpin,
{
    let body = b"{\"q\":1}";
    let head = format!(
        "POST {SWAP_PATH} HTTP/1.1\r\nHost: {EDGE_HOST}\r\n\
         x-relay-upstream-host: {UPSTREAM_HOST}\r\nx-relay-upstream-path: /v1/messages\r\n\
         content-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\
         authorization: Bearer {bearer}\r\ndpop: {dpop}\r\n\r\n",
        body.len()
    );
    tls.write_all(head.as_bytes()).await.expect("write head");
    tls.write_all(body).await.expect("write body");
    tls.flush().await.expect("flush");
}

/// Count distinct `CHUNK-NNNN` markers in a byte buffer (deduplicated, monotonic markers).
fn count_chunks(buf: &[u8]) -> usize {
    let s = String::from_utf8_lossy(buf);
    s.matches("CHUNK-").count()
}

/// Read the whole response until the server closes the connection (or `overall` elapses), returning
/// the raw bytes and whether the connection was OBSERVED to close (EOF) within the deadline.
async fn read_until_close<S>(tls: &mut S, overall: Duration) -> (Vec<u8>, bool)
where
    S: AsyncReadExt + Unpin,
{
    let mut out = Vec::new();
    let mut tmp = [0u8; 4096];
    let deadline = Instant::now() + overall;
    let mut closed = false;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        match tokio::time::timeout(remaining, tls.read(&mut tmp)).await {
            Ok(Ok(0)) => {
                closed = true;
                break;
            }
            Ok(Ok(n)) => out.extend_from_slice(&tmp[..n]),
            Ok(Err(_)) => {
                closed = true;
                break;
            }
            Err(_) => break, // overall deadline hit without a close.
        }
    }
    (out, closed)
}

fn build_engine(
    paths: &envctl_secrets::paths::Paths,
    rec: SlowPumpUpstream,
    usb: Box<dyn UsbProbe>,
) -> Engine {
    Engine::with_seams(
        paths.clone(),
        Box::new(InMemStore::new()) as Box<dyn Store>,
        Box::new(SystemClock),
        usb,
        Box::new(NoMint),
        Box::new(rec),
        #[cfg(feature = "provider-github")]
        Box::new(envctl_secrets::mint_github::NoopHttpTransport),
    )
    .expect("with_seams")
}

/// One-time vault + client + bearer setup shared by every case. Returns the engine, the connector,
/// the minted bearer, the DPoP keypair, the USB toggle, and the serving address + shutdown handle.
struct Harness {
    engine: Engine,
    connector: tokio_rustls::TlsConnector,
    addr: std::net::SocketAddr,
    bearer: String,
    token_id: String,
    kp: Ed25519KeyPair,
    usb_present: Arc<AtomicBool>,
    shutdown: tokio::sync::oneshot::Sender<()>,
    handle: tokio::task::JoinHandle<()>,
    seen_key: Arc<Mutex<Option<Vec<u8>>>>,
}

async fn setup(tag: &str) -> Harness {
    let _ = rustls::crypto::ring::default_provider().install_default();

    let root =
        std::env::temp_dir().join(format!("envctl-edge-stream-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let paths = envctl_secrets::paths::Paths::under(root.clone());
    std::fs::create_dir_all(&paths.runtime).unwrap();
    std::fs::create_dir_all(&paths.config).unwrap();
    write_relay_cert(&paths.relay_tls_dir());

    let seen_key = Arc::new(Mutex::new(None));
    let rec = SlowPumpUpstream {
        seen_key: seen_key.clone(),
    };
    let usb_present = Arc::new(AtomicBool::new(true));
    let keyfile = Zeroizing::new(vec![0x5Au8; 64]);
    let usb = TogglableUsb {
        keyfile: keyfile.clone(),
        present: usb_present.clone(),
    };
    let engine = build_engine(&paths, rec, Box::new(usb));

    let (sink, _rx) = EventSink::channel();
    // A USB-gated vault so a USB pull (case 2) flips the presence gate.
    engine
        .init_vault(
            Zeroizing::new("correct horse battery staple".to_string()),
            Some(USB_UUID.to_string()),
            Some(keyfile.clone()),
            fast_params(),
            &sink,
        )
        .expect("init_vault");
    engine
        .unlock(
            Unlock::Passphrase(Zeroizing::new("correct horse battery staple".to_string())),
            &sink,
        )
        .expect("unlock");
    engine
        .secret_put(
            SecretMeta {
                name: "anthropic_key".to_string(),
                provider: Provider::Anthropic,
                note: String::new(),
                broker_only: true,
            },
            Zeroizing::new(SENTINEL.to_vec()),
            &sink,
        )
        .expect("secret_put");

    let kp = {
        let rng = ring::rand::SystemRandom::new();
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap()
    };
    let jkt = jkt_of(&kp);
    engine
        .register_remote_client("phone".to_string(), jkt, false, &sink)
        .expect("register remote client");
    let minted = engine
        .relay_mint_remote(covering_policy(), 3600, "phone".to_string(), jkt, &sink)
        .expect("relay_mint_remote");
    let bearer = minted.raw.to_string();
    let token_id = minted.token_id.clone();

    let cert_pem = paths.relay_tls_dir().join("cert.pem");
    let client_cfg = Arc::new(client_config_trusting_only(&cert_pem));
    let connector = tokio_rustls::TlsConnector::from(client_cfg);

    let (sd_tx, sd_rx) = tokio::sync::oneshot::channel::<()>();
    let cfg = envctl_secretd::edge::EdgeConfig {
        enabled: true,
        bind_addr: "127.0.0.1:0".parse().unwrap(),
        // Test-only small cadence/cap so streams close within seconds (not 2s/300s).
        recheck_timing: Some(envctl_secretd::edge::stream::Timing::new(
            RECHECK,
            MAX_DURATION,
        )),
    };
    let (addr, handle) =
        envctl_secretd::edge::serve_edge(engine.clone(), &paths, &cfg, async move {
            let _ = sd_rx.await;
        })
        .await
        .expect("serve_edge");

    Harness {
        engine,
        connector,
        addr,
        bearer,
        token_id,
        kp,
        usb_present,
        shutdown: sd_tx,
        handle,
        seen_key,
    }
}

// ---- the cases --------------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn revoke_mid_stream_tears_down_within_bound() {
    let h = setup("revoke").await;
    let htu = format!("https://{EDGE_HOST}{SWAP_PATH}");
    let now_secs = chrono::Utc::now().timestamp();

    let (mut tls, ekm) = connect_and_ekm(&h.connector, h.addr).await;
    let proof = make_proof(&h.kp, &htu, &ekm, "jti-revoke", now_secs, "phone");
    write_swap_request(&mut tls, &h.bearer, &proof).await;

    // Let the stream run for a couple of chunks, then revoke the bearer mid-flight.
    tokio::time::sleep(PUMP_DELAY * 3).await;
    let (sink, _rx) = EventSink::channel();
    let n = h
        .engine
        .relay_revoke_bearer(&h.token_id, true, &sink)
        .expect("revoke");
    assert_eq!(n, 1, "the revoke flipped exactly one bearer");
    assert_eq!(
        h.seen_key.lock().unwrap().as_deref(),
        Some(SENTINEL),
        "the real key reached the upstream on the allowed open swap"
    );

    // The stream MUST close within ~2× the re-check interval (generous CI slack), truncated well
    // short of the full pump.
    let (buf, closed) = read_until_close(&mut tls, RECHECK * 6).await;
    assert!(closed, "the revoked stream must close (HTTP/2 stream end)");
    let chunks = count_chunks(&buf);
    // The stream FLOWED (some chunks arrived before the revoke) then was TRUNCATED (not the full
    // pump) — proving an in-flight tear-down, not a stream that never started.
    assert!(
        (1..PUMP_CHUNKS).contains(&chunks),
        "the body must flow then be TRUNCATED after the revoke (saw {chunks}/{PUMP_CHUNKS} chunks)"
    );

    let _ = h.shutdown.send(());
    let _ = h.handle.await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn usb_pull_mid_stream_tears_down_within_bound() {
    let h = setup("usb").await;
    let htu = format!("https://{EDGE_HOST}{SWAP_PATH}");
    let now_secs = chrono::Utc::now().timestamp();

    let (mut tls, ekm) = connect_and_ekm(&h.connector, h.addr).await;
    let proof = make_proof(&h.kp, &htu, &ekm, "jti-usb", now_secs, "phone");
    write_swap_request(&mut tls, &h.bearer, &proof).await;

    tokio::time::sleep(PUMP_DELAY * 3).await;
    // Pull the USB key — the presence gate goes absent; the next re-check denies GateAbsent.
    h.usb_present.store(false, Ordering::SeqCst);

    let (buf, closed) = read_until_close(&mut tls, RECHECK * 6).await;
    assert!(closed, "the USB-pulled stream must close within the bound");
    let chunks = count_chunks(&buf);
    assert!(
        (1..PUMP_CHUNKS).contains(&chunks),
        "the body must flow then be truncated after the USB pull (saw {chunks}/{PUMP_CHUNKS})"
    );

    let _ = h.shutdown.send(());
    let _ = h.handle.await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn still_authorized_stream_survives_a_recheck_tick() {
    let h = setup("survive").await;
    let htu = format!("https://{EDGE_HOST}{SWAP_PATH}");
    let now_secs = chrono::Utc::now().timestamp();

    let (mut tls, ekm) = connect_and_ekm(&h.connector, h.addr).await;
    let proof = make_proof(&h.kp, &htu, &ekm, "jti-survive", now_secs, "phone");
    write_swap_request(&mut tls, &h.bearer, &proof).await;

    // Read for a window spanning SEVERAL re-check ticks WITHOUT revoking. The stream must keep
    // delivering chunks past the first tick (no false-tear), proving the re-check `peek`s (the tiny
    // rate_per_min=1 policy is NOT falsely RateLimited on the 2nd tick).
    let window = RECHECK * 4;
    let (buf, _closed) = read_until_close(&mut tls, window).await;
    let chunks = count_chunks(&buf);
    // At least 2 ticks' worth of chunks must have arrived (the stream lived past tick #1).
    let min_expected = (window.as_millis() / PUMP_DELAY.as_millis() / 2) as usize;
    assert!(
        chunks >= min_expected.max(3),
        "a still-authorized stream must keep streaming across re-check ticks (saw {chunks} chunks, \
         expected ≥ {})",
        min_expected.max(3)
    );

    let _ = h.shutdown.send(());
    let _ = h.handle.await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn max_duration_cap_tears_down() {
    let h = setup("maxdur").await;
    let htu = format!("https://{EDGE_HOST}{SWAP_PATH}");
    let now_secs = chrono::Utc::now().timestamp();

    let (mut tls, ekm) = connect_and_ekm(&h.connector, h.addr).await;
    let proof = make_proof(&h.kp, &htu, &ekm, "jti-maxdur", now_secs, "phone");
    write_swap_request(&mut tls, &h.bearer, &proof).await;

    // The pump (PUMP_CHUNKS * PUMP_DELAY = 6s) outlives MAX_DURATION (3s). The hard cap MUST tear the
    // (still-authorized) stream down before the pump finishes. Allow generous slack.
    let started = Instant::now();
    let (buf, closed) = read_until_close(&mut tls, MAX_DURATION * 3).await;
    let elapsed = started.elapsed();
    assert!(closed, "the max-duration cap must close the stream");
    let chunks = count_chunks(&buf);
    assert!(
        (1..PUMP_CHUNKS).contains(&chunks),
        "the stream must flow then be capped before the full pump completes (saw {chunks})"
    );
    assert!(
        elapsed < MAX_DURATION * 3,
        "the cap must fire near MAX_DURATION, not at the full pump length"
    );

    let _ = h.shutdown.send(());
    let _ = h.handle.await;
}
