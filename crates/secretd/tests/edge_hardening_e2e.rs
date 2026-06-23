//! F2 relay-edge HARDENING acceptance tests (TASK-0031-PR2).
//!
//! End-to-end over the REAL `serve_edge` listener, exercising the PR-2 anti-abuse + mTLS gates with
//! SMALL injected params so CI without real caps passes quickly:
//!   * server-issued DPoP-Nonce: a nonce-less proof → 401 + `DPoP-Nonce`; the retry echoing the nonce
//!     → 200 (the real key reaches the recording upstream). A stale/unknown nonce → 401 (a FRESH
//!     challenge header is present so a genuine retry can recover).
//!   * per-IP rate limit: with `burst = 1`, a SECOND swap on the same source IP → 429, and the
//!     recording upstream NEVER saw a key for the shed request (the shed happened BEFORE verify +
//!     decide()).
//!   * body caps: an oversized body → 413 (small injected `max_body_bytes`).
//!   * body-read timeout: a stalled body (Content-Length promises more than is sent) → 408 (small
//!     injected `idle_timeout`).
//!   * mTLS: `require_client_cert = true` → a no-client-cert handshake fails (no 200); a valid
//!     client cert minted off the configured client-CA → 200.
//!
//! Reuses the `edge_e2e.rs` harness shape (fake USB, recording Upstream, `Engine::with_seams`).
#![cfg(feature = "relay-edge")]

use std::sync::{Arc, Mutex};
use std::time::Duration;

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

const SENTINEL: &[u8] = b"REAL-KEY-SENTINEL-HARD";
const USB_UUID: &str = "HARD-E2E-USB";
const UPSTREAM_HOST: &str = "api.anthropic.com";
const EDGE_HOST: &str = "edge.local";
const SWAP_PATH: &str = "/v1/relay/swap";
const UPSTREAM_BODY: &[u8] = b"{\"ok\":true,\"from\":\"hard-upstream\"}";
const EKM_LABEL: &[u8] = b"EXPORTER-envctl-relay-dpop-v1";
const EKM_LEN: usize = 32;

// ---- fakes -----------------------------------------------------------------------------------

struct PresentUsb(Zeroizing<Vec<u8>>);
impl UsbProbe for PresentUsb {
    fn keyfile_for(&self, uuid: &str) -> Option<Zeroizing<Vec<u8>>> {
        (uuid == USB_UUID).then(|| self.0.clone())
    }
}

