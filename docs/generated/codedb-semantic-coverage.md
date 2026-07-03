# CodeDB Semantic Coverage

This document summarizes the semantic surface envctl currently imports into `codedb_file_imports`.

## Summary

- import rows: `3549`
- blob-backed rows: `1866`
- metadata-only rows: `1683`
- flattened structured rows: `324281`

## Semantic Columns

- `logical_owner`
- `normalized_path`
- `source_of_truth_class`
- `file_kind`
- `parser_hint`
- `content_hash`
- `blob_ref`
- `import_safety_policy`
- `reproduction_policy`
- `import_mode`
- `import_status`
- `skip_reason`
- `structured_table`
- `structured_status`
- `structured_row_count`
- `structured_rows`
- `last_observed`
- `provenance`

## Structured Statuses

- `metadata_only`: `1640`
- `structured_rows_ready`: `1397`
- `unstructured_blob`: `512`

## Source-of-Truth Classes

- `envctl_control_surface`: `1039`
- `nix_store_package_output`: `366`
- `real_home_desktop_entry`: `2`
- `real_home_runtime_state`: `1335`
- `real_home_user_config`: `5`
- `repo_source`: `802`

## Structured Parser Hints

- `json`: `253`
- `jsonc`: `2`
- `kdl`: `15`
- `lua`: `12`
- `markdown`: `799`
- `nix`: `30`
- `shell`: `102`
- `toml`: `132`
- `yaml`: `52`
