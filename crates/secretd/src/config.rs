//! secretd runtime configuration: store-backend selection (in-memory vs the durable libSQL `remote`
//! store — OI-1 (a), Phase 1) and the libSQL connection parameters.
//!
//! ## Precedence (highest first)
//! environment variables > the optional TOML config file (`$META_ROOT/.config/env-ctl/secretd.toml` under envctl-managed execution) >
//! defaults (`backend = "inmem"`).
//!
//! ## Credential hygiene
//! The libSQL AUTH TOKEN is a credential and is therefore **never** read from the TOML file — only
//! from `SECRETD_LIBSQL_AUTH_TOKEN`, or from a file named by `SECRETD_LIBSQL_AUTH_TOKEN_FILE` (which
//! must be `0600` — a group/other-readable token file is refused, fail-closed). The CONFIG-layer
//! token copy is held in a [`Zeroizing`] buffer and never logged (Debug redacts it); note the
//! downstream libSQL client takes a plain `String` (its public API) and keeps its own non-zeroized
//! copy for the connection's lifetime — unavoidable without libSQL support.
//!
//! ## Transport safety (FS-S7 spirit)
//! The daemon's libSQL client uses a PLAINTEXT connector — the gate-clean choice, because libSQL's
//! `tls` feature would pull a SECOND rustls (`hyper-rustls 0.25 -> rustls 0.22`); see
//! `secrets-store-libsql/src/sync.rs` + DESIGN-NOTES OI-1. So a libSQL URL must be **loopback**
//! `http`/`ws` (`http://127.0.0.1`, `http://[::1]`, `http://localhost`). A plaintext URL to a
//! NON-loopback host is **refused** (the auth token + metadata + write-integrity would otherwise
//! cross the network in the clear). A direct TLS URL (`https`/`wss`/`libsql`) is also **refused**,
//! with guidance to front a remote sqld with a LOOPBACK TLS terminator (stunnel/spiped/cloudflared)
//! and point secretd at `http://127.0.0.1:<local-port>` — keeping the daemon's graph gate-clean. An
//! empty auth token is accepted for a loopback sqld (local/dev open auth); a token may still be
//! supplied (e.g. a loopback terminator forwarding to an authenticated remote).

use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context};
use zeroize::Zeroizing;

pub const ENV_BACKEND: &str = "SECRETD_STORE_BACKEND";
pub const ENV_URL: &str = "SECRETD_LIBSQL_URL";
pub const ENV_TOKEN: &str = "SECRETD_LIBSQL_AUTH_TOKEN";
pub const ENV_TOKEN_FILE: &str = "SECRETD_LIBSQL_AUTH_TOKEN_FILE";
pub const ENV_CONFIG: &str = "SECRETD_CONFIG";
/// FS-S4 strict-mlock toggle: when set to a truthy value (`1`/`true`/`yes`/`on`), an `mlockall`
/// failure at startup is FATAL (the daemon refuses to serve). Overrides `[security].require_mlock`.
pub const ENV_REQUIRE_MLOCK: &str = "SECRETD_REQUIRE_MLOCK";

/// Which persistence backend the daemon's engine is built on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// RAM-only store (ephemeral; the default until a durable store is configured).
    InMem,
    /// The durable libSQL `remote` store (talks HTTP/Hrana to a sqld).
    LibSql,
}

/// Resolved, validated store configuration.
pub struct StoreConfig {
    pub backend: Backend,
    /// Present iff `backend == LibSql` (validated non-empty + transport-safe).
    pub url: Option<String>,
    /// The libSQL auth token (possibly empty for a loopback sqld). Never logged.
    pub auth_token: Zeroizing<String>,
    /// FS-S4 strict-mlock mode (default `false`). When `true`, a failed in-process `mlockall` at
    /// startup is FATAL — `serve` refuses to come up (fail-closed; an operator-elected hardened
    /// mode per THREAT-MODEL.md "refuse-on-fail"). When `false` (the default), `mlockall` is
    /// best-effort: a failure is logged and the daemon continues. Sourced from
    /// `SECRETD_REQUIRE_MLOCK` (env wins) or `[security].require_mlock` in `secretd.toml`. Has NO
    /// effect on `--self-check`, which keeps `mlockall` best-effort regardless (non-serving).
    pub require_mlock: bool,
}

impl std::fmt::Debug for StoreConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoreConfig")
            .field("backend", &self.backend)
            .field("url", &self.url)
            .field("auth_token", &"<redacted>")
            .field("require_mlock", &self.require_mlock)
            .finish()
    }
}

/// The TOML file shape. The auth token is DELIBERATELY absent — credentials never live in the file.
#[derive(serde::Deserialize, Default)]
struct FileConfig {
    store: Option<FileStore>,
    #[cfg(feature = "relay-edge")]
    edge: Option<FileEdge>,
    security: Option<FileSecurity>,
    /// TASK-0033: the `[profile]` block (SERVER-MODE Profile A on-box vs Profile B VPS). Absent ⇒
    /// on-box (Profile A), the default.
    profile: Option<FileProfile>,
}

