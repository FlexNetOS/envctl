# Env Table Schema

Status: design contract for T027.

Source task: `execution_artifacts/revised_task_table.csv` row `T027`.

Related pack goals:
- `goal_pack/GOAL.md`
- `goal_pack/subgoals/01-envctl-table-doctrine.md`
- `goal_pack/subgoals/09-generator-contract.md`

## Intent

This document defines the canonical env table schema by extending the existing envctl
catalog model. It is not a second table system and it is not a bulk migration script.

Permanent flow:

```text
original files -> Nushell staging tables -> canonical envctl tables/views -> validation -> generated files -> runtime verification
```

The existing Rust catalog remains the bridge from repo/control-plane files into typed rows.
The new schema names the missing canonical classes, maps them onto current catalog rows
where possible, and marks the required extensions for follow-up implementation tasks.

## Current Catalog Baseline

`crates/engine/src/catalog.rs` already exposes these stable tables:

| Current table | Current row type | Existing role |
|---|---|---|
| `components` | `ComponentRow` | Component roster, dependency order, lifecycle availability, lock hash. |
| `component_hooks` | `ComponentHookRow` | Detect/install/verify/fix/remove hook commands and hook env. |
| `paths` | `PathRow` | `MetaLayout` paths, canonical/legacy/bridge/protected flags. |
| `settings` | `SettingRow` | Flattened config settings with source, scope, precedence, sensitivity, override metadata. |
| `env_vars` | `EnvVarRow` | Env variables from hooks, layout exports, and env schema references. |
| `agent_assets` | `AgentAssetRow` | Skills, agents, hooks, MCP assets, destinations, lock/drift state. |
| `registries` | `RegistryRow` | Hub/registry/MCP entries and component link state. |
| `config_files` | `ConfigFileRow` | Control-plane files, format, ownership, read/parse/drift status. |
| `migration_evidence` | `MigrationEvidenceRow` | Legacy-to-canonical evidence, verifier state, rollback and purge state. |
| `observed_facts` | `ObservedFactRow` | Read-only observations and row-count/verifier facts. |

The same module also defines table-adjacent reports that must be promoted into canonical
views as implementation progresses:

| Existing report row | Canonical target |
|---|---|
| `CatalogRenderedFile` | `generated_files` |
| `CatalogDriftRow` | `conflicts` and `validation_errors` |
| `CatalogSyncAction` | generator/apply work queue; may stay report-only until row edit support exists. |
| `CatalogLockReport` | release/table checksum provenance. |

The CLI exposes `envctl catalog scan`, `envctl catalog table <name>`, `envctl catalog
import`, `envctl catalog diff`, `envctl catalog render --out`, `envctl catalog sync`,
and `envctl catalog lock`. `scan`, `table`, `import`, `diff`, `render`, and `sync` are
read-only against the repo. `lock --apply` is the current explicit write surface and only
updates `manifest/envctl.lock`.

## Source Roles

Every row must declare one source role:

| Role | Meaning | Mutation policy |
|---|---|---|
| `original` | Existing file discovered on disk or in a repo. | Preserve path, checksum, read/parse status, and backup reference. |
| `staging` | Nushell-produced parse/import row before canonical normalization. | May be regenerated from original files. |
| `canonical` | Envctl table/view row accepted as source of truth. | Manual/reviewable row edits only; ledger required. |
| `generated` | Runtime output derived from canonical rows. | Never hand-edit as source of truth; regenerate and diff. |
| `deferred` | Required class intentionally not implemented for v0.1 slice. | Must include blocking reason and owner. |

## Common Columns

All canonical tables/views must expose these fields, either directly or through a stable
projection:

| Column | Type | Rule |
|---|---|---|
| `row_id` | string | Stable deterministic id. Use `<table>:<owner>:<subject>` where possible. |
| `table_name` | string | Canonical table/view name. |
| `schema_version` | string | `envctl.env_table_schema.v0` until the first incompatible change. |
| `owner` | string | Component, tool, repo, domain, or `operator`. |
| `source_role` | string | One of the source roles above. |
| `source_path` | string or null | Repo-relative or absolute source path. |
| `source_format` | string or null | `toml`, `json`, `yaml`, `nuon`, `csv`, `dotenv`, `shell`, `kdl`, `text`, `derived`, or `unknown`. |
| `source_checksum` | string or null | SHA-256 for file-backed rows; null for derived/deferred rows. |
| `source_row_ref` | string or null | Link back to staging/original row where available. |
| `scope` | string | `workspace`, `user`, `login`, `shell`, `runner`, `component`, `repo`, `host`, `release`, or `test`. |
| `precedence` | integer | Lower number wins unless the table states otherwise. |
| `sensitive` | bool | True when the row names, references, or derives secret-bearing data. |
| `secret_ref` | string or null | Reference id only; never a raw secret value. |
| `generated` | bool | True only for generated artifacts or generated projections. |
| `manual_override` | bool | True only when an operator-approved row overrides detected state. |
| `override_reason` | string or null | Required when `manual_override` is true. |
| `review_required` | bool | True when mutation, secret policy, or unresolved ownership needs review. |
| `validation_status` | string | `ok`, `warning`, `error`, `deferred`, or `unknown`. |
| `validation_message` | string or null | Human-readable verifier detail. |
| `conflict_id` | string or null | Link to `conflicts.row_id` when unresolved. |
| `deferred_reason` | string or null | Required when `validation_status = deferred`. |

Existing rows do not need to store duplicate physical columns if a typed view can project
the common fields deterministically.

## Canonical Tables

### `env_files`

Purpose: inventory every discovered env/config/state candidate before parsing.

Current base: `config_files`.

Primary key: `path`.

Required columns:
- `path`, `format`, `owner`, `source_role`, `runtime_role`, `mutability`, `exists`.
- `read_status`, `parse_status`, `source_checksum`, `generated`, `sensitive`.
- `backup_path`, `discovery_method`, `v0_1_status`, `deferred_reason`.

Validation:
- Every discovered input is classified as original, generated, state, cache, log, secret
  reference, or deferred.
- Generated files must also have `generated_files` rows.

### `env_vars`

Purpose: structured env variables with producer/consumer/scope/precedence/sensitivity.

Current base: `env_vars`; extend with precedence and conflict links.

Primary key: `var_name`, `scope`, `producer`, `consumer`.

Required columns:
- Existing `EnvVarRow` fields.
- `precedence`, `operation` (`set`, `append`, `prepend`, `remove`, `inherit`), `separator`,
  `secret_ref`, `conflict_id`, `validation_status`.

Validation:
- Secret-looking names must have `value = null` or redacted value plus `secret_ref`.
- Duplicate effective variables at the same scope must produce deterministic conflict rows.

### `tool_versions`

Purpose: installed tools, expected versions, source refs, binary paths, and release inclusion.

Current base: `components`, `component_hooks`, `observed_facts`, `settings`.

Primary key: `tool_name`, `owner`.

Required columns:
- `tool_name`, `owner`, `install_source`, `version_command`, `detected_version`,
  `expected_version`, `binary_path`, `repo_url`, `repo_rev`, `package_source`,
  `install_status`, `release_manifest`, `validation_status`.

Validation:
- If `release_manifest = true`, detected version and binary checksum are required.

### `path_entries`

Purpose: search path entries and ordering for `PATH`, `LD_LIBRARY_PATH`, `CPATH`,
`PKG_CONFIG_PATH`, `MANPATH`, CUDA paths, Cargo/Nix paths, and binary exposure paths.

Current base: `paths` plus `env_vars`.

Primary key: `path_var`, `order`, `path`.

Required columns:
- `path_var`, `path`, `order`, `operation`, `owner`, `path_kind`, `canonical`,
  `legacy`, `bridge`, `protected`, `exists`, `duplicate_of`, `conflict_id`.

Validation:
- Protected and canonical entries must not be shadowed by lower-precedence legacy entries
  without a conflict row.

### `config_files`

Purpose: original/imported config file inventory and parser state.

Current base: `config_files`; extend with checksum and replacement policy.

Primary key: `config_id`.

