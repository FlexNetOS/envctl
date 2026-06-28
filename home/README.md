# home/ — the canonical home tree (ADR-0006)

This directory is the **single source of truth for user-global, non-secret configuration** on a
FlexNetOS workstation. The portability principle (locked 2026-06-12): *real file in meta, symlink
outside, never the reverse* — `$HOME` paths are symlinks into this tree, wired by the
`portability-links` components (`manifest/components.d/portability-links.toml`).

```
$HOME/.gitconfig                    -> envctl/home/.gitconfig
$HOME/.claude/settings.json         -> envctl/home/.claude/settings.json     (claude-global-links)
$HOME/.config/rtk                   -> envctl/home/.config/rtk               (rtk-config-links)
$HOME/.config/yazelix/settings.jsonc-> envctl/home/.config/yazelix/...       (home-config-links)
$HOME/.config/systemd/user/*.service-> envctl/home/.config/systemd/user/...  (home-config-links)
$ENVCTL_REAL_HOME/.local            -> $META_ROOT/.local                    (only real-home bridge)
$META_ROOT/usr/bin/<tool>        -> $META_ROOT/.toolchains/... or meta/<repo>/target/release/<tool>
```

## Rules (review gates — this repo is PUBLIC)

1. **No secrets, ever.** Credentials delegate outward (`.gitconfig` uses `gh auth git-credential`;
   `~/.claude/.credentials.json`, `~/.config/gh/hosts.yml`, keyrings are NEVER added). The envctl
   secrets stack / relay is the sanctioned channel for secret material.
2. **No host-home state.** Histories, caches, sessions, `vox.db`, piper voices, and envctl-owned
   share/state/cache data live under canonical `$META_ROOT` roots (`var/lib`, `var/cache`,
   `var/log`, `var/tmp`, or meta-home XDG roots such as `.local/share` when a desktop/XDG
   contract requires it). The only real-home `.local` object is the single bridge back to meta.
3. **Archive-first.** The wiring components move any pre-existing real file to
   `~/Desktop/_archives/home-links-<date>/` before linking — originals are never deleted.
4. **Every file is reviewed individually** before it lands here (no bulk `cp -r` of live dirs).

## Layering

- **envctl** (this repo) = OS/toolchain/box layer — owns this tree and the symlink wiring.
- **agent-env** = agent layer (skills/MCP into `.claude`/`.codex`) — authoritative project state is
  `agent-env.yaml` + `agent-env.lock`, driven by `envctl agent`. The historical
  `home/.config/kasetto/kasetto.yaml` file is retained only as a reviewed source artifact from the
  absorbed kasetto lineage; it is not the generated output authority.
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
to prove an existing manifest declares the expected `[[component]] id = "cache-<component_key>"`
before any named `--migrate-cache-child NAME` apply run.

- `home/.claude/settings.json` is rendered from the tracked template; materialized absolute
  marketplace/statusline paths are expected for this workstation, not a reason to reintroduce
  real-home install roots.
- `home/.config/yazelix/mission-control.kdl` is a generated host layout and may carry this box's
  pane paths until the owning component regenerates it.
- `home/.config/nushell/config.nu` no longer hardcodes `/home/drdave`; it sources the meta path
  module relative to the overlay.
- `home/.config/yazelix/shell_bash.sh` still carries a compatibility fallback for older launches;
  treat it as a reviewed residual, not an install target.
- `repowire.service` is carried for the record but disabled on the box (binary missing — see header).
- RTK config is tracked here; RTK command history and tee logs remain machine-local state under
  `$META_ROOT/.local/share/rtk/` only when RTK requires XDG data semantics; otherwise use
  `$META_ROOT/var/lib/rtk/`. The single real-home `.local` bridge remains compatibility-only.
