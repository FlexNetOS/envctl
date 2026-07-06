---
id: 019f246e-3f8c-74d1-bd1e-fac84abd949b
slug: expand-codedb-nu-plugin-coverage-beyond-file-impor
title: "Expand CodeDB Nu plugin coverage beyond file import lists"
type: task
status: completed
priority: medium
---

## Summary

Used the `codedb-config-tables` skill to turn the original narrow request into a
complete tracked task set:

- `tasks/codedb-import-target-inventory`
- `tasks/codedb-content-blob-inventory`
- `tasks/codedb-metadata-only-inventory`
- `tasks/codedb-nu-plugin-semantic-coverage`

## Outcome

The generated inventories now have explicit KB records, and the broader Nu
plugin semantics are documented in-repo rather than left implicit in source.

## Deliverables

- `agent-skills/codedb-config-tables/`
- `docs/generated/codedb-upload-inventory.md`
- `docs/generated/codedb-semantic-coverage.md`
- the four completed task docs above

## Notes

This umbrella task is complete because the work was split into concrete
inventory and semantics tasks and each one now has committed KB content and
matching repository artifacts.
