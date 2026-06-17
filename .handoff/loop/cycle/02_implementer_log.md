# Implementation log: TASK-0020-COMPLETE — FROZEN `mint-github` consumer-contract surface

STATUS: GREEN

Base: `g2-mint-github` off `origin/g2-native-mint` (714b187). REUSED (not rebuilt) the G2 primitive
+ `DaemonHttpTransport` + `GitHubAppMint`/`mint_scoped`/`build_token_request_body` per the plan.

## Changes
- `crates/secrets-proto/proto/control.proto`: added `rpc MintGithub(MintGithubReq) returns (MintGithubResp)` to `service Vault`; added the two frozen messages (`MintGithubReq{installation_id,repository_ids,permissions,ttl_secs}`, `MintGithubResp{token,expires_at_unix}`).
- `crates/secrets-engine/src/seam.rs`: added `repo_ids: Vec<u64>` to `MintRequest` (mutually exclusive with `repos`); blanket `impl<C: Clock + ?Sized> Clock for &C` so a borrowed `&dyn Clock` satisfies the generic bound on `GitHubAppMint::new`.
- `crates/secrets-engine/src/mint_github.rs`: added `NoopHttpTransport` (fail-closed default seam), `GithubMintParams{installation_id,repository_ids,permissions,ttl_secs,api_base}`, blanket `impl<T: HttpTransport + ?Sized> HttpTransport for &T`; taught `build_token_request_body(repos, repo_ids, perms)` to emit a numeric `repository_ids` array and REJECT `repos`+`repo_ids` together (GitHub 422); 2 new unit tests.
- `crates/secrets-engine/src/lib.rs`: added `github_transport: Box<dyn HttpTransport>` field to `EngineInner` + the `#[cfg(feature="provider-github")]` param to `with_seams` (default `NoopHttpTransport`); flat secret-name consts `GITHUB_APP_KEY_NAME`/`GITHUB_APP_ID_META`; `Engine::put_github_app_id` (TASK-0026 enroll seam) + `Engine::mint_github_token(params, sink) -> ScopedToken` (broker-only unseal vs live DEK, per-call `GitHubAppMint`, metadata-only audit/event, fail-closed expiry/empty checks); updated all in-lib + test `with_seams` callers; 4 new unit tests in `native_mint_tests`.
- `crates/secretd/src/transport.rs`: added `DaemonHttpTransport::from_handle(rt)` (off-reactor-safe constructor for the libSQL `spawn_blocking` path).
- `crates/secretd/src/main.rs`: `build_engine` captures `Handle::current()` and threads it through `engine_with_daemon_seams`, which now installs `DaemonHttpTransport::from_handle(rt)` as `github_transport` (`#[cfg(feature="provider-github")]`).
- `crates/secretd/src/grpc.rs`: `Vault::mint_github` handler (numeric `repository_ids` parse + non-numeric reject → `invalid_argument`; `ttl_secs` non-negative check; `spawn_blocking(engine.mint_github_token)`; `ENVCTL_GITHUB_API_BASE` override read in the daemon, engine stays env-free) + `map_mint_github_err` (locked→`failed_precondition`, transport/HTTP→`unavailable`, else→`permission_denied`); `#[cfg(not(provider-github))]` `Unimplemented` stub so the trait is satisfied feature-off; token never logged, materialized to `String` only at the response.
- `crates/secretctl/src/cli.rs`: top-level `mint-github` subcommand `MintGithubArgs` with the EXACT frozen flags (`--installation-id` required, `--repository-ids a,b` comma-split, `--permissions name:access,...` comma-split verbatim, `--ttl-secs` required, `--output` required).
- `crates/secretctl/src/main.rs`: `mint_github` dispatcher — `--output json` only; prints ONLY the compact two-field `{"token":"…","expires_at_unix":<i64>}` to stdout via `serde_json` (no serde-derive dep); 3 new tests incl. the differential contract test.
- `crates/secretd/tests/native_mint_e2e.rs`: 3 new daemon e2e tests for `Vault.MintGithub` (frozen response, non-numeric-id reject, locked-vault refusal) against the mock GitHub.
- `crates/secrets-engine/tests/{relay,inject,vault}.rs`, `crates/secretd/tests/{e2e,mitm_e2e,proxy_swap_e2e}.rs`, `crates/secretd/src/proxy.rs`: appended the `#[cfg(feature="provider-github")] Box::new(NoopHttpTransport)` arg to each `with_seams` caller.

## Engine API (the parity contract)
- `Engine::mint_github_token(&self, params: GithubMintParams, sink: &EventSink) -> anyhow::Result<ScopedToken>` — per-call mint; requires Unlocked (`EngineError::Locked` else); opens `github-app-private-key` broker-only vs live DEK; reads `github-app-id`; builds a per-call `GitHubAppMint`; metadata-only audit (`github_token_minted`: installation_id, repo/perm counts, expires_at) + `RelayMinted` event; rejects non-positive expiry / empty token. Token stays `Zeroizing`.
- `Engine::put_github_app_id(&self, app_id: &str)` — flat App-id enrollment seam (Unlocked-gated; for TASK-0026 `secretctl github-app enroll`).
- `Engine::with_seams(..)` — new trailing `github_transport: Box<dyn HttpTransport>` param under `provider-github` (daemon: `DaemonHttpTransport`; default: `NoopHttpTransport`).
- `MintRequest.repo_ids: Vec<u64>` (numeric, mutually exclusive with `repos`).
- New exports: `GithubMintParams`, `NoopHttpTransport`.
- Wire: `Vault.MintGithub` RPC. CLI: `secretctl mint-github` → stdout `{"token":"…","expires_at_unix":<i64>}` (matches `flexnetos_github_app/crates/app-core/src/mint.rs` `build_argv` + `Out{token:String, expires_at_unix:u64}`).

