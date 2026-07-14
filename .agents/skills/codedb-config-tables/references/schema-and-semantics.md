# Schema, Semantics, and Blob Discipline

CodeDB is not just a file inventory. It is a typed store with identity, semantics, blobs, and explicit incompleteness reporting.

## Identity context

Preserve the identity context when proving or explaining captures:

- `toolchain_id`
- `target_triple`
- `feature_set_hash`
- `cfg_hash`
- `cargo_lock_hash`
- `profile`
- `edition`

Do not flatten or discard these fields when reporting what a scan means.

## Table families

The schema spans multiple table families:

- core identity
- store/blob
- filesystem/source
- cargo
- rust static
- macro/build/native
- proof/artifact
- agent/export

When the user asks to transform "all config, settings, environments, and files," interpret that as covering both content rows and the related proof/blob rows needed to reproduce and validate them.

## Blob handling

Raw bytes remain first-class:

- raw source bytes
- token streams
- generated outputs
- raw proof logs

These are represented as blobs with hashes and blob-reference rows. Do not pretend table rows replace the exact bytes. Report both the semantic row layer and the blob-reference layer.

## Structured rows versus metadata-only capture

Some files can produce structured rows:

- JSON/JSONC can flatten into key/value paths
- TOML, KDL, Nu, YAML, desktop files, shell/config text can yield text-structured rows

Other files may remain:

- `metadata_only`
- `unstructured_blob`

That is acceptable when surfaced honestly. Never invent structure for files the parser did not actually decode.

## Gaps and validation failures

Missing or incomplete capture must become explicit artifacts:

- `capture_gaps`
- `validation_errors`

Do not silently omit missing semantics, proc-macro/runtime-only facts, or unsupported files.
