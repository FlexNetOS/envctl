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

## Reusable Runtime Contract

Supply `package_root`, `package_name`, `target_descriptor`, and `target_id` to execute this adapter. Before import, the package must contain: `PACKAGE_MANIFEST.json`, `prompts/ARTIFACT_CONTRACT_FULL.md`, `expected-output/migration-artifacts-tree.md`, `prompts/MASTER_PROMPT.md`.

```text
envctl migration package inspect {package_root}
envctl migration package import {package_root} --name {package_name}
envctl migration target add --descriptor {target_descriptor}
envctl migration plan --target {target_id} --contract {contract_id} --recipe {recipe_id}
envctl migration run {plan_id} --mode approval-gated
envctl migration events {run_id}
envctl migration artifacts {run_id}
envctl migration replay {run_id} --verify-hashes
envctl migration export {run_id} --format json
```

The `run` command remains approval-gated; no command here authorizes a target mutation without the recorded human decision.

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
