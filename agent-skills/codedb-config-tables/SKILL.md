---
name: codedb-config-tables
description: Transform config files, settings trees, environment/runtime state, and file inventories into CodeDB tables through `codedb` and `nu_plugin_codedb`, with full semantic rows, blob references, structured-row extraction, and explicit gap/error reporting. Use when Codex needs to catalog or query repo configs, Yazelix-generated state, Nushell/plugin surfaces, or envctl-exported files as database-like tables rather than ad hoc text parsing.
---

# Codedb Config Tables

Transform file and environment surfaces into CodeDB rows with the real plugin/CLI contract, then prove the capture quality. Treat CodeDB as a typed store with semantics, blobs, and explicit incompleteness reporting, not as a loose “scan files and print JSON” helper.

Load [commands-and-surfaces.md](references/commands-and-surfaces.md), [schema-and-semantics.md](references/schema-and-semantics.md), and [yazelix-proof-patterns.md](references/yazelix-proof-patterns.md) before acting.

## Decide the capture lane

Choose the narrowest valid lane for the request:

1. Use generic scan/export lane when the user wants repo configs, manifests, dotfiles, or mixed file trees captured into tables.
2. Use Nu plugin lane when the user wants interactive tables/records/lists directly from Nushell.
3. Use Yazelix generated-bridge lane when the request mentions Yazelix runtime state, generated initializers, or existing Yazelix smoke patterns.
4. Use registry lane only when the user explicitly wants `plugin add` or persistent registration. Prefer temporary `HOME` and isolated `--plugin-config` for proofs.

## Capture workflow

Follow this sequence:

1. Identify the source surface:
   - config tree
   - settings/runtime directory
   - envctl-exported files
   - Yazelix-generated state
   - mixed repo path
2. Inspect schema support first with `codedb schema` or `codedb tables`.
3. Run the appropriate capture:
   - `codedb scan <path>`
   - Nu plugin table commands
   - `codedb generate-yazelix-bridge --out-dir <dir>` before Yazelix-mode launch
4. Immediately inspect quality surfaces:
   - `codedb tables`
   - `codedb gaps`
   - `codedb validation-errors`
5. Report both:
   - semantic/structured rows
   - blob/proof coverage and any metadata-only fallbacks
6. If a requested fact is absent, surface the gap or validation error explicitly instead of papering over it.

## Preserve semantic fidelity

Keep the full CodeDB shape intact:

- preserve identity context such as toolchain/profile/feature hashes when it matters
- distinguish semantic rows from raw blobs
- keep `content_hash`, `blob_ref`, `byte_length`, parser hints, and safety policies
- do not collapse `metadata_only`, `unstructured_blob`, `structured_rows_ready`, and gap/error states into one vague status
- prefer real table output over re-parsing CLI logs

When a file becomes structured rows, say how. When it stays blob-backed or metadata-only, say that plainly.

## Use the Yazelix proof pattern exactly

When proving a Yazelix flow:

- build `codedb` and `nu_plugin_codedb`
- create temporary `HOME` and XDG roots
- generate bridge artifacts into a Yazelix-like initializer directory
- set `IN_YAZELIX_SHELL`, `YAZELIX_RUNTIME_DIR`, `YAZELIX_CODEDB_BIN`, and `YAZELIX_CODEDB_PLUGIN_BIN`
- launch `nu` with isolated config/plugin state
- prove the bridge did not create or mutate a real plugin registry unless persistent registry mode was explicitly requested

Use `envctl_yazelix_file_import` and `envctl_yazelix_file_structured_rows` when the user wants runtime-owned config/settings/files represented as tables.

## Output expectations

A good result usually includes:

- what surface was scanned or loaded
- which CodeDB tables were populated
- which files became structured rows and which stayed blob-backed
- any capture gaps or validation errors
- whether Yazelix/Nu/plugin registry state stayed isolated
- exact commands or proof snippets needed for reruns

Never describe the capture as complete if `codedb gaps` or `codedb validation-errors` still show unresolved rows.
