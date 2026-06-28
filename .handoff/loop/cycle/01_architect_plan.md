# TASK-0078 plan — cache-child manifest id validation

Date: 2026-06-28
Worktree: `/home/drdave/Desktop/meta/.worktrees/task-0078-cache-manifest-validation/envctl`
Branch: `task-0078-cache-manifest-validation`

## Verified baseline

- `origin/master` includes PR #368 (`35a542b`) gating `--migrate-cache-child NAME` on the existence of `manifest/components.d/cache-<component>.toml`.
- Gap found: existence alone lets an empty or unrelated manifest unlock a cache-child migration, which is weaker than the backlog/docs phrase "reviewed cache component manifest".
- Live state remains owner-supervised: as of the PR #368 evidence, current cache-child manifests are missing and no live cache-child apply has been performed.

## Design

Add a narrow manifest-content guard for named cache-child migration:

1. Derive the expected component id as `cache-$(cache_child_component_key NAME)`.
2. Parse only the hinted manifest enough to find a matching `[[component]]` table with `id = "<expected>"`.
3. For an existing manifest that does not declare the expected id:
   - dry-run reports a refusal and returns without moving state;
   - `--apply` fails closed before open-handle and target-collision checks;
   - source and target remain unchanged.
4. Preserve the already-approved path when the manifest declares the matching id.

## Runtime surface

- `scripts/audit-meta-local-paths.sh --migrate-cache-child NAME`
- `scripts/tests/test-meta-local-path-audit.sh`
- `ci/gates/meta-local-policy.sh`
- Live non-mutating `--migrate-cache-child .wasm-pack` evidence against the current real home.
