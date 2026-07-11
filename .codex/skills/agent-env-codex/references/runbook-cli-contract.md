# Runbook and CLI contract

## Agent-env convergence

- Desired state flows `agent-env.yaml -> agent-env.lock -> envctl agent sync`.
- Preview with `envctl agent sync --json --color never`.
- Materialize only when requested with `envctl agent sync --apply --color never`.
- Check drift with `envctl agent lock --check` and the repo agent-env gate.
- Synced skills are replaced from owning sources; do not hand-edit generated copies.

## Session initialization probes

```bash
/home/flexnetos/.nix-profile/bin/yzx status --versions
/home/flexnetos/.nix-profile/bin/yzx inspect --json
/home/flexnetos/.nix-profile/toolbin/nu --version
/home/flexnetos/.nix-profile/bin/rtk git-kb list --path context/ --json
/home/flexnetos/.nix-profile/bin/rtk grit status
ICM_READONLY=1 /home/flexnetos/.nix-profile/bin/rtk icm wake-up --max-tokens 200
/home/flexnetos/.nix-profile/bin/rtk meta git status
/home/flexnetos/.nix-profile/bin/rtk meta exec --include envctl -- git status --short --branch
command -v weave || true
```

Missing `.grit`, an absent ICM DB, or a missing Weave executable is a recorded gap, not permission to initialize implicitly or stop unrelated work.

## Command routing

| Intent | Route |
| --- | --- |
| Yazelix command discovery | profile `yzx --help`; `yzx inspect --json` command metadata |
| Profile/runtime proof | `yzx status --versions`; `yzx status --json`; `yzx inspect --json`; `yzx doctor --json` |
| Yazelix owner update | exactly one of `yzx update local_source`, `yzx update upstream`, or `yzx update home_manager` |
| Nushell | profile `nu -c` or `nu -l -c` when login config is required |
| Bash/Zsh compatibility | `yzx run bash -lc "<cmd>"`; `yzx run zsh -lc "<cmd>"` |
| Single checkout git | `rtk meta exec --include <repo> -- git <command>` |
| Meta fleet git | `rtk meta git ...` |
| Unlisted fleet git | `rtk meta exec --include <repo> -- git <command>` |
| GitKB | `rtk git-kb ...` |
| Grit | `rtk grit ...` |
| ICM | `rtk icm ...` |

The full researched command inventory, update/convergence transaction, latest
toolchain rules, and plugin/add-on ownership contract live in
`yazelix-cli-plugin-policy.md`.

## Professional harness probes

```bash
cargo run -p envctl -- --help
cargo run -p envctl -- auto-detect --json
cargo run -p envctl -- doctor --help
cargo run -p envctl -- graph --help
cargo run -p envctl -- lock --help
cargo run -p envctl -- dashboard --help
```

For hardware-aware work, parse evidence for dual RTX 5090, NVIDIA-SMI, `nvidia-open`, CUDA toolkit 13.3, NVIDIA Container Toolkit + CDI, cuda-oxide, PyTorch cu132, GPU smoke scripts, `kache`, and `wild`. Do not bypass envctl component owners.

## Automation contracts

- `env-install-loop`: `doctor -> install -> auto-fix`, durable backlog, checkpoint/handoff.
- `auto-provision`: fresh-context unattended wrapper around the install loop.
- Component research/audit: real exercise, currency/advisories, hook hygiene, side effects, cross-component skew.
- Dashboard panes default to shell; `envctl-open-Codex` is human opt-in and preserves mesh identity.
