# Verification report: FULL kasetto v3.2.0 CLI/GUI option parity (TASK-0019)

## Verdict — **PASS**

All four CI gates, cargo fmt/clippy/test, and the GUI build pass; every NON-NEGOTIABLE
invariant holds against the actual delivered code; the 3 no-downgrade spot-checks confirm the
ports are faithful to kasetto v3.2.0. The implementer's GREEN is corroborated by independent
evidence. No blocking findings.

## Gate results (exit codes pasted)
| Gate | Result | Evidence |
|------|--------|----------|
| `ci/gates/no-c.sh` | **PASS** exit=0 | `rustls=['0.23.40'] on ring=['0.17.14']; zero aws-lc/openssl/C-SQLite` |
| `ci/gates/shape.sh` | **PASS** exit=0 | `SHAPE GATE PASS` |
| `ci/gates/enable.sh` | **PASS** exit=0 | `ENABLE GATE PASS` |
| `ci/gates/p7.sh` | **PASS** exit=0 | `P7 GATE PASS` |

## cargo
| Check | Result | Evidence |
|-------|--------|----------|
| `cargo fmt --all --check` | **PASS** exit=0 | clean |
| `cargo clippy --workspace -- -D warnings` | **PASS** exit=0 | full workspace incl. GUI, `Finished` 0 warnings |
| `cargo test -p envctl-engine -p envctl -p envctl-agent-env` | **PASS** exit=0 | see summary below |
| `cargo build -p envctl-gui` | **PASS** exit=0 | `Finished dev profile`, no system-lib block |

Test summary (passed/failed lines): engine lib `58 passed; 0 failed`; envctl bin/lib
`19 passed; 0 failed` (+ bin/integration suites `13/4/24/12/15/20` all 0 failed); agent-env
`251 passed; 0 failed; 1 ignored` (+ `82 passed; 0 failed`). **Total: 0 failed across all suites.**

## Invariant checks
1. **No C in trust boundary** — PASS. Resolved graph (independently probed): exactly one `rustls
   v0.23.40` on `ring v0.17.14`; zero `aws-lc-rs`; `flate2 v1.1.9` backed by pure-Rust
   `miniz_oxide v0.8.9` (no `libz-sys`); `tar` pure-Rust. no-c.sh PASS.
2. **clap_complete is the only new crate + pure-Rust** — PASS. `cargo tree -p clap_complete`
   subtree = clap/clap_builder/clap_lex/anstyle/syn/proc-macro2 only; grep for `-sys|openssl|
   aws-lc|libsqlite|cc` → none. Cargo.toml:35 `clap_complete = "4.5"` (resolves 4.6.5).
3. **Engine non-printing / sync / pure-Rust** — PASS. Grep of the 4 new engine modules
   (`self_update.rs`, `self_uninstall.rs`, `update_notifier.rs`, `agent/doctor.rs`) for
   `println!/eprintln!/eprint!/print!/stdout/stderr` → ZERO matches. Decision logic confirmed in
   the engine returning typed data: `is_newer`/`verify_checksum`/`plan_self_update`
   (self_update.rs), `cache_is_fresh`/`available_update` (update_notifier.rs), doctor assembly +
   `Event::AgentDoctored` (agent/doctor.rs), uninstall removal decision + `Event::SelfUninstall`
   (self_uninstall.rs). The only engine printlns in the tree are the **pre-existing**
   `addrepo.rs:389-402` (interactive `--refactor=ai` guidance, last touched by an unrelated
   toolchain-pin commit `3a1219e`, NOT by TASK-0019) — not a regression.
4. **Destructive op fail-closed + dry-run default (item 5 self uninstall)** — PASS.
   `self_uninstall.rs`: `dry_run = !spec.apply` (L77); all `fs::remove_*` gated behind `if
   spec.apply` (L107) so no-flag ⇒ ZERO writes; binary-removal guard computes
   `current_exe()` file-stem ∈ {envctl, envctl-gui} BEFORE any write (L92-105) and refuses
   otherwise. CLI arm `run_self_uninstall` (main.rs:798-815): `apply && !yes && !stdin().is_terminal()`
   → errors "pass --yes to confirm uninstall in non-interactive mode"; TTY `[y/N]` otherwise.
   Refusal-path test `preview_writes_nothing_and_guard_refuses_non_envctl_binary`
   (self_uninstall.rs:252-293) asserts dry-run, zero config/data/cache/binary/gui removal, AND
   the guard refusing the non-envctl test-harness stem. Proven, not asserted.
5. **CLI+GUI parity (item 1 agent doctor)** — PASS (see Parity check). REQUIRED-parity item is
   genuinely dual-front-end.
