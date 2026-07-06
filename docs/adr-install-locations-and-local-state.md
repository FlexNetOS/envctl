# ADR — Install-location map + local-state model

**Status:** accepted (2026-06-23), amended 2026-06-26 · **Owner:** envctl · **Builds on:**
[adr-meta-tool-location-and-portability](adr-meta-tool-location-and-portability.md), ADR-0006
(home/ overlay portability).

## 2026-06-26 amendment: META_ROOT is the install root

`envctl` is the path authority for meta installs. All envctl-owned tools, application payloads,
frontdoors, support files, state, cache, and temporary files must resolve under:

```text
$META_ROOT/{usr/bin,usr/lib,usr/share,etc,var/lib,var/cache,var/log,var/tmp,opt} plus XDG meta-home roots
$META_ROOT/.toolchains/        # legacy compatibility prefix while older manifests migrate
$META_ROOT/.config/            # tracked/configured dot roots that are intentionally meta-hosted
```

Yazelix owns the active Nix profile under the real user-home `.local` state tree:

```text
$ENVCTL_REAL_HOME/.nix-profile -> $ENVCTL_REAL_HOME/.local/state/nix/profiles/profile
```

Envctl must not replace that whole `.local` tree. It may archive known per-tool user-bin shadows
after the replacement frontdoor exists in the Yazelix profile or a META_ROOT-owned prefix. No
component may install into any real-home/user-home/systemd-home local-bin spelling, or any
leading-tilde local path. Managed hooks run with `HOME=$META_ROOT`; hook bodies that truly need
the real user home must use `ENVCTL_REAL_HOME` and must document why.

## Install-location map

### Root 1 — `$META_ROOT/{usr,etc,var,opt}` plus meta-XDG roots: canonical envctl install tree

`crates/engine/src/layout.rs::MetaLayout` defines the canonical topology. New components should use
these paths through the layout registry rather than re-deriving strings by hand.

| surface | canonical location | rule |
|---|---|---|
| CLI/application frontdoors | `$META_ROOT/usr/bin` or the Yazelix profile | Exposed on PATH by the envctl/Yazelix PATH blocks. Links inside `$META_ROOT/usr/bin` may point to other `$META_ROOT` paths; real-home user-bin shadows are archived. |
| Libraries/support files | `$META_ROOT/usr/lib` | Component-owned libraries and support payloads. |
| Private executables | `$META_ROOT/usr/libexec` | Non-PATH helper binaries, including private envctl/secrets executables. |
| Config/trust pins | `$META_ROOT/etc/envctl` | Envctl-owned config fragments and trust pins that are not reviewed dotfiles. |
| Read-only shared payloads | `$META_ROOT/usr/share` | Envctl-owned read-only shared assets, templates, manpages, and generated drop-ins when they are not desktop/XDG assets. |
| XDG data | `$META_ROOT/.local/share` | Only for host/XDG contracts such as desktop entries, icons, fonts, and component data that explicitly requires XDG data semantics. |
| Mutable envctl state | `$META_ROOT/var/lib/envctl` | Envctl-owned durable operational state such as migration ledgers, repo stores, and logs that are not app-XDG state. |
| Logs | `$META_ROOT/var/log/envctl` | Envctl-owned log files when separated from state. |
| XDG state | `$META_ROOT/.local/state` | Only for host/XDG app contracts that explicitly require XDG state semantics. |
| Cache | `$META_ROOT/var/cache/envctl` | Envctl-owned regenerable caches. |
| XDG cache | `$META_ROOT/.cache` | Only for host/XDG app caches that explicitly require XDG cache semantics. |
| Temp | `$META_ROOT/var/tmp` | Meta-local temp space. |
| Component prefixes | `$META_ROOT/opt/<component>` | Preferred install prefix for new third-party component payloads. |

### Root 2 — `$META_ROOT/.toolchains/`: compatibility toolchain store

The existing manager homes (`BUN_INSTALL`, `CARGO_HOME`, `RUSTUP_HOME`, `UV_TOOL_DIR`,
`UV_PYTHON_INSTALL_DIR`, `MISE_DATA_DIR`, `LIBCLANG_PATH`, `GCC_PATH`, and similar) may continue to
live under `$META_ROOT/.toolchains` while manifests migrate. This prefix is gitignored,
regenerable, and still meta-hosted; it is not a user-global install.

### Root 3 — `$META_ROOT/.config/` and `home/`: dot-root policy

The tracked `home/` overlay remains the source of truth for reviewed dotfiles and user service
units. Runtime dot directories that contain mutable state should move to `$META_ROOT/var/lib`,
`$META_ROOT/var/cache`, `$META_ROOT/var/log`, or an explicit meta-XDG root only when an upstream
contract requires XDG semantics. A future central dot-root
may add one bridge per top-level dot directory, but it must follow the same rule: real file in meta,
host path is only a bridge.

