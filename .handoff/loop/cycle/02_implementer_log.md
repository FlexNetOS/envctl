# Implementation log: G2 — native GitHub App installation-token minting wired through secretd

Implemented the architect plan (`01_architect_plan.md`) in full, U1→U6, engine-first, in the
`g2-native-mint` worktree. DD-1 resolved via Option A (late-bind the provider on unlock). No commit
made (orchestrator commits after the guardian passes).

## Per-unit status

| Unit | Status | What landed |
|------|--------|-------------|
| U1 — `DaemonHttpTransport` | GREEN | New `crates/secretd/src/transport.rs`; reuses `proxy::build_upstream_client` (frozen webpki-roots/ring, `.no_proxy()`); sync→async via captured `Handle::block_on` off-reactor. |
| U2 — late-bind provider + App-credential custody | GREEN | `EngineInner.provider` → `RwLock<Box<dyn ProviderMint>>`; `install_provider`/`clear_provider` (ungated); `app_credential_pem`/`put_app_credential_meta` (gated); `lock()` clears the provider; daemon `rebuild_github_provider` on unlock RPC. |
| U3 — `resolve_injection` (mint / fallback / refuse) | GREEN | `Engine::resolve_injection` is the single native-subtoken decision site; grpc `mint` routes through it; injects the MINTED token, falls back to proxy-swap on `Unsupported`, REFUSES (durable Refused, `injection:None`) on `Other`. |
| U4 — repos/perms scope + `MintReq.mode` gap fix | GREEN | proto `MintReq` gains `mode`/`repos`/`perms`; `SwapMode::NativeSubToken` gains `repos`/`perms`; `mint_req_to_policy` now honors `req.mode` (was hardcoded `BaseUrlRepoint` ⇒ native was unreachable). |
| U5 — secretctl client surface | GREEN | `relay mint --mode/--provider/--repo/--perm`; pure builder `mint_req_for_relay_mint` (least-priv `checks:write` default for native github); `render_mint` notes native, never prints the token. CLI-only (GUI parity is a follow-up — see Deviations). |
| U6 — daemon e2e (mock GitHub) | GREEN | New `crates/secretd/tests/native_mint_e2e.rs`: 3 `#[tokio::test]` covering inject-minted-token + no-leak, HTTP-error-refuses, no-credential-falls-back. |

## Changes (files touched)

Engine (`crates/secrets-engine`):
- `src/lib.rs` — `EngineInner.provider: RwLock<Box<dyn ProviderMint>>` (was `Box<…>`); `with_seams` wraps in `RwLock::new` (signature unchanged → 3 callers source-compatible). New `pub type AppCredential`; meta-key helpers `app_id_meta_key`/`installation_id_meta_key`. New methods: `install_provider`, `clear_provider`, `app_credential_pem`, `put_app_credential_meta`, `resolve_injection`. `lock()` now calls `clear_provider()`. New `#[cfg(all(test, feature = "provider-github"))] mod native_mint_tests`.
- `src/broker/policy.rs` — `SwapMode::NativeSubToken { ttl_secs, #[serde(default)] repos, #[serde(default)] perms }` (back-compat for old serialized form).
- `tests/phase0.rs` — updated the `NativeSubToken` literal for the new fields.

