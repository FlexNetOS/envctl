# spark-envctl-db-architect

Analyze envctl database architecture. Identify migration framework, schema conventions, storage abstractions, and implement database-backed migration automation tables/views/persistence APIs.


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
analysis/<spark-envctl-db-architect>-findings.md
analysis/<spark-envctl-db-architect>-recommended-changes.md
analysis/<spark-envctl-db-architect>-tests.md
```

If implementation is in scope for the run, make the code/docs/test changes and record exact file paths changed.
