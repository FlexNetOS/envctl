//! The behavioral seams (the envctl `HookRunner` family) — all `Send + Sync` so the `Engine`
//! stays `Send + Sync`. Real impls live here; fakes for tests are injected via `Engine::with_seams`.
use zeroize::Zeroizing;

/// Wall + monotonic clock. `boottime_ms` is a `CLOCK_BOOTTIME` cross-check for clock-rollback
/// defense on the 24h relay window (OI-6).
pub trait Clock: Send + Sync {
    fn now(&self) -> chrono::DateTime<chrono::Utc>;
    fn boottime_ms(&self) -> i64;
}
/// Forward through a reference (incl. `&dyn Clock`) so a borrowed engine clock satisfies the
/// generic `Clock` bound on `GitHubAppMint::new` (TASK-0020).
impl<C: Clock + ?Sized> Clock for &C {
    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        (**self).now()
    }
    fn boottime_ms(&self) -> i64 {
        (**self).boottime_ms()
    }
}
/// External **trusted time** source (OI-SM-3, SERVER-MODE Profile B). A VPS clock is
/// hypervisor-controlled, so the presence-token verifier MUST NOT trust the local wall clock to
/// decide whether a token is expired — a hostile host could wind the clock back to keep a stale
/// token "valid". `now_ms()` returns `Some(t)` ONLY when a fresh, externally-attested time is
/// available; `None` ⇒ time is unverified/stale and token issuance/acceptance MUST be refused
/// (fail-closed). On-box (Profile A) the local clock is trusted, so [`SystemClockTrustedTime`]
/// always returns `Some`.
pub trait TrustedTime: Send + Sync {
    /// The current externally-attested wall-clock epoch-ms, or `None` when no fresh trusted time is
    /// available (fail-closed: callers REFUSE to issue/accept a token).
    fn now_ms(&self) -> Option<i64>;
}

/// Forward through an `Arc` so the daemon can hold an `Arc<OperatorBoxTrustedTime>` (to push
/// attestations from the authorizer task) AND pass a `Box::new(arc.clone())` into `with_seams` —
/// both sides share ONE trusted-time source.
impl<T: TrustedTime + ?Sized> TrustedTime for std::sync::Arc<T> {
    fn now_ms(&self) -> Option<i64> {
        (**self).now_ms()
    }
}

/// Profile A trusted-time: the local wall clock is authoritative on a box the operator controls, so
/// this always returns `Some(now)`. The engine default (Profile B installs
/// [`OperatorBoxTrustedTime`] instead, which returns `None` when stale/unverified).
pub struct SystemClockTrustedTime;
impl TrustedTime for SystemClockTrustedTime {
    fn now_ms(&self) -> Option<i64> {
        Some(chrono::Utc::now().timestamp_millis())
    }
}

/// Profile B (VPS) trusted-time: holds the last externally-attested wall-clock reading (fed by the
/// operator-box authorizer link over mTLS, U14). `now_ms()` returns `Some(attested_now)` ONLY while
/// the attestation is still within its freshness window; otherwise `None` (fail-closed: a VPS whose
/// trusted-time feed has stalled refuses to issue or accept any presence token). The attested time
/// and its receipt instant are interior-mutable (`Mutex`) so the async authorizer task can refresh
/// it while the per-request egress path reads it.
pub struct OperatorBoxTrustedTime {
    /// `(attested_now_ms, received_at_local_ms)` of the most recent fresh attestation, or `None`
    /// when never attested. Stale once `local_now - received_at > freshness_ms`.
    state: std::sync::Mutex<Option<(i64, i64)>>,
    /// How long (ms) an attestation stays fresh before `now_ms()` falls back to `None`.
    freshness_ms: i64,
    /// Local monotonic-ish wall clock used only to AGE the attestation (never to answer `now_ms`).
    local_clock: Box<dyn Clock>,
}

impl OperatorBoxTrustedTime {
    /// Default freshness window for an attested external time (10 min — coherent with the presence
    /// token's default TTL: once a token could not have been issued under fresh time, trusted time
    /// is useless anyway).
    pub const DEFAULT_FRESHNESS_MS: i64 = 600_000;

    /// Build with the default freshness window over the given local clock (used to AGE attestations).
    #[must_use]
    pub fn new(local_clock: Box<dyn Clock>) -> Self {
        Self::with_freshness(local_clock, Self::DEFAULT_FRESHNESS_MS)
    }

