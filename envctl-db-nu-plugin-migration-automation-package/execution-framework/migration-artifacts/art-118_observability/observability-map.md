# ART-118 Observability Map

Generated: `2026-07-28T11:40:01+00:00`

This map covers logs, metrics, traces, dashboards, alerts, SLOs, and runbooks from generated envctl reports plus a safe scan of the target filesystem. It records evidence categories and control-plane observability; it does not claim deployed external observability services unless the evidence is present.

## Target

- Target: `flexnetos-vs-lifeos`
- Primary root: `/home/flexnetos/FlexNetOS`
- Compare root: `/home/flexnetos/lifeos`
- Target registry status: `passed`
- Safe scan visited files: `9980`
- Safe scan content-checked files: `4239`

## Coverage

| Category | Status | Evidence count | Sample evidence |
| --- | --- | --- | --- |
| logs | repo_evidence_found | 80 | `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` (content_keyword:log:20)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/README.md` (content_keyword:log:6)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/AGENTS.md` (content_keyword:log:10)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/tests/fixtures/nushell/env-config.nu` (content_keyword:log:11)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/Cargo.toml` (content_keyword:log:14) |
| metrics | repo_evidence_found | 80 | `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/src/vault/crypto.rs` (content_keyword:counter:8)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/agent-env/src/driver.rs` (content_keyword:counter:6)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/agent-env/src/report.rs` (content_keyword:counter:3)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/tests/ruvector_codec_compat.rs` (content_keyword:metric:13)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/semantic.rs` (content_keyword:metric:14) |
| traces | repo_evidence_found | 80 | `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` (content_keyword:trace:14)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/src/ca.rs` (content_keyword:span:494)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/db_query.rs` (content_keyword:trace:52)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/wiring.rs` (content_keyword:span:305)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secretd/src/edge/listener.rs` (content_keyword:tracing:20) |
| dashboards | repo_evidence_found | 80 | `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/Cargo.toml` (content_keyword:dashboard:25)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/tests/golden/mission-control.kdl` (content_keyword:dashboard:1)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/command.rs` (content_keyword:dashboard:8)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/model.rs` (content_keyword:dashboard:65)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/dashboard.rs` (path_signal:dashboard) |
| alerts | repo_evidence_found | 80 | `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/db_watch.rs` (content_keyword:notify:14)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-store-libsql/Cargo.toml` (content_keyword:alert:18)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.codex/prompts/goal.md` (content_keyword:incident:48)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.codex/prompts/prompt:substrate-init.inherit.md` (content_keyword:notify:23)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/ci/gates/enable.sh` (content_keyword:notify:79) |
| slos | repo_evidence_found | 80 | `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` (content_keyword:sla:22)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/tests/relay.rs` (content_keyword:slo:12)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/tests/phase0.rs` (content_keyword:slo:4)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/src/mint_github.rs` (content_keyword:slo:39)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/src/error.rs` (content_keyword:slo:13) |
| runbooks | repo_evidence_found | 80 | `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/src/sqld_auth.rs` (content_keyword:bootstrap:4)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/src/seam.rs` (content_keyword:rollback:5)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/src/broker/authorizer.rs` (content_keyword:replay:15)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/src/broker/decide.rs` (content_keyword:rollback:34)<br>`src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/src/broker/mod.rs` (content_keyword:replay:24) |

## Control-Plane Observability

| Control | Type | Coverage | Evidence |
| --- | --- | --- | --- |
| task-logs | logs | present | `generated/task_graph.csv`<br>`generated/execution_packets/ART-118_OBSERVABILITY.json`<br>`logs/` |
| heartbeat-state | metrics | present | `state/`<br>`docs/GOAL_LOOP_PROTOCOL.md` |
| proof-ledger | audit_trace | present | `proof_records/proof_ledger.jsonl`<br>`schemas/proof_record.schema.json` |
| live-visuals | dashboard | present | `generated/live_visuals.json`<br>`generated/live_visuals.md`<br>`docs/SHARED_PROTOCOL_SCHEMAS.md` |
| validation-scorecard | slo | modeled | `generated/envctl_validation_evidence_report.json`<br>`migration-artifacts/art-128_readiness_scorecard/readiness-scorecard.json`<br>`generated/envctl_artifact_registry_report.json` |
| operator-runbook-surface | runbook | present | `docs/INSTALL_BOOTSTRAP.md`<br>`examples/nu/operator-session-template.nu`<br>`docs/OPERATION_STATE_MACHINE.md`<br>`docs/ENVCTL_RUN_LEDGER.md` |
| alerting-status-streams | alert | partial | `docs/SHARED_PROTOCOL_SCHEMAS.md`<br>`generated/REQ-034_PLUGIN_STATUS_STREAMS.contract.json` |

## Signal Flow

| From | Signal | To |
| --- | --- | --- |
| task execution | logs_uri and task log file | logs |
| task execution | state heartbeat JSON | heartbeat |
| artifact registry | content hash and validation rows | validation ledger |
| plugin status streams | status table/live visual record | operator dashboard |
| proof ledger | proof status and next_action | runbook gate |

## Gaps

No empty observability categories in the safe scan.

## Evidence Boundary

- Secret-like paths are excluded by policy: `**/.env`, `**/secrets/**`, `**/private_keys/**`, `**/*.pem`, `**/*.key`.
- Source evidence records path, evidence kind, keyword, and line number only; source line content is not copied.
- External paging, APM, log aggregation, and SLO systems are marked as gaps unless represented in repo or generated envctl evidence.
