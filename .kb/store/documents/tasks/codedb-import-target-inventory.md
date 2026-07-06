---
id: 019f2470-b002-7dc1-aa15-a5d016a3adf5
slug: tasks/codedb-import-target-inventory
title: "Capture CodeDB import target inventory"
type: task
status: completed
priority: medium
---

## Summary

Captured the authoritative plain-text inventory for every Yazelix file target that
becomes an envctl CodeDB import row.

## Deliverables

- `docs/generated/codedb-import-targets.txt`
- `docs/generated/codedb-upload-inventory.md`
- `scripts/export-codedb-upload-lists.sh`

## Evidence

- The generated list contains `3549` paths:
  - `wc -l docs/generated/codedb-import-targets.txt`
- The source manifest contains the same `3549` rows:
  - `jq -r 'length' /home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json`
- envctl ingests this surface through `codedb_file_imports`:
  - `crates/engine/src/catalog.rs` maps `codedb_file_imports` and
    `envctl_yazelix_file_import`
  - `scan()` calls `ingest_codedb_file_imports()`

## Notes

This task is intentionally about the full import-row target set, not just blob
uploads. The broader Nu plugin semantics are tracked separately in
`tasks/codedb-nu-plugin-semantic-coverage`.