    /// Build with a tuned freshness window (lets a test shrink the staleness boundary).
    #[must_use]
    pub fn with_freshness(local_clock: Box<dyn Clock>, freshness_ms: i64) -> Self {
        Self {
            state: std::sync::Mutex::new(None),
            freshness_ms,
            local_clock,
        }
    }

    /// Record a fresh externally-attested wall-clock reading (called by the authorizer link when it
    /// verifies a signed time from the operator box). Resets the freshness window. A poisoned lock is
    /// treated as "no attestation" on the read side (fail-closed), so we ignore a poisoned write.
    pub fn attest(&self, attested_now_ms: i64) {
        let received_at = self.local_clock.now().timestamp_millis();
        if let Ok(mut g) = self.state.lock() {
            *g = Some((attested_now_ms, received_at));
        }
    }
}

impl TrustedTime for OperatorBoxTrustedTime {
    fn now_ms(&self) -> Option<i64> {
        let g = self.state.lock().ok()?;
        let (attested, received_at) = (*g)?;
        let local_now = self.local_clock.now().timestamp_millis();
        // Stale (or a backwards local jump that makes the age negative-then-huge is bounded by
        // saturating_sub) ⇒ fail closed.
        if local_now.saturating_sub(received_at) > self.freshness_ms {
            return None;
        }
        // Report the attested time advanced by however long we've held it locally (so a long-lived
        // engine still expires tokens correctly within the freshness window).
        Some(attested.saturating_add(local_now.saturating_sub(received_at)))
    }
}

pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }
    /// `CLOCK_BOOTTIME` in milliseconds: a monotonic counter since boot that INCLUDES suspend time
    /// and CANNOT be stepped backward by the operator, NTP, or a settimeofday() rollback — exactly
    /// the property the OI-6 relay rollback fence needs. Read via `rustix::time::clock_gettime`
    /// (pure-Rust linux_raw syscall on Linux; no C). Saturating ms conversion; never panics.
    fn boottime_ms(&self) -> i64 {
        let ts = rustix::time::clock_gettime(rustix::time::ClockId::Boottime);
        ts.tv_sec
            .saturating_mul(1000)
            .saturating_add(ts.tv_nsec / 1_000_000)
    }
}

/// USB key probe. Resolves the GPT PARTUUID as a pre-filter, then returns the keyfile bytes so
/// the engine can PROVE possession (by unwrapping the USB keyslot). `None` => USB absent or
/// possession unproven (fail-closed). UUID match alone is NOT presence (CF-4/OI-5).
pub trait UsbProbe: Send + Sync {
    fn keyfile_for(&self, partition_uuid: &str) -> Option<Zeroizing<Vec<u8>>>;
}

/// Production USB possession probe.
///
/// **Default build** (no `seed-factor`): no hardware backend is compiled in, so this returns
/// `None` — "USB absent", the correct fail-closed default (callers gate on `Some`; this is *not*
/// a panic).
///
/// **Under `seed-factor`**: possession is proven by the **Cognitum Seed** hardware root of trust.
/// The Seed's Ed25519 device key (private key never leaves the device) deterministically signs a
/// fixed, PARTUUID-bound domain-separated message via `POST /api/v1/custody/sign`. Ed25519 signing
/// is deterministic (verified by spike 2026-06-13, stable across a device restart), so the 64-byte
/// signature is reproducible key material that ONLY a holder of the Seed can produce — exactly the
/// IKM that [`crate::keyslot::kek_from_usb`] expects. The signature is fetched by a **direct,
/// pure-Rust HTTPS call** (ring-only `rustls`, already in the resolved graph) to the Seed over the
/// USB link-local interface, validating the Seed's TLS against the **pinned Cognitum CA** — no
/// `ssh`, no `known_hosts`, no agent, no `$HOME` access, so the daemon's Seed path works unchanged
/// under the `env-ctl.service` systemd sandbox AND the no-C trust-boundary gate stays green. Any
/// failure → `None` (fail-closed).
pub struct RealUsbProbe;

impl UsbProbe for RealUsbProbe {
    #[cfg(not(feature = "seed-factor"))]
    fn keyfile_for(&self, _uuid: &str) -> Option<Zeroizing<Vec<u8>>> {
        None
    }

    #[cfg(feature = "seed-factor")]
    fn keyfile_for(&self, partition_uuid: &str) -> Option<Zeroizing<Vec<u8>>> {
        seed_factor::keyfile_for(partition_uuid)
    }
}

