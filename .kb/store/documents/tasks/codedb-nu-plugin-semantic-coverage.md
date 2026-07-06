---
id: 019f2470-c9c7-7f11-b645-3a98b40aee32
slug: tasks/codedb-nu-plugin-semantic-coverage
title: "Expand CodeDB Nu plugin semantic coverage"
type: task
status: completed
priority: medium
---

## Summary

Expanded the documented scope from "file/blob inventories" to the wider CodeDB
Nu plugin contract: semantic rows, structured-row extraction, blob references,
gap reporting, validation errors, and table inventory surfaces.

## Deliverables

- `docs/generated/codedb-semantic-coverage.md`
- `agent-skills/codedb-config-tables/SKILL.md`
- `agent-skills/codedb-config-tables/references/schema-and-semantics.md`

## Evidence

- The Nu plugin exposes more than inventory lists:
  - `codedb rust items`
  - `codedb rust macros`
  - `codedb rust cfg`
  - `codedb build scripts`
  - `codedb tables`
  - `codedb gaps`
  - `codedb validation errors`
  - `codedb schema`
  - `codedb doctor`
- Yazelix import rows also carry semantic columns beyond path/blob presence:
  - `logical_owner`, `normalized_path`, `source_of_truth_class`, `parser_hint`
  - `content_hash`, `blob_ref`, `import_status`, `skip_reason`
  - `structured_table`, `structured_status`, `structured_row_count`,
    `structured_rows`
- `scan()` reaches `ingest_codedb_file_imports()` in the engine call graph,
  confirming that the inventory rows are part of a larger catalog scan rather
  than a one-off export path.

## Notes

This closes the gap the user called out: the Nu plugin surface is broader than
files, blobs, and metadata flags alone, and the repo now carries a dedicated
artifact documenting that breadth.
