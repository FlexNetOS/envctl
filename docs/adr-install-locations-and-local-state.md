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
meta-owned — (c) recognized as a **host prerequisite** meta only detects/verifies (never owns).
(c) is not a free pass; see the Corrected classification below — after review, (c) covers only the
GPU driver + the OS build-floor (pre-meta host facts), and `/nix` is removable, not a permanent
exception.

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
| **libgccjit** (for `rustc_codegen_gcc`) | rust-lang/gcc (asset) · pinned by rustc_codegen_gcc | **SHIPPED** (TASK-0062, `libgccjit` component) — downloads the prebuilt CI `libgccjit.so` from the `rust-lang/gcc` release `master-${COMMIT}` whose `${COMMIT}` is read from rustc_codegen_gcc's own `libgccjit.version` (reproducible, matches the backend; not floating-latest) → `.toolchains/libgccjit/lib/libgccjit.so` (+ `.so.0` SONAME); exposed via the `GCC_PATH` env seam — runtime `.so`, no `~/.local/bin` symlink. **System GCC NOT required.** | MEDIUM |
| **nix** (Determinate, host root `/nix`) | DavHau/nix-portable | **COMPONENT SHIPPED — ADDITIVE** (TASK-0066, `nix-portable` component): the `DavHau/nix-portable` static binary (pinned via the releases API) → `.toolchains/nix-portable/bin/nix-portable` + `~/.local/bin` symlink. Gives bwrap-isolated nix (home-dir store, logical `/nix/store` preserved → binary cache works) with **no host root `/nix`**. The **destructive** migration (remove host `/nix`, re-provision yazelix off Determinate) is **DEFERRED to supervised TASK-0067** — it touches the owner's live interactive shell (yazelix runs from `/nix`). | MEDIUM |
| **CUDA toolkit** (apt → `/usr/local/cuda-13.3`) | NVIDIA (runfile) | `.run --silent --toolkit --toolkitpath=.toolchains/cuda --override` (toolkit → meta; **still needs root** for udev/pkgconfig side-paths). Conda-forge `nvidia-cuda-toolkit` = rootless alt | HARD |
| **Nsight (`nsys`/`nsys-ui`)** (`/etc/alternatives`) | NVIDIA (bundled in CUDA runfile) | lands under `--toolkitpath/nsight-systems`, or standalone `.run --prefix=.toolchains/nsight-systems` | MEDIUM |

### Corrected classification (owner review 2026-06-23 — neither prior "irreducible" claim held)

An earlier draft labelled the nvidia driver and the nix store "genuinely irreducible `system:`
cases." Owner review (and a rigorous why-pass) showed both were cop-outs. The accurate picture:

- **nvidia-open kernel driver — NOT a meta concern at all (pre-meta host prerequisite).** It was
  installed *before* meta existed, so meta does not own it regardless. meta's job is to
  **detect/verify GPU readiness** (`gpu.toml` verify hooks), never to install or "declare-own" the
  driver. (Aside, settled by hardware: RTX 5090 = Blackwell/sm_120 has *open* kernel modules only —
  proprietary kernel modules don't support Blackwell — and the open module is what's loaded. So the
  open-vs-closed question is moot; the point is meta has no ownership here.) Kernel modules are
  genuinely OS-global (`/lib/modules`, `modprobe`), but that's the host's affair, not a meta
  `system:` component.
