# TASK-0078 cache-child manifest writer plan

## Target
Add an explicit owner-reviewed materialization path for deterministic cache-child component manifest stubs, without migrating cache data.

## Contract
New flag: `--write-cache-child-component-manifest NAME`.

- `NAME` must be one direct child of real-home `.cache`.
- Dry-run by default prints the manifest write; `--apply` writes only `manifest/components.d/cache-<component>.toml`.
- Written TOML is deterministic and matches the scaffold report's minimal component stub.
- Existing matching manifests are OK/no-op; existing wrong/non-regular manifests fail closed and are never overwritten.
- Missing sources, external symlink sources, non-directory sources, and invalid/path-like names fail closed.
- It never moves cache data and preserves `--migrate-cache-child`'s reviewed-manifest precondition. Writer execution intentionally follows migration attempts, so one invocation cannot write and immediately migrate without review.

## Runtime surface
`runtime_verifiable? yes` — drive the shell script against live `.wasm-pack` in dry-run mode and prove no manifest file is created.

## Verification plan
- Red: focused test/probe fails because the flag is unknown.
- Green: fixture tests for dry-run, apply write, idempotent existing manifest, wrong-manifest refusal, invalid name, and missing source.
- Runtime: live dry-run for `.wasm-pack`, validation report unchanged/non-mutating.
