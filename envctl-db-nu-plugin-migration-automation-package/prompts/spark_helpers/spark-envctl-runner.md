# spark-envctl-runner

Analyze envctl runner/CLI architecture. Design and implement operation state machine, event appends, package adapter, target descriptor execution, replay, checkpoints, and rollback metadata.


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
analysis/<spark-envctl-runner>-findings.md
analysis/<spark-envctl-runner>-recommended-changes.md
analysis/<spark-envctl-runner>-tests.md
```

If implementation is in scope for the run, make the code/docs/test changes and record exact file paths changed.
