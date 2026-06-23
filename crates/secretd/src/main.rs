//! secretd — the env-ctl control-plane daemon (gRPC over a Unix-domain socket).
//!
//! `main` stays SYNC and builds a multi-thread runtime explicitly so process hardening runs BEFORE
//! any async/key path (ops doc §3). Bring-up order inside [`serve`]:
//!   1. install the rustls RING `CryptoProvider` (CF-2) — never aws-lc-rs;
//!   2. process hardening (FS-S4): `RLIMIT_CORE=0` + `RLIMIT_MEMLOCK` raised + in-process
//!      `mlockall(MCL_CURRENT|MCL_FUTURE)` so secret material (DEK / vault plaintext / PEMs, all
//!      allocated post-`Lock.Unlock`) can never reach swap. (See the NOTE below on the best-effort
//!      vs. `require_mlock` strict behavior; systemd `LimitMEMLOCK`/`LimitCORE` are the
//!      defense-in-depth backstop.)
//!   3. `Paths::resolve()` + create runtime/data/state dirs `0700`;
//!   4. `Engine::open(paths)`; first-run bootstrap leaves vault init out-of-band (no `Vault.Init`
//!      RPC; the vault stays Locked until an explicit `Lock.Unlock`);
//!   5. bind the UDS `0600` (stale-socket reaped), serve the gRPC services behind the SO_PEERCRED
//!      owner interceptor with graceful shutdown on SIGINT/SIGTERM.
//!
//! NOTE (FS-S4 mlockall behavior): `mlockall(MCL_CURRENT|MCL_FUTURE)` is called in-process via the
//! pure-Rust `libc` FFI bindings (chosen over a rustix `mm` feature widen — `libc` is already in
//! secretd's resolved tree and adds no new lockfile crate, and `mm` would broaden the SHARED rustix
//! pin for every consumer). `MCL_FUTURE` is load-bearing: it covers the DEK / vault-plaintext / PEM
//! allocations that happen AFTER startup (post-unlock), not just the pages mapped at startup. The
//! call is **Linux-gated** (`#[cfg(target_os = "linux")]`) so non-Linux dev builds compile.
//!
//! It is **best-effort by default**: `mlockall` commonly fails `EPERM` (no `CAP_IPC_LOCK`) or
//! `ENOMEM`; on failure the daemon logs a metadata-only WARN (errno/strerror — never secret bytes,
//! of which there are none pre-unlock anyway) and CONTINUES, relying on `RLIMIT_CORE=0` + the
//! systemd unit's `LimitMEMLOCK=infinity` / `LimitCORE=0`. An operator can opt into a hardened mode
//! via `[security].require_mlock = true` (or `SECRETD_REQUIRE_MLOCK=1`): when set, a failed
//! `mlockall` is FATAL and `serve` refuses to start (fail-closed). `--self-check` keeps `mlockall`
//! best-effort regardless of `require_mlock` (it is a non-serving pre-flight).
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anyhow::Context;
use clap::Parser;
use envctl_secretd::{config, peercred, server};
use envctl_secrets::paths::Paths;
use envctl_secrets::Engine;
use rustix::process::{setrlimit, Resource, Rlimit};

/// secretd — the env-ctl control-plane secrets daemon (gRPC over a Unix-domain socket).
///
/// With no flags it serves the control plane (the systemd `ExecStart` path). The one option is the
/// non-serving health probe used by the envctl manifest `verify` hook.
#[derive(Parser)]
#[command(
    name = "secretd",
    version,
    about = "env-ctl control-plane secrets daemon (gRPC over a UDS)"
)]
struct Cli {
    /// Run startup pre-flight checks (ring crypto provider, XDG paths, store config) and EXIT,
    /// without binding the control socket or serving. Exit 0 = the daemon could come up here; a
    /// non-zero exit names the reason it would fail to start. Safe to run alongside a live daemon —
    /// it never binds the socket, connects the store, or mutates the vault.
    #[arg(long = "self-check")]
    self_check: bool,
    /// TASK-0033 (FS-S22): allow an on-box (Profile A) daemon to serve passphrase-only — i.e. start
    /// even when a USB keyslot is enrolled but USB possession is currently unproven. Without this,
    /// an on-box daemon with an enrolled-but-unproven USB keyslot REFUSES to start (the gate would
    /// back nothing). Has no effect on a VPS (Profile B) or a vault with no USB keyslot.
    #[arg(long = "allow-passphrase-only")]
    allow_passphrase_only: bool,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    if cli.self_check {
        return self_check();
    }

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("building the tokio runtime")?;
    rt.block_on(serve(cli.allow_passphrase_only))
}