6. **Rust-native, no drift** — PASS. Only `clap_complete` added as a new crate;
   reqwest/tar/flate2/sha2 are pre-existing workspace pins reused; no foreign-language file
   appeared. Accepted sole divergence = `baby-mimalloc` (Rust allocator) replacing kasetto's
   `mimalloc`(C) — upgrade-only, verified C-free.
7. **Lock honesty** — PASS. Implementer claims no envctl.lock/agent-env.lock/manifest change for
   TASK-0019; working-tree status confirms the only TASK-0019 mutations are Cargo.toml/Cargo.lock
   (clap_complete), the 5 new modules, and the wiring files — no lock-tracked component drift.
   (`manifest/envctl.lock` in `develop...HEAD` is from the branch's older base, not this task.)

## Parity check (Engine method → CLI caller / GUI caller)
- `Engine::agent_doctor` (engine `agent/doctor.rs:32`)
  - CLI: `crates/cli/src/main.rs:1540` `eng.agent_doctor(spec, &sink)` → `AgentResult::Doctor` →
    `render_agent_doctor` (main.rs:1251/1261).
  - GUI: `crates/gui/src/main.rs:1366` `AgentCommandSpec::Doctor(self.agent_doctor_spec())` →
    `command.rs:255` `engine.agent_doctor(s, &sink)`; `Event::AgentDoctored` handled at
    `gui/main.rs:444-455` (real handler — updates status + stores typed `agent_last_doctor` for
    `agent_doctor_tables` render). **Both front-ends drive the identical Engine method.**
- All other items (completions / self update+uninstall / notifier / global flags / --frozen) are
  CLI-only with the documented justifications (clap-tree introspection, self-replacing running
  binary, end-of-run terminal concept, terminal presentation) — accepted per plan §Invariants.

## No-downgrade spot-checks vs kasetto v3.2.0 (`meta/kasetto`)
- (a) `is_newer` semver compare — **MATCH (verbatim)**. Identical `(u64,u64,u64)` tuple parse
   (`split('.').filter_map(parse).collect`) and `parse(latest) > parse(current)` in both
   `kasetto/src/commands/self_update.rs:186` and `engine/src/self_update.rs`.
- (b) update_notifier suppression set + TTL — **MATCH**. `TTL_SECS = 24*60*60` and the
   `now - checked_at < TTL_SECS` freshness boundary are identical. `should_suppress_notice`:
   completions + self(`Manage`/`ManageSelf`) always suppressed, json/quiet suppressed, `Init`
   NOT suppressed — same intent; envctl additionally suppresses `Env` (envctl-specific
   machine-readable eval verb), a correct addition, not a downgrade.
- (c) agent doctor field set vs kasetto `DoctorOutput` — **MATCH (1:1)**. All 11 fields present
   and update_check INCLUDED: version, lock_file, scope, skills, installation_path, last_sync,
   failures, mcps, commands, command_dirs, update_check. Substructs `AgentCommandDirCheck{path,
   writable}` and `AgentUpdateCheck{status,latest_version,checked_at,age_seconds}` mirror
   kasetto `CommandDirCheck`/`UpdateCheckOutput`. (Also spot-checked `verify_checksum` — faithful
   port; asset names retargeted kasetto→envctl as designed.)

## Deviations reviewed — both ACCEPTABLE (not behavior downgrades)
- **lock `--check` carries only `frozen` alias** (not `locked`): envctl's `agent lock` already
   exposes a distinct real `--locked` zero-network flag that kasetto's Lock lacks; a `locked`
   alias on `--check` would collide in clap (`long option names must be unique`). Correct
   no-collision mapping for envctl's richer Lock surface. The other 3 flags (sync/add/remove)
   carry `visible_alias = "frozen"` (main.rs:412/454/484/499). No capability lost.
- **quiet/verbose/color via `OUTPUT: OnceLock<OutputCtx>`**: deliberate front-end-only
   presentation seam (engine still emits the full event stream non-printing). Failures/refusals
   are never suppressed under `--quiet`. No engine signature churn, no behavior change.

## Findings
None blocking. One informational note (carried, not a finding): the `develop...HEAD` diff is
large because the worktree is built on a feature-rich base ahead of the current `develop`; the
TASK-0019 delta itself is the uncommitted working-tree set, which matches the implementer log
exactly (5 new modules + the wiring files). Verification was performed against that working tree.

## Re-test needed
None. If any fix lands later, re-run (raw, via `rtk proxy`):
```
bash ci/gates/no-c.sh; echo exit=$?
rtk proxy cargo clippy --workspace -- -D warnings; echo exit=$?
rtk proxy cargo test -p envctl-engine -p envctl -p envctl-agent-env; echo exit=$?
rtk proxy cargo build -p envctl-gui; echo exit=$?
```
