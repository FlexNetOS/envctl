# Implementation log: FULL kasetto v3.2.0 CLI/GUI option parity (TASK-0019)

Branch `task-0019-frontend-gaps` (off develop). All 7 plan items implemented in leaf-first order.
The only accepted divergence remains mimalloc→baby-mimalloc (pre-existing, allocator-only).

## Per-item status
- **Item 7 — global options** DONE. `-q/--quiet` (Count), `-v/--verbose` (Count), `--color {auto,always,never}`, `--no-color` (deprecated alias). `ColorMode` enum + `resolve_plain` (CLICOLOR_FORCE on `always`; stderr deprecation on `--no-color`; NO_COLOR/TTY honored on `auto`). Threaded through `print_event`/agent renderers via a process-global `OUTPUT: OnceLock<OutputCtx>` (presentation-only; engine untouched). `-f` short added to `agent init` force.
- **Item 4a — self-update CORE** DONE (engine). `fetch_latest_release`/`is_newer`/`current_target`/`verify_checksum`/`plan_self_update`, `GITHUB_REPO="FlexNetOS/envctl"`, asset names retargeted to envctl. Reuses `envctl_agent_env::source::http_client()` (blocking, rustls→ring) — ZERO new HTTP/TLS deps.
- **Item 6 — update_notifier** DONE (engine + CLI). Non-printing cache/check core (env override `ENVCTL_CACHE_DIR`, 24h TTL, `available_update`). CLI `main()` spawns the background check, gates via `should_suppress_notice`, renders the end-of-run notice (TTY-gated) via `render_update_notice`/`upgrade_command` (cargo + installer arms; brew inert).
- **Item 1 — agent doctor** DONE (engine + CLI + **GUI parity**). `Engine::agent_doctor(AgentDoctorSpec)->AgentDoctorReport`, read-only, emits one `Event::AgentDoctored`. CLI `envctl agent doctor [--scope]` grouped render (Environment/Inventory/Checks/Command dirs/Failures), honors quiet/color, `--json` round-trips. GUI Doctor sub-tab dispatches `AgentCommandSpec::Doctor` and renders the identical report.
- **Item 4b — self update CLI** DONE. `envctl self update [--json]` in `crates/cli/src/self_update.rs` — download matched asset, verify checksum, atomic replace (`.old` backup + restore-on-fail, 0o755) keyed off `current_exe()`; tar-slip `..` guard preserved; in-archive binaries `envctl`/`envctl-gui`.
- **Item 5 — self uninstall** DONE (engine + CLI). DESTRUCTIVE, fail-closed: dry-run by default (zero writes), `--apply` deletes, TTY `[y/N]` unless `--yes`, non-TTY+`--apply` requires `--yes` (errors otherwise). Binary-removal GUARD refuses unless `current_exe()` file-stem ∈ {envctl,envctl-gui}. Asset teardown delegates to `Engine::agent_clean(apply)`; removes config/data/cache dirs + binary. Emits `Event::SelfUninstall`.
- **Item 2 — completions** DONE. `envctl completions <shell>` via `clap_complete::generate` over envctl's own clap tree (pure-Rust dep `clap_complete = "4.5"`).
- **Item 3 — --frozen alias** DONE. `visible_alias = "frozen"` on the agent `--locked` flags (sync/add/remove) and on lock's `--check`. NOTE: lock's `--check` carries ONLY the `frozen` alias (not `locked`) — envctl's Lock has a distinct real `--locked` zero-network flag, so a `locked` alias on `--check` would collide in clap (kasetto's Lock had no separate `--locked`). Documented in code + ledger.

