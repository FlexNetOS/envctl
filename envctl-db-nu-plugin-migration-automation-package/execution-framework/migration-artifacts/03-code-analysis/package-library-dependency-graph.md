# ART-103 Package/library dependency graph

Generated: 2026-07-04T23:19:59+00:00
Target root: `/home/flexnetos/FlexNetOS`

## Scope

This graph models the FlexNetOS migration workspace as a control-plane service graph. The target root is a hollow workspace containing independent peer repositories, so the useful service boundaries are environment authority, control plane, plugin/operator interface, runner, artifact registry, and migration artifact sinks.

## Summary

- Nodes: 16
- Service edges: 21
- Task dependency edges: 143
- Migration waves: 7
- Service definitions found in bounded scan: 46
- Manifest/config files found in bounded scan: 10314

## Service Call And Ownership Edges

| From | Relationship | To | Confidence | Evidence |
|---|---|---|---|---|
| `workspace:flexnetos` | `contains_peer` | `service:meta-control-plane` | high | AGENTS.md, WORKSPACE_LAYOUT.md |
| `workspace:flexnetos` | `contains_peer` | `service:envctl` | high | AGENTS.md, WORKSPACE_LAYOUT.md |
| `workspace:flexnetos` | `contains_peer` | `service:flexnetos-runner` | high | AGENTS.md, WORKSPACE_LAYOUT.md |
| `workspace:flexnetos` | `contains_peer` | `service:yazelix-runtime` | high | AGENTS.md |
| `service:meta-control-plane` | `uses_memory_graph` | `service:gitkb-memory` | medium | AGENTS.md |
| `service:envctl` | `owns_state_tables` | `service:envctl-state` | high | WORKSPACE_LAYOUT.md |
| `service:envctl` | `owns_database_contract` | `service:envctl-db` | high | generated/envctl_migration_db_model.json |
| `service:envctl-db` | `provides_tables_for` | `service:artifact-registry` | high | scripts/artifact_registry.py, generated/envctl_artifact_registry_report.json |
| `service:artifact-registry` | `registers_hashes_for` | `service:artifact-set` | high | generated/envctl_artifact_registry_report.json |
| `service:shared-protocol` | `defines_records_for` | `service:envctl-db` | high | generated/shared_protocol_validation_report.json |
| `service:nu-plugin-control` | `implements_protocol` | `service:shared-protocol` | high | generated/nu_plugin_command_manifest.json, generated/shared_protocol_manifest.json |
| `service:nu-plugin-control` | `calls_control_operations` | `service:envctl-db` | medium | generated/task_graph.normalized.json |
| `service:execution-framework` | `registers_outputs_with` | `service:artifact-registry` | high | generated/execution_packets/ART-103_SERVICE_DEP_GRAPH.json |
| `service:execution-framework` | `produces` | `service:artifact-set` | high | generated/contract_manifest.json |
| `service:flexnetos-adapter` | `reuses_packets_and_replay` | `service:execution-framework` | high | generated/task_graph.normalized.json |
| `service:yazelix-runtime` | `consumes_environment_contracts` | `service:envctl` | medium | AGENTS.md, LOCAL_WORKAROUNDS.md |
| `service:rtk` | `supports_command_capture` | `service:execution-framework` | medium | /home/flexnetos/.codex/RTK.md, /home/flexnetos/.codex/AGENTS.rtk.md |
| `service:upstream-evidence` | `feeds_scan_evidence` | `service:execution-framework` | medium | src/release-workspace.meta.yaml |
| `task:REQ-024_ENVCTL_ARTIFACT_REGISTRY` | `migration_wave_dependency` | `task:ART-103_SERVICE_DEP_GRAPH` | high | generated/task_graph.normalized.json |
| `task:REQ-040_SHARED_PROTOCOL_SCHEMAS` | `migration_wave_dependency` | `task:ART-103_SERVICE_DEP_GRAPH` | high | generated/task_graph.normalized.json |
| `task:ART-103_SERVICE_DEP_GRAPH` | `blocks_validation` | `task:VER-300_UNIT_VALIDATION` | high | generated/task_graph.normalized.json |

