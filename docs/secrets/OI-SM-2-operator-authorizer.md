# OI-SM-2 — Operator-box presence-token authorizer (design)

Status: ACTIVE (resolves audit open-item OI-SM-2 + OI-SM-3; unblocks TASK-0033 / audit F7/F8/F9)
Scope: the operator-box-signed, short-lived **presence token** that substitutes for on-box USB
possession when `secretd` runs OFF the operator box (a VPS — SERVER-MODE Profile B), and the
external **trusted time** (OI-SM-3) the VPS uses to verify it.
Corpus: SERVER-MODE.md §6 (Profile B), audits/AUDIT-server-mode.md F7/F8/F9 (lines 71-73) +
FS-S21/S22/S23/S24 (lines 306-309) + OI-SM-2/OI-SM-3 (line 321), THREAT-MODEL A19/A20,
OI-SM-1 (the sibling `jti`/nonce stores this reuses).

This spec resolves the operator-box authorizer protocol (OI-SM-2) and the external trusted-time
requirement (OI-SM-3). It builds ON the OI-SM-1 [`NonceStore`] + [`JtiReplayStore`] (the same
bounded, fail-closed anti-replay primitives) rather than reinventing them.

## 1. What a presence token is and the threat

In Profile A (on-box) the egress presence gate's possession factor is a **live USB/Seed probe**:
the daemon proves it can read the keyfile (or get a fresh Ed25519 signature from the Cognitum
Seed). That probe is meaningless on a VPS — there is no USB to physically possess, and a VPS clock
+ filesystem are hypervisor-controlled. Without a substitute, a VPS would either run UNGATED (the
gate backs nothing — FS-S21/FS-S22) or trust a boot-unwrapped DEK forever (FS-S23).

A **presence token** is the substitute: a short-lived, **Ed25519-signed** assertion minted on the
**operator box** (which DOES hold the USB/Seed) and pushed to the VPS over mTLS. A valid token is
proof that, *recently* (within the token TTL), the operator box still held possession — exactly the
property the on-box probe gives Profile A. When the operator box loses possession (USB pulled), it
stops minting; the VPS's last token expires within the TTL and the gate flips closed.

Threats this closes:
- **THREAT-MODEL A19 — "VPS runs with no possession factor (silent USB-gating downgrade)."** The
  install-time gate (F7/FS-S21) + the startup guard refuse a VPS with no configured authorizer.
- **THREAT-MODEL A20 — "captured presence token replayed."** Channel binding + a single-use server
  nonce + a `jti` replay store make a captured token unusable elsewhere or twice.

What it does NOT defend: operator-box compromise (an attacker who owns the operator box can mint
tokens) — that is out of scope, the operator box is the root of trust by construction (it holds the
USB). The token bounds *VPS* exposure, not operator-box compromise.

## 2. Token format

The token is a serde struct ([`broker::authorizer::PresenceToken`]); carries NO secret material —
the signature authenticates it and every field is public metadata:

| field | type | meaning |
|-------|------|---------|
| `v` | `u8` = 1 | wire version; any other value is rejected before crypto |
| `ts_ms` | `i64` | mint time (operator-box trusted wall-clock epoch-ms) |
| `vps_instance_id` | `String` | the VPS this token authorizes (binds to one deployment) |
| `server_nonce` | `String` | the VPS-issued single-use nonce the token answers |
| `vps_cert_fp` | `[u8;32]` | SHA-256 of the VPS edge cert (channel binding) |
| `expiry_ms` | `i64` | absolute expiry (operator-box trusted epoch-ms) |
| `jti` | `String` | unique token id (replay defense) |

## 3. Signed bytes (the binding)

The Ed25519 signature covers [`presence_token_signing_bytes`] — a canonical, domain-separated,
length-prefixed encoding (the same no-collision discipline as `bearer_row_mac_message`):

```
b"env-ctl/v1/presence-token"
  ‖ v (u8)
  ‖ ts_ms.to_be_bytes
  ‖ (len u32 be)‖vps_instance_id
  ‖ (len u32 be)‖server_nonce
  ‖ vps_cert_fp[32]
  ‖ expiry_ms.to_be_bytes
  ‖ (len u32 be)‖jti
```

Every variable-length field is length-prefixed and every fixed field is fixed-width, so no two
distinct tokens can collide on the same message (no boundary-shift forgery). The domain prefix is
distinct from every other signed/MAC'd message in the crate, so a presence-token signature can never
be confused with a bearer-row MAC, a Seed KEK signature, or the audit-head anchor.

The token is bound THREE ways: (a) to the operator key (the signature), (b) to THIS VPS endpoint
(`vps_cert_fp` channel binding — a stripped-mTLS man-in-the-middle cannot forward it elsewhere), and
(c) to a live VPS challenge (`server_nonce` single-use).

## 4. TTL

