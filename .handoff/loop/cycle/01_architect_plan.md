# Plan: FULL kasetto v3.2.0 CLI/GUI option parity (TASK-0019)

**VERDICT: GO** — 1 repo (envctl), >3 modules → multi-commit/multi-PR on one branch. Owner directive: NOTHING left out of kasetto; every command/flag/option ports. Only accepted divergence = mimalloc(C)→baby-mimalloc(Rust) global allocator (verified upgrade-only, already in place).

## Delta already-present (DO NOT re-port)
envctl has: agent {sync,add,remove,lock,list,clean,init}; `--json`(global), `--apply`(fail-closed inversion of kasetto `--dry-run` — KEEP), `--locked`, `--update [NAME..]`, `--upgrade-package`, `--scope/--config/--skill/--mcp/--command/--ref/--branch/--sub-dir/--no-sync/--no-verify/--check`. Init already has `--force`+`--global`.

## Target repos
1 repo `envctl`. Modules: workspace Cargo.toml (+clap_complete), crates/engine (agent_doctor, self_update core, self_uninstall, update_notifier; event.rs, command.rs, lib.rs), crates/agent-env (reuse only; maybe notifier cache helper), crates/cli (completions, `self` tree, global flags, --frozen, notifier wiring, doctor render), crates/gui (Doctor tab parity), ci/gates/no-c.sh (re-run only).

## SEQUENCING (dependency order — do not reorder)
1. Item 7 global options (printer/color path foundation)
2. Item 4a self-update CORE (engine: fetch_latest_release/is_newer/current_target/verify_checksum/plan_self_update) — notifier imports it
3. Item 6 update_notifier (engine fetch+cache + CLI render; produces cache item 1 reads)
4. Item 1 agent doctor (depends 6 for update_check block, 7 for quiet/json/color)
5. Item 4b self update CLI (download/extract/atomic-replace)
6. Item 5 self uninstall (destructive, fail-closed)
7. Items 2 (completions) + 3 (--frozen) — independent

## Item 1 — `agent doctor` (engine-first; GUI parity REQUIRED)
Source kasetto src/commands/doctor.rs:15-318 (DoctorOutput, CommandDirCheck, UpdateCheckOutput, collect_command_dirs, is_writable, build_update_check, format_age). Substrate exists: agent-env runtime.rs:73 load_latest_failures (tests-only today), report.rs SyncFailure, lib.rs:44-45 command_*_targets, lock.rs:222/235 list_installed_{mcps,commands}, lock.state().skills.
Engine: new crates/engine/src/agent/doctor.rs — `Engine::agent_doctor(AgentDoctorSpec{scope_override},sink)->AgentDoctorReport`, read-only, non-printing, emits one Event::AgentDoctored. report.rs adds AgentCommandDirCheck{path,writable}, AgentUpdateCheck{status,latest_version,checked_at,age_seconds}, AgentDoctorReport{version,lock_file,scope,skills,installation_path,last_sync,failures:Vec<SyncFailure>,mcps,commands,command_dirs,update_check}. AgentVerb::Doctor. event.rs Event::AgentDoctored. command.rs AgentCommandSpec::Doctor + dispatch (GUI worker drives identical method).
CLI: `envctl agent doctor [--scope]` (+global --json). Human render = kasetto grouped (Environment/Inventory/Checks/Command dirs/Failures), honor quiet (quiet&&!json→no-op) + color. AgentResult::Doctor, always exit 0.
GUI (parity): Doctor sub-tab (AgentVerbTab::Doctor gui main.rs:72), "Run diagnostics" button → AgentCommandSpec::Doctor; handle Event::AgentDoctored into agent_last_doctor:Option<AgentDoctorReport>, render grouped tables.
update_check block depends on Item 6.

## Item 2 — `completions <shell>` (CLI-only)
kasetto src/commands/completions.rs. Top-level `envctl completions <shell>` (clap_complete::Shell positional) → `generate(shell,&mut Cli::command(),"envctl",&mut stdout())` (needs clap::CommandFactory). Generates envctl's OWN tree. Add to should_suppress_notice; not json-gated. Dep: workspace + crates/cli `clap_complete = "4.5"` (pure-Rust, clap+clap_lex only — run no-c.sh). CLI-only justified (clap-tree introspection, no engine logic/GUI analog).

