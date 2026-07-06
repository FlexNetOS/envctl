# System Inventory

Generated at: `2026-07-04T23:21:18+00:00`
Task: `ART-100_SYSTEM_INVENTORY`
Target: `flexnetos-vs-lifeos`
Target root: `/home/flexnetos/FlexNetOS`
Descriptor hash: `sha256:b3f653f9e9cda7991821687f041cb540ce3e4342bde03ef333aa0e72a6b42384`

## Coverage

- Files scanned: `106597`
- Blocked paths skipped: `247`
- Directories skipped by generated/cache policy: `408`

| category | count |
|---|---:|
| apis | 116 |
| applications | 2159 |
| databases | 201 |
| jobs | 976 |
| queues | 17251 |
| reports | 491 |
| schedulers | 111 |
| scripts | 6480 |
| services | 45 |

## Applications

| kind | path | detail |
|---|---|---|
| rust_workspace | `src/Cargo.toml` | name=src; members=loop_lib, meta_plugin_protocol |
| rust_workspace | `src/cuda-oxide/Cargo.toml` | name=cuda-oxide; members=crates/cuda-device, crates/cuda-host, crates/cuda-macros, crates/llvm-export, crates/dialect-mir, crates/dialect-nvvm, crates/mir-importer, crates/mir-lower |
| rust_crate | `src/cuda-oxide/crates/cargo-oxide/Cargo.toml` | name=cargo-oxide |
| rust_crate | `src/cuda-oxide/crates/cuda-bindings/Cargo.toml` | name=cuda-bindings |
| rust_crate | `src/cuda-oxide/crates/cuda-core/Cargo.toml` | name=cuda-core |
| rust_crate | `src/cuda-oxide/crates/cuda-device/Cargo.toml` | name=cuda-device |
| rust_crate | `src/cuda-oxide/crates/cuda-host/Cargo.toml` | name=cuda-host |
| rust_crate | `src/cuda-oxide/crates/cuda-macros/Cargo.toml` | name=cuda-macros |
| rust_crate | `src/cuda-oxide/crates/dialect-mir/Cargo.toml` | name=dialect-mir |
| rust_crate | `src/cuda-oxide/crates/dialect-nvvm/Cargo.toml` | name=dialect-nvvm |
| rust_crate | `src/cuda-oxide/crates/libnvvm-sys/Cargo.toml` | name=libnvvm-sys |
| rust_crate | `src/cuda-oxide/crates/llvm-export/Cargo.toml` | name=llvm-export |
| rust_crate | `src/cuda-oxide/crates/mir-importer/Cargo.toml` | name=mir-importer |
| rust_crate | `src/cuda-oxide/crates/mir-transforms/Cargo.toml` | name=mir-transforms |
| rust_crate | `src/cuda-oxide/crates/nvjitlink-sys/Cargo.toml` | name=nvjitlink-sys |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/Cargo.toml` | name=rustc_codegen_cuda |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/abi_hmm/Cargo.toml` | name=abi_hmm |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/addressof_sharedarray/Cargo.toml` | name=addressof_sharedarray |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/array_constants/Cargo.toml` | name=array_constants |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/array_for_loop/Cargo.toml` | name=array_for_loop |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/array_index/Cargo.toml` | name=array_index |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/array_init/Cargo.toml` | name=array_init |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/asin_acos_smoke/Cargo.toml` | name=asin_acos_smoke |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/async_mlp/Cargo.toml` | name=async_mlp |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/async_vecadd/Cargo.toml` | name=async_vecadd |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/atomic_f16/Cargo.toml` | name=atomic_f16 |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/atomics/Cargo.toml` | name=atomics |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/barrier/Cargo.toml` | name=barrier |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/bf16x2_arith/Cargo.toml` | name=bf16x2_arith |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/bf16x2_fma/Cargo.toml` | name=bf16x2_fma |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/carrying_mul_add/Cargo.toml` | name=carrying_mul_add |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/cast_tests/Cargo.toml` | name=cast_tests |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/cbrt_smoke/Cargo.toml` | name=cbrt_smoke |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/checked_arith/Cargo.toml` | name=checked_arith |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/clc/Cargo.toml` | name=clc |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/cluster/Cargo.toml` | name=cluster |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/compiler_features/Cargo.toml` | name=compiler_features |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/complex_mul/Cargo.toml` | name=complex_mul |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/const_aggregate/Cargo.toml` | name=const_aggregate |
| rust_crate | `src/cuda-oxide/crates/rustc-codegen-cuda/examples/const_generic/Cargo.toml` | name=const_generic |
| ... | ... | 160 additional entries in JSON artifact |

## Services

