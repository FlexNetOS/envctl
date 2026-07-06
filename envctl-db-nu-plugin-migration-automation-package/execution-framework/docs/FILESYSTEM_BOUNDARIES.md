# Filesystem Boundaries

Task: `REQ-042_FILESYSTEM_BOUNDS`
Scope: `package-root`
Generated at: `2026-07-04T23:19:35+00:00`

## Decision Order

- normalize logical path
- apply blocked_paths first
- apply allowed_paths second
- deny paths that match neither list

## Allowed Paths

| id | pattern | source | purpose |
|---|---|---|---|
| `envctl_repo` | `${ENVCTL_REPO}/**` | `ENVCTL_REPO` | envctl repository root selected by the operator. |
| `nu_plugin_repo` | `${NU_PLUGIN_REPO}/**` | `NU_PLUGIN_REPO` | nu_plugin repository root selected by the operator. |
| `execution_framework` | `execution-framework/**` | `package` | package-local execution framework state, generated artifacts, logs, and proofs. |
| `execution_templates` | `execution-templates/**` | `package` | package-local reusable execution templates. |

## Blocked Paths

| id | pattern | reason |
|---|---|---|
| `dot_env_files` | `**/.env` | environment files are never read into logs, prompts, proofs, or generated artifacts. |
| `secrets_directories` | `**/secrets/**` | secret directories are outside helper read and write scope. |
| `private_key_directories` | `**/private_keys/**` | private key directories are outside helper read and write scope. |
| `pem_files` | `**/*.pem` | PEM material is excluded even inside otherwise allowed roots. |
| `key_files` | `**/*.key` | key material is excluded even inside otherwise allowed roots. |

## Safe Workspaces

| id | path | write policy |
|---|---|---|
| `helper_state` | `execution-framework/state/helpers/${HELPER_ID}/` | append-or-replace files owned by the active helper_id only |
| `generated_artifacts` | `execution-framework/generated/` | additive task-specific files; do not overwrite unrelated task output |
| `logs` | `execution-framework/logs/` | task-specific log files named by task_id |
| `proof_records` | `execution-framework/proof_records/` | task-specific proof file plus proof_ledger.jsonl append/update |

## Helper Rules

- Blocked patterns take precedence over allowed paths.
- Background helpers must not read or write paths that match blocked_paths.
- Background helpers may read and write only paths that match allowed_paths and do not match blocked_paths.
- ENVCTL_REPO and NU_PLUGIN_REPO are operator-provided roots; if unset, helpers must treat those lanes as unavailable.
- Helper scratch writes must use execution-framework/state/helpers/${HELPER_ID}/ unless the task packet names a narrower output path.
- Logs and proof evidence must be redacted using helpers/redaction_patterns.txt before capture.
- Symlink traversal must be evaluated against the final resolved path and denied when it escapes the matched allowed root.

## Verification

Status: `passed`

| check | value |
|---|---|
| `allowed_path_count` | `4` |
| `blocked_path_count` | `5` |
| `safe_workspace_count` | `4` |
| `redaction_pattern_count` | `4` |
| `path_sample_count` | `13` |
| `dependency_status` | `completed` |

Path samples:

- `execution-framework/generated/task_graph.csv`: `allowed` via `allowed_pattern`
- `execution-framework/logs/REQ-042_FILESYSTEM_BOUNDS.log`: `allowed` via `allowed_pattern`
- `execution-templates/TASK_GRAPH_TEMPLATE.csv`: `allowed` via `allowed_pattern`
- `${ENVCTL_REPO}/src/lib.rs`: `allowed` via `allowed_pattern`
- `${NU_PLUGIN_REPO}/src/lib.rs`: `allowed` via `allowed_pattern`
- `execution-framework/.env`: `blocked` via `blocked_pattern`
- `execution-framework/secrets/token.txt`: `blocked` via `blocked_pattern`
- `${ENVCTL_REPO}/private_keys/id_rsa`: `blocked` via `blocked_pattern`
- `${NU_PLUGIN_REPO}/certs/local.pem`: `blocked` via `blocked_pattern`
- `${ENVCTL_REPO}/config/local.key`: `blocked` via `blocked_pattern`
- `history/pre_execution_framework_manifest.json`: `blocked` via `outside_allowed_paths`
- `../outside-package.txt`: `blocked` via `outside_allowed_paths`
- `../execution-framework/generated/task_graph.csv`: `blocked` via `outside_allowed_paths`

No filesystem boundary errors were found.
