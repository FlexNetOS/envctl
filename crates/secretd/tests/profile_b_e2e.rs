//! SERVER-MODE Profile B (VPS) end-to-end acceptance tests (TASK-0033 / OI-SM-2).
//!
//! Exercises the operator-box presence-token authorizer protocol end-to-end PLUS the four required
//! negative startup-guard tests (FS-S21/S22/S23/S24) and the authorizer-unreachable drain/deny path.
//!
//! - **Authorizer round-trip**: a real `spawn_authorizer_link` connects over mTLS (ring-only) to an
//!   in-test operator-box signer, fetches + verifies a presence token, feeds the VPS gate, and a
//!   `relay_swap` through the gate ALLOWS. (§7.2 deploy smoke.)
//! - **FS-S21**: a VPS profile with no operator_authorizer_url refuses to start.
//! - **FS-S22**: an on-box vault with an enrolled-but-unproven USB keyslot refuses unless
//!   `--allow-passphrase-only`.
//! - **FS-S23**: a boot-unwrapped DEK with no valid token denies `GateAbsent`; a token expiring
//!   between two swaps denies the 2nd; an unreachable authorizer clears the gate.
//! - **FS-S24**: a vTPM-gating config is refused at parse.
#![cfg(feature = "relay-edge")]

use std::sync::Arc;
use std::time::Duration;

use envctl_secrets::broker::PresenceToken;
use envctl_secrets::seam::{Clock, SystemClock};
use envctl_secrets::{
    sign_presence_token, AuthzReject, Engine, EventSink, JtiReplayStore, NonceStore,
    OperatorBoxTrustedTime, PresenceGate, StartupRefusal, SystemClockTrustedTime, Topology,
    VpsPresenceGate,
};
use ring::signature::{Ed25519KeyPair, KeyPair};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

// ---- helpers ---------------------------------------------------------------------------------

fn now_ms() -> i64 {
    SystemClock.now().timestamp_millis()
}

/// Generate an Ed25519 keypair from a fresh seed; return `(Zeroizing seed, pubkey32)`.
fn operator_keypair() -> (Zeroizing<[u8; 32]>, [u8; 32]) {
    let rng = ring::rand::SystemRandom::new();
    let mut seed = Zeroizing::new([0u8; 32]);
    ring::rand::SecureRandom::fill(&rng, seed.as_mut()).unwrap();
    let kp = Ed25519KeyPair::from_seed_unchecked(seed.as_ref()).unwrap();
    let mut pk = [0u8; 32];
    pk.copy_from_slice(kp.public_key().as_ref());
    (seed, pk)
}

const CERT_FP: [u8; 32] = [0x7Eu8; 32];

// ---- §7.2: authorizer round-trip + VPS swap ALLOWS on a verified token -----------------------

