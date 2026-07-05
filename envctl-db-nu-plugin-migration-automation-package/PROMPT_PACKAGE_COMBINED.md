---

# FILE: prompts/MASTER_PROMPT_ENVCTL_DB_NU_PLUGIN.md

# MASTER PROMPT — Codex 5.4 envctl Database + nu_plugin Migration Automation Builder

You are Codex 5.4 running locally through the Codex Rust CLI on Ubuntu 26.04+.

Your mission is to upgrade the local `envctl` database tooling and the connected `nu_plugin` repository so migration execution becomes a built-in, reproducible, agent-controllable database capability with live human visuals and optional human involvement.

## Input variables

The runner may provide:

```text
RUN_CONTEXT_FILE=<path>
PROMPT_PACKAGE_DIR=<path>
ENVCTL_REPO=<path>
NU_PLUGIN_REPO=<path>
FLEXNETOS_PACKAGE=<path>
```

Read the run context first. Resolve all paths with `realpath` and record the resolved paths in a working note inside the envctl repo.

## Absolute non-negotiables

- No simulation.
- No fabricated repository structure.
- No invented database engine, schema conventions, plugin crate versions, command surfaces, or test frameworks.
- Use real local evidence from `ENVCTL_REPO`, `NU_PLUGIN_REPO`, and `FLEXNETOS_PACKAGE`.
- Make reviewable code/docs/test changes only in the provided repos.
- Do not delete user data.
- Do not run destructive DB migrations against production databases.
- Do not install packages without approval.
- Redact secrets.
- If something is absent or unknown, mark it as absent/unknown with evidence.

## Required package context

Read and apply all of these files from `PROMPT_PACKAGE_DIR`:

```text
prompts/STRATEGY_DECISION.md
prompts/GAP_ANALYSIS.md
prompts/UTILIZE_FLEXNETOS_PACKAGE.md
prompts/DATABASE_FEATURE_SPEC.md
prompts/NU_PLUGIN_CONTRACT.md
prompts/AGENT_CONTROL_PROTOCOL.md
prompts/LIVE_VISUALS_AND_HUMAN_CONTROL.md
prompts/ANY_TARGET_EXTENSION_SPEC.md
prompts/SECURITY_REPRODUCIBILITY_MODEL.md
prompts/IMPLEMENTATION_PHASES.md
prompts/ACCEPTANCE_CRITERIA.md
prompts/spark_helpers/*.md
schemas/*.json
sql/*.sql
expected-output/*.md
source/codex-flexnetos-migration-prompt-package/PROMPT_PACKAGE_COMBINED.md
source/codex-flexnetos-migration-prompt-package/prompts/*.md
source/codex-flexnetos-migration-prompt-package/expected-output/migration-artifacts-tree.md
source/previous-migration-artifact-context.md
source/current-user-request.md
```

Do not summarize away the prior migration artifact context. Treat it as the artifact contract that envctl must be able to execute and reproduce.

## Model and helper requirements

1. Verify that the orchestrator label resolves to `codex-5.4`. If not, stop and report the exact unavailable label.
2. Verify that helper/subagent label `spark-5.3` resolves. If not, stop and report the exact unavailable label.
3. Use Spark 5.3 helpers/subagents for at least these workstreams:
   - envctl database architecture
   - envctl operation runner and replay
   - nu_plugin protocol and command surface
   - live visuals and human control
   - security and reproducibility
   - FlexNetOS package adapter
   - testing and validation
   - GitHub issue/PR integration

## Required high-level architecture

The envctl database is the source of truth. The Nushell plugin is a controlled operator and visualization surface.

```text
Codex / agents / humans
        ↓
nu_plugin command surface and live visuals
        ↓ controlled protocol / CLI / API
       envctl database automation layer
        ↓
operation queue + event log + approvals + evidence + artifacts
        ↓
target adapters + scanners + package executors + validators
        ↓
actual target repositories / systems / databases / files
```

## Required implementation outcomes

Codex must inspect actual repo layout, then implement/scaffold repo-native changes for:

1. **Artifact contract registry**
   - Import the prior FlexNetOS migration artifact contract.
   - Store artifact contract versions.
   - Preserve links, required artifacts, evidence rules, and validation requirements.

2. **Generic target descriptor**
   - Add parser/validator for target descriptors.
   - Support codebase, data, infrastructure, integration, and mixed migrations.
   - Support `primary_root`, `compare_root`, include/exclude patterns, scanner capabilities, safety policy, and output policy.

3. **Migration recipe DSL**
   - Define reproducible migration recipes: phases, operations, dependencies, approval gates, expected artifacts, validators, and rollback hooks.

4. **envctl database schema**
   - Add tables or repo-native equivalents for targets, contracts, recipes, runs, events, operations, evidence, artifacts, graph edges, approvals, checkpoints, rollbacks, validations, plugin sessions, and agent sessions.
   - Event records must support replay integrity using hashes or monotonic sequence provenance.

5. **Operation runner**
   - Execute packages and native collectors as idempotent operations.
   - Append every state change to the event log.
   - Support pause/resume/cancel where safe.
   - Support approval gates for destructive or high-risk operations.

6. **FlexNetOS package adapter**
   - Add envctl command/API path to import and execute `codex-flexnetos-migration-prompt-package` as a migration artifact bundle.
   - Convert its artifacts, evidence, helper outputs, and manifest into envctl records.
   - Make the adapter generic enough that other packages can be executed by descriptor + recipe.

7. **nu_plugin command surface**
   - Add or modify plugin commands for migration target registration, planning, running, event streaming, status, graph views, artifact opening/listing, approvals, pause/resume, replay, and rollback initiation.
   - Use Nushell-native structured tables/records.
   - Do not make the plugin a separate state store.

8. **Live visuals and human involvement**
   - Provide human modes:
     - `observer`: see live visuals only.
     - `approval-gated`: approve/deny risky operations.
     - `operator`: manually steer or pause/resume.
     - `agent-only`: agents proceed inside pre-approved safe boundaries.
   - Visuals must derive from envctl event streams/materialized views.

9. **Security and reproducibility**
   - Redact secrets.
   - Hash evidence/artifact contents.
   - Record tool versions, command hashes, target descriptor hashes, recipe hashes, package hashes, and environment facts.
   - Support replay from target descriptor + recipe + contract + event log + evidence hashes.

10. **Tests and acceptance**
   - Add repo-native tests for schema, command/API behavior, plugin output schema, approval gates, artifact ingestion, event replay, and error cases.

## Work plan

### Phase 0 — Discovery and repo evidence

Inspect both repos:

```bash
pwd
find "$ENVCTL_REPO" -maxdepth 3 -type f | sed "s|$ENVCTL_REPO/||" | sort | head -300
find "$NU_PLUGIN_REPO" -maxdepth 3 -type f | sed "s|$NU_PLUGIN_REPO/||" | sort | head -300
git -C "$ENVCTL_REPO" status --short
git -C "$NU_PLUGIN_REPO" status --short
```

Find language/tooling, database layer, CLI layer, plugin entrypoints, existing tests, migrations, schemas, and issue templates.

### Phase 1 — Contract lock

Extract a durable artifact contract from the prior package and store it in the envctl repo in the repo-native location. If no location exists, create a namespaced location and document it.

The contract must include every artifact from the prior response: system inventory, dependency maps, data flow, schema maps, validation, cutover, rollback, risk register, decision log, code maps, advanced artifacts, link graph, persistent memory, and wiki/index outputs.

### Phase 2 — Shared protocol

Define shared schema/types between envctl and `nu_plugin` before building commands. The boundary must cover:

- target descriptors
- migration recipes
- run status
- event envelopes
- operation records
- approval requests
- artifact records
- graph summaries
- validation results
- replay requests

### Phase 3 — envctl database implementation

Implement database migrations/schema and persistence APIs according to the actual envctl architecture. Use `sql/*.sql` as the canonical feature intent, adapting to repo conventions.

### Phase 4 — envctl runner/adapter implementation

Implement the external package adapter first, then native collectors. The adapter must be able to run/import `codex-flexnetos-migration-prompt-package` and convert outputs into envctl database records.

### Phase 5 — nu_plugin implementation

Implement the Nushell plugin commands using the shared protocol. Commands must output structured data suitable for live tables and graph views.

### Phase 6 — Tests and validation

Run repo-native tests. Add new tests. If commands cannot run, record exact blockers.

### Phase 7 — Issue updates and PR-ready summary

Create/update issue text using:

```text
prompts/ISSUE_ENVCTL.md
prompts/ISSUE_NU_PLUGIN.md
prompts/ISSUE_SHARED_PROTOCOL.md
```

## Final response required from Codex

Report:

1. Completion status: complete, partial, or blocked.
2. envctl repo path and inspected commit.
3. nu_plugin repo path and inspected commit.
4. Files changed.
5. Database migrations/schema added.
6. Plugin commands added/changed.
7. How the FlexNetOS package is imported/executed.
8. How any target can be added.
9. Live visual/human involvement modes implemented.
10. Tests run and results.
11. Remaining blockers.

Do not claim completion if required acceptance criteria are missing.


---

# FILE: prompts/STRATEGY_DECISION.md

# Strategy decision — package first, tooling first, or parallel?

## Decision

Use **contract-first parallel implementation**.

This is not a pure sequence where Codex fully implements the FlexNetOS package first. It is also not a pure tooling-first build where envctl is designed in the abstract. The correct path is:

1. **Lock the artifact contract first.** Extract the prior migration-artifact requirements and FlexNetOS package shape into versioned envctl database records/schemas.
2. **Run the prior package through an adapter early.** The existing package becomes the first executable migration bundle and acceptance fixture.
3. **Build envctl database execution in parallel.** envctl must persist targets, recipes, operations, evidence, approvals, artifacts, graph links, validations, checkpoints, rollbacks, and replay metadata.
4. **Build nu_plugin in parallel after shared protocol lock.** The plugin reads the envctl event/materialized views and appends controlled operator decisions; it does not own state.
5. **Gradually replace external shell helpers with native envctl collectors.** Native collectors are better long-term, but the adapter gives immediate continuity with the prior package.

## Why not implement the prior package first?

That would produce useful artifacts but still leave migration execution outside the database. It would fail the bigger requirement: envctl must reproduce migration operations anytime.

## Why not build envctl tooling first?

That risks building a generic orchestration database that cannot run the actual artifact contract. The FlexNetOS package is the concrete truth test.

## Why parallel?

Because the hard boundary is not code generation; it is state ownership. envctl must own state and replay. `nu_plugin` must expose control/visuals. The prior package must become an executable artifact contract. These three surfaces have to converge around the same event and artifact schema.

