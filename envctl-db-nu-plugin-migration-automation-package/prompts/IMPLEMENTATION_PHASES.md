# Implementation phases

## Phase A — Discovery

- Inspect envctl repo layout.
- Inspect nu_plugin repo layout.
- Identify database/migration framework.
- Identify CLI/API architecture.
- Identify plugin protocol/crate versions.
- Identify test commands.

## Phase B — Contract and schema lock

- Add shared JSON schemas/types.
- Add target descriptor schema.
- Add migration recipe schema.
- Add artifact contract registry format.
- Add envctl database migration tables/views.

## Phase C — envctl package adapter MVP

- Inspect prior package manifest.
- Import package as artifact contract.
- Create run from target descriptor + recipe.
- Run/import external package output as operations/events/artifacts.
- Store evidence and artifact hashes.

## Phase D — nu_plugin read-only MVP

- List targets/runs/status/events/artifacts/validations.
- Render structured tables.
- Handle envctl unavailable/error states.

## Phase E — approval and control

- Add approval queue.
- Add approve/deny commands.
- Add pause/resume/cancel where safe.
- Add risk gates.

## Phase F — replay and rollback

- Implement replay verification.
- Implement rollback planning metadata.
- Approval-gate destructive rollback execution.

## Phase G — native collectors and visuals

- Replace shell helper behavior with native collectors where practical.
- Add graph exports and richer dashboards.

## Phase H — test and issue integration

- Add tests.
- Run checks.
- Generate issue text and PR-ready summary.
