# ART-116 Infrastructure Topology Map

Generated: `2026-07-28T11:31:38+00:00`

This map is built from the target descriptor, generated repo/package scan, envctl database reports, and a safe filename-only scan of the target filesystem. It records repo evidence and database control-plane topology; it does not claim live cloud/runtime inventory unless that evidence is present.

## Target

- Target: `flexnetos-vs-lifeos`
- Primary root: `/home/flexnetos/FlexNetOS`
- Compare root: `/home/flexnetos/lifeos`
- Target registry status: `passed`
- Safe scan visited files: `9984`

## Coverage

| Category | Status | Evidence count | Sample evidence |
| --- | --- | --- | --- |
| compute | repo_evidence_found | 40 | `src/flexnetos_runner/_work/actions-runner-02-work/envctl/Cargo.toml`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/Cargo.toml`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/Cargo.toml`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/tests/relay.rs`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/tests/inject.rs` |
| networking | repo_evidence_found | 40 | `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.mcp.json`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/mcp_hub/registry.json`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/mcp_hub/entries/playwright-mcp.md`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/mcp_hub/entries/n8n-mcp.md`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/tests/relay.rs` |
| storage | repo_evidence_found | 40 | `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/tests/db_parity.rs`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/tests/db_refactor_fixtures.rs`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/db.rs`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/db_components.rs`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/db_watch.rs` |
| dns | repo_evidence_found | 4 | `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/third_party/ruvector-postgres-2.0.5/src/domain_expansion/mod.rs`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/third_party/ruvector-postgres-2.0.5/src/domain_expansion/operators.rs`<br>`src/flexnetos_runner/_work/actions-runner-01-work/envctl/envctl/third_party/ruvector-postgres-2.0.5/src/domain_expansion/mod.rs`<br>`src/flexnetos_runner/_work/actions-runner-01-work/envctl/envctl/third_party/ruvector-postgres-2.0.5/src/domain_expansion/operators.rs` |
| load_balancers | not_found_in_safe_scan | 0 | none |
| firewalls | repo_evidence_found | 2 | `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.codex/prompts/prompt:codex-gpt-harness-v3-full-access-no-sandbox.prompt.md`<br>`src/flexnetos_runner/_work/actions-runner-01-work/envctl/envctl/.codex/prompts/prompt:codex-gpt-harness-v3-full-access-no-sandbox.prompt.md` |
| certificates | repo_evidence_found | 2 | `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secretd/src/edge/tls.rs`<br>`src/flexnetos_runner/_work/actions-runner-01-work/envctl/envctl/crates/secretd/src/edge/tls.rs` |

## Topology Nodes

| Node | Kind | Evidence |
| --- | --- | --- |
| FlexNetOS versus lifeos target | migration_target | `generated/envctl_target_registry.json`<br>`../examples/target-descriptors/flexnetos-vs-lifeos.yaml` |
| Workspace source repositories and local build/runtime surfaces | compute | `src/flexnetos_runner/_work/actions-runner-02-work/envctl/Cargo.toml`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/Cargo.toml`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/Cargo.toml`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/tests/relay.rs`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/tests/inject.rs` |
| Local service, socket, relay, and container networking evidence | networking | `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.mcp.json`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/mcp_hub/registry.json`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/mcp_hub/entries/playwright-mcp.md`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/mcp_hub/entries/n8n-mcp.md`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/tests/relay.rs` |
| envctl migration database and repository storage surfaces | storage | `generated/envctl_migration_db_model.json`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/tests/db_parity.rs`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/tests/db_refactor_fixtures.rs`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/db.rs`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/db_components.rs` |
| DNS/domain configuration evidence in target filesystem | dns | `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/third_party/ruvector-postgres-2.0.5/src/domain_expansion/mod.rs`<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/third_party/ruvector-postgres-2.0.5/src/domain_expansion/operators.rs`<br>`src/flexnetos_runner/_work/actions-runner-01-work/envctl/envctl/third_party/ruvector-postgres-2.0.5/src/domain_expansion/mod.rs`<br>`src/flexnetos_runner/_work/actions-runner-01-work/envctl/envctl/third_party/ruvector-postgres-2.0.5/src/domain_expansion/operators.rs` |
| Reverse proxy/load-balancer evidence in target filesystem | load_balancer | none |
| Network policy, sandbox, and firewall-adjacent controls | firewall | `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.codex/prompts/prompt:codex-gpt-harness-v3-full-access-no-sandbox.prompt.md`<br>`src/flexnetos_runner/_work/actions-runner-01-work/envctl/envctl/.codex/prompts/prompt:codex-gpt-harness-v3-full-access-no-sandbox.prompt.md` |
| TLS/certificate dependency and configuration evidence | certificates | `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secretd/src/edge/tls.rs`<br>`src/flexnetos_runner/_work/actions-runner-01-work/envctl/envctl/crates/secretd/src/edge/tls.rs` |
| envctl artifact registry | control_plane | `generated/envctl_artifact_registry_report.json`<br>`scripts/artifact_registry.py` |

## Topology Edges

| From | Type | To |
| --- | --- | --- |
| target:flexnetos-vs-lifeos | contains | compute:workspace-repos |
| target:flexnetos-vs-lifeos | contains | network:local-services |
| target:flexnetos-vs-lifeos | contains | storage:migration-db |
| compute:workspace-repos | exposes_or_runs | network:local-services |
| network:local-services | may_resolve_through | dns:repo-config |
| network:local-services | may_front | load_balancer:repo-config |
| firewall:policy | constrains | network:local-services |
| certificates:tls | secures | network:local-services |
| registry:artifact-registry | persists_to | storage:migration-db |

## Gaps

| Category | Gap | Next evidence needed |
| --- | --- | --- |
| load_balancers | No safe-scan filesystem evidence found for this infrastructure category. | Collect runtime inventory, IaC state, or deployment platform export for this category. |

## Evidence Boundary

- Secret-like paths are excluded by policy: `**/.env`, `**/secrets/**`, `**/private_keys/**`, `**/*.pem`, `**/*.key`.
- The envctl migration database is represented from generated schema/report artifacts and is exercised as SQLite in-memory in this execution framework.
- Load balancer, DNS, firewall, and certificate rows are evidence categories, not confirmed deployed infrastructure, unless backed by runtime inventory in a later artifact.