## Required first PR sequence

1. Shared target descriptor + artifact contract schema.
2. envctl database migration automation tables and persistence layer.
3. envctl adapter for `codex-flexnetos-migration-prompt-package`.
4. nu_plugin status/events/artifacts read-only commands.
5. Approval/pause/resume/replay mutating commands.
6. Native collectors and richer visualizations.


---

# FILE: prompts/GAP_ANALYSIS.md

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


---

# FILE: prompts/UTILIZE_FLEXNETOS_PACKAGE.md

# How to utilize codex-flexnetos-migration-prompt-package

The prior package is the first real migration bundle and the acceptance fixture for envctl database automation.

## Treat it as four things

1. **Artifact contract source**
   - `source/previous-migration-artifact-context.md`
   - `prompts/ARTIFACT_CONTRACT_FULL.md`
   - `expected-output/migration-artifacts-tree.md`

2. **Execution recipe source**
   - `prompts/MASTER_PROMPT.md`
   - `prompts/EXECUTION_STYLE.md`
   - `helpers/background_scan.sh`
   - `prompts/spark_helpers/*.md`

3. **Adapter fixture**
   - The package defines how a bundle provides prompts, helpers, schemas, expected outputs, and source context.
   - envctl must import this metadata into database records.

4. **Regression fixture**
   - The adapter must prove it can run/import the package, ingest artifacts, preserve evidence, and expose status through `nu_plugin`.

## Required envctl commands/API capabilities

Codex must adapt names to the existing repo, but the capability surface is:

```text
envctl migration package inspect <package-path>
envctl migration package import <package-path> --name flexnetos-artifact-contract
envctl migration target add --descriptor <target.yaml>
envctl migration plan --target <target-id> --contract <contract-id> --recipe <recipe-id>
envctl migration run <plan-id> --mode approval-gated
envctl migration events <run-id>
envctl migration artifacts <run-id>
envctl migration replay <run-id> --verify-hashes
envctl migration export <run-id> --format wiki|json|markdown
```

## Required package import mapping

| Prior package file | envctl database record |
|---|---|
| `PACKAGE_MANIFEST.json` | package version, file hashes, provenance |
| `prompts/ARTIFACT_CONTRACT_FULL.md` | artifact contract |
| `expected-output/migration-artifacts-tree.md` | artifact templates / required output tree |
| `prompts/MASTER_PROMPT.md` | recipe / execution plan source |
| `prompts/spark_helpers/*.md` | helper operation definitions |
| `helpers/background_scan.sh` | external collector operation |
| `schemas/*.json` | validation schemas |
| generated `migration-artifacts/**` | artifact records + evidence links |

## Required adapter behavior

- Inspect package manifest and compute hashes.
- Refuse to run if required package files are missing.
- Create a migration contract record.
- Create a recipe record for the package phases.
- Create operation records for shell helper, Spark helper scans, artifact synthesis, linking, validation, and final summary.
- Append events for operation start, stdout/stderr evidence, artifact creation, validation results, and blockers.
- Store raw evidence as immutable files or blobs referenced by hash.
- Convert generated artifact tree into database artifact records.
- Link artifacts using `artifact_id`, file path, link graph, and parent/related edges.
- Expose run through nu_plugin.

## Genericization requirement

Do not bake FlexNetOS into envctl. FlexNetOS should be one target descriptor and one package fixture. The same envctl machinery must run against any target with:

- descriptor
- recipe
- artifact contract
- safety policy
- scanner capability map
- output policy


---

# FILE: prompts/DATABASE_FEATURE_SPEC.md

# envctl database feature specification

## Product objective

Make migration execution a first-class, database-backed envctl capability that agents can control safely from CLI workflows and humans can observe/intervene in through live visuals.

## Database responsibilities

The envctl database must persist:

- target descriptors
- artifact contracts
- migration recipes
- migration runs
- operation state machine
- append-only events
- evidence references and hashes
- generated artifacts and link graph
- dependency graphs and data flow graphs
- approval requests and decisions
- validation results
- checkpoints, snapshots, and rollback metadata
- tool versions and environment facts
- agent sessions and human sessions
- plugin sessions
- package imports and package execution results

## State machine

Run statuses:

```text
created -> planning -> awaiting_approval -> running -> paused -> validating -> completed
                         ↓                 ↓         ↓          ↓
                      denied            blocked   failed     cancelled
```

Operation statuses:

```text
queued -> ready -> awaiting_approval -> running -> succeeded
                         ↓                 ↓           ↓
                       denied           blocked      failed
```

Every status transition must append a run event.

## Event envelope

Every event must include:

```json
{
  "run_id": "...",
  "event_seq": 123,
  "event_type": "operation.started",
  "phase": "discovery",
  "actor_type": "agent|human|system|plugin",
  "actor_id": "...",
  "operation_id": "...",
  "timestamp_utc": "...",
  "payload": {},
  "evidence_refs": [],
  "previous_event_hash": "...",
  "event_hash": "..."
}
```

## Materialized views / query surfaces

Codex must implement repo-native equivalents for:

- latest run status
- timeline/events
- operation queue
- open approvals
- artifact index
- evidence register
- dependency graph edges
- validation scorecard
- replay plan

## Artifact contract registry

Artifact contracts are versioned. A run must reference exactly one artifact contract version and one recipe version. Contract changes create new versions, not silent edits.

## Replay

Replay must verify:

- target descriptor hash
- recipe hash
- artifact contract hash
- package hash
- tool versions
- command hashes
- evidence hashes
- artifact hashes
- approval decisions

Replay may be `verify-only`, `dry-run-plan`, or `execute-again`. Destructive replay requires explicit approval.

## Agent control

Agents may create plans, execute safe read-only operations, synthesize artifacts, run validations, and request approvals. Agents may not silently perform destructive operations, mutate production targets, or bypass safety policy.


---

# FILE: prompts/NU_PLUGIN_CONTRACT.md

# nu_plugin contract

## Role

`nu_plugin` is the operator shell and visual surface for envctl migration automation. It does not own durable migration state. It talks to envctl through a controlled CLI/API/shared library boundary.

## Required command capabilities

Codex must adapt command names to the repo's existing conventions, but the functional surface must include:

```text
envctl migration targets
envctl migration target add
envctl migration packages
envctl migration package inspect
envctl migration package import
envctl migration plan
envctl migration run
envctl migration status
envctl migration events
envctl migration timeline
envctl migration ops
envctl migration approvals
envctl migration approve
envctl migration deny
envctl migration pause
envctl migration resume
envctl migration artifacts
envctl migration artifact open
envctl migration graph
envctl migration validations
envctl migration replay
envctl migration rollback plan
envctl migration export
```

## Structured outputs

Commands must return Nushell-native structured records/tables. Examples of output columns:

### `migration status`

```text
run_id | target | status | phase | percent | open_approvals | failed_ops | artifact_count | last_event_at
```

### `migration events`

```text
seq | time | phase | event_type | actor | operation | status | summary
```

### `migration approvals`

```text
approval_id | run_id | operation | risk | requested_by | status | reason | created_at
```

### `migration graph`

```text
from | to | edge_type | source_artifact | confidence | evidence
```

## Mutating command rule

Every mutating plugin command must append an event through envctl. The plugin must not write directly to arbitrary database tables unless envctl's architecture explicitly defines the plugin as an authorized DB client with transaction APIs.

## Live visuals

The plugin should support:

- status tables
- timelines
- operation queue views
- approval queue views
- artifact index views
- graph edge views
- validation scorecards
- replay plan summaries

Visuals must degrade gracefully in plain terminal mode.

## Failure behavior

If envctl is unavailable, the plugin returns a structured error. If a run is blocked, the plugin displays the blocker and the next safe action. If an approval is required, the plugin must not proceed automatically.


---

# FILE: prompts/AGENT_CONTROL_PROTOCOL.md

# Agent control protocol

## Principle

Agents and humans use the same operation queue and event ledger. The difference is authority, not state.

## Actors

```text
system      envctl internal scheduler/runner
agent       Codex or other automation agent
human       shell user/operator
plugin      nu_plugin session acting on behalf of a human
external    imported package/helper process
```

## Authority levels

```text
read_only          can inspect targets, events, artifacts, and plans
safe_execute       can run read-only scanners and artifact generation
approval_request  can request approval for risky operations
operator           can approve, deny, pause, resume, or rollback
admin              can alter policies/contracts/schemas
```

## Operation risk classes

```text
R0 read-only inspection
R1 writes only to envctl artifact/evidence store
R2 modifies local working copy artifacts
R3 modifies target repository code/config
R4 modifies databases/infrastructure/secrets
R5 destructive or production-impacting operation
```

R3+ requires explicit approval unless policy has a pre-approved scoped rule. R4/R5 must default to approval-gated.

## Event append contract

All actions append events. No hidden side effects.

## Idempotency

Operations must include an idempotency key derived from:

```text
run_id + operation_type + target_descriptor_hash + recipe_step_id + input_hash
```

## Concurrency

Multiple agents may work in parallel only if operations have non-conflicting target scopes or the database obtains a lease/lock. Locks must be visible in live status.


---

# FILE: prompts/LIVE_VISUALS_AND_HUMAN_CONTROL.md

# Live visuals and human involvement

## Human involvement modes

| Mode | Meaning | Allowed behavior |
|---|---|---|
| `observer` | Human watches only. | Agents execute within safe policy; visuals update live. |
| `approval-gated` | Human approves risky operations. | R3+ operations wait for approval. |
| `operator` | Human can steer execution. | Human can pause, resume, deny, retry, or choose branches. |
| `agent-only` | No live human involvement inside pre-approved policy. | Only R0-R2 unless policy explicitly allows more. |

## Visual surfaces

The plugin must expose:

- Run overview.
- Phase progress.
- Operation queue.
- Event timeline.
- Open approvals.
- Artifact index.
- Evidence register.
- Dependency graph edges.
- Validation scorecard.
- Replay readiness.
- Rollback readiness.

## Visual correctness rule

The plugin may format and summarize, but every visual must derive from envctl database records. No plugin-only inferred state should be treated as truth.

## Minimum live table set

```text
migration status
migration timeline
migration ops
migration approvals
migration artifacts
migration graph
migration validations
migration replay status
```

## Optional richer visuals

- Mermaid graph export.
- DOT graph export.
- TUI dashboard if existing repo supports it.
- Streaming event subscription if existing architecture supports it.
- Static HTML/wiki export if existing artifact tooling supports it.


---

