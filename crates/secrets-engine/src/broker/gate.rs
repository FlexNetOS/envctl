//! Presence-gate abstraction (audit F14, SERVER-MODE §5.1).
//!
//! The engine's sole egress gate input is generalized from a USB-specific
//! `usb_absent_since_ms` to a topology-agnostic `gate_absent_since_ms`, fed by a single
//! [`PresenceGate`]. Profile A (the daemon runs on the box that physically holds the USB)
//! implements it with a USB-possession probe; Profile B (VPS) would implement it with an
//! operator-box presence-token verifier. `decide()` consumes ONLY the resolved
//! `gate_absent_since_ms`, so there is one choke point and a substitute factor can never be
//! bolted beside `decide()` (REQ-SEC-13).

/// Resolved presence-gate state.
///
/// `decide()` treats [`GateState::Unproven`] EXACTLY like [`GateState::AbsentSince`]`(now)` —
/// immediate deny, no grace — so an unproven/unconfigured/misread gate fails closed
/// (SERVER-MODE §5.1, REQ-SEC-13).
///
/// Representation note: the absent-since timestamp is carried as wall-clock epoch
/// milliseconds (`i64`), consistent with every other engine gate/clock input (`now_ms`,
/// `issuance_floor_ms`) and the `gate_absent_since_ms: Option<i64>` that `decide()` consumes.
/// This is a deliberate deviation from the illustrative `AbsentSince(Instant)` in the spec
/// sketch: an `Instant` has no epoch anchor and would force a lossy conversion at the single
/// mapping site ([`gate_absent_since_ms`]); the engine is wall-ms throughout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateState {
    /// The presence factor is currently proven (USB present / a valid operator-box token).
    Present,
    /// The factor has been absent since this wall-clock epoch-ms reading.
    AbsentSince(i64),
    /// The factor's state cannot be established (no probe, no valid token, gate
    /// misconfigured). Fails closed: mapped to `AbsentSince(now)` — immediate deny, no grace.
    Unproven,
}

/// The daemon-side presence factor. Profile A: a USB-possession probe. Profile B: an
/// operator-box presence-token verifier. Defined engine-side so the gate is a single typed
/// choke point the daemon injects into — the engine itself never opens a socket, reads a
/// USB, or verifies a token.
pub trait PresenceGate: Send + Sync {
    /// Resolve the current gate state. Implementations MUST fail closed
    /// ([`GateState::Unproven`]) on any uncertainty rather than guessing `Present`.
    fn resolve(&self) -> GateState;
}

/// Forward through an `Arc` so the daemon can hold an `Arc<VpsPresenceGate>` (to feed verified
/// tokens from the async authorizer task) AND pass a `Box::new(arc.clone())` into `with_seams` —
/// both the engine read path and the authorizer write path then share ONE gate.
impl<G: PresenceGate + ?Sized> PresenceGate for std::sync::Arc<G> {
    fn resolve(&self) -> GateState {
        (**self).resolve()
    }
}

/// Map a resolved [`GateState`] to the `gate_absent_since_ms` value `decide()` consumes.
///
/// [`GateState::Unproven`] collapses to `AbsentSince(now_ms)` (REQ-SEC-13: treated exactly
/// like absent-now), so any uncertainty denies immediately with no grace.
#[must_use]
pub fn gate_absent_since_ms(state: GateState, now_ms: i64) -> Option<i64> {
    match state {
        GateState::Present => None,
        GateState::AbsentSince(ms) => Some(ms),
        GateState::Unproven => Some(now_ms),
    }
}

/// The default-injected presence gate: ALWAYS [`GateState::Unproven`]. It is the fail-closed
/// engine default for the Profile-B gate seam — a daemon that has not installed a real gate (or a
/// VPS whose authorizer link has never delivered a valid token) can prove nothing, so it denies
/// immediately with no grace (REQ-SEC-13). Profile A never consults the injected gate (it keeps the
/// on-box USB/Seed path), so this default is invisible to default builds.
pub struct UnprovenGate;

impl PresenceGate for UnprovenGate {
    fn resolve(&self) -> GateState {
        GateState::Unproven
    }
}