/// Non-serving startup pre-flight — the manifest `verify` predicate (`secretd --self-check`).
///
/// Runs the SAME bring-up steps as [`serve`] up to — but deliberately NOT including — binding the
/// UDS, opening the store, or serving. That keeps it (a) non-blocking (a bare `serve` would run
/// forever, which is why the old `--self-check`-less binary made the verify hook hang), (b) safe to
/// run while the real daemon already holds the socket (it never binds), and (c) offline +
/// side-effect-free on the vault (it never connects the store). It still catches the realistic
/// startup failures: a broken crypto-provider pin, unresolvable/locked-down XDG paths, or an invalid
/// store config (a non-loopback libSQL URL, a group-readable token file — see [`config`]). Any check
/// that errors bubbles up and the process exits non-zero (fail-closed).
fn self_check() -> anyhow::Result<()> {
    // 1. The ring CryptoProvider must be installable as the process default (CF-2): the daemon
    // refuses aws-lc-rs, so if ring can't be the provider the TLS edge can't stand up. In a fresh
    // process this installs; an `Err` only means "already installed" (idempotent) — not a failure.
    let _ = rustls::crypto::ring::default_provider().install_default();

    // 2. Process hardening (FS-S4) is best-effort here exactly as in `serve` — a `setrlimit` or
    // `mlockall` failure is logged, not fatal (systemd's LimitCORE/LimitMEMLOCK are authoritative),
    // so it never fails the self-check on its own. `require_mlock` strict mode is DELIBERATELY NOT
    // honored here: `--self-check` is a non-serving pre-flight, so a missing CAP_IPC_LOCK must not
    // fail the manifest `verify` predicate. The mlock outcome is therefore discarded.
    let _ = harden_process();

    // 3. XDG paths resolve and the runtime/data/state dirs exist 0700 (idempotent; the install step
    // already created them — this re-asserts they are present and own-only).
    let paths = Paths::resolve().context("resolving XDG paths")?;
    ensure_dir_0700(&paths.runtime)?;
    ensure_dir_0700(&paths.data)?;
    ensure_dir_0700(&paths.state)?;

    // 4. The store config loads + validates (backend selection + the libSQL transport-safety and
    // token-file-mode rules). This is the check most likely to surface a real misconfiguration; it
    // validates WITHOUT connecting, so the self-check stays offline and cannot block.
    let _store_cfg =
        config::StoreConfig::load(&paths.config_file()).context("loading store config")?;

    // 5. The `[profile]` (Profile A on-box / B VPS) config loads + validates. This is the CONFIG-LEVEL
    // half of the TASK-0033 startup guards and is what makes `--self-check`'s contract honest for a
    // VPS deploy: a `topology = "remote"` config missing its substitute presence factor fails HERE
    // (FS-S21: `operator_authorizer_url` required; plus `vps_instance_id`/`operator_pubkey_hex`/…),
    // and a `vtpm_gating` config is rejected at parse (FS-S24) — the same fatal `ProfileSettings::load`
    // bail `serve` hits before binding. It stays offline + side-effect-free (pure TOML parse, no vault).
    // NOTE: FS-S22 (on-box USB-keyslot presence) and FS-S23 (VPS gate primed by the live authorizer
    // link) need serve-time engine/link state and are DELIBERATELY out of scope here — self-check
    // covers the config-level guards only, not the runtime-state ones.
    let _profile =
        config::ProfileSettings::load(&paths.config_file()).context("loading [profile] config")?;

    println!("secretd --self-check: OK");
    Ok(())
}