# FILE: prompts/ANY_TARGET_EXTENSION_SPEC.md

# Any-target extension specification

The FlexNetOS/lifeos case is only the first target. envctl must support arbitrary migration targets.

## Target descriptor required fields

```yaml
schema_version: 1
target_id: stable-human-readable-id
target_type: codebase | data | infrastructure | integration | mixed
primary_root: /absolute/or/repo-relative/path
compare_root: /optional/path
output_root: migration-artifacts
include:
  - "**/*"
exclude:
  - ".git/**"
  - "node_modules/**"
  - "target/**"
safety:
  default_mode: approval-gated
  max_auto_risk: R2
  allow_network: false
  allow_destructive: false
collectors:
  filesystem: true
  git: true
  package_managers: true
  databases: false
  infrastructure: true
  apis: true
artifact_contract:
  name: full-migration-artifact-contract
  version: 1
recipe:
  name: full-migration-discovery-and-execution
  version: 1
```

## Adapter requirements

A target adapter must define:

- what can be scanned
- what commands are safe
- how evidence is collected
- how secrets are redacted
- what artifact templates apply
- what validation rules apply
- how rollback/checkpoints are represented

## Collector capability map

```text
filesystem
repo/git
language/package manager
build/test
runtime entrypoints
config/secrets references
api contracts
event/message contracts
database schema
data lineage
infra/IaC
observability
security/IAM
business process metadata
```

## Generic migration applicability

envctl can apply the migration design to any target only when it has:

1. A target descriptor.
2. An artifact contract.
3. A migration recipe.
4. A safety policy.
5. Collector capability map.
6. Evidence redaction policy.
7. Validation policy.
8. Output/export policy.
9. Replay policy.
10. Human involvement mode.


---

# FILE: prompts/SECURITY_REPRODUCIBILITY_MODEL.md

# Security and reproducibility model

## Security requirements

- Redact secrets before artifact persistence.
- Store command strings in redacted form when they may contain credentials.
- Store raw evidence only in approved evidence locations.
- Mark evidence as redacted/unredacted.
- Hash every evidence file and generated artifact.
- Approval-gate risky operations.
- Never bypass sandbox/approval controls.
- Keep plugin mutating commands auditable.
- Prevent agents from writing directly to production targets without policy.

## Reproducibility identity

A migration run identity must include:

```text
target_descriptor_hash
artifact_contract_hash
recipe_hash
package_hashes
collector_versions
tool_versions
operation_input_hashes
evidence_hashes
artifact_hashes
approval_decision_hashes
```

## Replay modes

| Mode | Behavior |
|---|---|
| `verify-only` | Recompute hashes and confirm artifacts/evidence still match. |
| `dry-run-plan` | Reconstruct operation plan without executing target-affecting commands. |
| `execute-safe` | Re-run R0-R2 operations. |
| `execute-full` | Requires explicit approval for R3+. |

## Chain integrity

Run events should be chained:

```text
event_hash = sha256(previous_event_hash + canonical_event_json)
```

If the repo already uses a different integrity model, preserve it and map this requirement to the native model.


---

# FILE: prompts/IMPLEMENTATION_PHASES.md

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


---

# FILE: prompts/ACCEPTANCE_CRITERIA.md

# Acceptance criteria

Codex may only claim completion when all applicable criteria pass or are explicitly marked blocked with evidence.

## envctl criteria

- [ ] A target descriptor can be parsed and validated.
- [ ] The prior FlexNetOS package can be inspected/imported as a package/artifact contract.
- [ ] A migration recipe can be created from contract phases.
- [ ] A migration run can be created in the database.
- [ ] Operations append events.
- [ ] Evidence records include path/URI, kind, hash, redaction status, and operation link.
- [ ] Artifact records include artifact ID, status, hash, path, evidence, and links.
- [ ] Approval requests block risky operations.
- [ ] Approval decisions are appended as events.
- [ ] Validation results are queryable.
- [ ] Replay can reconstruct or verify a run.
- [ ] Rollback metadata exists for operations that need rollback.
- [ ] Tests cover the above.

## nu_plugin criteria

- [ ] Plugin command signatures are registered according to the actual nu_plugin protocol used by the repo.
- [ ] Read-only commands return structured Nushell records/tables.
- [ ] Mutating commands call envctl through a controlled boundary.
- [ ] Status/timeline/artifacts/approvals/graph/validation commands work against test data.
- [ ] Approval commands append auditable envctl events.
- [ ] Errors are structured and actionable.
- [ ] Tests cover command outputs and failure cases.

## Integration criteria

- [ ] A run created by envctl appears in the plugin.
- [ ] Events appended by envctl appear in the plugin timeline.
- [ ] Plugin approval changes operation state in envctl.
- [ ] Artifact records link back to evidence and can be listed/opened.
- [ ] Replay verification result is visible through plugin.
- [ ] The FlexNetOS-vs-lifeos target descriptor is one fixture, not hardcoded into the engine.

## Documentation/issue criteria

- [ ] envctl issue prompt is updated.
- [ ] nu_plugin issue prompt is updated.
- [ ] Shared protocol issue prompt is updated.
- [ ] The strategy decision and gap analysis are present in docs.


---

# FILE: prompts/ISSUE_ENVCTL.md

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


---

# FILE: prompts/ISSUE_NU_PLUGIN.md

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


---

# FILE: prompts/ISSUE_SHARED_PROTOCOL.md

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


---

# FILE: prompts/spark_helpers/spark-envctl-db-architect.md

# spark-envctl-db-architect

Analyze envctl database architecture. Identify migration framework, schema conventions, storage abstractions, and implement database-backed migration automation tables/views/persistence APIs.


## Shared constraints

- Inspect real repo files before proposing edits.
- Do not invent architecture.
- Mark unknowns explicitly.
- Preserve envctl as source of truth.
- Preserve nu_plugin as CLI/control/visual surface.
- No destructive actions without approval.
- Produce concrete files/functions/tests to change.


Required output:

```text
analysis/<spark-envctl-db-architect>-findings.md
analysis/<spark-envctl-db-architect>-recommended-changes.md
analysis/<spark-envctl-db-architect>-tests.md
```

If implementation is in scope for the run, make the code/docs/test changes and record exact file paths changed.


---

# FILE: prompts/spark_helpers/spark-envctl-runner.md

# spark-envctl-runner

Analyze envctl runner/CLI architecture. Design and implement operation state machine, event appends, package adapter, target descriptor execution, replay, checkpoints, and rollback metadata.


## Shared constraints

- Inspect real repo files before proposing edits.
- Do not invent architecture.
- Mark unknowns explicitly.
- Preserve envctl as source of truth.
- Preserve nu_plugin as CLI/control/visual surface.
- No destructive actions without approval.
- Produce concrete files/functions/tests to change.


Required output:

```text
analysis/<spark-envctl-runner>-findings.md
analysis/<spark-envctl-runner>-recommended-changes.md
analysis/<spark-envctl-runner>-tests.md
```

If implementation is in scope for the run, make the code/docs/test changes and record exact file paths changed.


---

# FILE: prompts/spark_helpers/spark-flexnetos-adapter.md

# spark-flexnetos-adapter

Map codex-flexnetos-migration-prompt-package into envctl package import/execution records. Ensure FlexNetOS is a fixture, not hardcoded.


## Shared constraints

- Inspect real repo files before proposing edits.
- Do not invent architecture.
- Mark unknowns explicitly.
- Preserve envctl as source of truth.
- Preserve nu_plugin as CLI/control/visual surface.
- No destructive actions without approval.
- Produce concrete files/functions/tests to change.


Required output:

```text
analysis/<spark-flexnetos-adapter>-findings.md
analysis/<spark-flexnetos-adapter>-recommended-changes.md
analysis/<spark-flexnetos-adapter>-tests.md
```

If implementation is in scope for the run, make the code/docs/test changes and record exact file paths changed.


---

# FILE: prompts/spark_helpers/spark-issue-integrator.md

# spark-issue-integrator

Generate/update GitHub issue text and PR sequence for envctl, nu_plugin, and shared protocol based on actual repo findings.


## Shared constraints

- Inspect real repo files before proposing edits.
- Do not invent architecture.
- Mark unknowns explicitly.
- Preserve envctl as source of truth.
- Preserve nu_plugin as CLI/control/visual surface.
- No destructive actions without approval.
- Produce concrete files/functions/tests to change.


Required output:

```text
analysis/<spark-issue-integrator>-findings.md
analysis/<spark-issue-integrator>-recommended-changes.md
analysis/<spark-issue-integrator>-tests.md
```

If implementation is in scope for the run, make the code/docs/test changes and record exact file paths changed.


---

# FILE: prompts/spark_helpers/spark-live-visuals.md

# spark-live-visuals

Design live visual tables and graph/timeline/status views from envctl database events and materialized views. Implement terminal-safe outputs and optional richer exports if repo supports them.


## Shared constraints

- Inspect real repo files before proposing edits.
- Do not invent architecture.
- Mark unknowns explicitly.
- Preserve envctl as source of truth.
- Preserve nu_plugin as CLI/control/visual surface.
- No destructive actions without approval.
- Produce concrete files/functions/tests to change.


Required output:

```text
analysis/<spark-live-visuals>-findings.md
analysis/<spark-live-visuals>-recommended-changes.md
analysis/<spark-live-visuals>-tests.md
```

If implementation is in scope for the run, make the code/docs/test changes and record exact file paths changed.


---

# FILE: prompts/spark_helpers/spark-nu-plugin-protocol.md

# spark-nu-plugin-protocol

Analyze nu_plugin architecture and actual nu_plugin/Nushell versions. Design and implement command signatures, serialization, typed output records, envctl boundary, and structured errors.


## Shared constraints

- Inspect real repo files before proposing edits.
- Do not invent architecture.
- Mark unknowns explicitly.
- Preserve envctl as source of truth.
- Preserve nu_plugin as CLI/control/visual surface.
- No destructive actions without approval.
- Produce concrete files/functions/tests to change.


Required output:

```text
analysis/<spark-nu-plugin-protocol>-findings.md
analysis/<spark-nu-plugin-protocol>-recommended-changes.md
analysis/<spark-nu-plugin-protocol>-tests.md
```

If implementation is in scope for the run, make the code/docs/test changes and record exact file paths changed.


---

# FILE: prompts/spark_helpers/spark-security-reproducibility.md

# spark-security-reproducibility

Design approval gates, risk classes, redaction, evidence hashing, event hash chains, tool version capture, and replay verification.


## Shared constraints

