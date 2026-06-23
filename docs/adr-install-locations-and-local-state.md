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

**Corrected doctrine (owner, 2026-06-23):** there are **NO sanctioned system-depth exceptions.**
meta and its peers use no system-depth installs (apt `/usr`, `/usr/local`, the nix `/nix` store,
the kernel) — and neither should any peer. Every current system-depth install has an upstream repo
and MUST be either (a) installed at a meta-directed prefix (`$META_ROOT/.toolchains/<x>` via
tarball / `cargo install --root` / runfile `--toolkitpath`), or (b) cloned + added as a source
repo (`.meta.yaml` peer / `add-repo` source build), or — only when it physically cannot be
meta-owned — (c) declared as an explicit **irreducible `system:` component** with written
rationale. (c) is not a free pass; it is reserved for the two cases proven irreducible below.

## The install-location map (verified live)

Two declared roots; system-depth installs are tracked as drift in the convergence plan (below).

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
| `claude` | `~/.local/share/claude/versions/<v>`        | `ai-clis.toml` claude-code-cli | ⚠️ vendor self-updater (user-scope, not system) |
| `kimi`   | `~/.local/bin/kimi` (real file)             | `ai-clis.toml` kimi-cli | ⚠️ vendor installer (user-scope, not system) |
| `bun`    | `.toolchains/.bun/bin/bun`                  | `base.toml` bun | ✅ |
| `cargo`/`rustc`/`rustup` | `~/.cargo/bin/* → .toolchains/cargo/bin/rustup` | `base.toml` rustup | ✅ (`RUSTUP_HOME` now exported — this ADR) |
| `meta`/`envctl`/`weave`/`grit`/… | `meta/<repo>/target/release/<tool>` | `components.d/portability-links.toml` meta-tool-links | ✅ |
| `node`   | `~/.local/node/bin/node`                    | `base.toml` node-real (n8n carve-out) | ⚠️ user-scope tarball (see drifts) |

## System-depth convergence plan (the drift, and how each is meta-owned)

Live audit (2026-06-23) of every system-depth install meta touches, its upstream repo, and the
meta-owned method. Tier: **EASY** = Rust/Go `cargo/go install --root` or a relocatable tarball;
**MEDIUM** = C/C++ cmake/make-to-prefix or prebuilt-with-side-libs; **HARD** = huge build or
genuinely irreducible.

| install (current system location) | upstream repo | meta-owned method (→ `.toolchains/<x>` + `~/.local/bin` symlink) | tier |
|---|---|---|---|
| **mold** (apt → `/usr/bin/mold`) | rui314/mold | **replace with `wild`** (owner pref) — `cargo install --locked wild --root .toolchains/wild`; wire via `RUSTFLAGS --ld-path`. (mold fallback: release tarball → `.toolchains/mold`) | EASY |
| **wild** (preferred linker, not yet installed) | davidlattimore/wild | `cargo install --locked wild --root .toolchains/wild` | EASY |
| **kache / hurry / zccache** (owner-preferred cache, not installed) | (owner/meta tooling) | source-build into `.toolchains/<x>` (carried from prior ADR drift) | EASY–MEDIUM |
| **gh** (apt → `/usr/bin/gh`) | cli/cli | release tarball → `.toolchains/gh` + symlink | EASY |
| **nushell `nu`** (nix profile) | nushell/nushell | musl static release tarball → `.toolchains/nushell` (or `cargo install nu --root`) | EASY |
| **zellij** (nix profile) | zellij-org/zellij | musl static release tarball → `.toolchains/zellij` | EASY |
| **mise** (bundled in yazelix) | jdx/mise | static binary → `.toolchains/mise/bin`; `MISE_DATA_DIR` already meta | EASY |
| **ollama** (`/usr/local/bin/ollama`, no peer) | ollama/ollama | prebuilt binary → `.toolchains/ollama/bin` + redirect GPU `.so` via `OLLAMA_LIBRARY_PATH` | MEDIUM |
| **archon** (`/usr/local/bin/archon`, real binary) | FlexNetOS/Archon (**already a `.meta.yaml` peer**) | DRIFT — just symlink `~/.local/bin/archon → meta/Archon/target/release/archon` via `meta-tool-links` (same as `vox`) | EASY |
| **clang / llvm-21** (apt) | llvm/llvm-project | **prebuilt** `clang+llvm-*-x86_64-linux` tarball → `.toolchains/llvm` (source build is 30–50 GB, impractical) | MEDIUM |
| **libgccjit** (for `rustc_codegen_gcc`) | rust-lang/rustc_codegen_gcc | `y.sh` downloads the CI `libgccjit.so`; place at `.toolchains/libgccjit/lib` — **system GCC NOT required** | MEDIUM |
| **CUDA toolkit** (apt → `/usr/local/cuda-13.3`) | NVIDIA (runfile) | `.run --silent --toolkit --toolkitpath=.toolchains/cuda --override` (toolkit → meta; **still needs root** for udev/pkgconfig side-paths). Conda-forge `nvidia-cuda-toolkit` = rootless alt | HARD |
| **Nsight (`nsys`/`nsys-ui`)** (`/etc/alternatives`) | NVIDIA (bundled in CUDA runfile) | lands under `--toolkitpath/nsight-systems`, or standalone `.run --prefix=.toolchains/nsight-systems` | MEDIUM |