#[derive(Clone)]
struct RecordingUpstream {
    seen_key: Arc<Mutex<Option<Vec<u8>>>>,
}
#[async_trait::async_trait]
impl Upstream for RecordingUpstream {
    async fn send(
        &self,
        _req: EgressReq,
        real_key: &Zeroizing<Vec<u8>>,
    ) -> Result<EgressResp, UpstreamError> {
        *self.seen_key.lock().unwrap() = Some(real_key.to_vec());
        envctl_secretd::proxy::__test_pump_response_body(hyper::body::Bytes::from_static(
            UPSTREAM_BODY,
        ))
        .await;
        Ok(EgressResp {
            status: 200,
            headers: vec![("content-type".to_string(), "application/json".to_string())],
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

fn covering_policy() -> RelayPolicy {
    RelayPolicy {
        relay_id: "hard-e2e".to_string(),
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
        rate_per_min: None,
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

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

/// Build a signed DPoP proof, optionally echoing a server-issued `nonce` (PR-2). Matches the
/// verifier's signing-input convention (raw base64url segments, RFC 7515 §5).
fn make_proof(
    kp: &Ed25519KeyPair,
    htu: &str,
    ekm: &[u8],
    jti: &str,
    iat_secs: i64,
    client_id: &str,
    nonce: Option<&str>,
) -> String {
    let x = URL_SAFE_NO_PAD.encode(kp.public_key().as_ref());
    let header = serde_json::json!({
        "typ": "dpop+jwt", "alg": "EdDSA",
        "jwk": { "kty": "OKP", "crv": "Ed25519", "x": x },
    });
    let mut payload = serde_json::json!({
        "htm": "POST", "htu": htu, "jti": jti, "iat": iat_secs,
        "ekm": URL_SAFE_NO_PAD.encode(ekm), "client_id": client_id,
    });
    if let Some(n) = nonce {
        payload
            .as_object_mut()
            .unwrap()
            .insert("nonce".to_string(), serde_json::json!(n));
    }
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

/// Send one POST /v1/relay/swap; return (status, optional DPoP-Nonce header). `body_override` lets
/// the caller exercise the body-cap (oversized) and stalled-body (Content-Length > actual) paths.
async fn post_swap<S>(
    tls: &mut S,
    bearer: Option<&str>,
    dpop: Option<&str>,
    body: &[u8],
    content_length_override: Option<usize>,
) -> (u16, Option<String>)
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    let clen = content_length_override.unwrap_or(body.len());
    let mut head = format!(
        "POST {SWAP_PATH} HTTP/1.1\r\nHost: {EDGE_HOST}\r\n\
         x-relay-upstream-host: {UPSTREAM_HOST}\r\nx-relay-upstream-path: /v1/messages\r\n\
         content-type: application/json\r\ncontent-length: {clen}\r\nconnection: close\r\n"
    );
    if let Some(b) = bearer {
        head.push_str(&format!("authorization: Bearer {b}\r\n"));
    }
    if let Some(d) = dpop {
        head.push_str(&format!("dpop: {d}\r\n"));
    }
    head.push_str("\r\n");
    tls.write_all(head.as_bytes()).await.expect("write head");
    tls.write_all(body).await.expect("write body");
    tls.flush().await.expect("flush");

    let mut out = Vec::new();
    let mut tmp = [0u8; 512];
    // A timeout/close from the server (e.g. the 408 path may close abruptly) ends the read with an
    // Err (→ `while let` exits); an n==0 EOF or the response head being fully seen also ends it.
    while let Ok(n) = tls.read(&mut tmp).await {
        if n == 0 {
            break;
        }
        out.extend_from_slice(&tmp[..n]);
        // Once the response head is seen we have the status (+ for a 200 the upstream body) — stop.
        if out.windows(4).any(|w| w == b"\r\n\r\n") && out.len() > 16 {
            break;
        }
    }
    parse_status_and_nonce(&out)
}

fn parse_status_and_nonce(raw: &[u8]) -> (u16, Option<String>) {
    let head = String::from_utf8_lossy(raw);
    let status = head
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|c| c.parse::<u16>().ok())
        .unwrap_or(0);
    let nonce = head.lines().find_map(|l| {
        let (k, v) = l.split_once(':')?;
        k.trim()
            .eq_ignore_ascii_case("dpop-nonce")
            .then(|| v.trim().to_string())
    });
    (status, nonce)
}

/// Fetch a fresh server-issued nonce by sending a nonce-less proof and reading the challenge header.
async fn fetch_nonce(
    connector: &tokio_rustls::TlsConnector,
    addr: std::net::SocketAddr,
    kp: &Ed25519KeyPair,
    htu: &str,
    bearer: &str,
    jti: &str,
    now_secs: i64,
) -> String {
    let (mut tls, ekm) = connect_and_ekm(connector, addr).await;
    let proof = make_proof(kp, htu, &ekm, jti, now_secs, "phone", None);
    let (status, nonce) = post_swap(&mut tls, Some(bearer), Some(&proof), b"{\"q\":1}", None).await;
    assert_eq!(status, 401, "a nonce-less proof must be a 401 challenge");
    nonce.expect("the 401 challenge must carry a DPoP-Nonce header")
}

fn build_engine(
    paths: &envctl_secrets::paths::Paths,
    rec: RecordingUpstream,
    keyfile: Zeroizing<Vec<u8>>,
) -> Engine {
    Engine::with_seams(
        paths.clone(),
        Box::new(InMemStore::new()) as Box<dyn Store>,
        Box::new(SystemClock),
        Box::new(PresentUsb(keyfile)),
        Box::new(NoMint),
        Box::new(rec),
        #[cfg(feature = "provider-github")]
        Box::new(envctl_secrets::mint_github::NoopHttpTransport),
    )
    .expect("with_seams")
}

/// Full unlock + secret + registration + mint; returns (engine, kp, raw_bearer).
fn provision(
    paths: &envctl_secrets::paths::Paths,
    rec: &RecordingUpstream,
    keyfile: &Zeroizing<Vec<u8>>,
) -> (Engine, Ed25519KeyPair, String) {
    let engine = build_engine(paths, rec.clone(), keyfile.clone());
    let (sink, _rx) = EventSink::channel();
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
    let bearer = engine
        .relay_mint_remote(covering_policy(), 3600, "phone".to_string(), jkt, &sink)
        .expect("relay_mint_remote");
    let raw = bearer.raw.to_string();
    (engine, kp, raw)
}

/// A unique temp root per test (no `tempfile` dep), cleaned best-effort by the caller.
fn temp_root(tag: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!(
        "envctl-edge-hard-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&p);
    p
}

/// Caps with a SMALL body cap + idle timeout (for the 413/408 paths) but a GENEROUS admission burst
/// — every test connection shares one source IP (127.0.0.1) and thus one bucket, so a stingy burst
/// would shed unrelated requests. The rate-limit test uses its OWN edge with `burst = 1`.
fn small_caps() -> envctl_secretd::edge::IngressCaps {
    envctl_secretd::edge::IngressCaps {
        handshake_timeout: Duration::from_secs(5),
        header_read_timeout: Duration::from_secs(5),
        // Small body-read (idle) timeout so the stalled-body test resolves to 408 quickly.
        idle_timeout: Duration::from_millis(600),
        // Small body cap so a modest oversized body trips 413.
        max_body_bytes: 64,
        // Generous burst (rate-limit is tested on a dedicated burst=1 edge).
        admission: Some((60, 256, 1024)),
    }
}

// ---- nonce + anti-abuse e2e -------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn edge_nonce_and_anti_abuse() {
    let _ = rustls::crypto::ring::default_provider().install_default();

    let root = temp_root("aa");
    let paths = envctl_secrets::paths::Paths::under(root.clone());
    std::fs::create_dir_all(&paths.runtime).unwrap();
    std::fs::create_dir_all(&paths.config).unwrap();
    write_relay_cert(&paths.relay_tls_dir());

    let rec = RecordingUpstream {
        seen_key: Arc::new(Mutex::new(None)),
    };
    let keyfile = Zeroizing::new(vec![0x5Au8; 64]);
    let (engine, kp, raw_bearer) = provision(&paths, &rec, &keyfile);

    let cert_pem = paths.relay_tls_dir().join("cert.pem");
    let client_cfg = Arc::new(client_config_trusting_only(&cert_pem));
    let connector = tokio_rustls::TlsConnector::from(client_cfg);

    let (sd_tx, sd_rx) = tokio::sync::oneshot::channel::<()>();
    let cfg = envctl_secretd::edge::EdgeConfig {
        enabled: true,
        bind_addr: "127.0.0.1:0".parse().unwrap(),
        recheck_timing: None,
        require_client_cert: false,
        client_ca_path: None,
        client_revocations_path: None,
        ingress_caps: Some(small_caps()),
    };
    let (addr, handle) =
        envctl_secretd::edge::serve_edge(engine.clone(), &paths, &cfg, async move {
            let _ = sd_rx.await;
        })
        .await
        .expect("serve_edge");

    let htu = format!("https://{EDGE_HOST}{SWAP_PATH}");
    let now_secs = chrono::Utc::now().timestamp();

    // ---- 1. nonce challenge → retry → 200 (real key reaches upstream) ----
    {
        let nonce = fetch_nonce(&connector, addr, &kp, &htu, &raw_bearer, "jti-n0", now_secs).await;
        let (mut tls, ekm) = connect_and_ekm(&connector, addr).await;
        let proof = make_proof(&kp, &htu, &ekm, "jti-n1", now_secs, "phone", Some(&nonce));
        let (status, _) = post_swap(
            &mut tls,
            Some(&raw_bearer),
            Some(&proof),
            b"{\"q\":1}",
            None,
        )
        .await;
        assert_eq!(status, 200, "a nonce'd retry must be 200");
        assert_eq!(
            rec.seen_key.lock().unwrap().as_deref(),
            Some(SENTINEL),
            "the real key reaches the upstream on the nonce'd accept"
        );
    }

    // ---- 2. stale/unknown nonce → 401 with a FRESH challenge header ----
    {
        let (mut tls, ekm) = connect_and_ekm(&connector, addr).await;
        let proof = make_proof(
            &kp,
            &htu,
            &ekm,
            "jti-n2",
            now_secs,
            "phone",
            Some("deadbeefnonexistentnonce"),
        );
        let (status, fresh) = post_swap(
            &mut tls,
            Some(&raw_bearer),
            Some(&proof),
            b"{\"q\":1}",
            None,
        )
        .await;
        assert_eq!(status, 401, "an unknown nonce must be 401");
        assert!(
            fresh.is_some(),
            "an unknown-nonce 401 must carry a FRESH DPoP-Nonce so a genuine retry can recover"
        );
    }

    let _ = sd_tx.send(());
    let _ = handle.await;
    let _ = std::fs::remove_dir_all(&root);
}

// ---- per-IP rate limit (dedicated edge with burst=2: one token for the nonce challenge, one for
// the nonce'd 200; a THIRD request is shed at admission BEFORE verify/decide) -------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn edge_rate_limit_sheds_before_decide() {
    let _ = rustls::crypto::ring::default_provider().install_default();

    let root = temp_root("rl");
    let paths = envctl_secrets::paths::Paths::under(root.clone());
    std::fs::create_dir_all(&paths.runtime).unwrap();
    std::fs::create_dir_all(&paths.config).unwrap();
    write_relay_cert(&paths.relay_tls_dir());

    let rec = RecordingUpstream {
        seen_key: Arc::new(Mutex::new(None)),
    };
    let keyfile = Zeroizing::new(vec![0x5Au8; 64]);
    let (engine, kp, raw_bearer) = provision(&paths, &rec, &keyfile);

    let cert_pem = paths.relay_tls_dir().join("cert.pem");
    let connector =
        tokio_rustls::TlsConnector::from(Arc::new(client_config_trusting_only(&cert_pem)));

    // burst = 2 with a 0/min refill: exactly two admits ever (the nonce challenge + the nonce'd 200),
    // then a hard shed. (Refill 0 so no token trickles back during the test.)
    let mut caps = small_caps();
    caps.admission = Some((0, 2, 1024));

    let (sd_tx, sd_rx) = tokio::sync::oneshot::channel::<()>();
    let cfg = envctl_secretd::edge::EdgeConfig {
        enabled: true,
        bind_addr: "127.0.0.1:0".parse().unwrap(),
        recheck_timing: None,
        require_client_cert: false,
        client_ca_path: None,
        client_revocations_path: None,
        ingress_caps: Some(caps),
    };
    let (addr, handle) =
        envctl_secretd::edge::serve_edge(engine.clone(), &paths, &cfg, async move {
            let _ = sd_rx.await;
        })
        .await
        .expect("serve_edge (rate limit)");

    let htu = format!("https://{EDGE_HOST}{SWAP_PATH}");
    let now_secs = chrono::Utc::now().timestamp();

    // Token 1: the nonce challenge. Token 2: the nonce'd retry → 200.
    let nonce = fetch_nonce(
        &connector,
        addr,
        &kp,
        &htu,
        &raw_bearer,
        "jti-rl0",
        now_secs,
    )
    .await;
    let (mut tls1, ekm1) = connect_and_ekm(&connector, addr).await;
    let p1 = make_proof(&kp, &htu, &ekm1, "jti-rl1", now_secs, "phone", Some(&nonce));
    let (s1, _) = post_swap(&mut tls1, Some(&raw_bearer), Some(&p1), b"{\"q\":1}", None).await;
    assert_eq!(s1, 200, "the nonce'd swap consumes the second burst token");

    // The bucket is now empty (refill 0). Reset the upstream observation: a THIRD request must be SHED
    // at admission (429) BEFORE the verify ladder / decide() / the recording upstream is reached.
    *rec.seen_key.lock().unwrap() = None;
    let (mut tls2, ekm2) = connect_and_ekm(&connector, addr).await;
    let p2 = make_proof(&kp, &htu, &ekm2, "jti-rl2", now_secs, "phone", None);
    let (s2, _) = post_swap(&mut tls2, Some(&raw_bearer), Some(&p2), b"{\"q\":1}", None).await;
    assert_eq!(
        s2, 429,
        "a third request from the same IP is rate-limited (429)"
    );
    assert!(
        rec.seen_key.lock().unwrap().is_none(),
        "a rate-shed request must NEVER reach decide()/the recording upstream"
    );

    let _ = sd_tx.send(());
    let _ = handle.await;
    let _ = std::fs::remove_dir_all(&root);
}

// ---- body caps + timeouts e2e -----------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn edge_body_caps_and_timeouts() {
    let _ = rustls::crypto::ring::default_provider().install_default();

    let root = temp_root("bc");
    let paths = envctl_secrets::paths::Paths::under(root.clone());
    std::fs::create_dir_all(&paths.runtime).unwrap();
    std::fs::create_dir_all(&paths.config).unwrap();
    write_relay_cert(&paths.relay_tls_dir());

    let rec = RecordingUpstream {
        seen_key: Arc::new(Mutex::new(None)),
    };
    let keyfile = Zeroizing::new(vec![0x5Au8; 64]);
    let (engine, kp, raw_bearer) = provision(&paths, &rec, &keyfile);

    let cert_pem = paths.relay_tls_dir().join("cert.pem");
    let client_cfg = Arc::new(client_config_trusting_only(&cert_pem));
    let connector = tokio_rustls::TlsConnector::from(client_cfg);

    // Use a generous admission burst here (this test is about body caps, not rate limits).
    let mut caps = small_caps();
    caps.admission = Some((60, 64, 1024));

    let (sd_tx, sd_rx) = tokio::sync::oneshot::channel::<()>();
    let cfg = envctl_secretd::edge::EdgeConfig {
        enabled: true,
        bind_addr: "127.0.0.1:0".parse().unwrap(),
        recheck_timing: None,
        require_client_cert: false,
        client_ca_path: None,
        client_revocations_path: None,
        ingress_caps: Some(caps),
    };
    let (addr, handle) =
        envctl_secretd::edge::serve_edge(engine.clone(), &paths, &cfg, async move {
            let _ = sd_rx.await;
        })
        .await
        .expect("serve_edge");

    let htu = format!("https://{EDGE_HOST}{SWAP_PATH}");
    let now_secs = chrono::Utc::now().timestamp();

    // ---- oversized body → 413 (body exceeds max_body_bytes = 64) ----
    {
        let nonce = fetch_nonce(
            &connector,
            addr,
            &kp,
            &htu,
            &raw_bearer,
            "jti-bc0",
            now_secs,
        )
        .await;
        let (mut tls, ekm) = connect_and_ekm(&connector, addr).await;
        let proof = make_proof(&kp, &htu, &ekm, "jti-bc1", now_secs, "phone", Some(&nonce));
        let big = vec![b'x'; 4096]; // >> 64-byte cap
        let (status, _) = post_swap(&mut tls, Some(&raw_bearer), Some(&proof), &big, None).await;
        assert_eq!(status, 413, "an oversized body must be 413");
    }

    // ---- stalled body → 408 (Content-Length promises 4096 but we send only a few bytes) ----
    {
        let nonce = fetch_nonce(
            &connector,
            addr,
            &kp,
            &htu,
            &raw_bearer,
            "jti-bc2",
            now_secs,
        )
        .await;
        let (mut tls, ekm) = connect_and_ekm(&connector, addr).await;
        let proof = make_proof(&kp, &htu, &ekm, "jti-bc3", now_secs, "phone", Some(&nonce));
        // Send a SHORT body but advertise a much larger Content-Length: the body read stalls waiting
        // for bytes that never arrive, tripping the small idle_timeout → 408.
        let short = b"{\"q\":1}";
        let (status, _) =
            post_swap(&mut tls, Some(&raw_bearer), Some(&proof), short, Some(4096)).await;
        assert_eq!(status, 408, "a stalled body read must be 408");
    }

    let _ = sd_tx.send(());
    let _ = handle.await;
    let _ = std::fs::remove_dir_all(&root);
}

// ---- mTLS (PR-2b) e2e -------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn edge_mtls_requires_client_cert() {
    let _ = rustls::crypto::ring::default_provider().install_default();

    let root = temp_root("mtls");
    let paths = envctl_secrets::paths::Paths::under(root.clone());
    std::fs::create_dir_all(&paths.runtime).unwrap();
    std::fs::create_dir_all(&paths.config).unwrap();
    write_relay_cert(&paths.relay_tls_dir());

    // Provision a remote-clients-CA: a CA cert (the trust anchor) + a client leaf signed by it.
    let ca = rcgen::generate_simple_self_signed(vec!["remote-clients-ca".to_string()])
        .expect("gen client CA");
    let client_ca_path = root.join("clients-ca.pem");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(&client_ca_path, ca.cert.pem()).unwrap();

    let rec = RecordingUpstream {
        seen_key: Arc::new(Mutex::new(None)),
    };
    let keyfile = Zeroizing::new(vec![0x5Au8; 64]);
    let (engine, kp, raw_bearer) = provision(&paths, &rec, &keyfile);

    let cert_pem = paths.relay_tls_dir().join("cert.pem");

    // Generous admission burst (this test is about mTLS, not rate limits).
    let mut caps = small_caps();
    caps.admission = Some((60, 64, 1024));

    let (sd_tx, sd_rx) = tokio::sync::oneshot::channel::<()>();
    let cfg = envctl_secretd::edge::EdgeConfig {
        enabled: true,
        bind_addr: "127.0.0.1:0".parse().unwrap(),
        recheck_timing: None,
        require_client_cert: true,
        client_ca_path: Some(client_ca_path),
        client_revocations_path: None,
        ingress_caps: Some(caps),
    };
    let (addr, handle) =
        envctl_secretd::edge::serve_edge(engine.clone(), &paths, &cfg, async move {
            let _ = sd_rx.await;
        })
        .await
        .expect("serve_edge (mTLS)");

    let htu = format!("https://{EDGE_HOST}{SWAP_PATH}");
    let now_secs = chrono::Utc::now().timestamp();

    // ---- no client cert → the connection NEVER yields a 200 swap (mTLS rejects it). The server
    // aborts a client that presents no cert; depending on TLS timing rustls surfaces this either as a
    // failed `connect()` OR as a reset/closed connection on the first request. Either way the request
    // must NOT succeed (no 200) — that is the load-bearing assertion. ----
    {
        let no_cert_cfg = Arc::new(client_config_trusting_only(&cert_pem));
        let connector = tokio_rustls::TlsConnector::from(no_cert_cfg);
        let tcp = tokio::net::TcpStream::connect(addr).await.expect("tcp");
        let server_name = rustls::pki_types::ServerName::try_from(EDGE_HOST).unwrap();
        let rejected = match connector.connect(server_name, tcp).await {
            // Handshake failed outright (the common case) — mTLS rejected the anonymous client.
            Err(_) => true,
            // Handshake "completed" client-side; the server aborts. A raw write+read on the aborted
            // connection errors (BrokenPipe / reset) or returns no 200 — any of those means rejected.
            Ok(mut tls) => {
                let req = format!(
                    "POST {SWAP_PATH} HTTP/1.1\r\nHost: {EDGE_HOST}\r\nconnection: close\r\n\r\n"
                );
                if tls.write_all(req.as_bytes()).await.is_err() {
                    true
                } else {
                    let mut buf = Vec::new();
                    // A reset/closed connection reads EOF/err; a (hypothetical) success would echo a 200.
                    let _ = tls.read_to_end(&mut buf).await;
                    !String::from_utf8_lossy(&buf).contains(" 200 ")
                }
            }
        };
        assert!(
            rejected,
            "a client with no certificate must NEVER complete a swap when mTLS is required"
        );
    }

    // ---- valid client cert (minted off the configured client-CA) → handshake + swap succeed → 200 ----
    {
        // Build a client leaf SIGNED BY the client-CA the edge trusts.
        let mut params = rcgen::CertificateParams::new(vec!["phone-client".to_string()]).unwrap();
        params.is_ca = rcgen::IsCa::NoCa;
        let client_key = rcgen::KeyPair::generate().unwrap();
        let ca_kp =
            rcgen::KeyPair::from_pem(&ca.key_pair.serialize_pem()).expect("reload CA keypair");
        let ca_params =
            rcgen::CertificateParams::new(vec!["remote-clients-ca".to_string()]).unwrap();
        let ca_cert = ca_params.self_signed(&ca_kp).expect("rebuild CA cert");
        let client_leaf = params
            .signed_by(&client_key, &ca_cert, &ca_kp)
            .expect("sign client leaf");

        // A client config that trusts the relay cert (server side) AND presents the client leaf.
        let pem = std::fs::read(&cert_pem).expect("read relay cert");
        let mut rd = std::io::BufReader::new(&pem[..]);
        let mut roots = rustls::RootCertStore::empty();
        for cert in rustls_pemfile::certs(&mut rd) {
            roots.add(cert.expect("parse cert")).expect("add root");
        }
        let client_chain = vec![client_leaf.der().clone()];
        let client_pk =
            rustls::pki_types::PrivateKeyDer::try_from(client_key.serialize_der()).unwrap();
        let with_cert = rustls::ClientConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .expect("ring safe protocol versions")
        .with_root_certificates(roots)
        .with_client_auth_cert(client_chain, client_pk)
        .expect("client auth cert");
        let connector = tokio_rustls::TlsConnector::from(Arc::new(with_cert));

        let nonce = fetch_nonce(
            &connector,
            addr,
            &kp,
            &htu,
            &raw_bearer,
            "jti-mtls0",
            now_secs,
        )
        .await;
        let (mut tls, ekm) = connect_and_ekm(&connector, addr).await;
        let proof = make_proof(
            &kp,
            &htu,
            &ekm,
            "jti-mtls1",
            now_secs,
            "phone",
            Some(&nonce),
        );
        let (status, _) = post_swap(
            &mut tls,
            Some(&raw_bearer),
            Some(&proof),
            b"{\"q\":1}",
            None,
        )
        .await;
        assert_eq!(
            status, 200,
            "a valid client cert + nonce'd swap must be 200"
        );
    }

    let _ = sd_tx.send(());
    let _ = handle.await;
    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn edge_mtls_rejects_revoked_client_cert() {
    let _ = rustls::crypto::ring::default_provider().install_default();

    let root = temp_root("mtls-revoke");
    let paths = envctl_secrets::paths::Paths::under(root.clone());
    std::fs::create_dir_all(&paths.runtime).unwrap();
    std::fs::create_dir_all(&paths.config).unwrap();
    write_relay_cert(&paths.relay_tls_dir());

    let ca = rcgen::generate_simple_self_signed(vec!["remote-clients-ca".to_string()])
        .expect("gen client CA");
    let client_ca_path = root.join("clients-ca.pem");
    std::fs::write(&client_ca_path, ca.cert.pem()).unwrap();

    let revocations_path = root.join("clients-revoked.txt");
    std::fs::write(&revocations_path, b"").unwrap();

    let rec = RecordingUpstream {
        seen_key: Arc::new(Mutex::new(None)),
    };
    let keyfile = Zeroizing::new(vec![0x5Au8; 64]);
    let (engine, _kp, _raw_bearer) = provision(&paths, &rec, &keyfile);

    let cert_pem = paths.relay_tls_dir().join("cert.pem");
    let pem = std::fs::read(&cert_pem).expect("read relay cert");
    let mut rd = std::io::BufReader::new(&pem[..]);
    let mut roots = rustls::RootCertStore::empty();
    for cert in rustls_pemfile::certs(&mut rd) {
        roots.add(cert.expect("parse cert")).expect("add root");
    }

    let mut caps = small_caps();
    caps.admission = Some((60, 64, 1024));

    let (sd_tx, sd_rx) = tokio::sync::oneshot::channel::<()>();
    let cfg = envctl_secretd::edge::EdgeConfig {
        enabled: true,
        bind_addr: "127.0.0.1:0".parse().unwrap(),
        recheck_timing: None,
        require_client_cert: true,
        client_ca_path: Some(client_ca_path.clone()),
        client_revocations_path: Some(revocations_path.clone()),
        ingress_caps: Some(caps),
    };
    let (addr, handle) =
        envctl_secretd::edge::serve_edge(engine.clone(), &paths, &cfg, async move {
            let _ = sd_rx.await;
        })
        .await
        .expect("serve_edge (mTLS revoke)");

    let mut params = rcgen::CertificateParams::new(vec!["phone-client".to_string()]).unwrap();
    params.is_ca = rcgen::IsCa::NoCa;
    let client_key = rcgen::KeyPair::generate().unwrap();
    let ca_kp = rcgen::KeyPair::from_pem(&ca.key_pair.serialize_pem()).expect("reload CA keypair");
    let ca_params = rcgen::CertificateParams::new(vec!["remote-clients-ca".to_string()]).unwrap();
    let ca_cert = ca_params.self_signed(&ca_kp).expect("rebuild CA cert");
    let client_leaf = params
        .signed_by(&client_key, &ca_cert, &ca_kp)
        .expect("sign client leaf");
    let revoked_fp = hex_lower(&Sha256::digest(client_leaf.der().as_ref()));
    std::fs::write(&revocations_path, format!("{revoked_fp}\n")).unwrap();

    let client_chain = vec![client_leaf.der().clone()];
    let client_pk = rustls::pki_types::PrivateKeyDer::try_from(client_key.serialize_der()).unwrap();
    let with_cert = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .expect("ring safe protocol versions")
    .with_root_certificates(roots)
    .with_client_auth_cert(client_chain, client_pk)
    .expect("client auth cert");
    let connector = tokio_rustls::TlsConnector::from(Arc::new(with_cert));
    let tcp = tokio::net::TcpStream::connect(addr).await.expect("tcp");
    let server_name = rustls::pki_types::ServerName::try_from(EDGE_HOST).unwrap();
    let rejected = match connector.connect(server_name, tcp).await {
        Err(_) => true,
        Ok(mut tls) => {
            let req = format!(
                "POST {SWAP_PATH} HTTP/1.1\r\nHost: {EDGE_HOST}\r\nconnection: close\r\n\r\n"
            );
            if tls.write_all(req.as_bytes()).await.is_err() {
                true
            } else {
                let mut buf = Vec::new();
                let _ = tls.read_to_end(&mut buf).await;
                !String::from_utf8_lossy(&buf).contains(" 200 ")
            }
        }
    };
    assert!(
        rejected,
        "a revoked client cert must never complete a usable swap"
    );

    let _ = sd_tx.send(());
    let _ = handle.await;
    let _ = std::fs::remove_dir_all(&root);
}
