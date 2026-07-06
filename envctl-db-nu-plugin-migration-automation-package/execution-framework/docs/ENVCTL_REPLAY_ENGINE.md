# envctl replay engine

Generated at: `2026-07-05T04:43:15+00:00`
Status: `passed`

## Replay surface

- Reconstructs target descriptor, artifact contract, recipe, package manifest, operation inputs, proof/evidence hashes, artifact hashes, tool versions, approvals, checkpoints, and hash-chained run state.
- Produces dry-run and apply-mode replay results using `ReplayRequest` / `ReplayResult` compatible fields from the shared protocol schema.
- Fails closed on hash mismatches, blocked paths, open approvals, and non-deterministic operations.

## Runtime smoke

- Dry-run status: `pass`
- Apply status: `blocked`
- Mismatch status: `fail`
- Event chain valid: `True`
- Evidence matches: `4`
- Artifact matches: `1`
- Required approvals in apply replay: `1`
- Non-deterministic operations in apply replay: `1`

## Commands

```bash
envctl replay dry-run --run-id run-req027-replay --replay-id replay-req027-dry-run --requested-by helper-envctl-replay-01 --operation-ids op-req027-replay-hash --reason verify deterministic replay inputs before integration
envctl replay apply --run-id run-req027-replay --replay-id replay-req027-apply-blocked --requested-by helper-envctl-replay-01 --operation-ids op-req027-manual-cutover --reason prove approvals block R4 apply replay
python3 scripts/verify_replay_engine.py
```