## Item 3 — `--frozen` alias (CLI-only)
Add `visible_alias = "frozen"` to the 4 agent `--locked` flags: Sync main.rs:266, Add:308, Remove:339, Lock(--check):356. No semantic change.

## Item 4 — `self update` (engine-first core, CLI binary-replace)
kasetto src/commands/self_update.rs:1-360. RETARGET GITHUB_REPO → "FlexNetOS/envctl"; asset names kasetto/kst→envctl. Reuse envctl_agent_env::source::http_client() (pub, source.rs:30, blocking) + workspace tar/flate2(rust_backend)/sha2 → ZERO new deps (preferred over flipping engine reqwest to blocking).
Engine: new crates/engine/src/self_update.rs — SelfUpdateRelease/Asset, fetch_latest_release(), is_newer(verbatim semver tuple), current_target(verbatim), verify_checksum(SHA-256 vs checksums.txt), plan_self_update()->SelfUpdateCheck{current,latest,status}. Non-printing.
CLI: top-level `self` subcommand (mirror kasetto ManageSelf) with SelfAction: `envctl self update [--json]` — check→if newer download matched asset, verify checksum, atomic replace (.old backup + restore-on-fail, 0o755) keyed off current_exe(). In should_suppress_notice. CLI half prints progress + does replace (running-binary concern). GUI: CLI-only justified (self-replacing running binary, no GUI analog).

## Item 5 — `self uninstall` (DESTRUCTIVE — fail-closed, dry-run by default)
kasetto src/commands/uninstall.rs:13-131. Faithful removal set: agent assets via existing Engine::agent_clean (agent/clean.rs:27, clears runtime), config dir dirs_agent_env_config(), data dir dirs_agent_env_data(), cache dirs_agent_env_cache(), + the running binary (envctl/envctl-gui).
INVARIANT: default = PREVIEW (no flag → dry-run, zero writes, prints what would be removed). `--apply` required for deletion; TTY `[y/N]` confirm unless `--yes`; non-TTY requires `--yes` (port uninstall.rs:14-19). GUARD: binary removal refuses unless current_exe() file-stem ∈ {envctl,envctl-gui} (fail-closed, NotLiveDevice-style).
Engine: new crates/engine/src/self_uninstall.rs — Engine::self_uninstall(SelfUninstallSpec{apply,yes},sink)->SelfUninstallOutcome{dry_run,skills_removed,mcps_removed,command_dirs_unlinked,config_removed,data_removed,binary_removed,gui_removed,refused:Option<String>}; delegates asset removal to agent_clean(apply); emits Event::SelfUninstall. Refuses to act when apply==false. CLI keeps the [y/N] prompt.
Surface: `envctl self uninstall` (under `self`, sibling of update). GUI: CLI-only justified (would delete running stack; no kasetto GUI).

## Item 6 — update_notifier (end-of-run "new version available" notice)
kasetto src/update_notifier.rs:1-289 + app.rs:191-219 (should_suppress_notice, current_program_name). cache dir dirs_agent_env_cache(); env override KASETTO_CACHE_DIR→ENVCTL_CACHE_DIR; repo via item-4 self-update core (FlexNetOS/envctl); 24h TTL.
Engine (non-printing): new crates/engine/src/update_notifier.rs — spawn_background_check()->Option<handle>, wait_for_check(handle,timeout), read_cached_entry()->Option<UpdateCacheEntry> (used by item 1), now_unix_secs(), available_update()->Option<(current,latest)>.
CLI: mirror kasetto app.rs::run — spawn check up front; should_suppress_notice(&cli.cmd) (suppress for --json/--quiet/completions/self/machine-readable verbs; never for install/reset/auto-fix human runs); wait_for_check(800ms) unless suppressed; at end on success render notice via available_update() gated by TTY+suppress. Port current_program_name (default "envctl"). upgrade_command(): keep cargo arm (cargo install envctl) + installer arm (→ envctl self update); brew arm inert. GUI: none (end-of-run CLI concept).

