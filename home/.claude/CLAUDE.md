@RTK.md

# FlexNetOS user-global operating contract (lean pointers, no prose)

- **Laws:** `~/.claude/rules/laws.md` — the 8 operating laws, hook-enforced. Read once, obey always.
- **Git topology:** `~/.claude/rules/git-topology.md` — main/develop only, superset merges, worktree ritual.
- **Workspace boundaries:** `~/.claude/rules/flexnetos-boundaries.md` (FlexNetOS paths), `~/.claude/rules/rust-conventions.md` (prompt_hub paths).
- **Toolchain:** `~/.claude/rules/toolchain.md` — nix-profile only; cargo via fenix, bun/bunx for node, no ad-hoc global installs.
- **Harness operations** (team spawn/cleanup, kill switch, budget, recovery): invoke the `harness-ops` skill.
- **Source of truth for this file and everything in ~/.claude:** `FlexNetOS/src/envctl/home/.claude/` (ADR-0006: real file in meta, symlink outside). Edit via envctl worktree on develop, never in place.
- **Runtime state:** ledger `$HARNESS_VAR/log/claude-harness/ledger.jsonl` (append-only), decisions `$HARNESS_VAR/lib/claude-harness/decisions/`, kill switch `/home/flexnetos/lifeos/src/envctl/home/bin/harness-halt.sh` (not on `PATH` — full path).
- Report in the terminal only. Show raw output for every completion claim.

(The former ICM-mandate block was removed 2026-07-07: `icm` is not installed on this workstation — a mandate on a missing binary breaks every session. Archived at ~/.claude/archive/20260707T111730Z/envctl-home-claude/CLAUDE.md; restore when icm ships via the foundation profile.)