/// Profile B (VPS) presence gate (audit F9 / FS-S23 / OI-SM-2). Possession is substituted by an
/// operator-box-signed **presence token**: the async authorizer link (secretd `edge::authorizer`)
/// verifies a token and feeds its `valid_until` (the token's `expiry_ms`) here via
/// [`VpsPresenceGate::accept_token`]. `resolve()` re-reads the stored `valid_until` against the
/// engine clock on EVERY swap (the engine never caches a gate result across swaps — FS-S23), so a
/// token expiring between two swaps flips the 2nd swap to absent.
///
/// Fail-closed: never-proven ⇒ [`GateState::Unproven`] (deny, no grace); a token that has expired ⇒
/// [`GateState::AbsentSince`]`(expiry)`. Interior-mutable (`Mutex`) so the async authorizer writes
/// while the per-request egress path reads. A poisoned lock resolves to `Unproven` (fail-closed).
pub struct VpsPresenceGate {
    /// `Some(valid_until_wall_ms)` of the most-recently accepted token, else `None` (never proven).
    valid_until_ms: std::sync::Mutex<Option<i64>>,
    /// Engine wall clock used to compare `valid_until` against "now" on each `resolve()`.
    clock: std::boxed::Box<dyn crate::seam::Clock>,
}

impl VpsPresenceGate {
    /// Build a never-proven VPS gate over the given engine clock. The authorizer link must call
    /// [`Self::accept_token`] before any swap can pass the gate.
    #[must_use]
    pub fn new(clock: std::boxed::Box<dyn crate::seam::Clock>) -> Self {
        Self {
            valid_until_ms: std::sync::Mutex::new(None),
            clock,
        }
    }

    /// Record that a presence token valid until `valid_until_ms` (its `expiry_ms`) was just verified
    /// by the authorizer link. Subsequent `resolve()` calls report `Present` until the engine clock
    /// passes `valid_until_ms`. A poisoned lock is ignored (the read side fails closed anyway).
    pub fn accept_token(&self, valid_until_ms: i64) {
        if let Ok(mut g) = self.valid_until_ms.lock() {
            *g = Some(valid_until_ms);
        }
    }

    /// Drop any accepted token so the gate reverts to never-proven (`Unproven`). Called when the
    /// authorizer link goes unreachable (FS-S23: deny new egress while possession can't be re-proven).
    pub fn clear(&self) {
        if let Ok(mut g) = self.valid_until_ms.lock() {
            *g = None;
        }
    }
}

impl PresenceGate for VpsPresenceGate {
    fn resolve(&self) -> GateState {
        let Ok(g) = self.valid_until_ms.lock() else {
            return GateState::Unproven; // poisoned ⇒ fail closed
        };
        match *g {
            None => GateState::Unproven,
            Some(valid_until) => {
                let now = self.clock.now().timestamp_millis();
                if now < valid_until {
                    GateState::Present
                } else {
                    // Expired: absent since the token's expiry instant (deterministic, not "now").
                    GateState::AbsentSince(valid_until)
                }
            }
        }
    }
}

/// Profile S — the **Cognitum Seed** presence gate. Possession is proven *freshly* on each
/// `resolve()`: a random 32-byte challenge is signed by the Seed's Ed25519 device key (over the
/// REST custody API, TLS-pinned to the Cognitum CA, via [`crate::seam::seed_factor::sign_hex`])
/// and the returned signature is
/// verified with `ring` against the operator-pinned device public key. A fresh nonce each call
/// makes the check replay-proof; verification against the pinned key authenticates the responder.
///
/// Fails closed to [`GateState::Unproven`] on ANY uncertainty: no pinned key configured, Seed
/// unreachable/unpaired, malformed signature, or verification failure. (A brief-blip *grace*
/// window — `AbsentSince` tracking — is a deliberate later refinement; Unproven denies with no
/// grace, which is strictly safe.)
#[cfg(feature = "seed-factor")]
pub struct SeedPresenceGate {
    /// Operator-pinned 32-byte Ed25519 device public key. `None` ⇒ `resolve()` is `Unproven`.
    pubkey: Option<[u8; 32]>,
}

#[cfg(feature = "seed-factor")]
impl SeedPresenceGate {
    /// Build from `ENVCTL_SEED_PUBKEY` (64 hex chars = the raw Ed25519 device key). Absent or
    /// malformed ⇒ the gate is `Unproven` (fail-closed).
    #[must_use]
    pub fn from_env() -> Self {
        let pubkey = std::env::var("ENVCTL_SEED_PUBKEY")
            .ok()
            .and_then(|h| decode_pubkey_hex(&h));
        Self { pubkey }
    }

