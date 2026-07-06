# Shadow Traffic Comparison Report

- Task: `ART-130_SHADOW_TRAFFIC`
- Target: `flexnetos-vs-lifeos`
- Primary root: `/home/flexnetos/FlexNetOS`
- Compare root: `/home/flexnetos/lifeos`
- Contract artifact: `06-testing-validation-shadow-traffic-comparison-report-md`
- Canonical path: `migration-artifacts/06-testing-validation/shadow-traffic-comparison-report.md`
- Generated: `2026-07-04T23:20:50+00:00`

## Result

The artifact registry and shared protocol dependencies are present, and this report records the old/new comparison surfaces needed for shadow traffic validation. No mirrored live request/response payload capture was present in the target descriptor, repo scan, envctl database model, generated logs, or registry reports available to this task, so runtime parity is not certified here.

## Evidence Inputs

| input | path | sha256 |
| --- | --- | --- |
| `target_descriptor` | `examples/target-descriptors/flexnetos-vs-lifeos.yaml` | `71f4aab77e91d0fa9a414350dee50f23fc3c6492b265c607fe9e4de93c3fe190` |
| `repo_scan` | `execution-framework/generated/package_scan.json` | `0d851e9f6a67044bfd6b6df77231c518292a385304e1830d579e6520634b9487` |
| `envctl_db_model` | `execution-framework/generated/envctl_migration_db_model.json` | `7347f6629cc4fb0dbae56105dec65d5937a2a55860e0df4215b309e3ef766cef` |
| `envctl_db_validation` | `execution-framework/generated/envctl_migration_db_validation_report.json` | `652f839cfd8a46e3d9057588a051c56fc090dce0fbf7ed8c0407c032b4e19d84` |
| `artifact_registry_report` | `execution-framework/generated/envctl_artifact_registry_report.json` | `b3d33175a5ec64d95bb2a2dc0b503ca897c97990bf78fd80fc159780b3df113c` |
| `shared_protocol_schema` | `execution-framework/schemas/shared_protocol.schema.json` | `a897b67a1ec37804d74cbc60eb1360667d823b4661016a76a29906f016275007` |
| `contract_manifest` | `execution-framework/generated/contract_manifest.json` | `3c2e2a883b6dfc7f135c4dc101484cced9f877191b46bb378f1cc4fcd07e1270` |
| `req024_proof` | `execution-framework/proof_records/REQ-024_ENVCTL_ARTIFACT_REGISTRY.proof.json` | `fb6103ba1e8ca4c520a43fa3a18bdae971af6296f714f96c9aab6eee34d37d64` |
| `req040_proof` | `execution-framework/proof_records/REQ-040_SHARED_PROTOCOL_SCHEMAS.proof.json` | `88ecbe9517d4b24401ea1c5bbd072e1472aa0b9938c34e2c210173f521f1bea1` |

## Source Summary

- Repo scan folders: `10`
- Repo scan files: `111`
- Envctl DB tables expected/actual: `16` / `16`
- Registry validation links in REQ-024 smoke: `2`

## Comparison Matrix

| traffic class | old behavior source | new behavior source | comparison status | decision |
| --- | --- | --- | --- | --- |
| envctl migration run lifecycle | prompt/package shell execution without durable run row | envctl_migration_runs plus envctl_migration_operations | `schema_ready_capture_required` | Shadow traffic can be evaluated once mirrored run events are captured into the run ledger. |
| artifact publication | filesystem artifact path only | envctl_migration_artifacts row with content_hash and producer operation | `registry_path_hash_verified` | Use content_hash equality and validation links as the minimum old/new comparison gate. |
| plugin/user status reads | human reads logs and generated reports directly | shared protocol records for artifacts, evidence, validations, and proof | `protocol_ready_capture_required` | Replay plugin reads against captured envctl rows before cutover. |

## Validation Gates

| gate | status | evidence |
| --- | --- | --- |
| `path_registered` | `pass` | `migration-artifacts/art-130_shadow_traffic/shadow-traffic-comparison-report.md`, `migration-artifacts/06-testing-validation/shadow-traffic-comparison-report.md` |
| `hash_recorded` | `pass_after_registry_write` | `envctl_migration_artifacts.content_hash` |
| `validation_evidence_linked` | `pass_after_registry_write` | `envctl_migration_validations`, `envctl_migration_evidence` |
| `real_traffic_payloads_present` | `warn` | No mirrored request/response payload captures were present in target descriptor, repo scan, or envctl reports. |

## Capture Contract For Real Traffic

Before this can become a runtime parity certification, each mirrored production sample should store these redacted fields: `correlation_id`, `route_or_command`, `request_shape_hash`, old/new response shape hashes, old/new status, old/new latency, `diff_class`, and `redaction_profile`.

## Target Descriptor Excerpt

```yaml
schema_version: 1
target_id: flexnetos-vs-lifeos
target_type: mixed
primary_root: /home/flexnetos/FlexNetOS
compare_root: /home/flexnetos/lifeos
output_root: migration-artifacts
include:
  - "**/*"
exclude:
  - ".git/**"
  - "node_modules/**"
  - "target/**"
  - ".venv/**"
  - "__pycache__/**"
safety:
  default_mode: approval-gated
  max_auto_risk: R2
  allow_network: false
  allow_destructive: false
collectors:
  filesystem: true
  git: true
  package_managers: true
  databases: true
```