/// `[profile]` table (TASK-0033 / SERVER-MODE §6). Carries the deployment topology and — for a VPS —
/// the operator-box authorizer link parameters. DELIBERATELY carries NO secret: the operator key is
/// a PUBLIC key (hex), the client cert/key live on disk as PEM paths (like the relay-tls key).
#[derive(serde::Deserialize, Default)]
struct FileProfile {
    /// `"onbox"` (default) or `"remote"`/`"vps"` (Profile B).
    topology: Option<String>,
    /// Profile B: `https://operator.box:PORT` of the operator-box authorizer. REQUIRED when
    /// `topology = "remote"` (FS-S21; enforced at startup).
    operator_authorizer_url: Option<String>,
    /// Profile B: this VPS's instance id (binds the token to this deployment).
    vps_instance_id: Option<String>,
    /// Profile B: operator-pinned Ed25519 public key (64-hex) the token signature is verified against.
    operator_pubkey_hex: Option<String>,
    /// Profile B: PEM path of the operator-box CA (mTLS server-trust; frozen roots).
    operator_ca_path: Option<String>,
    /// Profile B: PEM path of this VPS's mTLS client certificate.
    client_cert_path: Option<String>,
    /// Profile B: PEM path of this VPS's mTLS client private key.
    client_key_path: Option<String>,
    /// Profile B: PEM path of this VPS's OWN edge cert (to compute its `vps_cert_fp` channel binding).
    /// Defaults to `relay_tls_dir/cert.pem` when absent.
    edge_cert_path: Option<String>,
    /// FS-S24: FORBIDDEN — gating DEK release on a vTPM. Any truthy value is REJECTED at parse time
    /// (a vTPM has no hardware isolation boundary on a VPS). Present here only so the refusal is
    /// explicit if an operator tries to enable it.
    #[serde(default)]
    vtpm_gating: bool,
}
#[derive(serde::Deserialize, Default)]
struct FileStore {
    backend: Option<String>,
    url: Option<String>,
}
/// `[security]` table. FS-S4 process-hardening toggles (no credentials live here either).
#[derive(serde::Deserialize, Default)]
struct FileSecurity {
    /// Strict-mlock mode: when `true`, a failed `mlockall` at startup is fatal. Defaults to `false`
    /// (best-effort) so a stock daemon without `CAP_IPC_LOCK` still comes up.
    #[serde(default)]
    require_mlock: bool,
}

/// The `[edge]` block (F2 / TASK-0031, `relay-edge` feature only). DELIBERATELY carries NO secret —
/// the relay TLS cert/key live on disk under `relay_tls_dir()`, not in this file.
#[cfg(feature = "relay-edge")]
#[derive(serde::Deserialize, Default)]
struct FileEdge {
    /// Serve the public remote edge at all (default `false` — a stock secretd binds no edge).
    enabled: Option<bool>,
    /// The bind address (e.g. `"0.0.0.0:8443"` or `"127.0.0.1:8443"`). Required when `enabled`.
    bind_addr: Option<String>,
    /// PR-2b: require a verified client certificate (mTLS hardened mode). Default `false`.
    require_client_cert: Option<bool>,
    /// PR-2b: path to the operator-provisioned remote-clients-CA PEM (required when
    /// `require_client_cert`). NEVER the MITM CA / server cert — a separate trust input (FS-S25).
    client_ca_path: Option<String>,
    /// PR-2b: optional revocation-set path for hardened mTLS client certs.
    client_revocations_path: Option<String>,
}

impl StoreConfig {
    /// Load + validate the config: read the TOML file (env `SECRETD_CONFIG` overrides
    /// `default_config_path`; a missing file is fine), apply environment overrides, then [`resolve`].
    pub fn load(default_config_path: &Path) -> anyhow::Result<StoreConfig> {
        let cfg_path = std::env::var_os(ENV_CONFIG)
            .map(PathBuf::from)
            .unwrap_or_else(|| default_config_path.to_path_buf());
        let file_text = match std::fs::read_to_string(&cfg_path) {
            Ok(t) => Some(t),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e).with_context(|| format!("reading {}", cfg_path.display())),
        };
        let file: FileConfig = match &file_text {
            Some(t) => toml::from_str(t).context("parsing secretd config TOML")?,
            None => FileConfig::default(),
        };
        let fstore = file.store.unwrap_or_default();
        let fsecurity = file.security.unwrap_or_default();

        let backend = env_nonempty(ENV_BACKEND).or(fstore.backend);
        let url = env_nonempty(ENV_URL).or(fstore.url);
        let token = load_token().context("loading the libSQL auth token")?;
        // FS-S4 strict-mlock: env override (truthy) wins over the `[security].require_mlock` file
        // value; absent in both => false (best-effort default).
        let require_mlock = env_bool(ENV_REQUIRE_MLOCK).unwrap_or(fsecurity.require_mlock);

