# Configuration Inventory

Generated: `2026-07-04T23:23:26+00:00`
Task: `ART-115_CONFIG_INVENTORY`
Target root: `/home/flexnetos/FlexNetOS`

## Summary

| metric | value |
|---|---:|
| `scanned_files` | 60000 |
| `scanned_text_files` | 37344 |
| `config_file_count` | 8052 |
| `env_var_reference_count` | 16332 |
| `feature_flag_reference_count` | 5188 |
| `secret_reference_count` | 1409 |
| `blocked_path_count` | 7 |

## Config Files

| path | kind | secret-named |
|---|---|---:|
| `.gitignore` | `.gitignore` | false |
| `AGENTS.md` | `md` | false |
| `release-baseline.json` | `json` | false |
| `src/Cargo.toml` | `toml` | false |
| `src/beads_rust/.agent-mail.yaml` | `yaml` | false |
| `src/beads_rust/.beads/.gitignore` | `.gitignore` | false |
| `src/beads_rust/.beads/config.yaml` | `yaml` | false |
| `src/beads_rust/.beads/metadata.json` | `json` | false |
| `src/beads_rust/.cargo/audit.toml` | `toml` | false |
| `src/beads_rust/.github/dependabot.yml` | `yml` | false |
| `src/beads_rust/.github/workflows/audit.yml` | `yml` | false |
| `src/beads_rust/.github/workflows/ci.yml` | `yml` | false |
| `src/beads_rust/.github/workflows/conformance.yml` | `yml` | false |
| `src/beads_rust/.github/workflows/doctor.yml` | `yml` | false |
| `src/beads_rust/.github/workflows/e2e-full.yml` | `yml` | false |
| `src/beads_rust/.github/workflows/notify-acfs.yml` | `yml` | false |
| `src/beads_rust/.github/workflows/release.yml` | `yml` | false |
| `src/beads_rust/.github/workflows/update-package-manifests.yml` | `yml` | false |
| `src/beads_rust/.gitignore` | `.gitignore` | false |
| `src/beads_rust/AGENTS.md` | `md` | false |
| `src/beads_rust/Cargo.toml` | `toml` | false |
| `src/beads_rust/agent_baseline/errors/show_not_found.json` | `json` | false |
| `src/beads_rust/agent_baseline/examples/list_limit3.json` | `json` | false |
| `src/beads_rust/agent_baseline/examples/ready.json` | `json` | false |
| `src/beads_rust/agent_baseline/examples/show_one.json` | `json` | false |
| `src/beads_rust/agent_baseline/examples/version.json` | `json` | false |
| `src/beads_rust/agent_baseline/schemas/cli_schema.json` | `json` | false |
| `src/beads_rust/agent_baseline/schemas/schema_all.json` | `json` | false |
| `src/beads_rust/agent_baseline/schemas/schema_error.json` | `json` | false |
| `src/beads_rust/agent_baseline/schemas/schema_issue_details.json` | `json` | false |
| `src/beads_rust/docs/agent/AGENTS.md` | `md` | false |
| `src/beads_rust/flake.nix` | `nix` | false |
| `src/beads_rust/fuzz/Cargo.toml` | `toml` | false |
| `src/beads_rust/fuzz/corpus/merge_issue/close_vs_edit.json` | `json` | false |
| `src/beads_rust/fuzz/corpus/merge_issue/delete_vs_update.json` | `json` | false |
| `src/beads_rust/fuzz/corpus/merge_issue/dependency_tombstone.json` | `json` | false |
| `src/beads_rust/fuzz/corpus/merge_issue/equal_timestamp_conflict.json` | `json` | false |
| `src/beads_rust/fuzz/corpus/merge_issue/prefix_mismatch.json` | `json` | false |
| `src/beads_rust/packaging/scoop/br.json` | `json` | false |
| `src/beads_rust/rust-toolchain.toml` | `toml` | false |
| `src/beads_rust/sample_beads_db_files/repro_beadsrust_import_write.M6eaGY/import.json` | `json` | false |
| `src/beads_rust/sample_beads_db_files/repro_beadsrust_import_write.M6eaGY/update.json` | `json` | false |
| `src/beads_rust/scripts/dev-local-frankensqlite.toml` | `toml` | false |
| `src/beads_rust/src/config/mod.rs` | `rs` | false |
| `src/beads_rust/src/config/routing.rs` | `rs` | false |
| `src/beads_rust/tarpaulin.toml` | `toml` | false |
| `src/beads_rust/temp_test/.beads/.gitignore` | `.gitignore` | false |
| `src/beads_rust/temp_test/.beads/config.yaml` | `yaml` | false |
| `src/beads_rust/temp_test/.beads/metadata.json` | `json` | false |
| `src/beads_rust/temp_test_2/.beads/.gitignore` | `.gitignore` | false |
| `src/beads_rust/temp_test_2/.beads/config.yaml` | `yaml` | false |
| `src/beads_rust/temp_test_2/.beads/metadata.json` | `json` | false |
| `src/beads_rust/tests/fixtures/json_baseline/blocked.json` | `json` | false |
| `src/beads_rust/tests/fixtures/json_baseline/comments_list.json` | `json` | false |
| `src/beads_rust/tests/fixtures/json_baseline/count.json` | `json` | false |
| `src/beads_rust/tests/fixtures/json_baseline/dep_list.json` | `json` | false |
| `src/beads_rust/tests/fixtures/json_baseline/doctor.json` | `json` | false |
| `src/beads_rust/tests/fixtures/json_baseline/label_list.json` | `json` | false |
| `src/beads_rust/tests/fixtures/json_baseline/label_list_all.json` | `json` | false |
| `src/beads_rust/tests/fixtures/json_baseline/list.json` | `json` | false |
| `src/beads_rust/tests/fixtures/json_baseline/list_all.json` | `json` | false |
| `src/beads_rust/tests/fixtures/json_baseline/list_priority_0_1.json` | `json` | false |
| `src/beads_rust/tests/fixtures/json_baseline/ready.json` | `json` | false |
| `src/beads_rust/tests/fixtures/json_baseline/search.json` | `json` | false |
| `src/beads_rust/tests/fixtures/json_baseline/show_multiple.json` | `json` | false |
| `src/beads_rust/tests/fixtures/json_baseline/show_single.json` | `json` | false |
| `src/beads_rust/tests/fixtures/json_baseline/stats.json` | `json` | false |
| `src/beads_rust/tests/fixtures/json_baseline/version.json` | `json` | false |
| `src/beads_rust/tests/fixtures/workspace_failures/corrupt_db_text/.beads/.br_history/issues.20260312_113748_481446452.jsonl.meta.json` | `json` | false |
| `src/beads_rust/tests/fixtures/workspace_failures/corrupt_db_text/.beads/.gitignore` | `.gitignore` | false |
| `src/beads_rust/tests/fixtures/workspace_failures/corrupt_db_text/.beads/config.yaml` | `yaml` | false |
| `src/beads_rust/tests/fixtures/workspace_failures/corrupt_db_text/.beads/metadata.json` | `json` | false |
| `src/beads_rust/tests/fixtures/workspace_failures/corrupt_db_text/beads/.br_history/issues.20260312_113748_481446452.jsonl.meta.json` | `json` | false |
| `src/beads_rust/tests/fixtures/workspace_failures/corrupt_db_text/beads/.gitignore` | `.gitignore` | false |
| `src/beads_rust/tests/fixtures/workspace_failures/corrupt_db_text/beads/config.yaml` | `yaml` | false |
| `src/beads_rust/tests/fixtures/workspace_failures/corrupt_db_text/beads/metadata.json` | `json` | false |
| `src/beads_rust/tests/fixtures/workspace_failures/corrupt_db_text/fixture.json` | `json` | false |
| `src/beads_rust/tests/fixtures/workspace_failures/db_jsonl_disagreement/.beads/.br_history/issues.20260312_113748_481446452.jsonl.meta.json` | `json` | false |
| `src/beads_rust/tests/fixtures/workspace_failures/db_jsonl_disagreement/.beads/.gitignore` | `.gitignore` | false |
| `src/beads_rust/tests/fixtures/workspace_failures/db_jsonl_disagreement/.beads/config.yaml` | `yaml` | false |

