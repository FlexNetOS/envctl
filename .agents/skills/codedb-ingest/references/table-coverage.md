# Table Coverage Checklist

Use this checklist before declaring a CodeDB ingestion run complete.

## Core Runtime Proof

- CodeDB repo and runtime tool paths identified.
- `codedb doctor --nu` checked.
- Yazelix bridge status checked when the task involves Yazelix.
- Transient `nu --plugins` smoke returned `codedb tables`.
- No mutation of the real Nushell plugin registry.

## Filesystem and Source

- `filesystem_entries` / `codedb fs entries`
- `source_files` / `codedb source files`
- Source root and file hashes when available.
- Non-Rust/config files represented through `envctl_yazelix_file_import` inventory rows.

## Cargo and Rust Semantics

- `cargo_packages`
- `cargo_dependencies`
- `cargo_sources`
- `rust_items`
- `rust_macros`
- `rust_cfg`
- `build_scripts`

If Cargo is unavailable or a repo is not a Cargo workspace, record that as a capture gap or explicit non-applicable evidence instead of inventing semantic rows.

## Blob and Structured Config Semantics

- `source_blobs`, `artifact_blobs`, `blob_refs`, and `blob_policies` accounted for where exported.
- Inventory rows with `content_blob` produce `content_hash` and `blob_ref`.
- Inventory rows with supported parser hints produce `structured_status = structured_rows_ready`.
- Metadata-only rows have clear `skip_reason` or `import_safety_policy`.
- Raw bytes are not substituted for table rows in reports; reports cite hashes, row counts, and policy.

## Gaps, Validation, and Safety

- `codedb gaps` captured.
- `codedb validation errors` captured.
- Secret-like values are not emitted in stdout, stderr, summaries, or artifacts.
- Unsafe capture modes are not used unless the user explicitly requests them and the policy permits it.
- Runtime logs and real-home state default to metadata-only.

## Export and Downstream Envctl Surface

- Export format chosen: JSON, NUON, or CSV.
- Table names, row counts, and checksums are recorded where available.
- Envctl is treated as a downstream consumer; do not read redb internals or rederive CodeDB facts inside envctl.
- Reports include:
  - target roots;
  - commands run;
  - table families covered;
  - inventory artifact path if one was created;
  - row counts and representative table names;
  - gaps/errors;
  - safety policies used;
  - any unavailable tooling or unsupported parser hints.
