# Implementation log: TASK-0027 — GitHub installation-token early-revoke (DELETE /installation/token)

**STATUS: GREEN**

Additive `DELETE /installation/token` early-revoke through the EXISTING `HttpTransport` seam (zero new
deps). Exposed as a new `RevokeGithubToken` RPC + `secretctl github-app revoke-token` verb, plus a
best-effort `relay_revoke` native-plane tie-in. The frozen `mint-github` contract is untouched.

## Changes
- `crates/secrets-engine/src/mint_github.rs`: + `build_revoke_request(api_base, user_agent, installation_token) -> HttpRequest` (DELETE {base}/installation/token, token only in Authorization header, empty body, never-`{:?}`-log comment); + `revoke_installation_token<T: HttpTransport + ?Sized>(transport, api_base, user_agent, token) -> Result<(), MintError>` (204⇒Ok, transport/non-204⇒Err with ≤200-char snippet, no token); + 5 unit tests.
- `crates/secrets-engine/src/event.rs`: + metadata-only `SecretEvent::GithubTokenRevoked { installation_id: Option<u64>, outcome: String }` (outcome ∈ revoked/dry_run/best_effort_failed; never the token).
- `crates/secrets-engine/src/lib.rs`: + `pub use` of the two new mint_github fns; + `native_token_cache: Mutex<HashMap<String, Zeroizing<Vec<u8>>>>` field (cfg provider-github) populated in the `resolve_injection` NativeSubtoken success branch (replace prior), cleared in `clear_provider()` (⇒ also on `lock()`); + `revoke_github_token(token, apply, api_base, sink) -> anyhow::Result<bool>`; + `relay_revoke` best-effort tie-in (after the existing policy+bearer revoke; remove+revoke the cached relay token, success⇒revoked event/audit, failure⇒best_effort_failed audit/event SWALLOWED); + consts `GITHUB_API_BASE_DEFAULT`/`GITHUB_REVOKE_USER_AGENT`; + `use std::collections::HashMap` / `Mutex`; + 8 engine unit tests (native_mint_tests).
- `crates/secrets-proto/proto/control.proto`: + `rpc RevokeGithubToken (RevokeGithubTokenReq) returns (RevokeResp)` on `service Vault`; + `message RevokeGithubTokenReq { bytes token = 1; bool apply = 2; uint64 installation_id = 3; }` (reuses existing `RevokeResp`).
- `crates/secretd/src/grpc.rs`: + `revoke_github_token` Vault handler (empty-token⇒invalid_argument; token→Zeroizing; api_base from `ENVCTL_GITHUB_API_BASE`; installation_id `(req!=0).then_some`; spawn_blocking; errors via existing `map_mint_github_err`) + `#[cfg(not(feature="provider-github"))]` companion returning `Status::unimplemented`.
- `crates/secretd/src/conv.rs`: `GithubTokenRevoked` added to the no-proto-twin `return None` funnel (metadata-only, like RelayRevoked).
- `crates/secretctl/src/cli.rs`: + `GithubAppCmd::RevokeToken { token: String (--token, `-`=stdin / path / file), installation_id: Option<u64>, apply: bool }`.
- `crates/secretctl/src/main.rs`: `github_app` split into a dispatcher + `github_app_enroll` + new `github_app_revoke_token` (read token via `read_token` into Zeroizing, refuse empty; no `--apply`⇒stderr dry-run preview + optional `{"revoked":false,"dry_run":true}` json, no egress; `--apply`⇒`Vault.RevokeGithubToken`, drain RevokeResp, `{"revoked":<bool>,"dry_run":<bool>}` to stdout / human text to stderr; token never printed) + 3 clap round-trip tests; existing enroll tests adjusted to `let-else` (enum now multi-variant).
- `crates/secretd/tests/native_mint_e2e.rs`: + 3 e2e tests over the loopback mock-GitHub + `ENVCTL_GITHUB_API_BASE` harness (204⇒{count_revoked:1,dry_run:false}; dry-run contacts nothing; locked vault⇒failed_precondition).

