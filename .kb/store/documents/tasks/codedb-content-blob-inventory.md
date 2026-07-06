---
id: 019f2470-c9ac-7b72-961d-b36d19f602fc
slug: tasks/codedb-content-blob-inventory
title: "Capture CodeDB content blob inventory"
type: task
status: completed
priority: medium
---

## Summary

Captured the exact subset of inventory targets whose bytes are uploaded as
content blobs rather than metadata-only rows.

## Deliverables

- `docs/generated/codedb-content-blob-targets.txt`
- `docs/generated/codedb-upload-inventory.md`
- `scripts/export-codedb-upload-lists.sh`

## Evidence

- The content-blob list contains `1909` paths:
  - `wc -l docs/generated/codedb-content-blob-targets.txt`
- The source manifest contains the same `1909` `content_blob` rows:
  - `jq -r '[.[] | select(.import_mode=="content_blob")] | length' /home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json`
- The Nu plugin hashes readable content-blob targets and emits:
  - `content_hash`
  - `blob_ref`
  - `import_status = "blob_metadata_ready"`
  - `structured_status = "structured_rows_ready"` when decoding succeeds
- Proof surfaces:
  - `../nu_plugin/crates/nu_plugin_codedb/src/main.rs`
  - `crates/cli/tests/cli_contract.rs`

## Notes

This task only tracks the byte-upload subset. It does not claim to cover the
metadata-only targets or the wider CodeDB table families.