## Tests added (exact)
mint_github.rs (engine unit): `repository_ids_emit_numeric_array_in_body`, `repositories_and_repository_ids_are_mutually_exclusive`.
lib.rs `native_mint_tests` (engine unit): `mint_github_token_happy_path_mints_and_audits_metadata_only`, `mint_github_token_refuses_when_locked`, `mint_github_token_refuses_when_key_absent_naming_remediation`, `mint_github_token_refuses_on_http_error_never_a_token`.
secretctl main.rs (CLI): `mint_github_argv_round_trips_through_clap`, `mint_github_argv_round_trips_without_optional_scopes`, `stdout_json_deserializes_into_consumer_out_shape` (the DIFFERENTIAL contract test — replicates the consumer's `build_argv` + `Out{token,expires_at_unix:u64}` verbatim and round-trips both directions; asserts compact two-field stdout + numeric `expires_at_unix`).
native_mint_e2e.rs (daemon e2e): `mint_github_returns_frozen_two_field_response`, `mint_github_rejects_non_numeric_repository_id`, `mint_github_locked_vault_fails_precondition`.

## Build/test status
- `cargo build -p envctl-secrets-engine --features provider-github -p envctl-secretd -p envctl-secretctl` — PASS. Also builds secretd WITH and WITHOUT `provider-github` (feature gating verified).
- `cargo test -p envctl-secrets-engine --features provider-github` — PASS (160 total: 118 lib + 4 + 6 + 17 + 15).
- `cargo test -p envctl-secretctl` — PASS (7).
- `cargo test -p envctl-secretd` lib(31) + native_mint_e2e(6) + e2e(5) + mitm_e2e(2) + proxy_swap_e2e(1) — PASS.
- `cargo test -p envctl-secrets-proto` — PASS.
- `cargo fmt --all --check` — PASS.
- `cargo clippy -p envctl-secrets-engine -p envctl-secretd -p envctl-secretctl -p envctl-secrets-proto --all-targets -- -D warnings` — PASS (all touched crates clean).
- `ci/gates/no-c.sh` — PASS (rustls=0.23.40 on ring=0.17.14; zero aws-lc/openssl/C-SQLite; NO NEW DEP). `shape.sh` — PASS. `enable.sh` — PASS.

## Deviations
- Added `api_base: Option<String>` to `GithubMintParams` (NOT in the plan's field list). Rationale: the plan's mandated daemon e2e + GHES support need a REST-base override, but the engine must stay env-free. The engine accepts the override as a param; the daemon fills it from `ENVCTL_GITHUB_API_BASE` — identical discipline to the existing relay-native `rebuild_github_provider`. Default `None` ⇒ real GitHub, so the frozen wire contract is unchanged.
- Added `Engine::put_github_app_id` (a small public enrollment seam) so the daemon e2e can seed the flat `github-app-id` without test-only access to the private store. It is the legitimate engine seam TASK-0026 (`secretctl github-app enroll`) will drive, not test scaffolding.
- `with_seams`' new `github_transport` param is `#[cfg(feature="provider-github")]`-gated (the plan said "3 callers"); under that feature ALL `with_seams` callers (incl. the workspace's test files) require the arg, so I added `NoopHttpTransport` to every caller. The engine's default build (feature off) keeps the original 6-arg signature.

## Handoff notes (for the invariant-guardian — targeted checks)
- FAIL-CLOSED proof points: `mint_github_token` refuses on (a) locked vault → `EngineError::Locked` (`mint_github_token_refuses_when_locked`); (b) absent `github-app-private-key` → error NAMING `secretctl github-app enroll` (`..._refuses_when_key_absent_naming_remediation`); (c) GitHub HTTP error → no token (`..._refuses_on_http_error_never_a_token` + daemon `map_mint_github_err`); (d) non-positive expiry / empty token → bail. Daemon boundary: non-numeric `repository_ids` → `invalid_argument`; locked → `failed_precondition`.
- NO-SECRET-IN-LOGS: token stays `Zeroizing` in the engine; audit/event are metadata-only (`github_token_minted` detail = installation_id + repo/perm counts + expires_at). Verified by `mint_github_token_happy_path...` (asserts the token never appears in any emitted event) and the daemon e2e (asserts the token never crosses the event-stream wire). The token materializes as a `String` ONLY at `MintGithubResp` + the secretctl stdout write.
- FROZEN CONTRACT: the differential test `stdout_json_deserializes_into_consumer_out_shape` + `mint_github_argv_round_trips_through_clap` pin parity with `flexnetos_github_app/crates/app-core/src/mint.rs` (`build_argv` argv + `Out{token:String, expires_at_unix:u64}`). stdout is compact, exactly two fields, `expires_at_unix` a JSON NUMBER. All non-JSON output in secretctl goes via clap/anyhow to stderr; the `mint_github` success path writes ONLY the one `println!`.
- repository_ids/repositories mutual exclusion (GitHub 422) is enforced in `build_token_request_body` (`repositories_and_repository_ids_are_mutually_exclusive`); the `mint-github` path sets only `repo_ids`.
- NO-C: reused `DaemonHttpTransport` (reqwest/rustls-ring/frozen webpki-roots) verbatim; added NO dependency. `no-c.sh` green, one rustls ring-only.
- PRE-EXISTING (not my change): `cargo clippy -p envctl-gui --all-targets -- -D warnings` fails with `doc_list_item_without_indentation` at `crates/gui/src/main.rs:1997` under clippy 1.96. Verified it fails IDENTICALLY on the base (714b187) with my changes stashed — toolchain drift in an untouched crate, not a TASK-0020 regression. The full-workspace `--all-targets` clippy run trips on it; my four touched crates are clean.