async fn serve(allow_passphrase_only: bool) -> anyhow::Result<()> {
    // 1. Install the rustls RING crypto provider (CF-2). Idempotent; ignore "already installed".
    if rustls::crypto::ring::default_provider()
        .install_default()
        .is_err()
    {
        tracing::debug!("rustls ring CryptoProvider already installed");
    }

    // 2. Process hardening (FS-S4): no core dumps; raise the memlock ceiling; `mlockall` the address
    // space (best-effort here — see `harden_process`). The mlock outcome is captured so we can apply
    // the operator's `require_mlock` strict-mode decision AFTER the config loads (step 4 below).
    let mlock = harden_process();

    // 3. Resolve paths and create the runtime/data/state dirs 0700.
    let paths = Paths::resolve().context("resolving XDG paths")?;
    ensure_dir_0700(&paths.runtime)?;
    ensure_dir_0700(&paths.data)?;
    ensure_dir_0700(&paths.state)?;

    // 4. Select the store backend from config (env > secretd.toml > inmem default) and open the
    // engine (Arc-backed; Clone + Send + Sync). First-run bootstrap: there is no `Vault.Init` RPC in
    // the control proto, so a fresh vault stays Locked until an out-of-band init + an explicit
    // `Lock.Unlock`. We do not auto-init here (no passphrase/USB to enroll).
    let store_cfg =
        config::StoreConfig::load(&paths.config_file()).context("loading store config")?;

    // FS-S4 strict mode: `harden_process` runs BEFORE config is available, so the mlockall fatality
    // decision is applied HERE, once `require_mlock` is known. When strict + the in-process lock
    // could not be established, the daemon REFUSES to serve (fail-closed) — secret material would be
    // swappable, which the operator has elected to forbid. The default (`require_mlock = false`)
    // keeps mlockall best-effort and never reaches this bail.
    if store_cfg.require_mlock && mlock.failed() {
        anyhow::bail!(
            "require_mlock is set but mlockall failed; refusing to start with potentially \
             swappable secret memory. Grant secretd CAP_IPC_LOCK (or raise LimitMEMLOCK) and retry"
        );
    }

    // TASK-0033: load the Profile (A on-box / B VPS) config. FS-S24 vTPM gating is rejected at parse.
    let profile =
        config::ProfileSettings::load(&paths.config_file()).context("loading [profile] config")?;

    let (engine, profile_b_seams) = build_engine(paths.clone(), store_cfg, profile.clone()).await?;
    // The Profile-B seam handles are consumed ONLY by the `relay-edge` authorizer spawn below; a
    // default (no-edge) build holds them inert (a VPS daemon needs the edge feature anyway).
    #[cfg(not(feature = "relay-edge"))]
    let _ = &profile_b_seams;

    // TASK-0033 startup guards (FS-S21/S22/S23/S24). Run ALL four against the engine's resolved
    // state + the profile knobs BEFORE binding; a refusal is FATAL (fail-closed — never serve a
    // downgraded config). For a VPS, FS-S23 (gate not Unproven) is satisfied AFTER the authorizer
    // link delivers a token; we spawn the link below, then re-assert FS-S23 once it has primed —
    // but FS-S21/S22/S24 are config-level and checked here unconditionally.
    if let Err(refusal) = engine.assert_profile_b_startup(
        profile.operator_authorizer_url.as_deref(),
        allow_passphrase_only,
        profile.vtpm_gating_requested,
    ) {
        // FS-S23 (VpsGateUnprovenAtStartup) is the ONLY refusal a VPS legitimately hits before the
        // authorizer link primes the gate — it is re-checked after spawn (below). Every other
        // refusal is a config error and is fatal here.
        if !matches!(
            refusal,
            envctl_secrets::StartupRefusal::VpsGateUnprovenAtStartup
        ) {
            anyhow::bail!("startup refused: {refusal}");
        }
        tracing::info!("VPS gate unproven at startup — will prime via the authorizer link");
    }

    // 5. Bind the UDS (reaping a stale socket from a dead daemon), chmod 0600.
    let sock = paths.control_socket();
    let listener = bind_uds(&sock).await?;

    let owner_uid = peercred::owner_uid();
    tracing::info!(socket = %sock.display(), owner_uid, "secretd listening");

    // Graceful shutdown on SIGINT / SIGTERM, FANNED OUT to both servers (gRPC control plane + relay
    // proxy) via a broadcast: a single signal task fires once, and each server awaits its own
    // receiver so neither steals the signal from the other.
    let (shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(1);
    let signal_tx = shutdown_tx.clone();
    tokio::spawn(async move {
        let mut sigterm =
            match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!(error = %e, "failed to install SIGTERM handler");
                    return;
                }
            };
        tokio::select! {
            _ = tokio::signal::ctrl_c() => tracing::info!("SIGINT received; shutting down"),
            _ = sigterm.recv() => tracing::info!("SIGTERM received; shutting down"),
        }
        let _ = signal_tx.send(());
    });
    let recv_shutdown = |mut rx: tokio::sync::broadcast::Receiver<()>| async move {
        let _ = rx.recv().await;
    };
    let grpc_shutdown = recv_shutdown(shutdown_tx.subscribe());

    // Relay data-plane proxy (PR-2a): bind an ephemeral loopback port and serve it under the SAME
    // graceful shutdown, alongside the gRPC control plane. Publish the bound 127.0.0.1:<port> into
    // the shared DaemonState so PR-2b's Relay.Mint can repoint the child at it. A bind failure is
    // logged but NOT fatal — the control plane still serves (the proxy is opt-in via `env-ctl run`).
    let state = envctl_secretd::grpc::DaemonState::default();
    let proxy_handle = match envctl_secretd::proxy::serve_proxy(
        engine.clone(),
        owner_uid,
        recv_shutdown(shutdown_tx.subscribe()),
    )
    .await
    {
        Ok((addr, handle)) => {
            let _ = state.proxy_addr.set(addr);
            tracing::info!(proxy = %addr, "relay proxy listening (loopback)");
            Some(handle)
        }
        Err(e) => {
            tracing::warn!(error = %e, "relay proxy failed to bind (control plane continues)");
            None
        }
    };

    // F2 remote relay edge (TASK-0031 PR-1): when the `relay-edge` feature is built AND `[edge]` is
    // explicitly enabled in secretd.toml, bind the PUBLIC HTTPS edge (in-process TLS + DPoP/EKM) under
    // the SAME broadcast shutdown as the proxy. OFF by default (no feature / no `[edge]` block ⇒ no
    // bind — a stock secretd serves no public edge). A cert-load or bind failure here is FATAL because
    // the operator explicitly turned the edge ON (fail-closed: we do NOT silently fall back to no
    // edge after the operator asked for one).
    // TASK-0033 (U14): Profile-B operator-box authorizer link. When `topology == Vps`, spawn the
    // async mTLS task that fetches + verifies presence tokens and feeds the SHARED gate +
    // trusted-time the engine reads. Gated behind `relay-edge` (the authorizer is part of the edge
    // plane). On Profile A this is inert (no VPS seams were built).
    #[cfg(feature = "relay-edge")]
    let authorizer_handle = if profile.topology == config::Topology::Vps {
        let (Some(gate), Some(trusted_time)) = (
            profile_b_seams.gate.clone(),
            profile_b_seams.trusted_time.clone(),
        ) else {
            anyhow::bail!("VPS topology but Profile-B seams were not built (internal error)");
        };
        // Resolve this VPS's edge-cert fingerprint (channel binding the operator-box token is
        // checked against). Default to relay_tls_dir/cert.pem when not explicitly configured.
        let edge_cert_path = profile
            .edge_cert_path
            .clone()
            .unwrap_or_else(|| paths.relay_tls_dir().join("cert.pem"));
        let vps_cert_fp =
            envctl_secretd::edge::authorizer::cert_fingerprint_from_pem(&edge_cert_path)
                .context("computing this VPS's edge-cert fingerprint for the authorizer binding")?;
        let auth_cfg = envctl_secretd::edge::authorizer::AuthorizerConfig {
            url: profile
                .operator_authorizer_url
                .clone()
                .expect("Profile B guarantees a URL"),
            vps_instance_id: profile
                .vps_instance_id
                .clone()
                .expect("Profile B guarantees an instance id"),
            vps_cert_fp,
            operator_pubkey: profile
                .operator_pubkey
                .expect("Profile B guarantees a pubkey"),
            operator_ca_path: profile
                .operator_ca_path
                .clone()
                .expect("Profile B guarantees a CA path"),
            client_cert_path: profile
                .client_cert_path
                .clone()
                .expect("Profile B guarantees a client cert"),
            client_key_path: profile
                .client_key_path
                .clone()
                .expect("Profile B guarantees a client key"),
        };
        let (asink, _arx) = envctl_secrets::EventSink::channel();
        let handle = envctl_secretd::edge::authorizer::spawn_authorizer_link(
            auth_cfg,
            engine.clone(),
            gate,
            trusted_time,
            asink,
            recv_shutdown(shutdown_tx.subscribe()),
        )
        .context("starting the operator-box authorizer link")?;
        tracing::info!("operator-box authorizer link started (Profile B / VPS)");
        Some(handle)
    } else {
        None
    };

    #[cfg(feature = "relay-edge")]
    let edge_handle = {
        let edge_cfg =
            config::EdgeSettings::load(&paths.config_file()).context("loading [edge] config")?;
        if let Some(path) = edge_cfg.client_revocations_path.clone() {
            let _ = state.client_revocations_path.set(path);
        }
        if edge_cfg.enabled {
            let bind_addr = edge_cfg
                .bind_addr
                .expect("EdgeSettings guarantees a bind_addr when enabled");
            let cfg = envctl_secretd::edge::EdgeConfig {
                enabled: true,
                bind_addr,
                // Production streaming re-check cadence/cap (TASK-0032 / FS-S5).
                recheck_timing: None,
                // PR-2b mTLS hardened mode (OI-SM-4): opt-in via the [edge] block; default-OFF.
                require_client_cert: edge_cfg.require_client_cert,
                client_ca_path: edge_cfg.client_ca_path.clone(),
                client_revocations_path: edge_cfg.client_revocations_path.clone(),
                // Production ingress caps (PR-2: body size + handshake/header/idle timeouts).
                ingress_caps: None,
            };
            let (addr, handle) = envctl_secretd::edge::serve_edge(
                engine.clone(),
                &paths,
                &cfg,
                recv_shutdown(shutdown_tx.subscribe()),
            )
            .await
            .context("starting the remote relay edge (relay-tls cert + bind)")?;
            tracing::info!(edge = %addr, "remote relay edge listening (public)");
            Some(handle)
        } else {
            tracing::debug!("[edge] disabled — no public remote edge bound");
            None
        }
    };

    // READY=1 (FS — Type=notify): the UDS is bound, 0600, and the service stack is about to serve, so
    // the daemon is now reachable by the owner. Telling systemd we are ready closes the crash loop:
    // without this, `Type=notify` waits the full `TimeoutStartSec` (~90s), kills the "still starting"
    // daemon, and `Restart=on-failure` storms. A no-op when `$NOTIFY_SOCKET` is unset (tests / a
    // non-systemd run), so it is always safe to call. Best-effort: a notify failure must not abort a
    // healthy daemon, so we log and serve regardless.
    if let Err(e) = sd_notify::notify(false, &[sd_notify::NotifyState::Ready]) {
        tracing::warn!(error = %e, "sd_notify READY failed (continuing to serve)");
    }

    let result = server::serve_with_state(engine, owner_uid, listener, state, grpc_shutdown).await;

    // STOPPING=1 on graceful shutdown so systemd does not race the teardown against a restart. No-op
    // without `$NOTIFY_SOCKET`; best-effort.
    if let Err(e) = sd_notify::notify(false, &[sd_notify::NotifyState::Stopping]) {
        tracing::warn!(error = %e, "sd_notify STOPPING failed");
    }

    // Wait for the relay proxy task to wind down under the shared shutdown before exiting.
    if let Some(handle) = proxy_handle {
        let _ = handle.await;
    }

    // Likewise wait for the remote relay edge (if it was started) to wind down.
    #[cfg(feature = "relay-edge")]
    if let Some(handle) = edge_handle {
        let _ = handle.await;
    }

    // Wait for the Profile-B authorizer link (if it was started) to wind down.
    #[cfg(feature = "relay-edge")]
    if let Some(handle) = authorizer_handle {
        let _ = handle.await;
    }

    // Best-effort cleanup of the socket on graceful exit.
    let _ = std::fs::remove_file(&sock);
    result.context("serving the control plane")?;
    Ok(())
}