| kind | path | detail |
|---|---|---|
| systemd_service | `src/envctl-codedb-config-to-tables/home/.config/systemd/user/env-ctl.service` | description=env-ctl secrets vault + credential broker |
| systemd_service | `src/envctl-codedb-config-to-tables/home/.config/systemd/user/repowire.service` | description=Repowire Daemon |
| systemd_service | `src/envctl-codedb-config-to-tables/home/.config/systemd/user/sqld.service` | description=sqld (libSQL server, loopback) — durable store backend for env-ctl secretd |
| systemd_service | `src/envctl-db-nu-migration-req020/home/.config/systemd/user/env-ctl.service` | description=env-ctl secrets vault + credential broker |
| systemd_service | `src/envctl-db-nu-migration-req020/home/.config/systemd/user/repowire.service` | description=Repowire Daemon |
| systemd_service | `src/envctl-db-nu-migration-req020/home/.config/systemd/user/sqld.service` | description=sqld (libSQL server, loopback) — durable store backend for env-ctl secretd |
| systemd_service | `src/envctl-hooks-clean-baseline/home/.config/systemd/user/env-ctl.service` | description=env-ctl secrets vault + credential broker |
| systemd_service | `src/envctl-hooks-clean-baseline/home/.config/systemd/user/repowire.service` | description=Repowire Daemon |
| systemd_service | `src/envctl-hooks-clean-baseline/home/.config/systemd/user/sqld.service` | description=sqld (libSQL server, loopback) — durable store backend for env-ctl secretd |
| systemd_service | `src/envctl-nix-components-db/home/.config/systemd/user/env-ctl.service` | description=env-ctl secrets vault + credential broker |
| systemd_service | `src/envctl-nix-components-db/home/.config/systemd/user/repowire.service` | description=Repowire Daemon |
| systemd_service | `src/envctl-nix-components-db/home/.config/systemd/user/sqld.service` | description=sqld (libSQL server, loopback) — durable store backend for env-ctl secretd |
| systemd_service | `src/envctl-nu-plugin-db-skill/home/.config/systemd/user/env-ctl.service` | description=env-ctl secrets vault + credential broker |
| systemd_service | `src/envctl-nu-plugin-db-skill/home/.config/systemd/user/repowire.service` | description=Repowire Daemon |
| systemd_service | `src/envctl-nu-plugin-db-skill/home/.config/systemd/user/sqld.service` | description=sqld (libSQL server, loopback) — durable store backend for env-ctl secretd |
| systemd_service | `src/envctl/home/.config/systemd/user/env-ctl.service` | description=env-ctl secrets vault + credential broker |
| systemd_service | `src/envctl/home/.config/systemd/user/repowire.service` | description=Repowire Daemon |
| systemd_service | `src/envctl/home/.config/systemd/user/sqld.service` | description=sqld (libSQL server, loopback) — durable store backend for env-ctl secretd |
| systemd_service | `src/flexnetos_runner/systemd/user/flexnetos-cache-maintenance.service` | description=FlexNetOS runner cache maintenance (%i) |
| compose_stack | `src/hermes-agent/docker-compose.yml` | services=gateway, dashboard |
| systemd_service | `src/hermes-agent/plugins/kanban/systemd/hermes-kanban-dispatcher.service` | description=Hermes Kanban dispatcher (DEPRECATED standalone daemon — prefer gateway-embedded dispatch) |
| compose_stack | `src/hermes-agent/tests/e2e/matrix_xsign_bootstrap/docker-compose.yml` | services=homeserver |
| systemd_service | `src/meta-ruvector/crates/ruos-thermal/deploy/ruos-thermal.service` | description=ruOS thermal supervisor — Pi 5 + Hailo-8 thermal snapshot |
| systemd_service | `src/meta-ruvector/crates/ruvector-hailo-cluster/deploy/ruvector-hailo-worker.service` | description=ruvector Hailo embedding worker (ADR-172 §3a hardened) |
| systemd_service | `src/meta-ruvector/crates/ruvector-hailo-cluster/deploy/ruvector-mmwave-bridge.service` | description=ruvector mmWave bridge — 60 GHz radar UART → cluster embed RPC (ADR-063, ADR-167 §8) |
| systemd_service | `src/meta-ruvector/crates/ruvector-hailo-cluster/deploy/ruview-csi-bridge.service` | description=ruvector RuView CSI bridge — ADR-018 UDP → cluster embed RPC (ADR-171, iter 123) |
| systemd_service | `src/meta-ruvector/crates/ruvector-hailo-cluster/deploy/ruvllm-pi-worker.service` | description=ruvllm Pi LLM completion worker (ADR-179) |
| compose_stack | `src/meta-ruvector/crates/ruvector-postgres/docker/docker-compose.yml` | services=postgres, test-runner, benchmark, dev, postgres-pg14, postgres-pg15, postgres-pg16 |
| compose_stack | `src/meta-ruvector/examples/edge-net/dashboard/docker-compose.yml` | services=dashboard, dashboard-dev |
| compose_stack | `src/meta-ruvector/npm/packages/ruvbot/docker-compose.yml` | services=ruvbot, postgres, redis, adminer |
| compose_stack | `src/meta-ruvector/scripts/n8n/docker-compose.yml` | services=n8n, ruvector |
| compose_stack | `src/meta-ruvector/tests/integration/distributed/docker-compose.yml` | services=raft-node-1, raft-node-2, raft-node-3, raft-node-4, raft-node-5, test-runner |
| compose_stack | `src/meta-ruvector/ui/ruvocal/docker-compose.yml` | services=mongo |
| compose_stack | `src/network-control/infrastructure/mcp/docker-compose.yaml` | services=omada-controller, omada-mcp, omada-mcp-http |
| compose_stack | `src/prompt_hub/docker/docker-compose.yml` | services=prompthub |
| compose_stack | `src/ruflo/ruflo/docker-compose.yml` | services=mongodb, mcp-bridge, nginx, chat-ui |
| compose_stack | `src/ruflo/ruflo/src/ruvocal/docker-compose.yml` | services=mongo |
| compose_stack | `src/ruflo/tests/docker-regression/docker-compose.yml` | services=test-runner, mcp-server, unit-tests, integration-tests, benchmark-tests, security-tests |
| compose_stack | `src/ruflo/v3/@claude-flow/browser/docker/docker-compose.yml` | services=browser-tests, browser-debug, browser-e2e, test-server |
| compose_stack | `src/ruflo/v3/@claude-flow/cli/docker/docker-compose.yml` | services=mcp, ruflo-full, worker |
| ... | ... | 5 additional entries in JSON artifact |

