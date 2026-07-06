# CodeDB Upload Inventory

This report is generated from the live `codedb_file_imports` catalog rows.

## Summary

- total targets: `3549`
- `content_blob` targets: `1909`
- `metadata_only` targets: `1640`
- `blob_metadata_ready` rows: `1866`
- `structured_rows_ready` rows: `1397`
- `structured_status = metadata_only` rows: `1640`

## Import Modes

- `content_blob`: `1909`
- `metadata_only`: `1640`

## Parser Hints

- `desktop_entry`: `3`
- `directory`: `177`
- `json`: `317`
- `jsonc`: `5`
- `jsonl`: `57`
- `kdl`: `15`
- `log`: `42`
- `lua`: `12`
- `markdown`: `868`
- `nix`: `30`
- `nu`: `24`
- `opaque`: `1518`
- `plain_or_binary`: `77`
- `plain_text`: `2`
- `scheme`: `5`
- `shell`: `107`
- `systemd_unit`: `3`
- `toml`: `235`
- `yaml`: `52`

## Import Safety Policies

- `generated_content_import_allowed`: `68`
- `nix_store_metadata_first`: `366`
- `real_home_metadata_first`: `7`
- `runtime_state_no_content_import`: `1267`
- `source_content_import_allowed`: `1841`
