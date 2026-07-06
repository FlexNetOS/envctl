---
name: codedb-ingest
description: Use CodeDB and nu_plugin_codedb to transform repositories, configs, settings, environment snapshots, Yazelix/runtime files, and source trees into database-shaped tables with filesystem rows, source rows, Cargo/Rust semantics, blob metadata, structured config rows, capture gaps, validation errors, and export checksums. Use when asked to ingest files into CodeDB, convert files/config/env/settings to tables, validate CodeDB/Yazelix/Nushell table coverage, or prepare envctl-visible CodeDB exports.
---

# CodeDB Ingest

## Overview

Use this skill to drive the real CodeDB/Nushell path rather than ad hoc file parsing. CodeDB owns file-to-table conversion, blob policy, source inventory, Rust/Cargo semantics, capture gaps, validation errors, and export checksums; envctl consumes those rows as a downstream materializer.

Default CodeDB repo: `/home/flexnetos/FlexNetOS/src/nu_plugin`.

## Workflow

1. Confirm the CodeDB surface exists and identify the current target roots.
   - For repo semantics, the target is usually the active repo checkout.
   - For config/settings/environment/runtime files, collect explicit roots such as the repo, Yazelix source, generated Yazelix state, `$META_ROOT/etc`, `$META_ROOT/usr/share`, or other requested config roots.

2. Load `references/commands.md` before executing CodeDB or Nu plugin commands.
   - Prefer the transient `nu --plugins <nu_plugin_codedb>` pattern from the Yazelix tests so real user Nushell plugin registries are not mutated.
   - Run `codedb doctor` and a plugin `codedb tables` smoke before trusting exports.

3. For repository semantics, use CodeDB table commands.
   - Capture `codedb scan`, `codedb fs entries`, `codedb source files`, Cargo tables, Rust static tables, macro/build rows, `codedb gaps`, `codedb validation errors`, and schema/table inventory.
   - Treat gaps and validation errors as first-class rows, not failures to hide.

4. For configs, settings, environments, and arbitrary files, build a Yazelix-style inventory artifact and import it through the Nu plugin command:
   - `codedb envctl import inventory <inventory.json>`
   - Load `references/inventory-contract.md` for required fields, parser hints, blob policy, and safety modes.

5. Check breadth with `references/table-coverage.md`.
   - The run is incomplete until it accounts for filesystem/source rows, Cargo/Rust semantic rows, blob refs/policies, structured config rows, capture gaps, validation errors, proof/export rows, and runtime integration rows or explicit current limitations.

6. Preserve safety boundaries.
   - Do not read CodeDB redb internals.
   - Do not emit raw secret-like values.
   - Do not write tracked Yazelix Nushell config to register a plugin; use transient plugin loading or generated runtime bridge artifacts.
   - Use metadata-only inventory rows for runtime state, secrets, binaries, sockets, package outputs, and anything whose content policy is unclear.

## References

- `references/commands.md`: current CLI/Nu plugin command patterns, including Yazelix transient plugin smoke flow.
- `references/inventory-contract.md`: inventory JSON schema for turning config/settings/env/runtime files into `envctl_yazelix_file_import` rows.
- `references/table-coverage.md`: required table families, blob/semantic coverage checklist, and evidence to report.
