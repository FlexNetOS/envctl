# Yazelix/Codex Runtime Alignment Ledger

Status: source gate validated locally; live shadow cleanup pending after source merge.

This ledger records the Yazelix ownership model that Codex runtime surfaces must
mirror. Current state is evidence only. When current Codex state conflicts with
Yazelix ownership, the conflict is repair work, not an alternate source of
truth.

## Source Truth

Upstream Yazelix source repository:
`https://github.com/luccahuguet/yazelix`.

The upstream README's everyday model says to edit
`~/.config/yazelix/settings.jsonc` and treat generated runtime state under
`~/.local/share/yazelix` as Yazelix-owned output. The upstream POSIX/XDG docs
separate the managed config root from the generated state root, with
`~/.config/yazelix` as the default config root and `~/.local/share/yazelix` as
the default generated state root. The upstream Home Manager docs say `yzx`
comes from the Home Manager profile, typically `~/.nix-profile/bin/yzx`, and
that stale `~/.local/bin/yzx` plus old user-local desktop entries are migration
shadows to remove or archive. The upstream runtime-root contract rejects path
confusion: generated state is derived state, not handwritten config.

## Required Model

| Role | Required path |
|---|---|
| Editable input/config source | `/home/flexnetos/.config/yazelix/` |
| Generated runtime/proof output | `/home/flexnetos/.local/share/yazelix/` |
| Active profile frontdoor | `/home/flexnetos/.nix-profile/bin/yzx` |
| Active profile layout | `/home/flexnetos/.nix-profile/configs/zellij/layouts/flexnetos_agent_workspace.kdl` |
| Stale binary shadow | `/home/flexnetos/.local/bin/yzx` |
| Stale launcher shadows | `/home/flexnetos/.local/share/applications/*` |

Codex binary/runtime ownership must mirror this model. User-bin shadows,
repo-cache materializations, temp plugin bundles, marketplace caches, and
generated-output files are not alternate active locations.

## Repair Ledger

| surface | current_state | yazelix_model | violation | owner_path | action | proof |
|---|---|---|---|---|---|---|
| Codex command runner shell | `command -v codex` resolves to the Yazelix foundation `toolbin/codex`; `/home/flexnetos/.nix-profile/bin/codex` resolves to the packaged Codex store binary. `codex --version` reports `codex-cli 0.143.0-alpha.35`. The app-server daemon socket is not currently present. | Active frontdoors and runtime helpers must resolve through the profile-owned installed runtime, not a vanished store path or stale user-bin shadow. | The prior vanished-store shell path is not observed in current command resolution. A lower-priority `/home/flexnetos/.local/bin/codex` shadow still appears in `type -a` and remains live cleanup debt. | Yazelix foundation profile plus envctl `home-local-single-link` shadow archive component. | Source repair now preserves the Yazelix-owned Nix profile and makes the shadow cleanup archive-only/fail-closed after replacement frontdoors exist. Live apply remains pending until this source gate PR is merged. | `command -v codex`, `readlink -f /home/flexnetos/.nix-profile/bin/codex`, and `codex --version` pass; `codex app-server daemon version` reports missing control socket. |
| Repo-local MCP baseline | `agent-skills/mcps/n8n-mcp.json` existed as an active envctl source-pack asset, and MCP source defaults still pointed at `/home/drdave/Desktop/meta`. | Envctl MCP baseline is exactly `github`, `context7`, `exa`, `memory`, `playwright`, and `sequential-thinking`, rooted at the current FlexNetOS workspace. | Retired/noisy MCP source or stale workspace roots could be regenerated into active runtime surfaces. | `agent-skills/mcps/` plus `agent-env.yaml -> agent-env.lock` generated outputs. | Removed `agent-skills/mcps/n8n-mcp.json`, removed `n8n-mcp` from `agent-env.yaml`, updated MCP source defaults to `/home/flexnetos/FlexNetOS`, ran `envctl agent sync --apply`, and refreshed `agent-env.lock`. | `bash ci/gates/yazelix-codex-runtime.sh` and `bash ci/gates/agent-env.sh` pass locally. |
| Repo-local ownership regression checks | No focused gate existed for Yazelix/Codex ownership drift across MCP sources, manifest Codex shadows, and generated Yazelix runtime wording. | Regressions must fail closed at the declared source before they recreate non-Yazelix runtime ownership. | Drift could return through generated MCP assets, old workspace roots, or real-home Codex symlink logic. | `scripts/tests/test-yazelix-codex-ownership-gate.sh`. | Added source gate for retired MCPs, stale `/home/drdave/Desktop/meta` roots, real-home Codex shadows, and generated-runtime-as-input language. | `bash scripts/tests/test-yazelix-codex-ownership-gate.sh` passes locally. |

## Verification Still Required

- After this source gate PR merges, run the guarded `home-local-single-link`
  component to archive lower-priority real-home user-bin shadows without
  replacing the Yazelix-owned real-home Nix profile state.
- Re-check `codex app-server daemon version` when the app-server daemon is
  expected to be running; current proof shows the control socket is absent, not
  that the command resolves through the wrong binary.
- Continue verifying `yzx`, `codex`, `rtk`, `claude`, `git-kb`, and relevant
  toolchain binaries with `command -v`, `type -a`, and `readlink -f` during
  runtime changes.