- Inspect real repo files before proposing edits.
- Do not invent architecture.
- Mark unknowns explicitly.
- Preserve envctl as source of truth.
- Preserve nu_plugin as CLI/control/visual surface.
- No destructive actions without approval.
- Produce concrete files/functions/tests to change.


Required output:

```text
analysis/<spark-security-reproducibility>-findings.md
analysis/<spark-security-reproducibility>-recommended-changes.md
analysis/<spark-security-reproducibility>-tests.md
```

If implementation is in scope for the run, make the code/docs/test changes and record exact file paths changed.


---

# FILE: prompts/spark_helpers/spark-test-validation.md

# spark-test-validation

Find test harnesses and add tests for schema, operation lifecycle, plugin commands, event append, approval, artifact import, validation, and replay.


## Shared constraints

- Inspect real repo files before proposing edits.
- Do not invent architecture.
- Mark unknowns explicitly.
- Preserve envctl as source of truth.
- Preserve nu_plugin as CLI/control/visual surface.
- No destructive actions without approval.
- Produce concrete files/functions/tests to change.


Required output:

```text
analysis/<spark-test-validation>-findings.md
analysis/<spark-test-validation>-recommended-changes.md
analysis/<spark-test-validation>-tests.md
```

If implementation is in scope for the run, make the code/docs/test changes and record exact file paths changed.


---

# FILE: source/current-user-request.md

# Current user request

Create a package that applies the previous migration-artifact response and the `codex-flexnetos-migration-prompt-package` to envctl database + `nu_plugin` automation.

Goal: envctl database should provide built-in migration database tooling controllable by CLI agents. Human users should have live visuals and may choose active involvement or passive supervision. envctl database must be able to reproduce the migration operation anytime, making the migration design faster, more secure, more accurate, and repeatable. The package must be additive to current Codex prompts intended for envctl and nu_plugin GitHub issues.

Required design questions:

1. How should the `codex-flexnetos-migration-prompt-package` be utilized to design database automation feature upgrades?
2. Should Codex implement the package first, build envctl tooling to execute the package first, or do both in parallel?
3. What needs to be added so envctl database can apply the migration to any targets?
4. Find the gaps.
5. Provide a full updated tar.gz package.


---

# FILE: source/previous-migration-artifact-context.md

For a migration project, the most useful artifacts are **maps, inventories, dependency graphs, validation evidence, and cutover controls**. The goal is to reduce unknowns: what exists, what depends on what, what breaks if moved, how data changes, how to prove parity, and how to roll back.

The best artifacts are **generated, version-controlled, diffable, and tied to real systems** rather than hand-drawn diagrams that rot.

## High-value migration artifacts

| Artifact                                      | What it answers                                                                                                 | Why it matters                                                                        |
| --------------------------------------------- | --------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| **System inventory**                          | What applications, services, jobs, databases, queues, APIs, reports, scripts, and schedulers exist?             | Prevents hidden systems from being missed.                                            |
| **Directory and subdirectory hierarchy tree** | Where is the code/config/data actually located?                                                                 | Useful for repo cleanup, ownership mapping, build discovery, and dead-code detection. |
| **Repository map**                            | Which repos exist, what they contain, who owns them, and how active they are?                                   | Helps split migration scope into manageable chunks.                                   |
| **Application/service dependency graph**      | Which services call which other services?                                                                       | Critical for sequencing migration waves.                                              |
| **Toolchain dependency tree**                 | What compilers, runtimes, package managers, SDKs, build tools, CI/CD tools, and deploy tools are required?      | Exposes version traps and platform incompatibilities.                                 |
| **Package/library dependency graph**          | Which packages depend on vulnerable, deprecated, or incompatible libraries?                                     | Useful for modernization, security, and upgrade planning.                             |
| **Runtime dependency map**                    | What does the app need at runtime: databases, env vars, secrets, filesystems, queues, third-party APIs, caches? | Prevents “works in build, fails in prod” problems.                                    |
| **Data flow graph**                           | How does data enter, move, transform, persist, and exit the system?                                             | Essential for data migration, compliance, and debugging.                              |
| **Database schema map**                       | What tables, columns, indexes, constraints, procedures, triggers, and views exist?                              | Needed for data migration and performance parity.                                     |
| **Data lineage map**                          | Where does each critical field originate, transform, and get consumed?                                          | Prevents silent data corruption.                                                      |
| **API contract catalog**                      | What endpoints/events exist, request/response schemas, auth, consumers, versions?                               | Prevents breaking integrations.                                                       |
| **Event/message contract map**                | What topics, queues, payloads, producers, consumers, retry semantics, and DLQs exist?                           | Critical for async architectures.                                                     |
| **Code ownership map**                        | Who owns each module/service/data pipeline?                                                                     | Speeds up decisions and incident resolution.                                          |
| **Code map for debugging**                    | Main execution paths, entry points, error paths, logging points, external calls.                                | Helps teams troubleshoot migrated workloads faster.                                   |
| **Environment matrix**                        | Dev/stage/prod differences: versions, configs, infra, secrets, endpoints, feature flags.                        | Finds environment drift before cutover.                                               |
| **Configuration inventory**                   | Env vars, config files, flags, secrets references, runtime parameters.                                          | Configuration is often where migrations fail.                                         |
| **Infrastructure topology map**               | Compute, networking, storage, DNS, load balancers, firewalls, certificates.                                     | Needed for cloud, data center, Kubernetes, or platform migrations.                    |
| **IAM/security access matrix**                | Users, roles, service accounts, permissions, secrets, certificates, tokens.                                     | Prevents broken auth and over-permissioned systems.                                   |
| **Observability map**                         | Logs, metrics, traces, dashboards, alerts, SLOs, runbooks.                                                      | Migration is dangerous without visibility.                                            |
| **Business process map**                      | Which business workflows depend on which systems?                                                               | Helps prioritize what truly matters.                                                  |
| **Migration wave plan**                       | What moves together, in what order, and why?                                                                    | Converts analysis into execution.                                                     |
| **Cutover checklist**                         | Exact steps for go-live.                                                                                        | Reduces human error during high-pressure migration windows.                           |
| **Rollback plan**                             | How to undo or fail back safely.                                                                                | Non-negotiable for serious migrations.                                                |
| **Validation and reconciliation reports**     | Did the migrated system produce the same outputs? Did all records move?                                         | The evidence layer. Without this, migration success is just a claim.                  |
| **Test coverage matrix**                      | What has unit, integration, regression, performance, security, and user acceptance coverage?                    | Shows where risk remains.                                                             |
| **Risk register**                             | Known risks, owners, mitigations, severity, status.                                                             | Keeps ugly truths visible.                                                            |
| **Decision log / ADRs**                       | What decisions were made, by whom, and why?                                                                     | Prevents re-litigating architecture choices later.                                    |

---

## The most useful artifacts by migration type

### 1. **Codebase migration**

Useful when moving languages, frameworks, monoliths to services, legacy apps to modern platforms, or repos to new standards.

Key artifacts:

| Artifact                     | Use                                                                                                         |
| ---------------------------- | ----------------------------------------------------------------------------------------------------------- |
| **Codebase hierarchy graph** | Shows folders, modules, packages, entry points, generated code, tests, configs.                             |
| **Import/dependency graph**  | Shows module-to-module dependencies. Great for identifying circular dependencies and extraction boundaries. |
| **Call graph**               | Shows function/method invocation paths. Useful for debugging and refactoring.                               |
| **Dead-code report**         | Identifies unused modules, functions, routes, jobs, and feature flags.                                      |
| **Hotspot map**              | Combines code churn, bug history, complexity, and ownership. Shows risky areas.                             |
| **Build graph**              | Shows how source code becomes artifacts: binaries, containers, packages, deployables.                       |
| **Runtime entry-point map**  | Shows app start commands, workers, jobs, handlers, APIs, and scheduled tasks.                               |
| **Compatibility matrix**     | Shows supported runtime versions, framework versions, OS versions, and package constraints.                 |

**Especially useful:**
A **code map for debugging** that includes:

```text
Request/Event Entry Point
  → Controller/Handler
    → Service Layer
      → Business Logic
        → Repository/Data Access
          → Database/External API/Queue
    → Error Handling
    → Logging/Tracing
    → Response/Event Output
```

This is more useful than a pretty architecture diagram because it shows where failures actually happen.

---

### 2. **Data migration**

Useful when moving databases, warehouses, lakes, ETL/ELT pipelines, schemas, or reporting layers.

Key artifacts:

| Artifact                          | Use                                                                                        |
| --------------------------------- | ------------------------------------------------------------------------------------------ |
| **Source-to-target mapping**      | Maps old fields/tables to new fields/tables.                                               |
| **Transformation rules catalog**  | Defines type conversions, normalization, joins, defaults, deduplication, enrichment.       |
| **Data lineage graph**            | Shows where data originates and where it flows.                                            |
| **Data quality profile**          | Null rates, duplicates, invalid values, outliers, referential integrity.                   |
| **Reconciliation report**         | Compares source vs. target record counts, checksums, aggregates, business totals.          |
| **Schema diff report**            | Shows changes between old and new schema.                                                  |
| **Critical field inventory**      | Identifies fields that drive billing, reporting, compliance, auth, or customer experience. |
| **Backfill plan**                 | Defines historical data load strategy.                                                     |
| **Incremental sync plan**         | Defines CDC, dual writes, event replay, or batch sync strategy.                            |
| **Data retention/compliance map** | Shows PII, PHI, PCI, retention rules, deletion obligations.                                |

The artifact that usually saves the project is the **reconciliation report**. A migration is not done because the pipeline ran. It is done when the numbers prove it.

---

### 3. **Cloud or infrastructure migration**

Useful when moving from on-prem to cloud, cloud-to-cloud, VMs to containers, or legacy hosting to Kubernetes/serverless.

Key artifacts:

| Artifact                               | Use                                                                        |
| -------------------------------------- | -------------------------------------------------------------------------- |
| **Infrastructure topology map**        | Shows compute, networking, storage, ingress, DNS, security boundaries.     |
| **Network dependency map**             | Ports, protocols, firewalls, allowlists, private links, VPNs, VPCs/VNETs.  |
| **Environment parity matrix**          | Compares old vs. new dev/stage/prod environments.                          |
| **Resource inventory**                 | Servers, VMs, containers, databases, buckets, volumes, certificates.       |
| **IaC coverage report**                | Shows what is codified vs. manually configured.                            |
| **Secrets and certificates inventory** | Expirations, owners, rotation requirements, consuming services.            |
| **Capacity baseline**                  | CPU, memory, storage, IOPS, throughput, latency, peak load.                |
| **Cost baseline and forecast**         | Current cost vs. expected target cost.                                     |
| **DR/backup map**                      | Recovery point objective, recovery time objective, backups, restore tests. |
| **Security control matrix**            | Encryption, network policies, IAM, audit logs, vulnerability controls.     |

