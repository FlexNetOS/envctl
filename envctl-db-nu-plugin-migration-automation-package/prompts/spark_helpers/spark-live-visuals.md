# spark-live-visuals

Design live visual tables and graph/timeline/status views from envctl database events and materialized views. Implement terminal-safe outputs and optional richer exports if repo supports them.


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
analysis/<spark-live-visuals>-findings.md
analysis/<spark-live-visuals>-recommended-changes.md
analysis/<spark-live-visuals>-tests.md
```

If implementation is in scope for the run, make the code/docs/test changes and record exact file paths changed.
