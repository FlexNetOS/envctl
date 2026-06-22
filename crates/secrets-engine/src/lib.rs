//! env-ctl secrets engine: the single shared library. No printing, no UI, no clap.
//!
//! Both the daemon (`secretd`) and any future GUI drive the vault + credential broker through
//! the *identical* `Engine` API below; the CLI (`secretctl`) talks to the daemon over gRPC.
//! Mirrors `envctl_engine`: the engine never prints — it emits a structured `SecretEvent`
//! stream over an `std::sync::mpsc` channel. The engine core is synchronous; only the egress
//! swap path (`Engine::relay_swap` + the `Upstream` seam) is async.
//!
//! Phase 1b makes the vault functional: `init_vault` mints the DEK + enrolls keyslots,
//! `unlock`/`lock` drive the locked/unlocked state machine (the DEK is zeroized on `lock`), and
//! `secret_put`/`secret_get` seal/open per-record ciphertext through the `Store`. Every security
//! op appends a DURABLE, hash-chained audit row BEFORE returning (HF-14) and emits a `SecretEvent`.
//! A refused op is `Ok`-with-a-`GuardRefused`-event + a `Refused` audit row — NOT an `Err`
//! (error.rs discipline). The relay/run paths and destructive CA lifecycle verbs continue to land
//! incrementally behind fail-closed guards.
#![allow(dead_code)] // Some scaffold fields/bodies are placeholders until later phases.

pub mod broker; // Broker, RelayPolicy, Bearer, decide(), token verify, clamp_ttl, SwapOutcome
pub mod ca; // LocalCa (feature mitm-ca)
pub mod error; // EngineError (thiserror, setup-time only), VaultState
pub mod event; // SecretEvent, EventSink (std mpsc), Stream, AuditRecord
pub mod guard; // SecGuard, check_sec_guards, UnlockContext (fail-closed)
pub mod inject;
pub mod keyslot; // Keyslot, Kdf, Argon2Params, wrap/unwrap (LUKS-style dual KEK) + header MAC
#[cfg(feature = "provider-github")]
pub mod mint_github; // GitHubAppMint: RS256 App-JWT → installation token (ProviderMint seam)
pub mod paths; // Paths (XDG, env-ctl-namespaced)
pub mod seam; // Clock, UsbProbe, ProviderMint, Upstream + SystemClock/RealUsbProbe + fakes
pub mod vault; // Vault state machine + Store trait + crypto (seal/open) + canonical AAD + audit // ChildEnvPlan, ResolvedInjection, injection_template, run_wrapped

pub use broker::{
    clamp_ttl, AdmissionLimiter, Admit, Bearer, DenyReason, JtiReject, JtiReplayStore, Method,
    NonceReject, NonceStore, Provider, RelayDecision, RelayId, RelayKind, RelayPolicy, SwapMode,
    SwapOutcome, MAX_BEARER_TTL_SECS,
};
pub use error::{EngineError, VaultState};
pub use event::{AuditRecord, EventSink, SecretEvent, Stream};
pub use guard::{check_sec_guards, Destructiveness, SecGuard, UnlockContext};
pub use keyslot::{Argon2Params, Factor, Kdf, Keyslot};
#[cfg(feature = "provider-github")]
pub use mint_github::{
    build_app_jwt, build_revoke_request, revoke_installation_token, GitHubAppMint,
    GithubMintParams, HttpRequest, HttpResponse, HttpTransport, NoopHttpTransport, TransportError,
    MAX_JWT_TTL_SECS,
};
pub use seam::{Clock, ProviderMint, RealUsbProbe, SystemClock, Upstream, UsbProbe};

#[cfg(feature = "provider-github")]
use std::collections::HashMap;
#[cfg(feature = "provider-github")]
use std::sync::Mutex;
use std::sync::{Arc, RwLock};
use zeroize::Zeroizing;

use event::AuditOutcome;
use keyslot::{
    kek_from_passphrase, kek_from_usb, keyslot_aad, verify_header_mac, wrap_dek, Dek,
    ARGON2_M_KIB_FLOOR, ARGON2_T_COST_FLOOR,
};
use vault::aad::{record_aad, TableTag};
use vault::store::{BearerRow, CertRow, RelayPolicyRow, SecretRow};

use broker::{
    bearer_row_mac_message, broker_hmac_key, broker_row_mac_key, canonical_upstreams, decide,
    mac_bearer, mac_bearer_row, parse_bearer, verify_bearer, verify_bearer_row, CanonRequest,
    VerifiedBearer,
};

// Meta keys for the vault header (non-secret; persisted plaintext through the Store).
const META_HEADER_MAC: &str = "vault.header_mac";
const META_ISSUANCE_FLOOR_MS: &str = "vault.issuance_floor_ms";
const META_DEK_GENERATION: &str = "vault.dek_generation";
/// DEK-keyed anchor over the audit chain TAIL (`max_seq` + tail `row_hash`) AND the monotonic
/// high-water (`META_AUDIT_HIGH_WATER`), rewritten on every successful audit append while the vault
/// is unlocked. The chain itself is unkeyed (its hashes are public), so a store-level attacker could
/// drop trailing rows and re-link a perfectly clean shorter chain that `verify_chain` accepts. This
/// anchor binds the EXPECTED tail AND the highest anchored seq to the DEK; `verify_audit_anchor`
/// reconstructs the MAC against the row at `seq == high_water` (the tail AS OF the last advance — NOT
/// the current live tail, which may sit above it after rows were appended while LOCKED) and REJECTS a
/// live chain whose max-seq is below the high-water (truncation), so a truncated/rewritten chain —
/// including a stale-anchor replay — is caught (only an unlocked vault can advance it). The full
/// verification rule lives on `verify_audit_anchor_with`. Domain-separated; see `audit_head_mac`.
const META_AUDIT_HEAD: &str = "vault.audit_head";
/// The strictly-non-decreasing high-water of the anchored tail seq, persisted as an `i64` decimal
/// string through the same plaintext meta KV as `META_AUDIT_HEAD`. It is the rollback FENCE: a
/// verifier rejects any live chain whose current max-seq is BELOW it (the live chain is shorter than
/// the highest tail we ever anchored => truncation). It is ALSO folded into `audit_head_mac`, so a
/// store-level attacker cannot lower the plaintext counter without invalidating the MAC, nor raise
/// the MAC-bound counter without the DEK. The plaintext copy lets `verify` reject precisely and lets
/// `advance` enforce monotonicity cheaply; the MAC-bound copy is the unforgeable authority. (Honest
/// residual: a FULL consistent snapshot rollback that rewinds rows + `META_AUDIT_HEAD` +
/// `META_AUDIT_HIGH_WATER` in lock-step is NOT detectable in-store — see THREAT-MODEL A2.)
const META_AUDIT_HIGH_WATER: &str = "vault.audit_high_water";

/// Reserved secret name under which the local MITM CA's PKCS#8 private-key DER is sealed (feature
/// `mitm-ca`). Stored as a normal `broker_only` `SecretRow` so it inherits the record AAD binding +
/// the un-revealable HF-5 gate (`secret_get` refuses a `broker_only` reveal): the CA key can never
/// be read out through the operator surface, only reconstructed into the in-RAM issuer at unlock.
#[cfg(feature = "mitm-ca")]
const META_MITM_CA_KEY_NAME: &str = "__mitm_ca_key";
/// Meta key holding the PUBLIC local-CA certificate DER (hex), written by `ca_init`. Its presence is
/// the "CA initialized" signal the unlock tail keys off to rebuild the issuer.
#[cfg(feature = "mitm-ca")]
const META_MITM_CA_CERT_DER: &str = "mitm.ca_cert_der";
/// Meta key holding the local-CA cert `not_after` (RFC3339), for visibility / rotation tooling.
#[cfg(feature = "mitm-ca")]
const META_MITM_CA_NOT_AFTER: &str = "mitm.ca_not_after";

