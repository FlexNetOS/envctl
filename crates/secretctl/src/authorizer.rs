//! Operator-box presence-token SIGNER (SERVER-MODE Profile B / OI-SM-2, TASK-0033 U15).
//!
//! This is the OPERATOR-box half of the Profile-B protocol: the box that physically holds the
//! USB/Seed runs `secretctl authorizer serve`, which listens for a remote VPS's challenge over
//! **mTLS** (ring-only) and returns a freshly-signed [`PresenceToken`] + the operator's attested
//! trusted time. The token signing primitive is the engine's
//! [`envctl_secrets::sign_presence_token`] — the CLI owns ONLY the thin mTLS server I/O + the seed
//! file read; it never reimplements the crypto.
//!
//! Invariants:
//! - **mTLS required (fail-closed).** The signer REFUSES to start without a client-CA (it never
//!   signs for an unauthenticated VPS). Ring-only rustls (no aws-lc), single pinned rustls.
//! - **Seed never leaves the process.** The Ed25519 seed is read into [`Zeroizing`] and wiped.
//! - **Thin.** Policy (token format / signing bytes) lives in the engine; this is I/O glue.

use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, bail, Context};
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use zeroize::Zeroizing;

use envctl_secrets::broker::PresenceToken;
use envctl_secrets::sign_presence_token;

use crate::cli::AuthorizerCmd;

/// Audited token TTL band (seconds): 5–15 min. The signer clamps `--ttl-secs` into this band.
const TTL_MIN_SECS: i64 = 300;
const TTL_MAX_SECS: i64 = 900;

/// Dispatch the `authorizer` subcommand.
pub async fn authorizer(cmd: AuthorizerCmd, json: bool) -> anyhow::Result<()> {
    match cmd {
        AuthorizerCmd::Status => status(json),
        AuthorizerCmd::Serve {
            bind,
            seed_file,
            cert,
            key,
            client_ca,
            ttl_secs,
        } => {
            serve(
                &bind,
                Path::new(&seed_file),
                Path::new(&cert),
                Path::new(&key),
                Path::new(&client_ca),
                ttl_secs,
            )
            .await
        }
    }
}

/// `authorizer status` — local, read-only inspection of the `[profile]` topology. Reads
/// `secretd.toml` directly (no daemon round-trip): a VPS prints whether its operator-authorizer link
/// is configured; an on-box (default) deployment says so.
fn status(json: bool) -> anyhow::Result<()> {
    // Minimal local config probe: look for [profile] in the default config path (or $SECRETD_CONFIG).
    let cfg_path = std::env::var_os("SECRETD_CONFIG")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            let home = std::env::var_os("HOME")?;
            Some(std::path::PathBuf::from(home).join(".config/env-ctl/secretd.toml"))
        });
    let (topology, has_url) = match cfg_path
        .as_deref()
        .and_then(|p| std::fs::read_to_string(p).ok())
    {
        Some(text) => {
            let topo = text
                .lines()
                .find_map(|l| l.trim().strip_prefix("topology"))
                .map(|v| {
                    let v = v.trim_start_matches([' ', '=']).trim().trim_matches('"');
                    v.to_string()
                })
                .unwrap_or_else(|| "onbox".to_string());
            let has_url = text.contains("operator_authorizer_url");
            (topo, has_url)
        }
        None => ("onbox".to_string(), false),
    };
    let is_vps = matches!(topology.to_ascii_lowercase().as_str(), "remote" | "vps");
    if json {
        println!(
            "{}",
            serde_json::json!({
                "topology": topology,
                "profile": if is_vps { "B" } else { "A" },
                "operator_authorizer_configured": has_url,
            })
        );
    } else {
        let profile = if is_vps { "B (VPS)" } else { "A (on-box)" };
        println!("topology: {topology}  (Profile {profile})");
        if is_vps {
            println!(
                "operator-authorizer link: {}",
                if has_url {
                    "configured"
                } else {
                    "NOT configured (FS-S21 will refuse start)"
                }
            );
        }
    }
    Ok(())
}

