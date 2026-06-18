# TASK-0028 — GUI parity (mint-github / relay-mint / revoke) · VERDICT: GO

**Architecture: B — `envctl-gui` shells out to the installed `secretctl` binary as a subprocess and parses its `--json` stdout.** Zero new crate deps, GUI stays pure-sync, divergence is structurally impossible (the GUI drives the identical clap surface). All three controls render METADATA ONLY, default to dry-run (revoke `apply=false`), feed the revoke token over `secretctl` stdin (never argv), never persist any secret byte (eframe persistence is already off — no `save()` impl exists).

## Architecture decision (A vs B)
Option B is strictly dominant. The GUI worker (`crates/gui/src/main.rs:248-252`) drives `envctl_engine::Engine` over mpsc and the GUI crate deps are only `envctl-engine, eframe, egui, egui_extras, baby-mimalloc` (`crates/gui/Cargo.toml:11-16`). The secrets verbs are unreachable from that engine — they require a tonic `VaultClient` over the daemon UDS that `secretctl` builds in its own runtime (`crates/secretctl/src/main.rs:24-31,59-73`). Option A would graft a second async runtime + tonic + prost + secrets-proto into a pure-sync egui app, force a fresh `ci/gates/no-c.sh` re-proof, and re-implement the `mint_req_for_relay_mint`/`MintGithubReq` builders that already exist in secretctl — the exact CLI↔GUI divergence the invariants forbid. Option B drives the identical clap surface so the GUI CANNOT mint/revoke differently; adds zero deps; stays pure-sync; reuses secretctl's own no-argv-leak stdin path for the revoke token.

## Target repos
Single repo: **envctl** (no cross-repo / multi-crate scale trigger → sequential single-crew).
- `crates/gui/src/main.rs` — NEW `Screen::Secrets` + screen fn + state fields + result events; NEW pure spec→argv builders (unit-testable, mirror secretctl) + subprocess dispatch path. Linear chain (Screen enum → state → builders → dispatch → render) → sequential.
- `crates/gui/Cargo.toml` — NO change under Option B.
- `crates/engine/src/lib.rs` (+ event loop) — NEW `EngineCommand::Secrets { argv, stdin }` + `Event::SecretsResult { verb, json_stdout, stderr, code }`; engine spawns `secretctl`, writes stdin, captures stdout/stderr/exit, emits event — holds NO secret bytes after the child exits.

