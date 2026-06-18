# Verification report: TASK-0031-PR2 — F2 relay-edge hardening (default-OFF `relay-edge`)

## Verdict — **PASS**

Independent cross-boundary verification of the working-tree TASK-0031-PR2 changeset on branch
`task-0031-pr2`. All four sub-features (server-issued DPoP-Nonce, per-IP token-bucket admission,
body caps + timeouts, opt-in mTLS) land clean. Every NON-NEGOTIABLE invariant holds; all real
gates + cargo checks are green from raw `rtk proxy` passthrough (verified exit codes, not the
implementer's word). ZERO new lockfile crates.

### Changeset scope (PR-2 = uncommitted working tree)
**The PR-2 work is the uncommitted working tree** — `git log origin/develop..HEAD` is EMPTY (HEAD =
`9ba53ae` TASK-0035 #108). 15 modified + 3 new untracked files:
`crates/secrets-engine/src/{broker/{nonce.rs(NEW),admission.rs(NEW),mod.rs},event.rs,lib.rs}`,
`crates/secrets-engine/Cargo.toml`, `crates/secretd/src/{config,conv,main,proxy}.rs`,
`crates/secretd/src/edge/{dpop,listener,mod,tls}.rs`,
`crates/secretd/tests/{edge_e2e,edge_stream_e2e}.rs`, NEW `crates/secretd/tests/edge_hardening_e2e.rs`.
**No proto change. No Cargo.lock change.**

## Gate results — exit codes captured
| Gate | Command | Exit | Result |
|------|---------|------|--------|
| no-c | `bash ci/gates/no-c.sh` | **0** | PASS — `rustls=['0.23.40'] on ring=['0.17.14']; zero aws-lc/openssl/C-SQLite` |
| shape | `bash ci/gates/shape.sh` | **0** | PASS — `SHAPE GATE PASS` |
| enable | `bash ci/gates/enable.sh` | **0** | PASS — `ENABLE GATE PASS` |
| p7 | `bash ci/gates/p7.sh` | **0** | PASS — `P7 GATE PASS` |

## cargo — exit codes captured
| Check | Command | Exit | Result |
|-------|---------|------|--------|
| fmt | `cargo fmt --all -- --check` | **0** | PASS |
| clippy (ws) | `cargo clippy --workspace --all-targets -- -D warnings` | **0** | PASS |
| clippy (relay-edge) | `cargo clippy -p envctl-secretd --features relay-edge --all-targets -- -D warnings` | **0** | PASS |
| test engine | `cargo test -p envctl-secrets-engine` | **0** | PASS — 133 lib (incl. **7 nonce + 5 admission**), relay 22, vault 15, inject 4, phase0 6; **0 failed** |
| test secretd relay-edge | `cargo test -p envctl-secretd --features relay-edge` | **0** | PASS — edge_hardening_e2e **4**, edge_e2e/stream, grpc_surface 6, mitm 1, native_mint 11, proxy_swap 2, self_check 2; **0 failed** |
| build relay-edge-OFF | `cargo build -p envctl-secretd` | **0** | PASS — OFF build unaffected (`challenge_nonce` gated) |

edge_hardening_e2e isolated run: `edge_mtls_requires_client_cert`, `edge_rate_limit_sheds_before_decide`,
`edge_nonce_and_anti_abuse`, `edge_body_caps_and_timeouts` — all 4 ok.

## Invariant checks
1. **No C / one ring-only rustls — PASS.** no-c.sh exit=0; `git diff origin/develop --stat Cargo.lock`
   EMPTY (ZERO new crates). `ring` made non-optional in the engine (Cargo.toml) — already in the
   resolved graph via rustls' ring provider, so zero lockfile delta. mTLS verifier built with the
   EXPLICIT ring provider: `tls.rs:119` `WebPkiClientVerifier::builder_with_provider(Arc::new(roots),
   Arc::new(rustls::crypto::ring::default_provider()))`. Grep for `aws_lc`/`aws-lc` in edge+broker:
   only doc-comment NEGATIONS ("never aws-lc-rs"). No sqlite/openssl/mimalloc.
2. **Engine = single sync non-printing library — PASS.** Grep `println!/eprintln!/print!/stdout` in
   nonce.rs/admission.rs/event.rs/lib.rs/broker/mod.rs: NONE. `NonceStore` (nonce.rs) + `AdmissionLimiter`
   (admission.rs) are sync, `std`+`ring::rand`-only, hold the security policy (issue/check_and_consume,
   token-bucket admit decision), return typed rejects (`NonceReject`, `Admit`). The edge does only I/O:
   emits headers (`challenge_nonce`), 429/413/408, drops.
3. **`decide()` is the SOLE Allow authority — PASS.** listener.rs: admission is STEP-0 (`:305-323`,
   before any crypto) and can ONLY reject early (`Throttled⇒429`). The nonce gate sits inside
   `verify_remote_presentation` (`:516-546`) AFTER `verify_dpop_proof`, BEFORE the jti record, and
   returns a `Refusal` (never a `RemotePeer`). The full verify ladder (EKM→DPoP→nonce→jti→client_id→
   registry/jkt) then `swap_and_respond_streaming` → `relay_swap`/`decide()` run on every non-shed req.
   e2e `edge_rate_limit_sheds_before_decide` (edge_hardening_e2e.rs:533-542) resets
   `RecordingUpstream.seen_key=None`, sends a 3rd request, asserts `429` AND `seen_key.is_none()` —
   proving the shed request NEVER reached the recording upstream. mTLS is ADDITIVE (same ServerConfig,
   extra client-cert gate) — DPoP/EKM/decide untouched.
4. **Fail-closed / no panic — PASS.** Matrix verified in source: missing/unknown/expired nonce⇒401
   re-challenge or fresh challenge; store-full-on-issue⇒401 no nonce (nonce.rs:98 `Err(())`); poisoned
   nonce/admit/jti lock⇒401/429 (every `.lock()` is matched, never unwrapped); rate breach⇒429;
   admission key-table-full+new key⇒Throttled (admission.rs:133, never evict-to-admit); body>cap⇒413;
   body-read timeout⇒408; handshake timeout⇒drop; mTLS required+no-CA⇒startup `Err` (mod.rs serve_edge
   `:bail!` + config.rs load-time `Err`). Single-use nonce: `check_and_consume` REMOVES on accept
   (nonce.rs:124 `remove`); unit `second_consume_is_unknown` + `concurrent_consume_single_winner` prove
   a second consume rejects. Grep of the listener production path (lines 290-608) for
   `unwrap/expect/panic/unreachable/unchecked-index`: NONE. (`challenge_nonce` uses
   `unwrap_or_else(bare(401))` — a fail-closed fallback, not a panic.)
5. **No secret bytes in logs/audit — PASS.** All new shed/refusal `tracing` lines are metadata-only:
   `peer` (source IP — operational, not secret), `status.as_u16()`, a fixed label. No bearer/proof/
   EKM/key/nonce-value/body is logged (the nonce is a public anti-replay token; the challenge log line
   carries no value). New `SecretEvent::EdgeRequestShed{reason,client_or_ip,count}` (event.rs) is
   metadata-only by construction and routes to CLI+GUI via conv.rs (no-proto-twin arm).
6. **relay-tls only / FS-S25 + EKM — PASS.** shape.sh exit=0. `tls.rs` imports NO MITM-CA type (grep
   `mitm/MitmCa/ca_pem` in tls.rs: NONE; local var renamed `ca_pem`→`anchors_pem`). The mTLS client-CA
   is a SEPARATE operator-provisioned input (`client_ca_path`, `load_client_ca_roots`) on the SAME
   relay-tls `ServerConfig` (`with_client_cert_verifier` on the same builder, `:125-128`) — never the
   MITM CA, never the server cert. The EKM path (`export_keying_material`, listener.rs:246) is untouched.
7. **Default-OFF `relay-edge`; mTLS additionally opt-in — PASS.** All new edge code is under
   `#[cfg(feature="relay-edge")]` (`challenge_nonce` gated; the whole `edge` module is feature-gated).
   `require_client_cert` defaults `false` (config.rs + EdgeConfig). `tls.rs` keeps `.with_no_client_auth()`
   BYTE-FOR-BYTE when `client_ca_path` is `None` (`:111-114`, the `load_from_dir` delegate path; unit
   `no_client_ca_matches_pr1_default`). relay-edge-OFF build: `cargo build -p envctl-secretd` exit=0.
8. **RFC 9449 correctness — PASS.** `challenge_nonce` (proxy.rs) emits `401` + `DPoP-Nonce: <nonce>` +
   `WWW-Authenticate: DPoP error="use_dpop_nonce"`, empty body. The proof's `nonce` claim is surfaced
   at parse (dpop.rs `VerifiedDpop.nonce`) and validated against `NonceStore` in the caller. e2e
   `edge_nonce_and_anti_abuse` exercises the full round-trip: nonce-less proof⇒401 + DPoP-Nonce header
   (`fetch_nonce`), retry echoing the nonce⇒200 with the real key reaching upstream
   (edge_hardening_e2e.rs:415-432), and stale/unknown nonce⇒401 carrying a FRESH challenge (:436-459).

## Parity check (front-end reach)
Edge-internal hardening; no new operator-facing Engine verb (the security stores are edge-driven
policy, like `JtiReplayStore`). The mTLS toggle reaches the daemon via config (`[edge].require_client_cert`
/ `client_ca_path`, env `SECRETD_EDGE_REQUIRE_CLIENT_CERT` / `SECRETD_EDGE_CLIENT_CA_PATH`) →
`EdgeConfig` → `serve_edge` → `serve_edge_listener` → `load_from_dir_with_client_auth`. The new
`SecretEvent::EdgeRequestShed` reaches CLI+GUI identically via the conv.rs no-proto-twin arm (same path
as `RelayStreamTornDown`). No CLI/GUI code change required or expected for this cycle.

## Findings
None blocking. Notes:
- **N1 (not a defect — branch hygiene):** `git diff origin/develop` shows `manifest/env-ctl.toml` +
  `manifest/envctl.lock` changes. These are NOT part of the PR-2 changeset — they are COMMITTED in the
  branch base (`ee24394` #115) and predate develop's #121; the diff appears only because the branch was
  cut before #121. `manifest/` is CLEAN in the working tree (not touched by PR-2). The orchestrator will
  reconcile branch-base divergence at merge/rebase; no code action.
- **N2 (deviation, accepted):** nonce is lowercase-hex (not base64url) to keep the always-built engine
  path dependency-free (`base64` is optional behind `provider-github`). A nonce is an opaque public
  token; any unambiguous encoding is equivalent. ZERO new dep. Acceptable.
- **N3 (deviation, accepted):** test RNG is the real `SystemRandom` (ring's `SecureRandom` is sealed —
  no seeded impl constructible). Tests assert behavior/bounds, never a nonce value; clock still injected.
  The architect's "seeded RNG" intent met by injection. Acceptable.
- **N4 (note):** `EdgeRequestShed` is wired through `conv.rs` and the shed paths log via
  `tracing::debug` (metadata-only) rather than `sink.emit`. The variant exists and is CLI+GUI-routable;
  surfacing it on the sink is a trivial no-secret-leak follow-up if desired. No invariant impact.

## Re-test needed
None — all gates + tests green on this changeset. If the optional `EdgeRequestShed` sink emission (N4)
is pursued, re-run `cargo test -p envctl-secrets-engine -p envctl-secretd --features relay-edge` +
`cargo clippy --workspace --all-targets -- -D warnings`.
