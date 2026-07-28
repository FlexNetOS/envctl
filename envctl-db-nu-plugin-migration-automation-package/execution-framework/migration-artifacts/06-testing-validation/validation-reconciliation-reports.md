# Validation reconciliation reports

Generated at: `2026-07-28T11:49:34+00:00`
Status: `passed`

## Parity

| Check | Count |
|---|---:|
| Task graph rows | 80 |
| Execution packets | 80 |
| Status report tasks | 80 |
| Proof records | 88 |
| Successful tasks | 80 |
| Missing packets | 0 |
| Successful tasks without proof | 0 |

## Counts

| Area | Metric | Count |
|---|---|---:|
| Artifact registry | evidence rows | 3 |
| Artifact registry | graph edges | 4 |
| Artifact registry | validation rows | 2 |
| Validation evidence | validation rows | 3 |
| Validation evidence | evidence rows | 4 |
| Validation evidence | hashed evidence rows | 4 |
| Shared protocols | samples passed | 14 / 14 |
| Final verification | missing outputs | 0 |
| Final verification | unresolved gaps | 0 |

## Phase Status Counts

| Phase | Tasks | Complete or passed | Pending | Other |
|---|---:|---:|---:|---:|
| 00-framework | 8 | 8 | 0 | 0 |
| 01-contract | 1 | 1 | 0 | 0 |
| 02-envctl-db | 9 | 9 | 0 | 0 |
| 03-nu-plugin | 5 | 5 | 0 | 0 |
| 04-shared | 6 | 6 | 0 | 0 |
| 05-artifacts | 37 | 37 | 0 | 0 |
| 06-flexnetos | 3 | 3 | 0 | 0 |
| 07-verification | 5 | 5 | 0 | 0 |
| 08-release | 2 | 2 | 0 | 0 |
| 09-drive-maintenance | 4 | 4 | 0 | 0 |

## Checksums

| Path | SHA-256 |
|---|---|
| `execution-framework/generated/contract_manifest.json` | `3c2e2a883b6dfc7f135c4dc101484cced9f877191b46bb378f1cc4fcd07e1270` |
| `execution-framework/generated/envctl_artifact_registry_report.json` | `fdf98b2bc184a8021726591983f603b1929497ba66af13eeb97f68345d8a1351` |
| `execution-framework/generated/envctl_migration_db_model.json` | `494ba4e90d62c06522875f0d257c1914fd09141346872572114c446e17b13eb6` |
| `execution-framework/generated/envctl_target_registry.json` | `b348da84d166d774e13d64f9f9ff4bfab72925d36e25760ca3f2a1d3391eff23` |
| `execution-framework/generated/envctl_validation_evidence_report.json` | `f28f76c36e46718a8593e22a0351da1599ce9ca41ccc0505c53027159841d67f` |
| `execution-framework/generated/final_verification_report.json` | `d8a0a8932eedf0401b7073cf2e77fae86fc187d886c2d85432518709a50ab1ec` |
| `execution-framework/generated/package_scan.json` | `0d851e9f6a67044bfd6b6df77231c518292a385304e1830d579e6520634b9487` |
| `execution-framework/generated/shared_protocol_validation_report.json` | `0863b9b28e702c704c96f50a50954181f42b35aaa16e4ecb6643497093c4c4d0` |
| `execution-framework/generated/status_from_proofs.json` | `a3cac733b0fd6064f55126ca2db938b0550215f08545f6c87de7fa658fd9685c` |
| `execution-framework/generated/task_graph.csv` | `b752be4c4cf53cb3db3daf5daaef180ec291269ce67a7bb7f0dbb2e835168009` |

## Contract Mapping

- Contract row: `artifact:06-testing-validation-validation-reconciliation-reports-md`
- Canonical path: `migration-artifacts/06-testing-validation/validation-reconciliation-reports.md`
- Task-scoped Markdown: `migration-artifacts/art-123_validation_reconciliation/validation-reconciliation-reports.md`
- Task-scoped JSON: `migration-artifacts/art-123_validation_reconciliation/validation-reconciliation-reports.json`

## Output Gate

The artifact registry gate is satisfied when the task-scoped Markdown, task-scoped JSON, and canonical contract Markdown paths are registered with SHA-256 content hashes and linked to validation evidence.