## Real-home dot-entry relocation map

The TASK-0078 audit/migration loop classifies every top-level real-home dot entry before any
mutation. Default audit mode is read-only; mutation requires an explicit opt-in flag and a named
allowlisted target. The current canonical map is:

| real-home source | canonical target | mutation rule |
|---|---|---|
| `$ENVCTL_REAL_HOME/.local/state/nix/profiles` | Yazelix/Nix profile state | Preserved in place because `$ENVCTL_REAL_HOME/.nix-profile` resolves through it. |
| Known per-tool real-home user-bin shadows | Yazelix profile or `$META_ROOT/usr/bin` | Archive after the replacement frontdoor exists; never replace the whole real-home `.local` tree. |
| Safe duplicate shell dotfiles (`.bash_logout`, `.profile`, `.zshenv`, `.zshrc`) | `$META_ROOT/<dot-entry>` | `--apply-shell-dotfiles` moves only duplicate/safe sources; differing files stay owner-supervised. |
| History/backup dot entries | `$META_ROOT/var/lib/envctl/real-home-dotfile-migration/history-or-backup/<dot-entry>` | `--apply-history-archives` is required in addition to `--apply`; stale backup-only archive mode is intentionally absent. |
| `.ideavimrc` | `$META_ROOT/.ideavimrc` | Named `--migrate-dot .ideavimrc` only; refuses non-regular-file sources. |
| `.gphoto` | `$META_ROOT/.config/gphoto` | Named `--migrate-dot .gphoto` only; refuses non-directory sources. |
| `.vscode-shared` | `$META_ROOT/.local/share/vscode-shared` | Named `--migrate-dot .vscode-shared` only; refuses non-directory sources. |
| `.claude.json` | `$META_ROOT/.local/share/claude/claude.json` | Named `--migrate-dot .claude.json` only; preserves a real-home bridge for app compatibility. |
| `.ollama` | `$META_ROOT/var/lib/ollama` | Named `--migrate-dot .ollama` only; model/state payloads remain meta-local and outside git. |
| Known agent/app config dirs (`.agents`, `.ampcode`, `.claude`, `.codex`, `.codeium`, `.copilot`, `.cursor`, `.gemini`, `.goose_recipes`, `.junie`, `.kimi`, `.kimi-code`, `.roo`, `.vscode`, `.windsurf`, `.mozilla`, `.thunderbird`, `.repomix`) | `$META_ROOT/.local/share/<name>` | Named `--migrate-dot <entry>` only; conflicts/different existing targets stay owner-supervised. |
| Broad config/cache/credential stores (`.config`, `.cache`, `.aws`, `.gnupg`, `.ssh`, and similar) | owner-supervised-vault-or-bridge | No automatic move; audit reports the class so a later component-specific upgrade can prove safety first. |

For direct `.cache` children, `scripts/audit-meta-local-paths.sh` keeps the upgrade path read-only
until a component manifest has been reviewed. Use `--owner-supervised-cache-child-component-plan`
to derive the bounded component key and manifest hint, then
`--owner-supervised-cache-child-component-manifest-status` to prove whether
`manifest/components.d/cache-<component_key>.toml` already exists, and
`--owner-supervised-cache-child-component-manifest-validation` to prove whether an existing
manifest declares the expected `[[component]] id = "cache-<component_key>"`. When a manifest is
missing, `--owner-supervised-cache-child-component-manifest-scaffold` emits the same validation
state plus a deterministic escaped TOML stub (`manifest_stub`) for owner review; it does not write
the file. Missing manifests must be created and reviewed before a named `--migrate-cache-child NAME`
run; existing manifests must be reviewed before use; existing manifests with a missing or unrelated
component id must be fixed before migration. These reports intentionally leave `apply_command` empty
and do not move owner-supervised cache state.

The review loop must keep this map synchronized with `scripts/audit-meta-local-paths.sh`,
`scripts/tests/test-meta-local-path-audit.sh`, `home/README.md`, and `ci/gates/meta-local-policy.sh`.

## Decision: no `.local` peer repo

A separate `.local` git peer is the wrong shape. `$META_ROOT/.local` and `$META_ROOT/.toolchains`
contain large, host-specific, regenerable install state. The correct reproducibility surface is:

1. manifests and component hooks,
2. `manifest/envctl.lock`,
3. `envctl doctor --json` / `envctl auto-detect --json`, and
4. targeted component state under `$META_ROOT/var/lib/envctl`, `$META_ROOT/var/cache/envctl`, `$META_ROOT/opt/<component>`, or an explicitly declared meta-XDG root.

Tracking a snapshot of `.local` would bloat git, pin one host's absolute state, and duplicate the
declarative manifests. The invariant is declaration plus verification, not filesystem snapshotting.

## Cleanup and migration rule

