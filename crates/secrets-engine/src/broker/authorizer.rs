//! Operator-box **presence-token authorizer** (audit F8 / OI-SM-2; SERVER-MODE Profile B).
//!
//! When `secretd` runs OFF the operator box (a VPS), there is no USB to physically possess, so the
//! egress presence gate's possession factor is substituted by a short-lived, Ed25519-signed
//! **presence token** minted on the operator box (where the USB/Seed actually lives) and pushed to
//! the VPS over an mTLS link. A valid token is proof that, *recently*, the operator box still held
//! possession — exactly the property the on-box USB probe gives Profile A.
//!
//! This module is the engine-side, **sync, non-printing, pure-Rust** core:
//!   * [`PresenceToken`] — the wire shape (serde).
//!   * [`presence_token_signing_bytes`] — the canonical, domain-separated, length-prefixed message
//!     the Ed25519 signature covers (mirrors `bearer_row_mac_message`'s no-collision discipline).
//!   * [`sign_presence_token`] — operator-box signer (ring Ed25519 from a 32-byte seed).
//!   * [`verify_presence_token`] — VPS verifier: an ORDERED, fail-closed ladder over trusted time,
//!     the signature, the cert binding, the server nonce, the validity window, and replay.
//!
//! Design corpus: `docs/secrets/OI-SM-2-operator-authorizer.md`.
//!
//! Zero new dependencies: `ring` (Ed25519), `serde`, and the existing [`NonceStore`] /
//! [`JtiReplayStore`] are already in the resolved graph. No C, one rustls (ring), engine stays
//! sync + non-printing.

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::broker::jti::{JtiReject, JtiReplayStore};
use crate::broker::nonce::{NonceReject, NonceStore};
use crate::seam::TrustedTime;

/// Domain-separation prefix for the presence-token signing message. Distinct from every other
/// signed/MAC'd message in the crate (`bearer_row_mac_message`, the seed KEK context, the audit-head
/// anchor) so a presence-token signature can never be confused with any other authenticator.
const PRESENCE_TOKEN_DOMAIN: &[u8] = b"env-ctl/v1/presence-token";

/// The current presence-token wire version. Bumping this is a clean break: [`verify_presence_token`]
/// rejects any other value as [`AuthzReject::MalformedVersion`] before doing any crypto.
pub const PRESENCE_TOKEN_VERSION: u8 = 1;

/// Default presence-token TTL clamp (ms). The operator box mints tokens with a short lifetime so a
/// captured token cannot keep a VPS authorized for long after possession is lost; a fresh mint
/// re-checks USB possession on the operator box. 10 min — the middle of the audited 5–15 min band.
pub const DEFAULT_TOKEN_TTL_MS: i64 = 600_000;

/// Allowed clock skew (ms) when checking `ts_ms` against trusted time, so a token minted a moment in
/// the operator box's future (vs. the VPS's trusted-time reading) is not spuriously rejected.
pub const TOKEN_SKEW_MS: i64 = 30_000;

/// An operator-box-signed proof of *recent* possession, substituting for on-box USB possession when
/// `secretd` runs on a VPS (Profile B). Carries NO secret material — the signature authenticates it,
/// and every field is public metadata. Serialized for transport over the mTLS authorizer link.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresenceToken {
    /// Wire version. MUST be [`PRESENCE_TOKEN_VERSION`]; any other value is rejected before crypto.
    pub v: u8,
    /// When the token was minted (operator-box trusted wall-clock epoch-ms). Bounds how far in the
    /// future a token may claim to have been minted (`ts_ms <= trusted_now + skew`).
    pub ts_ms: i64,
    /// The VPS instance this token authorizes — binds the token to a specific deployment so a token
    /// minted for one VPS cannot be replayed to another.
    pub vps_instance_id: String,
    /// The server-issued, single-use nonce the VPS challenged with — consumed on accept so a token
    /// cannot be replayed against the SAME VPS (anti-replay, paired with `jti`).
    pub server_nonce: String,
    /// SHA-256 fingerprint of the VPS's edge/mTLS certificate — channel binding: a token is valid
    /// ONLY when presented over the link whose cert matches, so a man-in-the-middle that strips mTLS
    /// cannot forward a token to a different endpoint.
    pub vps_cert_fp: [u8; 32],
    /// Absolute expiry (operator-box trusted wall-clock epoch-ms). The VPS denies once trusted time
    /// passes this; a fresh mint (which re-checks USB possession) is then required.
    pub expiry_ms: i64,
    /// Unique token id (operator-box minted) — recorded on accept in the [`JtiReplayStore`] so the
    /// SAME token cannot be accepted twice within its window (replay defense, paired with the nonce).
    pub jti: String,
}