/// Build the engine on the configured store backend (OI-1 (a), Phase 1).
///
/// The libSQL store drives its OWN current-thread runtime via `block_on`, so it is constructed on a
/// `spawn_blocking` thread — NEVER on the async reactor, where a nested `block_on` would panic (see
/// `secrets-store-libsql/src/sync.rs`). `InMemStore` does no async and is built inline.
async fn build_engine(
    paths: Paths,
    cfg: config::StoreConfig,
    profile: config::ProfileSettings,
) -> anyhow::Result<(Engine, ProfileBSeams)> {
    // Capture the runtime handle in this ASYNC context so the GitHub mint transport (TASK-0020) can
    // be built even on the OFF-reactor `spawn_blocking` thread the libSQL store is constructed on
    // (where `Handle::current()` would panic). Cheap to clone; moved into the blocking closure.
    let rt = tokio::runtime::Handle::current();
    match cfg.backend {
        config::Backend::InMem => {
            tracing::info!(
                "store backend = in-memory (ephemeral; set [store] in secretd.toml for durability)"
            );
            engine_with_daemon_seams(
                paths,
                Box::new(envctl_secrets::vault::InMemStore::new()),
                rt,
                &profile,
            )
            .context("opening the engine on the in-memory store")
        }
        config::Backend::LibSql => {
            let url = cfg
                .url
                .expect("resolve() guarantees a URL for the libSQL backend");
            tracing::info!(url = %url, "store backend = libSQL remote (durable)");
            let token = cfg.auth_token; // Zeroizing; moved into + dropped by the blocking task
            tokio::task::spawn_blocking(move || -> anyhow::Result<(Engine, ProfileBSeams)> {
                let store = envctl_secrets_store_libsql::LibSqlStoreBuilder::new(
                    url,
                    token.as_str().to_owned(),
                )
                .build()
                .context("opening the libSQL remote store (is sqld reachable?)")?;
                engine_with_daemon_seams(paths, Box::new(store), rt, &profile)
                    .context("opening the engine on the libSQL store")
            })
            .await
            .context("the libSQL store-construction task panicked")?
        }
    }
}

