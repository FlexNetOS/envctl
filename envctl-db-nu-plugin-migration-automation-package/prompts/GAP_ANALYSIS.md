# Gap analysis — prior FlexNetOS package vs envctl database automation

The prior package is a strong artifact-generation contract, but it is not yet a durable migration execution platform. Codex must close these gaps.

## Gaps in the prior package

| Gap | Why it matters | Required envctl addition |
|---|---|---|
| No persistent run ledger | A migration cannot be reliably replayed from markdown artifacts alone. | `migration_runs`, event log, operation records, reproducibility hashes. |
| No generic target descriptor | FlexNetOS paths are hardcoded into prompts. | Versioned target descriptor schema supporting any code/data/infra/integration target. |
| No migration recipe DSL | Phases exist as prompt instructions, not executable dependency graph. | Recipe records with phases, operations, dependencies, approval gates, validators. |
| No database artifact registry | Artifacts are files, not queryable records. | Artifact table with contract ID, status, path, hash, links, evidence. |
| No event-sourced operation state | Shell helper output is raw evidence but not an operation ledger. | Append-only events with sequence/hash chain and materialized views. |
| No approval model | Safety is prompt-level only. | Approval requests/decisions tied to operations and actors. |
| No live event stream | Humans cannot watch progress structurally. | Events/materialized views consumed by `nu_plugin`. |
| No human involvement modes | Human presence is not modeled. | Observer, approval-gated, operator, and agent-only modes. |
| No replay engine | Reproduction requires manually rerunning prompts. | Replay from target descriptor + recipe + contract + event/evidence hashes. |
| No plugin boundary | No CLI visual/control surface. | Shared envctl/nu_plugin protocol and commands. |
| No package adapter contract | Prior package output is not importable as a database bundle. | Importer for manifests, artifact trees, evidence ledgers, link graphs, run metadata. |
| No multi-target abstraction | Only FlexNetOS/lifeos comparison is explicit. | Target adapters and scanner capabilities. |
| No standardized security provenance | Redaction exists, but persistence model does not. | Evidence redaction flags, secret finding records, command redaction, tool versions. |
| No DB-backed validation ledger | Validation reports are artifacts, not queryable pass/fail records. | `migration_validation_results`, quality gates, scorecards. |
| No rollback handles | Rollback plan exists as markdown, not machine-controlled checkpoints. | Checkpoint/snapshot/rollback metadata and approval gates. |

## Gaps Codex must explicitly search for in envctl

- Existing database backend and migration framework.
- Existing CLI/API command architecture.
- Existing automation/agent abstractions.
- Existing schema versioning.
- Existing event/log/audit patterns.
- Existing plugin or IPC protocol conventions.
- Existing security/redaction policy.
- Existing test harness.

## Gaps Codex must explicitly search for in nu_plugin

- Actual Nushell plugin crate/protocol version.
- Plugin executable name and command registration pattern.
- Existing command output schema conventions.
- Error model and structured `LabeledError` style.
- Plugin registry/use instructions.
- Testing approach for plugin commands.
- Whether streaming output is already supported.

## Done means

A migration run is not complete because a prompt finished. It is complete when envctl can query:

```text
target descriptor hash
+ recipe hash
+ artifact contract version
+ operation log
+ evidence hashes
+ approval decisions
+ validation results
+ artifact hashes
+ replay result
```

and reproduce or explain the run without relying on human memory.
