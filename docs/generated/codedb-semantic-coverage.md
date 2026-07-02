# CodeDB Semantic Coverage

Date: 2026-07-02
Scope: `envctl` CodeDB catalog ingestion plus `nu_plugin_codedb` table surfaces

## Why this exists

The three generated inventory lists are useful, but they only answer one narrow
question: which file targets become `envctl_yazelix_file_import` rows, and which
of those rows upload bytes as blobs. The CodeDB Nu plugin contract is broader
than that.

This document captures the semantic/table layer that sits on top of those file
lists so we do not mistake "inventory exported" for "plugin surface covered."

## Inventory rows are semantic rows, not just path lists

The Yazelix inventory path is ingested by envctl through
`ingest_codedb_file_imports()` in `crates/engine/src/catalog.rs`. The engine
maps this data into `envctl_yazelix_file_import` rows and records the companion
structured-row table name `envctl_yazelix_file_structured_rows`.

Each import row carries more than a path:

- `target_id`
- `logical_owner`
- `absolute_path`
- `normalized_path`
- `source_of_truth_class`
- `file_kind`
- `parser_hint`
- `content_hash`
- `byte_length`
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

That means the import lists are only one projection of a richer table contract.

## Blob and metadata discipline

The Nu plugin follows the same semantics the engine uses:

1. `content_blob` plus a readable regular file:
   - hashes bytes
   - emits `content_hash`
   - emits `blob_ref`
   - marks `import_status = "blob_metadata_ready"`
2. `content_blob` plus a non-regular or unreadable target:
   - falls back to `import_status = "metadata_only"`
   - preserves an explicit `skip_reason`
3. `metadata_only` import mode:
   - does not upload file bytes
   - preserves policy as `skip_reason`
   - keeps blob fields empty

This distinction is covered in both the engine and the plugin tests.

## Structured rows are a separate layer

When bytes are present and the parser can decode the content, the plugin and the
engine expose `structured_rows` and mark
`structured_status = "structured_rows_ready"`.

When bytes exist but no decoder produces rows, the state is
`structured_status = "unstructured_blob"`.

When the target is policy-limited metadata only, the state is
`structured_status = "metadata_only"`.

So "blob uploaded" and "semantic rows extracted" are related but distinct.

## The Nu plugin exposes broader table families

The plugin command surface includes:

- `codedb rust items`
- `codedb rust macros`
- `codedb rust cfg`
- `codedb build scripts`
- `codedb tables`
- `codedb gaps`
- `codedb validation errors`
- `codedb schema`
- `codedb doctor`

That matters because the user request explicitly called out "all config,
settings, environments, and files" and noted that the plugin is "way more than
blobs, metadata, and files." The plugin itself agrees: it exposes compiler-like
facts, table inventory, validation failures, and capture gaps, not just import
rows.

## Gap and validation surfaces

Two explicit incompleteness/reporting commands are part of the real contract:

- `codedb gaps`
- `codedb validation errors`

Any future claim that the repo has fully covered a CodeDB capture needs to check
these surfaces, not just confirm that a file list exists.

## Practical interpretation for this repo

- `docs/generated/codedb-import-targets.txt`
  - answers: which targets become import rows
- `docs/generated/codedb-content-blob-targets.txt`
  - answers: which targets upload bytes as blobs
- `docs/generated/codedb-metadata-only-targets.txt`
  - answers: which targets remain metadata-only rows
- this document
  - answers: what semantic/table contract those rows participate in, and which
    broader CodeDB surfaces still matter for truthful coverage
