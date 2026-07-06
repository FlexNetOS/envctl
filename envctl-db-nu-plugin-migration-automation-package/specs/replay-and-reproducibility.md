# Replay and reproducibility

A run is reproducible when envctl can reconstruct the plan and verify all hashes and decisions from stored records.

## Required replay inputs

- target descriptor
- artifact contract
- recipe
- package manifest
- command/operation inputs
- evidence hashes
- artifact hashes
- tool versions
- approval decisions

## Replay outputs

- replay status
- hash matches/mismatches
- missing evidence
- non-deterministic operations
- required approvals
- safe next action