## Jobs

| kind | path | detail |
|---|---|---|
| github_workflow | `src/beads_rust/.github/workflows/audit.yml` |  |
| github_workflow | `src/beads_rust/.github/workflows/ci.yml` |  |
| github_workflow | `src/beads_rust/.github/workflows/conformance.yml` |  |
| github_workflow | `src/beads_rust/.github/workflows/doctor.yml` |  |
| github_workflow | `src/beads_rust/.github/workflows/e2e-full.yml` |  |
| github_workflow | `src/beads_rust/.github/workflows/notify-acfs.yml` |  |
| github_workflow | `src/beads_rust/.github/workflows/release.yml` |  |
| github_workflow | `src/beads_rust/.github/workflows/update-package-manifests.yml` |  |
| github_workflow | `src/cuda-oxide/.github/workflows/book.yml` |  |
| github_workflow | `src/cuda-oxide/.github/workflows/cargo-deny.yml` |  |
| github_workflow | `src/cuda-oxide/.github/workflows/ci.yml` |  |
| github_workflow | `src/cuda-oxide/.github/workflows/clippy.yml` |  |
| github_workflow | `src/cuda-oxide/.github/workflows/cuda-bindings-guard.yml` |  |
| github_workflow | `src/cuda-oxide/.github/workflows/differential-drive.yml` |  |
| github_workflow | `src/cuda-oxide/.github/workflows/docs.yml` |  |
| github_workflow | `src/cuda-oxide/.github/workflows/examples-compile.yml` |  |
| github_workflow | `src/cuda-oxide/.github/workflows/fmt.yml` |  |
| github_workflow | `src/cuda-oxide/.github/workflows/naming-guard.yml` |  |
| github_workflow | `src/cuda-oxide/.github/workflows/status-guard.yml` |  |
| github_workflow | `src/cuda-oxide/.github/workflows/unit-tests.yml` |  |
| justfile | `src/cuda-oxide/Justfile` |  |
| makefile | `src/cuda-oxide/cuda-oxide-book/Makefile` |  |
| github_workflow | `src/envctl/.github/workflows/ci.yml` |  |
| github_workflow | `src/envctl/.github/workflows/sync-master.yml` |  |
| github_workflow | `src/hermes-agent/.github/workflows/contributor-check.yml` |  |
| github_workflow | `src/hermes-agent/.github/workflows/deploy-site.yml` |  |
| github_workflow | `src/hermes-agent/.github/workflows/differential-drive.yml` |  |
| github_workflow | `src/hermes-agent/.github/workflows/docker-lint.yml` |  |
| github_workflow | `src/hermes-agent/.github/workflows/docker-publish.yml` |  |
| github_workflow | `src/hermes-agent/.github/workflows/docs-site-checks.yml` |  |
| github_workflow | `src/hermes-agent/.github/workflows/history-check.yml` |  |
| github_workflow | `src/hermes-agent/.github/workflows/lint.yml` |  |
| github_workflow | `src/hermes-agent/.github/workflows/nix-lockfile-fix.yml` |  |
| github_workflow | `src/hermes-agent/.github/workflows/nix.yml` |  |
| github_workflow | `src/hermes-agent/.github/workflows/osv-scanner.yml` |  |
| github_workflow | `src/hermes-agent/.github/workflows/skills-index-freshness.yml` |  |
| github_workflow | `src/hermes-agent/.github/workflows/skills-index.yml` |  |
| github_workflow | `src/hermes-agent/.github/workflows/supply-chain-audit.yml` |  |
| github_workflow | `src/hermes-agent/.github/workflows/tests.yml` |  |
| github_workflow | `src/hermes-agent/.github/workflows/upload_to_pypi.yml` |  |
| ... | ... | 160 additional entries in JSON artifact |

