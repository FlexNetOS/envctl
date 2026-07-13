# envctl

A first-class **meta** peer member — the fully-automated, **agentic environment manager for
the whole meta workspace**. It brings every tool, dependency, provider, vendor, CLI, and
config to a declared state and installs it **into meta** through envctl's canonical
system-shaped prefix (`$META_ROOT/{usr/bin,usr/lib,usr/share,etc,var/lib,var/cache,var/log,var/tmp,opt} plus XDG meta-home roots`), with
`.toolchains/` retained only as a legacy compatibility store for manager-specific roots.
There are **no system-depth or user-global installs**: anything meta uses lives in meta, portable
wherever meta is cloned. One Rust workspace: a shared engine, a CLI (`envctl`), and a native
egui desktop app (`envctl-gui`). It manages the environment declaratively — every tool is a
TOML **component** whose lifecycle hooks *wrap the proven bash* from the Desktop kit
(`yazelix-setup.sh`, `ubuntu-boot-repair.sh`, …) rather than rewriting it. Its deployment
target today is a GPU-aware dual-RTX-5090 Ubuntu 26.04 workstation.

## Verbs

| verb | what it does | default |
|---|---|---|
| `auto-detect` | read-only inventory: host, GPU (works pre-driver), tools, component drift | — |
| `install` | bring components to present+verified, in dependency order; **idempotent** | acts |
| `auto-fix` | repair broken/partial components | **dry-run** (`--apply`) |
| `reset` | uninstall + unwire back toward baseline; gates `--all/--confirm/--cascade/--purge` | **dry-run** (`--apply`) |
| `add-repo` | build any repo from source (as-is / cherry-pick / rename / **AI port-to-Rust**) + wire-in; `--connect` for a supervised agent session | **preview** (`--build`) |
| `graph` | dependency-DAG intelligence: summary, `--impact` blast-radius, `--why` paths, `--dot`/`--json` | — |
| `lock` | content-hashed `envctl.lock` (reproducible) + `--check` CI gate (exit 1 on drift) | writes |
| `doctor` | read-only health: writability, toolchains, sudo, UEFI/Secure-Boot, GPU, last-op | — |
| `migrate` | adopt legacy/global installs into the `$META_ROOT` FHS/XDG layout, preserve agent assets, protect shared meta substrates, and refuse unsafe purge | read-only (`apply --apply` materializes dirs) |

## Quick start

```bash
# Rust is required (latest nightly is the dev default; exact Rust 1.89.0 is the MSRV lane):
#   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && . "$HOME/.cargo/env"

cargo build -p envctl-engine -p envctl     # engine + CLI (zero system deps)
cargo run  -p envctl -- auto-detect        # read-only; safe to run anytime
cargo run  -p envctl -- auto-detect --json # machine-readable EnvReport
cargo run  -p envctl -- install bun --dry-run
cargo run  -p envctl -- reset boot-repair-dev      # dry-run by default
cargo run  -p envctl -- migrate scan               # migration/adoption inventory
cargo run  -p envctl -- migrate apply --apply      # materialize canonical META_ROOT FHS/XDG dirs
```

The manifest dir defaults to `./manifest` (override with `ENVCTL_MANIFEST_DIR`).

### META_ROOT FHS/XDG layout

envctl is the path authority for meta installs. Components and add-repo drop-ins should not
hand-spell ad hoc host paths; they should resolve through the engine layout and land under:

| purpose | canonical path |
|---|---|
| executable exposure | `$META_ROOT/usr/bin` |
| libraries | `$META_ROOT/usr/lib` |
| read-only shared data + generated drop-ins | `$META_ROOT/usr/share` |
| XDG data (desktop/icons/fonts/app contracts only) | `$META_ROOT/.local/share` |
| add-repo source/build store | `$META_ROOT/var/lib/envctl/repos` |
| durable envctl state/logs | `$META_ROOT/var/lib/envctl` / `$META_ROOT/var/log/envctl` |
| XDG state (app contracts only) | `$META_ROOT/.local/state` |
| envctl caches | `$META_ROOT/var/cache/envctl` |
| XDG caches (app contracts only) | `$META_ROOT/.cache` |
| temporary files | `$META_ROOT/var/tmp` |
| component prefixes | `$META_ROOT/opt/<component>` |

