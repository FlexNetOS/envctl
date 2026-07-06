# GitHub issue prompt — envctl database migration automation

## Title

Add database-backed migration automation engine with artifact contracts, replay, approvals, and agent control

## Problem

envctl needs first-class migration automation. The current migration prompt package can generate a rich migration artifact set, but execution is not yet durable, queryable, replayable, or safely controllable by agents through the envctl database.

## Goal

Build envctl database features that make migration execution reproducible and auditable:

- target descriptors
- migration recipes
- artifact contract registry
- run/event/operation ledger
- evidence/artifact store
- approvals
- checkpoints and rollback metadata
- validation ledger
- replay verification
- package adapter for `codex-flexnetos-migration-prompt-package`
- APIs/CLI commands consumed by `nu_plugin`

## Design direction

Use contract-first parallel implementation. The FlexNetOS package becomes the first imported artifact contract and regression fixture. envctl remains the source of truth. `nu_plugin` is a visualization/control surface.

## Acceptance criteria

- Parse target descriptor.
- Import FlexNetOS package contract.
- Create migration run.
- Append operation events.
- Persist evidence/artifact hashes.
- Gate risky operations behind approvals.
- Expose run status/events/artifacts/approvals/validations/replay to `nu_plugin`.
- Add tests for schema, run lifecycle, event append, approval, artifact ingestion, and replay.

## Non-goals

- No production-destructive actions by default.
- No plugin-owned migration state.
- No one-off FlexNetOS-only implementation.
