# Verification report: TASK-0026 — `secretctl github-app enroll`

## Verdict
**PASS** — enroll writes EXACTLY what the mint reads; all gates + cargo checks green; every invariant verified against the code.

## Headline cross-boundary check — enroll ↔ mint name match (the anti-drift gate)
| Item | Mint READER (`Engine::mint_github_token`, lib.rs) | Enroll WRITER | Same const? |
|------|--------------------------------------------------|---------------|-------------|
| App PEM secret name | `GITHUB_APP_KEY_NAME` ("github-app-private-key"), read via `get_secret_latest`+`open_real_key` (lib.rs:1739/1746) | secretctl `AddSecretReq.name = envctl_secrets::GITHUB_APP_KEY_NAME` (main.rs `github_app`) | **YES** — same `pub const` |
| App id meta key | `GITHUB_APP_ID_META` ("github-app-id"), read via `get_meta` (lib.rs:1753) | `Vault.SetGithubAppId` → `engine.put_github_app_id` → `put_meta(GITHUB_APP_ID_META)` (lib.rs:1693) | **YES** — same `pub const` |

Both consts were flipped from private to `pub const` in `crates/secrets-engine/src/lib.rs:120/122` with **values unchanged** (verified in `git diff`) — single source of truth, literal-drift impossible. `installation_id` is correctly NOT enrolled (comes per-mint from `MintGithubReq`). **PASS.**

Round-trip e2e is genuine (NOT a tautology): `enroll_then_mint_github_round_trips` (native_mint_e2e.rs:670-697) inits+unlocks, enrolls over the WIRE via `Vault.Add{broker_only}` + `Vault.SetGithubAppId{apply:true}` (no engine seed helper), then drives the real `Vault.MintGithub` against a mock GitHub server and asserts `resp.token == ENROLL_TOKEN`. If the writer/reader names ever diverge, this fails. **PASS.**

## Gate results (worktree `/home/drdave/Desktop/meta/.worktrees/task-0026-enroll/envctl`)
| Gate | Result | Evidence (exit code) |
|------|--------|----------------------|
| `bash ci/gates/no-c.sh` | **PASS** (exit 0) | "resolved graph clean: rustls=['0.23.40'] on ring=['0.17.14']; zero aws-lc/openssl/C-SQLite" |
| `bash ci/gates/shape.sh` | **PASS** (exit 0) | "SHAPE GATE PASS" |
| `bash ci/gates/enable.sh` | **PASS** (exit 0) | "ENABLE GATE PASS" |
| `bash ci/gates/p7.sh` | **PASS** (exit 0) | "P7 GATE PASS" |

## cargo
| Check | Result | Evidence |
|-------|--------|----------|
| `cargo fmt --all --check` | **PASS** (exit 0) | clean |
| `cargo clippy --workspace -- -D warnings` (gate form) | **PASS** (exit 0) | "No issues found" |
| `cargo test -p envctl-secretctl` | **PASS** | 10 passed; 0 failed (3 NEW github-app parse tests) |
| `cargo test -p envctl-secretd --features provider-github` | **PASS** | native_mint_e2e: 11 passed; 0 failed; other secretd bins all 0 fail (~165s e2e) |
| `cargo test -p envctl-secrets-engine --features provider-github` | **PASS** | all bins 0 failed (lib 15 + others; `pub const` change → no regression) |
| default-build touched crates (`-p secretd -p secretctl -p secrets-engine -p secrets-proto`) | **PASS** (exit 0) | compiles |
| engine default-feature build (no provider-github) | **PASS** (exit 0) | compiles |

New TASK-0026 tests confirmed RUN+PASS:
- secretctl: `github_app_enroll_parses_app_id_keypath_and_apply`, `..._defaults_to_dry_run_and_accepts_stdin_dash`, `..._requires_app_id_and_private_key`.
- secretd e2e: `enroll_then_mint_github_round_trips` (load-bearing), `enrolled_pem_is_broker_only_and_reveal_is_refused`, `set_github_app_id_empty_is_invalid_argument`, `set_github_app_id_dry_run_mutates_nothing`, `set_github_app_id_locked_vault_fails_precondition`.

### Pre-existing clippy lint (NOT a blocker — verified)
`cargo clippy --workspace --all-targets` fires `doc_lazy_continuation` at `crates/gui/...:1997`. Verified GENUINELY pre-existing: `git diff HEAD~1 -- crates/gui/` AND `git diff <task-base 3c48da8> -- crates/gui/` are both EMPTY (GUI untouched). It does NOT fire under the gate command form (`--workspace` without `--all-targets`). Floating-toolchain (1.96) doc-lint drift, exactly the class flagged in verification.md. **Not a TASK-0026 regression.**

