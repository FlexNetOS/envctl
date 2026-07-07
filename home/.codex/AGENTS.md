@/home/flexnetos/.codex/RTK.md
@/home/flexnetos/.codex/AGENTS.rtk.md

## FlexNetOS Runtime Frontdoor

When working under `/home/flexnetos/FlexNetOS`, read
`/home/flexnetos/FlexNetOS/AGENTS.md` before acting. For Yazelix/Codex installed
runtime behavior, the active frontdoor is the Yazelix/Nix profile path, not raw
repo source:

```text
/home/flexnetos/.nix-profile/bin/yzx
/home/flexnetos/.nix-profile/configs/zellij/layouts/flexnetos_agent_workspace.kdl
```

The visible FlexNetOS Agent desktop entry should launch the profile `yzx` with
that profile layout override unless the install owner deliberately changes.
Treat `src/yazelix/...` as product-development input that must be consumed
through the Yazelix/Nix profile owner before it proves installed behavior.

Yazelix path ownership is explicit and Codex must follow it:

```text
editable user input: /home/flexnetos/.config/yazelix/
generated runtime:   /home/flexnetos/.local/share/yazelix/
active frontdoor:    /home/flexnetos/.nix-profile/bin/yzx
```

Codex binary and runtime ownership must mirror the Yazelix binary/runtime model
with no drift in ownership assumptions:

```text
Codex editable input:   /home/flexnetos/.config/yazelix/   (same ownership model)
Codex generated output: /home/flexnetos/.local/share/yazelix/  as runtime proof
Codex active frontdoor: /home/flexnetos/.nix-profile/bin/yzx
remove shadows:         /home/flexnetos/.local/bin/yzx
                        /home/flexnetos/.local/share/applications/* stale launchers
```

Ownership model Yazelix documents:

- `~/.config/yazelix/...` is the user/config-source editable input surface, including `settings.jsonc` and managed overrides, as documented in `/home/flexnetos/FlexNetOS/src/yazelix/docs/customization.md:5` and `/home/flexnetos/FlexNetOS/src/yazelix/docs/posix_xdg.md:21`
- `~/.local/share/yazelix/...` is Yazelix-generated runtime output; "edit the config inputs, not generated runtime files" appears in `/home/flexnetos/FlexNetOS/src/yazelix/docs/customization.md:7`, and `README.md` calls it Yazelix-owned output at `/home/flexnetos/FlexNetOS/src/yazelix/README.md:133`
- `~/.nix-profile/bin/yzx` is the install-owner/profile frontdoor, documented at `/home/flexnetos/FlexNetOS/src/yazelix/home_manager/README.md:306`
- `~/.local/bin/yzx` is a stale legacy shadow if present and should be removed or archived when it shadows the profile-owned command, documented at `/home/flexnetos/FlexNetOS/src/yazelix/home_manager/README.md:310`
- Old user-local desktop entries under `~/.local/share/applications/` are stale shadows if they shadow the active profile entry, documented at `/home/flexnetos/FlexNetOS/src/yazelix/home_manager/README.md:309` and `/home/flexnetos/FlexNetOS/src/yazelix/docs/troubleshooting.md:187`

Additional direct proof to preserve:

- Yazelix "keeps user-edited config separate from generated runtime output" at `/home/flexnetos/FlexNetOS/src/yazelix/README.md:282`
- Generated runtime output includes generated Zellij/Yazi/Helix/terminal configs under `~/.local/share/yazelix` at `/home/flexnetos/FlexNetOS/src/yazelix/README.md:285` and `/home/flexnetos/FlexNetOS/src/yazelix/docs/posix_xdg.md:31`
- Generated Zellij runtime files are not a manual edit surface at `/home/flexnetos/FlexNetOS/src/yazelix/docs/zellij-configuration.md:53`
- The runtime-root contract warns against treating generated state as handwritten config at `/home/flexnetos/FlexNetOS/src/yazelix/docs/contracts/runtime_root_contract.md:20`

Do not hand-edit generated runtime files under `~/.local/share/yazelix`.
Change the owning config inputs under `~/.config/yazelix`, rebuild/relaunch
through the install owner, and use the generated runtime tree only as proof of
what the active install materialized.

When stale local shadow paths are present, remove or archive them instead of
treating them as active ownership layers. In particular, stale
`~/.local/bin/yzx` wrappers and stale user-local desktop entries under
`~/.local/share/applications/` can shadow the profile-owned install and should
not be kept as parallel "just in case" control paths.

## Runtime Plugin And MCP Narrowing

The active runtime authority for Codex plugin and MCP breadth is
`/home/flexnetos/.codex/config.toml`. Do not widen that surface from workspace
mirrors, envctl catalog renders, cached plugin bundles, or temp marketplace
materialization under `~/.codex/.tmp/plugins/`.

Unless the owner explicitly requests otherwise, preserve removal of noisy or
duplicate plugin/catalog surfaces. In particular, do not re-add or infer active
use of `superhuman`, `digitalocean`, `openai-curated` marketplace fanout, or
duplicate command/skill inventories merely because they remain present in a
marketplace source tree or cache.

## Operating Rules Source

The durable operating rules live in `/home/flexnetos/.codex/RULES.md`. For
LifeOS, Planning Spine, task-graph, pipeline-integrity, Google planning surface,
or GitHub-protocol work, read and follow that file before acting. Its rules are
additive to this AGENTS contract and include the main north-star rule, the
Yazelix/Nix foundation model, source-provenance and proof-ledger requirements,
and the clean GitHub finish-state policy below.

## GitHub And Branch Hygiene

For any GitHub, PR, merge, publish, or repo-cleanup work, do not end the turn
with a dirty branch, an open PR that can be merged, or stale merged PR branches
left behind. Before the final response, prove the target repo status with
`git status --short --branch`, verify open PR inventory with
`gh pr list --state open`, confirm merge/check state for PRs touched, and delete local/remote
branches for PRs that are already merged when safe.

If self-hosted runners or generated repo state dirty the tree after checks or
merge, settle the state through the repo-owned command or policy, archive or
ignore generated artifacts deliberately, commit and push the resulting state
when appropriate, and re-run the clean-status and open-PR proof. If a direct
state-only `[skip ci]` push is used to break a runner self-dirty loop, report it
explicitly with proof. Do not call the work done while active runner state,
stashes, temp worktrees, or unmerged cleanup PRs remain unresolved.