impl PresenceToken {
    /// Build a token with `v` set to the current version.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ts_ms: i64,
        vps_instance_id: String,
        server_nonce: String,
        vps_cert_fp: [u8; 32],
        expiry_ms: i64,
        jti: String,
    ) -> Self {
        Self {
            v: PRESENCE_TOKEN_VERSION,
            ts_ms,
            vps_instance_id,
            server_nonce,
            vps_cert_fp,
            expiry_ms,
            jti,
        }
    }
}

/// Canonical, unambiguous byte encoding of the presence-token fields the Ed25519 signature
/// authenticates. Domain-separated and length-prefixed so no two distinct tokens collide on the same
/// message (the same discipline as [`super::bearer_row_mac_message`]):
///
/// `domain ‖ v ‖ ts_ms.be ‖ (len u32 be)‖vps_instance_id ‖ len‖server_nonce ‖ vps_cert_fp[32] ‖
///  expiry_ms.be ‖ len‖jti`
///
/// Every variable-length field is preceded by its big-endian `u32` byte length, and every fixed
/// field is fixed-width, so a value boundary can never be shifted to forge a different-but-colliding
/// token.
#[must_use]
pub fn presence_token_signing_bytes(tok: &PresenceToken) -> Vec<u8> {
    let id = tok.vps_instance_id.as_bytes();
    let nonce = tok.server_nonce.as_bytes();
    let jti = tok.jti.as_bytes();
    let mut m = Vec::with_capacity(
        PRESENCE_TOKEN_DOMAIN.len()
            + 1
            + 8
            + 4
            + id.len()
            + 4
            + nonce.len()
            + 32
            + 8
            + 4
            + jti.len(),
    );
    m.extend_from_slice(PRESENCE_TOKEN_DOMAIN);
    m.push(tok.v);
    m.extend_from_slice(&tok.ts_ms.to_be_bytes());
    m.extend_from_slice(&(id.len() as u32).to_be_bytes());
    m.extend_from_slice(id);
    m.extend_from_slice(&(nonce.len() as u32).to_be_bytes());
    m.extend_from_slice(nonce);
    m.extend_from_slice(&tok.vps_cert_fp);
    m.extend_from_slice(&tok.expiry_ms.to_be_bytes());
    m.extend_from_slice(&(jti.len() as u32).to_be_bytes());
    m.extend_from_slice(jti);
    m
}

/// Sign a presence token on the operator box with the Ed25519 device/operator key (32-byte seed).
/// The seed is held in [`Zeroizing`] so it is wiped after use. Returns the 64-byte Ed25519
/// signature over [`presence_token_signing_bytes`]. `ring`'s key construction can fail on a
/// structurally invalid seed; that surfaces as an `Err` (never a panic).
pub fn sign_presence_token(
    seed: &Zeroizing<[u8; 32]>,
    tok: &PresenceToken,
) -> anyhow::Result<[u8; 64]> {
    let kp = ring::signature::Ed25519KeyPair::from_seed_unchecked(seed.as_ref())
        .map_err(|e| anyhow::anyhow!("invalid Ed25519 seed for presence-token signer: {e}"))?;
    let msg = presence_token_signing_bytes(tok);
    let sig = kp.sign(&msg);
    let mut out = [0u8; 64];
    let bytes = sig.as_ref();
    if bytes.len() != 64 {
        anyhow::bail!("unexpected Ed25519 signature length {}", bytes.len());
    }
    out.copy_from_slice(bytes);
    Ok(out)
}