There is no generic “delete everything old” command. Strict-upgrade cleanup happens only inside the
component that owns a migration, and only after the replacement path under `$META_ROOT` exists and
is verified. Foreign or different legacy binaries are surfaced by `doctor`/migration reports rather
than removed. `reset` remains the reversible component removal path; destructive purges require the
component's explicit guarded reset flow.

## Active follow-up table

| surface | authoritative location | cleanup / ownership rule |
|---|---|---|
| Component registry | `manifest/*.toml` plus `manifest/components.d/*.toml`, loaded by `Registry::load`; pinned by `manifest/envctl.lock` | `envctl lock --check` gates manifest drift. |
| Hub/tool registry | each `<name>_hub/registry.json` under the envctl workspace root; today `mcp_hub/registry.json` is discovered by `envctl registry --json` | Read-only federation; `envctl registry --check` fails when a hub entry binds to a missing component. |
| Runtime/last-run state | `$META_ROOT/var/cache/envctl/<hash-of-manifest-dir>/state.json` when envctl owns it; otherwise explicit component state under `$META_ROOT/var/lib/<component>` or a declared meta-XDG state root | Machine-local and intentionally uncommitted. |
| CLI exposure | `$META_ROOT/usr/bin` or the Yazelix profile | Recreated by `envctl install` or the profile owner; host per-tool user-bin shadows are archive-only migration debt. |
| Secrets daemon binaries | `$META_ROOT/usr/libexec/envctl/secrets/bin/{secretd,secretctl}` with frontdoors in `$META_ROOT/usr/bin` | Legacy compatibility paths are retained only until migrated and parity-proven. |
| Secrets config | `$META_ROOT/.config/env-ctl/secretd.toml` | Preserved unless the owning reset flow explicitly removes it; auth tokens are never stored here. |
| Secrets data/audit | `$META_ROOT/.local/share/env-ctl` and `$META_ROOT/.local/state/env-ctl` | Data paths are deleted only by the guarded `envctl reset env-ctl --purge --confirm --apply` flow. |
| Cognitum Seed Device CA pin | `$META_ROOT/etc/envctl/secrets/ca/cognitum-ca.crt` | `env-ctl.service` exports `ENVCTL_SEED_CA` to this path. |
| Local MITM/remote-client CAs | sealed in the encrypted vault store (`ca_key`/`certs` rows and `mitm.ca_cert_der` / `remote_clients.ca_cert_der` metadata) | Private keys never leave the daemon; public trust apply remains explicit/dry-run by default. |

## System-depth convergence principle

System-depth installs are not the envctl target. A host prerequisite such as the kernel, GPU driver,
or OS build floor may be detected and verified, but envctl-owned tools must be installed into
`$META_ROOT`. Existing system-depth or user-global tools are migration debt unless a component
explicitly classifies them as host prerequisites.

## Complete `$META_ROOT/usr` mirror + the three PATH surfaces

`$META_ROOT/usr` is a **structural mirror of `/usr`**, not just `bin/lib`. `MetaLayout`
(`crates/engine/src/layout.rs`) exposes the full FHS skeleton — `bin`, `sbin`, `lib`, `lib64`,
`libexec`, `include`, `share` (+ `share/man`), `src`, `games`, and `local/{bin,sbin,lib,lib64,
include,share}` — all `Canonical`, so `ensure_dirs()` materializes the complete tree on install.
This is a *structure* mirror (a canonical prefix ready to receive meta-native installs), never a
content symlink-farm of the host `/usr` (which would re-introduce system-depth tools).

`envctl env --toolchains` carries that mirror onto every search path, prepend-with-fallback so an
inherited value (e.g. the CUDA `LD_LIBRARY_PATH` block) is preserved, never clobbered:
`PATH` (bin/sbin/local/bin/local/sbin), `LD_LIBRARY_PATH` (lib/lib64/local), `CPATH` (include),
`PKG_CONFIG_PATH` (lib+share `pkgconfig`), `MANPATH` (share/man). The skeleton starts empty, so no
system binary/lib/header is shadowed until meta actually installs into it.

Three session surfaces consume this — because each reads a different startup file:
- **bash/zsh** — the `eval "$(envctl env --toolchains)"` shell-rc block. The `yazelix auto-enter`
  block evals it *before* `yzx enter` so re-exec'd zellij/nushell panes inherit the full PATH.
- **nushell/yazelix** — the version-controlled overlay module `home/.config/nushell/meta-usr-path.nu`,
  sourced (relative, `$HOME`-independent) from `config.nu` and `yazelix/shell_nu.nu`.
- **graphical/desktop login** — the `meta-session-env` component renders `systemd` user
  `environment.d/10-meta.conf` from `envctl env`, so `.desktop` launchers + GUI sessions (which
  never read `~/.bashrc`) resolve `$META_ROOT/usr/bin`.
