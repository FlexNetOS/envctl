# FlexNetOS Operating Laws (always in force)

1. **NEVER DELETE — ALWAYS ARCHIVE.** Before modifying or replacing an existing file, copy it to `~/.claude/archive/<UTC-ISO>/<original-relative-path>`. For deletions, use `~/.claude/hooks/harness-archive.sh <path>` (moves into the archive). Deletion of user data is forbidden; `rm` is hook-denied outside scratch paths.
2. **UPGRADE ONLY, NEVER DOWNGRADE.** No config, feature, or capability regresses. Merges are superset merges.
3. **HEAL, DO NOT HARM.** If a step risks breaking a working system, stop and ask via AskUserQuestion.
4. **REAL EXECUTION ONLY.** "Done" requires a command actually run and output actually observed. Never assert unproven state. Show raw output.
5. **NO NEW DOCUMENTS OR REPORTS.** Operational config files are deliverables; prose reports/READMEs/status docs are forbidden — report in the terminal.
6. **CONTAINMENT BEFORE CAPABILITY.** No nested Claude sessions (hook-denied). Subagents and teammates never spawn agents (depth-1). Max 6 active agents. Budget ceiling: 80% of any rate-limit window. Kill switch: `harness-halt.sh`.
7. **STOP MEANS STOP.** Operator decisions go through AskUserQuestion and block. To make a decision survive a stop, write it as a marker file in `$HARNESS_VAR/lib/claude-harness/decisions/<slug>.pending`; rename to `.answered` when resolved. Never loop on a waiting state; never leak scaffold markers into output.
8. **MODEL ROUTING IS AN OPERATOR DECISION.** Everything runs on Fable unless the operator says otherwise. If the safety classifier reroutes to Opus, the statusline flags it — notify and ask before continuing (`/model fable` to return).