/// Open the engine with the DAEMON's real seams: `SystemClock`, `RealUsbProbe`, `NoMint`, the
/// [`proxy::DaemonUpstream`] egress sender (webpki-roots TLS, FS-S7) in place of the engine's default
/// `NullUpstream`, and — under `provider-github` (TASK-0020) — the [`transport::DaemonHttpTransport`]
/// HTTP seam for the per-call GitHub App mint (same frozen-roots/ring TLS, reused verbatim, no new
/// dep). This is the ONLY place the live seams are installed; the engine API is untouched (they are
/// injected through the public `Engine::with_seams`). `rt` is the runtime handle captured in the
/// async `build_engine`, so the transport can be constructed even on the off-reactor `spawn_blocking`
/// thread the libSQL store is built on.
fn engine_with_daemon_seams(
    paths: Paths,
    store: Box<dyn envctl_secrets::vault::Store>,
    #[allow(unused_variables)] rt: tokio::runtime::Handle,
    profile: &config::ProfileSettings,
) -> anyhow::Result<(Engine, ProfileBSeams)> {
    // TASK-0033: Profile-B seams. For Profile A (default) these are the engine's fail-closed
    // defaults (UnprovenGate, never consulted on-box; SystemClockTrustedTime). For Profile B we
    // build a SHARED `Arc<VpsPresenceGate>` + `Arc<OperatorBoxTrustedTime>` and hand a clone to the
    // engine AND keep a clone so `serve()` can spawn the authorizer link feeding the SAME instances.
    let (presence_gate, trusted_time, b_seams): (
        Box<dyn envctl_secrets::PresenceGate>,
        Box<dyn envctl_secrets::TrustedTime>,
        ProfileBSeams,
    ) = match profile.topology {
        config::Topology::OnBox => (
            Box::new(envctl_secrets::broker::UnprovenGate),
            Box::new(envctl_secrets::SystemClockTrustedTime),
            ProfileBSeams::default(),
        ),
        config::Topology::Vps => {
            let gate = std::sync::Arc::new(envctl_secrets::VpsPresenceGate::new(Box::new(
                envctl_secrets::seam::SystemClock,
            )));
            let trusted = std::sync::Arc::new(envctl_secrets::OperatorBoxTrustedTime::new(
                Box::new(envctl_secrets::seam::SystemClock),
            ));
            (
                Box::new(gate.clone()),
                Box::new(trusted.clone()),
                ProfileBSeams {
                    gate: Some(gate),
                    trusted_time: Some(trusted),
                },
            )
        }
    };

    let engine = Engine::with_seams(
        paths,
        store,
        Box::new(envctl_secrets::seam::SystemClock),
        Box::new(envctl_secrets::seam::RealUsbProbe),
        Box::new(envctl_secrets::seam::NoMint),
        Box::new(envctl_secretd::proxy::DaemonUpstream::new()),
        #[cfg(feature = "provider-github")]
        Box::new(envctl_secretd::transport::DaemonHttpTransport::from_handle(
            rt,
        )),
        presence_gate,
        trusted_time,
        profile.topology.to_engine(),
    )?;
    Ok((engine, b_seams))
}