## Databases

| kind | path | detail |
|---|---|---|
| schema_or_migration | `src/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/generated/contract_manifest.seed.sql` |  |
| schema_or_migration | `src/envctl/envctl-db-nu-plugin-migration-automation-package/sql/001_migration_automation_schema.sql` |  |
| schema_or_migration | `src/envctl/envctl-db-nu-plugin-migration-automation-package/sql/002_views_and_indexes.sql` |  |
| schema_or_migration | `src/envctl/envctl-db-nu-plugin-migration-automation-package/sql/003_seed_artifact_contract.sql` |  |
| schema_or_migration | `src/hermes-agent/optional-skills/migration/DESCRIPTION.md` |  |
| schema_or_migration | `src/hermes-agent/optional-skills/migration/openclaw-migration/SKILL.md` |  |
| schema_or_migration | `src/hermes-agent/optional-skills/migration/openclaw-migration/scripts/openclaw_to_hermes.py` |  |
| schema_or_migration | `src/hermes-agent/website/docs/user-guide/skills/optional/migration/migration-openclaw-migration.md` |  |
| schema_or_migration | `src/hermes-agent/website/i18n/zh-Hans/docusaurus-plugin-content-docs/current/user-guide/skills/optional/migration/migration-openclaw-migration.md` |  |
| schema_or_migration | `src/lifeos/crates/lifeos-core/migrations/0001_accounts.sql` |  |
| schema_or_migration | `src/lifeos/crates/lifeos-core/migrations/0002_mempalace.sql` |  |
| schema_or_migration | `src/lifeos/crates/lifeos-core/migrations/0003_ruvector.sql` |  |
| schema_or_migration | `src/meta-ruvector/crates/ruvector-cli/sql/hooks_schema.sql` |  |
| schema_or_migration | `src/meta-ruvector/crates/ruvector-postgres/benches/sql/benchmark_workload.sql` |  |
| schema_or_migration | `src/meta-ruvector/crates/ruvector-postgres/benches/sql/quick_benchmark.sql` |  |
| compose_database_candidate | `src/meta-ruvector/crates/ruvector-postgres/docker/docker-compose.yml` | service=postgres |
| compose_database_candidate | `src/meta-ruvector/crates/ruvector-postgres/docker/docker-compose.yml` | service=test-runner |
| compose_database_candidate | `src/meta-ruvector/crates/ruvector-postgres/docker/docker-compose.yml` | service=benchmark |
| compose_database_candidate | `src/meta-ruvector/crates/ruvector-postgres/docker/docker-compose.yml` | service=dev |
| compose_database_candidate | `src/meta-ruvector/crates/ruvector-postgres/docker/docker-compose.yml` | service=postgres-pg14 |
| compose_database_candidate | `src/meta-ruvector/crates/ruvector-postgres/docker/docker-compose.yml` | service=postgres-pg15 |
| compose_database_candidate | `src/meta-ruvector/crates/ruvector-postgres/docker/docker-compose.yml` | service=postgres-pg16 |
| schema_or_migration | `src/meta-ruvector/crates/ruvector-postgres/docker/init-integration.sql` |  |
| schema_or_migration | `src/meta-ruvector/crates/ruvector-postgres/docker/init.sql` |  |
| schema_or_migration | `src/meta-ruvector/crates/ruvector-postgres/docs/examples/self-learning-usage.sql` |  |
| schema_or_migration | `src/meta-ruvector/crates/ruvector-postgres/examples/sparse_example.sql` |  |
| schema_or_migration | `src/meta-ruvector/crates/ruvector-postgres/sql/access_methods.sql` |  |
| schema_or_migration | `src/meta-ruvector/crates/ruvector-postgres/sql/embeddings.sql` |  |
| schema_or_migration | `src/meta-ruvector/crates/ruvector-postgres/sql/graph_examples.sql` |  |
| schema_or_migration | `src/meta-ruvector/crates/ruvector-postgres/sql/hnsw_index.sql` |  |
| schema_or_migration | `src/meta-ruvector/crates/ruvector-postgres/sql/ivfflat_am.sql` |  |
| schema_or_migration | `src/meta-ruvector/crates/ruvector-postgres/sql/routing_example.sql` |  |
| schema_or_migration | `src/meta-ruvector/crates/ruvector-postgres/sql/ruvector--0.1.0.sql` |  |
| schema_or_migration | `src/meta-ruvector/crates/ruvector-postgres/sql/ruvector--0.3.0.sql` |  |
| schema_or_migration | `src/meta-ruvector/crates/ruvector-postgres/sql/ruvector--2.0.0--0.3.0.sql` |  |
| schema_or_migration | `src/meta-ruvector/crates/ruvector-postgres/sql/ruvector--2.0.0.sql` |  |
| schema_or_migration | `src/meta-ruvector/crates/ruvector-postgres/tests/hnsw_index_tests.sql` |  |
| schema_or_migration | `src/meta-ruvector/crates/ruvector-postgres/tests/ivfflat_am_test.sql` |  |
| schema_or_migration | `src/meta-ruvector/docs/examples/sparsevec_examples.sql` |  |
| schema_or_migration | `src/meta-ruvector/docs/sql/parallel-examples.sql` |  |
| ... | ... | 160 additional entries in JSON artifact |

