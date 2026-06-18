//! Relay policies (the "virtual card" + its limits) and the single TTL choke point.
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    Anthropic,
    Openai,
    Github,
    Generic,
}

/// Canonical upstream host allowlist per provider (HF-11) — the swap REFUSES any other host, so a
/// relay can never be re-pointed at an attacker-controlled endpoint. This is a hard-coded, frozen
/// fence: even a tampered `host_allow` that lists an attacker host is still rejected because the
/// host must ALSO be in this provider-pinned set. `Generic` returns the empty slice, so a `Generic`
/// relay is `UpstreamNotAllowed` by default (default-deny posture) unless the daemon supplies a
/// per-policy upstream above this trait.
pub fn canonical_upstreams(p: Provider) -> &'static [&'static str] {
    match p {
        Provider::Anthropic => &["api.anthropic.com"],
        Provider::Openai => &["api.openai.com"],
        Provider::Github => &["api.github.com", "uploads.github.com"],
        Provider::Generic => &[],
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    Get,
    Head,
    Post,
    Put,
    Patch,
    Delete,
    Connect,
    Options,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelayKind {
    Named,
    Ephemeral,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum SwapMode {
    BaseUrlRepoint {
        upstream_base: String,
    },
    ProxyMitm,
    /// Native scoped sub-token mint (G2): mint a provider-native short-lived token (GitHub App
    /// installation token) instead of repointing/proxying. `repos` / `perms` scope the minted token
    /// (empty `perms` ⇒ the installation's full default scope; empty `repos` ⇒ all installed repos).
    /// `ttl_secs` is advisory — GitHub fixes the installation-token TTL (~1h) and we surface its
    /// authoritative `expires_at`. Back-compat: deserializing an old `{ "ttl_secs": N }` yields empty
    /// `repos`/`perms` (serde `default`), the pre-G2 behavior.
    NativeSubToken {
        ttl_secs: i64,
        #[serde(default)]
        repos: Vec<String>,
        #[serde(default)]
        perms: Vec<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelayId(pub String);

/// A relay policy. The `policy_ttl_secs` is the long lifetime (1y/90d); the WIRE bearer minted
/// under it is always clamped to `<=24h`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelayPolicy {
    pub relay_id: String,
    pub kind: RelayKind,
    pub provider: Provider,
    pub secret_name: String,
    pub swap: SwapMode,
    pub host_allow: Vec<String>,
    pub path_allow: Vec<String>,
    pub method_allow: Vec<Method>,
    pub policy_ttl_secs: i64,
    pub rate_per_min: Option<u32>,
    /// Max TOTAL request COUNT over the bearer's life (the request budget). Distinct scale from the
    /// byte budget below: one scalar cannot sensibly cap both a request count and a byte count
    /// (1_000_000 means "1M requests" here, not "1 MB of egress"). `None` => no request cap.
    #[serde(default, alias = "quota_total")]
    pub quota_total_requests: Option<u64>,
    /// Max TOTAL egress BYTES over the bearer's life (the byte budget). `None` => no byte cap.
    #[serde(default)]
    pub quota_total_bytes: Option<u64>,
    pub enabled: bool,
    pub revoked: bool,
}

/// The minted wire bearer returned to clients. Only its hash is persisted; `raw` never touches
/// disk and is zeroized on drop.
pub struct Bearer {
    pub relay_id: String,
    pub token_id: String,
    pub raw: Zeroizing<String>,
    pub expires_at: String,
}

pub const MAX_BEARER_TTL_SECS: i64 = 24 * 60 * 60;

/// The single TTL choke point (HF-15): clamps the requested TTL against the policy TTL AND the
/// 24h ceiling, saturating (never wraps), and refuses a dead/negative TTL (FS-S3). Returns the
/// absolute `expires_at` epoch-seconds, or `None` to refuse.
///
/// `requested_ttl_secs == 0` means "unspecified" — the caller (e.g. `env-ctl run`, which mints with
/// `ttl_secs: 0`) is deferring entirely to the policy ceiling + the 24h cap. Treating 0 as a literal
/// `min()` operand would force the result to 0 and refuse every such mint, so 0 falls back to the 24h
/// ceiling here. A *negative* requested TTL stays malformed and is still refused (FS-S3); a
/// non-positive `policy_ttl_secs` (expired/disabled policy) also still refuses.
pub fn clamp_ttl(now: i64, policy_ttl_secs: i64, requested_ttl_secs: i64) -> Option<i64> {
    let requested = if requested_ttl_secs == 0 {
        MAX_BEARER_TTL_SECS
    } else {
        requested_ttl_secs
    };
    let ttl = requested.min(policy_ttl_secs).min(MAX_BEARER_TTL_SECS);
    if ttl <= 0 {
        return None;
    }
    Some(now.saturating_add(ttl))
}