`$META_ROOT/.toolchains` is a compatibility prefix for existing manager homes
(`BUN_INSTALL`, `CARGO_HOME`, `RUSTUP_HOME`, `UV_*`, etc.) while manifests migrate to the
FHS/XDG meta-root tree. `envctl env --toolchains` exports both the canonical `ENVCTL_*`
paths and the legacy manager variables, with `$META_ROOT/usr/bin` first on `PATH`.
See [`docs/MIGRATION-ADOPTION.md`](docs/MIGRATION-ADOPTION.md) for the upgrade-only
scan/plan/apply/verify/purge contract, including why `loop_lib` and agent/Codex assets are
protected during adoption.

### Native GUI

The `envctl-gui` crate needs system dev libs (winit/glow + a native file dialog):

```bash
sudo apt-get install -y cmake libxkbcommon-dev libwayland-dev \
  libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libgl1-mesa-dev libgtk-3-dev
cargo run -p envctl-gui
```

Dashboard (live GPU/CPU/mem telemetry) · Components grid (install/fix per row) ·
Add-Repo form · Live Logs · Settings. The engine runs on a worker thread; the UI never blocks.

#### Install as a desktop app

To make the GUI a first-class desktop application (binary on `PATH`, app-menu
launcher, and icon — all user-scoped, no sudo), either run the packaging
installer directly:

```bash
bash packaging/install-desktop.sh            # build (release) + install for the current user
bash packaging/install-desktop.sh --no-build # install an already-built binary
bash packaging/install-desktop.sh --uninstall
```

or drive it through the engine like any other component:

```bash
cargo run -p envctl -- install desktop-app   # idempotent; reset removes the launcher + icon
```

It installs `$META_ROOT/usr/bin/envctl-gui`, an
`$META_ROOT/.local/share/applications/envctl-gui.desktop` launcher
(`Categories=System;Monitor;`), and a scalable icon under
`$META_ROOT/.local/share/icons/hicolor/scalable/apps/`. Re-running is a no-op;
`reset desktop-app` (or `--uninstall`) unwinds it.

## Status

**Phase 0 + a working `auto-detect`.** The workspace compiles green on the latest nightly and
under the exact Rust 1.89.0 MSRV check; the
read-only verb is fully implemented and validated on the live dual-5090 box (PCI-floor GPU
detection that works even before the driver loads). `install`/`reset`/`auto-fix`/`add-repo`
are wired end-to-end with the real safety machinery (fail-closed guards, dry-run defaults,
idempotent install, hardened add-repo), with their deeper behavior staged in
[`docs/ROADMAP.md`](docs/ROADMAP.md).

## Safety model (boot-repair discipline)

Destructive operations follow `ubuntu-boot-repair.sh`'s gold standard:
**resolve + re-verify, refuse on ambiguity, dry-run by default, back up before clobber,
never touch user data.** Guards (`UuidResolves` / `NotLiveDevice` / `NotMounted`) are
implemented **fail-closed** — when they can't prove an op is safe, they *refuse* (a unit
test enforces this). See [`docs/DESIGN-NOTES.md`](docs/DESIGN-NOTES.md).

## Layout

```
crates/engine/   # envctl_engine: Component model, Registry, the 5 verbs, detect, guards, GUI worker API
crates/cli/      # envctl
crates/gui/      # envctl-gui (eframe/egui)
manifest/        # declarative components (base.toml, cuda.toml, boot-repair.toml) + components.d/ drop-ins
assets/scripts/  # the proven Desktop kit, referenced verbatim by ShippedScript hooks
scripts/         # operational helpers and hermetic test fixtures (meta-fleet-sync fail-closed fleet sync, reaper, audits)
docs/            # ARCHITECTURE.md · ROADMAP.md · DESIGN-NOTES.md · MIGRATION-ADOPTION.md
```

Design produced by a multi-agent design swarm and adversarially reviewed; the applied
review fixes are listed in `docs/DESIGN-NOTES.md`.

## References and Acknowledgments 

- pivoshenko/kasetto
