# Envctl Agent Control API

Status: implemented and locally verified.

This control surface exposes database-backed operations for agents, helpers, plugins, and humans. It uses the existing envctl migration tables for runs, operations, approvals, and append-only events.

## CLI Commands

- `status RUN_ID`: returns run status, approval summary, operation counts, visible locks, recent events, and event-chain validation.
- `events RUN_ID`: returns the hash-chained event timeline.
- `enqueue RUN_ID OPERATION_TYPE`: creates an idempotent controlled operation. R0-R2 operations become ready; R3-R5 operations open an approval and enter `awaiting_approval`.
- `decision APPROVAL_ID`: approves or denies an open approval. Only `operator` and `admin` authority can decide.
- `transition OPERATION_ID`: applies the existing operation state machine. R3+ operations cannot start until approved.
- `lease RUN_ID` and `release-lease RUN_ID LEASE_ID`: record visible run-scoped target locks as append-only control events.

## Verification

- Library smoke status: `pass`
- Event chain valid: `True`
- CLI event count observed: `12`
- Proof record: `proof_records/REQ-028_ENVCTL_AGENT_CONTROL_API.proof.json`
