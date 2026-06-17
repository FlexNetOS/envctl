# Verification report: TASK-0032 (F5, P0) — streaming-revocation tear-down (FS-S5)

## Verdict — PASS

All 8 in-scope invariants hold with running evidence; all 4 CI gates and every cargo check
exit 0. The change is additive, default-OFF behind `relay-edge`, introduces ZERO new lockfile
crates, and does not touch the crypto/TLS/cert/EKM surface. No guard was weakened.

Scope note: the branch `task-0032-stream` has NO commits vs `origin/develop` — the TASK-0032
work is the *uncommitted working tree*. The accurate change is the working tree vs the merge-base
`491525ea` (matches the implementer log exactly). The raw `git diff origin/develop` mixes in
develop's 2 ahead-commits (ci.yml / kdf-feature-off.sh / keyslot.rs / manifest) which are NOT
part of this change and were ignored.

## Gate results
- `ci/gates/no-c.sh` : PASS (exit=0) — `rustls=['0.23.40'] on ring=['0.17.14']; zero aws-lc/openssl/C-SQLite`
- `ci/gates/shape.sh` : PASS (exit=0) — edge-never-references-MITM-CA + native-roots invariants hold
- `ci/gates/enable.sh` : PASS (exit=0)
- `ci/gates/p7.sh` : PASS (exit=0)

## cargo
- `cargo fmt --all -- --check` : PASS (exit=0)
- `cargo clippy --workspace --all-targets -- -D warnings` : PASS (exit=0)
- `cargo clippy -p envctl-secretd --features relay-edge --all-targets -- -D warnings` : PASS (exit=0)
- `cargo test -p envctl-secrets-engine` : PASS (exit=0) — 110 unit + 22 relay (incl. 4 new) + 6 inject + 15 vault + 4 phase0, 0 failed
- `cargo test -p envctl-secretd --features relay-edge` : PASS (exit=0) — 52 unit; e2e 5; **edge_stream_e2e 4**; edge_e2e 1; mitm 1; native_mint 11; **proxy_swap_e2e 2**; self_check 2; 0 failed

## Invariant checks
1. **No C in trust boundary** — PASS. `no-c.sh` green from resolved `cargo metadata`. `git diff
   491525ea -- Cargo.lock` is EMPTY → zero new crates. No new rustls backend / aws-lc / sqlite /
   openssl / mimalloc.
2. **Exactly one rustls, ring-only** — PASS. `rustls=0.23.40 on ring=0.17.14`; no TLS dep changed
   (Cargo.lock unchanged; the re-check does no crypto).
3. **Engine single sync NON-PRINTING library** — PASS. `git diff` of `crates/secrets-engine/`
   added lines: ZERO `println!`/`eprintln!`/`print!`/`stdout` (grep exit=1); none anywhere in the
   engine src. ALL re-check policy lives in `Engine::relay_stream_authorized → authorize_relay(bump=false)
   → decide()` (lib.rs ~1556–1660). `edge/stream.rs` is select/forward/drop I/O only — no independent
   allow branch; it can only tear DOWN or break on client/upstream EOF.
4. **`decide()` is the only Allow authority** — PASS. `relay_stream_authorized` returns `Authorized`
   ONLY on `Ok(Authz::Allow{..})`, reachable only via `decide() → RelayDecision::Allow`;
   `Err(_) => TearDown`, `Deny => TearDown(reason)`. Swap path observable behavior unchanged: the
   `authorize_relay` factoring preserves the inline `deny_swap(...)` audit shape and the on-Allow key
   fetch; **`proxy_swap_e2e` (2) incl. `proxy_swap_delivers_real_key_only_and_bearer_never_leaks` PASS**,
   and the edge `wrap_body` supervisor runs ONLY on the already-authorized `Allowed` body (the swap
   still drives `relay_swap` unchanged; the local proxy passes identity `|rx| rx`).
