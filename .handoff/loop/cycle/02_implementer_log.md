# TASK-0078 cache-child manifest writer implementer log

## Red
Added focused expectations for `--write-cache-child-component-manifest`; the first probe exited 2 with `unknown argument: --write-cache-child-component-manifest`.

## Change
- Added the new explicit writer flag and usage documentation.
- Split cache-child manifest stub generation into a TOML file-body helper plus escaped TSV helper so scaffold output stays stable while apply writes real TOML.
- Added fail-closed source/name/manifest validation before any write.
- Added dry-run/no-op and `--apply` materialization behavior for `manifest/components.d/cache-<component>.toml`.
- Ordered writer execution after migration attempts to preserve the migration precondition in combined invocations.
- Extended `scripts/tests/test-meta-local-path-audit.sh` with dry-run, apply, idempotence, wrong-manifest, invalid-name, and missing-source coverage.

## Green
- `bash -n scripts/audit-meta-local-paths.sh scripts/tests/test-meta-local-path-audit.sh`
- `bash scripts/tests/test-meta-local-path-audit.sh`
- `git diff --check`
- `bash ci/gates/meta-local-policy.sh`
- `bash ci/gates/harness-scripts.sh`
- `bash ci/gates/p7.sh`

## Runtime
Live dry-run for `.wasm-pack` reported it would write `manifest/components.d/cache-wasm-pack.toml`, created no manifest, emitted validation rows for 84 cache children with `bad_apply=0`, and moved no live cache state.
