# Live visuals and human involvement

## Human involvement modes

| Mode | Meaning | Allowed behavior |
|---|---|---|
| `observer` | Human watches only. | Agents execute within safe policy; visuals update live. |
| `approval-gated` | Human approves risky operations. | R3+ operations wait for approval. |
| `operator` | Human can steer execution. | Human can pause, resume, deny, retry, or choose branches. |
| `agent-only` | No live human involvement inside pre-approved policy. | Only R0-R2 unless policy explicitly allows more. |

## Visual surfaces

The plugin must expose:

- Run overview.
- Phase progress.
- Operation queue.
- Event timeline.
- Open approvals.
- Artifact index.
- Evidence register.
- Dependency graph edges.
- Validation scorecard.
- Replay readiness.
- Rollback readiness.

## Visual correctness rule

The plugin may format and summarize, but every visual must derive from envctl database records. No plugin-only inferred state should be treated as truth.

## Minimum live table set

```text
migration status
migration timeline
migration ops
migration approvals
migration artifacts
migration graph
migration validations
migration replay status
```

## Optional richer visuals

- Mermaid graph export.
- DOT graph export.
- TUI dashboard if existing repo supports it.
- Streaming event subscription if existing architecture supports it.
- Static HTML/wiki export if existing artifact tooling supports it.
