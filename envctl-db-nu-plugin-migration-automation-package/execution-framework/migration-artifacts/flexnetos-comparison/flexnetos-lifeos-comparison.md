# FlexNetOS vs lifeos comparison evidence

- Task: `REQ-201_FLEXNETOS_LIFEOS_COMPARISON`
- Generated: `2026-07-04T23:30:59+00:00`
- Primary root exists: `True`
- Declared compare root: `/home/flexnetos/lifeos` exists=`False`
- lifeos peer root: `/home/flexnetos/FlexNetOS/src/lifeos` exists=`True`

## Finding

FlexNetOS was used as the active hollow workspace, control plane, environment authority, runtime/release lane, and migration artifact sink. lifeos was present as a peer Vue/Tauri product app under src/lifeos and as a release catalog bundle, not as the active filesystem root or control-plane service.

The artifacts show FlexNetOS was used for the operating/control-plane work that lifeos did not own: repo orchestration, runtime configuration, environment state, releases, local proof capture, and migration artifact generation. lifeos was consumed by that system as an application artifact.

## Role split

### FlexNetOS was used for
- Workspace root and peer-repo coordinator for the source fleet.
- Meta/gitkb control plane and policy surface.
- Envctl environment authority and state/catalog contract.
- Runner/release/provenance lane that packages tools and product bundles.
- Yazelix/Codex/local runtime proof surface and migration evidence sink.

### lifeos was used for
- Vue 3/Vite/Tauri desktop and web product app.
- Release catalog entry packaged through the FlexNetOS runner lane as a bun-tauri bundle.
- Planning-spine state/proof work inside its own repo branch.
- Future Rust workspace for lifeos-core, lifeos-daemon, and Tauri shell work.

## Artifact and code-map evidence

- Repository map: `403` repositories; scope rollup `{'envctl-control-plane': 7, 'meta-control-plane': 18, 'nix-packaging': 9, 'nu-plugin-surface': 1, 'python-codebase': 3, 'runner-runtime': 43, 'rust-codebase': 22, 'supporting-content': 5, 'terminal-runtime': 294, 'workspace-root': 1}`.
- lifeos repository entry: path `src/lifeos`, branch `codex/lifeos-planning-spine-v0`, head `6f8905d01701`, files `442`.
- Service graph: `16` nodes and `21` service edges; lifeos is not modeled as the active control-plane service.
- Debug code map summary: `{'entry_point_count': 160, 'control_flow_count': 160, 'external_call_count': 160, 'error_path_count': 160, 'log_signal_count': 160, 'metrics_alert_count': 160, 'runbook_signal_count': 160, 'hotspot_count': 80}`.

## Dependency and package evidence

- lifeos tree count: `534` files, `118` directories; skipped blocked paths `3`.
- Top suffixes: `{'.png': 122, '.md': 101, '.json': 101, '.js': 42, '.html': 26, '.vue': 25, '.ts': 18, '.rs': 18, '.toml': 9, '.jsx': 7, '.py': 7, '.css': 5, '.sh': 5, '[none]': 4, '.yml': 4, '.csv': 4, '.svg': 4, '.lock': 3, '.sql': 3, '.jsonl': 3}`.
- Release catalog lifeos rows: `[{'component': 'lifeos', 'kind': 'bun-tauri', 'source': 'src/lifeos', 'manifest': 'src/lifeos/package.json', 'bins': 'lifeos', 'asset_profile': 'lifeos-bundle', 'notes': 'LifeOS desktop app bundle for the local Ubuntu release lane (defaults to deb).'}]`.
- lifeos git state: branch `codex/lifeos-planning-spine-v0`, head `6f8905d01701`.

## Source line evidence

### flexnetos_agents
- L3: `This is the first file to read when working under `/home/flexnetos/FlexNetOS`.`
- L4: `The workspace root is a hollow orchestration area, not a monorepo. Real`
- L5: `implementation work belongs in independent peer repositories under `src/`.`
- L6: ``
- L7: `## Canonical Repositories`
- L8: ``
- L9: `- `/home/flexnetos/FlexNetOS/src/yazelix`: FlexNetOS-owned Yazelix foundation repo.`
- L10: `- `/home/flexnetos/FlexNetOS/src/meta`: FlexNetOS-owned meta repo.`
- L11: `- `/home/flexnetos/FlexNetOS/src/envctl`: environment authority repo.`
- L12: `- `/home/flexnetos/FlexNetOS/src/flexnetos_runner`: release runner/provenance repo.`
- L13: `- `/home/flexnetos/FlexNetOS/src/upstream/<owner>/<repo>`: upstream evidence mirrors only.`
- L14: ``
- L15: `Yazelix and meta are both organization-owned peer repos in this workspace. Do`
- L16: `not place them under `src/upstream/` unless creating a separate comparison clone.`
- L17: ``
- L18: `## Meta Control-Plane Rules`