## Invariant checks
1. **No-C / one ring-only rustls** — PASS. no-c.sh authoritative (`cargo metadata` resolve.nodes) reports one rustls 0.23.40 on ring, zero banned C. secretctl's new feature enable (`provider-github`) + `zeroize` pull only pure-Rust `rsa v0.9.10` + `base64 v0.22.1` + `sha2 v0.10.9` + `zeroize v1.8.2` (`cargo tree -p envctl-secretctl --all-features` shows no sqlite/openssl/aws-lc/mimalloc). Workspace-wide `aws-lc` count = 0. (Note: `cargo tree --workspace --all-features` lists rustls 0.26/0.27 as unselected registry candidates with zero actual dependents (`cargo tree -i rustls@0.26.4` empty) — they are NOT in the resolved graph the gate parses.)
2. **Fail-closed + dry-run-by-default** — PASS. Empty app_id → `invalid_argument` BEFORE any write (grpc.rs handler; test `set_github_app_id_empty_is_invalid_argument`). `apply=false` → DRY-RUN Log, mutates nothing (handler; test `set_github_app_id_dry_run_mutates_nothing` PROVES via a downstream mint that fails closed). Locked vault + `apply=true` → `put_github_app_id` returns `Err(Locked)` → `map_set_app_id_err` → `failed_precondition` (test `set_github_app_id_locked_vault_fails_precondition`). Non-PEM `--private-key` → secretctl `build_app_jwt` validation bails before any RPC (main.rs `github_app` step 2). secretctl dry-run default writes nothing (no `--apply`).
3. **No secret in logs/audit** — PASS. `secret_put` audit row is name + `{version}` only (lib.rs:725-730); PEM body is `Zeroizing`, dropped at scope end. grpc `set_github_app_id` logs only app_id (non-secret) + meta-key name. secretctl holds PEM in `Zeroizing` (`read_pem`) until it crosses the UDS in `AddSecretReq.value`; dry-run preview echoes only metadata to STDERR. `enrolled_pem_is_broker_only_and_reveal_is_refused` asserts PEM bytes never appear in a refused-reveal error.
4. **Broker-only sealing** — PASS. Enroll seals `AddSecretReq.broker_only = true` (main.rs). Post-enroll `Vault.Get{reveal,apply,confirm}` on the PEM ⇒ `permission_denied`, PEM bytes absent from the error (test `enrolled_pem_is_broker_only_and_reveal_is_refused`, native_mint_e2e.rs:716-735). The mint reads it only via the internal `open_real_key` path (proven by the round-trip).
5. **Engine purity** — PASS. Engine diff is EXACTLY the 2 `pub const` exports (value-preserving) + doc comments; no new `println!`/`eprint!`/`print!`/`stdout` in the engine. No new Engine method (the pre-existing `put_github_app_id` seam is now wired). secretd + secretctl are thin; client-side PEM validation reuses `build_app_jwt` (no new engine logic).
6. **Default (no-feature) build of touched crates** — PASS. secretd/secrets-engine default build compiles; secretctl hard-enables engine `provider-github` (intentional — it needs `build_app_jwt`/consts) and compiles.

## Parity check
This is the secrets-daemon stack, NOT the env-manager engine. `service Vault` has a single client (`secretctl`); the env-manager GUI does not drive the vault daemon → no GUI parity surface required (consistent with TASK-0020's `mint-github`, same stack). New proto RPC `Vault.SetGithubAppId`:
- secretd handler: `crates/secretd/src/grpc.rs::set_github_app_id` (feature-gated + a `#[cfg(not)]` Unimplemented arm).
- secretctl caller: `crates/secretctl/src/main.rs::github_app` → `c.set_github_app_id(...)` + `c.add(...)`.
CLI-only surface justified. **PASS.**

## Findings
None blocking. One pre-existing, out-of-scope note: the workspace `--all-targets` clippy `doc_lazy_continuation` lint at `crates/gui/...:1997` (floating-toolchain drift, GUI untouched) — a candidate for a separate cleanup task, not this cycle.

## Re-test needed
None. If any fix lands, re-run: `bash ci/gates/no-c.sh`, `cargo clippy --workspace -- -D warnings`, `cargo test -p envctl-secretd --features provider-github` (the round-trip is the gate).

VERDICT: **PASS**
