# Mined live ~/.codex extraction policy

Source: /home/flexnetos/.codex
Generated UTC: 2026-07-07T20:54:36Z

## Active projection copies

- AGENTS.md, RULES.md, RTK.md, AGENTS.rtk.md
- config.toml and envctl-*.config.toml profile files
- model catalog/cache JSON and version metadata
- agents/*.toml, prompts/, tools/, skills/.system/

## Private local copy

All non-log live ~/.codex entries, including private/sensitive state such as auth.json, attachments, state/goals/memory SQLite files, cache, plugin cache, config backups, and raw memories, are copied locally under:

```text
home/agent-env/private-codex-state/data/.codex/
```

That data directory is intentionally git-ignored. The tracked manifest is `private-copy-manifest.tsv`.

## Logs deferred to portable log plan

Log-like surfaces are not copied into private-codex-state. They are assigned to the researched portable log root in `home/agent-env/PORTABLE_CODEX_LOGS.md`:

```text
logs_2.sqlite*
history.jsonl
sessions/
shell_snapshots/
execution-reports/
```


## Inactive mined snapshots and extracted inventories

- live rules/default.rules is stored under mined-live/rules/default.rules and is not activated from home/.codex/rules/ to avoid widening policy accidentally.
- per-domain inventory TSV files remain under mined-live/ for proof and packaging planning.

See mined-live/inventory.tsv for the full pass inventory.
