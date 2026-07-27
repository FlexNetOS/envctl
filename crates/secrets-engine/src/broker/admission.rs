//! Bounded, in-memory, per-process per-key token-bucket admission limiter for the F2 relay edge
//! (TASK-0031-PR2; CVE-2024-47609 / SERVER-MODE §6.2). Sibling to [`crate::broker::jti`] /
//! [`crate::broker::nonce`]: same bounded, fail-closed shape. The edge keys this by SOURCE IP and
//! sheds an over-rate connection at STEP 0 — BEFORE any crypto/verify/decide — so a flood cannot burn
//! signature-verification or vault work. It is an early-reject SHED only: it can never substitute for
//! or weaken `decide()`; the full verify ladder + `decide()` still run on every non-shed request.
//!
//! Token-bucket per key: a bucket starts full (`burst` tokens) and refills at `refill_per_min`
//! tokens/minute up to `burst`. `admit` refills by elapsed time, sweeps idle buckets, then tries to
//! consume one token: success ⇒ `Allow`, empty ⇒ `Throttled`. A full key table presented with a NEW
//! key ⇒ `Throttled` (never grow past `MAX_KEYS`, never evict a live bucket to admit — fail-closed
//! DoS backstop). Poisoned-lock handling is the caller's (a poisoned `Mutex` ⇒ the edge returns 429).
//!
//! Sync, non-printing, `std`-only. ZERO new deps. Holds no secret material — only a per-key token
//! count + last-refill instant.

use std::collections::HashMap;

/// Steady-state token refill rate (tokens per minute) per key. Sustained admit rate once the burst
/// allowance is spent.
pub const RATE_REFILL_PER_MIN: u32 = 120;

/// Bucket capacity (max tokens) — the burst a single key may spend before being rate-limited to the
/// steady refill. Also the value a fresh bucket starts at.
pub const BUCKET_BURST: u32 = 60;

/// Hard size cap on the number of tracked keys (DoS backstop). A flood of unique source IPs cannot
/// grow the table without bound: at `MAX_KEYS` a new key is `Throttled` (never inserted, never
/// evicting a live bucket).
pub const MAX_KEYS: usize = 65_536;

/// One key's admission outcome. `Allow` consumed a token (the request proceeds to the full verify
/// ladder + `decide()`); `Throttled` is an early SHED (the edge returns 429) — it NEVER means
/// "admitted without verify".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Admit {
    Allow,
    Throttled,
}

/// One key's token bucket. Tokens are tracked in milli-tokens (fixed-point) so a sub-token refill
/// from a small elapsed interval is not silently truncated to zero (which would stall the refill).
#[derive(Clone, Copy, Debug)]
struct Bucket {
    /// Available tokens × 1000 (milli-tokens), clamped to `burst × 1000`.
    millitokens: u64,
    /// Wall-ms of the last refill computation for this bucket.
    last_ms: i64,
}

/// Bounded, in-memory, per-process per-key token-bucket limiter. Not interior-mutable: the caller
/// (the edge) owns the single `Mutex<AdmissionLimiter>` so `admit` is atomic across concurrent
/// connections from the same key.
pub struct AdmissionLimiter {
    refill_per_min: u32,
    burst: u32,
    max_keys: usize,
    buckets: HashMap<String, Bucket>,
}

impl AdmissionLimiter {
    /// A limiter with the audited defaults (`RATE_REFILL_PER_MIN` / `BUCKET_BURST` / `MAX_KEYS`).
    pub fn new() -> Self {
        Self {
            refill_per_min: RATE_REFILL_PER_MIN,
            burst: BUCKET_BURST,
            max_keys: MAX_KEYS,
            buckets: HashMap::new(),
        }
    }

    /// A limiter with tuned rate/burst/cap (lets a test drive throttle/refill/idle-sweep/full-table
    /// with small numbers as a one-liner).
    pub fn with_params(refill_per_min: u32, burst: u32, max_keys: usize) -> Self {
        Self {
            refill_per_min,
            burst,
            max_keys,
            buckets: HashMap::new(),
        }
    }

