# spark-test-validation

Find test harnesses and add tests for schema, operation lifecycle, plugin commands, event append, approval, artifact import, validation, and replay.


## Shared constraints

- Inspect real repo files before proposing edits.
- Do not invent architecture.
- Mark unknowns explicitly.
- Preserve envctl as source of truth.
- Preserve nu_plugin as CLI/control/visual surface.
- No destructive actions without approval.
- Produce concrete files/functions/tests to change.


Required output:

```text
analysis/<spark-test-validation>-findings.md
analysis/<spark-test-validation>-recommended-changes.md
analysis/<spark-test-validation>-tests.md
```

If implementation is in scope for the run, make the code/docs/test changes and record exact file paths changed.