## Environment Variable References

| name | refs | sample paths |
|---|---:|---|
| `A100` | 3 | `src/hermes-agent/skills/research/research-paper-writing/SKILL.md`, `src/hermes-agent/website/docs/user-guide/skills/bundled/research/research-research-paper-writing.md`, `src/hermes-agent/website/i18n/zh-Hans/docusaurus-plugin-content-docs/current/user-guide/skills/bundled/research/research-research-paper-writing.md` |
| `A11` | 1 | `src/meta-ruvector/examples/wasm/ios/src/ios_capabilities.rs` |
| `A12` | 1 | `src/meta-ruvector/examples/wasm/ios/src/ios_capabilities.rs` |
| `A2A` | 4 | `src/envctl-codedb-config-to-tables/.handoff/loop/plan/findings/rules-policy-org-weave.md`, `src/envctl-nix-components-db/.handoff/loop/plan/findings/rules-policy-org-weave.md`, `src/envctl-nu-plugin-db-skill/.handoff/loop/plan/findings/rules-policy-org-weave.md` |
| `A5` | 3 | `src/hermes-agent/optional-skills/finance/lbo-model/SKILL.md`, `src/hermes-agent/website/docs/user-guide/skills/optional/finance/finance-lbo-model.md`, `src/hermes-agent/website/i18n/zh-Hans/docusaurus-plugin-content-docs/current/user-guide/skills/optional/finance/finance-lbo-model.md` |
| `A88` | 3 | `src/hermes-agent/optional-skills/finance/dcf-model/SKILL.md`, `src/hermes-agent/website/docs/user-guide/skills/optional/finance/finance-dcf-model.md`, `src/hermes-agent/website/i18n/zh-Hans/docusaurus-plugin-content-docs/current/user-guide/skills/optional/finance/finance-dcf-model.md` |
| `AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA` | 3 | `src/ruflo/v3/@claude-flow/plugin-agent-federation/__tests__/unit/wg-coordinator-integration.test.ts`, `src/ruflo/v3/@claude-flow/plugin-agent-federation/__tests__/unit/wg-mesh-service.test.ts`, `src/ruflo/v3/docs/adr/ADR-107-federation-tls.md` |
| `AAD` | 8 | `src/envctl-codedb-config-to-tables/crates/secrets-engine/src/lib.rs`, `src/envctl-codedb-config-to-tables/crates/secrets-engine/src/vault/crypto.rs`, `src/envctl-nix-components-db/crates/secrets-engine/src/lib.rs` |
| `AAD_LEN` | 4 | `src/envctl-codedb-config-to-tables/crates/secrets-engine/src/vault/aad.rs`, `src/envctl-nix-components-db/crates/secrets-engine/src/vault/aad.rs`, `src/envctl-nu-plugin-db-skill/crates/secrets-engine/src/vault/aad.rs` |
| `AB` | 3 | `src/hermes-agent/skills/research/research-paper-writing/SKILL.md`, `src/hermes-agent/website/docs/user-guide/skills/bundled/research/research-research-paper-writing.md`, `src/hermes-agent/website/i18n/zh-Hans/docusaurus-plugin-content-docs/current/user-guide/skills/bundled/research/research-research-paper-writing.md` |
| `ABB` | 1 | `src/hermes-agent/skills/creative/ascii-video/references/effects.md` |
| `ABC` | 6 | `src/hermes-agent/agent/browser_provider.py`, `src/hermes-agent/agent/image_gen_provider.py`, `src/hermes-agent/agent/transcription_provider.py` |
| `ABI` | 1 | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/enum_abi_multi_payload/src/main.rs` |
| `ABORTED` | 1 | `src/hermes-agent/optional-skills/mlops/slime/references/api-reference.md` |
| `ABSOLUTE_MAX_BYTES` | 1 | `src/hermes-agent/gateway/platforms/wecom.py` |
| `ABSTRACTS_FILE` | 1 | `src/meta-ruvector/crates/mcp-brain-server/scripts/pubmed_discover.sh` |
| `ACCELERATE_LOG_LEVEL` | 3 | `src/hermes-agent/optional-skills/mlops/simpo/SKILL.md`, `src/hermes-agent/website/docs/user-guide/skills/optional/mlops/mlops-simpo.md`, `src/hermes-agent/website/i18n/zh-Hans/docusaurus-plugin-content-docs/current/user-guide/skills/optional/mlops/mlops-simpo.md` |
| `ACCELERATOR_TYPE` | 1 | `src/meta-ruvector/examples/google-cloud/src/cuda.rs` |
| `ACCENT` | 6 | `src/hermes-agent/skills/creative/manim-video/SKILL.md`, `src/hermes-agent/skills/creative/manim-video/references/paper-explainer.md`, `src/hermes-agent/skills/creative/manim-video/references/scene-planning.md` |
| `ACCEPTED_DUPLICATE_ADRS` | 1 | `src/rusty-idd/openspec/changes/archive/harness-session-frontdoor/tasks.md` |
| `ACCEPTED_REWRITE_RETURN_CODES` | 1 | `src/rtk-tokenkill/hooks/hermes/rtk-rewrite/__init__.py` |
| `ACCESS_ALLOWED_CALLBACK_ACE` | 1 | `src/yazelix-helix/helix-stdx/src/faccess.rs` |
| `ACCESS_TOKEN` | 3 | `src/hermes-agent/optional-skills/productivity/shop-app/SKILL.md`, `src/hermes-agent/website/docs/user-guide/skills/optional/productivity/productivity-shop-app.md`, `src/hermes-agent/website/i18n/zh-Hans/docusaurus-plugin-content-docs/current/user-guide/skills/optional/productivity/productivity-shop-app.md` |
| `ACCESS_TOKEN_REFRESH_SKEW_SECONDS` | 1 | `src/hermes-agent/hermes_cli/auth.py` |
| `ACCESS_TOKEN_TTL_SECONDS` | 1 | `src/hermes-agent/gateway/platforms/wecom_callback.py` |
| `ACCURACY` | 1 | `src/yazelix-helix/runtime/grammars/sources/styx/crates/styx-lsp/src/semantic_tokens.rs` |
| `ACF` | 2 | `src/meta-ruvector/docs/research/exotic-structure-discovery/04-experiment-output.md`, `src/meta-ruvector/examples/boundary-discovery/src/main.rs` |
| `ACFS_NOTIFY_TOKEN` | 1 | `src/beads_rust/.github/workflows/notify-acfs.yml` |
| `ACFS_REPO` | 2 | `src/beads_rust/.github/workflows/notify-acfs.yml`, `src/beads_rust/tests/workflow_notify_acfs.rs` |
| `ACL_HBM_MEM` | 1 | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/inferrs/inferrs/src/engine.rs` |
| `ACL_SIZE_INFORMATION` | 1 | `src/yazelix-helix/helix-stdx/src/faccess.rs` |
| `ACP_MARKER_BASE_URL` | 1 | `src/hermes-agent/agent/copilot_acp_client.py` |
| `ACP_REGISTRY_MANIFEST` | 1 | `src/hermes-agent/scripts/release.py` |
| `ACT` | 1 | `src/beads_rust/.github/workflows/release.yml` |
| `ACTION` | 45 | `src/envctl-codedb-config-to-tables/assets/scripts/meta-session-env.sh`, `src/envctl-codedb-config-to-tables/manifest/cognitum-seed-autounlock.toml`, `src/envctl-codedb-config-to-tables/manifest/cognitum-seed-net.toml` |
| `ACTION_VOCAB` | 1 | `src/ruflo/v3/@claude-flow/cli/src/memory/structured-distill.ts` |
| `ACTIVATION_MARKER` | 2 | `src/beads_rust/scripts/activate-dev-local-patch.sh`, `src/beads_rust/scripts/deactivate-dev-local-patch.sh` |
| `ACTIVE` | 8 | `src/hermes-agent/optional-skills/mlops/instructor/references/validation.md`, `src/meta-ruvector/.claude/statusline-command.sh`, `src/meta-ruvector/.codex/mirror/.claude/statusline-command.sh` |
| `ACTIVE_AGENTS` | 1 | `src/ruflo/v3/implementation/hooks/STATUSLINE-DAEMONS.md` |
| `ACTIVE_ALGOS` | 3 | `src/meta-ruvector/.claude/statusline-command.sh`, `src/meta-ruvector/.codex/mirror/.claude/statusline-command.sh`, `src/meta-ruvector/crates/ruvector-cli/scripts/statusline-command.sh` |
| `ACTIVE_HOOK` | 1 | `src/envctl/scripts/tests/test-flexnetos-codex-runtime-gate.sh` |
| `ACTIVE_HOOKS_JSON` | 1 | `src/envctl/scripts/tests/test-flexnetos-codex-runtime-gate.sh` |
| `ACTIVE_PATHS` | 4 | `src/envctl-codedb-config-to-tables/ci/gates/meta-local-policy.sh`, `src/envctl-nix-components-db/ci/gates/meta-local-policy.sh`, `src/envctl-nu-plugin-db-skill/ci/gates/meta-local-policy.sh` |
| `ACTIVE_PIPELINE_STATES` | 1 | `src/hermes-agent/plugins/teams_pipeline/pipeline.py` |
| `ACTIVE_PROCESSES` | 2 | `src/meta-ruvector/examples/vibecast-7sense/.claude/statusline.sh`, `src/ruflo/.claude/statusline.sh` |
| `ACTIVITY_INDICATOR` | 2 | `src/meta-ruvector/examples/vibecast-7sense/.claude/statusline.sh`, `src/ruflo/.claude/statusline.sh` |
| `ACTIVITY_LIMIT` | 1 | `src/hermes-agent/ui-tui/src/app/turnController.ts` |
| `ACTOR` | 5 | `src/beads_rust/.claude/skills/br/SKILL.md`, `src/beads_rust/.claude/skills/br/references/COMMANDS.md`, `src/beads_rust/.claude/skills/br/references/INTEGRATION.md` |
| `ACTOR_TYPES` | 1 | `src/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/envctl_run_ledger.py` |
| `ACTUAL` | 2 | `src/beads_rust/.github/workflows/release.yml`, `src/rtk-tokenkill/install.sh` |
| `ACTUAL_SEGS` | 1 | `src/meta-ruvector/scripts/deploy-wet-job.sh` |
| `ACTUAL_TOTAL` | 1 | `src/beads_rust/tests/e2e_scripts/sync_safety_witness.sh` |
| `ACTUAL_VIOLATIONS` | 1 | `src/beads_rust/tests/e2e_scripts/sync_safety_witness.sh` |
| `AD` | 4 | `src/envctl-codedb-config-to-tables/assets/scripts/autoinstall.yaml`, `src/envctl-nix-components-db/assets/scripts/autoinstall.yaml`, `src/envctl-nu-plugin-db-skill/assets/scripts/autoinstall.yaml` |
| `ADAPTER` | 5 | `src/envctl/.codex/plugins/cache/meta-plugins-codex/hugging-face/1.0.3/skills/llm-trainer/references/local_training_macos.md`, `src/meta-plugins/codex/plugins/hugging-face/skills/llm-trainer/references/local_training_macos.md`, `src/meta-ruvector/.github/workflows/ruvector-npm-ci.yml` |
| `ADAPTER_FILE` | 4 | `src/envctl-codedb-config-to-tables/.handoff/loop/plan/findings/governance-config-rusty-idd.md`, `src/envctl-nix-components-db/.handoff/loop/plan/findings/governance-config-rusty-idd.md`, `src/envctl-nu-plugin-db-skill/.handoff/loop/plan/findings/governance-config-rusty-idd.md` |
| `ADAPTER_MODEL` | 2 | `src/envctl/.codex/plugins/cache/meta-plugins-codex/hugging-face/1.0.3/skills/llm-trainer/scripts/convert_to_gguf.py`, `src/meta-plugins/codex/plugins/hugging-face/skills/llm-trainer/scripts/convert_to_gguf.py` |
| `ADAPTER_WEIGHTS` | 1 | `src/meta-ruvector/crates/ruvllm/src/training/real_trainer.rs` |
| `ADAPTIVE_EFFORT_MAP` | 1 | `src/hermes-agent/agent/anthropic_adapter.py` |
| `ADD` | 2 | `src/hermes-agent/tools/patch_parser.py`, `src/meta-ruvector/docs/research/rvf/spec/05-overlay-epochs.md` |
| `ADDED` | 1 | `src/hermes-agent/.github/workflows/supply-chain-audit.yml` |
| `ADDITION` | 3 | `src/meta-ruvector/npm/packages/ruvector-extensions/docs/TEMPORAL.md`, `src/meta-ruvector/npm/packages/ruvector-extensions/src/temporal.d.ts`, `src/meta-ruvector/npm/packages/ruvector-extensions/src/temporal.ts` |
| `ADDITIONAL_HYPERLINK_TERMINALS` | 1 | `src/hermes-agent/ui-tui/packages/hermes-ink/src/ink/supports-hyperlinks.ts` |
| `ADDR` | 4 | `src/envctl-codedb-config-to-tables/manifest/cognitum-seed-net.toml`, `src/envctl-nix-components-db/manifest/cognitum-seed-net.toml`, `src/envctl-nu-plugin-db-skill/manifest/cognitum-seed-net.toml` |
| `ADD_COLS` | 1 | `src/hermes-agent/optional-skills/research/osint-investigation/scripts/fetch_ofac_sdn.py` |
| `ADD_RESOURCE_SCHEMA` | 1 | `src/hermes-agent/plugins/memory/openviking/__init__.py` |
| `ADD_URL` | 1 | `src/hermes-agent/optional-skills/research/osint-investigation/scripts/fetch_ofac_sdn.py` |
| `ADJECTIVES` | 2 | `src/envctl/.codex/plugins/cache/meta-plugins-codex/digitalocean/0.2.2/skills/provision-droplet/scripts/keygen.py`, `src/meta-plugins/codex/plugins/digitalocean/skills/provision-droplet/scripts/keygen.py` |
| `ADMIN` | 1 | `src/meta-ruvector/docs/implementation/IMPROVEMENT_ROADMAP.md` |
| `ADMIN_API_ADDR` | 1 | `src/meta-ruvector/crates/ruvector-tiny-dancer-core/docs/API_QUICK_REFERENCE.md` |
| `ADMIN_API_PORT` | 1 | `src/meta-ruvector/crates/ruvector-tiny-dancer-core/docs/API_QUICK_REFERENCE.md` |
| `ADMIN_API_TOKEN` | 1 | `src/meta-ruvector/crates/ruvector-tiny-dancer-core/docs/API_QUICK_REFERENCE.md` |
| `ADMIN_AUTH_TOKEN` | 1 | `src/meta-ruvector/crates/ruvector-tiny-dancer-core/examples/README.md` |
| `ADMIN_CLI_LOGIN` | 2 | `src/meta-ruvector/ui/ruvocal/src/lib/server/adminToken.ts`, `src/ruflo/ruflo/src/ruvocal/src/lib/server/adminToken.ts` |
| `ADMIN_SECRET` | 2 | `src/meta-ruvector/ui/ruvocal/src/lib/server/hooks/handle.ts`, `src/ruflo/ruflo/src/ruvocal/src/lib/server/hooks/handle.ts` |
| `ADR` | 28 | `src/ruflo/plugins/ruflo-adr/scripts/smoke.sh`, `src/ruflo/plugins/ruflo-agent/scripts/smoke.sh`, `src/ruflo/plugins/ruflo-aidefence/scripts/smoke.sh` |
| `ADR2` | 1 | `src/ruflo/plugins/ruflo-workflows/scripts/smoke.sh` |
| `ADRS` | 7 | `src/meta-ruvector/.claude/helpers/adr-compliance.sh`, `src/meta-ruvector/.codex/helpers/adr-compliance.sh`, `src/meta-ruvector/.codex/mirror/.claude/helpers/adr-compliance.sh` |
| `ADRS_COMPLIANT` | 1 | `src/meta-ruvector/examples/vibecast-7sense/.claude/statusline.sh` |
| `ADRS_TOTAL` | 1 | `src/meta-ruvector/examples/vibecast-7sense/.claude/statusline.sh` |