/// `authorizer serve` — the operator-box mTLS signer loop.
async fn serve(
    bind: &str,
    seed_file: &Path,
    cert_path: &Path,
    key_path: &Path,
    client_ca_path: &Path,
    ttl_secs: i64,
) -> anyhow::Result<()> {
    let _ = rustls::crypto::ring::default_provider().install_default();

    let seed = read_seed(seed_file)?;
    let ttl_ms = ttl_secs.clamp(TTL_MIN_SECS, TTL_MAX_SECS) * 1000;
    let acceptor = build_acceptor(cert_path, key_path, client_ca_path)?;

    let listener = TcpListener::bind(bind)
        .await
        .with_context(|| format!("binding the operator authorizer at {bind}"))?;
    eprintln!("operator authorizer signing for VPS clients at {bind} (mTLS, ring-only)");

    loop {
        let (tcp, peer) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("accept error: {e}");
                continue;
            }
        };
        let acceptor = acceptor.clone();
        let seed = seed.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_conn(acceptor, tcp, &seed, ttl_ms).await {
                eprintln!("authorizer connection from {peer} failed: {e}");
            }
        });
    }
}

/// Handle one VPS connection: mTLS handshake, read the challenge, sign a token, respond.
async fn handle_conn(
    acceptor: TlsAcceptor,
    tcp: tokio::net::TcpStream,
    seed: &Zeroizing<[u8; 32]>,
    ttl_ms: i64,
) -> anyhow::Result<()> {
    let mut tls = acceptor.accept(tcp).await.context("mTLS handshake")?;

    // Read the request (small; read until headers+body or EOF up to a cap).
    let mut buf = vec![0u8; 8192];
    let n = tls.read(&mut buf).await.context("reading challenge")?;
    let req = String::from_utf8_lossy(&buf[..n]).into_owned();

    let vps_instance_id = json_str(&req, "vps_instance_id")
        .ok_or_else(|| anyhow!("challenge missing vps_instance_id"))?;
    let server_nonce =
        json_str(&req, "server_nonce").ok_or_else(|| anyhow!("challenge missing server_nonce"))?;
    let cert_fp_hex =
        json_str(&req, "vps_cert_fp").ok_or_else(|| anyhow!("challenge missing vps_cert_fp"))?;
    let vps_cert_fp =
        parse_fp_hex(&cert_fp_hex).ok_or_else(|| anyhow!("vps_cert_fp not 64-hex"))?;

    // The operator box's trusted wall clock IS authoritative (it holds the USB; OI-SM-3 trusted
    // time originates here). Mint a token valid for ttl_ms.
    let now_ms = now_epoch_ms();
    let jti = format!("{now_ms}-{}", &server_nonce[..server_nonce.len().min(16)]);
    let tok = PresenceToken::new(
        now_ms,
        vps_instance_id,
        server_nonce,
        vps_cert_fp,
        now_ms + ttl_ms,
        jti.clone(),
    );
    let sig = sign_presence_token(seed, &tok).context("signing presence token")?;

    let body = serde_json::json!({
        "ts_ms": tok.ts_ms,
        "expiry_ms": tok.expiry_ms,
        "jti": tok.jti,
        "sig": hex_encode(&sig),
        "attested_time_ms": now_ms,
    })
    .to_string();
    let resp = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\n\
         Content-Length: {}\r\n\r\n{body}",
        body.len()
    );
    tls.write_all(resp.as_bytes())
        .await
        .context("writing token")?;
    tls.flush().await.ok();
    Ok(())
}

/// Read the Ed25519 signing seed: 32 raw bytes OR 64 hex chars. Held in `Zeroizing`.
fn read_seed(path: &Path) -> anyhow::Result<Zeroizing<[u8; 32]>> {
    let raw =
        std::fs::read(path).with_context(|| format!("reading operator seed {}", path.display()))?;
    let mut out = Zeroizing::new([0u8; 32]);
    if raw.len() == 32 {
        out.copy_from_slice(&raw);
        return Ok(out);
    }
    // Try 64-hex (trim trailing whitespace/newline).
    let txt = String::from_utf8_lossy(&raw);
    let txt = txt.trim();
    if txt.len() == 64 {
        for (i, chunk) in txt.as_bytes().as_chunks::<2>().0.iter().enumerate() {
            let hi = (chunk[0] as char)
                .to_digit(16)
                .ok_or_else(|| anyhow!("seed file is not valid hex"))?;
            let lo = (chunk[1] as char)
                .to_digit(16)
                .ok_or_else(|| anyhow!("seed file is not valid hex"))?;
            out[i] = ((hi << 4) | lo) as u8;
        }
        return Ok(out);
    }
    bail!(
        "operator seed must be 32 raw bytes or 64 hex chars (got {} bytes)",
        raw.len()
    );
}

