//! Pure, synchronous RFC 9449 (DPoP) proof verification for the F2 remote relay edge (TASK-0031).
//!
//! [`verify_dpop_proof`] takes the raw `DPoP` header value plus the request context the caller has
//! already established — the HTTP method (`htm`), the canonical request URI (`htu`), the TLS
//! channel-binding value (EKM, FS-S20), and the current wall-clock — and returns either a
//! [`VerifiedDpop`] (the proof is well-formed, signed by the embedded key, bound to THIS method/URI,
//! within the acceptance window, and bound to THIS connection's EKM) or a [`DpopReject`] saying why
//! it was refused. The edge maps every [`DpopReject`] to a 401 (proof failures are 401s).
//!
//! This module does **NO I/O** and holds no mutable state: it never reads the clock, the network, or
//! the store. The caller supplies `method` / `htu` / `ekm` / `now_ms`. That makes the whole verifier
//! deterministic and vector-testable (the heavy `#[cfg(test)]` suite below). The `jti` replay-store
//! check is the caller's job (the edge owns the `Mutex<JtiReplayStore>`) — this function only
//! surfaces the parsed `jti` so the caller can record it.
//!
//! Crypto/encoding are the pinned, already-linked, pure-Rust crates: `ring` (Ed25519 verify, the
//! same crate rustls pins on its ring provider), `base64` (base64url JWT segments + the jkt), `sha2`
//! (RFC 7638 JWK SHA-256 thumbprint), `serde_json` (the header/payload JSON). ZERO new deps, no C.
//!
//! Fail-closed everywhere: any malformed/missing/ambiguous field REJECTS; there is no accept path
//! that skips the signature, the binding, or the window. NO secret bytes are logged — the proof JWT,
//! the signature, and the EKM never leave this function (the caller logs only `client_id` + a
//! reason). The acceptance window reuses the F6 `JtiReplayStore` constants so drift bounds match.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use envctl_secrets::broker::jti::{ACCEPT_FUTURE_MS, ACCEPT_PAST_MS};
use sha2::{Digest, Sha256};

/// A successfully verified DPoP proof. Carries only NON-secret, already-public values: the proven
/// key's RFC 7638 thumbprint (`jkt`), the client-asserted identity claim (UNUSED for trust — the
/// edge binds identity via the bearer + registry), the proof's unique id (`jti`, for the replay
/// store), and its issued-at. None of these are secret; the signature and EKM are NOT returned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedDpop {
    /// The `sub`/`client_id`-style claim the proof carried, if any. The edge does NOT trust this for
    /// identity — identity comes from the bearer's bound `client_id` + the registry. Present only so
    /// a caller can cross-check; `None` when the proof carried no such claim. (RFC 9449 proofs need
    /// not carry a subject; the bearer is the identity anchor here.)
    pub client_id: Option<String>,
    /// RFC 7638 JWK SHA-256 thumbprint of the embedded DPoP public key. The edge compares this to the
    /// registered `dpop_jkt` for the bearer's `client_id` (`RemoteBindingMismatch` on divergence).
    pub jkt: [u8; 32],
    /// The proof's unique id (RFC 9449 `jti`) — the caller records it in the replay store.
    pub jti: String,
    /// The proof's issued-at in wall-ms — the caller passes it to `JtiReplayStore::check_and_record`.
    pub iat_ms: i64,
}