## Feature Flag References

| name | refs | sample paths |
|---|---:|---|
| `0017_phase2_selection_flag` | 1 | `src/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/migration-artifacts/art-100_system_inventory/system_inventory.json` |
| `0037_remote_control_enrollments_enabled` | 1 | `src/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/migration-artifacts/art-100_system_inventory/system_inventory.json` |
| `02_feature_matrix` | 43 | `src/rusty-idd/.idd/knowledge/plan-context.json`, `src/rusty-idd/.idd/knowledge/system-architecture.json`, `src/rusty-idd/AI_MERGE/34_grit_full_integration/adoption-evidence.md` |
| `20251211093327_add_featured_prompts` | 6 | `src/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/migration-artifacts/art-100_system_inventory/system_inventory.json`, `src/rusty-idd/AI_MERGE/34_grit_full_integration/01_rusty_idd_inventory_before_adoption.md`, `src/rusty-idd/AI_MERGE/34_grit_full_integration/plan-workspace/AI_MERGE/00_repo_a_inventory.md` |
| `20251222132600_add_user_flagged_and_unusual_activity` | 6 | `src/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/migration-artifacts/art-100_system_inventory/system_inventory.json`, `src/rusty-idd/AI_MERGE/34_grit_full_integration/01_rusty_idd_inventory_before_adoption.md`, `src/rusty-idd/AI_MERGE/34_grit_full_integration/plan-workspace/AI_MERGE/00_repo_a_inventory.md` |
| `20Experiments` | 1 | `src/hermes-agent/optional-skills/mlops/training/unsloth/references/llms-txt.md` |
| `20feature` | 2 | `src/hermes-agent/optional-skills/mlops/training/unsloth/references/llms-txt.md`, `src/rusty-idd/third_party/upstream/ai-prompt/examples/how-to-vibecode-a-gutenberg-block.json` |
| `389800174_Experimental_Realization_of_Discrete_Time_Quasicrystals` | 1 | `src/meta-ruvector/examples/exo-ai-2025/research/03-time-crystal-cognition/RESEARCH.md` |
| `3KuZ8r8C3uVFlAg` | 5 | `src/hermes-agent/website/package-lock.json`, `src/ruflo/package-lock.json`, `src/ruflo/v3/package-lock.json` |
| `5BFeature` | 1 | `src/yazelix-helix/runtime/grammars/sources/cel/.github/CONTRIBUTING.md` |
| `ADMIN_ENABLED` | 1 | `src/meta-ruvector/docs/adr/ADR-030-rvf-cognitive-container.md` |
| `ADVANCED_FEATURES` | 5 | `src/meta-ruvector/crates/ruvector-core/README.md`, `src/meta-ruvector/crates/ruvector-router-cli/README.md`, `src/meta-ruvector/docs/INDEX.md` |
| `AGENTDB_ENABLED` | 7 | `src/meta-ruvector/.agents/skills/agentdb-advanced/SKILL.md`, `src/meta-ruvector/.claude/skills/agentdb-advanced/SKILL.md`, `src/meta-ruvector/.codex/mirror/.claude/skills/agentdb-advanced/SKILL.md` |
| `AGENTIC_FLOW_HNSW_ENABLED` | 1 | `src/ruflo/v3/implementation/optimization/V3-OPTIMIZATION-ROADMAP.md` |
| `AGENTIC_FLOW_SONA_ENABLED` | 1 | `src/ruflo/v3/implementation/optimization/V3-OPTIMIZATION-ROADMAP.md` |
| `AGENT_BROWSER_CHROME_FLAGS` | 1 | `src/hermes-agent/tools/browser_tool.py` |
| `API_SERVER_ENABLED` | 21 | `src/hermes-agent/.plans/openai-api-server.md`, `src/hermes-agent/gateway/config.py`, `src/hermes-agent/hermes_cli/config.py` |
| `ATOM_DISABLE_SHELLING_OUT_FOR_ENVIRONMENT` | 1 | `src/yazelix-helix/runtime/grammars/sources/bash/examples/atom.sh` |
| `ATTR_KEY_FAST_MATH_FLAGS` | 1 | `src/cuda-oxide/crates/llvm-export/src/export/ops.rs` |
| `ATTR_KEY_INTEGER_OVERFLOW_FLAGS` | 3 | `src/cuda-oxide/crates/llvm-export/tests/ops_test.rs`, `src/cuda-oxide/crates/mir-lower/src/convert/ops/arithmetic.rs`, `src/cuda-oxide/crates/nvvm-transforms/src/legalize.rs` |
| `AUTH_ENABLED` | 9 | `src/envctl-codedb-config-to-tables/docs/odysseus-adoption.md`, `src/envctl-codedb-config-to-tables/manifest/odysseus.toml`, `src/envctl-nix-components-db/docs/odysseus-adoption.md` |
| `AUTH_FLAG` | 1 | `src/meta-ruvector/crates/mcp-brain-server/deploy.sh` |
| `AUTH_OAUTH_ENABLE_PKCE` | 20 | `src/rusty-idd/AI_MERGE/34_grit_full_integration/01_rusty_idd_inventory_before_adoption.json`, `src/rusty-idd/AI_MERGE/34_grit_full_integration/01_rusty_idd_inventory_before_adoption.md`, `src/rusty-idd/AI_MERGE/34_grit_full_integration/plan-workspace/AI_MERGE/00_repo_a_inventory.json` |
| `AUTH_OIDC_ENABLE_PKCE` | 20 | `src/rusty-idd/AI_MERGE/34_grit_full_integration/01_rusty_idd_inventory_before_adoption.json`, `src/rusty-idd/AI_MERGE/34_grit_full_integration/01_rusty_idd_inventory_before_adoption.md`, `src/rusty-idd/AI_MERGE/34_grit_full_integration/plan-workspace/AI_MERGE/00_repo_a_inventory.json` |
| `AUTOPILOT_ENABLED` | 1 | `src/ruflo/v3/implementation/adrs/ADR-051-infinite-context-compaction-bridge.md` |
| `AUX_ENABLES` | 2 | `src/meta-ruvector/crates/ruvix/crates/bcm2711/src/mini_uart.rs`, `src/meta-ruvector/crates/ruvix/crates/rpi-boot/src/early_uart.rs` |
| `AUX_ENABLES_MU` | 1 | `src/meta-ruvector/crates/ruvix/crates/bcm2711/src/mini_uart.rs` |
| `AUX_ENABLES_SPI1` | 1 | `src/meta-ruvector/crates/ruvix/crates/bcm2711/src/mini_uart.rs` |
| `AUX_ENABLES_SPI2` | 1 | `src/meta-ruvector/crates/ruvix/crates/bcm2711/src/mini_uart.rs` |
| `AUX_MU_ENABLE` | 1 | `src/meta-ruvector/crates/ruvix/crates/rpi-boot/src/early_uart.rs` |
| `AWS_EC2_METADATA_DISABLED` | 2 | `src/hermes-agent/hermes_cli/doctor.py`, `src/hermes-agent/tests/conftest.py` |
| `AccessFeatures` | 6 | `src/meta-ruvector/examples/exo-ai-2025/research/05-memory-mapped-neural-fields/BREAKTHROUGH_HYPOTHESIS.md`, `src/meta-ruvector/examples/exo-ai-2025/research/05-memory-mapped-neural-fields/architecture.md`, `src/meta-ruvector/examples/exo-ai-2025/research/05-memory-mapped-neural-fields/benches/neural_field_bench.rs` |
| `AceFlags` | 1 | `src/yazelix-helix/helix-stdx/src/faccess.rs` |
| `AcousticFeature` | 2 | `src/meta-ruvector/examples/vibecast-7sense/crates/sevensense-api/src/openapi.rs`, `src/meta-ruvector/examples/vibecast-7sense/crates/sevensense-api/src/rest/handlers.rs` |
| `AgentProgressSummariesEnabled` | 1 | `src/meta-ruvector/docs/research/claude-code-rvsource/09-agent-and-subagent-system.md` |
| `AppliedDocFeature` | 1 | `src/flexnetos_runner/crates/runner-cli/src/forge_loop.rs` |
| `ArchiveNotEnabled` | 1 | `src/beads_rust/src/cli/commands/upgrade.rs` |
| `AttentionFeatures` | 1 | `src/meta-ruvector/examples/ruvLLM/docs/SONA/06-COMPONENTS.md` |
| `AttestationFlags` | 2 | `src/meta-ruvector/.understand-anything/slice-10a-knowledge-graph.json`, `src/meta-ruvector/crates/ruvix/crates/boot/src/attestation.rs` |
| `AttributionFeature` | 2 | `src/ruflo/plugins/ruflo-neural-trader/skills/trader-explain/SKILL.md`, `src/ruflo/plugins/ruflo-neural-trader/src/signed-attribution.ts` |
| `AuditFlags` | 4 | `src/meta-ruvector/.understand-anything/slice-10a-knowledge-graph.json`, `src/meta-ruvector/crates/ruvix/crates/cap/README.md`, `src/meta-ruvector/crates/ruvix/crates/cap/src/audit.rs` |
| `AvailableFeatures` | 1 | `src/ruflo/v3/implementation/architecture/SDK-ARCHITECTURE-ANALYSIS.md` |
| `BNNS_FLAGS_NONE` | 1 | `src/meta-ruvector/crates/ruvllm/src/kernels/ane_ops.rs` |
| `BRAIN_ENABLE` | 1 | `src/meta-ruvector/crates/ruvector-kalshi/examples/paper_trade.rs` |
| `BROWSER_ENABLE_MEMORY` | 1 | `src/ruflo/v3/@claude-flow/browser/README.md` |
| `BR_DISABLE_PARALLEL_JSONL_EXPORT` | 1 | `src/beads_rust/src/sync/mod.rs` |
| `BR_DISABLE_READ_ONLY_FAST_OPEN` | 5 | `src/beads_rust/docs/SWARM_SCALE_TUNING.md`, `src/beads_rust/src/main.rs`, `src/beads_rust/tests/common/cli.rs` |
| `BUSY_INPUT_FLAG` | 4 | `src/hermes-agent/agent/onboarding.py`, `src/hermes-agent/cli.py`, `src/hermes-agent/gateway/run.py` |
| `BatchFeature` | 2 | `src/envctl/.codex/plugins/cache/meta-plugins-codex/hugging-face/1.0.3/skills/vision-trainer/scripts/object_detection_training.py`, `src/meta-plugins/codex/plugins/hugging-face/skills/vision-trainer/scripts/object_detection_training.py` |
| `BinauralFeatures` | 1 | `src/meta-ruvector/docs/examples/musica/src/hearing_aid.rs` |
| `Bitflag` | 1 | `src/meta-ruvector/.understand-anything/slice-10a-knowledge-graph.json` |
| `Bitflags` | 2 | `src/meta-ruvector/.understand-anything/slice-10a-knowledge-graph.json`, `src/yazelix-helix/helix-tui/src/widgets/mod.rs` |
| `BufferUsageFlags` | 4 | `src/meta-ruvector/crates/prime-radiant/src/gpu/buffer.rs`, `src/meta-ruvector/crates/prime-radiant/src/gpu/mod.rs`, `src/meta-ruvector/crates/prime-radiant/src/lib.rs` |
| `BuildEdgeFeatures` | 1 | `src/meta-ruvector/examples/ruvLLM/docs/sparc/02-pseudocode.md` |
| `BuildRouterFeatures` | 1 | `src/meta-ruvector/examples/ruvLLM/docs/sparc/02-pseudocode.md` |
| `BuilderFlag` | 2 | `src/hermes-agent/skills/mlops/models/segment-anything/references/advanced-usage.md`, `src/meta-ruvector/examples/scipix/docs/09_OPTIMIZATION.md` |
| `CANDLE_METAL_ENABLE_FAST_MATH` | 1 | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/inferrs/inferrs-kernels/candle-metal-kernels/src/kernel.rs` |
| `CARGO_CFG_TARGET_FEATURE` | 1 | `src/meta-ruvector/crates/ruvector-solver/build.rs` |
| `CARGO_ENCODED_RUSTFLAGS` | 3 | `src/cuda-oxide/crates/cargo-oxide/README.md`, `src/cuda-oxide/crates/cargo-oxide/src/commands.rs`, `src/flexnetos_runner/scripts/collect-runner-queue-evidence.sh` |
| `CARGO_FEATURES` | 1 | `src/meta-ruvector/docs/reviews/RUVLLM_OPTIMIZATION_CHECKLIST.md` |
| `CARGO_FEATURE_CANN` | 1 | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/cluaiz/interface-engines/llama/build.rs` |
| `CARGO_FEATURE_CUDA` | 1 | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/cluaiz/interface-engines/llama/build.rs` |
| `CARGO_FEATURE_FILTERED_SEARCH` | 1 | `src/meta-ruvector/crates/ruvector-postgres/build.rs` |
| `CARGO_FEATURE_HAILO` | 1 | `src/meta-ruvector/crates/hailort-sys/build.rs` |
| `CARGO_FEATURE_HYBRID_SEARCH` | 1 | `src/meta-ruvector/crates/ruvector-postgres/build.rs` |
| `CARGO_FEATURE_INDEX_HNSW` | 1 | `src/meta-ruvector/crates/ruvector-postgres/build.rs` |
| `CARGO_FEATURE_INDEX_IVFFLAT` | 1 | `src/meta-ruvector/crates/ruvector-postgres/build.rs` |
| `CARGO_FEATURE_METAL` | 1 | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/cluaiz/interface-engines/llama/build.rs` |
| `CARGO_FEATURE_NEON_COMPAT` | 1 | `src/meta-ruvector/crates/ruvector-postgres/build.rs` |
| `CARGO_FEATURE_OPENVINO` | 1 | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/cluaiz/interface-engines/llama/build.rs` |
| `CARGO_FEATURE_QNN` | 1 | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/cluaiz/interface-engines/llama/build.rs` |
| `CARGO_FEATURE_QUANTIZATION_BINARY` | 1 | `src/meta-ruvector/crates/ruvector-postgres/build.rs` |
| `CARGO_FEATURE_QUANTIZATION_PRODUCT` | 1 | `src/meta-ruvector/crates/ruvector-postgres/build.rs` |
| `CARGO_FEATURE_QUANTIZATION_SCALAR` | 1 | `src/meta-ruvector/crates/ruvector-postgres/build.rs` |
| `CARGO_FEATURE_ROCM` | 1 | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/cluaiz/interface-engines/llama/build.rs` |
| `CARGO_FEATURE_SIMD_AVX2` | 1 | `src/meta-ruvector/crates/ruvector-postgres/build.rs` |
| `CARGO_FEATURE_SIMD_AVX512` | 1 | `src/meta-ruvector/crates/ruvector-postgres/build.rs` |
| `CARGO_FEATURE_SIMD_NATIVE` | 1 | `src/meta-ruvector/crates/ruvector-postgres/build.rs` |
| `CARGO_FEATURE_SYCL` | 1 | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/cluaiz/interface-engines/llama/build.rs` |
| `CARGO_FEATURE_VULKAN` | 1 | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/cluaiz/interface-engines/llama/build.rs` |

## Secret References

Secret values were not captured. Only names and reference locations are listed.

| name | kind | refs | sample paths |
|---|---|---:|---|
| `+server.ts` | `path_name` | 4 | `src/meta-ruvector/ui/ruvocal/src/routes/.well-known/oauth-cimd/+server.ts`, `src/meta-ruvector/ui/ruvocal/src/routes/api/user/validate-token/+server.ts`, `src/ruflo/ruflo/src/ruvocal/src/routes/.well-known/oauth-cimd/+server.ts` |
| `.gitignore` | `path_name` | 6 | `src/envctl-codedb-config-to-tables/crates/secrets-store-libsql/.gitignore`, `src/envctl-nix-components-db/crates/secrets-store-libsql/.gitignore`, `src/envctl-nu-plugin-db-skill/crates/secrets-store-libsql/.gitignore` |
| `.release-please-manifest.json` | `path_name` | 1 | `src/rtk-tokenkill/.release-please-manifest.json` |
| `.semgrep.yml` | `path_name` | 1 | `src/rtk-tokenkill/.semgrep.yml` |
| `0002-normalize-env-and-secrets.md` | `path_name` | 4 | `src/rusty-idd/AI_MERGE/34_grit_full_integration/plan-workspace/AI_MERGE/07_tasks/0002-normalize-env-and-secrets.md`, `src/rusty-idd/AI_MERGE/35_e2e_test_suite/plan-workspace/AI_MERGE/07_tasks/0002-normalize-env-and-secrets.md`, `src/rusty-idd/AI_MERGE/unify-handoff-prompthub/07_handoff_tasks/0002-normalize-env-and-secrets.md` |
| `03_env_and_secret_contracts.json` | `path_name` | 2 | `src/rusty-idd/AI_MERGE/34_grit_full_integration/plan-workspace/AI_MERGE/03_env_and_secret_contracts.json`, `src/rusty-idd/AI_MERGE/35_e2e_test_suite/plan-workspace/AI_MERGE/03_env_and_secret_contracts.json` |
| `03_env_and_secret_contracts.md` | `path_name` | 3 | `src/rusty-idd/AI_MERGE/03_env_and_secret_contracts.md`, `src/rusty-idd/AI_MERGE/34_grit_full_integration/plan-workspace/AI_MERGE/03_env_and_secret_contracts.md`, `src/rusty-idd/AI_MERGE/35_e2e_test_suite/plan-workspace/AI_MERGE/03_env_and_secret_contracts.md` |
| `03_handoff_env_secret_contracts.json` | `path_name` | 1 | `src/rusty-idd/AI_MERGE/unify-handoff-prompthub/03_handoff_env_secret_contracts.json` |
| `03_handoff_env_secret_contracts.md` | `path_name` | 1 | `src/rusty-idd/AI_MERGE/unify-handoff-prompthub/03_handoff_env_secret_contracts.md` |
| `03_prompthub_env_secret_contracts.json` | `path_name` | 1 | `src/rusty-idd/AI_MERGE/unify-handoff-prompthub/03_prompthub_env_secret_contracts.json` |
| `03_prompthub_env_secret_contracts.md` | `path_name` | 1 | `src/rusty-idd/AI_MERGE/unify-handoff-prompthub/03_prompthub_env_secret_contracts.md` |
| `08-secretd-store-config.md` | `path_name` | 4 | `src/envctl-codedb-config-to-tables/docs/ops/08-secretd-store-config.md`, `src/envctl-nix-components-db/docs/ops/08-secretd-store-config.md`, `src/envctl-nu-plugin-db-skill/docs/ops/08-secretd-store-config.md` |
| `ACCESS_TOKEN` | `env_or_setting_name` | 3 | `src/hermes-agent/optional-skills/productivity/shop-app/SKILL.md`, `src/hermes-agent/website/docs/user-guide/skills/optional/productivity/productivity-shop-app.md`, `src/hermes-agent/website/i18n/zh-Hans/docusaurus-plugin-content-docs/current/user-guide/skills/optional/productivity/productivity-shop-app.md` |
| `ACCESS_TOKEN_REFRESH_SKEW_SECONDS` | `env_or_setting_name` | 1 | `src/hermes-agent/hermes_cli/auth.py` |
| `ACCESS_TOKEN_TTL_SECONDS` | `env_or_setting_name` | 1 | `src/hermes-agent/gateway/platforms/wecom_callback.py` |
| `ACCURACY` | `path_name` | 1 | `src/yazelix-helix/runtime/grammars/sources/styx/crates/styx-lsp/src/semantic_tokens.rs` |
| `ACFS_NOTIFY_TOKEN` | `env_or_setting_name` | 1 | `src/beads_rust/.github/workflows/notify-acfs.yml` |
| `ACTION` | `path_name` | 4 | `src/meta-ruvector/.claude/agents/v3/claims-authorizer.md`, `src/meta-ruvector/.codex/mirror/.claude/agents/v3/claims-authorizer.md`, `src/ruflo/v3/@claude-flow/cli/.claude/agents/v3/claims-authorizer.md` |
| `ADMIN_API_TOKEN` | `env_or_setting_name` | 1 | `src/meta-ruvector/crates/ruvector-tiny-dancer-core/docs/API_QUICK_REFERENCE.md` |
| `ADMIN_AUTH_TOKEN` | `env_or_setting_name` | 1 | `src/meta-ruvector/crates/ruvector-tiny-dancer-core/examples/README.md` |
| `ADMIN_CLI_LOGIN` | `path_name` | 2 | `src/meta-ruvector/ui/ruvocal/src/lib/server/adminToken.ts`, `src/ruflo/ruflo/src/ruvocal/src/lib/server/adminToken.ts` |
| `ADMIN_SECRET` | `env_or_setting_name` | 2 | `src/meta-ruvector/ui/ruvocal/src/lib/server/hooks/handle.ts`, `src/ruflo/ruflo/src/ruvocal/src/lib/server/hooks/handle.ts` |
| `ADR-144-agent-authorization-propagation.md` | `path_name` | 1 | `src/ruflo/v3/docs/adr/ADR-144-agent-authorization-propagation.md` |
| `ADR-G021-human-authority-and-irreversibility.md` | `path_name` | 1 | `src/ruflo/v3/@claude-flow/guidance/docs/adrs/ADR-G021-human-authority-and-irreversibility.md` |
| `AGENT_ID` | `path_name` | 5 | `src/meta-ruvector/.claude/agents/v3/claims-authorizer.md`, `src/meta-ruvector/.codex/agents/claude/claude-v3-claims-authorizer.toml`, `src/meta-ruvector/.codex/mirror/.claude/agents/v3/claims-authorizer.md` |
| `AGI_TAG_AUTHORITY_CONFIG` | `env_or_setting_name` | 1 | `src/meta-ruvector/crates/rvf/rvf-runtime/src/agi_container.rs` |
| `AIRTABLE_API_KEY` | `env_or_setting_name` | 6 | `src/hermes-agent/skills/productivity/airtable/SKILL.md`, `src/hermes-agent/website/docs/user-guide/skills/bundled/productivity/productivity-airtable.md`, `src/hermes-agent/website/i18n/zh-Hans/docusaurus-plugin-content-docs/current/user-guide/skills/bundled/productivity/productivity-airtable.md` |
| `ALLOW_INSECURE_COOKIES` | `path_name` | 2 | `src/meta-ruvector/ui/ruvocal/src/lib/server/auth.ts`, `src/ruflo/ruflo/src/ruvocal/src/lib/server/auth.ts` |
| `ALPHAVANTAGE_API_KEY` | `env_or_setting_name` | 1 | `src/meta-ruvector/examples/data/framework/examples/economic_discovery.rs` |
| `ALT_SECRET` | `env_or_setting_name` | 1 | `src/hermes-agent/tests/hermes_cli/test_config_env_refs.py` |
| `ANSPIRE_API_KEY` | `env_or_setting_name` | 1 | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/BettaFish/SingleEngineApp/media_engine_streamlit_app.py` |
| `ANTHROPIC_API_KEY` | `env_or_setting_name` | 127 | `src/envctl-codedb-config-to-tables/crates/secrets-engine/src/inject.rs`, `src/envctl-codedb-config-to-tables/crates/secrets-engine/tests/inject.rs`, `src/envctl-codedb-config-to-tables/docs/ops/02-envctl-component.md` |
| `ANTHROPIC_TOKEN` | `env_or_setting_name` | 5 | `src/hermes-agent/agent/anthropic_adapter.py`, `src/hermes-agent/hermes_cli/web_server.py`, `src/hermes-agent/tests/hermes_cli/test_config.py` |
| `API_KEY` | `env_or_setting_name` | 43 | `src/hermes-agent/optional-skills/health/fitness-nutrition/SKILL.md`, `src/hermes-agent/optional-skills/health/fitness-nutrition/scripts/nutrition_search.py`, `src/hermes-agent/optional-skills/mlops/tensorrt-llm/references/serving.md` |
| `API_KEYS` | `env_or_setting_name` | 2 | `src/meta-ruvector/scripts/seed-brain-all.py`, `src/meta-ruvector/scripts/seed-specialized.py` |
| `API_KEY_LENGTH` | `env_or_setting_name` | 1 | `src/rusty-idd/third_party/upstream/prompts.chat/src/lib/api-key.ts` |
| `API_KEY_PATH` | `env_or_setting_name` | 1 | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/splitrail/.github/workflows/release.yml` |
| `API_KEY_PATTERNS` | `env_or_setting_name` | 5 | `src/meta-ruvector/.claude/agents/v3/pii-detector.md`, `src/meta-ruvector/.codex/agents/claude/claude-v3-pii-detector.toml`, `src/meta-ruvector/.codex/mirror/.claude/agents/v3/pii-detector.md` |
| `API_KEY_PREFIX` | `env_or_setting_name` | 1 | `src/rusty-idd/third_party/upstream/prompts.chat/src/lib/api-key.ts` |
| `API_KEY_SOURCE` | `env_or_setting_name` | 1 | `src/hermes-agent/optional-skills/productivity/here-now/scripts/publish.sh` |
| `API_TOKEN` | `env_or_setting_name` | 5 | `src/envctl-codedb-config-to-tables/crates/engine/src/catalog.rs`, `src/envctl-nix-components-db/crates/engine/src/catalog.rs`, `src/envctl-nu-plugin-db-skill/crates/engine/src/catalog.rs` |
| `APPLE_API_KEY` | `env_or_setting_name` | 1 | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/splitrail/.github/workflows/release.yml` |
| `APPLE_API_KEY_PATH` | `env_or_setting_name` | 1 | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/splitrail/.github/workflows/release.yml` |
| `APPLE_CODE_SIGNING_CERT_PASSWORD` | `env_or_setting_name` | 1 | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/splitrail/.github/workflows/release.yml` |
| `APP_SECRET` | `env_or_setting_name` | 3 | `src/hermes-agent/optional-skills/productivity/shopify/SKILL.md`, `src/hermes-agent/website/docs/user-guide/skills/optional/productivity/productivity-shopify.md`, `src/hermes-agent/website/i18n/zh-Hans/docusaurus-plugin-content-docs/current/user-guide/skills/optional/productivity/productivity-shopify.md` |
| `APP_STORE_CONNECT_API_KEY` | `env_or_setting_name` | 1 | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/splitrail/.github/workflows/release.yml` |
| `APP_STORE_CONNECT_API_KEY_ID` | `env_or_setting_name` | 1 | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/splitrail/.github/workflows/release.yml` |
| `APP_STORE_CONNECT_API_KEY_ISSUER` | `env_or_setting_name` | 1 | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/splitrail/.github/workflows/release.yml` |
| `AP_TOKENS` | `env_or_setting_name` | 1 | `src/ruflo/.claude/statusline.sh` |
| `ARCHITECTURE.md` | `path_name` | 1 | `src/rtk-tokenkill/docs/contributing/ARCHITECTURE.md` |
| `ARGUMENTS` | `path_name` | 4 | `src/meta-ruvector/.codex/prompts/analysis-token-efficiency.md`, `src/meta-ruvector/.codex/prompts/analysis-token-usage.md`, `src/meta-ruvector/.codex/prompts/analysis:token-efficiency.md` |
| `ATTRIBUTE` | `path_name` | 1 | `src/yazelix-helix/runtime/grammars/sources/styx/crates/styx-lsp/src/semantic_tokens.rs` |
| `ATTRIBUTES` | `path_name` | 1 | `src/yazelix-helix/runtime/grammars/sources/styx/crates/styx-lsp/src/semantic_tokens.rs` |
| `AUDIT_GUIDE.md` | `path_name` | 1 | `src/rtk-tokenkill/docs/usage/AUDIT_GUIDE.md` |
| `AUTH` | `env_or_setting_name` | 14 | `src/hermes-agent/skills/github/github-code-review/SKILL.md`, `src/hermes-agent/skills/github/github-issues/SKILL.md`, `src/hermes-agent/skills/github/github-pr-workflow/SKILL.md` |
| `AUTH.md` | `path_name` | 1 | `src/ruflo/ruflo/docs/AUTH.md` |
| `AUTHENTICATION` | `env_or_setting_name` | 1 | `src/meta-ruvector/docs/research/federated-rvf/ARCHITECTURE.md` |
| `AUTHOR` | `env_or_setting_name` | 2 | `src/hermes-agent/.github/workflows/contributor-check.yml`, `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/cluaiz/.github/release-drafter.yml` |
| `AUTHORITY_SCOPES` | `env_or_setting_name` | 1 | `src/ruflo/v3/@claude-flow/guidance/src/manifest-validator.ts` |
| `AUTHOR_MAP` | `env_or_setting_name` | 1 | `src/hermes-agent/scripts/release.py` |
| `AUTH_APPLE_ID` | `env_or_setting_name` | 1 | `src/rusty-idd/third_party/upstream/prompts.chat/src/lib/plugins/auth/apple.ts` |
| `AUTH_APPLE_SECRET` | `env_or_setting_name` | 1 | `src/rusty-idd/third_party/upstream/prompts.chat/src/lib/plugins/auth/apple.ts` |
| `AUTH_ARGS` | `env_or_setting_name` | 1 | `src/hermes-agent/optional-skills/productivity/here-now/scripts/publish.sh` |
| `AUTH_AZURE_AD_CLIENT_ID` | `env_or_setting_name` | 1 | `src/rusty-idd/third_party/upstream/prompts.chat/AGENTS.md` |
| `AUTH_AZURE_AD_CLIENT_SECRET` | `env_or_setting_name` | 1 | `src/rusty-idd/third_party/upstream/prompts.chat/AGENTS.md` |
| `AUTH_AZURE_AD_ISSUER` | `env_or_setting_name` | 1 | `src/rusty-idd/third_party/upstream/prompts.chat/AGENTS.md` |
| `AUTH_CACHE_TTL` | `env_or_setting_name` | 1 | `src/rusty-idd/third_party/upstream/prompts.chat/src/pages/api/mcp.ts` |
| `AUTH_DECISION` | `env_or_setting_name` | 4 | `src/meta-ruvector/.claude/agents/v3/claims-authorizer.md`, `src/meta-ruvector/.codex/mirror/.claude/agents/v3/claims-authorizer.md`, `src/ruflo/v3/@claude-flow/cli/.claude/agents/v3/claims-authorizer.md` |
| `AUTH_ENABLED` | `env_or_setting_name` | 9 | `src/envctl-codedb-config-to-tables/docs/odysseus-adoption.md`, `src/envctl-codedb-config-to-tables/manifest/odysseus.toml`, `src/envctl-nix-components-db/docs/odysseus-adoption.md` |
| `AUTH_ENDPOINT` | `env_or_setting_name` | 1 | `src/hermes-agent/agent/google_oauth.py` |
| `AUTH_FAILED_CODES` | `env_or_setting_name` | 1 | `src/hermes-agent/gateway/platforms/yuanbao.py` |
| `AUTH_FILE` | `env_or_setting_name` | 1 | `src/hermes-agent/optional-skills/security/web-pentest/scripts/recon-scan.sh` |
| `AUTH_FLAG` | `env_or_setting_name` | 1 | `src/meta-ruvector/crates/mcp-brain-server/deploy.sh` |
| `AUTH_GITHUB_ID` | `env_or_setting_name` | 2 | `src/rusty-idd/third_party/upstream/prompts.chat/AGENTS.md`, `src/rusty-idd/third_party/upstream/prompts.chat/SELF-HOSTING.md` |
| `AUTH_GITHUB_SECRET` | `env_or_setting_name` | 2 | `src/rusty-idd/third_party/upstream/prompts.chat/AGENTS.md`, `src/rusty-idd/third_party/upstream/prompts.chat/SELF-HOSTING.md` |
| `AUTH_GOOGLE_ID` | `env_or_setting_name` | 2 | `src/rusty-idd/third_party/upstream/prompts.chat/AGENTS.md`, `src/rusty-idd/third_party/upstream/prompts.chat/SELF-HOSTING.md` |
| `AUTH_GOOGLE_SECRET` | `env_or_setting_name` | 2 | `src/rusty-idd/third_party/upstream/prompts.chat/AGENTS.md`, `src/rusty-idd/third_party/upstream/prompts.chat/SELF-HOSTING.md` |
| `AUTH_HEADER` | `env_or_setting_name` | 3 | `src/meta-ruvector/scripts/create-brainpedia.py`, `src/meta-ruvector/scripts/deploy-wet-job.sh`, `src/meta-ruvector/scripts/historical-crawl-import.sh` |
| `AUTH_KEY` | `env_or_setting_name` | 1 | `src/ruflo/v3/@claude-flow/guidance/README.md` |
| `AUTH_LOCK_TIMEOUT_SECONDS` | `env_or_setting_name` | 1 | `src/hermes-agent/hermes_cli/auth.py` |

## Scan Policy

- Blocked patterns: `**/.env, **/secrets/**, **/private_keys/**, **/*.pem, **/*.key`
- Secret values captured: `false`