/// The Profile-B (VPS) shared seam handles `serve()` needs to spawn the authorizer link feeding the
/// SAME gate + trusted-time the engine reads. Empty for Profile A.
#[derive(Default)]
#[cfg_attr(not(feature = "relay-edge"), allow(dead_code))]
struct ProfileBSeams {
    gate: Option<std::sync::Arc<envctl_secrets::VpsPresenceGate>>,
    trusted_time: Option<std::sync::Arc<envctl_secrets::OperatorBoxTrustedTime>>,
}

/// Outcome of the in-process `mlockall` attempt (FS-S4). Returned by [`harden_process`] so a caller
/// can apply the strict-mode (`require_mlock`) fatality decision AFTER the config is loaded — the
/// syscall itself is always attempted best-effort regardless of strict mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MlockOutcome {
    /// `mlockall(MCL_CURRENT|MCL_FUTURE)` succeeded — this and all future pages are pinned (no swap).
    Locked,
    /// `mlockall` failed (e.g. `EPERM` without `CAP_IPC_LOCK`, or `ENOMEM`). `errno` is the raw OS
    /// error number; the message was already WARN-logged (metadata only — no secret bytes).
    Failed { errno: i32 },
    /// Not a Linux build — `mlockall` is a Linux syscall, so there is nothing to attempt here.
    /// Only constructed by the `#[cfg(not(target_os = "linux"))]` `mlock_all_pages`, so on a Linux
    /// build it is (correctly) never constructed; the narrow allow keeps the variant present so the
    /// `failed()`/strict-mode logic stays target-uniform without a broad crate-level allow.
    #[cfg_attr(target_os = "linux", allow(dead_code))]
    NotApplicable,
}