        resolve(backend, url, token, require_mlock)
    }
}

/// Read an env var as a boolean: `1`/`true`/`yes`/`on` (case-insensitive) => `Some(true)`;
/// `0`/`false`/`no`/`off` => `Some(false)`; unset/empty/unrecognized => `None`.
fn env_bool(key: &str) -> Option<bool> {
    match std::env::var(key)
        .ok()?
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

/// Resolved, validated remote-edge configuration (F2 / TASK-0031, `relay-edge` feature only).
///
/// Off by default: a missing `[edge]` block, or `enabled = false`, yields `EdgeSettings { enabled:
/// false, .. }` and the daemon binds NO public edge. When `enabled = true`, a parseable `bind_addr`
/// is REQUIRED (a missing/invalid address is an `Err` — fail-closed; the edge never binds a
/// half-configured listener). The relay TLS cert is NOT named here — it is loaded fail-closed from
/// `Paths::relay_tls_dir()` at edge startup.
#[cfg(feature = "relay-edge")]
#[derive(Debug, Clone)]
pub struct EdgeSettings {
    pub enabled: bool,
    /// `Some` iff `enabled` (validated parseable). `None` when disabled.
    pub bind_addr: Option<std::net::SocketAddr>,
    /// PR-2b: require a verified client certificate (mTLS hardened mode). Default `false`.
    pub require_client_cert: bool,
    /// PR-2b: the remote-clients-CA PEM path the client cert is verified against. `Some` only when
    /// configured; a `require_client_cert = true` with `None` here is a fail-closed startup `Err`
    /// (enforced in `serve_edge`).
    pub client_ca_path: Option<std::path::PathBuf>,
    /// PR-2b: optional newline-delimited revocation-set path. Handshakes consult it freshly for each
    /// connection, so a revoked leaf is rejected on the next accept.
    pub client_revocations_path: Option<std::path::PathBuf>,
}

#[cfg(feature = "relay-edge")]
impl EdgeSettings {
    /// Load + validate the `[edge]` block from the same `secretd.toml` the store config reads (env
    /// `SECRETD_CONFIG` overrides `default_config_path`; a missing file ⇒ disabled). An env override
    /// `SECRETD_EDGE_ENABLED` / `SECRETD_EDGE_BIND_ADDR` takes precedence (mirrors the store-config
    /// precedence) so an operator can toggle the edge without editing the file.
    pub fn load(default_config_path: &Path) -> anyhow::Result<EdgeSettings> {
        let cfg_path = std::env::var_os(ENV_CONFIG)
            .map(PathBuf::from)
            .unwrap_or_else(|| default_config_path.to_path_buf());
        let file: FileConfig = match std::fs::read_to_string(&cfg_path) {
            Ok(t) => toml::from_str(&t).context("parsing secretd config TOML for [edge]")?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => FileConfig::default(),
            Err(e) => return Err(e).with_context(|| format!("reading {}", cfg_path.display())),
        };
        let fedge = file.edge.unwrap_or_default();

        // env > file. `SECRETD_EDGE_ENABLED` truthy = "1"/"true"/"yes" (case-insensitive).
        let enabled = match env_nonempty("SECRETD_EDGE_ENABLED") {
            Some(v) => matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"),
            None => fedge.enabled.unwrap_or(false),
        };
        if !enabled {
            return Ok(EdgeSettings {
                enabled: false,
                bind_addr: None,
                require_client_cert: false,
                client_ca_path: None,
                client_revocations_path: None,
            });
        }
        let bind_raw = env_nonempty("SECRETD_EDGE_BIND_ADDR")
            .or(fedge.bind_addr)
            .ok_or_else(|| {
                anyhow!(
                    "[edge].enabled = true requires a bind_addr (SECRETD_EDGE_BIND_ADDR or \
                     [edge].bind_addr, e.g. \"0.0.0.0:8443\")"
                )
            })?;
        let bind_addr: std::net::SocketAddr = bind_raw.trim().parse().with_context(|| {
            format!("parsing [edge].bind_addr {bind_raw:?} as a socket address")
        })?;
        // PR-2b mTLS (env > file). Default-OFF (additionally opt-in on top of the default-OFF feature).
        let require_client_cert = match env_nonempty("SECRETD_EDGE_REQUIRE_CLIENT_CERT") {
            Some(v) => matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"),
            None => fedge.require_client_cert.unwrap_or(false),
        };
        let client_ca_path = env_nonempty("SECRETD_EDGE_CLIENT_CA_PATH")
            .or(fedge.client_ca_path)
            .map(std::path::PathBuf::from);
        let client_revocations_path = env_nonempty("SECRETD_EDGE_CLIENT_REVOCATIONS_PATH")
            .or(fedge.client_revocations_path)
            .map(std::path::PathBuf::from);
        // Fail-closed at load time too (serve_edge re-checks): requiring a client cert with no CA to
        // verify against would accept any/none — refuse the half-built mTLS config early.
        if require_client_cert && client_ca_path.is_none() {
            return Err(anyhow!(
                "[edge].require_client_cert = true requires [edge].client_ca_path \
                 (SECRETD_EDGE_CLIENT_CA_PATH or the file key) — the mTLS gate fails closed"
            ));
        }
        Ok(EdgeSettings {
            enabled: true,
            bind_addr: Some(bind_addr),
            require_client_cert,
            client_ca_path,
            client_revocations_path,
        })
    }
}

/// Read an env var, treating unset OR empty/whitespace as `None`.
fn env_nonempty(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.trim().is_empty())
}