Required columns:
- Existing `ConfigFileRow` fields.
- `source_checksum`, `replacement_policy`, `backup_path`, `owner_review_status`.

Validation:
- `generated = true` requires a matching `generated_files` row.
- `manual_override = true` requires reason, owner, and timestamp.

### `generated_files`

Purpose: declared output artifacts and the checksum relationship to source tables.

Current base: `CatalogRenderedFile` and generated `config_files` rows.

Primary key: `path`.

Required columns:
- `path`, `format`, `owner`, `source_tables`, `source_table_checksums`,
  `output_checksum`, `generator`, `generation_timestamp`, `header_status`,
  `manual_edits_allowed`, `diff_status`, `runtime_verifier`, `runtime_status`,
  `replacement_policy`.

Validation:
- `manual_edits_allowed` must be false for envctl-owned runtime config.
- Every generated file must include a provenance header with generator, source table,
  source checksum, timestamp, and do-not-edit notice.
- Save/apply must refuse undeclared output paths.

### `secrets`

Purpose: secret references only.

Current base: `settings`, `env_vars`, secrets-engine config/schema observations.

Primary key: `secret_ref`.

Required columns:
- `secret_ref`, `provider`, `scope`, `required_by`, `consumer`, `env_var_name`,
  `backing_path_ref`, `rotation_policy`, `validation_status`, `last_verified_at`.

Validation:
- Raw secret values are invalid in tracked rows, generated files, and logs.
- Secret refs may name env vars or vault rows but must not embed tokens, cookies, or keys.

### `mcp_servers`

Purpose: MCP server declarations, allowlists, env refs, and mutation policy.

Current base: `registries`, `agent_assets`, `settings`.

Primary key: `server_name`, `target_config`.

Required columns:
- `server_name`, `target_config`, `command`, `args`, `owner`, `source_file`,
  `env_refs`, `secret_refs`, `allowlist`, `mutation_policy`, `transport`,
  `enabled`, `validation_status`.

Validation:
- Env entries that reference secrets must point to `secrets.secret_ref`.
- Mutation policy must be explicit before generated Codex/MCP fragments are accepted.

### `codex_settings`

Purpose: Codex config/profile/auth policy/session/log/MCP/sandbox/context rows.

Current base: `settings`, `config_files`, `agent_assets`, `registries`.

Primary key: `setting_key`, `scope`, `profile`.

Required columns:
- `profile`, `setting_key`, `value`, `source_file`, `scope`, `precedence`,
  `auth_policy_ref`, `mcp_server_refs`, `log_dir_ref`, `state_dir_ref`,
  `sandbox_policy`, `approval_policy`, `context_policy`, `validation_status`.

Validation:
- Auth policy rows may reference official auth state, but must not expose auth material.

### `yazelix_settings`

Purpose: Yazelix settings fragments, popup commands, status widgets, entrypoints, child
assets, and generated runtime ownership.

Current base: `settings`, `config_files`, `agent_assets`, `path_entries`.

Primary key: `setting_key`, `source_file`, `surface`.

Required columns:
- `surface`, `setting_key`, `value`, `source_file`, `format`, `owner`,
  `generated_target`, `runtime_path`, `path_entry_refs`, `child_asset_refs`,
  `validation_status`.

Validation:
- Generated Yazelix fragments must be diffed against existing settings before replacement.

### `rtk_settings`

Purpose: RTK telemetry, tee/raw-log policy, hooks, exclusions, config paths, and local
state directories.

Current base: `settings`, `config_files`, `log_dirs`, `cache_dirs`.

Primary key: `setting_key`, `scope`.

Required columns:
- `setting_key`, `value`, `scope`, `config_path`, `telemetry_policy`, `tee_policy`,
  `raw_log_policy`, `exclude_rule`, `hook_owner`, `state_dir_ref`, `log_dir_ref`,
  `validation_status`.

Validation:
- Raw logs must remain preserved and redaction policy must be explicit.

### `gitkb_settings`

Purpose: GitKB `.kb`, MCP, task docs, index policy, backup, cleanup, and memory policy.

