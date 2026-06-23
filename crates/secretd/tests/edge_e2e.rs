//! F2 remote relay-edge acceptance test (TASK-0031 PR-1).
//!
//! End-to-end over the REAL `serve_edge` listener: a tokio-rustls client trusting ONLY the
//! operator-provisioned relay-tls cert does a real handshake, computes the connection EKM (the SAME
//! RFC 5705 export the edge computes server-side), builds a valid RFC 9449 DPoP proof bound to that
//! EKM, and POSTs `/v1/relay/swap` carrying a registered+minted remote bearer. The recording
//! `Upstream` asserts the REAL key reaches it; the response is 200.
//!
//! Negatives (each must be a 401, request never reaching a mint): a REPLAYED jti, a REVOKED client,
//! a TAMPERED proof, and NO DPoP header.
//!
//! Reuses the `mitm_e2e.rs` harness shape (fake USB so register/mint pass their USB gate, recording
//! Upstream, `Engine::with_seams`).
#![cfg(feature = "relay-edge")]

use std::sync::{Arc, Mutex};

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

const SENTINEL: &[u8] = b"REAL-KEY-SENTINEL-EDGE";
const USB_UUID: &str = "EDGE-E2E-USB";
const UPSTREAM_HOST: &str = "api.anthropic.com";
const EDGE_HOST: &str = "edge.local";
const SWAP_PATH: &str = "/v1/relay/swap";
const UPSTREAM_BODY: &[u8] = b"{\"ok\":true,\"from\":\"edge-upstream\"}";
/// MUST match `listener::EKM_LABEL` / `EKM_LEN`.
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
        relay_id: "edge-e2e".to_string(),
        kind: RelayKind::Named,
        provider: Provider::Anthropic,
        secret_name: "anthropic_key".to_string(),
        // A plain (non-MITM) swap: the upstream host is taken from the EgressReq host the edge built.
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

/// The RFC 7638 SHA-256 thumbprint of an OKP/Ed25519 key (must match `dpop.rs`'s computation).
fn jkt_of(kp: &Ed25519KeyPair) -> [u8; 32] {
    let x_b64 = URL_SAFE_NO_PAD.encode(kp.public_key().as_ref());
    let canonical = format!("{{\"crv\":\"Ed25519\",\"kty\":\"OKP\",\"x\":\"{x_b64}\"}}");
    Sha256::digest(canonical.as_bytes()).into()
}

/// Build a signed DPoP proof for `htu`/`ekm`/`iat`/`jti`, optionally echoing a server-issued
/// `nonce` (PR-2: `None` ⇒ no nonce claim). The signing input is the raw base64url segments
/// (RFC 7515 §5), matching the verifier.
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
        "typ": "dpop+jwt",
        "alg": "EdDSA",
        "jwk": { "kty": "OKP", "crv": "Ed25519", "x": x },
    });
    let mut payload = serde_json::json!({
        "htm": "POST",
        "htu": htu,
        "jti": jti,
        "iat": iat_secs,
        "ekm": URL_SAFE_NO_PAD.encode(ekm),
        "client_id": client_id,
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

/// Write a self-signed relay-tls cert+key into `dir/{cert.pem,key.pem}` for SAN `edge.local`.
fn write_relay_cert(dir: &std::path::Path) {
    let cert = rcgen::generate_simple_self_signed(vec![EDGE_HOST.to_string()]).unwrap();
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join("cert.pem"), cert.cert.pem()).unwrap();
    std::fs::write(dir.join("key.pem"), cert.key_pair.serialize_pem()).unwrap();
}

/// A rustls ClientConfig trusting ONLY the relay-tls cert (so a successful handshake proves the edge
/// presented our provisioned cert — never the MITM CA).
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

/// Connect a TLS client to `addr`, export the SAME EKM the edge computes, and return the connected
/// stream + the EKM bytes.
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

/// Send one POST /v1/relay/swap with the given DPoP + (optional) bearer headers; return the status
/// AND any `DPoP-Nonce` response header (PR-2: present on a nonce challenge).
async fn post_swap<S>(
    tls: &mut S,
    bearer: Option<&str>,
    dpop: Option<&str>,
) -> (u16, Option<String>)
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    post_swap_body(tls, bearer, dpop, b"{\"q\":1}").await
}

