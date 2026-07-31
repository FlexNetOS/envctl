# Private Codex state copy

This directory is the local, ignored staging area for the portable/confined
all-in-one app's Codex private state.

- Data root: `home/agent-env/private-codex-state/data/.codex/`
- Source root: `/home/flexnetos/meta/var/lib/codex/`
- Logs are intentionally excluded from the data copy.
- The tracked proof is the manifest at
  `profile-runtime/codex/mined-live/private-copy-manifest.tsv`.
- The tracked log placement plan is
  `home/agent-env/PORTABLE_CODEX_LOGS.md`.

Do not commit the `data/` directory.