/// Vault meta-key suffixes for a native-mint provider's App credential (G2). The App PEM itself is
/// stored as a `broker_only` `SecretRow` under the relay's `secret_name` (un-revealable, opened only
/// via `open_real_key`); the App id + installation id are non-secret integers persisted as plaintext
/// meta keys, integrity-covered by the header MAC, under `"{secret_name}.app_id"` /
/// `"{secret_name}.installation_id"`.
///
/// A native-mint App credential read from the unlocked vault: `(app_key_pem, app_id, installation_id)`.
/// The PEM is `Zeroizing` (single-owner, wiped on drop); it flows ONLY into the daemon's
/// `GitHubAppMint` constructor — never an Event, audit row, or log.
#[cfg(feature = "provider-github")]
pub type AppCredential = (Zeroizing<Vec<u8>>, String, u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CertListItem {
    pub cn: String,
    pub is_ca: bool,
    pub not_after: String,
    pub revoked: bool,
    pub sans: Vec<String>,
    pub usage: String,
}

/// TASK-0020 flat-convention secret/meta names for the per-call `mint-github` path. The App PEM is
/// sealed as a broker-only `SecretRow` under this name (un-revealable; opened only against the live
/// DEK, never through `secret_get`); the App id is a non-secret integer string in the plaintext meta
/// KV (integrity-covered by the header MAC). Enrolled by TASK-0026 (`secretctl github-app enroll`),
/// which references these `pub const`s VERBATIM so the enroll-writer and the mint-reader can never
/// drift apart; if a credential is absent `mint_github_token` fails closed naming that remediation.
///
/// Exported (`pub`) so the thin `secretctl` enroll path seals/labels the App PEM under EXACTLY the
/// name `mint_github_token` reads — single source of truth, no literal-drift.
#[cfg(feature = "provider-github")]
pub const GITHUB_APP_KEY_NAME: &str = "github-app-private-key";
#[cfg(feature = "provider-github")]
pub const GITHUB_APP_ID_META: &str = "github-app-id";

/// TASK-0027 — the GitHub REST base + User-Agent the engine uses for the BEST-EFFORT native
/// early-revoke fired from `relay_revoke`. The explicit-token verb (`revoke_github_token`) threads
/// an `api_base` from the request (so it works against GHES); the relay tie-in has no request-level
/// base, so it targets the public default. A GHES relay's native early-revoke is therefore a
/// documented best-effort limitation — its policy+bearer revoke is authoritative regardless.
#[cfg(feature = "provider-github")]
const GITHUB_API_BASE_DEFAULT: &str = "https://api.github.com";
#[cfg(feature = "provider-github")]
const GITHUB_REVOKE_USER_AGENT: &str = "flexnetos-github-app";

fn app_id_meta_key(secret_name: &str) -> String {
    format!("{secret_name}.app_id")
}
fn installation_id_meta_key(secret_name: &str) -> String {
    format!("{secret_name}.installation_id")
}

/// BLAKE3 `derive_key` context for the audit-head anchor key (DEK-keyed, domain-separated from the
/// header MAC and every other BLAKE3 use in the crate).
const AUDIT_HEAD_KEY_INFO: &str = "env-ctl/v1/audit-head/key";
/// Domain-separation prefix for the audit-head anchor message.
const AUDIT_HEAD_DOMAIN: &[u8] = b"env-ctl/v1/audit-head";

/// Top-level engine handle: owns the vault, the broker, and an optional local CA, plus the
/// `Send + Sync` seams. Cheaply cloneable (`Arc` inside) so it can move into worker tasks.
#[derive(Clone)]
pub struct Engine {
    inner: Arc<EngineInner>,
}

struct EngineInner {
    paths: paths::Paths,
    vault: RwLock<vault::Vault>, // Locked | Unlocked { dek: Dek }
    broker: RwLock<broker::Broker>,
    ca: RwLock<Option<ca::LocalCa>>,
    store: Box<dyn vault::Store>, // persistence seam; default InMemStore (libSQL slots in later)
    // dyn-dispatched seams; the supertrait `: Send + Sync` keeps Engine Send+Sync.
    clock: Box<dyn Clock>,
    usb: Box<dyn UsbProbe>,
    /// The native sub-token minter, LATE-BOUND on vault unlock (DD-1, Option A). At startup and
    /// while Locked this is `NoMint` (every mint falls through to the proxy-swap path). The daemon
    /// reads the App-credential secret from the now-unlocked vault on unlock, builds a
    /// `GitHubAppMint`, and installs it via `install_provider`; `lock()` reinstalls `NoMint`,
    /// dropping the `Zeroizing` App PEM held inside the minter. `RwLock` so the per-request read path
    /// (`resolve_injection`) never blocks the (rare) install/clear writes. Mirrors the `mitm-ca`
    /// rebuild-on-unlock precedent (the sealed CA key likewise opens only against the live DEK).
    provider: RwLock<Box<dyn ProviderMint>>,
    upstream: Box<dyn Upstream>, // pins frozen webpki roots in the daemon impl (FS-S7)
    /// The HTTP egress seam for the PER-CALL GitHub App mint (`mint_github_token`, TASK-0020). The
    /// daemon installs `DaemonHttpTransport` (reqwest/rustls-on-ring, frozen webpki roots); the
    /// engine's own default is the fail-closed `NoopHttpTransport` (no egress ⇒ a stray mint refuses
    /// rather than silently succeeding). Distinct from the late-bound `provider` minter: this path
    /// builds a fresh `GitHubAppMint` per request from the request's `installation_id`, so it needs
    /// a transport it can hand to that minter even before any provider is installed.
    #[cfg(feature = "provider-github")]
    github_transport: Box<dyn mint_github::HttpTransport>,
    /// TASK-0027 — the last engine-minted NATIVE installation token per relay, keyed by `relay_id`.
    /// Populated in the `NativeSubtoken` success branch of [`Engine::resolve_injection`] (replacing
    /// any prior entry for that relay) so `relay_revoke(apply=true)` can fire a BEST-EFFORT
    /// `DELETE /installation/token` early-revoke against the live token bytes the engine still holds
    /// in-process. NEVER persisted; CLEARED on `lock()` / `clear_provider()` (fail-closed: a locked
    /// vault holds no live token). The values are `Zeroizing` (wiped on drop).
    #[cfg(feature = "provider-github")]
    native_token_cache: Mutex<HashMap<String, Zeroizing<Vec<u8>>>>,
    owner_uid: u32,
    /// Short-TTL cache for the **network** presence factor (Profile S, the Cognitum Seed):
    /// `(proven, resolved_at_wall_ms)`. The per-request egress path (`relay_swap`) must not do a
    /// ~1-2s SSH probe per request; a live challenge runs at most once per `PRESENCE_GATE_TTL_MS`.
    /// Only present under `seed-factor` — the fast on-box USB probe (Profile A) is never cached, so
    /// default builds keep the no-grace gate unchanged. See [`Engine::seed_presence_cached`].
    #[cfg(feature = "seed-factor")]
    presence_cache: std::sync::Mutex<Option<(bool, i64)>>,
}

/// Which unlock factor the operator is presenting.
pub enum Unlock {
    Usb,
    Passphrase(Zeroizing<String>),
}

#[derive(Debug)]
pub struct SecretMeta {
    pub name: String,
    pub provider: Provider,
    pub note: String,
    pub broker_only: bool,
}

/// One row of `secret_list`: NON-SECRET metadata for the latest version of a stored secret. Carries
/// ONLY the safe `SecretRow` fields plus the latest `version` + its `created_ts` — NEVER the `nonce`,
/// the `ct_tag` ciphertext, or any plaintext (no value crosses this boundary). `broker_only` is a
/// plain bool flag here; the reveal gate is unaffected.
#[derive(Debug)]
pub struct SecretListItem {
    pub name: String,
    pub provider: Provider,
    pub note: String,
    pub broker_only: bool,
    pub version: u32,
    pub created_ts: String,
}

/// A canonicalized egress request as seen by the broker (host is the *verified* inner Host).
pub struct EgressReq {
    pub method: Method,
    pub host: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub bytes_out: u64,
    pub peer_uid: Option<u32>,
    pub peer_pid: Option<u32>,
    /// The SNI the proxy actually OBSERVED on the inbound TLS handshake, when the request was
    /// terminated by a MITM ingress (PR-3b). `None` for non-MITM planes (no TLS is terminated at the
    /// relay, so there is no observed SNI). For `SwapMode::ProxyMitm`, `trusted_sni_for` returns this
    /// value so `decide` can enforce SNI==Host (anti-fronting) against a TLS-observed name rather than
    /// a sentinel. `None` under a MITM swap fails closed (`SniHostMismatch`).
    pub observed_sni: Option<String>,
    /// The verified REMOTE presentation context (Phase 8 / F2), set by the `secretd` remote relay
    /// edge AFTER it terminated TLS in-process, verified the RFC 9449 DPoP proof against the
    /// registered `jkt`, and bound the proof to the TLS channel (EKM). `None` for every LOCAL (UDS /
    /// loopback proxy) request — those carry no remote context, so `decide()` denies a remote bearer
    /// presented over a local path (`CrossKindPresentation`). When `Some`, `relay_swap_prepare`
    /// forwards it verbatim into the `CanonRequest`, so `decide()`'s clause 11a re-asserts the
    /// binding fail-closed (`RemoteNoDPoP` if `dpop_verified` is false). The engine NEVER sets this
    /// itself — it is the single additive seam the edge fills.
    pub remote: Option<broker::decide::RemotePeer>,
}

pub struct EgressResp {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub allowed: bool,
}

impl Engine {
    /// Open an engine backed by the real seams (`SystemClock`, `RealUsbProbe`, ...) and the
    /// default RAM-backed `InMemStore`. Equivalent to [`Engine::open_with_store`] with the in-memory
    /// store; `secretd` selects the durable libSQL store via `open_with_store` (OI-1 (a), Phase 1).
    pub fn open(paths: paths::Paths) -> anyhow::Result<Engine> {
        Self::open_with_store(paths, Box::new(vault::InMemStore::new()))
    }

    /// Open an engine backed by the real seams and an operator-selected `store`. The store is the
    /// ONLY seam that varies between the daemon's two backends (`InMemStore` vs the libSQL-backed
    /// store, OI-1 (a)); both implement the identical [`vault::Store`] trait, so nothing else changes.
    ///
    /// NOTE: the libSQL store drives its own current-thread runtime via `block_on`, so it must be
    /// CONSTRUCTED off the async reactor (e.g. before entering the tokio runtime, or on a
    /// `spawn_blocking` thread) — see `secretd`'s bring-up. `open_with_store` itself does no async.
    pub fn open_with_store(
        paths: paths::Paths,
        store: Box<dyn vault::Store>,
    ) -> anyhow::Result<Engine> {
        Self::with_seams(
            paths,
            store,
            Box::new(SystemClock),
            Box::new(RealUsbProbe),
            Box::new(seam::NoMint),
            Box::new(NullUpstream),
            // TASK-0020: the default (non-daemon) build has no GitHub egress ⇒ fail-closed no-op.
            #[cfg(feature = "provider-github")]
            Box::new(mint_github::NoopHttpTransport),
        )
    }

    /// Construct an engine with injected seams + store (the `envctl with_runner` analogue, for
    /// tests). `store` is the `DryRunRunner` analogue: pass `InMemStore` for an in-RAM vault.
    ///
    /// TASK-0020: under `provider-github` this also takes `github_transport`, the HTTP egress seam for
    /// the per-call `mint_github_token` path (the daemon passes `DaemonHttpTransport`; tests a fake;
    /// the non-daemon default is `NoopHttpTransport`).
    pub fn with_seams(
        paths: paths::Paths,
        store: Box<dyn vault::Store>,
        clock: Box<dyn Clock>,
        usb: Box<dyn UsbProbe>,
        provider: Box<dyn ProviderMint>,
        upstream: Box<dyn Upstream>,
        #[cfg(feature = "provider-github")] github_transport: Box<dyn mint_github::HttpTransport>,
    ) -> anyhow::Result<Engine> {
        let owner_uid = current_uid();
        Ok(Engine {
            inner: Arc::new(EngineInner {
                paths,
                vault: RwLock::new(vault::Vault::Locked),
                broker: RwLock::new(broker::Broker::default()),
                ca: RwLock::new(None),
                store,
                clock,
                usb,
                provider: RwLock::new(provider),
                upstream,
                #[cfg(feature = "provider-github")]
                github_transport,
                #[cfg(feature = "provider-github")]
                native_token_cache: Mutex::new(HashMap::new()),
                owner_uid,
                #[cfg(feature = "seed-factor")]
                presence_cache: std::sync::Mutex::new(None),
            }),
        })
    }

    /// Initialize a fresh vault: mint a random DEK (OsRng), derive the passphrase KEK
    /// (`kek_from_passphrase`, Argon2id) and — when `usb_keyfile` is `Some` — the USB KEK
    /// (`kek_from_usb`, HKDF), wrap the DEK into one `Keyslot` per factor (`wrap_dek`, AAD =
    /// `keyslot_aad`), persist each slot (`store.save_keyslot`), compute the vault header MAC over
    /// the slot set (`header_mac`, keyed by the DEK) and persist it + the issuance floor under meta
    /// keys (`"vault.header_mac"` hex, `"vault.issuance_floor_ms"`, `"vault.dek_generation" = 1`).
    /// Appends a durable `vault_init` audit row; emits no DEK. Refuses (`Err`) if a vault already
    /// exists (meta `"vault.header_mac"` present) or if `params` are below the Argon2 floors.
    /// Returns to `Locked` state.
    pub fn init_vault(
        &self,
        passphrase: Zeroizing<String>,
        usb_partition_uuid: Option<String>,
        usb_keyfile: Option<Zeroizing<Vec<u8>>>,
        params: keyslot::Argon2Params,
        sink: &EventSink,
    ) -> anyhow::Result<()> {
        let inner = &self.inner;

        // Refuse to clobber an existing vault.
        if inner.store.get_meta(META_HEADER_MAC)?.is_some() {
            anyhow::bail!("vault already initialized (refusing to overwrite)");
        }
        // Validate Argon2 params at-or-above the downgrade floors BEFORE deriving (FS-S13). This is
        // a setup-time refusal (Err), not a runtime guard-refusal.
        if params.m_kib < ARGON2_M_KIB_FLOOR {
            anyhow::bail!(
                "argon2 m_kib {} is below the {} KiB floor",
                params.m_kib,
                ARGON2_M_KIB_FLOOR
            );
        }
        if params.t_cost < ARGON2_T_COST_FLOOR {
            anyhow::bail!(
                "argon2 t_cost {} is below the {} iteration floor",
                params.t_cost,
                ARGON2_T_COST_FLOOR
            );
        }

        let dek_generation: i64 = 1;
        let issuance_floor_ms: i64 = inner.clock.now().timestamp_millis();

        // Mint a fresh random DEK from the OS CSPRNG.
        let dek = mint_dek();

        // Enroll the passphrase keyslot (id = 1). The salt is a fresh 16-byte CSPRNG value.
        let mut slots: Vec<Keyslot> = Vec::new();
        let pp_bytes = Zeroizing::new(passphrase.as_bytes().to_vec());
        let pp_salt = random_bytes(16);
        let mut pp_slot = Keyslot {
            id: 1,
            factor: Factor::Passphrase,
            label: "passphrase".to_string(),
            kdf: Kdf::Argon2id(params),
            salt: pp_salt.clone(),
            usb_partition_uuid: None,
            wrap_nonce: Vec::new(),
            wrapped_dek: Vec::new(),
            dek_generation,
            enabled: true,
        };
        let pp_aad = keyslot_aad(&pp_slot);
        let pp_kek = kek_from_passphrase(&pp_bytes, &pp_slot.salt, params);
        let (pp_nonce, pp_wrapped) = wrap_dek(pp_kek, &dek, &pp_aad);
        pp_slot.wrap_nonce = pp_nonce;
        pp_slot.wrapped_dek = pp_wrapped;
        slots.push(pp_slot);

        // Optional USB keyslot (id = 2). Requires both a UUID (slot identity, OI-5) and the keyfile
        // bytes (the IKM). The keyfile is HKDF IKM only — it is never persisted.
        if let Some(keyfile) = usb_keyfile.as_ref() {
            let uuid = usb_partition_uuid.clone().ok_or_else(|| {
                anyhow::anyhow!("usb keyfile provided without a usb_partition_uuid")
            })?;
            let usb_salt = random_bytes(32);
            let mut usb_slot = Keyslot {
                id: 2,
                factor: Factor::Usb,
                label: "usb".to_string(),
                kdf: Kdf::HkdfSha256,
                salt: usb_salt,
                usb_partition_uuid: Some(uuid),
                wrap_nonce: Vec::new(),
                wrapped_dek: Vec::new(),
                dek_generation,
                enabled: true,
            };
            let usb_aad = keyslot_aad(&usb_slot);
            let usb_kek = kek_from_usb(keyfile, &usb_slot.salt);
            let (usb_nonce, usb_wrapped) = wrap_dek(usb_kek, &dek, &usb_aad);
            usb_slot.wrap_nonce = usb_nonce;
            usb_slot.wrapped_dek = usb_wrapped;
            slots.push(usb_slot);
        }

        // Persist each slot, then the header MAC over the canonical slot set + issuance floor.
        for slot in &slots {
            inner.store.save_keyslot(slot)?;
        }
        let mac = keyslot::header_mac(&dek, &slots, issuance_floor_ms);
        inner.store.put_meta(META_HEADER_MAC, &hex_encode(&mac))?;
        inner
            .store
            .put_meta(META_ISSUANCE_FLOOR_MS, &issuance_floor_ms.to_string())?;
        inner
            .store
            .put_meta(META_DEK_GENERATION, &dek_generation.to_string())?;

        // Durable audit BEFORE returning (HF-14). vault_init carries the slot count, not any key.
        self.audit_ok(
            sink,
            "vault_init",
            None,
            serde_json::json!({ "slots": slots.len(), "dek_generation": dek_generation }),
        )?;
        // Anchor the genesis (`vault_init`) row with the local DEK while it is still alive (the
        // vault is Locked, so the in-`audit` anchor advance was a no-op). This DEK-keys the chain
        // tail from the very first row and seeds the monotonic high-water at the `vault_init` seq.
        let (seq, tail_hash) = match inner.store.last_audit()? {
            Some(r) => (r.seq, r.row_hash),
            None => (0i64, Vec::new()),
        };
        self.write_audit_anchor(&dek, seq, &tail_hash)?;

        // The DEK never leaves this function; it is dropped (zeroized) here. The vault stays Locked
        // until an explicit `unlock`.
        drop(dek);
        Ok(())
    }

    pub fn unlock(&self, u: Unlock, sink: &EventSink) -> anyhow::Result<VaultState> {
        let inner = &self.inner;
        // State guard: unlocking an already-unlocked vault is idempotent. We short-circuit BEFORE
        // any KEK derivation/probe so a wrong factor presented to a live vault can never (a) be
        // observed as an error while the vault silently stays unlocked, nor (b) grind a fresh
        // Argon2 derivation against a live DEK. The on-the-wire failure for a locked vault stays
        // the single generic UnlockFailed (no oracle).
        if inner.vault.read().expect("vault lock").is_unlocked() {
            return Ok(VaultState::Unlocked);
        }
        let slots = inner.store.load_keyslots()?;
        let stored_mac = self.load_header_mac()?;
        let issuance_floor_ms = self.load_issuance_floor()?;

        // Per-factor probe: try to unwrap the DEK from each enabled slot of the requested factor.
        // On the FIRST success, verify the header MAC over ALL slots, then commit Unlocked.
        let (want_factor, recovered): (Factor, Option<Dek>) = match &u {
            Unlock::Passphrase(pp) => {
                let pp_bytes = Zeroizing::new(pp.as_bytes().to_vec());
                let mut dek = None;
                for slot in slots
                    .iter()
                    .filter(|s| s.enabled && s.factor == Factor::Passphrase)
                {
                    // Validate the slot's KDF params against the floors AND the argon2 structural
                    // invariants BEFORE deriving. `kek_from_passphrase` calls `Params::new(..)
                    // .expect(..)`, which PANICS for `p_lanes == 0` (ThreadsTooFew) or `m_kib <
                    // 8 * p_lanes` (MemoryTooLittle). A corrupt/hostile keyslot header must surface
                    // as a clean skip -> generic UnlockFailed, never a panic, so we reject those
                    // here. (The flipped p_lanes is also bound into the slot AAD and would fail the
                    // tag, but the panic would happen before the tag check — so the filter must
                    // reject it first.)
                    let params = match slot.kdf {
                        Kdf::Argon2id(p)
                            if p.m_kib >= ARGON2_M_KIB_FLOOR
                                && p.t_cost >= ARGON2_T_COST_FLOOR
                                && p.p_lanes >= 1
                                && p.m_kib >= p.p_lanes.saturating_mul(8) =>
                        {
                            p
                        }
                        _ => continue, // wrong KDF, sub-floor, or structurally invalid: skip.
                    };
                    let kek = kek_from_passphrase(&pp_bytes, &slot.salt, params);
                    let aad = keyslot_aad(slot);
                    if let Some(d) =
                        keyslot::unwrap_dek(kek, &slot.wrap_nonce, &slot.wrapped_dek, &aad)
                    {
                        dek = Some(d);
                        break;
                    }
                }
                (Factor::Passphrase, dek)
            }
            Unlock::Usb => {
                let mut dek = None;
                for slot in slots
                    .iter()
                    .filter(|s| s.enabled && s.factor == Factor::Usb)
                {
                    // UUID match is NOT possession (CF-4): we must actually obtain the keyfile.
                    let Some(uuid) = slot.usb_partition_uuid.as_deref() else {
                        continue;
                    };
                    let Some(keyfile) = inner.usb.keyfile_for(uuid) else {
                        continue; // keyfile absent => possession unproven => skip.
                    };
                    let kek = kek_from_usb(&keyfile, &slot.salt);
                    let aad = keyslot_aad(slot);
                    if let Some(d) =
                        keyslot::unwrap_dek(kek, &slot.wrap_nonce, &slot.wrapped_dek, &aad)
                    {
                        dek = Some(d);
                        break;
                    }
                }
                (Factor::Usb, dek)
            }
        };

        let dek = match recovered {
            Some(d) => d,
            None => {
                // Single generic message (OI-17); never reveals which slot failed.
                self.audit_failed(sink, "vault_unlock", None, serde_json::json!({}))?;
                return Err(EngineError::UnlockFailed.into());
            }
        };

        // Header MAC: recompute over ALL slots and compare (FS-S13). A mismatch means the keyslot
        // set was tampered; zeroize the dek and refuse.
        if !verify_header_mac(&dek, &slots, issuance_floor_ms, &stored_mac) {
            drop(dek); // ZeroizeOnDrop wipes it.
            self.audit_failed(
                sink,
                "vault_unlock",
                None,
                serde_json::json!({ "reason": "header_mac" }),
            )?;
            return Err(EngineError::HeaderMacMismatch.into());
        }

        // dek_generation binding: the standalone `META_DEK_GENERATION` scalar is load-bearing for
        // the record AAD (`secret_put` seals against it) but is NOT covered by the header MAC
        // directly. Each keyslot's `dek_generation` IS bound by the MAC (via `keyslot_aad`), so now
        // that the slot set is authenticated we cross-check the meta scalar against the trusted
        // slots. A tampered/cleared meta generation is caught here as HeaderMacMismatch instead of
        // silently mis-binding new records after a future DEK rotation.
        let stored_generation = self.load_dek_generation()?;
        let slot_generation = slots.iter().map(|s| s.dek_generation).max().unwrap_or(1);
        if stored_generation != slot_generation {
            drop(dek);
            self.audit_failed(
                sink,
                "vault_unlock",
                None,
                serde_json::json!({ "reason": "dek_generation" }),
            )?;
            return Err(EngineError::HeaderMacMismatch.into());
        }

        // Audit-chain integrity: verify the unkeyed chain AND the DEK-keyed tail anchor against the
        // live chain (truncation/rewrite detection), using the just-recovered DEK before it is
        // committed into the vault. A broken/truncated chain refuses the unlock.
        if let Err(e) = self.verify_audit_anchor_with(&dek) {
            drop(dek);
            self.audit_failed(
                sink,
                "vault_unlock",
                None,
                serde_json::json!({ "reason": "audit_chain" }),
            )?;
            return Err(e);
        }

        // HF-14 (transactional ordering): append the durable `vault_unlocked` audit row BEFORE
        // committing `Unlocked` into RAM, so a failed audit append can never leave the vault
        // unlocked while `unlock` returns `Err`. If the audit fails the dek is dropped (zeroized)
        // and the vault stays Locked.
        self.audit_ok(
            sink,
            "vault_unlocked",
            None,
            serde_json::json!({ "factor": factor_str(want_factor) }),
        )?;
        {
            let mut v = inner.vault.write().expect("vault lock");
            *v = vault::Vault::Unlocked { dek };
        }
        // Now that the DEK is resident, advance the anchor to cover the just-appended
        // `vault_unlocked` row (it was appended while still Locked, so the in-`audit` advance was a
        // no-op). This leaves the freshly-unlocked vault with a current tail anchor.
        self.advance_audit_anchor_if_unlocked()?;
        // If a local MITM CA has been initialized, rebuild its in-RAM issuer now that the DEK is
        // resident (the sealed CA key opens against the live DEK). A failure here is non-fatal to the
        // unlock (the vault is already usable; CA issuance simply stays unavailable until re-init).
        #[cfg(feature = "mitm-ca")]
        self.rebuild_ca_if_initialized(sink)?;
        sink.emit(SecretEvent::VaultUnlocked {
            factor: want_factor,
        });
        Ok(VaultState::Unlocked)
    }

    /// Rebuild the in-RAM CA issuer from the sealed key + persisted public cert, iff CA meta is
    /// present. Idempotent: overwrites any existing issuer. Called from the `unlock` tail.
    #[cfg(feature = "mitm-ca")]
    fn rebuild_ca_if_initialized(&self, sink: &EventSink) -> anyhow::Result<()> {
        let inner = &self.inner;
        let Some(cert_hex) = inner.store.get_meta(META_MITM_CA_CERT_DER)? else {
            return Ok(()); // no CA initialized.
        };
        let ca_cert_der =
            hex_decode(&cert_hex).ok_or_else(|| anyhow::anyhow!("malformed ca_cert_der meta"))?;
        // Open the sealed CA key directly (NOT via secret_get, which refuses a broker_only reveal):
        // we reconstruct the AAD from the row identity and open against the live DEK.
        let key_der = match self.open_mitm_ca_key()? {
            Some(k) => k,
            None => {
                // Cert meta present but the sealed key is missing/unopenable: do not half-build a CA.
                self.audit_failed(
                    sink,
                    "ca_rebuild",
                    None,
                    serde_json::json!({ "reason": "ca_key_unavailable" }),
                )?;
                return Ok(());
            }
        };
        let ca = ca::LocalCa::from_material(key_der, &ca_cert_der)?;
        {
            let mut slot = inner.ca.write().expect("ca lock");
            *slot = Some(ca);
        }
        Ok(())
    }

    /// Open the sealed `__mitm_ca_key` SecretRow against the live DEK, returning its plaintext
    /// PKCS#8 DER (Zeroizing). `None` if the row is absent or fails authentication. Requires the
    /// vault to be Unlocked. This bypasses `secret_get`'s reveal gate (a `broker_only` secret is
    /// never revealed through the operator surface) BY DESIGN: the CA key is consumed internally to
    /// build the issuer and is never returned to a caller.
    #[cfg(feature = "mitm-ca")]
    fn open_mitm_ca_key(&self) -> anyhow::Result<Option<Zeroizing<Vec<u8>>>> {
        let inner = &self.inner;
        let v = inner.vault.read().expect("vault lock");
        let dek = match v.dek() {
            Some(d) => d,
            None => return Err(EngineError::Locked.into()),
        };
        let row = match inner.store.get_secret_latest(META_MITM_CA_KEY_NAME)? {
            Some(r) => r,
            None => return Ok(None),
        };
        let aad = record_aad(
            TableTag::SecretVersion,
            row.row_id,
            row.version as i64,
            row.dek_generation,
        );
        Ok(vault::crypto::open(dek, &aad, &row.nonce, &row.ct_tag))
    }

    /// Zeroizes the DEK + CA issuer in RAM (the true panic stop). Idempotent when already Locked.
    pub fn lock(&self, sink: &EventSink) -> anyhow::Result<()> {
        {
            let mut v = self.inner.vault.write().expect("vault lock");
            // Replacing Unlocked{dek} with Locked drops the old Dek => ZeroizeOnDrop wipes it.
            *v = vault::Vault::Locked;
        }
        {
            let mut ca = self.inner.ca.write().expect("ca lock");
            *ca = None; // drop the in-RAM CA issuer.
        }
        // Drop any installed native sub-token minter (DD-1): reinstall NoMint so the locked vault
        // holds no live App PEM. Defense-in-depth — the daemon's lock RPC handler also calls this,
        // but a direct `Engine::lock` (e.g. a test, or a future in-process caller) must clear it too.
        self.clear_provider();
        self.audit_ok(sink, "vault_locked", None, serde_json::json!({}))?;
        sink.emit(SecretEvent::VaultLocked);
        Ok(())
    }

    pub fn secret_put(
        &self,
        m: SecretMeta,
        body: Zeroizing<Vec<u8>>,
        sink: &EventSink,
    ) -> anyhow::Result<()> {
        let inner = &self.inner;
        // Requires Unlocked. We hold the WRITE lock for the whole reserve->seal->put so two
        // concurrent puts cannot interleave: this serializes the `version = max+1` read and the
        // store-side `row_id` reservation against the insert, closing the AAD/row_id divergence (a
        // racing pair could otherwise seal against the same id while the store stored distinct ids,
        // permanently de-authenticating the loser's ciphertext). The write lock also guarantees the
        // DEK can't be zeroized out from under us mid-op.
        let v = inner.vault.write().expect("vault lock");
        let dek = match v.dek() {
            Some(d) => d,
            None => return Err(EngineError::Locked.into()),
        };

        // dek_generation is load-bearing for the AAD binding (a wrong generation de-authenticates
        // the record). It is bound into the header MAC and verified at unlock, so a missing/garbled
        // value here is a setup-time failure, NOT a silent default.
        let dek_generation = self.load_dek_generation()?;
        let version = inner.store.max_secret_version(&m.name)? + 1;
        // The store is the sole authority for row_ids: reserve the id under the store's own lock,
        // seal the AAD against EXACTLY that id, then insert a row carrying it. `put_secret` persists
        // the id verbatim and rejects any id it never reserved, so the stored row_id can never
        // diverge from the id the ciphertext was sealed under (HF-2).
        let row_id = inner.store.reserve_secret_row_id()?;
        let aad = record_aad(
            TableTag::SecretVersion,
            row_id,
            version as i64,
            dek_generation,
        );
        let (nonce, ct_tag) = vault::crypto::seal(dek, &aad, &body);
        let created_ts = inner.clock.now().to_rfc3339();

        let row = SecretRow {
            row_id,
            name: m.name.clone(),
            version,
            provider: m.provider,
            note: m.note,
            broker_only: m.broker_only,
            dek_generation,
            nonce,
            ct_tag,
            created_ts,
        };
        let assigned = inner.store.put_secret(row)?;
        // Hard runtime check (NOT a debug_assert, which compiles out in release): a divergent id
        // must never be allowed to persist an un-openable record.
        if assigned != row_id {
            anyhow::bail!(
                "store assigned row_id {assigned} but the ciphertext was sealed against {row_id}"
            );
        }

        // The dek borrow + body drop happen at end of scope; release the write lock before audit so
        // we never hold a lock across a store write that itself takes a lock.
        drop(v);

        self.audit_ok(
            sink,
            "secret_written",
            Some(m.name.clone()),
            serde_json::json!({ "version": version }),
        )?;
        sink.emit(SecretEvent::SecretWritten {
            name: m.name,
            version,
        });
        Ok(())
    }

    /// `reveal` is apply-gated + audited + refused for `broker_only` secrets (HF-5/OI-2).
    pub fn secret_get(
        &self,
        name: &str,
        reveal: bool,
        apply: bool,
        sink: &EventSink,
    ) -> anyhow::Result<Zeroizing<Vec<u8>>> {
        let inner = &self.inner;
        let v = inner.vault.read().expect("vault lock");
        let dek = match v.dek() {
            Some(d) => d,
            None => return Err(EngineError::Locked.into()),
        };

        let row = match inner.store.get_secret_latest(name)? {
            Some(r) => r,
            None => {
                drop(v);
                self.audit_failed(
                    sink,
                    "secret_read",
                    Some(name.to_string()),
                    serde_json::json!({ "reason": "not_found" }),
                )?;
                anyhow::bail!("unknown secret '{name}'");
            }
        };

        // Reconstruct the SAME canonical AAD from the row's identity (HF-2) and open.
        let aad = record_aad(
            TableTag::SecretVersion,
            row.row_id,
            row.version as i64,
            row.dek_generation,
        );
        let plaintext = match vault::crypto::open(dek, &aad, &row.nonce, &row.ct_tag) {
            Some(pt) => pt,
            None => {
                // Tamper / corruption: the AEAD tag is the sole correctness oracle.
                drop(v);
                self.audit_failed(
                    sink,
                    "secret_read",
                    Some(name.to_string()),
                    serde_json::json!({ "reason": "tamper", "version": row.version }),
                )?;
                anyhow::bail!("secret '{name}' failed authentication (tampered or corrupt)");
            }
        };
        drop(v); // release the vault read lock; `plaintext` is now owned (Zeroizing).

        // REVEAL GATE (HF-5/OI-2): a broker-only secret never reveals; a reveal is apply-gated.
        if reveal {
            if row.broker_only {
                self.refuse(
                    sink,
                    "secret_read",
                    name,
                    "reveal refused: secret is broker-only",
                )?;
                anyhow::bail!("reveal refused: '{name}' is broker-only");
            }
            if !apply {
                self.refuse(
                    sink,
                    "secret_read",
                    name,
                    "reveal refused: apply not set (dry-run)",
                )?;
                anyhow::bail!("reveal refused: '{name}' requires --apply");
            }
            // Allowed reveal: audit + emit, then return the plaintext verbatim.
            let by_uid = inner.owner_uid;
            self.audit_ok(
                sink,
                "secret_read",
                Some(name.to_string()),
                serde_json::json!({ "version": row.version, "revealed": true }),
            )?;
            sink.emit(SecretEvent::SecretRead {
                name: name.to_string(),
                by_uid,
            });
            return Ok(plaintext);
        }

        // reveal = false: the plaintext is consumed internally (e.g. for injection) and NOT
        // returned to the caller verbatim. We audit the (non-revealing) read and return an empty
        // buffer; the apply gate does NOT apply when no reveal was requested.
        self.audit_ok(
            sink,
            "secret_read",
            Some(name.to_string()),
            serde_json::json!({ "version": row.version, "revealed": false }),
        )?;
        sink.emit(SecretEvent::SecretRead {
            name: name.to_string(),
            by_uid: inner.owner_uid,
        });
        // Drop the plaintext (Zeroizing wipes it) and hand back an empty buffer.
        drop(plaintext);
        Ok(Zeroizing::new(Vec::new()))
    }

    /// METADATA-ONLY list of stored secrets (the latest version of each), optionally filtered to a
    /// single `provider`. Gates on an unlocked vault (fail-closed, consistent with `secret_get`): a
    /// locked vault returns [`EngineError::Locked`]. NEVER returns a value, nonce, or ciphertext — only
    /// the non-secret [`SecretListItem`] fields. This is a read; it writes NO audit row and emits no
    /// event (listing metadata is not a security outcome, and an audit-per-list would flood the chain).
    pub fn secret_list(
        &self,
        provider: Option<Provider>,
        _sink: &EventSink,
    ) -> anyhow::Result<Vec<SecretListItem>> {
        let inner = &self.inner;
        // Fail-closed: require an unlocked vault even though no plaintext is read — listing is an
        // owner-only capability and we keep parity with `secret_get`'s gate. The DEK borrow is dropped
        // before the per-name store reads below.
        {
            let v = inner.vault.read().expect("vault lock");
            if v.dek().is_none() {
                return Err(EngineError::Locked.into());
            }
        }
        let mut out = Vec::new();
        for name in inner.store.list_secret_names()? {
            let Some(row) = inner.store.get_secret_latest(&name)? else {
                continue; // raced removal between list + get; skip.
            };
            if let Some(p) = provider {
                if row.provider != p {
                    continue;
                }
            }
            out.push(SecretListItem {
                name: row.name,
                provider: row.provider,
                note: row.note,
                broker_only: row.broker_only,
                version: row.version,
                created_ts: row.created_ts,
            });
        }
        Ok(out)
    }

    /// NON-SECRET metadata for the latest version of `name`, or `None` if unknown. Gates on an
    /// unlocked vault (fail-closed). Returns NO value/ciphertext. UN-AUDITED by design: this backs the
    /// `GetSecret.meta` field on a metadata read, and `secret_get` already writes the `secret_read`
    /// audit row — auditing here too would double-row every Get.
    pub fn secret_meta(&self, name: &str) -> anyhow::Result<Option<SecretMeta>> {
        let inner = &self.inner;
        {
            let v = inner.vault.read().expect("vault lock");
            if v.dek().is_none() {
                return Err(EngineError::Locked.into());
            }
        }
        Ok(inner.store.get_secret_latest(name)?.map(|row| SecretMeta {
            name: row.name,
            provider: row.provider,
            note: row.note,
            broker_only: row.broker_only,
        }))
    }

    /// DESTRUCTIVE removal of EVERY version of `name`, fail-closed + dry-run by default (template =
    /// [`Engine::relay_revoke`]). Refuses on a locked vault. `apply=false` (the default) is a DRY-RUN:
    /// it counts the versions that WOULD be removed (via `list_secret_versions`) and mutates NOTHING.
    /// `apply=true` removes the rows via [`vault::Store::delete_secret`], writes a durable audit row,
    /// and emits a `SecretWritten`-class event. Returns the count removed (dry-run: would-remove). NO
    /// secret bytes ever touch the audit row, the event, or the return value.
    pub fn secret_rm(&self, name: &str, apply: bool, sink: &EventSink) -> anyhow::Result<u32> {
        let inner = &self.inner;
        // Fail-closed: a locked vault cannot remove (consistent with the other mutating verbs). The
        // refusal writes a durable Refused row + GuardRefused event BEFORE returning.
        {
            let v = inner.vault.read().expect("vault lock");
            if v.dek().is_none() {
                drop(v);
                self.refuse(sink, "secret_removed", name, "vault is locked")?;
                return Err(EngineError::Locked.into());
            }
        }

        if !apply {
            // Dry-run: count the versions that WOULD be removed, mutate nothing.
            let would = inner.store.list_secret_versions(name)?.len() as u32;
            self.audit_ok(
                sink,
                "secret_removed",
                Some(name.to_string()),
                serde_json::json!({ "apply": false, "would_remove": would }),
            )?;
            return Ok(would);
        }

        // apply: remove every version. The store returns the count of rows removed.
        let removed = inner.store.delete_secret(name)?;
        self.audit_ok(
            sink,
            "secret_removed",
            Some(name.to_string()),
            serde_json::json!({ "apply": true, "removed": removed }),
        )?;
        sink.emit(SecretEvent::GuardRefused {
            subject: name.to_string(),
            reason: format!("secret '{name}' removed ({removed} versions)"),
        });
        Ok(removed)
    }

    /// Rotate `name` by appending a fresh sealed version carrying the SAME provider/note/broker_only
    /// as the current latest (carry-forward meta), fail-closed + dry-run by default. Refuses on a
    /// locked vault (no DEK to seal) or an unknown secret. `apply=false` (the default) is a DRY-RUN:
    /// it confirms the secret exists + reports the next version WITHOUT writing anything. `apply=true`
    /// appends the new version via [`Engine::secret_put`] (which monotonically picks `version=max+1`
    /// and writes its own `secret_written` audit row). `new_value` is held in `Zeroizing`. NO secret
    /// bytes touch the audit row, the event, or the return value.
    pub fn secret_rotate(
        &self,
        name: &str,
        new_value: Zeroizing<Vec<u8>>,
        apply: bool,
        sink: &EventSink,
    ) -> anyhow::Result<()> {
        let inner = &self.inner;
        // Fail-closed: rotation seals a new version, which requires the live DEK.
        {
            let v = inner.vault.read().expect("vault lock");
            if v.dek().is_none() {
                drop(v);
                self.refuse(sink, "secret_rotated", name, "vault is locked")?;
                return Err(EngineError::Locked.into());
            }
        }

        // Carry-forward meta from the current latest. An unknown secret is refused (rotation rotates
        // an EXISTING secret; use `secret_put` to create one). This read takes no value/plaintext.
        let Some(row) = inner.store.get_secret_latest(name)? else {
            self.refuse(sink, "secret_rotated", name, "unknown secret")?;
            anyhow::bail!("secret_rotate refused: unknown secret '{name}'");
        };
        let meta = SecretMeta {
            name: row.name.clone(),
            provider: row.provider,
            note: row.note.clone(),
            broker_only: row.broker_only,
        };

        if !apply {
            // Dry-run: confirm + report the next version, mutate nothing (no seal, no put).
            let next = row.version + 1;
            // The Zeroizing value is dropped here unused (wiped); a dry-run reads no plaintext.
            drop(new_value);
            self.audit_ok(
                sink,
                "secret_rotated",
                Some(name.to_string()),
                serde_json::json!({ "apply": false, "would_rotate_to_version": next }),
            )?;
            return Ok(());
        }

        // apply: append the new sealed version. `secret_put` reserves the row_id, seals against the
        // canonical AAD, writes its own `secret_written` audit row, and emits `SecretWritten`. We add
        // a `secret_rotated` audit row on top so the rotation intent is recorded distinctly.
        let new_version = row.version + 1;
        self.secret_put(meta, new_value, sink)?;
        self.audit_ok(
            sink,
            "secret_rotated",
            Some(name.to_string()),
            serde_json::json!({ "apply": true, "version": new_version }),
        )?;
        sink.emit(SecretEvent::RelayRotated {
            relay: name.to_string(),
            expires_at: String::new(),
        });
        Ok(())
    }

    /// List stored relay policies; filters out `revoked` policies unless `include_revoked`. Read path
    /// (no audit row, no event). Available regardless of unlock state — a policy row carries no secret
    /// (only non-secret metadata), like the other store reads. Returns the engine `RelayPolicy`s.
    pub fn relay_list(
        &self,
        include_revoked: bool,
        _sink: &EventSink,
    ) -> anyhow::Result<Vec<RelayPolicy>> {
        let rows = self.inner.store.list_relay_policies()?;
        Ok(rows
            .into_iter()
            .map(|r| r.policy)
            .filter(|p| include_revoked || !p.revoked)
            .collect())
    }

    /// Create (or upsert) a named relay policy ADDITIVELY via [`vault::Store::save_relay_policy`].
    /// Non-destructive — no unlock gate (a policy carries no secret); the store assigns the row id
    /// (`id: 0` ⇒ mint/reuse). Writes a durable `relay_created` audit row and returns the policy id.
    pub fn relay_create(&self, policy: RelayPolicy, sink: &EventSink) -> anyhow::Result<i64> {
        let inner = &self.inner;
        let relay_id = policy.relay_id.clone();
        let id = inner
            .store
            .save_relay_policy(RelayPolicyRow { id: 0, policy })?;
        self.audit_ok(
            sink,
            "relay_created",
            Some(relay_id),
            serde_json::json!({ "policy_id": id }),
        )?;
        Ok(id)
    }

    /// Read a window of the durable, hash-chained audit log: rows with `seq > since_seq`, up to
    /// `limit` (CLAMPED to <=1000 to bound the response). Already metadata-only — `AuditRecord`s
    /// carry no secret bytes (the engine never writes a value into an audit detail). Read path: no new
    /// audit row, no event.
    pub fn audit_query(
        &self,
        since_seq: i64,
        limit: usize,
        _sink: &EventSink,
    ) -> anyhow::Result<Vec<AuditRecord>> {
        const AUDIT_QUERY_MAX: usize = 1000;
        let limit = limit.min(AUDIT_QUERY_MAX);
        self.inner.store.query_audit(since_seq, limit)
    }

    /// USB-possession-gated, `<=24h`, peer-bound.
    ///
    /// Mints a fresh wire bearer (`evrelay_{token_id}_{secret}`) against `spec`, persisting ONLY its
    /// keyed MAC (`BearerRow.mac`); the raw bearer is returned to the caller and NEVER stored,
    /// audited, or emitted. USB possession is proven before any key material is touched (HF-14: the
    /// refusal writes its durable `Refused` row + `GuardRefused` event BEFORE returning). The TTL is
    /// clamped to `<=24h` through the single `clamp_ttl` choke point.
    /// Shared mint core for BOTH planes (F12/F15). `binding` selects LOCAL (uid/pid) vs REMOTE
    /// (client_id + DPoP jkt); everything else — the USB gate, TTL clamp, policy persist, wire MAC,
    /// plane-bound row MAC, and durable audit — is identical. Public callers: [`Engine::relay_mint`]
    /// (local) and [`Engine::relay_mint_remote`] (remote).
    fn mint_bearer_core(
        &self,
        spec: RelayPolicy,
        requested_ttl_secs: i64,
        binding: broker::BearerBinding,
        sink: &EventSink,
    ) -> anyhow::Result<Bearer> {
        let inner = &self.inner;
        // Destructure the plane binding into the row fields (mutually exclusive by construction:
        // a LOCAL bearer has uid/pid + no client_id/jkt; a REMOTE bearer has client_id/jkt + no
        // uid/pid). Both planes are bound into the plane-tagged row MAC below (F12).
        let (client_uid, client_pid, client_id, dpop_jkt): (
            Option<u32>,
            Option<u32>,
            Option<String>,
            Option<[u8; 32]>,
        ) = match binding {
            broker::BearerBinding::Local { peer_uid, peer_pid } => (peer_uid, peer_pid, None, None),
            broker::BearerBinding::Remote {
                client_id,
                dpop_jkt,
            } => (None, None, Some(client_id), Some(dpop_jkt)),
        };
        let now_ms = inner.clock.now().timestamp_millis();
        // Monotonic anchor captured at mint (OI-6): the rollback fence in `decide` measures elapsed
        // lifetime against THIS, not the rewindable wall clock. It is bound into the row MAC.
        let issued_boottime_ms = inner.clock.boottime_ms();

        // Hold the vault READ lock for the whole mint so the DEK cannot be zeroized out from under
        // us between the gate check and the MAC.
        let v = inner.vault.read().expect("vault lock");
        let dek = match v.dek() {
            Some(d) => d,
            None => return Err(EngineError::Locked.into()),
        };

        // PRINCIPAL GATE: a bearer MUST be bound to some principal — a LOCAL peer (uid and/or pid) OR
        // a REMOTE client_id. Refuse a both-null binding so the two `Store` backends agree (the libSQL
        // `relay_bearers` CHECK `(client_uid IS NOT NULL) OR (client_id IS NOT NULL)` rejects it; this
        // refuses it engine-side too, fail-closed, rather than letting InMemStore accept what libSQL
        // would reject). Unreachable through the peercred-gated daemon (the owner uid is always set),
        // but guards a direct `relay_mint(None, None)` misuse.
        if client_uid.is_none() && client_pid.is_none() && client_id.is_none() {
            drop(v);
            self.refuse(
                sink,
                "relay_mint",
                &spec.relay_id,
                "bearer binding has no principal (uid/pid/client_id all absent)",
            )?;
            anyhow::bail!(
                "relay_mint refused: binding has neither a local peer nor a remote client_id"
            );
        }

        // USB-GATE (HF-14): prove possession of the keyfile backing an enabled USB keyslot BEFORE
        // touching any key material. A UUID match alone is not possession (CF-4) — `keyfile_for`
        // must actually return the bytes. Absence is a REFUSAL (durable Refused row + GuardRefused
        // event), then a typed `UsbAbsent` Err; the real key is never derived.
        if !self.presence_proven()? {
            drop(v);
            self.refuse(
                sink,
                "relay_mint",
                &spec.relay_id,
                "usb possession not proven",
            )?;
            return Err(EngineError::UsbAbsent.into());
        }

        // TTL CLAMP (HF-15): the single choke point min()'s requested vs policy_ttl vs the 24h
        // ceiling (all in SECONDS, where `MAX_BEARER_TTL_SECS` lives) and refuses a dead/negative
        // TTL. `clamp_ttl(now_secs, ...)` returns the absolute expiry in the SAME unit, so we feed it
        // epoch-seconds and convert the result back to the millis the bearer row stores.
        let now_secs = now_ms.div_euclid(1000);
        let expires_at_secs = match clamp_ttl(now_secs, spec.policy_ttl_secs, requested_ttl_secs) {
            Some(e) => e,
            None => {
                drop(v);
                self.refuse(
                    sink,
                    "relay_mint",
                    &spec.relay_id,
                    "ttl clamp refused (non-positive)",
                )?;
                anyhow::bail!("relay_mint refused: clamped TTL is non-positive");
            }
        };
        let expires_at_ms = expires_at_secs.saturating_mul(1000);

        // Resolve / generate the relay_id. Ephemeral relays own a fresh generated id when blank.
        let mut spec = spec;
        if matches!(spec.kind, RelayKind::Ephemeral) && spec.relay_id.is_empty() {
            spec.relay_id = format!("eph_{}", hex_encode(&random_bytes(8)));
        }

        // Persist the policy (upsert by relay_id; the assigned id IS the bearer linkage key).
        let policy_id = inner.store.save_relay_policy(RelayPolicyRow {
            id: 0,
            policy: spec.clone(),
        })?;

        // MINT the raw bearer from the OS CSPRNG. token_id is a public, opaque index (lowercase
        // hex, no separator char); secret is the actual 32-byte authenticator (base64url-no-pad).
        let token_id = hex_encode(&random_bytes(16));
        let secret = b64url_nopad(&random_bytes(32));
        let raw = Zeroizing::new(format!("{}{}_{}", broker::BEARER_PREFIX, token_id, secret));

        // MAC the WHOLE wire string under the DEK-derived bearer key (Zeroizing, dropped at scope
        // end). We persist ONLY the MAC — the raw bearer never touches disk.
        let hmac_key = broker_hmac_key(dek);
        let mac = mac_bearer(&hmac_key, &raw);
        drop(hmac_key);

        // Authenticate the clear-text row metadata with a SEPARATE DEK-keyed MAC (CRITICAL fix). This
        // binds `revoked`/`expires_at_ms`/`issued_at_ms`/`issued_boottime_ms`/`policy_id`/peer ids, so
        // a store-level attacker cannot flip any of them to forge an Allow — the swap path re-verifies
        // this before `decide`, and a tamper fails closed (UnknownBearer).
        let row_mac_key = broker_row_mac_key(dek);
        let row_mac = mac_bearer_row(
            &row_mac_key,
            &bearer_row_mac_message(
                &token_id,
                policy_id,
                expires_at_ms,
                now_ms,
                issued_boottime_ms,
                client_uid,
                client_pid,
                client_id.as_deref(),
                dpop_jkt.as_ref(),
                false,
            ),
        );
        drop(row_mac_key);

        inner.store.save_bearer(BearerRow {
            token_id: token_id.clone(),
            policy_id,
            mac: mac.to_vec(),
            expires_at_ms,
            issued_at_ms: now_ms,
            issued_boottime_ms,
            client_uid,
            client_pid,
            client_id,
            dpop_jkt,
            revoked: false,
            row_mac: row_mac.to_vec(),
        })?;

        // Release the vault lock BEFORE the audit store write (never hold a lock across a store write
        // that takes its own lock).
        drop(v);

        let expires_at = ms_to_rfc3339(expires_at_ms);
        // Durable audit BEFORE return WITHOUT the bearer; only the public token_id appears.
        self.audit_ok(
            sink,
            "relay_minted",
            Some(spec.relay_id.clone()),
            serde_json::json!({
                "token_id": token_id,
                "kind": spec.kind,
                "expires_at_ms": expires_at_ms,
            }),
        )?;
        sink.emit(SecretEvent::RelayMinted {
            relay: spec.relay_id.clone(),
            kind: spec.kind,
            expires_at: expires_at.clone(),
        });

        Ok(Bearer {
            relay_id: spec.relay_id,
            token_id,
            raw,
            expires_at,
        })
    }

    /// Mint a LOCAL (uid/pid-bound) relay bearer over the control plane (HF-8). Public API unchanged;
    /// delegates to [`Engine::mint_bearer_core`] with a `Local` binding.
    pub fn relay_mint(
        &self,
        spec: RelayPolicy,
        requested_ttl_secs: i64,
        peer_uid: Option<u32>,
        peer_pid: Option<u32>,
        sink: &EventSink,
    ) -> anyhow::Result<Bearer> {
        self.mint_bearer_core(
            spec,
            requested_ttl_secs,
            broker::BearerBinding::Local { peer_uid, peer_pid },
            sink,
        )
    }

    /// Register (or re-register) a remote client for the Phase-8 relay edge (F15). USB-gated like a
    /// mint: only the operator in physical possession may enroll a remote principal. Stores the
    /// client's DPoP public-key thumbprint (`dpop_jkt`, RFC 7638) + the `hardware_bound` attestation
    /// — `false` means the binding is bearer-only (replay-BOUNDED by scope/TTL, not replay-PREVENTED;
    /// audit F20/OI-SM-5). Idempotent (upsert by `client_id`). Refuses an empty `client_id`.
    pub fn register_remote_client(
        &self,
        client_id: String,
        dpop_jkt: [u8; 32],
        hardware_bound: bool,
        sink: &EventSink,
    ) -> anyhow::Result<()> {
        if client_id.trim().is_empty() {
            anyhow::bail!("register_remote_client refused: empty client_id");
        }
        let inner = &self.inner;
        let now_ms = inner.clock.now().timestamp_millis();
        // USB possession (operator gate), same as mint — registering a remote principal is privileged.
        if !self.presence_proven()? {
            self.refuse(
                sink,
                "register_remote_client",
                &client_id,
                "usb possession not proven",
            )?;
            return Err(EngineError::UsbAbsent.into());
        }
        inner.store.save_remote_client(crate::vault::RemoteClient {
            client_id: client_id.clone(),
            dpop_jkt,
            enabled: true,
            hardware_bound,
            created_at_ms: now_ms,
            revoked_at_ms: None,
        })?;
        self.audit_ok(
            sink,
            "remote_client_registered",
            Some(client_id),
            serde_json::json!({ "hardware_bound": hardware_bound }),
        )?;
        Ok(())
    }

    /// Read-only lookup of a registered remote client by `client_id` (Phase 8 / F2). Additive,
    /// non-mutating accessor for the `secretd` remote relay edge: the edge consults it BEFORE
    /// `decide()` so an unknown or revoked client is refused at the edge (401) without ever reaching
    /// the swap/mint path — mirroring how `UnknownBearer` is raised before `decide()`. Returns
    /// `Ok(None)` for an unregistered `client_id`; `Ok(Some(c))` for a registered one (the caller
    /// inspects `c.enabled` / `c.revoked_at_ms` to map `RemoteClientUnknown` / `RemoteClientRevoked`).
    /// The `RemoteClient` row holds NO secret (the `jkt` is a public RFC-7638 thumbprint). A store
    /// error is surfaced as `Err` so the edge fails closed (treat as a refusal, never an accept).
    pub fn load_remote_client(
        &self,
        client_id: &str,
    ) -> anyhow::Result<Option<crate::vault::RemoteClient>> {
        self.inner.store.load_remote_client(client_id)
    }

    /// Mint a REMOTE (client_id + DPoP-jkt-bound) relay bearer (Phase 8, F15). The client MUST be a
    /// registered, enabled remote client whose registered DPoP thumbprint equals `dpop_jkt`
    /// (proof-of-possession is bound at mint; the edge re-verifies the live per-request proof). Like
    /// every mint it is USB-gated (push-mint). Refuses (no bearer, durable Refused row) on an
    /// unknown/disabled/revoked client or a jkt mismatch — default-deny.
    pub fn relay_mint_remote(
        &self,
        spec: RelayPolicy,
        requested_ttl_secs: i64,
        client_id: String,
        dpop_jkt: [u8; 32],
        sink: &EventSink,
    ) -> anyhow::Result<Bearer> {
        // Validate the registration BEFORE any key material (default-deny; no DEK needed for this).
        // The `dpop_jkt` is a PUBLIC RFC-7638 thumbprint (not a secret) and this path is USB-gated +
        // operator-only, so a plain `==` is intentional (no constant-time comparison needed — unlike
        // the secret wire/row MACs).
        let registered = self.inner.store.load_remote_client(&client_id)?;
        match registered {
            Some(c) if c.enabled && c.revoked_at_ms.is_none() && c.dpop_jkt == dpop_jkt => {}
            Some(_) => {
                self.refuse(
                    sink,
                    "relay_mint_remote",
                    &spec.relay_id,
                    "remote client disabled/revoked or jkt mismatch",
                )?;
                anyhow::bail!("relay_mint_remote refused: client not enabled, or DPoP jkt does not match registration");
            }
            None => {
                self.refuse(
                    sink,
                    "relay_mint_remote",
                    &spec.relay_id,
                    "unknown remote client",
                )?;
                anyhow::bail!("relay_mint_remote refused: client {client_id:?} is not registered");
            }
        }
        self.mint_bearer_core(
            spec,
            requested_ttl_secs,
            broker::BearerBinding::Remote {
                client_id,
                dpop_jkt,
            },
            sink,
        )
    }

    /// Fail-closed; returns the count of bearers/policies flipped (HF-16). When `apply`, the relay
    /// policy is marked `revoked` AND every live bearer hanging off it is revoked; a store error is
    /// an `Err` (the revoke must NOT silently no-op). When `!apply` (dry-run) the count that WOULD be
    /// revoked is returned without mutating. The durable audit row is written BEFORE returning.
    pub fn relay_revoke(
        &self,
        relay_id: &str,
        apply: bool,
        sink: &EventSink,
    ) -> anyhow::Result<u32> {
        let inner = &self.inner;

        if !apply {
            // Dry-run: count the live bearers that WOULD be revoked, mutate nothing.
            let would = inner
                .store
                .list_bearers_for_relay(relay_id)?
                .into_iter()
                .filter(|b| !b.revoked)
                .count() as u32;
            self.audit_ok(
                sink,
                "relay_revoked",
                Some(relay_id.to_string()),
                serde_json::json!({ "apply": false, "would_revoke": would }),
            )?;
            return Ok(would);
        }

        // apply: flip the policy revoked flag, then revoke every live bearer.
        if let Some(mut row) = inner.store.load_relay_policy(relay_id)? {
            row.policy.revoked = true;
            inner.store.save_relay_policy(row)?;
        }
        // Flip + re-MAC every live bearer in the ENGINE (DEK live) rather than via the store-side
        // `revoke_bearers_for_relay`, which would set `revoked` without recomputing the DEK-keyed row
        // MAC and so leave the rows failing their own authenticity check on the next swap. We flip the
        // authenticated `revoked` flag and reseal the row MAC over it, keeping the row valid AND
        // revoked. A locked vault cannot revoke (no DEK) — fail closed with an Err.
        let mut n = 0u32;
        for mut b in inner.store.list_bearers_for_relay(relay_id)? {
            if !b.revoked {
                b.revoked = true;
                self.reseal_bearer_row(&mut b)?;
                inner.store.save_bearer(b)?;
                n += 1;
            }
        }

        self.audit_ok(
            sink,
            "relay_revoked",
            Some(relay_id.to_string()),
            serde_json::json!({ "apply": true, "revoked": n }),
        )?;
        sink.emit(SecretEvent::RelayRevoked {
            relay: relay_id.to_string(),
            reason: "operator revoke".to_string(),
        });

        // TASK-0027 — BEST-EFFORT native early-revoke. If the engine still holds this relay's last
        // engine-minted native installation token, fire a `DELETE /installation/token` to kill it
        // immediately (instead of waiting out the ~1h expiry). This is the ONE relay-revoke plane
        // where the engine has live token bytes to revoke (a handed-off/rotated or
        // BaseUrlRepoint/MitM relay has nothing to auto-revoke; its policy+bearer revoke above is
        // authoritative). Failure is SWALLOWED — `relay_revoke` MUST still return its bearer count;
        // worst case the native token lives out its ≤1h TTL (today's behavior). The entry is removed
        // regardless (we hold no live token after revoking, success or not).
        #[cfg(feature = "provider-github")]
        {
            let cached = self
                .inner
                .native_token_cache
                .lock()
                .expect("native token cache")
                .remove(relay_id);
            if let Some(token) = cached {
                match mint_github::revoke_installation_token(
                    self.inner.github_transport.as_ref(),
                    GITHUB_API_BASE_DEFAULT,
                    GITHUB_REVOKE_USER_AGENT,
                    &token,
                ) {
                    Ok(()) => {
                        self.audit_ok(
                            sink,
                            "github_token_revoked",
                            Some(relay_id.to_string()),
                            serde_json::json!({ "outcome": "revoked" }),
                        )?;
                        sink.emit(SecretEvent::GithubTokenRevoked {
                            installation_id: None,
                            outcome: "revoked".to_string(),
                        });
                    }
                    Err(_) => {
                        // Swallow: the error text never carries the token, but we surface only a
                        // fixed outcome label (metadata-only) — relay revoke still succeeds.
                        self.audit_failed(
                            sink,
                            "github_token_revoked",
                            Some(relay_id.to_string()),
                            serde_json::json!({ "outcome": "best_effort_failed" }),
                        )?;
                        sink.emit(SecretEvent::GithubTokenRevoked {
                            installation_id: None,
                            outcome: "best_effort_failed".to_string(),
                        });
                    }
                }
            }
        }
        Ok(n)
    }

    /// Single-bearer revocation (OI-10). When `apply` and the bearer exists and is not already
    /// revoked, flip it and return 1; an already-revoked or unknown bearer returns 0 (fail-closed
    /// count). Dry-run returns the would-flip count (0/1) without mutating. The durable audit row is
    /// written BEFORE returning (HF-14).
    pub fn relay_revoke_bearer(
        &self,
        token_id: &str,
        apply: bool,
        sink: &EventSink,
    ) -> anyhow::Result<u32> {
        let inner = &self.inner;
        let row = inner.store.load_bearer(token_id)?;
        let would_flip = matches!(&row, Some(b) if !b.revoked);

        if !apply {
            self.audit_ok(
                sink,
                "relay_bearer_revoked",
                Some(token_id.to_string()),
                serde_json::json!({ "apply": false, "would_revoke": would_flip as u32 }),
            )?;
            return Ok(would_flip as u32);
        }

        let n = if would_flip {
            let mut b = row.expect("would_flip implies Some");
            b.revoked = true;
            // Re-authenticate the row under the live DEK so the flipped `revoked` is bound into the
            // row MAC (else the swap path's row-MAC verify would reject the legitimately-revoked
            // row as tampered). A locked vault cannot revoke — fail closed with an Err.
            self.reseal_bearer_row(&mut b)?;
            inner.store.save_bearer(b)?;
            1u32
        } else {
            0u32
        };
        self.audit_ok(
            sink,
            "relay_bearer_revoked",
            Some(token_id.to_string()),
            serde_json::json!({ "apply": true, "revoked": n }),
        )?;
        if n == 1 {
            sink.emit(SecretEvent::RelayRevoked {
                relay: token_id.to_string(),
                reason: "bearer revoke".to_string(),
            });
        }
        Ok(n)
    }
    /// Hot path: default-deny by construction — the real key is fetched only inside `Allowed`;
    /// any internal error becomes `InternalRefused` (a durable-audited 403), never `send()` (CF-9).
    ///
    /// The real secret is read from the unlocked vault ONLY inside the `Allow` branch and goes ONLY
    /// to `Upstream::send` — it is NEVER put in a `SecretEvent`, an audit row, an `Err`, or the
    /// return value. A `Deny` (or any internal error) never fetches the key and never reaches the
    /// upstream. All locks are dropped before the `.await` (the real key is moved out as an owned
    /// `Zeroizing<Vec<u8>>`).
    pub async fn relay_swap(&self, bearer: &str, req: &EgressReq, sink: &EventSink) -> SwapOutcome {
        // The whole pre-await body is fallible; funnel any `?` (lock poison / store error) into a
        // durable-audited `InternalRefused` so an internal error can NEVER fail-open into a send.
        match self.relay_swap_prepare(bearer, req, sink) {
            Err(e) => {
                let msg = e.to_string();
                let _ = self.audit_failed(
                    sink,
                    "relay_swapped",
                    None,
                    serde_json::json!({ "reason": "internal", "detail": msg }),
                );
                SwapOutcome::InternalRefused(msg)
            }
            // Deny: already audited + emitted inside prepare; the key was never fetched.
            Ok(Prepared::Deny(reason)) => SwapOutcome::Denied(reason),
            // Allow: prepare already extracted the real key (under the now-released lock) and the
            // matched relay metadata. ONLY NOW do we await the upstream.
            Ok(Prepared::Allow(allow)) => {
                let owned = EgressReq {
                    method: req.method,
                    host: req.host.clone(),
                    path: req.path.clone(),
                    headers: req.headers.clone(),
                    bytes_out: req.bytes_out,
                    peer_uid: req.peer_uid,
                    peer_pid: req.peer_pid,
                    observed_sni: req.observed_sni.clone(),
                    // The remote context is already consumed by `decide()` (clause 11a) inside
                    // `relay_swap_prepare`; carried here only to keep the owned request a faithful
                    // copy. `Upstream::send` never reads it.
                    remote: req.remote.clone(),
                };
                // HF-11 send-site fence (belt-and-suspenders): re-assert that the EXACT host about to
                // receive the real key is in the provider's frozen canonical allowlist, immediately
                // before send. `decide` already checked this, but re-checking here against the host
                // carried in `allow` (and `owned.host`) forecloses any divergence if the actual
                // upstream target ever becomes a function of an adapter/base-url rewrite. A miss
                // refuses WITHOUT sending — the key (still in `allow`) is dropped/zeroized.
                if !canonical_upstreams(allow.provider).iter().any(|h| {
                    h.eq_ignore_ascii_case(&owned.host) && h.eq_ignore_ascii_case(&allow.host)
                }) {
                    let _ = self.audit_failed(
                        sink,
                        "relay_swapped",
                        Some(allow.relay_id.clone()),
                        serde_json::json!({
                            "token_id": allow.token_id,
                            "reason": "upstream_fence",
                            "allowed": false,
                        }),
                    );
                    return SwapOutcome::InternalRefused("upstream host fence".to_string());
                }
                match self.inner.upstream.send(owned, &allow.real_key).await {
                    Ok(resp) => {
                        let _ = self.audit_ok(
                            sink,
                            "relay_swapped",
                            Some(allow.relay_id.clone()),
                            serde_json::json!({
                                "token_id": allow.token_id,
                                "host": req.host,
                                "method": method_str(req.method),
                                "allowed": true,
                            }),
                        );
                        sink.emit(SecretEvent::RelaySwapped {
                            relay: allow.relay_id,
                            host: req.host.clone(),
                            method: method_str(req.method).to_string(),
                            allowed: true,
                            token_id: allow.token_id,
                            client_uid: req.peer_uid.unwrap_or(self.inner.owner_uid),
                            client_label: String::new(),
                        });
                        SwapOutcome::Allowed(resp)
                    }
                    Err(ue) => {
                        // The real key went ONLY to send(); it is dropped (zeroized) with `allow`.
                        // CRITICAL containment: an upstream adapter is the one component that just
                        // received the REAL key. Its error STRING is untrusted — a buggy/hostile
                        // adapter could echo the auth header / key bytes into `ue.to_string()`. We
                        // therefore NEVER propagate the raw error text into the durable audit row or
                        // the caller-visible outcome; we map it to a fixed, key-free DISCRIMINANT
                        // label only, preserving the "never in an audit row / Err / return value"
                        // invariant.
                        let kind = upstream_error_kind(&ue);
                        let _ = self.audit_failed(
                            sink,
                            "relay_swapped",
                            Some(allow.relay_id.clone()),
                            serde_json::json!({
                                "token_id": allow.token_id,
                                "reason": "upstream",
                                "kind": kind,
                            }),
                        );
                        SwapOutcome::InternalRefused(format!("upstream send failed ({kind})"))
                    }
                }
            }
        }
    }

    /// The synchronous, fallible pre-await half of `relay_swap`: parse + verify the bearer, snapshot
    /// the clock/floor/USB gate, run the PURE `decide`, and — only on `Allow` — extract the real key
    /// while still holding the vault read lock, then release every lock before returning so the
    /// caller can `.await` the upstream with no guard held. A `Deny` is audited + emitted here (the
    /// key is never fetched); any `Err` is mapped to `InternalRefused` by the caller.
    fn relay_swap_prepare(
        &self,
        bearer: &str,
        req: &EgressReq,
        sink: &EventSink,
    ) -> anyhow::Result<Prepared> {
        // The swap path runs the shared authorization prelude WITH a counter bump (this request
        // consumes rate/budget) and WITH the real-key fetch on Allow. The observable behavior is
        // byte-for-byte what it was before the `authorize_relay` factoring: a Deny is audited + emitted
        // here exactly as `deny_swap` did inline; an Allow carries the extracted key.
        match self.authorize_relay(bearer, req, true)? {
            Authz::Deny {
                relay_id,
                token_id,
                reason,
            } => Ok(Prepared::Deny(self.deny_swap(
                sink,
                relay_id,
                token_id.as_deref(),
                reason,
            )?)),
            Authz::Allow {
                relay_id,
                token_id,
                provider,
                host,
                real_key,
            } => Ok(Prepared::Allow(AllowPrepared {
                relay_id,
                token_id,
                provider,
                host,
                // `bump == true` always fetches the key inside `authorize_relay` — its absence here
                // would be an internal contract break, so fail closed (InternalRefused) rather than
                // ever reaching `send` with no key.
                real_key: real_key
                    .ok_or_else(|| anyhow::anyhow!("authorize_relay(bump) returned no key"))?,
            })),
        }
    }

    /// The shared, sync authorization prelude for the relay egress + the streaming re-check
    /// (TASK-0032, FS-S5). Parses + constant-time-verifies the bearer (wire MAC AND DEK-keyed row
    /// MAC), loads the matched policy, snapshots the wall/monotonic clocks + issuance floor + USB
    /// presence gate, accumulates the usage tallies, and runs the PURE, default-deny [`decide`].
    ///
    /// `bump` selects the caller:
    ///   * `true` (the swap path) — `Broker::bump` records this request (consuming rate/budget) and,
    ///     on `Allow`, the real key is extracted while the vault read lock is still held (the DEK is
    ///     live), then every lock is released. `req.bytes_out` is counted.
    ///   * `false` (the streaming re-check) — `Broker::peek` recomputes the SAME tallies WITHOUT
    ///     mutating any counter and the key is NOT fetched (the re-check only answers "still
    ///     authorized?"). Callers pass `bytes_out == 0`, so the re-check enforces the live ceilings
    ///     against the accumulated totals without consuming any further budget.
    ///
    /// Every internal failure (parse handled inline as `UnknownBearer`; poisoned lock, locked vault,
    /// store error) returns `Err` so the caller fails closed — the swap maps it to `InternalRefused`,
    /// the re-check to `TearDown`. NEVER audits/emits here: the swap caller owns the `relay_swapped`
    /// audit (so the row shape is unchanged) and the re-check caller owns the metadata-only tear-down
    /// audit. No key, bearer, or secret byte is ever returned in a `Deny` or an `Err`.
    fn authorize_relay(&self, bearer: &str, req: &EgressReq, bump: bool) -> anyhow::Result<Authz> {
        let inner = &self.inner;

        // 1. Parse. A malformed / foreign bearer is UnknownBearer (no store hit, no crypto).
        let Some((token_id, raw)) = parse_bearer(bearer) else {
            return Ok(Authz::deny(None, None, DenyReason::UnknownBearer));
        };

        // 2. Snapshot under the vault READ lock. A poisoned lock fails closed (mapped to
        // InternalRefused / TearDown by the caller), never a panic that unwinds past the deny funnel.
        let v = inner
            .vault
            .read()
            .map_err(|_| anyhow::anyhow!("vault lock poisoned"))?;
        let dek = match v.dek() {
            Some(d) => d,
            // A locked vault returns Err (never a send) — fail-closed.
            None => anyhow::bail!("vault is locked"),
        };
        let now_ms = inner.clock.now().timestamp_millis();
        let boottime_now_ms = inner.clock.boottime_ms();
        let issuance_floor_ms = self.load_issuance_floor()?;

        // Load the bearer row by the public token_id (O(1)); a miss is UnknownBearer.
        let Some(row) = inner.store.load_bearer(token_id)? else {
            drop(v);
            return Ok(Authz::deny(None, Some(token_id), DenyReason::UnknownBearer));
        };

        // Constant-time MAC verify over the WHOLE wire string. A forged/wrong secret cannot be
        // distinguished from an absent bearer => UnknownBearer (no oracle).
        let hmac_key = broker_hmac_key(dek);
        if !verify_bearer(&hmac_key, raw, &row.mac) {
            drop(hmac_key);
            drop(v);
            return Ok(Authz::deny(None, Some(token_id), DenyReason::UnknownBearer));
        }
        drop(hmac_key);

        // Constant-time verify of the DEK-keyed ROW MAC over the clear-text metadata (CRITICAL fix).
        // This is what stops a store-level attacker from flipping `revoked`, raising `expires_at_ms`,
        // rewriting the peer binding, or repointing `policy_id` to forge an Allow: any such tamper
        // makes the recomputed MAC diverge from the stored one. A mismatch is indistinguishable from
        // an absent/forged bearer => UnknownBearer (no oracle), and the real key is never fetched.
        let row_mac_key = broker_row_mac_key(dek);
        let row_msg = bearer_row_mac_message(
            &row.token_id,
            row.policy_id,
            row.expires_at_ms,
            row.issued_at_ms,
            row.issued_boottime_ms,
            row.client_uid,
            row.client_pid,
            row.client_id.as_deref(),
            row.dpop_jkt.as_ref(),
            row.revoked,
        );
        if !verify_bearer_row(&row_mac_key, &row_msg, &row.row_mac) {
            drop(row_mac_key);
            drop(v);
            return Ok(Authz::deny(None, Some(token_id), DenyReason::UnknownBearer));
        }
        drop(row_mac_key);

        // Load the matched policy by the bearer's policy_id (the linkage key). A miss, or a policy
        // whose assigned id disagrees with the bearer, is treated as UnknownBearer (never a
        // successful Allow against a mismatched pair).
        let policy_row = match self.find_policy_by_id(row.policy_id)? {
            Some(pr) => pr,
            None => {
                drop(v);
                return Ok(Authz::deny(None, Some(token_id), DenyReason::UnknownBearer));
            }
        };
        let relay_id = policy_row.policy.relay_id.clone();
        let secret_name = policy_row.policy.secret_name.clone();

        // Presence gate snapshot (F14, SERVER-MODE §5.1): `presence_proven` resolves the egress gate
        // through one choke point — Profile A (on-box USB keyfile probe) by default, Profile S (the
        // Cognitum Seed: fresh-challenge Ed25519 verify) under `seed-factor` — behind a short-TTL
        // cache so this per-request path never probes the network factor live per request. `decide()`
        // treats Unproven EXACTLY like AbsentSince(now); absence fails closed (REQ-SEC-13), subject to
        // at most one TTL of presence staleness for the network factor. The streaming re-check reads
        // this FRESH each tick, so a USB pull (gate absent) tears the stream down within one interval.
        let gate_state = if self.presence_proven()? {
            crate::broker::GateState::Present
        } else {
            crate::broker::GateState::Unproven
        };
        let gate_absent_since_ms = crate::broker::gate_absent_since_ms(gate_state, now_ms);

        // 3. Accumulate the usage tallies. The swap path BUMPS (this request consumes rate/budget);
        // the streaming re-check PEEKS (read-only — it must never consume budget or it would starve a
        // legitimate long-lived stream). A poisoned broker lock fails closed (Err), never a panic.
        let (total_requests, total_bytes, rate_in_window) = if bump {
            let mut broker = inner
                .broker
                .write()
                .map_err(|_| anyhow::anyhow!("broker lock poisoned"))?;
            broker.bump(&row.token_id, now_ms, req.bytes_out)
        } else {
            let broker = inner
                .broker
                .read()
                .map_err(|_| anyhow::anyhow!("broker lock poisoned"))?;
            broker.peek(&row.token_id, now_ms)
        };

        let vb = VerifiedBearer {
            policy_id: row.policy_id,
            token_id: row.token_id.clone(),
            expires_at_ms: row.expires_at_ms,
            issued_at_ms: row.issued_at_ms,
            issued_boottime_ms: row.issued_boottime_ms,
            client_uid: row.client_uid,
            client_pid: row.client_pid,
            // The remote binding (F15) read from the authenticated row: `None` for a local bearer,
            // `Some(..)` for one minted via `relay_mint_remote`. The row MAC above (F12) authenticated
            // these, so `decide()`'s remote clause acts on trusted fields — a remote bearer presented
            // over this local UDS path (req.remote == None) is denied CrossKindPresentation.
            client_id: row.client_id.clone(),
            dpop_jkt: row.dpop_jkt,
            revoked: row.revoked,
        };
        let canon = CanonRequest {
            method: req.method,
            host: req.host.clone(),
            sni: trusted_sni_for(&policy_row.policy.swap, req),
            path: req.path.clone(),
            bytes_out: req.bytes_out,
            peer_uid: req.peer_uid,
            peer_pid: req.peer_pid,
            usage_requests: total_requests,
            usage_bytes: total_bytes,
            rate_in_window,
            // The verified remote presentation context, forwarded VERBATIM from the request. `None`
            // for a local (UDS / loopback proxy) request; `Some(RemotePeer{..})` only when the
            // Phase-8 remote relay edge already terminated TLS in-process, verified the RFC 9449 DPoP
            // proof against the registered `jkt`, and bound it to the TLS channel (EKM). `decide()`'s
            // clause 11a re-asserts the binding fail-closed (`RemoteNoDPoP` if `dpop_verified` is
            // false; `CrossKindPresentation` on a plane mismatch vs the bearer's kind). The streaming
            // re-check passes the SAME `RemotePeer` captured at open, so dpop_verified/jkt is
            // re-asserted each tick.
            remote: req.remote.clone(),
        };

        // 4. The PURE, default-deny decision (expiry fenced against BOTH the wall and monotonic
        // clocks).
        match decide(
            &policy_row.policy,
            &vb,
            &canon,
            now_ms,
            boottime_now_ms,
            gate_absent_since_ms,
            issuance_floor_ms,
        ) {
            RelayDecision::Deny { reason } => {
                drop(v);
                Ok(Authz::deny(Some(relay_id), Some(token_id), reason))
            }
            RelayDecision::Allow => {
                // On the SWAP path (`bump`), fetch the real secret NOW — internal open,
                // reveal=false-internal — producing an owned Zeroizing<Vec<u8>>. We are still holding
                // the vault read lock, so the DEK is live; we extract the key, then drop EVERY lock
                // before returning so the caller can await with no guard held. The streaming re-check
                // (`!bump`) NEVER fetches a key — it only needs the Allow/Deny verdict.
                let real_key = if bump {
                    Some(self.open_real_key(dek, &secret_name)?)
                } else {
                    None
                };
                drop(v);
                // Carry the provider + the canonical host so `relay_swap` can re-assert the HF-11
                // upstream-host fence IMMEDIATELY before send (belt-and-suspenders: decide() already
                // checked it, but the send-site gate forecloses any future divergence between the
                // host decide saw and the host actually sent).
                Ok(Authz::Allow {
                    relay_id,
                    token_id: row.token_id,
                    provider: policy_row.policy.provider,
                    host: req.host.clone(),
                    real_key,
                })
            }
        }
    }

    /// READ-ONLY, NON-MUTATING streaming re-check (TASK-0032, FS-S5): is a long-lived in-flight stream
    /// STILL authorized? Re-runs the SAME default-deny [`decide`] the swap ran at open — with FRESH
    /// reads of the wall/monotonic clocks, the bearer `revoked` flag, the policy, and the USB presence
    /// gate — but WITHOUT fetching the real key and WITHOUT bumping any counter (`bytes_out == 0`,
    /// `Broker::peek`). The `req` MUST carry the SAME `RemotePeer` captured at the stream's open so
    /// `decide`'s clause 11a re-asserts `dpop_verified` + the client_id/jkt binding each tick.
    ///
    /// Fail-closed by construction: ANY uncertainty tears the stream down. A `decide` Deny →
    /// `TearDown(reason)`; a locked vault, a poisoned lock, a store error, a vanished bearer row, an
    /// absent USB gate, or a MAC failure all surface as an `Err` here which the caller maps to
    /// `TearDown(InternalRefused)`. There is no panic/`unwrap`/`expect` on this path. The streaming
    /// driver in `secretd::edge::stream` calls this on a fixed interval; the engine performs NO I/O
    /// beyond the same store/clock reads the swap already does.
    ///
    /// `sink` is reserved for symmetry with the swap path (the engine never prints); this method emits
    /// nothing itself — the edge owns the metadata-only tear-down audit ({reason, client_id,
    /// token_id} only, never a key/body).
    pub fn relay_stream_authorized(
        &self,
        bearer: &str,
        req: &EgressReq,
        _sink: &EventSink,
    ) -> StreamAuthz {
        // The re-check is bytes_out-free: it must not advance the byte budget. Callers already pass a
        // zero-byte `req`, but we re-assert it here so a mis-built request can never consume budget.
        debug_assert_eq!(
            req.bytes_out, 0,
            "the streaming re-check must observe zero bytes (peek, not bump)"
        );
        match self.authorize_relay(bearer, req, false) {
            // Any internal error (locked vault, poisoned lock, store error, ...) tears the stream
            // DOWN — fail-closed. We map it to a fixed reason discriminant; no secret/detail escapes.
            Err(_) => StreamAuthz::TearDown(DenyReason::UnknownBearer),
            Ok(Authz::Deny { reason, .. }) => StreamAuthz::TearDown(reason),
            Ok(Authz::Allow { .. }) => StreamAuthz::Authorized,
        }
    }

    /// Open the real secret for an Allowed swap, reconstructing the canonical record AAD exactly as
    /// `secret_get` does. Returns the plaintext as an owned `Zeroizing<Vec<u8>>` — this is the ONLY
    /// place the real key materializes on the swap path, and it flows ONLY to `Upstream::send`.
    fn open_real_key(
        &self,
        dek: &keyslot::Dek,
        secret_name: &str,
    ) -> anyhow::Result<Zeroizing<Vec<u8>>> {
        let row = self
            .inner
            .store
            .get_secret_latest(secret_name)?
            .ok_or_else(|| anyhow::anyhow!("relay secret '{secret_name}' not found"))?;
        let aad = record_aad(
            TableTag::SecretVersion,
            row.row_id,
            row.version as i64,
            row.dek_generation,
        );
        vault::crypto::open(dek, &aad, &row.nonce, &row.ct_tag)
            .ok_or_else(|| anyhow::anyhow!("relay secret '{secret_name}' failed authentication"))
    }

    /// Install a native sub-token minter (DD-1, Option A). Called by the daemon AFTER the vault is
    /// unlocked and the App credential is unsealed — the engine never names the concrete minter type
    /// (the daemon owns the `GitHubAppMint` + transport), it only swaps the boxed `ProviderMint`.
    /// Idempotent: replaces any previously installed minter. Ungated (the `NoMint` default is always
    /// available); the daemon's call site is `#[cfg(feature = "provider-github")]`.
    pub fn install_provider(&self, provider: Box<dyn ProviderMint>) {
        *self.inner.provider.write().expect("provider lock") = provider;
    }

    /// Reinstall the `NoMint` default, dropping any installed minter (and the `Zeroizing` App PEM it
    /// holds). Called on `lock()` and by the daemon's lock RPC handler — fail-closed: a locked vault
    /// must hold no live native-mint key. Idempotent. Also DROPS the native-token cache (TASK-0027):
    /// a locked/cleared vault holds no live engine-minted installation token.
    pub fn clear_provider(&self) {
        *self.inner.provider.write().expect("provider lock") = Box::new(seam::NoMint);
        #[cfg(feature = "provider-github")]
        self.inner
            .native_token_cache
            .lock()
            .expect("native token cache")
            .clear();
    }

    /// Read a native-mint provider's App credential from the UNLOCKED vault: the App private-key PEM
    /// (opened via [`open_real_key`](Self::open_real_key) — the same un-revealable path the MITM CA
    /// key uses, so a `broker_only` App PEM never leaves through the operator surface) plus the
    /// `app_id` (string) and `installation_id` (u64) meta values. Returns `Ok(None)` when no App PEM
    /// is enrolled under `secret_name`. Fails (`Err(Locked)`) when the vault is locked — the App key
    /// can only materialize post-unlock (the structural fail-closed gate). The PEM is `Zeroizing`;
    /// it flows ONLY into the daemon's `GitHubAppMint` constructor, never into an Event or audit row.
    #[cfg(feature = "provider-github")]
    pub fn app_credential_pem(&self, secret_name: &str) -> anyhow::Result<Option<AppCredential>> {
        let inner = &self.inner;
        let v = inner.vault.read().expect("vault lock");
        let dek = match v.dek() {
            Some(d) => d,
            None => return Err(EngineError::Locked.into()),
        };
        // No App PEM enrolled under this name ⇒ no native minter (caller keeps NoMint, falls through).
        if inner.store.get_secret_latest(secret_name)?.is_none() {
            return Ok(None);
        }
        let pem = self.open_real_key(dek, secret_name)?;
        drop(v);
        let app_id = inner
            .store
            .get_meta(&app_id_meta_key(secret_name))?
            .ok_or_else(|| anyhow::anyhow!("missing app_id meta for '{secret_name}'"))?;
        let installation_id = inner
            .store
            .get_meta(&installation_id_meta_key(secret_name))?
            .ok_or_else(|| anyhow::anyhow!("missing installation_id meta for '{secret_name}'"))?
            .parse::<u64>()
            .map_err(|_| anyhow::anyhow!("malformed installation_id meta for '{secret_name}'"))?;
        Ok(Some((pem, app_id, installation_id)))
    }

    /// Persist the non-secret App `app_id` + `installation_id` meta keys for a native-mint relay
    /// (G2 enrollment seam; the App PEM itself is written via the normal `secret_put` as a
    /// `broker_only` secret). Requires the vault Unlocked (the meta KV is the same store the sealed
    /// secret lives in; we gate on the DEK so enrollment is an unlocked-only op, fail-closed).
    /// The values are non-secret and are integrity-covered by the header MAC.
    #[cfg(feature = "provider-github")]
    pub fn put_app_credential_meta(
        &self,
        secret_name: &str,
        app_id: &str,
        installation_id: u64,
    ) -> anyhow::Result<()> {
        let inner = &self.inner;
        {
            let v = inner.vault.read().expect("vault lock");
            if v.dek().is_none() {
                return Err(EngineError::Locked.into());
            }
        }
        inner
            .store
            .put_meta(&app_id_meta_key(secret_name), app_id)?;
        inner.store.put_meta(
            &installation_id_meta_key(secret_name),
            &installation_id.to_string(),
        )?;
        Ok(())
    }

    /// TASK-0020 — persist the non-secret flat `github-app-id` for the per-call `mint-github` path
    /// (the App PEM itself is sealed via the normal `secret_put` as a `broker_only` secret under
    /// `github-app-private-key`). This is the engine seam the TASK-0026 `secretctl github-app enroll`
    /// verb drives. Requires the vault Unlocked (the meta KV shares the store the sealed secret lives
    /// in; gating on the DEK keeps enrollment an unlocked-only op, fail-closed). The id is non-secret
    /// and integrity-covered by the header MAC.
    #[cfg(feature = "provider-github")]
    pub fn put_github_app_id(&self, app_id: &str) -> anyhow::Result<()> {
        {
            let v = self.inner.vault.read().expect("vault lock");
            if v.dek().is_none() {
                return Err(EngineError::Locked.into());
            }
        }
        self.inner.store.put_meta(GITHUB_APP_ID_META, app_id)?;
        Ok(())
    }

    /// TASK-0020 — mint a GitHub App **installation access token** for a REQUEST-SUPPLIED
    /// installation, behind the FROZEN `mint-github` consumer contract.
    ///
    /// Unlike the relay-native path (which uses the late-bound `provider` minter built at unlock),
    /// `installation_id` here comes from the request, so we build a FRESH [`GitHubAppMint`] per call
    /// from the vault-sealed App key (`github-app-private-key`, broker-only) + the non-secret App id
    /// (`github-app-id`), using the engine clock and the [`github_transport`](EngineInner) seam.
    ///
    /// Steps mirror [`open_mitm_ca_key`](Self::open_mitm_ca_key) (broker-only unseal against the live
    /// DEK, NOT `secret_get`):
    ///   1. require the vault Unlocked (locked ⇒ `Err(Locked)`, fail-closed: no key ⇒ no mint).
    ///   2. open `github-app-private-key` broker-only against the live DEK ⇒ `Zeroizing` PEM. Absent
    ///      ⇒ a fail-closed error naming the remediation (`secretctl github-app enroll`, TASK-0026).
    ///   3. read the `github-app-id` non-secret meta/secret.
    ///   4. build a per-call `GitHubAppMint::new(app_id, installation_id, pem, clock, &transport)` and
    ///      `mint_scoped` with `repo_ids` (numeric) + `perms`.
    ///   5. emit a METADATA-ONLY audit row + event (installation_id, repo/perm counts, expires_at —
    ///      NEVER the token or PEM).
    ///
    /// Returns the `ScopedToken` (its `token` is `Zeroizing`). The token materializes as a `String`
    /// only at the `MintGithubResp` / secretctl stdout boundary — never here, in a log, or in audit.
    #[cfg(feature = "provider-github")]
    pub fn mint_github_token(
        &self,
        params: mint_github::GithubMintParams,
        sink: &EventSink,
    ) -> anyhow::Result<seam::ScopedToken> {
        use crate::broker::Provider;

        // 1. Require Unlocked. Open the sealed App PEM directly against the live DEK while holding the
        // read lock (the un-revealable broker-only path; `secret_get` would refuse a broker_only
        // reveal). A locked vault has no DEK ⇒ Err(Locked) ⇒ fail-closed (no key ⇒ no mint).
        let pem = {
            let v = self.inner.vault.read().expect("vault lock");
            let dek = match v.dek() {
                Some(d) => d,
                None => return Err(EngineError::Locked.into()),
            };
            // 2. Absent App key ⇒ fail closed, naming the enrollment remediation (TASK-0026).
            if self
                .inner
                .store
                .get_secret_latest(GITHUB_APP_KEY_NAME)?
                .is_none()
            {
                anyhow::bail!(
                    "GitHub App key not enrolled — run `secretctl github-app enroll` (TASK-0026)"
                );
            }
            self.open_real_key(dek, GITHUB_APP_KEY_NAME)?
        };

        // 3. The App id is non-secret; it is enrolled alongside the key. Absent ⇒ same remediation.
        let app_id = self
            .inner
            .store
            .get_meta(GITHUB_APP_ID_META)?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "GitHub App id not enrolled — run `secretctl github-app enroll` (TASK-0026)"
                )
            })?;

        // 4. Build a PER-CALL minter from the request's installation_id + the sealed key, and mint.
        // `&dyn HttpTransport` lets the daemon's `DaemonHttpTransport` (or a test fake) drive egress
        // without the engine naming the concrete type. A non-GitHub clock is the engine's own clock.
        let mut minter = mint_github::GitHubAppMint::new(
            app_id,
            params.installation_id,
            pem,
            self.inner.clock.as_ref(),
            self.inner.github_transport.as_ref(),
        );
        if let Some(base) = &params.api_base {
            minter = minter.with_api_base(base.clone());
        }
        let req = seam::MintRequest {
            provider: Provider::Github,
            repos: Vec::new(), // mint-github scopes ONLY by numeric repository_ids (mutually excl.)
            repo_ids: params.repository_ids.clone(),
            perms: params.permissions.clone(),
            ttl_secs: params.ttl_secs,
        };
        let scoped = minter
            .mint_scoped(&req)
            .map_err(|e| anyhow::anyhow!("github mint failed: {e}"))?;

        // Defensive: a non-positive epoch is never a valid GitHub expiry. Fail closed rather than
        // emit a bogus expires_at the consumer would treat as already-expired / garbage.
        if scoped.expires_at <= 0 {
            anyhow::bail!("github mint returned a non-positive expires_at");
        }
        // An empty token is a broker denial in disguise — never surface it as success (fail-closed).
        if scoped.token.is_empty() {
            anyhow::bail!("github mint returned an empty token");
        }

        // 5. METADATA-ONLY audit + event: installation_id, repo/perm counts, expires_at — NEVER the
        // token or PEM. `expires_at` is GitHub's authoritative epoch, surfaced honestly.
        let expires_at_rfc3339 = chrono::DateTime::from_timestamp(scoped.expires_at, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_default();
        self.audit_ok(
            sink,
            "github_token_minted",
            Some(format!("installation:{}", params.installation_id)),
            serde_json::json!({
                "installation_id": params.installation_id,
                "repository_id_count": params.repository_ids.len(),
                "permission_count": params.permissions.len(),
                "expires_at": expires_at_rfc3339,
            }),
        )?;
        sink.emit(SecretEvent::RelayMinted {
            relay: format!("github-installation:{}", params.installation_id),
            kind: RelayKind::Ephemeral,
            expires_at: expires_at_rfc3339,
        });
        Ok(scoped)
    }

    /// TASK-0027 — early-revoke a GitHub installation access token via `DELETE /installation/token`.
    /// The token is supplied by the HOLDER (the explicit kill-switch verb): GitHub authenticates the
    /// revoke with the TOKEN ITSELF as the bearer (not the App-JWT), so no App credential is needed —
    /// only an UNLOCKED vault (`EngineError::Locked` if locked, matching mint's auth floor).
    ///
    /// `apply=false` ⇒ DRY-RUN: a `{apply:false}` audit row + `GithubTokenRevoked{outcome:"dry_run"}`
    /// event, NO egress, returns `Ok(false)`. `apply=true` ⇒ drive
    /// [`revoke_installation_token`](mint_github::revoke_installation_token) over the engine's
    /// `github_transport` seam: on GitHub's **204** ⇒ `GithubTokenRevoked{outcome:"revoked"}` + ok
    /// audit + `Ok(true)`; on transport error / non-204 ⇒ propagate `Err` (NEVER a false success).
    ///
    /// The token (`Zeroizing`) lives ONLY in the revoke request's auth header; it NEVER enters the
    /// audit detail, the event, or the returned `Err`. `api_base` threads the GHES base the same way
    /// `mint_github_token` does (env reads stay in the daemon — the engine is env-free).
    #[cfg(feature = "provider-github")]
    pub fn revoke_github_token(
        &self,
        token: Zeroizing<Vec<u8>>,
        apply: bool,
        api_base: Option<String>,
        sink: &EventSink,
    ) -> anyhow::Result<bool> {
        // Auth floor: require the vault Unlocked (mirrors mint). A locked vault ⇒ Err(Locked).
        {
            let v = self.inner.vault.read().expect("vault lock");
            if v.dek().is_none() {
                return Err(EngineError::Locked.into());
            }
        }

        if !apply {
            // Dry-run: preview only, NO egress. Metadata-only audit + event.
            self.audit_ok(
                sink,
                "github_token_revoked",
                None,
                serde_json::json!({ "apply": false }),
            )?;
            sink.emit(SecretEvent::GithubTokenRevoked {
                installation_id: None,
                outcome: "dry_run".to_string(),
            });
            return Ok(false);
        }

        let base = api_base
            .as_deref()
            .unwrap_or(GITHUB_API_BASE_DEFAULT)
            .to_string();
        // Drive the DELETE; 204 ⇒ Ok, transport/non-204 ⇒ Err (no false success). The error text
        // never contains the token (built without it), so it is safe to propagate.
        mint_github::revoke_installation_token(
            self.inner.github_transport.as_ref(),
            &base,
            GITHUB_REVOKE_USER_AGENT,
            &token,
        )
        .map_err(|e| anyhow::anyhow!("github revoke failed: {e}"))?;

        // Metadata-only audit + event — NEVER the token.
        self.audit_ok(
            sink,
            "github_token_revoked",
            None,
            serde_json::json!({ "apply": true, "outcome": "revoked" }),
        )?;
        sink.emit(SecretEvent::GithubTokenRevoked {
            installation_id: None,
            outcome: "revoked".to_string(),
        });
        Ok(true)
    }

    /// Build the child-env injection for a freshly-minted bearer (G2). This is the SINGLE place the
    /// native-subtoken decision lives — the front-ends (CLI/GUI/daemon) call THIS, never
    /// [`inject::injection_template`] directly, so the mint/inject logic can't diverge.
    ///
    /// - `BaseUrlRepoint` / `HttpsProxyMitm`: pure shaping via `injection_template` (the bearer is the
    ///   relay bearer; the real key stays in the daemon's upstream swap). `Ok(Some(_))`.
    /// - `NativeSubtoken`: call the installed [`ProviderMint`] (`mint_scoped`):
    ///     - `Ok(scoped)` ⇒ inject the **minted** token (NOT the relay bearer) into the provider's key
    ///       var(s). A durable `relay_native_minted` audit row + `RelayMinted` event carry **only**
    ///       `relay` + `expires_at` (token_id-equivalent) — never the minted token.
    ///     - `Err(MintError::Unsupported)` ⇒ fall back to the proxy-swap shape (`HttpsProxyMitm` env
    ///       built from `proxy_url`/`ca_pem_path`), so a vault that can't mint natively still works.
    ///     - `Err(MintError::Other(_))` ⇒ REFUSE: a durable `Refused` row + `GuardRefused` event,
    ///       `Ok(None)` (no token emitted). Fail-closed (transport/HTTP/allowlist error).
    ///
    /// `expires_at` is GitHub's authoritative value, surfaced honestly (the ~1h installation-token
    /// TTL is fixed by GitHub and never clamped here). `relay` names the relay for audit only.
    #[allow(clippy::too_many_arguments)]
    pub fn resolve_injection(
        &self,
        provider: Provider,
        relay: &str,
        bearer: &str,
        proxy_url: &str,
        ca_pem_path: &str,
        mode: inject::DataPlaneMode,
        repos: Vec<String>,
        perms: Vec<String>,
        native_ttl_secs: i64,
        sink: &EventSink,
    ) -> anyhow::Result<Option<inject::ResolvedInjection>> {
        use inject::DataPlaneMode;
        // Non-native planes: pure shaping (no mint). The bearer is the relay bearer.
        if !matches!(mode, DataPlaneMode::NativeSubtoken) {
            return Ok(Some(inject::injection_template(
                provider,
                bearer,
                proxy_url,
                ca_pem_path,
                mode,
            )));
        }

        // Native plane. Fail-closed allowlist: only a provider whose canonical upstream set carries
        // a GitHub mint host may attempt a native mint here (Github). Any other provider has no
        // native minter ⇒ treat as Unsupported (fall through to the proxy-swap shape).
        let mint_allowlisted = matches!(provider, Provider::Github)
            && canonical_upstreams(provider).contains(&"api.github.com");

        let mint_result = if mint_allowlisted {
            let req = seam::MintRequest {
                provider,
                repos,
                repo_ids: Vec::new(), // relay-native path scopes by repo NAME, not id (TASK-0020)
                perms,
                ttl_secs: native_ttl_secs,
            };
            self.inner
                .provider
                .read()
                .expect("provider lock")
                .mint_scoped(&req)
        } else {
            Err(seam::MintError::Unsupported)
        };

        match mint_result {
            Ok(scoped) => {
                // TASK-0027 — retain the engine-minted NATIVE token bytes (in-memory, Zeroizing,
                // never persisted) keyed by relay so a later `relay_revoke(apply=true)` can fire a
                // best-effort `DELETE /installation/token`. Replaces any prior entry for this relay;
                // cleared on lock()/clear_provider(). The token NEVER leaves the cache except as the
                // revoke request's auth-header bearer.
                #[cfg(feature = "provider-github")]
                {
                    self.inner
                        .native_token_cache
                        .lock()
                        .expect("native token cache")
                        .insert(relay.to_string(), scoped.token.clone());
                }
                // Inject the MINTED token (never the relay bearer) into the provider's key var(s).
                // `expires_at` is GitHub's authoritative value (RFC3339 from epoch secs), surfaced
                // honestly for the audit/event metadata — the minted token itself NEVER appears.
                let token = String::from_utf8_lossy(&scoped.token).into_owned();
                let injection = inject::injection_template(
                    provider,
                    &token,
                    proxy_url,
                    ca_pem_path,
                    DataPlaneMode::NativeSubtoken,
                );
                let expires_at = chrono::DateTime::from_timestamp(scoped.expires_at, 0)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_default();
                // Metadata-only audit + event: relay + expires_at, NEVER the minted token.
                self.audit_ok(
                    sink,
                    "relay_native_minted",
                    Some(relay.to_string()),
                    serde_json::json!({ "expires_at": expires_at }),
                )?;
                sink.emit(SecretEvent::RelayMinted {
                    relay: relay.to_string(),
                    kind: RelayKind::Ephemeral,
                    expires_at,
                });
                Ok(Some(injection))
            }
            Err(seam::MintError::Unsupported) => {
                // No native minter (locked vault ⇒ NoMint, or non-native provider): fall back to the
                // proxy-swap shape so the relay bearer still repoints the child through the daemon.
                Ok(Some(inject::injection_template(
                    provider,
                    bearer,
                    proxy_url,
                    ca_pem_path,
                    DataPlaneMode::HttpsProxyMitm,
                )))
            }
            Err(seam::MintError::Other(_)) => {
                // Transport / HTTP / allowlist error: REFUSE. Durable Refused row + GuardRefused
                // event, NO token emitted (fail-closed). The error text (which never contains the
                // token) is intentionally NOT surfaced into the audit detail — a fixed reason only.
                self.refuse(sink, "relay_native_minted", relay, "native_mint_failed")?;
                Ok(None)
            }
        }
    }

    /// Find a relay policy row by its assigned id (the bearer linkage key). Linear scan of the
    /// policy set; the store has no id index in 1b/Phase 4 InMem.
    fn find_policy_by_id(&self, policy_id: i64) -> anyhow::Result<Option<RelayPolicyRow>> {
        Ok(self
            .inner
            .store
            .list_relay_policies()?
            .into_iter()
            .find(|r| r.id == policy_id))
    }

    /// Recompute the DEK-keyed row MAC over a bearer row's CURRENT (security-critical) fields and
    /// write it back into `row.row_mac`. Called on every legitimate row mutation (mint reseals
    /// inline; revoke reseals here) so the persisted row always carries a MAC that matches its
    /// clear-text state. Requires the vault unlocked (`Err(Locked)` otherwise — a locked vault can
    /// neither mint nor revoke, fail-closed).
    fn reseal_bearer_row(&self, row: &mut BearerRow) -> anyhow::Result<()> {
        let v = self
            .inner
            .vault
            .read()
            .map_err(|_| anyhow::anyhow!("vault lock poisoned"))?;
        let dek = match v.dek() {
            Some(d) => d,
            None => return Err(EngineError::Locked.into()),
        };
        let row_mac_key = broker_row_mac_key(dek);
        row.row_mac = mac_bearer_row(
            &row_mac_key,
            &bearer_row_mac_message(
                &row.token_id,
                row.policy_id,
                row.expires_at_ms,
                row.issued_at_ms,
                row.issued_boottime_ms,
                row.client_uid,
                row.client_pid,
                row.client_id.as_deref(),
                row.dpop_jkt.as_ref(),
                row.revoked,
            ),
        )
        .to_vec();
        drop(row_mac_key);
        Ok(())
    }

    /// The egress/relay presence gate, as the relay paths consume it.
    ///
    /// **Profile A** (default build, or `seed-factor` with no pinned `ENVCTL_SEED_PUBKEY`): the fast
    /// on-box USB keyfile probe ([`Self::usb_possession_proven`]) — **uncached, immediate, no grace**
    /// (unchanged from before; default builds are byte-identical).
    ///
    /// **Profile S** (`seed-factor` + `ENVCTL_SEED_PUBKEY`): a fresh random-challenge Ed25519 verify
    /// against the Cognitum Seed, behind a short-TTL cache ([`Self::seed_presence_cached`]) — only
    /// the *network* factor is cached, because the per-request egress path can't afford a ~1-2s SSH
    /// probe per request.
    fn presence_proven(&self) -> anyhow::Result<bool> {
        #[cfg(feature = "seed-factor")]
        {
            if std::env::var_os("ENVCTL_SEED_PUBKEY").is_some() {
                return self.seed_presence_cached();
            }
        }
        self.usb_possession_proven()
    }

    /// Profile S (Cognitum Seed) presence behind a short-TTL cache. A live random-challenge verify
    /// runs at most once per `PRESENCE_GATE_TTL_MS`; within the window the last result is reused (≤
    /// one TTL of presence staleness — the owner-approved grace for a network factor, the only
    /// deviation from the no-grace rule). Vacuously `true` when no USB keyslot is enrolled (a
    /// passphrase-only vault is not USB-gated). Fails closed on a poisoned cache lock.
    #[cfg(feature = "seed-factor")]
    fn seed_presence_cached(&self) -> anyhow::Result<bool> {
        const PRESENCE_GATE_TTL_MS: i64 = 5_000;
        let now_ms = self.inner.clock.now().timestamp_millis();
        {
            let cache = self
                .inner
                .presence_cache
                .lock()
                .map_err(|_| anyhow::anyhow!("presence cache lock poisoned"))?;
            if let Some((proven, at)) = *cache {
                if now_ms.saturating_sub(at) < PRESENCE_GATE_TTL_MS {
                    return Ok(proven);
                }
            }
        }
        let slots = self.inner.store.load_keyslots()?;
        let proven = if !slots.iter().any(|s| s.enabled && s.factor == Factor::Usb) {
            true // vacuous: no USB keyslot enrolled
        } else {
            use crate::broker::{GateState, PresenceGate, SeedPresenceGate};
            matches!(SeedPresenceGate::from_env().resolve(), GateState::Present)
        };
        let mut cache = self
            .inner
            .presence_cache
            .lock()
            .map_err(|_| anyhow::anyhow!("presence cache lock poisoned"))?;
        *cache = Some((proven, now_ms));
        Ok(proven)
    }

    /// Whether USB possession is currently PROVEN: some enabled USB keyslot's keyfile is obtainable
    /// (a UUID match alone is not possession, CF-4). When the vault has NO USB keyslot enrolled, the
    /// gate is vacuously satisfied (a passphrase-only vault is not USB-gated).
    fn usb_possession_proven(&self) -> anyhow::Result<bool> {
        let slots = self.inner.store.load_keyslots()?;
        let usb_slots: Vec<_> = slots
            .iter()
            .filter(|s| s.enabled && s.factor == Factor::Usb)
            .collect();
        if usb_slots.is_empty() {
            return Ok(true);
        }
        for s in usb_slots {
            if let Some(uuid) = s.usb_partition_uuid.as_deref() {
                if self.inner.usb.keyfile_for(uuid).is_some() {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// Audit + emit a denied swap (the real key is NEVER fetched on this branch) and return the
    /// reason so the caller can wrap it in `SwapOutcome::Denied`.
    fn deny_swap(
        &self,
        sink: &EventSink,
        relay_id: Option<String>,
        token_id: Option<&str>,
        reason: DenyReason,
    ) -> anyhow::Result<DenyReason> {
        let reason_str = serde_json::to_value(reason)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_else(|| format!("{reason:?}"));
        self.audit(
            sink,
            "relay_swapped",
            relay_id.clone(),
            serde_json::json!({
                "token_id": token_id,
                "reason": reason_str,
                "allowed": false,
            }),
            AuditOutcome::Refused,
        )?;
        sink.emit(SecretEvent::RelaySwapped {
            relay: relay_id.unwrap_or_default(),
            host: String::new(),
            method: String::new(),
            allowed: false,
            token_id: token_id.unwrap_or("").to_string(),
            client_uid: self.inner.owner_uid,
            client_label: String::new(),
        });
        Ok(reason)
    }
    /// Initialize the local MITM CA (feature `mitm-ca`). Requires `Unlocked` (mirrors `secret_put`'s
    /// DEK guard). Generates a self-signed CA; when `apply`, seals the CA private-key DER into the
    /// vault as a `broker_only` `SecretRow` under `__mitm_ca_key` (un-revealable, HF-5), `put_meta`s
    /// the PUBLIC cert DER + `not_after`, and builds the in-RAM issuer. When `!apply`, a dry-run that
    /// previews keygen and persists NOTHING. Idempotent (`Ok` no-op if a CA is already present).
    /// Audited (`ca_initialized`) + emits `SecretEvent::CaIssued`. The private key is never logged.
    #[cfg(feature = "mitm-ca")]
    pub fn ca_init(&self, apply: bool, sink: &EventSink) -> anyhow::Result<()> {
        let inner = &self.inner;

        // DEK guard (mirror secret_put): CA keygen seals against the live DEK, so an unlocked vault
        // is required. Refusing on a locked vault is a setup-time error, not a guard refusal.
        {
            let v = inner.vault.read().expect("vault lock");
            if v.dek().is_none() {
                return Err(EngineError::Locked.into());
            }
        }

        // Idempotent: an already-initialized CA (cert meta present) is an Ok no-op.
        if inner.store.get_meta(META_MITM_CA_CERT_DER)?.is_some() {
            return Ok(());
        }

        let now_unix = inner.clock.now().timestamp();
        let generated = ca::LocalCa::generate(now_unix)?;

        if !apply {
            // Dry-run: previews that keygen succeeds; persists/seals NOTHING and builds no issuer.
            self.audit(
                sink,
                "ca_initialized",
                None,
                serde_json::json!({ "applied": false, "not_after": generated.not_after_rfc3339 }),
                AuditOutcome::Refused,
            )?;
            sink.emit(SecretEvent::GuardRefused {
                subject: ca::CA_COMMON_NAME.to_string(),
                reason: "ca_init dry-run: pass --apply to persist".to_string(),
            });
            return Ok(());
        }

        // Seal the CA private key as a broker_only SecretRow (un-revealable via secret_get). The
        // plaintext key DER is consumed here and dropped (Zeroizing) at end of scope.
        self.secret_put(
            SecretMeta {
                name: META_MITM_CA_KEY_NAME.to_string(),
                provider: Provider::Generic,
                note: "local MITM CA private key (sealed; never revealable)".to_string(),
                broker_only: true,
            },
            generated.key_der.clone(),
            sink,
        )?;

        // Persist the PUBLIC cert DER (hex) + not_after as non-secret meta.
        inner
            .store
            .put_meta(META_MITM_CA_CERT_DER, &hex_encode(&generated.cert_der))?;
        inner
            .store
            .put_meta(META_MITM_CA_NOT_AFTER, &generated.not_after_rfc3339)?;

        // Build the in-RAM issuer.
        let ca = ca::LocalCa::from_material(generated.key_der.clone(), &generated.cert_der)?;
        {
            let mut slot = inner.ca.write().expect("ca lock");
            *slot = Some(ca);
        }

        self.audit_ok(
            sink,
            "ca_initialized",
            None,
            serde_json::json!({ "applied": true, "not_after": generated.not_after_rfc3339 }),
        )?;
        sink.emit(SecretEvent::CaIssued {
            serial: String::new(),
            cn: ca::CA_COMMON_NAME.to_string(),
            not_after: generated.not_after_rfc3339,
        });
        Ok(())
    }

    /// Write the PUBLIC CA certificate PEM to a `0600` file under the runtime dir and return its
    /// path. Feeds the child-trust bundle (`inject::injection_template`'s `ca_pem_path`) for
    /// `HttpsProxyMitm` mode. Refuses (`Err`) when no CA is initialized. Public material only.
    #[cfg(feature = "mitm-ca")]
    pub fn ca_pem_path(&self) -> anyhow::Result<std::path::PathBuf> {
        use std::io::Write;
        let inner = &self.inner;
        let pem = {
            let ca = inner.ca.read().expect("ca lock");
            match ca.as_ref() {
                Some(c) => c.ca_cert_pem(),
                None => return Err(EngineError::NoCa.into()),
            }
        };
        let dir = inner.paths.runtime.clone();
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("mitm-ca.pem");
        // Create 0600 (owner read/write only) BEFORE writing any bytes, so the public bundle is
        // never world-readable even momentarily. The CA cert is public, but tight perms are the
        // house style for runtime artifacts.
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let mut f = opts.open(&path)?;
        f.write_all(pem.as_bytes())?;
        f.flush()?;
        // Re-assert mode in case the file pre-existed with looser perms (O_CREAT mode applies only
        // on creation).
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(path)
    }

    /// Mint a MITM leaf for `host` IFF the vault is Unlocked, the CA issuer is present, AND `host` is
    /// covered by an enabled, non-revoked `RelayPolicy.host_allow` of an active relay (the
    /// relay-coverage gate, CF-5). This is the ONLY path that mints MITM leaves; PR-3b's proxy calls
    /// it. Default-deny: a locked vault, an absent issuer, or an uncovered host yields a refusal and
    /// NO leaf (`Err`), never a cert.
    #[cfg(feature = "mitm-ca")]
    pub fn issue_leaf_for_covered_host(
        &self,
        host: &str,
        sink: &EventSink,
    ) -> anyhow::Result<(
        Vec<rustls_pki_types::CertificateDer<'static>>,
        rustls_pki_types::PrivateKeyDer<'static>,
    )> {
        let inner = &self.inner;

        // Unlocked guard.
        {
            let v = inner.vault.read().expect("vault lock");
            if v.dek().is_none() {
                self.refuse(sink, "leaf_minted", host, "vault locked")?;
                return Err(EngineError::Locked.into());
            }
        }

        // Relay-coverage gate: host must match an enabled, non-revoked policy's host_allow.
        let covered = inner.store.list_relay_policies()?.into_iter().any(|row| {
            let p = &row.policy;
            p.enabled && !p.revoked && p.host_allow.iter().any(|h| h.eq_ignore_ascii_case(host))
        });
        if !covered {
            self.refuse(
                sink,
                "leaf_minted",
                host,
                "host not covered by any active relay",
            )?;
            anyhow::bail!("leaf refused: host '{host}' is not covered by an active relay");
        }

        // Issuer must be resident.
        let now_unix = inner.clock.now().timestamp();
        let result = {
            let ca = inner.ca.read().expect("ca lock");
            match ca.as_ref() {
                Some(c) => c.issue_leaf(host, now_unix)?,
                None => {
                    self.refuse(sink, "leaf_minted", host, "CA not initialized")?;
                    return Err(EngineError::NoCa.into());
                }
            }
        };

        self.audit_ok(
            sink,
            "leaf_minted",
            Some(host.to_string()),
            serde_json::json!({ "mitm": true }),
        )?;
        sink.emit(SecretEvent::LeafMinted {
            sni: host.to_string(),
            relay: String::new(),
            not_after: String::new(),
        });
        Ok(result)
    }

    /// Operator-issued NON-MITM leaves only; REFUSES `usage = "mitm_leaf"` (CF-5). A MITM leaf may be
    /// minted ONLY through the relay-gated [`Engine::issue_leaf_for_covered_host`], never through
    /// this operator surface. The refusal is a durable `Refused` row + `GuardRefused` event mapped to
    /// an `Err`. Valid control-plane usages mint a public cert, persist its public DER/metadata, and
    /// emit/audit the issuance; the private key is never returned by this RPC shape.
    #[cfg(feature = "mitm-ca")]
    pub fn ca_issue(
        &self,
        cn: &str,
        sans: &[String],
        ttl_days: u64,
        usage: &str,
        sink: &EventSink,
    ) -> anyhow::Result<String> {
        // CF-5: the operator path NEVER mints a MITM leaf. Refuse before touching any key material.
        let usage_key = usage.trim().replace('-', "_").to_ascii_lowercase();
        if usage_key == "mitm_leaf" {
            self.refuse(
                sink,
                "ca_issued",
                usage,
                "operator ca issue refused: mitm_leaf may only be minted via the relay-gated path (CF-5)",
            )?;
            anyhow::bail!("ca issue refused: usage 'mitm_leaf' is not operator-issuable (CF-5)");
        }

        let operator_usage = match usage_key.as_str() {
            "control_plane_server" | "control_server" | "server" => {
                ca::OperatorLeafUsage::ControlPlaneServer
            }
            "control_plane_client" | "control_client" | "client" => {
                ca::OperatorLeafUsage::ControlPlaneClient
            }
            _ => {
                self.refuse(
                    sink,
                    "ca_issued",
                    usage,
                    "operator ca issue refused: usage must be control_plane_server or control_plane_client",
                )?;
                anyhow::bail!(
                    "ca issue refused: usage must be control_plane_server or control_plane_client"
                );
            }
        };

        let inner = &self.inner;
        {
            let v = inner.vault.read().expect("vault lock");
            if v.dek().is_none() {
                return Err(EngineError::Locked.into());
            }
        }

        let now_unix = inner.clock.now().timestamp();
        let leaf = {
            let ca = inner.ca.read().expect("ca lock");
            match ca.as_ref() {
                Some(ca) => ca.issue_operator_leaf(cn, sans, ttl_days, operator_usage, now_unix)?,
                None => {
                    self.refuse(sink, "ca_issued", cn, "CA not initialized")?;
                    return Err(EngineError::NoCa.into());
                }
            }
        };

        let (_rem, cert) = x509_parser::parse_x509_certificate(&leaf.cert_der)
            .map_err(|e| anyhow::anyhow!("ca issue: parse issued cert: {e}"))?;
        let serial = cert.tbs_certificate.raw_serial_as_string();
        inner.store.save_cert(CertRow {
            serial: serial.clone(),
            cn: cn.to_string(),
            not_after: leaf.not_after_rfc3339.clone(),
            der: leaf.cert_der,
        })?;

        self.audit_ok(
            sink,
            "ca_issued",
            Some(cn.to_string()),
            serde_json::json!({
                "serial": serial,
                "cn": cn,
                "sans": sans,
                "usage": operator_usage.label(),
                "not_after": leaf.not_after_rfc3339,
            }),
        )?;
        sink.emit(SecretEvent::CaIssued {
            serial: serial.clone(),
            cn: cn.to_string(),
            not_after: leaf.not_after_rfc3339,
        });
        Ok(serial)
    }

    #[cfg(feature = "mitm-ca")]
    pub fn ca_list(&self) -> anyhow::Result<Vec<CertListItem>> {
        let inner = &self.inner;
        let mut items = Vec::new();
        if let Some(not_after) = inner.store.get_meta(META_MITM_CA_NOT_AFTER)? {
            if inner.store.get_meta(META_MITM_CA_CERT_DER)?.is_some() {
                items.push(CertListItem {
                    cn: ca::CA_COMMON_NAME.to_string(),
                    is_ca: true,
                    not_after,
                    revoked: false,
                    sans: Vec::new(),
                    usage: "ca".to_string(),
                });
            }
        }
        items.extend(
            inner
                .store
                .list_certs()?
                .into_iter()
                .map(Self::cert_row_to_list_item),
        );
        Ok(items)
    }

    #[cfg(feature = "mitm-ca")]
    fn cert_row_to_list_item(row: CertRow) -> CertListItem {
        let mut sans = Vec::new();
        let mut usage = String::new();
        if let Ok((_rem, cert)) = x509_parser::parse_x509_certificate(&row.der) {
            if let Ok(Some(ext)) = cert.subject_alternative_name() {
                sans = ext
                    .value
                    .general_names
                    .iter()
                    .filter_map(|name| match name {
                        x509_parser::extensions::GeneralName::DNSName(dns) => {
                            Some((*dns).to_string())
                        }
                        _ => None,
                    })
                    .collect();
            }
            if let Ok(Some(ext)) = cert.extended_key_usage() {
                usage = match (ext.value.server_auth, ext.value.client_auth) {
                    (true, false) => "control_plane_server".to_string(),
                    (false, true) => "control_plane_client".to_string(),
                    (true, true) => "control_plane_server,control_plane_client".to_string(),
                    _ => String::new(),
                };
            }
        }
        CertListItem {
            cn: row.cn,
            is_ca: false,
            not_after: row.not_after,
            revoked: false,
            sans,
            usage,
        }
    }

    /// CA-less build: `ca_issue` still exists but the whole CA surface is unavailable.
    #[cfg(not(feature = "mitm-ca"))]
    pub fn ca_issue(
        &self,
        _cn: &str,
        _sans: &[String],
        _ttl_days: u64,
        _usage: &str,
        _sink: &EventSink,
    ) -> anyhow::Result<String> {
        anyhow::bail!("ca issue: built without the `mitm-ca` feature")
    }

    #[cfg(not(feature = "mitm-ca"))]
    pub fn ca_list(&self) -> anyhow::Result<Vec<CertListItem>> {
        anyhow::bail!("ca list: built without the `mitm-ca` feature")
    }

    /// Spawn a child with the provider env delta overlaid onto the inherited parent env, streaming
    /// its stdout/stderr line-by-line as `SecretEvent::Log` and returning its true exit code
    /// (128+signal on signal death). Fail-closed: refuses an empty argv or a program that does not
    /// resolve on PATH (durable `Refused` row + `GuardRefused` event), never printing.
    ///
    /// The real vault key is structurally absent here: only `plan.injection.env` (which carries the
    /// relay *bearer*, never a real key — see `inject::injection_template`) is overlaid.
    pub fn run_child(
        &self,
        plan: inject::ChildEnvPlan,
        argv: Vec<String>,
        sink: &EventSink,
    ) -> anyhow::Result<i32> {
        use std::io::{BufRead, BufReader};
        use std::process::{Command, Stdio};

        // Fail-closed: an empty argv has no program to resolve.
        let program = match argv.first() {
            Some(p) if !p.is_empty() => p.clone(),
            _ => {
                self.refuse(sink, "run_child", "<empty>", "empty argv")?;
                anyhow::bail!("run_child refused: empty argv");
            }
        };

        // Resolve the program on PATH via the existing `which` dep; refuse if unresolvable.
        let resolved = match which::which(&program) {
            Ok(p) => p,
            Err(e) => {
                self.refuse(
                    sink,
                    "run_child",
                    &program,
                    &format!("program not found on PATH: {e}"),
                )?;
                anyhow::bail!("run_child refused: program not found: {program}");
            }
        };

        let mut cmd = Command::new(&resolved);
        cmd.args(&argv[1..])
            // Overlay the provider env delta onto the INHERITED parent env (bearer only).
            .envs(plan.injection.env.iter())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                self.refuse(sink, "run_child", &program, &format!("spawn failed: {e}"))?;
                anyhow::bail!("run_child failed to spawn {program}: {e}");
            }
        };

        let source = program.clone();
        // Stream stdout + stderr concurrently so a full pipe on one stream can't deadlock the other.
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        std::thread::scope(|scope| {
            if let Some(out) = stdout {
                let sink = sink.clone();
                let source = source.clone();
                scope.spawn(move || {
                    for line in BufReader::new(out).lines().map_while(Result::ok) {
                        sink.emit(SecretEvent::Log {
                            source: source.clone(),
                            stream: Stream::Stdout,
                            line,
                        });
                    }
                });
            }
            if let Some(err) = stderr {
                let sink = sink.clone();
                let source = source.clone();
                scope.spawn(move || {
                    for line in BufReader::new(err).lines().map_while(Result::ok) {
                        sink.emit(SecretEvent::Log {
                            source: source.clone(),
                            stream: Stream::Stderr,
                            line,
                        });
                    }
                });
            }
        });

        let status = child.wait()?;
        // True exit code: 128 + signal on signal death (POSIX convention), else the exit code.
        let code = match status.code() {
            Some(c) => c,
            None => {
                use std::os::unix::process::ExitStatusExt;
                128 + status.signal().unwrap_or(0)
            }
        };

        sink.emit(SecretEvent::ChildExited { code });
        let mut summary = event::RunSummary::default();
        if code != 0 {
            summary.failed.push(source.clone());
        }
        sink.emit(SecretEvent::RunFinished { summary });
        let _ = plan.child_pid_hint; // peer-binding is a PR-2 concern (HF-8).
        Ok(code)
    }

    // ---- internal helpers ---------------------------------------------------------------------

    /// Build + persist a durable `Ok` audit row, then mirror it onto the (cosmetic) event channel.
    fn audit_ok(
        &self,
        sink: &EventSink,
        event_type: &str,
        subject: Option<String>,
        detail: serde_json::Value,
    ) -> anyhow::Result<()> {
        self.audit(sink, event_type, subject, detail, AuditOutcome::Ok)
    }

    fn audit_failed(
        &self,
        sink: &EventSink,
        event_type: &str,
        subject: Option<String>,
        detail: serde_json::Value,
    ) -> anyhow::Result<()> {
        self.audit(sink, event_type, subject, detail, AuditOutcome::Failed)
    }

    /// Emit a `GuardRefused` event + a durable `Refused` audit row (the engine's refusal discipline:
    /// a refused op is NOT an `Err` at the audit/event layer — the caller decides whether to map it
    /// to an `Err`/empty per its gate).
    fn refuse(
        &self,
        sink: &EventSink,
        event_type: &str,
        subject: &str,
        reason: &str,
    ) -> anyhow::Result<()> {
        self.audit(
            sink,
            event_type,
            Some(subject.to_string()),
            serde_json::json!({ "reason": reason }),
            AuditOutcome::Refused,
        )?;
        sink.emit(SecretEvent::GuardRefused {
            subject: subject.to_string(),
            reason: reason.to_string(),
        });
        Ok(())
    }

    fn audit(
        &self,
        sink: &EventSink,
        event_type: &str,
        subject: Option<String>,
        detail: serde_json::Value,
        outcome: AuditOutcome,
    ) -> anyhow::Result<()> {
        let ts = self.inner.clock.now().to_rfc3339();
        let actor_uid = Some(self.inner.owner_uid);
        let rec = vault::audit::new_row(ts, actor_uid, event_type, subject, detail, outcome);
        // Durable BEFORE return (HF-14): the store links + pushes synchronously.
        let seq = self.inner.store.append_audit(&rec)?;
        // Advance the DEK-keyed tail anchor when the vault is unlocked, so a store-level attacker
        // who later drops trailing rows (e.g. a refused reveal) cannot re-link a clean shorter
        // chain that `verify_chain` would accept — the anchor's `(seq, row_hash)` no longer match.
        // Rows written while LOCKED (init-before-unlock, failed unlock, lock) are not DEK-anchorable
        // at append time; they are covered by the unkeyed chain linkage forward from the anchored
        // row. Best-effort under a read lock; a failure to read the tail is non-fatal to the op.
        self.advance_audit_anchor_if_unlocked()?;
        // Mirror onto the cosmetic event channel with the sealed seq (best-effort).
        let mut mirrored = rec;
        mirrored.seq = seq;
        sink.emit(SecretEvent::Audit(mirrored));
        Ok(())
    }

    /// If the vault is unlocked, recompute + persist the DEK-keyed anchor over the CURRENT chain
    /// tail, advancing the monotonic high-water. No-op when locked (no resident DEK to key the
    /// anchor with).
    fn advance_audit_anchor_if_unlocked(&self) -> anyhow::Result<()> {
        let v = self.inner.vault.read().expect("vault lock");
        let Some(dek) = v.dek() else {
            return Ok(());
        };
        let (seq, tail_hash) = match self.inner.store.last_audit()? {
            Some(r) => (r.seq, r.row_hash),
            None => (0i64, Vec::new()),
        };
        self.write_audit_anchor(dek, seq, &tail_hash)
    }

    /// The single monotonic anchor-write choke point (used by both `advance_audit_anchor_if_unlocked`
    /// and the `init_vault` genesis anchor). Raises the persisted high-water to
    /// `max(stored_high_water, new_seq)` — a NON-DECREASING fence: a no-op read that did not grow the
    /// chain can never lower it. In the steady state `high_water == new_seq`.
    ///
    /// CRASH WINDOW (M-2 residual, fails CLOSED): the two writes — `META_AUDIT_HIGH_WATER` FIRST
    /// (`= N`), then the MAC bound to `(N, N, row@N)` — are NOT atomic on the `InMemStore`/single-key
    /// `put_meta` backend (no multi-key transaction). A crash BETWEEN them persists `high_water = N`
    /// while the MAC still commits to the previous high-water `N-1` (`MAC@(N-1, N-1, row@(N-1))`). On
    /// the next unlock, `verify_audit_anchor_with` runs against the honest live chain (`cur_seq = N`):
    /// the floor passes (`N < N` is false), but step 4 reconstructs `audit_head_mac(dek, N, N, row@N)`,
    /// which does NOT equal the stored `MAC@(N-1)` => `AuditChainBroken` => the NEXT UNLOCK IS REFUSED.
    /// So the true worst case is a hard unlock-DoS on an honest vault with NO in-engine recovery path
    /// (recovery needs an out-of-band re-anchor), NOT a "stale-by-one MAC that still verifies".
    /// Reversing the write order does not help (the MAC binds `high_water` either way). Security is
    /// preserved (it fails closed, never falsely PASSES a rolled-back chain). A true fix is a single
    /// atomic store transaction over the `(high_water, MAC)` pair (the libSQL backend, behind the
    /// `Store` trait) or persisting both under one `put_meta` blob; for the RAM-only / single-operator
    /// model this availability cost is the accepted M-2 residual (see THREAT-MODEL §5 A2 / M-2).
    fn write_audit_anchor(&self, dek: &Dek, new_seq: i64, tail_hash: &[u8]) -> anyhow::Result<()> {
        let prev_hw: i64 = self
            .inner
            .store
            .get_meta(META_AUDIT_HIGH_WATER)?
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let high_water = prev_hw.max(new_seq);
        self.inner
            .store
            .put_meta(META_AUDIT_HIGH_WATER, &high_water.to_string())?;
        // On every advance the live tail is the highest seq in the (only-growing) chain, so
        // `new_seq >= prev_hw` and `high_water == new_seq`: the anchored position IS the high-water,
        // and `tail_hash` is the row at it. We bind `high_water` into BOTH MAC seq fields so the
        // commitment is exactly what `verify_audit_anchor_with` reconstructs (`(hw, hw, row@hw)`),
        // leaving room in the wire shape (two fields) for a future L-1 locked-append window where the
        // high-water could exceed the anchored tail.
        //
        // SIGNER/VERIFIER AGREEMENT (cross-ref `verify_audit_anchor_with` step 4): the verifier
        // reconstructs the MAC over the `row_hash` at `seq == high_water`. Here we sign over
        // `tail_hash` = the row at `new_seq`. They agree ONLY while `high_water == new_seq`. If a
        // future L-1 locked-append window ever lets `prev_hw > new_seq` (the high-water exceeds the
        // just-anchored tail), this MUST instead fetch and bind the `row_hash` AT `high_water`, or the
        // signer (row@new_seq) and verifier (row@high_water) would diverge into a FALSE
        // `AuditChainBroken` on a legitimate chain. The invariant is asserted in debug so any such
        // change trips loudly here rather than shipping a silent verifier divergence.
        debug_assert!(
            high_water == new_seq,
            "write_audit_anchor invariant: high_water ({high_water}) must equal the anchored tail \
             seq ({new_seq}); the verifier reconstructs the MAC over row@high_water (see \
             verify_audit_anchor_with step 4). A locked-append window that raises high_water above \
             the anchored tail must bind row@high_water here, not row@new_seq."
        );
        let mac = audit_head_mac(dek, high_water, high_water, tail_hash);
        let _ = new_seq;
        self.inner
            .store
            .put_meta(META_AUDIT_HEAD, &hex_encode(&mac))?;
        Ok(())
    }

    /// Verify the DEK-keyed audit anchor against the live chain (truncation/rewrite detection).
    /// Requires the vault to be unlocked (the anchor is DEK-keyed). See `verify_audit_anchor_with`
    /// for the verification rule. Returns `Err(EngineError::AuditChainBroken)` on any mismatch.
    pub fn verify_audit_anchor(&self, _sink: &EventSink) -> anyhow::Result<()> {
        let v = self.inner.vault.read().expect("vault lock");
        let dek = match v.dek() {
            Some(d) => d,
            None => return Err(EngineError::Locked.into()),
        };
        self.verify_audit_anchor_with(dek)
    }

    /// Verify the DEK-keyed audit anchor against the live chain using an explicit DEK (so it can be
    /// driven from `unlock` with the just-recovered DEK, before it is committed into the vault).
    ///
    /// Rule (the H-1 fix):
    ///   1. the unkeyed `verify_chain` must pass (partial-mutation tamper-evidence);
    ///   2. read the monotonic high-water (`META_AUDIT_HIGH_WATER`);
    ///   3. **HIGH-WATER FLOOR** — reject if the live chain's current max-seq is BELOW the high-water
    ///      (the chain is shorter than the highest tail we ever anchored => truncation);
    ///   4. **ANCHORED-ROW MATCH** — the stored MAC must equal `audit_head_mac(dek, high_water,
    ///      high_water, anchored_row_hash)`, where `anchored_row_hash` is the `row_hash` of the row at
    ///      `seq == high_water` (the tail AS OF the last advance — NOT the current live tail, which
    ///      may sit above it after rows appended while LOCKED, and NOT "any row in the chain", the
    ///      defective old rule). `high_water == 0` (empty chain) uses the empty slice. Constant-time
    ///      compare. SIGNER SIDE: `write_audit_anchor` commits exactly this `(hw, hw, row@hw)` shape;
    ///      the two agree only while `high_water == anchored tail seq` (a `debug_assert` there guards
    ///      it). The step-4 compare is the load-bearing half closing covered-row rewrite AND a stale
    ///      lower-seq MAC replayed while `cur_seq >= high_water` (regression-pinned by
    ///      `stale_anchor_replay_caught_at_mac_not_floor`).
    ///
    /// Why match the row at `seq == high_water` rather than the live tail: `advance` always anchors
    /// the tail it just observed and sets `high_water == that seq`, so the anchored position IS the
    /// high-water. Rows appended while LOCKED (init / failed-unlock / lock / unlock markers) only ADD
    /// rows ABOVE the anchored seq — the anchored row stays present at `seq == high_water` — and the
    /// post-unlock advance re-anchors to the new tail. The contiguity guaranteed by `verify_chain`
    /// (1..=cur_seq) plus the floor (`cur_seq >= high_water`) means a row at `seq == high_water`
    /// always exists when `high_water >= 1`.
    ///
    /// Why this catches the stale-anchor replay the old "match any row" rule missed: after honest
    /// growth to seq N, `advance` raised the high-water (and the MAC) to N. Truncating back to k < N
    /// rows is rejected at (3) (`cur_max_seq = k < high_water = N`). Restoring an OLD captured anchor
    /// (high_water = k) WITHOUT also rewinding the plaintext high-water is rejected at (4) (the MAC is
    /// recomputed against the stored high-water N at row N, so the seq-k MAC won't match). Rewriting
    /// any covered field of the anchored row changes its `row_hash` and is caught at (4). The ONLY
    /// in-store-undetectable case is a FULL consistent snapshot rollback (rows + MAC + high-water
    /// rewound together) — the documented residual (THREAT-MODEL A2; needs off-box anchoring).
    fn verify_audit_anchor_with(&self, dek: &Dek) -> anyhow::Result<()> {
        use subtle::ConstantTimeEq;
        // 1. The chain itself must verify first (partial-mutation tamper-evidence).
        self.inner.store.verify_audit_chain()?;

        let Some(stored_hex) = self.inner.store.get_meta(META_AUDIT_HEAD)? else {
            // No anchor was ever written (only ever logged while locked); the unkeyed chain still
            // verified above, so there is nothing to anchor against.
            return Ok(());
        };
        let stored_mac = hex_decode(&stored_hex).ok_or(EngineError::AuditChainBroken(0))?;

        // 2. The high-water is mandatory once an anchor exists; a missing/garbled counter is a broken
        // chain (the fence was dropped), not a silent pass.
        let stored_hw: i64 = self
            .inner
            .store
            .get_meta(META_AUDIT_HIGH_WATER)?
            .ok_or(EngineError::AuditChainBroken(0))?
            .parse()
            .map_err(|_| EngineError::AuditChainBroken(0))?;

        let rows = self.inner.store.query_audit(0, usize::MAX)?;
        let cur_seq = rows.last().map_or(0i64, |r| r.seq);

        // 3. HIGH-WATER FLOOR: a live chain shorter than the highest anchored tail is a truncation.
        if cur_seq < stored_hw {
            return Err(EngineError::AuditChainBroken(cur_seq).into());
        }

        // 4. ANCHORED-ROW MATCH: reconstruct the anchor against the row AT the high-water seq (the
        // tail as of the last advance; rows appended while LOCKED sit above it). `verify_chain`
        // guarantees rows are 1..=cur_seq contiguous, so when `stored_hw >= 1` a row at that seq is
        // present at index `stored_hw - 1`.
        let anchored_hash: &[u8] = if stored_hw == 0 {
            &[]
        } else {
            match rows.get((stored_hw - 1) as usize) {
                Some(r) if r.seq == stored_hw => r.row_hash.as_slice(),
                _ => return Err(EngineError::AuditChainBroken(cur_seq).into()),
            }
        };
        let expect = audit_head_mac(dek, stored_hw, stored_hw, anchored_hash);
        if !bool::from(expect.as_slice().ct_eq(&stored_mac)) {
            return Err(EngineError::AuditChainBroken(cur_seq).into());
        }
        Ok(())
    }

    fn load_header_mac(&self) -> anyhow::Result<Vec<u8>> {
        let hexed = self
            .inner
            .store
            .get_meta(META_HEADER_MAC)?
            .ok_or(EngineError::UnlockFailed)?;
        hex_decode(&hexed).ok_or_else(|| EngineError::UnlockFailed.into())
    }

    fn load_issuance_floor(&self) -> anyhow::Result<i64> {
        let s = self
            .inner
            .store
            .get_meta(META_ISSUANCE_FLOOR_MS)?
            .ok_or(EngineError::UnlockFailed)?;
        s.parse::<i64>()
            .map_err(|_| EngineError::UnlockFailed.into())
    }

    /// Load the DEK generation, which is load-bearing for the record AAD binding. The value is
    /// bound into the header MAC (verified at unlock), so a missing or garbled meta value here is a
    /// setup-time failure — NOT a silent `unwrap_or(1)` default, which would convert a
    /// tamper/corruption signal into records sealed under the wrong generation.
    fn load_dek_generation(&self) -> anyhow::Result<i64> {
        let s = self
            .inner
            .store
            .get_meta(META_DEK_GENERATION)?
            .ok_or_else(|| anyhow::anyhow!("dek_generation missing"))?;
        s.parse::<i64>()
            .map_err(|_| anyhow::anyhow!("dek_generation is not a valid integer"))
    }
}

/// The result of `relay_swap_prepare`: either a (already-audited) deny, or an allow carrying the
/// extracted real key + the metadata `relay_swap` needs to audit/emit the successful send. The real
/// key lives here only until `send()` consumes it; `Zeroizing` wipes it on drop.
enum Prepared {
    Deny(DenyReason),
    Allow(AllowPrepared),
}

/// The result of the shared [`Engine::authorize_relay`] prelude — either a (NOT-yet-audited) deny
/// carrying the relay/token context the caller needs to audit + emit, or an allow carrying the
/// metadata the swap needs (plus the extracted real key when the caller bumped). Unlike `Prepared`,
/// this is the un-audited intermediate that BOTH the swap path and the streaming re-check share; the
/// caller owns the audit so the two callers can keep their distinct audit shapes (`relay_swapped` vs
/// the metadata-only tear-down). On the re-check path `real_key` is always `None` (no key fetched).
enum Authz {
    Deny {
        relay_id: Option<String>,
        token_id: Option<String>,
        reason: DenyReason,
    },
    Allow {
        relay_id: String,
        token_id: String,
        provider: Provider,
        host: String,
        /// The real secret on the SWAP path (`bump == true`); `None` on the streaming re-check
        /// (`bump == false`), which never materializes a key. `Zeroizing` wipes it on drop.
        real_key: Option<Zeroizing<Vec<u8>>>,
    },
}

impl Authz {
    /// Build a `Deny`, taking the token_id by `&str` for the (frequent) call-site ergonomics.
    fn deny(relay_id: Option<String>, token_id: Option<&str>, reason: DenyReason) -> Authz {
        Authz::Deny {
            relay_id,
            token_id: token_id.map(str::to_string),
            reason,
        }
    }
}

/// The verdict of the streaming re-check ([`Engine::relay_stream_authorized`], TASK-0032 / FS-S5):
/// the in-flight stream is still authorized, or it must be torn down carrying the `decide` reason
/// (for the edge's metadata-only tear-down audit). `DenyReason` is re-exported at the crate root.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamAuthz {
    Authorized,
    TearDown(DenyReason),
}

