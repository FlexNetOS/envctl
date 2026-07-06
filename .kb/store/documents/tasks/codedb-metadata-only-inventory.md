---
id: 019f2470-c9bd-7e00-bcdd-2340ba863a03
slug: tasks/codedb-metadata-only-inventory
title: "Capture CodeDB metadata-only inventory"
type: task
status: completed
priority: medium
---

## Summary

Captured the exact subset of inventory targets that become CodeDB rows without a
blob-byte upload.

## Deliverables

- `docs/generated/codedb-metadata-only-targets.txt`
- `docs/generated/codedb-upload-inventory.md`
- `scripts/export-codedb-upload-lists.sh`

## Evidence

- The metadata-only list contains `1640` paths:
  - `wc -l docs/generated/codedb-metadata-only-targets.txt`
- The source manifest contains the same `1640` `metadata_only` rows:
  - `jq -r '[.[] | select(.import_mode=="metadata_only")] | length' /home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json`
- The engine and plugin preserve metadata-only state explicitly via:
  - `import_status = "metadata_only"`
  - empty `content_hash` / `blob_ref`
  - `structured_status = "metadata_only"`
  - a concrete `skip_reason` / safety policy
- Proof surfaces:
  - `crates/engine/src/catalog.rs`
  - `../nu_plugin/crates/nu_plugin_codedb/src/main.rs`
  - `crates/cli/tests/cli_contract.rs`

## Notes

These rows still become database rows. They are not missing data; they are
explicitly represented as metadata-only according to the import policy.
