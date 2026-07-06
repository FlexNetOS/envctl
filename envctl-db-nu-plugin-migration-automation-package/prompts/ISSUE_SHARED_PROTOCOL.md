# GitHub issue prompt — shared envctl / nu_plugin migration protocol

## Title

Define shared migration automation protocol between envctl database and nu_plugin

## Problem

envctl and `nu_plugin` need a stable boundary so database-backed migration automation can be controlled and visualized from Nushell without state drift.

## Required shared records

- TargetDescriptor
- MigrationRecipe
- MigrationRun
- RunEvent
- Operation
- ApprovalRequest
- ApprovalDecision
- ArtifactRecord
- EvidenceRecord
- GraphEdge
- ValidationResult
- ReplayRequest
- ReplayResult

## Required properties

- Versioned schemas.
- Backward-compatible evolution.
- Structured errors.
- Auditable mutating calls.
- Clear source-of-truth rule: envctl database owns state.
- Plugin output shapes optimized for Nushell tables/records.

## Acceptance criteria

- Shared schemas/types exist.
- envctl can serialize records.
- `nu_plugin` can deserialize and render records.
- Mutating calls append events.
- Tests cover schema compatibility.