struct AllowPrepared {
    relay_id: String,
    token_id: String,
    /// Provider + the exact host that will be sent — carried so `relay_swap` can re-assert the HF-11
    /// canonical-upstream fence at the send site (not solely inside `decide`).
    provider: Provider,
    host: String,
    /// The real secret — flows ONLY to `Upstream::send`; never audited/emitted/returned.
    real_key: Zeroizing<Vec<u8>>,
}

/// Decide the SNI value `decide` binds against the verified inner Host (HF-9), per swap mode.
///
/// SECURITY NOTE (anti-fronting): the `sni` value here is read from a request header, which is
/// CLIENT-CONTROLLED — a malicious client can set it to match its Host (or omit it) to no-op the
/// check. So it is NOT a security control in modes where the engine does not observe the real TLS
/// SNI. We therefore split by `SwapMode`:
///
///   * `ProxyMitm` — the relay terminates TLS, so a genuine TLS-observed SNI is REQUIRED to enforce
///     anti-fronting. Until the proxy plumbs the observed SNI as a trusted field (Phase-4+), we fail
///     CLOSED: synthesize a sentinel SNI that can never equal the inner Host, so `decide` returns
///     `SniHostMismatch` rather than silently trusting the client header.
///   * everything else — there is no TLS termination at the relay, so there is nothing for the
///     engine to observe; we return `None` (the check is a documented no-op) instead of pretending a
///     client-supplied header is a real SNI.
fn trusted_sni_for(swap: &SwapMode, req: &EgressReq) -> Option<String> {
    match swap {
        // PR-3b: the MITM ingress terminates the child's TLS and records the handshake SNI in
        // `observed_sni`. Trust THAT (it is what the proxy actually saw on the wire, anti-fronting-
        // checked by the resolver), so `decide` enforces SNI == inner Host against a real name.
        // `None` (no SNI plumbed / not a MITM termination) fails closed with a sentinel that can
        // never equal a real host (the leading NUL byte is illegal in a DNS name) → SniHostMismatch.
        SwapMode::ProxyMitm => Some(
            req.observed_sni
                .clone()
                .unwrap_or_else(|| "\u{0}untrusted-sni-not-observed".to_string()),
        ),
        _ => None,
    }
}

