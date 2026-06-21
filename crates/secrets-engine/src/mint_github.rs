//! GitHub App installation-token minting — the `provider-github` realization of the
//! [`ProviderMint`](crate::seam::ProviderMint) seam (ADR-0008 S1, ADR-0007).
//!
//! envctl is the **trusted writer's** credential source: the GitHub App private key is sealed
//! in the vault, and this module exchanges it (an RS256 **App-JWT**) for a short-lived, scoped
//! **installation access token** via `POST /app/installations/{id}/access_tokens`. The token
//! is what `flexnetos_github_app` uses to post check-runs / drive the merge gate — replacing the
//! long-lived `PARENT_REPO_PAT` with a per-repo, per-permission, **relay-rotated** credential
//! (the raw GitHub token is ~1h; envctl's relay re-mints + re-injects it on a ≤24h policy — see
//! "TTL truth" below).
//!
//! ## Why it lives behind a seam (and is fully offline-testable)
//! Per envctl's invariants the engine LIB is pure-Rust, non-printing, and pushes all I/O to a
//! `Send + Sync` seam (cf. [`Upstream`](crate::seam::Upstream)). The network call here is the
//! [`HttpTransport`] trait; the daemon (`secretd`) supplies the real reqwest/rustls-on-ring impl
//! that pins the frozen webpki roots (FS-S7). Everything in THIS module — JWT construction,
//! request shaping, response parsing — is pure and unit-tested with a fake transport, so no
//! live GitHub App is needed to prove it correct.
//!
//! ## TTL truth — two layers, two lifetimes (ADR-0008 §B); do NOT conflate them
//! **1. Raw GitHub installation token (provider mechanics).** GitHub fixes the installation-token
//! lifetime at **~1 hour and it is NOT client-configurable**;
//! [`MintRequest::ttl_secs`](crate::seam::MintRequest) is therefore advisory — the authoritative
//! `expires_at` is taken from GitHub's response. The App-JWT itself is the only lifetime we
//! control, and GitHub caps it at 10 minutes; we issue ≤[`MAX_JWT_TTL_SECS`] with the `iat`
//! back-dated 60s for clock-drift tolerance.
//!
//! **2. Relay-rotation policy (the consumer-facing lifetime).** envctl is a *relay*: it holds the
//! long-lived credential ([`RelayPolicy::policy_ttl_secs`](crate::broker::policy::RelayPolicy),
//! 1y/90d) and re-mints + re-injects the scoped token on a **≤24h rotation** (the WIRE bearer is
//! always clamped to ≤24h — see [`broker::policy`](crate::broker::policy)). So a consumer (e.g. a
//! CI job) does NOT receive a one-shot 1h token: it receives a *continuously rotated* credential
//! whose value changes every 24h — a virtual-credit-card model that bounds blast radius and makes
//! a leak fast to detect and short-lived by construction. The ~1h raw GitHub token from layer 1 is
//! an internal implementation detail the relay refreshes underneath the 24h policy.
//!
//! ## Gating (USB / vault presence)
//! This seam holds an *already-unsealed* App key, so minting is structurally gated upstream: the
//! key only leaves the vault when it is **unlocked**, which (per the keyslot model) requires the
//! USB factor to be present. A locked vault ⇒ no key ⇒ no `GitHubAppMint` ⇒ fail-closed.

use crate::broker::Provider;
use crate::seam::{Clock, MintError, MintRequest, ProviderMint, ScopedToken};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs1v15::SigningKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::signature::{SignatureEncoding, Signer};
use rsa::RsaPrivateKey;
use serde::Deserialize;
use sha2::Sha256;
use zeroize::Zeroizing;

/// GitHub's hard cap on the App-JWT lifetime is 10 minutes; we stay safely under it (clock skew).
pub const MAX_JWT_TTL_SECS: i64 = 540;

/// The default GitHub REST base. Overridable (tests / GHES) via [`GitHubAppMint::with_api_base`].
const GITHUB_API_BASE: &str = "https://api.github.com";

/// A minimal, transport-agnostic HTTP request. The fields are exactly what the GitHub call needs;
/// `headers` is an ordered list so a fake transport can assert on it deterministically.
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: &'static str,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// The transport's reply. Only the status + raw body are needed to parse the token response.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("transport error: {0}")]
    Io(String),
}

/// The single I/O seam for the mint path. The daemon supplies a reqwest/rustls-on-ring impl that
/// verifies TLS against the FROZEN webpki roots (never the OS or local CA) — same discipline as
/// [`Upstream`](crate::seam::Upstream). Synchronous: the mint path is request/response, no streaming.
pub trait HttpTransport: Send + Sync {
    fn execute(&self, req: &HttpRequest) -> Result<HttpResponse, TransportError>;
}

