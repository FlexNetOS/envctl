# TASK-0014b architect plan — GUI Agent panel (Epic C front-end, GUI half)

VERDICT: GO (one cycle, all 6 verbs). GUI drives the IDENTICAL Engine::agent_* API as the CLI (#90/#91).

## Engine deltas (transport only — NOT agent logic):
1. crates/engine/src/command.rs: add `AgentCommandSpec { Sync|Add|Remove|Lock|List|Clean(Agent*Spec) }`
   + `EngineCommand::Agent { spec }` + run_event_loop arm dispatching to engine.agent_*; Err→emit_setup_error.
2. crates/engine/src/event.rs: add `Event::AgentListed { list: AgentList }` + `Event::AgentEdited { outcome: AgentEditOutcome }`.
   WHY: agent_list emits only AgentRunStarted; the AgentList lives only in the typed return (CLI prints via
   render_agent_list). add/remove PREVIEW edit-outcome items are in NO event. GUI worker→UI is event-only, so
   emit these two at the tail of agent_list / agent_add / agent_remove (in the engine, before returning).
   sync/clean already emit AgentRunFinished{report}; lock emits AgentLockChecked{drift} — reuse those.
3. ⚠️ Adding Event variants may force a new arm in CLI print_event (cli/src/main.rs ~1099) — add no-op arms if
   the match is exhaustive. CLI typed-return render stays unchanged (additive events).

## GUI (crates/gui/src/main.rs, worker-thread model: EngineCommand/EngineEvent + try_recv drain):
- Screen::Agent variant + label() + nav-tab + match arm.
- EnvctlApp state: agent_verb tab, form inputs (config/scope/source/skills/mcps/commands CSV/git_ref/branch/
  sub_dir/apply/no_sync/no_verify/locked/update CSV/lock_check/list_kind), result holders (agent_list,
  agent_last_edit, agent_last_report, agent_lock_drift, agent_status). Init in new().
- PURE state→Spec builders (unit-testable, NO egui types): agent_{sync,add,remove,lock,list,clean}_spec(&self)
  -> Agent*Spec + agent_command(&self) -> EngineCommand::Agent. Mirror CLI field maps (cli/src/main.rs:986-1090).
- drain() arms: AgentRunFinished→report+status; AgentLockChecked→drift; AgentListed→agent_list;
  AgentEdited→edit; AgentAction→push_log.
- agent_screen(ui): 6 verb sub-tabs, per-verb controls, Preview/Apply gating (apply default FALSE; mirror
  add_repo two-button preview/apply main.rs:956), result render (list TableBuilder, edit items, report summary,
  lock drift). clean gets a WARN + Apply gate.

## Field maps (specs at engine/src/agent/mod.rs:115-196): same as CLI TASK-0014.
sync: config_path/scope_override/apply/lock_mode(locked,update). add: source+AgentSectionSel+git_ref/branch/
sub_dir/config/scope/apply/no_sync/no_verify/locked. remove: add minus no_verify. lock: config/scope/check/
upgrade_only/lock_mode. list: scope/kind. clean: scope/apply.

## Invariants: engine single non-printing lib (GUI builds Spec + renders only; 2 events are engine-emitted
transport); fail-closed apply=false default on sync/add/remove/clean; UI thread never blocks (worker + try_recv);
no new dep (no-c); one rustls ring-only.

## Verification: GUI needs system dev libs + display to RUN. Headless-provable = (a) pure state→Spec unit tests
(#[cfg(test)] mod agent_spec_tests, mirror cli agent_cmd_tests), (b) engine test that agent_list/add emit the
new events (drain EventSink rx). + cargo build -p envctl-gui + clippy (needs libs). If GUI build blocked by
missing system libs in this env → scope guardian to engine build/test + gui clippy --lib if possible + note it.

## Open Q: (1) recommend MOVING lock_mode_from into engine (AgentLockMode::from_flags) so CLI+GUI share one
source (stronger parity); CLI test moves with it. (2) EngineCommand::Agent large variant — command.rs already
has #[allow(clippy::large_enum_variant)]. (3) check CLI print_event exhaustiveness for the 2 new Events.
