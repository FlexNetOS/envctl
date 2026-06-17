# G2 — native GitHub App installation-token minting wired end-to-end through secretd

VERDICT: GO

## Target repos
- **envctl** (single repo). Crates touched: `secrets-engine` (lib.rs, seam.rs, inject.rs, mint_github.rs, broker/policy.rs), `secretd` (main.rs, grpc.rs, conv.rs, proxy.rs, transport.rs[new], tests/), `secrets-proto` (control.proto), `secretctl` (main.rs, cli). **5 crates, ~10 modules** → single-repo, >3 modules ⇒ route as **sequential single-crew** (units are linearly dependent U1→U6, so no intra-repo parallel benefit). No A2.

## DD-1 resolution — Option A (late-bind the provider on unlock), CHOSEN
Make `EngineInner.provider` a `RwLock<Box<dyn ProviderMint>>`; add `Engine::install_provider`/`clear_provider`; on vault **unlock** the daemon reads the App-credential secret from the now-unlocked vault, builds `GitHubAppMint::new(app_id, installation_id, pem, SystemClock, DaemonHttpTransport)`, installs it; on **lock** reinstall `NoMint` (drops the `Zeroizing` PEM). Mirrors the existing **`mitm-ca` rebuild-on-unlock precedent** at `lib.rs:514-518` / `rebuild_ca_if_initialized` (sealed CA key likewise opens only against the live DEK).

Rejected B (ephemeral per-mint minter + held HttpTransport seam): forces the engine to know App-credential storage + re-open the secret on every mint, splits the seam in two, widens `with_seams`. Rejected startup-construction: impossible (PEM unsealable only post-unlock — this is why DD-1 is forced).

### Engine API delta + blast radius
- Field `crates/secrets-engine/src/lib.rs:124`: `provider: Box<dyn ProviderMint>` → `provider: RwLock<Box<dyn ProviderMint>>`. `with_seams` signature UNCHANGED (wraps in `RwLock::new` at ~:221) → its 3 callers (`open_with_store` lib.rs:191, `engine_with_daemon_seams` secretd main.rs:264, test `unlocked_engine` lib.rs:2592) stay source-compatible. `self.provider.mint_scoped()` has **zero non-test callers today**. Risk: Low.
- New API (generic install/clear ungated; `GitHubAppMint`/`HttpTransport`-naming code `#[cfg(feature = "provider-github")]`):
```rust
pub fn install_provider(&self, provider: Box<dyn ProviderMint>) { *self.inner.provider.write().expect("provider lock") = provider; }
pub fn clear_provider(&self) { *self.inner.provider.write().expect("provider lock") = Box::new(seam::NoMint); }
```
Read side: `self.inner.provider.read().expect("provider lock").mint_scoped(&req)`.

## Vault credential convention
Referenced by the relay policy's existing `secret_name` (e.g. `"github_app/flexnetos"`):
- **PEM** stored at the secret name itself (provider `Github`, `broker_only = true`); read via existing **`open_real_key(dek, secret_name)`** (lib.rs:1518) — no new crypto path.
- **`app_id`** + **`installation_id`** as vault **meta keys** `"{secret_name}.app_id"` / `"{secret_name}.installation_id"` (integrity-covered by the header MAC).
Enrollment (`secretctl github-app enroll`) is a FOLLOW-UP, not a G2 blocker — operator/test seeds via existing `secret_put` + `put_meta`.

## Units (engine-first, one PR each)

### U1 — DaemonHttpTransport (secretd `impl HttpTransport`)
- New `crates/secretd/src/transport.rs`; `lib.rs` `pub mod transport;`; add `provider-github` to secretd enabled features; expose `crate::proxy::build_upstream_client()`.
- `struct DaemonHttpTransport { client: reqwest::Client, rt: tokio::runtime::Handle }`; `impl mint_github::HttpTransport`.
- **Sync→async bridge (load-bearing):** `execute` is sync, runs inside `spawn_blocking` (grpc.rs:340) → use captured `Handle::current().block_on(...)` (NOT on a reactor thread). Mirror libSQL store off-reactor block_on (lib.rs:184-186). reqwest error → `TransportError::Io(<fixed key-free string>)` (mirror DaemonUpstream "never echo error text", proxy.rs:308).
- Invariant: reuses `build_upstream_client` ⇒ frozen webpki-roots/ring, `.no_proxy()` (FS-S7/CF-6); NO new dep.
- Test: `#[cfg(test)]` request-shaping round-trip (no network).
- Deps: none.

