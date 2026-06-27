# TASK-0078 guardian report

Date: 2026-06-27

## Verification commands

```bash
bash scripts/tests/test-meta-local-path-audit.sh
bash ci/gates/meta-local-policy.sh
bash -n scripts/audit-meta-local-paths.sh scripts/tests/test-meta-local-path-audit.sh ci/gates/meta-local-policy.sh
bash ci/gates/harness-scripts.sh
```

All commands passed.

## Live runtime verification

Read-only live audit command:

```bash
./scripts/audit-meta-local-paths.sh \
  --inventory /tmp/.../inventory.tsv \
  --inventory-summary /tmp/.../summary.tsv \
  --deep-link-inventory /tmp/.../deep.tsv \
  --deep-link-summary /tmp/.../deep-summary.tsv \
  --shell-dotfile-conflict-report /tmp/.../shell-conflicts.tsv \
  --meta-root /home/drdave/Desktop/meta \
  --real-home /home/drdave \
  --envctl-home-source /home/drdave/Desktop/meta/envctl/home
```

Observed:

- exit code: 0
- `changed=0`
- inventory summary:
  - `already-meta=34`
  - `app-config-state=35`
  - `bridge=1`
  - `cache=1`
  - `managed-dotfile=2`
  - `sensitive=5`
- shell conflict report: header only
- deep-link summary: `inside-meta=5060`, `external-system=1119`, `missing-target=329`, no `real-home-leak` rows
- backup/history sample rows (`.bash_history`, `.zsh_history`, `.bashrc.bak.*`, `.claude.json.bak.*`, `.n8n.bak.*`, `.zshrc.bak.*`) all resolve inside `$META_ROOT/var/lib/envctl/real-home-dotfile-migration/history-or-backup` as `already-meta`.

## Result

PASS. The slice adds test-proven migration affordances without weakening the default non-mutating owner-supervised policy.