    /// Build with an explicitly pinned device public key.
    #[must_use]
    pub fn with_pubkey(pubkey: [u8; 32]) -> Self {
        Self {
            pubkey: Some(pubkey),
        }
    }
}

/// Decode 64 hex chars into a 32-byte Ed25519 public key. `None` on wrong length / non-hex.
#[cfg(feature = "seed-factor")]
fn decode_pubkey_hex(s: &str) -> Option<[u8; 32]> {
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

/// Lowercase-hex encode bytes (the challenge `data` string the Seed signs).
#[cfg(feature = "seed-factor")]
fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

#[cfg(feature = "seed-factor")]
impl PresenceGate for SeedPresenceGate {
    fn resolve(&self) -> GateState {
        // No pinned key ⇒ we cannot authenticate the responder. Fail closed.
        let Some(pubkey) = self.pubkey else {
            return GateState::Unproven;
        };
        // Fresh random challenge — replay-proof.
        let mut nonce = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut nonce);
        let challenge = encode_hex(&nonce);
        // Ask the Seed to sign it, then verify the signature against the pinned device key.
        let Some(sig) = crate::seam::seed_factor::sign_hex(&challenge)
            .as_deref()
            .and_then(crate::seam::seed_factor::parse_sig_hex)
        else {
            return GateState::Unproven;
        };
        let key = ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, &pubkey);
        match key.verify(challenge.as_bytes(), &sig) {
            Ok(()) => GateState::Present,
            Err(_) => GateState::Unproven,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn present_opens_the_gate() {
        // Proven presence => no absence timestamp => `decide()`'s gate check passes.
        assert_eq!(gate_absent_since_ms(GateState::Present, 1_000), None);
    }

    #[test]
    fn absent_since_passes_the_timestamp_through() {
        assert_eq!(
            gate_absent_since_ms(GateState::AbsentSince(42), 1_000),
            Some(42)
        );
    }

    /// A test clock that returns a caller-set fixed wall-ms (the `boottime_ms` is irrelevant here).
    struct TestClock(std::sync::Mutex<i64>);
    impl crate::seam::Clock for TestClock {
        fn now(&self) -> chrono::DateTime<chrono::Utc> {
            chrono::DateTime::from_timestamp_millis(*self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner))
                .expect("valid timestamp")
        }
        fn boottime_ms(&self) -> i64 {
            0
        }
    }

    #[test]
    fn unproven_gate_is_always_unproven() {
        assert_eq!(UnprovenGate.resolve(), GateState::Unproven);
    }

    #[test]
    fn vps_gate_never_proven_is_unproven() {
        let clock = Box::new(TestClock(std::sync::Mutex::new(1_000)));
        let gate = VpsPresenceGate::new(clock);
        assert_eq!(
            gate.resolve(),
            GateState::Unproven,
            "a VPS gate with no accepted token denies (fail-closed)"
        );
    }

    #[test]
    fn vps_gate_valid_token_is_present() {
        let clock = Box::new(TestClock(std::sync::Mutex::new(1_000)));
        let gate = VpsPresenceGate::new(clock);
        gate.accept_token(5_000); // valid_until well ahead of now=1000
        assert_eq!(gate.resolve(), GateState::Present);
    }

    #[test]
    fn vps_gate_expired_token_is_absent_since_expiry() {
        // now (2000) is past valid_until (1500): absent since the token's expiry instant.
        let clock = Box::new(TestClock(std::sync::Mutex::new(2_000)));
        let gate = VpsPresenceGate::new(clock);
        gate.accept_token(1_500);
        assert_eq!(gate.resolve(), GateState::AbsentSince(1_500));
    }

    #[test]
    fn vps_gate_clear_reverts_to_unproven() {
        let clock = Box::new(TestClock(std::sync::Mutex::new(1_000)));
        let gate = VpsPresenceGate::new(clock);
        gate.accept_token(5_000);
        assert_eq!(gate.resolve(), GateState::Present);
        gate.clear();
        assert_eq!(
            gate.resolve(),
            GateState::Unproven,
            "after the authorizer goes unreachable, the gate denies again"
        );
    }

    #[test]
    fn vps_gate_reresolves_on_clock_advance() {
        // FS-S23: the gate is re-read against the clock on EVERY resolve — a token valid at one read
        // is absent at a later read once the clock passes its expiry, with no re-acceptance.
        let now = std::sync::Mutex::new(1_000);
        let clock_handle = std::sync::Arc::new(now);
        struct SharedClock(std::sync::Arc<std::sync::Mutex<i64>>);
        impl crate::seam::Clock for SharedClock {
            fn now(&self) -> chrono::DateTime<chrono::Utc> {
                chrono::DateTime::from_timestamp_millis(*self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner)).unwrap()
            }
            fn boottime_ms(&self) -> i64 {
                0
            }
        }
        let gate = VpsPresenceGate::new(Box::new(SharedClock(clock_handle.clone())));
        gate.accept_token(2_000);
        assert_eq!(gate.resolve(), GateState::Present, "valid at t=1000");
        *clock_handle.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = 2_500; // advance past expiry
        assert_eq!(
            gate.resolve(),
            GateState::AbsentSince(2_000),
            "same token now absent at t=2500 — re-resolved per call, not cached"
        );
    }

    #[test]
    fn unproven_is_absent_now_fail_closed() {
        // REQ-SEC-13: Unproven is treated EXACTLY like AbsentSince(now) — Some(now) drives the
        // GateAbsent deny in `decide()`, with no grace. A VPS/misconfigured gate denies here.
        assert_eq!(
            gate_absent_since_ms(GateState::Unproven, 1_000),
            Some(1_000)
        );
    }

    // Profile S (Cognitum Seed) — crypto verified offline with the real spike triple (2026-06-13).
    #[cfg(feature = "seed-factor")]
    mod seed {
        use super::super::{decode_pubkey_hex, SeedPresenceGate};
        use crate::broker::gate::{GateState, PresenceGate};

        // Device public key + a real Ed25519 signature over the message bytes
        // `b"envctl/usb-kek/v1/spike"`, captured from the live Seed.
        const PUBKEY_HEX: &str = "86e6121ebee4d34fcee94abf20bb5cd5f5bd7c5c04f89dc66950e09b0dc4bc06";
        const SIG_HEX: &str = "90017fccf53948ce509c216d1cf64c6cdd75d50a9f28e63cef27d6706a7b4c765de7a2849dc8c1d6b19f5ee6e3211b8142b669ca8b6c1fb16a6dc989dc5fa60e";
        const MSG: &[u8] = b"envctl/usb-kek/v1/spike";

        #[test]
        fn ring_verifies_real_seed_signature() {
            // Confirms BOTH the verifier wiring and the Seed's wire-format: the device signs the
            // raw UTF-8 bytes of `data` with standard Ed25519 (not prehashed/enveloped).
            let pubkey = decode_pubkey_hex(PUBKEY_HEX).expect("64-hex pubkey");
            let sig = crate::seam::seed_factor::parse_sig_hex(SIG_HEX).expect("128-hex sig");
            let key = ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, &pubkey);
            assert!(
                key.verify(MSG, &sig).is_ok(),
                "real Seed signature must verify"
            );
        }

        #[test]
        fn ring_rejects_tampered_signature() {
            let pubkey = decode_pubkey_hex(PUBKEY_HEX).unwrap();
            let mut sig = crate::seam::seed_factor::parse_sig_hex(SIG_HEX).unwrap();
            sig[0] ^= 0x01; // flip one bit
            let key = ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, &pubkey);
            assert!(
                key.verify(MSG, &sig).is_err(),
                "tampered signature must fail"
            );
        }

        #[test]
        fn ring_rejects_wrong_message() {
            let pubkey = decode_pubkey_hex(PUBKEY_HEX).unwrap();
            let sig = crate::seam::seed_factor::parse_sig_hex(SIG_HEX).unwrap();
            let key = ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, &pubkey);
            assert!(key.verify(b"different message", &sig).is_err());
        }

        #[test]
        fn no_pinned_key_is_unproven() {
            // Fail-closed without touching the network (early return before any SSH).
            let gate = SeedPresenceGate { pubkey: None };
            assert_eq!(gate.resolve(), GateState::Unproven);
        }

        #[test]
        fn decode_pubkey_hex_rejects_malformed() {
            assert!(decode_pubkey_hex("dead").is_none(), "too short");
            assert!(decode_pubkey_hex(&"zz".repeat(32)).is_none(), "non-hex");
        }
    }
}