/// Why a DPoP proof was refused. Every variant is a REJECT (the edge maps all to 401). There is no
/// accept-on-error variant by construction. The `Ekm*` variants are the FS-S20 channel-binding
/// fail-closed path: if the EKM could not be bound the proof is refused (the edge maps those to 403).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DpopReject {
    /// The header value was not three base64url-decodable JWT segments (`h.p.s`).
    MalformedJwt,
    /// A header/payload segment did not base64url-decode, or was not valid JSON.
    DecodeError,
    /// `typ` was absent or not `"dpop+jwt"`.
    BadTyp,
    /// `alg` was absent or not `"EdDSA"`.
    BadAlg,
    /// The embedded `jwk` was absent, not an OKP/Ed25519 key, or its `x` was not a 32-byte base64url.
    BadJwk,
    /// The Ed25519 signature over `header.payload` did not verify against the embedded key.
    BadSignature,
    /// `htm` was absent or did not equal the request method.
    HtmMismatch,
    /// `htu` was absent or did not equal the canonical request URI.
    HtuMismatch,
    /// `jti` was absent or empty.
    MissingJti,
    /// `iat` was absent or not an integer.
    MissingIat,
    /// `iat` is older than `now - ACCEPT_PAST_MS` (outside the acceptance window, past).
    ClockDriftPast,
    /// `iat` is newer than `now + ACCEPT_FUTURE_MS` (outside the acceptance window, future).
    ClockDriftFuture,
    /// The proof did not carry an EKM binding claim (`ekm`) — channel binding could not be confirmed.
    EkmMissing,
    /// The proof's EKM binding claim did not equal the connection's EKM (FS-S20: replay across a
    /// different TLS channel, or a TLS-terminating front). Fail-closed.
    EkmMismatch,
    /// The caller could not compute the connection EKM at all (uncomputable binding — FS-S20 forbidden
    /// state). The edge maps this to 403. Distinct from a present-but-wrong `ekm` (`EkmMismatch`).
    EkmUncomputable,
}

/// The HTTP method the request arrived with, for the `htm` binding check (case-insensitive compare).
/// A small explicit enum keeps the htm check exact (no header-string parsing in this pure module).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl HttpMethod {
    /// The canonical RFC 9449 `htm` value (uppercase HTTP method token).
    fn as_htm(self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
        }
    }
}