/// A fixed, key-free label for an `UpstreamError` DISCRIMINANT — never its `Display` string (which
/// is adapter-controlled and could echo the real key). Used for the audit row + the refused outcome.
fn upstream_error_kind(e: &seam::UpstreamError) -> &'static str {
    match e {
        seam::UpstreamError::Io(_) => "io",
        seam::UpstreamError::HostNotAllowed(_) => "host_not_allowed",
    }
}

fn method_str(m: Method) -> &'static str {
    match m {
        Method::Get => "GET",
        Method::Head => "HEAD",
        Method::Post => "POST",
        Method::Put => "PUT",
        Method::Patch => "PATCH",
        Method::Delete => "DELETE",
        Method::Connect => "CONNECT",
        Method::Options => "OPTIONS",
    }
}

/// Format epoch-millis as an RFC3339 UTC string for the bearer's `expires_at` (cosmetic; the
/// authoritative deadline is the stored `expires_at_ms`).
fn ms_to_rfc3339(ms: i64) -> String {
    chrono::DateTime::<chrono::Utc>::from_timestamp_millis(ms)
        .unwrap_or_else(|| chrono::DateTime::<chrono::Utc>::from_timestamp_millis(0).unwrap())
        .to_rfc3339()
}

/// base64url, no padding (RFC 4648 §5). Used for the bearer secret (the actual authenticator). Pure
/// table-driven encode — no extra dependency.
fn b64url_nopad(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((n >> 6) & 0x3f) as usize] as char);
        }
        if chunk.len() > 2 {
            out.push(TABLE[(n & 0x3f) as usize] as char);
        }
    }
    out
}

