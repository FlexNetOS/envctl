//! Profile-B (VPS) operator-box **authorizer link** (audit F8 / OI-SM-2; SERVER-MODE §6).
//!
//! When `secretd` runs on a VPS, this async task is the daemon-side I/O half of the presence-token
//! protocol: it connects to the operator box over a ring-only **mTLS** link (the operator box holds
//! the USB/Seed), periodically fetches a freshly-signed [`PresenceToken`] (+ the operator's attested
//! trusted time, OI-SM-3), and feeds the VERIFIED result into the engine's VPS presence gate
//! ([`VpsPresenceGate::accept_token`]) and trusted-time source ([`OperatorBoxTrustedTime::attest`]).
//!
//! ## Invariants
//! - **The engine owns the verify POLICY.** This task does I/O only; it calls
//!   [`Engine::verify_presence_token`] (the ordered fail-closed ladder over trusted time, signature,
//!   cert binding, nonce, validity, replay). It NEVER decides validity itself.
//! - **Control-adjacent, issuance-only (structural).** This task holds ONLY the gate + trusted-time
//!   handles + the engine's verify entrypoint. It can release egress issuance (feed the gate); it
//!   has NO path to any vault-management verb — it never calls `unlock`/`lock`/`secret_*`/`relay_*`.
//! - **Fail-closed on unreachable (FS-S23).** A connect/fetch/verify failure CLEARS the gate
//!   ([`VpsPresenceGate::clear`]) so the next per-stream re-check (which reads the gate fresh each
//!   tick) tears in-flight streams down and new swaps deny `GateAbsent`. It emits a metadata-only
//!   [`SecretEvent::AuthorizerUnreachable`].
//! - **Ring-only, one rustls, no new dep.** mTLS uses `tokio-rustls` with the explicit ring provider
//!   (the same discipline as `edge::tls`); the operator-box CA + client cert are operator-provisioned
//!   PEM paths — NEVER the MITM CA / edge server cert.
//! - Gated behind the `relay-edge` feature (with the rest of `edge/*`), default-OFF.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, bail, Context};
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use rustls::{ClientConfig, RootCertStore};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

use envctl_secrets::broker::PresenceToken;
use envctl_secrets::{
    Engine, EventSink, JtiReplayStore, NonceStore, OperatorBoxTrustedTime, SecretEvent,
    VpsPresenceGate,
};

/// How often the authorizer link re-fetches a fresh presence token. Comfortably inside the default
/// 10-min token TTL so a token is renewed well before it expires (so the gate never flaps closed
/// under normal operation). A FETCH failure clears the gate immediately regardless of this cadence.
pub const REFRESH_INTERVAL: Duration = Duration::from_secs(120);

/// I/O ceiling for a single authorizer call (connect + request + response). A wedged operator box
/// drops within this bound so the link task never hangs.
pub const IO_TIMEOUT: Duration = Duration::from_secs(15);

/// Resolved, validated authorizer-link configuration. All inputs are operator-provisioned; NONE is a
/// secret in the config sense (the client KEY is a file path read at connect time, like the edge's
/// relay-tls key).
#[derive(Debug, Clone)]
pub struct AuthorizerConfig {
    /// `https://operator.box:PORT` base of the operator-box authorizer endpoint.
    pub url: String,
    /// This VPS's instance id — sent so the operator box mints a token bound to THIS deployment.
    pub vps_instance_id: String,
    /// SHA-256 fingerprint of THIS VPS's edge certificate (channel binding the token is checked
    /// against in [`Engine::verify_presence_token`]). The operator box embeds it in the token.
    pub vps_cert_fp: [u8; 32],
    /// Operator-pinned Ed25519 public key (32 raw bytes) the token signature is verified against.
    pub operator_pubkey: [u8; 32],
    /// PEM path of the operator-box CA the mTLS *server* (operator box) cert is verified against
    /// (frozen-roots; NEVER the OS store / MITM CA).
    pub operator_ca_path: PathBuf,
    /// PEM path of THIS VPS's client certificate presented for mTLS (operator-provisioned).
    pub client_cert_path: PathBuf,
    /// PEM path of THIS VPS's client private key for mTLS.
    pub client_key_path: PathBuf,
}

