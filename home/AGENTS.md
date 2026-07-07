# FlexNetOS home navigation

Read this first for sessions that start in `/home/flexnetos`.

## Active control paths

- Codex runtime config: `/home/flexnetos/.codex/config.toml`.
- Codex operating rules: `/home/flexnetos/.codex/RULES.md`.
- RTK policy: `/home/flexnetos/.codex/AGENTS.rtk.md` and
  `/home/flexnetos/AGENTS.rtk.md`.
- envctl source of truth: `/home/flexnetos/lifeos/src/envctl`.
- envctl home projection: `/home/flexnetos/lifeos/src/envctl/home`.

## New envctl session procedure

```bash
cd /home/flexnetos/lifeos/src/envctl
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
