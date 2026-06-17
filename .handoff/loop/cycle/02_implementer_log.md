# Implementation log: TASK-0026 — `secretctl github-app enroll`

Seal the GitHub App credential into the unlocked vault so TASK-0020's per-call
`Engine::mint_github_token` can read it. Enroll writes EXACTLY what the mint reads (byte-for-byte
names): broker-only secret `github-app-private-key` (PEM) + non-secret meta `github-app-id`.
installation-id is NOT enrolled (it is supplied per-mint).

## Changes
- `crates/secrets-proto/proto/control.proto`: added `rpc SetGithubAppId (SetGithubAppIdReq) returns (stream Event)` to `service Vault` + `message SetGithubAppIdReq { string app_id = 1; bool apply = 2; }` (apply=false => dry-run, CF-8).
- `crates/secrets-engine/src/lib.rs`: exported `GITHUB_APP_KEY_NAME` / `GITHUB_APP_ID_META` as `pub const` (were private) so secretctl references them verbatim — kills literal-drift between the enroll writer and the mint reader. Engine logic otherwise UNTOUCHED.
- `crates/secretd/src/grpc.rs`: added `type SetGithubAppIdStream = EventStream;` + the `set_github_app_id` handler (next to `add`) + the `map_set_app_id_err` classifier. Feature-gated on `provider-github` (Unimplemented without it, mirroring `mint_github`).
- `crates/secretctl/Cargo.toml`: enabled `envctl-secrets-engine` feature `provider-github` (for `build_app_jwt` + `MAX_JWT_TTL_SECS` + the consts; pure-Rust rsa+base64, no new C) and added `zeroize` (PEM stays `Zeroizing`). No NEW workspace dependency.
- `crates/secretctl/src/cli.rs`: added `Cmd::GithubApp { cmd: GithubAppCmd }` + `GithubAppCmd::Enroll { --app-id, --private-key, --apply }`.
- `crates/secretctl/src/main.rs`: dispatch arm + `github_app` fn + `read_pem` helper. + 3 cli-parse tests.
- `crates/secretd/tests/native_mint_e2e.rs`: 5 new e2e tests (round-trip + broker-only refusal + 3 negatives) reusing the mock-GitHub harness.

## Engine API (the parity contract)
- No new Engine method. The engine seam `Engine::put_github_app_id(&self, app_id: &str) -> anyhow::Result<()>` (already present, returns `Err(EngineError::Locked)` if locked) is now wired to the new RPC. This is a secrets-stack (daemon) feature — there is no GUI parity surface (the env-manager GUI does not drive the vault daemon); CLI = `secretctl`, the sole client of `service Vault`.
- New proto contract: `Vault.SetGithubAppId(SetGithubAppIdReq{app_id, apply}) -> stream Event`.
- secretd handler signature: `async fn set_github_app_id(&self, Request<v1::SetGithubAppIdReq>) -> Result<Response<Self::SetGithubAppIdStream>, Status>`.
- secretctl: `async fn github_app(cmd: GithubAppCmd, sock: PathBuf, json: bool) -> anyhow::Result<()>`; `fn read_pem(source: &str) -> anyhow::Result<zeroize::Zeroizing<Vec<u8>>>`.

## Tests added (exact counts)
secretctl `src/main.rs` unit tests (10 total in the file; 3 NEW):
- `github_app_enroll_parses_app_id_keypath_and_apply` — proves `--app-id/--private-key/--apply` parse.
- `github_app_enroll_defaults_to_dry_run_and_accepts_stdin_dash` — `--private-key -` selects stdin; apply defaults false.
- `github_app_enroll_requires_app_id_and_private_key` — both flags required (clap error otherwise).

secretd `tests/native_mint_e2e.rs` (11 total in the file; 5 NEW; all over the REAL `serve` wire):
- `enroll_then_mint_github_round_trips` — **LOAD-BEARING**: init+unlock → enroll over the wire (Add broker-only PEM + SetGithubAppId "4044997") → `Vault.MintGithub` against the mock SUCCEEDS reading exactly what enroll wrote. Proves no name-drift writer↔reader.
- `enrolled_pem_is_broker_only_and_reveal_is_refused` — after enroll, `Vault.Get{reveal,apply,confirm}` on the PEM ⇒ `permission_denied`, and the PEM bytes never appear in the error.
- `set_github_app_id_empty_is_invalid_argument` — whitespace-only app_id ⇒ `invalid_argument`, nothing written.
- `set_github_app_id_dry_run_mutates_nothing` — `apply=false` emits a DRY-RUN Log; a later mint fails closed (no App id enrolled) proving no write.
- `set_github_app_id_locked_vault_fails_precondition` — `apply=true` on a locked vault ⇒ `failed_precondition`, nothing written.