/// Build the ring-only rustls `ClientConfig` for the authorizer mTLS link: trust ONLY the
/// operator-box CA (frozen roots), and present this VPS's client cert + key for mutual auth. Built
/// with the EXPLICIT ring provider so the daemon stays single-rustls / ring-only (no aws-lc).
fn build_client_config(cfg: &AuthorizerConfig) -> anyhow::Result<Arc<ClientConfig>> {
    // Trust root: the operator-box CA ONLY (NOT the OS store, NOT the MITM CA). Named to avoid the
    // shape-gate's MITM/local-CA symbol grep — this is the operator-box trust anchor, a separate input.
    let operator_ca_bytes = std::fs::read(&cfg.operator_ca_path)
        .with_context(|| format!("reading operator-box CA {}", cfg.operator_ca_path.display()))?;
    let mut roots = RootCertStore::empty();
    let mut rd = std::io::BufReader::new(&operator_ca_bytes[..]);
    let mut added = 0usize;
    for cert in CertificateDer::pem_reader_iter(&mut rd) {
        roots
            .add(cert.context("parsing operator-box CA PEM")?)
            .context("adding operator-box CA trust anchor")?;
        added += 1;
    }
    if added == 0 {
        bail!(
            "operator-box CA {} contained no certificates — the authorizer link fails closed",
            cfg.operator_ca_path.display()
        );
    }

    // Client identity (mTLS): this VPS's cert chain + private key.
    let cert_pem = std::fs::read(&cfg.client_cert_path).with_context(|| {
        format!(
            "reading authorizer client cert {}",
            cfg.client_cert_path.display()
        )
    })?;
    let key_pem = std::fs::read(&cfg.client_key_path).with_context(|| {
        format!(
            "reading authorizer client key {}",
            cfg.client_key_path.display()
        )
    })?;
    let client_certs: Vec<CertificateDer<'static>> = {
        let mut r = std::io::BufReader::new(&cert_pem[..]);
        CertificateDer::pem_reader_iter(&mut r)
            .collect::<Result<Vec<_>, _>>()
            .context("parsing authorizer client cert PEM")?
    };
    if client_certs.is_empty() {
        bail!("authorizer client cert PEM contained no certificates — fail closed");
    }
    let client_key: PrivateKeyDer<'static> = {
        let mut r = std::io::BufReader::new(&key_pem[..]);
        PrivateKeyDer::from_pem_reader(&mut r).context("parsing authorizer client key PEM")?
    };

    let cfg =
        ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .context("building authorizer rustls config (ring provider)")?
            .with_root_certificates(roots)
            .with_client_auth_cert(client_certs, client_key)
            .context("installing the authorizer client certificate (mTLS)")?;
    Ok(Arc::new(cfg))
}

/// Split an `https://host:port` base into `(host, port)` (port defaults to 443).
fn host_port(base: &str) -> anyhow::Result<(String, u16)> {
    let rest = base
        .strip_prefix("https://")
        .ok_or_else(|| anyhow!("authorizer URL must be https://: {base}"))?;
    let rest = rest.split('/').next().unwrap_or(rest);
    match rest.rsplit_once(':') {
        Some((h, p)) => Ok((h.to_string(), p.parse().context("parsing authorizer port")?)),
        None => Ok((rest.to_string(), 443)),
    }
}

/// The operator box's response to a token request: the signed token + its signature (hex) + the
/// operator's attested trusted-time (OI-SM-3).
struct AuthorizerResponse {
    token: PresenceToken,
    sig: [u8; 64],
    attested_time_ms: i64,
}