/// Forward through a reference (incl. `&dyn HttpTransport`) so the engine can hand a borrowed,
/// boxed transport to the generic `GitHubAppMint::new` without moving or cloning it (TASK-0020).
impl<T: HttpTransport + ?Sized> HttpTransport for &T {
    fn execute(&self, req: &HttpRequest) -> Result<HttpResponse, TransportError> {
        (**self).execute(req)
    }
}

/// The fail-closed default `HttpTransport` for the engine's `github_transport` seam (TASK-0020). A
/// non-daemon build (or a test that does not inject a real transport) has NO egress, so any mint
/// attempt that reaches the network MUST refuse rather than silently succeed: `execute` always
/// returns a fixed `TransportError`. The daemon overrides this with `DaemonHttpTransport`
/// (reqwest/rustls-on-ring, frozen webpki roots); the engine never reaches the wire on its own.
pub struct NoopHttpTransport;

impl HttpTransport for NoopHttpTransport {
    fn execute(&self, _req: &HttpRequest) -> Result<HttpResponse, TransportError> {
        Err(TransportError::Io(
            "no GitHub HTTP transport configured (mint requires the daemon's transport)"
                .to_string(),
        ))
    }
}

/// Scoped parameters for a single GitHub App installation-token mint (TASK-0020). `installation_id`
/// is request-supplied (the daemon builds a per-call minter); `repository_ids` are NUMERIC repo IDs
/// (the consumer contract passes IDs, mutually exclusive with the name-based `repositories` path);
/// `permissions` are `name:access`; `ttl_secs` is advisory (GitHub fixes the RAW token lifetime
/// ~1h — the relay re-mints + re-injects it on a ≤24h rotation; see the module-level "TTL truth").
#[derive(Debug, Clone)]
pub struct GithubMintParams {
    pub installation_id: u64,
    pub repository_ids: Vec<u64>,
    pub permissions: Vec<String>,
    pub ttl_secs: i64,
    /// REST base override (GitHub Enterprise Server / an e2e mock). `None` ⇒ the default
    /// `https://api.github.com`. The engine stays env-free; the daemon fills this from
    /// `ENVCTL_GITHUB_API_BASE` (mirroring the existing relay-native `rebuild_github_provider`
    /// discipline — env reads live in `secretd`, never the pure engine lib).
    pub api_base: Option<String>,
}

/// Build the RS256-signed GitHub **App JWT**. Pure + deterministic given (`app_id`, `now`, key):
/// header `{"alg":"RS256","typ":"JWT"}`, claims `{"iat": now-60, "exp": now+ttl, "iss": app_id}`,
/// base64url-segments, signed over `header.claims` with PKCS#1 v1.5 / SHA-256. `ttl` is clamped to
/// `[1, MAX_JWT_TTL_SECS]`. Accepts the GitHub-issued PKCS#1 (`BEGIN RSA PRIVATE KEY`) PEM, falling
/// back to PKCS#8 (`BEGIN PRIVATE KEY`).
pub fn build_app_jwt(
    app_id: &str,
    now_unix: i64,
    jwt_ttl_secs: i64,
    key_pem: &[u8],
) -> Result<String, MintError> {
    let ttl = jwt_ttl_secs.clamp(1, MAX_JWT_TTL_SECS);
    let iat = now_unix.saturating_sub(60); // back-date for clock drift (GitHub guidance)
    let exp = now_unix.saturating_add(ttl);

    // Fixed header; claims via serde so a non-numeric `iss` can't break out of the JSON.
    const HEADER_JSON: &[u8] = br#"{"alg":"RS256","typ":"JWT"}"#;
    let claims = serde_json::json!({ "iat": iat, "exp": exp, "iss": app_id });
    let claims_json =
        serde_json::to_vec(&claims).map_err(|e| MintError::Other(format!("jwt claims: {e}")))?;

    let signing_input = format!(
        "{}.{}",
        URL_SAFE_NO_PAD.encode(HEADER_JSON),
        URL_SAFE_NO_PAD.encode(&claims_json)
    );
    let sig = rs256_sign(key_pem, signing_input.as_bytes())?;
    Ok(format!("{signing_input}.{}", URL_SAFE_NO_PAD.encode(sig)))
}

/// RS256-sign `msg`, accepting a PKCS#1 or PKCS#8 PEM private key. The key bytes are wiped by the
/// caller's `Zeroizing`; the parsed `RsaPrivateKey` is dropped at function end.
fn rs256_sign(key_pem: &[u8], msg: &[u8]) -> Result<Vec<u8>, MintError> {
    let pem = std::str::from_utf8(key_pem)
        .map_err(|_| MintError::Other("App private key PEM is not valid UTF-8".into()))?;
    let key = RsaPrivateKey::from_pkcs1_pem(pem)
        .or_else(|_| RsaPrivateKey::from_pkcs8_pem(pem))
        .map_err(|e| MintError::Other(format!("App private key is not a valid RSA PEM: {e}")))?;
    let signing_key = SigningKey::<Sha256>::new(key);
    let sig = signing_key
        .try_sign(msg)
        .map_err(|e| MintError::Other(format!("RS256 signing failed: {e}")))?;
    Ok(sig.to_bytes().into_vec())
}

