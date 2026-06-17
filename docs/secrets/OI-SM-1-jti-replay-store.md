# OI-SM-1 — DPoP `jti` replay store (design)

Status: ACTIVE (resolves audit open-item OI-SM-1; unblocks TASK-0030 / audit F6)
Scope: the bounded, in-memory, per-process `jti` replay-dedup store the Phase-8 remote
relay edge uses to make each DPoP (RFC 9449) proof single-use within its acceptance window.
Corpus: SERVER-MODE.md §4.2 (line 118), audits/AUDIT-server-mode.md F6 (line 70) + OI-SM-1
(line 320), research/12 §2.2 (lines 30-32, 100), THREAT-MODEL A14/A16, DESIGN-NOTES OI-6.

This spec resolves ONLY the `jti` replay-store half of OI-SM-1. The server-issued nonce
lifecycle and the `remote_clients` schema (`jkt`/`hardware_bound`) are the other OI-SM-1
sub-items; their interaction is referenced here (§6) but their full design is deferred to the
F2 edge work (TASK-0031) and F15 schema work — this store is built to plug into them.

## 1. What a `jti` is and the threat

Per RFC 9449 §4.2, a DPoP proof is a JWT (`typ: "dpop+jwt"`) the client sends in the `DPoP`
header on every request. It carries the public `jwk`, the bound HTTP method (`htm`) and URI
(`htu`), an issued-at (`iat`), and a **`jti`** — a unique identifier the client generates fresh
per proof. The bearer is bound to the key via `cnf.jkt` (the RFC 7638 SHA-256 JWK thumbprint).

The proof binds the *request context* (method + URI), so a captured proof cannot be replayed
against a *different* endpoint (SERVER-MODE §4.2; research/12 §2.2 line 31). What it does NOT
defend by itself is replay against the **same** endpoint within the proof's acceptance window:
an attacker who captures one valid proof (e.g. an on-path observer, or owner-session malware
that scraped a request) can resend the identical `DPoP` header + bearer and the edge would,
without dedup, accept it again. RFC 9449 §11.1 names exactly two server-side defenses for this:
**`jti` tracking** and an **optional server-issued nonce**. This store implements the `jti`
tracking half. (It does not — and per the threat model cannot — defend owner-session malware
that reads the live signing key out of the agent's memory and mints *fresh* proofs; that is
A2/A10, "bounded by scope/TTL, not prevented", SERVER-MODE §4.4, research/12 §3.)

The threat this store closes: **THREAT-MODEL A14 — "stolen remote bearer/proof replayed".**
Without it, the only same-endpoint replay bound is the bearer TTL (≤24h) — far too wide
(research/12 §2.2 line 24: best practice is a minutes-wide window).

## 2. Acceptance window (the retention bound)

The store's retention is derived from — and can never exceed — the DPoP proof acceptance
window. A proof is accepted iff its `iat` is "reasonably near" now (RFC 9449 §4.3, §11.1):

| Parameter | Default | Rationale |
|-----------|---------|-----------|
| `ACCEPT_PAST`   | **300 s (5 min)** | Audit F6 recommends a 5-min validity window; matches OAuth short-token norm (research/12 line 24). Wide enough for genuine clock skew + network latency, narrow enough that the replay window is minutes, not the 24h bearer TTL. |
| `ACCEPT_FUTURE` | **30 s** | A proof from the future is almost always clock skew, never legitimate latency. A tight future bound caps how long a *pre-minted* proof could sit waiting to be replayed. Must be `<<` `ACCEPT_PAST`. |

A proof with `iat < now - ACCEPT_PAST` or `iat > now + ACCEPT_FUTURE` is **rejected before the
store is even consulted** (`ClockDriftPast` / `ClockDriftFuture`). This is what makes the store
*bounded by time*: a `jti` only needs to be remembered until `iat + ACCEPT_PAST` has elapsed,
because after that any proof bearing it would be rejected on the drift check anyway and could
never replay. **Retention horizon = `ACCEPT_PAST` + a small `SWEEP_SLACK` (30 s)** to cover the
fact that "now" advances between insert and the latest moment a replay could arrive.