/// Why a presence token was REJECTED by [`verify_presence_token`]. Every variant is a REJECT; there
/// is no accept-on-error path. The ladder is fail-closed and ORDERED (cheap/structural checks first,
/// then crypto, then binding, then liveness, then replay) so a malformed/forged token never reaches
/// the replay store and a single-use nonce/jti is only consumed on a token that has already passed
/// every other gate.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AuthzReject {
    /// `v` is not [`PRESENCE_TOKEN_VERSION`] — rejected before any crypto.
    #[error("presence token version is not supported")]
    MalformedVersion,
    /// Trusted time is unavailable (OI-SM-3): the VPS has no fresh externally-attested clock, so it
    /// cannot decide expiry safely. Refuse rather than trust the hypervisor-controlled local clock.
    #[error("trusted time unavailable — cannot verify presence token expiry")]
    TrustedTimeUnavailable,
    /// The Ed25519 signature does not verify against the pinned operator public key.
    #[error("presence token signature invalid")]
    BadSignature,
    /// The token's `vps_cert_fp` does not match this VPS's edge certificate fingerprint (channel
    /// binding failed — the token was minted for a different endpoint).
    #[error("presence token cert-fingerprint mismatch")]
    CertFpMismatch,
    /// The `server_nonce` was never issued by this VPS (or was already consumed — single-use).
    #[error("presence token server-nonce unknown")]
    NonceUnknown,
    /// Trusted time is at or past `expiry_ms`.
    #[error("presence token expired")]
    Expired,
    /// `ts_ms` claims the token was minted further in the future than the allowed skew.
    #[error("presence token not yet valid (minted in the future beyond skew)")]
    NotYetValid,
    /// The token's `jti` was already accepted within its window — a replay.
    #[error("presence token replayed")]
    Replayed,
}

/// A fixed, metadata-only discriminant label for a rejection (for the `PresenceTokenRejected`
/// event). Carries NO token/sig/key bytes — only the reason class.
#[must_use]
pub fn authz_reject_label(r: &AuthzReject) -> &'static str {
    match r {
        AuthzReject::MalformedVersion => "malformed_version",
        AuthzReject::TrustedTimeUnavailable => "trusted_time_unavailable",
        AuthzReject::BadSignature => "bad_signature",
        AuthzReject::CertFpMismatch => "cert_fp_mismatch",
        AuthzReject::NonceUnknown => "nonce_unknown",
        AuthzReject::Expired => "expired",
        AuthzReject::NotYetValid => "not_yet_valid",
        AuthzReject::Replayed => "replayed",
    }
}

/// Map a [`NonceReject`] to the single nonce-failure verdict (all nonce failures mean the same thing
/// to the VPS: the token's server-nonce is not a live, server-issued challenge → refuse).
impl From<NonceReject> for AuthzReject {
    fn from(_: NonceReject) -> Self {
        AuthzReject::NonceUnknown
    }
}

