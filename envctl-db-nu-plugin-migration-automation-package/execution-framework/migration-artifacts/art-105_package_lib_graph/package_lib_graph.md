# Package/library dependency graph

Task: `ART-105_PACKAGE_LIB_GRAPH`
Generated: `2026-07-04T23:20:41+00:00`

## Summary

- Nodes: 8
- Edges: 6
- Python files scanned: 49
- Third-party Python imports: 2
- Shell scripts scanned: 6
- Issues: 4

## Dependency graph

| from | relation | to | evidence |
|---|---|---|---|
| `package:envctl-db-nu-plugin-migration-automation-package` | `bundles_source_package` | `package:codex-flexnetos-migration-prompt-package` | `source/codex-flexnetos-migration-prompt-package/PACKAGE_MANIFEST.json` |
| `package:envctl-db-nu-plugin-migration-automation-package` | `uses_runtime_libraries` | `runtime:python-stdlib` | `execution-framework/scripts/*.py`, `helpers/*.py` |
| `package:envctl-db-nu-plugin-migration-automation-package` | `persists_registry_state` | `runtime:sqlite` | `sql/001_migration_automation_schema.sql`, `sql/002_views_and_indexes.sql`, `sql/003_seed_artifact_contract.sql`, `execution-framework/generated/contract_manifest.seed.sql` |
| `package:envctl-db-nu-plugin-migration-automation-package` | `executed_by` | `tool:codex-cli` | `generated/execution_packets/ART-105_PACKAGE_LIB_GRAPH.json` |
| `contract:artifact-contract` | `validated_by` | `schema:json-schema` | `schemas/artifact_record.schema.json`, `schemas/migration_recipe.schema.json`, `schemas/operation.schema.json`, `schemas/run_event.schema.json`, `schemas/target_descriptor.schema.json`, `schemas/validation_result.schema.json`, `execution-framework/schemas/shared_protocol.schema.json`, `execution-framework/schemas/proof_record.schema.json` |
| `contract:shared-protocol` | `materialized_as` | `schema:json-schema` | `execution-framework/schemas/shared_protocol.schema.json` |

## Components

| id | kind | evidence |
|---|---|---|
| `package:envctl-db-nu-plugin-migration-automation-package` | `package` | `PACKAGE_MANIFEST.json`, `execution-framework/generated/package_scan.json` |
| `package:codex-flexnetos-migration-prompt-package` | `source_package` | `source/codex-flexnetos-migration-prompt-package/PACKAGE_MANIFEST.json` |
| `runtime:python-stdlib` | `runtime_library` | `execution-framework/scripts/*.py`, `helpers/*.py` |
| `runtime:sqlite` | `embedded_database` | `sql/001_migration_automation_schema.sql`, `sql/002_views_and_indexes.sql`, `sql/003_seed_artifact_contract.sql`, `execution-framework/generated/contract_manifest.seed.sql` |
| `tool:codex-cli` | `tool` | `RUN_WITH_CODEX_ENVCTL.sh`, `codex/envctl-nu-plugin-migration.config.toml` |
| `schema:json-schema` | `schema_contract` | `schemas/artifact_record.schema.json`, `schemas/migration_recipe.schema.json`, `schemas/operation.schema.json`, `schemas/run_event.schema.json`, `schemas/target_descriptor.schema.json`, `schemas/validation_result.schema.json`, `execution-framework/schemas/shared_protocol.schema.json`, `execution-framework/schemas/proof_record.schema.json` |
| `contract:shared-protocol` | `protocol_contract` | `execution-framework/generated/shared_protocol_manifest.json` |
| `contract:artifact-contract` | `artifact_contract` | `execution-framework/generated/contract_manifest.json` |

## Vulnerabilities, deprecations, incompatibilities

| id | category | severity | status | finding |
|---|---|---|---|---|
| `ART105-VULN-001` | vulnerability | unknown | `no_external_lockfile_detected` | The package contains no third-party package lockfile or version-pinned library manifest for CVE correlation; observed external Python imports are recorded without versions, so CVE status is unknown rather than clean. |
| `ART105-DEP-001` | deprecation | medium | `present_in_source_manifest` | The source package manifest includes CPython 3.13 bytecode cache files; treat them as generated compatibility artifacts, not authoritative migration source. |
| `ART105-INCOMPAT-001` | incompatibility | medium | `requires_sqlite_features` | Artifact registry verification depends on SQLite foreign keys, views, unique constraints, and upsert semantics; alternate DB backends need compatible migrations before replay. |
| `ART105-INCOMPAT-002` | incompatibility | medium | `tool_required` | Execution packets require codex, python3, and shell tools. The package does not vendor these tools, so target hosts must provide them through the envctl runtime. |

## Python import scan