- **Nix `/nix` store — meta-owned-ISOLATED, not system-depth and not irreducible.** Verified: a
  nix-built binary's ELF *program interpreter* + RUNPATH are absolute `/nix/store/<hash>/…` paths,
  and the store prefix is part of every derivation's hash identity (fingerprint includes
  `:/nix/store:`). So the **logical** `/nix/store` is inescapable — but a **root-owned `/nix` is
  not**. The correct meta strategy (owner, 2026-06-23) is to run nix fully **isolated inside a
  meta/home-owned sandbox** so the host never has a real `/nix`:
  - **`nix-portable` (bubblewrap-backed) — the chosen path.** A single static binary
    (`DavHau/nix-portable`) that bwrap-mounts a home-dir store (`~/.nix-portable`) as `/nix`
    inside a namespace. Keeps the **logical `/nix/store`** → the **binary cache still works** (no
    from-source rebuilds). `ls /nix` on the host returns not-found. ~0% compute overhead,
    ~20 ms one-shot shell-spawn cost. GPU passthrough for ghostty via `nixGL` (path-linker, no
    draw-call interception — 0% framerate cost on the dual RTX 5090s).
    - **Component SHIPPED (TASK-0066) — ADDITIVE only.** The meta-owned `nix-portable`
      component (`manifest/components.d/epic-h-toolchains.toml`) installs the static binary into
      `.toolchains/nix-portable` + `~/.local/bin/nix-portable`; it **never touches the host
      `/nix`**. The **destructive** migration — removing the host `/nix`, re-provisioning yazelix
      off Determinate nix, retiring `manifest/nix-yazelix.toml` id=`nix` — is **DEFERRED to
      supervised TASK-0067** (it mutates the owner's live interactive shell, which runs from
      `/nix`), mirroring the install-vs-risky-part split used by TASK-0054/0055.
  - **Verified on THIS box (Ubuntu 26.04, kernel 7.0.0-22):** `apparmor_restrict_unprivileged_userns=1`
    is ACTIVE → **raw** unprivileged userns is blocked (`unshare --user --map-root-user` →
    `uid_map: Operation not permitted`); `bwrap` works (0.11.1, sanctioned AppArmor profile).
    So with the knob as-is, `nix-portable` works and `nix-user-chroot` would not.
  - **But this box is single-admin, no-human-in-the-loop, full-agentic (owner, 2026-06-23):** that
    AppArmor restriction exists to protect multi-user / untrusted contexts that don't apply here —
    it's an **owner-tunable knob, not a wall.** The meta-correct handling is for **envctl to
    DECLARE the policy** (a sudo-phase component setting `apparmor_restrict_unprivileged_userns=0`,
    or installing the bwrap userns profile) so the host state is reproducible — not a hand-flipped
    sysctl. With the knob owned, BOTH nix-portable and nix-user-chroot are available; **nix-portable
    stays the recommended default on merit** (single static binary, no daemon, bwrap-isolated, what
    the owner's research recommends), not because the alternative is blocked.
  - **General principle (this box):** "system-depth" is NEVER gated by *permission* — the single
    admin has all of it. It is gated only by *meta ownership / reproducibility*. So every Epic-H
    convergence is blocked by *work to be done*, not by *can't* — including host-policy knobs and
    sudo-phase installs, which envctl declares and applies like any other component.
  - **Custom `store-dir=`** (relocating the *logical* prefix) is the wrong path: it **destroys the
    binary cache** (every artifact rebuilds from C source). Never use it.
  - **End-state:** nix exists on this box solely to deliver **yazelix** (nu/zellij/mise). Converging
    those (TASK-0058/0059) + de-nixing yazelix (TASK-0064) removes nix entirely. nix-portable is the
    immediate isolation (works today, no host `/nix`); the yazelix rust-core de-nix is the eventual
    full removal. Either way **nix is never a system-depth install.**
  - **yazelix status (verified, local checkout post-v17.7 / v17.7=2026-06-15):** `yzx` is now a
    standalone Rust binary (`rust_core/yazelix_core` → `[[bin]] yzx`) and the project is extracting
    its subsystems into standalone cargo crates — the direction is off-nix. BUT the current
    `docs/installation.md` still states "Yazelix requires Nix with flakes," and runtime *assembly*
    (zellij/yazi/helix/nu/mise → runtime tree) is still nix (`packaging/mk_runtime_tree.nix`).
    The non-nix install path is **in-flight, not yet released**. TASK-0064 tracks landing on it.

> `build-essential`/`cmake`/`pkg-config`/`libssl-dev`/system GCC are the OS **build-floor** every
> from-source convergence depends on — a host prerequisite (like the kernel/driver), not a meta
> component. meta detects them; it does not vendor the platform C toolchain.

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

## 2026-06-26 follow-up — registry, CA, and legacy-path cleanup

This follow-up answers the full-system install questions with current source truth (`envctl env
--toolchains --json`, `envctl registry --json`, `manifest/env-ctl.toml`, and the secrets CA seam):

| surface | authoritative location | cleanup / ownership rule |
|---|---|---|
| Component registry | `manifest/*.toml` plus `manifest/components.d/*.toml`, loaded by `Registry::load`; pinned by `manifest/envctl.lock` | `envctl lock --check` gates manifest drift. |
| Hub/tool registry | each `<name>_hub/registry.json` under the envctl workspace root; today `mcp_hub/registry.json` is discovered by `envctl registry --json` | read-only federation; `envctl registry --check` fails when a hub entry binds to a missing component. |
| Runtime/last-run state | `$XDG_CACHE_HOME/envctl/<hash-of-manifest-dir>/state.json` | machine-local, intentionally uncommitted; best-effort only. |
| Meta toolchain store | `$META_ROOT/.toolchains/` (`BUN_INSTALL`, `CARGO_HOME`, `RUSTUP_HOME`, `UV_*`, `MISE_DATA_DIR`, `LIBCLANG_PATH`, `GCC_PATH`, etc.) | gitignored and regenerated by `envctl install`; no `.local` peer repo. |
| CLI exposure | `~/.local/bin` symlinks into `$META_ROOT/.toolchains/...` or `meta/<repo>/target/release/...` | symlink farm only; reset removes envctl-owned links. |
| Secrets daemon binaries | `$META_ROOT/.toolchains/secrets/bin/{secretd,secretctl}` with `~/.local/bin/{secretd,secretctl}` symlinks | install/fix now cleans legacy `~/.cargo/bin/{secretd,secretctl}` only when proven safe: duplicate symlinks are removed, byte-identical regular files are archived under `~/.local/state/envctl/legacy-archives/`, and different/foreign binaries are left in place and surfaced by verify. |
| Secrets config | `~/.config/env-ctl/secretd.toml` | preserved unless `reset ... --keep-config` is not used; auth tokens are never stored here. |
| Secrets data/audit | `~/.local/share/env-ctl` and `~/.local/state/env-ctl` | data paths are deleted only by `envctl reset env-ctl --purge --confirm --apply`. |
| Cognitum Seed Device CA pin | `$META_ROOT/.toolchains/secrets/ca/cognitum-ca.crt` | `env-ctl.service` exports `ENVCTL_SEED_CA` to this path; the code fallback now also uses the meta prefix instead of legacy `/usr/local/share/ca-certificates`. |
| Local MITM/remote-client CAs | sealed in the encrypted vault store (`ca_key`/`certs` rows and `mitm.ca_cert_der` / `remote_clients.ca_cert_der` metadata) | private keys never leave the daemon; public trust apply remains explicit/dry-run by default (`envctl secret ca trust ... --apply --confirm`). |

There is no generic “delete everything old” command by design: strict upgrade-only cleanup happens
inside the component that owns the migration, after the new meta-owned path exists and equivalence is
proven. `reset` is the reversible component removal path; `agent clean` is only for agent assets;
foreign/different legacy binaries are reported rather than deleted.

## Epic H — eliminate system-depth installs (carded)

Each row of the convergence plan becomes an envctl component (new `manifest/*.toml` or a
`.meta.yaml` peer source-build) plus a `meta-tool-links`/`.toolchains` install. Sequenced
EASY → HARD; all dry-run-safe by default (apply on the box via `envctl install`/`auto-fix`,
driven by `env-install-loop`). The GPU driver + OS build-floor are host prerequisites (detect/
verify only, never meta-owned); `/nix` is removed by finishing the yazelix de-nix (TASK-0064), not
declared a permanent exception. Backlog: `.handoff/loop/backlog.md` Epic H.
