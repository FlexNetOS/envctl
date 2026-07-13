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
- **Render/remove commands** at agent-native command destinations
- **Merge MCPs** into agent-native settings files (additive merge; never overwrite existing servers)
- **Write the v3 lock and private runtime ownership/report ledger** for the selected scope

envctl is designed around a "tracked-only" principle:

- **Skills/commands**: an existing output is replaced or deleted only when its identity and current
  hash match an exact ownership proof
- **MCPs**: only exact server fragments envctl proved it installed are removed during cleanup

Desired lock entries are not ownership. Project-scope `installed_outputs` attestations are created
only by a successful apply and may remain as tombstones until removal commits. Global ownership is
kept in the machine-local `managed_outputs` ledger. Lock/check operations may preserve proofs but
never synthesize them.

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

## Lock v3 Filesystem Boundary

- Skill hashing and installation use the same immutable tree snapshot. Source symlinks are followed
  only when their effective targets remain inside the materialized source; cycles, escapes, special
  entries, and symlinks anywhere in an installed destination are refused.
- Project destinations must remain below the project root. Portable ownership paths are relative
  and limited to known native/current custom targets. A retired project custom-root tombstone also
  requires its exact runtime proof. Global custom-root removal requires an exact secure runtime
  proof and matching current content.
- Lock/runtime files and their existing parent chains are opened no-follow and must be regular,
  current-user-owned paths. Existing modes are preserved; new project locks default to `0644`,
  while new global locks/runtime files default to `0600`.
- Sync apply stages outputs, the v3 lock, and runtime state together, revalidates live inputs, and
  commits them as one strict rollback boundary. Failed coherent commits do not advance ownership.
  Clean enumerates complete proof units, refuses drift/incomplete proof, and retains proof after an
  output-cleanup failure so a retry remains recoverable.

## Zero-Network And First-Install Rules

Only `--locked`/`--frozen` guarantees zero network. It rejects remote config roots and remote
`extends`, audits local sources in place, and validates remote revision/hash/selector bindings from
the v3 lock. A remote locked failure may be reported first as typed selector/proof drift, but no
fetch or destination write occurs.

A fresh/synthetic v3 desired lock has no installation authority. Use a first plain
`sync --apply` into absent destinations to record proofs, then commit the resulting project lock;
clean clones can use
`sync --locked --apply`. Versionless/v2 locks are rejected in locked mode and cannot be directly
restamped. Their exact-output bootstrap is available only inside the atomic plain-apply migration.

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
