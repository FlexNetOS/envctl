# API Catalog

Task: `ART-110_API_CATALOG`
Generated at: `2026-07-04T23:31:52+00:00`
Target: `flexnetos-vs-lifeos`
Target root: `/home/flexnetos/FlexNetOS`

## Summary

| Surface | Count |
|---|---:|
| Endpoints | 160 |
| Events | 160 |
| Schemas | 14 |
| Auth mechanisms | 9 |
| Consumers | 6 |

## Endpoints

| Service | Method | Path | Version | Auth | Source |
|---|---|---|---|---|---|
| beads_viewer | `HEAD` | `//` | `unversioned` | `unknown` | `src/beads_viewer/pkg/correlation/extractor_snapshot.go` |
| envctl | `GET` | `/:id/actions` | `unversioned` | `unknown` | `src/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/migration-artifacts/art-111_event_map/event-message-contract-map.json` |
| envctl | `POST` | `/api` | `unversioned` | `unknown` | `src/envctl/.handoff/loop/plan/findings/distributed-compute-weave.md` |
| envctl | `GET` | `/api/daily_papers` | `unversioned` | `unknown` | `src/envctl/.codex/plugins/cache/meta-plugins-codex/hugging-face/1.0.3/skills/papers/SKILL.md` |
| envctl | `GET` | `/api/health` | `unversioned` | `unknown` | `src/envctl/.handoff/loop/plan/graph/icm.graph.md` |
| envctl | `POST` | `/api/health/decay` | `unversioned` | `unknown` | `src/envctl/.handoff/loop/plan/graph/icm.graph.md` |
| envctl | `POST` | `/api/health/prune` | `unversioned` | `unknown` | `src/envctl/.handoff/loop/plan/graph/icm.graph.md` |
| envctl | `GET` | `/api/memoirs` | `unversioned` | `unknown` | `src/envctl/.handoff/loop/plan/graph/icm.graph.md` |
| envctl | `GET` | `/api/memories` | `unversioned` | `unknown` | `src/envctl/.handoff/loop/plan/graph/icm.graph.md` |
| envctl | `GET` | `/api/memories/search` | `unversioned` | `unknown` | `src/envctl/.handoff/loop/plan/graph/icm.graph.md, src/envctl/.handoff/loop/plan/reports/codemap-icm.md` |
| envctl | `GET` | `/api/papers` | `unversioned` | `unknown` | `src/envctl/.codex/plugins/cache/meta-plugins-codex/hugging-face/1.0.3/skills/papers/SKILL.md` |
| envctl | `POST` | `/api/papers/index` | `unversioned` | `unknown` | `src/envctl/.codex/plugins/cache/meta-plugins-codex/hugging-face/1.0.3/skills/papers/SKILL.md` |
| envctl | `GET` | `/api/papers/search` | `unversioned` | `unknown` | `src/envctl/.codex/plugins/cache/meta-plugins-codex/hugging-face/1.0.3/skills/papers/SKILL.md` |
| envctl | `POST` | `/api/papers/{paperId}/links` | `unversioned` | `unknown` | `src/envctl/.codex/plugins/cache/meta-plugins-codex/hugging-face/1.0.3/skills/papers/SKILL.md` |
| envctl | `POST` | `/api/settings/papers/claim` | `unversioned` | `unknown` | `src/envctl/.codex/plugins/cache/meta-plugins-codex/hugging-face/1.0.3/skills/papers/SKILL.md` |
| envctl | `GET` | `/api/stats` | `unversioned` | `unknown` | `src/envctl/.handoff/loop/plan/graph/icm.graph.md` |
| envctl | `GET` | `/api/topics` | `unversioned` | `unknown` | `src/envctl/.handoff/loop/plan/graph/icm.graph.md` |
| envctl | `POST` | `/api/topics/{n}/consolidate` | `unversioned` | `unknown` | `src/envctl/.handoff/loop/plan/graph/icm.graph.md` |
| envctl | `GET` | `/api/topics/{n}/health` | `unversioned` | `unknown` | `src/envctl/.handoff/loop/plan/graph/icm.graph.md` |
| envctl | `POST` | `/api/v1/audit/verify` | `v1` | `unknown` | `src/envctl/.handoff/loop/plan/reports/codemap-prompt-hub.md` |
| envctl | `PUT` | `/api/v1/budget/budget` | `v1` | `unknown` | `src/envctl/.handoff/loop/plan/reports/codemap-prompt-hub.md` |
| envctl | `POST` | `/api/v1/context/gather/smart` | `v1` | `unknown` | `src/envctl/.handoff/loop/plan/reports/codemap-prompt-hub.md` |
| envctl | `POST` | `/api/v1/cost/estimate` | `v1` | `unknown` | `src/envctl/.handoff/loop/plan/reports/codemap-prompt-hub.md` |
| envctl | `POST` | `/api/v1/custody/sign` | `v1` | `unknown` | `src/envctl/crates/secrets-engine/src/seam.rs, src/envctl/docs/adr-seed-usb-possession-factor.md` |
| envctl | `POST` | `/api/v1/gc/run` | `v1` | `unknown` | `src/envctl/.handoff/loop/plan/reports/codemap-prompt-hub.md` |
| envctl | `POST` | `/api/v1/goal/emit` | `v1` | `unknown` | `src/envctl/.handoff/loop/plan/findings/architecture-prompt-hub.md, src/envctl/.handoff/loop/plan/instances/prompt-hub/.handoff/loop/plan/findings/verdicts.md` |
| envctl | `GET` | `/api/v1/identity` | `v1` | `unknown` | `src/envctl/docs/adr-seed-usb-possession-factor.md` |
| envctl | `POST` | `/api/v1/input/process` | `v1` | `unknown` | `src/envctl/.handoff/loop/plan/graph/prompt-hub.graph.md, src/envctl/.handoff/loop/plan/reports/prompt-hub-plan.md` |
| envctl | `GET` | `/api/v1/lineage/ancestry/{version_id}` | `v1` | `unknown` | `src/envctl/.handoff/loop/plan/reports/codemap-prompt-hub.md` |
| envctl | `POST` | `/api/v1/moderation/check` | `v1` | `unknown` | `src/envctl/.handoff/loop/plan/reports/codemap-prompt-hub.md` |
| envctl | `POST` | `/api/v1/prompts` | `v1` | `unknown` | `src/envctl/.handoff/loop/plan/reports/codemap-prompt-hub.md` |
| envctl | `PATCH` | `/api/v1/prompts/{id}` | `v1` | `unknown` | `src/envctl/.handoff/loop/plan/reports/codemap-prompt-hub.md` |
| envctl | `POST` | `/api/v1/providers/register` | `v1` | `unknown` | `src/envctl/.handoff/loop/plan/reports/codemap-prompt-hub.md` |
| envctl | `POST` | `/api/v1/rollouts/advance` | `v1` | `unknown` | `src/envctl/.handoff/loop/plan/reports/codemap-prompt-hub.md` |
| envctl | `POST` | `/api/v1/satisfaction/csat` | `v1` | `unknown` | `src/envctl/.handoff/loop/plan/reports/codemap-prompt-hub.md` |
| envctl | `GET` | `/api/v1/swarm/bundle` | `v1` | `unknown` | `src/envctl/.handoff/loop/plan/graph/prompt-hub.cross-repo.md, src/envctl/.handoff/loop/plan/reports/codemap-prompt-hub.md` |
| envctl | `GET` | `/api/v1/users/:id` | `v1` | `unknown` | `src/envctl/.codex/plugins/cache/meta-plugins-codex/notion/0.1.5/skills/notion-spec-to-implementation/examples/api-feature.md` |
| envctl | `PUT` | `/api/v1/users/:id` | `v1` | `unknown` | `src/envctl/.codex/plugins/cache/meta-plugins-codex/notion/0.1.5/skills/notion-spec-to-implementation/examples/api-feature.md` |
| envctl | `POST` | `/api/v1/users/:id/avatar` | `v1` | `unknown` | `src/envctl/.codex/plugins/cache/meta-plugins-codex/notion/0.1.5/skills/notion-spec-to-implementation/examples/api-feature.md` |
| envctl | `GET` | `/api/v1/users/:id/public` | `v1` | `unknown` | `src/envctl/.codex/plugins/cache/meta-plugins-codex/notion/0.1.5/skills/notion-spec-to-implementation/examples/api-feature.md` |
| envctl | `GET` | `/api/v1/users/search` | `v1` | `unknown` | `src/envctl/.codex/plugins/cache/meta-plugins-codex/notion/0.1.5/skills/notion-spec-to-implementation/examples/api-feature.md` |
| envctl | `POST` | `/app/installations/140063898/access_tokens` | `unversioned` | `unknown` | `src/envctl/.handoff/loop/backlog.md` |
| envctl | `POST` | `/app/installations/{id}/access_tokens` | `unversioned` | `unknown` | `src/envctl/crates/secrets-engine/src/mint_github.rs, src/envctl/.handoff/loop/_done/task-0020.01_architect_plan.md` |
| envctl | `GET` | `/authorize` | `unversioned` | `unknown` | `src/envctl/.codex/plugins/cache/meta-plugins-codex/figma/2.0.12/skills/figma-generate-diagram/references/sequence.md` |
| envctl | `POST` | `/close-env` | `unversioned` | `unknown` | `src/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/migration-artifacts/art-111_event_map/event-message-contract-map.json` |
| envctl | `POST` | `/embed` | `unversioned` | `unknown` | `src/envctl/.codex/plugins/cache/meta-plugins-codex/hugging-face/1.0.3/skills/transformers.js/references/EXAMPLES.md` |
| envctl | `GET` | `/health` | `unversioned` | `unknown` | `src/envctl/.handoff/loop/plan/graph/icm.graph.md, src/envctl/.handoff/loop/plan/reports/codemap-prompt-hub.md` |
| envctl | `DELETE` | `/installation/token` | `unversioned` | `unknown` | `src/envctl/crates/secrets-engine/src/mint_github.rs, src/envctl/crates/secrets-engine/src/lib.rs` |
| envctl | `POST` | `/interview` | `unversioned` | `unknown` | `src/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/migration-artifacts/art-111_event_map/event-message-contract-map.json` |
| envctl | `POST` | `/login` | `unversioned` | `unknown` | `src/envctl/.codex/plugins/cache/meta-plugins-codex/figma/2.0.12/skills/figma-generate-diagram/references/sequence.md` |
| envctl | `GET` | `/repos/{owner}/{repo}/code-scanning/alerts` | `unversioned` | `unknown` | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/skills/triage-finding/references/github-rest-intake.md` |
| envctl | `GET` | `/repos/{owner}/{repo}/code-scanning/alerts/{alert_number}/instances` | `unversioned` | `unknown` | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/skills/triage-finding/references/github-rest-intake.md` |
| envctl | `GET` | `/repos/{owner}/{repo}/dependabot/alerts` | `unversioned` | `unknown` | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/skills/triage-finding/references/github-rest-intake.md` |
| envctl | `GET` | `/repos/{owner}/{repo}/issues/{issue_number}` | `unversioned` | `unknown` | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/skills/triage-finding/references/github-rest-intake.md` |
| envctl | `GET` | `/repos/{owner}/{repo}/security-advisories` | `unversioned` | `unknown` | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/skills/track-findings/references/github-security-advisories.md, src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/skills/triage-finding/references/github-rest-intake.md` |
| envctl | `POST` | `/repos/{owner}/{repo}/security-advisories` | `unversioned` | `unknown` | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/skills/track-findings/references/github-security-advisories.md` |
| envctl | `GET` | `/repos/{owner}/{repo}/security-advisories/{ghsa_id}` | `unversioned` | `unknown` | `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/skills/track-findings/references/github-security-advisories.md` |
| envctl | `GET` | `/resource` | `unversioned` | `unknown` | `src/envctl/.codex/plugins/cache/meta-plugins-codex/figma/2.0.12/skills/figma-generate-diagram/references/sequence.md` |
| envctl | `POST` | `/start` | `unversioned` | `unknown` | `src/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/migration-artifacts/art-111_event_map/event-message-contract-map.json` |
| envctl | `POST` | `/token` | `unversioned` | `unknown` | `src/envctl/.codex/plugins/cache/meta-plugins-codex/figma/2.0.12/skills/figma-generate-diagram/references/sequence.md` |
| envctl | `POST` | `/v1/messages` | `v1` | `unknown` | `src/envctl/crates/secretd/tests/mitm_e2e.rs, src/envctl/crates/secretd/tests/proxy_swap_e2e.rs` |
| envctl | `POST` | `/v1/presence/token` | `v1` | `unknown` | `src/envctl/crates/secretd/src/edge/authorizer.rs` |
| envctl | `POST` | `/v1/presentations/{presentationId}:batchUpdate` | `v1` | `unknown` | `src/envctl/.codex/plugins/cache/meta-plugins-codex/google-drive/0.1.7/skills/google-slides/references/reference-google-slides-mcp-discovery.md` |
| envctl | `POST` | `/v1/relay/swap` | `v1` | `unknown` | `src/envctl/crates/secretd/tests/edge_hardening_e2e.rs, src/envctl/crates/secretd/tests/edge_e2e.rs` |
| envctl | `GET` | `/v1/runs/:id` | `v1` | `unknown` | `src/envctl/.claude/skills/rust-port/references/symbol-map.md, src/envctl/.agents/skills/rust-port/references/symbol-map.md` |
| envctl-codedb-config-to-tables | `POST` | `/api` | `unversioned` | `unknown` | `src/envctl-codedb-config-to-tables/.handoff/loop/plan/findings/distributed-compute-weave.md` |
| envctl-codedb-config-to-tables | `GET` | `/api/health` | `unversioned` | `unknown` | `src/envctl-codedb-config-to-tables/.handoff/loop/plan/graph/icm.graph.md` |
| envctl-codedb-config-to-tables | `POST` | `/api/health/decay` | `unversioned` | `unknown` | `src/envctl-codedb-config-to-tables/.handoff/loop/plan/graph/icm.graph.md` |
| envctl-codedb-config-to-tables | `POST` | `/api/health/prune` | `unversioned` | `unknown` | `src/envctl-codedb-config-to-tables/.handoff/loop/plan/graph/icm.graph.md` |
| envctl-codedb-config-to-tables | `GET` | `/api/memoirs` | `unversioned` | `unknown` | `src/envctl-codedb-config-to-tables/.handoff/loop/plan/graph/icm.graph.md` |
| envctl-codedb-config-to-tables | `GET` | `/api/memories` | `unversioned` | `unknown` | `src/envctl-codedb-config-to-tables/.handoff/loop/plan/graph/icm.graph.md` |
| envctl-codedb-config-to-tables | `GET` | `/api/memories/search` | `unversioned` | `unknown` | `src/envctl-codedb-config-to-tables/.handoff/loop/plan/graph/icm.graph.md, src/envctl-codedb-config-to-tables/.handoff/loop/plan/reports/codemap-icm.md` |
| envctl-codedb-config-to-tables | `GET` | `/api/stats` | `unversioned` | `unknown` | `src/envctl-codedb-config-to-tables/.handoff/loop/plan/graph/icm.graph.md` |
| envctl-codedb-config-to-tables | `GET` | `/api/topics` | `unversioned` | `unknown` | `src/envctl-codedb-config-to-tables/.handoff/loop/plan/graph/icm.graph.md` |
| envctl-codedb-config-to-tables | `POST` | `/api/topics/{n}/consolidate` | `unversioned` | `unknown` | `src/envctl-codedb-config-to-tables/.handoff/loop/plan/graph/icm.graph.md` |
| envctl-codedb-config-to-tables | `GET` | `/api/topics/{n}/health` | `unversioned` | `unknown` | `src/envctl-codedb-config-to-tables/.handoff/loop/plan/graph/icm.graph.md` |
| envctl-codedb-config-to-tables | `POST` | `/api/v1/audit/verify` | `v1` | `unknown` | `src/envctl-codedb-config-to-tables/.handoff/loop/plan/reports/codemap-prompt-hub.md` |
| envctl-codedb-config-to-tables | `PUT` | `/api/v1/budget/budget` | `v1` | `unknown` | `src/envctl-codedb-config-to-tables/.handoff/loop/plan/reports/codemap-prompt-hub.md` |
| envctl-codedb-config-to-tables | `POST` | `/api/v1/context/gather/smart` | `v1` | `unknown` | `src/envctl-codedb-config-to-tables/.handoff/loop/plan/reports/codemap-prompt-hub.md` |
| envctl-codedb-config-to-tables | `POST` | `/api/v1/cost/estimate` | `v1` | `unknown` | `src/envctl-codedb-config-to-tables/.handoff/loop/plan/reports/codemap-prompt-hub.md` |