Daemon (`crates/secretd`):
- `Cargo.toml` — new `provider-github` feature (forwards engine's); added to `default`.
- `src/lib.rs` — `#[cfg(feature = "provider-github")] pub mod transport;`.
- `src/transport.rs` — NEW. `DaemonHttpTransport` impl of `mint_github::HttpTransport`.
- `src/proxy.rs` — `build_upstream_client` is now `pub(crate)` (reused by transport; comment added).
- `src/conv.rs` — `swapmode_from_proto` gains `repos`/`perms`; `mint_req_to_policy` honors `req.mode` (the gap fix); `policy_from_proto` updated; new test `mint_req_with_native_mode_and_scope_builds_native_policy`; updated the `dataplane_mode_from_swap` test literal.
- `src/grpc.rs` — `mint` routes through `engine.resolve_injection` (native branch needs no proxy_addr); `unlock` calls `rebuild_github_provider` (non-fatal, mirrors `rebuild_ca_if_initialized`); `lock_now` calls `clear_provider`. New `#[cfg(feature = "provider-github")] fn rebuild_github_provider`.
- `tests/e2e.rs` — added the new `MintReq` fields to 4 literals (no behavior change).
- `tests/native_mint_e2e.rs` — NEW (U6).

Proto (`crates/secrets-proto`):
- `proto/control.proto` — `MintReq` gains `DataPlaneMode mode = 6; repeated string repos = 7; repeated string perms = 8;` (back-compat; default 0 = base-url = pre-G2).

CLI (`crates/secretctl`):
- `src/cli.rs` — `RelayCmd::Mint` gains `--mode`/`--provider`/`--repo`/`--perm`.
- `src/main.rs` — `Mint` handler uses new pure builder `mint_req_for_relay_mint`; `mint_req_for_run` updated for new fields; new test `mint_req_for_github_native_sets_mode_and_scope`.
- `src/render.rs` — `render_mint` detects native injection, prints the "TTL fixed ~1h by GitHub" note, never prints the token; JSON gains `"native"`.

## Engine API (the parity contract)

```rust
// late-bind seam (ungated — NoMint always available)
pub fn install_provider(&self, provider: Box<dyn ProviderMint>);
pub fn clear_provider(&self);

// App-credential custody (gated: provider-github)
pub type AppCredential = (Zeroizing<Vec<u8>>, String, u64); // (pem, app_id, installation_id)
pub fn app_credential_pem(&self, secret_name: &str) -> anyhow::Result<Option<AppCredential>>; // Err(Locked) when locked; Ok(None) when unenrolled
pub fn put_app_credential_meta(&self, secret_name: &str, app_id: &str, installation_id: u64) -> anyhow::Result<()>;

// the single native-subtoken decision site (ungated)
#[allow(clippy::too_many_arguments)]
pub fn resolve_injection(
    &self, provider: Provider, relay: &str, bearer: &str, proxy_url: &str, ca_pem_path: &str,
    mode: inject::DataPlaneMode, repos: Vec<String>, perms: Vec<String>, native_ttl_secs: i64,
    sink: &EventSink,
) -> anyhow::Result<Option<inject::ResolvedInjection>>; // Ok(None) == REFUSED (durable Refused row written)
```

`SwapMode::NativeSubToken` now carries `{ ttl_secs, repos, perms }`. Both front-ends (CLI today;
daemon `mint`) drive `resolve_injection` — the GUI is the only non-parity surface (no relay-mint
GUI exists yet; logic is fully engine-side, so the follow-up is pure wiring).

## Tests added (what they prove)

Engine `native_mint_tests` (8):
- `provider_install_replace_and_clear` — install swaps NoMint→minter→NoMint; native resolve mints when installed, falls back when cleared.
- `lock_clears_installed_provider` — `lock()` drops the minter (defense-in-depth).
- `app_credential_pem_reads_pem_and_meta_when_unlocked` / `..._refuses_when_locked` — custody read + the Locked fail-closed gate.
- `native_subtoken_injects_minted_token_not_bearer` — minted token in `GITHUB_TOKEN`/`GH_TOKEN`, relay bearer never injected, RelayMinted event carries relay+expires_at only, minted token absent from every event body.
- `native_subtoken_unsupported_falls_back_to_proxy_swap` — NoMint ⇒ `HttpsProxyMitm` shape with the relay bearer.
- `native_subtoken_other_error_refuses` — 404 ⇒ `Ok(None)` + `GuardRefused` + no token.
- `native_scope_threads_repos_and_perms_to_the_minter` — repos/perms reach `MintRequest` verbatim.

conv (1): `mint_req_with_native_mode_and_scope_builds_native_policy` — `mode=native` now builds a `NativeSubToken` swap with scope (the gap fix); `mode=0` stays base-url (back-compat).

secretctl (1): `mint_req_for_github_native_sets_mode_and_scope` — `--mode native --provider github` sets mode/provider/scope; default `checks:write`; explicit `--perm` overrides; non-native default unchanged.

transport (1): `execute_round_trips_against_a_loopback_server` — request shaping + sync→async bridge against a loopback server, driven from `spawn_blocking` (production call shape).

U6 e2e (3): `native_mint_injects_minted_token_and_event_never_leaks_it`, `native_mint_http_error_refuses_with_no_injection`, `native_mint_without_credential_falls_back_to_proxy_swap` — full daemon stack against a mock GitHub endpoint. Serialized by a `tokio::sync::Mutex` (process-global env vars) and use floor Argon2 params for speed.

## Build/test status (exact commands run + result)

- `cargo build -p envctl-secretd --features provider-github` — PASS. Also PASS: default features, and `--no-default-features --features mitm-ca` (provider-github off). Engine builds both default and `--features provider-github`.
- `cargo test -p envctl-secrets-engine --features provider-github` — PASS (112 lib + 4 + 6 + 17 + 15 integration; native_mint_tests all green).
- `cargo test -p envctl-secretd --features provider-github --lib` — PASS (31; conv U4 included).
- `cargo test -p envctl-secretd --features provider-github --test native_mint_e2e` — PASS (3/3), both single-threaded and parallel (serial-guarded).
- `cargo test -p envctl-secretctl` — PASS (4).
- `cargo test -p envctl-secretd --features provider-github --test e2e` — PASS (5/5; existing suite unaffected by the new `MintReq` fields).
- `cargo fmt --all` — clean (`--check` passes).
- `cargo clippy --workspace --features envctl-secretd/provider-github -- -D warnings` (the plan/CLAUDE.md gate form) — **PASS, clean.**
- CI gates: `no-c.sh` PASS (rustls=0.23.40 single, ring-only, zero aws-lc/openssl/C-SQLite — rsa/base64 are pure-Rust), `shape.sh` PASS, `enable.sh` PASS.

## Deviations

1. **`cargo clippy --workspace --all-targets`** (the *stricter* form, not the gate form) reports ONE
   pre-existing error in **untouched** `crates/gui/src/main.rs:1997` ("doc list item without
   indentation", inside a `#[cfg(test)]` doc comment) under clippy 1.96. It is NOT a G2 regression
   (gui is untouched; `git status` shows no gui changes) and does NOT fire under the plan's gate
   command (`cargo clippy --workspace -- -D warnings`, which is clean). All G2-touched crates are
   clean under `--all-targets`. Flagging so the guardian doesn't mis-attribute it.
2. **GUI parity** — G2 is CLI-only (the plan's R5; there is no relay-mint GUI surface today). The
   native-mint logic is entirely engine-side (`resolve_injection`), so the GUI follow-up is pure
   wiring. Noted as a parity follow-up, consistent with the plan.
3. **App-credential secret name** — the daemon's unlock rebuild reads a well-known secret name
   (`ENVCTL_GITHUB_APP_SECRET`, default `github_app`) rather than per-relay, because unlock has no
   relay context. `ENVCTL_GITHUB_API_BASE` overrides the REST base (GHES / the e2e mock; default
   real GitHub). Enrollment (`secretctl github-app enroll`) remains the immediate follow-up (plan
   R2); tests seed via `secret_put` + `put_app_credential_meta`.

## Handoff notes (targeted checks for the guardian)

- **Fail-closed refusal path** (invariant #4): verify `native_subtoken_other_error_refuses`
  (engine) + `native_mint_http_error_refuses_with_no_injection` (e2e) prove `Ok(None)` + a durable
  Refused row + NO token on HTTP/transport error. The grpc `mint` ships `injection: None` on refuse.
- **No secret on the wire / in logs/audit** (invariant): `resolve_injection`'s RelayMinted
  event + `relay_native_minted` audit carry only `relay` + `expires_at`; the e2e asserts the minted
  token never appears in the event-stream wire capture. The minted token is delivered ONLY in the
  injection (owner-only peercred-gated UDS). ScopedToken stays `Zeroizing` until the env String.
- **No-C / single-rustls** (invariants #1/#2): `provider-github` pulls only `rsa` + `base64`
  (pure-Rust RustCrypto, already declared); the transport reuses the existing rustls-ring reqwest
  client (no new dep). `no-c.sh` PASS confirms rustls=0.23.40 single, ring-only.
- **`lock()` clears the minter** (drops the Zeroizing App PEM) — engine `lock_clears_installed_provider`
  + the daemon `lock_now` also calls `clear_provider` (belt-and-suspenders). Confirm the locked-vault
  path holds no live App key.
- **Sync→async bridge** (R1): `DaemonHttpTransport` captures `Handle::current()` in `new()` (called
  from the async unlock RPC) and `block_on`s only inside `execute`, which runs on a `spawn_blocking`
  thread (off-reactor) — both via `run_streaming`'s spawn_blocking (unlock rebuild) and the mint's
  `spawn_blocking(resolve_injection)`. U6 exercises the live bridge against the mock.
- **`MintReq.mode` gap** (U4): before G2, `mint_req_to_policy` hardcoded `BaseUrlRepoint` so native
  was unreachable via Mint. Confirm `mint_req_with_native_mode_and_scope_builds_native_policy` and
  the e2e (which drives `mode=native` over the wire) prove it's now reachable, and that `mode=0`
  still yields base-url (back-compat — verified by the legacy arm of that test + unchanged e2e
  literals).
- **Back-compat of `SwapMode::NativeSubToken`**: the new `repos`/`perms` are `#[serde(default)]`, so
  an old serialized `{ "ttl_secs": N }` still deserializes (covered by `phase0.rs`'s round-trip).

STATUS: GREEN
