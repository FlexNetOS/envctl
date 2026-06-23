# Security Model

> **Ported from kasetto.dev/docs** (Kasetto v3.2.0, absorbed into envctl `crates/agent-env`).
> Renamed kasetto→`envctl agent`; `kasetto.yaml`→`agent-env.yaml`; mimalloc removed.
> Source: https://www.kasetto.dev/docs/security. The standalone `kasetto` binary is retired — this is the `envctl agent` surface.

Security model and threat boundaries.

**When you need this:** You want to know what envctl can modify on disk, how it handles credentials, and whether syncing from URLs is safe.

**What you'll learn:**

- What envctl touches (and what it avoids)
- How tokens are provided (env vars only)
- How remote config fetching is authenticated

## What envctl Changes On Disk

During `envctl agent sync --apply`, envctl may:

- **Install/update skills** by copying skill directories into the chosen destination path
- **Remove skills** that are no longer in your config (for the selected scope)
- **Merge MCPs** into agent-native settings files (additive merge; never overwrite existing servers)
- **Write the lock file** for the selected scope

envctl is designed around a "tracked-only" principle:

- **Skills**: fully managed at their install paths for entries tracked in the lock
- **MCPs**: only server entries that envctl installed (tracked in the lock) are removed during cleanup

See [How Sync Works](./how-sync-works.md) for details.

## MCP Server Installation

New MCPs are merged into agent settings files on `envctl agent sync --apply` — no confirmation prompt. Because envctl is **preview-by-default**, run `envctl agent sync` *without* `--apply` first to see exactly what would be written before committing.

## Self-Update Integrity

> **envctl note:** Kasetto's `self update` downloads a release binary and verifies it against
> `checksums.txt` via SHA-256. **envctl's agent-env engine ships inside the `envctl` binary** (the
> standalone binary was retired), so there is no agent-env-binary self-update step. You upgrade by
> rebuilding from the meta Cargo workspace (`git pull && cargo build -p envctl`). The download →
> checksum-verify → atomic-swap-with-rollback pattern is preserved in envctl's own component-build
> safety doctrine for any binary/artifact a component builds. Upstream text follows for reference:

Kasetto's `self update` verified the downloaded binary against `checksums.txt` from the same GitHub release using SHA-256 — the same verification the shell installer (`install.sh`) performed. If the checksum didn't match, the update was aborted and the existing binary left untouched.

## What envctl Does Not Do

- It does not run skill code.
- It does not overwrite existing MCP server entries.
- It does not require (or write) a credentials file.

## Credentials And Tokens

envctl reads tokens from environment variables (per host).

Examples:

- GitHub / GitHub Enterprise: `GITHUB_TOKEN` or `GH_TOKEN`
- GitLab / self-hosted GitLab: `GITLAB_TOKEN` or `CI_JOB_TOKEN`
- Bitbucket Cloud: `BITBUCKET_EMAIL` + `BITBUCKET_TOKEN` (or app password variants)

See [Authentication](./authentication.md) for the full list and host detection instructions.

## Remote Config Fetching (--config https://...)

When you pass a URL to `--config`, envctl:

- Fetches the YAML over HTTPS
- Applies the same host-based token selection instructions as skill/MCP sources

This means a private config hosted on a git provider can be accessed by setting the appropriate token env var for that host.

## Practical Recommendations

- Prefer pinning remote sources to immutable refs (`ref: v1.2.3` or a commit SHA) for stable rollouts.
- In CI, run the default preview (no `--apply`) — and ideally `--json` — to validate without writing changes; use `--locked` to fail on drift.
