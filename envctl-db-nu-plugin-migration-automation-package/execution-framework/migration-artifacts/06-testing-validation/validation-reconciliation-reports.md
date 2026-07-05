# Validation reconciliation reports

Generated at: `2026-07-04T23:29:16+00:00`
Status: `passed`

## Parity

| Check | Count |
|---|---:|
| Task graph rows | 80 |
| Execution packets | 80 |
| Status report tasks | 80 |
| Proof records | 60 |
| Successful tasks | 56 |
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
| 02-envctl-db | 9 | 6 | 3 | 0 |
| 03-nu-plugin | 5 | 4 | 1 | 0 |
| 04-shared | 6 | 4 | 2 | 0 |
| 05-artifacts | 37 | 28 | 9 | 0 |
| 06-flexnetos | 3 | 1 | 2 | 0 |
| 07-verification | 5 | 0 | 5 | 0 |
| 08-release | 2 | 0 | 2 | 0 |
| 09-drive-maintenance | 4 | 4 | 0 | 0 |

## Checksums

| Path | SHA-256 |
|---|---|
| `execution-framework/generated/contract_manifest.json` | `3c2e2a883b6dfc7f135c4dc101484cced9f877191b46bb378f1cc4fcd07e1270` |
| `execution-framework/generated/envctl_artifact_registry_report.json` | `b3d33175a5ec64d95bb2a2dc0b503ca897c97990bf78fd80fc159780b3df113c` |
| `execution-framework/generated/envctl_migration_db_model.json` | `7347f6629cc4fb0dbae56105dec65d5937a2a55860e0df4215b309e3ef766cef` |
| `execution-framework/generated/envctl_target_registry.json` | `e74615f1274f5cbc530b859fa74e774f99f99ef33c7acb7b2a3cc75f0f01e77c` |
| `execution-framework/generated/envctl_validation_evidence_report.json` | `0ac5c601eb16c8df54d7f70573aacf950b18a0ecfe56606ebe9a3ab9d092d3bf` |
| `execution-framework/generated/final_verification_report.json` | `fe82ef8932163155bb724ee0093c4a133abf73157b1f365ccd018110cc2c1e52` |
| `execution-framework/generated/package_scan.json` | `0d851e9f6a67044bfd6b6df77231c518292a385304e1830d579e6520634b9487` |
| `execution-framework/generated/shared_protocol_validation_report.json` | `0863b9b28e702c704c96f50a50954181f42b35aaa16e4ecb6643497093c4c4d0` |
| `execution-framework/generated/status_from_proofs.json` | `72a97f935a3e589a84abebd22ae2cd78c9947611d5c8d6316a343c4667167ea5` |
| `execution-framework/generated/task_graph.csv` | `6388bfe8495f78fb2d89e87d183d258eae9a079bcf67e91ec60cc80a9f36d670` |

## Contract Mapping

- Contract row: `artifact:06-testing-validation-validation-reconciliation-reports-md`
- Canonical path: `migration-artifacts/06-testing-validation/validation-reconciliation-reports.md`
- Task-scoped Markdown: `migration-artifacts/art-123_validation_reconciliation/validation-reconciliation-reports.md`
- Task-scoped JSON: `migration-artifacts/art-123_validation_reconciliation/validation-reconciliation-reports.json`

## Output Gate

The artifact registry gate is satisfied when the task-scoped Markdown, task-scoped JSON, and canonical contract Markdown paths are registered with SHA-256 content hashes and linked to validation evidence.