## Queues

| kind | path | detail |
|---|---|---|
| queue_signal | `.kb/AGENTS.md` |  |
| queue_signal | `.kb/config.toml` |  |
| queue_signal | `.kb/skills/kb-board/SKILL.md` |  |
| queue_signal | `.kb/skills/kb-create/SKILL.md` |  |
| queue_signal | `AGENTS.md` |  |
| queue_signal | `LOCAL_WORKAROUNDS.md` |  |
| queue_signal | `WORKLOG.md` |  |
| queue_signal | `WORKSPACE_LAYOUT.md` |  |
| queue_signal | `release-baseline.json` |  |
| queue_signal | `src/OWNERSHIP.md` |  |
| queue_signal | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/references/config-preflight.md` |  |
| queue_signal | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/references/final-report.md` |  |
| queue_signal | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/references/scan-contract.md` |  |
| queue_signal | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/scripts/finalize_scan_contract.py` |  |
| queue_signal | `src/envctl/.kb/AGENTS.md` |  |
| queue_signal | `src/envctl/.kb/config.toml` |  |
| queue_signal | `src/envctl/.kb/skills/kb-board/SKILL.md` |  |
| queue_signal | `src/envctl/.kb/skills/kb-create/SKILL.md` |  |
| queue_signal | `src/envctl/.kb/workspaces/main/tasks/import-host-artifacts-yazelix-mission-control-20260703.md` |  |
| queue_signal | `src/envctl/AGENTS.md` |  |
| queue_signal | `src/envctl/CLAUDE.md` |  |
| queue_signal | `src/envctl/Cargo.toml` |  |
| queue_signal | `src/envctl/HANDOFF.md` |  |
| queue_signal | `src/envctl/LESSONS.md` |  |
| queue_signal | `src/envctl/crates/agent-env/src/extend.rs` |  |
| queue_signal | `src/envctl/crates/agent-env/src/mcp.rs` |  |
| queue_signal | `src/envctl/crates/agent-env/tests/parity_vs_kasetto.rs` |  |
| queue_signal | `src/envctl/crates/cli/src/main.rs` |  |
| queue_signal | `src/envctl/crates/engine/Cargo.toml` |  |
| queue_signal | `src/envctl/crates/engine/src/addrepo.rs` |  |
| queue_signal | `src/envctl/crates/engine/src/command.rs` |  |
| queue_signal | `src/envctl/crates/engine/src/component.rs` |  |
| queue_signal | `src/envctl/crates/engine/src/dashboard.rs` |  |
| queue_signal | `src/envctl/crates/engine/src/event.rs` |  |
| queue_signal | `src/envctl/crates/engine/src/executor.rs` |  |
| queue_signal | `src/envctl/crates/engine/src/install.rs` |  |
| queue_signal | `src/envctl/crates/engine/src/lib.rs` |  |
| queue_signal | `src/envctl/crates/engine/src/model.rs` |  |
| queue_signal | `src/envctl/crates/engine/src/peer.rs` |  |
| queue_signal | `src/envctl/crates/engine/src/runner.rs` |  |
| ... | ... | 160 additional entries in JSON artifact |

## Apis

