# GitHub issue prompt — nu_plugin envctl migration control and visuals

## Title

Add Nushell plugin commands for envctl migration status, live visuals, approvals, artifacts, graph views, and replay

## Problem

Human operators and agents need a structured CLI surface for envctl migration automation. The plugin must provide live visuals and control without becoming a second state store.

## Goal

Add `nu_plugin` commands that expose envctl migration data as Nushell-native records/tables and controlled mutating operations:

- targets
- runs/status
- event timelines
- operation queues
- approval queues
- artifact index/open
- graph edges
- validation scorecards
- pause/resume
- approve/deny
- replay status
- rollback planning

## Design direction

The envctl database is the source of truth. The plugin talks through a shared protocol/CLI/API boundary. Every mutating command appends an envctl event.

## Acceptance criteria

- Commands register according to the actual nu_plugin protocol/crate version in the repo.
- Read-only commands return structured tables/records.
- Mutating commands call envctl boundary and append events.
- Approval commands affect envctl operation state.
- Status/timeline/artifact/graph/validation/replay views render test data.
- Failure states are structured and actionable.
- Tests cover command signature/output/error cases.

## Non-goals

- No direct plugin-owned migration database.
- No hidden state transitions.
- No bypassing envctl approvals.
