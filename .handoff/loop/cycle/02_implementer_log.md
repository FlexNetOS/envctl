# Implementation log: TASK-0028 — GUI parity (mint-github / relay-mint / revoke) · STATUS: GREEN

Architecture **B** (decided, not deviated): `envctl-gui` drives the installed `secretctl`
binary as a subprocess via an engine-owned seam. ZERO new GUI crate deps. The GUI builds an
argv `Vec<String>` (the identical secretctl clap surface), the engine spawns/captures, the GUI
renders metadata only.

## Changes
- `crates/engine/src/secrets.rs` (NEW): engine-owned subprocess seam. `resolve_secretctl()`
  (current-exe dir → `$HOME/.cargo/bin/secretctl` → PATH, fail-closed `None`) +
  `run_secretctl(verb, argv, stdin, sink)` which spawns `secretctl`, writes the optional
  `Zeroizing` stdin buffer to the child pipe, captures stdout/stderr/exit, emits one
  `Event::SecretsResult`, and drops the buffer. Holds no token after the child exits; never panics.
  Inline `#[cfg(test)]` proves the not-found path emits a fail-closed result (not a panic).
- `crates/engine/src/lib.rs`: `pub mod secrets;` + `pub use zeroize::Zeroizing;` (re-export so the
  GUI builds the secret stdin buffer WITHOUT a direct `zeroize` dep — keeps the GUI dep set frozen).
- `crates/engine/src/event.rs`: NEW `Event::SecretsResult { verb, json_stdout, stderr, code }`.
- `crates/engine/src/command.rs`: NEW `EngineCommand::Secrets { verb, argv, stdin: Option<Zeroizing<Vec<u8>>> }`
  + a `run_event_loop` arm delegating to `secrets::run_secretctl`.
- `crates/engine/Cargo.toml`: added `zeroize = { workspace = true }` (already a workspace dep;
  pure-Rust, links no C — no-c graph unaffected).
- `crates/gui/src/main.rs`: NEW `Screen::Secrets` (+ label, nav entry, central-dispatch arm);
  `enum SecretsVerbTab { MintGithub, RelayMint, Revoke }`; `struct RelayMintMeta {token_id,expires_at,native}`
  (NO bearer field); new state fields on `EnvctlApp` (+ both constructors); pure argv builders
  `mint_github_argv` / `relay_mint_argv` / `revoke_argv`; `secrets_form_ready` validation gate;
  `build_secrets_command` (moves revoke token into `Zeroizing`, clears the field); `secrets_screen`
  + per-verb forms + `secrets_action_button` + metadata-only `secrets_results`; the
  `Event::SecretsResult` drain arm → `handle_secrets_result` (metadata-only parse + fail-closed);
  three tiny pure-Rust JSON field scanners (`json_string_field`/`json_number_field`/`json_bool_field`)
  so the GUI takes NO `serde_json` dep.
- `crates/gui/Cargo.toml`: **UNCHANGED** (zero new deps — verified).

## Engine API (the parity contract)
- `EngineCommand::Secrets { verb: String, argv: Vec<String>, stdin: Option<Zeroizing<Vec<u8>>> }`
- `Event::SecretsResult { verb: String, json_stdout: String, stderr: String, code: Option<i32> }`
- `envctl_engine::Zeroizing` (re-export of `zeroize::Zeroizing`)
- `secrets::run_secretctl(verb, argv, stdin, sink)` (sync, non-printing; emits the Event)

## Parity mapping (GUI control → secretctl argv → daemon field rendered)
| GUI verb | argv built (verbatim secretctl surface) | rendered (metadata only) |
|---|---|---|
| Mint GitHub | `mint-github --installation-id <u64> --ttl-secs <i64> --output json [--repository-ids csv] [--permissions csv]` | `expires_at_unix` + transient copy-once token (dropped after one copy; token never persisted/logged) |
| Relay mint | `relay mint <name> [--ttl] [--mode] [--provider] (--repo)* (--perm)* --json` | `{token_id, expires_at, native}` — `bearer` dropped/never rendered |
| Revoke | `github-app revoke-token --token - [--installation-id] [--apply] --json` | `{revoked, dry_run}`; dry-run default |

The GUI does NOT inject the native `checks:write` default (left to secretctl, test-enforced).

## Secret-handling enforcement
- **No serde_json/zeroize dep added to the GUI**; eframe persistence remains OFF (no `save()` added,
  no `#[derive(Serialize)]` on `EnvctlApp`/secret fields).
- **Mint token**: parsed once; only `expires_at_unix:i64` + `sec_mint_has_token:bool` persist. Token
  held transiently in `sec_mint_copy_once: Option<String>`, copied via `ui.output().copied_text` then
  `take()`-dropped. Mint stdout NEVER flows through `push_log`/`self.log` (Test 7 asserts).
