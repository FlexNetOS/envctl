# envctl database automation architecture

## Architecture principle

The migration is not a script. It is a database-backed, replayable operation graph.

## Core components

```text
Artifact Contract Registry
Target Descriptor Registry
Migration Recipe Registry
Run Manager
Operation Queue
Event Log
Evidence Store
Artifact Store
Approval Gate
Validation Ledger
Replay Engine
Rollback/Checkpoint Registry
nu_plugin Boundary
```

## Source of truth

envctl database owns durable state. Files may hold exported artifacts, but the DB records their identity, hashes, links, and status.

## Package adapter

External prompt packages are imported as versioned packages. Execution becomes operations/events; output becomes artifacts/evidence records.
