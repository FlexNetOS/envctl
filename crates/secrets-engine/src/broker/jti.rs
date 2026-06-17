//! Bounded, in-memory, per-process DPoP `jti` replay-dedup store (resolves audit open-item
//! OI-SM-1 / audit F6; SERVER-MODE §4.2). Makes each DPoP (RFC 9449) proof single-use within its
//! acceptance window so a captured proof+bearer cannot be replayed against the same endpoint
//! (THREAT-MODEL A14). The F2 edge listener that CALLS this store is TASK-0031 — NOT built here.
//!
//! Design corpus: `docs/secrets/OI-SM-1-jti-replay-store.md`.
//!
//! Semantics (all under the caller's single lock — §7 atomicity):
//! 1. **Drift gate.** A proof whose `iat` is outside `[now - ACCEPT_PAST, now + ACCEPT_FUTURE]`
//!    (boundaries inclusive) is rejected *before the store is consulted*. This is what bounds the
//!    store by time: a `jti` only needs to be remembered until `iat + ACCEPT_PAST + SWEEP_SLACK`,
//!    because after that any proof bearing it is rejected on the drift check anyway.
//! 2. **Sweep.** Opportunistically evict every entry whose stored expiry `<= now`.
//! 3. **Dedup.** A still-present `(client_id, jti)` is a replay → reject.
//! 4. **Cap.** If the (post-sweep) map is at `MAX_ENTRIES`, reject the NEW proof — a fail-closed
//!    backstop, never a live-entry LRU eviction (evicting a live entry would re-open the replay
//!    window for the evicted `jti`). See OI-SM-1 §4.
//! 5. **Record.** Insert with expiry `iat + ACCEPT_PAST + SWEEP_SLACK` → accept.
//!
//! Fail-closed everywhere: every uncertain outcome REJECTS; there is no accept-on-error path.
//! No secret bytes are stored — only the `jti` (a public proof identifier) keyed by `client_id`,
//! plus an expiry int. Sync, non-printing, zero new deps (`std::collections` only).

use std::collections::HashMap;

/// Maximum age (ms) of a DPoP proof's `iat` relative to `now` before it is rejected on the drift
/// gate. Audit F6's recommended 5-min validity window; also the store's primary time bound.
pub const ACCEPT_PAST_MS: i64 = 300_000;

/// Maximum a DPoP proof's `iat` may lead `now` (ms) before rejection. A proof from the future is
/// almost always clock skew; a tight bound caps how long a pre-minted proof can wait to be replayed.
pub const ACCEPT_FUTURE_MS: i64 = 30_000;

/// Extra retention (ms) past `iat + ACCEPT_PAST_MS` so a `jti` is remembered until the latest
/// moment a replay of it could still arrive (covers `now` advancing between insert and replay).
pub const SWEEP_SLACK_MS: i64 = 30_000;

/// Hard size cap (DoS backstop). At ~64 B/entry this is ≈ 1 MiB worst-case — the explicit
/// memory-exhaustion bound against a flood of unique-`jti` proofs (OI-SM-1 §4).
pub const MAX_ENTRIES: usize = 16_384;

/// Why a `check_and_record` rejected a proof. Every variant is a REJECT; the edge maps all four to
/// a 401 (proof failures are 401 at the edge). There is no accept-on-error variant by construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JtiReject {
    /// `(client_id, jti)` was already seen and is still within its retention window.
    Replayed,
    /// `iat` is older than `now - ACCEPT_PAST_MS` — outside the acceptance window (past).
    ClockDriftPast,
    /// `iat` is newer than `now + ACCEPT_FUTURE_MS` — outside the acceptance window (future).
    ClockDriftFuture,
    /// The store is at `max_entries` after the sweep; the NEW proof is refused (fail-closed cap).
    StoreFull,
}

/// Bounded, in-memory, per-process DPoP `jti` replay store. Holds no secret material — only the
/// `(client_id, jti)` key and an expiry instant per entry. Not interior-mutable: the caller (the
/// edge) owns the single `Mutex<JtiReplayStore>` so the check-and-record is atomic (§7).
pub struct JtiReplayStore {
    accept_past_ms: i64,
    accept_future_ms: i64,
    sweep_slack_ms: i64,
    max_entries: usize,
    /// key = `format!("{client_id}\u{0}{jti}")`, value = expiry instant (wall-ms).
    seen: HashMap<String, i64>,
}

impl JtiReplayStore {
    /// A store with the audited defaults (`ACCEPT_PAST_MS` / `ACCEPT_FUTURE_MS` / `SWEEP_SLACK_MS`
    /// / `MAX_ENTRIES`).
    pub fn new() -> Self {
        Self {
            accept_past_ms: ACCEPT_PAST_MS,
            accept_future_ms: ACCEPT_FUTURE_MS,
            sweep_slack_ms: SWEEP_SLACK_MS,
            max_entries: MAX_ENTRIES,
            seen: HashMap::new(),
        }
    }

