# 01 — Architect plan: engine-owned doctor and manifest-lock proof

The current top-level `envctl doctor` is a CLI-local implementation that writes and removes
probe files, resolves a retired `~/Desktop/meta` fallback, reports stale roots, and returns
success even when the report is unhealthy. The repair is engine-first and additive: preserve
the separate agent-env `envctl agent doctor` surface while replacing only the top-level doctor.

## Target repo and routing

One repo (`envctl`) with a linear dependency chain: engine types/behavior → CLI rendering and
exit semantics → GUI screen → non-mutating CI gate. Route sequentially through one implementer.

## Engine contract

- Add `crates/engine/src/doctor.rs` with typed, serializable `DoctorSpec`, `Status`, `Summary`,
  `PathState`, `PathCheck`, `ManifestLockStatus`, `ManifestLockReport`, and `DoctorReport`.
- `Engine::doctor(&DoctorSpec) -> Result<DoctorReport>` owns all health logic. It stays sync,
  pure-Rust, and non-printing.
- Root resolution is fail-closed and workspace-aware. Priority is explicit spec root, then
  `META_ROOT`, then upward `.meta.yaml` discovery, then managed-worktree normalization to the
  owning meta workspace. It must never fall back to `~/Desktop/meta`.
- Path checks use metadata/access observations only. No temporary probe file, directory, or
  cleanup mutation is allowed. EFI checks read the existing efivar filesystem directly.
- The report includes focused boundary/driver checks and a typed manifest-lock report.
- `Engine::manifest_lock_check` compares the declarative manifest against `envctl.lock` without
  mutation and returns `ManifestLockStatus` plus details.
- Add `Event::Doctored { report }` emitted exactly once per `Engine::doctor` call and
  `EngineCommand::Doctor { spec }` routed through the standard engine dispatcher.

## Front ends

- CLI top-level `doctor` only parses input, calls `Engine::doctor`, and renders the returned
  report. Valid JSON is written before an unhealthy report exits 1; only `Status::Error` makes
  the command unhealthy. Human output is a pure projection of the same report.
- Existing `envctl agent doctor` remains separate and unchanged.
- GUI gains a top-level Doctor screen that calls the same `Engine::doctor` method and consumes
  the same `DoctorReport`; no replicated health logic is permitted.

## Manifest-lock gate

- Add `ci/gates/manifest-lock.sh`. It hashes tracked inputs before and after and invokes
  `cargo run --locked -p envctl -- --color never lock --check`. It must fail closed on command
  failure or any mutation and must not update the lock.
- Wire the gate into CI beside the existing invariant gates.

## Tests

- Engine unit tests cover every root-resolution priority, managed-worktree normalization,
  missing/ambiguous-root refusal, status/summary aggregation, exactly-one event emission,
  manifest-lock typed states, and a before/after filesystem snapshot proving doctor does not
  mutate.
- CLI tests cover healthy exit 0, unhealthy exit 1 after parseable JSON, and no retired
  `Desktop/meta` fallback.
- GUI tests prove the Doctor screen consumes the engine-owned report and uses the same method.
- The manifest-lock gate gets a hermetic mutation check.

## Runtime surface

runtime_verifiable: yes. Build the real `envctl`, run `doctor --json` against a controlled
healthy and unhealthy root, parse the JSON, verify the respective exit codes, and prove no
filesystem entries changed. Drive `manifest-lock.sh` and the GUI Doctor screen where the native
GUI test harness permits.

## Invariants

No new dependency, no C trust-boundary change, no generated-home/profile mutation, no probe
files, no retired fallback, and no downgrade or bypass of the existing agent doctor.