/// Cognitum Seed possession backend for [`RealUsbProbe`]. Isolated so the default build compiles
/// none of it. See `PLAN-cognitum-seed-envctl-vault-factor.md` (meta root) for the design + spike
/// evidence.
///
/// # Transport (systemd-sandbox-safe)
/// The Seed is reached by a **direct, blocking, pure-Rust HTTPS client** (`rustls`, ring-only —
/// already in the resolved graph, so the no-C gate stays green). The server's TLS is validated
/// against the **pinned Cognitum CA only** (loaded from `ENVCTL_SEED_CA`; frozen-roots discipline
/// per FS-S7 — never the OS trust store). This replaces the former `ssh genesis@…` + on-device
/// `curl` tunnel, which broke under `env-ctl.service` (`ProtectHome=read-only` ⇒ no writable
/// `known_hosts`, no agent). No `ssh`, no `$HOME` access, no subprocess.
///
/// # Auth (bearer token, possession-floored)
/// `custody/sign` requires a bearer token minted by the **USB-only** pair window. The token is
/// device-bound and revocable (not a master secret); it is resolved from `ENVCTL_SEED_TOKEN`, else
/// the token file (`ENVCTL_SEED_TOKEN_FILE`, default `$XDG_DATA_HOME/env-ctl/seed-token`, else
/// `$META_ROOT/.local/share/env-ctl/seed-token`, which is inside the unit's `ReadWritePaths`). If
/// absent or rejected, the daemon **re-mints on demand** by
/// re-opening the USB-only pair window (possession of the USB is the floor of trust — ADR-057), so
/// a lost/expired token is self-healing as long as the Seed is present. Every device call is bound
/// by `IO_TIMEOUT` so a wedged device can never hang the synchronous unlock path.
#[cfg(feature = "seed-factor")]
pub(crate) mod seed_factor {
    use std::io::{Read, Write};
    use std::net::{TcpStream, ToSocketAddrs};
    use std::sync::Arc;
    use std::time::Duration;
    use zeroize::Zeroizing;

    /// Base URL of the Seed REST API. Default = the USB link-local address from the device docs;
    /// overridable for mDNS (`.local`) / WiFi addressing.
    fn api_base() -> String {
        std::env::var("ENVCTL_SEED_API").unwrap_or_else(|_| "https://169.254.42.1:8443".to_string())
    }

    /// Pinned Cognitum CA (PEM). The CA is name-constrained to `169.254.x.x` + `.local`; we
    /// pin THIS root explicitly (FS-S7 frozen-roots) rather than trusting the OS store. The
    /// envctl unit sets `ENVCTL_SEED_CA`; the fallback is the canonical meta-local share path,
    /// with a read-only legacy `.toolchains` fallback only when that file already exists.
    fn ca_path() -> String {
        if let Ok(path) = std::env::var("ENVCTL_SEED_CA") {
            return path;
        }
        let meta = meta_root();
        let canonical = canonical_seed_ca(&meta);
        if canonical.is_file() {
            return canonical.to_string_lossy().into_owned();
        }
        let legacy = legacy_seed_ca(&meta);
        if legacy.is_file() {
            return legacy.to_string_lossy().into_owned();
        }
        canonical.to_string_lossy().into_owned()
    }

    fn meta_root() -> std::path::PathBuf {
        if let Ok(meta) = std::env::var("META_ROOT") {
            return std::path::PathBuf::from(meta);
        }
        if let Ok(home) = std::env::var("HOME") {
            return std::path::PathBuf::from(home).join("Desktop/meta");
        }
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    }

    fn canonical_seed_ca(meta: &std::path::Path) -> std::path::PathBuf {
        meta.join(".local/share/envctl/secrets/ca/cognitum-ca.crt")
    }

    fn legacy_seed_ca(meta: &std::path::Path) -> std::path::PathBuf {
        meta.join(".toolchains/secrets/ca/cognitum-ca.crt")
    }

    /// Stable pairing-client name for the daemon. Re-pairing under the same name replaces the
    /// previous token (no per-unlock client leak).
    const CLIENT_NAME: &str = "envctl-daemon";

    /// I/O ceiling for any single device call (connect, read, write). A wedged device drops within
    /// this bound so the synchronous unlock path can never block indefinitely.
    const IO_TIMEOUT: Duration = Duration::from_secs(15);

