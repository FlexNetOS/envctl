# Verification report: TASK-0020-COMPLETE — FROZEN `mint-github` consumer-contract surface

## Verdict — PASS-WITH-NOTES

All non-negotiable invariants, all 4 CI gates, fmt, the gate-form clippy, and every test suite
(engine/secretctl/secretd, feature-on + default-off builds) PASS. The single NOTE is a confirmed
PRE-EXISTING GUI lint that does not fire under the gate command and is in a file TASK-0020 never
touched. No blocking findings.

Verified in place at `/home/drdave/Desktop/meta/.worktrees/g2-mintgh/envctl` (branch
`g2-mint-github`). TASK-0020 changes are present as an UNCOMMITTED working tree on top of base
`714b187` (origin/g2-native-mint); all checks below ran against that working tree.

## Frozen-contract verdict — PASS (the headline check)
Read both sides: producer `crates/secretctl/src/{cli.rs,main.rs}` + `crates/secrets-proto/proto/control.proto`
vs. consumer `flexnetos_github_app/crates/app-core/src/mint.rs`.

- **Flags match `build_argv` exactly.** Producer `MintGithubArgs` (cli.rs:75-93) declares
  `--installation-id` (u64, required), `--ttl-secs` (i64, required), `--output` (String, required),
  `--repository-ids` (comma `value_delimiter`), `--permissions` (comma `value_delimiter`, verbatim).
  Consumer `build_argv` (mint.rs:149-187) emits precisely `mint-github --installation-id N --ttl-secs T
  --output json [--repository-ids a,b] [--permissions name:access,...]`. The differential test
  `mint_github_argv_round_trips_through_clap` (main.rs:874) parses the consumer's OWN `build_argv`
  output (replicated verbatim, main.rs:799-837) through clap 1:1; `mint_github_argv_round_trips_without_optional_scopes`
  (main.rs:909) covers the no-scopes path. Both PASS.
- **stdout is ONLY `{"token":"…","expires_at_unix":<i64>}`, field names byte-for-byte, number not string.**
  `mint_github` (main.rs:200-224) builds the JSON `Value` EXPLICITLY (not from the proto struct) and
  writes a single compact `println!` to stdout; `--output` other than `json` bails to stderr; every
  other diagnostic goes via clap/anyhow to stderr. Field names `token`/`expires_at_unix` equal the
  consumer's `Out{token:String, expires_at_unix:u64}` (mint.rs:131-143).
- **The differential test is NOT a tautology.** `stdout_json_deserializes_into_consumer_out_shape`
  (main.rs:924) deserializes our stdout via a hand-rolled `ConsumerOut` that REPLICATES the consumer's
  `serde_json::from_slice` + typed extraction (main.rs:844-871) — `expires_at_unix` is read with
  `as_u64()`, which REJECTS a JSON string, exactly as the consumer's `u64` field would; it also asserts
  compact (no `\n`, no `  `), exactly two keys, and `is_number()`. It tests the consumer's shape, not
  the producer's struct. PASS.
- **Wire shape matches.** proto `MintGithubReq{uint64 installation_id=1; repeated string repository_ids=2;
  repeated string permissions=3; int64 ttl_secs=4;}` / `MintGithubResp{string token=1; int64 expires_at_unix=2;}`
  (control.proto) matches the plan's frozen RPC contract.

## Gate results
- `ci/gates/no-c.sh` — **PASS** (exit 0). `resolved graph clean: rustls=['0.23.40'] on ring=['0.17.14'];
  zero aws-lc/openssl/C-SQLite`. No new dependency added (provider-github deps are pure-Rust `rsa`+`base64`).
- `ci/gates/shape.sh` — **PASS** (exit 0). `SHAPE GATE PASS`.
- `ci/gates/enable.sh` — **PASS** (exit 0). `ENABLE GATE PASS`.
- `ci/gates/p7.sh` — **PASS** (exit 0). `P7 GATE PASS`.