## Engine API (as implemented)
```rust
// mint_github.rs
pub fn build_revoke_request(api_base: &str, user_agent: &str, installation_token: &[u8]) -> HttpRequest;
pub fn revoke_installation_token<T: HttpTransport + ?Sized>(
    transport: &T, api_base: &str, user_agent: &str, installation_token: &[u8],
) -> Result<(), MintError>;

// lib.rs (Engine, #[cfg(feature = "provider-github")])
pub fn revoke_github_token(
    &self, token: Zeroizing<Vec<u8>>, apply: bool, api_base: Option<String>, sink: &EventSink,
) -> anyhow::Result<bool>;

// event.rs
SecretEvent::GithubTokenRevoked { installation_id: Option<u64>, outcome: String }
```
Deviation from plan signature: `revoke_installation_token` has `T: HttpTransport + ?Sized` (not bare
`T: HttpTransport`) so the engine can pass `self.inner.github_transport.as_ref()` (`&dyn HttpTransport`,
unsized) — identical to how `mint_github_token` hands the boxed transport to `GitHubAppMint::new`. The
plan-named call surface is unchanged; this is the minimal bound needed to compile against the `Box<dyn>`
seam. (Justified, no behavior change.)

## Proto delta (as implemented)
```proto
service Vault { ... rpc RevokeGithubToken (RevokeGithubTokenReq) returns (RevokeResp); }
message RevokeGithubTokenReq { bytes token = 1; bool apply = 2; uint64 installation_id = 3; }
// RevokeResp reused as-is (count_revoked ∈ {0,1}, dry_run). Additive ⇒ wire round-trip drift test green.
```

## Tests added
- mint_github.rs (5): `revoke_builds_correct_delete_request`, `revoke_204_is_success`, `revoke_non_204_is_failure_without_token` (401, no token in err), `revoke_transport_error_is_failure`, `revoke_token_only_in_auth_header_not_in_error`.
- lib.rs native_mint_tests (8): `revoke_github_token_dry_run_no_egress`, `revoke_github_token_apply_204_succeeds_metadata_only`, `revoke_github_token_non_204_is_err_no_false_success`, `revoke_github_token_locked_vault_fails_closed`, `relay_revoke_native_tie_in_best_effort_success` (DELETE fired + revoked event), `relay_revoke_native_tie_in_best_effort_failure_still_returns` (500 ⇒ relay_revoke STILL returns + best_effort_failed event), `relay_revoke_dry_run_no_native_egress`, `lock_clears_native_token_cache` (post-lock relay_revoke fires no DELETE).
- secretctl (3): `github_app_revoke_token_parses_token_installation_and_apply`, `github_app_revoke_token_defaults_to_dry_run_and_accepts_stdin_dash`, `github_app_revoke_token_requires_token`.
- secretd e2e (3): `revoke_github_token_over_wire_204_succeeds`, `revoke_github_token_dry_run_contacts_nothing`, `revoke_github_token_locked_vault_fails_precondition`.

## Build/test status (exact commands + exit codes; all via `rtk proxy`)
- `cargo fmt --all -- --check` → **PASS** exit=0
- `cargo clippy --workspace --all-targets -- -D warnings` → **PASS** exit=0
- `cargo clippy -p envctl-secrets-engine --features provider-github --all-targets -- -D warnings` → **PASS** exit=0 (default workspace clippy doesn't enable engine `provider-github`; secretd's default DOES pull it transitively, and this explicit run covers the gated code directly)
- `cargo test -p envctl-secrets-engine` (default) → **PASS** exit=0
- `cargo test -p envctl-secrets-engine --features provider-github` → **PASS** exit=0 (lib 16 incl. revoke units; integration suites green)
- `cargo test -p envctl-secretd` → **PASS** exit=0 (14 native_mint_e2e incl. 3 new revoke tests; proxy_swap_e2e + self_check green)
- `cargo test -p envctl-secretctl` → **PASS** exit=0 (13 incl. 3 new clap tests)
- `bash ci/gates/no-c.sh` → **PASS** exit=0 (resolved graph unchanged: rustls 0.23.40 on ring 0.17.14, zero aws-lc/openssl/C-SQLite)
- `bash ci/gates/shape.sh` → **PASS** exit=0
- `git diff --stat` over all Cargo.toml + Cargo.lock → **empty** (ZERO new dependencies)