/// A [`ProviderMint`] that mints GitHub App **installation access tokens**.
///
/// Constructed by the daemon AFTER unsealing the App private key from the (unlocked) vault, so it
/// never reaches in from a locked vault. `C`/`T` are the injected clock + transport seams.
pub struct GitHubAppMint<C: Clock, T: HttpTransport> {
    app_id: String,
    installation_id: u64,
    app_key_pem: Zeroizing<Vec<u8>>,
    api_base: String,
    user_agent: String,
    jwt_ttl_secs: i64,
    clock: C,
    transport: T,
}

impl<C: Clock, T: HttpTransport> GitHubAppMint<C, T> {
    /// Build a minter for one installation. `app_id` is the GitHub App ID (or client id); the PEM
    /// is the App private key (kept in `Zeroizing`, never logged).
    pub fn new(
        app_id: impl Into<String>,
        installation_id: u64,
        app_key_pem: Zeroizing<Vec<u8>>,
        clock: C,
        transport: T,
    ) -> Self {
        Self {
            app_id: app_id.into(),
            installation_id,
            app_key_pem,
            api_base: GITHUB_API_BASE.to_string(),
            user_agent: "flexnetos-github-app".to_string(),
            jwt_ttl_secs: MAX_JWT_TTL_SECS,
            clock,
            transport,
        }
    }

    /// Override the REST base (GitHub Enterprise Server, or a test double).
    pub fn with_api_base(mut self, base: impl Into<String>) -> Self {
        self.api_base = base.into();
        self
    }

    /// Override the `User-Agent` (GitHub requires one; defaults to `flexnetos-github-app`).
    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = ua.into();
        self
    }
}

impl<C: Clock, T: HttpTransport> ProviderMint for GitHubAppMint<C, T> {
    fn mint_scoped(&self, p: &MintRequest) -> Result<ScopedToken, MintError> {
        // This minter only speaks GitHub; anything else falls through to the proxy-swap path.
        if !matches!(p.provider, Provider::Github) {
            return Err(MintError::Unsupported);
        }

        let now = self.clock.now().timestamp();
        let jwt = build_app_jwt(&self.app_id, now, self.jwt_ttl_secs, &self.app_key_pem)?;
        let body = build_token_request_body(&p.repos, &p.repo_ids, &p.perms)?;
        let url = format!(
            "{}/app/installations/{}/access_tokens",
            self.api_base, self.installation_id
        );
        let req = HttpRequest {
            method: "POST",
            url,
            headers: vec![
                ("Authorization".into(), format!("Bearer {jwt}")),
                ("Accept".into(), "application/vnd.github+json".into()),
                ("X-GitHub-Api-Version".into(), "2022-11-28".into()),
                ("User-Agent".into(), self.user_agent.clone()),
                ("Content-Type".into(), "application/json".into()),
            ],
            body,
        };

        let resp = self
            .transport
            .execute(&req)
            .map_err(|e| MintError::Other(format!("GitHub transport: {e}")))?;

        // GitHub returns 201 Created on success. Anything else is a failure; the error body never
        // contains a token, so it is safe to surface a truncated snippet for diagnosis.
        if resp.status != 201 {
            let snippet: String = String::from_utf8_lossy(&resp.body)
                .chars()
                .take(200)
                .collect();
            return Err(MintError::Other(format!(
                "GitHub returned {} creating installation token: {snippet}",
                resp.status
            )));
        }
        parse_token_response(&resp.body)
    }
}

/// Build the `DELETE /installation/token` early-revoke request (TASK-0027). GitHub revokes the
/// installation access token **immediately** when this endpoint is called authenticated with the
/// token ITSELF as the bearer (NOT the App-JWT) — it returns **204 No Content** on success. The
/// request is pure + transport-agnostic, mirroring [`GitHubAppMint::mint_scoped`]'s request shaping.
///
/// The `installation_token` appears ONLY in the `Authorization` header; the body is EMPTY. The
/// returned [`HttpRequest`] therefore carries the secret in `headers` — it MUST NEVER be
/// `{:?}`-logged (its `Debug` impl would print the bearer). Callers pass it straight to the
/// transport and drop it.
pub fn build_revoke_request(
    api_base: &str,
    user_agent: &str,
    installation_token: &[u8],
) -> HttpRequest {
    let bearer = format!("Bearer {}", String::from_utf8_lossy(installation_token));
    HttpRequest {
        method: "DELETE",
        url: format!("{api_base}/installation/token"),
        headers: vec![
            ("Authorization".into(), bearer),
            ("Accept".into(), "application/vnd.github+json".into()),
            ("X-GitHub-Api-Version".into(), "2022-11-28".into()),
            ("User-Agent".into(), user_agent.to_string()),
        ],
        body: Vec::new(),
    }
}

