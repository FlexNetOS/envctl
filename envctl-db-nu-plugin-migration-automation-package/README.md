# envctl Database + nu_plugin Migration Automation Prompt Package

## Agent navigation + backtrace metadata

Metadata tag: README_NAV_BACKTRACE_2026-07-04

Package root: `~/envctl-db-nu-plugin-migration-automation-package`

History root: `~/envctl-db-nu-plugin-migration-automation-package/history`

Backtrace rule: Before changing execution behavior, review `history/v0` through `history/v5` and compare the current package state against prior manifests. Use `history/v0` as the restored baseline/backtrace anchor when validating upgrade-only intent.

### v0-v5 history anchors

- `history/v0/VERSION_MANIFEST.md`
- `history/v0/Migration_Project_Artifacts.md`
- `history/v1/VERSION_MANIFEST.md`
- `history/v1/README.md`
- `history/v2/VERSION_MANIFEST.md`
- `history/v2/README.md`
- `history/v3/VERSION_MANIFEST.md`
- `history/v3/README.md`
- `history/v4/VERSION_MANIFEST.md`
- `history/v4/README.md`
- `history/v5/VERSION_MANIFEST.md`
- `history/v5/README.md`

### Codex entry pointers

- `CODEX_FINAL_EXECUTION_PROMPT.md`
- `prompts/CODEX_FINAL_EXECUTION_PROMPT.md`
- `execution-framework/generated/task_graph.csv`
- `execution-framework/generated/execution_manifest.json`
- `execution-framework/generated/execution_packets/`
- `execution-framework/proof_records/proof_ledger.jsonl`

Agent quick path: `history/` → `CODEX_FINAL_EXECUTION_PROMPT.md` → `execution-framework/generated/task_graph.csv` → `execution-framework/generated/execution_manifest.json` → `execution-framework/generated/execution_packets/` → `execution-framework/state/goal_loop_state.json` → `execution-framework/proof_records/proof_ledger.jsonl` → `execution-framework/generated/final_verification_report.json`.

No-gap status: `execution-framework/generated/live_drive_gap_closure_report.json` reports `pass_no_gaps_drive_live_synchronized`.

Generated: `2026-07-04T17:45:00Z`

This package extends the prior `codex-flexnetos-migration-prompt-package` into an additive implementation prompt package for two connected codebases:

1. **envctl database tooling** — the persistent migration automation engine, source of truth, event ledger, artifact store, replay layer, approval queue, validation ledger, and reproducibility substrate.
2. **`nu_plugin` CLI/user layer** — the Nushell plugin surface that exposes live visuals, status, approvals, artifact browsing, graph views, and agent/human control commands.

The package is intended for Codex running locally on Ubuntu 26.04+ against real `envctl` and `nu_plugin` repositories. It does not assume their exact current layout. Codex must inspect the actual repos before editing.

## Bottom-line strategy

Do **contract-first parallel implementation**.

Do not fully implement the FlexNetOS prompt package first as a one-off, and do not build envctl tooling blindly without proving it can execute the package. Codex should:

1. Extract and lock the migration artifact contract from the existing FlexNetOS package.
2. Add envctl database tables, event sourcing, target descriptors, migration recipes, artifact contracts, evidence storage, approvals, checkpoints, rollbacks, replay, and validation records.
3. Build a thin envctl adapter that can import and run the prior package as an external artifact-generation bundle.
4. In parallel, build the Nushell plugin protocol/commands that read from and append controlled actions into envctl's migration database.
5. Replace shell-oriented collectors with native envctl collectors incrementally, only after the adapter proves parity.

This sequencing avoids the two worst outcomes: a one-off migration package that cannot be reproduced, or a polished database tool that cannot execute the real migration contract.

## Package contents

```text
envctl-db-nu-plugin-migration-automation-package/
├── README.md
├── RUN_WITH_CODEX_ENVCTL.sh
├── INSTALL_IN_REPOS.sh
├── PROMPT_PACKAGE_COMBINED.md
├── PACKAGE_MANIFEST.json
├── codex/
│   ├── envctl-nu-plugin-migration.config.toml
│   ├── AGENTS.envctl.md.template
│   ├── AGENTS.nu_plugin.md.template
│   └── agents/
├── prompts/
│   ├── MASTER_PROMPT_ENVCTL_DB_NU_PLUGIN.md
│   ├── STRATEGY_DECISION.md
│   ├── GAP_ANALYSIS.md
│   ├── UTILIZE_FLEXNETOS_PACKAGE.md
│   ├── DATABASE_FEATURE_SPEC.md
│   ├── NU_PLUGIN_CONTRACT.md
│   ├── AGENT_CONTROL_PROTOCOL.md
│   ├── LIVE_VISUALS_AND_HUMAN_CONTROL.md
│   ├── ANY_TARGET_EXTENSION_SPEC.md
│   ├── SECURITY_REPRODUCIBILITY_MODEL.md
│   ├── IMPLEMENTATION_PHASES.md
│   ├── ACCEPTANCE_CRITERIA.md
│   ├── ISSUE_ENVCTL.md
│   ├── ISSUE_NU_PLUGIN.md
│   ├── ISSUE_SHARED_PROTOCOL.md
│   └── spark_helpers/
├── specs/
├── schemas/
├── sql/
├── examples/
├── helpers/
├── expected-output/
└── source/
    ├── codex-flexnetos-migration-prompt-package/
    ├── previous-migration-artifact-context.md
    ├── current-uploaded-context.md
    └── current-user-request.md
```

## Use locally

```bash
cd /path/to/envctl-db-nu-plugin-migration-automation-package
chmod +x RUN_WITH_CODEX_ENVCTL.sh INSTALL_IN_REPOS.sh helpers/*.sh

./RUN_WITH_CODEX_ENVCTL.sh   --envctl-repo /path/to/envctl   --nu-plugin-repo /path/to/nu_plugin   --flexnetos-package ./source/codex-flexnetos-migration-prompt-package
```

The runner creates a run context and invokes `codex exec` with the combined master prompt. Codex must inspect both repositories and then make real, reviewable changes.

## Non-negotiables

- No simulation.
- No fabricated repo structure.
- No invented database backend.
- No destructive migrations without explicit approval.
- No secret exposure.
- Every implementation decision must be backed by inspected repo evidence.
- The envctl database is the durable source of truth.
- The Nushell plugin is a controlled command/visualization surface, not an independent source of truth.
- Reproducibility requires target descriptor + recipe + artifact contract + event log + evidence hashes + tool versions.

## Deliverables Codex must produce in the target repositories

Codex must implement or scaffold real code/docs/tests according to the actual repo architecture:

- envctl migration automation database schema/migrations.
- envctl artifact contract registry.
- envctl target descriptor parser and validator.
- envctl migration recipe loader.
- envctl operation/event ledger.
- envctl approval/checkpoint/rollback/replay model.
- envctl import/execution adapter for the prior FlexNetOS prompt package.
- envctl command/API layer for agents.
- `nu_plugin` commands for live status, visual tables, graphs, approvals, replay, artifacts, and run control.
- Shared protocol schemas between envctl and `nu_plugin`.
- Tests proving run creation, event append, artifact registration, approval flow, replay, and plugin command output.
- GitHub issue text updates for envctl, nu_plugin, and the shared protocol boundary.

## Verification

Before packaging or opening PRs, Codex must run the repository-native checks it discovers, plus package helper checks:

```bash
python3 helpers/validate_package.py .
python3 helpers/package_manifest.py . > PACKAGE_MANIFEST.json
```

If repo-native tests cannot run because dependencies are missing, Codex must record the exact blocker and the command that failed.