/// Deployment topology selector (mirrors `envctl_secrets::Topology`, parsed from `[profile].topology`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Topology {
    /// Profile A: on the operator box (default).
    #[default]
    OnBox,
    /// Profile B: on a VPS (uses the operator-box authorizer link).
    Vps,
}

impl Topology {
    /// Map to the engine's `Topology`.
    #[must_use]
    pub fn to_engine(self) -> envctl_secrets::Topology {
        match self {
            Topology::OnBox => envctl_secrets::Topology::OnBox,
            Topology::Vps => envctl_secrets::Topology::Vps,
        }
    }
}

/// Resolved, validated `[profile]` configuration (TASK-0033). For Profile A the VPS fields are all
/// `None`. For Profile B the loader REQUIRES the authorizer URL + binding inputs (a half-configured
/// VPS profile is an `Err` — fail-closed, never a silent on-box downgrade).
#[derive(Debug, Clone)]
pub struct ProfileSettings {
    pub topology: Topology,
    pub operator_authorizer_url: Option<String>,
    pub vps_instance_id: Option<String>,
    pub operator_pubkey: Option<[u8; 32]>,
    pub operator_ca_path: Option<PathBuf>,
    pub client_cert_path: Option<PathBuf>,
    pub client_key_path: Option<PathBuf>,
    pub edge_cert_path: Option<PathBuf>,
    /// FS-S24: whether the (forbidden) vTPM gating was requested in config. The loader REJECTS it at
    /// parse time, so a successfully-loaded `ProfileSettings` always has this `false`; the field is
    /// kept so the engine guard receives an explicit input.
    pub vtpm_gating_requested: bool,
}

impl ProfileSettings {
    /// Load + validate the `[profile]` block. Env overrides (`SECRETD_TOPOLOGY`,
    /// `SECRETD_OPERATOR_AUTHORIZER_URL`) win over the file. FS-S24: a truthy `vtpm_gating` is
    /// REJECTED here (config-parse reject). The deeper FS-S21/S23 startup refusals run in
    /// `serve()` via the engine guards.
    pub fn load(default_config_path: &Path) -> anyhow::Result<ProfileSettings> {
        let cfg_path = std::env::var_os(ENV_CONFIG)
            .map(PathBuf::from)
            .unwrap_or_else(|| default_config_path.to_path_buf());
        let file: FileConfig = match std::fs::read_to_string(&cfg_path) {
            Ok(t) => toml::from_str(&t).context("parsing secretd config TOML for [profile]")?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => FileConfig::default(),
            Err(e) => return Err(e).with_context(|| format!("reading {}", cfg_path.display())),
        };
        let fp = file.profile.unwrap_or_default();
        resolve_profile(
            fp,
            env_nonempty("SECRETD_TOPOLOGY"),
            env_nonempty("SECRETD_OPERATOR_AUTHORIZER_URL"),
        )
    }
}

