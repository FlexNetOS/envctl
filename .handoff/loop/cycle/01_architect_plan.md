# TASK-0078 plan — migration blocker sensitive hint visibility

Date: 2026-06-27
Worktree: `/home/drdave/Desktop/meta/.worktrees/meta-local-blocker-sensitive-hints/envctl`
Branch: `meta-local-blocker-sensitive-hints`

## Verified baseline

- `origin/master` is at `c8f2e48 Generate Codex model swarm baseline (#335)` (and includes `5be9041` from #334).
- Focused baseline passed:
  - `bash scripts/tests/test-meta-local-path-audit.sh`
  - `bash ci/gates/meta-local-policy.sh`
- Live read-only audit reported `meta-local audit: PASS warnings=10 changed=0 dot_entries=79`.
- Remaining real-home blockers are `.aws`, `.cache`, `.config`, `.docker`, `.fxapp-gh-profile`, `.gnupg`, `.lane`, `.mcp-auth`, `.pki`, `.ssh`.
- `.pki` is an apply-safe app-config migration target, but Chrome has open handles and the blocker report did not expose NSS/private-state hint counts.

## Design

Add `sensitive_hints` to every `--migration-blockers-report` row, immediately after `canonical_target`, so surgical migration planning sees credential/private-key/NSS hints even when a dot entry is classified as app-config-state rather than sensitive.

## Runtime surface

- `scripts/audit-meta-local-paths.sh --migration-blockers-report`
- `scripts/tests/test-meta-local-path-audit.sh`
- `ci/gates/meta-local-policy.sh`
- Live read-only audit against `/home/drdave` and `/home/drdave/Desktop/meta`