/// Verify a presence token on the VPS. ORDERED, fail-closed ladder (audit F8 / OI-SM-2):
///
/// 1. **version** — `v == PRESENCE_TOKEN_VERSION` else [`AuthzReject::MalformedVersion`].
/// 2. **trusted time** — `trusted_time.now_ms()` is `Some(t)` else [`AuthzReject::TrustedTimeUnavailable`]
///    (OI-SM-3: never trust the VPS's local clock for expiry).
/// 3. **signature** — ring Ed25519 verify over [`presence_token_signing_bytes`] with `pubkey`, else
///    [`AuthzReject::BadSignature`].
/// 4. **cert binding** — `tok.vps_cert_fp == *expected_cert_fp` else [`AuthzReject::CertFpMismatch`].
/// 5. **nonce** — [`NonceStore::check_and_consume`] (single-use) else [`AuthzReject::NonceUnknown`].
/// 6. **validity** — `t < expiry_ms` (else [`AuthzReject::Expired`]) and
///    `t + TOKEN_SKEW_MS >= ts_ms` (else [`AuthzReject::NotYetValid`]).
/// 7. **replay** — [`JtiReplayStore::check_and_record`] else [`AuthzReject::Replayed`].
///
/// The nonce/jti single-use stores are consumed under the caller's lock (the authorizer owns the
/// `Mutex`es), so the check-and-consume/record is atomic. A token that fails ANY earlier step never
/// reaches the nonce/jti consume — so a malformed/forged/expired token never burns a live nonce.
///
/// NOTE on order: nonce-consume runs BEFORE the expiry check so that a single-use challenge is spent
/// the moment a cryptographically valid, cert-bound token presents it — preventing an attacker from
/// re-presenting the same valid signature with a different (still-live) nonce. The jti replay-record
/// runs LAST (after expiry/not-yet-valid) so the bounded replay store only ever holds tokens that
/// were live at accept time.
#[allow(clippy::too_many_arguments)]
pub fn verify_presence_token(
    pubkey: &[u8; 32],
    tok: &PresenceToken,
    sig: &[u8; 64],
    expected_cert_fp: &[u8; 32],
    trusted_time: &dyn TrustedTime,
    nonce_store: &mut NonceStore,
    jti_store: &mut JtiReplayStore,
) -> Result<(), AuthzReject> {
    // 1. Version — structural, before any crypto.
    if tok.v != PRESENCE_TOKEN_VERSION {
        return Err(AuthzReject::MalformedVersion);
    }

    // 2. Trusted time (OI-SM-3) — refuse if the VPS has no fresh externally-attested clock.
    let now = trusted_time
        .now_ms()
        .ok_or(AuthzReject::TrustedTimeUnavailable)?;

    // 3. Signature — ring Ed25519 over the canonical signing bytes.
    let msg = presence_token_signing_bytes(tok);
    let key = ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, pubkey);
    if key.verify(&msg, sig).is_err() {
        return Err(AuthzReject::BadSignature);
    }

    // 4. Channel binding — the token must be bound to THIS VPS's edge cert.
    if &tok.vps_cert_fp != expected_cert_fp {
        return Err(AuthzReject::CertFpMismatch);
    }

    // 5. Server nonce — single-use; consumed now that the token is cryptographically valid + bound.
    nonce_store.check_and_consume(&tok.server_nonce, now)?;

    // 6. Validity window against TRUSTED time.
    if now >= tok.expiry_ms {
        return Err(AuthzReject::Expired);
    }
    if tok.ts_ms > now + TOKEN_SKEW_MS {
        return Err(AuthzReject::NotYetValid);
    }

    // 7. Replay — record the jti last (bounded store holds only live-at-accept tokens). Use the
    //    instance id as the replay-store client scope (a jti is unique per operator/VPS pair). `iat`
    //    for the drift gate is the token's mint time `ts_ms`.
    jti_store
        .check_and_record(&tok.vps_instance_id, &tok.jti, tok.ts_ms, now)
        .map_err(|e| match e {
            JtiReject::Replayed => AuthzReject::Replayed,
            // A jti that drifts outside the replay store's acceptance window after we already passed
            // the explicit expiry/not-yet-valid checks is treated as a replay-class refusal
            // (fail-closed: never accept). In practice the explicit window above is tighter, so this
            // is defense-in-depth.
            JtiReject::ClockDriftPast | JtiReject::ClockDriftFuture => AuthzReject::Expired,
            JtiReject::StoreFull => AuthzReject::Replayed,
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seam::SystemClockTrustedTime;
    use ring::rand::SystemRandom;
    use ring::signature::{Ed25519KeyPair, KeyPair};

    /// A fixed trusted-time source for deterministic verification tests.
    struct FixedTrustedTime(Option<i64>);
    impl TrustedTime for FixedTrustedTime {
        fn now_ms(&self) -> Option<i64> {
            self.0
        }
    }

    const NOW: i64 = 1_000_000_000;
    const CERT_FP: [u8; 32] = [0xABu8; 32];

    /// Generate an Ed25519 keypair from a fresh 32-byte seed; return `(seed, pubkey32)`.
    fn keypair() -> (Zeroizing<[u8; 32]>, [u8; 32]) {
        let rng = SystemRandom::new();
        let mut seed = Zeroizing::new([0u8; 32]);
        ring::rand::SecureRandom::fill(&rng, seed.as_mut()).expect("seed");
        let kp = Ed25519KeyPair::from_seed_unchecked(seed.as_ref()).expect("kp");
        let mut pk = [0u8; 32];
        pk.copy_from_slice(kp.public_key().as_ref());
        (seed, pk)
    }

    /// A token + the live nonce it carries (issued from `store`), valid at `NOW`.
    fn issue_token(nonce_store: &mut NonceStore) -> PresenceToken {
        let rng = SystemRandom::new();
        let nonce = nonce_store.issue(NOW, &rng).expect("issue nonce");
        PresenceToken::new(
            NOW,
            "vps-1".to_string(),
            nonce,
            CERT_FP,
            NOW + DEFAULT_TOKEN_TTL_MS,
            "jti-1".to_string(),
        )
    }

    #[test]
    fn signing_bytes_are_domain_separated_and_length_prefixed() {
        let t = PresenceToken::new(NOW, "vps".into(), "n".into(), CERT_FP, NOW + 1, "j".into());
        let bytes = presence_token_signing_bytes(&t);
        assert!(bytes.starts_with(PRESENCE_TOKEN_DOMAIN));
        // A different vps_instance_id must NOT collide with a different server_nonce that happens to
        // concatenate the same way without length prefixes.
        let a = PresenceToken::new(NOW, "ab".into(), "c".into(), CERT_FP, NOW + 1, "j".into());
        let b = PresenceToken::new(NOW, "a".into(), "bc".into(), CERT_FP, NOW + 1, "j".into());
        assert_ne!(
            presence_token_signing_bytes(&a),
            presence_token_signing_bytes(&b),
            "length-prefixing prevents a boundary-shift collision"
        );
    }

    #[test]
    fn sign_then_verify_roundtrips() {
        let (seed, pk) = keypair();
        let mut nonce_store = NonceStore::new();
        let mut jti_store = JtiReplayStore::new();
        let tok = issue_token(&mut nonce_store);
        let sig = sign_presence_token(&seed, &tok).expect("sign");
        let tt = FixedTrustedTime(Some(NOW));
        assert_eq!(
            verify_presence_token(
                &pk,
                &tok,
                &sig,
                &CERT_FP,
                &tt,
                &mut nonce_store,
                &mut jti_store
            ),
            Ok(())
        );
    }

    #[test]
    fn reject_malformed_version_before_crypto() {
        let (seed, pk) = keypair();
        let mut nonce_store = NonceStore::new();
        let mut jti_store = JtiReplayStore::new();
        let mut tok = issue_token(&mut nonce_store);
        let sig = sign_presence_token(&seed, &tok).expect("sign");
        tok.v = 2; // unsupported
        let tt = FixedTrustedTime(Some(NOW));
        assert_eq!(
            verify_presence_token(
                &pk,
                &tok,
                &sig,
                &CERT_FP,
                &tt,
                &mut nonce_store,
                &mut jti_store
            ),
            Err(AuthzReject::MalformedVersion)
        );
    }

    #[test]
    fn reject_when_trusted_time_unavailable() {
        let (seed, pk) = keypair();
        let mut nonce_store = NonceStore::new();
        let mut jti_store = JtiReplayStore::new();
        let tok = issue_token(&mut nonce_store);
        let sig = sign_presence_token(&seed, &tok).expect("sign");
        let tt = FixedTrustedTime(None); // OI-SM-3: no fresh external time
        assert_eq!(
            verify_presence_token(
                &pk,
                &tok,
                &sig,
                &CERT_FP,
                &tt,
                &mut nonce_store,
                &mut jti_store
            ),
            Err(AuthzReject::TrustedTimeUnavailable)
        );
    }

    #[test]
    fn reject_bad_signature() {
        let (seed, pk) = keypair();
        let mut nonce_store = NonceStore::new();
        let mut jti_store = JtiReplayStore::new();
        let tok = issue_token(&mut nonce_store);
        let mut sig = sign_presence_token(&seed, &tok).expect("sign");
        sig[0] ^= 0x01; // flip a bit
        let tt = FixedTrustedTime(Some(NOW));
        assert_eq!(
            verify_presence_token(
                &pk,
                &tok,
                &sig,
                &CERT_FP,
                &tt,
                &mut nonce_store,
                &mut jti_store
            ),
            Err(AuthzReject::BadSignature)
        );
    }

    #[test]
    fn reject_cert_fp_mismatch() {
        let (seed, pk) = keypair();
        let mut nonce_store = NonceStore::new();
        let mut jti_store = JtiReplayStore::new();
        let tok = issue_token(&mut nonce_store);
        let sig = sign_presence_token(&seed, &tok).expect("sign");
        let other_fp = [0x11u8; 32];
        let tt = FixedTrustedTime(Some(NOW));
        assert_eq!(
            verify_presence_token(
                &pk,
                &tok,
                &sig,
                &other_fp,
                &tt,
                &mut nonce_store,
                &mut jti_store
            ),
            Err(AuthzReject::CertFpMismatch)
        );
    }

    #[test]
    fn reject_unknown_nonce() {
        let (seed, pk) = keypair();
        let mut nonce_store = NonceStore::new();
        let mut jti_store = JtiReplayStore::new();
        // A token whose nonce was never issued by this store.
        let tok = PresenceToken::new(
            NOW,
            "vps-1".into(),
            "never-issued".into(),
            CERT_FP,
            NOW + DEFAULT_TOKEN_TTL_MS,
            "jti-1".into(),
        );
        let sig = sign_presence_token(&seed, &tok).expect("sign");
        let tt = FixedTrustedTime(Some(NOW));
        assert_eq!(
            verify_presence_token(
                &pk,
                &tok,
                &sig,
                &CERT_FP,
                &tt,
                &mut nonce_store,
                &mut jti_store
            ),
            Err(AuthzReject::NonceUnknown)
        );
    }

    #[test]
    fn reject_expired() {
        let (seed, pk) = keypair();
        let mut nonce_store = NonceStore::new();
        let mut jti_store = JtiReplayStore::new();
        // Short-lived token (expiry NOW+1000) so the check at NOW+1001 is past expiry while the
        // freshly-issued server nonce (300s TTL) is still live — isolating the Expired verdict from
        // a stale-nonce verdict (the nonce is consumed in step 5, before the expiry check).
        let rng = SystemRandom::new();
        let nonce = nonce_store.issue(NOW, &rng).expect("issue");
        let tok = PresenceToken::new(
            NOW,
            "vps-1".into(),
            nonce,
            CERT_FP,
            NOW + 1_000,
            "jti-exp".into(),
        );
        let sig = sign_presence_token(&seed, &tok).expect("sign");
        // Trusted time has advanced past the token's expiry but well within the nonce's TTL.
        let tt = FixedTrustedTime(Some(NOW + 1_001));
        assert_eq!(
            verify_presence_token(
                &pk,
                &tok,
                &sig,
                &CERT_FP,
                &tt,
                &mut nonce_store,
                &mut jti_store
            ),
            Err(AuthzReject::Expired)
        );
    }

    #[test]
    fn reject_not_yet_valid() {
        let (seed, pk) = keypair();
        let mut nonce_store = NonceStore::new();
        let mut jti_store = JtiReplayStore::new();
        let rng = SystemRandom::new();
        let nonce = nonce_store.issue(NOW, &rng).expect("issue");
        // Minted far in the future relative to trusted-now (beyond skew).
        let tok = PresenceToken::new(
            NOW + TOKEN_SKEW_MS + 5_000,
            "vps-1".into(),
            nonce,
            CERT_FP,
            NOW + TOKEN_SKEW_MS + 5_000 + DEFAULT_TOKEN_TTL_MS,
            "jti-future".into(),
        );
        let sig = sign_presence_token(&seed, &tok).expect("sign");
        let tt = FixedTrustedTime(Some(NOW));
        assert_eq!(
            verify_presence_token(
                &pk,
                &tok,
                &sig,
                &CERT_FP,
                &tt,
                &mut nonce_store,
                &mut jti_store
            ),
            Err(AuthzReject::NotYetValid)
        );
    }

    #[test]
    fn reject_replay_same_jti() {
        let (seed, pk) = keypair();
        let mut nonce_store = NonceStore::new();
        let mut jti_store = JtiReplayStore::new();
        let rng = SystemRandom::new();
        // First token (fresh nonce #1) accepts.
        let n1 = nonce_store.issue(NOW, &rng).expect("n1");
        let tok1 = PresenceToken::new(
            NOW,
            "vps-1".into(),
            n1,
            CERT_FP,
            NOW + DEFAULT_TOKEN_TTL_MS,
            "jti-dup".into(),
        );
        let sig1 = sign_presence_token(&seed, &tok1).expect("sign1");
        let tt = FixedTrustedTime(Some(NOW));
        assert_eq!(
            verify_presence_token(
                &pk,
                &tok1,
                &sig1,
                &CERT_FP,
                &tt,
                &mut nonce_store,
                &mut jti_store
            ),
            Ok(())
        );
        // Second token: SAME jti, a FRESH nonce #2 (so the nonce check passes and the replay check
        // is what fires) ⇒ Replayed.
        let n2 = nonce_store.issue(NOW, &rng).expect("n2");
        let tok2 = PresenceToken::new(
            NOW,
            "vps-1".into(),
            n2,
            CERT_FP,
            NOW + DEFAULT_TOKEN_TTL_MS,
            "jti-dup".into(),
        );
        let sig2 = sign_presence_token(&seed, &tok2).expect("sign2");
        assert_eq!(
            verify_presence_token(
                &pk,
                &tok2,
                &sig2,
                &CERT_FP,
                &tt,
                &mut nonce_store,
                &mut jti_store
            ),
            Err(AuthzReject::Replayed)
        );
    }

    #[test]
    fn nonce_is_single_use_across_two_tokens() {
        // Re-presenting the SAME nonce with a different valid token is refused (single-use).
        let (seed, pk) = keypair();
        let mut nonce_store = NonceStore::new();
        let mut jti_store = JtiReplayStore::new();
        let tok1 = issue_token(&mut nonce_store);
        let sig1 = sign_presence_token(&seed, &tok1).expect("sign1");
        let tt = FixedTrustedTime(Some(NOW));
        assert_eq!(
            verify_presence_token(
                &pk,
                &tok1,
                &sig1,
                &CERT_FP,
                &tt,
                &mut nonce_store,
                &mut jti_store
            ),
            Ok(())
        );
        // Reuse tok1's (now-consumed) nonce in a new token with a fresh jti.
        let tok2 = PresenceToken::new(
            NOW,
            "vps-1".into(),
            tok1.server_nonce.clone(),
            CERT_FP,
            NOW + DEFAULT_TOKEN_TTL_MS,
            "jti-2".into(),
        );
        let sig2 = sign_presence_token(&seed, &tok2).expect("sign2");
        assert_eq!(
            verify_presence_token(
                &pk,
                &tok2,
                &sig2,
                &CERT_FP,
                &tt,
                &mut nonce_store,
                &mut jti_store
            ),
            Err(AuthzReject::NonceUnknown)
        );
    }

    #[test]
    fn system_clock_trusted_time_is_always_some() {
        assert!(SystemClockTrustedTime.now_ms().is_some());
    }
}
