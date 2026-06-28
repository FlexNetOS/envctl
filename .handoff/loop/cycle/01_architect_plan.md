# TASK-0078 reviewed .wasm-pack cache-child manifest plan

## Target
Commit the first reviewed cache-child component manifest for live `.wasm-pack` so the existing narrow `--migrate-cache-child .wasm-pack` path can pass its reviewed-manifest precondition in dry-run mode.

## Contract
- Add `manifest/components.d/cache-wasm-pack.toml` declaring `[[component]] id = "cache-wasm-pack"`.
- Do not move cache state and do not run `--migrate-cache-child --apply`.
- Preserve the reviewed manifest gate: absent/wrong manifests still refuse; matching manifests only allow the existing dry-run/apply path to continue to normal safety checks.
- Add a regression fixture proving a repo-reviewed `.wasm-pack` manifest changes the dry-run from missing-manifest refusal to would-move/would-link without touching source or target.

## Runtime surface
`runtime_verifiable? yes` — run the live audit in dry-run mode and prove `/home/drdave/.cache/.wasm-pack` remains in place while `/home/drdave/Desktop/meta/.local/cache/.wasm-pack` is absent.

## Verification plan
- Red: before the manifest, live dry-run refuses because `manifest/components.d/cache-wasm-pack.toml` is missing; focused fixture fails.
- Green: add only the manifest and the fixture; run audit tests and harness gates.
- Runtime: live dry-run emits would-move/would-link plus validation `manifest_exists=yes` / `manifest_declares_expected_id=yes` / empty `apply_command`.
