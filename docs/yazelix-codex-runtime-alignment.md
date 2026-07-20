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
| Repo-local MCP baseline | Legacy local-launch MCPs used Meta `bunx` or repo-source scripts as active runtime paths. | Envctl's current generated baseline is remote `exa` only; retired local launchers return only after they have profile-owned Yazelix-mirrored frontdoors. | Rehydrating cached/local MCP launchers would recreate parallel runtime ownership. | `agent-skills/capability-packs/agent-env-config/`, `agent-env.yaml -> agent-env.lock`, and generated `.mcp.json` / `.codex/config.toml`. | Removed non-profile local launch entries and retained the remote URL baseline without widening global runtime config. | `bash ci/gates/yazelix-codex-runtime.sh` and `bash ci/gates/agent-env.sh` pass locally. |
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
