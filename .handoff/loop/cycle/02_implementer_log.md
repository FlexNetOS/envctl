# TASK-0078 reviewed starship cache-child manifest implementer log

## Red
Before adding the starship manifest, the live non-mutating probe with the manifest temporarily absent reported a dry-run refusal because `manifest/components.d/cache-starship.toml` was missing.

The first candidate, `zellij`, was rejected for this slice after live dry-run with a matching manifest reported an existing target collision at `/home/drdave/Desktop/meta/.local/cache/zellij`. No zellij manifest was committed and no state was moved.

## Change
- Added `manifest/components.d/cache-starship.toml` with the minimal reviewed component declaration for `cache-starship`.
- Added fixture coverage for a repo-reviewed `starship` cache child: dry-run reports would-move/would-link and leaves both source and target untouched.
- Did not change the migration engine and did not run any live apply migration.

## Green
- `bash -n scripts/audit-meta-local-paths.sh scripts/tests/test-meta-local-path-audit.sh`
- `bash scripts/tests/test-meta-local-path-audit.sh`
- `git diff --check`
- `bash ci/gates/loop-state.sh`
- `bash ci/gates/meta-local-policy.sh`
- `bash ci/gates/harness-scripts.sh`
- `bash ci/gates/p7.sh`

## Runtime
Live dry-run reports it would move `/home/drdave/.cache/starship` to `/home/drdave/Desktop/meta/.local/cache/starship` and link it back. Validation row for `.cache/starship` reports `manifest_exists=yes`, `manifest_declares_expected_id=yes`, `review-existing-cache-component-manifest-before-migration`, and empty `apply_command`. The source cache still exists, the meta cache target is absent, and the apply command count is zero.
