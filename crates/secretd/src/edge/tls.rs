//! The remote relay edge's server-TLS config (F2 / TASK-0031). [`RelayTlsConfig`] is a newtype that
//! can ONLY build a rustls [`ServerConfig`] from the operator-provisioned, publicly-trusted relay
//! cert under `paths.relay_tls_dir()` (`~/.config/env-ctl/relay-tls/{cert.pem,key.pem}`).
//!
//! ## FS-S18 / FS-S25 (structural, not grep-only)
//! This module is the structural enforcement of "the relay edge cert is NEVER the local MITM CA":
//! it imports NO MITM-CA type and references NO MITM-CA path — the ONLY source it can read is the
//! `relay_tls_dir()`. There is no code path here that can load a leaf minted by the engine's MITM CA.
//! A CI grep (`ci/gates/shape.sh`) is retained as defense-in-depth, but the load-bearing guarantee is
//! that this loader's sole input is the relay-tls directory.
//!
//! ## Fail-closed
//! A missing directory, a missing/empty `cert.pem` or `key.pem`, an unparsable PEM, or a
//! ServerConfig build error all return `Err` — the edge then refuses to start (the public listener
//! NEVER binds with a half-built or absent cert). There is NO fallback to any other cert source.
//!
//! ring-only crypto provider (the single pinned rustls 0.23, never aws-lc-rs). No client auth in
//! PR-1 (mTLS hardened mode is PR-2). Zero new deps — rustls/rustls-pemfile are already linked.

use std::path::Path;
use std::sync::Arc;

use anyhow::{bail, Context};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig};

/// The relay edge's server-TLS configuration, loaded ONLY from the relay-tls directory. Wraps the
/// built rustls [`ServerConfig`] so the edge listener never constructs a server config from any other
/// source (the MITM-CA path is not reachable from this type — FS-S25 structural).
pub struct RelayTlsConfig(Arc<ServerConfig>);

impl RelayTlsConfig {
    /// Load the relay server cert + key from `relay_tls_dir/{cert.pem,key.pem}` and build a ring-only
    /// rustls `ServerConfig` with NO client auth (the PR-1 default). Fail-closed: any missing/empty/
    /// unparsable input is an `Err`.
    ///
    /// `relay_tls_dir` MUST be the value of `Paths::relay_tls_dir()` — the caller passes it so this
    /// module never computes or references any other path (it cannot reach the MITM-CA location).
    pub fn load_from_dir(relay_tls_dir: &Path) -> anyhow::Result<Self> {
        Self::load_from_dir_with_client_auth(relay_tls_dir, None)
    }

    /// PR-2b: load the relay server cert + key (as [`load_from_dir`](Self::load_from_dir)) and build
    /// the ring-only rustls `ServerConfig`, OPTIONALLY requiring a verified client certificate (mTLS).
    ///
    /// When `client_ca_path` is `None` the config uses `.with_no_client_auth()` BYTE-FOR-BYTE as PR-1
    /// (the default-OFF behavior is unchanged). When `Some(ca)`, the SAME relay-tls ServerConfig
    /// additionally requires a client cert verified against the operator-provisioned remote-clients-CA
    /// PEM at `ca` (SERVER-MODE §6.1) — a SEPARATE config input, NEVER the MITM CA and NEVER the server
    /// cert (FS-S25 preserved: this loader still reads no MITM-CA path). The verifier is built with the
    /// EXPLICIT ring provider (`WebPkiClientVerifier::builder_with_provider(roots, ring)`), so the edge
    /// stays single-rustls / ring-only (no aws-lc).
    ///
    /// Fail-closed: a missing/empty/unparsable client-CA PEM, an empty trust-anchor set, or a verifier
    /// build error is an `Err` (the edge refuses to start rather than serve a half-built mTLS gate).
    pub fn load_from_dir_with_client_auth(
        relay_tls_dir: &Path,
        client_ca_path: Option<&Path>,
    ) -> anyhow::Result<Self> {
        let cert_path = relay_tls_dir.join("cert.pem");
        let key_path = relay_tls_dir.join("key.pem");

        if !relay_tls_dir.is_dir() {
            bail!(
                "relay-tls directory {} is absent — the remote edge fails closed (provision a \
                 publicly-trusted cert there: cert.pem + key.pem)",
                relay_tls_dir.display()
            );
        }

        let cert_pem = std::fs::read(&cert_path)
            .with_context(|| format!("reading relay edge cert {}", cert_path.display()))?;
        let key_pem = std::fs::read(&key_path)
            .with_context(|| format!("reading relay edge key {}", key_path.display()))?;

        let certs: Vec<CertificateDer<'static>> = {
            let mut rd = std::io::BufReader::new(&cert_pem[..]);
            rustls_pemfile::certs(&mut rd)
                .collect::<Result<Vec<_>, _>>()
                .context("parsing relay edge cert.pem")?
        };
        if certs.is_empty() {
            bail!(
                "relay edge cert.pem at {} contained no certificates",
                cert_path.display()
            );
        }

