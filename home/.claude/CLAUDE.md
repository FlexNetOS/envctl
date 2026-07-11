@RTK.md

# FlexNetOS user-global operating contract (lean pointers, no prose)

- **Laws:** `~/.claude/rules/laws.md` — the 8 operating laws, hook-enforced. Read once, obey always.
- **Git topology:** `~/.claude/rules/git-topology.md` — main/develop only, superset merges, worktree ritual.
- **Workspace boundaries:** `~/.claude/rules/flexnetos-boundaries.md` (FlexNetOS paths), `~/.claude/rules/rust-conventions.md` (prompt_hub paths).
- **Toolchain:** `~/.claude/rules/toolchain.md` — nix-profile only; cargo via fenix, bun/bunx for node, no ad-hoc global installs.
- **Harness operations** (team spawn/cleanup, kill switch, budget, recovery): invoke the `harness-ops` skill.
- **Source of truth for this file and everything in ~/.claude:** `lifeos/src/envctl/home/.claude/` (ADR-0006: real file in meta, symlink outside). Edit via envctl worktree on develop, never in place.
- **Runtime state:** ledger `$HARNESS_VAR/log/claude-harness/ledger.jsonl` (append-only), decisions `$HARNESS_VAR/lib/claude-harness/decisions/`, kill switch `/home/flexnetos/meta/src/envctl/home/bin/harness-halt.sh` (not on `PATH` — full path).
- Report in the terminal only. Show raw output for every completion claim.

(ICM mandate restored 2026-07-11: icm 0.10.57 ships in the foundation profile; the 2026-07-07 removal reason no longer holds.)

<!-- icm:start -->
## Persistent memory (ICM) — MANDATORY

This project uses [ICM](https://github.com/rtk-ai/icm) for persistent memory across sessions.
You MUST use it actively. Not optional.

### Recall (before starting work)
```bash
icm recall "query"                        # search memories
icm recall "query" -t "topic-name"        # filter by topic
icm recall-context "query" --limit 5      # formatted for prompt injection
```

### Store — MANDATORY triggers
You MUST call `icm store` when ANY of the following happens:
1. **Error resolved** → `icm store -t errors-resolved -c "description" -i high -k "keyword1,keyword2"`
2. **Architecture/design decision** → `icm store -t decisions-{project} -c "description" -i high`
3. **User preference discovered** → `icm store -t preferences -c "description" -i critical`
4. **Significant task completed** → `icm store -t context-{project} -c "summary of work done" -i high`
5. **Conversation exceeds ~20 tool calls without a store** → store a progress summary

Do this BEFORE responding to the user. Not after. Not later. Immediately.

Do NOT store: trivial details, info already in CLAUDE.md, ephemeral state (build logs, git status).

### Other commands
```bash
icm update <id> -c "updated content"     # edit memory in-place
icm health                                # topic hygiene audit
icm topics                                # list all topics
```
<!-- icm:end -->
