# spark-security-reproducibility

Design approval gates, risk classes, redaction, evidence hashing, event hash chains, tool version capture, and replay verification.


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
analysis/<spark-security-reproducibility>-findings.md
analysis/<spark-security-reproducibility>-recommended-changes.md
analysis/<spark-security-reproducibility>-tests.md
```

If implementation is in scope for the run, make the code/docs/test changes and record exact file paths changed.
