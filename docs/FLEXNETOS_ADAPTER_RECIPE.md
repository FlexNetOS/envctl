# FlexNetOS Adapter Recipe

Status: `validated contract`; apply remains approval-gated  
Task: `REQ-202_FLEXNETOS_ADAPTER_RECIPE`  
Recipe ID: `flexnetos-codex-package-target-adapter`  
Version: `1.0.0`

## Goal

Make the prior Codex FlexNetOS migration package executable as a reusable envctl migration-target adapter. The adapter is repo-scoped, replay-aware, and cannot mutate a target until an operator approval is recorded.

## Inputs and contract

- Prior package: `source/codex-flexnetos-migration-prompt-package/**`
- Comparison dependency: `REQ-201_FLEXNETOS_LIFEOS_COMPARISON`
- Replay dependency: `REQ-027_ENVCTL_REPLAY_ENGINE`
- Read-only documentation inputs: `${ENVCTL_REPO}/docs/**`, `${NU_PLUGIN_REPO}/docs/**`, and `${MIGRATION_TARGET_ROOT}/docs/**`
- Write scope: envctl run ledger, recipe registry, and task-owned generated artifacts only.
- Excluded paths: `**/.env`, `**/secrets/**`, `**/private_keys/**`, `**/*.pem`, and `**/*.key`.

## Execution plan

| Phase | Gate | Operations |
|---|---|---|
| `01-ingest-evidence` | no | link prior inputs; import comparison findings |
| `02-render-adapter` | no | render recipe; register target adapter |
| `03-verify-replay-readiness` | no | validate contract; verify dry-run replay compatibility |
| `04-approved-apply` | yes | operator reviews target docs; envctl applies the target adapter |

The `04-approved-apply` phase contains the only `R4`/`R5` operations. It requires an approval record before apply and provides the rollback checkpoint `history/pre_execution_framework_manifest.json`.

## Use

Validate and render with `python3 scripts/verify_flexnetos_adapter_recipe.py`. An envctl executor then consumes `generated/flexnetos_adapter_recipe.json`; it must enforce the approval gate before `apply-flexnetos-target-adapter`.

## Evidence

The verifier emits `generated/flexnetos_adapter_recipe_validation_report.json`, `proof_records/REQ-202_FLEXNETOS_ADAPTER_RECIPE.proof.json`, and a heartbeat. Dependency inputs are intentionally read-only and may be materialized by the caller's envctl workspace.