/// Drive an installation-token early-revoke over the supplied [`HttpTransport`] (TASK-0027). On
/// GitHub's **204 No Content** success ⇒ `Ok(())`; a transport error OR any non-204 status ⇒
/// `Err(MintError::Other(..))` carrying a ≤200-char body snippet (the revoke error body never
/// contains a token, so the snippet is safe — mirrors the mint path's diagnostic). The token is
/// supplied to [`build_revoke_request`] as the bearer ONLY; it never enters the returned error.
pub fn revoke_installation_token<T: HttpTransport + ?Sized>(
    transport: &T,
    api_base: &str,
    user_agent: &str,
    installation_token: &[u8],
) -> Result<(), MintError> {
    let req = build_revoke_request(api_base, user_agent, installation_token);
    let resp = transport
        .execute(&req)
        .map_err(|e| MintError::Other(format!("GitHub transport: {e}")))?;
    if resp.status == 204 {
        return Ok(());
    }
    // Non-204: surface a truncated snippet for diagnosis. The revoke error body never carries a
    // token, so this is safe (same discipline as the mint path).
    let snippet: String = String::from_utf8_lossy(&resp.body)
        .chars()
        .take(200)
        .collect();
    Err(MintError::Other(format!(
        "GitHub returned {} revoking installation token: {snippet}",
        resp.status
    )))
}

/// Shape the `create installation access token` request body. `repositories` (repo names),
/// `repository_ids` (NUMERIC ids), and `permissions` are each omitted when empty (⇒ the
/// installation's full default scope). Each permission is `"name:access"` (e.g. `"checks:write"`);
/// a bare `"name"` defaults to `read`.
///
/// TASK-0020: `repository_ids` and `repositories` are MUTUALLY EXCLUSIVE on the GitHub endpoint —
/// sending both is a 422. They are fail-closed REJECTED here if both are non-empty; the
/// `mint-github` consumer path sets only `repository_ids`, the relay-native path only `repos`.
fn build_token_request_body(
    repos: &[String],
    repo_ids: &[u64],
    perms: &[String],
) -> Result<Vec<u8>, MintError> {
    let mut map = serde_json::Map::new();
    if !repos.is_empty() && !repo_ids.is_empty() {
        return Err(MintError::Other(
            "repositories (names) and repository_ids are mutually exclusive (GitHub 422)".into(),
        ));
    }
    if !repos.is_empty() {
        map.insert("repositories".into(), serde_json::json!(repos));
    }
    if !repo_ids.is_empty() {
        // Emit a JSON array of INTEGERS (not strings) — GitHub requires numeric repository_ids.
        map.insert("repository_ids".into(), serde_json::json!(repo_ids));
    }
    if !perms.is_empty() {
        let mut perm_obj = serde_json::Map::new();
        for p in perms {
            let (name, access) = match p.split_once(':') {
                Some((n, a)) => (n.trim(), a.trim()),
                None => (p.trim(), "read"),
            };
            if name.is_empty() {
                return Err(MintError::Other(format!("empty permission name in '{p}'")));
            }
            perm_obj.insert(
                name.to_string(),
                serde_json::Value::String(access.to_string()),
            );
        }
        map.insert("permissions".into(), serde_json::Value::Object(perm_obj));
    }
    serde_json::to_vec(&serde_json::Value::Object(map))
        .map_err(|e| MintError::Other(format!("token request body: {e}")))
}