## cargo
- `cargo fmt --all --check` — **PASS** (exit 0).
- `cargo clippy --workspace -- -D warnings` (the CLAUDE.md gate form) — **PASS** (exit 0).
- `cargo test -p envctl-secrets-engine --features provider-github` — **PASS** (exit 0):
  118 lib + 4 + 6 + 17 + 15 = 160, 0 failed. Includes all 4 fail-closed unit tests + the 2
  body-shaping tests.
- `cargo test -p envctl-secretctl` — **PASS** (exit 0): 7, 0 failed. Includes all 3 contract tests
  (the differential `stdout_json_deserializes_into_consumer_out_shape` + 2 argv round-trips).
- `cargo test -p envctl-secretd --features provider-github` — **PASS** (exit 0): lib 31 + native_mint_e2e 6
  + e2e 5 + mitm_e2e 2 + proxy_swap_e2e 1, 0 failed. native_mint_e2e includes the 3 daemon-wire
  contract tests: `mint_github_returns_frozen_two_field_response`, `mint_github_rejects_non_numeric_repository_id`,
  `mint_github_locked_vault_fails_precondition`.
- `cargo build -p envctl-secretd -p envctl-secrets-engine` (default features) — **PASS** (exit 0):
  engine default = provider-github OFF ⇒ exercises the 6-arg `with_seams` path. Compiles clean.
- `cargo build -p envctl-secretd --no-default-features` (provider-github OFF) — **PASS** (exit 0):
  the `#[cfg(not(provider-github))]` `Unimplemented` stub satisfies the trait feature-off.

## Invariant checks
1. **No-C** — PASS. `no-c.sh` green; one rustls on ring; zero aws-lc/openssl/C-SQLite. provider-github
   pulls only `rsa`+`base64` (RustCrypto, pure-Rust; Cargo.toml secrets-engine:16,48). `api_base`/
   `ENVCTL_GITHUB_API_BASE` add no dep (grpc.rs:286, mint_github.rs:117). NO NEW DEP.
2. **Fail-closed** — PASS. `Engine::mint_github_token` (lib.rs:1715): locked vault → `EngineError::Locked`
   (lib.rs:1729); absent `github-app-private-key` → bail NAMING `secretctl github-app enroll` (lib.rs:1738);
   absent app-id → same remediation (lib.rs:1750); transport/HTTP error → `MintError` propagated, no token
   (mint_github.rs:242-258); non-positive expiry → bail (lib.rs:1782); empty token → bail (lib.rs:1786).
   Daemon boundary (grpc.rs): non-numeric repository_id → `invalid_argument` (grpc.rs:273); negative ttl →
   `invalid_argument` (grpc.rs:280); `map_mint_github_err` maps locked→`failed_precondition`,
   transport/`github returned`→`unavailable`, else→`permission_denied` (grpc.rs:330-344). Each refusal
   path is unit/e2e-tested (engine `native_mint_tests` ×4; daemon e2e ×3).
3. **No secret in logs/audit/wire** — PASS. Token stays `Zeroizing<Vec<u8>>` in the engine; audit row +
   `RelayMinted` event carry metadata only (installation_id, repository_id_count, permission_count,
   expires_at — lib.rs:1795-1810). Grep: zero token/PEM logging in grpc.rs (only a doc comment); token
   materialized to `String` exactly once at `MintGithubResp` (grpc.rs:313). `repository_ids` and name-based
   `repositories` are mutually-exclusive-rejected in `build_token_request_body` (mint_github.rs:277-281,
   test `repositories_and_repository_ids_are_mutually_exclusive`); the mint-github path sends ONLY
   `repo_ids` (lib.rs:1771) ⇒ no GitHub 422. Test `mint_github_token_happy_path_mints_and_audits_metadata_only`
   asserts the token never appears in any emitted event.
4. **Engine purity** — PASS. Logic lives in `mint_github_token` in the engine; grep finds zero non-test
   `println!`/`eprintln!`/`print!`/`dbg!` in `lib.rs` or `mint_github.rs`. secretd is handler+seam only;
   secretctl `mint_github` writes exactly one stdout `println!` (main.rs:222). The engine default
   `github_transport` is the fail-closed `NoopHttpTransport` (lib.rs:244, gated; mint_github.rs:92-101)
   which errors — non-daemon builds cannot reach the wire.