The dangerous gaps are usually **DNS, certificates, firewall rules, secrets, and undocumented cron jobs**.

---

### 4. **Application integration migration**

Useful when replacing SaaS systems, ERP/CRM platforms, identity providers, payment systems, messaging layers, or API gateways.

Key artifacts:

| Artifact                            | Use                                                                              |
| ----------------------------------- | -------------------------------------------------------------------------------- |
| **Integration catalog**             | Lists every inbound/outbound integration.                                        |
| **API contract map**                | Shows endpoints, methods, schemas, auth, rate limits, consumers.                 |
| **Webhook/event map**               | Shows event producers, consumers, payloads, retries, delivery guarantees.        |
| **Third-party dependency register** | Vendors, credentials, SLAs, rate limits, support contacts.                       |
| **Auth flow diagram**               | OAuth, SAML, OIDC, API keys, service accounts, token lifetimes.                  |
| **Failure-mode map**                | What happens on timeout, retry, partial failure, duplicate message, bad payload. |
| **Backward compatibility plan**     | Versioning, adapter layers, proxies, shims, redirects.                           |

For integrations, the best artifact is often a **consumer map**:

```text
API / Event / File Feed
  → Producer
  → Consumers
  → Payload Contract
  → Auth Method
  → Frequency
  → Failure Behavior
  → Business Owner
  → Technical Owner
  → Migration Status
```

---

## Artifacts specifically matching your examples

### Directory and subdirectory hierarchy graph

Useful for:

* Discovering project structure.
* Finding duplicated modules.
* Identifying generated files.
* Separating source, tests, configs, docs, scripts, and build outputs.
* Planning repo splits or consolidation.

Example shape:

```text
repo-root/
├── apps/
│   ├── web/
│   ├── api/
│   └── worker/
├── packages/
│   ├── shared-models/
│   ├── auth/
│   └── logging/
├── infra/
│   ├── terraform/
│   └── k8s/
├── scripts/
├── tests/
└── docs/
```

Better version: a graph that tags each folder with:

```text
owner
language
runtime
build tool
deployment target
test coverage
last modified
dependency count
risk score
```

---

### Toolchain dependency tree

Useful for identifying what must exist before the app can build, test, deploy, and run.

Example:

```text
Application
├── Runtime
│   ├── Node 20
│   ├── Python 3.12
│   └── Java 21
├── Build
│   ├── pnpm
│   ├── Maven
│   └── Docker
├── CI/CD
│   ├── GitHub Actions
│   ├── Artifact Registry
│   └── Deployment Runner
├── Infrastructure
│   ├── Terraform
│   ├── Kubernetes
│   └── Helm
└── Security
    ├── SAST
    ├── Dependency scanning
    └── Container scanning
```

For a migration, this should include **version constraints** and **upgrade blockers**.

---

### Code map for debugging

Useful for understanding how to troubleshoot the migrated system.

A good debugging map includes:

| Layer          | What to document                                                  |
| -------------- | ----------------------------------------------------------------- |
| Entry points   | HTTP routes, CLI commands, jobs, queue consumers, event handlers. |
| Control flow   | Main execution path through controllers/services/modules.         |
| External calls | Databases, APIs, queues, caches, filesystems.                     |
| Failure points | Known exception sources, retry paths, timeout risks.              |
| Logs           | Log statements, correlation IDs, trace IDs, severity levels.      |
| Metrics        | Counters, latency, errors, saturation, business KPIs.             |
| Alerts         | What fires, who receives it, what action to take.                 |
| Runbooks       | How to diagnose and recover.                                      |

The useful version is not just “how the code is organized.” It is **how failure moves through the system**.

---

### Data flow graph

Useful for seeing how data actually behaves across systems.

Example:

```text
Customer submits form
  → Web App
  → API Gateway
  → Customer Service
  → Validation Layer
  → PostgreSQL
  → Change Data Capture
  → Event Bus
  → Analytics Pipeline
  → Data Warehouse
  → BI Dashboard
```

Enhance it with:

```text
data owner
system owner
schema
PII classification
latency expectation
retention policy
transformation logic
validation rule
downstream consumers
```

---

## Minimum viable artifact set

For a serious migration, I would not start execution without these:

| Priority | Artifact                             |
| -------: | ------------------------------------ |
|        1 | **System inventory**                 |
|        2 | **Dependency graph**                 |
|        3 | **Data flow graph**                  |
|        4 | **Source-to-target mapping**         |
|        5 | **Environment/configuration matrix** |
|        6 | **Toolchain dependency tree**        |
|        7 | **API/event contract catalog**       |
|        8 | **Test and validation matrix**       |
|        9 | **Cutover checklist**                |
|       10 | **Rollback plan**                    |
|       11 | **Risk register**                    |
|       12 | **Decision log**                     |

That is the core set. Everything else is useful, but those are the artifacts that directly reduce migration failure risk.

---

## Advanced artifacts that are especially valuable

These are higher-effort, but they tend to expose the problems nobody wants to talk about.

| Artifact                             | Why it is powerful                                                               |
| ------------------------------------ | -------------------------------------------------------------------------------- |
| **Blast-radius map**                 | Shows what breaks if a service, table, queue, API, or credential fails.          |
| **Migration readiness scorecard**    | Scores each app/service/data domain against readiness criteria.                  |
| **Business capability map**          | Connects technical systems to actual business functions.                         |
| **Shadow traffic comparison report** | Compares old vs. new system behavior under real traffic.                         |
| **Golden dataset**                   | Known-good sample data used to validate transformations and outputs.             |
| **Parity dashboard**                 | Real-time comparison of old and new system metrics.                              |
| **Deprecation map**                  | Shows what gets retired, replaced, wrapped, or preserved.                        |
| **Exception inventory**              | Captures special cases, one-off scripts, manual processes, and tribal knowledge. |
| **Ownership/RACI matrix**            | Defines who approves, builds, validates, cuts over, and supports.                |
| **Technical debt ledger**            | Separates “must fix before migration” from “carry temporarily” from “delete.”    |

---

## Recommended artifact structure

A practical migration artifact repository might look like this:

```text
migration-artifacts/
├── 00-executive-summary/
├── 01-current-state/
│   ├── system-inventory.md
│   ├── architecture-current.md
│   ├── dependency-graph.md
│   ├── data-flow-current.md
│   └── risk-hotspots.md
├── 02-target-state/
│   ├── architecture-target.md
│   ├── platform-design.md
│   ├── security-model.md
│   └── operating-model.md
├── 03-code-analysis/
│   ├── repo-map.md
│   ├── directory-tree.md
│   ├── package-dependencies.md
│   ├── call-graph.md
│   └── dead-code-report.md
├── 04-data-migration/
│   ├── schema-map.md
│   ├── source-target-mapping.md
│   ├── transformation-rules.md
│   ├── data-quality-report.md
│   └── reconciliation-plan.md
├── 05-integrations/
│   ├── api-catalog.md
│   ├── event-catalog.md
│   ├── third-party-dependencies.md
│   └── auth-flows.md
├── 06-testing-validation/
│   ├── test-strategy.md
│   ├── regression-matrix.md
│   ├── performance-baseline.md
│   └── validation-evidence.md
├── 07-cutover/
│   ├── wave-plan.md
│   ├── cutover-checklist.md
│   ├── rollback-plan.md
│   └── communication-plan.md
├── 08-operations/
│   ├── observability-map.md
│   ├── runbooks.md
│   ├── alerting.md
│   └── incident-response.md
└── 09-governance/
    ├── decision-log.md
    ├── risk-register.md
    ├── ownership-matrix.md
    └── status-dashboard.md
```

---

## Best formats

| Artifact type         | Good formats                                        |
| --------------------- | --------------------------------------------------- |
| Trees and hierarchies | Markdown, Mermaid, Graphviz DOT, JSON               |
| Dependency graphs     | Graphviz, Mermaid, Neo4j, D3, Cytoscape             |
| Inventories           | CSV, spreadsheet, database table                    |
| API contracts         | OpenAPI, AsyncAPI, Postman collections              |
| Data mappings         | Spreadsheet, dbt docs, YAML, SQL metadata           |
| Architecture          | C4 diagrams, Mermaid, PlantUML, Structurizr         |
| Security/IAM          | Matrix tables, policy-as-code, JSON/YAML exports    |
| Test evidence         | CI reports, JUnit XML, coverage reports, dashboards |
| Cutover controls      | Markdown checklist, runbook, project board          |

---

## The artifact hierarchy I would use

For a migration, artifacts should ladder from **discovery** to **execution**:

```text
1. Inventory
   ↓
2. Dependency Map
   ↓
3. Risk Map
   ↓
4. Target Design
   ↓
5. Migration Plan
   ↓
6. Validation Plan
   ↓
7. Cutover Plan
   ↓
8. Rollback Plan
   ↓
9. Operational Runbook
   ↓
10. Decommission Plan
```

The most common mistake is overbuilding architecture diagrams and underbuilding validation artifacts. Pretty diagrams do not prove the migration worked. **Reconciliation, test evidence, parity dashboards, and rollback plans do.**


---

# FILE: source/codex-flexnetos-migration-prompt-package/PROMPT_PACKAGE_COMBINED.md

<!-- BEGIN prompts/EXECUTION_STYLE.md -->
# Execution Style Contract

## Required local execution model

- Orchestrator: `Codex 5.4` through the local Codex Rust CLI.
- Helpers/subagents: multiple `Spark 5.3` helpers.
- Shell strategy: run background shell helpers first to collect raw evidence, then have Spark helpers independently analyze specific domains, then have Codex 5.4 merge, link, and validate all artifacts.

## Model verification

Before generating any migration finding, Codex must verify the active model/provider state. If `codex-5.4` or `spark-5.3` cannot be resolved by the local setup, Codex must create:

- `migration-artifacts/00-executive-summary/model-resolution-blocker.md`

and stop before creating factual analysis artifacts. It may still create empty scaffolding with `status: blocked`.

## Background shell helper requirements

Run `helpers/background_scan.sh` in the background scan phase. The helper must write raw evidence under:

```text
migration-artifacts/_raw/YYYYMMDDTHHMMSSZ/
```

The raw scan must include:

