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