The operator box mints with a short lifetime so a captured token cannot keep a VPS authorized long
after possession is lost; a fresh mint re-checks USB possession on the operator box. The audited
band is **5–15 min, default 10 min** ([`DEFAULT_TOKEN_TTL_MS`] = 600_000; the signer clamps
`--ttl-secs` into 300–900s). The VPS authorizer link refreshes well inside the TTL
([`REFRESH_INTERVAL`] = 120s) so the gate never flaps closed under normal operation.

## 5. Verify algorithm (ordered, fail-closed ladder)

[`verify_presence_token`] runs on the VPS, under the authorizer's lock (the nonce/jti stores are
single-use and consumed atomically). The order is cheap/structural → crypto → binding → liveness →
replay, so a malformed/forged/expired token never burns a live nonce:

1. **version** — `v == 1` else `MalformedVersion` (before any crypto).
2. **trusted time** (OI-SM-3) — `trusted_time.now_ms()` is `Some(t)` else `TrustedTimeUnavailable`.
3. **signature** — ring Ed25519 verify over the signing bytes with the pinned operator pubkey, else
   `BadSignature`.
4. **cert binding** — `vps_cert_fp == expected` else `CertFpMismatch`.
5. **nonce** — [`NonceStore::check_and_consume`] (single-use) else `NonceUnknown`.
6. **validity** — `t < expiry_ms` else `Expired`; `ts_ms <= t + skew` else `NotYetValid`
   (`TOKEN_SKEW_MS` = 30s).
7. **replay** — [`JtiReplayStore::check_and_record`] (scoped by `vps_instance_id`) else `Replayed`.

Every failure is a typed `AuthzReject`; there is NO accept-on-error path. The engine owns this
ladder (sync, non-printing); the daemon's authorizer link only feeds it I/O.

## 6. Replay window

Replay defense is two single-use stores (reused from OI-SM-1):
- **server nonce** — the VPS mints a fresh nonce per challenge ([`NonceStore`], 256-bit random,
  `NONCE_TTL_MS` = 5 min) and consumes it on accept, so the SAME token presented twice (or a
  different valid token reusing a spent nonce) fails `NonceUnknown`.
- **`jti`** — recorded in [`JtiReplayStore`] keyed by `vps_instance_id`, so a token whose nonce
  somehow passed but whose `jti` was already accepted fails `Replayed`. The store is bounded
  (fail-closed cap, time-swept against the token TTL band), same as the DPoP `jti` store.

## 7. Outage behavior (OI-SM-3 + FS-S23)

- **Trusted time stale/unavailable (OI-SM-3).** A VPS clock is hypervisor-controlled, so the
  verifier NEVER trusts the local wall clock for expiry. It uses [`OperatorBoxTrustedTime`], fed by
  the operator's attested time on each fetch; when no fresh attestation exists (stale beyond the
  freshness window) it returns `None` and verify refuses `TrustedTimeUnavailable`. Fail-closed: a
  VPS that loses its trusted-time feed denies new egress.
- **Authorizer unreachable (FS-S23).** A connect/fetch/verify failure CLEARS the VPS gate
  ([`VpsPresenceGate::clear`]). New swaps then deny `GateAbsent`, and the per-stream re-check (which
  re-reads the gate fresh each tick — FS-S23: the engine never caches a gate result across swaps)
  tears in-flight relay streams down. A metadata-only `AuthorizerUnreachable` event is emitted (no
  token/sig/key bytes).
- **Wrapping vs gating (FS-S23).** The DEK may be unwrapped at boot (so the daemon can serve once
  authorized), but egress is GATED on a currently-valid token re-resolved per swap — a boot-unwrapped
  DEK never authorizes egress on its own.

## 8. Forbidden states this resolves

| ID | forbidden state | how it's refused |
|----|-----------------|------------------|
| FS-S21 | VPS with no substitute factor | install-hook FATAL + `assert_vps_factor_configured` startup guard |
| FS-S22 | on-box with enrolled-but-unproven USB keyslot serving USB-gated egress | `assert_onbox_usb_keyslot_or_override` (refuse unless `--allow-passphrase-only`) |
| FS-S23 | VPS egress on a boot-unwrapped DEK with no valid token | per-swap gate re-resolve + `assert_gate_not_unproven_at_startup` |
| FS-S24 | vTPM-gated DEK release | config-parse reject + `assert_no_vtpm_gating` |

## 9. Architecture (who owns what)

- **Engine** (`secrets-engine`, sync, non-printing): the token type, signing bytes, sign/verify
  primitives, the VPS gate + trusted-time seams, and the startup guards. The verify POLICY.
- **secretd** (`edge::authorizer`, async, `relay-edge` feature): the I/O — the mTLS client that
  fetches tokens, feeds the shared gate/trusted-time, and fails closed on unreachable. It is
  control-adjacent and issuance-only: it holds ONLY the gate + verify entrypoint, with NO path to
  any vault-management verb (structural).
- **secretctl** (`authorizer serve|status`): the operator-box signer (mTLS server reusing
  `engine::sign_presence_token`) + local profile status. Thin.
