# home/ — the canonical home tree (ADR-0006)

This directory is the **single source of truth for user-global, non-secret configuration** on a
FlexNetOS workstation. The portability principle (locked 2026-06-12): *real file in meta, symlink
outside, never the reverse* — `$HOME` paths are symlinks into this tree, wired by the
`portability-links` components (`manifest/components.d/portability-links.toml`).

```
$HOME/.gitconfig                    -> envctl/home/.gitconfig
$HOME/.claude/settings.json         -> envctl/home/.claude/settings.json     (claude-global-links)
$HOME/.claude/CLAUDE.md             -> envctl/home/.claude/CLAUDE.md         (harness 2026-07-07, owner-supervised)
$HOME/.claude/rules                 -> envctl/home/.claude/rules             (harness 2026-07-07, owner-supervised)
$HOME/.claude/hooks                 -> envctl/home/.claude/hooks             (harness 2026-07-07, owner-supervised)
$HOME/.claude/agents                -> envctl/home/.claude/agents            (harness 2026-07-07, owner-supervised)
$HOME/.claude/skills                -> envctl/home/.claude/skills            (harness 2026-07-07, owner-supervised)
$HOME/.claude/commands              -> envctl/home/.claude/commands          (harness 2026-07-07, owner-supervised)
$HOME/.config/rtk                   -> envctl/home/.config/rtk               (rtk-config-links)
$HOME/.config/yazelix/settings.jsonc-> envctl/home/.config/yazelix/...       (home-config-links)
$HOME/.config/systemd/user/*.service-> $META_ROOT/.config/systemd/user/...   (engine-owned discovery bridge)
$HOME/.codex/config.toml            -> active runtime config (not generated from mirrors)
envctl/home/.codex                  -> reviewed project/home Codex layer
$ENVCTL_REAL_HOME/.nix-profile      -> real-home Nix profile state          (Yazelix-owned)
workspace usr/bin/<tool>            -> LEGACY pack residue (runtime ownership = Nix profile)
```

## Rules (review gates — this repo is PUBLIC)

1. **No secrets, ever.** Credentials delegate outward (`.gitconfig` uses `gh auth git-credential`;
   `~/.claude/.credentials.json`, `~/.config/gh/hosts.yml`, keyrings are NEVER added). The envctl
   secrets stack / relay is the sanctioned channel for secret material.
2. **No envctl-owned host-home state.** Histories, caches, sessions, `vox.db`, piper voices, and envctl-owned
   share/state/cache data live under the workspace root's `var/` trees (`var/lib`, `var/cache`,
   `var/log`, `var/tmp`, or meta-home XDG roots such as `.local/share` when a desktop/XDG
   contract requires it). Yazelix-owned real-home Nix profile state is preserved; per-tool
   real-home user-bin shadows are not install targets.
3. **Archive-first.** The wiring components move any pre-existing real file to
   `~/Desktop/_archives/home-links-<date>/` before linking — originals are never deleted.
4. **Every file is reviewed individually** before it lands here (no bulk `cp -r` of live dirs).

## Layering

- **envctl** (this repo) = OS/toolchain/box layer — owns this tree and the symlink wiring.
- **agent-env** = agent layer (skills/MCP into `.claude`/`.codex`) — authoritative project state is
  `agent-env.yaml` + `agent-env.lock`, driven by `envctl agent`. The historical
  `home/.config/kasetto/kasetto.yaml` file is retained only as a reviewed source artifact from the
  absorbed kasetto lineage; it is not the generated output authority.
- **Codex runtime** = `/home/flexnetos/.codex/config.toml` plus the reviewed
  home projection here. Keep `$HOME/.codex/RULES.md`, `$HOME/.codex/RTK.md`,
  `$HOME/.codex/AGENTS.rtk.md`, `$HOME/AGENTS.md`, and
  `$HOME/AGENTS.rtk.md` aligned from the tracked `home/.codex/` and `home/`
  copies before applying home/runtime sync. The old workspace mirror paths
  `/home/flexnetos/lifeos/.codex` and `/home/flexnetos/FlexNetOS/.codex` are
  retired because they were the same symlinked directory and caused repeated
  ownership confusion. Do not regenerate or consult them; archive if they
  reappear.
- **Toolchains** = Nix/Yazelix foundation profile: nightly cargo/rustc via the
  profile toolchain, kache for compiler caching, wild via clang linker flags,
  bun/bunx for Node.js package execution, and the supported Bash/Zsh/Fish/Nushell
  binaries. Host shell startup may select a shell, but it must inherit or prepend
  `~/.nix-profile/{toolbin,bin}`; retired `$META_ROOT/.toolchains/zsh` builds,
  `$META_ROOT/usr/bin/zsh` wrappers, and migration launchers are not fallback
  owners. Avoid npm/npx/global Cargo installs.
- **meta** = repo/workspace layer — `meta/scripts/bootstrap.sh` sequences rustup → clone → build →
  `envctl install` → `envctl agent sync --locked` → `envctl doctor && envctl lock --check`.

## Review loop and known materialized host-local paths

Run the relocation audit before each dot-entry slice so the residual list is evidence-backed:

```bash
scripts/audit-meta-local-paths.sh \
  --inventory /tmp/meta-local-inventory.tsv \
  --inventory-summary /tmp/meta-local-summary.tsv \
  --deep-link-inventory /tmp/meta-local-deep.tsv \
  --deep-link-summary /tmp/meta-local-deep-summary.tsv \
  --fail-real-home-deep-links
```

For owner-supervised `.cache` child upgrades, keep the loop read-only until the component surface
is declared: add `--owner-supervised-cache-child-component-plan /tmp/cache-plan.tsv` and
`--owner-supervised-cache-child-component-manifest-status /tmp/cache-manifest-status.tsv` to prove
the bounded component key and whether `manifest/components.d/cache-<component_key>.toml` exists,
then `--owner-supervised-cache-child-component-manifest-validation /tmp/cache-manifest-validation.tsv`
to prove an existing manifest declares the expected `[[component]] id = "cache-<component_key>"`.
For missing manifests, add
`--owner-supervised-cache-child-component-manifest-scaffold /tmp/cache-manifest-scaffold.tsv` to
produce a deterministic escaped TOML `manifest_stub` for owner review; the report is read-only and
must be reviewed/materialized before any named `--migrate-cache-child NAME` apply run.

- `home/.claude/settings.json` is rendered from the tracked template; materialized absolute
  marketplace/statusline paths are expected for this workstation, not a reason to reintroduce
  real-home install roots.
- `home/.config/yazelix/mission-control.kdl` is a generated host layout and may carry this box's
  pane paths until the owning component regenerates it.
- `home/.config/nushell/config.nu` no longer hardcodes `/home/drdave`; it sources the meta path
  module relative to the overlay.
- `home/.config/yazelix/shell_bash.sh` still carries a compatibility fallback for older launches;
  treat it as a reviewed residual, not an install target.
- Systemd user units are not tracked home-tree projections. Their component manifests render the
  sole authoritative copies under `$META_ROOT/.config/systemd/user`; the wiring engine creates one
  verified symlink in the real user-manager XDG search path so `systemctl --user` can discover them.
  Historical `home/.config/systemd/user` copies are intentionally absent, and a foreign bridge is
  refused before the engine mutates the canonical unit.
- RTK config is tracked here; RTK command history and tee logs remain machine-local state under
  the workspace `.local/share/rtk/` only when RTK requires XDG data semantics; otherwise use
  the workspace `var/lib/rtk/`. The Yazelix profile is the real-home runtime owner; per-tool
  real-home user-bin shadows remain compatibility debt.