impl MlockOutcome {
    /// `true` iff the in-process lock could NOT be established (strict mode treats this as fatal).
    /// `NotApplicable` is NOT a failure (non-Linux dev build has no syscall to fail).
    fn failed(self) -> bool {
        matches!(self, MlockOutcome::Failed { .. })
    }
}

/// `RLIMIT_CORE=0` (no core dumps that could leak key material) + raise `RLIMIT_MEMLOCK` so an
/// `mlock` can succeed, then `mlockall(MCL_CURRENT|MCL_FUTURE)` so secret material (DEK / vault
/// plaintext / PEMs) — all allocated AFTER startup, once `Lock.Unlock` runs — can never reach swap
/// (FS-S4). `MCL_FUTURE` is load-bearing: it covers those post-unlock allocations.
///
/// Best-effort throughout: a `setrlimit` failure is logged, not fatal (the systemd unit's
/// `LimitCORE`/`LimitMEMLOCK` are the authoritative defense-in-depth). `mlockall` is likewise
/// attempted best-effort and NEVER panics — it commonly fails `EPERM` (no `CAP_IPC_LOCK`) or
/// `ENOMEM`; on failure the daemon CONTINUES (relying on `RLIMIT_CORE=0` + systemd `LimitMEMLOCK`),
/// emitting a metadata-only WARN. The returned [`MlockOutcome`] lets `serve` enforce the operator's
/// `require_mlock` strict mode (fail-closed) AFTER config load; the syscall here stays best-effort.
fn harden_process() -> MlockOutcome {
    if let Err(e) = setrlimit(
        Resource::Core,
        Rlimit {
            current: Some(0),
            maximum: Some(0),
        },
    ) {
        tracing::warn!(error = %e, "could not set RLIMIT_CORE=0 (relying on systemd LimitCORE)");
    }
    if let Err(e) = setrlimit(
        Resource::Memlock,
        Rlimit {
            current: None, // None => infinity, raising the ceiling for the mlock below
            maximum: None,
        },
    ) {
        tracing::warn!(error = %e, "could not raise RLIMIT_MEMLOCK (relying on systemd LimitMEMLOCK)");
    }
    mlock_all_pages()
}

/// Pin the whole address space (current + future pages) into RAM so secret material is never
/// swapped to disk (FS-S4). Linux-only (`mlockall` + `MCL_*` are Linux); on other targets this is a
/// no-op [`MlockOutcome::NotApplicable`] so dev builds still compile. Best-effort: on `-1` it logs a
/// metadata-only WARN (errno + strerror, NEVER secret bytes) and returns `Failed{errno}` WITHOUT
/// panicking — pre-unlock there are no secrets in the address space anyway, and `RLIMIT_CORE=0`
/// independently mitigates core-dump leakage.
#[cfg(target_os = "linux")]
fn mlock_all_pages() -> MlockOutcome {
    // SAFETY: `mlockall` is a trivial syscall taking only an int flags bitmask and touching no
    // Rust-owned memory; the flags are valid `libc` constants. It cannot violate memory safety.
    let rc = unsafe { libc::mlockall(libc::MCL_CURRENT | libc::MCL_FUTURE) };
    if rc == -1 {
        let err = std::io::Error::last_os_error();
        let errno = err.raw_os_error().unwrap_or(0);
        tracing::warn!(
            errno,
            error = %err,
            "mlockall(MCL_CURRENT|MCL_FUTURE) failed; secret material may be swappable. \
             Relying on RLIMIT_CORE=0 + systemd LimitMEMLOCK. Grant CAP_IPC_LOCK (or set \
             [security].require_mlock to refuse startup) to enforce the in-process lock."
        );
        MlockOutcome::Failed { errno }
    } else {
        MlockOutcome::Locked
    }
}

/// Non-Linux fallback: `mlockall` is a Linux syscall, so there is nothing to attempt. Lets dev
/// builds on macOS/Windows compile; the daemon ships on Linux where the real path above runs.
#[cfg(not(target_os = "linux"))]
fn mlock_all_pages() -> MlockOutcome {
    MlockOutcome::NotApplicable
}

/// Create `dir` (and parents) with mode 0700, tightening perms if it already exists.
fn ensure_dir_0700(dir: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))
        .with_context(|| format!("chmod 0700 {}", dir.display()))?;
    Ok(())
}