/// Read the effective owner uid (the real uid the daemon runs as). Falls back to 0 on platforms
/// without `getuid` exposed through `rustix` — the engine never prints, so a best-effort value is
/// acceptable for the audit `actor_uid` and `SecretRead.by_uid`.
fn current_uid() -> u32 {
    rustix::process::getuid().as_raw()
}

/// Mint a fresh 32-byte DEK from the OS CSPRNG (getrandom-backed; the engine's nonce/key policy
/// mandates OsRng, OI-16). The scratch array is wrapped in `Zeroizing` so an early unwind wipes it
/// before the bytes are moved into the `Dek` (itself `ZeroizeOnDrop`).
fn mint_dek() -> Dek {
    let mut buf = Zeroizing::new([0u8; 32]);
    getrandom::getrandom(buf.as_mut()).expect("OS CSPRNG must produce 32 bytes for the DEK");
    Dek(*buf)
}

/// Fresh CSPRNG bytes (salts). `getrandom` is the OS CSPRNG; salts are non-secret but must be
/// unpredictable per slot so two slots never share a KDF salt.
fn random_bytes(n: usize) -> Vec<u8> {
    let mut v = vec![0u8; n];
    getrandom::getrandom(&mut v).expect("OS CSPRNG must produce salt bytes");
    v
}

