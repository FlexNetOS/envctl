# Verification report: TASK-0028 — GUI parity (mint-github / relay-mint / revoke)

## Verdict — **PASS-WITH-NOTES**

Independent cross-boundary verification of the TASK-0028 changeset (the **uncommitted working
tree** vs `HEAD = 003e19c` #116; `git log origin/develop..HEAD` is empty — the work is the
working tree). Architecture **B** followed exactly: the GUI builds an argv `Vec<String>` (the
identical `secretctl` clap surface) and a new engine `EngineCommand::Secrets` spawns the
subprocess + emits `Event::SecretsResult`. Every NON-NEGOTIABLE invariant holds; all real gates +
both clippy axes + tests are green from raw `rtk proxy` passthrough (verified exit codes, not the
implementer's word). The notes are forward-looking (runtime coupling to PR #124) and a tiny
defense-in-depth observation — none block.

### Changeset scope (working tree vs HEAD 003e19c)
NEW `crates/engine/src/secrets.rs`; modified `crates/engine/src/{lib,command,event}.rs`,
`crates/engine/Cargo.toml` (+`zeroize`), `crates/gui/src/main.rs` (+1009 lines), `Cargo.lock`
(+1 line: engine `zeroize`). `crates/gui/Cargo.toml` **UNCHANGED** (verified — zero new GUI deps).

## Gate results — exit codes captured
| Gate | Command | Exit | Result |
|------|---------|------|--------|
| no-c | `bash ci/gates/no-c.sh` | **0** | PASS — `rustls=['0.23.40'] on ring=['0.17.14']; zero aws-lc/openssl/C-SQLite` |
| shape | `bash ci/gates/shape.sh` | **0** | PASS — `SHAPE GATE PASS` |
| enable | `bash ci/gates/enable.sh` | **0** | PASS — `ENABLE GATE PASS` |

## cargo — exit codes captured
| Check | Command | Exit | Result |
|-------|---------|------|--------|
| fmt | `rtk proxy cargo fmt --all -- --check` | **0** | PASS |
| clippy (gate axis) | `rtk proxy cargo clippy -p envctl-engine -p envctl-gui -- -D warnings` | **0** | PASS |
| clippy (--all-targets) | `rtk proxy cargo clippy -p envctl-engine -p envctl-gui --all-targets -- -D warnings` | **0** | PASS — test code clean |
| test | `rtk proxy cargo test -p envctl-engine -p envctl-gui` | **0** | PASS — engine lib 60, gui **25**, engine integ 24+12+15+20; **0 failed** |
| build CLI | `rtk proxy cargo build -p envctl-engine -p envctl` | **0** | PASS — CLI/engine still build (engine API delta non-breaking) |

GUI test binary executed `running 25 tests` (NOT "0 tests"). All plan tests present and passed
(`--list` confirmed): `mint_github_argv_round_trips_through_replica`,
`mint_github_argv_omits_blank_optional_scopes`, `relay_mint_argv_maps_mode_provider_repos_perms`,
`relay_mint_argv_omits_blank_optionals`, `revoke_argv_defaults_dry_run_uses_stdin_token`,
`revoke_apply_toggle_defaults_false`, `revoke_dispatch_moves_token_to_stdin_and_clears_field`,
`mint_and_relay_dispatch_have_no_stdin`, `handle_revoke_result_keeps_only_metadata`,
`handle_relay_result_drops_bearer`, `handle_mint_result_holds_token_transiently_not_logged`,
`handle_failure_renders_danger_no_success_card`, `handle_nonzero_exit_surfaces_stderr_not_stdout`,
`json_scanners_extract_named_fields`; engine `secrets::tests::missing_binary_emits_failclosed_result_not_panic`.

## Invariant checks
1. **Engine single sync non-printing authority; CLI↔GUI cannot diverge — PASS.** `secrets.rs` is
   sync, `std`+`zeroize`+`which`-only; grep for `println!/eprint!/print!/stdout`-write in the
   library path: NONE (the only `stderr` hits are struct-field names + test asserts). New logic
   lives in the engine (`secrets.rs` / `command.rs:269` arm), NOT in `main.rs`. The GUI holds
   ZERO mint/revoke logic — it only builds argv strings + parses metadata. Argv-parity proof:
   `mint_github_argv_round_trips_through_replica` (gui:3080) parses the GUI argv through
   `MintGithubArgsReplica` (gui:3038-3076), which I independently compared field-for-field against
   the REAL `secretctl/src/cli.rs::MintGithubArgs` (`cli.rs:101-120`): same `--installation-id`
   (u64), `--ttl-secs` (i64), `--output` (fixed "json"), `--repository-ids`/`--permissions`
   (comma-delimited `Vec` via `value_delimiter=','`). Faithful.
2. **No secret bytes rendered/persisted by the GUI — PASS.** (a) `impl eframe::App for EnvctlApp`
   (gui:683) has NO `fn save` override; `NativeOptions` (gui:32) sets no storage; no
   `Serialize`/`Deserialize` on `EnvctlApp` or any secret field. (b) Mint: only `expires_at_unix`
   + `sec_mint_has_token:bool` persist; the token sits in `sec_mint_copy_once:Option<String>`,
   `take()`-dropped after one `ui.output().copied_text` (gui:2387-2405); never to `self.log`. (c)
   Mint stdout never flows through `push_log` — `handle_secrets_result` (gui:604-623) extracts the
   token into the transient holder; the only `push_log` in the path (gui:596) logs `stderr` only.
   `handle_mint_result_holds_token_transiently_not_logged` (gui:3287) asserts the log never
   contains the token. (d) Relay `bearer` NEVER extracted — `RelayMintMeta` (gui:94-99) has no
   bearer field; only `{token_id,expires_at,native}` (gui:624-636); `handle_relay_result_drops_bearer`
   (gui:3272) feeds a real bearer and proves it's dropped + absent from status. (e) Revoke token
   moved into `Zeroizing::new(...)` and piped via stdin (`--token -`), NEVER argv; `sec_revoke_token`
   `clear()`ed (gui:2361-2370); input is `password(true)` (gui:2303);
   `revoke_dispatch_moves_token_to_stdin_and_clears_field` (gui:3218) + `revoke_argv...` (gui:3187)
   assert the literal token never appears in argv.
3. **Fail-closed / dry-run default — PASS.** `sec_revoke_apply` defaults `false` ⇒ `--apply`
   omitted ⇒ daemon dry-run, no egress (`revoke_argv` gui:2127; Tests 4/5). On `code != Some(0)`,
   `handle_secrets_result` (gui:585-598) surfaces `stderr` only in a `⛔` DANGER status — stdout is
   NOT parsed (no synthesized success); `handle_nonzero_exit_surfaces_stderr_not_stdout` (gui:3320)
   proves a token in stdout on a non-zero exit is never parsed/shown. secretctl-not-found ⇒
   `run_secretctl` emits `SecretsResult{code:None}` with an explanatory message and NEVER panics
   (engine:60-70 + `missing_binary_emits_failclosed_result_not_panic`); the GUI renders it as the
   DANGER state (`handle_failure_renders_danger_no_success_card` gui:3305). Mint/relay forms gated
   by `secrets_form_ready` (gui:2136) — no invocation on an invalid form.
4. **No-C trust boundary unchanged — PASS.** `crates/gui/Cargo.toml` byte-unchanged (verified;
   deps = `envctl-engine, eframe, egui, egui_extras, baby-mimalloc`). Engine's added dep is
   `zeroize` (pure-Rust, already a workspace dep; `which` was already an engine dep). `Cargo.lock`
   delta = exactly `+ "zeroize"` on the engine. `no-c.sh` exit=0.
5. **default-OFF / feature-gating parity — PASS (by construction).** secretctl ships
   `provider-github` unconditionally, so there is no CLI feature gate to mirror; the GUI degrades
   gracefully when the binary is absent (`handle_failure_renders_danger_no_success_card`). The GUI
   adds no `cfg` gate. Confirmed.

## Parity check (Engine method → CLI / GUI callers)
- `EngineCommand::Secrets { verb, argv, stdin }` (`command.rs:71`) → dispatched in `run_event_loop`
  (`command.rs:269`) → `secrets::run_secretctl` (`secrets.rs:54`) → emits `Event::SecretsResult`
  (`event.rs` new variant) → GUI drain arm (`gui/src/main.rs:564`) → `handle_secrets_result`.
- GUI argv builders ↔ REAL `secretctl` clap surface (independently cross-checked):
  - `mint_github_argv` (gui:2057) ↔ `Cmd::MintGithub(MintGithubArgs)` `#[command(name="mint-github")]`
    (`cli.rs:70`, args `cli.rs:101-120`). MATCH.
  - `relay_mint_argv` (gui:2083) ↔ `Cmd::Relay → RelayCmd::Mint` (`cli.rs:54`, `:220-239`):
    positional `<name>`, optional `--ttl/--mode/--provider`, repeated `--repo/--perm`, `--json`.
    MATCH. Correctly does NOT inject the native `checks:write` default (left CLI-side).
  - `revoke_argv` (gui:2116) ↔ `github-app revoke-token --token - [--installation-id] [--apply] --json`.
    **NOTE:** `GithubAppCmd` (`cli.rs:82-99`) on this branch base has ONLY `Enroll` — the
    `RevokeToken` variant (PR #124) is NOT yet present. Under Architecture B the GUI does not
    compile-depend on secretctl, so this is a RUNTIME coupling only (see Findings N1), exactly as
    the architect's Risk-1 documented. The argv shape matches the planned #124 surface.

## Argv-parity faithfulness assessment
The replica `MintGithubArgsReplica` (gui:3038-3076) is a hand-rolled parser, not the real clap
struct (deliberate — avoids pulling tonic/tokio into the GUI dev graph). I did NOT take its
faithfulness on trust: I read the real `secretctl/src/cli.rs` and compared field-for-field.
Result — FAITHFUL today: identical arg names, u64/i64 types, fixed `--output json`, and
comma-delimited `Vec` semantics for `--repository-ids`/`--permissions`. The relay/revoke builders
were likewise checked against `RelayCmd::Mint` (`cli.rs:220-239`) and the planned `github-app
revoke-token` surface. No drift between the GUI argv and the real CLI surface.

## Clippy findings (axis × origin)
None. Both axes (gate `-D warnings` and `--all-targets`) returned exit=0 for `envctl-engine` +
`envctl-gui`. No touched-code lints; no inherited-red surfaced in the verified packages. Nothing
was silently "fixed".

## Findings
None blocking. Notes:
- **N1 (note — runtime coupling, planned):** PR #124 (`github-app revoke-token`) is not on the
  branch base; `GithubAppCmd` has only `Enroll` (`secretctl/src/cli.rs:82-99`). The GUI builds/
  tests fine (revoke argv is pure strings; the parity test uses verbatim replication, no secretctl
  import). At runtime, an installed secretctl lacking the verb errors with a non-zero exit → the
  GUI renders the fail-closed DANGER card (`handle_secrets_result` gui:585). #124 must be on the
  installed `secretctl` for the revoke control to function — a deploy-ordering note, not a defect.
- **N2 (defense-in-depth, low):** `build_secrets_command` (gui:2364) builds the `Zeroizing` stdin
  buffer from the token then calls `self.sec_revoke_token.clear()`. `String::clear` resets the
  length but does NOT zero the freed backing capacity, so the token bytes may linger in the
  `String`'s heap buffer until reallocated. The transient input is a `password(true)` field and the
  `Zeroizing` copy IS zeroized on drop, so the exposure window is small; this matches the
  architect's specified `String::clear` design. Optional hardening: drop + reassign a fresh
  `String` (or `Zeroizing` the field) for full zeroization. No invariant impact.
- **N3 (note — verbatim-replica drift risk, mitigated):** the parity test asserts against a
  hand-rolled replica, not the real clap struct. I confirmed it faithful TODAY; if `MintGithubArgs`
  ever changes, the replica must be updated in lockstep — the cross-reference comment at
  gui:3034-3037 already flags this.

## Re-test needed
None — all gates + both clippy axes + tests green on this changeset. If N2 hardening is pursued,
re-run `rtk proxy cargo test -p envctl-gui` + `rtk proxy cargo clippy -p envctl-gui --all-targets -- -D warnings`.
When PR #124 lands on the installed `secretctl`, manually exercise the GUI Revoke control end-to-end
against a running daemon (runtime-only path, not coverable by the GUI unit tests).