## Changes (files touched)
- `Cargo.toml`: + `clap_complete = "4.5"` (workspace dep).
- `Cargo.lock`: regenerated for clap_complete.
- `crates/engine/Cargo.toml`: + `reqwest`/`tar`/`flate2`/`sha2` (all pre-existing workspace pins; no new C).
- `crates/cli/Cargo.toml`: + `clap_complete`/`tar`/`flate2`/`sha2`/`reqwest`/`envctl-agent-env` (path).
- `crates/agent-env/src/report.rs`: `SyncFailure` now derives `Deserialize` (needed by the doctor Event).
- `crates/engine/src/self_update.rs` (NEW): self-update CORE + golden tests.
- `crates/engine/src/update_notifier.rs` (NEW): notifier cache/check core + tests.
- `crates/engine/src/self_uninstall.rs` (NEW): `Engine::self_uninstall` + guard/preview tests.
- `crates/engine/src/agent/doctor.rs` (NEW): `Engine::agent_doctor` + `AgentDoctorSpec` + `format_age` + tests.
- `crates/engine/src/agent/{mod,report}.rs`: `AgentVerb::Doctor`, `AgentDoctorReport`/`AgentCommandDirCheck`/`AgentUpdateCheck`, re-exports.
- `crates/engine/src/event.rs`: + `Event::AgentDoctored`, + `Event::SelfUninstall`.
- `crates/engine/src/command.rs`: + `AgentCommandSpec::Doctor` + worker dispatch.
- `crates/engine/src/lib.rs`: new modules + re-exports.
- `crates/cli/src/self_update.rs` (NEW): self-update CLI half (download/extract/atomic-replace).
- `crates/cli/src/main.rs`: global opts + `ColorMode`/`resolve_plain`/`OUTPUT`/`paint`/`emit`; `Cmd::Manage`(`#[command(name="self")]`)+`SelfAction`; `Cmd::Completions`; `--frozen`/`-f`; notifier wiring + end-of-run notice; `AgentCmd::Doctor`; `AgentResult::Doctor` + `render_agent_doctor`; `run_completions`/`run_self_uninstall`; quiet/verbose/plain-aware `print_event`; new `frontend_gaps_tests` module.
- `crates/gui/src/main.rs`: `AgentVerbTab::Doctor`, `agent_last_doctor` state, `agent_doctor_spec`, command arm, Doctor tab + form + `agent_doctor_tables` render, `Event::AgentDoctored` handler.
- `.handoff/loop/rust-port/parity-ledger.md`: flipped FRONTEND-01/02/07/08/09/10 to `[x]` with TASK-0019 closure notes (FRONTEND-03/04/05/06 = binary/ui/banner/colors stay `[≠]` envctl-owned).

## Engine API delta (the parity contract)
- NEW `Engine::agent_doctor(AgentDoctorSpec{scope_override}) -> AgentDoctorReport` (read-only, emits `Event::AgentDoctored`).
- NEW `Engine::self_uninstall(SelfUninstallSpec{apply,yes}) -> SelfUninstallOutcome` (fail-closed; emits `Event::SelfUninstall`).
- NEW free fns: `self_update::{fetch_latest_release,is_newer,current_target,verify_checksum,plan_self_update}` + types `SelfUpdateRelease/SelfUpdateAsset/SelfUpdateCheck`, `GITHUB_REPO`.
- NEW `update_notifier::{spawn_background_check,wait_for_check,read_cached_entry,now_unix_secs,available_update}` + `UpdateCacheEntry`.
- NEW Events: `AgentDoctored{report}`, `SelfUninstall{outcome}`. NEW `AgentVerb::Doctor`, `AgentCommandSpec::Doctor`. NEW report types `AgentDoctorReport/AgentCommandDirCheck/AgentUpdateCheck`.

## Tests added (what they prove)
- engine self_update: is_newer ×5, current_target non-empty, verify_checksum match/mismatch/missing-asset/multi, plan_self_update status map (kasetto golden vectors, asset names retargeted).
- engine update_notifier: cache round-trip, TTL boundary (fresh at TTL-1, stale at TTL), missing-cache None, available_update env-override + newer/same/older compare.
- engine agent::doctor: is_writable ancestor-walk (+ read-only-ancestor refusal, root-skipped), format_age boundaries (s/m/h/d), build_update_check "unknown" without cache, AgentDoctorReport JSON round-trip full field-set (kasetto DoctorOutput field names).
- engine self_uninstall: exe_stem extraction, guard known-stems, remove_dir noop/delete, **preview writes nothing + guard refuses non-envctl binary** (the fail-closed refusal path).
- CLI frontend_gaps_tests: completions non-empty ×4 shells + bin-name, completions parse; --frozen sets locked on sync/add/remove; --check/--frozen on lock; -qq→2/-vvv→3; color modes; --no-color; init -f; resolve_plain never→plain / always→CLICOLOR_FORCE; self update --json parse; self uninstall apply/yes parse + default-preview; agent doctor --scope parse.
- GUI: compile-level parity (Doctor tab → `AgentCommandSpec::Doctor`, `Event::AgentDoctored` handled, `agent_doctor_tables` render).