/// Pure, testable resolution core for [`ProfileSettings::load`] — turns the file `[profile]` block
/// plus the env overrides into a validated [`ProfileSettings`]. Enforces FS-S24 (vTPM reject) and
/// the Profile-B required-input rules (FS-S21 substitute factor plus binding inputs). No I/O.
fn resolve_profile(
    fp: FileProfile,
    topology_env: Option<String>,
    url_env: Option<String>,
) -> anyhow::Result<ProfileSettings> {
    {
        // FS-S24: vTPM gating is forbidden — reject at parse time (fail-closed).
        if fp.vtpm_gating {
            bail!(
                "[profile].vtpm_gating is forbidden (FS-S24): a vTPM has no hardware isolation \
                 boundary on a VPS; refusing to gate DEK release on it"
            );
        }

        let topology = match topology_env
            .or(fp.topology)
            .map(|s| s.trim().to_ascii_lowercase())
            .as_deref()
        {
            None | Some("") | Some("onbox") | Some("on-box") | Some("local") => Topology::OnBox,
            Some("remote") | Some("vps") => Topology::Vps,
            Some(other) => bail!(
                "unknown [profile].topology {other:?} (expected \"onbox\" or \"remote\"/\"vps\")"
            ),
        };

        let operator_authorizer_url = url_env.or(fp.operator_authorizer_url);

        if topology == Topology::OnBox {
            return Ok(ProfileSettings {
                topology,
                operator_authorizer_url,
                vps_instance_id: None,
                operator_pubkey: None,
                operator_ca_path: None,
                client_cert_path: None,
                client_key_path: None,
                edge_cert_path: None,
                vtpm_gating_requested: false,
            });
        }

        // Profile B: require the binding inputs (fail-closed — a half-built VPS profile never serves).
        let operator_authorizer_url = operator_authorizer_url.clone().ok_or_else(|| {
            anyhow!(
                "[profile].topology = \"remote\" requires operator_authorizer_url (FS-S21 substitute \
                 presence factor)"
            )
        })?;
        let vps_instance_id = fp
            .vps_instance_id
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| anyhow!("[profile].topology = \"remote\" requires vps_instance_id"))?;
        let operator_pubkey =
            parse_pubkey_hex(fp.operator_pubkey_hex.as_deref().ok_or_else(|| {
                anyhow!("[profile] remote requires operator_pubkey_hex (64-hex)")
            })?)
            .ok_or_else(|| anyhow!("[profile].operator_pubkey_hex must be 64 hex chars"))?;
        let operator_ca_path = PathBuf::from(
            fp.operator_ca_path
                .filter(|s| !s.trim().is_empty())
                .ok_or_else(|| anyhow!("[profile] remote requires operator_ca_path (PEM)"))?,
        );
        let client_cert_path = PathBuf::from(
            fp.client_cert_path
                .filter(|s| !s.trim().is_empty())
                .ok_or_else(|| anyhow!("[profile] remote requires client_cert_path (PEM)"))?,
        );
        let client_key_path = PathBuf::from(
            fp.client_key_path
                .filter(|s| !s.trim().is_empty())
                .ok_or_else(|| anyhow!("[profile] remote requires client_key_path (PEM)"))?,
        );
        let edge_cert_path = fp
            .edge_cert_path
            .filter(|s| !s.trim().is_empty())
            .map(PathBuf::from);

        Ok(ProfileSettings {
            topology,
            operator_authorizer_url: Some(operator_authorizer_url),
            vps_instance_id: Some(vps_instance_id),
            operator_pubkey: Some(operator_pubkey),
            operator_ca_path: Some(operator_ca_path),
            client_cert_path: Some(client_cert_path),
            client_key_path: Some(client_key_path),
            edge_cert_path,
            vtpm_gating_requested: false,
        })
    }
}

/// Decode a 64-char hex Ed25519 public key into 32 bytes. `None` on malformed input.
fn parse_pubkey_hex(s: &str) -> Option<[u8; 32]> {
    let s = s.trim();
    if s.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, chunk) in s.as_bytes().chunks_exact(2).enumerate() {
        let hi = (chunk[0] as char).to_digit(16)?;
        let lo = (chunk[1] as char).to_digit(16)?;
        out[i] = ((hi << 4) | lo) as u8;
    }
    Some(out)
}

/// Load the auth token from `SECRETD_LIBSQL_AUTH_TOKEN`, else from the `0600` file at
/// `SECRETD_LIBSQL_AUTH_TOKEN_FILE`. A group/other-readable token file is refused (fail-closed).
fn load_token() -> anyhow::Result<Option<Zeroizing<String>>> {
    if let Some(t) = env_nonempty(ENV_TOKEN) {
        return Ok(Some(Zeroizing::new(t)));
    }
    if let Some(p) = env_nonempty(ENV_TOKEN_FILE) {
        let path = PathBuf::from(p);
        check_token_file_mode(&path)?;
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("reading token file {}", path.display()))?;
        return Ok(Some(Zeroizing::new(raw.trim().to_string())));
    }
    Ok(None)
}

/// Refuse a token file that is group/other-readable (mode & 0o077 != 0).
fn check_token_file_mode(path: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let meta =
        std::fs::metadata(path).with_context(|| format!("stat token file {}", path.display()))?;
    let mode = meta.permissions().mode();
    if mode & 0o077 != 0 {
        bail!(
            "token file {} is group/other-accessible (mode {:o}); chmod 0600 it",
            path.display(),
            mode & 0o7777
        );
    }
    Ok(())
}

/// Pure, testable validation core: turn the (already env/file-merged) raw values into a validated
/// [`StoreConfig`]. See the module docs for the rules enforced here.
fn resolve(
    backend: Option<String>,
    url: Option<String>,
    token: Option<Zeroizing<String>>,
    require_mlock: bool,
) -> anyhow::Result<StoreConfig> {
    let backend = parse_backend(backend.as_deref())?;
    match backend {
        Backend::InMem => Ok(StoreConfig {
            backend,
            url: None,
            auth_token: Zeroizing::new(String::new()),
            require_mlock,
        }),
        Backend::LibSql => {
            let url = url.filter(|u| !u.trim().is_empty()).ok_or_else(|| {
                anyhow!("store backend = \"libsql\" requires a URL ({ENV_URL} or [store].url)")
            })?;
            url_is_acceptable(&url)?;
            let auth_token = token.unwrap_or_else(|| Zeroizing::new(String::new()));
            if auth_token.is_empty() && !url_host_is_loopback(&url) {
                bail!(
                    "store backend = \"libsql\" to a non-loopback URL requires an auth token \
                     ({ENV_TOKEN} or {ENV_TOKEN_FILE})"
                );
            }
            Ok(StoreConfig {
                backend,
                url: Some(url),
                auth_token,
                require_mlock,
            })
        }
    }
}