    /// Device-bound bearer token at rest. Default lives in the unit's `ReadWritePaths`
    /// (`$META_ROOT/.local/share/env-ctl`) so the daemon can both read and refresh it under the sandbox.
    fn token_file() -> std::path::PathBuf {
        if let Ok(p) = std::env::var("ENVCTL_SEED_TOKEN_FILE") {
            return std::path::PathBuf::from(p);
        }
        let base = std::env::var("XDG_DATA_HOME")
            .ok()
            .filter(|p| !p.is_empty())
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                let root = std::env::var("META_ROOT")
                    .ok()
                    .filter(|p| !p.is_empty())
                    .or_else(|| std::env::var("HOME").ok().filter(|p| !p.is_empty()))
                    .unwrap_or_default();
                std::path::PathBuf::from(root).join(".local/share")
            });
        base.join("env-ctl").join("seed-token")
    }

    /// Resolve a bearer token: explicit env override first, then the token file. Trimmed; empty
    /// ⇒ `None`.
    fn resolve_token() -> Option<Zeroizing<String>> {
        if let Ok(t) = std::env::var("ENVCTL_SEED_TOKEN") {
            let t = t.trim().to_string();
            if !t.is_empty() {
                return Some(Zeroizing::new(t));
            }
        }
        let raw = std::fs::read_to_string(token_file()).ok()?;
        let t = raw.trim().to_string();
        if t.is_empty() {
            None
        } else {
            Some(Zeroizing::new(t))
        }
    }

    /// Persist a freshly minted token at `0600` (best-effort; failure just means we re-pair next
    /// time).
    fn store_token(token: &str) {
        use std::os::unix::fs::OpenOptionsExt;
        let path = token_file();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&path)
        {
            let _ = f.write_all(token.as_bytes());
        }
    }

    /// Domain-separated, PARTUUID-bound context the Seed signs. Binding the slot UUID into the
    /// message means a different slot derives a different KEK from the same device key.
    fn kek_context(partition_uuid: &str) -> String {
        std::env::var("ENVCTL_SEED_KEK_CONTEXT")
            .unwrap_or_else(|_| format!("envctl/usb-kek/v1/{partition_uuid}"))
    }

    /// Decode a 128-char hex Ed25519 signature into 64 bytes. `None` on any malformed input
    /// (wrong length / non-hex) — fail-closed.
    pub(crate) fn parse_sig_hex(s: &str) -> Option<[u8; 64]> {
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

    /// Decode a 64-char hex Ed25519 public key into 32 bytes. `None` on any malformed input.
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

    /// The operator-pinned Seed device public key (`ENVCTL_SEED_PUBKEY`, 64-hex), if configured.
    /// Same knob the presence gate (Profile S) pins on — when set, the KEK probe authenticates the
    /// signature too; when unset, neither path can verify (backward-compatible).
    fn pinned_pubkey() -> Option<[u8; 32]> {
        parse_pubkey_hex(&std::env::var("ENVCTL_SEED_PUBKEY").ok()?)
    }

    /// Build the pinned-CA, ring-only rustls client config. Loads ONLY the Cognitum CA as the trust
    /// root (frozen-roots; NOT the OS store). `None` if the CA is missing / unreadable / empty.
    fn tls_config() -> Option<Arc<rustls::ClientConfig>> {
        let pem = std::fs::read(ca_path()).ok()?;
        let mut roots = rustls::RootCertStore::empty();
        let mut rd = std::io::BufReader::new(&pem[..]);
        for cert in rustls_pemfile::certs(&mut rd) {
            roots.add(cert.ok()?).ok()?;
        }
        if roots.is_empty() {
            return None;
        }
        let cfg = rustls::ClientConfig::builder_with_provider(
            rustls::crypto::ring::default_provider().into(),
        )
        .with_safe_default_protocol_versions()
        .ok()?
        .with_root_certificates(roots)
        .with_no_client_auth();
        Some(Arc::new(cfg))
    }

    /// Split an `https://host:port` base into `(host, port)` (port defaults to 443).
    fn host_port(base: &str) -> Option<(String, u16)> {
        let rest = base.strip_prefix("https://")?;
        let rest = rest.split('/').next().unwrap_or(rest);
        match rest.rsplit_once(':') {
            Some((h, p)) => Some((h.to_string(), p.parse().ok()?)),
            None => Some((rest.to_string(), 443)),
        }
    }

    /// Parse the numeric status code from an HTTP/1.1 status line (`HTTP/1.1 200 OK`).
    fn parse_status(resp: &str) -> Option<u16> {
        resp.lines().next()?.split_whitespace().nth(1)?.parse().ok()
    }

    /// Extract a JSON string field value (`"name":"value"`) by scanning the raw response — robust
    /// against chunked transfer framing (chunk-size lines never contain the field name). The value
    /// is read up to the next quote (the Seed's signature/token values contain no escaped quotes).
    fn extract_field(resp: &str, name: &str) -> Option<String> {
        let key = format!("\"{name}\"");
        let after = &resp[resp.find(&key)? + key.len()..];
        let after = after.trim_start().strip_prefix(':')?.trim_start();
        let after = after.strip_prefix('"')?;
        let end = after.find('"')?;
        let val = &after[..end];
        if val.is_empty() {
            None
        } else {
            Some(val.to_string())
        }
    }

    /// One blocking HTTPS request to the Seed; returns `(status, raw_response_text)`.
    /// `Connection: close` makes the body close-delimited, so we read to EOF (rustls may surface a
    /// non-graceful TCP close as an error *after* the body is already buffered — tolerated). `None`
    /// on transport failure.
    fn https(
        cfg: &Arc<rustls::ClientConfig>,
        method: &str,
        path: &str,
        headers: &[(&str, &str)],
        body: &str,
    ) -> Option<(u16, String)> {
        let (host, port) = host_port(&api_base())?;
        let server_name = rustls::pki_types::ServerName::try_from(host.clone()).ok()?;
        let mut conn = rustls::ClientConnection::new(Arc::clone(cfg), server_name).ok()?;
        let addr = (host.as_str(), port).to_socket_addrs().ok()?.next()?;
        let mut sock = TcpStream::connect_timeout(&addr, IO_TIMEOUT).ok()?;
        sock.set_read_timeout(Some(IO_TIMEOUT)).ok()?;
        sock.set_write_timeout(Some(IO_TIMEOUT)).ok()?;
        let mut tls = rustls::Stream::new(&mut conn, &mut sock);

        let mut req = format!(
            "{method} {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\nContent-Length: {}\r\n",
            body.len()
        );
        for (k, v) in headers {
            req.push_str(k);
            req.push_str(": ");
            req.push_str(v);
            req.push_str("\r\n");
        }
        req.push_str("\r\n");
        req.push_str(body);
        tls.write_all(req.as_bytes()).ok()?;
        tls.flush().ok()?;

        let mut buf = Vec::new();
        let _ = tls.read_to_end(&mut buf); // non-graceful close after full body ⇒ tolerated
        let text = String::from_utf8_lossy(&buf).into_owned();
        let status = parse_status(&text)?;
        Some((status, text))
    }

    /// `POST /api/v1/custody/sign` with a bearer token. Returns `(status, signature?)`.
    fn custody_sign(
        cfg: &Arc<rustls::ClientConfig>,
        token: &str,
        data: &str,
    ) -> Option<(u16, Option<String>)> {
        let body = format!("{{\"data\":\"{data}\"}}");
        let auth = format!("Bearer {token}");
        let (status, resp) = https(
            cfg,
            "POST",
            "/api/v1/custody/sign",
            &[
                ("Authorization", &auth),
                ("Content-Type", "application/json"),
            ],
            &body,
        )?;
        Some((status, extract_field(&resp, "signature")))
    }

    /// `(status, signature)` → the signature only when the call was `2xx` and non-empty.
    fn ok_sig(res: Option<(u16, Option<String>)>) -> Option<String> {
        let (status, sig) = res?;
        if !(200..300).contains(&status) {
            return None;
        }
        sig.filter(|s| !s.is_empty())
    }

    /// Re-mint a device-bound bearer token via the **USB-only** pair window (possession floor) and
    /// persist it. `None` if the window/pair is unavailable (e.g. Seed absent).
    fn pair_and_store(cfg: &Arc<rustls::ClientConfig>) -> Option<Zeroizing<String>> {
        let (w, _) = https(cfg, "POST", "/api/v1/pair/window", &[], "")?;
        if !(200..300).contains(&w) {
            return None;
        }
        let body = format!("{{\"client_name\":\"{CLIENT_NAME}\"}}");
        let (p, resp) = https(
            cfg,
            "POST",
            "/api/v1/pair",
            &[("Content-Type", "application/json")],
            &body,
        )?;
        if !(200..300).contains(&p) {
            return None;
        }
        let token = extract_field(&resp, "token")?;
        store_token(&token);
        Some(Zeroizing::new(token))
    }

    /// Sign arbitrary `data` with the Seed's Ed25519 device key over the REST custody API and return
    /// the 128-char hex signature. `None` on any failure (Seed unreachable / unpaired / empty).
    /// Single implementation shared by the KEK probe and the presence gate (Profile S).
    ///
    /// Flow: validate TLS against the pinned Cognitum CA → try the stored/env bearer token → on a
    /// missing or rejected token, re-mint once via the USB-only pair window (possession floor) and
    /// retry. Every device call is bounded by `IO_TIMEOUT` so a wedged device cannot hang the
    /// synchronous unlock path.
    pub(crate) fn sign_hex(data: &str) -> Option<String> {
        let cfg = tls_config()?;

        // 1. Try an already-provisioned token (env override or token file).
        if let Some(token) = resolve_token() {
            if let Some(sig) = ok_sig(custody_sign(&cfg, &token, data)) {
                return Some(sig);
            }
            // else: token revoked / expired / forbidden — fall through to re-mint.
        }

        // 2. Re-mint on demand (USB possession is the trust floor) and retry once.
        let token = pair_and_store(&cfg)?;
        ok_sig(custody_sign(&cfg, &token, data))
    }

    /// Resolve the USB keyslot keyfile from the Seed: the deterministic signature over the
    /// PARTUUID-bound KEK context, as 64 raw bytes. `partition_uuid` binds the derived KEK to the
    /// specific slot. Returns `None` on any failure so the engine fails closed.
    ///
    /// HARDENING: when a device public key is pinned (`ENVCTL_SEED_PUBKEY` — the same knob the
    /// presence gate authenticates with), the signature is **verified with `ring` Ed25519 against
    /// that key before it is trusted as KEK material**. A forged/mismatched responder is rejected
    /// here as a clean possession denial (`None`), instead of silently producing wrong key bytes
    /// that surface only as a downstream KEK-unwrap failure. With no pinned key the behavior is
    /// unchanged (the operator hasn't given us anything to authenticate against).
    pub(super) fn keyfile_for(partition_uuid: &str) -> Option<Zeroizing<Vec<u8>>> {
        let ctx = kek_context(partition_uuid);
        let sig = parse_sig_hex(&sign_hex(&ctx)?)?;
        if let Some(pubkey) = pinned_pubkey() {
            let key = ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, &pubkey);
            if key.verify(ctx.as_bytes(), &sig).is_err() {
                return None; // responder not authenticated by the pinned key — fail closed
            }
        }
        Some(Zeroizing::new(sig.to_vec()))
    }

    #[cfg(test)]
    mod tests {
        use super::{
            ca_path, canonical_seed_ca, extract_field, host_port, legacy_seed_ca, parse_pubkey_hex,
            parse_sig_hex, parse_status, token_file,
        };
        use std::sync::{Mutex, MutexGuard, OnceLock};

        fn env_lock() -> MutexGuard<'static, ()> {
            static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
            ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
        }

        fn restore_var(key: &str, value: Option<String>) {
            match value {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }

        #[test]
        fn parse_sig_hex_roundtrips_64_bytes() {
            // The spike signature (2026-06-13) — a real 128-hex Ed25519 signature.
            let hex = "90017fccf53948ce509c216d1cf64c6cdd75d50a9f28e63cef27d6706a7b4c765de7a2849dc8c1d6b19f5ee6e3211b8142b669ca8b6c1fb16a6dc989dc5fa60e";
            let b = parse_sig_hex(hex).expect("valid 128-hex parses");
            assert_eq!(b.len(), 64);
            assert_eq!(b[0], 0x90);
            assert_eq!(b[63], 0x0e);
        }

        #[test]
        fn parse_sig_hex_rejects_malformed() {
            assert!(parse_sig_hex("dead").is_none(), "too short");
            assert!(parse_sig_hex(&"zz".repeat(64)).is_none(), "non-hex");
            assert!(
                parse_sig_hex(&"00".repeat(63)).is_none(),
                "126 hex = wrong length"
            );
        }

        #[test]
        fn ca_path_defaults_to_meta_local_share() {
            let _guard = env_lock();
            let old_seed_ca = std::env::var("ENVCTL_SEED_CA").ok();
            let old_meta = std::env::var("META_ROOT").ok();
            std::env::remove_var("ENVCTL_SEED_CA");
            std::env::set_var("META_ROOT", "/tmp/meta-envctl-test");
            assert_eq!(
                ca_path(),
                "/tmp/meta-envctl-test/.local/share/envctl/secrets/ca/cognitum-ca.crt"
            );
            restore_var("ENVCTL_SEED_CA", old_seed_ca);
            restore_var("META_ROOT", old_meta);
        }

        #[test]
        fn ca_path_uses_legacy_toolchains_when_present() {
            let _guard = env_lock();
            let old_seed_ca = std::env::var("ENVCTL_SEED_CA").ok();
            let old_meta = std::env::var("META_ROOT").ok();
            let tmp = std::env::temp_dir().join(format!(
                "envctl-seed-ca-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("system clock after unix epoch")
                    .as_nanos()
            ));
            let legacy = legacy_seed_ca(&tmp);
            std::fs::create_dir_all(legacy.parent().expect("legacy parent")).expect("mkdir");
            std::fs::write(&legacy, b"legacy ca").expect("write legacy ca");
            assert!(!canonical_seed_ca(&tmp).exists());
            std::env::remove_var("ENVCTL_SEED_CA");
            std::env::set_var("META_ROOT", &tmp);
            assert_eq!(ca_path(), legacy.to_string_lossy().into_owned());
            restore_var("ENVCTL_SEED_CA", old_seed_ca);
            restore_var("META_ROOT", old_meta);
            let _ = std::fs::remove_dir_all(tmp);
        }

        #[test]
        fn ca_path_honors_explicit_override() {
            let _guard = env_lock();
            let old_seed_ca = std::env::var("ENVCTL_SEED_CA").ok();
            std::env::set_var("ENVCTL_SEED_CA", "/tmp/custom-ca.pem");
            assert_eq!(ca_path(), "/tmp/custom-ca.pem");
            restore_var("ENVCTL_SEED_CA", old_seed_ca);
        }

        #[test]
        fn token_file_defaults_to_meta_local_share() {
            let _guard = env_lock();
            let old_token_file = std::env::var("ENVCTL_SEED_TOKEN_FILE").ok();
            let old_xdg_data = std::env::var("XDG_DATA_HOME").ok();
            let old_meta = std::env::var("META_ROOT").ok();
            let old_home = std::env::var("HOME").ok();
            std::env::remove_var("ENVCTL_SEED_TOKEN_FILE");
            std::env::remove_var("XDG_DATA_HOME");
            std::env::set_var("META_ROOT", "/tmp/meta-envctl-test");
            std::env::set_var("HOME", "/tmp/real-home");
            assert_eq!(
                token_file(),
                std::path::PathBuf::from("/tmp/meta-envctl-test/.local/share/env-ctl/seed-token")
            );
            restore_var("ENVCTL_SEED_TOKEN_FILE", old_token_file);
            restore_var("XDG_DATA_HOME", old_xdg_data);
            restore_var("META_ROOT", old_meta);
            restore_var("HOME", old_home);
        }

        #[test]
        fn token_file_honors_explicit_overrides() {
            let _guard = env_lock();
            let old_token_file = std::env::var("ENVCTL_SEED_TOKEN_FILE").ok();
            let old_xdg_data = std::env::var("XDG_DATA_HOME").ok();
            std::env::set_var("ENVCTL_SEED_TOKEN_FILE", "/tmp/seed-token");
            assert_eq!(token_file(), std::path::PathBuf::from("/tmp/seed-token"));

            std::env::remove_var("ENVCTL_SEED_TOKEN_FILE");
            std::env::set_var("XDG_DATA_HOME", "/tmp/xdg-data");
            assert_eq!(
                token_file(),
                std::path::PathBuf::from("/tmp/xdg-data/env-ctl/seed-token")
            );
            restore_var("ENVCTL_SEED_TOKEN_FILE", old_token_file);
            restore_var("XDG_DATA_HOME", old_xdg_data);
        }

        #[test]
        fn parse_pubkey_hex_roundtrips_32_bytes() {
            // A real 64-hex Ed25519 device public key (Seed `/api/v1/identity`, 2026-06-13).
            let hex = "804781357690631a9f98e3ccd91b5e2df2cca8d2be8ac94ea525ceaf82b3470a";
            let b = parse_pubkey_hex(hex).expect("valid 64-hex parses");
            assert_eq!(b.len(), 32);
            assert_eq!(b[0], 0x80);
            assert_eq!(b[31], 0x0a);
            assert!(parse_pubkey_hex("dead").is_none(), "too short");
            assert!(parse_pubkey_hex(&"zz".repeat(32)).is_none(), "non-hex");
            assert!(
                parse_pubkey_hex(&"00".repeat(32)).is_some(),
                "64 hex = valid"
            );
            assert!(
                parse_pubkey_hex(&"00".repeat(64)).is_none(),
                "128 hex = sig length, not a pubkey"
            );
            assert!(parse_pubkey_hex(&"00".repeat(31)).is_none(), "62 hex");
        }

        #[test]
        fn host_port_splits_base_url() {
            assert_eq!(
                host_port("https://169.254.42.1:8443"),
                Some(("169.254.42.1".to_string(), 8443))
            );
            assert_eq!(
                host_port("https://seed.local:8443/api/v1"),
                Some(("seed.local".to_string(), 8443))
            );
            // No explicit port ⇒ HTTPS default.
            assert_eq!(
                host_port("https://seed.local"),
                Some(("seed.local".to_string(), 443))
            );
            assert_eq!(host_port("http://nope"), None, "https only");
        }

        #[test]
        fn parse_status_reads_code() {
            assert_eq!(parse_status("HTTP/1.1 200 OK\r\n\r\n{}"), Some(200));
            assert_eq!(parse_status("HTTP/1.1 401 Unauthorized\r\n"), Some(401));
            assert_eq!(parse_status("garbage"), None);
        }

        #[test]
        fn extract_field_scans_json_value() {
            // A 2xx custody/sign body — note the value is the (public) hex signature.
            let body = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"signature\": \"90017fcc0e\", \"client\":\"envctl-daemon\"}";
            assert_eq!(
                extract_field(body, "signature"),
                Some("90017fcc0e".to_string())
            );
            assert_eq!(
                extract_field(body, "client"),
                Some("envctl-daemon".to_string())
            );
            // Tolerates chunked framing: a chunk-size line between header and body.
            let chunked = "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n1f\r\n{\"token\":\"abc.def.ghi\"}\r\n0\r\n\r\n";
            assert_eq!(
                extract_field(chunked, "token"),
                Some("abc.def.ghi".to_string())
            );
            assert_eq!(extract_field(body, "missing"), None);
        }
    }
}

