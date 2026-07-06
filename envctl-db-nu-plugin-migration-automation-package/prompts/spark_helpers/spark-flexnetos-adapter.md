# spark-flexnetos-adapter

Map codex-flexnetos-migration-prompt-package into envctl package import/execution records. Ensure FlexNetOS is a fixture, not hardcoded.


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
analysis/<spark-flexnetos-adapter>-findings.md
analysis/<spark-flexnetos-adapter>-recommended-changes.md
analysis/<spark-flexnetos-adapter>-tests.md
```

If implementation is in scope for the run, make the code/docs/test changes and record exact file paths changed.
