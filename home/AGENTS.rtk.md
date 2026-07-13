# Home RTK policy pointer

The canonical RTK policy is `/home/flexnetos/.codex/AGENTS.rtk.md`.

Every Codex shell command must use the exact profile frontdoor
`/home/flexnetos/.nix-profile/bin/rtk`. For gate and root-cause commands, retain
raw output through `/home/flexnetos/.nix-profile/bin/rtk proxy -- ...`:

- `git fetch origin --prune`
- `git worktree add ...`
- `git pull --ff-only`
- `envctl agent lock --check --color never`
- `envctl agent sync --json --color never`
- `bash ci/gates/agent-env.sh`
- `bash ci/gates/yazelix-codex-runtime.sh`
- `codex doctor --json --summary`

Routine exploratory commands use the same exact RTK binary without `proxy --`
when summarized output is acceptable. Direct raw command invocation is not an
alternate path.
