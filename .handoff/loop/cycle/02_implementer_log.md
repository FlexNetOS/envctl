STATUS: GREEN

# 02 — Implementer log: engine-owned doctor and manifest-lock proof

## Delivered

- Added the typed, non-printing engine doctor in `crates/engine/src/doctor.rs`:
  `DoctorSpec`, `Status`, `Summary`, `PathState`, `PathCheck`, `ToolCheck`,
  `ManifestLockStatus`, `ManifestLockReport`, and `DoctorReport`.
- Added `Engine::doctor` and `Engine::manifest_lock_check`, `Event::Doctored`, and
  `EngineCommand::Doctor`. Each doctor run emits exactly one typed event.
- Root priority is explicit `--root` → `META_ROOT` → upward `.meta.yaml` → managed-worktree
  owner normalization. Missing and ambiguous roots return a typed error report. There is no
  `~/Desktop/meta` fallback.
- Replaced write/delete probe files with metadata plus `access(2)` checks. Missing canonical
  directories are classified `Creatable` only when the nearest existing directory proves write
  access. EFI Secure Boot and NVIDIA driver state are read directly from existing kernel files.
- Replaced the CLI-local doctor implementation with pure rendering of `DoctorReport`. JSON is
  emitted before exit 1, and only `Status::Error` is unhealthy. The separate agent doctor was
  not changed.
- Added a top-level GUI Doctor screen driven by the same `EngineCommand::Doctor` and
  `DoctorReport`; it contains no duplicate probe/decision logic.
- Added `ci/gates/manifest-lock.sh`, wired it into CI, and added a hermetic mutation-detection
  regression test. The gate hashes tracked manifest inputs before/after and runs exactly
  `cargo run --locked -p envctl -- --color never lock --check`.
- Reconciled `manifest/envctl.lock` after source-history review: only the intended
  `codex-global-baseline` hash update (#481) and `postgres-ruvector` row (#470) changed.
- Archived the unrelated prior Blueprint Feature Forge cycle under `.handoff/loop/_done/`.

## TDD evidence

- `cargo test -p envctl-engine doctor --locked`: 14 passed.
- `cargo test -p envctl --test cli_contract doctor --locked`: 4 passed.
- `cargo test -p envctl-gui top_level_doctor --locked`: 2 passed.
- `cargo test -p envctl-engine -p envctl -p envctl-gui --locked`: all package unit,
  integration, parity, and doc tests passed (engine 161, CLI contract 15, GUI 27, plus the
  remaining package suites).
- `cargo clippy -p envctl-engine -p envctl -p envctl-gui --all-targets --locked -- -D warnings`:
  passed.
- `cargo fmt --all -- --check`: passed.
- `bash ci/gates/{no-c,shape,enable,manifest-lock,actionlint}.sh`: passed.
- `bash scripts/tests/test-manifest-lock-gate.sh`: passed, including the intentionally mutating
  fake-cargo refusal case.

## Runtime observation

The real built surface was driven with:

`envctl --json --color never doctor --root /home/flexnetos/meta`

It emitted valid JSON and then exited 1 with 57 OK / 2 warnings / 1 error. The lock report was
clean. The sole error is the independently confirmed stale boundary policy that currently marks
the one-profile Yazelix Nix-store frontdoors (`meta`, `icm`, `grit`, `weave`, etc.) as foreign.
That is an integration dependency on the parallel profile-ownership repair, not a doctor defect:
the new doctor correctly fails closed and names the exact violations. After that detector policy
lands, this same runtime check must be rerun and should exit 0 with warnings.

## Invariants

- No new crate dependency and no C trust-boundary change; only the already-resolved `rustix`
  dependency gained its pure-Rust `fs` API feature for `access(2)`.
- No generated home state, active Nix profile, main profile worktree, or user/global wrapper was
  modified.
- No commit or push was made.