- OS and command availability
- path resolution for FlexNetOS and lifeos candidates
- filesystem tree and file inventory
- git status, remotes, branches, tags, and recent history
- manifest inventory
- package/build/toolchain files
- config/environment key inventory with values redacted
- dependency references
- API/event/webhook references
- database/schema/migration references
- infrastructure/IaC references
- observability references
- test/CI references
- FlexNetOS-vs-lifeos reference search
- checksums and job status

## Spark 5.3 helper allocation

Spawn at least these eight helpers:

1. `spark-filesystem-repo`
2. `spark-toolchain-deps`
3. `spark-code-runtime-debug`
4. `spark-data-schema-lineage`
5. `spark-infra-security-obs`
6. `spark-integrations-contracts`
7. `spark-migration-controls`
8. `spark-flexnetos-investigator`

Each helper must write a domain report under:

```text
migration-artifacts/_spark/<helper-name>.md
migration-artifacts/_spark/<helper-name>.json
```

Each helper must include:

- Scope
- Commands or raw scan files used
- Proven findings
- Unknowns
- Artifact files it recommends creating/updating
- Evidence links

## Orchestrator merge requirements

Codex 5.4 must consolidate Spark helper reports into final artifacts. It must not merely paste helper reports. It must reconcile contradictions, mark unresolved conflicts, link evidence, and produce a coherent wiki-style artifact graph.

<!-- END prompts/EXECUTION_STYLE.md -->


<!-- BEGIN prompts/FLEXNETOS_INVESTIGATION_PROMPT.md -->
# FlexNetOS Investigation Prompt

## Investigation question

What was `FlexNetOS` used for at `~/home/flexnetos/FlexNetOS` instead of `~/home/flexnetos/lifeos`?

## Path resolution rules

Treat the user-provided paths as candidates, not guaranteed truths. Resolve and document:

```text
~/home/flexnetos/FlexNetOS
~/home/flexnetos/lifeos
/home/flexnetos/FlexNetOS
/home/flexnetos/lifeos
```

For each candidate, record:

- exists / missing
- realpath
- symlink target
- mountpoint
- owner/group/mode
- git repository root
- remote URLs redacted only if credential-bearing
- active branch, tags, and recent commits
- top-level tree
- manifests and service definitions

## Evidence classes to inspect

Use real local evidence only:

1. README, docs, markdown, design notes, ADRs
2. package manifests: `package.json`, `Cargo.toml`, `pyproject.toml`, `requirements.txt`, `go.mod`, `pom.xml`, `build.gradle`, Dockerfiles, Compose files, Helm charts, Terraform
3. service definitions: `*.service`, `systemd`, init scripts, supervisord, PM2, cron specs, GitHub/GitLab/CI workflow files
4. code identifiers: namespaces, package names, app names, CLI names, API routes, event topics, database names, table prefixes
5. configuration keys: env var names only, config section names, feature flags, domain names, ports, hosts, queue/topic names
6. git history: commit messages, renames, branch names, tags, merge messages, file-level history
7. test fixtures and golden data
8. generated artifacts or old migration notes
9. references to `FlexNetOS`, `flexnetos`, `FLEXNETOS`, `lifeos`, `LifeOS`, `LIFEOS`, `FlexNet`, `Life OS`

## Required analysis questions

Answer these with proof or `UNKNOWN`:

- Was FlexNetOS a fork, rename, wrapper, replacement, migration target, prototype, deployment packaging, or unrelated repo?
- Was lifeos present as a predecessor, dependency, submodule, namespace, archived copy, or comparison target?
- What runtime/application role did FlexNetOS have?
- What services, jobs, APIs, UI apps, databases, queues, schedulers, or infrastructure did it define or depend on?
- What business or operational capability did it appear to serve?
- Why might the path `FlexNetOS` have been used instead of `lifeos`?
- What evidence contradicts the leading explanation?
- What evidence is missing?

## Required outputs

Create or update:

```text
migration-artifacts/00-executive-summary/flexnetos-purpose-summary.md
migration-artifacts/01-current-state/flexnetos-vs-lifeos-evidence.md
migration-artifacts/01-current-state/flexnetos-path-resolution.md
migration-artifacts/01-current-state/flexnetos-reference-index.md
migration-artifacts/03-code-analysis/flexnetos-entrypoints.md
migration-artifacts/05-integrations/flexnetos-contracts.md
migration-artifacts/09-governance/flexnetos-open-questions.md
```

The executive summary must contain:

- Bottom-line answer
- Confidence level: High / Medium / Low
- Top 10 evidence items
- What would disprove the conclusion
- Next follow-up task recommendation

<!-- END prompts/FLEXNETOS_INVESTIGATION_PROMPT.md -->


<!-- BEGIN prompts/LINKING_AND_MEMORY_PROMPT.md -->
# Linking and Persistent Memory Prompt

## Goal

Build a persistent memory layer so future Codex migration tasks can understand the repository without rediscovery.

## Required files

```text
migration-artifacts/MIGRATION_MEMORY.md
migration-artifacts/index.md
migration-artifacts/artifact-manifest.json
migration-artifacts/artifact-manifest.md
migration-artifacts/evidence-register.md
migration-artifacts/link-graph.md
migration-artifacts/wiki-home.md
migration-artifacts/_meta/scan-runs.jsonl
migration-artifacts/_meta/artifact-status.tsv
```

## Wiki-style linking rules

- Every artifact must have a stable relative Markdown link from `migration-artifacts/index.md`.
- Every artifact must include a backlink to `../index.md` or `../../index.md`.
- Use relative links only; no absolute local file links in final Markdown.
- Every graph source file must link to its rendered SVG/PNG if generated.
- Every finding must link to evidence in `evidence-register.md` or cite a raw scan file path.

## Persistent memory sections

`MIGRATION_MEMORY.md` must include:

1. Project identity
2. Proven purpose of FlexNetOS
3. Relationship to lifeos
4. Current-state architecture summary
5. Runtime and deployment summary
6. Data and integration summary
7. Toolchain summary
8. Known risks and blockers
9. Artifact map
10. Open questions
11. Commands used to create memory
12. Last verified timestamp

## Link validation

Run the link/index helper after artifacts are written:

```bash
python3 <prompt-package>/helpers/artifact_manifest.py migration-artifacts
python3 <prompt-package>/helpers/make_wiki_index.py migration-artifacts
```

If links break, fix the links before finalizing.

<!-- END prompts/LINKING_AND_MEMORY_PROMPT.md -->


<!-- BEGIN prompts/ARTIFACT_CONTRACT_FULL.md -->
# Full Migration Artifact Contract

This file contains the complete artifact context that must not be reduced or omitted. It was sourced from the prior migration-artifact answer and preserved verbatim in `source/previous-migration-artifact-context.md`.

Generated: `2026-07-04T16:53:36Z`

## Non-reduction rule

Codex must build and link every artifact named below. If evidence is unavailable, create the artifact with `status: unknown` or `status: partial`, explain the evidence gap, and link it from the index.

---

For a migration project, the most useful artifacts are **maps, inventories, dependency graphs, validation evidence, and cutover controls**. The goal is to reduce unknowns: what exists, what depends on what, what breaks if moved, how data changes, how to prove parity, and how to roll back.

The best artifacts are **generated, version-controlled, diffable, and tied to real systems** rather than hand-drawn diagrams that rot.

## High-value migration artifacts

| Artifact                                      | What it answers                                                                                                 | Why it matters                                                                        |
| --------------------------------------------- | --------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| **System inventory**                          | What applications, services, jobs, databases, queues, APIs, reports, scripts, and schedulers exist?             | Prevents hidden systems from being missed.                                            |
| **Directory and subdirectory hierarchy tree** | Where is the code/config/data actually located?                                                                 | Useful for repo cleanup, ownership mapping, build discovery, and dead-code detection. |
| **Repository map**                            | Which repos exist, what they contain, who owns them, and how active they are?                                   | Helps split migration scope into manageable chunks.                                   |
| **Application/service dependency graph**      | Which services call which other services?                                                                       | Critical for sequencing migration waves.                                              |
| **Toolchain dependency tree**                 | What compilers, runtimes, package managers, SDKs, build tools, CI/CD tools, and deploy tools are required?      | Exposes version traps and platform incompatibilities.                                 |
| **Package/library dependency graph**          | Which packages depend on vulnerable, deprecated, or incompatible libraries?                                     | Useful for modernization, security, and upgrade planning.                             |
| **Runtime dependency map**                    | What does the app need at runtime: databases, env vars, secrets, filesystems, queues, third-party APIs, caches? | Prevents “works in build, fails in prod” problems.                                    |
| **Data flow graph**                           | How does data enter, move, transform, persist, and exit the system?                                             | Essential for data migration, compliance, and debugging.                              |
| **Database schema map**                       | What tables, columns, indexes, constraints, procedures, triggers, and views exist?                              | Needed for data migration and performance parity.                                     |
| **Data lineage map**                          | Where does each critical field originate, transform, and get consumed?                                          | Prevents silent data corruption.                                                      |
| **API contract catalog**                      | What endpoints/events exist, request/response schemas, auth, consumers, versions?                               | Prevents breaking integrations.                                                       |
| **Event/message contract map**                | What topics, queues, payloads, producers, consumers, retry semantics, and DLQs exist?                           | Critical for async architectures.                                                     |
| **Code ownership map**                        | Who owns each module/service/data pipeline?                                                                     | Speeds up decisions and incident resolution.                                          |
| **Code map for debugging**                    | Main execution paths, entry points, error paths, logging points, external calls.                                | Helps teams troubleshoot migrated workloads faster.                                   |
| **Environment matrix**                        | Dev/stage/prod differences: versions, configs, infra, secrets, endpoints, feature flags.                        | Finds environment drift before cutover.                                               |
| **Configuration inventory**                   | Env vars, config files, flags, secrets references, runtime parameters.                                          | Configuration is often where migrations fail.                                         |
| **Infrastructure topology map**               | Compute, networking, storage, DNS, load balancers, firewalls, certificates.                                     | Needed for cloud, data center, Kubernetes, or platform migrations.                    |
| **IAM/security access matrix**                | Users, roles, service accounts, permissions, secrets, certificates, tokens.                                     | Prevents broken auth and over-permissioned systems.                                   |
| **Observability map**                         | Logs, metrics, traces, dashboards, alerts, SLOs, runbooks.                                                      | Migration is dangerous without visibility.                                            |
| **Business process map**                      | Which business workflows depend on which systems?                                                               | Helps prioritize what truly matters.                                                  |
| **Migration wave plan**                       | What moves together, in what order, and why?                                                                    | Converts analysis into execution.                                                     |
| **Cutover checklist**                         | Exact steps for go-live.                                                                                        | Reduces human error during high-pressure migration windows.                           |
| **Rollback plan**                             | How to undo or fail back safely.                                                                                | Non-negotiable for serious migrations.                                                |
| **Validation and reconciliation reports**     | Did the migrated system produce the same outputs? Did all records move?                                         | The evidence layer. Without this, migration success is just a claim.                  |
| **Test coverage matrix**                      | What has unit, integration, regression, performance, security, and user acceptance coverage?                    | Shows where risk remains.                                                             |
| **Risk register**                             | Known risks, owners, mitigations, severity, status.                                                             | Keeps ugly truths visible.                                                            |
| **Decision log / ADRs**                       | What decisions were made, by whom, and why?                                                                     | Prevents re-litigating architecture choices later.                                    |

