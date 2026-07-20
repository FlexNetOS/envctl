---
name: env-toolchain-install
description: "How to install and configure environment tooling the way this box does it — the nix profile owns every toolchain binary; envctl's declarative components (detect→install→verify→fix→remove lifecycle per manifest/*.toml) own the meta-local install/wiring layer. Use whenever installing, repairing, or configuring environment tooling or authoring a new component. Triggers: 'install the toolchain', 'set up the environment', 'add a component', 'why is X not on PATH', 'repair the environment', 'the install isn't idempotent'."
---

# Environment Toolchain Install (envctl-grounded)

Two owners, one discipline. **The nix profile `lifeos-foundation-yzx` (yazelix flakes) owns
every toolchain binary and runtime** — rust (fenix), bun/node, claude/codex, rtk/meta/grit/icm/
git-kb, the shells themselves. A missing or outdated toolchain binary is a yazelix-flake change
(`yzx update local_source` rebuilds), never an ad-hoc install (`curl | bash`, `npm -g`, rustup —
all forbidden). **envctl's declarative component model owns the meta-local install/wiring
layer** — repo `/home/flexnetos/meta/src/envctl`, components in `manifest/*.toml`, everything
targeting `$META_ROOT` surfaces (no system-depth or user-global installs), pinned by
`envctl.lock`. Mirror this discipline whenever you touch the environment: if it isn't declared
in the flake or a manifest component, it doesn't exist.

## The Component Contract

Every component declares an idempotent lifecycle:

```toml
[[component]]
id = "<id>"
name = "<human name>"
description = "<what it provides>"
requires = ["<dep-id>"]            # ordering / dependency edges
[component.detect]   kind = "command"  # already installed? (cheap, side-effect-free)
[component.install]  kind = "script"   # idempotent install
[component.verify]   kind = "command"  # post-install proof it works
[component.fix]      kind = "script"   # repair present-but-broken
[component.remove]   kind = "command"  # clean uninstall
[component.wiring]   path_entries = ["..."]  # declared PATH + shell_rc markers
```

**Why each hook exists:**
- **detect** must be side-effect-free and PATH-robust: check the binary path directly, not just
  `command -v` in whatever shell happens to be configured.
- **install** must be safe to run twice — a second run is a no-op or harmless refresh.
- **verify** is separate from detect: detect answers "is it here?", verify answers "does it
  work?". Detects-present + fails-verify = **broken**, not installed.
- **fix** repairs present-but-broken without full remove/reinstall.
- **remove** leaves no PATH/shell_rc residue.
- **wiring** records PATH entries and shell_rc markers so PATH is *declared*, not accreted.

## The Reference Component Set

**Enumerate `manifest/*.toml` (plus drop-ins in `manifest/components.d/`) — do not trust a
hard-coded table.** The set is 21+ components and moves; `envctl.lock` pins the current truth
and `envctl graph` shows the dependency edges. The manifest dir defaults to `./manifest`
(override: `ENVCTL_MANIFEST_DIR`).

## Working Rules

- **Route to the right owner first.** Toolchain binary → yazelix flake / nix profile.
  Meta-local wiring, daemons, links, config surfaces → envctl component. Agent toolkit →
  `envctl agent` (see `env-stabilize`).
- **Author, don't improvise.** Need something installed? Add or invoke a component (or flake
  entry) — never run a bare install command. The environment stays fully described.
- **Order by `requires`.** envctl resolves install order from the dependency edges.
- **Idempotency is the test.** Run install twice, then verify. A second install that errors, or
  a verify that fails, means the component is wrong.
- **PATH is declared via wiring** — never "just add it to your PATH".
- **Secrets are out of scope here.** Credentials belong to the secretd/secretctl stack; never
  bake tokens into install scripts.

## Verify Your Work

After installing/repairing: run the component's `verify` hook, confirm the wiring, and confirm
a **fresh shell** sees the tool — check all the shells the box actually runs:
`bash -c 'command -v <tool>'` (non-login carries toolbin here), `bash -lc '…'` (login), and
`nu -l -c 'which <tool>'` (nushell is the default shell; rtk-wrapped tools report type
`custom`). The environment is reproducible iff every component detects-present and
verifies-green from a clean shell, and `envctl doctor` + `envctl agent lock --check --locked` are green.
