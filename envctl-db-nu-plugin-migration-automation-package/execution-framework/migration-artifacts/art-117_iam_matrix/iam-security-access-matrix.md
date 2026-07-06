# ART-117 IAM and Security Access Matrix

Generated at: `2026-07-04T23:19:57+00:00`
Status: `complete`
Target: `flexnetos-vs-lifeos` (mixed)

## Target Safety

| field | value |
|---|---|
| primary root | `/home/flexnetos/FlexNetOS` |
| compare root | `/home/flexnetos/lifeos` |
| safety mode | `approval-gated` |
| max automatic risk | `R2` |
| descriptor hash | `sha256:b3f653f9e9cda7991821687f041cb540ce3e4342bde03ef333aa0e72a6b42384` |

## Access Matrix

| principal | type | authority | permissions | scope | risk | evidence |
|---|---|---|---|---|---|---|
| `human:migration-operator` | user | approval-gated human control | review generated plans and artifacts<br>approve or deny operations represented in envctl_migration_approvals<br>operate nu_plugin approval commands from the operator session template | R3+ and destructive/high-risk migration steps; no direct secret material recorded | medium | `schemas/approval_request.schema.json`<br>`examples/nu/operator-session-template.nu`<br>`execution-framework/docs/SHARED_PROTOCOL_SCHEMAS.md` |
| `agent:artifact-agent` | agent | workspace-write artifact generation | read target descriptors, package scan, and envctl DB model evidence<br>write migration-artifacts/art-117_iam_matrix outputs<br>register artifacts, evidence links, validations, and graph edges | allowed_paths from packet; blocked secret paths are excluded | low | `execution-framework/generated/execution_packets/ART-117_IAM_MATRIX.json`<br>`execution-framework/scripts/artifact_registry.py`<br>`execution-framework/generated/envctl_artifact_registry_report.json` |
| `service:envctl-migration-db` | service | persistence for migration control plane | persist targets, packages, contracts, recipes, runs, operations, evidence, artifacts, graph edges, approvals, validations, checkpoints, rollbacks, agent sessions, and plugin sessions<br>store redacted command strings and evidence/artifact hashes | SQLite-compatible envctl migration schema in this package | medium | `sql/001_migration_automation_schema.sql`<br>`execution-framework/docs/ENVCTL_DB_SCHEMA.md`<br>`execution-framework/generated/envctl_migration_db_model.json` |
| `service:target-filesystem-collector` | service | read-only discovery collector | read included target filesystem paths<br>skip excluded and blocked paths<br>produce discovery evidence for downstream artifacts | include **/* minus .git, node_modules, target, virtualenv, __pycache__, .env, secrets, private_keys, pem, and key files | low | `examples/target-descriptors/flexnetos-vs-lifeos.yaml`<br>`execution-framework/generated/envctl_target_registry.json`<br>`execution-framework/generated/package_scan.json` |
| `plugin:nu_plugin-operator-surface` | plugin | human-facing control and status surface | start plans and runs through envctl migration commands<br>list and submit approval decisions<br>display status streams, visual tables, graph views, and replay/rollback surfaces | operator-mediated plugin commands; no credential persistence in plugin session schema | medium | `codex/AGENTS.nu_plugin.md.template`<br>`execution-framework/docs/SHARED_PROTOCOL_SCHEMAS.md`<br>`sql/001_migration_automation_schema.sql` |
| `agent:spark-security-reproducibility` | agent | design/review helper for security controls | design sandboxing, approvals, redaction, evidence hashing, provenance, and replay safety controls<br>produce additive prompts or artifact inputs | workspace-write, on-request approval policy in helper config | low | `codex/agents/spark-security-reproducibility.config.toml`<br>`prompts/SECURITY_REPRODUCIBILITY_MODEL.md` |
| `external:github-integration` | external_integration | external collaboration surface | prepare GitHub issue text updates and PR sequencing artifacts<br>consume GitHub auth outside this artifact package when live publishing is performed | issue text/spec surfaces only in this package; no token values observed or persisted | medium | `specs/github-issues-index.md`<br>`codex/agents/spark-issue-integrator.config.toml` |

## Credentials, Certs, And Tokens

| item | type | status | storage | evidence |
|---|---|---|---|---|
| `blocked-path-policy` | secret_path_policy | enforced | policy only | `execution-framework/generated/execution_packets/ART-117_IAM_MATRIX.json`<br>`execution-framework/scripts/artifact_registry.py` |
| `command-redaction` | redaction_control | modeled | envctl_migration_operations.command_redacted | `sql/001_migration_automation_schema.sql`<br>`prompts/SECURITY_REPRODUCIBILITY_MODEL.md` |
| `token-references` | token_reference | references-only | redacted source signal list | `execution-framework/migration-artifacts/art-117_iam_matrix/iam-security-access-matrix.json` |
| `certificate-key-material` | certificate_or_key | not_collected | not persisted | `execution-framework/generated/execution_packets/ART-117_IAM_MATRIX.json` |

## Source Signal Summary

| category | count |
|---|---:|
| approval_control | 34 |
| external_identity_reference | 7 |
| iam_reference | 2 |
| runtime_boundary | 2 |
| secret_or_credential_reference | 75 |

## Redaction Notes

- Secret-bearing path patterns were not opened or copied into this artifact.
- Source signals record file path, line number, keyword category, and redaction status only.
- Token, certificate, and credential references are treated as inventory leads, not as secret values.