        let key: PrivateKeyDer<'static> = {
            let mut rd = std::io::BufReader::new(&key_pem[..]);
            rustls_pemfile::private_key(&mut rd)
                .context("parsing relay edge key.pem")?
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "relay edge key.pem at {} contained no private key",
                        key_path.display()
                    )
                })?
        };

        // ring-only provider + safe protocol versions are common to both modes. A build error (e.g.
        // cert/key mismatch, unsupported key) fails closed.
        let builder =
            ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
                .with_safe_default_protocol_versions()
                .context("ring safe protocol versions for the relay edge")?;

        let config = match client_ca_path {
            // Default (PR-1): no client auth. BYTE-FOR-BYTE the prior behavior.
            None => builder
                .with_no_client_auth()
                .with_single_cert(certs, key)
                .context("building the relay edge ServerConfig (cert/key mismatch?)")?,
            // PR-2b mTLS: require a client cert verified against the remote-clients-CA bundle. Same
            // relay-tls ServerConfig, an ADDITIONAL gate; the explicit ring provider keeps it ring-only.
            Some(ca) => {
                let roots = load_client_ca_roots(ca)?;
                let verifier = WebPkiClientVerifier::builder_with_provider(
                    Arc::new(roots),
                    Arc::new(rustls::crypto::ring::default_provider()),
                )
                .build()
                .context("building the relay edge client-cert verifier (remote-clients-CA)")?;
                builder
                    .with_client_cert_verifier(verifier)
                    .with_single_cert(certs, key)
                    .context("building the relay edge mTLS ServerConfig (cert/key mismatch?)")?
            }
        };

        Ok(RelayTlsConfig(Arc::new(config)))
    }

    /// The built `ServerConfig`, ready to hand to a `tokio_rustls::TlsAcceptor`.
    pub fn server_config(&self) -> Arc<ServerConfig> {
        Arc::clone(&self.0)
    }
}

