# TASK-0078 reviewed starship cache-child manifest guardian report

## Status
PASS — PR TBD.

## Invariants checked
- The slice commits only a reviewed component manifest and a regression test.
- No live cache data was moved; no `--migrate-cache-child --apply` was run.
- `--migrate-cache-child` remains narrow, dry-run by default, and gated by matching component manifest content.
- Owner-supervised planning/validation rows still carry empty `apply_command`.
- The broad `.cache` root remains owner-supervised/component-managed rather than auto-migrated.
- `zellij` was not committed after live dry-run found an existing meta target; the chosen `starship` candidate has live source present and meta target absent.

## Verification
- `bash -n scripts/audit-meta-local-paths.sh scripts/tests/test-meta-local-path-audit.sh`
- `bash scripts/tests/test-meta-local-path-audit.sh`
- `git diff --check`
- `bash ci/gates/loop-state.sh`
- `bash ci/gates/meta-local-policy.sh`
- `bash ci/gates/harness-scripts.sh`
- `bash ci/gates/p7.sh`

## Runtime check
Live non-mutating `starship` dry-run emitted `DRY-RUN: would move /home/drdave/.cache/starship to /home/drdave/Desktop/meta/.local/cache/starship and link ...`, `changed=0`, validation row `manifest_exists=yes` and `manifest_declares_expected_id=yes`, and the live state guards confirmed the source cache still exists while the meta cache target is absent. Apply command count was 0.