fn parse_backend(s: Option<&str>) -> anyhow::Result<Backend> {
    match s.map(|s| s.trim().to_ascii_lowercase()).as_deref() {
        None | Some("") | Some("inmem") | Some("in-mem") | Some("memory") => Ok(Backend::InMem),
        Some("libsql") => Ok(Backend::LibSql),
        Some(other) => bail!("unknown store backend {other:?} (expected \"inmem\" or \"libsql\")"),
    }
}

/// Split a URL into `(scheme_lowercase, host_lowercase)`, stripping userinfo and port (and ipv6
/// brackets). Returns `None` if there is no `scheme://`.
fn split_scheme_host(url: &str) -> Option<(String, String)> {
    let (scheme, rest) = url.split_once("://")?;
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let authority = match authority.rsplit_once('@') {
        Some((_userinfo, host)) => host,
        None => authority,
    };
    let host = if let Some(after) = authority.strip_prefix('[') {
        // [ipv6]:port
        after.split(']').next().unwrap_or("")
    } else {
        authority
            .rsplit_once(':')
            .map(|(h, _)| h)
            .unwrap_or(authority)
    };
    Some((scheme.to_ascii_lowercase(), host.to_ascii_lowercase()))
}

fn host_is_loopback(host: &str) -> bool {
    if host == "localhost" {
        return true;
    }
    if let Ok(v4) = host.parse::<std::net::Ipv4Addr>() {
        return v4.is_loopback();
    }
    if let Ok(v6) = host.parse::<std::net::Ipv6Addr>() {
        return v6.is_loopback();
    }
    false
}

fn url_host_is_loopback(url: &str) -> bool {
    split_scheme_host(url).is_some_and(|(_, host)| host_is_loopback(&host))
}

