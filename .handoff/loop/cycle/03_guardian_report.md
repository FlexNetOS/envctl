# Verification report: G2 — native GitHub App installation-token minting wired through secretd

## Verdict
**PASS-WITH-NOTES** — all non-negotiable invariants hold, all 4 CI gates PASS, fmt/clippy (gate
form) clean, every test suite green. Two NOTES (neither blocking): (1) `provider-github` was added
to secretd's `default` features — verified harmless (default + provider-off builds compile; no-c
graph clean); (2) GUI parity is a documented follow-up (CLI-only G2, logic fully engine-side).

## Gate results (exact exit codes)
| Gate | Result | Exit | Evidence |
|------|--------|------|----------|
| `bash ci/gates/no-c.sh` | **PASS** | 0 | `resolved graph clean: rustls=['0.23.40'] on ring=['0.17.14']; zero aws-lc/openssl/C-SQLite` |
| `bash ci/gates/shape.sh` | **PASS** | 0 | `SHAPE GATE PASS` |
| `bash ci/gates/enable.sh` | **PASS** | 0 | `ENABLE GATE PASS` |
| `bash ci/gates/p7.sh` | **PASS** | 0 | `P7 GATE PASS` |

The no-c gate reads the AUTHORITATIVE full `cargo metadata --format-version 1` graph. Because the
implementer added `provider-github` to secretd's `default`, that gate's resolved graph DOES exercise
the new `rsa` + `base64` deps — and it still reports exactly one rustls (0.23.40) on ring, zero
aws-lc/openssl/C-SQLite. The U1 risk (adding `provider-github` to secretd's graph) is therefore
closed by the gate itself. `rsa`/`base64`/`reqwest` are pre-existing **workspace** deps
(`Cargo.toml:88/89/65`, pure-Rust RustCrypto + rustls-tls); root `Cargo.toml` is UNCHANGED. No new
dependency was introduced (only secretd's feature-forward `provider-github =
["envctl-secrets-engine/provider-github"]`).

## cargo
| Check | Result | Exit | Notes |
|-------|--------|------|-------|
| `cargo fmt --all --check` (rtk proxy) | **PASS** | 0 | clean |
| `cargo clippy --workspace -- -D warnings` (CLAUDE.md/gate form) | **PASS** | 0 | clean |
| `cargo clippy --workspace --all-targets …` (stricter, not the gate) | 1 error | 101 | ONLY `crates/gui/src/main.rs:1997` — see Findings (NOT a G2 blocker) |
| `cargo build -p envctl-secretd` (default) | **PASS** | 0 | |
| `cargo build -p envctl-secretd --no-default-features --features mitm-ca` (provider-github OFF) | **PASS** | 0 | gated code does not break the no-feature build (invariant #6) |
| `cargo test -p envctl-secrets-engine --features provider-github` | **PASS** | 0 | 112 lib + 4 + 6 + 17 + 15; all 8 `native_mint_tests` green |
| `cargo test -p envctl-secretd --features provider-github` (all suites) | **PASS** | 0 | lib 31; e2e 5/5 (no regression); **native_mint_e2e 3/3**; mitm 1; proxy_swap 2; self_check 2 |
| `cargo test -p envctl-secretctl` | **PASS** | 0 | 4/4 incl. `mint_req_for_github_native_sets_mode_and_scope` |

native_mint_e2e (3/3): `native_mint_without_credential_falls_back_to_proxy_swap`,
`native_mint_http_error_refuses_with_no_injection`, `native_mint_injects_minted_token_and_event_never_leaks_it`.

## Invariant checks
1. **No-C trust boundary — PASS.** no-c.sh clean (above). No new dep; rsa/base64 pure-Rust;
   transport reuses `proxy::build_upstream_client` (frozen webpki-roots/ring, `.no_proxy()`) so no
   second TLS backend (`transport.rs:45`). Exactly one rustls (0.23.40) on ring.
2. **Fail-closed — PASS.** `Engine::resolve_injection` (`lib.rs:1665-1761`) is the single decision
   site:
   - locked vault ⇒ daemon `rebuild_github_provider` keeps `NoMint` (`grpc.rs:579` `Err(_) =>
     return`) ⇒ `MintError::Unsupported` ⇒ proxy-swap fallback (still safe; relay bearer, no native
     token).
   - transport/HTTP error ⇒ `MintError::Other` ⇒ `refuse(...)` + durable Refused row + `GuardRefused`
     event + `Ok(None)` = NO token (`lib.rs:1753-1758`); grpc ships `injection: None`
     (`grpc.rs:429/439`).
   - off-allowlist mint host ⇒ `mint_allowlisted` requires `Provider::Github` AND `api.github.com ∈
     canonical_upstreams` (`lib.rs:1693-1694`); else `Unsupported` (fallback, no off-host send).
   - non-201 from GitHub ⇒ `MintError::Other` (`mint_github.rs:208-217`) ⇒ refuse.
   - `MintError::Other` REFUSES while `MintError::Unsupported` FALLS BACK to proxy-swap
     (`lib.rs:1742-1759`) — confirmed: non-GitHub providers still work.
   - Refusal path is unit-tested (`native_subtoken_other_error_refuses`) AND e2e-tested
     (`native_mint_http_error_refuses_with_no_injection`).
3. **No secret in logs/audit/wire — PASS.** `relay_native_minted` audit + `RelayMinted` event carry
   only `relay` + `expires_at` (`lib.rs:1729-1739`); minted token never enters an event body.
   `ScopedToken.token` is `Zeroizing<Vec<u8>>` (`seam.rs:507-509`); App PEM is `Zeroizing`
   (`mint_github.rs:131/145`, `app_credential_pem` returns Zeroizing). `parse_token_response` moves
   the token straight into `Zeroizing` (`mint_github.rs:266`). Transport maps every error to a FIXED
   key-free `TransportError::Io` string — never the error text or URL (`transport.rs:56/64/74/80`).
   `clear_provider` drops the minter (and its Zeroizing PEM) on lock (`lib.rs:1580-1582`); called by
   `lock()` (`lib.rs:624`) AND the daemon lock RPC (`grpc.rs:552`). `render_mint` never prints the
   token — only bearer/token_id/expires_at/native flag (`render.rs`); minted token rides only in the
   injection (owner-only peercred-gated UDS). e2e asserts the minted token never appears in the
   event-stream wire capture (`native_mint_injects_minted_token_and_event_never_leaks_it`).
4. **Engine purity + parity — PASS (CLI-only, GUI noted).** Zero new
   `println!/eprintln!/print!/io::stdout` in the engine lib path (diff grep of lib.rs/mint_github.rs/
   seam.rs/broker/policy.rs = empty). All mint/resolve logic is in `secrets-engine`
   (`resolve_injection`); secretd only supplies transport + lock/unlock hooks; secretctl is a thin
   driver. CLI reaches `resolve_injection` (daemon `mint` RPC). GUI relay-mint surface does not exist
   yet — documented as a follow-up (plan R5, log Deviation #2), NOT silently skipped.
5. **GitHub ~1h TTL — PASS.** `expires_at` is parsed verbatim from GitHub's response body
   (`mint_github.rs:262-264`, `parse_token_response`) and surfaced honestly; NO fake clamp anywhere.
   Engine re-renders it from the epoch secs for the event (`lib.rs:1725-1727`).
6. **`provider` field change / default build — PASS.** `EngineInner.provider` →
   `RwLock<Box<dyn ProviderMint>>` with `with_seams` signature unchanged (wraps in `RwLock::new`);
   `cargo clippy --workspace` (builds all 3 `with_seams` callers) is clean ⇒ they compile.
   `provider-github`-gated code (`#[cfg(feature = "provider-github")]` on `app_credential_pem`,
   `put_app_credential_meta`, `transport` module, `rebuild_github_provider`) does NOT break the
   DEFAULT/no-feature build — verified `cargo build -p envctl-secretd --no-default-features --features
   mitm-ca` exit 0.

## Parity check
| Engine method | CLI caller | GUI caller |
|---------------|-----------|-----------|
| `Engine::resolve_injection` | secretd `mint` RPC `grpc.rs:383` (native) / `grpc.rs:413` (proxy/base), driven by `secretctl relay mint` | none yet — documented follow-up (plan R5, log Deviation #2); logic is fully engine-side so the GUI follow-up is pure wiring |
| `install_provider` / `clear_provider` | daemon unlock `grpc.rs:535` / lock `grpc.rs:552` + `Engine::lock` `lib.rs:624` | n/a (daemon-internal seam) |
| `app_credential_pem` | daemon `rebuild_github_provider` `grpc.rs:576` | n/a |

CLI-only surface is acceptable per the plan (the plan explicitly scoped G2 CLI-only with a GUI
parity follow-up). Parity is upheld because the decision logic lives in the engine, not in a
front-end.

## Findings
1. **severity: none (informational, NOT a G2 blocker)** — `crates/gui/src/main.rs:1997` ("doc list
   item without indentation", inside a `#[cfg(test)] mod` doc comment) fires ONLY under
   `cargo clippy --workspace --all-targets -- -D warnings` (the stricter form). Verified
   PRE-EXISTING: `crates/gui` is NOT in the G2 diff (`git diff --name-only` shows no gui files), the
   error is in an untouched file, and it does NOT fire under the plan/CLAUDE.md gate command
   (`cargo clippy --workspace -- -D warnings`, which is clean). All G2-touched crates are clean under
   `--all-targets`. The implementer's Deviation #1 is accurate; G2 did not introduce or move it.
2. **severity: note** — `provider-github` was added to secretd's `default` feature list
   (`Cargo.toml:17`). The plan said "add provider-github to secretd enabled features", so this is in
   scope, but making it a DEFAULT means the daemon ships native-mint code by default. Verified
   harmless: the gated code compiles cleanly when toggled OFF, the no-c graph (which now includes
   rsa/base64 by default) stays clean, and the default `mint` path still falls back to proxy-swap
   when no App credential is enrolled. No action required; flagged for awareness.
3. **severity: note** — App-credential secret name is a well-known env (`ENVCTL_GITHUB_APP_SECRET`,
   default `github_app`) rather than per-relay, because unlock has no relay context (log Deviation
   #3). Consistent with the plan's enrollment-follow-up (R2). Enrollment verb
   (`secretctl github-app enroll`) remains the immediate follow-up — out of G2 scope.

## Re-test needed
None — verdict stands. If any fix touches the trust boundary or deps, re-run:
```
bash ci/gates/no-c.sh
cargo build -p envctl-secretd --no-default-features --features mitm-ca
cargo test -p envctl-secrets-engine --features provider-github
cargo test -p envctl-secretd --features provider-github
```

VERDICT: PASS-WITH-NOTES
