# FlexNetOS RTK Policy

RTK is cost control. It is not the source of truth for failure evidence.

Use `rtk` for routine discovery commands when summarized output is acceptable.

Run these commands raw when capturing gate evidence or root-cause diagnostics:

- `envctl ...`
- `fxrun ...`
- `cargo test`, `cargo build`, `cargo check`, `cargo clippy`, and `cargo nextest`
- `nix build`, `nix flake check`, `nix develop`, and `nix log`
- `gh run view`, `gh run download`, and `gh pr checks`
- `journalctl`, `dmesg`, and `systemctl status`
- `yzx doctor` and `yzx inspect` when the log is validation evidence

If a command would normally be routed through RTK but raw output is required, run it raw with a tee log. Use `rtk proxy <cmd>` only when tracking is useful and the raw output is still preserved.

Preserve raw failure logs under the execution pack `logs/` directory or `/home/flexnetos/meta/var/log/raw`. Do not replace root-cause evidence with RTK summaries.

For Yazelix runtime-path verification, prefer raw proof of:

- `~/.nix-profile/bin/yzx` as the active frontdoor
- `~/.config/yazelix` as the editable input surface
- `~/.local/share/yazelix` as generated runtime output
- stale `~/.local/bin/yzx` or user-local desktop entries only as shadow-path findings to remove, not as alternate ownership roots

For Codex runtime-path verification, mirror the same model with no drift:

- preserve the Yazelix ownership table and source citations from `src/yazelix/docs/customization.md`, `docs/posix_xdg.md`, `home_manager/README.md`, `docs/troubleshooting.md`, and `docs/contracts/runtime_root_contract.md`
- treat generated runtime trees as proof/output, never as the editable source
- treat stale wrapper or launcher shadows as cleanup findings, never as legitimate fallback ownership

For envctl/Codex navigation work, capture these raw, not through RTK summaries:

- `git fetch origin --prune`, new worktree creation, `git pull --ff-only`
- `envctl agent lock --check --color never`
- `envctl agent sync --json --color never`
- `bash ci/gates/agent-env.sh`
- `bash ci/gates/yazelix-codex-runtime.sh`
- `codex doctor --json --summary`
- `codex mcp list`, `codex plugin list`
- `command -v/readlink -f` checks for `cargo`, `rustc`, `kache`,
  `kache-rustc-wrapper`, `wild`, `bun`, `bunx`, `codex`, `yzx`, and `rtk`

Do not use `/home/flexnetos/lifeos/.codex` or
`/home/flexnetos/FlexNetOS/.codex` as evidence of active Codex runtime state.
Those mirrors are retired; active runtime evidence comes from
`/home/flexnetos/.codex` plus envctl `agent-env.yaml`/`agent-skills`.