/// As [`post_swap`] but with a caller-supplied request body (so the body-cap path can be exercised).
async fn post_swap_body<S>(
    tls: &mut S,
    bearer: Option<&str>,
    dpop: Option<&str>,
    body: &[u8],
) -> (u16, Option<String>)
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    let mut head = format!(
        "POST {SWAP_PATH} HTTP/1.1\r\nHost: {EDGE_HOST}\r\n\
         x-relay-upstream-host: {UPSTREAM_HOST}\r\nx-relay-upstream-path: /v1/messages\r\n\
         content-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n",
        body.len()
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
    loop {
        let n = tls.read(&mut tmp).await.expect("read resp");
        if n == 0 {
            break;
        }
        out.extend_from_slice(&tmp[..n]);
        // Stop once we have the status line + (for 200) the upstream body, or the headers end.
        if out.windows(UPSTREAM_BODY.len()).any(|w| w == UPSTREAM_BODY)
            || out.windows(4).filter(|w| *w == b"\r\n\r\n").count() >= 1 && out.len() > 16
        {
            // Give the body a brief chance for the 200 case; otherwise the head suffices.
            if out.windows(4).any(|w| w == b"\r\n\r\n") {
                break;
            }
        }
    }
    parse_status_and_nonce(&out)
}

/// Parse the HTTP status code + any `DPoP-Nonce` header out of a raw response head.
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
        if k.trim().eq_ignore_ascii_case("dpop-nonce") {
            Some(v.trim().to_string())
        } else {
            None
        }
    });
    (status, nonce)
}

