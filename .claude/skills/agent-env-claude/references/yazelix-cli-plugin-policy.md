# Yazelix (yzx) CLI, update, and plugin policy — claude half

Claude-adapted from the codex sibling's `agent-env-codex/references/yazelix-cli-plugin-policy.md`
(shared contract — keep the two halves aligned). Compact contracts live in the prompt/SKILL.md;
this file carries depth. Verified against live `yzx` v17.9, 2026-07-11 — live profile metadata is
newer authority.

## Live authority (profile frontdoor, never repo scripts / generated runtime)

`/home/flexnetos/.nix-profile/bin/yzx` → `…-lifeos-foundation-yzx/bin/yzx`. Editable input:
`~/.config/yazelix/`; generated proof (never hand-edit): `~/.local/share/yazelix/`. Discover the
live surface before acting: `yzx --version-full`, `yzx --help`, `yzx inspect --json`
(`command_metadata.commands` is the machine-readable registry).

## Verb surface (v17.9 snapshot — families)

- **Root/session**: `yzx`, `agent`, `enter`, `env`, `launch`, `restart`, `run`
- **Config/edit/import**: `config[· set·ui·unset]`, `edit[· config]`, `import[· helix·yazi·zellij]`,
  `onboard`, `reset[· config]`
- **Runtime health**: `doctor` (`--json`, `--fix-plan --json`, `--fix`), `inspect --json`,
  `status [--versions --json]`, `whats_new`, `dev[· inspect_session·perf·profile]`
- **Update owners**: `update`, `update local_source`, `update upstream`, `update home_manager`,
  `update nix`
- **Workspace/desktop**: `menu`, `popup`, `reveal`, `sidebar[· refresh·yazi]`, `desktop[· install·
  launch·uninstall]`, `cursors[· ghostty setup]`, `home_manager[· prepare]`
- **Discovery**: `keys[· hx·nu·yazi·yzx]`, `tutor …`, `why`, `sponsor`, `screen`

## Update = a transaction, NOT a mythical `yzx sync`

There is **no `yzx sync` command** — do not invent one. "Sync Yazelix" = the owner-update +
generated-state convergence transaction. After any yazelix source/flake/plugin/add-on/child-package
change:

1. Build + validate the changed source before profile mutation; publish child source first when the
   main flake consumes a child revision; update the main lock to the published rev.
2. `yzx inspect --json` + `nix profile list --json` → pick exactly ONE install owner.
3. Run one route — this box is a local checkout: **`yzx update local_source`** (else `update
   upstream` / `update home_manager` + its printed `home-manager switch`).
4. Prove with the upgraded frontdoor: `yzx status --json`, `yzx inspect --json`, `yzx doctor --json`.
5. If repair indicated: `yzx doctor --fix-plan --json` → `yzx doctor --fix` → re-prove.
6. Prove a fresh session loads the upgraded runtime + connected plugins. **Never `yzx restart`
   without operator approval** — it kills the live session (a human-wall-adjacent action); use a new
   window/session for proof. An update that stops at source tests / profile upgrade / file existence
   is unfinished.

## Plugin & add-on consolidation owner (single durable home)

`/home/flexnetos/meta/src/yazelix-yazi-assets` (FlexNetOS org SSH) is the ONLY durable owner for all
yazelix plugin/add-on source, package, registry, and manifest authority. Current topology is
migration evidence, not multiple owners: `yazelix-helix` (Steel defaults — preserve the Helix fork,
migrate Steel plugin/add-on assets to the owner), `yazelix_helix_cogs_noop_wt` (an active main-yazelix
worktree / migration evidence — finish or merge, not a durable owner), main-yazelix `configs/yazi/
plugins` + Zellij `.wasm` orchestrator/bar/popup child artifacts (migrate source authority, preserve
wasm/ABI). Migration is strict upgrade-only (inventory → add ≥-equivalent target contracts → prove
integrated behavior → rewire lock → remove a superseded owner only after patch/behavior-parity proof
→ reap merged branches/worktrees under the GitHub execution policy). Never copy code and leave two
authorities; never delete a working source first.

## Connectivity proof (presence is necessary, not sufficient)

`yzx doctor --json` is the live connectivity oracle. Required evidence by plugin class: **Yazi**
packaged/generated `.yazi` dirs load in managed Yazi; **Helix Steel** healthy Steel command surface +
grammars (doctor: "Helix runtime healthy with N grammars", "Managed Helix Steel command surface is
healthy"); **Zellij** `yazelix_pane_orchestrator.wasm`/`yzpp.wasm`/`zjstatus.wasm` packaged + doctor
reports orchestrator permissions + managed pane connectivity; **runtime add-ons** (`ccboard`/CodeDB)
present in `yzx inspect --json` tool registry from the target owner. Connection proof needs the
profile-owned runtime + generated materialization + permission state + a fresh-session behavior check.