/// Parse GitHub's success body into a [`ScopedToken`]. The token is moved straight into
/// `Zeroizing` so the secret has a single owner that wipes on drop; `expires_at` is GitHub's
/// authoritative RFC-3339 timestamp (≈1h out), converted to epoch seconds.
fn parse_token_response(body: &[u8]) -> Result<ScopedToken, MintError> {
    #[derive(Deserialize)]
    struct Resp {
        token: String,
        expires_at: String,
    }
    let r: Resp = serde_json::from_slice(body)
        .map_err(|e| MintError::Other(format!("malformed token response: {e}")))?;
    let expires_at = chrono::DateTime::parse_from_rfc3339(&r.expires_at)
        .map_err(|e| MintError::Other(format!("bad expires_at '{}': {e}", r.expires_at)))?
        .timestamp();
    Ok(ScopedToken {
        token: Zeroizing::new(r.token.into_bytes()),
        expires_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::pkcs1v15::{Signature, VerifyingKey};
    use rsa::signature::Verifier;
    use std::sync::Mutex;

    // ── TEST-ONLY throwaway key (1024-bit, weak BY DESIGN). Generated locally with
    //    `openssl genrsa -traditional 1024`; NEVER a real credential. PKCS#1 is GitHub's
    //    download format; the PKCS#8 form is the SAME key (asserted by `pkcs8_form_also_parses`).
    const TEST_PKCS1_PEM: &str = r#"-----BEGIN RSA PRIVATE KEY-----
MIICXgIBAAKBgQDw1EvUY2q80CzzraBZxIBLq1xjF9Eu5PsEseAd2bD+oJo4QQkI
pGycm26vJalBiW/rdzcSPaxPUT7KgH1IeftkUL0pbDG6nN08MgJM0/LjVKx3fK5A
2Lq+CCh+eHfRGxcX8haBzWcwi4tfb90/7Vi9CGh7IXyyMTWLNW/mBVoH8wIDAQAB
AoGBAMSPYbzdz9Z/ytCwm7noyhX4rRUr8U3nEoIIdDWo4e9RQc48NpVZLlS8ACDw
Ci81b6WtzcMTlzm9xBQfvyGSff0S/cCPAWEfGNItWOg5jeLSNftDVh4yM06BPEOI
f+FwkGPiQYtCnhSXLhQq0ClODymjHyW+M7MBf8iyqnd8bnUhAkEA/q8Z5C7YQSFq
IbywMegUkmCykiX8oCrvykg8i5oOjZXhIp/hnxv6jYynZd0PV1oOtbVTuvEve8kr
Cj+84GCPKQJBAPIS3i9C1VaaecCoSlnSY6FHWXmbLsm4wqXGbcyS0m4tQclIXfsd
uDO4AUTu6Xc893Xfa3M/4Jpl7Fs5TReVbbsCQQCUFIlQVDBmxh/oV8Z2bgMwDMsn
ELEvC2f6zD9vx/Y4OnH5aM6NbX4juSlHn92go3s0CacSZdN+/LtqrR6Ls3jpAkBC
/DOdUlokf9SHGkqQtmY5X7wDqYx153l9U/5YKJywPjfBEhRng57QOO+o+o+CHk2/
wVZDav6k2uVfjOinSQM3AkEApokk6NycDKY657zkXPtlhKBsvyxfVW+evW9XjoHi
EnHNytN8c6NOpZMjmzxgSUoOpAI4OVMIH00OvKHIIpvN0w==
-----END RSA PRIVATE KEY-----"#;

    const TEST_PKCS8_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MIICeAIBADANBgkqhkiG9w0BAQEFAASCAmIwggJeAgEAAoGBAPDUS9RjarzQLPOt
oFnEgEurXGMX0S7k+wSx4B3ZsP6gmjhBCQikbJybbq8lqUGJb+t3NxI9rE9RPsqA
fUh5+2RQvSlsMbqc3TwyAkzT8uNUrHd8rkDYur4IKH54d9EbFxfyFoHNZzCLi19v
3T/tWL0IaHshfLIxNYs1b+YFWgfzAgMBAAECgYEAxI9hvN3P1n/K0LCbuejKFfit
FSvxTecSggh0Najh71FBzjw2lVkuVLwAIPAKLzVvpa3NwxOXOb3EFB+/IZJ9/RL9
wI8BYR8Y0i1Y6DmN4tI1+0NWHjIzToE8Q4h/4XCQY+JBi0KeFJcuFCrQKU4PKaMf
Jb4zswF/yLKqd3xudSECQQD+rxnkLthBIWohvLAx6BSSYLKSJfygKu/KSDyLmg6N
leEin+GfG/qNjKdl3Q9XWg61tVO68S97ySsKP7zgYI8pAkEA8hLeL0LVVpp5wKhK
WdJjoUdZeZsuybjCpcZtzJLSbi1ByUhd+x24M7gBRO7pdzz3dd9rcz/gmmXsWzlN
F5VtuwJBAJQUiVBUMGbGH+hXxnZuAzAMyycQsS8LZ/rMP2/H9jg6cflozo1tfiO5
KUef3aCjezQJpxJl0378u2qtHouzeOkCQEL8M51SWiR/1IcaSpC2ZjlfvAOpjHXn
eX1T/lgonLA+N8ESFGeDntA476j6j4IeTb/BVkNq/qTa5V+M6KdJAzcCQQCmiSTo
3JwMpjrnvORc+2WEoGy/LF9Vb569b1eOgeIScc3K03xzo06lkyObPGBJSg6kAjg5
UwgfTQ68ocgim83T
-----END PRIVATE KEY-----"#;

    /// A fixed clock for deterministic JWT timestamps.
    struct FixedClock(i64);
    impl Clock for FixedClock {
        fn now(&self) -> chrono::DateTime<chrono::Utc> {
            chrono::DateTime::from_timestamp(self.0, 0).expect("valid ts")
        }
        fn boottime_ms(&self) -> i64 {
            0
        }
    }

    /// Captures the request and replays a canned response — no network.
    struct FakeTransport {
        response: HttpResponse,
        seen: Mutex<Option<HttpRequest>>,
    }
    impl FakeTransport {
        fn new(status: u16, body: &str) -> Self {
            Self {
                response: HttpResponse {
                    status,
                    body: body.as_bytes().to_vec(),
                },
                seen: Mutex::new(None),
            }
        }
        fn captured(&self) -> HttpRequest {
            self.seen
                .lock()
                .unwrap()
                .clone()
                .expect("a request was made")
        }
    }
    impl HttpTransport for FakeTransport {
        fn execute(&self, req: &HttpRequest) -> Result<HttpResponse, TransportError> {
            *self.seen.lock().unwrap() = Some(req.clone());
            Ok(self.response.clone())
        }
    }

    fn header_value<'a>(req: &'a HttpRequest, name: &str) -> Option<&'a str> {
        req.headers
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    fn decode_segment(seg: &str) -> serde_json::Value {
        let bytes = URL_SAFE_NO_PAD.decode(seg).expect("base64url segment");
        serde_json::from_slice(&bytes).expect("json segment")
    }

    #[test]
    fn app_jwt_has_correct_structure_and_signature_verifies() {
        let now = 1_700_000_000;
        let jwt = build_app_jwt("12345", now, MAX_JWT_TTL_SECS, TEST_PKCS1_PEM.as_bytes()).unwrap();
        let parts: Vec<&str> = jwt.split('.').collect();
        assert_eq!(parts.len(), 3, "header.claims.signature");

        let header = decode_segment(parts[0]);
        assert_eq!(header["alg"], "RS256");
        assert_eq!(header["typ"], "JWT");

        let claims = decode_segment(parts[1]);
        assert_eq!(claims["iss"], "12345");
        assert_eq!(claims["iat"], now - 60, "iat is back-dated 60s");
        assert_eq!(claims["exp"], now + MAX_JWT_TTL_SECS);

        // The RS256 signature must verify against the public half of the test key.
        let priv_key = RsaPrivateKey::from_pkcs1_pem(TEST_PKCS1_PEM).unwrap();
        let vk = VerifyingKey::<Sha256>::new(priv_key.to_public_key());
        let signing_input = format!("{}.{}", parts[0], parts[1]);
        let sig_bytes = URL_SAFE_NO_PAD.decode(parts[2]).unwrap();
        let sig = Signature::try_from(sig_bytes.as_slice()).unwrap();
        vk.verify(signing_input.as_bytes(), &sig)
            .expect("signature verifies");
    }

    #[test]
    fn jwt_ttl_is_clamped_to_github_cap() {
        let now = 1_700_000_000;
        // Over the cap → clamped down.
        let big = build_app_jwt("1", now, 99_999, TEST_PKCS1_PEM.as_bytes()).unwrap();
        let exp = decode_segment(big.split('.').nth(1).unwrap())["exp"]
            .as_i64()
            .unwrap();
        assert_eq!(exp, now + MAX_JWT_TTL_SECS);
        // Non-positive → clamped up to 1 (never a dead/negative window).
        let tiny = build_app_jwt("1", now, 0, TEST_PKCS1_PEM.as_bytes()).unwrap();
        let exp = decode_segment(tiny.split('.').nth(1).unwrap())["exp"]
            .as_i64()
            .unwrap();
        assert_eq!(exp, now + 1);
    }

    #[test]
    fn pkcs8_form_also_parses_and_signs() {
        // GitHub ships PKCS#1, but accept PKCS#8 too; the SAME key must produce an identical sig.
        let now = 1_700_000_000;
        let a = build_app_jwt("1", now, 300, TEST_PKCS1_PEM.as_bytes()).unwrap();
        let b = build_app_jwt("1", now, 300, TEST_PKCS8_PEM.as_bytes()).unwrap();
        assert_eq!(
            a, b,
            "PKCS#1 and PKCS#8 of one key sign identically (RS256 is deterministic)"
        );
    }

    fn github_minter(fake: FakeTransport) -> GitHubAppMint<FixedClock, FakeTransport> {
        GitHubAppMint::new(
            "42",
            99,
            Zeroizing::new(TEST_PKCS1_PEM.as_bytes().to_vec()),
            FixedClock(1_700_000_000),
            fake,
        )
        .with_api_base("https://gh.test")
    }

    #[test]
    fn mint_builds_correct_request_and_parses_token() {
        let fake = FakeTransport::new(
            201,
            r#"{"token":"ghs_exampletoken","expires_at":"2026-06-12T23:00:00Z","permissions":{"checks":"write"}}"#,
        );
        let minter = github_minter(fake);
        let req = MintRequest {
            provider: Provider::Github,
            repos: vec!["meta".into()],
            repo_ids: vec![],
            perms: vec!["checks:write".into(), "contents:read".into()],
            ttl_secs: 3600,
        };
        let tok = minter.mint_scoped(&req).expect("mint succeeds");
        assert_eq!(&*tok.token, b"ghs_exampletoken");
        assert_eq!(
            tok.expires_at,
            chrono::DateTime::parse_from_rfc3339("2026-06-12T23:00:00Z")
                .unwrap()
                .timestamp()
        );

        // Verify the wire request the transport saw.
        let sent = minter.transport.captured();
        assert_eq!(sent.method, "POST");
        assert_eq!(
            sent.url,
            "https://gh.test/app/installations/99/access_tokens"
        );
        assert!(header_value(&sent, "Authorization")
            .unwrap()
            .starts_with("Bearer "));
        assert_eq!(
            header_value(&sent, "Accept"),
            Some("application/vnd.github+json")
        );
        assert_eq!(
            header_value(&sent, "X-GitHub-Api-Version"),
            Some("2022-11-28")
        );
        assert_eq!(
            header_value(&sent, "User-Agent"),
            Some("flexnetos-github-app")
        );

        let body: serde_json::Value = serde_json::from_slice(&sent.body).unwrap();
        assert_eq!(body["repositories"], serde_json::json!(["meta"]));
        assert_eq!(body["permissions"]["checks"], "write");
        assert_eq!(body["permissions"]["contents"], "read");
    }

    #[test]
    fn bare_permission_defaults_to_read_and_empty_scope_is_omitted() {
        let fake = FakeTransport::new(
            201,
            r#"{"token":"ghs_x","expires_at":"2026-06-12T23:00:00Z"}"#,
        );
        let minter = github_minter(fake);
        let req = MintRequest {
            provider: Provider::Github,
            repos: vec![],
            repo_ids: vec![],
            perms: vec!["metadata".into()],
            ttl_secs: 0,
        };
        minter.mint_scoped(&req).unwrap();
        let body: serde_json::Value =
            serde_json::from_slice(&minter.transport.captured().body).unwrap();
        assert_eq!(body["permissions"]["metadata"], "read");
        assert!(body.get("repositories").is_none(), "empty repos omitted");
    }

    #[test]
    fn repository_ids_emit_numeric_array_in_body() {
        // TASK-0020: the `mint-github` consumer path scopes by NUMERIC repo IDs. The body must carry
        // `repository_ids` as a JSON ARRAY OF INTEGERS (never strings), and NOT a `repositories` key.
        let fake = FakeTransport::new(
            201,
            r#"{"token":"ghs_byid","expires_at":"2026-06-12T23:00:00Z"}"#,
        );
        let minter = github_minter(fake);
        let req = MintRequest {
            provider: Provider::Github,
            repos: vec![],
            repo_ids: vec![10, 4044997],
            perms: vec!["checks:write".into()],
            ttl_secs: 3600,
        };
        minter.mint_scoped(&req).expect("mint succeeds");
        let body: serde_json::Value =
            serde_json::from_slice(&minter.transport.captured().body).unwrap();
        assert_eq!(body["repository_ids"], serde_json::json!([10, 4_044_997]));
        assert!(
            body["repository_ids"][0].is_i64() || body["repository_ids"][0].is_u64(),
            "ids are numbers, not strings"
        );
        assert!(
            body.get("repositories").is_none(),
            "name-based repositories key omitted when minting by id"
        );
    }

    #[test]
    fn repositories_and_repository_ids_are_mutually_exclusive() {
        // Sending BOTH `repositories` and `repository_ids` is a GitHub 422 — the body builder must
        // refuse fail-closed (never construct a doomed request) BEFORE any network call.
        let fake = FakeTransport::new(201, "{}");
        let minter = github_minter(fake);
        let req = MintRequest {
            provider: Provider::Github,
            repos: vec!["meta".into()],
            repo_ids: vec![10],
            perms: vec![],
            ttl_secs: 60,
        };
        assert!(matches!(
            minter.mint_scoped(&req),
            Err(MintError::Other(ref m)) if m.contains("mutually exclusive")
        ));
    }

    #[test]
    fn non_github_provider_is_unsupported() {
        let minter = github_minter(FakeTransport::new(201, "{}"));
        let req = MintRequest {
            provider: Provider::Openai,
            repos: vec![],
            repo_ids: vec![],
            perms: vec![],
            ttl_secs: 60,
        };
        assert!(matches!(
            minter.mint_scoped(&req),
            Err(MintError::Unsupported)
        ));
    }

    #[test]
    fn http_error_status_is_surfaced() {
        let minter = github_minter(FakeTransport::new(404, r#"{"message":"Not Found"}"#));
        let req = MintRequest {
            provider: Provider::Github,
            repos: vec![],
            repo_ids: vec![],
            perms: vec![],
            ttl_secs: 60,
        };
        // NB: ScopedToken has no Debug (it holds a secret), so match the Result directly.
        let result = minter.mint_scoped(&req);
        assert!(matches!(result, Err(MintError::Other(ref m)) if m.contains("404")));
    }

    #[test]
    fn malformed_success_body_is_error() {
        let minter = github_minter(FakeTransport::new(201, r#"{"not":"a token"}"#));
        let req = MintRequest {
            provider: Provider::Github,
            repos: vec![],
            repo_ids: vec![],
            perms: vec![],
            ttl_secs: 60,
        };
        assert!(matches!(minter.mint_scoped(&req), Err(MintError::Other(_))));
    }

    // ---- TASK-0027: DELETE /installation/token early-revoke ----------------------------------

    const REVOKE_TOKEN: &[u8] = b"ghs_revoke_me_now";

    #[test]
    fn revoke_builds_correct_delete_request() {
        // The revoke request is a DELETE to {base}/installation/token with the token as the bearer,
        // the standard Accept / api-version / user-agent headers, and an EMPTY body.
        let req = build_revoke_request("https://gh.test", "flexnetos-github-app", REVOKE_TOKEN);
        assert_eq!(req.method, "DELETE");
        assert_eq!(req.url, "https://gh.test/installation/token");
        assert_eq!(
            header_value(&req, "Authorization"),
            Some("Bearer ghs_revoke_me_now")
        );
        assert_eq!(
            header_value(&req, "Accept"),
            Some("application/vnd.github+json")
        );
        assert_eq!(
            header_value(&req, "X-GitHub-Api-Version"),
            Some("2022-11-28")
        );
        assert_eq!(
            header_value(&req, "User-Agent"),
            Some("flexnetos-github-app")
        );
        assert!(req.body.is_empty(), "DELETE carries no body");
    }

    #[test]
    fn revoke_204_is_success() {
        let fake = FakeTransport::new(204, "");
        let out = revoke_installation_token(
            &fake,
            "https://gh.test",
            "flexnetos-github-app",
            REVOKE_TOKEN,
        );
        assert!(out.is_ok(), "204 No Content ⇒ Ok(())");
        // The token reached the wire only as the auth header bearer.
        let sent = fake.captured();
        assert_eq!(sent.method, "DELETE");
        assert_eq!(
            header_value(&sent, "Authorization"),
            Some("Bearer ghs_revoke_me_now")
        );
    }

    #[test]
    fn revoke_non_204_is_failure_without_token() {
        // 401 (or any non-204) ⇒ Err; the error must NEVER echo the token.
        let fake = FakeTransport::new(401, r#"{"message":"Bad credentials"}"#);
        let err = revoke_installation_token(
            &fake,
            "https://gh.test",
            "flexnetos-github-app",
            REVOKE_TOKEN,
        )
        .expect_err("non-204 is an error");
        let msg = err.to_string();
        assert!(msg.contains("401"), "status surfaced: {msg}");
        assert!(
            !msg.contains("ghs_revoke_me_now"),
            "token must never appear in the error: {msg}"
        );
    }

    #[test]
    fn revoke_transport_error_is_failure() {
        // A transport-layer error ⇒ Err (never a false success that a 204 implies).
        let err = revoke_installation_token(
            &NoopHttpTransport,
            "https://gh.test",
            "flexnetos-github-app",
            REVOKE_TOKEN,
        )
        .expect_err("transport error is an error");
        assert!(matches!(err, MintError::Other(ref m) if m.contains("transport")));
    }

    #[test]
    fn revoke_token_only_in_auth_header_not_in_error() {
        // Even on the failure path the token lives ONLY in the request's auth header — it is never
        // copied into the MintError Display (the request itself must never be {:?}-logged).
        let fake = FakeTransport::new(422, r#"{"message":"Unprocessable"}"#);
        let err = revoke_installation_token(
            &fake,
            "https://gh.test",
            "flexnetos-github-app",
            REVOKE_TOKEN,
        )
        .expect_err("422 is an error");
        assert!(!err.to_string().contains("ghs_revoke_me_now"));
        let sent = fake.captured();
        // The token is exactly once, in the Authorization header — never the url or body.
        assert!(!sent.url.contains("ghs_revoke_me_now"));
        assert!(sent.body.is_empty());
    }
}