Current base: `settings`, `registries`, `config_files`, `cache_dirs`, `log_dirs`.

Primary key: `setting_key`, `scope`.

Required columns:
- `kb_root`, `mcp_server_ref`, `index_policy`, `task_doc_path`, `backup_policy`,
  `cleanup_policy`, `memory_policy`, `cache_dir_ref`, `log_dir_ref`,
  `validation_status`.

Validation:
- GitKB memory must be explicit; no hidden session memory can satisfy release evidence.

### `meta_workspace_projects`

Purpose: `.meta.yaml` project rows, hollow workspace peers, source refs, tags, provides,
dependencies, and drift/deferred state.

Current base: `settings`, `config_files`, `registries`, `observed_facts`.

Primary key: `project_id`.

Required columns:
- `project_id`, `repo_url`, `checkout_path`, `rev`, `branch`, `tags`, `provides`,
  `depends_on`, `protected`, `hollow`, `owner`, `drift_status`, `validation_status`.

Validation:
- Protected sibling path dependencies must exist or produce blocking validation errors.

### `runner_settings`

Purpose: `flexnetos_runner` env/profile, preflight gates, approvals, raw logs, work dirs,
and release outputs.

Current base: `settings`, `env_vars`, `config_files`, `release_dirs`, `log_dirs`.

Primary key: `setting_key`, `profile`.

Required columns:
- `profile`, `setting_key`, `value`, `preflight_gate`, `approval_policy`,
  `codex_profile_ref`, `work_dir_ref`, `raw_log_dir_ref`, `release_dir_ref`,
  `secret_refs`, `validation_status`.

Validation:
- Runner profiles must record command provenance and release manifest checksums.

### `rust_toolchains`

Purpose: Fenix/stable/latest/nightly, rust-src, rust-analyzer, rustfmt, clippy, targets,
Cargo env, Kache, and Wild linker compatibility.

Current base: `tool_versions`, `path_entries`, `env_vars`, `settings`.

Primary key: `toolchain_id`, `target`.

Required columns:
- `toolchain_id`, `channel`, `target`, `components`, `cargo_home_ref`,
  `rustup_home_ref`, `rustc_wrapper_ref`, `linker_profile_ref`, `source_ref`,
  `detected_version`, `validation_status`.

Validation:
- Optional linker/cache accelerators must preserve fallback build/test paths.

### `cuda_nvidia`

Purpose: detected NVIDIA driver/toolkit facts, CUDA paths, source references, and
validate-only policy.

Current base: `observed_facts`, `tool_versions`, `path_entries`, `env_vars`.

Primary key: `host`, `device_or_tool`.

Required columns:
- `device_or_tool`, `detected_version`, `driver_version`, `cuda_home`,
  `path_entry_refs`, `library_path_refs`, `validation_command`, `validation_status`,
  `install_policy`.

Validation:
- Generator must not reinstall CUDA/NVIDIA; it may only render env fragments from
  detected and verified facts.

### `database_endpoints`

Purpose: SQLite/libSQL/sqld/Postgres/Redis/vector-store/local DB paths, URL refs, data
dirs, backup, cleanup, and smoke tests.

Current base: `settings`, `config_files`, `secrets`, `cache_dirs`, `log_dirs`.

Primary key: `endpoint_id`.

Required columns:
- `endpoint_id`, `engine`, `scope`, `url_ref`, `path`, `data_dir_ref`,
  `migration_policy`, `backup_policy`, `cleanup_policy`, `smoke_command`,
  `secret_refs`, `validation_status`.

Validation:
- URLs with credentials must be represented by refs, not raw values.

### `cache_dirs`

Purpose: cache directories, owners, retention, cleanup, tracked/untracked state.

Current base: `paths`, `settings`, `migration_evidence`.

Primary key: `path`, `owner`.

Required columns:
- `path`, `owner`, `cache_kind`, `retention_policy`, `cleanup_policy`,
  `tracked_status`, `size_limit`, `last_verified_at`, `validation_status`.

Validation:
- Cache dirs are regenerable unless the row explicitly says otherwise.

### `log_dirs`

