# TASK-0078 managed config deep-diff summary plan

## Target

Add a read-only owner-supervised deep-diff summary report for managed `.config` child conflicts so
existing real-home state can be reviewed with aggregate counts before any managed-source bridge is
attempted.

## Contract

New flag: `--owner-supervised-managed-config-child-deep-diff-summary PATH`.

TSV columns:

```text
dot_entry child_name real_path managed_source real_type managed_type real_deep_entries managed_deep_entries real_deep_files managed_deep_files shared_deep_entries real_only_deep_entries managed_only_deep_entries type_conflict_deep_entries differing_files deep_identical supervision next_action apply_command
```

Rows are emitted only for direct `.config/<child>` candidates whose action is
`owner-supervised-config-child-bridge` and where both the real-home child and
`$ENVCTL_HOME_SOURCE/.config/<child>` exist. The report is aggregate-only: it may expose the direct
child name and source roots already present in earlier owner-supervised reports, but it must not emit
nested relative paths or file contents.

Routing:

- deep-identical trees: `deep_identical=yes`, `next_action=review-then-bridge-identical-managed-config-child`
- differing/type-conflict/missing-entry trees: `deep_identical=no`,
  `next_action=owner-review-real-home-config-child-deep-diff-before-bridge`
- `supervision=owner-reviewed`
- `apply_command` stays empty; report-only, no bridge or archive mutation.

## Verification plan

- Red: focused fixture test fails because the new flag is unknown.
- Green: lock the 19-column schema plus identical and differing fixture branches, including
  real-only, managed-only, type-conflict, and differing shared-file counts.
- Runtime: live non-mutating audit should emit the current managed-config child conflict rows with
  empty apply commands and review-only next actions.