---

## The most useful artifacts by migration type

### 1. **Codebase migration**

Useful when moving languages, frameworks, monoliths to services, legacy apps to modern platforms, or repos to new standards.

Key artifacts:

| Artifact                     | Use                                                                                                         |
| ---------------------------- | ----------------------------------------------------------------------------------------------------------- |
| **Codebase hierarchy graph** | Shows folders, modules, packages, entry points, generated code, tests, configs.                             |
| **Import/dependency graph**  | Shows module-to-module dependencies. Great for identifying circular dependencies and extraction boundaries. |
| **Call graph**               | Shows function/method invocation paths. Useful for debugging and refactoring.                               |
| **Dead-code report**         | Identifies unused modules, functions, routes, jobs, and feature flags.                                      |
| **Hotspot map**              | Combines code churn, bug history, complexity, and ownership. Shows risky areas.                             |
| **Build graph**              | Shows how source code becomes artifacts: binaries, containers, packages, deployables.                       |
| **Runtime entry-point map**  | Shows app start commands, workers, jobs, handlers, APIs, and scheduled tasks.                               |
| **Compatibility matrix**     | Shows supported runtime versions, framework versions, OS versions, and package constraints.                 |

**Especially useful:**
A **code map for debugging** that includes:

```text
Request/Event Entry Point
  → Controller/Handler
    → Service Layer
      → Business Logic
        → Repository/Data Access
          → Database/External API/Queue
    → Error Handling
    → Logging/Tracing
    → Response/Event Output
```

This is more useful than a pretty architecture diagram because it shows where failures actually happen.

---

### 2. **Data migration**

Useful when moving databases, warehouses, lakes, ETL/ELT pipelines, schemas, or reporting layers.

Key artifacts:

| Artifact                          | Use                                                                                        |
| --------------------------------- | ------------------------------------------------------------------------------------------ |
| **Source-to-target mapping**      | Maps old fields/tables to new fields/tables.                                               |
| **Transformation rules catalog**  | Defines type conversions, normalization, joins, defaults, deduplication, enrichment.       |
| **Data lineage graph**            | Shows where data originates and where it flows.                                            |
| **Data quality profile**          | Null rates, duplicates, invalid values, outliers, referential integrity.                   |
| **Reconciliation report**         | Compares source vs. target record counts, checksums, aggregates, business totals.          |
| **Schema diff report**            | Shows changes between old and new schema.                                                  |
| **Critical field inventory**      | Identifies fields that drive billing, reporting, compliance, auth, or customer experience. |
| **Backfill plan**                 | Defines historical data load strategy.                                                     |
| **Incremental sync plan**         | Defines CDC, dual writes, event replay, or batch sync strategy.                            |
| **Data retention/compliance map** | Shows PII, PHI, PCI, retention rules, deletion obligations.                                |

The artifact that usually saves the project is the **reconciliation report**. A migration is not done because the pipeline ran. It is done when the numbers prove it.

---

### 3. **Cloud or infrastructure migration**

Useful when moving from on-prem to cloud, cloud-to-cloud, VMs to containers, or legacy hosting to Kubernetes/serverless.

Key artifacts:

| Artifact                               | Use                                                                        |
| -------------------------------------- | -------------------------------------------------------------------------- |
| **Infrastructure topology map**        | Shows compute, networking, storage, ingress, DNS, security boundaries.     |
| **Network dependency map**             | Ports, protocols, firewalls, allowlists, private links, VPNs, VPCs/VNETs.  |
| **Environment parity matrix**          | Compares old vs. new dev/stage/prod environments.                          |
| **Resource inventory**                 | Servers, VMs, containers, databases, buckets, volumes, certificates.       |
| **IaC coverage report**                | Shows what is codified vs. manually configured.                            |
| **Secrets and certificates inventory** | Expirations, owners, rotation requirements, consuming services.            |
| **Capacity baseline**                  | CPU, memory, storage, IOPS, throughput, latency, peak load.                |
| **Cost baseline and forecast**         | Current cost vs. expected target cost.                                     |
| **DR/backup map**                      | Recovery point objective, recovery time objective, backups, restore tests. |
| **Security control matrix**            | Encryption, network policies, IAM, audit logs, vulnerability controls.     |

The dangerous gaps are usually **DNS, certificates, firewall rules, secrets, and undocumented cron jobs**.

---

### 4. **Application integration migration**

Useful when replacing SaaS systems, ERP/CRM platforms, identity providers, payment systems, messaging layers, or API gateways.

Key artifacts:

| Artifact                            | Use                                                                              |
| ----------------------------------- | -------------------------------------------------------------------------------- |
| **Integration catalog**             | Lists every inbound/outbound integration.                                        |
| **API contract map**                | Shows endpoints, methods, schemas, auth, rate limits, consumers.                 |
| **Webhook/event map**               | Shows event producers, consumers, payloads, retries, delivery guarantees.        |
| **Third-party dependency register** | Vendors, credentials, SLAs, rate limits, support contacts.                       |
| **Auth flow diagram**               | OAuth, SAML, OIDC, API keys, service accounts, token lifetimes.                  |
| **Failure-mode map**                | What happens on timeout, retry, partial failure, duplicate message, bad payload. |
| **Backward compatibility plan**     | Versioning, adapter layers, proxies, shims, redirects.                           |

For integrations, the best artifact is often a **consumer map**:

```text
API / Event / File Feed
  → Producer
  → Consumers
  → Payload Contract
  → Auth Method
  → Frequency
  → Failure Behavior
  → Business Owner
  → Technical Owner
  → Migration Status
```

---

## Artifacts specifically matching your examples

### Directory and subdirectory hierarchy graph

Useful for:

* Discovering project structure.
* Finding duplicated modules.
* Identifying generated files.
* Separating source, tests, configs, docs, scripts, and build outputs.
* Planning repo splits or consolidation.

Example shape:

```text
repo-root/
├── apps/
│   ├── web/
│   ├── api/
│   └── worker/
├── packages/
│   ├── shared-models/
│   ├── auth/
│   └── logging/
├── infra/
│   ├── terraform/
│   └── k8s/
├── scripts/
├── tests/
└── docs/
```

Better version: a graph that tags each folder with:

```text
owner
language
runtime
build tool
deployment target
test coverage
last modified
dependency count
risk score
```

---

### Toolchain dependency tree

Useful for identifying what must exist before the app can build, test, deploy, and run.

Example:

```text
Application
├── Runtime
│   ├── Node 20
│   ├── Python 3.12
│   └── Java 21
├── Build
│   ├── pnpm
│   ├── Maven
│   └── Docker
├── CI/CD
│   ├── GitHub Actions
│   ├── Artifact Registry
│   └── Deployment Runner
├── Infrastructure
│   ├── Terraform
│   ├── Kubernetes
│   └── Helm
└── Security
    ├── SAST
    ├── Dependency scanning
    └── Container scanning
```

For a migration, this should include **version constraints** and **upgrade blockers**.

---

### Code map for debugging

Useful for understanding how to troubleshoot the migrated system.

A good debugging map includes:

| Layer          | What to document                                                  |
| -------------- | ----------------------------------------------------------------- |
| Entry points   | HTTP routes, CLI commands, jobs, queue consumers, event handlers. |
| Control flow   | Main execution path through controllers/services/modules.         |
| External calls | Databases, APIs, queues, caches, filesystems.                     |
| Failure points | Known exception sources, retry paths, timeout risks.              |
| Logs           | Log statements, correlation IDs, trace IDs, severity levels.      |
| Metrics        | Counters, latency, errors, saturation, business KPIs.             |
| Alerts         | What fires, who receives it, what action to take.                 |
| Runbooks       | How to diagnose and recover.                                      |

The useful version is not just “how the code is organized.” It is **how failure moves through the system**.

---

### Data flow graph

Useful for seeing how data actually behaves across systems.

Example:

```text
Customer submits form
  → Web App
  → API Gateway
  → Customer Service
  → Validation Layer
  → PostgreSQL
  → Change Data Capture
  → Event Bus
  → Analytics Pipeline
  → Data Warehouse
  → BI Dashboard
```

Enhance it with:

```text
data owner
system owner
schema
PII classification
latency expectation
retention policy
transformation logic
validation rule
downstream consumers
```

---

## Minimum viable artifact set

For a serious migration, I would not start execution without these:

| Priority | Artifact                             |
| -------: | ------------------------------------ |
|        1 | **System inventory**                 |
|        2 | **Dependency graph**                 |
|        3 | **Data flow graph**                  |
|        4 | **Source-to-target mapping**         |
|        5 | **Environment/configuration matrix** |
|        6 | **Toolchain dependency tree**        |
|        7 | **API/event contract catalog**       |
|        8 | **Test and validation matrix**       |
|        9 | **Cutover checklist**                |
|       10 | **Rollback plan**                    |
|       11 | **Risk register**                    |
|       12 | **Decision log**                     |

That is the core set. Everything else is useful, but those are the artifacts that directly reduce migration failure risk.

---

## Advanced artifacts that are especially valuable

These are higher-effort, but they tend to expose the problems nobody wants to talk about.

| Artifact                             | Why it is powerful                                                               |
| ------------------------------------ | -------------------------------------------------------------------------------- |
| **Blast-radius map**                 | Shows what breaks if a service, table, queue, API, or credential fails.          |
| **Migration readiness scorecard**    | Scores each app/service/data domain against readiness criteria.                  |
| **Business capability map**          | Connects technical systems to actual business functions.                         |
| **Shadow traffic comparison report** | Compares old vs. new system behavior under real traffic.                         |
| **Golden dataset**                   | Known-good sample data used to validate transformations and outputs.             |
| **Parity dashboard**                 | Real-time comparison of old and new system metrics.                              |
| **Deprecation map**                  | Shows what gets retired, replaced, wrapped, or preserved.                        |
| **Exception inventory**              | Captures special cases, one-off scripts, manual processes, and tribal knowledge. |
| **Ownership/RACI matrix**            | Defines who approves, builds, validates, cuts over, and supports.                |
| **Technical debt ledger**            | Separates “must fix before migration” from “carry temporarily” from “delete.”    |