| file | local imports | stdlib imports | third-party imports |
|---|---|---|---|
| `execution-framework/scripts/_common.py` |  | `__future__`, `csv`, `datetime`, `hashlib`, `json`, `os`, `pathlib`, `sys`, `time` |  |
| `execution-framework/scripts/artifact_registry.py` |  | `__future__`, `dataclasses`, `fnmatch`, `hashlib`, `json`, `pathlib`, `re`, `sqlite3`, `typing` |  |
| `execution-framework/scripts/build_art126_decision_log.py` | `_common`, `artifact_registry`, `verify_envctl_db_schema` | `__future__`, `json`, `pathlib`, `sqlite3`, `typing` |  |
| `execution-framework/scripts/envctl_run_ledger.py` | `_common` | `__future__`, `dataclasses`, `hashlib`, `json`, `pathlib`, `sqlite3`, `typing` |  |
| `execution-framework/scripts/generate_art102_repository_map.py` | `_common`, `artifact_registry`, `verify_envctl_db_schema` | `__future__`, `collections`, `json`, `os`, `pathlib`, `sqlite3`, `subprocess`, `typing` |  |
| `execution-framework/scripts/generate_art104_toolchain_tree.py` | `_common`, `artifact_registry`, `verify_envctl_db_schema` | `__future__`, `collections`, `fnmatch`, `json`, `os`, `pathlib`, `sqlite3`, `subprocess`, `tomllib`, `typing` |  |
| `execution-framework/scripts/generate_art112_code_ownership.py` | `_common`, `artifact_registry`, `verify_envctl_db_schema` | `__future__`, `hashlib`, `json`, `os`, `pathlib`, `sqlite3`, `typing` |  |
| `execution-framework/scripts/generate_art113_debug_code_map.py` | `_common`, `artifact_registry`, `verify_envctl_db_schema` | `__future__`, `collections`, `json`, `os`, `pathlib`, `re`, `sqlite3`, `typing` |  |
| `execution-framework/scripts/generate_art116_infra_topology.py` | `_common`, `artifact_registry`, `verify_envctl_db_schema` | `__future__`, `datetime`, `fnmatch`, `hashlib`, `json`, `os`, `pathlib`, `re`, `sqlite3`, `typing` |  |
| `execution-framework/scripts/generate_art125_risk_register.py` | `_common`, `artifact_registry`, `status_from_proofs`, `verify_envctl_db_schema` | `__future__`, `json`, `pathlib`, `sqlite3`, `typing` |  |
| `execution-framework/scripts/generate_art128_readiness_scorecard.py` | `_common`, `artifact_registry`, `verify_envctl_db_schema` | `__future__`, `hashlib`, `json`, `pathlib`, `sqlite3`, `typing` |  |
| `execution-framework/scripts/generate_art_117_iam_matrix.py` | `_common`, `artifact_registry`, `verify_envctl_db_schema` | `__future__`, `fnmatch`, `json`, `pathlib`, `re`, `sqlite3`, `typing` |  |
| `execution-framework/scripts/generate_art_127_blast_radius.py` | `_common`, `artifact_registry`, `verify_envctl_db_schema` | `__future__`, `json`, `pathlib`, `sqlite3`, `typing` |  |
| `execution-framework/scripts/generate_art_129_business_capability.py` | `_common`, `artifact_registry`, `verify_envctl_db_schema` | `__future__`, `json`, `pathlib`, `sqlite3`, `typing` |  |
| `execution-framework/scripts/generate_config_inventory.py` | `_common`, `artifact_registry`, `verify_envctl_db_schema` | `__future__`, `collections`, `fnmatch`, `hashlib`, `json`, `os`, `pathlib`, `re`, `sqlite3`, `typing` |  |
| `execution-framework/scripts/generate_directory_tree_artifact.py` | `_common`, `artifact_registry`, `verify_envctl_db_schema` | `__future__`, `collections`, `fnmatch`, `hashlib`, `json`, `os`, `pathlib`, `sqlite3`, `subprocess`, `typing` |  |
| `execution-framework/scripts/generate_env_config_matrix.py` | `_common`, `artifact_registry`, `verify_envctl_db_schema` | `__future__`, `json`, `pathlib`, `sqlite3`, `typing` |  |
| `execution-framework/scripts/generate_package_lib_graph.py` | `_common`, `artifact_registry`, `verify_envctl_db_schema` | `__future__`, `ast`, `json`, `pathlib`, `re`, `sqlite3`, `sys`, `typing` |  |
| `execution-framework/scripts/generate_system_inventory.py` | `_common`, `artifact_registry`, `verify_envctl_db_schema` | `__future__`, `collections`, `json`, `os`, `pathlib`, `re`, `sqlite3`, `tomllib`, `typing` |  |
| `execution-framework/scripts/goal_loop.py` | `_common` | `__future__`, `collections`, `json`, `pathlib`, `sys` |  |
| `execution-framework/scripts/goal_to_task_graph.py` | `_common` | `__future__`, `json`, `pathlib` |  |
| `execution-framework/scripts/lock_contract.py` | `_common` | `__future__`, `hashlib`, `json`, `pathlib`, `re` |  |
| `execution-framework/scripts/merge_proofs.py` | `_common` | `__future__` |  |
| `execution-framework/scripts/operation_state_machine.py` |  | `__future__`, `dataclasses`, `enum`, `typing` |  |
| `execution-framework/scripts/redaction_controls.py` | `_common` | `__future__`, `dataclasses`, `fnmatch`, `hashlib`, `json`, `pathlib`, `re`, `typing` |  |
| `execution-framework/scripts/render_live_visuals.py` | `_common` | `__future__`, `argparse`, `collections`, `pathlib` |  |
| `execution-framework/scripts/scan_package.py` | `_common` | `__future__`, `argparse`, `pathlib` |  |
| `execution-framework/scripts/status_from_proofs.py` | `_common` | `__future__` |  |
| `execution-framework/scripts/task_graph_to_packets.py` | `_common` | `__future__`, `json`, `pathlib`, `sys` |  |
| `execution-framework/scripts/validate_task_graph.py` | `_common` | `__future__`, `pathlib`, `sys` |  |
| `execution-framework/scripts/validation_evidence.py` |  | `__future__`, `dataclasses`, `fnmatch`, `hashlib`, `json`, `pathlib`, `re`, `sqlite3`, `typing` |  |
| `execution-framework/scripts/verify_artifact_registry.py` | `_common`, `artifact_registry`, `verify_envctl_db_schema` | `__future__`, `json`, `pathlib`, `sqlite3` |  |
| `execution-framework/scripts/verify_envctl_db_schema.py` | `_common` | `__future__`, `json`, `pathlib`, `sqlite3` |  |
| `execution-framework/scripts/verify_envctl_run_ledger.py` | `_common`, `envctl_run_ledger` | `__future__`, `json`, `pathlib`, `sqlite3` |  |
| `execution-framework/scripts/verify_filesystem_boundaries.py` | `_common` | `__future__`, `copy`, `fnmatch`, `json`, `posixpath` | `jsonschema` |
| `execution-framework/scripts/verify_flexnetos_target_descriptor.py` | `_common`, `verify_target_registry` | `__future__`, `json`, `pathlib`, `re`, `typing` |  |
| `execution-framework/scripts/verify_history_and_completeness.py` | `_common` | `__future__`, `json`, `pathlib` |  |
| `execution-framework/scripts/verify_install_bootstrap.py` | `_common` | `__future__`, `json`, `pathlib`, `shlex` |  |
| `execution-framework/scripts/verify_operation_state_machine.py` | `_common`, `operation_state_machine`, `verify_envctl_db_schema` | `__future__`, `json`, `pathlib`, `sqlite3`, `typing` |  |
| `execution-framework/scripts/verify_plugin_command_surface.py` |  | `__future__`, `json`, `pathlib`, `sys` |  |
| `execution-framework/scripts/verify_security_redaction.py` | `_common`, `redaction_controls` | `__future__`, `json`, `pathlib` |  |
| `execution-framework/scripts/verify_shared_protocol_schemas.py` | `_common` | `__future__`, `copy`, `json`, `pathlib` | `jsonschema` |
| `execution-framework/scripts/verify_target_registry.py` | `_common`, `verify_envctl_db_schema` | `__future__`, `copy`, `hashlib`, `json`, `pathlib`, `sqlite3`, `typing` | `yaml` |
| `execution-framework/scripts/verify_validation_evidence.py` | `_common`, `validation_evidence`, `verify_envctl_db_schema` | `__future__`, `json`, `sqlite3` |  |
| `helpers/build_combined_prompt.py` |  | `pathlib`, `sys` |  |
| `helpers/package_manifest.py` |  | `hashlib`, `json`, `pathlib`, `sys` |  |
| `helpers/validate_package.py` |  | `json`, `pathlib`, `sqlite3`, `subprocess`, `sys`, `tempfile` |  |
| `source/codex-flexnetos-migration-prompt-package/helpers/artifact_manifest.py` |  | `datetime`, `hashlib`, `json`, `pathlib`, `sys` |  |
| `source/codex-flexnetos-migration-prompt-package/helpers/make_wiki_index.py` |  | `datetime`, `pathlib`, `sys` |  |

## Shell tool scan

| script | tools |
|---|---|
| `INSTALL_IN_REPOS.sh` | `bash`, `codex`, `cp`, `mkdir` |
| `RUN_WITH_CODEX_ENVCTL.sh` | `bash`, `codex`, `date` |
| `helpers/copy_to_repos.sh` | `bash` |
| `source/codex-flexnetos-migration-prompt-package/INSTALL_IN_REPO.sh` | `bash`, `codex`, `cp`, `date`, `mkdir` |
| `source/codex-flexnetos-migration-prompt-package/RUN_WITH_CODEX.sh` | `bash`, `codex`, `mkdir` |
| `source/codex-flexnetos-migration-prompt-package/helpers/background_scan.sh` | `bash`, `date`, `find`, `git`, `mkdir`, `python`, `python3`, `sed`, `sha256sum` |
