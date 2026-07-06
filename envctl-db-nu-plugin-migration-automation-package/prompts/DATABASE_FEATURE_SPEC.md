# envctl database feature specification

## Product objective

Make migration execution a first-class, database-backed envctl capability that agents can control safely from CLI workflows and humans can observe/intervene in through live visuals.

## Database responsibilities

The envctl database must persist:

- target descriptors
- artifact contracts
- migration recipes
- migration runs
- operation state machine
- append-only events
- evidence references and hashes
- generated artifacts and link graph
- dependency graphs and data flow graphs
- approval requests and decisions
- validation results
- checkpoints, snapshots, and rollback metadata
- tool versions and environment facts
- agent sessions and human sessions
- plugin sessions
- package imports and package execution results

## State machine

Run statuses:

```text
created -> planning -> awaiting_approval -> running -> paused -> validating -> completed
                         ↓                 ↓         ↓          ↓
                      denied            blocked   failed     cancelled
```

Operation statuses:

```text
queued -> ready -> awaiting_approval -> running -> succeeded
                         ↓                 ↓           ↓
                       denied           blocked      failed
```

Every status transition must append a run event.

## Event envelope

Every event must include:

```json
{
  "run_id": "...",
  "event_seq": 123,
  "event_type": "operation.started",
  "phase": "discovery",
  "actor_type": "agent|human|system|plugin",
  "actor_id": "...",
  "operation_id": "...",
  "timestamp_utc": "...",
  "payload": {},
  "evidence_refs": [],
  "previous_event_hash": "...",
  "event_hash": "..."
}
```

## Materialized views / query surfaces

Codex must implement repo-native equivalents for:

- latest run status
- timeline/events
- operation queue
- open approvals
- artifact index
- evidence register
- dependency graph edges
- validation scorecard
- replay plan

## Artifact contract registry

Artifact contracts are versioned. A run must reference exactly one artifact contract version and one recipe version. Contract changes create new versions, not silent edits.

## Replay

Replay must verify:

- target descriptor hash
- recipe hash
- artifact contract hash
- package hash
- tool versions
- command hashes
- evidence hashes
- artifact hashes
- approval decisions

Replay may be `verify-only`, `dry-run-plan`, or `execute-again`. Destructive replay requires explicit approval.

## Agent control

Agents may create plans, execute safe read-only operations, synthesize artifacts, run validations, and request approvals. Agents may not silently perform destructive operations, mutate production targets, or bypass safety policy.