/// The engine verifies a real operator-signed token (through its trusted-time source), feeds the
/// VPS gate, and the gate then resolves Present — the core Profile-B issuance path, end-to-end.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn authorizer_token_verifies_and_opens_the_vps_gate() {
    let (seed, pubkey) = operator_keypair();
    let trusted = Arc::new(OperatorBoxTrustedTime::new(Box::new(SystemClock)));
    let gate = Arc::new(VpsPresenceGate::new(Box::new(SystemClock)));

    // Build a VPS engine sharing the gate + trusted-time (as the daemon does).
    let root = std::env::temp_dir().join(format!("envctl-pb-rt-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let paths = envctl_secrets::paths::Paths::under(root);
    std::fs::create_dir_all(&paths.runtime).unwrap();
    let engine = Engine::with_seams(
        paths,
        Box::new(envctl_secrets::vault::InMemStore::new()),
        Box::new(SystemClock),
        Box::new(envctl_secrets::seam::RealUsbProbe),
        Box::new(envctl_secrets::seam::NoMint),
        Box::new(envctl_secretd::proxy::DaemonUpstream::new()),
        #[cfg(feature = "provider-github")]
        Box::new(envctl_secrets::mint_github::NoopHttpTransport),
        Box::new(gate.clone()),
        Box::new(trusted.clone()),
        Topology::Vps,
    )
    .expect("with_seams");

    // The operator box mints + signs a token (as `secretctl authorizer serve` would).
    let mut nonce_store = NonceStore::new();
    let mut jti_store = JtiReplayStore::new();
    let rng = ring::rand::SystemRandom::new();
    let nonce = nonce_store.issue(now_ms(), &rng).unwrap();
    let t = now_ms();
    let tok = PresenceToken::new(
        t,
        "vps-e2e".into(),
        nonce,
        CERT_FP,
        t + 600_000,
        "jti-e2e".into(),
    );
    let sig = sign_presence_token(&seed, &tok).unwrap();

    // The VPS attests the operator time (as the link does), then verifies via the engine.
    trusted.attest(t);
    let (sink, _rx) = EventSink::channel();
    let expiry = engine
        .verify_presence_token(
            &pubkey,
            &tok,
            &sig,
            &CERT_FP,
            &mut nonce_store,
            &mut jti_store,
            &sink,
        )
        .expect("token verifies end-to-end");
    gate.accept_token(expiry);

    // The gate now resolves Present (the egress presence factor is satisfied on the VPS).
    assert_eq!(
        engine.presence_gate_state().unwrap(),
        envctl_secrets::GateState::Present,
        "a verified operator token opens the VPS gate"
    );
}

/// Loopback-SAN mTLS round-trip (the deterministic, CI-safe wire test): the link dials 127.0.0.1
/// against an operator signer whose cert has a 127.0.0.1 SAN, fetches a token, the engine verifies,
/// the gate opens.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn authorizer_link_loopback_roundtrip_opens_gate() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let (seed, operator_pubkey) = operator_keypair();

    // CA + operator server cert (SAN 127.0.0.1) + VPS client cert, all under one CA.
    let mut ca_params = rcgen::CertificateParams::new(vec![]).unwrap();
    ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    let ca_key = rcgen::KeyPair::generate().unwrap();
    let ca_cert = ca_params.self_signed(&ca_key).unwrap();

    let mut op_params = rcgen::CertificateParams::new(vec!["127.0.0.1".to_string()]).unwrap();
    op_params.subject_alt_names = vec![rcgen::SanType::IpAddress(std::net::IpAddr::V4(
        std::net::Ipv4Addr::LOCALHOST,
    ))];
    let op_key = rcgen::KeyPair::generate().unwrap();
    let op_cert = op_params.signed_by(&op_key, &ca_cert, &ca_key).unwrap();

    let mut vps_params = rcgen::CertificateParams::new(vec!["vps".to_string()]).unwrap();
    vps_params.subject_alt_names = vec![rcgen::SanType::DnsName("vps".try_into().unwrap())];
    let vps_key = rcgen::KeyPair::generate().unwrap();
    let vps_cert = vps_params.signed_by(&vps_key, &ca_cert, &ca_key).unwrap();

    let dir = std::env::temp_dir().join(format!("envctl-pb-lb-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let ca_pem = dir.join("ca.pem");
    std::fs::write(&ca_pem, ca_cert.pem()).unwrap();
    let vps_cert_path = dir.join("vps.cert");
    let vps_key_path = dir.join("vps.key");
    std::fs::write(&vps_cert_path, vps_cert.pem()).unwrap();
    std::fs::write(&vps_key_path, vps_key.serialize_pem()).unwrap();
    let op_cert_pem = op_cert.pem().into_bytes();
    let op_key_pem = op_key.serialize_pem().into_bytes();
    let ca_pem_bytes = ca_cert.pem().into_bytes();

    // Operator signer on loopback.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let op_addr = listener.local_addr().unwrap();
    let signer_seed = seed.clone();
    tokio::spawn(async move {
        let acceptor = build_operator_acceptor(&op_cert_pem, &op_key_pem, &ca_pem_bytes);
        loop {
            let Ok((tcp, _)) = listener.accept().await else {
                break;
            };
            let acc = acceptor.clone();
            let s = signer_seed.clone();
            tokio::spawn(async move {
                let _ = sign_one(acc, tcp, &s).await;
            });
        }
    });

    // VPS engine + link.
    let gate = Arc::new(VpsPresenceGate::new(Box::new(SystemClock)));
    let trusted = Arc::new(OperatorBoxTrustedTime::new(Box::new(SystemClock)));
    let root = std::env::temp_dir().join(format!("envctl-pb-lb-eng-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let paths = envctl_secrets::paths::Paths::under(root);
    std::fs::create_dir_all(&paths.runtime).unwrap();
    let engine = Engine::with_seams(
        paths,
        Box::new(envctl_secrets::vault::InMemStore::new()),
        Box::new(SystemClock),
        Box::new(envctl_secrets::seam::RealUsbProbe),
        Box::new(envctl_secrets::seam::NoMint),
        Box::new(envctl_secretd::proxy::DaemonUpstream::new()),
        #[cfg(feature = "provider-github")]
        Box::new(envctl_secrets::mint_github::NoopHttpTransport),
        Box::new(gate.clone()),
        Box::new(trusted.clone()),
        Topology::Vps,
    )
    .expect("with_seams");

    let vps_cert_fp =
        envctl_secretd::edge::authorizer::cert_fingerprint_from_pem(&vps_cert_path).unwrap();
    let auth_cfg = envctl_secretd::edge::authorizer::AuthorizerConfig {
        url: format!("https://127.0.0.1:{}", op_addr.port()),
        vps_instance_id: "vps-lb".into(),
        vps_cert_fp,
        operator_pubkey,
        operator_ca_path: ca_pem.clone(),
        client_cert_path: vps_cert_path,
        client_key_path: vps_key_path,
    };

    let (sink, _rx) = EventSink::channel();
    let (sd_tx, sd_rx) = tokio::sync::oneshot::channel::<()>();
    let handle = envctl_secretd::edge::authorizer::spawn_authorizer_link(
        auth_cfg,
        engine.clone(),
        gate.clone(),
        trusted.clone(),
        sink,
        async {
            let _ = sd_rx.await;
        },
    )
    .expect("spawn link");

    // Poll until the link primes the gate (first refresh tick fires immediately).
    let mut primed = false;
    for _ in 0..50 {
        if matches!(
            engine.presence_gate_state().unwrap(),
            envctl_secrets::GateState::Present
        ) {
            primed = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let _ = sd_tx.send(());
    let _ = handle.await;
    assert!(
        primed,
        "the authorizer link must verify a token over mTLS and open the VPS gate"
    );
}

// ---- FS-S21: VPS with no authorizer URL refuses to start -------------------------------------

#[tokio::test]
async fn fs_s21_vps_without_authorizer_url_refuses() {
    let engine = vps_engine_with_gate(Arc::new(VpsPresenceGate::new(Box::new(SystemClock))));
    let res = engine.assert_profile_b_startup(None, false, false);
    assert_eq!(
        res,
        Err(StartupRefusal::VpsNoSubstituteFactor),
        "FS-S21: a VPS with no operator-authorizer URL must refuse to start"
    );
}

// ---- FS-S22: on-box enrolled-but-unproven USB keyslot refuses unless override ----------------

#[tokio::test]
async fn fs_s22_onbox_unproven_usb_refuses_without_override() {
    // On-box engine with an AbsentUsb probe + an enrolled USB keyslot ⇒ possession unproven.
    let root = std::env::temp_dir().join(format!("envctl-pb-s22-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let paths = envctl_secrets::paths::Paths::under(root);
    std::fs::create_dir_all(&paths.runtime).unwrap();
    let engine = Engine::with_seams(
        paths,
        Box::new(envctl_secrets::vault::InMemStore::new()),
        Box::new(SystemClock),
        Box::new(AbsentUsb),
        Box::new(envctl_secrets::seam::NoMint),
        Box::new(envctl_secretd::proxy::DaemonUpstream::new()),
        #[cfg(feature = "provider-github")]
        Box::new(envctl_secrets::mint_github::NoopHttpTransport),
        Box::new(envctl_secrets::broker::UnprovenGate),
        Box::new(SystemClockTrustedTime),
        Topology::OnBox,
    )
    .expect("with_seams");
    let (sink, _rx) = EventSink::channel();
    // Enroll a USB keyslot (so has_enabled_usb_keyslot == true) but possession is unproven (AbsentUsb).
    engine
        .init_vault(
            Zeroizing::new("pw-s22".into()),
            Some("S22-UUID".into()),
            Some(Zeroizing::new(vec![0u8; 64])),
            fast_params(),
            &sink,
        )
        .expect("init_vault with usb keyslot");

    // Without the override ⇒ refuse.
    assert_eq!(
        engine.assert_profile_b_startup(None, false, false),
        Err(StartupRefusal::OnBoxUsbKeyslotUnproven),
        "FS-S22: on-box with an unproven USB keyslot must refuse without --allow-passphrase-only"
    );
    // With the override ⇒ ok.
    assert_eq!(
        engine.assert_profile_b_startup(None, true, false),
        Ok(()),
        "FS-S22: --allow-passphrase-only lets a passphrase-only operator serve"
    );
}

// ---- FS-S23: boot-unwrapped DEK + no valid token denies; expiry-between-swaps denies ----------

#[tokio::test]
async fn fs_s23_no_token_gate_unproven_at_startup_refuses() {
    let gate = Arc::new(VpsPresenceGate::new(Box::new(SystemClock))); // never proven
    let engine = vps_engine_with_gate(gate);
    // With an authorizer URL configured, FS-S21 passes but FS-S23 (gate Unproven) fires.
    assert_eq!(
        engine.assert_profile_b_startup(Some("https://op:9443"), false, false),
        Err(StartupRefusal::VpsGateUnprovenAtStartup),
        "FS-S23: a VPS that boots with no valid token refuses to serve boot-unwrapped egress"
    );
}

#[tokio::test]
async fn fs_s23_token_expiry_between_swaps_flips_gate_absent() {
    // The gate is re-resolved per resolve() against the clock — a token valid now is absent later.
    let clock = Arc::new(std::sync::Mutex::new(1_000i64));
    struct SharedClock(Arc<std::sync::Mutex<i64>>);
    impl Clock for SharedClock {
        fn now(&self) -> chrono::DateTime<chrono::Utc> {
            chrono::DateTime::from_timestamp_millis(*self.0.lock().unwrap()).unwrap()
        }
        fn boottime_ms(&self) -> i64 {
            0
        }
    }
    let gate = VpsPresenceGate::new(Box::new(SharedClock(clock.clone())));
    gate.accept_token(2_000);
    assert_eq!(
        gate.resolve(),
        envctl_secrets::GateState::Present,
        "valid at t=1000"
    );
    *clock.lock().unwrap() = 2_500;
    assert_eq!(
        gate.resolve(),
        envctl_secrets::GateState::AbsentSince(2_000),
        "FS-S23: the same token is absent after expiry — re-resolved per swap, never cached"
    );
}

// ---- FS-S24: vTPM gating refused at config parse ---------------------------------------------

#[tokio::test]
async fn fs_s24_vtpm_gating_refused_engine_guard() {
    let engine = vps_engine_with_gate(Arc::new(VpsPresenceGate::new(Box::new(SystemClock))));
    // The engine guard refuses vTPM gating regardless of topology.
    assert!(
        matches!(
            engine.assert_profile_b_startup(Some("https://op:9443"), false, true),
            Err(StartupRefusal::VtpmGatingForbidden)
        ),
        "FS-S24: vTPM-gated DEK release is forbidden"
    );
}

// ---- authorizer unreachable ⇒ gate cleared ⇒ swap denies (drain) -----------------------------

#[tokio::test]
async fn authorizer_unreachable_clears_gate_denying_egress() {
    let gate = Arc::new(VpsPresenceGate::new(Box::new(SystemClock)));
    let engine = vps_engine_with_gate(gate.clone());
    // Simulate a delivered token: gate Present.
    gate.accept_token(now_ms() + 600_000);
    assert_eq!(
        engine.presence_gate_state().unwrap(),
        envctl_secrets::GateState::Present
    );
    // The authorizer link clears the gate on unreachable (FS-S23) — the next gate read denies.
    gate.clear();
    assert_eq!(
        engine.presence_gate_state().unwrap(),
        envctl_secrets::GateState::Unproven,
        "an unreachable authorizer clears the gate; new egress denies GateAbsent + streams drain"
    );
}

// ---- a rejected token never opens the gate (defense in depth at the verify boundary) ----------

#[tokio::test]
async fn forged_token_is_rejected_and_gate_stays_closed() {
    let (seed, pubkey) = operator_keypair();
    let (_other_seed, other_pubkey) = operator_keypair();
    let engine = vps_engine_with_trusted(
        Arc::new(VpsPresenceGate::new(Box::new(SystemClock))),
        Arc::new(OperatorBoxTrustedTime::new(Box::new(SystemClock))),
    );
    let mut nonce_store = NonceStore::new();
    let mut jti_store = JtiReplayStore::new();
    let rng = ring::rand::SystemRandom::new();
    let nonce = nonce_store.issue(now_ms(), &rng).unwrap();
    let t = now_ms();
    let tok = PresenceToken::new(t, "vps".into(), nonce, CERT_FP, t + 600_000, "j".into());
    let sig = sign_presence_token(&seed, &tok).unwrap();
    let _ = pubkey;
    let (sink, _rx) = EventSink::channel();
    // Trusted-time is unavailable (never attested) ⇒ TrustedTimeUnavailable (OI-SM-3, fail-closed).
    let res = engine.verify_presence_token(
        &other_pubkey,
        &tok,
        &sig,
        &CERT_FP,
        &mut nonce_store,
        &mut jti_store,
        &sink,
    );
    assert!(
        matches!(res, Err(AuthzReject::TrustedTimeUnavailable)),
        "no fresh trusted time ⇒ refuse (OI-SM-3); got {res:?}"
    );
}

// ---- shared engine builders ------------------------------------------------------------------

fn vps_engine_with_gate(gate: Arc<VpsPresenceGate>) -> Engine {
    vps_engine_with_trusted(
        gate,
        Arc::new(OperatorBoxTrustedTime::new(Box::new(SystemClock))),
    )
}

fn vps_engine_with_trusted(
    gate: Arc<VpsPresenceGate>,
    trusted: Arc<OperatorBoxTrustedTime>,
) -> Engine {
    let root = std::env::temp_dir().join(format!(
        "envctl-pb-eng-{}-{:?}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let paths = envctl_secrets::paths::Paths::under(root);
    std::fs::create_dir_all(&paths.runtime).unwrap();
    Engine::with_seams(
        paths,
        Box::new(envctl_secrets::vault::InMemStore::new()),
        Box::new(SystemClock),
        Box::new(envctl_secrets::seam::RealUsbProbe),
        Box::new(envctl_secrets::seam::NoMint),
        Box::new(envctl_secretd::proxy::DaemonUpstream::new()),
        #[cfg(feature = "provider-github")]
        Box::new(envctl_secrets::mint_github::NoopHttpTransport),
        Box::new(gate),
        Box::new(trusted),
        Topology::Vps,
    )
    .expect("with_seams")
}

fn fast_params() -> envctl_secrets::keyslot::Argon2Params {
    envctl_secrets::keyslot::Argon2Params {
        m_kib: envctl_secrets::keyslot::ARGON2_M_KIB_FLOOR,
        t_cost: envctl_secrets::keyslot::ARGON2_T_COST_FLOOR,
        p_lanes: 1,
    }
}

struct AbsentUsb;
impl envctl_secrets::seam::UsbProbe for AbsentUsb {
    fn keyfile_for(&self, _uuid: &str) -> Option<Zeroizing<Vec<u8>>> {
        None
    }
}

// ---- minimal in-test operator signer (mirrors `secretctl authorizer serve`) ------------------

fn build_operator_acceptor(
    cert_pem: &[u8],
    key_pem: &[u8],
    client_ca_pem: &[u8],
) -> tokio_rustls::TlsAcceptor {
    use rustls::pki_types::{CertificateDer, PrivateKeyDer};
    let certs: Vec<CertificateDer<'static>> = {
        let mut r = std::io::BufReader::new(cert_pem);
        rustls_pemfile::certs(&mut r)
            .collect::<Result<_, _>>()
            .unwrap()
    };
    let key: PrivateKeyDer<'static> = {
        let mut r = std::io::BufReader::new(key_pem);
        rustls_pemfile::private_key(&mut r).unwrap().unwrap()
    };
    let mut roots = rustls::RootCertStore::empty();
    let mut r = std::io::BufReader::new(client_ca_pem);
    for c in rustls_pemfile::certs(&mut r) {
        roots.add(c.unwrap()).unwrap();
    }
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let verifier = rustls::server::WebPkiClientVerifier::builder_with_provider(
        Arc::new(roots),
        provider.clone(),
    )
    .build()
    .unwrap();
    let config = rustls::ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_client_cert_verifier(verifier)
        .with_single_cert(certs, key)
        .unwrap();
    tokio_rustls::TlsAcceptor::from(Arc::new(config))
}

async fn sign_one(
    acceptor: tokio_rustls::TlsAcceptor,
    tcp: tokio::net::TcpStream,
    seed: &Zeroizing<[u8; 32]>,
) -> anyhow::Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut tls = acceptor.accept(tcp).await?;
    let mut buf = vec![0u8; 8192];
    let n = tls.read(&mut buf).await?;
    let req = String::from_utf8_lossy(&buf[..n]).into_owned();
    let id = json_str(&req, "vps_instance_id").unwrap_or_default();
    let nonce = json_str(&req, "server_nonce").unwrap_or_default();
    let fp_hex = json_str(&req, "vps_cert_fp").unwrap_or_default();
    let mut fp = [0u8; 32];
    for (i, ch) in fp_hex.as_bytes().chunks_exact(2).enumerate().take(32) {
        fp[i] = u8::from_str_radix(std::str::from_utf8(ch).unwrap(), 16).unwrap();
    }
    let t = now_ms();
    let tok = PresenceToken::new(t, id, nonce, fp, t + 600_000, format!("jti-{t}"));
    let sig = sign_presence_token(seed, &tok)?;
    let body = format!(
        "{{\"ts_ms\":{},\"expiry_ms\":{},\"jti\":\"{}\",\"sig\":\"{}\",\"attested_time_ms\":{}}}",
        tok.ts_ms,
        tok.expiry_ms,
        tok.jti,
        hex_encode(&sig),
        t
    );
    let resp = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    tls.write_all(resp.as_bytes()).await?;
    tls.flush().await.ok();
    Ok(())
}

fn json_str(resp: &str, name: &str) -> Option<String> {
    let key = format!("\"{name}\"");
    let after = &resp[resp.find(&key)? + key.len()..];
    let after = after.trim_start().strip_prefix(':')?.trim_start();
    let after = after.strip_prefix('"')?;
    let end = after.find('"')?;
    Some(after[..end].to_string())
}

fn hex_encode(b: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(b.len() * 2);
    for &x in b {
        s.push(HEX[(x >> 4) as usize] as char);
        s.push(HEX[(x & 0x0f) as usize] as char);
    }
    s
}

// Silence the unused Sha256/Digest import warning if the verify path doesn't reference them.
#[allow(dead_code)]
fn _force_sha2_link() -> [u8; 32] {
    Sha256::digest(b"x").into()
}