| kind | path | detail |
|---|---|---|
| api_surface | `oh-my-codex-main/.codex/.tmp/plugins/plugins/cloudflare/skills/cloudflare/references/workerd/patterns.md` |  |
| api_surface | `oh-my-codex-main/.codex/.tmp/plugins/plugins/cloudflare/skills/cloudflare/references/workers/frameworks.md` |  |
| api_surface | `oh-my-codex-main/.codex/.tmp/plugins/plugins/zoom/skills/oauth/examples/s2s-oauth-redis.md` |  |
| api_surface | `oh-my-codex-main/.codex/.tmp/plugins/plugins/zoom/skills/rest-api/references/openapi.md` |  |
| api_surface | `oh-my-codex-main/.codex/.tmp/plugins/plugins/zoom/skills/zoom-apps-sdk/examples/in-client-oauth.md` |  |
| api_surface | `src/envctl/envctl-db-nu-plugin-migration-automation-package/schemas/envctl_migration_api.openapi.json` |  |
| api_surface | `src/hermes-agent/hermes_cli/dashboard_auth/routes.py` |  |
| api_surface | `src/hermes-agent/plugins/example-dashboard/dashboard/plugin_api.py` |  |
| api_surface | `src/hermes-agent/plugins/hermes-achievements/dashboard/plugin_api.py` |  |
| api_surface | `src/hermes-agent/plugins/kanban/dashboard/plugin_api.py` |  |
| api_surface | `src/hermes-agent/tests/agent/test_model_metadata.py` |  |
| api_surface | `src/hermes-agent/tests/agent/test_models_dev.py` |  |
| api_surface | `src/hermes-agent/tests/cli/test_cli_shutdown_memory_messages.py` |  |
| api_surface | `src/hermes-agent/tests/cli/test_reasoning_command.py` |  |
| api_surface | `src/hermes-agent/tests/cli/test_session_boundary_hooks.py` |  |
| api_surface | `src/hermes-agent/tests/cli/test_surrogate_sanitization.py` |  |
| api_surface | `src/hermes-agent/tests/cron/test_file_permissions.py` |  |
| api_surface | `src/hermes-agent/tests/gateway/test_api_server.py` |  |
| api_surface | `src/hermes-agent/tests/gateway/test_api_server_toolset.py` |  |
| api_surface | `src/hermes-agent/tests/gateway/test_email.py` |  |
| api_surface | `src/hermes-agent/tests/gateway/test_feishu_comment.py` |  |
| api_surface | `src/hermes-agent/tests/gateway/test_feishu_onboard.py` |  |
| api_surface | `src/hermes-agent/tests/gateway/test_homeassistant.py` |  |
| api_surface | `src/hermes-agent/tests/gateway/test_matrix_voice.py` |  |
| api_surface | `src/hermes-agent/tests/gateway/test_mattermost.py` |  |
| api_surface | `src/hermes-agent/tests/gateway/test_media_download_retry.py` |  |
| api_surface | `src/hermes-agent/tests/gateway/test_session_boundary_hooks.py` |  |
| api_surface | `src/hermes-agent/tests/gateway/test_setup_feishu.py` |  |
| api_surface | `src/hermes-agent/tests/gateway/test_webhook_adapter.py` |  |
| api_surface | `src/hermes-agent/tests/gateway/test_wecom.py` |  |
| api_surface | `src/hermes-agent/tests/gateway/test_weixin.py` |  |
| api_surface | `src/hermes-agent/tests/hermes_cli/test_cmd_update.py` |  |
| api_surface | `src/hermes-agent/tests/hermes_cli/test_cmd_update_docker.py` |  |
| api_surface | `src/hermes-agent/tests/hermes_cli/test_copilot_context.py` |  |
| api_surface | `src/hermes-agent/tests/hermes_cli/test_copilot_token_exchange.py` |  |
| api_surface | `src/hermes-agent/tests/hermes_cli/test_models.py` |  |
| api_surface | `src/hermes-agent/tests/hermes_cli/test_plugins_cmd.py` |  |
| api_surface | `src/hermes-agent/tests/hermes_cli/test_skills_config.py` |  |
| api_surface | `src/hermes-agent/tests/hermes_cli/test_skills_skip_confirm.py` |  |
| api_surface | `src/hermes-agent/tests/hermes_cli/test_update_yes_flag.py` |  |
| ... | ... | 76 additional entries in JSON artifact |

## Reports

