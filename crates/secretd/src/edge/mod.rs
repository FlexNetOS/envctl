//! The F2 remote relay-edge plane (TASK-0031 PR-1) — the public HTTPS edge that lets a registered,
//! DPoP-sender-constrained remote thin client reach EXACTLY the engine's `relay_swap` (NEW-2: no
//! control plane, scoped/≤24h egress only). Gated behind the `relay-edge` cargo feature and OFF by
//! default; `[edge].enabled` in `secretd.toml` is the runtime switch. When the feature is off OR the
//! `[edge]` block is absent/disabled, NO public listener is bound (a stock secretd serves no edge).
//!
//! ## Invariants enforced here / in the submodules
//! - `tls.rs` ([`tls::RelayTlsConfig`]): the edge server cert comes ONLY from `relay_tls_dir()` —
//!   never the MITM CA (FS-S18 / FS-S25, structural).
//! - `dpop.rs` ([`dpop::verify_dpop_proof`]): pure RFC 9449 verification + EKM channel binding
//!   (FS-S20); fail-closed on every malformed/unbound/replayed proof.
//! - `listener.rs`: terminates TLS in-process, computes EKM, runs the fail-closed verify ladder, then
//!   drives the SAME `proxy::swap_and_respond` core as the local proxy (MINT + DECIDE stay in the
//!   engine; the edge does I/O + proof verification only — no policy in the edge, no `println!`).
//!
//! The whole plane is config-gated: the *presence* of an enabled `[edge]` block is the `--apply`
//! analogue for this network listener (a destructive-surface guard by construction). A cert-load or
//! bind failure when the edge is explicitly enabled is FATAL (the caller propagates the `Err`).

pub mod dpop;
pub mod listener;
pub mod stream;
pub mod tls;

use std::net::SocketAddr;

use envctl_secrets::paths::Paths;
use envctl_secrets::Engine;

/// Resolved, validated remote-edge configuration (parsed from `[edge]` in `secretd.toml`).
#[derive(Debug, Clone)]
pub struct EdgeConfig {
    /// Whether the public remote edge is served at all. `false` (or an absent `[edge]` block) ⇒ no
    /// listener is bound.
    pub enabled: bool,
    /// The socket address the edge binds. Unlike the loopback proxy, the edge is intended to be
    /// reachable remotely (publicly-trusted TLS); the operator chooses the bind (e.g. `0.0.0.0:8443`
    /// behind an L4 front, or a loopback that a reverse tunnel forwards to — SERVER-MODE §6.5).
    pub bind_addr: SocketAddr,
    /// TASK-0032 / FS-S5 streaming re-check cadence + lifetime cap. `None` ⇒ the production default
    /// ([`stream::Timing::production`]: 2s re-check, 300s cap). A test-only override (so the e2e closes
    /// a stream within seconds rather than sleeping the production interval); the daemon always passes
    /// `None`.
    pub recheck_timing: Option<stream::Timing>,
}

/// Start the remote relay edge as a tokio task under the caller's shutdown future. Loads the
/// relay-tls cert (fail-closed), binds `cfg.bind_addr`, and serves `POST /v1/relay/swap`. Returns the
/// bound address + the serving task handle.
///
/// Errors (propagated to the caller, which makes them FATAL when the edge is explicitly enabled):
/// a missing/invalid relay-tls cert, or a bind failure. The engine carries the same `DaemonUpstream`
/// seam as the proxy, so the edge drives the identical swap core.
pub async fn serve_edge(
    engine: Engine,
    paths: &Paths,
    cfg: &EdgeConfig,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) -> anyhow::Result<(SocketAddr, tokio::task::JoinHandle<()>)> {
    let relay_tls_dir = paths.relay_tls_dir();
    let timing = cfg
        .recheck_timing
        .unwrap_or_else(stream::Timing::production);
    listener::serve_edge_listener(engine, &relay_tls_dir, cfg.bind_addr, timing, shutdown).await
}
