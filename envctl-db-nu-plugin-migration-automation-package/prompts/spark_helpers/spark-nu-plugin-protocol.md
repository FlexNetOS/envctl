# spark-nu-plugin-protocol

Analyze nu_plugin architecture and actual nu_plugin/Nushell versions. Design and implement command signatures, serialization, typed output records, envctl boundary, and structured errors.


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
analysis/<spark-nu-plugin-protocol>-findings.md
analysis/<spark-nu-plugin-protocol>-recommended-changes.md
analysis/<spark-nu-plugin-protocol>-tests.md
```

If implementation is in scope for the run, make the code/docs/test changes and record exact file paths changed.