| kind | path | detail |
|---|---|---|
| report_artifact | `src/beads_rust/docs/agent/AGENT_FRIENDLINESS_REPORT.md` |  |
| report_artifact | `src/beads_rust/refactor/artifacts/2026-04-24-shrink-pass-1/verify_report.md` |  |
| report_artifact | `src/beads_rust/tests/artifacts/perf/beads-perf-20260504T-swarm-capacity-planning/report.json` |  |
| report_artifact | `src/beads_rust/tests/artifacts/perf/beads-perf-20260504T-swarm-capacity-planning/report.md` |  |
| report_artifact | `src/cuda-oxide/.github/ISSUE_TEMPLATE/bug_report.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/_done/g2-native-mint.03_guardian_report.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/_done/prev-loop.2026-06-17.03_guardian_report.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/_done/task-0019.2026-06-17.03_guardian_report.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/cycle/03_guardian_report.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/cycle/_done/03_guardian_report-task0033-20260627.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/cycle/_done/03_guardian_report-task0078-next-inventory-20260627-pre-sensitive-hints.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/ADR-DRAFT-handoff-rusty-idd-union.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/ADR-DRAFT-rusty-idd-convergence-boundary.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/ADR-DRAFT-weave-a2a-interop.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/ADR-DRAFT-weave-cross-vendor-model-lane.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/ROADMAP-grit.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/ROADMAP-handoff.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/ROADMAP-prompt-hub.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/ROADMAP-rusty-idd.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/ROADMAP-weave.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/TASK-0078-migration-adoption-engine-v2-design.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/adr-draft-grit-reconciler.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/adr-draft-prompt-hub-goal-artifact.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/agent-run-ledger-grit.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/agent-run-ledger-handoff.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/agent-run-ledger-icm.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/agent-run-ledger-prompt-hub.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/agent-run-ledger-rusty-idd.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/agent-run-ledger-weave.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/codemap-grit.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/codemap-handoff.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/codemap-icm.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/codemap-prompt-hub.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/codemap-rusty-idd.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/codemap-weave.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/grit-plan.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/handoff-plan.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/icm-plan.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/north-star-DRAFT.md` |  |
| report_artifact | `src/envctl-nix-components-db/.handoff/loop/plan/reports/prompt-hub-plan.md` |  |
| ... | ... | 160 additional entries in JSON artifact |

## Scripts