## Item 7 — global options -q/--quiet, -v/--verbose, --color, --no-color (+Init -f)
kasetto cli.rs:52-95,170-187 + colors.rs:74-82. Add to top-level Cli (all global=true): quiet:u8 (ArgAction::Count, short q), verbose:u8 (Count, short v), color:ColorMode (auto|always|never, default auto), no_color:bool (hide=true, deprecated alias for --color never). Define ColorMode ValueEnum lowercase. Port resolve_plain: always→set CLICOLOR_FORCE=1; no_color→never + stderr deprecation note; never→plain; auto→respect NO_COLOR/TTY.
REAL effects wired through CLI printer (print_event main.rs:1204 + human renderers): quiet>=1 drops Log/StepStarted/info, keeps failures/refusals (not --json); verbose unfilters Event::Log detail; color/no-color gate ANSI via resolved plain. Engine: NONE (pure front-end; engine already emits full stream non-printing).
Init parity: add `short='f'` to agent init force (kasetto Init has -f). --global+--force already present.
GUI: none (terminal concepts). CLI-only justified.

## Invariants
no-C: only clap_complete added (pure-Rust); self_update/notifier reuse existing reqwest-rustls-ring + tar + flate2(rust_backend) + sha2 — zero new C (prefer reusing agent-env http_client()). Run `bash ci/gates/no-c.sh` after dep change. One ring-only rustls (no new TLS). Engine single/sync/non-printing (all decision logic in engine, all printing + binary-replace + [y/N] prompt in CLI). CLI+GUI parity: doctor REQUIRES GUI parity; completions/self/notifier/global-flags/--frozen CLI-only with documented justifications. Destructive (uninstall) fail-closed + dry-run default + binary-stem guard.

## Lock/manifest
No envctl.lock/agent-env.lock/manifest change. Commit regenerated Cargo.lock (clap_complete). Flip FRONTEND-01..10 + --frozen + AP rows to [x] in .handoff/loop/rust-port/parity-ledger.md as closed (note mimalloc the sole [≠], upgrade-only).

## Work breakdown (leaf-first — implementer follows in order)
1. Item 7 global options + ColorMode + resolve_plain + thread quiet/verbose/color through print_event/renderers + `-f` short on agent init. Tests.
2. Item 4a self-update core (engine/src/self_update.rs, reuse agent-env http_client). Port golden tests. Export lib.rs.
3. Item 6 update_notifier (engine/src/update_notifier.rs + CLI wiring in main()). Port tests.
4. Item 1 agent doctor (engine/src/agent/doctor.rs + report/event/command/lib; CLI agent doctor + render + AgentResult::Doctor; GUI Doctor tab + agent_last_doctor + Event handling). Port doctor unit tests.
5. Item 4b self update CLI (cli/src/self_update.rs download/extract/atomic-replace; `self` subcommand tree).
6. Item 5 self uninstall (engine/src/self_uninstall.rs preview/apply/guard delegating agent_clean + Event::SelfUninstall; CLI self uninstall confirm/--yes/--apply). Tests.
7. Items 2+3 (clap_complete + completions <shell>; --frozen visible_alias ×4+lock-check). Tests.
8. All 4 CI gates + cargo fmt --all + cargo clippy --workspace -D warnings + full cargo test --workspace.

## Tests (kasetto golden vectors — port verbatim)
Engine: is_newer ×5, verify_checksum match/mismatch/missing-asset/multi, current_target non-empty; notifier cache round-trip/TTL boundary/render plain+color/classify_install_path/missing-cache; doctor is_writable ancestor-walk/build_update_check status map/format_age boundaries; self_uninstall preview=zero-writes/binary-stem-guard-refuses/apply-removes-temp-tree.
CLI: completions ×4 shells non-empty+exit0; --frozen sets locked ×4; -qq→2/-vvv→3/--no-color→plain+deprecation(stderr)/--color always→CLICOLOR_FORCE; agent doctor --json round-trips; self uninstall non-TTY+--apply w/o --yes errors.
GUI: compile-level parity (Doctor tab dispatches AgentCommandSpec::Doctor, handles Event::AgentDoctored).
CI: no-c.sh (MUST, after clap_complete) + shape.sh expect PASS; enable.sh/p7.sh untouched; fmt+clippy.

## Suggested PR split (single branch)
PR-1 Item7 global options. PR-2 Items 4a+6 self-update-core+notifier. PR-3 Item1 agent doctor (engine+CLI+GUI). PR-4 Items 4b+5 self update CLI + uninstall. PR-5 Items 2+3 completions + --frozen.

## Implementer defaults (non-blocking)
(a) self-update HTTP = reuse envctl_agent_env::source::http_client() (zero new deps). (b) notifier upgrade_command keeps cargo+installer arms (installer→`envctl self update`), brew inert.
