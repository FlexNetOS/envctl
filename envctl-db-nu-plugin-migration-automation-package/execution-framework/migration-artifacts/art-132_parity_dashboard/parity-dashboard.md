# Parity dashboard

Task: `ART-132_PARITY_DASHBOARD`
Generated at: `2026-07-04T23:29:06+00:00`
Status: `complete_with_live_capture_gap`

## Scope

This dashboard defines the real-time old/new comparison surface for the FlexNetOS versus lifeos migration target. It is grounded in envctl database rows, artifact registry hashes, validation evidence, shared protocol records, nu_plugin read surfaces, and the shadow traffic comparison report.

## Summary

- Metric tiles: `5`
- Comparison streams: `4`
- Shared protocol required records: `14`
- Artifact registry controls passed: `7/7`
- Current parity status: `warn`
- Shadow traffic status: `complete_with_capture_gap`

## Metric Tiles

| metric | old source | new source | current value | status |
|---|---|---|---|---|
| Artifact registry coverage | filesystem artifact paths | envctl_migration_artifacts plus envctl_migration_artifact_index | `"7/7 registry controls passed"` | `pass` |
| Validation evidence | manual reconciliation notes | envctl_migration_validations and evidence rows | `{"blocked": 0, "fail": 0, "pass": 2, "unknown": 0, "warn": 1}` | `warn` |
| Shared protocol coverage | direct file and log reads | envctl to nu_plugin shared records | `"14 required record contracts"` | `pass` |
| Live status projection | background logs and proof files | envctl migration visuals and status stream projections | `"dashboard_markdown"` | `pass` |
| Task completion state | packet queue | proof status report and live visuals | `{"blocked_count": 56, "complete_count": 16, "failed_count": 0, "lane_count": 7, "parallel_group_count": 16, "proof_missing_count": 68, "proof_present_count": 12, "ready_count": 8, "source_status_report": {"approval_blocker_count": 2, "dispatch_count": 7, "generated_at": "2026-07-04T23:09:04+00:00", "runnable_count": 8}, "task_count": 80}` | `warn` |

## Comparison Streams

| stream | refresh source | old metric | new metric | threshold | status |
|---|---|---|---|---|---|
| artifact-publication | `envctl_migration_artifact_index` | artifact file exists | artifact row path and content_hash match disk | all published dashboard files have sha256 content_hash | `pass` |
| validation-parity | `envctl_migration_validation_scorecard` | manual parity summary | pass, warn, fail, blocked, unknown validation counts | fail == 0 and blocked == 0; warn allowed only for missing live captures | `warn` |
| protocol-shape | `shared_protocol_manifest` | implicit file shape | record contract count and sample validation status | all required records validated | `pass` |
| status-stream | `envctl_status_stream` | log tail and proof directory scan | event rows joined to proof task status | cursorable read-only projection returns event and proof views | `pass` |

## Refresh Contract

- Mode: `read-only polling or status stream subscription`
- Minimum refresh seconds: `5`
- Mutation policy: `dashboard is read-only; envctl remains source of truth`
- Record contracts: `ArtifactRecord, EvidenceRecord, ValidationResult, RunEvent, ProofRecord`
- nu_plugin surfaces: `envctl migration visuals, codedb envctl status stream, envctl migration proof, envctl migration status`

## Gates

| gate | status | rule |
|---|---|---|
| `hash_recorded` | `pass_after_registry_write` | All dashboard artifacts must have envctl registry content_hash values. |
| `validation_evidence_linked` | `pass_after_registry_write` | Dashboard artifacts must link artifact registry, validation, protocol, and status evidence. |
| `no_unredacted_secret_paths` | `pass` | Dashboard evidence excludes blocked path patterns from the execution packet. |
| `live_capture_payloads` | `warn` | No mirrored production old/new payload stream is present; dashboard is ready to project envctl rows once captures arrive. |

## Contract Mapping

- Contract row: `artifact:06-testing-validation-parity-dashboard-md`
- Canonical path: `migration-artifacts/06-testing-validation/parity-dashboard.md`
- Task Markdown: `migration-artifacts/art-132_parity_dashboard/parity-dashboard.md`
- Task JSON: `migration-artifacts/art-132_parity_dashboard/parity-dashboard.json`

## Evidence Inputs

| input | path |
|---|---|
| target_descriptor | `examples/target-descriptors/flexnetos-vs-lifeos.yaml` |
| repo_scan | `execution-framework/generated/package_scan.json` |
| envctl_database | `execution-framework/generated/envctl_migration_db_model.json` |
| artifact_registry | `execution-framework/generated/envctl_artifact_registry_report.json` |
| validation_evidence | `execution-framework/generated/envctl_validation_evidence_report.json` |
| shared_protocol | `execution-framework/generated/shared_protocol_manifest.json` |
| nu_plugin_command_manifest | `execution-framework/generated/nu_plugin_command_manifest.json` |
| live_visuals | `execution-framework/generated/live_visuals.json` |
| shadow_traffic | `execution-framework/migration-artifacts/art-130_shadow_traffic/shadow-traffic-comparison-report.json` |