/// DEK-keyed MAC over the audit chain tail `(seq, tail_row_hash)` AND the monotonic `high_water` —
/// the durable anchor that makes tail-truncation/rewrite AND stale-anchor replay detectable (the
/// unkeyed chain alone is only tamper-EVIDENT against partial mutation). Folding `high_water` in
/// makes the MAC a commitment to "the chain has reached AT LEAST `high_water` rows, whose tail at
/// anchoring time was `(seq, tail_row_hash)`"; for a current anchor `high_water == seq` (they advance
/// together). BLAKE3 `keyed_hash` is a 256-bit MAC; the key is derived from the DEK via BLAKE3
/// `derive_key` (domain-separated context) so the anchor is unforgeable without the unlocked DEK and
/// cannot be confused with the header MAC. Message layout (big-endian ints):
/// `AUDIT_HEAD_DOMAIN || high_water || seq || tail_row_hash`. `tail_row_hash` is the empty slice for
/// an empty chain (`seq == 0`).
fn audit_head_mac(dek: &Dek, high_water: i64, seq: i64, tail_row_hash: &[u8]) -> Vec<u8> {
    let key = blake3::derive_key(AUDIT_HEAD_KEY_INFO, &dek.0);
    let mut msg = Vec::with_capacity(AUDIT_HEAD_DOMAIN.len() + 16 + tail_row_hash.len());
    msg.extend_from_slice(AUDIT_HEAD_DOMAIN);
    msg.extend_from_slice(&high_water.to_be_bytes());
    msg.extend_from_slice(&seq.to_be_bytes());
    msg.extend_from_slice(tail_row_hash);
    blake3::keyed_hash(&key, &msg).as_bytes().to_vec()
}

/// Lowercase hex (no separators) — for the non-secret header MAC stored in meta.
fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(char::from_digit((b >> 4) as u32, 16).unwrap());
        s.push(char::from_digit((b & 0x0f) as u32, 16).unwrap());
    }
    s
}

/// Decode lowercase/uppercase hex with no separators; `None` on any malformed input.
fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let hi = (bytes[i] as char).to_digit(16)?;
        let lo = (bytes[i + 1] as char).to_digit(16)?;
        out.push(((hi << 4) | lo) as u8);
        i += 2;
    }
    Some(out)
}

fn factor_str(f: Factor) -> &'static str {
    match f {
        Factor::Usb => "usb",
        Factor::Passphrase => "passphrase",
    }
}

/// A do-nothing `Upstream` for `Engine::open` until the daemon wires the real (webpki-pinned)
/// sender. The 1b vault path never reaches `send()` (the relay path stays `todo!()`).
struct NullUpstream;

#[async_trait::async_trait]
impl Upstream for NullUpstream {
    async fn send(
        &self,
        _req: EgressReq,
        _real_key: &Zeroizing<Vec<u8>>,
    ) -> Result<EgressResp, seam::UpstreamError> {
        Err(seam::UpstreamError::Io("upstream not wired".to_string()))
    }
}

#[cfg(all(test, feature = "mitm-ca"))]
mod ca_tests {
    //! PR-3a CA-stack acceptance tests, driven through the PUBLIC `Engine` API over an `InMemStore`.
    //! No tokio: the CA path is fully synchronous. The shared store handle lets a test inspect audit
    //! rows + seed a relay policy for the coverage gate.
    use super::*;
    use crate::keyslot::{Argon2Params, ARGON2_M_KIB_FLOOR, ARGON2_T_COST_FLOOR};
    use crate::vault::store::{CertRow, RelayPolicyRow};
    use crate::vault::{InMemStore, Store};
    use std::sync::Arc;

    const NOW_MS: i64 = 1_700_000_000_000;

    /// A USB probe that never returns a keyfile (the CA tests use the passphrase factor only).
    struct NoUsb;
    impl UsbProbe for NoUsb {
        fn keyfile_for(&self, _uuid: &str) -> Option<Zeroizing<Vec<u8>>> {
            None
        }
    }

    /// Fixed-time clock so leaf/CA validity windows are deterministic.
    struct FixedClock;
    impl Clock for FixedClock {
        fn now(&self) -> chrono::DateTime<chrono::Utc> {
            chrono::DateTime::<chrono::Utc>::from_timestamp_millis(NOW_MS).unwrap()
        }
        fn boottime_ms(&self) -> i64 {
            NOW_MS
        }
    }

