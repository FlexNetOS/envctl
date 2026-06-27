# TASK-0078 next slice plan — app-config target inventory + backup archive guard

Date: 2026-06-27
Worktree: `/home/drdave/Desktop/meta/.worktrees/task-0078-next-inventory/envctl`
Branch: `task-0078-next-inventory`

## Verified baseline

- `origin/master` already contains the prior TASK-0078 slices: PR #290/#291/#293/#296 are merged.
- Live audit from this worktree reported `dot_entries=78`, `changed=0`, and no remaining `shell-dotfile` class or shell conflicts.
- Current class counts: `already-meta=34`, `app-config-state=35`, `bridge=1`, `cache=1`, `managed-dotfile=2`, `sensitive=5`.
- History/backup entries that were real-home state in an earlier inventory now resolve inside `$META_ROOT/var/lib/envctl/real-home-dotfile-migration/history-or-backup`, so no live backup mutation is needed today.

## Design

1. Keep default audit/apply conservative: no broad app state migration happens unless an owner names a dot entry with `--migrate-dot`.
2. Extend the supervised app-config allowlist with canonical targets for known agent/app state that appears in the live inventory:
   - `.gemini`, `.kimi-code`, `.agents`, `.ampcode`, `.codeium`, `.copilot`, `.cursor`, `.goose_recipes`, `.junie`, `.kimi`, `.roo`, `.vscode`, `.windsurf`, `.mozilla`, `.thunderbird` -> `$META_ROOT/.local/share/<name>`.
   - `.ollama` -> `$META_ROOT/var/lib/ollama` to preserve the existing model-store decision.
   - `.claude.json` -> `$META_ROOT/.local/share/claude/claude.json`.
3. Add a separate backup-only archive mode, `--archive-backup-dotfiles`, requiring `--apply`, for backup-like top-level dot entries only (`*.bak`, `*.bak.*`, `*.backup`, `*.backup.*`). Active shell histories stay owner-supervised.
4. Add gate checks so future regressions cannot drop the new app-config target function, backup archive mode, or TDD coverage.

## Runtime surface

- `scripts/tests/test-meta-local-path-audit.sh`
- `ci/gates/meta-local-policy.sh`
- `ci/gates/harness-scripts.sh`
- Live read-only audit against `/home/drdave` and `/home/drdave/Desktop/meta` with inventory, summaries, shell conflict report, and deep link summaries.
