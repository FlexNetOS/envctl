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
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine as _;
use const_num_traits::Ct;
use fixed_bigint::FixedUInt;
use pkcs1::der::asn1::UintRef;
use ring::rand::{SecureRandom, SystemRandom};
use ring::signature::{RsaPublicKeyComponents, RSA_PKCS1_2048_8192_SHA256};
use rsa_ct::modmath_support::ModMathParams;
use rsa_ct::pkcs1v15::GenericSigningKey;
use rsa_ct::traits::FixedWidthUnsignedInt;
use rsa_ct::{GenericRsaPrivateKey, GenericRsaPublicKey};
use serde::Deserialize;
use sha2_rs256::Sha256;
use std::fmt;
use zeroize::{Zeroize, Zeroizing};

/// GitHub's hard cap on the App-JWT lifetime is 10 minutes; we stay safely under it (clock skew).
pub const MAX_JWT_TTL_SECS: i64 = 540;

/// The default GitHub REST base. Overridable (tests / GHES) via [`GitHubAppMint::with_api_base`].
const GITHUB_API_BASE: &str = "https://api.github.com";

/// GitHub requires RSA keys of at least 2048 bits. The 8192-bit ceiling matches ring's independent
/// verifier and prevents an operator-supplied key from turning JWT minting into an unbounded CPU or
/// stack denial of service.
const MIN_RSA_MODULUS_BITS: usize = 2048;
const MAX_RSA_MODULUS_BITS: usize = 8192;
const MAX_RSA_PUBLIC_EXPONENT: u64 = (1 << 33) - 1;

/// Fixed-width Ct carriers deliberately split at 4096 bits. A shorter modulus in either carrier
/// retains its true encoded signature length; the carrier width only fixes the private ladder's
/// iteration count. The 4096 carrier keeps ordinary GitHub RSA-2048/3072/4096 keys inexpensive,
/// while the separate 8192 carrier preserves the full accepted upper range.
type Rsa4096 = FixedUInt<u64, 64, Ct>;
type Rsa8192 = FixedUInt<u64, 128, Ct>;

/// Adapt ring's already-sanctioned OS RNG to rsa_heapless's fallible rand_core 0.10 interface.
/// Errors intentionally carry no provider internals or key material.
#[derive(Debug)]
struct RsaRngError;

impl fmt::Display for RsaRngError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("system RNG failed")
    }
}

impl std::error::Error for RsaRngError {}

struct RingRsaRng(SystemRandom);

impl rsa_ct::rand_core::TryRng for RingRsaRng {
    type Error = RsaRngError;

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        let mut bytes = [0_u8; 4];
        self.try_fill_bytes(&mut bytes)?;
        Ok(u32::from_ne_bytes(bytes))
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        let mut bytes = [0_u8; 8];
        self.try_fill_bytes(&mut bytes)?;
        Ok(u64::from_ne_bytes(bytes))
    }

    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
        self.0.fill(dst).map_err(|_| RsaRngError)
    }
}

impl rsa_ct::rand_core::TryCryptoRng for RingRsaRng {}

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

/// Decode one exact PEM block without copying its private-key text into an ordinary allocation.
/// The temporary base64 and DER buffers are both wiped on drop.
fn decode_private_key_pem(pem: &str, label: &str) -> Result<Option<Zeroizing<Vec<u8>>>, MintError> {
    let begin = format!("-----BEGIN {label}-----");
    let end = format!("-----END {label}-----");
    let mut lines = pem.lines().filter(|line| !line.trim().is_empty());
    let Some(first) = lines.next() else {
        return Ok(None);
    };
    if first.trim() != begin {
        return Ok(None);
    }

    let mut encoded = Zeroizing::new(String::new());
    let mut ended = false;
    for line in &mut lines {
        let line = line.trim();
        if line == end {
            ended = true;
            break;
        }
        if line.starts_with("-----") || line.is_empty() {
            return Err(MintError::Other(
                "App private key contains a malformed PEM block".into(),
            ));
        }
        encoded.push_str(line);
    }
    if !ended || lines.any(|line| !line.trim().is_empty()) {
        return Err(MintError::Other(
            "App private key must contain exactly one complete PEM block".into(),
        ));
    }
    let der = STANDARD
        .decode(encoded.as_bytes())
        .map_err(|_| MintError::Other("App private key PEM body is not valid base64".into()))?;
    Ok(Some(Zeroizing::new(der)))
}

