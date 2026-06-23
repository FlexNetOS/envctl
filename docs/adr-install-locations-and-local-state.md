# ADR — Install-location map + local-state model (do we need a `.local` peer repo?)

**Status:** accepted (2026-06-23) · **Owner:** envctl · **Builds on:**
[adr-meta-tool-location-and-portability](adr-meta-tool-location-and-portability.md),
ADR-0006 (home/ overlay portability).

## Context

Owner audit question: *where are the CLI installs (codex/claude/kimi/gemini); where are the
toolchains/dependents and meta's `lib`/`bin` (cuda, cargo, kache, mise, yazelix, nushell, …);
and do we need a `.local` peer repo for meta?* This ADR is the authoritative answer, verified
against a live audit of the box on 2026-06-23.

The owner invariant (from the prior ADR) stands: **every tool/dotfile/`.local`/`lib`/`bin`
resolves inside `meta`; user-global (`~/.local`, `~/.cargo`, `~/.claude`) holds ONLY symlinks
into meta; no config hardcodes a meta path.**

## The install-location map (verified live)

Two declared roots, plus three sanctioned exceptions.

### Root 1 — `$META_ROOT/.toolchains/` : the meta-owned toolchain store
Declared by `crates/cli/src/main.rs::run_env` (`envctl env --toolchains`) and the `rustup`/`bun`
components in `manifest/base.toml`. **Gitignored** (`.gitignore:85`), not a git repo — it is
*durable runtime state regenerable from declaration* (`envctl install`).

| manager | prefix env var | path |
|---------|----------------|------|
| bun     | `BUN_INSTALL`            | `.toolchains/.bun` |
| cargo   | `CARGO_HOME`            | `.toolchains/cargo` |
| rustup  | `RUSTUP_HOME`           | `.toolchains/rustup` |
| uv      | `UV_TOOL_DIR` / `UV_PYTHON_INSTALL_DIR` | `.toolchains/uv/{tools,python}` |
| mise    | `MISE_DATA_DIR`         | `.toolchains/mise` (data dir; binary bundled by yazelix) |
| src builds | (add-repo)           | `.toolchains/src/<repo>` |

### Root 2 — `~/.local/bin/` : the canonical symlink farm
`crates/engine/src/install.rs::local_bin()`. Every managed CLI is a symlink here into Root 1 or
into a `meta/<repo>/target/release/<tool>` build. Live, all resolve as declared.

| tool | `~/.local/bin/<tool>` resolves to | declared in | meta-owned? |
|------|-----------------------------------|-------------|-------------|
| `codex`  | `meta/codex/codex-rs/target/release/codex` | `ai-clis.toml` codex-cli | ✅ source build |
| `gemini` | `.toolchains/.bun/.../@google/gemini-cli`   | `ai-clis.toml` gemini-cli | ✅ bun-global |
| `claude` | `~/.local/share/claude/versions/<v>`        | `ai-clis.toml` claude-code-cli | ⚠️ vendor self-updater (see exceptions) |
| `kimi`   | `~/.local/bin/kimi` (real file)             | `ai-clis.toml` kimi-cli | ⚠️ vendor installer (see exceptions) |
| `bun`    | `.toolchains/.bun/bin/bun`                  | `base.toml` bun | ✅ |
| `cargo`/`rustc`/`rustup` | `~/.cargo/bin/* → .toolchains/cargo/bin/rustup` | `base.toml` rustup | ✅ (`RUSTUP_HOME` now exported — this ADR) |
| `meta`/`envctl`/`weave`/`grit`/… | `meta/<repo>/target/release/<tool>` | `components.d/portability-links.toml` meta-tool-links | ✅ |
| `node`   | `~/.local/node/bin/node`                    | `base.toml` node-real (n8n carve-out) | ⚠️ user-scope tarball (see drifts) |

