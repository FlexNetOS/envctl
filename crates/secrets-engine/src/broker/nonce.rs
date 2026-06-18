//! Bounded, in-memory, per-process server-issued DPoP-Nonce store (OI-SM-1 nonce half; RFC 9449
//! §8–9 `DPoP-Nonce`). Sibling to [`crate::broker::jti::JtiReplayStore`]: same bounded, fail-closed
//! shape. The F2 relay edge (TASK-0031-PR2) issues a fresh nonce on a missing/unknown/expired one
//! and challenges the client (`401 + DPoP-Nonce`); a present+valid nonce is consumed **single-use**
//! so a captured proof+nonce cannot be replayed (a genuine retry simply re-challenges → fresh
//! nonce → fresh jti, the strongest replay posture).
//!
//! Semantics (all under the caller's single `Mutex` — the edge owns it, the check-and-consume is
//! atomic):
//! 1. **issue.** Sweep expired entries, then (if room) mint a fresh `NONCE_LEN`-byte random nonce,
//!    hex-encode it, record it with expiry `now + NONCE_TTL_MS`, and return it. A full store
//!    *after* the sweep ⇒ `Err(())` so the caller fails closed (a 401 with NO nonce — never an
//!    accept-on-error path, never an eviction of a live nonce).
//! 2. **check_and_consume.** A present, unexpired nonce is REMOVED and accepted (single-use); a
//!    nonce we never issued ⇒ `Unknown`; a known-but-expired nonce ⇒ `Expired` (and is dropped); an
//!    empty/missing value ⇒ `Missing`.
//!
//! Fail-closed everywhere: every uncertain outcome REJECTS. No secret bytes are stored — a nonce is
//! a public anti-replay challenge, not a credential. Sync, non-printing, `std` + `ring::rand` only
//! (the RNG is injected as a `&dyn ring::rand::SecureRandom` trait object so the engine stays pure
//! and the edge supplies one `SystemRandom`). ZERO new deps (`ring` is already in the resolved graph
//! via the rustls ring provider).

use std::collections::HashMap;

use ring::rand::SecureRandom;

/// How long (ms) a server-issued nonce stays valid. Coherent with the F6 `ACCEPT_PAST_MS` (5 min):
/// a nonce older than the DPoP acceptance window is useless anyway, so they expire together.
pub const NONCE_TTL_MS: i64 = 300_000;

/// Hard size cap (DoS backstop) on outstanding un-consumed nonces. At ~64 B/entry this is ≈ 1 MiB
/// worst-case — the explicit memory-exhaustion bound against a flood of `issue` calls. A full store
/// (post-sweep) refuses to mint (fail-closed), never evicts a live nonce to make room.
pub const MAX_NONCES: usize = 16_384;

/// Random bytes per nonce before hex encoding (256 bits — ample unpredictability so a nonce cannot
/// be guessed/forged by a client).
pub const NONCE_LEN: usize = 32;

/// Why a `check_and_consume` rejected. Every variant is a REJECT; the edge maps all three to a fresh
/// nonce challenge (401 + `DPoP-Nonce`). There is no accept-on-error variant by construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NonceReject {
    /// The proof carried no nonce claim (empty/absent) — the client must use a server-issued one.
    Missing,
    /// The nonce was not one this store issued (or was already consumed — single-use).
    Unknown,
    /// The nonce was issued by this store but has passed its `NONCE_TTL_MS` window.
    Expired,
}

/// Bounded, in-memory, per-process server-issued DPoP-Nonce store. Holds no secret material — only
/// the issued nonce string and its expiry instant. Not interior-mutable: the caller (the edge) owns
/// the single `Mutex<NonceStore>` so issue/check-and-consume are atomic.
pub struct NonceStore {
    ttl_ms: i64,
    max: usize,
    /// key = the hex nonce string, value = expiry instant (wall-ms).
    issued: HashMap<String, i64>,
}

impl NonceStore {
    /// A store with the audited defaults (`NONCE_TTL_MS` / `MAX_NONCES`).
    pub fn new() -> Self {
        Self {
            ttl_ms: NONCE_TTL_MS,
            max: MAX_NONCES,
            issued: HashMap::new(),
        }
    }

    /// A store with a tuned TTL + cap (lets a test shrink the window/size as a one-liner).
    pub fn with_params(ttl_ms: i64, max: usize) -> Self {
        Self {
            ttl_ms,
            max,
            issued: HashMap::new(),
        }
    }