| kind | path | detail |
|---|---|---|
| script | `src/envctl/.codex/archive/lifecycle-hooks-20260703T024950Z/hooks/flexnetos-runtime-gate.sh.md` |  |
| script | `src/envctl/.codex/archive/lifecycle-hooks-20260703T024950Z/hooks/hf-checkpoint.sh.md` |  |
| script | `src/envctl/.codex/archive/lifecycle-hooks-20260703T024950Z/hooks/install-flexnetos-runtime-hooks.sh.md` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/scripts/config_preflight.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/scripts/filesystem_identity.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/scripts/finalize_scan_contract.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/scripts/finding_preview.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/scripts/generate_rank_input.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/scripts/report_projection.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/scripts/snapshot_sqlite.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/scripts/validate_report_format.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/scripts/validate_tracking_source.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/scripts/windows_scan_local_files.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/scripts/workbench_constants.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/scripts/workbench_db.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/scripts/workbench_progress.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/scripts/workbench_remediation.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/scripts/workbench_schema.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/scripts/workbench_source_excerpt.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/scripts/workbench_target.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/scripts/workbench_validation.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/digitalocean/0.2.2/skills/provision-droplet/scripts/configure_ssh.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/digitalocean/0.2.2/skills/provision-droplet/scripts/keygen.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/figma/2.0.12/scripts/post_write_figma_parity_check.sh` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/figma/2.0.12/skills/figma-generate-library/scripts/bindVariablesToComponent.js` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/figma/2.0.12/skills/figma-generate-library/scripts/cleanupOrphans.js` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/figma/2.0.12/skills/figma-generate-library/scripts/createComponentWithVariants.js` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/figma/2.0.12/skills/figma-generate-library/scripts/createDocumentationPage.js` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/figma/2.0.12/skills/figma-generate-library/scripts/createSemanticTokens.js` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/figma/2.0.12/skills/figma-generate-library/scripts/createVariableCollection.js` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/figma/2.0.12/skills/figma-generate-library/scripts/inspectFileStructure.js` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/figma/2.0.12/skills/figma-generate-library/scripts/rehydrateState.js` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/figma/2.0.12/skills/figma-generate-library/scripts/validateCreation.js` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/github/0.1.6/skills/gh-address-comments/scripts/fetch_comments.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/github/0.1.6/skills/gh-fix-ci/scripts/inspect_pr_checks.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/google-calendar/1.2.3/skills/google-calendar-daily-brief/scripts/render_day_brief.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/hugging-face/1.0.3/skills/community-evals/scripts/inspect_eval_uv.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/hugging-face/1.0.3/skills/community-evals/scripts/inspect_vllm_uv.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/hugging-face/1.0.3/skills/community-evals/scripts/lighteval_vllm_uv.py` |  |
| script | `src/envctl/.codex/plugins/cache/meta-plugins-codex/hugging-face/1.0.3/skills/jobs/scripts/cot-self-instruct.py` |  |
| ... | ... | 160 additional entries in JSON artifact |

## Schedulers

| kind | path | detail |
|---|---|---|
| cron_candidate | `oh-my-codex-main/.codex/.tmp/plugins/plugins/cloudflare/skills/cloudflare/references/cron-triggers/README.md` |  |
| cron_candidate | `oh-my-codex-main/.codex/.tmp/plugins/plugins/cloudflare/skills/cloudflare/references/cron-triggers/api.md` |  |
| cron_candidate | `oh-my-codex-main/.codex/.tmp/plugins/plugins/cloudflare/skills/cloudflare/references/cron-triggers/configuration.md` |  |
| cron_candidate | `oh-my-codex-main/.codex/.tmp/plugins/plugins/cloudflare/skills/cloudflare/references/cron-triggers/gotchas.md` |  |
| cron_candidate | `oh-my-codex-main/.codex/.tmp/plugins/plugins/cloudflare/skills/cloudflare/references/cron-triggers/patterns.md` |  |
| scheduled_workflow | `oh-my-codex-main/.codex/.tmp/plugins/plugins/render/.github/workflows/sync-skills.yml` |  |
| cron_candidate | `oh-my-codex-main/.codex/.tmp/plugins/plugins/render/skills/render-cron-jobs/SKILL.md` |  |
| cron_candidate | `oh-my-codex-main/.codex/.tmp/plugins/plugins/render/skills/render-cron-jobs/agents/openai.yaml` |  |
| cron_candidate | `oh-my-codex-main/.codex/.tmp/plugins/plugins/render/skills/render-cron-jobs/references/cron-patterns.md` |  |
| cron_candidate | `oh-my-codex-main/.codex/.tmp/plugins/plugins/render/skills/render-cron-jobs/references/migration-from-scheduler.md` |  |
| cron_candidate | `oh-my-codex-main/.codex/.tmp/plugins/plugins/vercel/skills/cron-jobs/SKILL.md` |  |
| cron_candidate | `oh-my-codex-main/.codex/.tmp/plugins/plugins/vercel/skills/cron-jobs/agents/openai.yaml` |  |
| scheduled_workflow | `src/beads_rust/.github/workflows/audit.yml` |  |
| scheduled_workflow | `src/beads_rust/.github/workflows/conformance.yml` |  |
| scheduled_workflow | `src/beads_rust/.github/workflows/e2e-full.yml` |  |
| scheduled_workflow | `src/beads_viewer/.github/workflows/flake-update.yml` |  |
| scheduled_workflow | `src/beads_viewer/.github/workflows/fuzz.yml` |  |
| scheduled_workflow | `src/cuda-oxide/.github/workflows/cargo-deny.yml` |  |
| scheduled_workflow | `src/flexnetos_runner/.github/workflows/agentic-system-watch.yml` |  |
| scheduled_workflow | `src/flexnetos_runner/.github/workflows/runner-black-factor-watch.yml` |  |
| scheduled_workflow | `src/flexnetos_runner/.github/workflows/runner-sustain.yml` |  |
| systemd_timer | `src/flexnetos_runner/systemd/user/flexnetos-cache-maintenance.timer` | description=Run FlexNetOS runner cache maintenance during idle windows |
| scheduled_workflow | `src/hermes-agent/.github/workflows/osv-scanner.yml` |  |
| scheduled_workflow | `src/hermes-agent/.github/workflows/skills-index-freshness.yml` |  |
| scheduled_workflow | `src/hermes-agent/.github/workflows/skills-index.yml` |  |
| cron_candidate | `src/hermes-agent/cron/__init__.py` |  |
| cron_candidate | `src/hermes-agent/cron/jobs.py` |  |
| cron_candidate | `src/hermes-agent/cron/scheduler.py` |  |
| cron_candidate | `src/hermes-agent/hermes_cli/cron.py` |  |
| cron_candidate | `src/hermes-agent/tests/cron/__init__.py` |  |
| cron_candidate | `src/hermes-agent/tests/cron/test_codex_execution_paths.py` |  |
| cron_candidate | `src/hermes-agent/tests/cron/test_compute_next_run_last_run_at.py` |  |
| cron_candidate | `src/hermes-agent/tests/cron/test_cron_context_from.py` |  |
| cron_candidate | `src/hermes-agent/tests/cron/test_cron_inactivity_timeout.py` |  |
| cron_candidate | `src/hermes-agent/tests/cron/test_cron_no_agent.py` |  |
| cron_candidate | `src/hermes-agent/tests/cron/test_cron_profile.py` |  |
| cron_candidate | `src/hermes-agent/tests/cron/test_cron_prompt_injection_skill.py` |  |
| cron_candidate | `src/hermes-agent/tests/cron/test_cron_script.py` |  |
| cron_candidate | `src/hermes-agent/tests/cron/test_cron_workdir.py` |  |
| cron_candidate | `src/hermes-agent/tests/cron/test_cronjob_schema.py` |  |
| ... | ... | 71 additional entries in JSON artifact |
