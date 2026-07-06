---
id: 019f2588-5a5e-7420-b390-22040e0af7fd
slug: tasks/catalog-runtime-closure-and-toolchain-proof
title: "Close Catalog Runtime and Toolchain Proof Gaps"
type: task
status: active
priority: high
tags: [rust, catalog, codedb, nu-plugin, toolchain, runtime, verification]
---

## Overview

The recent CodeDB and `nu_plugin` work proved a large part of the intended table model: envctl can analyze the catalog, flatten structured config into rows, and render producible artifacts from those rows. A fresh `envctl catalog analyze --json` run on 2026-07-02 reported 11 tables, 10,351 rows, 346 config files, 105 environment variables, 1,169 toolchain signals, and 3,549 CodeDB import rows; `envctl catalog render` emitted a proof bundle with table dumps plus generated inventories for config files, environment variables, toolchain signals, and CodeDB semantic coverage.

That is real progress, but it is not runtime closure yet. The live audit also surfaced hard gaps between the table story and the runtime story: `migration_evidence` is still empty, every `paths` row is `not_checked`, the generated toolchain evidence is noisy rather than decisive, the env-var inventory mixes schema/layout rows with many unresolved runtime values, and the current Yazelix-managed runtime does not actually expose `cargo`, `rustc`, or `rustup` on PATH. The GUI binary can be launched, but this audit did not prove a stable visible app window.

This task is the canonical follow-on for closing those proof gaps. It complements [[tasks/envctl-codex-mcp-runtime-import]]: that task owns bringing Codex/MCP config into envctl-controlled tables, while this one owns making the broader catalog/runtime/database story verifiable end to end.

## Goals

- Preserve the proven parts of the catalog surface with explicit evidence-backed acceptance criteria.
- Close the gap between table-owned intent and observed runtime reality for toolchains, environment variables, paths, and launch surfaces.
- Make the "database-backed config can render producible artifacts" claim queryable and testable rather than aspirational.
- Ensure envctl can distinguish modeled rows from observed/effective runtime facts so operators can trust what the catalog is telling them.

## Implementation

Use the fresh audit evidence as the baseline and drive the follow-up work in these slices:

1. **Migration evidence and provenance closure**
   - Define what qualifies as `migration_evidence` for config/settings/env/runtime imports and start writing those rows during catalog/import flows.
   - Ensure the rendered proof bundle can point back to concrete provenance rows instead of only generated markdown summaries.

2. **Path verification closure**
   - Replace the current all-`not_checked` state in `paths` with actual verification logic for existence, type, ownership, writability, symlink targets, or an explicit fail-closed reason when a path cannot be proven.
   - Surface those results in machine-readable table output and generated operator-facing docs.

3. **Toolchain runtime proof**
   - Reconcile the manifest/component intent with the live Yazelix runtime so the expected Rust toolchain surface is actually present.
   - Prove the intended runtime frontdoor for `cargo`, `rustc`, `rustup`, nightly selection, linker wiring, and cache wrapper wiring from the envctl/Yazelix-managed environment.
   - Make `$META_ROOT/.cargo/config.toml` and related toolchain settings part of the observable proof surface when they are expected to exist.

4. **Environment-variable semantics**
   - Distinguish declared/schema/layout env vars from observed/effective runtime env vars.
   - Make unset, missing, defaulted, generated, and sensitive states first-class and queryable.
   - Ensure the env table can answer concrete operator questions like "which Rust toolchain vars are effective right now?" without depending on broad text search.

5. **Signal quality and database ergonomics**
   - Refine `toolchain_signals` so it is not dominated by generic CodeDB import rows when the operator is asking for crisp toolchain facts.
   - Add narrower query surfaces or derived summaries for high-signal runtime answers.

6. **App launch and runtime verification**
   - Add an envctl-managed runtime verification path for the CLI and GUI surfaces that can prove whether the app launched successfully, remained resident, and opened a visible window when a display server is available.
   - If a launch cannot be proven, record the blocker in catalog/runtime evidence rather than silently treating "process executed" as success.

7. **Artifact reproducibility**
   - Ensure the catalog-rendered artifacts can be regenerated deterministically from current rows and validated against the intended source surfaces.
   - Keep the generated docs useful, but treat them as derived evidence, not the only proof layer.

## Acceptance Criteria

- [ ] `envctl catalog analyze --json` reports non-zero `migration_evidence` rows for the intended imported/configured surfaces, with provenance that links generated artifacts back to table-owned evidence.
- [ ] `paths` verification no longer reports every row as `not_checked`; each row is either proven with a concrete status or fails closed with an explicit reason.
- [ ] The envctl/Yazelix-managed runtime can prove the effective Rust toolchain surface (`cargo`, `rustc`, `rustup`, nightly selection, linker/cache wiring) from the actual runtime environment rather than from manifest intent alone.
- [ ] The environment-variable inventory distinguishes declared rows from observed/effective rows and can answer concrete Rust/toolchain/runtime questions directly.
- [ ] Toolchain-signal output is queryable at a high-signal level and is not overwhelmed by generic import rows when proving runtime state.
- [ ] There is a reproducible runtime verification path for launching envctl surfaces, and the evidence clearly distinguishes "binary executed" from "app window proven".
- [ ] Generated proof artifacts remain renderable from the database-backed tables and are explicitly treated as derived outputs backed by table-owned evidence.

## Spec References

- [[tasks/envctl-codex-mcp-runtime-import]] — adjacent import slice for Codex/MCP runtime-config ownership
- [[expand-codedb-nu-plugin-coverage-beyond-file-impor]] — umbrella task for the wider CodeDB/Nu plugin expansion
- [[tasks/codedb-import-target-inventory]] — imported target inventory that seeded the current scan surface
- [[tasks/codedb-nu-plugin-semantic-coverage]] — semantic/blob/structured coverage that proved the table model is broad enough