## Events

| Service | Event | Version | Schema | Source |
|---|---|---|---|---|
| beads_rust | `assignee_changed` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_rust/docs/porting/EXISTING_BEADS_STRUCTURE_AND_ARCHITECTURE.md` |
| beads_rust | `closed` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_rust/docs/porting/EXISTING_BEADS_STRUCTURE_AND_ARCHITECTURE.md` |
| beads_rust | `command` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_rust/tests/common/harness.rs` |
| beads_rust | `commented` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_rust/docs/porting/EXISTING_BEADS_STRUCTURE_AND_ARCHITECTURE.md` |
| beads_rust | `compacted` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_rust/docs/porting/EXISTING_BEADS_STRUCTURE_AND_ARCHITECTURE.md` |
| beads_rust | `created` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_rust/docs/porting/EXISTING_BEADS_STRUCTURE_AND_ARCHITECTURE.md` |
| beads_rust | `deleted` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_rust/docs/porting/EXISTING_BEADS_STRUCTURE_AND_ARCHITECTURE.md` |
| beads_rust | `dependency_added` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_rust/docs/porting/EXISTING_BEADS_STRUCTURE_AND_ARCHITECTURE.md` |
| beads_rust | `dependency_removed` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_rust/docs/porting/EXISTING_BEADS_STRUCTURE_AND_ARCHITECTURE.md` |
| beads_rust | `label_added` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_rust/docs/porting/EXISTING_BEADS_STRUCTURE_AND_ARCHITECTURE.md` |
| beads_rust | `label_removed` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_rust/docs/porting/EXISTING_BEADS_STRUCTURE_AND_ARCHITECTURE.md` |
| beads_rust | `priority_changed` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_rust/docs/porting/EXISTING_BEADS_STRUCTURE_AND_ARCHITECTURE.md` |
| beads_rust | `reopened` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_rust/docs/porting/EXISTING_BEADS_STRUCTURE_AND_ARCHITECTURE.md` |
| beads_rust | `restored` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_rust/docs/porting/EXISTING_BEADS_STRUCTURE_AND_ARCHITECTURE.md` |
| beads_rust | `snapshot` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_rust/tests/common/harness.rs` |
| beads_rust | `status_changed` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_rust/docs/porting/EXISTING_BEADS_STRUCTURE_AND_ARCHITECTURE.md` |
| beads_rust | `updated` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_rust/docs/porting/EXISTING_BEADS_STRUCTURE_AND_ARCHITECTURE.md` |
| beads_viewer | `claimed` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_viewer/pkg/correlation/types.go, src/beads_viewer/pkg/ui/history.go` |
| beads_viewer | `closed` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_viewer/pkg/correlation/types.go, src/beads_viewer/pkg/ui/history.go` |
| beads_viewer | `created` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_viewer/pkg/correlation/types.go, src/beads_viewer/pkg/ui/history.go` |
| beads_viewer | `end` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_viewer/pkg/export/force-graph.min.js` |
| beads_viewer | `modified` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_viewer/pkg/correlation/types.go, src/beads_viewer/docs/bead-history-feature-plan.md` |
| beads_viewer | `reopened` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_viewer/pkg/correlation/types.go, src/beads_viewer/pkg/ui/history.go` |
| beads_viewer | `start` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_viewer/pkg/export/force-graph.min.js` |
| beads_viewer | `zoom` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/beads_viewer/pkg/export/force-graph.min.js` |
| envctl | `Agent` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/engine/src/agent/mod.rs, src/envctl/crates/engine/src/agent/sync.rs` |
| envctl | `AgentAction` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/agent-env/src/driver.rs, src/envctl/crates/engine/src/agent/sync.rs` |
| envctl | `AgentDoctored` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/engine/src/agent/doctor.rs, src/envctl/crates/cli/src/main.rs` |
| envctl | `AgentEdited` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/engine/tests/agent_sync.rs, src/envctl/crates/engine/src/agent/edit.rs` |
| envctl | `AgentInitFinished` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/engine/src/agent/init.rs, src/envctl/crates/cli/src/main.rs` |
| envctl | `AgentListed` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/engine/tests/agent_sync.rs, src/envctl/crates/engine/src/agent/list.rs` |
| envctl | `AgentLockChecked` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/engine/src/agent/lock.rs, src/envctl/crates/cli/src/main.rs` |
| envctl | `AgentRunFinished` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/engine/tests/agent_sync.rs, src/envctl/crates/engine/src/agent/sync.rs` |
| envctl | `AgentRunStarted` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/engine/tests/agent_sync.rs, src/envctl/crates/engine/src/agent/init.rs` |
| envctl | `CatalogRendered` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/engine/src/command.rs, src/envctl/crates/gui/src/main.rs` |
| envctl | `Dashboard` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/engine/src/lib.rs, src/envctl/crates/engine/src/dashboard.rs` |
| envctl | `DashboardDeployed` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/engine/src/lib.rs, src/envctl/crates/gui/src/main.rs` |
| envctl | `GuardRefused` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/engine/src/error.rs, src/envctl/crates/engine/src/command.rs` |
| envctl | `Kind` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/secretd/tests/e2e.rs` |
| envctl | `Log` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/engine/tests/runner_parity.rs, src/envctl/crates/engine/src/install.rs` |
| envctl | `MigrationReported` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/engine/src/migration.rs, src/envctl/crates/cli/src/main.rs` |
| envctl | `Report` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/engine/src/lib.rs, src/envctl/crates/engine/src/detect.rs` |
| envctl | `RunEvent` | `1.0.0` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `schemas/run_event.schema.json` |
| envctl | `RunFinished` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/engine/src/executor.rs, src/envctl/crates/engine/src/addrepo.rs` |
| envctl | `RunStarted` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/engine/src/executor.rs` |
| envctl | `SecretsResult` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/engine/src/command.rs, src/envctl/crates/engine/src/secrets.rs` |
| envctl | `SelfUninstall` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/engine/src/self_uninstall.rs, src/envctl/.handoff/loop/_done/task-0019.2026-06-17.02_implementer_log.md` |
| envctl | `StepFinished` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/engine/src/executor.rs, src/envctl/crates/engine/src/addrepo.rs` |
| envctl | `StepStarted` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/engine/src/executor.rs, src/envctl/crates/engine/src/addrepo.rs` |
| envctl | `Telemetry` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/engine/src/command.rs, src/envctl/crates/gui/src/main.rs` |
| envctl | `default` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/migration-artifacts/art-111_event_map/event-message-contract-map.json` |
| envctl | `operation_started` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/verify_envctl_run_ledger.py` |
| envctl | `operation_succeeded` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/verify_envctl_run_ledger.py` |
| envctl | `out_of_order` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/verify_envctl_run_ledger.py` |
| envctl | `proof_linked` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/verify_envctl_run_ledger.py` |
| envctl | `run_completed` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/verify_envctl_run_ledger.py` |
| envctl | `run_created` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/verify_envctl_run_ledger.py` |
| envctl | `secret_read` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/secretd/tests/e2e.rs` |
| envctl | `unlock` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl/crates/secrets-store-libsql/src/tests.rs` |
| envctl-codedb-config-to-tables | `Agent` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl-codedb-config-to-tables/crates/engine/src/agent/mod.rs, src/envctl-codedb-config-to-tables/crates/engine/src/agent/sync.rs` |
| envctl-codedb-config-to-tables | `AgentAction` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl-codedb-config-to-tables/crates/agent-env/src/driver.rs, src/envctl-codedb-config-to-tables/crates/engine/src/agent/sync.rs` |
| envctl-codedb-config-to-tables | `AgentDoctored` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl-codedb-config-to-tables/crates/engine/src/agent/doctor.rs, src/envctl-codedb-config-to-tables/crates/cli/src/main.rs` |
| envctl-codedb-config-to-tables | `AgentEdited` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl-codedb-config-to-tables/crates/engine/tests/agent_sync.rs, src/envctl-codedb-config-to-tables/crates/engine/src/agent/edit.rs` |
| envctl-codedb-config-to-tables | `AgentInitFinished` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl-codedb-config-to-tables/crates/engine/src/agent/init.rs, src/envctl-codedb-config-to-tables/crates/cli/src/main.rs` |
| envctl-codedb-config-to-tables | `AgentListed` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl-codedb-config-to-tables/crates/engine/tests/agent_sync.rs, src/envctl-codedb-config-to-tables/crates/engine/src/agent/list.rs` |
| envctl-codedb-config-to-tables | `AgentLockChecked` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl-codedb-config-to-tables/crates/engine/src/agent/lock.rs, src/envctl-codedb-config-to-tables/crates/cli/src/main.rs` |
| envctl-codedb-config-to-tables | `AgentRunFinished` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl-codedb-config-to-tables/crates/engine/tests/agent_sync.rs, src/envctl-codedb-config-to-tables/crates/engine/src/agent/sync.rs` |
| envctl-codedb-config-to-tables | `AgentRunStarted` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl-codedb-config-to-tables/crates/engine/tests/agent_sync.rs, src/envctl-codedb-config-to-tables/crates/engine/src/agent/init.rs` |
| envctl-codedb-config-to-tables | `Dashboard` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl-codedb-config-to-tables/crates/engine/src/lib.rs, src/envctl-codedb-config-to-tables/crates/engine/src/dashboard.rs` |
| envctl-codedb-config-to-tables | `DashboardDeployed` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl-codedb-config-to-tables/crates/engine/src/lib.rs, src/envctl-codedb-config-to-tables/crates/gui/src/main.rs` |
| envctl-codedb-config-to-tables | `GuardRefused` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl-codedb-config-to-tables/crates/engine/src/error.rs, src/envctl-codedb-config-to-tables/crates/engine/src/command.rs` |
| envctl-codedb-config-to-tables | `Kind` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl-codedb-config-to-tables/crates/secretd/tests/e2e.rs` |
| envctl-codedb-config-to-tables | `Log` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl-codedb-config-to-tables/crates/engine/tests/runner_parity.rs, src/envctl-codedb-config-to-tables/crates/engine/src/install.rs` |
| envctl-codedb-config-to-tables | `MigrationReported` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl-codedb-config-to-tables/crates/engine/src/migration.rs, src/envctl-codedb-config-to-tables/crates/cli/src/main.rs` |
| envctl-codedb-config-to-tables | `Report` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl-codedb-config-to-tables/crates/engine/src/lib.rs, src/envctl-codedb-config-to-tables/crates/engine/src/detect.rs` |
| envctl-codedb-config-to-tables | `RunFinished` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl-codedb-config-to-tables/crates/engine/src/executor.rs, src/envctl-codedb-config-to-tables/crates/engine/src/addrepo.rs` |
| envctl-codedb-config-to-tables | `RunStarted` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl-codedb-config-to-tables/crates/engine/src/executor.rs` |
| envctl-codedb-config-to-tables | `SecretsResult` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl-codedb-config-to-tables/crates/engine/src/command.rs, src/envctl-codedb-config-to-tables/crates/engine/src/secrets.rs` |
| envctl-codedb-config-to-tables | `SelfUninstall` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl-codedb-config-to-tables/crates/engine/src/self_uninstall.rs, src/envctl-codedb-config-to-tables/.handoff/loop/_done/task-0019.2026-06-17.02_implementer_log.md` |
| envctl-codedb-config-to-tables | `StepFinished` | `unversioned` | `schemas/shared_protocol.schema.json#/$defs/RunEvent` | `src/envctl-codedb-config-to-tables/crates/engine/src/executor.rs, src/envctl-codedb-config-to-tables/crates/engine/src/addrepo.rs` |