## Build/test status
- `cargo build -p envctl-secrets-proto / -p envctl-secretd / -p envctl-secretctl` — PASS.
- `cargo test -p envctl-secretctl` — PASS (10 passed, 0 failed).
- `cargo test -p envctl-secretd --test native_mint_e2e` — PASS (11 passed, 0 failed; ~170s — the env-var tests are serialized by the existing `SERIAL` tokio mutex).
- `cargo test -p envctl-secrets-engine --lib` — PASS (96 passed, 0 failed; `pub const` change caused no regression).
- `cargo clippy -p envctl-secrets-proto -p envctl-secrets-engine -p envctl-secretd -p envctl-secretctl --all-targets -- -D warnings` — PASS (clean).
- `cargo fmt --all --check` — PASS.
- `bash ci/gates/no-c.sh` — PASS ("rustls=['0.23.40'] on ring=['0.17.14']; zero aws-lc/openssl/C-SQLite"). Enabling `provider-github` in secretctl adds only pure-Rust rsa+base64.
- `bash ci/gates/shape.sh` — PASS.

### Residual issue (PRE-EXISTING, NOT mine)
- `cargo clippy --workspace --all-targets -- -D warnings` FAILS on `crates/gui/...:1997` with `clippy::doc_lazy_continuation`. This is in `envctl-gui` test code, which is NOT in my diff (`git diff --name-only` shows only proto/engine/secretd/secretctl). It is floating-toolchain drift (rust-1.96.0 doc-lint strictness) pre-existing on the branch base, exactly the class flagged in `rust-feature-impl/references/verification.md`. All four TASK-0026 crates pass `--all-targets -D warnings`.

## Deviations
1. **secretctl enables the engine `provider-github` feature.** The plan calls `build_app_jwt` + `MAX_JWT_TTL_SECS` for client-side PEM validation and references `GITHUB_APP_KEY_NAME`; all three are `#[cfg(feature = "provider-github")]` engine exports, so secretctl must enable that feature. It pulls only pure-Rust `rsa` + `base64` (already in the trust boundary via the engine) — no-c gate green. Not a design change; the plan implied it.
2. **Locked → `failed_precondition` for the apply path is done OUTSIDE `run_streaming`.** The shared `run_streaming` maps every engine error to `Status::internal`; changing it would affect every streaming RPC. So the apply path runs `put_github_app_id` on `spawn_blocking` directly and classifies via `map_set_app_id_err` (mirrors the unary `map_mint_github_err`), then ships a one-item success stream. The dry-run path uses `run_streaming` (no engine mutation). Net behavior matches the plan exactly (Locked ⇒ failed_precondition; empty ⇒ invalid_argument; dry-run ⇒ preview, nothing written).
3. **SHA-256 fingerprint omitted** — per plan ("Skip the SHA-256 fingerprint if it would add a new secretctl dependency"); blake3/sha2 are not secretctl deps, so it was dropped to keep the dep surface minimal.

## Handoff notes (for the invariant-guardian)
- **No new dependency in the trust boundary**: secretctl now enables engine `provider-github` (rsa+base64, pure-Rust) and adds `zeroize` (already a workspace pin). `ci/gates/no-c.sh` is green — verify Gate 2 (`envctl-secretctl --all-features`) still finds no aws-lc/openssl/C-SQLite.
- **Fail-closed, dry-run by default** — three refusal paths are unit/e2e tested: (a) `set_github_app_id_empty_is_invalid_argument`, (b) `set_github_app_id_dry_run_mutates_nothing` (the proof is a downstream mint that fails closed), (c) `set_github_app_id_locked_vault_fails_precondition`. The secretctl `github_app` fn also validates the PEM (via `build_app_jwt`) BEFORE any RPC, so a non-PEM `--private-key` writes nothing (validated by the round-trip's reliance on a real key + the engine's parse).
- **PEM never printed/logged**: secretctl holds the PEM in `Zeroizing` from `read_pem` until it crosses the peercred-gated UDS in `AddSecretReq.value`; the dry-run preview prints only metadata to STDERR. The `enrolled_pem_is_broker_only_and_reveal_is_refused` test asserts the PEM bytes never appear in a refused-reveal error.
- **Broker-only PEM**: enroll seals with `broker_only=true`, so `secret get --reveal` is REFUSED (test covers it); the mint reads it via the internal `open_real_key` path only — proven by the round-trip.
- **The round-trip is the anti-drift gate**: it enrolls over the WIRE (not the engine seed helper) and then a real `Vault.MintGithub` succeeds — so if the enroll-writer and mint-reader names ever diverge, that test fails. The `pub const` export ensures both sides use one literal.
- Engine stays the single non-printing library; secretd + secretctl are thin. No GUI parity is required (this is the secrets-daemon stack, not the env-manager engine).

STATUS: GREEN