Purpose: raw command logs, RTK tee, Codex logs, runner logs, envctl logs, release logs,
retention, and redaction policy.

Current base: `paths`, `settings`, `observed_facts`.

Primary key: `path`, `owner`.

Required columns:
- `path`, `owner`, `log_kind`, `raw_preservation`, `redaction_policy`,
  `retention_policy`, `release_inclusion`, `validation_status`.

Validation:
- Raw logs required for gates must not be replaced by summaries.

### `release_dirs`

Purpose: release root, artifacts, BOM, checksums, manifests, logs, generated config
bundle, and handoff docs.

Current base: `paths`, `config_files`, `generated_files`, `observed_facts`.

Primary key: `release_id`, `path`.

Required columns:
- `release_id`, `path`, `artifact_kind`, `source_table_refs`, `checksum`,
  `manifest_path`, `handoff_path`, `validation_status`.

Validation:
- Release manifest fails if table or generated-file checksums are missing.

### `conflicts`

Purpose: duplicate/conflicting rows with severity and manual resolution state.

Current base: `CatalogDriftRow`; extend for precedence decisions.

Primary key: `conflict_id`.

Required columns:
- `conflict_id`, `table_name`, `subject`, `conflict_kind`, `candidate_rows`,
  `winner_row`, `severity`, `resolution_status`, `manual_owner`,
  `resolution_reason`, `validation_status`.

Validation:
- Generation must refuse unresolved `error` conflicts.

### `validation_errors`

Purpose: malformed rows, stale checksums, parse errors, secret leak findings, unsafe
generated outputs, and blocking severity.

Current base: `CatalogDriftRow`, `observed_facts`, validator reports.

Primary key: `error_id`.

Required columns:
- `error_id`, `table_name`, `row_id`, `severity`, `error_kind`, `message`,
  `source_path`, `verifier`, `blocking`, `fix_hint`, `validation_status`.

Validation:
- Every parser or policy failure must be visible as a row, not only as process output.

## Table Mapping Summary

| Required table/view | Existing base | T027 classification |
|---|---|---|
| `env_files` | `config_files` | Extend view. |
| `env_vars` | `env_vars` | Extend row with precedence/conflict fields. |
| `tool_versions` | `components`, `observed_facts` | New view. |
| `path_entries` | `paths`, `env_vars` | New view. |
| `config_files` | `config_files` | Extend row with checksum/replacement policy. |
| `generated_files` | `CatalogRenderedFile`, generated `config_files` | New table/view. |
| `secrets` | `settings`, `env_vars`, secrets engine surfaces | New table/view. |
| `mcp_servers` | `registries`, `agent_assets`, `settings` | New view. |
| `codex_settings` | `settings`, `config_files`, `registries` | New view. |
| `yazelix_settings` | `settings`, `config_files`, `path_entries` | New view. |
| `rtk_settings` | `settings`, `config_files`, `log_dirs` | New view. |
| `gitkb_settings` | `settings`, `registries`, `cache_dirs`, `log_dirs` | New view. |
| `meta_workspace_projects` | `.meta.yaml` via `config_files`/`settings` | New parser/view. |
| `runner_settings` | `settings`, `env_vars`, `release_dirs` | New view. |
| `rust_toolchains` | `tool_versions`, `path_entries`, `env_vars` | New view. |
| `cuda_nvidia` | `observed_facts`, `tool_versions`, `path_entries` | New view. |
| `database_endpoints` | `settings`, `config_files`, `secrets` | New table/view. |
| `cache_dirs` | `paths`, `settings`, `migration_evidence` | New view. |
| `log_dirs` | `paths`, `settings`, `observed_facts` | New view. |
| `release_dirs` | `paths`, `generated_files`, `observed_facts` | New view. |
| `conflicts` | `CatalogDriftRow` | New table/view. |
| `validation_errors` | `CatalogDriftRow`, `observed_facts` | New table/view. |

## Nushell Staging Contract

Nushell staging tables are allowed to exist as CSV, NUON, JSON, or in-memory command output
while parser tasks mature. They are not the durable source of truth until normalized into
canonical envctl tables/views.