5. **expires_at_unix** — PASS. GitHub RFC3339 → i64 epoch via `.timestamp()` (mint_github.rs:321-323);
   surfaced as proto `int64`/JSON number (control.proto, main.rs:217-220). Defensive non-positive check
   in both engine (lib.rs:1782) and daemon (grpc.rs:308). The differential test asserts JSON-number form.
6. **`with_seams` signature change** — PASS. New trailing `#[cfg(feature="provider-github")]`
   `github_transport: Box<dyn HttpTransport>` (lib.rs:254-261). All callers updated: `open_with_store`
   (lib.rs:244, gated), secretd `engine_with_daemon_seams` (main.rs:285 → `DaemonHttpTransport::from_handle(rt)`),
   and every test-file caller (engine tests + secretd tests + proxy.rs) append the gated `NoopHttpTransport`.
   Feature-ON build, default (feature-OFF engine) build, and `--no-default-features` secretd build all compile.

## Parity check (Engine method → callers)
- `Engine::mint_github_token` (lib.rs:1715) → daemon `Vault::mint_github` handler (grpc.rs:299) →
  CLI `secretctl mint-github` over UDS (main.rs:213). This is a daemon/CLI machine surface (the frozen
  consumer contract), justifiably NOT a GUI surface — the plan scoped it as the headless contract
  `flexnetos_github_app` shells, and there is a parallel human-facing `relay mint --mode native --provider
  github` path (cli.rs:193-212) for interactive use. CLI-only surface is plan-justified.
- `Engine::put_github_app_id` (lib.rs:1682) → the TASK-0026 `secretctl github-app enroll` enrollment seam
  (exercised by the daemon e2e to seed `github-app-id`); a deviation the implementer disclosed and justified.

## Findings
- **NOTE (not blocking) — pre-existing GUI clippy lint under `--all-targets`.** `cargo clippy -p envctl-gui
  --all-targets -- -D warnings` fails with `doc list item without indentation` at `crates/gui/src/main.rs:1997`
  (clippy 1.96). VERIFIED pre-existing & out-of-scope: `git diff 714b187 -- crates/gui/` is EMPTY (the GUI
  crate is entirely untouched by TASK-0020), and the lint does NOT fire under the CLAUDE.md gate command
  `cargo clippy --workspace -- -D warnings` (which passed clean). It is toolchain drift in an untouched file,
  not a TASK-0020 regression. Severity: note. Owner: pre-existing (separate cleanup, not this task).
- **Deviation (accepted) — `GithubMintParams.api_base: Option<String>` not in the plan's field list.**
  Justified: the engine stays env-free; the daemon fills it from `ENVCTL_GITHUB_API_BASE` (grpc.rs:286),
  mirroring the existing `rebuild_github_provider` discipline. Default `None` ⇒ real GitHub ⇒ frozen wire
  contract unchanged. No invariant impact.
- **Deviation (accepted) — `Engine::put_github_app_id` added.** A legitimate Unlocked-gated enrollment seam
  for TASK-0026, used by the daemon e2e to seed app-id without private-store test access. No invariant impact.

## Re-test needed
None — verdict is PASS-WITH-NOTES with no blocking findings. If the working tree is committed/rebased,
re-run the fast gate set to confirm no regression:
```
cd /home/drdave/Desktop/meta/.worktrees/g2-mintgh/envctl
bash ci/gates/no-c.sh && bash ci/gates/shape.sh && bash ci/gates/enable.sh && bash ci/gates/p7.sh
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
cargo test -p envctl-secrets-engine --features provider-github
cargo test -p envctl-secretctl
cargo test -p envctl-secretd --features provider-github
cargo build -p envctl-secrets-engine          # default: provider-github OFF
cargo build -p envctl-secretd --no-default-features
```
The pre-existing GUI `--all-targets` lint is a separate, out-of-scope cleanup; it does not gate this task.
