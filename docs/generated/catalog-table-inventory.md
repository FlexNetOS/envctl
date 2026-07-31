# envctl Catalog Table Inventory

Generated from catalog snapshot `cd5a260942e115518a799f6a4f67a461f96199b6420a3047b8779d2841a1863b`.

| table | rows | columns |
| --- | ---: | --- |
| `components` | `87` | `component_id, description, destructive, gpu_required, has_detect, has_fix, has_install, has_remove, has_verify, lock_hash, name, requires, resolved_order, source_file, status` |
| `nix_components` | `9` | `component_id, frontdoor_paths, lock_hash, name, nix_surface, original_url, owner_component, profile_entry, profile_url, requires, resolved_order, source_file, status, store_paths` |
| `component_hooks` | `339` | `args, command, component_id, env, hook_kind, login_shell, needs_sudo, path, phase, script, source_file` |
| `paths` | `74` | `artifact_kind, bridge, canonical, legacy, link_target_id, owner_component, owner_record_id, path, path_id, path_kind, protected, resolved_path, source, verification_status` |
| `settings` | `5259` | `drift_status, expires_at, generated, generated_conflict_policy, manual_override, override_owner, override_reason, override_timestamp, owner_component, precedence, review_required, scope, sensitive, setting_key, source_file, source_kind, value` |
| `env_vars` | `104` | `consumer, default_value, effective_value, generated_by, producer, scope, sensitive, source, value, var_name` |
| `agent_assets` | `7` | `asset_kind, destination, drift_status, hash, lock_status, name, source, source_revision` |
| `registries` | `4` | `component_id, drift_status, entry_id, name, registry_kind, source_file, status, tier` |
| `config_files` | `383` | `config_id, drift_status, exists, file_kind, format, generated, lock_hash, manual_override, owner_component, parse_status, path, read_status, source_role` |
| `codedb_file_imports` | `0` | `` |
| `migration_evidence` | `0` | `` |
| `observed_facts` | `778` | `fact_id, fact_kind, observed_at, source, status, subject_id, subject_kind, value, verifier` |