#[derive(Clone, Copy)]
enum RsaPrivateKeyFormat {
    Pkcs1,
    Pkcs8,
}

/// Parse a reference-only PKCS#1 key. For PKCS#8, require the exact rsaEncryption algorithm ID and
/// parse its inner PKCS#1 octets. No private component is copied out of the caller's zeroizing DER
/// buffer here. Multi-prime keys are rejected: the fixed-width signer intentionally implements the
/// ordinary two-prime GitHub key contract only.
fn parse_rsa_private_key(
    der: &[u8],
    format: RsaPrivateKeyFormat,
) -> Result<pkcs1::RsaPrivateKey<'_>, MintError> {
    let key = match format {
        RsaPrivateKeyFormat::Pkcs1 => pkcs1::RsaPrivateKey::try_from(der),
        RsaPrivateKeyFormat::Pkcs8 => {
            let info = pkcs8::PrivateKeyInfoRef::try_from(der).map_err(|_| {
                MintError::Other("App private key is not valid PKCS#8 RSA DER".into())
            })?;
            if info.algorithm != pkcs1::ALGORITHM_ID {
                return Err(MintError::Other(
                    "App private key PKCS#8 algorithm is not RSA".into(),
                ));
            }
            pkcs1::RsaPrivateKey::try_from(info.private_key.as_bytes())
        }
    }
    .map_err(|_| MintError::Other("App private key is not valid two-prime RSA DER".into()))?;

    if key.other_prime_infos.is_some() {
        return Err(MintError::Other(
            "multi-prime RSA App private keys are not supported".into(),
        ));
    }
    Ok(key)
}

fn modulus_bit_len(modulus: &[u8]) -> usize {
    let Some((first_nonzero, first)) = modulus.iter().enumerate().find(|(_, byte)| **byte != 0)
    else {
        return 0;
    };
    (modulus.len() - first_nonzero - 1) * 8 + (8 - first.leading_zeros() as usize)
}

fn validate_modulus_bits(modulus: &[u8]) -> Result<usize, MintError> {
    let bits = modulus_bit_len(modulus);
    if !(MIN_RSA_MODULUS_BITS..=MAX_RSA_MODULUS_BITS).contains(&bits) {
        return Err(MintError::Other(format!(
            "App RSA modulus must be {MIN_RSA_MODULUS_BITS}..={MAX_RSA_MODULUS_BITS} bits (got {bits})"
        )));
    }
    if modulus.last().is_none_or(|byte| byte & 1 == 0) {
        return Err(MintError::Other("App RSA modulus must be odd".into()));
    }
    Ok(bits)
}

fn public_exponent_u64(value: UintRef<'_>) -> Result<u64, MintError> {
    let mut exponent = 0_u64;
    for byte in value.as_bytes() {
        exponent = exponent
            .checked_mul(256)
            .and_then(|v| v.checked_add(u64::from(*byte)))
            .ok_or_else(|| MintError::Other("App RSA public exponent is too large".into()))?;
    }
    if !(3..=MAX_RSA_PUBLIC_EXPONENT).contains(&exponent) || exponent & 1 == 0 {
        return Err(MintError::Other(format!(
            "App RSA public exponent must be odd and in 3..={MAX_RSA_PUBLIC_EXPONENT}"
        )));
    }
    Ok(exponent)
}

