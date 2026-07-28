# Data Lineage Map

Task: `ART-109_DATA_LINEAGE`
Generated at: `2026-07-28T11:22:48+00:00`
Target root: `/home/flexnetos/FlexNetOS`

## Scope

This map traces critical migration fields from their schema or database origin, through static transformation evidence, to consuming protocol records, registry views, plugin-facing records, and proof artifacts. Blocked secret and private-key paths are excluded from the scan.

## Summary

| measure | value |
|---|---:|
| field count | 184 |
| critical field count | 27 |
| sql table count | 16 |
| schema source count | 37 |
| target files scanned | 2500 |
| fields with target references | 45 |
| protocol record count | 14 |

## Critical Field Lineage

| field | origin | transformation | consumption | target evidence |
|---|---|---|---|---:|
| `artifact_contract_id` | envctl_migration_recipes, envctl_migration_runs, MigrationRun | stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; database persistence requires the field before record insertion | registry/proof consumers only | 18 |
| `artifact_id` | envctl_migration_artifacts, envctl_migration_validations, envctl Migration Artifact Record, envctl Migration Validation Result, ArtifactRecord | stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; static target scan found producer/update/write references | registry/proof consumers only | 18 |
| `command_hash` | envctl_migration_operations | hash derivation and comparison through sha256 file/content digests | registry/proof consumers only | 18 |
| `compare_root` | envctl_migration_targets, envctl Migration Target Descriptor, TargetDescriptor | static lineage only; no transformation-specific reference found | registry/proof consumers only | 18 |
| `content_hash` | envctl_migration_artifacts, envctl Migration Artifact Record, ArtifactRecord | hash derivation and comparison through sha256 file/content digests; static target scan found producer/update/write references | registry/proof consumers only | 18 |
| `contract_hash` | envctl_migration_artifact_contracts | hash derivation and comparison through sha256 file/content digests; static target scan found producer/update/write references | registry/proof consumers only | 18 |
| `contract_id` | envctl Migration Artifact Record | stable identifier propagation through run, operation, artifact, evidence, and graph-edge records | registry/proof consumers only | 18 |
| `descriptor_hash` | envctl_migration_targets | hash derivation and comparison through sha256 file/content digests; database persistence requires the field before record insertion | registry/proof consumers only | 18 |
| `event_hash` | envctl_migration_run_events, envctl Migration Run Event, RunEvent | hash derivation and comparison through sha256 file/content digests | registry/proof consumers only | 18 |
| `evidence_refs` | envctl Migration Artifact Record, envctl Migration Run Event, envctl Migration Validation Result | artifact evidence and graph-link materialization in proof and registry payloads | registry/proof consumers only | 18 |
| `id` | envctl_migration_targets, envctl_migration_packages, envctl_migration_artifact_contracts, pathRule, workspace | stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; static target scan found producer/update/write references | EvidenceRecord->nu_plugin, ValidationResult->nu_plugin | 18 |
| `links` | envctl Migration Artifact Record, ArtifactRecord | artifact evidence and graph-link materialization in proof and registry payloads; static target scan found producer/update/write references | registry/proof consumers only | 18 |
| `operation_id` | envctl_migration_run_events, envctl_migration_evidence, envctl_migration_approvals, envctl Migration Approval Request, operation, envctl Migration Operation | stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; database persistence requires the field before record insertion | registry/proof consumers only | 18 |
| `package_hash` | envctl_migration_packages | hash derivation and comparison through sha256 file/content digests; static target scan found producer/update/write references | registry/proof consumers only | 18 |
| `path` | envctl_migration_artifacts, envctl Migration Artifact Record, workspace, ArtifactRecord | static lineage only; no transformation-specific reference found | registry/proof consumers only | 18 |
| `previous_event_hash` | envctl_migration_run_events, envctl Migration Run Event, RunEvent | hash derivation and comparison through sha256 file/content digests | registry/proof consumers only | 18 |
| `primary_root` | envctl_migration_targets, envctl Migration Target Descriptor, TargetDescriptor | database persistence requires the field before record insertion | registry/proof consumers only | 18 |
| `proof_uri` | AgentLane, ExecutionPacket, TaskGraphRow | artifact evidence and graph-link materialization in proof and registry payloads | registry/proof consumers only | 18 |
| `recipe_hash` | envctl_migration_recipes | hash derivation and comparison through sha256 file/content digests; database persistence requires the field before record insertion | registry/proof consumers only | 18 |
| `recipe_id` | envctl_migration_runs, envctl Migration Recipe, MigrationRecipe, MigrationRun | stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; database persistence requires the field before record insertion | registry/proof consumers only | 18 |
| `reproducibility_hash` | envctl_migration_runs, MigrationRun | hash derivation and comparison through sha256 file/content digests | registry/proof consumers only | 18 |
| `risk` | envctl_migration_operations, envctl_migration_approvals, envctl Migration Approval Request, operation, envctl Migration Operation | database persistence requires the field before record insertion | registry/proof consumers only | 18 |
| `run_id` | envctl_migration_operations, envctl_migration_run_events, envctl_migration_evidence, envctl Migration Approval Request, envctl Migration Artifact Record, envctl Migration Operation | stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; database persistence requires the field before record insertion | registry/proof consumers only | 18 |
| `sha256` | envctl_migration_evidence, EvidenceRecord | hash derivation and comparison through sha256 file/content digests | registry/proof consumers only | 18 |
| `status` | envctl_migration_runs, envctl_migration_operations, envctl_migration_artifacts, envctl Migration Approval Request, envctl Migration Artifact Record, envctl Migration Operation | state normalization through enum/check constraints and validation status records; static target scan found producer/update/write references | MigrationRun->nu_plugin | 18 |
| `target_id` | envctl_migration_targets, envctl_migration_runs, envctl Migration Target Descriptor, TargetDescriptor, MigrationRun | stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; database persistence requires the field before record insertion | registry/proof consumers only | 18 |
| `validator` | envctl_migration_validations, envctl Migration Validation Result, ValidationResult | database persistence requires the field before record insertion | registry/proof consumers only | 18 |

## Origin And Consumption Details

### `artifact_contract_id`

