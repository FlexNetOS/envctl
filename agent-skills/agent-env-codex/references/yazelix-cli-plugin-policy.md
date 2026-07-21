# Yazelix CLI, update, toolchain, and plugin policy

## Contents

1. Live authority
2. Researched command surface
3. Mandatory update transaction
4. Latest toolchain rule
5. Plugin and add-on consolidation
6. Installed connectivity proof

## Live authority

Use the profile frontdoor, not repo scripts or generated runtime:

```text
/home/flexnetos/.nix-profile/bin/yzx
  -> /nix/store/...-lifeos-foundation-yzx/bin/yzx
editable input:  /home/flexnetos/.config/yazelix/
generated proof: /home/flexnetos/var/lib/yazelix/
```

Discover the current command set before acting:

```bash
/home/flexnetos/.nix-profile/bin/yzx --version-full
/home/flexnetos/.nix-profile/bin/yzx --help
/home/flexnetos/.nix-profile/bin/yzx inspect --json
```

`inspect --json` exposes the machine-readable command registry under
`command_metadata.commands`. The inventory below is the verified v17.9
snapshot from 2026-07-11; live profile metadata is newer authority.

## Researched command surface

| Family | Current public commands |
| --- | --- |
| Root/session | `yzx`, `agent`, `enter`, `env`, `launch`, `restart`, `run` |
| Config/edit/import | `config`, `config set`, `config ui`, `config unset`, `edit`, `edit config`, `import`, `import helix`, `import yazi`, `import zellij`, `onboard`, `reset`, `reset config` |
| Runtime health | `doctor`, `inspect`, `status`, `whats_new` |
| Update owners | `update`, `update local_source`, `update upstream`, `update home_manager`, `update nix` |
| Desktop/cursors | `cursors`, `cursors ghostty setup`, `desktop`, `desktop install`, `desktop launch`, `desktop uninstall`, `desktop macos_preview install`, `desktop macos_preview uninstall`, `home_manager`, `home_manager prepare` |
| Workspace | `menu`, `popup`, `reveal`, `sidebar refresh`, `sidebar yazi` |
| Diagnostics | `dev`, `dev inspect_session`, `dev perf`, `dev profile` |
| Discovery/help | `keys`, `keys hx`, `keys helix`, `keys nu`, `keys nushell`, `keys yazi`, `keys yzx`, `tutor`, `tutor begin`, `tutor discovery`, `tutor hx`, `tutor helix`, `tutor list`, `tutor nu`, `tutor nushell`, `tutor tool_tutors`, `tutor troubleshooting`, `tutor workspace`, `why`, `sponsor`, `screen` |

Important options:

| Command | Contract |
| --- | --- |
| `yzx status --versions --json` | Runtime version matrix and generated-state summary. |
| `yzx inspect --json` | Runtime/config/install owner, command metadata, tool registry, session, and generated-state truth. |
| `yzx doctor --json` | Machine-readable health, plugin permissions, runtime assets, and generated-state findings. |
| `yzx doctor --fix-plan --json` | Non-mutating repair plan. |
| `yzx doctor --fix` | Apply Yazelix-owned safe generated-state repairs. |
| `yzx run <command> [args...]` | Execute through the Yazelix environment without launching UI. |
| `yzx import yazi [--force]` | Archive/copy native Yazi config, plugins, and flavors into managed editable input. |

Do not invent a `yzx sync` command when it is absent. "Sync Yazelix" means
complete the owner update plus generated-state convergence transaction below.
If a future profile exposes `yzx sync`, research its live help and incorporate
it without removing the owner-specific update proof.

## Mandatory update transaction

After any Yazelix source, flake, plugin, add-on, or child-package update:

1. Build and validate the changed source/package before profile mutation.
2. Publish child source first when the main flake consumes a child revision.
3. Update the main Yazelix lock to the published revision and validate without
   local overrides.
4. Inspect `yzx inspect --json` and `nix profile list --json`; select exactly
   one install owner.