## Scope
1. **mint-github** → `secretctl mint-github --installation-id … [--repository-ids …] [--permissions …] --ttl-secs … --output json` (`cli.rs:101-120`, `main.rs:292-316`). Render metadata only (`expires_at_unix` + transient copy-once token) — never persist the frozen `{token,expires_at_unix}`.
2. **relay-mint** → `secretctl relay mint <name> [--ttl …] [--mode …] [--provider …] [--repo …]… [--perm …]… --json` (`cli.rs:220-239`, `main.rs:495-513`, builder `main.rs:565-598`). Render `render_mint` metadata `{token_id, expires_at, native}` — do NOT display `bearer` (secret-class).
3. **revoke (installation-token)** → `secretctl github-app revoke-token --token - [--installation-id …] [--apply] --json` (PR #124 surface). Token via stdin. Render `{"revoked":bool,"dry_run":bool}`. Dry-run default (`--apply` only on explicit affordance).

Out of scope: relay-policy/bearer revoke GUI (stretch follow-up); Option A; secret/init/unlock/ca GUI surfaces.

## Daemon gRPC / CLI surface the GUI drives (verified)
| GUI control | secretctl invocation | RPC reached | Response rendered |
|---|---|---|---|
| Mint GitHub | `mint-github … --output json` | `Vault.MintGithub` → `MintGithubResp{token,expires_at_unix}` (FROZEN) | `expires_at_unix` + transient copy-once token; token never persisted |
| Relay mint | `relay mint <name> … --json` | `Relay.Mint(MintReq)` → `MintResp` | `{token_id,expires_at,native}` (NOT `bearer`) |
| Revoke install-token | `github-app revoke-token --token - [--installation-id N] [--apply] --json` | `Vault.RevokeGithubToken{token,apply,installation_id}` → `RevokeResp{count_revoked,dry_run}`; CLI prints `{"revoked":bool,"dry_run":bool}` | `{revoked,dry_run}`; apply=false default |

## CLI parity reference (each GUI control ↔ secretctl arg/field)
**Mint GitHub** — `MintGithubArgs` (`cli.rs:101-120`): `--installation-id`(u64,req) · `--repository-ids`(CSV,blank⇒omit) · `--permissions`(CSV `name:access`,blank⇒omit) · `--ttl-secs`(i64,req) · fixed `--output json`.
**Relay mint** — `RelayCmd::Mint` (`cli.rs:220-239`), builder `main.rs:565-598`: positional `<name>` · `--ttl`(blank⇒omit) · `--mode`(base-url/proxy/native, `mode_to_proto` `main.rs:86-94`) · `--provider`(anthropic/openai/github/generic, `provider_to_proto` `main.rs:75-84`) · repeated `--repo`(CSV) · repeated `--perm`(CSV; native default `checks:write` applied CLI-side `main.rs:583-587`, NOT in GUI).
**Revoke** — `github-app revoke-token` (#124): `--token -`(stdin) · `--installation-id`(u64,opt) · `--apply`(default OFF ⇒ dry-run, no egress) · fixed `--json` → `{"revoked":bool,"dry_run":bool}`.

## GUI changes (existing pattern mirrored + insertion points)
Pattern mirrored: the **Agent screen** (`gui/src/main.rs:1260-1873`) — pure state→spec builders, a verb sub-tab enum, a single dispatch button with apply/preview gating (`agent_action_button` `:1595-1625`), result holders filled from worker events (`drain` `:321-482`). Secrets screen mirrors it exactly but the "spec" is an argv `Vec<String>` and dispatch runs a subprocess.

1. **New `Screen::Secrets`** — `main.rs:43-53` add variant; label `:56-67`; nav `:539-548`; central dispatch `:568-577` → `Screen::Secrets => self.secrets_screen(ui)`.
2. **New verb sub-tab** — beside `AgentVerbTab` (`:70-94`): `enum SecretsVerbTab { MintGithub, RelayMint, Revoke }` + `label()`.
3. **New state fields** — append to `struct EnvctlApp` (`:165-232`) and BOTH constructors (`new` `:254-315`, test `test_app` `:2150-2211`): `secrets_verb`; mint form `sec_install_id/sec_repo_ids/sec_perms/sec_ttl_secs:String`; relay form `sec_relay_name/sec_relay_ttl/sec_relay_mode(def "base-url")/sec_relay_provider(def "generic")/sec_relay_repos/sec_relay_perms:String`; revoke form `sec_revoke_token:String`(transient secret), `sec_revoke_install_id:String`, `sec_revoke_apply:bool`(def false); results `sec_status:String`, `sec_mint_expires:Option<i64>`, `sec_mint_has_token:bool`, `sec_relay_result:Option<RelayMintMeta>`({token_id,expires_at,native} — no bearer), `sec_revoke_result:Option<(bool,bool)>`.
4. **Pure argv builders** (unit-testable; reuse `split_csv`/`opt_str` helpers `:2116-2133`):
   - `mint_github_argv(&self)->Vec<String>` — `["mint-github","--installation-id",id,"--ttl-secs",ttl,"--output","json"]` + conditional `--repository-ids`/`--permissions` (replicate comma-join rules `main.rs:292-303`, `:891-929`).
   - `relay_mint_argv(&self)->Vec<String>` — `["relay","mint",name]` + optional `--ttl`/`--mode`/`--provider`, repeated `--repo`/`--perm`, `--json`. Do NOT inject `checks:write` (left to secretctl).
   - `revoke_argv(&self)->Vec<String>` — `["github-app","revoke-token","--token","-"]` + optional `--installation-id`, conditional `--apply`, `--json`.
5. **Subprocess dispatch (engine-owned, recommended).** `EngineCommand::Secrets { argv:Vec<String>, stdin:Option<Zeroizing<Vec<u8>>> }` + `Event::SecretsResult { verb, json_stdout, stderr, code }` (engine `lib.rs` + `run_event_loop` match). Engine spawns `secretctl` via `std::process::Command`, writes stdin pipe, captures stdout/stderr, emits result — parses nothing secret, holds no token after child exits. GUI dispatch reuses `self.dispatch(cmd, Some("secrets".into()))` (`:492-497`); results land in `drain` (add arms after `:479`). Binary resolution: (a) alongside `std::env::current_exe`, (b) `$HOME/.cargo/bin/secretctl` (`manifest/env-ctl.toml:66`), (c) `PATH`; unresolved → `SecretsResult` error (matrix).
6. **New `secrets_screen(ui)`** — model on `agent_screen` (`:1371-1500`): verb sub-tab row, `theme::inset()` form per verb, single action button (label "Mint"/"Mint relay"/(apply?"Revoke (apply)":"Revoke (dry-run)"); fill `theme::WARN` when `sec_revoke_apply`, else `theme::ACCENT`); metadata-only results cards (mirror `agent_results` `:1627-1680`).

## Secret-handling design
1. **eframe persistence already OFF** — `EnvctlApp` has no `save()`, `run_native` (`:36-41`) has no storage path, `NativeOptions` (`:32-35`) sets no persist. No app state ever serialized. Plan FORBIDS adding a `save()` or `#[derive(Serialize)]` on `EnvctlApp`/secret fields.
2. **Minted token** — engine emits `SecretsResult.json_stdout` with frozen `{token,expires_at_unix}`. GUI parses once, stores ONLY `expires_at_unix:i64` + `sec_mint_has_token:bool`; holds the token string transiently in the `drain` arm for a single copy-once `ui.output_mut(|o| o.copied_text = token)`, dropped at arm end; never to `self.log`/`push_log`. Mint stdout must NOT go through `push_log` (`:485-490`) — only stderr/metadata.
3. **Relay `bearer`** — never shown/stored; only `token_id/expires_at/native` (`render.rs:153-182`).
4. **Revoke token input** — `sec_revoke_token:String`, fed via stdin (`--token -`), never argv. On dispatch moved into `Zeroizing<Vec<u8>>` for `EngineCommand::Secrets.stdin`, `sec_revoke_token` cleared (`String::clear`), never persisted. Engine writes the buffer to child stdin, drops it.
5. No secret to clipboard except the explicit copy-once mint affordance; never to `self.log`/`LogLine`.

## Fail-closed matrix
| Condition | Detection (B) | GUI behavior (never synthesized success) |
|---|---|---|
| secretd not running | secretctl non-zero; stderr "is secretd running?" (`main.rs:71`) | real stderr in DANGER card; no success card |
| Vault locked / USB possession not proven | daemon error; secretctl non-zero stderr | stderr verbatim DANGER; button stays enabled (retry after unlock) |
| secretctl binary not found | engine current_exe/cargo-bin/PATH all miss | `SecretsResult` error "secretctl not installed"; disabled/explanatory |
| Revoke without apply | `sec_revoke_apply=false` ⇒ omit `--apply` | `{dry_run:true}` "dry-run: would revoke (no egress)" — default |
| Mint missing required field | GUI pre-validates non-empty + parses u64/i64 | button `add_enabled(false)` until valid (`:1181-1190`); no invocation |
| Bad repo id / perm | secretctl/daemon rejects | stderr surfaced; no partial success |

## Tests (inline `#[cfg(test)] mod` in gui/src/main.rs, mirror `agent_spec_tests` `:2135-2344` via `test_app()`/`Engine::detached()`)
1. **Argv parity (anti-divergence):** each verb's GUI argv parses into the SAME secretctl clap struct/fields (GUI analog of `mint_github_argv_round_trips_through_clap` `main.rs:1033-1065`). **Use the verbatim-replication path (no `envctl-secretctl` dev-dep)** to avoid pulling tonic/tokio into even the dev graph — replicate the arg-struct assertion like secretctl's consumer builder (`main.rs:891-929`).
2. `mint_github_argv_omits_blank_optional_scopes` — blank repo-ids/perms ⇒ flags omitted.
3. `relay_mint_argv_maps_mode_provider_repos_perms` — combos + CSV → repeated flags; native default perm NOT injected by GUI.
4. `revoke_argv_defaults_dry_run_uses_stdin_token` — no `--apply` default; `--token -` always (never literal token in argv); `--apply` only when toggle on.
5. `revoke_apply_toggle_defaults_false`.
6. **No-persist:** guard that `EnvctlApp` has no `save` override and secret fields are plain `String`/`Zeroizing`; assert `sec_revoke_token` cleared after building dispatch.
7. **JSON metadata parse:** feed `{"revoked":true,"dry_run":false}` and `render_mint` JSON; assert only metadata retained (no `bearer`, no mint `token`).
8. **Degrade:** secretctl ships `provider-github` unconditionally (`secretctl/Cargo.toml:19`) → verbs always present → no GUI cfg gate; degrade test asserts the binary-absent path renders the explanatory state.

CI gates: `shape.sh` (new screen/module shape) green; `no-c.sh` UNAFFECTED (zero new deps); `cargo fmt --all` + `clippy --workspace -Dwarnings` + `test -p envctl-gui` before push.

## Invariants (each checkable)
1. Engine single authority / no divergence — GUI has zero mint/revoke logic; drives identical secretctl clap surface; argv-parity tests prove it. PASS.
2. No secret rendered/persisted — eframe persistence off (no `save()`; forbid adding); mint token transient copy-once then dropped; `bearer` never shown; revoke token via stdin in `Zeroizing`, field cleared; nothing secret to `push_log`. PASS (Tests 6,7).
3. Fail-closed / dry-run default — `sec_revoke_apply` def false ⇒ no `--apply` ⇒ daemon dry-run no egress; mint/relay require explicit fields, button disabled until valid. PASS (Tests 4,5).
4. No-C trust boundary — Option B adds ZERO crate deps; `no-c.sh` graph unchanged. PASS.
5. default-OFF / feature-gating parity — secretctl ships verbs unconditionally (no CLI feature gate to mirror); GUI degrades gracefully when binary absent (Test 8). PASS-by-construction.

## Risks
1. **PR #124 (`github-app revoke-token`) not yet on develop.** Under Architecture B the GUI does NOT compile-depend on secretctl, so the GUI builds/tests fine without #124 (revoke argv is pure strings; parity tests use verbatim replication, no secretctl import). The ONLY coupling is runtime — an installed secretctl without revoke-token errors gracefully (fail-closed matrix). #124 is auto-merging independently. **Mitigation: build off develop now with verbatim-replication tests; no rebase-on-#124 required.**
2. **Test 1 dep edge.** A `#[cfg(test)]` dev-dep on `envctl-secretctl` would pull tonic/tokio into the GUI dev graph — AVOID; use verbatim replication (Risk-1 mitigation doubles here).
3. **Subprocess stdout secret hygiene.** Engine must route mint stdout to structured `SecretsResult` only, never the generic log/stderr pump — one accidental `push_log` of mint stdout leaks the token. Enforced by routing + Test 7.

## Out of scope / named follow-ups
- Relay-policy revoke / bearer revoke GUI (`relay revoke`/`relay revoke-token` `cli.rs:201-214`) — follow-up TASK if owner wants full relay-revoke parity.
- Option A (embedded gRPC client) — revisit only if the GUI needs streaming daemon events the CLI doesn't expose.
- secretctl path-discovery hardening (configured install prefix) — start current-exe → `~/.cargo/bin` → PATH.