- Criticality: `critical`
- SQL origin: `envctl_migration_recipes.artifact_contract_id`, `envctl_migration_runs.artifact_contract_id`
- Schema origin: `MigrationRun`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; database persistence requires the field before record insertion

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/replay.rs` | 121 | `reference` | self.must_get(store::CONTRACTS, &run.artifact_contract_id, "contract")?; |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/model.rs` | 175 | `reference` | pub artifact_contract_id: String, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/model.rs` | 186 | `reference` | pub artifact_contract_id: String, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 294 | `reference` | artifact_contract_id: &str, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 298 | `reference` | self.must_get(store::CONTRACTS, artifact_contract_id, "artifact contract")?; |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 314 | `reference` | artifact_contract_id: artifact_contract_id.to_string(), |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 337 | `reference` | artifact_contract_id: recipe.artifact_contract_id.clone(), |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 421 | `reference` | self.must_get(store::CONTRACTS, &run.artifact_contract_id, "contract")?; |

### `artifact_id`

- Criticality: `critical`
- SQL origin: `envctl_migration_artifacts.artifact_id`, `envctl_migration_validations.artifact_id`
- Schema origin: `envctl Migration Artifact Record`, `envctl Migration Validation Result`, `ArtifactRecord`, `ValidationResult`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; static target scan found producer/update/write references; database persistence requires the field before record insertion

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/model.rs` | 253 | `reference` | pub artifact_id: String, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/model.rs` | 297 | `reference` | pub artifact_id: Option<String>, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 62 | `reference` | pub artifact_id: Option<String>, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 755 | `transformation` | /// Upsert an artifact record (UNIQUE(run_id, artifact_id) — updates refresh). |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 760 | `reference` | artifact_id: &str, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 773 | `reference` | let key = child_key(run_id, artifact_id); |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 782 | `reference` | artifact_id: artifact_id.to_string(), |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 809 | `reference` | json!({"artifact_id": artifact_id, "status": row.status.as_str(), "content_hash": content_hash}), |

### `command_hash`

- Criticality: `critical`
- SQL origin: `envctl_migration_operations.command_hash`
- Transformation: hash derivation and comparison through sha256 file/content digests

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/replay.rs` | 165 | `reference` | .filter(\|o\| match (&o.command_redacted, &o.command_hash) { |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/model.rs` | 209 | `reference` | pub command_hash: Option<String>, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 484 | `reference` | command_hash: spec |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/home/agent-env/codex-harness/src/lib.rs` | 1944 | `consumption` | json!({"event":"spawn","decision":"allow","job_id":job.job_id,"pid":pid,"kind":job_kind(argv),"provider":provider,"provider_contract_fingerprint":provider_contract_fingerprint,"pro |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/home/agent-env/codex-harness/src/lib.rs` | 2202 | `consumption` | json!({"event":"run","decision":"allow","exit_code":code,"command_hash":sha256_bytes(command_preview(argv).as_bytes()),"command_preview":command_preview(argv)}), |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/home/agent-env/codex-harness/src/bin/codex-harness-db.rs` | 54 | `reference` | command_hash TEXT, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/sql/001_migration_automation_schema.sql` | 81 | `origin` | command_hash TEXT, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/generated/envctl_replay_report.json` | 182 | `reference` | "command_hash": "sha256:5147442e504529c0b7d11cd880d150...", |

### `compare_root`

- Criticality: `critical`
- SQL origin: `envctl_migration_targets.compare_root`
- Schema origin: `envctl Migration Target Descriptor`, `TargetDescriptor`
- Transformation: static lineage only; no transformation-specific reference found

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/model.rs` | 140 | `reference` | pub compare_root: Option<String>, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 18 | `reference` | pub compare_root: Option<String>, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 172 | `reference` | compare_root: spec.compare_root, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/tests.rs` | 31 | `reference` | compare_root: None, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/tests.rs` | 79 | `reference` | compare_root: None, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/tests.rs` | 89 | `reference` | compare_root: None, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/tests.rs` | 104 | `reference` | compare_root: None, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/cli/src/migration_cmd.rs` | 383 | `reference` | compare_root: Option<String>, |

### `content_hash`

- Criticality: `critical`
- SQL origin: `envctl_migration_artifacts.content_hash`
- Schema origin: `envctl Migration Artifact Record`, `ArtifactRecord`
- Transformation: hash derivation and comparison through sha256 file/content digests; static target scan found producer/update/write references

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/tasks/codedb-nu-plugin-semantic-coverage.md` | 36 | `reference` | - `content_hash`, `blob_ref`, `import_status`, `skip_reason` |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/tasks/codedb-content-blob-inventory.md` | 28 | `reference` | - `content_hash` |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/tasks/codedb-metadata-only-inventory.md` | 29 | `reference` | - empty `content_hash` / `blob_ref` |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/commits/019f24bb-82cb-7462-9bb1-ed51988c969e.json` | 2 | `consumption` | "commit": {"author":"drdave-flexnet <flexnetos@de-flex.net>","changes":[{"change_type":"created","content_hash":"65fb5c8d0cb97ffe0391aea1cb2ae3...","doc_type":"task","document_id": |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/commits/019f24bb-82cb-7462-9bb1-ed51988c969e.json` | 3 | `reference` | "content_hash": "f60e3a28f4b2223078efb0b71bf08f...", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/commits/019f0078-ad4d-7513-acd0-6b51e28f4507.json` | 2 | `reference` | "commit": {"author":"drdave <revenaugh.david@gmail.com>","changes":[{"change_type":"created","content_hash":"8dd918d90abdba2359f8b0b10b1dc8...","doc_type":"context","document_id":" |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/commits/019f0078-ad4d-7513-acd0-6b51e28f4507.json` | 3 | `reference` | "content_hash": "1b210e773b9fd69b04ff0e00976115...", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/commits/019f25be-1090-7090-a9dc-8b5c6bb10b64.json` | 2 | `transformation` | "commit": {"author":"drdave <flexnetos@de-flex.net>","changes":[{"change_type":"modified","content_hash":"9685f1e3f3c30ef7a44aedc5456411...","doc_type":"task","document_id":"019f25 |

### `contract_hash`

- Criticality: `critical`
- SQL origin: `envctl_migration_artifact_contracts.contract_hash`
- Transformation: hash derivation and comparison through sha256 file/content digests; static target scan found producer/update/write references; database persistence requires the field before record insertion

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/replay.rs` | 125 | `reference` | recomputed == contract.contract_hash, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/replay.rs` | 128 | `reference` | contract.contract_hash, recomputed |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/replay.rs` | 264 | `reference` | contract.contract_hash, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/model.rs` | 165 | `reference` | pub contract_hash: String, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 278 | `reference` | contract_hash: sha256_hex(canonical_json(&contract).as_bytes()), |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 434 | `reference` | contract.contract_hash, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/sql/001_migration_automation_schema.sql` | 33 | `origin` | contract_hash TEXT NOT NULL, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/generated/envctl_replay_report.json` | 72 | `reference` | "contract_hash": "sha256:f44bd31a128471fb31d49b61a274c4..." |

### `contract_id`

- Criticality: `critical`
- Schema origin: `envctl Migration Artifact Record`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/proof_records/ART-120_WAVE_PLAN.proof.json` | 53 | `reference` | "contract_id": "contract-full-migration-artifa....0.0", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/proof_records/ART-120_WAVE_PLAN.proof.json` | 91 | `reference` | "contract_id": "contract-full-migration-artifa....0.0", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/proof_records/ART-120_WAVE_PLAN.proof.json` | 129 | `reference` | "contract_id": "contract-full-migration-artifa....0.0", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/proof_records/ART-120_WAVE_PLAN.proof.json` | 167 | `reference` | "contract_id": "contract-full-migration-artifa....0.0", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/proof_records/ART-120_WAVE_PLAN.proof.json` | 211 | `reference` | "contract_id": "contract-full-migration-artifa....0.0", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/proof_records/ART-120_WAVE_PLAN.proof.json` | 261 | `reference` | "contract_id": "contract-full-migration-artifa....0.0", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/proof_records/ART-120_WAVE_PLAN.proof.json` | 315 | `reference` | "contract_id": "contract-full-migration-artifa....0.0", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/proof_records/ART-120_WAVE_PLAN.proof.json` | 365 | `reference` | "contract_id": "contract-full-migration-artifa....0.0", |

### `descriptor_hash`

- Criticality: `critical`
- SQL origin: `envctl_migration_targets.descriptor_hash`
- Transformation: hash derivation and comparison through sha256 file/content digests; database persistence requires the field before record insertion

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/replay.rs` | 107 | `origin` | recomputed == target.descriptor_hash, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/replay.rs` | 110 | `origin` | target.descriptor_hash, recomputed |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/replay.rs` | 262 | `origin` | target.descriptor_hash, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/model.rs` | 142 | `origin` | pub descriptor_hash: String, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 173 | `origin` | descriptor_hash: sha256_hex(canonical_json(&spec.descriptor).as_bytes()), |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 359 | `origin` | "target_hash": target.descriptor_hash, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 432 | `origin` | target.descriptor_hash, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 463 | `origin` | target.descriptor_hash, |

### `event_hash`

- Criticality: `critical`
- SQL origin: `envctl_migration_run_events.event_hash`
- Schema origin: `envctl Migration Run Event`, `RunEvent`
- Transformation: hash derivation and comparison through sha256 file/content digests

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/replay.rs` | 144 | `reference` | clean.event_hash = None; |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/replay.rs` | 152 | `reference` | if Some(&recomputed) != ev.event_hash.as_ref() { |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/replay.rs` | 157 | `reference` | previous = ev.event_hash.clone(); |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/replay.rs` | 258 | `reference` | .and_then(\|e\| e.event_hash.clone()) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/model.rs` | 232 | `reference` | pub event_hash: Option<String>, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 112 | `reference` | prev.event_hash |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 128 | `reference` | event_hash: None, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 137 | `reference` | ev.event_hash = Some(sha256_hex(material.as_bytes())); |

### `evidence_refs`

- Criticality: `critical`
- Schema origin: `envctl Migration Artifact Record`, `envctl Migration Run Event`, `envctl Migration Validation Result`, `RunEvent`, `ArtifactRecord`, `ValidationResult`
- Transformation: artifact evidence and graph-link materialization in proof and registry payloads

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 102 | `reference` | evidence_refs: Option<Value>, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 126 | `reference` | evidence_refs_json: evidence_refs, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 650 | `reference` | evidence_refs: Option<Value>, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 679 | `reference` | evidence_refs, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/PROMPT_PACKAGE_COMBINED.md` | 491 | `reference` | "evidence_refs": [], |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/schemas/artifact_record.schema.json` | 44 | `origin` | "evidence_refs": { |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/schemas/validation_result.schema.json` | 39 | `origin` | "evidence_refs": { |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/schemas/run_event.schema.json` | 56 | `origin` | "evidence_refs": { |

### `id`

- Criticality: `critical`
- SQL origin: `envctl_migration_targets.id`, `envctl_migration_packages.id`, `envctl_migration_artifact_contracts.id`, `envctl_migration_recipes.id`, `envctl_migration_runs.id`, `envctl_migration_operations.id`
- Schema origin: `pathRule`, `workspace`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; static target scan found producer/update/write references; database persistence requires the field before record insertion
- Consumption: `EvidenceRecord` via `envctl_migration_evidence` to `nu_plugin`, `ValidationResult` via `envctl_migration_validations` to `nu_plugin`

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 39 | `transformation` | \| 2026-06-28 \| forge-loop \| Owner-supervised migrations should climb a review ladder in separate PRs — status/precondition, validation, scaffold, explicit writer, and only then any |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 42 | `consumption` | \| 2026-07-09 \| feature-forge \| Treat an upstream ANN/HNSW result set as a MULTISET, not a set: a k-NN index can return the SAME node twice (identical id+score) in one result set, s |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 74 | `reference` | cargo run -p envctl -- graph --why cuda-oxide # root->id dependency paths |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 100 | `reference` | cargo run -p envctl -- add-repo https://github.com/sharkdp/pastel --id pastel-rs \ |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 106 | `reference` | https://github.com/sharkdp/pastel --id pastel --build |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 111 | `reference` | cargo run -p envctl -- add-repo https://github.com/owner/repo --id thing \ |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/retire-broken-pre-cleanroom-codex-hook-baseline.md` | 2 | `reference` | id: 019f2a52-3abb-79a3-b941-73f974... |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/expand-codedb-nu-plugin-coverage-beyond-file-impor.md` | 2 | `reference` | id: 019f246e-3f8c-74d1-bd1e-fac84a... |

### `links`

- Criticality: `critical`
- Schema origin: `envctl Migration Artifact Record`, `ArtifactRecord`
- Transformation: artifact evidence and graph-link materialization in proof and registry payloads; static target scan found producer/update/write references

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 132 | `reference` | `lemon.c` codegen, which emits Rust and links nothing); (2) **transform it to a rust-native |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/AGENTS.md` | 122 | `reference` | `lemon.c` codegen, which emits Rust and links nothing); (2) **transform it to a rust-native |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/Cargo.toml` | 46 | `reference` | # Pure-Rust FFI bindings (extern "C" decls only — compiles no C, links no new C lib) used solely by |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/AGENTS.md` | 692 | `reference` | Go back and add links: |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/tasks/catalog-runtime-closure-and-toolchain-proof.md` | 62 | `reference` | - [ ] `envctl catalog analyze --json` reports non-zero `migration_evidence` rows for the intended imported/configured surfaces, with provenance that links generated artifacts back  |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/tasks/import-host-artifacts-yazelix-mission-control-20260703.md` | 69 | `reference` | - [ ] Cross-references: this task links to the merged yazelix PR (see references below) and to tasks/envctl-codex-mcp-runtime... which follows the same file-into-envctl pattern for |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/commits/019f2d99-0ab9-7940-9e86-7aa369ce278b.json` | 2 | `transformation` | "commit": {"author":"drdave <flexnetos@de-flex.net>","changes":[{"change_type":"modified","content_hash":"5f7c405cf62c29d8e6a6b0f0d4fd91...","doc_type":"task","document_id":"019f2a |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/kb-close/SKILL.md` | 38 | `reference` | Check if the document has a "Completion Evidence" section (commit hashes, PR links, test results, verification steps). This is distinct from the "Progress Log" (chronological work  |

### `operation_id`

- Criticality: `critical`
- SQL origin: `envctl_migration_run_events.operation_id`, `envctl_migration_evidence.operation_id`, `envctl_migration_approvals.operation_id`, `envctl_migration_validations.operation_id`, `envctl_migration_checkpoints.operation_id`, `envctl_migration_rollbacks.operation_id`
- Schema origin: `envctl Migration Approval Request`, `operation`, `envctl Migration Operation`, `envctl Migration Run Event`, `Operation`, `RunEvent`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; database persistence requires the field before record insertion

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/model.rs` | 228 | `reference` | pub operation_id: Option<String>, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/model.rs` | 240 | `reference` | pub operation_id: Option<String>, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/model.rs` | 283 | `reference` | pub operation_id: String, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/model.rs` | 298 | `reference` | pub operation_id: Option<String>, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/model.rs` | 310 | `reference` | pub operation_id: Option<String>, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/model.rs` | 322 | `reference` | pub operation_id: Option<String>, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/views.rs` | 37 | `consumption` | pub operation_id: Option<String>, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/views.rs` | 48 | `consumption` | pub operation_id: String, |

### `package_hash`

- Criticality: `critical`
- SQL origin: `envctl_migration_packages.package_hash`
- Transformation: hash derivation and comparison through sha256 file/content digests; static target scan found producer/update/write references; database persistence requires the field before record insertion

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/model.rs` | 154 | `reference` | pub package_hash: String, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 233 | `reference` | let package_hash = sha256_hex(material.as_bytes()); |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 239 | `reference` | package_hash, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/cli/src/migration_cmd.rs` | 799 | `reference` | "package_hash": pkg.package_hash, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/sql/001_migration_automation_schema.sql` | 22 | `origin` | package_hash TEXT NOT NULL, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/sql/001_migration_automation_schema.sql` | 25 | `origin` | UNIQUE(package_name, package_hash) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/generated/contract_manifest.seed.sql` | 4 | `transformation` | INSERT INTO envctl_migration_packages (id, package_name, package_path, package_hash, manifest_json) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/generated/contract_manifest.seed.sql` | 6 | `origin` | ON CONFLICT(package_name, package_hash) DO NOTHING; |

### `path`

- Criticality: `critical`
- SQL origin: `envctl_migration_artifacts.path`
- Schema origin: `envctl Migration Artifact Record`, `workspace`, `ArtifactRecord`
- Transformation: static lineage only; no transformation-specific reference found

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 14 | `reference` | \| 2026-06-17 \| feature-forge \| A stated gap usually implies an adjacent unstated gap — the architect must trace the full call path and fold any missing seam (missing field/hardcode |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 28 | `consumption` | \| 2026-06-22 \| feature-forge \| RECURRENCE-CONFIRM (no new upgrade): the verify-against-HEAD-before-des... class (rows 2026-06-17 Phase-0 step 4 + 2026-06-18 Phase-0 step 5) general |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 29 | `consumption` | \| 2026-06-23 \| forge-loop \| Don't dodge an infra limit — AUTHENTICATE with the infra the owner already built. When automation hits a quota/permission wall (GitHub API 403), the fix |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 34 | `reference` | \| 2026-06-23 \| forge-loop \| When a subagent dies mid-cycle (weekly model limit / terminal API error), the orchestrator must finish the cycle deterministically rather than abandon i |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 46 | `reference` | \| 2026-07-09 \| forge-loop \| A missing build capability (a cross-compile target's rust-std, a toolchain lib) is fixed at the OWNING nix-flake SOURCE + foundation rebuild — never rus |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 25 | `reference` | and tested — notably: `--connect` bypassed all spec validation (path traversal + |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/README.md` | 73 | `consumption` | envctl is the path authority for meta installs. Components and add-repo drop-ins should not |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/README.md` | 76 | `consumption` | \| purpose \| canonical path \| |

### `previous_event_hash`

- Criticality: `critical`
- SQL origin: `envctl_migration_run_events.previous_event_hash`
- Schema origin: `envctl Migration Run Event`, `RunEvent`
- Transformation: hash derivation and comparison through sha256 file/content digests

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/replay.rs` | 138 | `reference` | if ev.previous_event_hash != previous { |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/model.rs` | 231 | `reference` | pub previous_event_hash: Option<String>, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 127 | `reference` | previous_event_hash: previous.clone(), |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/tests.rs` | 154 | `reference` | ev.previous_event_hash, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/PROMPT_PACKAGE_COMBINED.md` | 492 | `reference` | "previous_event_hash": "...", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/PROMPT_PACKAGE_COMBINED.md` | 884 | `reference` | event_hash = sha256(previous_event_hash + canonical_event_json) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/schemas/run_event.schema.json` | 62 | `origin` | "previous_event_hash": { |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/prompts/DATABASE_FEATURE_SPEC.md` | 64 | `reference` | "previous_event_hash": "...", |

### `primary_root`

- Criticality: `critical`
- SQL origin: `envctl_migration_targets.primary_root`
- Schema origin: `envctl Migration Target Descriptor`, `TargetDescriptor`
- Transformation: database persistence requires the field before record insertion

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/model.rs` | 139 | `reference` | pub primary_root: String, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 17 | `reference` | pub primary_root: String, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 153 | `reference` | if spec.primary_root.trim().is_empty() { |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 154 | `reference` | return Err(MigrationDbError::Validation("primary_root is empty".into())); |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 171 | `reference` | primary_root: spec.primary_root, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/tests.rs` | 30 | `reference` | primary_root: "/tmp".into(), |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/tests.rs` | 78 | `reference` | primary_root: "/tmp".into(), |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/tests.rs` | 88 | `reference` | primary_root: "/tmp".into(), |

### `proof_uri`

- Criticality: `critical`
- Schema origin: `AgentLane`, `ExecutionPacket`, `TaskGraphRow`
- Transformation: artifact evidence and graph-link materialization in proof and registry payloads

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/home/.codex/RULES.md` | 145 | `reference` | - Rule of Proof: every entry in the Task Graph requires a `proof_uri`. If a |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/home/.codex/RULES.md` | 146 | `reference` | row exists without a `proof_uri`, the build must fail. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/source/execution-framework-task-prompt.md` | 137 | `reference` | - proof_uri |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/source/execution-framework-task-prompt.md` | 179 | `reference` | - proof_uri |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/source/execution-framework-task-prompt.md` | 320 | `origin` | - required proof_uri exists |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/source/execution-framework-task-prompt.md` | 326 | `reference` | - no task can be Complete without proof_uri |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/source/execution-framework-task-prompt.md` | 400 | `reference` | - proof_uri |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/schemas/task_graph.schema.json` | 39 | `origin` | "proof_uri", |

### `recipe_hash`

- Criticality: `critical`
- SQL origin: `envctl_migration_recipes.recipe_hash`
- Transformation: hash derivation and comparison through sha256 file/content digests; database persistence requires the field before record insertion

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/replay.rs` | 116 | `reference` | "recipe_hash", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/replay.rs` | 117 | `reference` | recomputed == recipe.recipe_hash, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/replay.rs` | 118 | `reference` | format!("recorded {} recomputed {}", recipe.recipe_hash, recomputed), |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/replay.rs` | 263 | `reference` | recipe.recipe_hash, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/model.rs` | 176 | `reference` | pub recipe_hash: String, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 315 | `reference` | recipe_hash: sha256_hex(canonical_json(&recipe).as_bytes()), |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 361 | `reference` | "recipe_hash": recipe.recipe_hash, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 433 | `reference` | recipe.recipe_hash, |

### `recipe_id`

- Criticality: `critical`
- SQL origin: `envctl_migration_runs.recipe_id`
- Schema origin: `envctl Migration Recipe`, `MigrationRecipe`, `MigrationRun`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; database persistence requires the field before record insertion

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/replay.rs` | 72 | `reference` | let recipe: Recipe = self.must_get(store::RECIPES, &run.recipe_id, "recipe")?; |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/replay.rs` | 113 | `reference` | let recipe: Recipe = self.must_get(store::RECIPES, &run.recipe_id, "recipe")?; |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/model.rs` | 185 | `reference` | pub recipe_id: String, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 27 | `reference` | pub recipe_id: String, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 331 | `reference` | let recipe: Recipe = self.must_get(store::RECIPES, &spec.recipe_id, "recipe")?; |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 336 | `reference` | recipe_id: recipe.id.clone(), |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 419 | `reference` | let recipe: Recipe = self.must_get(store::RECIPES, &run.recipe_id, "recipe")?; |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 1065 | `reference` | recipe: self.must_get(store::RECIPES, &run.recipe_id, "recipe")?, |

### `reproducibility_hash`

- Criticality: `critical`
- SQL origin: `envctl_migration_runs.reproducibility_hash`
- Schema origin: `MigrationRun`
- Transformation: hash derivation and comparison through sha256 file/content digests

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/replay.rs` | 255 | `reference` | if let Some(recorded) = &run.reproducibility_hash { |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/replay.rs` | 270 | `reference` | "reproducibility_hash", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/model.rs` | 193 | `reference` | pub reproducibility_hash: Option<String>, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/views.rs` | 73 | `consumption` | pub reproducibility_hash: Option<String>, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/views.rs` | 204 | `consumption` | has_reproducibility_hash: run.reproducibility_hash.is_some(), |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/views.rs` | 205 | `consumption` | reproducibility_hash: run.reproducibility_hash, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 344 | `reference` | reproducibility_hash: None, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 438 | `reference` | run.reproducibility_hash = Some(sha256_hex(material.as_bytes())); |

### `risk`

- Criticality: `critical`
- SQL origin: `envctl_migration_operations.risk`, `envctl_migration_approvals.risk`
- Schema origin: `envctl Migration Approval Request`, `operation`, `envctl Migration Operation`, `Operation`, `ApprovalRequest`
- Transformation: database persistence requires the field before record insertion

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 272 | `reference` | \| 2026-06-13 \| Eject `rust-port-merge` harness for the kasetto absorption (Epic C / TASK-0012) \| .claude/skills/{rust-port,rust-port-inventory,rust-port-translate,rust-port-parity, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 273 | `reference` | \| 2026-06-17 \| G2 retro: encode the instincts that made the run clean (evolution-steward, 5 low-risk APPLY) \| skills/feature-forge (Phase 0 verify-triggering-claim step; Phase 1.5  |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 439 | `reference` | - **MUST run impact analysis before editing any symbol.** Before modifying a function, class, or method, run `impact({target: "symbolName", direction: "upstream"})` and report the  |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 441 | `reference` | - **MUST warn the user** if impact analysis returns HIGH or CRITICAL risk before proceeding with edits. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 449 | `reference` | - NEVER ignore HIGH or CRITICAL risk warnings from impact analysis. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/AGENTS.md` | 317 | `reference` | - **MUST run impact analysis before editing any symbol.** Before modifying a function, class, or method, run `impact({target: "symbolName", direction: "upstream"})` and report the  |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/AGENTS.md` | 319 | `reference` | - **MUST warn the user** if impact analysis returns HIGH or CRITICAL risk before proceeding with edits. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/AGENTS.md` | 327 | `reference` | - NEVER ignore HIGH or CRITICAL risk warnings from impact analysis. |

### `run_id`

- Criticality: `critical`
- SQL origin: `envctl_migration_operations.run_id`, `envctl_migration_run_events.run_id`, `envctl_migration_evidence.run_id`, `envctl_migration_artifacts.run_id`, `envctl_migration_graph_edges.run_id`, `envctl_migration_approvals.run_id`
- Schema origin: `envctl Migration Approval Request`, `envctl Migration Artifact Record`, `envctl Migration Operation`, `envctl Migration Run Event`, `envctl Migration Validation Result`, `Operation`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; database persistence requires the field before record insertion

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/agent-env/tests/parity_vs_kasetto.rs` | 1694 | `reference` | run_id: "run-1".into(), |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/agent-env/tests/parity_vs_kasetto.rs` | 1710 | `reference` | assert_eq!(v["run_id"], "run-1"); |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/agent-env/src/report.rs` | 50 | `reference` | pub run_id: String, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/agent-env/src/report.rs` | 119 | `reference` | run_id: "run-1".into(), |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/agent-env/src/report.rs` | 135 | `reference` | assert_eq!(v["run_id"], "run-1"); |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/replay.rs` | 43 | `reference` | pub run_id: String, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/replay.rs` | 54 | `reference` | run_id: &str, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/replay.rs` | 59 | `reference` | ReplayMode::VerifyOnly => self.replay_verify(run_id, verify_files), |

### `sha256`

- Criticality: `critical`
- SQL origin: `envctl_migration_evidence.sha256`
- Schema origin: `EvidenceRecord`
- Transformation: hash derivation and comparison through sha256 file/content digests

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/retire-broken-pre-cleanroom-codex-hook-baseline.md` | 41 | `reference` | sha256: 2f3122f94d847314886fe1999b8cf6... |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/commits/019f2a56-e524-74a3-a034-3a3cc2473e3f.json` | 2 | `origin` | "commit": {"author":"drdave <flexnetos@de-flex.net>","changes":[{"change_type":"modified","content_hash":"bdb9105f2aeea09f400ad2bc160aeb...","doc_type":"incident","document_id":"01 |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/executor.rs` | 1937 | `reference` | for leaf in ["secretctl", "secretctl.sha256", "secretctl.source.sha256"] { |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/executor.rs` | 1983 | `reference` | for leaf in secretctl secretctl.sha256 secretctl.source.sha256; do |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/executor.rs` | 1993 | `reference` | for leaf in secretctl secretctl.sha256 secretctl.source.sha256; do |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/executor.rs` | 2053 | `reference` | for leaf in ["secretctl", "secretctl.sha256", "secretctl.source.sha256"] { |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/db_index.rs` | 412 | `reference` | assert_eq!(main.content_hash.len(), 64); // sha256 hex |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/catalog.rs` | 315 | `reference` | pub sha256: String, |

### `status`

- Criticality: `critical`
- SQL origin: `envctl_migration_runs.status`, `envctl_migration_operations.status`, `envctl_migration_artifacts.status`, `envctl_migration_approvals.status`, `envctl_migration_validations.status`, `envctl_migration_rollbacks.status`
- Schema origin: `envctl Migration Approval Request`, `envctl Migration Artifact Record`, `envctl Migration Operation`, `envctl Migration Validation Result`, `AgentLane`, `ProofRecord`
- Transformation: state normalization through enum/check constraints and validation status records; static target scan found producer/update/write references; database persistence requires the field before record insertion
- Consumption: `MigrationRun` via `envctl_migration_runs` to `nu_plugin`

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 8 | `reference` | \| date \| harness \| lesson (class, generalized) \| evidence (cycle/finding) \| recurrence \| routed-to \| status \| |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 11 | `reference` | \| date \| harness \| lesson (class, generalized) \| evidence \| recurrence \| routed-to \| status \| |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 39 | `transformation` | \| 2026-06-28 \| forge-loop \| Owner-supervised migrations should climb a review ladder in separate PRs — status/precondition, validation, scaffold, explicit writer, and only then any |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 51 | `consumption` | (`git fetch && git status` — confirm clean and even with `origin/master`): |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 69 | `reference` | name `envctl` to `meta git worktree status` unless this helper derived it from |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 266 | `origin` | \| 2026-06-05 \| Add A2 cross-repo parallel build (default-OFF, scale auto-trigger) \| skills/{feature-forge,forge-loop,session-relay}; agents/{rust-implementer,continuity-steward} \|  |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 269 | `consumption` | \| 2026-06-12 \| Migrate harness durable state `_workspace/`→`.handoff/loop/`; add kasetto-absorption capability + handoff-sync skill + hf-aware continuity \| skills/{forge-loop,featu |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 274 | `consumption` | \| 2026-06-17 \| Anti-drift: backlog reconcile is now a FAIL-CLOSED wrap-up gate (+ full drift sweep) \| skills/session-relay-wrap-up (new step 3b); .handoff/loop/backlog.md (reconcil |

### `target_id`

- Criticality: `critical`
- SQL origin: `envctl_migration_targets.target_id`, `envctl_migration_runs.target_id`
- Schema origin: `envctl Migration Target Descriptor`, `TargetDescriptor`, `MigrationRun`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records; database persistence requires the field before record insertion

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/catalog.rs` | 684 | `reference` | pub target_id: String, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/catalog.rs` | 719 | `reference` | target_id: String, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/catalog.rs` | 877 | `reference` | .then(a.target_id.cmp(&b.target_id)) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/catalog.rs` | 1345 | `reference` | key: row.target_id.clone(), |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/catalog.rs` | 2664 | `reference` | let target_id = nix_store_path_id(entry_name, &frontdoor.store_path); |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/catalog.rs` | 2673 | `reference` | link_target_id: Some(target_id), |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/catalog.rs` | 3266 | `reference` | target_id: item.target_id, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/catalog.rs` | 4808 | `reference` | row.target_id == "repo_settings_default" |

### `validator`

- Criticality: `critical`
- SQL origin: `envctl_migration_validations.validator`
- Schema origin: `envctl Migration Validation Result`, `ValidationResult`
- Transformation: database persistence requires the field before record insertion

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/model.rs` | 299 | `reference` | pub validator: String, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 60 | `reference` | pub validator: String, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 832 | `reference` | validator: spec.validator.clone(), |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 852 | `reference` | json!({"validator": spec.validator, "status": spec.status.as_str()}), |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/tests.rs` | 335 | `reference` | validator: "byte-parity".into(), |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/cli/src/migration_cmd.rs` | 174 | `consumption` | long_about = "Record a validation result (pass/fail/warn/blocked/unknown) from a named validator, optionally linked to an artifact and operation. Rows feed the scorecard view; reco |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/cli/src/migration_cmd.rs` | 176 | `reference` | "envctl migration validation run-000001 --validator byte-parity --status pass --details '{\"mismatched\":0}'", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/cli/src/migration_cmd.rs` | 182 | `reference` | validator: String, |

### `actor`

- Criticality: `supporting`
- Schema origin: `ProofRecord`, `ProofRecord`
- Transformation: static lineage only; no transformation-specific reference found

### `actor_id`

- Criticality: `supporting`
- SQL origin: `envctl_migration_run_events.actor_id`
- Schema origin: `envctl Migration Run Event`, `RunEvent`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/model.rs` | 227 | `reference` | pub actor_id: Option<String>, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/views.rs` | 36 | `consumption` | pub actor_id: Option<String>, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/views.rs` | 130 | `consumption` | actor_id: e.actor_id, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 99 | `reference` | actor_id: Option<&str>, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 123 | `reference` | actor_id: actor_id.map(str::to_string), |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 329 | `reference` | pub fn create_run(&self, spec: RunSpec, actor: ActorType, actor_id: &str) -> Result<Run> { |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 355 | `reference` | Some(actor_id), |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 383 | `reference` | actor_id: &str, |

### `actor_type`

- Criticality: `supporting`
- SQL origin: `envctl_migration_run_events.actor_type`
- Schema origin: `envctl Migration Run Event`, `RunEvent`
- Transformation: database persistence requires the field before record insertion

### `agent_name`

- Criticality: `supporting`
- SQL origin: `envctl_migration_agent_sessions.agent_name`
- Transformation: database persistence requires the field before record insertion

### `agent_runtime`

- Criticality: `supporting`
- Schema origin: `AgentLane`, `ExecutionPacket`, `TaskGraphRow`
- Transformation: static lineage only; no transformation-specific reference found

### `allow_destructive`

- Criticality: `supporting`
- Schema origin: `safety`
- Transformation: static lineage only; no transformation-specific reference found

### `allow_network`

- Criticality: `supporting`
- Schema origin: `safety`
- Transformation: static lineage only; no transformation-specific reference found

### `allowed_paths`

- Criticality: `supporting`
- Schema origin: `AgentLane`, `ExecutionPacket`, `envctl Background Helper Filesystem Boundaries`, `TaskGraphRow`
- Transformation: static lineage only; no transformation-specific reference found

### `approval_id`

- Criticality: `supporting`
- Schema origin: `envctl Migration Approval Request`, `ApprovalRequest`, `ApprovalDecision`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/views.rs` | 46 | `consumption` | pub approval_id: String, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/views.rs` | 169 | `consumption` | approval_id: a.id, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 634 | `reference` | json!({"approval_id": id, "risk": op.risk.as_str()}), |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 645 | `reference` | approval_id: &str, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 652 | `reference` | let mut approval: Approval = self.must_get(store::APPROVALS, approval_id, "approval")?; |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 655 | `consumption` | "approval {approval_id} already {}", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 666 | `reference` | self.put(store::APPROVALS, approval_id, &approval, false)?; |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 675 | `reference` | "approval_id": approval_id, |

### `approval_policy`

- Criticality: `supporting`
- SQL origin: `envctl_migration_runs.approval_policy`
- Transformation: static lineage only; no transformation-specific reference found

### `artifact_contract`

- Criticality: `supporting`
- Schema origin: `envctl Migration Target Descriptor`, `TargetDescriptor`
- Transformation: static lineage only; no transformation-specific reference found

### `artifact_type`

- Criticality: `supporting`
- SQL origin: `envctl_migration_artifacts.artifact_type`
- Schema origin: `envctl Migration Artifact Record`, `ArtifactRecord`
- Transformation: static lineage only; no transformation-specific reference found

### `authority_level`

- Criticality: `supporting`
- SQL origin: `envctl_migration_agent_sessions.authority_level`
- Transformation: static lineage only; no transformation-specific reference found

### `blocked_paths`

- Criticality: `supporting`
- Schema origin: `AgentLane`, `ExecutionPacket`, `envctl Background Helper Filesystem Boundaries`, `TaskGraphRow`
- Transformation: static lineage only; no transformation-specific reference found

### `blocks`

- Criticality: `supporting`
- Schema origin: `ExecutionPacket`, `TaskGraphRow`
- Transformation: static lineage only; no transformation-specific reference found

### `can_run_parallel`

- Criticality: `supporting`
- Schema origin: `ExecutionPacket`, `TaskGraphRow`
- Transformation: static lineage only; no transformation-specific reference found

### `checksums`

- Criticality: `supporting`
- Schema origin: `ProofRecord`, `ProofRecord`
- Transformation: static lineage only; no transformation-specific reference found

### `code`

- Criticality: `supporting`
- Schema origin: `StructuredError`
- Transformation: static lineage only; no transformation-specific reference found

### `command_redacted`

- Criticality: `supporting`
- SQL origin: `envctl_migration_operations.command_redacted`
- Transformation: static lineage only; no transformation-specific reference found

### `command_template`

- Criticality: `supporting`
- Schema origin: `ExecutionPacket`, `TaskGraphRow`
- Transformation: static lineage only; no transformation-specific reference found

### `commands`

- Criticality: `supporting`
- Schema origin: `nu_plugin envctl Migration Command Manifest`
- Transformation: static lineage only; no transformation-specific reference found

### `commands_run`

- Criticality: `supporting`
- Schema origin: `ProofRecord`, `ProofRecord`
- Transformation: static lineage only; no transformation-specific reference found

### `completed_at`

- Criticality: `supporting`
- Schema origin: `ProofRecord`, `ProofRecord`
- Transformation: static lineage only; no transformation-specific reference found

### `completed_at_utc`

- Criticality: `supporting`
- SQL origin: `envctl_migration_runs.completed_at_utc`, `envctl_migration_operations.completed_at_utc`
- Schema origin: `ReplayResult`
- Transformation: static lineage only; no transformation-specific reference found

### `completion_gate`

- Criticality: `supporting`
- Schema origin: `ExecutionPacket`, `TaskGraphRow`
- Transformation: static lineage only; no transformation-specific reference found

### `confidence`

- Criticality: `supporting`
- SQL origin: `envctl_migration_graph_edges.confidence`
- Transformation: static lineage only; no transformation-specific reference found

### `contract_json`

- Criticality: `supporting`
- SQL origin: `envctl_migration_artifact_contracts.contract_json`
- Transformation: database persistence requires the field before record insertion

### `contract_name`

- Criticality: `supporting`
- SQL origin: `envctl_migration_artifact_contracts.contract_name`
- Transformation: database persistence requires the field before record insertion

### `contract_version`

- Criticality: `supporting`
- SQL origin: `envctl_migration_artifact_contracts.contract_version`
- Transformation: database persistence requires the field before record insertion

### `created_at_utc`

- Criticality: `supporting`
- SQL origin: `envctl_migration_targets.created_at_utc`, `envctl_migration_artifact_contracts.created_at_utc`, `envctl_migration_recipes.created_at_utc`, `envctl_migration_runs.created_at_utc`, `envctl_migration_operations.created_at_utc`, `envctl_migration_run_events.created_at_utc`
- Schema origin: `MigrationRun`
- Transformation: database persistence requires the field before record insertion

### `current_task`

- Criticality: `supporting`
- Schema origin: `AgentLane`
- Transformation: static lineage only; no transformation-specific reference found

### `decided_at_utc`

- Criticality: `supporting`
- SQL origin: `envctl_migration_approvals.decided_at_utc`
- Schema origin: `ApprovalDecision`
- Transformation: static lineage only; no transformation-specific reference found

### `decided_by`

- Criticality: `supporting`
- SQL origin: `envctl_migration_approvals.decided_by`
- Schema origin: `envctl Migration Approval Request`, `ApprovalRequest`, `ApprovalDecision`
- Transformation: static lineage only; no transformation-specific reference found

### `decision`

- Criticality: `supporting`
- Schema origin: `ApprovalDecision`
- Transformation: static lineage only; no transformation-specific reference found
- Consumption: `ApprovalDecision` via `envctl_migration_approvals` to `nu_plugin`

### `default_mode`

- Criticality: `supporting`
- Schema origin: `safety`
- Transformation: static lineage only; no transformation-specific reference found

### `depends_on`

- Criticality: `supporting`
- Schema origin: `phase`, `ExecutionPacket`, `TaskGraphRow`
- Transformation: static lineage only; no transformation-specific reference found

### `description`

- Criticality: `supporting`
- Schema origin: `pathRule`
- Transformation: static lineage only; no transformation-specific reference found

### `descriptor_json`

- Criticality: `supporting`
- SQL origin: `envctl_migration_targets.descriptor_json`
- Transformation: database persistence requires the field before record insertion

### `details_json`

- Criticality: `supporting`
- SQL origin: `envctl_migration_validations.details_json`
- Transformation: static lineage only; no transformation-specific reference found

### `edge_id`

- Criticality: `supporting`
- Schema origin: `GraphEdge`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/schemas/shared_protocol.schema.json` | 871 | `origin` | "edge_id", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/schemas/shared_protocol.schema.json` | 878 | `origin` | "edge_id": { |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/artifact_registry.py` | 294 | `reference` | edge_id = _stable_id("edge", item.run_id, item.artifact_id, link["to"], link["type"]) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/artifact_registry.py` | 309 | `reference` | edge_id, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/artifact_registry.py` | 319 | `reference` | rows.append({"id": edge_id, "to": link["to"], "type": link["type"]}) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/verify_shared_protocol_schemas.py` | 151 | `origin` | "required": ["edge_id", "run_id", "from_node", "to_node", "edge_type"], |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/verify_shared_protocol_schemas.py` | 153 | `origin` | "edge_id": {"type": "string", "minLength": 1}, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/verify_shared_protocol_schemas.py` | 380 | `origin` | "edge_id": "edge-001", |

### `edge_type`

- Criticality: `supporting`
- SQL origin: `envctl_migration_graph_edges.edge_type`
- Schema origin: `GraphEdge`
- Transformation: database persistence requires the field before record insertion

### `error_json`

- Criticality: `supporting`
- SQL origin: `envctl_migration_operations.error_json`
- Transformation: static lineage only; no transformation-specific reference found

### `event_id`

- Criticality: `supporting`
- Schema origin: `ApprovalDecision`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/schemas/shared_protocol.schema.json` | 811 | `origin` | "event_id": { |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/generated/nu_plugin_command_manifest.json` | 70 | `consumption` | "event_id", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/generated/nu_plugin_command_manifest.json` | 117 | `consumption` | "event_id", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/generated/nu_plugin_command_manifest.json` | 207 | `consumption` | "event_id", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/generated/nu_plugin_command_manifest.json` | 223 | `consumption` | "event_id", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/generated/nu_plugin_command_manifest.json` | 347 | `consumption` | "event_id", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/generated/nu_plugin_command_manifest.json` | 363 | `consumption` | "event_id", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/generated/nu_plugin_command_manifest.json` | 392 | `consumption` | "event_id", |

### `event_seq`

- Criticality: `supporting`
- SQL origin: `envctl_migration_run_events.event_seq`
- Schema origin: `envctl Migration Run Event`, `RunEvent`
- Transformation: database persistence requires the field before record insertion

### `event_type`

- Criticality: `supporting`
- SQL origin: `envctl_migration_run_events.event_type`
- Schema origin: `envctl Migration Run Event`, `RunEvent`
- Transformation: database persistence requires the field before record insertion

### `evidence`

- Criticality: `supporting`
- Schema origin: `ProofRecord`, `ProofRecord`
- Transformation: static lineage only; no transformation-specific reference found
- Consumption: `EvidenceRecord` via `envctl_migration_evidence` to `nu_plugin`

### `evidence_id`

- Criticality: `supporting`
- Schema origin: `EvidenceRecord`
- Transformation: stable identifier propagation through run, operation, artifact, evidence, and graph-edge records

| file | line | role | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/migration_db/api.rs` | 742 | `reference` | json!({"evidence_id": id, "uri": uri, "kind": kind, "sha256": sha256}), |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/schemas/shared_protocol.schema.json` | 824 | `origin` | "evidence_id", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/schemas/shared_protocol.schema.json` | 831 | `origin` | "evidence_id": { |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/generated/envctl_run_ledger_report.json` | 89 | `reference` | "evidence_id": "evidence-9673d92f06e5ade87c6892ff", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/generated/envctl_run_ledger_report.json` | 96 | `reference` | "evidence_id": "evidence-8c1d94286c7e5bbfa87d54f1", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/generated/security_redaction_controls.json` | 95 | `reference` | "evidence_id", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/envctl_run_ledger.py` | 426 | `reference` | evidence_id = f"evidence-{stable_id(run_id, uri, evidence_kind)}" |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/envctl_run_ledger.py` | 434 | `reference` | evidence_id, |

### `evidence_json`

- Criticality: `supporting`
- SQL origin: `envctl_migration_artifacts.evidence_json`, `envctl_migration_graph_edges.evidence_json`, `envctl_migration_validations.evidence_json`
- Transformation: static lineage only; no transformation-specific reference found

### `evidence_kind`

- Criticality: `supporting`
- SQL origin: `envctl_migration_evidence.evidence_kind`
- Schema origin: `EvidenceRecord`
- Transformation: database persistence requires the field before record insertion

### `evidence_refs_json`

- Criticality: `supporting`
- SQL origin: `envctl_migration_run_events.evidence_refs_json`
- Transformation: static lineage only; no transformation-specific reference found

### `execution_cell`

- Criticality: `supporting`
- Schema origin: `ExecutionPacket`, `TaskGraphRow`
- Transformation: static lineage only; no transformation-specific reference found

### `failure_reason`

- Criticality: `supporting`
- Schema origin: `ProofRecord`, `ProofRecord`
- Transformation: static lineage only; no transformation-specific reference found

### `files_changed`

- Criticality: `supporting`
- Schema origin: `ProofRecord`, `ProofRecord`
- Transformation: static lineage only; no transformation-specific reference found

## Validation

- Files scanned: `2500`
- Truncated: `True`
- Skipped: `{"max_files_reached": 1, "too_large": 11, "unsupported_suffix": 344}`
- Artifact registry persisted path and content hash for the canonical markdown, task markdown, and JSON artifact.
- Registry links include REQ-024 artifact registry, REQ-040 shared protocol schemas, and VER-300 unit validation.
