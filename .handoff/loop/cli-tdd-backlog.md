# CLI Autonomous TDD Loop Backlog

Status legend: `[ ]` todo · `[~]` in-flight · `[x]` done · `[!]` blocked.

## Loop contract

Each cycle picks one CLI contract gap, writes the failing test first, implements the minimum Rust-native fix, runs focused tests, then broadens to the CLI crate and workspace gates. The loop never marks a cycle done until the PR is confirmed merged; supervised/live-hardware actions become blocked items instead of stopping the whole loop.

## Backlog

- [x] CLI-TDD-0001: Add hermetic CLI contract tests for root surface, manifest-independent commands, fail-closed reset refusal, and `lock --check --json` truth.
  - Red proof: `lock_check_json_reports_clean_lock_as_locked_and_drift_empty` failed with `{ "locked": false, "drift": [] }` after writing a fresh lock.
  - Green fix: `lock --check --json` now reports `locked=true` when drift is empty.
- [ ] CLI-TDD-0002: Expand machine-readable JSON shape coverage for `auto-detect`, `doctor`, `graph`, and `registry` fixture paths.
- [ ] CLI-TDD-0003: Add `secret` command wrapper contract tests that assert frozen `secretctl` argv/JSON surfaces without requiring a live daemon.
- [ ] CLI-TDD-0004: Add dry-run/no-write contract tests for `install`, `auto-fix`, `reset <target>`, `add-repo`, and `self uninstall` against hermetic fixtures.
- [ ] CLI-TDD-0005: Add exit-code matrix coverage for unknown components, conflicting flags, missing required confirmation, and invalid enum values.
