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
envctl agent lock --check --locked --color never
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

## Codex harness session bootstrap

Use `/agent-env-codex` as the single harness skill. Init, sync, status, full,
restricted, and capability toggles are internal commands of that skill, not
separate top-level skills. `/permissions` is the sole live sandbox, approval,
and network authority.

Session bootstrap is non-mutating:

- prove profile-owned `yzx`, `rtk`, `meta`, `git-kb`, `grit`, `icm`, `git`,
  `codex`, `claude`, and `toolbin/nu`;
- start GitKB context with `rtk git-kb list --path context/ --json`;
- probe Grit only when `.grit/` already exists;
- use `ICM_READONLY=1 rtk icm wake-up --max-tokens 200`;
- route fleet Git through `rtk meta git`, unlisted fleet Git through
  `rtk meta exec -- git`, and one-checkout Git through `rtk git`;
- use `rtk init --show` to inspect RTK without changing integration state.

Never initialize GitKB, Grit, ICM, Meta, RTK, or Weave merely because a session
started. Keep Sol for high-stakes work, Terra as the professional workhorse,
and Luna for simple high-volume work. Do not restore routeable GPT-5.5
assignments or tracked model-cache authority.