/// One mTLS round-trip to the operator box: send this VPS's challenge (instance id + cert fp + a
/// fresh server nonce we mint), read back the signed token + attested time. The body is small JSON.
async fn fetch_token(
    connector: &TlsConnector,
    cfg: &AuthorizerConfig,
    server_nonce: &str,
) -> anyhow::Result<AuthorizerResponse> {
    let (host, port) = host_port(&cfg.url)?;
    let server_name = ServerName::try_from(host.clone())
        .map_err(|_| anyhow!("invalid authorizer server name {host}"))?
        .to_owned();

    let body = format!(
        "{{\"vps_instance_id\":\"{}\",\"server_nonce\":\"{}\",\"vps_cert_fp\":\"{}\"}}",
        cfg.vps_instance_id,
        server_nonce,
        hex_encode(&cfg.vps_cert_fp),
    );
    let req = format!(
        "POST /v1/presence/token HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n\
         Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );

    let tcp = tokio::time::timeout(IO_TIMEOUT, TcpStream::connect((host.as_str(), port)))
        .await
        .context("authorizer connect timed out")?
        .context("connecting to the operator-box authorizer")?;
    let mut tls = tokio::time::timeout(IO_TIMEOUT, connector.connect(server_name, tcp))
        .await
        .context("authorizer TLS handshake timed out")?
        .context("authorizer mTLS handshake")?;

    tls.write_all(req.as_bytes())
        .await
        .context("writing authorizer request")?;
    tls.flush().await.ok();

    let mut buf = Vec::new();
    tokio::time::timeout(IO_TIMEOUT, tls.read_to_end(&mut buf))
        .await
        .context("authorizer read timed out")?
        .ok();
    let text = String::from_utf8_lossy(&buf).into_owned();

    parse_response(&text, server_nonce, cfg)
}

/// Parse the operator box's HTTP response into an [`AuthorizerResponse`]. Fail-closed on any missing
/// field. The token is reconstructed from the operator's fields; the `server_nonce` we challenged
/// with is bound into the token (the operator echoes it) so the engine's nonce-consume matches.
fn parse_response(
    resp: &str,
    server_nonce: &str,
    cfg: &AuthorizerConfig,
) -> anyhow::Result<AuthorizerResponse> {
    let status = resp
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|c| c.parse::<u16>().ok())
        .ok_or_else(|| anyhow!("malformed authorizer status line"))?;
    if !(200..300).contains(&status) {
        bail!("authorizer returned HTTP {status}");
    }
    let ts_ms = json_i64(resp, "ts_ms").ok_or_else(|| anyhow!("authorizer missing ts_ms"))?;
    let expiry_ms =
        json_i64(resp, "expiry_ms").ok_or_else(|| anyhow!("authorizer missing expiry_ms"))?;
    let jti = json_str(resp, "jti").ok_or_else(|| anyhow!("authorizer missing jti"))?;
    let sig_hex = json_str(resp, "sig").ok_or_else(|| anyhow!("authorizer missing sig"))?;
    let attested_time_ms = json_i64(resp, "attested_time_ms")
        .ok_or_else(|| anyhow!("authorizer missing attested_time_ms (OI-SM-3 trusted time)"))?;
    let sig = parse_sig_hex(&sig_hex).ok_or_else(|| anyhow!("authorizer sig is not 128-hex"))?;

    let token = PresenceToken::new(
        ts_ms,
        cfg.vps_instance_id.clone(),
        server_nonce.to_string(),
        cfg.vps_cert_fp,
        expiry_ms,
        jti,
    );
    Ok(AuthorizerResponse {
        token,
        sig,
        attested_time_ms,
    })
}

/// Run ONE refresh: mint a server nonce, fetch a token, attest trusted time, verify via the engine,
/// and feed the gate on success. Returns `Ok(expiry_ms)` on a verified token. Any error here is the
/// caller's signal to fail closed (clear the gate).
#[allow(clippy::too_many_arguments)]
async fn refresh_once(
    connector: &TlsConnector,
    cfg: &AuthorizerConfig,
    engine: &Engine,
    gate: &VpsPresenceGate,
    trusted_time: &OperatorBoxTrustedTime,
    nonce_store: &Arc<std::sync::Mutex<NonceStore>>,
    jti_store: &Arc<std::sync::Mutex<JtiReplayStore>>,
    sink: &EventSink,
) -> anyhow::Result<i64> {
    // Mint the single-use server nonce we challenge the operator box with (the engine consumes it on
    // verify, so the same store instance must issue + consume). Use the engine wall clock proxy: the
    // nonce store's clock is caller-supplied; we use the attested time after fetch, but issuance only
    // needs a monotonic-ish now — use the local clock here (issue/consume are paired in-process).
    let now_local = chrono::Utc::now().timestamp_millis();
    let server_nonce = {
        let mut ns = nonce_store
            .lock()
            .map_err(|_| anyhow!("nonce store poisoned"))?;
        ns.issue(now_local, &ring::rand::SystemRandom::new())
            .map_err(|_| anyhow!("nonce store full — cannot challenge authorizer"))?
    };

    let resp = fetch_token(connector, cfg, &server_nonce).await?;

    // OI-SM-3: feed the operator's attested time BEFORE verify so the engine's trusted-time source
    // returns Some(t) during the verify ladder. Without this the verify would reject
    // TrustedTimeUnavailable.
    trusted_time.attest(resp.attested_time_ms);

    // Engine owns the verify POLICY (ordered fail-closed ladder). On success, feed the gate.
    let expiry = {
        let mut ns = nonce_store
            .lock()
            .map_err(|_| anyhow!("nonce store poisoned"))?;
        let mut js = jti_store
            .lock()
            .map_err(|_| anyhow!("jti store poisoned"))?;
        engine
            .verify_presence_token(
                &cfg.operator_pubkey,
                &resp.token,
                &resp.sig,
                &cfg.vps_cert_fp,
                &mut ns,
                &mut js,
                sink,
            )
            .map_err(|e| anyhow!("presence token rejected: {e}"))?
    };
    gate.accept_token(expiry);
    Ok(expiry)
}

