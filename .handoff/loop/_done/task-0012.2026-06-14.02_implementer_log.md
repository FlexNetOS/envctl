# Implementation log: TASK-0014 — envctl `agent` CLI group

## Changes
- crates/cli/src/main.rs: thin `agent {sync,add,remove,lock,list,clean}` adapter over the
  existing `Engine::agent_*`. Added (line refs):
  - imports of the agent spec/enum types (main.rs:5-10).
  - `Cmd::Agent { cmd: AgentCmd }` variant (main.rs:~203).
  - `ScopeArg`→`AgentScope` (main.rs:213) and `ListKindArg`→`AgentListKind` (main.rs:229).
  - `lock_mode_from(locked, update)`→`AgentLockMode` (main.rs:250): `--locked`→`Locked`
    (wins), `--update [names]`→`Update{only}`, else `Plain`.
  - `AgentCmd` enum with the six verbs + field-by-field flags (main.rs:262); `apply` defaults
    FALSE; `--json` inherited from the global `Cli.json` (no per-verb flag).
  - explicit dispatch `Cmd::Agent { cmd } => run_agent(...)` in `main()` (main.rs:525), like
    Dashboard/Env; added `Cmd::Agent { .. }` to the `run_action` unreachable arm (main.rs:852).
  - `AgentResult` enum + `to_json`/`ok` (main.rs:880) — the pure exit/render decision helper.
  - `run_agent(engine, AgentCmd, json)` (main.rs:918): worker thread builds the Spec, calls the
    Engine method with the channel sink; main thread drains `rx.iter()` (human only) then prints
    the uniform serialized RETURN value under `--json`, and maps the fail-closed exit.
  - 4 `Event::Agent*` arms in `print_event` (main.rs:1176): RunStarted / Action / RunFinished /
    LockChecked (previously fell through `_ => {}`).
  - `#[cfg(test)] mod agent_cmd_tests` (main.rs:~722): `lock_mode_from` + `ScopeArg`→`AgentScope`.
- crates/cli/tests/agent.rs: NEW integration test (modeled on tests/dashboard.rs), hermetic
  `Fixture` (own cwd + `agent-env.yaml` + empty `ENVCTL_MANIFEST_DIR` + isolated
  `XDG_DATA_HOME`/`XDG_CONFIG_HOME`; `Drop` cleans up).
- NO engine `src/` changes. NO `lib.rs` re-export added — every type the CLI needs
  (`AgentAddSpec`, `AgentCleanSpec`, `AgentListKind`, `AgentListSpec`, `AgentLockMode`,
  `AgentLockSpec`, `AgentRemoveSpec`, `AgentScope`, `AgentSectionSel`, `AgentSyncSpec`, and the
  return types `AgentReport`/`AgentEditOutcome`/`AgentLockOutcome`/`AgentList`) was already
  re-exported from `envctl_engine`.

## Engine API
No engine API change. CLI consumes the already-merged, parity-verified contract:
`Engine::agent_sync(AgentSyncSpec)->AgentReport`, `agent_add(AgentAddSpec)->AgentEditOutcome`,
`agent_remove(AgentRemoveSpec)->AgentEditOutcome`, `agent_lock(AgentLockSpec)->AgentLockOutcome`,
`agent_list(AgentListSpec)->AgentList`, `agent_clean(AgentCleanSpec)->AgentReport`; events
`Event::Agent{RunStarted,Action,RunFinished,LockChecked}`.

## Tests added
Unit (crates/cli/src/main.rs):
- `agent_cmd_tests::lock_mode_from_maps_each_flag` — each flag combo → the right `AgentLockMode`
  (`--locked` wins; `--update []`=all; `--update foo bar`; default `Plain`).
- `agent_cmd_tests::scope_arg_converts_to_agent_scope` — `ScopeArg`→`AgentScope` both arms.

Integration (crates/cli/tests/agent.rs), driving `CARGO_BIN_EXE_envctl`:
- `sync_dry_run_writes_nothing`, `add_dry_run_writes_nothing`, `clean_dry_run_writes_nothing` —
  no `--apply` ⇒ exit 0 AND config + `agent-env.lock` + dest dir byte-identical before/after
  (fail-closed invariant; the dest-absent check is non-vacuous, asserted separately).
- `list_json_has_agent_list_shape` — `agent list --json` parses with the `AgentList` shape
  (`skills`/`mcps`/`commands` arrays + `merged_scopes` bool).
- `lock_check_json_has_outcome_shape` — `agent lock --check --json` parses with the
  `AgentLockOutcome` shape (`check=true`, `saved=false`, `drift` array) AND wrote no lock.
