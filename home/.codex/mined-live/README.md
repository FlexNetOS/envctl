# Mined live ~/.codex extraction policy

Source: /home/flexnetos/.codex
Generated UTC: 2026-07-07T20:46:56Z

## Active projection copies

- AGENTS.md, RULES.md, RTK.md, AGENTS.rtk.md
- config.toml and envctl-*.config.toml profile files
- model catalog/cache JSON and version metadata
- agents/*.toml, prompts/, tools/, skills/.system/

## Inactive mined snapshots and extracted inventories

- live rules/default.rules is stored under mined-live/rules/default.rules and is not activated from home/.codex/rules/ to avoid widening policy accidentally.
- config-backups, raw memories, sessions, history, sqlite stores, attachments, cache, and plugin cache are inventory-extracted only to avoid committing private runtime or secrets.
- per-domain inventory TSV files are stored under mined-live/.

See mined-live/inventory.tsv for the full pass inventory.