## Migration Wave Dependencies

| Wave | Name | Task Count | Representative Tasks |
|---:|---|---:|---|
| 0 | Contract lock | 1 | REQ-010_CONTRACT_LOCK |
| 1 | Shared contracts and protocol surfaces | 3 | REQ-020_ENVCTL_DB_SCHEMA, REQ-030_PLUGIN_PROTOCOL_MANIFEST, REQ-040_SHARED_PROTOCOL_SCHEMAS |
| 2 | Envctl registries and operation state | 6 | REQ-021_ENVCTL_TARGET_REGISTRY, REQ-022_ENVCTL_RUN_LEDGER, REQ-023_ENVCTL_OPERATION_STATE, REQ-024_ENVCTL_ARTIFACT_REGISTRY, REQ-025_ENVCTL_VALIDATION_EVIDENCE, REQ-026_ENVCTL_ROLLBACK_CHECKPOINTS |
| 3 | Control APIs, replay, and plugin UX | 7 | REQ-027_ENVCTL_REPLAY_ENGINE, REQ-028_ENVCTL_AGENT_CONTROL_API, REQ-031_PLUGIN_COMMAND_SURFACE, REQ-032_PLUGIN_LIVE_VISUALS, REQ-033_PLUGIN_HUMAN_APPROVAL, REQ-034_PLUGIN_STATUS_STREAMS, REQ-041_TWO_REPO_INTEGRATION |
| 4 | Artifact generation | 37 | ART-100_SYSTEM_INVENTORY, ART-101_DIRECTORY_TREE, ART-102_REPOSITORY_MAP, ART-103_SERVICE_DEP_GRAPH, ART-104_TOOLCHAIN_TREE, ART-105_PACKAGE_LIB_GRAPH, ART-106_RUNTIME_DEP_MAP, ART-107_DATA_FLOW_GRAPH ... |
| 5 | FlexNetOS target adapter | 3 | REQ-200_FLEXNETOS_TARGET_DESCRIPTOR, REQ-201_FLEXNETOS_LIFEOS_COMPARISON, REQ-202_FLEXNETOS_ADAPTER_RECIPE |
| 6 | Validation and release | 7 | VER-300_UNIT_VALIDATION, VER-301_SQL_SCHEMA_TEST, VER-302_PACKET_SCHEMA_VALIDATION, VER-303_GOAL_LOOP_COMPUTE, VER-304_FINAL_COMPLETENESS, REL-400_PACKAGE_ARCHIVE, REL-401_HANDOFF |

## Contract Paths Covered

| Artifact ID | Required Path | Title |
|---|---|---|
| `01-current-state-dependency-graph-md` | `migration-artifacts/01-current-state/dependency-graph.md` | Dependency Graph |
| `01-current-state-application-service-dependency-graph-md` | `migration-artifacts/01-current-state/application-service-dependency-graph.md` | Application Service Dependency Graph |
| `03-code-analysis-import-dependency-graph-md` | `migration-artifacts/03-code-analysis/import-dependency-graph.md` | Import Dependency Graph |
| `03-code-analysis-package-library-dependency-graph-md` | `migration-artifacts/03-code-analysis/package-library-dependency-graph.md` | Package Library Dependency Graph |

## Evidence Notes

- `AGENTS.md` and `WORKSPACE_LAYOUT.md` establish the hollow workspace and peer repository boundaries.
- `generated/task_graph.normalized.json` supplies ART-103 dependencies on `REQ-024_ENVCTL_ARTIFACT_REGISTRY` and `REQ-040_SHARED_PROTOCOL_SCHEMAS`, plus its validation block on `VER-300_UNIT_VALIDATION`.
- `generated/envctl_artifact_registry_report.json` proves the artifact registry model can persist paths, hashes, producers, validation links, and graph edges.
- Secret/key path patterns were excluded from the scan: `**/.env, **/secrets/**, **/private_keys/**, **/*.pem, **/*.key`.

## Limits

No live process tracing, endpoint probing, or credentialed service calls were performed. Edges marked `medium` are inferred from repository contracts and generated task metadata rather than direct runtime traffic.
