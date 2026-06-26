# CLI Autonomous TDD Loop Backlog

Status legend: `[ ]` todo · `[~]` in-flight · `[x]` done · `[!]` blocked.

## Loop contract

Each cycle picks one CLI contract gap, writes the failing test first, implements the minimum Rust-native fix, runs focused tests, then broadens to the CLI crate and workspace gates. The loop never marks a cycle done until the PR is confirmed merged; supervised/live-hardware actions become blocked items instead of stopping the whole loop.

## Backlog

- [x] CLI-TDD-0001: Add hermetic CLI contract tests for root surface, manifest-independent commands, fail-closed reset refusal, and `lock --check --json` truth.
  - Red proof: `lock_check_json_reports_clean_lock_as_locked_and_drift_empty` failed with `{ "locked": false, "drift": [] }` after writing a fresh lock.
  - Green fix: `lock --check --json` now reports `locked=true` when drift is empty.
- [x] CLI-TDD-0002: Expand machine-readable JSON shape coverage for `auto-detect`, `doctor`, `graph`, and `registry` fixture paths.

  - Green proof: `json_shapes_cover_detect_doctor_graph_and_registry` pins required JSON keys against hermetic manifest/hub fixtures.
- [x] CLI-TDD-0003: Add `secret` command wrapper contract tests that assert frozen `secretctl` argv/JSON surfaces without requiring a live daemon.

  - Green proof: `secret_wrapper_forwards_frozen_argv_without_live_daemon` uses a fake `secretctl` in the canonical `~/.local/bin` symlink-farm path and asserts the frozen `ca trust -> ca trust-apply` argv.
  - Upgrade: engine `resolve_secretctl` now prefers `$META_ROOT/.toolchains/secrets/bin` and `~/.local/bin` before legacy `~/.cargo/bin`.
- [x] CLI-TDD-0004: Add dry-run/no-write contract tests for `install`, `auto-fix`, `reset <target>`, `add-repo`, and `self uninstall` against hermetic fixtures.

  - Green proof: `mutating_verbs_preview_without_writing_fixture_state` snapshots fixture state around install/auto-fix/reset/add-repo/self-uninstall previews.
- [x] CLI-TDD-0005: Add exit-code matrix coverage for unknown components, conflicting flags, missing required confirmation, and invalid enum values.

## Code intelligence/index evidence

- `git-kb code doctor --json` reported 3,811 Rust symbols / 142 Rust files with deep Rust support.
- `git-kb code index` indexed 3,811 symbols from 551 files and 32,458 call sites.
- `git-kb code symbols --file crates/cli/src/main.rs --json` and `git-kb code impact crates/cli/src/main.rs --json` were used before extending the CLI contract tests.