- `list_exits_zero` — exit-code contract for the read-only verb.
- `add_ref_and_branch_conflict_exits_nonzero` — `agent add src --ref a --branch b` ⇒ engine
  `bail!` propagates through the worker-join `?` ⇒ nonzero exit.
- `help_lists_the_six_verbs`, `fixture_dest_absent_until_apply` — surface/fixture sanity.

NOTE on the `sync --apply` failure-exit fixture: not built (would require a real failing network
fetch — explicitly disallowed by the plan: "don't fake a network fetch"). The `failed>0→exit`
decision is instead exercised by the pure `AgentResult::ok()` logic and asserted via the dry-run
exit-0 path + the engine-bail nonzero path; the `--apply` happy/sad split lives in the engine's
own `crates/engine/tests/agent_sync_parity.rs` (TASK-0012, #84).

## Build/test status
Run from worktree root:
- `cargo build -p envctl-engine -p envctl` — PASS.
- `cargo test -p envctl` — PASS: **18 passed; 0 failed** (5 lib unit + 9 tests/agent.rs +
  4 tests/dashboard.rs).
- `cargo fmt --all` then `cargo fmt --all --check` — clean (exit 0).
- `cargo clippy --workspace -- -D warnings` (the exact CI invocation in .github/workflows/ci.yml)
  — clean (exit 0).
- `bash ci/gates/no-c.sh` — PASS (rustls 0.23.40 on ring 0.17.14; zero aws-lc/openssl/C-SQLite).
- `bash ci/gates/shape.sh` — PASS. `bash ci/gates/enable.sh` — PASS.
- Manual smoke: `agent --help` lists all six verbs + the global `--json`; `agent list --json`
  emits `{"skills":[],"mcps":[],"commands":[],"merged_scopes":true}`.

## Deviations
- `cargo clippy --workspace --all-targets` (NOT the gate command) surfaces 4 pre-existing
  `unnecessary_to_owned` errors in `crates/engine/tests/agent_sync_parity.rs` — a file I did NOT
  touch (committed in #84). This repo's CI gate is `cargo clippy --workspace -- -D warnings`
  WITHOUT `--all-targets` (.github/workflows/ci.yml:48-49), which is clean. Per the per-repo
  CI-mirror rule, the `--all-targets` lints are out of scope for this task and for the CLI crate.
  Flagging for the guardian; not fixed here (would be an engine-test edit beyond TASK-0014's
  `crates/cli/*` scope).
- `lock_mode_from`/`AgentResult::ok` use `map_or(true, …)` rather than 1.82's `is_none_or`
  (MSRV is 1.80); clippy on the 1.96 dev toolchain does not flag it.

## Handoff notes
- Fail-closed verification target: every mutating verb defaults `apply:false`. The three
  `*_dry_run_writes_nothing` tests prove zero on-disk mutation (config + lock + dest) — verify the
  dest-absent assertion is non-vacuous (it is: `fixture_dest_absent_until_apply` confirms the
  fixture's dest does not pre-exist).
- Exit-code mapping lives ENTIRELY in `AgentResult::ok()` (main.rs:880) — sync/clean
  `summary.failed>0`; add/remove `outcome.sync.summary.failed>0` (None sync ⇒ ok); lock
  `--check && !drift.is_empty()`; list always ok. Engine `bail!`s propagate via the worker
  `join()??`. No business logic in the CLI — it only builds Specs, drains events, renders.
- `--json` path is silent on the EventSink and prints only the uniform pretty-serialized RETURN
  value (matches auto-detect/graph/lock); human path streams the per-action tree via `print_event`.
- No new dependency (no-c stays green); no engine `src/` or `lib.rs` change.

---

# Re-run note: TASK-0014b — GUI Agent panel (GUI half)

The ENGINE half (Event::AgentListed/AgentEdited, AgentCommandSpec, EngineCommand::Agent +
run_event_loop arm, AgentLockMode::from_flags, CLI print_event no-op arms) was already landed and
verified before this session and was NOT re-edited. This section covers only the GUI half.

## Changes (this session)
- `crates/engine/src/lib.rs`: the ONLY engine edit — additive re-exports so the GUI `use` resolves:
  added `AgentEditItem`, `AgentLockDriftItem` (from `agent`) and `AgentCommandSpec` (from `command`).
  No logic changed (diff is the two re-export lines).
- `crates/gui/src/main.rs`: the full Agent panel — screen, state, pure state→Spec builders, drain
  arms, render, tests.

## Engine API (parity contract the GUI now drives — unchanged, just consumed)
- `EngineCommand::Agent { spec: AgentCommandSpec }`, `AgentCommandSpec::{Sync,Add,Remove,Lock,List,Clean}(Agent*Spec)`.
- Events consumed: `AgentRunFinished{report}`, `AgentLockChecked{drift}`, `AgentListed{list}`, `AgentEdited{outcome}`, `AgentAction{..}`.
- `AgentLockMode::from_flags(locked, update)` shared by CLI + GUI.

## GUI additions (key file:line in crates/gui/src/main.rs)
- `Screen::Agent` variant + `label()` + nav-tab row + `Screen::Agent => self.agent_screen(ui)` arm.
- UI selector enums with pure mappings: `AgentVerbTab`, `AgentScopeSel::to_override()->Option<AgentScope>`,
  `AgentListKindSel::to_kind()->AgentListKind`.
- `EnvctlApp` agent state fields (verb tab, form inputs, lock toggles, result holders
  `agent_list/agent_last_edit/agent_last_report/agent_lock_drift/agent_status`), all init in `new()`.
- `drain()` arms for the 5 agent events (catch-all kept).
- PURE state→Spec builders (no egui types): `agent_lock_mode`, `agent_sync_spec`, `agent_section`,
  `agent_add_spec`, `agent_remove_spec`, `agent_lock_spec`, `agent_list_spec`, `agent_clean_spec`,
  `agent_command()->EngineCommand::Agent{spec}`. Field maps mirror `cli/src/main.rs:943-1060`
  field-for-field: blank→`None` via `opt_str`, CSV→`split_csv`→`Vec`, `agent_apply`→`apply`,
  scope/list-kind/lock-mode mappings; lock uses `from_flags(locked, None)` (no `--update` on lock).
- `opt_str` helper beside `split_csv`.
- `agent_screen` + helpers: 6 verb sub-tabs; per-verb controls; Preview/Apply gating (button carries
  `agent_apply`, default false → "Preview"; true → "Apply" in `theme::WARN`); `clean` WARN caution;
  results render list as TableBuilder tables (skills: name/skill/scope/source/updated; mcps+commands:
  name/scope/source), edit outcome (action + section/target items), report summary
  (installed/updated/removed/unchanged/failed), lock-drift list. Click → `self.dispatch(self.agent_command(), Some("agent".into()))`.

## Tests added (`#[cfg(test)] mod agent_spec_tests` in gui/src/main.rs — headless, no window)
Helper builds an `EnvctlApp` via `Engine::detached()` (manifest-independent; builders touch no
engine/channel). 11 tests: sync blank-config→None + apply-default-false; config/apply/scope map;
lock-mode locked-wins / update-CSV-split / update-off-Plain; add section-CSV + fields (no_sync/no_verify,
apply false); remove (no no_verify); lock check+upgrade+Locked; list kind+scope; clean apply-default-false;
agent_command wraps active verb.

## Build/test status (exact commands, worktree root)
- `cargo build -p envctl-engine -p envctl` — PASS.
- `cargo build -p envctl-gui` — PASS (13.46s; system dev libs ARE present in this env — NOT blocked).
- `cargo test -p envctl-gui` — PASS (11 passed).
- `cargo test -p envctl-engine -p envctl` — PASS (116 passed).
- `cargo fmt --all` + `cargo fmt --all --check` — PASS (exit 0).
- `cargo clippy -p envctl-gui -- -D warnings` — PASS (no issues).
- `cargo clippy --workspace -- -D warnings` — PASS (no issues).
- `bash ci/gates/no-c.sh` — PASS (rustls=0.23.40 on ring=0.17.14; zero aws-lc/openssl/C-SQLite).

## Deviations (GUI half)
- Test helper uses `Engine::detached()` not `Engine::load_default()`: the latter panics under
  `cargo test` (no `manifest/` in cwd). `detached()` is the engine's sanctioned manifest-independent
  constructor; the spec builders never read the registry, so this is correct/deterministic. No new
  constructor added.

## Handoff notes (GUI half)
- Engine edit is the single additive re-export block in `crates/engine/src/lib.rs` — verify no logic
  changed. The other modified engine/CLI files (cli/main.rs, agent/{edit,list,mod}.rs, command.rs,
  event.rs) are the PRE-EXISTING engine/CLI halves, untouched this session.
- Fail-closed targets: `agent_clean_spec`/`agent_sync_spec`/`agent_add_spec`/`agent_remove_spec` all
  default `apply=false` (tests `clean_apply_defaults_false`, `sync_blank_config_is_none_apply_defaults_false`).
- Parity: every `*_spec` builder is field-for-field with `cli/src/main.rs:943-1060`.
- UI-thread non-blocking: action button → `dispatch()` mpsc send only; results via `drain()`/`try_recv`.