- **Relay bearer**: never extracted from stdout (the scanner only reads `token_id`/`expires_at`/`native`);
  `RelayMintMeta` has no bearer field (test `handle_relay_result_drops_bearer`).
- **Revoke token**: input is `password(true)`, moved into `Zeroizing<Vec<u8>>` on dispatch and piped via
  stdin (`--token -`); `sec_revoke_token` cleared; the literal token never appears in argv (Tests 4, 6).
- **Fail-closed**: non-zero exit / `code=None` ⇒ DANGER status from stderr only; stdout not parsed, no
  success card, no synthesized success (Test 8 + `handle_nonzero_exit_surfaces_stderr_not_stdout`).
  Revoke `--apply` omitted by default (Tests 4, 5).

## Tests added (all pass; verbatim replication — no envctl-secretctl dev-dep)
- `mint_github_argv_round_trips_through_replica` (Test 1, anti-divergence via `MintGithubArgsReplica`)
- `mint_github_argv_omits_blank_optional_scopes` (Test 2)
- `relay_mint_argv_maps_mode_provider_repos_perms` + `relay_mint_argv_omits_blank_optionals` (Test 3)
- `revoke_argv_defaults_dry_run_uses_stdin_token` (Test 4)
- `revoke_apply_toggle_defaults_false` (Test 5)
- `revoke_dispatch_moves_token_to_stdin_and_clears_field` + `mint_and_relay_dispatch_have_no_stdin` (Test 6)
- `handle_revoke_result_keeps_only_metadata` / `handle_relay_result_drops_bearer` /
  `handle_mint_result_holds_token_transiently_not_logged` (Test 7)
- `handle_failure_renders_danger_no_success_card` / `handle_nonzero_exit_surfaces_stderr_not_stdout` (Test 8)
- `json_scanners_extract_named_fields` (scanner unit test)
- engine: `secrets::tests::missing_binary_emits_failclosed_result_not_panic`

## Build/test status (all via `rtk proxy`, raw passthrough)
- `cargo build -p envctl-engine -p envctl-gui` → exit=0
- `cargo build -p envctl` (CLI, engine API delta) → exit=0
- `cargo test -p envctl-gui` → 25 passed, 0 failed (exit=0)
- `cargo test -p envctl-engine --lib` → 60 passed, 0 failed (exit=0)
- `cargo fmt --all -- --check` → exit=0
- `cargo clippy -p envctl-engine -p envctl-gui -- -D warnings` → exit=0
- `cargo clippy -p envctl-engine -p envctl-gui --all-targets -- -D warnings` → exit=0 (test code clean)
- `bash ci/gates/no-c.sh` → NO-C GATE PASS (rustls=0.23.40 on ring; zero aws-lc/openssl/C-SQLite)
- `bash ci/gates/shape.sh` → SHAPE GATE PASS

## Deviations
None. (Architecture B followed exactly; no rebase on PR #124 — `revoke_argv` is pure strings and the
parity test uses verbatim replication, so the GUI builds/tests off develop with no secretctl import.)

## Handoff notes (for the invariant-guardian)
- **Engine-single-authority / no-divergence**: the GUI has no mint/revoke logic — it only builds argv +
  parses metadata. The argv-parity test (`mint_github_argv_round_trips_through_replica`) is the proof.
  Verify the replica matches `secretctl/src/cli.rs` `MintGithubArgs` (it does, copied verbatim).
- **No serde_json in the GUI**: the JSON field scanners (`gui/src/main.rs`, near `split_csv`/`opt_str`)
  are deliberately minimal (flat compact objects only — the daemon's `--json` shapes). Verify
  `json_string_field` un-escaping and that the relay scanner never reads `bearer`.
- **Zeroize re-export, not a new GUI dep**: `crates/gui/Cargo.toml` is unchanged; confirm `Cargo.lock`'s
  +1 line is the engine's zeroize (already in the workspace), and `no-c.sh` PASSes (it does).
- **Secret-hygiene paths to check** (all in `handle_secrets_result` / `build_secrets_command` /
  `secrets_results`): (1) mint stdout NEVER through push_log (Test 7); (2) revoke token via stdin only +
  field cleared (Test 6); (3) non-zero exit renders stderr-only DANGER, stdout ignored (Test 8 +
  `handle_nonzero_exit_surfaces_stderr_not_stdout`).
- **Fail-closed not-found**: `secrets::run_secretctl` emits a `SecretsResult{code:None}` when secretctl
  is unresolved (covered by the engine unit test) — the GUI renders that as the explanatory DANGER state.
- **Runtime coupling to PR #124 only**: an installed secretctl lacking `github-app revoke-token` errors
  gracefully (non-zero exit → DANGER card); the GUI compiles/tests without #124.

STATUS: GREEN