### The two genuinely irreducible `system:` cases (proven, not rubber-stamped)
- **nvidia-open kernel driver** (apt `nvidia-driver-595-open` + DKMS, `gpu.toml`) — repo
  NVIDIA/open-gpu-kernel-modules. Kernel `.ko` modules must build against the running kernel's
  headers and load into `/lib/modules/`; the module subsystem is OS-global and `modprobe` has no
  user-prefix. **No meta-prefix path exists.** Declare as `system:` with a `verify` hook; install
  stays apt/DKMS.
- **Nix store `/nix`** (Determinate installer, `nix-yazelix.toml`) — the `/nix/store` path is
  hardcoded into every derivation's content hash + ELF RPATH; relocating invalidates the whole
  store. **Non-relocatable by design.** Declare as `system:`. (The tools nix *delivers* —
  nushell/zellij/mise — are converged above, which removes nix as a *dependency path* for them;
  **yazelix** itself is a nix-flake meta-config, not an app — converging it means decomposing it
  into its config (already in `home/.config/yazelix`) + the separately-installed component
  binaries. Larger effort, carded in Epic H.)

> Note `build-essential`/`cmake`/`pkg-config`/`libssl-dev` are the OS build foundation; they are
> the bootstrap floor every from-source convergence above depends on. They are tracked as the
> `system:` build-floor (a third irreducible-in-practice class), not silently ignored.

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

**Operational note — rustup is currently split-brain (audit 2026-06-23).** Because the seam never
exported `RUSTUP_HOME`, toolchains installed via interactive shells (incl. the GPU component's
pinned **`nightly-2026-04-03`** + `rust-src`/`rustc-dev`/`llvm-tools`) landed in `~/.rustup`, while
the meta store `.toolchains/rustup` holds only `{1.96.0, nightly, stable}`. After this seam lands
and `envctl` is rebuilt, interactive shells point rustup at the meta store, which does **not** yet
contain `nightly-2026-04-03`. Convergence is automatic and safe: the GPU component's `detect`
misses against the meta store, so `envctl install`/`auto-fix` re-installs it INTO the meta store
(or rustup downloads on demand). No data lost — `~/.rustup` left intact. One-time full convergence:
`envctl install`.

## Identified drifts (routed)

| drift | matches owner pref | routing |
|-------|--------------------|---------|
| **system-depth installs** (mold/gh/nushell/zellij/mise/ollama/llvm/CUDA-toolkit/Nsight + archon relink) | no system-depth installs | **Epic H** (below) — per-item meta-prefix/clone+add components |
| **rustup split-brain**: GPU-pinned `nightly-2026-04-03` lives in `~/.rustup`, not meta store | toolchain meta-owned | one-time `envctl install` after the RUSTUP_HOME seam lands; see Operational note |
| **kache / hurry / zccache** cache wrappers not installed | owner wants kache as the meta cache path | folded into Epic H |
| **`node` → `~/.local/node`** while `node-via-bun` declares the meta path | toolchain meta-owned | `node-via-bun` fix on an unmerged branch — promote |
| **install-state record** (version/path) absent from locks | reproducibility | extend `envctl.lock` `resolved` field / `doctor --json` (future) |
| **claude / kimi** vendor self-updaters in `~/.local` (user-scope, not system) | meta-owned ideal | accept — vendor installer constraint; user-scope ≠ system-depth |

## Epic H — eliminate system-depth installs (carded)

Each row of the convergence plan becomes an envctl component (new `manifest/*.toml` or a
`.meta.yaml` peer source-build) plus a `meta-tool-links`/`.toolchains` install. Sequenced
EASY → HARD; all dry-run-safe by default (apply on the box via `envctl install`/`auto-fix`,
driven by `env-install-loop`). The two irreducible `system:` cases (nvidia-open, `/nix`) are
declared, not converted. Backlog: `.handoff/loop/backlog.md` Epic H.