    /// Refill-by-elapsed → sweep idle → try-consume one token. Returns `Allow` when a token was
    /// consumed (the request proceeds to verify + `decide()`), `Throttled` to SHED early. Steps:
    /// 1. **Refill** the key's bucket by the elapsed time since its last refill (a fresh key starts
    ///    `burst`-full), clamped to `burst`.
    /// 2. **Sweep** every OTHER bucket that has sat idle long enough to be provably full (so the table
    ///    stays bounded under churn) — never the key being admitted this call.
    /// 3. **Consume** one token if available (`Allow`); else `Throttled`. A NEW key when the table is
    ///    already at `max_keys` (post-sweep) is `Throttled` WITHOUT insertion (never grow, never
    ///    evict-to-admit — fail-closed).
    pub fn admit(&mut self, key: &str, now_ms: i64) -> Admit {
        let burst_mt = self.burst as u64 * 1000;
        // Milli-tokens added per ms = refill_per_min * 1000 / 60_000 = refill_per_min / 60 (per ms),
        // computed as (refill_per_min * 1000 * elapsed_ms) / 60_000 to keep integer precision.
        let refill_per_min = self.refill_per_min as u64;

        // (2) Sweep OTHER idle buckets that have refilled to full and have been idle a while, to keep
        // the table bounded under source-IP churn. A bucket is provably full once
        // `(now - last) * refill >= burst`; we additionally require it not be the active key. This is
        // an amortized reclamation, not a correctness requirement (a full idle bucket carries no
        // state a fresh one wouldn't).
        if self.buckets.len() > 1 {
            self.buckets.retain(|k, b| {
                if k == key {
                    return true;
                }
                let elapsed = now_ms.saturating_sub(b.last_ms).max(0) as u64;
                let added_mt = elapsed.saturating_mul(refill_per_min).saturating_mul(1000) / 60_000;
                let full = b.millitokens.saturating_add(added_mt) >= burst_mt;
                // Keep only buckets that are NOT yet provably full (still mid-refill / recently used).
                !full
            });
        }

        // (1)+(3) Refill the active key's bucket, then try to consume.
        match self.buckets.get_mut(key) {
            Some(b) => {
                let elapsed = now_ms.saturating_sub(b.last_ms).max(0) as u64;
                let added_mt = elapsed.saturating_mul(refill_per_min).saturating_mul(1000) / 60_000;
                b.millitokens = b.millitokens.saturating_add(added_mt).min(burst_mt);
                b.last_ms = now_ms;
                if b.millitokens >= 1000 {
                    b.millitokens -= 1000;
                    Admit::Allow
                } else {
                    Admit::Throttled
                }
            }
            None => {
                // A NEW key. If the table is already full (post-sweep), SHED without inserting
                // (fail-closed: never grow past max_keys, never evict a live bucket to admit).
                if self.buckets.len() >= self.max_keys {
                    return Admit::Throttled;
                }
                // A fresh bucket starts full, then spends one token for this request.
                self.buckets.insert(
                    key.to_string(),
                    Bucket {
                        millitokens: burst_mt.saturating_sub(1000),
                        last_ms: now_ms,
                    },
                );
                Admit::Allow
            }
        }
    }

    /// Tracked key count (test-only — the production type exposes no map internals).
    #[cfg(test)]
    fn len(&self) -> usize {
        self.buckets.len()
    }
}