---

## Recommended artifact structure

A practical migration artifact repository might look like this:

```text
migration-artifacts/
├── 00-executive-summary/
├── 01-current-state/
│   ├── system-inventory.md
│   ├── architecture-current.md
│   ├── dependency-graph.md
│   ├── data-flow-current.md
│   └── risk-hotspots.md
├── 02-target-state/
│   ├── architecture-target.md
│   ├── platform-design.md
│   ├── security-model.md
│   └── operating-model.md
├── 03-code-analysis/
│   ├── repo-map.md
│   ├── directory-tree.md
│   ├── package-dependencies.md
│   ├── call-graph.md
│   └── dead-code-report.md
├── 04-data-migration/
│   ├── schema-map.md
│   ├── source-target-mapping.md
│   ├── transformation-rules.md
│   ├── data-quality-report.md
│   └── reconciliation-plan.md
├── 05-integrations/
│   ├── api-catalog.md
│   ├── event-catalog.md
│   ├── third-party-dependencies.md
│   └── auth-flows.md
├── 06-testing-validation/
│   ├── test-strategy.md
│   ├── regression-matrix.md
│   ├── performance-baseline.md
│   └── validation-evidence.md
├── 07-cutover/
│   ├── wave-plan.md
│   ├── cutover-checklist.md
│   ├── rollback-plan.md
│   └── communication-plan.md
├── 08-operations/
│   ├── observability-map.md
│   ├── runbooks.md
│   ├── alerting.md
│   └── incident-response.md
└── 09-governance/
    ├── decision-log.md
    ├── risk-register.md
    ├── ownership-matrix.md
    └── status-dashboard.md
```

---

## Best formats

| Artifact type         | Good formats                                        |
| --------------------- | --------------------------------------------------- |
| Trees and hierarchies | Markdown, Mermaid, Graphviz DOT, JSON               |
| Dependency graphs     | Graphviz, Mermaid, Neo4j, D3, Cytoscape             |
| Inventories           | CSV, spreadsheet, database table                    |
| API contracts         | OpenAPI, AsyncAPI, Postman collections              |
| Data mappings         | Spreadsheet, dbt docs, YAML, SQL metadata           |
| Architecture          | C4 diagrams, Mermaid, PlantUML, Structurizr         |
| Security/IAM          | Matrix tables, policy-as-code, JSON/YAML exports    |
| Test evidence         | CI reports, JUnit XML, coverage reports, dashboards |
| Cutover controls      | Markdown checklist, runbook, project board          |

---

## The artifact hierarchy I would use

For a migration, artifacts should ladder from **discovery** to **execution**:

```text
1. Inventory
   ↓
2. Dependency Map
   ↓
3. Risk Map
   ↓
4. Target Design
   ↓
5. Migration Plan
   ↓
6. Validation Plan
   ↓
7. Cutover Plan
   ↓
8. Rollback Plan
   ↓
9. Operational Runbook
   ↓
10. Decommission Plan
```

The most common mistake is overbuilding architecture diagrams and underbuilding validation artifacts. Pretty diagrams do not prove the migration worked. **Reconciliation, test evidence, parity dashboards, and rollback plans do.**

<!-- END prompts/ARTIFACT_CONTRACT_FULL.md -->


<!-- BEGIN prompts/MASTER_PROMPT.md -->
# MASTER PROMPT — Codex 5.4 Local Migration Artifact Builder for FlexNetOS

You are Codex 5.4 running locally through the Codex Rust CLI on Ubuntu 26.04+.

Your mission is to build a complete, linked, persistent migration-artifact knowledge base for the repository/workspace rooted at the resolved FlexNetOS path, then answer the follow-up investigation question:

> What was `FlexNetOS` used for at `~/home/flexnetos/FlexNetOS` instead of `~/home/flexnetos/lifeos`?

## Input context

The runner may provide these variables at the top of stdin:

```text
RUN_CONTEXT_FILE=<path>
PRIMARY_ROOT=<path>
COMPARE_ROOT=<path>
PROMPT_PACKAGE_DIR=<path>
```

Read the run context if present. Use these default candidates if missing:

```text
PRIMARY_ROOT_CANDIDATES:
  - ~/home/flexnetos/FlexNetOS
  - /home/flexnetos/FlexNetOS
COMPARE_ROOT_CANDIDATES:
  - ~/home/flexnetos/lifeos
  - /home/flexnetos/lifeos
```

## Absolute non-negotiables

- No simulation.
- No demo artifacts.
- No invented facts.
- No fabricated ownership, services, dependencies, business process, data stores, routes, or histories.
- Use real shell commands and real local files.
- Redact secrets.
- Do not delete files.
- Do not mutate production systems.
- Do not write to databases.
- Do not install packages without explicit approval.
- If a required artifact cannot be populated from evidence, create it anyway with `status: unknown` or `status: partial` and document the evidence gap.

## Model and execution requirements

1. Verify that you are using the requested orchestrator label: `codex-5.4`.
2. Verify that helper/subagent role configs resolve to `spark-5.3`.
3. Use multiple Spark 5.3 helpers/subagents. Spawn at least eight helpers:
   - `spark-filesystem-repo`
   - `spark-toolchain-deps`
   - `spark-code-runtime-debug`
   - `spark-data-schema-lineage`
   - `spark-infra-security-obs`
   - `spark-integrations-contracts`
   - `spark-migration-controls`
   - `spark-flexnetos-investigator`
4. Run background shell helpers before final artifact synthesis.
5. The shell helper to run is:

```bash
bash "$PROMPT_PACKAGE_DIR/helpers/background_scan.sh" "$PRIMARY_ROOT" "$COMPARE_ROOT"
```

If `PROMPT_PACKAGE_DIR` is absent, locate this prompt package by searching upward/current known paths and record the search commands.

## Required source files from the prompt package

Read and apply all of the following. Do not reduce them:

```text
prompts/EXECUTION_STYLE.md
prompts/ARTIFACT_CONTRACT_FULL.md
prompts/FLEXNETOS_INVESTIGATION_PROMPT.md
prompts/LINKING_AND_MEMORY_PROMPT.md
prompts/spark_helpers/*.md
source/previous-migration-artifact-context.md
expected-output/migration-artifacts-tree.md
```

## Work plan

### Phase 0 — Resolve paths and safety

- Resolve `PRIMARY_ROOT` and `COMPARE_ROOT` with `realpath`.
- Verify which candidate paths exist.
- Confirm git repository state.
- Create `migration-artifacts/` if absent.
- Create a run ID using UTC timestamp.
- Write `migration-artifacts/_meta/run-context.md`.

### Phase 1 — Background evidence scan

Run the background scanner. It must collect raw evidence in parallel and write a run directory under:

```text
migration-artifacts/_raw/<UTC_RUN_ID>/
```

Do not skip this phase unless the shell cannot run. If it cannot run, write a blocker artifact.

### Phase 2 — Spark 5.3 helper analysis

Spawn the eight Spark helpers. Give each helper the relevant prompt from `prompts/spark_helpers/` plus the raw scan directory path.

Each helper must produce:

```text
migration-artifacts/_spark/<helper-name>.md
migration-artifacts/_spark/<helper-name>.json
```

Wait for all helpers. If a helper fails, record the failure and continue with explicit gap markers.

### Phase 3 — Build every migration artifact

Create the artifact tree defined in `expected-output/migration-artifacts-tree.md` and the full artifact contract.

Every artifact must include this front matter:

```yaml
---
artifact_id: <stable-kebab-id>
title: <title>
status: complete | partial | unknown | blocked
generated_at_utc: <timestamp>
source_root: <resolved PRIMARY_ROOT>
compare_root: <resolved COMPARE_ROOT or UNKNOWN>
evidence_paths:
  - <relative evidence path or UNKNOWN>
last_verified_utc: <timestamp>
links:
  parent_index: <relative link>
  related: []
---
```

Every artifact must contain:

```text
## Verified findings
## Evidence ledger
## Unknowns / gaps
## Commands run
## Related artifacts
## Next actions
```

### Phase 4 — FlexNetOS purpose answer

Create the explicit answer artifacts:

```text
migration-artifacts/00-executive-summary/flexnetos-purpose-summary.md
migration-artifacts/01-current-state/flexnetos-vs-lifeos-evidence.md
migration-artifacts/01-current-state/flexnetos-path-resolution.md
migration-artifacts/01-current-state/flexnetos-reference-index.md
migration-artifacts/03-code-analysis/flexnetos-entrypoints.md
migration-artifacts/05-integrations/flexnetos-contracts.md
migration-artifacts/09-governance/flexnetos-open-questions.md
```

The summary must answer the question directly using evidence only. Use confidence levels and explain uncertainty.

### Phase 5 — Link, validate, and write persistent memory

Generate:

```text
migration-artifacts/MIGRATION_MEMORY.md
migration-artifacts/index.md
migration-artifacts/wiki-home.md
migration-artifacts/evidence-register.md
migration-artifacts/link-graph.md
migration-artifacts/artifact-manifest.json
migration-artifacts/artifact-manifest.md
migration-artifacts/_meta/artifact-status.tsv
```

Run:

```bash
python3 "$PROMPT_PACKAGE_DIR/helpers/artifact_manifest.py" migration-artifacts
python3 "$PROMPT_PACKAGE_DIR/helpers/make_wiki_index.py" migration-artifacts
```

Fix broken links or missing artifacts.

## Required artifacts to build and link

Build all artifacts from the full contract, including all high-value artifacts, all migration-type artifacts, all example-specific artifacts, the minimum viable set, all advanced artifacts, and the recommended repository structure.

At minimum, this means all files listed in `expected-output/migration-artifacts-tree.md` must exist and be linked.

## Final response required from Codex

At the end, report:

1. Whether the run completed, partially completed, or blocked.
2. The resolved FlexNetOS path.
3. The resolved lifeos comparison path.
4. The bottom-line answer to what FlexNetOS was used for.
5. Confidence level.
6. Top evidence links.
7. Artifact index path.
8. Persistent memory path.
9. Remaining blockers.

Do not include secrets. Do not claim completion if required artifacts are missing.

<!-- END prompts/MASTER_PROMPT.md -->