/// Build the ring-only mTLS server acceptor. Client-CA is REQUIRED (fail-closed: never sign for an
/// unauthenticated VPS).
fn build_acceptor(
    cert_path: &Path,
    key_path: &Path,
    client_ca_path: &Path,
) -> anyhow::Result<TlsAcceptor> {
    let cert_pem = std::fs::read(cert_path)
        .with_context(|| format!("reading server cert {}", cert_path.display()))?;
    let key_pem = std::fs::read(key_path)
        .with_context(|| format!("reading server key {}", key_path.display()))?;
    let ca_pem = std::fs::read(client_ca_path)
        .with_context(|| format!("reading client CA {}", client_ca_path.display()))?;

    let certs: Vec<CertificateDer<'static>> = {
        let mut r = std::io::BufReader::new(&cert_pem[..]);
        CertificateDer::pem_reader_iter(&mut r)
            .collect::<Result<Vec<_>, _>>()
            .context("parsing server cert")?
    };
    if certs.is_empty() {
        bail!("server cert PEM had no certificates");
    }
    let key: PrivateKeyDer<'static> = {
        let mut r = std::io::BufReader::new(&key_pem[..]);
        PrivateKeyDer::from_pem_reader(&mut r).context("parsing server key")?
    };

    let mut roots = RootCertStore::empty();
    let mut r = std::io::BufReader::new(&ca_pem[..]);
    let mut added = 0usize;
    for c in CertificateDer::pem_reader_iter(&mut r) {
        roots
            .add(c.context("parsing client CA")?)
            .context("adding client CA")?;
        added += 1;
    }
    if added == 0 {
        bail!("client CA PEM had no certificates — the signer fails closed (mTLS required)");
    }
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let verifier = WebPkiClientVerifier::builder_with_provider(Arc::new(roots), provider.clone())
        .build()
        .context("building mTLS client verifier (ring)")?;
    let config = ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .context("building server config (ring)")?
        .with_client_cert_verifier(verifier)
        .with_single_cert(certs, key)
        .context("installing operator server cert")?;
    Ok(TlsAcceptor::from(Arc::new(config)))
}

// ---- helpers ---------------------------------------------------------------------------------

/// Current wall-clock epoch milliseconds (std-only; secretctl pulls no `chrono`).
fn now_epoch_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

fn parse_fp_hex(s: &str) -> Option<[u8; 32]> {
    let s = s.trim();
    if s.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, chunk) in s.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        let hi = (chunk[0] as char).to_digit(16)?;
        let lo = (chunk[1] as char).to_digit(16)?;
        out[i] = ((hi << 4) | lo) as u8;
    }
    Some(out)
}

fn json_str(resp: &str, name: &str) -> Option<String> {
    let key = format!("\"{name}\"");
    let after = &resp[resp.find(&key)? + key.len()..];
    let after = after.trim_start().strip_prefix(':')?.trim_start();
    let after = after.strip_prefix('"')?;
    let end = after.find('"')?;
    let val = &after[..end];
    (!val.is_empty()).then(|| val.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_seed_accepts_hex_and_raw() {
        let dir = std::env::temp_dir().join(format!("authz-seed-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let hexf = dir.join("hex");
        std::fs::write(&hexf, "ab".repeat(32)).unwrap();
        let s = read_seed(&hexf).unwrap();
        assert_eq!(s[0], 0xab);
        let rawf = dir.join("raw");
        std::fs::write(&rawf, [7u8; 32]).unwrap();
        let s2 = read_seed(&rawf).unwrap();
        assert_eq!(s2[0], 7);
        let badf = dir.join("bad");
        std::fs::write(&badf, b"short").unwrap();
        assert!(read_seed(&badf).is_err());
    }

    #[test]
    fn json_str_scans() {
        let body = "{\"vps_instance_id\":\"vps-1\",\"server_nonce\":\"abc\"}";
        assert_eq!(json_str(body, "vps_instance_id"), Some("vps-1".to_string()));
        assert_eq!(json_str(body, "missing"), None);
    }

    #[test]
    fn parse_fp_hex_validates() {
        assert!(parse_fp_hex(&"00".repeat(32)).is_some());
        assert!(parse_fp_hex("dead").is_none());
    }
}