    /// A store with tuned acceptance window + cap (keeps the default `SWEEP_SLACK_MS`). Lets the
    /// edge retune the window/size as a one-liner (e.g. once nonce binding absorbs larger drift).
    pub fn with_params(accept_past_ms: i64, accept_future_ms: i64, max_entries: usize) -> Self {
        Self {
            accept_past_ms,
            accept_future_ms,
            sweep_slack_ms: SWEEP_SLACK_MS,
            max_entries,
            seen: HashMap::new(),
        }
    }

    /// Atomic check-and-insert: the replay check IS the insert-if-absent. Returns `Ok(())` to
    /// ACCEPT the proof (it is fresh and now recorded) or `Err(JtiReject)` to REJECT. Times are
    /// wall-ms `i64` supplied by the caller (deterministic — no `Clock` dep). Steps run in the
    /// fixed order drift → sweep → dedup → cap → record so that:
    /// - a drift-rejected proof never touches the map,
    /// - the sweep reclaims time-expired entries before the cap is evaluated (so the cap reflects
    ///   only LIVE entries),
    /// - a full store rejects the NEW proof rather than evicting a live `jti` (no replay hole).
    pub fn check_and_record(
        &mut self,
        client_id: &str,
        jti: &str,
        iat_ms: i64,
        now_ms: i64,
    ) -> Result<(), JtiReject> {
        // (1) Drift gate — boundaries INCLUSIVE (== is Ok). Rejected before the store is consulted.
        if iat_ms < now_ms - self.accept_past_ms {
            return Err(JtiReject::ClockDriftPast);
        }
        if iat_ms > now_ms + self.accept_future_ms {
            return Err(JtiReject::ClockDriftFuture);
        }

        // (2) Sweep — drop every entry whose expiry has passed (amortized, bounded reclamation).
        self.seen.retain(|_, &mut exp| exp > now_ms);

        // (3) Dedup — a still-present key is a replay.
        let key = format!("{client_id}\u{0}{jti}");
        if self.seen.contains_key(&key) {
            return Err(JtiReject::Replayed);
        }

        // (4) Cap — fail-closed backstop: refuse the NEW proof, never evict a live entry.
        if self.seen.len() >= self.max_entries {
            return Err(JtiReject::StoreFull);
        }

        // (5) Record — remember until `iat + ACCEPT_PAST + SWEEP_SLACK`.
        self.seen
            .insert(key, iat_ms + self.accept_past_ms + self.sweep_slack_ms);
        Ok(())
    }

    /// Live entry count (test-only — the production type exposes no map internals).
    #[cfg(test)]
    fn len(&self) -> usize {
        self.seen.len()
    }
}

