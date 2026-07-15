# CI & Automation

> **Ported from kasetto.dev/docs** (Kasetto v3.2.0, absorbed into envctl `crates/agent-env`).
> Renamed kasetto→`envctl agent`; `kasetto.yaml`→`agent-env.yaml`; mimalloc removed.
> Source: https://www.kasetto.dev/docs/ci. The standalone `kasetto` binary is retired — this is the `envctl agent` surface.

Run envctl in CI pipelines.

**When you need this:** You want to validate `agent-env.yaml` in CI, keep team environments reproducible, or integrate envctl into scripts.

**What you'll learn:**

- Recommended flags (preview-by-default, `--locked`, `--json`, `--color never`, `--quiet`)
- Exit-code expectations
- A GitHub Actions example

> **envctl note:** Kasetto previews with `--dry-run`. **envctl previews by default** — running
> `envctl agent sync` *without* `--apply` is the validate-without-writing path. Every `--dry-run`
> in the upstream doc maps to "the default (omit `--apply`)" below.

## Recommended Commands

### Enforce The Locked Versions

If you commit `agent-env.lock` (recommended for teams — see [Cookbook](./cookbook.md)), use `--locked` to install exactly what the lock pins and **never fetch new versions**:

```
envctl agent sync --locked --apply
```

`--locked` (alias `--frozen`) errors if the config needs something the lock can't satisfy, so a stale or out-of-sync lock fails the build instead of drifting. It still repairs tampered destinations locally, but never resolves moving refs or downloads new content.

### Validate Without Writing

Run the default preview (no `--apply`) to check that sources resolve and that the plan matches expectations without touching disk:

```
envctl agent sync
```

### JSON Output for CI Logs

Use `--json` for structured logs (combine with the default preview):

```
envctl agent sync --json
```

### Strip Colors and Animations

envctl auto-detects non-TTY output and emits plain text, but you can force it:

- `--color never` (preferred)
- `-q` / `--quiet` to suppress non-error output entirely
- The `NO_COLOR` env var is honored

```
envctl agent sync --color never
```

## Exit Codes

envctl is designed to keep going when individual skills are missing/broken in a source, but failures that prevent reading sources/configs are treated as errors.

If you're depending on strict enforcement in CI, pair the preview run with `--json` and enforce policy in the CI step based on the report — or use `envctl agent lock --check` (alias `--frozen`), which **exits 1 on drift**.

## GitHub Actions Example

This validates a repository's `agent-env.yaml` (project scope) without writing changes.

> **envctl note:** envctl has no `curl | sh` installer (the standalone binary was retired). In a
> peer-repository CI job, do not try to build envctl from that peer's source tree. Supply the
> approved Meta-built/profile-owned binary or a verified envctl build artifact, then invoke it
> against the peer's committed config.

```yaml
name: envctl

on:
  pull_request:
  push:
    branches: [main]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Make approved envctl available
        # Obtain this from the Meta toolchain image or a verified build artifact.
        run: envctl --version

      - name: Audit agent-env.yaml
        env:
          # Add tokens if you pull from private sources:
          # GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        # Read-only, zero-network proof of config → lock → installed outputs.
        run: envctl agent audit --config agent-env.yaml --scope project --json
```