/// PR-2b: load the operator-provisioned remote-clients-CA bundle (PEM) into a rustls `RootCertStore`
/// (the trust anchors a presented client cert is verified against). The SAME `rustls_pemfile::certs`
/// path the server cert uses — a SEPARATE input from the relay-tls cert, NEVER the MITM CA. Fail-
/// closed: a missing/empty/unparsable file, or a file with zero usable anchors, is an `Err`.
fn load_client_ca_roots(client_ca_path: &Path) -> anyhow::Result<RootCertStore> {
    let anchors_pem = std::fs::read(client_ca_path).with_context(|| {
        format!(
            "reading remote-clients-CA bundle {}",
            client_ca_path.display()
        )
    })?;
    let anchors: Vec<CertificateDer<'static>> = {
        let mut rd = std::io::BufReader::new(&anchors_pem[..]);
        rustls_pemfile::certs(&mut rd)
            .collect::<Result<Vec<_>, _>>()
            .context("parsing the remote-clients-CA PEM")?
    };
    if anchors.is_empty() {
        bail!(
            "remote-clients-CA bundle at {} contained no certificates (mTLS fails closed)",
            client_ca_path.display()
        );
    }
    let mut roots = RootCertStore::empty();
    let (added, _ignored) = roots.add_parsable_certificates(anchors);
    if added == 0 {
        bail!(
            "remote-clients-CA bundle at {} had no usable trust anchors (mTLS fails closed)",
            client_ca_path.display()
        );
    }
    Ok(roots)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Generate a self-signed relay cert+key PEM pair into `dir` (cert.pem + key.pem). Uses rcgen,
    /// already in the resolved graph (engine's MITM CA). Self-signed is fine for the LOADER test —
    /// we are proving the loader builds a ServerConfig from the relay-tls path, not validating a
    /// chain (the public-root chain check is a startup self-check, PR-2).
    fn write_relay_cert(dir: &std::path::Path) {
        // rcgen 0.13: generate a simple self-signed cert for a test SAN.
        let cert = rcgen::generate_simple_self_signed(vec!["edge.example".to_string()])
            .expect("generate self-signed relay cert");
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join("cert.pem"), cert.cert.pem()).unwrap();
        std::fs::write(dir.join("key.pem"), cert.key_pair.serialize_pem()).unwrap();
    }

    fn install_ring() {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }

    #[test]
    fn loads_relay_tls_dir() {
        install_ring();
        let tmp = tempdir();
        let dir = tmp.join("relay-tls");
        write_relay_cert(&dir);
        let cfg = RelayTlsConfig::load_from_dir(&dir).expect("load relay cert");
        // The built ServerConfig is usable (Arc clone succeeds; no client auth assertion is implicit).
        let _ = cfg.server_config();
    }

    #[test]
    fn missing_dir_fails_closed() {
        install_ring();
        let tmp = tempdir();
        let dir = tmp.join("does-not-exist");
        let err = match RelayTlsConfig::load_from_dir(&dir) {
            Ok(_) => panic!("a missing relay-tls dir must fail closed"),
            Err(e) => e.to_string(),
        };
        assert!(err.contains("absent"), "unexpected error: {err}");
    }

    #[test]
    fn missing_key_fails_closed() {
        install_ring();
        let tmp = tempdir();
        let dir = tmp.join("relay-tls");
        std::fs::create_dir_all(&dir).unwrap();
        // cert present but no key.pem.
        let cert = rcgen::generate_simple_self_signed(vec!["edge.example".to_string()]).unwrap();
        std::fs::write(dir.join("cert.pem"), cert.cert.pem()).unwrap();
        assert!(RelayTlsConfig::load_from_dir(&dir).is_err());
    }

    #[test]
    fn empty_cert_fails_closed() {
        install_ring();
        let tmp = tempdir();
        let dir = tmp.join("relay-tls");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("cert.pem"), b"").unwrap();
        let cert = rcgen::generate_simple_self_signed(vec!["edge.example".to_string()]).unwrap();
        std::fs::write(dir.join("key.pem"), cert.key_pair.serialize_pem()).unwrap();
        assert!(RelayTlsConfig::load_from_dir(&dir).is_err());
    }

    // ---- PR-2b mTLS loader ----------------------------------------------------------------------

    #[test]
    fn mtls_loads_with_client_ca() {
        install_ring();
        let tmp = tempdir();
        let dir = tmp.join("relay-tls");
        write_relay_cert(&dir);
        // A separate client-CA PEM (NOT the relay cert, NOT the MITM CA) — the trust input for mTLS.
        let ca = rcgen::generate_simple_self_signed(vec!["clients-ca".to_string()]).unwrap();
        let ca_path = tmp.join("clients-ca.pem");
        std::fs::write(&ca_path, ca.cert.pem()).unwrap();
        let cfg = RelayTlsConfig::load_from_dir_with_client_auth(&dir, Some(&ca_path))
            .expect("mTLS ServerConfig builds with a client-CA");
        let _ = cfg.server_config();
    }

    #[test]
    fn mtls_missing_client_ca_file_fails_closed() {
        install_ring();
        let tmp = tempdir();
        let dir = tmp.join("relay-tls");
        write_relay_cert(&dir);
        let ca_path = tmp.join("does-not-exist-clients-ca.pem");
        assert!(
            RelayTlsConfig::load_from_dir_with_client_auth(&dir, Some(&ca_path)).is_err(),
            "a missing client-CA bundle must fail closed"
        );
    }

    #[test]
    fn mtls_empty_client_ca_fails_closed() {
        install_ring();
        let tmp = tempdir();
        let dir = tmp.join("relay-tls");
        write_relay_cert(&dir);
        let ca_path = tmp.join("empty-clients-ca.pem");
        std::fs::write(&ca_path, b"").unwrap();
        assert!(
            RelayTlsConfig::load_from_dir_with_client_auth(&dir, Some(&ca_path)).is_err(),
            "an empty client-CA bundle (no anchors) must fail closed"
        );
    }

    #[test]
    fn no_client_ca_matches_pr1_default() {
        // `load_from_dir` delegates to `..._with_client_auth(dir, None)` — the no-client-auth default
        // path still builds (byte-for-byte PR-1 behavior; no client cert required).
        install_ring();
        let tmp = tempdir();
        let dir = tmp.join("relay-tls");
        write_relay_cert(&dir);
        assert!(RelayTlsConfig::load_from_dir_with_client_auth(&dir, None).is_ok());
    }

    /// A minimal tempdir helper (no `tempfile` dep): a unique path under the OS temp dir, created on
    /// demand and cleaned best-effort. Returns a guard whose Drop removes the tree.
    fn tempdir() -> TempGuard {
        let mut p = std::env::temp_dir();
        let uniq = format!(
            "envctl-relay-tls-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        p.push(uniq);
        std::fs::create_dir_all(&p).unwrap();
        TempGuard(p)
    }

    struct TempGuard(std::path::PathBuf);
    impl std::ops::Deref for TempGuard {
        type Target = std::path::Path;
        fn deref(&self) -> &std::path::Path {
            &self.0
        }
    }
    impl Drop for TempGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}
