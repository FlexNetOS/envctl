# TASK-0078 implementer log

Date: 2026-06-27

## Changes made

- Added `--archive-backup-dotfiles` to `scripts/audit-meta-local-paths.sh`.
  - It is inert unless paired with `--apply`.
  - It archives only top-level backup-like dot entries under `$META_ROOT/var/lib/envctl/real-home-dotfile-migration/<timestamp>/`.
  - It refuses symlink backups and leaves active `.bash_history`, `.zsh_history`, and `.*_history` owner-supervised.
- Added `app_config_target_for_dot` / `is_app_config_dot` and routed known agent/app config state to canonical meta-owned targets.
- Expanded TDD coverage for:
  - app-config inventory targets (`.gemini`, `.kimi-code`, `.ollama`, `.claude.json`),
  - explicit `--migrate-dot` dry-run/apply behavior for those app config entries,
  - backup archive pre-summary and apply behavior,
  - active shell history remaining untouched.
- Strengthened `ci/gates/meta-local-policy.sh` to require the new backup/app-config audit affordances and tests.

## Notes

- No live `--apply --archive-backup-dotfiles` was necessary: the live backup/history rows currently resolve inside `$META_ROOT` as symlinks, and the read-only audit has no remaining `history-or-backup` real-home class.
- Broad unknown app config entries remain classified `owner-supervised-migration` with no canonical target and no automatic move.
