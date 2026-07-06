# nu_plugin contract

## Role

`nu_plugin` is the operator shell and visual surface for envctl migration automation. It does not own durable migration state. It talks to envctl through a controlled CLI/API/shared library boundary.

## Required command capabilities

Codex must adapt command names to the repo's existing conventions, but the functional surface must include:

```text
envctl migration targets
envctl migration target add
envctl migration packages
envctl migration package inspect
envctl migration package import
envctl migration plan
envctl migration run
envctl migration status
envctl migration events
envctl migration timeline
envctl migration ops
envctl migration approvals
envctl migration approve
envctl migration deny
envctl migration pause
envctl migration resume
envctl migration artifacts
envctl migration artifact open
envctl migration graph
envctl migration validations
envctl migration replay
envctl migration rollback plan
envctl migration export
```

## Structured outputs

Commands must return Nushell-native structured records/tables. Examples of output columns:

### `migration status`

```text
run_id | target | status | phase | percent | open_approvals | failed_ops | artifact_count | last_event_at
```

### `migration events`

```text
seq | time | phase | event_type | actor | operation | status | summary
```

### `migration approvals`

```text
approval_id | run_id | operation | risk | requested_by | status | reason | created_at
```

### `migration graph`

```text
from | to | edge_type | source_artifact | confidence | evidence
```

## Mutating command rule

Every mutating plugin command must append an event through envctl. The plugin must not write directly to arbitrary database tables unless envctl's architecture explicitly defines the plugin as an authorized DB client with transaction APIs.

## Live visuals

The plugin should support:

- status tables
- timelines
- operation queue views
- approval queue views
- artifact index views
- graph edge views
- validation scorecards
- replay plan summaries

Visuals must degrade gracefully in plain terminal mode.

## Failure behavior

If envctl is unavailable, the plugin returns a structured error. If a run is blocked, the plugin displays the blocker and the next safe action. If an approval is required, the plugin must not proceed automatically.