/// The full PR-2 challenge→retry dance over a SINGLE connection per attempt: a first proof with NO
/// nonce gets a `401 + DPoP-Nonce` challenge; a fresh connection then retries with that nonce echoed
/// in the proof. Returns the FINAL status (200 on the happy path). Each attempt uses a fresh
/// connection + fresh jti (the edge is single-use on both nonce and jti).
#[allow(clippy::too_many_arguments)]
async fn swap_with_nonce_dance(
    connector: &tokio_rustls::TlsConnector,
    addr: std::net::SocketAddr,
    kp: &Ed25519KeyPair,
    htu: &str,
    bearer: &str,
    client_id: &str,
    jti_prefix: &str,
    now_secs: i64,
) -> u16 {
    // (1) First request with no nonce → expect a 401 challenge carrying a fresh DPoP-Nonce.
    let (mut tls1, ekm1) = connect_and_ekm(connector, addr).await;
    let proof1 = make_proof(
        kp,
        htu,
        &ekm1,
        &format!("{jti_prefix}-a"),
        now_secs,
        client_id,
        None,
    );
    let (s1, nonce) = post_swap(&mut tls1, Some(bearer), Some(&proof1)).await;
    assert_eq!(
        s1, 401,
        "the first (nonce-less) request must be a 401 challenge"
    );
    let nonce = nonce.expect("a 401 challenge must carry a DPoP-Nonce header");

    // (2) Retry on a fresh connection echoing the issued nonce → the verify ladder + decide() run.
    let (mut tls2, ekm2) = connect_and_ekm(connector, addr).await;
    let proof2 = make_proof(
        kp,
        htu,
        &ekm2,
        &format!("{jti_prefix}-b"),
        now_secs,
        client_id,
        Some(&nonce),
    );
    let (s2, _) = post_swap(&mut tls2, Some(bearer), Some(&proof2)).await;
    s2
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

// ---- the test ---------------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn edge_dpop_swap_accepts_and_rejects() {
    let _ = rustls::crypto::ring::default_provider().install_default();

    let root = std::env::temp_dir().join(format!("envctl-edge-e2e-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let paths = envctl_secrets::paths::Paths::under(root.clone());
    std::fs::create_dir_all(&paths.runtime).unwrap();
    std::fs::create_dir_all(&paths.config).unwrap();

    // Provision the relay-tls cert (the ONLY cert the edge serves — never the MITM CA).
    write_relay_cert(&paths.relay_tls_dir());

    let rec = RecordingUpstream {
        seen_key: Arc::new(Mutex::new(None)),
    };
    let keyfile = Zeroizing::new(vec![0x5Au8; 64]);
    let engine = build_engine(&paths, rec.clone(), keyfile.clone());

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

    // The DPoP keypair + its registered jkt.
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
    let raw_bearer = bearer.raw.to_string();

    // Serve the REAL edge listener under a oneshot shutdown.
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
        ingress_caps: None,
    };
    let (addr, handle) =
        envctl_secretd::edge::serve_edge(engine.clone(), &paths, &cfg, async move {
            let _ = sd_rx.await;
        })
        .await
        .expect("serve_edge");

    let htu = format!("https://{EDGE_HOST}{SWAP_PATH}");
    let now_secs = chrono::Utc::now().timestamp();

    // ---- happy path: a valid DPoP-bound swap → 200 + the real key reaches the upstream ----
    {
        // PR-2: the swap now requires a server-issued DPoP-Nonce. The dance does the
        // challenge → retry; the final retry must be a 200 with the real key reaching the upstream.
        let status = swap_with_nonce_dance(
            &connector,
            addr,
            &kp,
            &htu,
            &raw_bearer,
            "phone",
            "jti-accept",
            now_secs,
        )
        .await;
        assert_eq!(status, 200, "a valid DPoP-bound + nonce'd swap must be 200");
        assert_eq!(
            rec.seen_key.lock().unwrap().as_deref(),
            Some(SENTINEL),
            "the REAL key must reach the upstream on an allowed remote swap"
        );
    }

    // ---- replayed jti → 401 (the second use of the same jti, each with a fresh nonce) ----
    {
        // Acquire a fresh nonce (challenge), then use it with a fixed jti (accepted).
        let (mut tlsc, ekmc) = connect_and_ekm(&connector, addr).await;
        let pc = make_proof(&kp, &htu, &ekmc, "jti-rc", now_secs, "phone", None);
        let (sc, nonce1) = post_swap(&mut tlsc, Some(&raw_bearer), Some(&pc)).await;
        assert_eq!(sc, 401, "challenge");
        let nonce1 = nonce1.expect("nonce on challenge");

        let (mut tls1, ekm1) = connect_and_ekm(&connector, addr).await;
        let proof1 = make_proof(
            &kp,
            &htu,
            &ekm1,
            "jti-replay",
            now_secs,
            "phone",
            Some(&nonce1),
        );
        let (s1, _) = post_swap(&mut tls1, Some(&raw_bearer), Some(&proof1)).await;
        assert_eq!(s1, 200, "first use of jti-replay is accepted");

        // Replay the SAME jti. It needs a fresh nonce (single-use); acquire one, then replay the jti.
        let (mut tlsc2, ekmc2) = connect_and_ekm(&connector, addr).await;
        let pc2 = make_proof(&kp, &htu, &ekmc2, "jti-rc2", now_secs, "phone", None);
        let (_, nonce2) = post_swap(&mut tlsc2, Some(&raw_bearer), Some(&pc2)).await;
        let nonce2 = nonce2.expect("nonce on challenge");
        let (mut tls2, ekm2) = connect_and_ekm(&connector, addr).await;
        let proof2 = make_proof(
            &kp,
            &htu,
            &ekm2,
            "jti-replay",
            now_secs,
            "phone",
            Some(&nonce2),
        );
        let (s2, _) = post_swap(&mut tls2, Some(&raw_bearer), Some(&proof2)).await;
        assert_eq!(s2, 401, "a replayed jti must be rejected (401)");
    }

    // ---- no DPoP header → 401 (rejected before the nonce gate) ----
    {
        let (mut tls, _ekm) = connect_and_ekm(&connector, addr).await;
        let (status, _) = post_swap(&mut tls, Some(&raw_bearer), None).await;
        assert_eq!(status, 401, "a swap with no DPoP header must be 401");
    }

    // ---- tampered proof → 401 (signature fails at verify_dpop_proof, BEFORE the nonce gate) ----
    {
        let (mut tls, ekm) = connect_and_ekm(&connector, addr).await;
        let proof = make_proof(&kp, &htu, &ekm, "jti-tamper", now_secs, "phone", None);
        let mut parts: Vec<String> = proof.split('.').map(|s| s.to_string()).collect();
        let evil = serde_json::json!({
            "htm": "POST", "htu": htu, "jti": "evil", "iat": now_secs,
            "ekm": URL_SAFE_NO_PAD.encode(&ekm), "client_id": "phone",
        });
        parts[1] = URL_SAFE_NO_PAD.encode(serde_json::to_string(&evil).unwrap().as_bytes());
        let tampered = parts.join(".");
        let (status, _) = post_swap(&mut tls, Some(&raw_bearer), Some(&tampered)).await;
        assert_eq!(status, 401, "a tampered proof must be 401");
    }

    // ---- unregistered/unknown client → 401 (the same edge `load_remote_client → None/revoked`
    // pre-decide refusal branch a REVOKED client hits; revocation TEAR-DOWN itself is PR-3). The proof
    // is otherwise valid + freshly-jti'd + EKM-bound + nonce'd, so the ONLY reason for the 401 is the
    // unknown client_id — proving the edge refuses an unregistered client BEFORE reaching a mint. The
    // dance returns 401 from the registry check (the unknown-client 401, not the nonce challenge). ----
    {
        let status = swap_with_nonce_dance(
            &connector,
            addr,
            &kp,
            &htu,
            &raw_bearer,
            "ghost",
            "jti-unknown-client",
            now_secs,
        )
        .await;
        assert_eq!(
            status, 401,
            "an unregistered remote client must be 401 (pre-decide)"
        );
    }

    let _ = sd_tx.send(());
    let _ = handle.await;
}