## Auth

| Name | Mechanism | Redaction | Evidence |
|---|---|---|---|
| `api-key` | `api-key` | pattern-only; secret values are excluded | src/teri/tests/agent_ltm_recall.rs, src/teri/src/config.rs |
| `apikey` | `apikey` | pattern-only; secret values are excluded | src/meta-ruvector/examples/neural-trader/system/data-connectors.js, src/meta-ruvector/examples/scipix/docs/14_SECURITY.md |
| `approval` | `approval` | pattern-only; secret values are excluded | src/teri/docs/AGENTIC-STORY.md, src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/skills/track-findings/SKILL.md |
| `auth` | `auth` | pattern-only; secret values are excluded | WORKLOG.md, .kb/config.toml |
| `basic` | `basic` | pattern-only; secret values are excluded | src/teri/.pre-commit-config.yaml, src/teri/src/services/simulation_runner.rs |
| `bearer` | `bearer` | pattern-only; secret values are excluded | src/teri/src/seed/community/pebesen.rs, src/teri/pebesen/crates/api/src/middleware.rs |
| `jwt` | `jwt` | pattern-only; secret values are excluded | src/teri/pebesen/DEVELOPMENT.md, src/teri/pebesen/crates/api/src/auth.rs |
| `oauth` | `oauth` | pattern-only; secret values are excluded | src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/skills/security-scan/references/repo-wide-artifacts-and-ledger.md, src/envctl/.codex/plugins/cache/meta-plugins-codex/google-drive/0.1.7/skills/google-slides/references/reference-google-slides-mcp-discovery.md |
| `token` | `token` | pattern-only; secret values are excluded | .kb/skills/kb-start/SKILL.md, .kb/skills/kb-context/SKILL.md |

## Evidence

- `execution-framework/generated/execution_packets/ART-110_API_CATALOG.json`
- `execution-framework/generated/envctl_target_registry.json`
- `execution-framework/generated/package_scan.json`
- `execution-framework/generated/shared_protocol_manifest.json`
- `execution-framework/generated/contract_manifest.json`
- `execution-framework/proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json`
- `execution-framework/proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json`