### flexnetos_layout
- L4: ``
- L5: `This is a hollow workspace for the fresh `flexnetos` user. The workspace root is not a monorepo, vendor directory, or submodule container.`
- L6: ``
- L7: `## Canonical Roots`
- L8: ``
- L10: `\|---\|---\|---\|`
- L11: `\| `FLEXNETOS_WORKSPACE` \| `/home/flexnetos/FlexNetOS` \| Top-level hollow workspace root. \|`
- L12: `\| `FLEXNETOS_SRC` \| `/home/flexnetos/FlexNetOS/src` \| Independent peer repository checkouts. \|`
- L13: `\| `META_ROOT` \| `/home/flexnetos/FlexNetOS/src/meta` \| FlexNetOS/meta checkout after T011. \|`
- L18: ``
- L19: `Primary FlexNetOS peer repos:`
- L20: ``
- L26: ``
- L27: `External upstream peer repos, when needed for inspection or pinned source evidence:`
- L28: ``
- L51: `\| `/home/flexnetos/FlexNetOS/var/lib/gitkb` \| GitKB local state target after GitKB install/init. \|`

### release_catalog_lifeos
- L25: `lifeos	bun-tauri	src/lifeos	src/lifeos/package.json	lifeos	lifeos-bundle	LifeOS desktop app bundle for the local Ubuntu release lane (defaults to deb).`

### lifeos_readme
- L1: `# LifeOS (ubuntu-lifeos)`
- L2: ``
- L3: `> **LifeOS — the AI agent that runs your home and your life.**`
- L4: `> Work, personal, and home automation in one operating system, with the assistant baked into every surface.`
- L5: ``
- L6: `Cross-platform desktop app built on **Vue 3 + Vite + Pinia + vue-router** with a **Tauri 2** native shell. Targets Linux / macOS / Windows desktop, with a web build that runs in Firefox / Chrome / Edge / mobile browsers.`
- L7: ``
- L8: `This repo is the **production implementation** of the [LifeOS Design System](./design-system-reference/README.md) handoff bundle. The bundle's authoritative tokens, fonts, brand assets, and component-level CSS are folded into this repo so it is fully self-contained.`
- L9: ``
- L20: `\| rustc / cargo \| 1.95.0 \|`
- L21: `\| tauri-cli \| 2.11.1 \|`
- L22: ``
- L26: ``
- L27: `# Web dev (Vite) — http://localhost:1420`
- L28: `bun run dev`
- L29: ``

### lifeos_package
- L1: `{`
- L2: `  "name": "lifeos-vue",`
- L3: `  "private": true,`
- L5: `  "type": "module",`
- L6: `  "description": "LifeOS — Vue 3 + Tauri UI kit (Path B). Sibling of ui_kits/lifeos_app/ React canon.",`
- L7: `  "scripts": {`
- L8: `    "dev": "vite",`
- L9: `    "build": "vue-tsc --noEmit && vite build",`
- L10: `    "preview": "vite preview",`
- L11: `    "planning-spine:verify": "bun scripts/verify-planning-spine.mjs",`
- L12: `    "test": "vitest run",`
- L13: `    "test:watch": "vitest",`
- L14: `    "test:coverage": "vitest run --coverage",`
- L15: `    "tauri": "tauri",`
- L16: `    "tauri:dev": "tauri dev",`
- L17: `    "tauri:build": "tauri build",`

### lifeos_cargo
- L1: `# LifeOS Cargo workspace`
- L2: `#`
- L3: `# Members:`
- L4: `#  - src-tauri/        — Tauri 2 desktop shell (binary + lifeos_lib staticlib/cdylib/rlib)`
- L5: `#  - crates/lifeos-core — portable Rust core (types, auth, MCP clients, sync protocols)`
- L6: `#`
- L9: `#                        own rust-toolchain.toml. Lives as a standalone sibling project,`
- L10: `#                        not a workspace member.`
- L11: `#`
- L12: `# The workspace exists to share the future `lifeos-daemon` bin crate and `lifeos-core``
- L13: `# across the desktop shell + headless Pi daemon without duplicating dependencies. The`
- L14: `# Tauri shell continues to be invoked via `cargo tauri dev` / `cargo tauri build` from`
- L15: `# the `src-tauri/` directory; cargo resolves up to this manifest as the workspace root.`
- L16: ``
- L17: `[workspace]`
- L18: `resolver = "2"`

## Secret exposure control

- The scanner read only selected safe evidence files and counted lifeos paths without reading blocked path categories.
- Blocked path policy: `['.env', '.git', '.venv', '__pycache__', 'node_modules', 'private_keys', 'secrets', 'target']`.
- Blocked suffix policy: `['.key', '.pem']`.