    /// A `Store` forwarding to a shared `Arc<InMemStore>` so a test keeps the concrete handle while
    /// the engine owns a `Box<dyn Store>`. (Mirrors tests/relay.rs SharedStore, trimmed.)
    struct SharedStore(Arc<InMemStore>);
    macro_rules! fwd {
        ($($f:ident($($a:ident : $t:ty),*) -> $r:ty;)*) => {
            $(fn $f(&self, $($a: $t),*) -> $r { self.0.$f($($a),*) })*
        };
    }
    impl Store for SharedStore {
        fwd! {
            get_meta(k: &str) -> anyhow::Result<Option<String>>;
            put_meta(k: &str, v: &str) -> anyhow::Result<()>;
            reserve_secret_row_id() -> anyhow::Result<i64>;
            put_secret(row: crate::vault::SecretRow) -> anyhow::Result<i64>;
            get_secret_latest(name: &str) -> anyhow::Result<Option<crate::vault::SecretRow>>;
            get_secret_version(name: &str, version: u32) -> anyhow::Result<Option<crate::vault::SecretRow>>;
            max_secret_version(name: &str) -> anyhow::Result<u32>;
            list_secret_names() -> anyhow::Result<Vec<String>>;
            list_secret_versions(name: &str) -> anyhow::Result<Vec<u32>>;
            delete_secret(name: &str) -> anyhow::Result<u32>;
            save_keyslot(slot: &Keyslot) -> anyhow::Result<()>;
            load_keyslots() -> anyhow::Result<Vec<Keyslot>>;
            load_keyslot(id: i64) -> anyhow::Result<Option<Keyslot>>;
            append_audit(rec: &AuditRecord) -> anyhow::Result<i64>;
            verify_audit_chain() -> anyhow::Result<()>;
            last_audit() -> anyhow::Result<Option<AuditRecord>>;
            query_audit(since_seq: i64, limit: usize) -> anyhow::Result<Vec<AuditRecord>>;
            save_relay_policy(row: RelayPolicyRow) -> anyhow::Result<i64>;
            load_relay_policy(relay_id: &str) -> anyhow::Result<Option<RelayPolicyRow>>;
            list_relay_policies() -> anyhow::Result<Vec<RelayPolicyRow>>;
            save_cert(row: CertRow) -> anyhow::Result<()>;
            load_cert(serial: &str) -> anyhow::Result<Option<CertRow>>;
            list_certs() -> anyhow::Result<Vec<CertRow>>;
        }
    }

    fn at_floor() -> Argon2Params {
        Argon2Params {
            m_kib: ARGON2_M_KIB_FLOOR,
            t_cost: ARGON2_T_COST_FLOOR,
            p_lanes: 1,
        }
    }

    fn paths() -> paths::Paths {
        // Unique per-test root so the 0600 PEM file does not collide between tests.
        let root = std::env::temp_dir().join(format!("env-ctl-ca-test-{}", std::process::id()));
        paths::Paths::under(root)
    }

    /// Build an engine over a SHARED in-mem store, init + unlock the vault (passphrase factor).
    fn unlocked_engine() -> (
        Engine,
        Arc<InMemStore>,
        EventSink,
        std::sync::mpsc::Receiver<SecretEvent>,
    ) {
        let store = Arc::new(InMemStore::new());
        let engine = Engine::with_seams(
            paths(),
            Box::new(SharedStore(store.clone())),
            Box::new(FixedClock),
            Box::new(NoUsb),
            Box::new(seam::NoMint),
            Box::new(NullUpstream),
            #[cfg(feature = "provider-github")]
            Box::new(crate::mint_github::NoopHttpTransport),
        )
        .expect("with_seams");
        let (sink, rx) = EventSink::channel();
        engine
            .init_vault(
                Zeroizing::new("correct horse battery staple".to_string()),
                None,
                None,
                at_floor(),
                &sink,
            )
            .expect("init_vault");
        engine
            .unlock(
                Unlock::Passphrase(Zeroizing::new("correct horse battery staple".to_string())),
                &sink,
            )
            .expect("unlock");
        (engine, store, sink, rx)
    }

    fn covering_policy(host: &str) -> RelayPolicyRow {
        RelayPolicyRow {
            id: 0,
            policy: RelayPolicy {
                relay_id: "claude-main".to_string(),
                kind: RelayKind::Named,
                provider: Provider::Anthropic,
                secret_name: "anthropic".to_string(),
                swap: SwapMode::ProxyMitm,
                host_allow: vec![host.to_string()],
                path_allow: vec!["/v1/".to_string()],
                method_allow: vec![Method::Post],
                policy_ttl_secs: 31_536_000,
                rate_per_min: None,
                quota_total_requests: None,
                quota_total_bytes: None,
                enabled: true,
                revoked: false,
            },
        }
    }

    fn audit_has(store: &InMemStore, event_type: &str, outcome: AuditOutcome) -> bool {
        store
            .audit_rows()
            .iter()
            .any(|r| r.event_type == event_type && r.outcome == outcome)
    }

    #[test]
    fn init_persist_unlock_roundtrip() {
        let (engine, store, sink, _rx) = unlocked_engine();
        // Fresh: no CA.
        assert!(store.get_meta(META_MITM_CA_CERT_DER).unwrap().is_none());

        engine.ca_init(true, &sink).expect("ca_init apply");
        assert!(store.get_meta(META_MITM_CA_CERT_DER).unwrap().is_some());
        assert!(store.get_meta(META_MITM_CA_NOT_AFTER).unwrap().is_some());
        assert!(audit_has(&store, "ca_initialized", AuditOutcome::Ok));

        // Idempotent: a second apply is an Ok no-op (no second key version).
        engine.ca_init(true, &sink).expect("ca_init idempotent");
        assert_eq!(
            store.max_secret_version(META_MITM_CA_KEY_NAME).unwrap(),
            1,
            "idempotent ca_init must not seal a second CA key"
        );

        // Survives lock -> unlock: the issuer is rebuilt and can issue again.
        engine.lock(&sink).expect("lock");
        engine
            .unlock(
                Unlock::Passphrase(Zeroizing::new("correct horse battery staple".to_string())),
                &sink,
            )
            .expect("re-unlock");
        store
            .save_relay_policy(covering_policy("api.anthropic.com"))
            .unwrap();
        let (chain, _k) = engine
            .issue_leaf_for_covered_host("api.anthropic.com", &sink)
            .expect("issue after re-unlock");
        assert_eq!(chain.len(), 2);
    }

    #[test]
    fn lock_zeroizes_issuer_no_issuance_when_locked() {
        let (engine, store, sink, _rx) = unlocked_engine();
        engine.ca_init(true, &sink).expect("ca_init");
        store
            .save_relay_policy(covering_policy("api.anthropic.com"))
            .unwrap();
        engine.lock(&sink).expect("lock");
        // Locked: no leaf, even for a covered host.
        let err = engine
            .issue_leaf_for_covered_host("api.anthropic.com", &sink)
            .unwrap_err();
        assert!(
            format!("{err}").contains("locked"),
            "locked vault must refuse issuance"
        );
    }

    #[test]
    fn cf5_ca_issue_refuses_mitm_leaf() {
        let (engine, store, sink, _rx) = unlocked_engine();
        engine.ca_init(true, &sink).expect("ca_init");

        // mitm_leaf is refused (Err + a Refused audit row + GuardRefused event).
        let err = engine
            .ca_issue("host", &["host".to_string()], 0, "mitm_leaf", &sink)
            .unwrap_err();
        assert!(format!("{err}").contains("CF-5"));
        assert!(audit_has(&store, "ca_issued", AuditOutcome::Refused));

        // A control-plane usage is NOT a CF-5 refusal: it mints and persists a public cert row.
        let serial = engine
            .ca_issue(
                "control.test",
                &["control.test".to_string()],
                7,
                "control_plane_client",
                &sink,
            )
            .expect("control-plane issue");
        assert!(!serial.is_empty());
        assert!(audit_has(&store, "ca_issued", AuditOutcome::Ok));
        let certs = engine.ca_list().expect("ca list");
        assert!(certs.iter().any(|c| c.is_ca));
        let leaf = certs
            .iter()
            .find(|c| !c.is_ca && c.cn == "control.test")
            .expect("issued leaf is listed");
        assert_eq!(leaf.usage, "control_plane_client");
        assert_eq!(leaf.sans, vec!["control.test".to_string()]);
    }

    #[test]
    fn relay_coverage_gate() {
        let (engine, store, sink, _rx) = unlocked_engine();
        engine.ca_init(true, &sink).expect("ca_init");

        // Uncovered host (no policy yet): refused, no leaf.
        let err = engine
            .issue_leaf_for_covered_host("evil.example.com", &sink)
            .unwrap_err();
        assert!(format!("{err}").contains("not covered"));
        assert!(audit_has(&store, "leaf_minted", AuditOutcome::Refused));

        // Add a covering active policy: now a leaf is minted.
        store
            .save_relay_policy(covering_policy("api.anthropic.com"))
            .unwrap();
        let (chain, _k) = engine
            .issue_leaf_for_covered_host("api.anthropic.com", &sink)
            .expect("covered host issues");
        assert_eq!(chain.len(), 2);
        assert!(audit_has(&store, "leaf_minted", AuditOutcome::Ok));

        // A disabled policy does NOT cover its host.
        let mut disabled = covering_policy("api.disabled.com");
        disabled.policy.enabled = false;
        store.save_relay_policy(disabled).unwrap();
        let err2 = engine
            .issue_leaf_for_covered_host("api.disabled.com", &sink)
            .unwrap_err();
        assert!(format!("{err2}").contains("not covered"));
    }

    #[test]
    fn apply_gate_dryrun_persists_nothing() {
        let (engine, store, sink, _rx) = unlocked_engine();
        engine.ca_init(false, &sink).expect("ca_init dry-run");
        assert!(
            store.get_meta(META_MITM_CA_CERT_DER).unwrap().is_none(),
            "dry-run must not persist the cert"
        );
        assert!(
            store
                .get_secret_latest(META_MITM_CA_KEY_NAME)
                .unwrap()
                .is_none(),
            "dry-run must not seal the key"
        );
        assert!(audit_has(&store, "ca_initialized", AuditOutcome::Refused));
    }

    #[test]
    fn ca_key_not_revealable_via_secret_get() {
        let (engine, _store, sink, _rx) = unlocked_engine();
        engine.ca_init(true, &sink).expect("ca_init");
        // The sealed CA key is broker_only: a reveal is refused (HF-5), never returns the key bytes.
        let err = engine
            .secret_get(META_MITM_CA_KEY_NAME, true, true, &sink)
            .unwrap_err();
        assert!(
            format!("{err}").contains("broker-only"),
            "ca key must be un-revealable via secret_get"
        );
    }

    #[test]
    fn ca_pem_path_is_0600_and_public_only() {
        let (engine, _store, sink, _rx) = unlocked_engine();
        engine.ca_init(true, &sink).expect("ca_init");
        let path = engine.ca_pem_path().expect("ca_pem_path");
        let contents = std::fs::read_to_string(&path).expect("read pem");
        assert!(contents.contains("BEGIN CERTIFICATE"));
        assert!(!contents.contains("PRIVATE KEY"), "PEM must be public-only");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600, "ca pem must be 0600");
        }
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn ca_pem_path_refuses_without_ca() {
        let (engine, _store, _sink, _rx) = unlocked_engine();
        let err = engine.ca_pem_path().unwrap_err();
        assert!(format!("{err}").contains("CA not initialized"));
    }

    // ---- TASK-0035: secret_list / secret_meta / secret_rm / secret_rotate / relay_* / audit ----

    fn put(engine: &Engine, sink: &EventSink, name: &str, provider: Provider, broker_only: bool) {
        engine
            .secret_put(
                SecretMeta {
                    name: name.to_string(),
                    provider,
                    note: format!("{name}-note"),
                    broker_only,
                },
                Zeroizing::new(b"SUPER-SECRET-VALUE".to_vec()),
                sink,
            )
            .expect("secret_put");
    }

    #[test]
    fn secret_list_is_metadata_only_and_provider_filtered() {
        let (engine, _store, sink, _rx) = unlocked_engine();
        put(&engine, &sink, "anth", Provider::Anthropic, false);
        put(&engine, &sink, "oai", Provider::Openai, true);

        // Unfiltered: both, metadata only.
        let all = engine.secret_list(None, &sink).expect("list");
        assert_eq!(all.len(), 2);
        assert!(all
            .iter()
            .all(|i| i.version == 1 && !i.created_ts.is_empty()));
        // broker_only is exposed as a plain flag (not a reveal).
        assert!(all.iter().any(|i| i.name == "oai" && i.broker_only));

        // Provider filter narrows to one.
        let filtered = engine
            .secret_list(Some(Provider::Anthropic), &sink)
            .expect("list filtered");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "anth");

        // NO secret bytes leak into the list output (serialize every item, scan for the sentinel).
        let dumped = all
            .iter()
            .map(|i| {
                format!(
                    "{} {:?} {} {} {}",
                    i.name, i.provider, i.note, i.version, i.created_ts
                )
            })
            .collect::<String>();
        assert!(
            !dumped.contains("SUPER-SECRET-VALUE"),
            "secret_list leaked plaintext"
        );
    }

    #[test]
    fn secret_list_and_meta_refuse_when_locked() {
        let (engine, _store, sink, _rx) = unlocked_engine();
        put(&engine, &sink, "anth", Provider::Anthropic, false);
        engine.lock(&sink).expect("lock");
        let e1 = engine.secret_list(None, &sink).unwrap_err();
        assert!(matches!(
            e1.downcast_ref::<EngineError>(),
            Some(EngineError::Locked)
        ));
        let e2 = engine.secret_meta("anth").unwrap_err();
        assert!(matches!(
            e2.downcast_ref::<EngineError>(),
            Some(EngineError::Locked)
        ));
    }

    #[test]
    fn secret_meta_returns_non_secret_fields() {
        let (engine, _store, sink, _rx) = unlocked_engine();
        put(&engine, &sink, "anth", Provider::Anthropic, true);
        let m = engine.secret_meta("anth").expect("meta").expect("some");
        assert_eq!(m.name, "anth");
        assert_eq!(m.provider, Provider::Anthropic);
        assert!(m.broker_only);
        assert!(engine.secret_meta("nope").expect("meta").is_none());
    }

    #[test]
    fn secret_rm_dry_run_mutates_nothing_apply_removes() {
        let (engine, store, sink, _rx) = unlocked_engine();
        put(&engine, &sink, "anth", Provider::Anthropic, false);

        // Dry-run: reports would-remove count, mutates nothing.
        let would = engine.secret_rm("anth", false, &sink).expect("rm dry");
        assert_eq!(would, 1);
        assert!(
            store.get_secret_latest("anth").unwrap().is_some(),
            "dry-run removed a row"
        );

        // Apply: removes the row, writes an Ok audit row.
        let removed = engine.secret_rm("anth", true, &sink).expect("rm apply");
        assert_eq!(removed, 1);
        assert!(
            store.get_secret_latest("anth").unwrap().is_none(),
            "apply did not remove"
        );
        assert!(audit_has(&store, "secret_removed", AuditOutcome::Ok));
        // No secret bytes in any audit row.
        let dumped = store
            .audit_rows()
            .iter()
            .map(|r| r.detail.to_string())
            .collect::<String>();
        assert!(
            !dumped.contains("SUPER-SECRET-VALUE"),
            "secret_rm leaked plaintext into audit"
        );
    }

    #[test]
    fn secret_rm_refuses_when_locked() {
        let (engine, store, sink, _rx) = unlocked_engine();
        put(&engine, &sink, "anth", Provider::Anthropic, false);
        engine.lock(&sink).expect("lock");
        let err = engine.secret_rm("anth", true, &sink).unwrap_err();
        assert!(matches!(
            err.downcast_ref::<EngineError>(),
            Some(EngineError::Locked)
        ));
        assert!(audit_has(&store, "secret_removed", AuditOutcome::Refused));
    }

    #[test]
    fn secret_rotate_dry_run_then_apply_appends_version() {
        let (engine, store, sink, _rx) = unlocked_engine();
        put(&engine, &sink, "anth", Provider::Anthropic, true);

        // Dry-run: no new version.
        engine
            .secret_rotate(
                "anth",
                Zeroizing::new(b"NEW-ROTATED-VALUE".to_vec()),
                false,
                &sink,
            )
            .expect("rotate dry");
        assert_eq!(
            store.max_secret_version("anth").unwrap(),
            1,
            "dry-run appended a version"
        );

        // Apply: appends version 2, carrying broker_only/provider forward.
        engine
            .secret_rotate(
                "anth",
                Zeroizing::new(b"NEW-ROTATED-VALUE".to_vec()),
                true,
                &sink,
            )
            .expect("rotate apply");
        assert_eq!(store.max_secret_version("anth").unwrap(), 2);
        let latest = store.get_secret_latest("anth").unwrap().unwrap();
        assert_eq!(latest.provider, Provider::Anthropic);
        assert!(latest.broker_only, "rotate must carry broker_only forward");
        assert!(audit_has(&store, "secret_rotated", AuditOutcome::Ok));
        // No plaintext in audit.
        let dumped = store
            .audit_rows()
            .iter()
            .map(|r| r.detail.to_string())
            .collect::<String>();
        assert!(
            !dumped.contains("NEW-ROTATED-VALUE"),
            "rotate leaked plaintext into audit"
        );
    }

    #[test]
    fn secret_rotate_refuses_locked_and_unknown() {
        let (engine, _store, sink, _rx) = unlocked_engine();
        // Unknown secret (unlocked).
        let err = engine
            .secret_rotate("nope", Zeroizing::new(b"x".to_vec()), true, &sink)
            .unwrap_err();
        assert!(format!("{err}").contains("unknown secret"));
        // Locked.
        put(&engine, &sink, "anth", Provider::Anthropic, false);
        engine.lock(&sink).expect("lock");
        let err = engine
            .secret_rotate("anth", Zeroizing::new(b"x".to_vec()), true, &sink)
            .unwrap_err();
        assert!(matches!(
            err.downcast_ref::<EngineError>(),
            Some(EngineError::Locked)
        ));
    }

    #[test]
    fn relay_create_persists_and_list_filters_revoked() {
        let (engine, store, sink, _rx) = unlocked_engine();
        let id = engine
            .relay_create(covering_policy("api.anthropic.com").policy, &sink)
            .expect("create");
        assert!(id > 0);
        assert!(audit_has(&store, "relay_created", AuditOutcome::Ok));

        // List excludes revoked by default, includes them with the flag.
        let active = engine.relay_list(false, &sink).expect("list active");
        assert_eq!(active.len(), 1);

        // Revoke it, then confirm filtering.
        engine
            .relay_revoke("claude-main", true, &sink)
            .expect("revoke");
        assert!(engine.relay_list(false, &sink).expect("list").is_empty());
        assert_eq!(engine.relay_list(true, &sink).expect("list all").len(), 1);
    }

    #[test]
    fn audit_query_clamps_limit_and_returns_rows() {
        let (engine, _store, sink, _rx) = unlocked_engine();
        put(&engine, &sink, "anth", Provider::Anthropic, false);
        // A huge limit is clamped; rows are returned metadata-only (AuditRecords carry no value).
        let rows = engine.audit_query(0, usize::MAX, &sink).expect("query");
        assert!(rows.len() <= 1000);
        assert!(rows.iter().any(|r| r.event_type == "secret_written"));
        let dumped = rows
            .iter()
            .map(|r| r.detail.to_string())
            .collect::<String>();
        assert!(
            !dumped.contains("SUPER-SECRET-VALUE"),
            "audit_query leaked plaintext"
        );
    }
}

/// G2 native-mint engine acceptance tests (DD-1 late-bind + `resolve_injection`), driven through the
/// PUBLIC `Engine` API over an `InMemStore`. Self-contained (its own minimal seams + helpers) so it
/// compiles under `--features provider-github` without requiring `mitm-ca`.
#[cfg(all(test, feature = "provider-github"))]
mod native_mint_tests {
    use super::*;
    use crate::keyslot::{Argon2Params, ARGON2_M_KIB_FLOOR, ARGON2_T_COST_FLOOR};
    use crate::mint_github::{
        GitHubAppMint, HttpRequest, HttpResponse, HttpTransport, TransportError,
    };
    use crate::seam::{MintError, MintRequest, ProviderMint, ScopedToken};
    use std::sync::Mutex;

    const NOW_MS: i64 = 1_700_000_000_000;

    /// A throwaway 1024-bit RSA key (PKCS#1) — weak BY DESIGN, never a real credential. Same fixture
    /// shape `mint_github`'s own tests use.
    const TEST_PEM: &str = "-----BEGIN RSA PRIVATE KEY-----\nMIICXgIBAAKBgQDw1EvUY2q80CzzraBZxIBLq1xjF9Eu5PsEseAd2bD+oJo4QQkI\npGycm26vJalBiW/rdzcSPaxPUT7KgH1IeftkUL0pbDG6nN08MgJM0/LjVKx3fK5A\n2Lq+CCh+eHfRGxcX8haBzWcwi4tfb90/7Vi9CGh7IXyyMTWLNW/mBVoH8wIDAQAB\nAoGBAMSPYbzdz9Z/ytCwm7noyhX4rRUr8U3nEoIIdDWo4e9RQc48NpVZLlS8ACDw\nCi81b6WtzcMTlzm9xBQfvyGSff0S/cCPAWEfGNItWOg5jeLSNftDVh4yM06BPEOI\nf+FwkGPiQYtCnhSXLhQq0ClODymjHyW+M7MBf8iyqnd8bnUhAkEA/q8Z5C7YQSFq\nIbywMegUkmCykiX8oCrvykg8i5oOjZXhIp/hnxv6jYynZd0PV1oOtbVTuvEve8kr\nCj+84GCPKQJBAPIS3i9C1VaaecCoSlnSY6FHWXmbLsm4wqXGbcyS0m4tQclIXfsd\nuDO4AUTu6Xc893Xfa3M/4Jpl7Fs5TReVbbsCQQCUFIlQVDBmxh/oV8Z2bgMwDMsn\nELEvC2f6zD9vx/Y4OnH5aM6NbX4juSlHn92go3s0CacSZdN+/LtqrR6Ls3jpAkBC\n/DOdUlokf9SHGkqQtmY5X7wDqYx153l9U/5YKJywPjfBEhRng57QOO+o+o+CHk2/\nwVZDav6k2uVfjOinSQM3AkEApokk6NycDKY657zkXPtlhKBsvyxfVW+evW9XjoHi\nEnHNytN8c6NOpZMjmzxgSUoOpAI4OVMIH00OvKHIIpvN0w==\n-----END RSA PRIVATE KEY-----";

    struct NoUsb;
    impl UsbProbe for NoUsb {
        fn keyfile_for(&self, _uuid: &str) -> Option<Zeroizing<Vec<u8>>> {
            None
        }
    }

    struct FixedClock;
    impl Clock for FixedClock {
        fn now(&self) -> chrono::DateTime<chrono::Utc> {
            chrono::DateTime::<chrono::Utc>::from_timestamp_millis(NOW_MS).unwrap()
        }
        fn boottime_ms(&self) -> i64 {
            NOW_MS
        }
    }

    /// In-process fake transport: replays a canned status+body (no network).
    struct FakeTransport {
        status: u16,
        body: String,
    }
    impl HttpTransport for FakeTransport {
        fn execute(&self, _req: &HttpRequest) -> Result<HttpResponse, TransportError> {
            Ok(HttpResponse {
                status: self.status,
                body: self.body.clone().into_bytes(),
            })
        }
    }

    /// A counting minter so we can prove install/replace/clear actually swap the boxed provider.
    struct CountingMint {
        token: String,
        calls: Mutex<u32>,
    }
    impl ProviderMint for CountingMint {
        fn mint_scoped(&self, _p: &MintRequest) -> Result<ScopedToken, MintError> {
            *self.calls.lock().unwrap() += 1;
            Ok(ScopedToken {
                token: Zeroizing::new(self.token.clone().into_bytes()),
                expires_at: 1_700_003_600,
            })
        }
    }

    fn at_floor() -> Argon2Params {
        Argon2Params {
            m_kib: ARGON2_M_KIB_FLOOR,
            t_cost: ARGON2_T_COST_FLOOR,
            p_lanes: 1,
        }
    }

    fn paths() -> paths::Paths {
        let root = std::env::temp_dir().join(format!(
            "env-ctl-native-test-{}-{}",
            std::process::id(),
            NOW_MS
        ));
        paths::Paths::under(root)
    }

    fn unlocked_engine() -> (Engine, EventSink, std::sync::mpsc::Receiver<SecretEvent>) {
        let engine = Engine::with_seams(
            paths(),
            Box::new(vault::InMemStore::new()),
            Box::new(FixedClock),
            Box::new(NoUsb),
            Box::new(seam::NoMint),
            Box::new(NullUpstream),
            #[cfg(feature = "provider-github")]
            Box::new(crate::mint_github::NoopHttpTransport),
        )
        .expect("with_seams");
        let (sink, rx) = EventSink::channel();
        engine
            .init_vault(
                Zeroizing::new("correct horse battery staple".to_string()),
                None,
                None,
                at_floor(),
                &sink,
            )
            .expect("init_vault");
        engine
            .unlock(
                Unlock::Passphrase(Zeroizing::new("correct horse battery staple".to_string())),
                &sink,
            )
            .expect("unlock");
        (engine, sink, rx)
    }

    fn drain(rx: &std::sync::mpsc::Receiver<SecretEvent>) -> Vec<SecretEvent> {
        let mut out = Vec::new();
        while let Ok(ev) = rx.try_recv() {
            out.push(ev);
        }
        out
    }

    // ---- U2: late-bind provider install / replace / clear ------------------------------------

    #[test]
    fn provider_install_replace_and_clear() {
        let (engine, sink, _rx) = unlocked_engine();
        // Default is NoMint: a native resolve is Unsupported ⇒ falls back to the proxy-swap shape.
        let r = engine
            .resolve_injection(
                Provider::Github,
                "gh",
                "relay-bearer",
                "http://127.0.0.1:9",
                "",
                inject::DataPlaneMode::NativeSubtoken,
                vec![],
                vec![],
                3600,
                &sink,
            )
            .unwrap()
            .expect("fallback injection present");
        assert_eq!(
            r.mode,
            inject::DataPlaneMode::HttpsProxyMitm,
            "NoMint ⇒ proxy-swap fallback"
        );

        // Install a counting minter; now a native resolve mints (the minted token is injected).
        engine.install_provider(Box::new(CountingMint {
            token: "ghs_installed".into(),
            calls: Mutex::new(0),
        }));
        let r = engine
            .resolve_injection(
                Provider::Github,
                "gh",
                "relay-bearer",
                "",
                "",
                inject::DataPlaneMode::NativeSubtoken,
                vec![],
                vec![],
                3600,
                &sink,
            )
            .unwrap()
            .expect("minted injection present");
        assert_eq!(r.mode, inject::DataPlaneMode::NativeSubtoken);
        assert_eq!(
            r.env.get("GITHUB_TOKEN").map(String::as_str),
            Some("ghs_installed")
        );

        // Clear ⇒ back to NoMint ⇒ proxy-swap fallback again.
        engine.clear_provider();
        let r = engine
            .resolve_injection(
                Provider::Github,
                "gh",
                "relay-bearer",
                "http://127.0.0.1:9",
                "",
                inject::DataPlaneMode::NativeSubtoken,
                vec![],
                vec![],
                3600,
                &sink,
            )
            .unwrap()
            .expect("fallback injection present after clear");
        assert_eq!(r.mode, inject::DataPlaneMode::HttpsProxyMitm);
    }