## Build/test status — commands run + result
- `cargo build -p envctl-engine -p envctl -p envctl-gui` — **PASS** (GUI compiles; no system-lib block hit).
- `cargo test -p envctl-engine -p envctl -p envctl-agent-env -p envctl-gui` — **PASS**. Key result lines:
  - engine lib: `test result: ok. 58 passed; 0 failed`
  - envctl bin (incl. frontend_gaps_tests): `test result: ok. 19 passed; 0 failed` (lib) + bin/integration suites all ok
  - agent-env: all ok (251/82/… passed)
  - gui: `test result: ok. 11 passed; 0 failed`
- `cargo fmt --all --check` — **PASS** (clean).
- `cargo clippy -p envctl-engine -p envctl -p envctl-gui -p envctl-agent-env --all-targets -- ` — **PASS** (0 warnings after clearing the `SelfCmd`→`Manage` enum-variant-name + a GUI doc-list warning).

## CI gates
- `bash ci/gates/no-c.sh` — **PASS** (`rustls=['0.23.40'] on ring=['0.17.14']; zero aws-lc/openssl/C-SQLite`). Confirms `clap_complete` + the reqwest/tar/flate2/sha2 reuse added NO C.
- `bash ci/gates/shape.sh` — **PASS**.
- `bash ci/gates/enable.sh` — **PASS**.
- `bash ci/gates/p7.sh` — **PASS**.

## GUI system-lib block
None. `cargo build -p envctl-gui` compiled cleanly in this worktree; the Doctor-tab parity code is in place and exercised by the GUI test suite.

## Deviations
- **Lock `--check` alias**: kasetto's `lock --check` carries aliases `[locked, frozen]`, but envctl's `agent lock` already exposes a distinct real `--locked` (zero-network audit) flag. Adding a `locked` alias to `--check` collides in clap (`long option names must be unique`). Resolved by giving `--check` ONLY the `frozen` alias; the real `--locked` flag is unchanged. This is the correct no-collision mapping for envctl's richer Lock surface (documented in code + ledger). No capability lost.
- The `--no-color` flag is the rename of kasetto's deprecated `--plain` (kasetto already treats `--plain` as a deprecated alias for `--color never`; envctl exposes it as `--no-color`, same semantics + stderr deprecation warning).

## Handoff notes (for the guardian)
- **Fail-closed uninstall** is the load-bearing invariant: verify `crates/engine/src/self_uninstall.rs::tests::preview_writes_nothing_and_guard_refuses_non_envctl_binary` covers (a) dry-run-by-default = zero writes and (b) the binary-removal guard refusing a non-{envctl,envctl-gui} stem. CLI side: `run_self_uninstall` errors on non-TTY `--apply` without `--yes` (smoke-confirmed: "pass --yes to confirm uninstall in non-interactive mode") and prompts `[y/N]` on a TTY.
- **No-C**: the only new crate is `clap_complete` (pure-Rust). self_update/notifier reuse `envctl_agent_env::source::http_client` + workspace tar/flate2(rust_backend)/sha2 — `no-c.sh` re-run PASS confirms zero new C and the single ring-only rustls.
- **Engine non-printing**: all decision/data logic is in engine modules (self_update/update_notifier/self_uninstall/agent::doctor); ALL printing + the binary-replace + the `[y/N]` prompt are in the CLI; GUI parity for `agent doctor` drives the identical `Engine::agent_doctor`.
- **Presentation globals**: quiet/verbose/color are threaded via `OUTPUT: OnceLock<OutputCtx>` (set once in `main`) + `paint()`/`emit()` — a deliberate front-end-only choice (the engine emits the full event stream regardless), avoiding a signature churn across every renderer. Failures/refusals are NEVER suppressed under `--quiet`.
- **Smoke checks run**: `completions bash` emits a real script; `self uninstall` (no flags) prints the preview + "dry-run: pass --apply…"; `agent doctor` correctly routes through `AgentCtx::resolve` (errored only because the worktree root has no agent-env config — expected).