/// Instantiate one of the two fixed-width Ct backends. The DER buffer remains the authoritative
/// zeroizing owner. The one fixed-width copy of `d` used to construct the key is explicitly wiped
/// after transfer, and the owning GenericRsaPrivateKey wipes its canonical `d` on drop. The modmath
/// backend's Montgomery forms and serialized private-operation values are ZeroizeOnDrop as well.
macro_rules! sign_with_fixed_rsa_carrier {
    ($carrier:ty, $key:expr, $exponent:expr, $msg:expr, $rng:expr) => {{
        let modulus =
            <$carrier as FixedWidthUnsignedInt>::try_from_be_bytes_vartime($key.modulus.as_bytes())
                .map_err(|_| {
                    MintError::Other("App RSA modulus does not fit its fixed carrier".into())
                })?;
        let exponent_bytes = $exponent.to_be_bytes();
        let exponent =
            <$carrier as FixedWidthUnsignedInt>::try_from_be_bytes_vartime(&exponent_bytes)
                .map_err(|_| MintError::Other("App RSA public exponent is invalid".into()))?;
        let params = ModMathParams::<$carrier, Ct>::new(modulus)
            .map_err(|_| MintError::Other("App RSA modulus is invalid".into()))?;
        let public = GenericRsaPublicKey::from_components(modulus, exponent, params)
            .map_err(|_| MintError::Other("App RSA public components are invalid".into()))?;

        let mut private_exponent = <$carrier as FixedWidthUnsignedInt>::try_from_be_bytes_vartime(
            $key.private_exponent.as_bytes(),
        )
        .map_err(|_| MintError::Other("App RSA private exponent is invalid".into()))?;
        let private = GenericRsaPrivateKey::from_public_and_d(public, private_exponent);
        // FixedUInt is Copy by design. Wipe the constructor's source slot immediately; `private`
        // owns the only intentional long-lived fixed-width copy and wipes it in Drop.
        private_exponent.zeroize();

        let signing_key = GenericSigningKey::<Sha256, _, _>::new(private);
        let signature_len = $key.modulus.as_bytes().len();
        let mut encoded_message = vec![0_u8; signature_len];
        let mut signature = vec![0_u8; signature_len];
        signing_key
            .try_sign_with_rng_into($rng, $msg, &mut encoded_message, &mut signature)
            .map_err(|_| MintError::Other("RS256 signing failed".into()))?;
        signature
    }};
}

/// RS256-sign `msg`, accepting a PKCS#1 or PKCS#8 PEM private key. This always uses a fresh
/// RNG-driven base-blinding factor with fixed-width Ct arithmetic, then independently verifies the
/// emitted signature using ring's separate RSA implementation before returning it. The operational
/// `(n,e,d)` tuple is therefore fail-closed even though redundant CRT fields are parsed only for
/// strict two-prime syntax and are intentionally not imported into the private operation.
fn rs256_sign_with_rng<R: rsa_ct::rand_core::TryCryptoRng + ?Sized>(
    key_pem: &[u8],
    msg: &[u8],
    rng: &mut R,
) -> Result<Vec<u8>, MintError> {
    let pem = std::str::from_utf8(key_pem)
        .map_err(|_| MintError::Other("App private key PEM is not valid UTF-8".into()))?;
    let (der, format) = if let Some(der) = decode_private_key_pem(pem, "RSA PRIVATE KEY")? {
        (der, RsaPrivateKeyFormat::Pkcs1)
    } else if let Some(der) = decode_private_key_pem(pem, "PRIVATE KEY")? {
        (der, RsaPrivateKeyFormat::Pkcs8)
    } else {
        return Err(MintError::Other(
            "App private key is not a supported RSA private-key PEM".into(),
        ));
    };

    let key = parse_rsa_private_key(&der, format)?;
    let modulus = key.modulus.as_bytes();
    let modulus_bits = validate_modulus_bits(modulus)?;
    let exponent = public_exponent_u64(key.public_exponent)?;

    let signature = if modulus_bits <= 4096 {
        sign_with_fixed_rsa_carrier!(Rsa4096, key, exponent, msg, rng)
    } else {
        sign_with_fixed_rsa_carrier!(Rsa8192, key, exponent, msg, rng)
    };

    // Independent production verification is intentional defense in depth: rsa_heapless also
    // verifies its private operation, while ring independently checks the final RS256 bytes using
    // only public components and enforces the same 2048..=8192-bit range.
    RsaPublicKeyComponents {
        n: modulus,
        e: key.public_exponent.as_bytes(),
    }
    .verify(&RSA_PKCS1_2048_8192_SHA256, msg, &signature)
    .map_err(|_| MintError::Other("RS256 signature self-verification failed".into()))?;
    Ok(signature)
}