5. **Fail-closed / fail-safe** — PASS. Every failure mode maps to tear-down: `decide()` Deny →
   `TearDown(reason)`; locked vault / poisoned RwLock (`map_err`, not `unwrap`) / store err / vanished
   bearer / MAC fail / USB absent all surface as engine `Err` → `TearDown`; max-duration deadline →
   unconditional tear-down; client/upstream EOF → clean break. Grep of the re-check hot path
   (`stream.rs` + lib.rs 1404–1690) for `unwrap(`/`expect(`/`panic!`/indexing: NONE (exit=1). The
   `emit_teardown` `unwrap_or_else` is a reason-string fallback, not auth logic / not a panic.
   Covered by `relay_stream_authorized_tears_down_on_{bearer_revoke,usb_pull,locked_vault}` (real
   Authorized→TearDown transitions) + the 4 e2e cases.
6. **No secret bytes in logs/audit** — PASS. New `SecretEvent::RelayStreamTornDown { relay, token_id,
   reason }` is metadata-only. `emit_teardown` carries only those 3 fields. The bearer in `stream.rs`
   is `Zeroizing<String>`; `token_id` is the public id via `broker::parse_bearer`; the real key /
   proxied body never appear. `Deny`/`Err` carry no key.
7. **relay-tls only, never MITM CA (FS-S25) + EKM (FS-S20)** — PASS. NO TLS/cert/EKM/DPoP file touched
   (`tls.rs`/`dpop.rs` diff empty); no `mitm|ca_|leaf|mint_cert|root_ca` token in the new edge code
   (grep exit=1); `shape.sh` PASS.
8. **Default-OFF `relay-edge`** — PASS. `pub mod edge;` is `#[cfg(feature="relay-edge")]` (lib.rs:13),
   so `edge/stream.rs` + the listener wiring + `EdgeConfig.recheck_timing` are reachable only under the
   feature. The engine method is inert unless the edge calls it. `cargo clippy --workspace` (no feature)
   is clean → a stock secretd build is unaffected.
9. **Parity (front-end)** — PASS. `RelayStreamTornDown` joins the conv.rs no-proto-twin drop set
   (alongside `RelayRevoked`): CLI + GUI drain `SecretEvent`s through the same funnel, no divergence,
   zero proto churn. No `Engine` public-method surface one front-end can reach and the other can't.

## Parity check
This feature adds no user-facing `Engine` verb needing CLI↔GUI wiring (the new engine method is an
internal edge-only re-check). Event parity: `SecretEvent::RelayStreamTornDown`
→ `crates/secretd/src/conv.rs::event_to_proto` (no-proto-twin drop set) → consumed identically by CLI
and GUI. No divergence.

## Findings
None blocking. Informational notes (consistent with the plan, not defects):
- INFO: the max-duration cap reports `DenyReason::PolicyExpired` (the deadline branch does NOT consult
  `decide()` — unconditional hard cap; all other tear-downs carry the real `decide()` reason).
  Disclosed in the implementer log "Deviations"; matches the plan's hard-cap intent.
- INFO: the tear-down event is best-effort cosmetic (edge wiring passes `EventSink::null()`); the open
  swap is already durably audited by the engine and the event is metadata-only either way. No durable
  audit ROW for the tear-down — consistent with how the existing relay-revoke surfaces.
- INFO (latency): revoke/lock/USB-pull detection is bounded by one `RECHECK_INTERVAL` (≤2s), not
  instant. The sub-second `tokio::sync::watch` push is a documented PR-4 follow-up, out of scope.
  Acceptable for P0.

## Re-test needed
None. If the working tree is later committed and rebased onto develop, re-run before merge:
`bash ci/gates/no-c.sh && bash ci/gates/shape.sh && rtk proxy cargo clippy --workspace --all-targets -- -D warnings`
and `rtk proxy cargo test -p envctl-secretd --features relay-edge` (regression-guard the `proxy_swap_e2e`
swap path + the 4 `edge_stream_e2e` cases).
