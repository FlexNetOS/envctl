# Implementation log: TASK-0033 — Secrets SERVER-MODE Profile B (VPS), in full

Branch: `task-0033-vps-profile-b` (off develop). All 19 units PRESENT + WIRED.

## Changes (files touched)
- crates/secrets-engine/src/seam.rs: TrustedTime trait + SystemClockTrustedTime + OperatorBoxTrustedTime (U5); Arc-forward impl.
- crates/secrets-engine/src/broker/authorizer.rs (NEW): PresenceToken, signing-bytes, sign/verify, AuthzReject, label (U2/U3/U4).
- crates/secrets-engine/src/broker/gate.rs: UnprovenGate + VpsPresenceGate (U6) + Arc-forward impl + tests.
- crates/secrets-engine/src/broker/mod.rs: re-exports.
- crates/secrets-engine/src/lib.rs: Topology enum; EngineInner gains presence_gate/trusted_time/topology (U1); with_seams +3 args; presence_proven dispatches on topology; verify_presence_token + assert_profile_b_startup + accessors; all callers updated.
- crates/secrets-engine/src/startup.rs (NEW): 4 startup guards + StartupRefusal (U8-U11) + tests.
- crates/secrets-engine/src/event.rs: PresenceTokenAccepted/Rejected/AuthorizerUnreachable (U13).
- crates/secrets-engine/tests/{relay,inject,vault}.rs: with_seams callers updated; relay.rs +2 F9 swap tests (U7).
- crates/secretd/src/edge/authorizer.rs (NEW): async mTLS authorizer link (U14).
- crates/secretd/src/edge/mod.rs: register module.
- crates/secretd/src/config.rs: Topology + [profile] + ProfileSettings::load + resolve_profile (U16) + 8 tests.
- crates/secretd/src/main.rs: --allow-passphrase-only; serve() startup guards + VPS seams + spawn authorizer link.
- crates/secretd/src/conv.rs: 3 new events -> no-proto-twin funnel.
- crates/secretd/src/proxy.rs + tests/*.rs: with_seams callers updated.
- crates/secretctl/{Cargo.toml,src/cli.rs,src/main.rs,src/authorizer.rs (NEW)}: authorizer serve|status (U15).
- manifest/env-ctl.toml: install+verify FATAL gate (U12); manifest/envctl.lock regenerated.
- docs/secrets/OI-SM-2-operator-authorizer.md (NEW, U17).
- crates/secretd/tests/profile_b_e2e.rs (NEW, U18): mTLS round-trip + FS-S21/S22/S23/S24 negatives.
- docs/secrets/{SERVER-MODE.md, audits/AUDIT-server-mode.md}: flipped non-shippable->RESOLVED (U19). THREAT-MODEL.md had no Profile-B rows.

## Engine API (parity contract)
- Engine::with_seams(.., presence_gate, trusted_time, topology) (+3 args; defaults UnprovenGate/SystemClockTrustedTime/OnBox).
- Engine::{topology, presence_gate_state, has_enabled_usb_keyslot, usb_possession_proven_pub, trusted_time_available, verify_presence_token, assert_profile_b_startup}.
- broker::{PresenceToken, sign_presence_token, verify_presence_token, AuthzReject, VpsPresenceGate, UnprovenGate}.
- seam::{TrustedTime, SystemClockTrustedTime, OperatorBoxTrustedTime}. startup::StartupRefusal. Topology enum.

## Tests added
- engine authorizer (12); engine gate (5 new); engine startup (10); engine relay (2 new, U7);
- secretd config (8); secretctl authorizer (3); secretd profile_b_e2e (9, relay-edge incl. full mTLS link round-trip).

## Build/test status — PASS
- cargo build -p envctl-secrets-engine -p envctl-secretd -p envctl-secretctl (+ --features relay-edge): PASS.
- cargo test -p envctl-secrets-engine: 213+ pass. profile_b_e2e (relay-edge): 9 pass. secretctl/secretd lib: pass.
- clippy --workspace -- -D warnings: engine/secretd/secretctl clean (CI invocation, no --all-targets). One INHERITED note vault/crypto.rs:100 (cfg(test) is_multiple_of) — pre-existing on develop, not in CI clippy, not mine.
- fmt clean. no-c gate PASS (one rustls 0.23.40 on ring; zero banned). lock --check rc=0 (79 components).

## FS-S21/S22/S23/S24 negative-test evidence
- FS-S21: fs_s21_vps_without_authorizer_url_refuses -> VpsNoSubstituteFactor; config test; manifest FATAL gate verified.
- FS-S22: fs_s22_onbox_unproven_usb_refuses_without_override -> OnBoxUsbKeyslotUnproven; passes with --allow-passphrase-only.
- FS-S23: fs_s23_no_token_gate_unproven_at_startup_refuses; fs_s23_token_expiry_between_swaps; authorizer_unreachable_clears_gate; relay vps_* deny GateAbsent.
- FS-S24: fs_s24_vtpm_gating_refused_engine_guard -> VtpmGatingForbidden; config parse reject.

## Deviations
- VpsPresenceGate uses AbsentSince(i64 wall-ms) NOT the task sketch's AbsentSince(Instant) — matches the LIVE gate.rs shape (prior deliberate epoch-ms deviation; Instant has no epoch anchor at the mapping site).
- presence_proven() dispatches on topology (OnBox body BYTE-IDENTICAL; Vps -> injected gate) rather than replacing wholesale, guaranteeing zero default-build drift while satisfying "resolve through the injected gate" for Profile B.
- Added tokio-rustls/rustls/rustls-pemfile/ring/sha2 as DIRECT secretctl deps for `authorizer serve` (operator mTLS signer). All ring-only + already in the resolved graph -> no new lockfile crate, no-c PASS.

## Handoff notes (for the guardian)
- verify_presence_token: confirm all 8 AuthzReject variants have dedicated tests; nonce-consume precedes expiry, jti-record last.
- FS-S23 load-bearing: gate re-resolved per swap (relay_swap_prepare -> presence_proven -> injected gate); F9 tests prove expiry-between-swaps flips 2nd to GateAbsent.
- Authorizer link is control-adjacent/issuance-only by construction (holds only gate + verify entrypoint; no Engine mut verb).
- mTLS ring-only both sides (build_client_config + build_acceptor builder_with_provider(ring)); no-c green.
- vault/crypto.rs:100 clippy note is inherited (develop, cfg(test), not in CI clippy) — a NOTE, not a regression.

## Re-run note
The cycle scratch files previously held TASK-0076 content; overwritten for TASK-0033.
