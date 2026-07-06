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