fn rs256_sign(key_pem: &[u8], msg: &[u8]) -> Result<Vec<u8>, MintError> {
    rs256_sign_with_rng(key_pem, msg, &mut RingRsaRng(SystemRandom::new()))
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
    use std::sync::Mutex;

    // ── TEST-ONLY throwaway key (2048-bit). Generated locally with
    //    `openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048`; NEVER a real credential.
    //    PKCS#1 is GitHub's
    //    download format; the PKCS#8 form is the SAME key (asserted by `pkcs8_form_also_parses`).
    const TEST_PKCS1_PEM: &str = r#"-----BEGIN RSA PRIVATE KEY-----
MIIEowIBAAKCAQEA0G8nmHDYnA1m1chZJg9pNMvWW+IdNLP63CdrDfh+orhv6dMP
U/XeVUz/1yRVxM3r0wtmoh0nIi6WbAT9Q58HK9/T6WyVRMSbZlDQWlo+xv2vFCpR
XCTJ2guoF/ND6iPEY3EPP1wWbUls3FVbEm8RHTbfwFx6HqChbuYw5Y0xTXo3Ei8U
j5cUzFCjSk+aY+cdus11Fb1/jN3KaAn8JsrAzwLBX75UVrZUsxmauZLZlwK64P+t
IFH0kKJKMFY+cS1vjaz0erkKhp8Fejzr/RJsg1G7/dCmqQ1FqL9cOie9s/t5rd8G
ZtBH8PimRw0t37Qh7emXVDDgwC6yYTFdar3YuQIDAQABAoIBAA0NNMnFIS8uYKbY
14o8U05oyCizQThsX7Q67KdwjL90NJ/L5U0Q24X0Xx3R6uP6r/5kW54hnLJ1f9pO
OqyqD9kluB+I+tTWSdPPkihiN8Wem4C0AKm4LQKQEcvEIhfOewzuBrIlOktIGn62
gpAmL8hoR/0D3Wq/DLTEycGKBJERDcOZpqROzKuEmrmmE3rm7Q9yXnsGen+9Do3r
nmMaYvvD0uHr0E0ABo14uGgqQaZSeCSBgGe0OzATS4Du+vWJw0JcctEUPghLmaMJ
QPWwbOtDyEHxxD6rmIbXLwP42I1EkS9Xxs2GGuKgSvYoxy4FmaVFWNahNhQqusQ3
TpqrSR0CgYEA/iRI/RdJys4Rqiw76O6zqf2o+206iBcYirI2dCs3wecXVno7YUpz
IUc+0xj5bjmCsHlNltmFTupZQAI51sv2zJKsjUzsu2ImJTroBRSy6C5Kj8oUUFl9
OLDLjbsnv6Mx6WR+JwiYKwfMuTQCUflO32Tr+LElBKisNY2PqmO4+tUCgYEA0fVP
3F2XKpBjy2Qs6y2Xo8/khcxSRqfwDkog7ZLRW1C7u6J8Rrq1ezke/IjOVLM4nZDP
3UceT9LzrkL6jcJ07A2gDwARg9ZYmPetXCCCKdHTgN8Y8mz0XToYZxKatr2D/r1R
VJClKYtahRorkqEWWhOmkw0HusGbKfCmd5gwUFUCgYBvVCvZGuuLeOwKFOiFqJNx
wxnUUkwSs7NfhqQODaSWP4pcqpz6iKeYi2I9DTKvE2hpsCnKDC22nThNruvxaVYK
1bHbEDif+WXmZ0CegSvCRA0LoiV18U3GmMQCqVrHO1ExAYG1zbEDIJ6Q/vSJPmJL
wCUSw18JBG6z4vhtVtQApQKBgQCvgL1W2SzJOZURqRUbKSs+lULSzO5hfXPenfxU
WouCJ0QmHjZ/8QZOkHrkYX8HsiA7JZd7wj0GQLHNEtPZt5iA0QrgPxBlAcFhbHeP
MOVdC7YeXV6/FnBVlYBceGK3KkexopLfe2F0DraF2FBf6yOB/DcbaKLza27GahDc
m2yXWQKBgEDwcbQUP17Yppbvz79w9A2ljzzMEgBulHLkN6iSqAfXMaQkadraPpsp
WEg8K5iVIXq7W+GvrkoafCom6FDuxb2Agrq69g6MuFqIWcy/Sjp9QMCHJRQ/EKSI
s3naIfplFX7rzQJRxNYdYivYJA6vRfPq9Ebc+VWPmivwoEVpdx0i
-----END RSA PRIVATE KEY-----"#;

    const TEST_PKCS8_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDQbyeYcNicDWbV
yFkmD2k0y9Zb4h00s/rcJ2sN+H6iuG/p0w9T9d5VTP/XJFXEzevTC2aiHSciLpZs
BP1Dnwcr39PpbJVExJtmUNBaWj7G/a8UKlFcJMnaC6gX80PqI8RjcQ8/XBZtSWzc
VVsSbxEdNt/AXHoeoKFu5jDljTFNejcSLxSPlxTMUKNKT5pj5x26zXUVvX+M3cpo
CfwmysDPAsFfvlRWtlSzGZq5ktmXArrg/60gUfSQokowVj5xLW+NrPR6uQqGnwV6
POv9EmyDUbv90KapDUWov1w6J72z+3mt3wZm0Efw+KZHDS3ftCHt6ZdUMODALrJh
MV1qvdi5AgMBAAECggEADQ00ycUhLy5gptjXijxTTmjIKLNBOGxftDrsp3CMv3Q0
n8vlTRDbhfRfHdHq4/qv/mRbniGcsnV/2k46rKoP2SW4H4j61NZJ08+SKGI3xZ6b
gLQAqbgtApARy8QiF857DO4GsiU6S0gafraCkCYvyGhH/QPdar8MtMTJwYoEkREN
w5mmpE7Mq4SauaYTeubtD3JeewZ6f70OjeueYxpi+8PS4evQTQAGjXi4aCpBplJ4
JIGAZ7Q7MBNLgO769YnDQlxy0RQ+CEuZowlA9bBs60PIQfHEPquYhtcvA/jYjUSR
L1fGzYYa4qBK9ijHLgWZpUVY1qE2FCq6xDdOmqtJHQKBgQD+JEj9F0nKzhGqLDvo
7rOp/aj7bTqIFxiKsjZ0KzfB5xdWejthSnMhRz7TGPluOYKweU2W2YVO6llAAjnW
y/bMkqyNTOy7YiYlOugFFLLoLkqPyhRQWX04sMuNuye/ozHpZH4nCJgrB8y5NAJR
+U7fZOv4sSUEqKw1jY+qY7j61QKBgQDR9U/cXZcqkGPLZCzrLZejz+SFzFJGp/AO
SiDtktFbULu7onxGurV7OR78iM5UszidkM/dRx5P0vOuQvqNwnTsDaAPABGD1liY
961cIIIp0dOA3xjybPRdOhhnEpq2vYP+vVFUkKUpi1qFGiuSoRZaE6aTDQe6wZsp
8KZ3mDBQVQKBgG9UK9ka64t47AoU6IWok3HDGdRSTBKzs1+GpA4NpJY/ilyqnPqI
p5iLYj0NMq8TaGmwKcoMLbadOE2u6/FpVgrVsdsQOJ/5ZeZnQJ6BK8JEDQuiJXXx
TcaYxAKpWsc7UTEBgbXNsQMgnpD+9Ik+YkvAJRLDXwkEbrPi+G1W1AClAoGBAK+A
vVbZLMk5lRGpFRspKz6VQtLM7mF9c96d/FRai4InRCYeNn/xBk6QeuRhfweyIDsl
l3vCPQZAsc0S09m3mIDRCuA/EGUBwWFsd48w5V0Lth5dXr8WcFWVgFx4YrcqR7Gi
kt97YXQOtoXYUF/rI4H8NxtoovNrbsZqENybbJdZAoGAQPBxtBQ/Xtimlu/Pv3D0
DaWPPMwSAG6UcuQ3qJKoB9cxpCRp2to+mylYSDwrmJUhertb4a+uShp8KiboUO7F
vYCCurr2Doy4WohZzL9KOn1AwIclFD8QpIizedoh+mUVfuvNAlHE1h1iK9gkDq9F
8+r0Rtz5VY+aK/CgRWl3HSI=
-----END PRIVATE KEY-----"#;

    // Former success fixture retained only to prove the 2048-bit floor rejects RSA-1024.
    const WEAK_RSA_1024_PKCS1_PEM: &str = r#"-----BEGIN RSA PRIVATE KEY-----
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

    // Additional throwaway size fixtures exercise both fixed carriers. They were generated locally
    // with OpenSSL and are test data only; none is an operator credential.
    const TEST_RSA_3072_PKCS1_PEM: &str = r#"-----BEGIN RSA PRIVATE KEY-----
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
-----END RSA PRIVATE KEY-----"#;

    const TEST_RSA_4096_PKCS8_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
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
/RHO66txbj6zvf8vctPgVfH0+Fx9
-----END PRIVATE KEY-----"#;

    const TEST_RSA_8192_PKCS1_PEM: &str = r#"-----BEGIN RSA PRIVATE KEY-----
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
-----END RSA PRIVATE KEY-----"#;

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
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone()
                .expect("a request was made")
        }
    }
    impl HttpTransport for FakeTransport {
        fn execute(&self, req: &HttpRequest) -> Result<HttpResponse, TransportError> {
            *self
                .seen
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(req.clone());
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

    fn fixture_der(pem: &str, label: &str) -> Zeroizing<Vec<u8>> {
        decode_private_key_pem(pem, label)
            .expect("fixture PEM decodes")
            .expect("fixture label matches")
    }

    fn pem_from_der(label: &str, der: &[u8]) -> Zeroizing<String> {
        let encoded = Zeroizing::new(STANDARD.encode(der));
        Zeroizing::new(format!(
            "-----BEGIN {label}-----\n{}\n-----END {label}-----",
            encoded.as_str()
        ))
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
        let der = decode_private_key_pem(TEST_PKCS1_PEM, "RSA PRIVATE KEY")
            .unwrap()
            .unwrap();
        let key = parse_rsa_private_key(&der, RsaPrivateKeyFormat::Pkcs1).unwrap();
        assert_eq!(
            modulus_bit_len(key.modulus.as_bytes()),
            2048,
            "test key is RSA-2048"
        );
        let signing_input = format!("{}.{}", parts[0], parts[1]);
        let sig_bytes = URL_SAFE_NO_PAD.decode(parts[2]).unwrap();
        RsaPublicKeyComponents {
            n: key.modulus.as_bytes(),
            e: key.public_exponent.as_bytes(),
        }
        .verify(
            &RSA_PKCS1_2048_8192_SHA256,
            signing_input.as_bytes(),
            &sig_bytes,
        )
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

    fn assert_fixture_signs(pem: &str, expected_bits: usize) {
        const MESSAGE: &[u8] = b"envctl RS256 fixed-width acceptance matrix";
        let signature = rs256_sign(pem.as_bytes(), MESSAGE).expect("fixture signs");
        let (der, format) = if let Some(der) =
            decode_private_key_pem(pem, "RSA PRIVATE KEY").expect("PKCS#1 decode")
        {
            (der, RsaPrivateKeyFormat::Pkcs1)
        } else {
            (
                decode_private_key_pem(pem, "PRIVATE KEY")
                    .expect("PKCS#8 decode")
                    .expect("supported PEM label"),
                RsaPrivateKeyFormat::Pkcs8,
            )
        };
        let key = parse_rsa_private_key(&der, format).expect("RSA DER parses");
        assert_eq!(modulus_bit_len(key.modulus.as_bytes()), expected_bits);
        assert_eq!(signature.len(), expected_bits.div_ceil(8));
        RsaPublicKeyComponents {
            n: key.modulus.as_bytes(),
            e: key.public_exponent.as_bytes(),
        }
        .verify(&RSA_PKCS1_2048_8192_SHA256, MESSAGE, &signature)
        .expect("independent ring verification passes");
    }

    #[test]
    fn fixed_width_signer_accepts_2048_3072_4096_and_8192() {
        assert_fixture_signs(TEST_PKCS1_PEM, 2048);
        assert_fixture_signs(TEST_RSA_3072_PKCS1_PEM, 3072);
        assert_fixture_signs(TEST_RSA_4096_PKCS8_PEM, 4096);
        assert_fixture_signs(TEST_RSA_8192_PKCS1_PEM, 8192);
    }

    #[test]
    fn rsa_key_size_bounds_are_fail_closed() {
        let weak = rs256_sign(WEAK_RSA_1024_PKCS1_PEM.as_bytes(), b"reject weak key")
            .expect_err("RSA-1024 must be rejected");
        assert!(
            matches!(weak, MintError::Other(message) if message.contains("2048..=8192") && message.contains("1024"))
        );

        let mut below_floor = vec![0_u8; 256];
        below_floor[0] = 0x40; // exactly 2047 significant bits
        below_floor[255] = 1;
        assert!(validate_modulus_bits(&below_floor).is_err());

        let mut above_ceiling = vec![0_u8; 1025];
        above_ceiling[0] = 1; // exactly 8193 significant bits
        above_ceiling[1024] = 1;
        assert!(validate_modulus_bits(&above_ceiling).is_err());
    }

    #[test]
    fn non_rsa_pkcs8_algorithm_is_rejected() {
        // rsaEncryption OID 1.2.840.113549.1.1.1 -> mutate to a different, still-valid OID while
        // preserving every DER length. PKCS#8 remains structurally valid but is not an RSA key.
        const RSA_ENCRYPTION_OID_DER: &[u8] = &[
            0x06, 0x09, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x01,
        ];
        let mut der = fixture_der(TEST_PKCS8_PEM, "PRIVATE KEY");
        let offset = der
            .windows(RSA_ENCRYPTION_OID_DER.len())
            .position(|window| window == RSA_ENCRYPTION_OID_DER)
            .expect("rsaEncryption OID present");
        der[offset + RSA_ENCRYPTION_OID_DER.len() - 1] = 0x02;
        let pem = pem_from_der("PRIVATE KEY", &der);
        let error = rs256_sign(pem.as_bytes(), b"reject non-RSA PKCS#8")
            .expect_err("non-RSA algorithm must fail");
        assert!(
            matches!(error, MintError::Other(message) if message == "App private key PKCS#8 algorithm is not RSA")
        );
    }

    #[test]
    fn multi_prime_pkcs1_version_is_rejected_without_alloc_backend() {
        let mut der = fixture_der(TEST_PKCS1_PEM, "RSA PRIVATE KEY");
        let version = der
            .windows(4)
            .position(|window| window == [0x02, 0x01, 0x00, 0x02])
            .expect("two-prime version followed by modulus");
        der[version + 2] = 1; // PKCS#1 version=multi; no-alloc OtherPrimeInfos is unconstructable.
        let pem = pem_from_der("RSA PRIVATE KEY", &der);
        let error = rs256_sign(pem.as_bytes(), b"reject multi-prime")
            .expect_err("multi-prime key must fail closed");
        assert!(
            matches!(error, MintError::Other(message) if message == "App private key is not valid two-prime RSA DER")
        );
    }

    #[test]
    fn malformed_and_trailing_pem_are_rejected() {
        let malformed = Zeroizing::new(
            "-----BEGIN RSA PRIVATE KEY-----\nnot-base64!\n-----END RSA PRIVATE KEY-----"
                .to_string(),
        );
        let error = rs256_sign(malformed.as_bytes(), b"reject malformed")
            .expect_err("malformed body must fail");
        assert!(
            matches!(error, MintError::Other(message) if message == "App private key PEM body is not valid base64")
        );

        let trailing = Zeroizing::new(format!("{TEST_PKCS1_PEM}\nsecond secret block"));
        let error = rs256_sign(trailing.as_bytes(), b"reject trailing")
            .expect_err("trailing data must fail");
        assert!(
            matches!(error, MintError::Other(message) if message == "App private key must contain exactly one complete PEM block")
        );
    }

    #[test]
    fn even_and_oversized_public_exponents_are_rejected() {
        let mut der = fixture_der(TEST_PKCS1_PEM, "RSA PRIVATE KEY");
        let exponent_last = {
            let key = parse_rsa_private_key(&der, RsaPrivateKeyFormat::Pkcs1).unwrap();
            let exponent_offset =
                key.public_exponent.as_bytes().as_ptr() as usize - der.as_ptr() as usize;
            exponent_offset + key.public_exponent.as_bytes().len() - 1
        };
        der[exponent_last] = 2; // 65537 -> 65538, preserving DER length.
        let pem = pem_from_der("RSA PRIVATE KEY", &der);
        let error = rs256_sign(pem.as_bytes(), b"reject even exponent")
            .expect_err("even exponent must fail");
        assert!(
            matches!(error, MintError::Other(message) if message.contains("public exponent must be odd"))
        );

        let oversized_bytes = [0x02, 0x00, 0x00, 0x00, 0x00]; // 2^33, one above the ceiling.
        let oversized = UintRef::new(&oversized_bytes).expect("canonical unsigned integer");
        let error = public_exponent_u64(oversized).expect_err("oversized exponent must fail");
        assert!(
            matches!(error, MintError::Other(message) if message.contains("public exponent must be odd"))
        );
    }

    #[test]
    fn mismatched_private_exponent_is_caught_before_emission() {
        let mut der = fixture_der(TEST_PKCS1_PEM, "RSA PRIVATE KEY");
        let d_last = {
            let key = parse_rsa_private_key(&der, RsaPrivateKeyFormat::Pkcs1).unwrap();
            let d_offset =
                key.private_exponent.as_bytes().as_ptr() as usize - der.as_ptr() as usize;
            d_offset + key.private_exponent.as_bytes().len() - 1
        };
        der[d_last] ^= 1; // structurally valid DER, but d no longer matches (n,e).
        let pem = pem_from_der("RSA PRIVATE KEY", &der);
        let error = rs256_sign(pem.as_bytes(), b"mismatched operational tuple")
            .expect_err("private-operation verify-back must fail");
        assert!(matches!(error, MintError::Other(message) if message == "RS256 signing failed"));
    }

    struct CountingRng {
        inner: RingRsaRng,
        fill_calls: usize,
        filled_bytes: usize,
    }

    impl CountingRng {
        fn new() -> Self {
            Self {
                inner: RingRsaRng(SystemRandom::new()),
                fill_calls: 0,
                filled_bytes: 0,
            }
        }
    }

    impl rsa_ct::rand_core::TryRng for CountingRng {
        type Error = RsaRngError;

        fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
            let mut bytes = [0_u8; 4];
            self.try_fill_bytes(&mut bytes)?;
            Ok(u32::from_ne_bytes(bytes))
        }

        fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
            let mut bytes = [0_u8; 8];
            self.try_fill_bytes(&mut bytes)?;
            Ok(u64::from_ne_bytes(bytes))
        }

        fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
            self.fill_calls += 1;
            self.filled_bytes += dst.len();
            self.inner.try_fill_bytes(dst)
        }
    }

    impl rsa_ct::rand_core::TryCryptoRng for CountingRng {}

    #[test]
    fn each_signature_consumes_a_fresh_blinding_sample() {
        let mut rng = CountingRng::new();
        let first = rs256_sign_with_rng(TEST_PKCS1_PEM.as_bytes(), b"same message", &mut rng)
            .expect("first blinded signature");
        let calls_after_first = rng.fill_calls;
        let bytes_after_first = rng.filled_bytes;
        assert!(calls_after_first > 0);
        assert!(bytes_after_first >= 512, "U4096 blinding sample consumed");

        let second = rs256_sign_with_rng(TEST_PKCS1_PEM.as_bytes(), b"same message", &mut rng)
            .expect("second blinded signature");
        assert!(rng.fill_calls > calls_after_first);
        assert!(rng.filled_bytes >= bytes_after_first + 512);
        assert_eq!(
            first, second,
            "base blinding randomizes the operation, not deterministic RS256 output"
        );
    }

    struct FailingRng;

    impl rsa_ct::rand_core::TryRng for FailingRng {
        type Error = RsaRngError;

        fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
            Err(RsaRngError)
        }

        fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
            Err(RsaRngError)
        }

        fn try_fill_bytes(&mut self, _dst: &mut [u8]) -> Result<(), Self::Error> {
            Err(RsaRngError)
        }
    }

    impl rsa_ct::rand_core::TryCryptoRng for FailingRng {}

    #[test]
    fn blinding_rng_failure_refuses_to_sign() {
        let error =
            rs256_sign_with_rng(TEST_PKCS1_PEM.as_bytes(), b"must not sign", &mut FailingRng)
                .expect_err("RNG failure is fatal");
        assert!(matches!(error, MintError::Other(message) if message == "RS256 signing failed"));
    }

    #[test]
    fn private_key_and_montgomery_owners_are_zeroize_on_drop() {
        fn assert_zeroize_on_drop<T: zeroize::ZeroizeOnDrop>() {}

        type Private4096 = GenericRsaPrivateKey<Rsa4096, ModMathParams<Rsa4096, Ct>>;
        type Private8192 = GenericRsaPrivateKey<Rsa8192, ModMathParams<Rsa8192, Ct>>;
        type Form4096 = rsa_ct::modmath_support::ModMathForm<Rsa4096, Ct>;
        type Form8192 = rsa_ct::modmath_support::ModMathForm<Rsa8192, Ct>;

        assert_zeroize_on_drop::<Private4096>();
        assert_zeroize_on_drop::<Private8192>();
        assert_zeroize_on_drop::<Form4096>();
        assert_zeroize_on_drop::<Form8192>();
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
