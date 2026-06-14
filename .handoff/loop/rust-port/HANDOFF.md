# HANDOFF — rust-port-merge + TASK-0014/0014b (kasetto → envctl, Epic C)
closed_utc: 2026-06-14 (session-relay-wrap-up, post-TASK-0014b)   branch: develop   worktree: create FRESH off origin/develop
cycle_budget: 3   cycles_total: 18   cycles_this_session: TASK-0014b GUI (1)
last_item: TASK-0014b GUI agent panel (PR #93 armed)
next_item: **OPTIONAL POLISH ONLY** — (a) close **S-15** live-net residue: add an engine fetch-DI seam (inject the HTTP getter into `materialize_source`) so the main→master retry is testable offline, flip S-15 [~]→[x]; (b) the 4 `--all-targets`-only `unnecessary_to_owned` lints in `crates/engine/tests/agent_sync_parity.rs` (NOT in CI's `--workspace` clippy — cheap cleanup). **Epic C is otherwise COMPLETE.**
orchestrator_phase: ⭐ EPIC C FRONT-END COMPLETE (engine → CLI → GUI, all parity-verified). parity 101 [x] / 1 [~] (S-15) / 13 [≠]
last_agent: invariant-guardian (TASK-0014b PASS — note closed) + orchestrator (recovered an API-dropped implementer)
gate_status: PASS (engine + cli + gui all green; gui 11 spec tests; engine agent_sync 8→10 with event-emission tests; clippy --workspace -D warnings / no-c / shape / enable / fmt green)   pr_url: #89/#90/#91 MERGED, #93 GUI armed

landed_this_session:
  - #93  gui: Agent panel {sync,add,remove,lock,list,clean} over Engine::agent_* — armed
  (prior sessions: #89 parity / #90 CLI / #91 CLI human-render fix — all MERGED)
decisions_and_dead_ends:
  - SUBAGENT API-DROP RECOVERY: a delegated implementer can die mid-run (ConnectionRefused). The
    TASK-0014b implementer finished the ENGINE half then dropped. Recovery = inspect git diff for what
    landed → verify the finished half (build+test) → delegate ONLY the remainder to a fresh agent.
    Salvage partial work, never restart. (ICM decisions-forge-loop.)
  - GUI parity = the SAME return-value-transport gap as the CLI, different surface: a verb whose result
    is in the RETURN (list→AgentList, add/remove→AgentEditOutcome) needs it to cross the front-end
    boundary. CLI = render the return in human mode (#91); GUI = engine EMITS it as an Event
    (AgentListed/AgentEdited) since the worker→UI channel is event-only. Closed the guardian's "no
    event-emission test" note BEFORE commit (agent_sync 8→10) — don't carry a known regression-class.
  - lock_mode_from MOVED into the engine (AgentLockMode::from_flags) — single source, CLI+GUI both call it.
  - Deep PR stacking under fast auto-merge is fragile → CONSOLIDATE >2-3 deep (cherry-pick onto fresh
    develop, one PR — did this for #85-#88 → #89).
  - S-15 is honest live-network residue (HTTPS-hardcoded, no fetch DI seam) — do NOT fake; close with a
    seam. Found+fixed one real downgrade (C-12 engine remote-config rejection) earlier in the arc.
icm_stored: decisions-forge-loop (×3), context-envctl, errors-resolved, decisions-envctl (recall on resume)
verify_on_resume: |
  git -C <fresh worktree off origin/develop> rev-parse --short origin/develop   # 850d504+ (#93 merge)
  cargo build -p envctl-engine -p envctl -p envctl-gui                          # all build
  cargo test -p envctl-engine -p envctl -p envctl-gui                           # engine/cli/gui green
  bash ci/gates/no-c.sh && bash ci/gates/shape.sh && bash ci/gates/enable.sh    # PASS
  target/debug/envctl agent --help                                             # 6 verbs (CLI)
resume_command: /forge-loop (S-15 seam or lint cleanup — OPTIONAL) — or /session-relay-resume from .handoff/loop/rust-port/HANDOFF.md

## ⭐ STATUS: the kasetto→envctl ABSORPTION is COMPLETE end-to-end — engine → CLI → GUI.
Epic C front-end DONE: parity (#89) + CLI (#90/#91) + GUI Agent panel (#93). All three front-ends drive
the IDENTICAL parity-verified `Engine::agent_*` API (CLI↔GUI Spec parity guardian-checked field-for-field).
parity ledger: 101 `[x]` / 1 `[~]` / 13 `[≠]`. The ONLY remaining items are OPTIONAL POLISH: close **S-15**
(`materialize_source` live main→master HTTP retry — CODE matches kasetto `src/source/mod.rs:93-100`
line-for-line; needs an engine fetch-DI seam to test offline; honest residue, never faked) and the 4
`--all-targets`-only lints in `crates/engine/tests/agent_sync_parity.rs` (not in CI's `--workspace` clippy).

## SESSION-2 (2026-06-14 successor, parity-verifier pass — 3 cycles, all PASS)
First landed session-1's stack (#80/#81/#82 all MERGED). Then:
- **Cycle 1 — leaves** (+6 [x]: XC-03, ST-01/02, P-01/02, CP-01; envctl renames verified). **PR #83**.
- **Cycle 2 — C-* sync engine** (+6 [x]: C-01..C-06) via NEW `crates/engine/tests/agent_sync_parity.rs` (+15). MCP additive/never-clobber + never-prune verified. **PR #84**.
- **Cycle 3 — C-* verbs** (+7 [x]: C-07/08/09/10/11/13/14) via NEW `crates/engine/tests/agent_command_parity.rs` (+22). **C-12 → [~]** (remote-config-reject GAP = **C-12-FIX**, a real no-downgrade engine fix — see parity-ledger top). **PR #85** stacked.
- parity 74→93 [x] (DONE-equiv 106/115). Engine tests 59→96. NEVER stubbed; the C-12 gap recorded, not hidden.

## ⚠️ REMAINING TO FULL DONE (9 rows, all network/engine residue + 1 fix)
2 `[ ]`: **M-22** (resolve_scope file-read fallback, engine path), **S-15** (main→master retry, network).
7 `[~]`: **S-07/S-12/S-13** (pub(crate)/network), **CFG-03** (remote http arm), **C-12** (engine remote-reject GAP).
Plan: ONE Engine/network integration cycle (exercise `Engine::agent_sync` materialize/download end-to-end)
closes S-07/S-15/CFG-03 + M-22 at once; **C-12-FIX** (make `resolve_local_config_path` return Result +
reject remote, edit.rs:352 + call sites :53,:151) + a `pub` test seam for S-12/S-13 closes the rest.
THEN **TASK-0014** = the 13 `[≠]` front-end (CLI `envctl agent {sync,add,remove,lock,list,clean}` + GUI;
thin adapters over the already-verified engine methods).

## ⚠️ PR-STACK — land in order, rebase each onto the prior
#80 MERGED. **#81 → #82 are a stack.** When #81 squash-merges to develop, rebase #82:
```
cd <cycle-3 worktree> && git fetch origin
git rebase --onto origin/develop <#81-tip-sha> task-0012-parity-pass-3   # drop the merged cycle-2 commit
git push --force-with-lease && gh pr merge 82 --auto --squash
```
(Same pattern already used to land #81 onto #80 — see commit history. The conflict is only the
shared loop_state.md / parity-ledger.md section lines; the test-file appends are non-overlapping.)

## 6 [~] residue (NOT faked — close together in ONE Engine integration cycle)
S-07 (tar-slip guard), S-12 (auth_env_inline_help), S-13 (http_fetch_auth_hint), S-15 (main→master
retry), CFG-03 (remote http arm), + M-24 `State`/L-03 `list_installed_*` design-folds. All are
`pub(crate)`/network-only/engine-folded — unreachable via the offline cross-crate public API. Close by
exercising `Engine::agent_sync` end-to-end (it drives materialize→download→merge→lock) in a
`crates/engine/tests/` integration test, OR add a `pub` test seam. Do NOT fake a passing vector.

**Resume with:** `/forge-loop resume the /rust-port-merge` (or `/harness:rust-port-merge`). State lives in
`.handoff/loop/rust-port/` (namespaced — NOT the flat `.handoff/loop/`, the forge-loop's). On resume:
first land the #81→#82 stack, then RESET cycles_this_session to 0 and pick the next cluster (C-*).

## Where it stands (all on origin/develop)
The kasetto absorption is **structurally COMPLETE through the Engine**. `crates/agent-env` = 18-module
pure-Rust port of **pivoshenko/kasetto v3.2.0**; `crates/engine/src/agent/*` = the 6 `Engine::agent_*`
methods. **merge-ledger: 102 `[~]` merged / 0 `[ ]` to-merge / 13 `[≠]` front-end / 22 `[x]` parity-verified.**

landed_this_session:
  - PR #71  agent-env seed + model/* port
  - PR #72  rust-port-merge harness eject + verify-merge classification
  - PR #73  MCP additive-never-clobber merge (MC-01/MC-02) — left-behind sweep catch
  - PR #75  command transforms (PR-01) + config_edit (FE-*) + fsops resolution
  - PR #76  source discovery + runtime/profile/config_path/sync — LIBRARY COMPLETE
  - PR #78  engine wiring — Engine::agent_{sync,add,remove,lock,list,clean} (TASK-0013)
  - (peer) meta PR #31 retarget kasetto repo; kasetto fork origin/main force-synced to v3.2.0
  - #74 was a duplicate of #71 (closed by owner — work already on develop)

## next_item — two tracks (either order)
1. **Parity-verifier pass:** drive the 80 remaining `[~]` → `[x]` by extending
   `crates/agent-env/tests/parity_vs_kasetto.rs` (golden vectors VERBATIM from kasetto v3.2.0's own
   `#[cfg(test)]` modules — `cargo test` in meta/kasetto @ ec01cca passes 216 of them). 22 done.
2. **TASK-0014 (front-end, the 13 `[≠]`):** CLI verbs `envctl agent {sync,add,remove,lock,list,clean}`
   (clap) + GUI parity. THIN ADAPTER over the engine methods — build `Agent*Spec`, call
   `Engine::agent_*`, drain the `EventSink` to render the tree/grid, map `report.summary.failed>0` →
   exit code, `--json` serializes the (already `Serialize`) return. The engine API was designed for this.

## findings / decisions_and_dead_ends (don't re-litigate)
- **Source of truth = pivoshenko/kasetto v3.2.0** (the `upstream` remote), NOT the FlexNetOS fork (was
  v3.0.0+divergent). Fork renamed env_manager_agent→FlexNetOS/kasetto, origin/main force-synced to
  v3.2.0; pre-sync divergence preserved (branch `flexnetos-divergence-backup-2026-06-13` + git bundle
  in `meta/.archives/`). Do NOT downgrade meta/kasetto.
- **Two locks are DELIBERATELY separate:** engine FNV-1a component lock (`crates/engine/src/lock.rs`,
  `envctl.lock`) vs agent-asset SHA-256 lock (`agent_env::lock`, `agent-env.lock`). Do NOT unify — that's
  the later TASK-0017. `crate::agent::lock` never imports `crate::lock`.
- **Engine is non-printing:** kasetto's `print_*`/`ui.rs`/`process::exit` are DROPPED (FRONTEND-04
  `[≠]`); agent verbs emit `Event::Agent*` + return `Serialize` data; front-end maps failed>0→exit.
- **Preview-default fail-closed:** `apply:false` = ZERO writes; `lock_mode=Locked` = zero-network; MCP
  merge additive never-clobber (broker/repowire/weave survive); never-prune-on-failure (remove_stale
  only when summary.failed==0). These are guardian-tested — keep them.
- **PR workflow (IMPORTANT):** auto-merge + fast CI squash-merges a PR in ~1-2 min, deleting its branch;
  a later push recreates it diverged. WORK ONE PR PER CYCLE OFF FRESH origin/develop. Recover a diverged
  branch via fresh-branch + cherry-pick of ONLY the net-new commits (`git diff origin/develop...branch
  --stat`), never deleting concurrent sibling files.
- **never-discard (owner directive):** stale/orphaned/uncommitted work is INCOMPLETE work to complete +
  carry forward, never `git restore`/delete as drift (the parity harness was carried forward this way).

icm_stored: context-envctl, errors-resolved, decisions-envctl (recall these on resume)

## verify_on_resume (run FIRST to confirm green)
```
cd <fresh worktree off origin/develop>
rtk proxy cargo test -p envctl-agent-env            # expect ~226 + parity_vs_kasetto (12)
rtk proxy cargo test -p envctl-engine               # expect agent_sync integration tests pass
bash ci/gates/no-c.sh && bash ci/gates/shape.sh && bash ci/gates/enable.sh   # all PASS
```
Red → write `.handoff/loop/rust-port/NEEDS-HUMAN` and stop.

resume_command: /harness:rust-port-merge   (or /feature-forge for TASK-0014)