(Server-issued nonce, OI-SM-1's other knob, is complementary: when the edge later issues a
`DPoP-Nonce` (RFC 9449 §8), a fresh nonce on a `use_dpop_nonce` 401 absorbs drift larger than
`ACCEPT_PAST` and lets the window stay tight. The store does not depend on it; §6.)

## 3. Store semantics

- **Key:** the `jti` string, scoped per-client. The dedup key is `(client_id, jti)` — a `jti`
  is only client-unique per RFC 9449, so two distinct clients may legitimately emit the same
  `jti` value. Scoping by `client_id` (already authenticated by the edge before this check)
  prevents one client's `jti` from blocking another's. (Implementation may concatenate into a
  single map key; see TASK-0030 plan.)
- **Value:** the **expiry instant** `iat + ACCEPT_PAST + SWEEP_SLACK` (wall-ms `i64`). No proof
  body, no signature, no bearer, no key bytes are stored — metadata only (§5).
- **The atomic operation (insert-if-absent = the replay check):** a single
  `check_and_record(client_id, jti, iat, now)`:
  1. **Drift gate:** reject if `iat` outside `[now - ACCEPT_PAST, now + ACCEPT_FUTURE]`.
  2. **Sweep:** opportunistically evict every entry whose stored expiry `<= now` (amortized,
     bounded; see §4).
  3. **Dedup:** if `(client_id, jti)` is already present (and unexpired) → **REJECT (`Replayed`)**.
  4. **Record:** otherwise insert with expiry `iat + ACCEPT_PAST + SWEEP_SLACK` → **ACCEPT**.
  Steps 1-4 happen under a single lock so the check and the record are indivisible (§7).

## 4. Bounded capacity (the DoS backstop)