### U2 / DD-1 — late-bind provider + App-credential custody
- Engine: field→RwLock; `install_provider`/`clear_provider`; ungated accessor `app_credential_pem(&self, secret_name) -> Result<Option<(Zeroizing<Vec<u8>>, String, u64)>>` (reads PEM via `open_real_key` + meta from UNLOCKED vault; Locked ⇒ Err); `lock()` also calls `clear_provider()` (defense-in-depth).
- Daemon owns the rebuild (engine can't name `DaemonHttpTransport`): unlock RPC handler calls `engine.app_credential_pem(secret_name)`, constructs `GitHubAppMint` w/ `DaemonHttpTransport`, `engine.install_provider(...)`; lock RPC handler calls `engine.clear_provider()`. Failure non-fatal to unlock (mirror `rebuild_ca_if_initialized`): audit, keep NoMint.
- Invariant: App PEM materializes only from unlocked vault (fail-closed); `clear_provider` drops Zeroizing PEM on lock; metadata-only audit.
- Test: engine `provider_install_replace_and_clear`.
- Deps: U1.

### U3 — wire NativeSubToken resolve → mint_scoped, inject ScopedToken, fall back on Unsupported
- Move native resolution INTO the engine: `Engine::resolve_injection(...)` — for `NativeSubtoken` calls `self.inner.provider.read()…mint_scoped(&MintRequest{provider, repos, perms, ttl_secs})`; `Ok(scoped)` ⇒ inject the minted token (NOT the relay bearer) into `provider_key_vars(p)`; `Err(Unsupported)` ⇒ fall back to proxy-swap shape; `Err(Other)` ⇒ refuse (durable Refused, `injection: None`, no token).
- `grpc.rs:361` native path calls the new engine `resolve_injection` (so `mint_scoped` lives in the engine). `injection_template` stays pure for non-native modes.
- Invariant: ScopedToken Zeroizing → env String only; RelayMinted event/audit = token_id+expires_at only, NEVER the token; mint host (`api.github.com`) asserted ∈ GitHub-mint allowlist before send (fail-closed if api_base off-allowlist; tests use `gh.test`).
- Tests: `native_subtoken_injects_minted_token_not_bearer`; `..._unsupported_falls_back_to_proxy_swap`; `..._other_error_refuses`.
- Deps: U2.

### U4 — repos/perms scope + the MintReq.mode gap fix
- Proto `MintReq`: add `DataPlaneMode mode = 6; repeated string repos = 7; repeated string perms = 8;` (back-compat; empty defaults = today's behavior). **Fixes latent gap:** `mint_req_to_policy` hardcodes `BaseUrlRepoint` (conv.rs:223) so NativeSubtoken is currently UNREACHABLE via Mint.
- Scope on `SwapMode::NativeSubToken { ttl_secs, repos: Vec<String>, perms: Vec<String> }` (not RelayPolicy — only meaningful for native mode). Empty perms ⇒ full installation scope (build_token_request_body mint_github.rs:225). Least-privilege default `["checks:write"]` lives in the U5 client surface, not the engine.
- `conv::mint_req_to_policy` maps `req.mode`→`swapmode_from_proto(req.mode, "", req.ttl_secs)` carrying repos/perms.
- Tests: conv `mint_req_with_native_mode_and_scope_builds_native_policy`; engine scope→MintRequest.
- Deps: U3.

### U5 — secretctl client surface
- `RelayCmd::Mint` (main.rs:372) + cli args `--mode native`, `--repo <r>...`, `--perm <p>...`, `--provider github`. Today hardcodes `provider: Generic`, no mode (:384). `--perm` defaults `["checks:write"]` for `--mode native --provider github`. `render_mint` notes "native GitHub installation token (TTL fixed ~1h by GitHub)"; never prints the token.
- CLI-only (no GUI relay-mint surface today; parity follow-up — logic is engine-side).
- Test: `mint_req_for_github_native_sets_mode_and_scope` (pure builder, like `mint_req_for_run` :441).
- Deps: U4.

### U6 — daemon e2e (mock GitHub endpoint)
- New `crates/secretd/tests/native_mint_e2e.rs` `#[tokio::test]`: mock HTTP server returns 201 + `{"token":"ghs_...","expires_at":"...Z"}`; init+unlock InMemStore engine; seed test PEM + meta; `GitHubAppMint::with_api_base` → mock; drive `Mint(mode=native, provider=github, perms=["checks:write"])`; assert (a) `ResolvedInjection.env[GITHUB_TOKEN]` == minted token (not bearer); (b) `expires_at` == GitHub value; (c) 404/500 ⇒ refuse (no injection, durable Refused); (d) locked vault ⇒ NoMint ⇒ falls through/refuses.
- Invariant: assert minted token NEVER in audit/event bodies (only token_id/expires_at); fail-closed on transport/HTTP error.
- Deps: U1–U5.

## Sequencing
U1 → U2 → U3 → U4 → U5 → U6. **In U1's PR:** add `provider-github` to secretd enabled features (currently enabled in NO crate) so `mint_github` + `HttpTransport` compile in the daemon; guard `GitHubAppMint`/`HttpTransport`-naming engine code with `#[cfg(feature = "provider-github")]`; keep install/clear ungated. Each PR: `cargo fmt --all && cargo clippy --workspace -- -D warnings` + 4 CI gates.

## CI gates touched
- **no-c.sh** — TOUCHED (adds provider-github to secretd graph). Confirm rsa/base64/reqwest resolve to existing pure-Rust/ring set; no SQLite/OpenSSL/aws-lc; exactly one rustls (ring). Run after U1.
- **shape.sh** — TOUCHED (engine stays non-printing; logic in engine not main/gui; no `println!` in engine).
- **enable.sh** — not touched. **p7.sh** — not touched.
- Tests: `cargo test -p envctl-secrets-engine --features provider-github`; `cargo test -p envctl-secretd --features provider-github --test native_mint_e2e`.

## Risks & open questions
- **R1 async bridge:** capture `Handle::current()` at `DaemonHttpTransport::new()` (called from async unlock RPC), `block_on` only inside `execute` (runs on blocking thread). Mirror libSQL off-reactor block_on. U6 exercises it.
- **R2 enrollment:** `secretctl github-app enroll` is the immediate follow-up; seed via `secret_put`+`put_meta` for now. Out of G2 scope.
- **R3 early revoke** (`DELETE /installation/token`): out of scope; 1h expiry is the kill switch.
- **R4 stateless ghs_ format:** broker passes token opaquely (parse_token_response reads `token` as string). No action.
- **R5 GUI parity:** CLI-only for G2; GUI follow-up.
- No owner decision required — GO. (App G1 = post verdict check-run, G5 = branch-protection flip are NOT envctl, out of scope.)