/// Enforce the transport rule for THIS build. The daemon's libSQL client uses a **plaintext**
/// connector (the gate-clean choice — libSQL's `tls` feature would pull a second rustls; see
/// `secrets-store-libsql/src/sync.rs` + DESIGN-NOTES OI-1). So only **loopback** `http`/`ws` is
/// accepted. A direct TLS URL is refused with guidance to front a remote sqld with a loopback TLS
/// terminator; a plaintext non-loopback URL is refused outright (FS-S7).
fn url_is_acceptable(url: &str) -> anyhow::Result<()> {
    let (scheme, host) = split_scheme_host(url)
        .ok_or_else(|| anyhow!("libSQL url {url:?} has no scheme:// prefix"))?;
    match scheme.as_str() {
        "http" | "ws" if host_is_loopback(&host) => Ok(()),
        "http" | "ws" => bail!(
            "plaintext libSQL url to non-loopback host {host:?} is refused (FS-S7); \
             point secretd at a LOOPBACK sqld (http://127.0.0.1:<port>)"
        ),
        "https" | "wss" | "libsql" => bail!(
            "direct TLS to a remote sqld is not supported in this build: libSQL's `tls` feature would \
             add a SECOND rustls (rustls 0.22 via hyper-rustls 0.25), breaking the single ring-only \
             rustls gate (DESIGN-NOTES OI-1). Run a loopback TLS terminator (stunnel/spiped/cloudflared) \
             and set the URL to http://127.0.0.1:<local-port>"
        ),
        other => bail!("unsupported libSQL url scheme {other:?} (use http/ws to a loopback sqld)"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_parsing() {
        assert_eq!(parse_backend(None).unwrap(), Backend::InMem);
        assert_eq!(parse_backend(Some("")).unwrap(), Backend::InMem);
        assert_eq!(parse_backend(Some("inmem")).unwrap(), Backend::InMem);
        assert_eq!(parse_backend(Some("  InMem ")).unwrap(), Backend::InMem);
        assert_eq!(parse_backend(Some("memory")).unwrap(), Backend::InMem);
        assert_eq!(parse_backend(Some("libsql")).unwrap(), Backend::LibSql);
        assert_eq!(parse_backend(Some(" LIBSQL ")).unwrap(), Backend::LibSql);
        assert!(parse_backend(Some("postgres")).is_err());
    }

    #[test]
    fn scheme_host_split() {
        assert_eq!(
            split_scheme_host("http://127.0.0.1:8080"),
            Some(("http".into(), "127.0.0.1".into()))
        );
        assert_eq!(
            split_scheme_host("https://Db.Turso.IO/path?x=1"),
            Some(("https".into(), "db.turso.io".into()))
        );
        assert_eq!(
            split_scheme_host("http://[::1]:8080/x"),
            Some(("http".into(), "::1".into()))
        );
        assert_eq!(
            split_scheme_host("libsql://user:pw@host.example:443"),
            Some(("libsql".into(), "host.example".into()))
        );
        assert_eq!(split_scheme_host("no-scheme"), None);
    }

    #[test]
    fn loopback_detection() {
        assert!(host_is_loopback("127.0.0.1"));
        assert!(host_is_loopback("127.5.6.7"));
        assert!(host_is_loopback("localhost"));
        assert!(host_is_loopback("::1"));
        assert!(!host_is_loopback("10.0.0.1"));
        assert!(!host_is_loopback("db.turso.io"));
        assert!(!host_is_loopback("0.0.0.0"));
    }

    #[test]
    fn url_acceptability() {
        // Plaintext to loopback: the ONLY accepted transport in this build.
        assert!(url_is_acceptable("http://127.0.0.1:8080").is_ok());
        assert!(url_is_acceptable("http://localhost:8080").is_ok());
        assert!(url_is_acceptable("http://[::1]:8080").is_ok());
        assert!(url_is_acceptable("ws://127.0.0.1:8080").is_ok());
        // Plaintext to a remote host: REFUSED (FS-S7).
        assert!(url_is_acceptable("http://db.turso.io:8080").is_err());
        assert!(url_is_acceptable("ws://10.0.0.1:8080").is_err());
        // Direct TLS: REFUSED (would add a 2nd rustls; use a loopback terminator).
        for u in [
            "https://db.turso.io",
            "wss://db.turso.io",
            "libsql://db.turso.io",
        ] {
            let e = url_is_acceptable(u).unwrap_err().to_string();
            assert!(
                e.contains("terminator") || e.contains("second rustls"),
                "unexpected msg for {u}: {e}"
            );
        }
        // Garbage / unsupported scheme.
        assert!(url_is_acceptable("ftp://x").is_err());
        assert!(url_is_acceptable("noscheme").is_err());
    }

    #[test]
    fn resolve_inmem_default_ignores_libsql_fields() {
        let c = resolve(None, Some("http://db.turso.io".into()), None, false).unwrap();
        assert_eq!(c.backend, Backend::InMem);
        assert!(c.url.is_none());
        assert!(c.auth_token.is_empty());
    }

    #[test]
    fn resolve_libsql_requires_url() {
        assert!(resolve(Some("libsql".into()), None, None, false).is_err());
        assert!(resolve(Some("libsql".into()), Some("   ".into()), None, false).is_err());
    }

    #[test]
    fn resolve_libsql_refuses_plaintext_remote() {
        let err = resolve(
            Some("libsql".into()),
            Some("http://db.turso.io:8080".into()),
            Some(Zeroizing::new("tok".into())),
            false,
        )
        .unwrap_err();
        assert!(err.to_string().contains("non-loopback"));
    }

    #[test]
    fn resolve_libsql_rejects_direct_tls() {
        // https is refused in this build (would add a 2nd rustls) — even WITH a token.
        let err = resolve(
            Some("libsql".into()),
            Some("https://db.turso.io".into()),
            Some(Zeroizing::new("tok".into())),
            false,
        )
        .unwrap_err()
        .to_string();
        assert!(
            err.contains("terminator") || err.contains("second rustls"),
            "unexpected msg: {err}"
        );
    }

    #[test]
    fn resolve_libsql_loopback_with_token_ok() {
        // A loopback sqld may still require a token (e.g. a loopback terminator to an auth'd remote).
        let c = resolve(
            Some("libsql".into()),
            Some("http://127.0.0.1:8080".into()),
            Some(Zeroizing::new("tok".into())),
            false,
        )
        .unwrap();
        assert_eq!(c.backend, Backend::LibSql);
        assert_eq!(c.url.as_deref(), Some("http://127.0.0.1:8080"));
        assert_eq!(&*c.auth_token, "tok");
    }

    #[test]
    fn resolve_libsql_loopback_allows_empty_token() {
        let c = resolve(
            Some("libsql".into()),
            Some("http://127.0.0.1:8080".into()),
            None,
            false,
        )
        .unwrap();
        assert_eq!(c.backend, Backend::LibSql);
        assert!(c.auth_token.is_empty());
    }

    #[test]
    fn debug_redacts_token() {
        let c = resolve(
            Some("libsql".into()),
            Some("http://127.0.0.1:8080".into()),
            Some(Zeroizing::new("super-secret".into())),
            false,
        )
        .unwrap();
        let s = format!("{c:?}");
        assert!(s.contains("<redacted>"));
        assert!(!s.contains("super-secret"));
    }

    #[test]
    fn require_mlock_defaults_false_and_threads_through() {
        // Default (best-effort) when unset.
        let c = resolve(None, None, None, false).unwrap();
        assert!(!c.require_mlock);
        // Strict mode threads through on both backends.
        let c = resolve(None, None, None, true).unwrap();
        assert!(c.require_mlock);
        let c = resolve(
            Some("libsql".into()),
            Some("http://127.0.0.1:8080".into()),
            None,
            true,
        )
        .unwrap();
        assert!(c.require_mlock);
    }

    #[test]
    fn env_bool_parsing() {
        // Direct unit check of the truthy/falsy/unrecognized mapping (env-independent).
        for t in ["1", "true", "TRUE", "Yes", "on"] {
            std::env::set_var("SECRETD_TEST_ENV_BOOL", t);
            assert_eq!(env_bool("SECRETD_TEST_ENV_BOOL"), Some(true), "for {t}");
        }
        for f in ["0", "false", "No", "OFF"] {
            std::env::set_var("SECRETD_TEST_ENV_BOOL", f);
            assert_eq!(env_bool("SECRETD_TEST_ENV_BOOL"), Some(false), "for {f}");
        }
        std::env::set_var("SECRETD_TEST_ENV_BOOL", "maybe");
        assert_eq!(env_bool("SECRETD_TEST_ENV_BOOL"), None);
        std::env::remove_var("SECRETD_TEST_ENV_BOOL");
        assert_eq!(env_bool("SECRETD_TEST_ENV_BOOL"), None);
    }

    // ---- TASK-0033: [profile] resolution (resolve_profile is pure — no env/file I/O) ----------

    fn profile_b_file() -> FileProfile {
        FileProfile {
            topology: Some("remote".into()),
            operator_authorizer_url: Some("https://operator.box:9443".into()),
            vps_instance_id: Some("vps-1".into()),
            operator_pubkey_hex: Some("00".repeat(32)),
            operator_ca_path: Some("/etc/op/ca.pem".into()),
            client_cert_path: Some("/etc/op/client.pem".into()),
            client_key_path: Some("/etc/op/client.key".into()),
            edge_cert_path: None,
            vtpm_gating: false,
        }
    }

    #[test]
    fn profile_default_is_onbox() {
        let p = resolve_profile(FileProfile::default(), None, None).expect("onbox default");
        assert_eq!(p.topology, Topology::OnBox);
        assert!(p.operator_authorizer_url.is_none());
    }

    #[test]
    fn profile_vtpm_gating_rejected_fs_s24() {
        let fp = FileProfile {
            vtpm_gating: true,
            ..FileProfile::default()
        };
        let err = resolve_profile(fp, None, None).unwrap_err().to_string();
        assert!(
            err.contains("FS-S24"),
            "vTPM gating must be rejected: {err}"
        );
    }

    #[test]
    fn profile_vps_requires_authorizer_url_fs_s21() {
        let mut fp = profile_b_file();
        fp.operator_authorizer_url = None;
        let err = resolve_profile(fp, None, None).unwrap_err().to_string();
        assert!(
            err.contains("operator_authorizer_url"),
            "VPS without authorizer URL must refuse: {err}"
        );
    }

    #[test]
    fn profile_vps_requires_binding_inputs() {
        // Missing instance id.
        let mut fp = profile_b_file();
        fp.vps_instance_id = None;
        assert!(resolve_profile(fp, None, None).is_err());
        // Bad pubkey length.
        let mut fp = profile_b_file();
        fp.operator_pubkey_hex = Some("deadbeef".into());
        assert!(resolve_profile(fp, None, None).is_err());
        // Missing client key.
        let mut fp = profile_b_file();
        fp.client_key_path = None;
        assert!(resolve_profile(fp, None, None).is_err());
    }

    #[test]
    fn profile_vps_full_config_resolves() {
        let p = resolve_profile(profile_b_file(), None, None).expect("full VPS profile");
        assert_eq!(p.topology, Topology::Vps);
        assert_eq!(
            p.operator_authorizer_url.as_deref(),
            Some("https://operator.box:9443")
        );
        assert_eq!(p.vps_instance_id.as_deref(), Some("vps-1"));
        assert_eq!(p.operator_pubkey, Some([0u8; 32]));
        assert!(!p.vtpm_gating_requested);
        assert_eq!(p.topology.to_engine(), envctl_secrets::Topology::Vps);
    }

    #[test]
    fn profile_env_topology_overrides_file() {
        // File says onbox; env forces remote (then the binding inputs are still required).
        let mut fp = profile_b_file();
        fp.topology = Some("onbox".into());
        let p = resolve_profile(fp, Some("remote".into()), None).expect("env override to remote");
        assert_eq!(p.topology, Topology::Vps);
    }

    #[test]
    fn profile_unknown_topology_rejected() {
        let fp = FileProfile {
            topology: Some("cloud".into()),
            ..FileProfile::default()
        };
        assert!(resolve_profile(fp, None, None).is_err());
    }

    #[test]
    fn parse_pubkey_hex_validates_length() {
        assert!(parse_pubkey_hex(&"ab".repeat(32)).is_some());
        assert!(parse_pubkey_hex("dead").is_none());
        assert!(parse_pubkey_hex(&"zz".repeat(32)).is_none());
    }
}
