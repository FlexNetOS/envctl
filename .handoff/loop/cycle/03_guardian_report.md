# Verification report: TASK-0030 — F6 bounded DPoP `jti` replay store

## Verdict — PASS

All NON-NEGOTIABLE invariants hold, all real gates green, all 9 jti tests run and pass.
Verified against the actual `crates/secrets-engine/src/broker/jti.rs` (read in full), not the log.

(This file previously held the TASK-0020 cycle report; superseded by this TASK-0030 verdict.)

## Gate results
| Gate | Command | Exit | Result |
|------|---------|------|--------|
| no-c | `bash ci/gates/no-c.sh` | 0 | **PASS** — `resolved graph clean: rustls=['0.23.40'] on ring=['0.17.14']; zero aws-lc/openssl/C-SQLite` |
| shape | `bash ci/gates/shape.sh` | 0 | **PASS** — `SHAPE GATE PASS` |

## cargo
| Check | Command | Exit | Result |
|-------|---------|------|--------|
| fmt | `cargo fmt --all -- --check` | 0 | **PASS** (no diff) |
| clippy | `cargo clippy --workspace -- -D warnings` | 0 | **PASS** — exact CI form (`.github/workflows/ci.yml:49`; CI does NOT use `--all-targets`) |
| test | `cargo test -p envctl-secrets-engine` | 0 | **PASS** — crate 105 + 4 + 6 + 17 + 15 = green; 0 failed / 0 ignored. All 9 jti tests ran: first_use_accepted, replay_rejected, different_clients_same_jti_both_accepted, expired_then_fresh_same_value_accepted, clock_drift_past_rejected, clock_drift_future_rejected, capacity_cap_fail_closed, sweep_reclaims_then_admits, concurrent_check_and_insert_single_winner |

## Invariant checks
1. **Fail-closed, no replay hole** — PASS. Every uncertain branch of `check_and_record` returns `Err`:
   ClockDriftPast (jti.rs:109-111), ClockDriftFuture (112-114), Replayed (121-123), StoreFull (126-128).
   The ONLY `Ok` path is the terminal insert (131-133). No early-Ok, no accept-on-error, no panic/unwrap
   in the production path (lines 1-148 grep clean). **Cap rejects the NEW proof and never evicts a live
   entry**: the `len() >= max_entries` check (126-128) returns before any insert; the sweep (117) drops
   only time-expired (`exp <= now`). Test `capacity_cap_fail_closed` (jti.rs:273-298) proves a prior LIVE
   `jti-0` still returns `Err(Replayed)` after the cap is hit (294-297) — no live-eviction replay hole.
2. **Atomicity** — PASS. `check_and_record(&mut self, …)` (jti.rs:101-107) is sync; no `.await`/yield
   between dedup (121) and insert (131) — a single `&mut self` call. Concurrent test (329-356) wraps
   `Arc<Mutex<JtiReplayStore>>` (332), races 32 threads on one (client,jti,iat), asserts EXACTLY one Ok
   and 31 Err(Replayed) + final len==1 (353-355).
3. **Drift gate correctness** — PASS. Drift checks (jti.rs:109-114) run FIRST, before sweep/dedup/insert,
   so a drift-rejected proof never touches the map. Boundaries inclusive: `iat < now - past` rejects ⇒
   `iat == now - past` Ok (tested 242-246); `iat > now + future` rejects ⇒ `iat == now + future` Ok
   (tested 265-269). Retention expiry = `iat_ms + accept_past_ms + sweep_slack_ms` (131-132). Sweep =
   `seen.retain(|_, &mut exp| exp > now_ms)` (117) — keeps `exp > now`, drops `exp <= now`, matching spec §3.2.
4. **No secret bytes / non-printing** — PASS. Store = `HashMap<String, i64>` (jti.rs:65): key
   `format!("{client_id}\u{0}{jti}")`, value `i64` expiry — no proof body/signature/bearer/key material.
   No `println!`/`eprintln!`/`print!`/`stdout` (grep NONE). Sync (no `async`), no `clap`/UI, std-only
   (`use std::collections::HashMap`, line 24).
5. **Zero new deps** — PASS. No `Cargo.toml`/`Cargo.lock` in the changeset (`git status --short` →
   only `jti.rs` + the 2 wiring edits + the spec/handoff docs). no-c.sh re-derived the resolved graph clean.
6. **Engine purity** — PASS. Module is in `crates/secrets-engine/src/broker/jti.rs`; no clap/UI/socket/TLS.
   F2 edge listener NOT built — `crates/secretd/src/edge` does not exist (out of scope, TASK-0031).
7. **Spec consistency** — PASS. Consts match `docs/secrets/OI-SM-1-jti-replay-store.md` §2/§4 exactly:
   `ACCEPT_PAST_MS=300_000` (jti.rs:28), `ACCEPT_FUTURE_MS=30_000` (32), `SWEEP_SLACK_MS=30_000` (36),
   `MAX_ENTRIES=16_384` (40).

## Parity check
Vacuous, as planned — no new user-facing `Engine` verb; this is a daemon-internal pure type with no
CLI/GUI surface (the `decide`/`gate` no-verb precedent). Cross-boundary re-export reachability for the
TASK-0031 caller confirmed:
- `crates/secrets-engine/src/broker/mod.rs:7` — `pub mod jti;`
- `crates/secrets-engine/src/broker/mod.rs:16` — `pub use jti::{JtiReject, JtiReplayStore};`
- `crates/secrets-engine/src/lib.rs:31` — `JtiReject, JtiReplayStore` in the `pub use broker::{…}` block,
  so `use envctl_secrets_engine::{JtiReplayStore, JtiReject};` resolves.

## Findings
None blocking.

- (note, informational) The implementation is currently **uncommitted** in the worktree: `jti.rs` +
  the OI-SM-1 doc are untracked; `broker/mod.rs`, `lib.rs`, and the two handoff artifacts are modified;
  HEAD == merge-base with origin/develop. Verification ran against the on-disk working tree. The
  orchestrator must commit the changeset for it to land/merge — the verified state is the working tree.
- (note, informational) `cargo clippy -p envctl-gui --all-targets` trips a pre-existing
  `doc_list_item_without_indentation` lint in the untouched `crates/gui/src/main.rs` under the floating
  `stable` toolchain. NOT in the repo's CI clippy form (`--workspace`, no `--all-targets`) and unrelated
  to this cycle — confirmed pre-existing, not a TASK-0030 regression. Not a blocker.

## Re-test needed
None for a PASS verdict. If the changeset is committed and any code is touched, re-run:
```
bash ci/gates/no-c.sh
bash ci/gates/shape.sh
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test -p envctl-secrets-engine
```
