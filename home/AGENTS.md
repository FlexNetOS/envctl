# FlexNetOS home navigation

Read this first for sessions that start in `/home/flexnetos`.

## Active control paths

- Codex runtime config: `/home/flexnetos/.codex/config.toml`.
- Codex operating rules: `/home/flexnetos/.codex/RULES.md`.
- RTK policy: `/home/flexnetos/.codex/AGENTS.rtk.md` and
  `/home/flexnetos/AGENTS.rtk.md`.
- envctl source of truth: `/home/flexnetos/meta/src/envctl`.
- envctl home projection: `/home/flexnetos/meta/src/envctl/home`.

## New envctl session procedure

```bash
cd /home/flexnetos/meta/src/envctl
git fetch origin --prune
git worktree add ../envctl-<task-slug> -b <task-branch> origin/master
cd ../envctl-<task-slug>
envctl agent lock --check --color never
envctl agent sync --json --color never
```

Use `envctl agent sync --apply` only after review. Do not update agent config
by editing generated runtime output directly.

## Retired paths

Do not use these paths as active Codex config, hook, plugin, MCP, marketplace,
or instruction sources:

- `/home/flexnetos/lifeos/.codex`
- `/home/flexnetos/FlexNetOS/.codex`

They were the same symlinked workspace mirror and have been retired to avoid
new-session navigation confusion. If either reappears, archive it and route the
change through `/home/flexnetos/.codex` or envctl `agent-env.yaml`.

## Toolchain/runtime policy

Use the Nix/Yazelix foundation frontdoors: profile-owned nightly cargo/rustc,
kache/kache-rustc-wrapper for Rust caching, wild via clang linker flags,
bun/bunx for Node.js package execution, and profile-owned `codex`, `yzx`, and
`rtk`. Do not install global npm/cargo/curl binaries to fix navigation.

## Session tool bootstrap

The old `CODEX-GPT-HARNESS` session prompt is now the `$harness-session`
skill. Use that skill as the compact controller for harness continuation or
repair, and keep the prompt files as provenance/compatibility shims instead of
the primary procedure store.

Session bootstrap initializes instructions and context, not filesystem state.
Run `$harness-init` at the start of a chat that needs repository context.

- Prove profile-owned `meta`, `git-kb`, `grit`, `icm`, `rtk`, `git`, `codex`,
  `claude`, and the profile `toolbin/nu`.
- GitKB starts with `git-kb list --path context/ --json`; MCP is primary when
  registered and the CLI is the fallback.
- ICM context starts with `icm --read-only wake-up --max-tokens 200`.
- Grit is inactive for read-only/single-agent work; if `.grit/` already exists,
  `grit status` is the readiness probe.
- Route Meta Git plugin commands through `rtk meta git`. Route unlisted fleet
  Git commands through `rtk meta exec -- git`, with `--include <repo>` for
  one-repository scope. Use `rtk git` only for a single checkout.
- `rtk init --show` verifies RTK integration. RTK compresses output; it does
  not grant permissions or choose repository scope.

Never run `git-kb init`, `grit init`, `icm init`, `meta init`, or a mutating
`rtk init` merely because a session started. Do not enable retired Codex hooks
to simulate startup. `/permissions` remains the only live sandbox/approval
authority, and harness capability presets must not initialize these tools.
