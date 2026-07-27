---
id: 019fa1d0-1d28-7733-9d4d-f462783f6799
slug: tasks/req-027-envctl-replay-engine
title: "Implement Migration Replay and Reproduce Engine"
type: task
status: completed
priority: high
parent: tasks/migration-db
tags: [rust, migration-db, replay, cli, phase2]
---

## Overview

Implement REQ-027 as a repo-native Rust replay engine over the existing redb migration ledger. The current replay surface verifies a limited set of hashes; this task adds request-scoped replay planning from target descriptors, artifact contracts, recipes, package manifests, recorded operations, proof/evidence and artifact hashes, approvals, checkpoints, tool versions, and hash-chained state.

Apply-mode remains fail-closed: envctl emits a replay plan only when inputs are in scope, hashes match, approvals are closed, and selected operations are deterministic. It does not execute redacted command strings as arbitrary shell. Parent: [[tasks/migration-db]].

## Goals

- Expose `envctl replay dry-run|apply` with typed structured input/output.
- Verify all stored replay inputs and filesystem-backed hashes within an explicit replay root.
- Report missing evidence, blocked references, non-deterministic operations, required approvals, checkpoints, and a safe next action.
- Preserve the existing `envctl migration run replay` compatibility surface.

## Implementation

Extend `crates/engine/src/migration_db/replay.rs` with typed request/result and fail-closed path/hash validation. Wire a top-level CLI command through `crates/cli/src/main.rs` and `migration_cmd.rs`. Add focused engine and CLI tests.

## Acceptance Criteria

- [ ] Dry-run reconstructs a deterministic operation plan from stored descriptors, recipes, operations, checkpoints, proofs/evidence, artifacts, tool versions, and event state.
- [ ] Hash mismatches, missing hashes/files, blocked/out-of-scope paths, broken event chains, open approvals, and non-deterministic apply operations are surfaced and block apply.
- [ ] Unknown requested operation IDs fail closed.
- [ ] Existing replay verification behavior remains compatible.
- [ ] Focused Rust tests and workspace format/check gates pass.

## Spec References

- [[tasks/migration-db]] — parent database and ledger implementation.
- `specs/replay-and-reproducibility.md` in the migration automation package — required inputs and outputs.