## Deviations
1. `revoke_installation_token` generic bound is `T: HttpTransport + ?Sized` (see Engine API note). Minimal, no behavior change.
2. `relay_revoke` best-effort tie-in targets `GITHUB_API_BASE_DEFAULT` (`https://api.github.com`) — the engine has no request-level api_base on the relay plane (the daemon's installed `GitHubAppMint` holds the GHES base, not visible to the engine here). The explicit-token verb DOES thread `api_base` (GHES-correct). A GHES relay's native early-revoke is therefore a documented best-effort limitation; its policy+bearer revoke remains authoritative. (Consistent with the plan's "best-effort, native-plane only" framing.)
3. grpc handler computes `_installation_id = (req.installation_id != 0).then_some(..)` as the plan specifies, but the engine method emits `installation_id: None` (the engine method takes no installation_id arg per the plan's signature), so it is bound-but-unused (`_`-prefixed). No functional impact.

## Handoff notes (for the invariant-guardian)
- **No new dep / no C**: `git diff` shows NO Cargo.toml/Cargo.lock change; `no-c.sh` green. Revoke reuses the existing `DaemonHttpTransport`/`HttpTransport` seam, `Zeroizing`, tonic/prost, clap.
- **Engine single sync non-printing authority**: request construction + 204/non-204 + the relay tie-in policy all live in `secrets-engine` via the `HttpTransport` seam (env-free); secretd supplies transport+RPC+env read; secretctl is thin. No `println!`/`eprintln!` added in `secrets-engine`; the engine emits `SecretEvent::GithubTokenRevoked`.
- **Fail-closed / no false success**: verify `revoke_github_token` returns `Err` on transport-error AND non-204 — covered by `revoke_github_token_non_204_is_err_no_false_success` + the mint_github.rs `revoke_non_204_is_failure_without_token`/`revoke_transport_error_is_failure`. `apply` defaults false everywhere (proto3 + clap); dry-run does NO egress — covered by `revoke_github_token_dry_run_no_egress` + e2e `revoke_github_token_dry_run_contacts_nothing` (unrouted base, no mock spawned).
- **No secret in logs/audit/err**: token is `Zeroizing`, lives ONLY in the revoke request's Authorization header; audit + event are metadata-only (installation_id, outcome). Token-leak guards: mint_github.rs `revoke_token_only_in_auth_header_not_in_error`, engine tests scan every emitted event JSON for the token, e2e scans the event-stream wire. The revoke `HttpRequest` is never `{:?}`-logged (comment in `build_revoke_request`).
- **Fail-closed cache clearing**: `native_token_cache` cleared on `clear_provider()` (and thus `lock()`) — `lock_clears_native_token_cache` proves a post-lock relay_revoke fires no DELETE. The cache lives only in `EngineInner` with `Zeroizing` values; never persisted.
- **relay_revoke still returns its bearer count on tie-in failure**: `relay_revoke_native_tie_in_best_effort_failure_still_returns` (500 DELETE ⇒ Ok + best_effort_failed event).
- **Frozen-contract safety**: `mint-github` flag/JSON shape + `MintGithub*` proto messages untouched (verify via diff of control.proto + secretctl mint path). Revoke is purely additive (new RPC/message/event/verb).
- **No grit/parallel mode**: sequential single-crew run (1 repo, 7 modules) — no grit locks claimed.