## Headline status
**GREEN** — all 7 items implemented (engine-first; CLI + GUI parity for doctor; CLI-only for the rest per plan justifications); engine + CLI + agent-env + GUI build and test pass; fmt + clippy clean; no-c/shape/enable/p7 gates PASS.

---

## Re-run note (2026-06-17): config-less `agent doctor` no-downgrade fix

`/verify` of TASK-0019 found a NO-DOWNGRADE bug: `envctl agent doctor` errored
(`failed to load agent config from agent-env.yaml`) when run in a dir with no
`agent-env.yaml`, whereas kasetto's `doctor` runs CONFIG-LESS and envctl's own
`agent list` already runs config-less. The original smoke-note above
("`agent doctor` correctly routes through `AgentCtx::resolve` … errored … expected")
mis-classified the error as expected; it was the downgrade itself.

### Changes
- `crates/engine/src/agent/doctor.rs`: made `Engine::agent_doctor` config-OPTIONAL,
  mirroring kasetto `doctor::run` + `Engine::agent_list`:
  - scope now resolves from `spec.scope_override` else the default `Scope::Global`
    (kasetto `resolve_scope(scope_override, None)`), WITHOUT loading a config —
    replacing the config-REQUIRED `AgentCtx::resolve(None, …)` path.
  - lock / runtime-state / lock-path / scope-root all keyed off `current_dir()` as
    `project_root` (matching kasetto + list.rs), via `agent_lock_path` + `scope_root`.
  - `collect_command_dirs` now loads the config best-effort
    (`match load_config_any(&default_config_path()) { Ok((cfg,_,_)) => cfg.agents(),
    Err(_) => Vec::new() }`) and on any error / empty agents falls back to
    `all_command_{project,global}_targets` — never errors on a missing config.
  - install path / skills / mcps / commands derive from the LOCK only
    (empty / "none" when nothing installed).
  - imports: dropped `AgentCtx`; added `default_config_path`, `load_config_any`,
    `scope_root`, `agent_lock_path`. Spec docstring updated (no longer "from config").

### Engine API
- `Engine::agent_doctor` signature unchanged (`AgentDoctorSpec`, `&EventSink` →
  `AgentDoctorReport`). Behavior change only: config-less is now Ok, not an error.
  CLI + GUI callers unaffected (parity preserved).

### Tests added
- `agent::doctor::tests::doctor_runs_config_less` — with isolated HOME/XDG + a cwd
  containing NO `agent-env.yaml`, `agent_doctor(default)` returns Ok; asserts empty
  skills/mcps/commands, `installation_path == "none"`, `scope == "global"`, and a
  NON-EMPTY `command_dirs` (the all-targets fallback). The 4 existing doctor tests
  remain green.

### Build/test status
- `target/debug/envctl agent doctor` and `--json agent doctor` both EXIT 0 config-less
  (isolated HOME/XDG + tmp cwd) — install path "none", 9-of-9 all-targets command dirs.
- `cargo test -p envctl-engine -p envctl` PASS (doctor lib tests: 5 passed).
- `cargo clippy --workspace -- -D warnings` clean.
- `cargo fmt --all` applied.
- `bash ci/gates/no-c.sh` PASS.

### Handoff notes (for the guardian)
- Verify the new `doctor_runs_config_less` test asserts the all-targets fallback and
  empty inventory (the no-downgrade contract). The fix is read-only — no guard touched,
  no dep added. `collect_command_dirs` swallows config-load errors BY DESIGN (kasetto
  parity: "what does envctl know how to write to?" debugging view).
- One residual: the test mutates process-global `current_dir`/env (HOME/XDG); it
  follows the file's existing env-mutation pattern and 3 back-to-back full lib-test
  runs were clean (no observed flakiness with parallel test threads).