### Sanctioned exceptions (NOT relocated into meta, by design)
- **CUDA toolkit** — `/usr/local/cuda-13.3` (apt, `manifest/gpu.toml::cuda-toolkit`). System
  package; relocating a multi-GB apt toolkit into meta is not worth it. `CUDA_HOME` is wired
  dynamically in `~/.bashrc`.
- **yazelix / nushell (`nu`) / zellij / mise binary** — nix profile (`~/.nix-profile/bin`, nix
  store), `manifest/nix-yazelix.toml::yazelix`. nushell+mise+zellij are *bundled inside* the
  yazelix nix runtime — nix is the meta-external-but-reproducible owner here.
- **claude / kimi** — vendor `curl|bash` self-updating installers land in `~/.local/{bin,share}`.
  envctl declares + verifies them but does not own their binary layout (the installer self-updates).

## Decision

### Q3 — **No, meta does NOT need a `.local` peer repo.** A new git peer would be the wrong shape:

1. **`.toolchains/` must stay untracked.** It is large vendor trees, regenerable from the
   manifests by `envctl install`. The meta pattern is *declaration → reproduce*, not
   *snapshot the filesystem*. Tracking it would bloat git and duplicate the manifests.
2. **`~/.local/bin` is 100% derivable** — pure symlinks into meta, recreated by `envctl install`
   (`portability-links` + per-component link steps). Tracking it stores zero new information.
3. **Host-coupling.** meta is multi-machine; a tracked `.local` snapshot would pin one host's
   absolute paths/versions, fighting the very portability seam this ADR upholds. The `home/`
   overlay README already rules `~/.local/share/*` machine-local.
4. **The config side is already tracked** — the `home/` overlay (ADR-0006) is meta's
   "track local config in git, symlink outward" pattern. That is the legitimate `.local`-ish
   peer, and it already exists *inside envctl* (not as a separate repo).

**What is actually missing is not a repo but an install-state RECORD** — "component X is installed
at version Y, path Z." Today `manifest/envctl.lock` records only declaration hashes (all
`resolved=""` for built-ins) and `agent-env.lock` records skill/MCP hashes; neither captures
installed version/location. The right home for install-state is the **already-present `resolved`
field in `envctl.lock`** (populated for add-repo SHAs today) and/or **`envctl doctor --json`** —
extending an existing seam, not a new peer repo. (Deferred — see drifts; out of scope for this ADR.)

### Q1/Q2 — answered by the map above. meta's `lib`/`bin` = Root 1 (`.toolchains/`, the store)
+ Root 2 (`~/.local/bin`, the symlink farm) + per-repo `meta/<repo>/target/release`.

## This ADR's fix — `RUSTUP_HOME` seam

`envctl env --toolchains` exported `CARGO_HOME` but **not** `RUSTUP_HOME`, so
`eval "$(envctl env --toolchains)"` shells used the meta cargo home while rustup silently fell
back to `~/.rustup` — missing the meta-owned nightly/codegen-gcc toolchain. The `rustup` component
already set `RUSTUP_HOME=.toolchains/rustup` in its *hooks*; this pairs the *shell seam* with the
hooks. Added to both the shell and `--json` output, with a regression test
(`crates/cli/tests/env.rs`). Additive / upgrade-only.

## Identified drifts (not fixed here — routed)

| drift | matches owner pref | routing |
|-------|--------------------|---------|
| **kache / hurry / zccache** Rust compiler-cache wrappers NOT declared or installed | owner wants kache as the meta-owned cache path | **feature** — new `manifest` components (`feature-forge`); large, needs build/wire/lock |
| **`node` → `~/.local/node`** (user-scope tarball) while `node-via-bun` declares the meta path | toolchain meta-owned | `node-via-bun` fix exists on a branch, unmerged — promote/merge |
| **install-state record** (version/path) absent from locks | reproducibility | extend `envctl.lock` `resolved` field or `doctor --json` (future) |
| **claude / kimi** vendor self-updaters in user-global | meta-owned ideal | accept — vendor installer constraint; envctl verifies presence only |