impl Default for JtiReplayStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;

    // Fixed wall-clock anchors (ms) — no real clock anywhere in these tests.
    const NOW: i64 = 1_000_000_000;

    #[test]
    fn first_use_accepted() {
        let mut store = JtiReplayStore::new();
        assert_eq!(
            store.check_and_record("client-a", "jti-1", NOW, NOW),
            Ok(())
        );
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn replay_rejected() {
        let mut store = JtiReplayStore::new();
        assert_eq!(
            store.check_and_record("client-a", "jti-1", NOW, NOW),
            Ok(())
        );
        // Same (client, jti) within the window → replay.
        assert_eq!(
            store.check_and_record("client-a", "jti-1", NOW, NOW),
            Err(JtiReject::Replayed)
        );
        assert_eq!(store.len(), 1, "a rejected replay must not add an entry");
    }

    #[test]
    fn different_clients_same_jti_both_accepted() {
        // A `jti` is only client-unique (RFC 9449): two distinct clients may emit the same value,
        // and per-client scoping must not let one block the other.
        let mut store = JtiReplayStore::new();
        assert_eq!(
            store.check_and_record("client-a", "jti-x", NOW, NOW),
            Ok(())
        );
        assert_eq!(
            store.check_and_record("client-b", "jti-x", NOW, NOW),
            Ok(())
        );
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn expired_then_fresh_same_value_accepted() {
        let mut store = JtiReplayStore::new();
        // Record at NOW (expiry = NOW + ACCEPT_PAST + SWEEP_SLACK).
        assert_eq!(
            store.check_and_record("client-a", "jti-1", NOW, NOW),
            Ok(())
        );

        // Advance `now` past the entry's expiry; a FRESH proof with the SAME jti value but a fresh
        // (in-window) iat is admitted — the sweep evicted the old entry, and re-admitting the value
        // is safe because any proof bearing the OLD iat is now drift-rejected anyway.
        let later = NOW + ACCEPT_PAST_MS + SWEEP_SLACK_MS + 1;
        assert_eq!(
            store.check_and_record("client-a", "jti-1", later, later),
            Ok(())
        );
        assert_eq!(store.len(), 1, "sweep must have evicted the stale entry");

        // A proof reusing the OLD iat (now far in the past relative to `later`) is drift-rejected
        // before the store — proving the stale value can never actually replay.
        assert_eq!(
            store.check_and_record("client-a", "jti-1", NOW, later),
            Err(JtiReject::ClockDriftPast)
        );
    }

    #[test]
    fn clock_drift_past_rejected() {
        let mut store = JtiReplayStore::new();
        // Just past the past edge → reject; the store is never consulted (len stays 0).
        let too_old = NOW - ACCEPT_PAST_MS - 1;
        assert_eq!(
            store.check_and_record("client-a", "jti-1", too_old, NOW),
            Err(JtiReject::ClockDriftPast)
        );
        assert_eq!(
            store.len(),
            0,
            "a drift-rejected proof must not be recorded"
        );

        // Inclusive boundary: iat == now - ACCEPT_PAST is INSIDE the window → Ok.
        let edge = NOW - ACCEPT_PAST_MS;
        assert_eq!(
            store.check_and_record("client-a", "jti-edge", edge, NOW),
            Ok(())
        );
    }

    #[test]
    fn clock_drift_future_rejected() {
        let mut store = JtiReplayStore::new();
        // Just past the future edge → reject.
        let too_new = NOW + ACCEPT_FUTURE_MS + 1;
        assert_eq!(
            store.check_and_record("client-a", "jti-1", too_new, NOW),
            Err(JtiReject::ClockDriftFuture)
        );
        assert_eq!(
            store.len(),
            0,
            "a drift-rejected proof must not be recorded"
        );

        // Inclusive boundary: iat == now + ACCEPT_FUTURE is INSIDE the window → Ok.
        let edge = NOW + ACCEPT_FUTURE_MS;
        assert_eq!(
            store.check_and_record("client-a", "jti-edge", edge, NOW),
            Ok(())
        );
    }

    #[test]
    fn capacity_cap_fail_closed() {
        // Small cap via with_params; all entries unexpired (iat == now), so the sweep reclaims none.
        let cap = 4;
        let mut store = JtiReplayStore::with_params(ACCEPT_PAST_MS, ACCEPT_FUTURE_MS, cap);
        for i in 0..cap {
            assert_eq!(
                store.check_and_record("client-a", &format!("jti-{i}"), NOW, NOW),
                Ok(())
            );
        }
        assert_eq!(store.len(), cap);

        // The (cap + 1)-th distinct, unexpired jti is refused — never an eviction.
        assert_eq!(
            store.check_and_record("client-a", "jti-overflow", NOW, NOW),
            Err(JtiReject::StoreFull)
        );
        assert_eq!(store.len(), cap, "StoreFull must not grow or evict the map");

        // A prior LIVE jti still returns Replayed afterward — proving the cap did NOT evict a live
        // entry to admit the new one (no live-eviction replay hole).
        assert_eq!(
            store.check_and_record("client-a", "jti-0", NOW, NOW),
            Err(JtiReject::Replayed)
        );
    }

    #[test]
    fn sweep_reclaims_then_admits() {
        // The cap is not a permanent wall: once entries time-expire, the sweep frees room.
        let cap = 2;
        let mut store = JtiReplayStore::with_params(ACCEPT_PAST_MS, ACCEPT_FUTURE_MS, cap);
        assert_eq!(
            store.check_and_record("client-a", "jti-0", NOW, NOW),
            Ok(())
        );
        assert_eq!(
            store.check_and_record("client-a", "jti-1", NOW, NOW),
            Ok(())
        );
        // Full at NOW.
        assert_eq!(
            store.check_and_record("client-a", "jti-2", NOW, NOW),
            Err(JtiReject::StoreFull)
        );

        // Advance past expiry: the sweep on the next call reclaims both, so a fresh proof is admitted.
        let later = NOW + ACCEPT_PAST_MS + SWEEP_SLACK_MS + 1;
        assert_eq!(
            store.check_and_record("client-a", "jti-2", later, later),
            Ok(())
        );
        assert_eq!(store.len(), 1, "sweep reclaimed the two expired entries");
    }

    #[test]
    fn concurrent_check_and_insert_single_winner() {
        // N threads race the SAME (client, jti, iat) through a shared Mutex<JtiReplayStore>.
        // The atomic check-and-record must yield EXACTLY one Ok and N-1 Err(Replayed).
        let store = Arc::new(Mutex::new(JtiReplayStore::new()));
        let n = 32;
        let handles: Vec<_> = (0..n)
            .map(|_| {
                let store = Arc::clone(&store);
                thread::spawn(move || {
                    let mut guard = store.lock().expect("lock not poisoned");
                    guard.check_and_record("client-a", "jti-race", NOW, NOW)
                })
            })
            .collect();

        let mut oks = 0;
        let mut replays = 0;
        for h in handles {
            match h.join().expect("thread panicked") {
                Ok(()) => oks += 1,
                Err(JtiReject::Replayed) => replays += 1,
                Err(other) => panic!("unexpected reject: {other:?}"),
            }
        }
        assert_eq!(oks, 1, "exactly one concurrent winner");
        assert_eq!(replays, n - 1, "all others see Replayed");
        assert_eq!(store.lock().unwrap().len(), 1);
    }
}