Time-based eviction alone is not enough: an attacker who floods *unique* `jti`s within the
window would grow the map unbounded between sweeps (audit F6: "memory-exhaustion DoS by a flood
of unique-`jti` proofs"). So a **hard size cap** is the backstop:

| Parameter | Default | Rationale |
|-----------|---------|-----------|
| `MAX_ENTRIES` | **16 384** | Proportional to (clients × proofs-per-window). At the edge's `rate_per_min` (per-client quota, decide.rs clause 15) and a 5-min window, a handful of clients fit comfortably; 16 384 × ~64 B ≈ **1 MiB** worst-case — the explicit DoS bound. |

**Eviction strategy:** the sweep in step 2 (drop all time-expired entries) is the *primary*
reclamation and keeps the map near (live-clients × window-traffic). The size cap is a
**fail-closed backstop**, NOT an LRU cache: when the map is at `MAX_ENTRIES` *after* the sweep,
a new `jti` is **REJECTED (`StoreFull`)** rather than evicting a live entry. Evicting a live
entry to admit a new one would re-open the replay window for the evicted `jti` (an attacker
could flood to force-evict a victim's `jti`, then replay it) — so we **reject the new proof
instead of forgetting an old one**. This is the fail-closed choice (§5): under flood, legitimate
new proofs are denied (the client retries / the per-client rate limit + revocation handle the
abuser), but no already-seen `jti` is ever forgotten while still replayable.

> Design decision (resolves the F6 "ring/LRU" suggestion): an LRU that *evicts a live entry to
> admit a new one* is **rejected** because it trades availability for a replay hole. We use
> time-expiry + a fail-closed hard cap. The cap is sized so that, under the per-client rate
> limit that already gates remote requests, a legitimate workload never reaches it.

## 5. Memory-only, fail-closed, invariant fit

- **In-memory, per-process, non-persisted.** The store lives in RAM in `secretd`. Justification:
  (a) a restart re-derives safety from the acceptance window — every `jti` minted before the
  restart has an `iat` that is now older than `ACCEPT_PAST`, so it would be rejected on the drift
  gate regardless of whether the store remembers it. There is no security loss across a restart.
  (b) It avoids putting the hot per-request replay check on the libSQL **remote** store's network
  round-trip (a latency + availability dependency on every proof), and avoids any persistence
  layer touching the no-C trust boundary. (c) The DoS bound is a fixed RAM ceiling (§4), not
  unbounded disk. **DoS bound: `MAX_ENTRIES` × entry-size ≈ 16 384 × 64 B ≈ 1 MiB.**
- **Fail-closed everywhere.** Any uncertainty REJECTS the proof, never accepts:
  drift-out-of-window → reject; store full → reject; duplicate → reject. A poisoned lock (if the
  edge wraps it) MUST map to reject, never to a bypass. There is no "accept on error" path.
- **No secret bytes.** A `jti` is a public proof identifier, not a secret — but the store is
  treated as metadata-only: it stores `jti` + an expiry int, never proof bodies, signatures,
  bearers, or key material. Audit/log emissions name the **DenyReason** and `client_id` only
  (mirroring the §4.3 audit-traceability rule: "never the bearer, never the real key"). Never
  log the proof JWT.
- **Pure-Rust, zero new deps.** `std::collections` + the engine's existing wall clock. No SQLite,
  no OpenSSL, no aws-lc; nothing the no-C gate forbids. Sync; non-printing (emits a typed reject
  reason the caller maps, never `println!`).

## 6. Interaction with the rest of OI-SM-1 (nonce + schema)

- **Server-issued nonce (RFC 9449 §8):** complementary, deferred to TASK-0031. When the edge
  issues a `DPoP-Nonce`, including it as a claim lets the acceptance window stay tight even under
  large clock skew, and a **fresh nonce on a genuine retry naturally yields a fresh proof with a
  new `jti`** — so a legitimate retry is accepted without the store needing retry-aware logic
  (audit F6 "genuine-retry recovery"). This store is nonce-agnostic: it dedups whatever `jti` it
  is given. If nonce binding is added, the dedup key MAY extend to `(client_id, nonce, jti)`.
- **`remote_clients` schema (`jkt`/`hardware_bound`):** the store keys on the `client_id` the
  edge already authenticated against the registered `jkt` (decide.rs clause 11a); it does not
  itself read the schema. F15 owns the columns.
- **Rate accounting:** each `check_and_record` probe SHOULD count against the client's
  `rate_per_min` (decide.rs clause 15) so a `jti`-flood is also rate-limited — wiring is the
  edge's job (TASK-0031), but the store is designed to be called once per proof so the count is
  exact.

## 7. Concurrency

The check-and-record MUST be **atomic**. The edge is concurrent (HTTP/2, many in-flight
requests). If two replays of the same proof arrive simultaneously and the check (read) and the
record (write) were separate, both could observe "absent" and both pass. The store therefore
performs steps §3.1-§3.4 under a **single mutual-exclusion lock** (a `Mutex` around the map, or
`&mut self` if the caller owns exclusivity). Property: for any `(client_id, jti)`, **at most one
concurrent `check_and_record` returns ACCEPT; all others return `Replayed`.** A unit test drives
N threads at one `jti` and asserts exactly one winner (§ TASK-0030 tests).

## 8. Resolved open questions (OI-SM-1, jti-store scope)

| Question (from audit) | Resolution |
|---|---|
| Store sizing | `MAX_ENTRIES = 16 384` (≈1 MiB), proportional to clients × window under the per-client rate cap. |
| Eviction | Time-expiry sweep (primary) + **fail-closed hard cap** (reject when full). NOT live-entry LRU eviction (would re-open a replay hole). |
| Replay/validity window | `ACCEPT_PAST = 300 s`, `ACCEPT_FUTURE = 30 s`; retention = `ACCEPT_PAST + 30 s` slack. |
| Same-`jti`-after-expiry | **Accepted** — once `iat + ACCEPT_PAST` has passed, a proof with that `jti` is rejected by the drift gate before the store, so re-admitting the value is safe and the store need not remember it forever. (Documented behavior, tested.) |
| Memory vs persisted | **In-memory, per-process.** Restart safety comes from the acceptance window, not persistence. |
| Genuine retry | Handled by the nonce layer (fresh nonce ⇒ fresh `jti`); the store stays simple/nonce-agnostic. |
| Nonce lifecycle / `remote_clients` schema | Deferred to TASK-0031 / F15; store designed to plug in (§6). |