impl Default for AdmissionLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;

    const NOW: i64 = 1_000_000_000;

    #[test]
    fn burst_then_throttled() {
        // burst=3, all at the same instant (no refill): exactly `burst` Allows, then Throttled.
        let mut l = AdmissionLimiter::with_params(60, 3, MAX_KEYS);
        assert_eq!(l.admit("1.2.3.4", NOW), Admit::Allow);
        assert_eq!(l.admit("1.2.3.4", NOW), Admit::Allow);
        assert_eq!(l.admit("1.2.3.4", NOW), Admit::Allow);
        assert_eq!(
            l.admit("1.2.3.4", NOW),
            Admit::Throttled,
            "the bucket is empty after `burst` admits with no elapsed refill"
        );
    }

    #[test]
    fn refill_restores_a_token() {
        // burst=1, refill=60/min = 1/sec. Spend the single token, get Throttled, then after 1s the
        // refill restores exactly one token → Allow again.
        let mut l = AdmissionLimiter::with_params(60, 1, MAX_KEYS);
        assert_eq!(l.admit("ip", NOW), Admit::Allow);
        assert_eq!(l.admit("ip", NOW), Admit::Throttled);
        // 999 ms: still < 1 token refilled.
        assert_eq!(l.admit("ip", NOW + 999), Admit::Throttled);
        // 1000 ms after the spend: exactly one token refilled → Allow.
        assert_eq!(l.admit("ip", NOW + 1000), Admit::Allow);
    }

    #[test]
    fn idle_buckets_swept() {
        // Two keys; one goes idle long enough to refill to full while another stays active. The idle
        // bucket is reclaimed by the sweep, keeping the table bounded.
        let mut l = AdmissionLimiter::with_params(60, 2, MAX_KEYS);
        assert_eq!(l.admit("idle", NOW), Admit::Allow); // idle bucket created (1 token left).
        assert_eq!(l.admit("active", NOW), Admit::Allow); // active bucket.
        assert_eq!(l.len(), 2);
        // Advance far enough that `idle` is provably full again; an `active` admit triggers the sweep
        // which reclaims the (now-full, untouched) `idle` bucket.
        let later = NOW + 10_000;
        assert_eq!(l.admit("active", later), Admit::Allow);
        assert_eq!(
            l.len(),
            1,
            "the idle, refilled-to-full bucket was swept; only the active key remains"
        );
    }

    #[test]
    fn full_key_table_throttles_new_key() {
        // max_keys=2 with both buckets kept non-full (so the sweep can't reclaim them): a THIRD,
        // distinct key is Throttled WITHOUT being inserted (never grow, never evict-to-admit).
        let mut l = AdmissionLimiter::with_params(60, 2, 2);
        // Drain each of the two keys to 0 tokens so neither is sweep-eligible (not full).
        assert_eq!(l.admit("a", NOW), Admit::Allow);
        assert_eq!(l.admit("a", NOW), Admit::Allow);
        assert_eq!(l.admit("b", NOW), Admit::Allow);
        assert_eq!(l.admit("b", NOW), Admit::Allow);
        assert_eq!(l.len(), 2);
        // A new key at the SAME instant: table full, neither existing bucket is full → no sweep room.
        assert_eq!(
            l.admit("c", NOW),
            Admit::Throttled,
            "a new key against a full table is shed, not inserted"
        );
        assert_eq!(l.len(), 2, "a throttled new key must not grow the table");
    }

    #[test]
    fn concurrent_admits_respect_burst() {
        // N threads race the SAME key through a shared Mutex<AdmissionLimiter> at the SAME instant
        // (no refill). EXACTLY `burst` must be admitted; the rest throttled.
        let burst = 10u32;
        let limiter = Arc::new(Mutex::new(AdmissionLimiter::with_params(
            60, burst, MAX_KEYS,
        )));
        let n = 50;
        let handles: Vec<_> = (0..n)
            .map(|_| {
                let limiter = Arc::clone(&limiter);
                thread::spawn(move || {
                    let mut g = limiter
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    g.admit("racer", NOW)
                })
            })
            .collect();

        let mut allows = 0;
        let mut throttles = 0;
        for h in handles {
            match h.join().expect("thread panicked") {
                Admit::Allow => allows += 1,
                Admit::Throttled => throttles += 1,
            }
        }
        assert_eq!(allows, burst as i32, "exactly `burst` concurrent admits");
        assert_eq!(throttles, n - burst as i32, "the rest are shed");
    }
}
