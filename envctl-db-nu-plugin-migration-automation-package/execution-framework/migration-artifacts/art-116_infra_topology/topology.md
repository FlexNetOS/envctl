# ART-116 Infrastructure Topology Map

Generated: `2026-07-04T23:20:52+00:00`

This map is built from the target descriptor, generated repo/package scan, envctl database reports, and a safe filename-only scan of the target filesystem. It records repo evidence and database control-plane topology; it does not claim live cloud/runtime inventory unless that evidence is present.

## Target

- Target: `flexnetos-vs-lifeos`
- Primary root: `/home/flexnetos/FlexNetOS`
- Compare root: `/home/flexnetos/lifeos`
- Target registry status: `passed`
- Safe scan visited files: `469467`

## Coverage

| Category | Status | Evidence count | Sample evidence |
| --- | --- | --- | --- |
| compute | repo_evidence_found | 40 | `src/Cargo.toml`<br>`src/teri/Cargo.toml`<br>`src/teri/src/services/zep_tools.rs`<br>`src/teri/src/services/graph_backend.rs`<br>`src/teri/src/services/graph_memory.rs` |
| networking | repo_evidence_found | 40 | `src/teri/src/report/mod.rs`<br>`src/teri/src/report/sink.rs`<br>`src/teri/src/report/manager.rs`<br>`src/teri/src/report/console_logger.rs`<br>`src/teri/src/report/logger.rs` |
| storage | repo_evidence_found | 40 | `COMMAND_LEDGER.csv`<br>`src/teri/code-research/references/research-ledger.md`<br>`src/teri/uploads/simulations/sim_2d08edb915b4/state.json`<br>`src/teri/uploads/simulations/sim_e01869db64aa/state.json`<br>`src/teri/frontend/src/components/HistoryDatabase.vue` |
| dns | repo_evidence_found | 40 | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/brain-in-the-fish/data/cross-domain-clinical.json`<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/brain-in-the-fish/data/cross-domain-policy.json`<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/brain-in-the-fish/data/cross-domain-tender.json`<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/sldbench/configs/domain_mixture_scaling_law.yaml`<br>`src/meta-ruvector/examples/data/discoveries/cross_domain_correlations.json` |
| load_balancers | repo_evidence_found | 40 | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/cluaiz/Inference-engine/engines/src/neural_foundry/runtime/mcp_gateway.rs`<br>`src/meta-ruvector/examples/vwm-viewer/nginx.conf`<br>`src/meta-ruvector/examples/edge-net/dashboard/nginx.conf`<br>`src/meta-ruvector/.codex/mirror/.claude/agents/optimization/load-balancer.md`<br>`src/meta-ruvector/.codex/agents/claude/claude-optimization-load-balancer.toml` |
| firewalls | repo_evidence_found | 40 | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/brain-in-the-fish/case-studies/02-prompt-firewall.md`<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/brain-in-the-fish/crates/core/src/firewall.rs`<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/cluaiz/Inference-engine/engines/src/runtime/execution/sandbox.rs`<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/cluaiz/Inference-engine/engines/src/neural_foundry/executor/sandbox.rs`<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/cluaiz/inference-cel/src/execution/wasm_sandbox.rs` |
| certificates | repo_evidence_found | 40 | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/alphaevolve_math_problems/uncertainty_ineq/config.yaml`<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/alphaevolve_math_problems/uncertainty_ineq/initial_program.py`<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/alphaevolve_math_problems/uncertainty_ineq/requirements.txt`<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/alphaevolve_math_problems/uncertainty_ineq/evaluator.py`<br>`src/envctl/crates/secretd/src/edge/tls.rs` |

## Topology Nodes

| Node | Kind | Evidence |
| --- | --- | --- |
| FlexNetOS versus lifeos target | migration_target | `generated/envctl_target_registry.json`<br>`../examples/target-descriptors/flexnetos-vs-lifeos.yaml` |
| Workspace source repositories and local build/runtime surfaces | compute | `src/Cargo.toml`<br>`src/teri/Cargo.toml`<br>`src/teri/src/services/zep_tools.rs`<br>`src/teri/src/services/graph_backend.rs`<br>`src/teri/src/services/graph_memory.rs` |
| Local service, socket, relay, and container networking evidence | networking | `src/teri/src/report/mod.rs`<br>`src/teri/src/report/sink.rs`<br>`src/teri/src/report/manager.rs`<br>`src/teri/src/report/console_logger.rs`<br>`src/teri/src/report/logger.rs` |
| envctl migration database and repository storage surfaces | storage | `generated/envctl_migration_db_model.json`<br>`COMMAND_LEDGER.csv`<br>`src/teri/code-research/references/research-ledger.md`<br>`src/teri/uploads/simulations/sim_2d08edb915b4/state.json`<br>`src/teri/uploads/simulations/sim_e01869db64aa/state.json` |
| DNS/domain configuration evidence in target filesystem | dns | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/brain-in-the-fish/data/cross-domain-clinical.json`<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/brain-in-the-fish/data/cross-domain-policy.json`<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/brain-in-the-fish/data/cross-domain-tender.json`<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/sldbench/configs/domain_mixture_scaling_law.yaml`<br>`src/meta-ruvector/examples/data/discoveries/cross_domain_correlations.json` |
| Reverse proxy/load-balancer evidence in target filesystem | load_balancer | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/cluaiz/Inference-engine/engines/src/neural_foundry/runtime/mcp_gateway.rs`<br>`src/meta-ruvector/examples/vwm-viewer/nginx.conf`<br>`src/meta-ruvector/examples/edge-net/dashboard/nginx.conf`<br>`src/meta-ruvector/.codex/mirror/.claude/agents/optimization/load-balancer.md`<br>`src/meta-ruvector/.codex/agents/claude/claude-optimization-load-balancer.toml` |
| Network policy, sandbox, and firewall-adjacent controls | firewall | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/brain-in-the-fish/case-studies/02-prompt-firewall.md`<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/brain-in-the-fish/crates/core/src/firewall.rs`<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/cluaiz/Inference-engine/engines/src/runtime/execution/sandbox.rs`<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/cluaiz/Inference-engine/engines/src/neural_foundry/executor/sandbox.rs`<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/cluaiz/inference-cel/src/execution/wasm_sandbox.rs` |
| TLS/certificate dependency and configuration evidence | certificates | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/alphaevolve_math_problems/uncertainty_ineq/config.yaml`<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/alphaevolve_math_problems/uncertainty_ineq/initial_program.py`<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/alphaevolve_math_problems/uncertainty_ineq/requirements.txt`<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/alphaevolve_math_problems/uncertainty_ineq/evaluator.py`<br>`src/envctl/crates/secretd/src/edge/tls.rs` |
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

No empty infrastructure categories in the safe scan.

## Evidence Boundary

- Secret-like paths are excluded by policy: `**/.env`, `**/secrets/**`, `**/private_keys/**`, `**/*.pem`, `**/*.key`.
- The envctl migration database is represented from generated schema/report artifacts and is exercised as SQLite in-memory in this execution framework.
- Load balancer, DNS, firewall, and certificate rows are evidence categories, not confirmed deployed infrastructure, unless backed by runtime inventory in a later artifact.
