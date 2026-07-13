# FlexNetOS RTK Policy

RTK is the mandatory command frontdoor for Codex sessions. It is cost control,
while the underlying native command remains the source of truth.

Invoke the profile-owned binary explicitly so inherited store paths and shell
aliases cannot bypass it:

```text
/home/flexnetos/.nix-profile/bin/rtk <supported-command> ...
```

Use RTK's native summarizing command for routine discovery. When exact output
is required, keep RTK accounting active and bypass summarization with:

```text
/home/flexnetos/.nix-profile/bin/rtk proxy -- <command> ...
```

Never invoke a shell command outside one of those two RTK paths. In particular,
use `rtk proxy --` for raw evidence from:

- `envctl ...`
- `fxrun ...`
- `cargo test`, `cargo build`, `cargo check`, `cargo clippy`, and `cargo nextest`
- `nix build`, `nix flake check`, `nix develop`, and `nix log`
- `gh run view`, `gh run download`, and `gh pr checks`
- `journalctl`, `dmesg`, and `systemctl status`
- `yzx doctor` and `yzx inspect` when the log is validation evidence

Do not hide several independent diagnostics inside `rtk proxy -- bash -lc
'cmd1; cmd2; ...'` and then count that as full adoption. Route each independent
command through RTK so its accounting and output policy remain observable. A
checked-in repository script is one command and may run its own internal
subprocesses normally.

`rtk proxy --` preserves the native command's stdout, stderr, and exit status.
Use a tee log behind that proxy when durable raw evidence is required.

Preserve raw failure logs under the execution pack `logs/` directory or
`/home/flexnetos/meta/var/log/raw`. Do not replace root-cause evidence with RTK
summaries.

For Yazelix runtime-path verification, prefer raw proof of:

- `~/.nix-profile/bin/yzx` as the active frontdoor
- `~/.config/yazelix` as the editable input surface
- `~/.local/share/yazelix` as generated runtime output
- stale `~/.local/bin/yzx` or user-local desktop entries only as shadow-path findings to remove, not as alternate ownership roots

For Codex runtime-path verification, mirror the same model with no drift:

- preserve the Yazelix ownership table and source citations from `src/yazelix/docs/customization.md`, `docs/posix_xdg.md`, `home_manager/README.md`, `docs/troubleshooting.md`, and `docs/contracts/runtime_root_contract.md`
- treat generated runtime trees as proof/output, never as the editable source
- treat stale wrapper or launcher shadows as cleanup findings, never as legitimate fallback ownership

For envctl/Codex navigation work, capture these through `rtk proxy --`, not
through RTK summaries:

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