    /// Mint a fresh single-use nonce: sweep expired entries FIRST (so the cap reflects only live
    /// nonces), then — if there is room — generate `NONCE_LEN` random bytes via the injected RNG,
    /// hex-encode them, record the value with expiry `now + ttl`, and return it. Returns
    /// `Err(())` when the store is full AFTER the sweep (fail-closed: the caller returns a 401 with NO
    /// nonce rather than evicting a live one) or when the RNG fails (never accept on a crypto error).
    ///
    /// The error carries no payload by design: every failure means the SAME thing to the edge — issue
    /// no nonce, fail closed (a bare 401). A typed error would add a discriminant the caller never
    /// branches on, so `()` is the honest signature here (clippy's `result_unit_err` is suppressed for
    /// this one method only).
    #[allow(clippy::result_unit_err)]
    pub fn issue(&mut self, now_ms: i64, rng: &dyn SecureRandom) -> Result<String, ()> {
        // (1) Sweep — drop every entry whose expiry has passed (amortized, bounded reclamation).
        self.issued.retain(|_, &mut exp| exp > now_ms);

        // (2) Cap — fail-closed backstop: refuse to mint, never evict a live nonce.
        if self.issued.len() >= self.max {
            return Err(());
        }

        // (3) Generate — `NONCE_LEN` random bytes; an RNG failure is a fail-closed Err (never mint a
        // weak/zero nonce). Encode as lowercase hex (std-only — the engine pulls in no base64 dep on
        // this always-built path); the nonce is an opaque, public anti-replay token.
        let mut raw = [0u8; NONCE_LEN];
        rng.fill(&mut raw).map_err(|_| ())?;
        let nonce = hex_encode(&raw);

        // (4) Record — remember until `now + ttl`. A collision on an already-issued (still-live) value
        // is astronomically unlikely; on the off chance, we simply refresh its expiry (still single
        // entry) — the value remains single-use on consume.
        self.issued.insert(nonce.clone(), now_ms + self.ttl_ms);
        Ok(nonce)
    }

    /// Atomic check-and-consume: a present, unexpired nonce is REMOVED (single-use) and accepted; any
    /// other outcome REJECTS with a typed reason. Times are wall-ms `i64` supplied by the caller
    /// (deterministic — no clock dep). An empty value is `Missing`; an unknown value is `Unknown`; a
    /// known-but-stale value is `Expired` (and dropped so it cannot be re-tried).
    pub fn check_and_consume(&mut self, nonce: &str, now_ms: i64) -> Result<(), NonceReject> {
        if nonce.is_empty() {
            return Err(NonceReject::Missing);
        }
        match self.issued.remove(nonce) {
            // Never issued (or already consumed — single-use means a second use looks unknown).
            None => Err(NonceReject::Unknown),
            // Issued but past its window — dropped on the `remove` above; reject as expired.
            Some(exp) if exp <= now_ms => Err(NonceReject::Expired),
            // Present + live — consumed (removed) and accepted.
            Some(_) => Ok(()),
        }
    }

    /// Live entry count (test-only — the production type exposes no map internals).
    #[cfg(test)]
    fn len(&self) -> usize {
        self.issued.len()
    }
}