pub struct MintRequest {
    pub provider: crate::broker::Provider,
    pub repos: Vec<String>,
    /// TASK-0020: NUMERIC repository IDs. MUTUALLY EXCLUSIVE with `repos` (names): the GitHub
    /// `create installation access token` endpoint accepts EITHER `repositories` (names) OR
    /// `repository_ids` (ints), and sending both is a 422. When non-empty, the mint body emits
    /// `repository_ids` and `repos` MUST be empty (the `mint-github` consumer path sets only this).
    pub repo_ids: Vec<u64>,
    pub perms: Vec<String>,
    pub ttl_secs: i64,
}
pub struct ScopedToken {
    pub token: Zeroizing<Vec<u8>>,
    pub expires_at: i64,
}
#[derive(Debug, thiserror::Error)]
pub enum MintError {
    #[error("provider does not support native sub-tokens")]
    Unsupported,
    #[error("{0}")]
    Other(String),
}
/// Optional native scoped sub-token minting (GitHub fine-grained PAT / App token, OpenAI project
/// key). Defaults to `Unsupported` so the proxy-swap path is the universal fallback.
pub trait ProviderMint: Send + Sync {
    fn mint_scoped(&self, _p: &MintRequest) -> Result<ScopedToken, MintError> {
        Err(MintError::Unsupported)
    }
}
pub struct NoMint;
impl ProviderMint for NoMint {}

#[derive(Debug, thiserror::Error)]
pub enum UpstreamError {
    #[error("upstream io: {0}")]
    Io(String),
    #[error("upstream host not allowlisted: {0}")]
    HostNotAllowed(String),
}
/// The egress sender. The daemon impl MUST verify TLS against the FROZEN webpki-roots store —
/// never the local CA or the OS store (FS-S7) — and only after the engine has confirmed the
/// upstream host is in the provider's canonical allowlist (HF-11).
#[async_trait::async_trait]
pub trait Upstream: Send + Sync {
    async fn send(
        &self,
        req: crate::EgressReq,
        real_key: &Zeroizing<Vec<u8>>,
    ) -> Result<crate::EgressResp, UpstreamError>;
}
