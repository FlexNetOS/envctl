# FlexNetOS Adapter Recipe

Status: `validated`
Task: `REQ-202_FLEXNETOS_ADAPTER_RECIPE`
Recipe ID: `flexnetos-codex-package-target-adapter`
Version: `1.0.0`

## Goal

Convert the earlier FlexNetOS Codex migration package into a reusable envctl migration target adapter that stays repo-scoped, replay-aware, and human-approved before any target mutation.

## Inputs

- Prior package source: `source/codex-flexnetos-migration-prompt-package/**`
- Comparison evidence: `REQ-201_FLEXNETOS_LIFEOS_COMPARISON`
- Replay semantics: `REQ-027_ENVCTL_REPLAY_ENGINE`
- Read-only docs: `${ENVCTL_REPO}/docs/**`, `${NU_PLUGIN_REPO}/docs/**`, `${MIGRATION_TARGET_ROOT}/docs/**`

## Execution Model

- Target descriptor: `flexnetos-vs-lifeos`
- Repo target: `repo_a`
- Repo path reference: `${ENVCTL_REPO}`
- Filesystem scope: `repo`
- Human approval required: `true`
- Verification command: `python3 scripts/verify_flexnetos_adapter_recipe.py`

## Phase Plan

| phase | approval gate | operation count | focus |
|---|---|---:|---|
| `01-ingest-evidence` | `no` | `2` | link-prior-package-inputs, capture-flexnetos-comparison-findings |
| `02-render-adapter` | `no` | `2` | render-adapter-recipe, register-adapter-for-envctl |
| `03-verify-replay-readiness` | `no` | `2` | validate-adapter-contract, prove-replay-compatibility |
| `04-approved-apply` | `yes` | `2` | operator-review-target-docs, apply-flexnetos-target-adapter |

## Safety

- Writes stay limited to the packet-owned execution-framework outputs.
- Apply work remains behind the `04-approved-apply` gate.
- Blocked paths remain excluded: `**/.env`, `**/secrets/**`, `**/private_keys/**`, `**/*.pem`, `**/*.key`.
- Replay compatibility is required before apply compatibility is claimed.

## Packet Alignment

- Packet command template: `codex exec < generated/execution_packets/REQ-202_FLEXNETOS_ADAPTER_RECIPE.json`
- Completion gate: `proof exists, validation passes, no secret exposure`
- Proof path: `proof_records/REQ-202_FLEXNETOS_ADAPTER_RECIPE.proof.json`
- Validation report: `generated/flexnetos_adapter_recipe_validation_report.json`

## Notes

- This adapter recipe intentionally references external repo docs as read-only runtime inputs; the recipe package itself does not widen write scope into those repos.
- The apply phase is intentionally abstracted as envctl-controlled execution so the same recipe can be reused against future FlexNetOS migration targets without changing the execution-framework package.
