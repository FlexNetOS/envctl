# TASK-0078 cache-child manifest writer guardian report

## Status
PASS — PR #373.

## Invariants checked
- Dry-run remains default; mutation requires `--apply`.
- The new writer writes only a reviewed deterministic component manifest stub and never moves cache data.
- Invalid child names, missing sources, non-directory/external sources, and existing wrong/non-regular manifests fail closed.
- Existing matching manifests are idempotent no-ops.
- `--migrate-cache-child` still requires a reviewed matching manifest before migration; writer execution follows migration attempts to prevent same-invocation materialize-and-migrate.
- `apply_command` in owner-supervised reports remains empty for review/planning surfaces.

## Verification
- `bash -n scripts/audit-meta-local-paths.sh scripts/tests/test-meta-local-path-audit.sh`
- `bash scripts/tests/test-meta-local-path-audit.sh`
- `git diff --check`
- `bash ci/gates/meta-local-policy.sh`
- `bash ci/gates/harness-scripts.sh`
- `bash ci/gates/p7.sh`

## Runtime check
Live non-mutating `.wasm-pack` dry-run printed `would write manifest/components.d/cache-wasm-pack.toml declaring cache-wasm-pack`, produced validation rows for 84 cache children with `bad_apply=0`, and confirmed `manifest/components.d/cache-wasm-pack.toml` was not created.
