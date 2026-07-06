# envctl Catalog Table Inventory

This report summarizes every normalized table currently present in the live catalog snapshot.

- repo root: `/home/flexnetos/FlexNetOS/src/envctl`
- manifest dir: `/home/flexnetos/FlexNetOS/src/envctl/manifest`
- tables: `11`
- total rows: `10203`

| table | rows | columns | purpose |
| --- | ---: | --- | --- |
| `components` | `97` | `component_id, description, destructive, gpu_required, has_detect, has_fix, has_install, has_remove, has_verify, lock_hash, name, requires, resolved_order, source_file, status` | component registry rows and lifecycle intent |
| `component_hooks` | `381` | `args, command, component_id, env, hook_kind, login_shell, needs_sudo, path, phase, script, source_file` | detect/install/fix/reset hook wiring |
| `paths` | `49` | `artifact_kind, bridge, canonical, legacy, owner_component, path, path_id, path_kind, protected, source, verification_status` | canonical, legacy, and bridged filesystem targets |
| `settings` | `4925` | `drift_status, expires_at, generated, generated_conflict_policy, manual_override, override_owner, override_reason, override_timestamp, owner_component, precedence, review_required, scope, sensitive, setting_key, source_file, source_kind, value` | normalized config/settings key-value rows |
| `env_vars` | `106` | `consumer, default_value, effective_value, generated_by, producer, scope, sensitive, source, value, var_name` | environment variables with producer and scope metadata |
| `agent_assets` | `44` | `asset_kind, destination, drift_status, hash, lock_status, name, source, source_revision` | skills, agents, hooks, and lock-tracked assets |
| `registries` | `9` | `component_id, drift_status, entry_id, name, registry_kind, source_file, status, tier` | hub and MCP registry entries |
| `config_files` | `344` | `config_id, drift_status, exists, file_kind, format, generated, lock_hash, manual_override, owner_component, parse_status, path, read_status, source_role` | source and generated config file inventory |
| `codedb_file_imports` | `3549` | `absolute_path, blob_ref, byte_length, content_hash, file_kind, import_mode, import_safety_policy, import_status, last_observed, logical_owner, normalized_path, parser_hint, provenance, reproduction_policy, skip_reason, source_of_truth_class, structured_row_count, structured_rows, structured_status, structured_table, table, target_id` | blob/structured import rows for file-backed code DB coverage |
| `migration_evidence` | `0` | `` | adoption and purge-safety evidence |
| `observed_facts` | `699` | `fact_id, fact_kind, observed_at, source, status, subject_id, subject_kind, value, verifier` | runtime observations and verifier-produced facts |