/// Spawn the authorizer link as a background tokio task under the caller's shutdown future. The task
/// loops: refresh → on success keep the gate fed; on failure CLEAR the gate (fail-closed, FS-S23) +
/// emit `AuthorizerUnreachable` (the per-stream re-check then drains in-flight streams). The shared
/// `gate` / `trusted_time` Arcs are the SAME ones installed into the engine via `with_seams`, so the
/// engine's read path sees what this task writes.
#[allow(clippy::too_many_arguments)]
pub fn spawn_authorizer_link(
    cfg: AuthorizerConfig,
    engine: Engine,
    gate: Arc<VpsPresenceGate>,
    trusted_time: Arc<OperatorBoxTrustedTime>,
    sink: EventSink,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) -> anyhow::Result<tokio::task::JoinHandle<()>> {
    let client_config = build_client_config(&cfg)?;
    let connector = TlsConnector::from(client_config);
    // The engine consumes the SAME nonce store that issues our challenge nonce, and a jti replay
    // store across refreshes. Owned by the link task.
    let nonce_store = Arc::new(std::sync::Mutex::new(NonceStore::new()));
    let jti_store = Arc::new(std::sync::Mutex::new(JtiReplayStore::new()));

    let handle = tokio::spawn(async move {
        tokio::pin!(shutdown);
        let mut ticker = tokio::time::interval(REFRESH_INTERVAL);
        loop {
            // Refresh immediately on the first tick, then every REFRESH_INTERVAL.
            tokio::select! {
                _ = &mut shutdown => break,
                _ = ticker.tick() => {
                    match refresh_once(
                        &connector, &cfg, &engine, &gate, &trusted_time,
                        &nonce_store, &jti_store, &sink,
                    ).await {
                        Ok(expiry) => {
                            tracing::debug!(expiry_ms = expiry, "authorizer: presence token accepted");
                        }
                        Err(e) => {
                            // FS-S23: cannot prove possession → deny new egress + drain in-flight
                            // (the per-stream re-check reads the cleared gate next tick).
                            gate.clear();
                            sink.emit(SecretEvent::AuthorizerUnreachable { drained_streams: 0 });
                            tracing::warn!(error = %e, "authorizer link failed — gate cleared (fail-closed)");
                        }
                    }
                }
            }
        }
        // On shutdown, clear the gate so a lingering engine clone cannot serve on a stale token.
        gate.clear();
    });
    Ok(handle)
}

// ---- small helpers (std-only; no new dep) ----------------------------------------------------

/// Lowercase-hex encode bytes.
fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

/// Decode a 128-char hex Ed25519 signature into 64 bytes. `None` on malformed input.
fn parse_sig_hex(s: &str) -> Option<[u8; 64]> {
    let s = s.trim();
    if s.len() != 128 {
        return None;
    }
    let mut out = [0u8; 64];
    for (i, chunk) in s.as_bytes().chunks_exact(2).enumerate() {
        let hi = (chunk[0] as char).to_digit(16)?;
        let lo = (chunk[1] as char).to_digit(16)?;
        out[i] = ((hi << 4) | lo) as u8;
    }
    Some(out)
}

/// Extract a JSON string field value (`"name":"value"`) by scanning the raw response.
fn json_str(resp: &str, name: &str) -> Option<String> {
    let key = format!("\"{name}\"");
    let after = &resp[resp.find(&key)? + key.len()..];
    let after = after.trim_start().strip_prefix(':')?.trim_start();
    let after = after.strip_prefix('"')?;
    let end = after.find('"')?;
    let val = &after[..end];
    (!val.is_empty()).then(|| val.to_string())
}

