# envctl rollback checkpoints

Generated at: `2026-07-05T04:32:27+00:00`
Status: `passed`

## Coverage

| capability | covered |
|---|---|
| checkpoint rows | yes |
| idempotent repeat | yes |
| rollback handles | yes |
| safe status transitions | yes |
| approval gated risky rollback | yes |
| event chain | yes |
| fail closed rejections | yes |

## Runtime smoke

- Run: `run-req026`
- Checkpoint rows: `2`
- Rollback handles: `2`
- Approval rows: `1`
- Event rows: `8`
- Safe rollback status: `succeeded`
- Risky rollback status after approval: `planned`
- Rejection cases: `3`

The smoke persists checkpoint boundaries into `envctl_migration_checkpoints`, creates rollback handles in `envctl_migration_rollbacks`, gates an R4 rollback through `envctl_migration_approvals`, records append-only run events for each mutation, and confirms blocked secret references, run/operation mismatches, and illegal rollback transitions fail closed.