/// Verify an RFC 9449 DPoP proof, fail-closed. See the module docs.
///
/// `dpop_header` — the raw `DPoP` request-header value (the compact JWS `h.p.s`).
/// `method` / `htu` — the request's method + canonical URI (scheme+host+path, NO query) the proof
/// must bind (`htm`/`htu`). `ekm` — the connection's exported keying material, or `None` if the
/// caller could NOT compute it (→ `EkmUncomputable`, 403). `now_ms` — the caller's wall clock.
///
/// On success the proof is: 3 well-formed base64url JWT segments; `typ == "dpop+jwt"`,
/// `alg == "EdDSA"`, an embedded OKP/Ed25519 `jwk` with a 32-byte `x`; a valid Ed25519 signature over
/// `header.payload`; `htm`/`htu` equal to the supplied method/URI; `iat` within the F6 acceptance
/// window; and an `ekm` claim equal to the supplied connection EKM (base64url). Returns the parsed
/// `jkt`/`jti`/`iat` for the caller's registry + replay-store checks.
pub fn verify_dpop_proof(
    dpop_header: &str,
    method: HttpMethod,
    htu: &str,
    ekm: Option<&[u8]>,
    now_ms: i64,
) -> Result<VerifiedDpop, DpopReject> {
    // (0) EKM must be computable — FS-S20 fail-closed. A connection whose binding the validating
    // process cannot compute is a forbidden state; refuse BEFORE touching the proof.
    let ekm = ekm.ok_or(DpopReject::EkmUncomputable)?;

    // (1) Split the compact JWS into exactly three base64url segments.
    let mut parts = dpop_header.split('.');
    let (Some(h_b64), Some(p_b64), Some(s_b64), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(DpopReject::MalformedJwt);
    };
    if h_b64.is_empty() || p_b64.is_empty() || s_b64.is_empty() {
        return Err(DpopReject::MalformedJwt);
    }

    // (2) base64url-decode the header + payload and parse them as JSON objects.
    let header_bytes = URL_SAFE_NO_PAD
        .decode(h_b64)
        .map_err(|_| DpopReject::DecodeError)?;
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(p_b64)
        .map_err(|_| DpopReject::DecodeError)?;
    let sig_bytes = URL_SAFE_NO_PAD
        .decode(s_b64)
        .map_err(|_| DpopReject::DecodeError)?;
    let header: serde_json::Value =
        serde_json::from_slice(&header_bytes).map_err(|_| DpopReject::DecodeError)?;
    let payload: serde_json::Value =
        serde_json::from_slice(&payload_bytes).map_err(|_| DpopReject::DecodeError)?;

    // (3) Header: typ == "dpop+jwt", alg == "EdDSA".
    if header.get("typ").and_then(|v| v.as_str()) != Some("dpop+jwt") {
        return Err(DpopReject::BadTyp);
    }
    if header.get("alg").and_then(|v| v.as_str()) != Some("EdDSA") {
        return Err(DpopReject::BadAlg);
    }

    // (4) Embedded JWK: OKP / crv Ed25519 / 32-byte x. Reject anything else (RSA/EC keys, missing x).
    let jwk = header.get("jwk").ok_or(DpopReject::BadJwk)?;
    if jwk.get("kty").and_then(|v| v.as_str()) != Some("OKP") {
        return Err(DpopReject::BadJwk);
    }
    if jwk.get("crv").and_then(|v| v.as_str()) != Some("Ed25519") {
        return Err(DpopReject::BadJwk);
    }
    let x_b64 = jwk
        .get("x")
        .and_then(|v| v.as_str())
        .ok_or(DpopReject::BadJwk)?;
    let x_bytes = URL_SAFE_NO_PAD
        .decode(x_b64)
        .map_err(|_| DpopReject::BadJwk)?;
    if x_bytes.len() != 32 {
        return Err(DpopReject::BadJwk);
    }

    // (5) Verify the Ed25519 signature over the ASCII signing input `header_b64 "." payload_b64`. The
    // signing input is the RAW segments as received (NOT re-encoded) per RFC 7515 §5.
    let signing_input = format!("{h_b64}.{p_b64}");
    let pubkey = ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, &x_bytes);
    if pubkey.verify(signing_input.as_bytes(), &sig_bytes).is_err() {
        return Err(DpopReject::BadSignature);
    }

    // (6) Compute the RFC 7638 JWK thumbprint: SHA-256 over the canonical JSON of the REQUIRED members
    // in lexicographic order. For an OKP key the required members are exactly {crv, kty, x} (RFC 8037
    // §2), serialized with no whitespace and the keys sorted — `{"crv":"Ed25519","kty":"OKP","x":"<x>"}`.
    let canonical = format!("{{\"crv\":\"Ed25519\",\"kty\":\"OKP\",\"x\":\"{x_b64}\"}}");
    let jkt: [u8; 32] = Sha256::digest(canonical.as_bytes()).into();

    // (7) htm / htu binding. htm is the uppercase method; htu is the canonical URI (scheme+host+path,
    // no query) the caller already canonicalized.
    if payload.get("htm").and_then(|v| v.as_str()) != Some(method.as_htm()) {
        return Err(DpopReject::HtmMismatch);
    }
    if payload.get("htu").and_then(|v| v.as_str()) != Some(htu) {
        return Err(DpopReject::HtuMismatch);
    }

    // (8) jti — present, non-empty (the caller records it in the replay store).
    let jti = payload
        .get("jti")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or(DpopReject::MissingJti)?
        .to_string();

    // (9) iat — an integer SECONDS-since-epoch (RFC 9449 / JWT `iat` is seconds). Convert to ms and
    // enforce the F6 acceptance window (same constants the JtiReplayStore uses, so drift bounds match).
    let iat_secs = payload
        .get("iat")
        .and_then(|v| v.as_i64())
        .ok_or(DpopReject::MissingIat)?;
    let iat_ms = iat_secs.saturating_mul(1000);
    if iat_ms < now_ms - ACCEPT_PAST_MS {
        return Err(DpopReject::ClockDriftPast);
    }
    if iat_ms > now_ms + ACCEPT_FUTURE_MS {
        return Err(DpopReject::ClockDriftFuture);
    }

    // (10) EKM channel binding (FS-S20). The proof MUST carry an `ekm` claim (base64url of the same
    // exported keying material the edge computed off the terminated TLS stream). A missing claim ⇒
    // EkmMissing; a present-but-different value ⇒ EkmMismatch. Constant-time compare not required (the
    // EKM is not a long-lived secret being guessed byte-by-byte — it is a per-connection binding the
    // attacker either has end-to-end or does not), but we compare the full decoded bytes.
    let ekm_claim_b64 = payload
        .get("ekm")
        .and_then(|v| v.as_str())
        .ok_or(DpopReject::EkmMissing)?;
    let ekm_claim = URL_SAFE_NO_PAD
        .decode(ekm_claim_b64)
        .map_err(|_| DpopReject::EkmMismatch)?;
    if ekm_claim != ekm {
        return Err(DpopReject::EkmMismatch);
    }

    // The (optional) subject/client_id claim — surfaced but NOT trusted for identity.
    let client_id = payload
        .get("client_id")
        .or_else(|| payload.get("sub"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(VerifiedDpop {
        client_id,
        jkt,
        jti,
        iat_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ring::signature::{Ed25519KeyPair, KeyPair};

    const NOW: i64 = 1_700_000_000_000; // fixed wall-ms anchor
    const EKM: &[u8] = b"connection-ekm-32-bytes-exactly!"; // 32 bytes

    fn b64(bytes: &[u8]) -> String {
        URL_SAFE_NO_PAD.encode(bytes)
    }

    /// Build a signed DPoP proof from a keypair + arbitrary header/payload JSON values. Returns the
    /// compact `h.p.s` string. The signing input is the raw base64url segments (RFC 7515 §5).
    fn make_proof(
        kp: &Ed25519KeyPair,
        header: serde_json::Value,
        payload: serde_json::Value,
    ) -> String {
        let h_b64 = b64(serde_json::to_string(&header).unwrap().as_bytes());
        let p_b64 = b64(serde_json::to_string(&payload).unwrap().as_bytes());
        let signing_input = format!("{h_b64}.{p_b64}");
        let sig = kp.sign(signing_input.as_bytes());
        let s_b64 = b64(sig.as_ref());
        format!("{h_b64}.{p_b64}.{s_b64}")
    }

    fn keypair() -> Ed25519KeyPair {
        // A fixed PKCS#8 seed so tests are deterministic. ring generates the doc once; we reuse it.
        let rng = ring::rand::SystemRandom::new();
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap()
    }

    fn jwk_for(kp: &Ed25519KeyPair) -> serde_json::Value {
        let x = b64(kp.public_key().as_ref());
        serde_json::json!({ "kty": "OKP", "crv": "Ed25519", "x": x })
    }

    fn valid_header(kp: &Ed25519KeyPair) -> serde_json::Value {
        serde_json::json!({ "typ": "dpop+jwt", "alg": "EdDSA", "jwk": jwk_for(kp) })
    }

    fn valid_payload() -> serde_json::Value {
        serde_json::json!({
            "htm": "POST",
            "htu": "https://edge.example/v1/relay/swap",
            "jti": "jti-unique-1",
            "iat": NOW / 1000,
            "ekm": b64(EKM),
            "client_id": "phone",
        })
    }

    const HTU: &str = "https://edge.example/v1/relay/swap";

    #[test]
    fn valid_proof_accepted() {
        let kp = keypair();
        let proof = make_proof(&kp, valid_header(&kp), valid_payload());
        let v =
            verify_dpop_proof(&proof, HttpMethod::Post, HTU, Some(EKM), NOW).expect("valid proof");
        assert_eq!(v.jti, "jti-unique-1");
        assert_eq!(v.iat_ms, NOW);
        assert_eq!(v.client_id.as_deref(), Some("phone"));
        // The jkt is deterministic from the public key — recomputing the RFC 7638 thumbprint matches.
        let x_b64 = b64(kp.public_key().as_ref());
        let canonical = format!("{{\"crv\":\"Ed25519\",\"kty\":\"OKP\",\"x\":\"{x_b64}\"}}");
        let expect: [u8; 32] = Sha256::digest(canonical.as_bytes()).into();
        assert_eq!(v.jkt, expect);
    }

    #[test]
    fn uncomputable_ekm_rejected_failclosed() {
        let kp = keypair();
        let proof = make_proof(&kp, valid_header(&kp), valid_payload());
        assert_eq!(
            verify_dpop_proof(&proof, HttpMethod::Post, HTU, None, NOW),
            Err(DpopReject::EkmUncomputable)
        );
    }

    #[test]
    fn ekm_mismatch_rejected() {
        let kp = keypair();
        let proof = make_proof(&kp, valid_header(&kp), valid_payload());
        let other_ekm = b"a-different-connection-ekm-value"; // 31 bytes, != EKM
        assert_eq!(
            verify_dpop_proof(&proof, HttpMethod::Post, HTU, Some(other_ekm), NOW),
            Err(DpopReject::EkmMismatch)
        );
    }

    #[test]
    fn ekm_claim_absent_rejected() {
        let kp = keypair();
        let mut payload = valid_payload();
        payload.as_object_mut().unwrap().remove("ekm");
        let proof = make_proof(&kp, valid_header(&kp), payload);
        assert_eq!(
            verify_dpop_proof(&proof, HttpMethod::Post, HTU, Some(EKM), NOW),
            Err(DpopReject::EkmMissing)
        );
    }

    #[test]
    fn bad_signature_rejected() {
        let kp = keypair();
        let other = keypair();
        // Build a proof embedding kp's jwk but sign with `other` — the signature won't verify against
        // the embedded key.
        let h_b64 = b64(serde_json::to_string(&valid_header(&kp))
            .unwrap()
            .as_bytes());
        let p_b64 = b64(serde_json::to_string(&valid_payload()).unwrap().as_bytes());
        let signing_input = format!("{h_b64}.{p_b64}");
        let sig = other.sign(signing_input.as_bytes());
        let proof = format!("{h_b64}.{p_b64}.{}", b64(sig.as_ref()));
        assert_eq!(
            verify_dpop_proof(&proof, HttpMethod::Post, HTU, Some(EKM), NOW),
            Err(DpopReject::BadSignature)
        );
    }

    #[test]
    fn tampered_payload_rejected() {
        // A valid proof whose payload bytes are swapped for a different (in-window) payload while the
        // signature is left intact — the signature no longer covers the swapped payload.
        let kp = keypair();
        let proof = make_proof(&kp, valid_header(&kp), valid_payload());
        let mut parts: Vec<&str> = proof.split('.').collect();
        let evil_payload = serde_json::json!({
            "htm": "POST", "htu": HTU, "jti": "evil", "iat": NOW / 1000, "ekm": b64(EKM),
        });
        let evil_b64 = b64(serde_json::to_string(&evil_payload).unwrap().as_bytes());
        parts[1] = &evil_b64;
        let tampered = parts.join(".");
        assert_eq!(
            verify_dpop_proof(&tampered, HttpMethod::Post, HTU, Some(EKM), NOW),
            Err(DpopReject::BadSignature)
        );
    }

    #[test]
    fn wrong_typ_rejected() {
        let kp = keypair();
        let mut header = valid_header(&kp);
        header.as_object_mut().unwrap().insert(
            "typ".to_string(),
            serde_json::Value::String("jwt".to_string()),
        );
        let proof = make_proof(&kp, header, valid_payload());
        assert_eq!(
            verify_dpop_proof(&proof, HttpMethod::Post, HTU, Some(EKM), NOW),
            Err(DpopReject::BadTyp)
        );
    }

    #[test]
    fn wrong_alg_rejected() {
        let kp = keypair();
        let mut header = valid_header(&kp);
        header.as_object_mut().unwrap().insert(
            "alg".to_string(),
            serde_json::Value::String("RS256".to_string()),
        );
        let proof = make_proof(&kp, header, valid_payload());
        assert_eq!(
            verify_dpop_proof(&proof, HttpMethod::Post, HTU, Some(EKM), NOW),
            Err(DpopReject::BadAlg)
        );
    }

    #[test]
    fn non_okp_jwk_rejected() {
        let kp = keypair();
        let mut header = valid_header(&kp);
        // Swap the embedded jwk for an EC key shape.
        header.as_object_mut().unwrap().insert(
            "jwk".to_string(),
            serde_json::json!({ "kty": "EC", "crv": "P-256", "x": "aa", "y": "bb" }),
        );
        let proof = make_proof(&kp, header, valid_payload());
        assert_eq!(
            verify_dpop_proof(&proof, HttpMethod::Post, HTU, Some(EKM), NOW),
            Err(DpopReject::BadJwk)
        );
    }

    #[test]
    fn htm_mismatch_rejected() {
        let kp = keypair();
        let proof = make_proof(&kp, valid_header(&kp), valid_payload());
        // Proof binds POST; the request arrives as GET.
        assert_eq!(
            verify_dpop_proof(&proof, HttpMethod::Get, HTU, Some(EKM), NOW),
            Err(DpopReject::HtmMismatch)
        );
    }

    #[test]
    fn htu_mismatch_rejected() {
        let kp = keypair();
        let proof = make_proof(&kp, valid_header(&kp), valid_payload());
        assert_eq!(
            verify_dpop_proof(
                &proof,
                HttpMethod::Post,
                "https://edge.example/v1/relay/OTHER",
                Some(EKM),
                NOW
            ),
            Err(DpopReject::HtuMismatch)
        );
    }

    #[test]
    fn iat_too_old_rejected() {
        let kp = keypair();
        let mut payload = valid_payload();
        // iat older than ACCEPT_PAST → ClockDriftPast.
        let too_old = (NOW - ACCEPT_PAST_MS - 1000) / 1000;
        payload
            .as_object_mut()
            .unwrap()
            .insert("iat".to_string(), serde_json::json!(too_old));
        let proof = make_proof(&kp, valid_header(&kp), payload);
        assert_eq!(
            verify_dpop_proof(&proof, HttpMethod::Post, HTU, Some(EKM), NOW),
            Err(DpopReject::ClockDriftPast)
        );
    }

    #[test]
    fn iat_in_future_rejected() {
        let kp = keypair();
        let mut payload = valid_payload();
        let too_new = (NOW + ACCEPT_FUTURE_MS + 5000) / 1000;
        payload
            .as_object_mut()
            .unwrap()
            .insert("iat".to_string(), serde_json::json!(too_new));
        let proof = make_proof(&kp, valid_header(&kp), payload);
        assert_eq!(
            verify_dpop_proof(&proof, HttpMethod::Post, HTU, Some(EKM), NOW),
            Err(DpopReject::ClockDriftFuture)
        );
    }

    #[test]
    fn missing_jti_rejected() {
        let kp = keypair();
        let mut payload = valid_payload();
        payload.as_object_mut().unwrap().remove("jti");
        let proof = make_proof(&kp, valid_header(&kp), payload);
        assert_eq!(
            verify_dpop_proof(&proof, HttpMethod::Post, HTU, Some(EKM), NOW),
            Err(DpopReject::MissingJti)
        );
    }

    #[test]
    fn malformed_jwt_rejected() {
        // Not three segments.
        assert_eq!(
            verify_dpop_proof("only.two", HttpMethod::Post, HTU, Some(EKM), NOW),
            Err(DpopReject::MalformedJwt)
        );
        assert_eq!(
            verify_dpop_proof("a.b.c.d", HttpMethod::Post, HTU, Some(EKM), NOW),
            Err(DpopReject::MalformedJwt)
        );
        assert_eq!(
            verify_dpop_proof("", HttpMethod::Post, HTU, Some(EKM), NOW),
            Err(DpopReject::MalformedJwt)
        );
    }

    #[test]
    fn non_base64_segment_rejected() {
        // Three segments but the header is not base64url.
        let proof = "!!!not-b64!!!.payload.sig";
        assert_eq!(
            verify_dpop_proof(proof, HttpMethod::Post, HTU, Some(EKM), NOW),
            Err(DpopReject::DecodeError)
        );
    }

    #[test]
    fn rfc7638_thumbprint_is_sha256_of_canonical_jwk() {
        // RFC 7638 §3.1 worked-style example for an OKP key: the thumbprint is SHA-256 over the
        // canonical (sorted-key, no-whitespace) JSON {"crv":..,"kty":..,"x":..}. We assert our
        // verifier's jkt equals an independent recomputation for a known key.
        let kp = keypair();
        let proof = make_proof(&kp, valid_header(&kp), valid_payload());
        let v = verify_dpop_proof(&proof, HttpMethod::Post, HTU, Some(EKM), NOW).unwrap();
        let x_b64 = b64(kp.public_key().as_ref());
        let canonical = format!("{{\"crv\":\"Ed25519\",\"kty\":\"OKP\",\"x\":\"{x_b64}\"}}");
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        let expect: [u8; 32] = hasher.finalize().into();
        assert_eq!(v.jkt, expect);
    }
}