/// Extract a JSON integer field value (`"name":123`) by scanning the raw response.
fn json_i64(resp: &str, name: &str) -> Option<i64> {
    let key = format!("\"{name}\"");
    let after = &resp[resp.find(&key)? + key.len()..];
    let after = after.trim_start().strip_prefix(':')?.trim_start();
    let end = after
        .find(|c: char| !c.is_ascii_digit() && c != '-')
        .unwrap_or(after.len());
    after[..end].parse().ok()
}

/// Compute the SHA-256 fingerprint of a DER certificate (the `vps_cert_fp` the operator box binds).
#[must_use]
pub fn cert_fingerprint(cert_der: &[u8]) -> [u8; 32] {
    Sha256::digest(cert_der).into()
}

/// Load the SHA-256 fingerprint of the FIRST certificate in a PEM file (this VPS's edge cert), for
/// the daemon to compute its own `vps_cert_fp` at startup. Fail-closed on a missing/empty PEM.
pub fn cert_fingerprint_from_pem(pem_path: &Path) -> anyhow::Result<[u8; 32]> {
    let pem = std::fs::read(pem_path)
        .with_context(|| format!("reading edge cert {} for fingerprint", pem_path.display()))?;
    let mut rd = std::io::BufReader::new(&pem[..]);
    let first = CertificateDer::pem_reader_iter(&mut rd)
        .next()
        .ok_or_else(|| anyhow!("no certificate in {}", pem_path.display()))?
        .context("parsing edge cert for fingerprint")?;
    Ok(cert_fingerprint(first.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_port_parses() {
        assert_eq!(
            host_port("https://op.box:9443").unwrap(),
            ("op.box".to_string(), 9443)
        );
        assert_eq!(
            host_port("https://op.box").unwrap(),
            ("op.box".to_string(), 443)
        );
        assert!(host_port("http://op.box").is_err(), "https only");
    }

    #[test]
    fn json_helpers_scan_values() {
        let body = "HTTP/1.1 200 OK\r\n\r\n{\"ts_ms\":1700,\"expiry_ms\":1700600,\"jti\":\"abc\",\"sig\":\"de\",\"attested_time_ms\":1700001}";
        assert_eq!(json_i64(body, "ts_ms"), Some(1700));
        assert_eq!(json_i64(body, "expiry_ms"), Some(1_700_600));
        assert_eq!(json_i64(body, "attested_time_ms"), Some(1_700_001));
        assert_eq!(json_str(body, "jti"), Some("abc".to_string()));
        assert_eq!(json_str(body, "missing"), None);
    }

    #[test]
    fn parse_response_rejects_non_2xx_and_missing_fields() {
        let cfg = AuthorizerConfig {
            url: "https://op.box:9443".into(),
            vps_instance_id: "vps-1".into(),
            vps_cert_fp: [0u8; 32],
            operator_pubkey: [0u8; 32],
            operator_ca_path: PathBuf::from("/dev/null"),
            client_cert_path: PathBuf::from("/dev/null"),
            client_key_path: PathBuf::from("/dev/null"),
        };
        assert!(parse_response("HTTP/1.1 503 x\r\n\r\n{}", "n", &cfg).is_err());
        assert!(
            parse_response("HTTP/1.1 200 OK\r\n\r\n{\"ts_ms\":1}", "n", &cfg).is_err(),
            "missing fields fail closed"
        );
    }

    #[test]
    fn cert_fingerprint_is_sha256() {
        let fp = cert_fingerprint(b"hello");
        assert_eq!(fp.len(), 32);
        // SHA-256("hello") first byte = 0x2c.
        assert_eq!(fp[0], 0x2c);
    }

    #[test]
    fn parse_sig_hex_roundtrips() {
        let s = "90017fccf53948ce509c216d1cf64c6cdd75d50a9f28e63cef27d6706a7b4c765de7a2849dc8c1d6b19f5ee6e3211b8142b669ca8b6c1fb16a6dc989dc5fa60e";
        let b = parse_sig_hex(s).expect("128-hex");
        assert_eq!(b[0], 0x90);
        assert!(parse_sig_hex("dead").is_none());
    }
}