impl Default for NonceStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Lowercase-hex encode bytes (std-only; the engine adds no base64 dependency on the always-built
/// nonce path). A nonce is an opaque public token, so any unambiguous text encoding is fine.
fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ring::rand::SystemRandom;
    use std::sync::{Arc, Mutex};
    use std::thread;

    const NOW: i64 = 1_000_000_000;

    // `ring::rand::SecureRandom` is a SEALED trait — no custom (seeded) impl is constructible outside
    // ring. The nonce tests therefore inject the real `SystemRandom`: they never assert a specific
    // nonce VALUE (only accept/reject behavior + bounds), and a real RNG yields the distinct,
    // unpredictable nonces these tests need. Clock is still injected (`now_ms`) so timing is exact.
    #[test]
    fn issue_then_consume_accepts_once() {
        let mut s = NonceStore::new();
        let rng = SystemRandom::new();
        let n = s.issue(NOW, &rng).expect("issue");
        assert_eq!(s.len(), 1);
        // First consume of a live nonce → Ok, and it is removed (single-use).
        assert_eq!(s.check_and_consume(&n, NOW), Ok(()));
        assert_eq!(s.len(), 0, "a consumed nonce must be removed (single-use)");
    }

    #[test]
    fn second_consume_is_unknown() {
        let mut s = NonceStore::new();
        let rng = SystemRandom::new();
        let n = s.issue(NOW, &rng).expect("issue");
        assert_eq!(s.check_and_consume(&n, NOW), Ok(()));
        // A second use of the SAME nonce looks unknown (it was removed on the first accept).
        assert_eq!(
            s.check_and_consume(&n, NOW),
            Err(NonceReject::Unknown),
            "single-use: re-consuming a spent nonce is Unknown"
        );
    }

    #[test]
    fn expired_nonce_rejected() {
        let mut s = NonceStore::with_params(1_000, MAX_NONCES);
        let rng = SystemRandom::new();
        let n = s.issue(NOW, &rng).expect("issue");
        // Consume just past the TTL window → Expired (and the entry is dropped).
        let later = NOW + 1_001;
        assert_eq!(s.check_and_consume(&n, later), Err(NonceReject::Expired));
        assert_eq!(s.len(), 0, "an expired nonce is dropped on consume");
    }

    #[test]
    fn missing_nonce_rejected() {
        let mut s = NonceStore::new();
        assert_eq!(s.check_and_consume("", NOW), Err(NonceReject::Missing));
    }

    #[test]
    fn unknown_nonce_rejected() {
        let mut s = NonceStore::new();
        assert_eq!(
            s.check_and_consume("never-issued-value", NOW),
            Err(NonceReject::Unknown)
        );
    }

    #[test]
    fn full_store_fails_closed_on_issue() {
        // A small cap; all entries live (issued at NOW), so the sweep reclaims none.
        let cap = 4;
        let mut s = NonceStore::with_params(NONCE_TTL_MS, cap);
        let rng = SystemRandom::new();
        for _ in 0..cap {
            assert!(s.issue(NOW, &rng).is_ok());
        }
        assert_eq!(s.len(), cap);
        // The (cap + 1)-th issue is refused — never an eviction of a live nonce.
        assert_eq!(s.issue(NOW, &rng), Err(()));
        assert_eq!(s.len(), cap, "a full store must not grow or evict on issue");
    }

    #[test]
    fn sweep_then_issue_reclaims_room() {
        // The cap is not a permanent wall: once nonces time-expire, the next issue's sweep frees room.
        let cap = 2;
        let mut s = NonceStore::with_params(1_000, cap);
        let rng = SystemRandom::new();
        assert!(s.issue(NOW, &rng).is_ok());
        assert!(s.issue(NOW, &rng).is_ok());
        // Full at NOW.
        assert_eq!(s.issue(NOW, &rng), Err(()));
        // Advance past expiry: the sweep on the next issue reclaims both, so a fresh mint succeeds.
        let later = NOW + 1_001;
        assert!(s.issue(later, &rng).is_ok());
        assert_eq!(s.len(), 1, "the sweep reclaimed the two expired nonces");
    }

    #[test]
    fn concurrent_consume_single_winner() {
        // N threads race to consume the SAME issued nonce through a shared Mutex<NonceStore>. The
        // single-use remove must yield EXACTLY one Ok and N-1 Err(Unknown).
        let store = Arc::new(Mutex::new(NonceStore::new()));
        let nonce = {
            let rng = SystemRandom::new();
            let mut g = store.lock().unwrap();
            g.issue(NOW, &rng).expect("issue")
        };
        let n = 32;
        let handles: Vec<_> = (0..n)
            .map(|_| {
                let store = Arc::clone(&store);
                let nonce = nonce.clone();
                thread::spawn(move || {
                    let mut g = store.lock().expect("lock not poisoned");
                    g.check_and_consume(&nonce, NOW)
                })
            })
            .collect();

        let mut oks = 0;
        let mut unknowns = 0;
        for h in handles {
            match h.join().expect("thread panicked") {
                Ok(()) => oks += 1,
                Err(NonceReject::Unknown) => unknowns += 1,
                Err(other) => panic!("unexpected reject: {other:?}"),
            }
        }
        assert_eq!(oks, 1, "exactly one concurrent winner consumes the nonce");
        assert_eq!(unknowns, n - 1, "all others see Unknown (already consumed)");
        assert_eq!(store.lock().unwrap().len(), 0);
    }
}