Preferred operations:

```nu
open
from toml
from json
from yaml
from csv
from nuon
detect columns
lines
parse
str trim
where
insert
upsert
merge
join
group-by
sort-by
uniq-by
to nuon
to csv
to json
to toml
```

Fallback text parsing must emit unsupported-format or parse-error rows instead of guessing.
`save --force` is allowed only for paths declared in `generated_files` after validation and
manual diff review.

## Generated File Contract

Every generated artifact must be declared in `generated_files` before writing. The generator
must refuse undeclared paths.

Every generated file must include equivalent metadata using the target file's comment syntax:

```text
generated by envctl
source table:
source table checksum:
generation timestamp:
do not edit directly; update envctl table instead
```

Generated targets required by the pack:

| Target | Required source tables |
|---|---|
| `bootstrap.nu` | `env_vars`, `path_entries`, `tool_versions`, `secrets`, `generated_files` |
| `bootstrap.sh` | `env_vars`, `path_entries`, `tool_versions`, `secrets`, `generated_files` |
| Codex fragments | `codex_settings`, `mcp_servers`, `secrets`, `log_dirs`, `cache_dirs` |
| MCP config | `mcp_servers`, `secrets`, `generated_files` |
| Yazelix fragments | `yazelix_settings`, `path_entries`, `generated_files` |
| RTK config | `rtk_settings`, `log_dirs`, `cache_dirs` |
| GitKB references | `gitkb_settings`, `mcp_servers`, `meta_workspace_projects` |
| meta fragments | `meta_workspace_projects`, `tool_versions`, `config_files` |
| runner env | `runner_settings`, `env_vars`, `secrets`, `release_dirs`, `log_dirs` |
| Rust/Fenix/Kache/wild env | `rust_toolchains`, `tool_versions`, `path_entries`, `cache_dirs` |
| CUDA/NVIDIA env | `cuda_nvidia`, `path_entries`, `tool_versions`, `validation_errors` |
| database env/config | `database_endpoints`, `secrets`, `cache_dirs`, `log_dirs` |

## Validation Gates

Validation must produce rows in `conflicts` or `validation_errors` for:

| Gate | Blocking condition |
|---|---|
| Parse/read | Missing file, unreadable file, unsupported format, malformed row. |
| Source freshness | Stale or missing `source_checksum`. |
| Precedence | Duplicate env vars, duplicate path order, or silent last-writer-wins behavior. |
| Secrets | Raw tokens, cookies, private keys, URLs with credentials, or unredacted secret values. |
| Ownership | Missing owner, generated file without declaration, file outside allowed roots. |
| Runtime | Generated output fails verifier or cannot be diffed against prior state. |
| Release | Missing table/generated-file checksum or missing raw log evidence. |

Generation and release gates must fail when any `validation_errors.blocking = true` row or
unresolved `conflicts.severity = error` row exists.

## Deferral Rules

A required class may be deferred only by emitting a row with:

- `table_name`
- `row_id`
- `validation_status = deferred`
- `deferred_reason`
- `owner`
- `unblocks_task`
- `followup_task`

Silent absence is not a valid deferral.

## Implementation Sequence

1. T028 discovers files into `env_files` staging rows.
2. T029 exposes Nushell-native inventory rows.
3. T030-T191 parse known env/config formats into staging rows.
4. T192 normalizes staging rows into this canonical schema.
5. T193-T194 validate, create `validation_errors`, and resolve `conflicts`.
6. T195 documents manual row-edit workflow and ledger requirements.
7. T196-T218 implement or explicitly defer each canonical table/view.
8. T219-T230 generate and diff declared outputs only.
9. T231-T233 verify generated runtime behavior.
10. T234 records table and generated-file checksums in the release manifest.

## T027 Acceptance

This design satisfies T027 when:

- Every required environment class is represented by a canonical table/view.
- Existing catalog tables are reused or extended, not bypassed.
- New table classes name their current base rows or parser source.
- The file-to-table-to-generated-file ownership boundary is explicit.
- Validation, conflict, secret-reference, generated-file, and deferral rules are defined.