/// Bind the control UDS at `sock`, reaping a stale socket left by a dead daemon (connect-probe: a
/// refused connection means no live daemon -> remove + rebind; a successful connection means a
/// daemon is already running -> bail). Sets the socket to 0600 after bind.
async fn bind_uds(sock: &Path) -> anyhow::Result<tokio::net::UnixListener> {
    if sock.exists() {
        match tokio::net::UnixStream::connect(sock).await {
            Ok(_) => anyhow::bail!("daemon already running at {}", sock.display()),
            Err(_) => {
                // No live peer; reap the stale socket.
                std::fs::remove_file(sock)
                    .with_context(|| format!("removing stale socket {}", sock.display()))?;
            }
        }
    }
    // NOTE (bind/chmod window): `bind` creates the socket with a umask-governed mode and the tighten
    // to 0600 is a SEPARATE call below, so for the window between the two the socket may be
    // group/other-readable. The LOAD-BEARING WALL during that window is the parent runtime dir, which
    // `ensure_dir_0700` created 0700 BEFORE this bind: a non-owner cannot traverse into it to reach
    // the socket, and the SO_PEERCRED `OwnerGuard` would deny on uid regardless. (A `umask(0o077)`
    // guard would close the window correct-by-construction at the socket level, but `rustix::umask`
    // needs the `fs` feature, which is not in the pinned feature set — no new deps, so we rely on the
    // 0700 dir + chmod here.)
    let listener = tokio::net::UnixListener::bind(sock)
        .with_context(|| format!("binding {}", sock.display()))?;
    std::fs::set_permissions(sock, std::fs::Permissions::from_mode(0o600))
        .with_context(|| format!("chmod 0600 {}", sock.display()))?;
    Ok(listener)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The mlockall wrapper must return cleanly and NEVER panic, whatever the syscall does. CI lacks
    /// `CAP_IPC_LOCK`, so this almost always exercises the `EPERM` -> `Failed` path; the test asserts
    /// only that a valid outcome comes back without panicking (it does NOT assert `Locked`, which
    /// would be unachievable in CI). On a privileged host it may return `Locked`; on non-Linux it
    /// returns `NotApplicable`. All three are acceptable — the contract is "no panic, handled value".
    #[test]
    fn mlockall_best_effort_does_not_panic() {
        let outcome = mlock_all_pages();
        match outcome {
            // EPERM/ENOMEM in CI: the handled not-locked path. Errno must be a real OS error.
            MlockOutcome::Failed { errno } => {
                assert_ne!(errno, 0, "Failed must carry a real errno")
            }
            // Privileged host or non-Linux dev build — also valid.
            MlockOutcome::Locked | MlockOutcome::NotApplicable => {}
        }
        // `failed()` is consistent with the variant (sanity on the strict-mode predicate).
        assert_eq!(
            outcome.failed(),
            matches!(outcome, MlockOutcome::Failed { .. })
        );
    }

    /// `harden_process()` runs the full FS-S4 hardening (RLIMIT_CORE/MEMLOCK + mlockall) and must
    /// return normally even when mlockall fails (EPERM in CI). It never panics and never blocks.
    #[test]
    fn harden_process_best_effort_default() {
        // Returns a valid outcome; default config (require_mlock=false) would not act on a failure.
        let outcome = harden_process();
        // No panic reaching here is the assertion; confirm the value is one of the known variants.
        assert!(matches!(
            outcome,
            MlockOutcome::Locked | MlockOutcome::Failed { .. } | MlockOutcome::NotApplicable
        ));
    }

    /// Pure-logic check of the strict-mode fatality rule: `require_mlock && outcome.failed()` is the
    /// exact predicate `serve` bails on. Deterministic — it does NOT depend on actually locking in
    /// CI; it drives the predicate with constructed outcomes.
    #[test]
    fn require_mlock_strict_fatal_when_unlocked() {
        // The bail condition `serve` uses.
        let bail = |require_mlock: bool, outcome: MlockOutcome| require_mlock && outcome.failed();

        // STRICT + failed lock => fatal (serve refuses to start).
        assert!(bail(
            true,
            MlockOutcome::Failed {
                errno: 1 /* EPERM */
            }
        ));
        // STRICT + locked => not fatal (the lock was established).
        assert!(!bail(true, MlockOutcome::Locked));
        // STRICT + not-applicable (non-Linux) => not fatal (no syscall to fail).
        assert!(!bail(true, MlockOutcome::NotApplicable));
        // DEFAULT (best-effort) + failed lock => never fatal.
        assert!(!bail(false, MlockOutcome::Failed { errno: 1 }));
        assert!(!bail(false, MlockOutcome::Locked));
    }
}