    #[test]
    fn lock_clears_installed_provider() {
        let (engine, sink, _rx) = unlocked_engine();
        engine.install_provider(Box::new(CountingMint {
            token: "ghs_x".into(),
            calls: Mutex::new(0),
        }));
        // Locking the vault must drop the minter (defense-in-depth) ⇒ native resolve falls back.
        engine.lock(&sink).expect("lock");
        // Re-unlock to call resolve (resolve itself doesn't require unlock, but keep it realistic).
        engine
            .unlock(
                Unlock::Passphrase(Zeroizing::new("correct horse battery staple".to_string())),
                &sink,
            )
            .expect("re-unlock");
        let r = engine
            .resolve_injection(
                Provider::Github,
                "gh",
                "relay-bearer",
                "http://127.0.0.1:9",
                "",
                inject::DataPlaneMode::NativeSubtoken,
                vec![],
                vec![],
                3600,
                &sink,
            )
            .unwrap()
            .expect("fallback injection present");
        assert_eq!(
            r.mode,
            inject::DataPlaneMode::HttpsProxyMitm,
            "lock cleared the minter"
        );
    }

    // ---- U2: app_credential_pem custody ------------------------------------------------------

    #[test]
    fn app_credential_pem_reads_pem_and_meta_when_unlocked() {
        let (engine, sink, _rx) = unlocked_engine();
        // Seed the App PEM as a broker_only secret + the app_id/installation_id meta keys.
        engine
            .secret_put(
                SecretMeta {
                    name: "github_app/flexnetos".into(),
                    provider: Provider::Github,
                    note: "test app key".into(),
                    broker_only: true,
                },
                Zeroizing::new(TEST_PEM.as_bytes().to_vec()),
                &sink,
            )
            .expect("secret_put");
        engine
            .put_app_credential_meta("github_app/flexnetos", "42", 99)
            .expect("put meta");

        let (pem, app_id, installation_id) = engine
            .app_credential_pem("github_app/flexnetos")
            .expect("ok")
            .expect("credential present");
        assert_eq!(&*pem, TEST_PEM.as_bytes());
        assert_eq!(app_id, "42");
        assert_eq!(installation_id, 99);

        // No credential enrolled under an unknown name ⇒ Ok(None).
        assert!(engine.app_credential_pem("nope").unwrap().is_none());
    }

    #[test]
    fn app_credential_pem_refuses_when_locked() {
        let (engine, sink, _rx) = unlocked_engine();
        engine
            .secret_put(
                SecretMeta {
                    name: "github_app/flexnetos".into(),
                    provider: Provider::Github,
                    note: "k".into(),
                    broker_only: true,
                },
                Zeroizing::new(TEST_PEM.as_bytes().to_vec()),
                &sink,
            )
            .expect("secret_put");
        engine
            .put_app_credential_meta("github_app/flexnetos", "42", 99)
            .expect("put meta");
        engine.lock(&sink).expect("lock");
        // Locked vault ⇒ the App PEM cannot materialize (structural fail-closed gate).
        assert!(engine.app_credential_pem("github_app/flexnetos").is_err());
    }

    // ---- U3: resolve_injection mint / fallback / refuse --------------------------------------

    fn install_github(engine: &Engine, status: u16, body: &str) {
        let minter = GitHubAppMint::new(
            "42",
            99,
            Zeroizing::new(TEST_PEM.as_bytes().to_vec()),
            FixedClock,
            FakeTransport {
                status,
                body: body.to_string(),
            },
        )
        .with_api_base("https://gh.test");
        engine.install_provider(Box::new(minter));
    }

    #[test]
    fn native_subtoken_injects_minted_token_not_bearer() {
        let (engine, sink, rx) = unlocked_engine();
        install_github(
            &engine,
            201,
            r#"{"token":"ghs_minted_abc","expires_at":"2026-06-12T23:00:00Z"}"#,
        );
        let _ = drain(&rx); // discard unlock/seed events
        let r = engine
            .resolve_injection(
                Provider::Github,
                "gh-relay",
                "relay-bearer-DO-NOT-LEAK",
                "",
                "",
                inject::DataPlaneMode::NativeSubtoken,
                vec!["meta".into()],
                vec!["checks:write".into()],
                3600,
                &sink,
            )
            .unwrap()
            .expect("minted injection");
        assert_eq!(r.mode, inject::DataPlaneMode::NativeSubtoken);
        // The MINTED token is injected, NOT the relay bearer.
        assert_eq!(
            r.env.get("GITHUB_TOKEN").map(String::as_str),
            Some("ghs_minted_abc")
        );
        assert_eq!(
            r.env.get("GH_TOKEN").map(String::as_str),
            Some("ghs_minted_abc")
        );
        for v in r.env.values() {
            assert_ne!(
                v, "relay-bearer-DO-NOT-LEAK",
                "relay bearer must NOT be injected"
            );
        }
        // The event carries expires_at + relay only — NEVER the minted token.
        let events = drain(&rx);
        let minted = events.iter().find_map(|e| match e {
            SecretEvent::RelayMinted {
                relay, expires_at, ..
            } => Some((relay.clone(), expires_at.clone())),
            _ => None,
        });
        let (relay, expires_at) = minted.expect("RelayMinted emitted");
        assert_eq!(relay, "gh-relay");
        assert!(!expires_at.is_empty(), "expires_at surfaced honestly");
        for e in &events {
            let json = serde_json::to_string(e).unwrap();
            assert!(
                !json.contains("ghs_minted_abc"),
                "minted token must never appear in an event: {json}"
            );
        }
    }

    #[test]
    fn native_subtoken_unsupported_falls_back_to_proxy_swap() {
        let (engine, sink, _rx) = unlocked_engine();
        // No minter installed (NoMint) ⇒ Unsupported ⇒ proxy-swap fallback shape.
        let r = engine
            .resolve_injection(
                Provider::Github,
                "gh-relay",
                "relay-bearer",
                "http://127.0.0.1:9443",
                "/run/ca.pem",
                inject::DataPlaneMode::NativeSubtoken,
                vec![],
                vec![],
                3600,
                &sink,
            )
            .unwrap()
            .expect("fallback injection");
        assert_eq!(r.mode, inject::DataPlaneMode::HttpsProxyMitm);
        // The relay bearer is what the proxy-swap fallback injects.
        assert_eq!(
            r.env.get("GITHUB_TOKEN").map(String::as_str),
            Some("relay-bearer")
        );
        assert_eq!(
            r.env.get("HTTPS_PROXY").map(String::as_str),
            Some("http://127.0.0.1:9443")
        );
    }

    #[test]
    fn native_subtoken_other_error_refuses() {
        let (engine, sink, rx) = unlocked_engine();
        // 404 from GitHub ⇒ MintError::Other ⇒ REFUSE: Ok(None), durable Refused row + GuardRefused.
        install_github(&engine, 404, r#"{"message":"Not Found"}"#);
        let _ = drain(&rx);
        let resolved = engine
            .resolve_injection(
                Provider::Github,
                "gh-relay",
                "relay-bearer",
                "",
                "",
                inject::DataPlaneMode::NativeSubtoken,
                vec![],
                vec![],
                3600,
                &sink,
            )
            .unwrap();
        assert!(resolved.is_none(), "Other error ⇒ refuse, NO injection");
        let events = drain(&rx);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, SecretEvent::GuardRefused { .. })),
            "a GuardRefused event must be emitted on refuse"
        );
        // No minted token (there isn't one) and no bearer leaked into any event.
        for e in &events {
            let json = serde_json::to_string(e).unwrap();
            assert!(!json.contains("relay-bearer"));
        }
    }

    // ---- U4: scope → MintRequest -------------------------------------------------------------

    #[test]
    fn native_scope_threads_repos_and_perms_to_the_minter() {
        // A minter that captures the MintRequest so we can assert repos/perms reached it verbatim.
        struct CapturingMint(Mutex<Option<(Vec<String>, Vec<String>)>>);
        impl ProviderMint for CapturingMint {
            fn mint_scoped(&self, p: &MintRequest) -> Result<ScopedToken, MintError> {
                *self.0.lock().unwrap() = Some((p.repos.clone(), p.perms.clone()));
                Ok(ScopedToken {
                    token: Zeroizing::new(b"ghs_scoped".to_vec()),
                    expires_at: 1_700_003_600,
                })
            }
        }
        let (engine, sink, _rx) = unlocked_engine();
        let cap = std::sync::Arc::new(CapturingMint(Mutex::new(None)));
        // install_provider takes a Box; install a clone-backed capturing minter via Arc indirection.
        struct ArcMint(std::sync::Arc<CapturingMint>);
        impl ProviderMint for ArcMint {
            fn mint_scoped(&self, p: &MintRequest) -> Result<ScopedToken, MintError> {
                self.0.mint_scoped(p)
            }
        }
        engine.install_provider(Box::new(ArcMint(cap.clone())));
        let _ = engine
            .resolve_injection(
                Provider::Github,
                "gh",
                "relay-bearer",
                "",
                "",
                inject::DataPlaneMode::NativeSubtoken,
                vec!["meta".into(), "envctl".into()],
                vec!["checks:write".into(), "contents:read".into()],
                3600,
                &sink,
            )
            .unwrap()
            .expect("minted");
        let seen = cap.0.lock().unwrap().clone().expect("mint_scoped called");
        assert_eq!(seen.0, vec!["meta".to_string(), "envctl".to_string()]);
        assert_eq!(
            seen.1,
            vec!["checks:write".to_string(), "contents:read".to_string()]
        );
    }

    // ---- TASK-0020: mint_github_token (per-call, frozen consumer contract) --------------------

    /// Build an UNLOCKED engine whose `github_transport` is the supplied `FakeTransport`, then enroll
    /// the flat-convention App key (`github-app-private-key` broker-only) + id (`github-app-id`) so
    /// `mint_github_token` finds them. Returns `(engine, sink, rx)`.
    fn unlocked_engine_with_transport(
        status: u16,
        body: &str,
    ) -> (Engine, EventSink, std::sync::mpsc::Receiver<SecretEvent>) {
        let engine = Engine::with_seams(
            paths(),
            Box::new(vault::InMemStore::new()),
            Box::new(FixedClock),
            Box::new(NoUsb),
            Box::new(seam::NoMint),
            Box::new(NullUpstream),
            Box::new(FakeTransport {
                status,
                body: body.to_string(),
            }),
        )
        .expect("with_seams");
        let (sink, rx) = EventSink::channel();
        engine
            .init_vault(
                Zeroizing::new("correct horse battery staple".to_string()),
                None,
                None,
                at_floor(),
                &sink,
            )
            .expect("init_vault");
        engine
            .unlock(
                Unlock::Passphrase(Zeroizing::new("correct horse battery staple".to_string())),
                &sink,
            )
            .expect("unlock");
        (engine, sink, rx)
    }

    fn enroll_github_app(engine: &Engine, sink: &EventSink) {
        engine
            .secret_put(
                SecretMeta {
                    name: GITHUB_APP_KEY_NAME.into(),
                    provider: Provider::Github,
                    note: "test app key".into(),
                    broker_only: true,
                },
                Zeroizing::new(TEST_PEM.as_bytes().to_vec()),
                sink,
            )
            .expect("secret_put app key");
        engine
            .inner
            .store
            .put_meta(GITHUB_APP_ID_META, "4044997")
            .expect("put app id meta");
    }

    #[test]
    fn mint_github_token_happy_path_mints_and_audits_metadata_only() {
        let (engine, sink, rx) = unlocked_engine_with_transport(
            201,
            r#"{"token":"ghs_frozen_contract","expires_at":"2026-06-12T23:00:00Z"}"#,
        );
        enroll_github_app(&engine, &sink);
        let _ = drain(&rx); // discard init/unlock/seed events

        let scoped = engine
            .mint_github_token(
                mint_github::GithubMintParams {
                    installation_id: 12345,
                    repository_ids: vec![10, 4_044_997],
                    permissions: vec!["checks:write".into()],
                    ttl_secs: 3600,
                    api_base: None,
                },
                &sink,
            )
            .expect("mint ok");
        assert_eq!(&*scoped.token, b"ghs_frozen_contract");
        assert_eq!(
            scoped.expires_at,
            chrono::DateTime::parse_from_rfc3339("2026-06-12T23:00:00Z")
                .unwrap()
                .timestamp()
        );
        assert!(scoped.expires_at > 0, "i64 epoch is positive");

        // The audit row + event carry metadata only — NEVER the minted token.
        let events = drain(&rx);
        for e in &events {
            let json = serde_json::to_string(e).unwrap();
            assert!(
                !json.contains("ghs_frozen_contract"),
                "token must never appear in an event: {json}"
            );
        }
        let minted = events
            .iter()
            .any(|e| matches!(e, SecretEvent::RelayMinted { .. }));
        assert!(minted, "a RelayMinted (metadata-only) event was emitted");
    }

    #[test]
    fn mint_github_token_refuses_when_locked() {
        let (engine, sink, _rx) = unlocked_engine_with_transport(201, "{}");
        enroll_github_app(&engine, &sink);
        engine.lock(&sink).expect("lock");
        // Locked vault ⇒ no DEK ⇒ no key ⇒ fail-closed (never a fabricated token).
        // NB: ScopedToken holds a secret and has no Debug, so match the Result directly.
        let err = match engine.mint_github_token(
            mint_github::GithubMintParams {
                installation_id: 1,
                repository_ids: vec![],
                permissions: vec![],
                ttl_secs: 3600,
                api_base: None,
            },
            &sink,
        ) {
            Ok(_) => panic!("locked vault must refuse"),
            Err(e) => e,
        };
        assert!(
            err.downcast_ref::<EngineError>()
                .map(|e| matches!(e, EngineError::Locked))
                .unwrap_or(false),
            "locked refusal is EngineError::Locked, got: {err}"
        );
    }

    #[test]
    fn mint_github_token_refuses_when_key_absent_naming_remediation() {
        // Unlocked, but the App key was never enrolled ⇒ fail-closed naming the enroll remediation.
        let (engine, sink, _rx) = unlocked_engine_with_transport(201, "{}");
        let err = match engine.mint_github_token(
            mint_github::GithubMintParams {
                installation_id: 1,
                repository_ids: vec![],
                permissions: vec![],
                ttl_secs: 3600,
                api_base: None,
            },
            &sink,
        ) {
            Ok(_) => panic!("absent key must refuse"),
            Err(e) => e,
        };
        let msg = err.to_string();
        assert!(
            msg.contains("github-app enroll") && msg.contains("not enrolled"),
            "absent-key error names the remediation, got: {msg}"
        );
    }

    #[test]
    fn mint_github_token_refuses_on_http_error_never_a_token() {
        let (engine, sink, _rx) = unlocked_engine_with_transport(404, r#"{"message":"Not Found"}"#);
        enroll_github_app(&engine, &sink);
        // A non-201 GitHub response ⇒ the per-call minter errors ⇒ we refuse (no token).
        let err = match engine.mint_github_token(
            mint_github::GithubMintParams {
                installation_id: 1,
                repository_ids: vec![10],
                permissions: vec![],
                ttl_secs: 3600,
                api_base: None,
            },
            &sink,
        ) {
            Ok(_) => panic!("http error must refuse"),
            Err(e) => e,
        };
        assert!(
            err.to_string().contains("github mint failed"),
            "surfaces a fail-closed mint error, got: {err}"
        );
    }

    // ---- TASK-0027: revoke_github_token (explicit-token verb) + relay_revoke tie-in -----------

    #[test]
    fn revoke_github_token_dry_run_no_egress() {
        // apply=false ⇒ no DELETE on the wire, audit/event say "dry_run", returns false.
        let (engine, sink, rx) = unlocked_engine_with_transport(204, "");
        let _ = drain(&rx);
        let revoked = engine
            .revoke_github_token(
                Zeroizing::new(b"ghs_secret_token".to_vec()),
                false,
                None,
                &sink,
            )
            .expect("dry-run ok");
        assert!(!revoked, "dry-run never reports a revoke");
        let events = drain(&rx);
        let saw_dry = events.iter().any(|e| {
            matches!(e, SecretEvent::GithubTokenRevoked { outcome, .. } if outcome == "dry_run")
        });
        assert!(saw_dry, "a dry_run GithubTokenRevoked event was emitted");
        for e in &events {
            assert!(
                !serde_json::to_string(e)
                    .unwrap()
                    .contains("ghs_secret_token"),
                "token never appears in an event"
            );
        }
    }

    #[test]
    fn revoke_github_token_apply_204_succeeds_metadata_only() {
        let (engine, sink, rx) = unlocked_engine_with_transport(204, "");
        let _ = drain(&rx);
        let revoked = engine
            .revoke_github_token(
                Zeroizing::new(b"ghs_secret_token".to_vec()),
                true,
                None,
                &sink,
            )
            .expect("204 ⇒ Ok(true)");
        assert!(revoked, "204 reports a successful revoke");
        let events = drain(&rx);
        let saw_revoked = events.iter().any(|e| {
            matches!(e, SecretEvent::GithubTokenRevoked { outcome, .. } if outcome == "revoked")
        });
        assert!(
            saw_revoked,
            "a revoked GithubTokenRevoked event was emitted"
        );
        for e in &events {
            assert!(
                !serde_json::to_string(e)
                    .unwrap()
                    .contains("ghs_secret_token"),
                "token never appears in an event"
            );
        }
    }

    #[test]
    fn revoke_github_token_non_204_is_err_no_false_success() {
        // A non-204 (401) ⇒ Err; never a fabricated success. The token must not appear in the error.
        let (engine, sink, _rx) = unlocked_engine_with_transport(401, r#"{"message":"Bad"}"#);
        let err = match engine.revoke_github_token(
            Zeroizing::new(b"ghs_secret_token".to_vec()),
            true,
            None,
            &sink,
        ) {
            Ok(_) => panic!("non-204 must be an error"),
            Err(e) => e,
        };
        let msg = err.to_string();
        assert!(msg.contains("github revoke failed"), "fail-closed: {msg}");
        assert!(
            !msg.contains("ghs_secret_token"),
            "no token in error: {msg}"
        );
    }

    #[test]
    fn revoke_github_token_locked_vault_fails_closed() {
        let (engine, sink, _rx) = unlocked_engine_with_transport(204, "");
        engine.lock(&sink).expect("lock");
        let err = match engine.revoke_github_token(
            Zeroizing::new(b"ghs_secret_token".to_vec()),
            true,
            None,
            &sink,
        ) {
            Ok(_) => panic!("locked vault must refuse"),
            Err(e) => e,
        };
        assert!(
            err.downcast_ref::<EngineError>()
                .map(|e| matches!(e, EngineError::Locked))
                .unwrap_or(false),
            "locked refusal is EngineError::Locked, got: {err}"
        );
    }

    /// Build an UNLOCKED engine with a method-routing transport (POST→mint, DELETE→revoke), enroll
    /// the App key, mint a native sub-token (which populates the engine's native-token cache for the
    /// relay), then assert `relay_revoke(apply=true)` fires the best-effort DELETE.
    fn unlocked_engine_with_method_transport(
        post_status: u16,
        post_body: &str,
        delete_status: u16,
    ) -> (
        Engine,
        EventSink,
        std::sync::mpsc::Receiver<SecretEvent>,
        Arc<Mutex<Vec<String>>>,
    ) {
        let seen = Arc::new(Mutex::new(Vec::new()));
        // The transport records into a shared Vec we can inspect after the engine drops its ref.
        struct Shared {
            post_status: u16,
            post_body: String,
            delete_status: u16,
            seen: Arc<Mutex<Vec<String>>>,
        }
        impl HttpTransport for Shared {
            fn execute(&self, req: &HttpRequest) -> Result<HttpResponse, TransportError> {
                self.seen.lock().unwrap().push(req.method.to_string());
                let (status, body) = match req.method {
                    "DELETE" => (self.delete_status, String::new()),
                    _ => (self.post_status, self.post_body.clone()),
                };
                Ok(HttpResponse {
                    status,
                    body: body.into_bytes(),
                })
            }
        }
        let engine = Engine::with_seams(
            paths(),
            Box::new(vault::InMemStore::new()),
            Box::new(FixedClock),
            Box::new(NoUsb),
            Box::new(seam::NoMint),
            Box::new(NullUpstream),
            Box::new(Shared {
                post_status,
                post_body: post_body.to_string(),
                delete_status,
                seen: seen.clone(),
            }),
        )
        .expect("with_seams");
        let (sink, rx) = EventSink::channel();
        engine
            .init_vault(
                Zeroizing::new("correct horse battery staple".to_string()),
                None,
                None,
                at_floor(),
                &sink,
            )
            .expect("init_vault");
        engine
            .unlock(
                Unlock::Passphrase(Zeroizing::new("correct horse battery staple".to_string())),
                &sink,
            )
            .expect("unlock");
        (engine, sink, rx, seen)
    }

    /// Install a `GitHubAppMint` minter so a NativeSubtoken resolve_injection mints (and caches the
    /// token), then drive resolve_injection to populate the engine's native-token cache.
    fn mint_native_for_relay(engine: &Engine, sink: &EventSink, relay: &str) {
        let minter = GitHubAppMint::new(
            "4044997",
            12345,
            Zeroizing::new(TEST_PEM.as_bytes().to_vec()),
            FixedClock,
            // The engine's own github_transport is what `revoke_installation_token` uses; the minter
            // needs its OWN transport for the POST. Reuse a small 201-returning fake.
            Mint201,
        );
        engine.install_provider(Box::new(minter));
        engine
            .resolve_injection(
                Provider::Github,
                relay,
                "relay-bearer",
                "",
                "",
                inject::DataPlaneMode::NativeSubtoken,
                vec!["meta".into()],
                vec!["checks:write".into()],
                3600,
                sink,
            )
            .expect("resolve ok")
            .expect("minted injection");
    }

    /// A 201-returning transport for the minter's own POST during `mint_native_for_relay`.
    struct Mint201;
    impl HttpTransport for Mint201 {
        fn execute(&self, _req: &HttpRequest) -> Result<HttpResponse, TransportError> {
            Ok(HttpResponse {
                status: 201,
                body: br#"{"token":"ghs_native_cached","expires_at":"2026-06-12T23:00:00Z"}"#
                    .to_vec(),
            })
        }
    }

    #[test]
    fn relay_revoke_native_tie_in_best_effort_success() {
        // The engine's github_transport returns 204 on DELETE. After a native mint caches the token,
        // relay_revoke(apply=true) fires the best-effort DELETE and emits a revoked event.
        let (engine, sink, rx, seen) = unlocked_engine_with_method_transport(201, "", 204);
        mint_native_for_relay(&engine, &sink, "gh");
        let _ = drain(&rx);
        let n = engine
            .relay_revoke("gh", true, &sink)
            .expect("relay revoke");
        // relay_revoke returns its bearer count (0 here — no bearers stored), and the tie-in fired.
        let _ = n;
        assert!(
            seen.lock().unwrap().iter().any(|m| m == "DELETE"),
            "the best-effort DELETE was fired"
        );
        let events = drain(&rx);
        assert!(
            events.iter().any(|e| matches!(
                e,
                SecretEvent::GithubTokenRevoked { outcome, .. } if outcome == "revoked"
            )),
            "a revoked GithubTokenRevoked event was emitted"
        );
    }

    #[test]
    fn relay_revoke_native_tie_in_best_effort_failure_still_returns() {
        // The DELETE returns 500 — the tie-in SWALLOWS the failure; relay_revoke still succeeds and
        // emits a best_effort_failed event (metadata only).
        let (engine, sink, rx, seen) = unlocked_engine_with_method_transport(201, "", 500);
        mint_native_for_relay(&engine, &sink, "gh");
        let _ = drain(&rx);
        let res = engine.relay_revoke("gh", true, &sink);
        assert!(
            res.is_ok(),
            "relay_revoke still returns Ok despite revoke failure"
        );
        assert!(
            seen.lock().unwrap().iter().any(|m| m == "DELETE"),
            "the best-effort DELETE was attempted"
        );
        let events = drain(&rx);
        assert!(
            events.iter().any(|e| matches!(
                e,
                SecretEvent::GithubTokenRevoked { outcome, .. } if outcome == "best_effort_failed"
            )),
            "a best_effort_failed event was emitted"
        );
    }

    #[test]
    fn relay_revoke_dry_run_no_native_egress() {
        // apply=false ⇒ no DELETE, no native egress (count only).
        let (engine, sink, rx, seen) = unlocked_engine_with_method_transport(201, "", 204);
        mint_native_for_relay(&engine, &sink, "gh");
        let _ = drain(&rx);
        let _ = engine.relay_revoke("gh", false, &sink).expect("dry-run");
        assert!(
            !seen.lock().unwrap().iter().any(|m| m == "DELETE"),
            "dry-run fires no DELETE"
        );
    }

    #[test]
    fn lock_clears_native_token_cache() {
        // After a native mint caches the token, lock() must drop it so a post-lock relay_revoke fires
        // no DELETE (fail-closed: a locked vault holds no live token).
        let (engine, sink, rx, seen) = unlocked_engine_with_method_transport(201, "", 204);
        mint_native_for_relay(&engine, &sink, "gh");
        engine.lock(&sink).expect("lock");
        seen.lock().unwrap().clear();
        // Re-unlock so relay_revoke's own DEK-gated reseal path doesn't trip; the cache is empty.
        engine
            .unlock(
                Unlock::Passphrase(Zeroizing::new("correct horse battery staple".to_string())),
                &sink,
            )
            .expect("unlock");
        let _ = drain(&rx);
        let _ = engine
            .relay_revoke("gh", true, &sink)
            .expect("relay revoke");
        assert!(
            !seen.lock().unwrap().iter().any(|m| m == "DELETE"),
            "lock() cleared the cache ⇒ no DELETE after re-unlock"
        );
    }
}