5. Run one route:
   - local checkout profile: `yzx update local_source`
   - upstream profile: `yzx update upstream`
   - Home Manager: `yzx update home_manager`, then its printed
     `home-manager switch`
6. Run the upgraded profile frontdoor:
   - `yzx status --json`
   - `yzx inspect --json`
   - `yzx doctor --json`
7. If repair is indicated, run `yzx doctor --fix-plan --json`, then
   `yzx doctor --fix`, and repeat all three proof commands.
8. Prove a fresh session loads the upgraded runtime and connected plugins.
   Never hot-reload a newly packaged pane-orchestrator. An agent must not run
   `yzx restart` without explicit approval because it kills the live session;
   use a new window/session for proof or obtain the restart toggle.

The transaction is one mandatory task. An update that stops at source tests,
profile upgrade, or file existence is unfinished.

## Latest toolchain rule

- Resolve versions from the profile at execution time; never hard-code a stale
  version as the desired target.
- Use the newest available Nix/Yazelix/fenix/Bun-owned toolchain as the primary
  build and runtime lane.
- Treat MSRV, lockfile pins, and older compatibility versions as additional
  gates. They may not replace or downgrade the primary latest lane.
- Remove or archive PATH shadows under user-bin, rustup, `~/.cargo`, npm, npx,
  or global package roots. Fix the owner; do not make Nix match the shadow.
- A failed tool/version path is an exact affected gap. Continue all other
  required work and repair the gap; never disable the capability or test.

## Plugin and add-on consolidation

Required durable consolidation owner:

```text
/home/flexnetos/meta/src/yazelix-yazi-assets
```

All Yazelix plugin and add-on source, package, registry, and manifest authority
must converge there. The current topology is migration evidence:

| Current source | Required treatment |
| --- | --- |
| `yazelix-yazi-assets/plugins/*.yazi` | Already in the target owner; preserve licenses and package contracts. |
| main Yazelix `configs/yazi/plugins` | Migrate Yazelix-specific reusable plugin source; keep only runtime integration adapters in main. |
| `yazelix-helix` Steel defaults | Preserve the Helix fork, migrate Steel plugin/add-on assets and manifests to the consolidation owner. |
| `yazelix_helix_cogs_noop_wt` | Treat as an active main-Yazelix worktree and migration evidence, not a durable owner. Finish or merge legitimate work before cleanup. |
| Zellij bar/popup/pane-orchestrator child artifacts | Migrate plugin/add-on source/package authority while preserving standalone wasm package and ABI behavior. |
| ccboard and CodeDB runtime tools currently packaged by yazi-assets | Preserve as add-on runtime-tool contracts; they are not `.yazi` Lua plugins. |

Migration is strict upgrade-only:

1. Inventory source, licenses, package outputs, flake inputs, manifests, tests,
   and runtime consumers.
2. Add equivalent or stronger target-owner package contracts.
3. Prove child and integrated runtime behavior from the target owner.
4. Update Yazelix lock/package wiring and run the mandatory update transaction.
5. Remove a superseded owner only after patch/behavior representation proof.
6. Delete merged migration branches/worktrees under the GitHub execution policy.

Never copy code and leave two authorities. Never delete a working source first.

## Installed connectivity proof

Run `scripts/check-yazelix-contract.py --root <envctl-root> --live`, then verify:

| Plugin class | Required evidence |
| --- | --- |
| Yazi | Packaged and generated `.yazi` directories contain expected source files; configured plugins load in managed Yazi. |
| Helix Steel | Packaged `configs/helix/steel_plugins` plus healthy `yzx doctor helix-steel --json` and public command metadata. |
| Zellij | Packaged/generated `yazelix_pane_orchestrator.wasm`, `yzpp.wasm`, and `zjstatus.wasm`; doctor reports orchestrator permissions and managed pane connectivity. |
| Runtime add-ons | `yzx inspect --json` runtime tool registry reports packaged tools such as ccboard/CodeDB from the target owner. |

File presence is necessary but insufficient. Connection proof requires the
profile-owned runtime, generated materialization, permission state, and a fresh
session behavior check.
