---
id: 019f2adb-59da-76c2-9164-1374e35534f9
slug: tasks/import-host-artifacts-yazelix-mission-control-20260703
title: "Import host artifacts from 2026-07-03 yazelix mission-control session"
type: task
status: active
priority: medium
tags: [nu-plugin, codedb, yazelix, host-artifacts, mission-control, ccboard]
---

## Overview

On 2026-07-03 an interactive session with Claude Code set up ccboard (FlorianBruniaux/ccboard) as a Mission Control tab and popup inside the yazelix flexnetos foundation runtime. The changes on the yazelix source side landed as FlexNetOS/yazelix#9 (branch codex/mission-control-ccboard-and-selfheal), but several files that back the same end-to-end experience live outside any git repo. Those files currently sit only in the host user tree and would be lost or drift silently unless envctl's nu_plugin import workflow captures them as declarative rows in the envctl catalog.

## Files to import

### 1. Shell environment surface (host user)

- ~/.bashrc — appended a "FlexNetOS yazelix rebuild env" block exporting FLEXNETOS_GIT_KB_PATH, FLEXNETOS_RTK_PATH, FLEXNETOS_CCBOARD_PATH pointing at their canonical live paths. NIXPKGS_ALLOW_UNFREE is intentionally NOT exported globally.
- ~/.config/nushell/env.nu — new file. Sets the same three FLEXNETOS_*_PATH env vars via nu record assignments and prepends ~/.nix-profile/bin to \$env.PATH so claude / codex / yzx resolve via the stable profile symlink instead of a session-baked store hash.

### 2. Yazelix runtime user config (host user)

- ~/.config/yazelix/settings.jsonc — added a second entry to zellij.custom_popups with id="ccboard", command=["ccboard"], keybindings=["Alt Shift D"], keep_alive=true. Everything else in the file was left untouched. This is the counterpart to the source-side layout change shipped in yazelix#9.
- ~/.config/yazelix/zellij.kdl — new managed native sidecar. Sets scrollback_lines_to_serialize 5000 (yazelix already enforces session_serialization true and serialize_pane_viewport true, so those keys are omitted). Merged into ~/.local/share/yazelix/configs/zellij/config.kdl at materialization time.

### 3. Desktop integration (host user)

- ~/.local/share/applications/com.flexnetos.Yazelix.Agent.desktop — Exec line rewritten to use ~/.nix-profile/configs/zellij/layouts/flexnetos_agent_workspace.kdl for YAZELIX_LAYOUT_OVERRIDE (was hardcoded to /home/flexnetos/FlexNetOS/src/yazelix/configs/zellij/layouts/…). The X-FlexNetOS-Managed=true marker keeps install_ownership_report from touching it.

### 4. Rebuild / doctor scripts (host user)

- ~/.local/bin/flexnetos-rebuild-yazelix — new bash script. Resolves ~/.nix-profile/toolbin/{git-kb,rtk} via readlink -f (nix rejects symlinks in the derivation src slot), sets FLEXNETOS_CCBOARD_PATH=~/FlexNetOS/usr/bin/ccboard, invokes nix profile upgrade --impure yazelix_flexnetos_foundation with NIXPKGS_ALLOW_UNFREE=1 scoped to the call, then invalidates ~/.local/share/yazelix/state/rebuild_hash and runs command yzx doctor --fix so the source-side initializer self-heal fires.
- ~/.local/bin/flexnetos-doctor — new bash script. Wraps command yzx doctor "\$@" to bypass any session-level shell shadowing.

### 5. Locally built binary (host staging, out of any repo)

- /home/flexnetos/FlexNetOS/usr/bin/ccboard — v0.24.0 built from FlorianBruniaux/ccboard main (Downloads ccboard-main.zip extracted to /tmp/ccboard-build/ccboard-main). Compiled with the fenix rust-mixed toolchain at /nix/store/b47aazvj6hmsd1i1a6sy9ch5yx8ylvxg-rust-mixed/bin/{cargo,rustc} and RUSTC_WRAPPER=/home/flexnetos/FlexNetOS/usr/bin/kache-rustc-wrapper. Wild linker was tried but dropped because gcc rejects absolute paths for -fuse-ld=<path>. The yazelix flake's packaging/ccboard_local_binary.nix picks this up via FLEXNETOS_CCBOARD_PATH.

## Goals

- Bring all host-side artifacts above into envctl catalog rows via the nu_plugin import workflow, so the same files can be reproduced from envctl declarative sources on a fresh install.
- Preserve provenance: which artifacts are source-owned (rebuilt from repo checkouts), which are generated runtime state, and which are one-off host tweaks that should become table-owned.
- Ensure NIXPKGS_ALLOW_UNFREE remains scoped (not globally exported) after the import — it should be a table row for the rebuild script's scope, not an ambient shell setting.
- Handle the ccboard binary provenance: envctl should record the source commit hash it was built from (upstream FlorianBruniaux/ccboard @main SHA at build time) and the toolchain fingerprint (rust-mixed store path + kache wrapper path), so the row is auditable and a rebuild can regenerate a byte-comparable binary.

## Acceptance criteria

- [ ] All eight files above are registered in envctl catalog tables with type + owner + provenance metadata.
- [ ] The Bun/kache/wild toolchain identification lands in a companion toolchain table so future ccboard-style bundles can reuse the recipe.
- [ ] The rebuild script (~/.local/bin/flexnetos-rebuild-yazelix) can be re-emitted from envctl rows on a fresh install; if envctl exposes a render step, that path is documented on this task.
- [ ] The nu_plugin import command surface used to bring these files in is documented in the task progress log, not just executed.
- [ ] Cross-references: this task links to the merged yazelix PR (see references below) and to tasks/envctl-codex-mcp-runtime-import which follows the same file-into-envctl pattern for the Codex MCP surface.

## References

- FlexNetOS/yazelix#9 — the yazelix-side changes (Mission Control tab, ccboard packaging via _local_binary.nix, claude routed through rtk, doctor --fix regenerates shell initializers, CLAUDE.md toolchain docs).
- [[tasks/envctl-codex-mcp-runtime-import]] — earlier precedent for importing user-config surfaces (Codex config, MCP descriptors) via nu_plugin.
- [[expand-codedb-nu-plugin-coverage-beyond-file-impor]] — umbrella task for broadening nu_plugin import coverage.
- Upstream ccboard project: github.com/FlorianBruniaux/ccboard (v0.24.0 main-branch source).

## Progress log

### 2026-07-03

- Task created from the yazelix mission-control session.
- Files enumerated but not yet imported. Next envctl session should run the nu_plugin import workflow against the eight paths listed above, starting with the two brand-new files (~/.local/bin/flexnetos-rebuild-yazelix and ~/.local/bin/flexnetos-doctor) as the smallest slice.