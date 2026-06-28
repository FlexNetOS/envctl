# TASK-0078 reviewed JNA cache-child manifest guardian report

## Status
PASS — reviewed manifest and fixture added; local gates and live non-mutating runtime dry-run passed. PR #379 created; merge pending.

## Invariants to check
- The slice commits only a reviewed component manifest and a regression test.
- No live cache data is moved; no `--migrate-cache-child --apply` is run.
- `--migrate-cache-child` remains narrow, dry-run by default, and gated by matching component manifest content.
- Owner-supervised planning/validation rows still carry empty `apply_command`.
- The broad `.cache` root remains owner-supervised/component-managed rather than auto-migrated.
- The chosen `JNA` candidate has live source present and meta target absent.

## Verification
PASS:

- `bash -n scripts/audit-meta-local-paths.sh scripts/tests/test-meta-local-path-audit.sh`
- `bash scripts/tests/test-meta-local-path-audit.sh` (`test-meta-local-path-audit: PASS`)
- `git diff --check`
- `bash ci/gates/loop-state.sh` (`LOOP-STATE GATE PASS`)
- `bash ci/gates/meta-local-policy.sh`
- `bash ci/gates/harness-scripts.sh` (`HARNESS-SCRIPTS GATE PASS`)
- `bash ci/gates/p7.sh` (`P7 GATE PASS`)

## Runtime check
PASS. Live dry-run with `--migrate-cache-child JNA` reported the expected would-move/would-link plan from `/home/drdave/.cache/JNA` to `/home/drdave/Desktop/meta/.local/cache/JNA`, `changed=0`, and a validation row with `manifest_exists=yes`, `manifest_declares_expected_id=yes`, `next_action=review-existing-cache-component-manifest-before-migration`, and an empty `apply_command`. Source remains present; target remains absent; no `--apply` migration was run.
