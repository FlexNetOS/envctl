# ART-118 Observability Map

Generated: `2026-07-04T23:29:39+00:00`

This map covers logs, metrics, traces, dashboards, alerts, SLOs, and runbooks from generated envctl reports plus a safe scan of the target filesystem. It records evidence categories and control-plane observability; it does not claim deployed external observability services unless the evidence is present.

## Target

- Target: `flexnetos-vs-lifeos`
- Primary root: `/home/flexnetos/FlexNetOS`
- Compare root: `/home/flexnetos/lifeos`
- Target registry status: `passed`
- Safe scan visited files: `458127`
- Safe scan content-checked files: `4162`

## Coverage

| Category | Status | Evidence count | Sample evidence |
| --- | --- | --- | --- |
| logs | repo_evidence_found | 80 | `WORKSPACE_LAYOUT.md` (content_keyword:log:43)<br>`WORKLOG.md` (path_signal:log)<br>`WORKLOG.md` (content_keyword:log:1)<br>`AGENTS.md` (content_keyword:log:26)<br>`src/OWNERSHIP.md` (content_keyword:log:25) |
| metrics | repo_evidence_found | 80 | `src/teri/src/memory/mod.rs` (content_keyword:counter:19)<br>`src/teri/src/services/simulation_manager.rs` (content_keyword:counter:112)<br>`src/teri/src/services/simulation_runner.rs` (content_keyword:counter:99)<br>`src/teri/.codex/prompts/teri-simulation-truth.md` (content_keyword:counter:2)<br>`src/teri/.worktrees/issue-86-source-wires/src/memory/mod.rs` (content_keyword:counter:19) |
| traces | repo_evidence_found | 80 | `LOCAL_WORKAROUNDS.md` (content_keyword:trace:106)<br>`src/teri/Cargo.toml` (content_keyword:trace:32)<br>`src/teri/tests/community_pipeline_e2e.rs` (content_keyword:trace:223)<br>`src/teri/src/preflight.rs` (content_keyword:tracing:190)<br>`src/teri/src/api/simulation.rs` (content_keyword:trace:55) |
| dashboards | repo_evidence_found | 80 | `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/brain-in-the-fish/crates/core/src/visualize.rs` (content_keyword:visualization:1)<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/CLAUDE.md` (content_keyword:visualization:50)<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/tests/test_visualization_sanitization.py` (path_signal:visualization)<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/tests/test_visualization_sanitization.py` (content_keyword:visualization:2)<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/splitrail/vscode-splitrail/package.json` (content_keyword:dashboard:4) |
| alerts | repo_evidence_found | 80 | `src/teri/src/api/mod.rs` (content_keyword:notification:88)<br>`src/teri/src/services/entity_reader.rs` (content_keyword:incident:593)<br>`src/teri/.worktrees/issue-86-source-wires/src/api/mod.rs` (content_keyword:notification:88)<br>`src/teri/.worktrees/issue-86-source-wires/src/services/entity_reader.rs` (content_keyword:incident:593)<br>`src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/brain-in-the-fish/data/asap-stratified-100-extractions.json` (content_keyword:alarm:1) |
| slos | repo_evidence_found | 80 | `src/teri/tests/community_loop_e2e.rs` (content_keyword:slo:86)<br>`src/teri/src/seed/community/pebesen.rs` (content_keyword:sla:118)<br>`src/teri/src/api/streaming.rs` (content_keyword:slo:6)<br>`src/teri/src/i18n/mod.rs` (content_keyword:sla:9)<br>`src/teri/src/i18n/locales/en.json` (content_keyword:slo:44) |
| runbooks | repo_evidence_found | 80 | `src/release-workspace.meta.yaml` (content_keyword:bootstrap:4)<br>`src/teri/SPRINT.md` (content_keyword:runbook:9)<br>`src/teri/FEATURE-PARITY.md` (content_keyword:runbook:8)<br>`src/teri/rust-toolchain.toml` (content_keyword:runbook:17)<br>`src/teri/RUNBOOK.md` (path_signal:runbook) |

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
