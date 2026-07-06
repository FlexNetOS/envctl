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
