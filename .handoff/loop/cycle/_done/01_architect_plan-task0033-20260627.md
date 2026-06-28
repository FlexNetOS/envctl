# TASK-0033 architect plan — Secrets SERVER-MODE Profile B (VPS), IN FULL

VERDICT: GO. Epic-F final gated sub-item. Delivers F7/FS-S21, F8/OI-SM-2, F9/FS-S23,
OI-SM-3, FS-S22, FS-S24. 19 units + E2E + smoke. Build leaf-first.

## Foundation confirmed (read, byte-for-byte)
- `broker/gate.rs`: `PresenceGate` trait `resolve()->GateState{Present,AbsentSince(i64),Unproven}`;
  `gate_absent_since_ms()` maps Unproven→Some(now) (no grace). NOTE: the live code uses
  `AbsentSince(i64 wall-ms)` NOT `AbsentSince(Instant)` — a deliberate prior deviation (epoch-ms
  throughout). VpsPresenceGate MUST follow the live shape (i64), not the task's `Instant` sketch.
- `broker/{jti.rs (JtiReplayStore::check_and_record), nonce.rs (NonceStore::check_and_consume)}` —
  caller-clock (i64), single-Mutex atomic, fail-closed.
- `seam.rs`: `Clock` trait (now/boottime_ms), `SystemClock`. `with_seams(paths,store,clock,usb,
  provider,upstream,[github_transport])` — callers: open_with_store + 3 test helpers + edge_hardening
  build_engine + secretd engine_with_daemon_seams.
- `lib.rs`: `EngineInner{clock,usb,...}`; `presence_proven()` (lib.rs:2599) resolves Profile A /
  Profile S. `relay_swap_prepare` (lib.rs:1805) snapshots the gate PER swap already (lib.rs:1949).
- `event.rs`: `SecretEvent` (serde tag=type snake_case), metadata-only.
- `edge/{mod,listener,tls,stream}.rs`: `relay-edge` (default-OFF), ring-only tokio-rustls,
  `serve_edge`, `EdgeConfig`, `stream::Timing::production`, drain. tls.rs has mTLS client-auth.
- `ring = "0.17"` UNCONDITIONAL engine dep ⇒ authorizer.rs uses ring Ed25519 with NO feature/dep.
- secretd config.rs: serde `FileConfig{store,edge,security}` + env>file>default. main.rs `serve()`.
- secretctl cli.rs: clap derive `Cmd` enum + per-verb subcommand enums.

## Unit build order (leaf-first)
ENGINE: U2 token+signing-bytes → U3 sign → U5 TrustedTime → U4 verify → U1 inject
gate+topology+trusted_time + refactor presence_proven + update with_seams callers → U6
VpsPresenceGate → U7 F9 per-swap test → U8-U11 startup guards → U13 events.
DAEMON: U14 edge/authorizer.rs → U16 config + serve() guards.
CLI: U15 secretctl authorizer.
DECL/DOCS: U12 manifest+lock → U17 OI-SM-2 doc → U18 E2E → U19 doc reconcile (only after U18 green).

## Key design decisions
- **U2 presence_token_signing_bytes**: domain `b"env-ctl/v1/presence-token"` ‖ v(u8) ‖ ts_ms.be ‖
  len(u32 be)+vps_instance_id ‖ len+server_nonce ‖ vps_cert_fp[32] ‖ expiry_ms.be ‖ len+jti.
- **U4 AuthzReject** 8 variants ordered: MalformedVersion, TrustedTimeUnavailable, BadSignature,
  CertFpMismatch, NonceUnknown, Expired, NotYetValid, Replayed.
- **U1**: keep `presence_proven()` dispatching on `topology`: OnBox → existing body (byte-identical
  default), Vps → injected `presence_gate.resolve()`. with_seams gains gate (default UnprovenGate),
  trusted_time (default SystemClockTrustedTime), topology (default OnBox).
- **U6 VpsPresenceGate**: interior `Mutex<Option<i64>>` valid_until + clock; `accept_token(valid_until)`
  the authorizer feeds; resolve(): valid→Present, expired→AbsentSince(expiry), None→Unproven.
- **U5 OperatorBoxTrustedTime**: `Mutex<Option<i64>>`; None when stale/unverified.
- **U7 F9**: add test — expiry between two swaps ⇒ 2nd denies GateAbsent.
- **U8-U11** `Result<(),StartupRefusal>` (thiserror). vTPM config-parse reject + assert.
- **U12** manifest install+verify FATAL on profile=remote w/o operator_authorizer_url. Regen lock.
- **U13** events PresenceTokenAccepted{jti,expiry_ms}/Rejected{reason}/AuthorizerUnreachable{drained_streams}.
- **U14** edge/authorizer.rs async mTLS client; verify via engine seam; feed gate; drain+deny.
  Control-adjacent: holds only gate handle + verify fn, no Engine mut verbs (structural).
- **U16** config `Topology` enum, `operator_authorizer_url`, `--allow-passphrase-only`.

## Invariants (guardian gates)
No new deps → no-c Gate-4 green; one rustls ring-only; engine sync+non-printing; fail-closed +
default-OFF (VPS default OnBox; authorizer behind relay-edge); metadata-only events; FS-S26 preserved.

## Runtime surface (Phase 3.5)
- `cargo test -p envctl-secrets-engine` green (token sign/verify ordered rejects, gate states, guards).
- `cargo test -p envctl-secretd --features relay-edge --test profile_b_e2e` green incl. FS-S21/S22/
  S23/S24 negatives + authorizer round-trip + unreachable-drain.
- `secretctl authorizer status` runs; manifest FATAL gate fires (bash -n + grep sim).
