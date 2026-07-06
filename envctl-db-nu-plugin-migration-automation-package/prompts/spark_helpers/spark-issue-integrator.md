# spark-issue-integrator

Generate/update GitHub issue text and PR sequence for envctl, nu_plugin, and shared protocol based on actual repo findings.


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
analysis/<spark-issue-integrator>-findings.md
analysis/<spark-issue-integrator>-recommended-changes.md
analysis/<spark-issue-integrator>-tests.md
```

If implementation is in scope for the run, make the code/docs/test changes and record exact file paths changed.
