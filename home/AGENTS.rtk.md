# Home RTK policy pointer

The canonical RTK policy is `/home/flexnetos/.codex/AGENTS.rtk.md`.

For envctl/Codex navigation, run gate and root-cause commands raw rather than
through summarized RTK output:

- `git fetch origin --prune`
- `git worktree add ...`
- `git pull --ff-only`
- `envctl agent lock --check --color never`
- `envctl agent sync --json --color never`
- `bash ci/gates/agent-env.sh`
- `bash ci/gates/yazelix-codex-runtime.sh`
- `codex doctor --json --summary`

Routine exploratory commands may still use `rtk` when summarized output is
acceptable.
