# Codex FlexNetOS Migration Artifact Prompt Package

Generated: `2026-07-04T16:53:36Z`

This package is designed for the OpenAI Codex Rust CLI running locally on Ubuntu 26.04+ to build a complete, linked migration-artifact knowledge base for the FlexNetOS follow-up task.

Primary follow-up goal:

> Determine what `FlexNetOS` was used for at `~/home/flexnetos/FlexNetOS` instead of `~/home/flexnetos/lifeos`, using only real local evidence.

The package intentionally includes the full migration artifact context from the prior answer in `source/previous-migration-artifact-context.md`. Do not remove or summarize that file before running Codex.

## Important version note

The prompt package uses these execution labels because the user requested them:

- Orchestrator: `Codex 5.4`
- Parallel helpers/subagents: `Spark 5.3`

The package treats those as local model labels/configuration names. The runner must verify that the local Codex installation can resolve them. If they are unavailable, Codex must stop and report that fact rather than silently downgrading.

## Package contents

```text
codex-flexnetos-migration-prompt-package/
├── README.md
├── RUN_WITH_CODEX.sh
├── INSTALL_IN_REPO.sh
├── codex/
│   ├── flexnetos-migration.config.toml
│   ├── AGENTS.md.template
│   └── agents/
├── helpers/
│   ├── background_scan.sh
│   ├── artifact_manifest.py
│   ├── make_wiki_index.py
│   └── redaction_patterns.txt
├── prompts/
│   ├── MASTER_PROMPT.md
│   ├── ARTIFACT_CONTRACT_FULL.md
│   ├── EXECUTION_STYLE.md
│   ├── FLEXNETOS_INVESTIGATION_PROMPT.md
│   ├── LINKING_AND_MEMORY_PROMPT.md
│   └── spark_helpers/
├── schemas/
│   ├── artifact_manifest.schema.json
│   ├── finding.schema.json
│   └── scan_run.schema.json
├── expected-output/
│   └── migration-artifacts-tree.md
└── source/
    └── previous-migration-artifact-context.md
```

## Use locally

From a terminal on the target machine:

```bash
cd /path/to/codex-flexnetos-migration-prompt-package
chmod +x RUN_WITH_CODEX.sh INSTALL_IN_REPO.sh helpers/background_scan.sh

# Preferred: run against the primary candidate path and compare with lifeos.
./RUN_WITH_CODEX.sh /home/flexnetos/FlexNetOS /home/flexnetos/lifeos
```

If your actual paths really are under `~/home/...`, pass the expanded paths explicitly:

```bash
./RUN_WITH_CODEX.sh "$HOME/home/flexnetos/FlexNetOS" "$HOME/home/flexnetos/lifeos"
```

The run should create or update a `migration-artifacts/` folder inside the selected repository/workspace and generate a persistent artifact memory file.

## What Codex must build

Codex must build every artifact named in the full context, including but not limited to:

- System inventory
- Directory and subdirectory hierarchy tree
- Repository map
- Application/service dependency graph
- Toolchain dependency tree
- Package/library dependency graph
- Runtime dependency map
- Data flow graph
- Database schema map
- Data lineage map
- API contract catalog
- Event/message contract map
- Code ownership map
- Code map for debugging
- Environment matrix
- Configuration inventory
- Infrastructure topology map
- IAM/security access matrix
- Observability map
- Business process map
- Migration wave plan
- Cutover checklist
- Rollback plan
- Validation and reconciliation reports
- Test coverage matrix
- Risk register
- Decision log / ADRs
- Codebase hierarchy graph
- Import/dependency graph
- Call graph
- Dead-code report
- Hotspot map
- Build graph
- Runtime entry-point map
- Compatibility matrix
- Source-to-target mapping
- Transformation rules catalog
- Data quality profile
- Schema diff report
- Critical field inventory
- Backfill plan
- Incremental sync plan
- Data retention/compliance map
- Network dependency map
- Environment parity matrix
- Resource inventory
- IaC coverage report
- Secrets and certificates inventory
- Capacity baseline
- Cost baseline and forecast
- DR/backup map
- Security control matrix
- Integration catalog
- API contract map
- Webhook/event map
- Third-party dependency register
- Auth flow diagram
- Failure-mode map
- Backward compatibility plan
- Consumer map
- Blast-radius map
- Migration readiness scorecard
- Business capability map
- Shadow traffic comparison report
- Golden dataset
- Parity dashboard
- Deprecation map
- Exception inventory
- Ownership/RACI matrix
- Technical debt ledger
- Persistent memory, artifact manifest, wiki index, and link graph

## Non-negotiables

- No simulation.
- No demo output.
- No fabricated architecture.
- No invented ownership, dependencies, services, APIs, data stores, or business logic.
- Every finding must cite a command output, file path, manifest, source file, git evidence, or an explicit `UNKNOWN` marker.
- Secrets must be redacted.
- Destructive changes are forbidden unless explicitly approved by the user outside this package.
