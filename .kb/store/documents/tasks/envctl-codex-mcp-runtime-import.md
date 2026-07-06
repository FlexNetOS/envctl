---
id: 019f24bb-6871-72d1-ad69-921ebc284482
slug: tasks/envctl-codex-mcp-runtime-import
title: "Import Codex MCP runtime config through envctl and nu_plugin"
type: task
status: active
priority: medium
---

## Overview

Move the recent ad hoc Codex runtime-config repair into envctl's real source of truth. The immediate trigger was a local fix that suppressed the unstable-features warning and repointed MCP servers away from bare `node` toward the workspace-managed Bun runtime, but that repair currently lives only in generated user state.

This task captures the envctl-side upgrade path: use the `nu_plugin` `/nu_plugin:import` workflow to import the relevant Codex/MCP/config files into the envctl catalog tables, so envctl can own the declarative rows and later render the generated runtime config from those rows instead of relying on hand edits in `~/.codex/config.toml`.

## Goals

- Import the Codex/MCP/runtime-config surfaces that currently matter for this repair into envctl-controlled catalog rows.
- Use the `nu_plugin` import workflow as the frontdoor for bringing those files into CodeDB/envctl tables.
- Record which files are source-owned, which are generated runtime state, and which settings should be table-owned going forward.
- Make the Bun-backed MCP runtime wiring and unstable-feature suppression reproducible from envctl rather than by editing generated user config directly.

## Implementation

Start from the current local repair evidence and import the smallest authoritative file set first:

- `/home/flexnetos/.codex/config.toml`
- `/home/flexnetos/FlexNetOS/.codex/config.toml`
- relevant plugin MCP descriptors under:
  - `/home/flexnetos/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/.mcp.json`
  - `/home/flexnetos/.codex/plugins/cache/meta-plugins-codex/openai-developers/1.2.3/.mcp.json`
- source-owned plugin or marketplace config in:
  - `src/meta-plugins/`
  - `src/meta/codex-plugins/`

Use the envctl catalog read-only surfaces first (`catalog scan`, `catalog table`, `catalog import`) and route the actual file import work through the `nu_plugin` import skill so the table design, provenance rows, and future render path stay Rust-first and evidence-backed.

## Acceptance Criteria

- [ ] A documented envctl task exists for importing Codex/MCP/runtime-config files through the `nu_plugin` import workflow.
- [ ] The task names the concrete files/surfaces to import and distinguishes source-owned files from generated runtime config.
- [ ] The task states that Bun-backed MCP command rows and unstable-feature suppression should become envctl-owned table data rather than ad hoc user-config edits.
- [ ] The task references the existing CodeDB/Nu plugin inventory work so this import slice lands in the established workflow instead of becoming a parallel one-off.

## References

- [[expand-codedb-nu-plugin-coverage-beyond-file-impor]] — umbrella CodeDB/Nu plugin import coverage task
- [[tasks/codedb-import-target-inventory]] — prior import target inventory
- [[tasks/codedb-nu-plugin-semantic-coverage]] — broader Nu plugin semantics coverage
