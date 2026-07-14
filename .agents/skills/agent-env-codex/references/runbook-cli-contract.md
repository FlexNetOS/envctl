# Runbook and CLI contract

## Agent-env convergence

- Desired state flows `agent-env.yaml -> agent-env.lock -> envctl agent sync`.
- Sync is LOCK-DRIVEN: after editing an `agent-skills/` source, regenerate the lock FIRST
  (`envctl agent lock --color never`), THEN `envctl agent sync --apply` — sync run against a
  stale lock reports the edited asset "unchanged" and propagates nothing (observed 2026-07-12).
- Preview with `envctl agent sync --json --color never`.
- Materialize only when requested with `envctl agent sync --apply --color never`.
- Check drift with `envctl agent lock --check` and the repo agent-env gate.
- Synced skills are replaced from owning sources; do not hand-edit generated copies.
- Substrate hook parity: the ADR-0006 source `home/.claude/settings.json` (and its
  `.tmpl`) carries the weave WL-084 hooks (`weave hook session|prompt|wake`) and the icm
  hooks (`icm hook start|pre|post|prompt|end|compact`) with PATH-resolved commands —
  never `/nix/store/<hash>`-pinned (a profile rebuild leaves pinned hooks firing stale
  builds, the `bash-to-nu.py: not found` rot class). A live-only hook that the source
  lacks is source lag: adopt it PATH-resolved. Enforced by
  `scripts/tests/test-agent-env-hooks.sh` via the harness-scripts gate.
- Settings template ownership (OWNER RULING 2026-07-07, enforced by the Rust gate
  `env_cmd_tests::settings_json_matches_rendered_tmpl_no_drift`): session wiring uses
  EXPLICIT REAL PATHS — no META_ROOT/LIFEOS_ROOT/placeholder root-var wiring in
  `home/.claude/settings.json` or its `.tmpl`. With no placeholders the portability-links
  render is an identity copy BY DESIGN, and the tmpl must equal settings.json
  byte-for-byte. Adopt any live-only top-level key (e.g. `effortLevel`) into BOTH files or
  a re-render drops it. Do NOT "fix" the inert render by parameterizing the tmpl — that
  exact change REDs CI (observed 2026-07-12, PR #495 first push).
- Tier-B ratification: session toggles (permission mode, effort) stay OUT of durable config
  by default (ANTI-LOCKOUT) — but the operator may RATIFY one into declared state via an
  answered decision marker, after which it lives in BOTH settings files and must survive
  re-renders (ratified 2026-07-12: `permissions.defaultMode:"auto"`, `effortLevel:"high"`;
  markers in $HARNESS_VAR/lib/claude-harness/decisions/). Unratified Tier-B state found
  hard-coded is still drift to sweep.

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
| Nushell | profile `nu -c` or `nu -l -c` using the Yazelix-owned `~yazelix/nushell/config` and `~yazelix/nushell/scripts` surfaces; prefer Nu scripts for repeatable harness commands |
| Bash/Zsh compatibility | Bash is configured inside the Yazelix/Nushell runtime; call it through Nushell/Yazelix when needed, but do not add separate bash wrappers, separate shell launchers, or parallel shell control paths |
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
