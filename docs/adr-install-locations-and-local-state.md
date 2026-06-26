# ADR — Install-location map + local-state model

**Status:** accepted (2026-06-23), amended 2026-06-26 · **Owner:** envctl · **Builds on:**
[adr-meta-tool-location-and-portability](adr-meta-tool-location-and-portability.md), ADR-0006
(home/ overlay portability).

## 2026-06-26 amendment: META_ROOT is the install root

`envctl` is the path authority for meta installs. All envctl-owned tools, application payloads,
frontdoors, support files, state, cache, and temporary files must resolve under:

```text
$META_ROOT/.local/{bin,lib,share,state,cache,tmp,opt}
$META_ROOT/.toolchains/        # legacy compatibility prefix while older manifests migrate
$META_ROOT/.config/            # tracked/configured dot roots that are intentionally meta-hosted
```

The only sanctioned object at the real user-home `.local` path is the bridge:

```text
$ENVCTL_REAL_HOME/.local -> $META_ROOT/.local
```

That bridge exists for host integrations and interactive shells that still ask XDG for the user's
home-local prefix. It is **not** a place for per-tool links, regular binaries, caches, or state.
No component may install into any real-home/user-home/systemd-home local-bin spelling, or any
leading-tilde local path. Managed hooks run with `HOME=$META_ROOT`; hook bodies that truly need
the real user home must use `ENVCTL_REAL_HOME` and must document why.

## Install-location map

### Root 1 — `$META_ROOT/.local/`: canonical envctl install tree

`crates/engine/src/layout.rs::MetaLayout` defines the canonical topology. New components should use
these paths through the layout registry rather than re-deriving strings by hand.

| surface | canonical location | rule |
|---|---|---|
| CLI/application frontdoors | `$META_ROOT/.local/bin` | Exposed on PATH by the envctl PATH block. Links inside this tree may point to other `$META_ROOT` paths; the host-home `.local` tree is only the single directory bridge. |
| Libraries/support files | `$META_ROOT/.local/lib` | Component-owned support payloads. |
| Shared data | `$META_ROOT/.local/share` | Envctl-owned persistent data, including component stores and secrets data. |
| Mutable state/logs | `$META_ROOT/.local/state` | Envctl-owned state and logs. |
| Cache | `$META_ROOT/.local/cache` | Regenerable caches. |
| Temp | `$META_ROOT/.local/tmp` | Meta-local temp space. |
| Component prefixes | `$META_ROOT/.local/opt/<component>` | Preferred install prefix for new third-party component payloads. |

### Root 2 — `$META_ROOT/.toolchains/`: compatibility toolchain store

The existing manager homes (`BUN_INSTALL`, `CARGO_HOME`, `RUSTUP_HOME`, `UV_TOOL_DIR`,
`UV_PYTHON_INSTALL_DIR`, `MISE_DATA_DIR`, `LIBCLANG_PATH`, `GCC_PATH`, and similar) may continue to
live under `$META_ROOT/.toolchains` while manifests migrate. This prefix is gitignored,
regenerable, and still meta-hosted; it is not a user-global install.

### Root 3 — `$META_ROOT/.config/` and `home/`: dot-root policy

The tracked `home/` overlay remains the source of truth for reviewed dotfiles and user service
units. Runtime dot directories that contain mutable state should move to `$META_ROOT/.config` or
`$META_ROOT/.local/{share,state,cache}` as their components are upgraded. A future central dot-root
may add one bridge per top-level dot directory, but it must follow the same rule: real file in meta,
host path is only a bridge.

## Decision: no `.local` peer repo

A separate `.local` git peer is the wrong shape. `$META_ROOT/.local` and `$META_ROOT/.toolchains`
contain large, host-specific, regenerable install state. The correct reproducibility surface is:

1. manifests and component hooks,
2. `manifest/envctl.lock`,
3. `envctl doctor --json` / `envctl auto-detect --json`, and
4. targeted component state under `$META_ROOT/.local`.

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
| Runtime/last-run state | `$META_ROOT/.local/cache/envctl/<hash-of-manifest-dir>/state.json` when envctl owns it; otherwise explicit component state under `$META_ROOT/.local/state` | Machine-local and intentionally uncommitted. |
| CLI exposure | `$META_ROOT/.local/bin` | Recreated by `envctl install`; host `$ENVCTL_REAL_HOME/.local` is only the directory bridge. |
| Secrets daemon binaries | `$META_ROOT/.local/lib/envctl/secrets/bin/{secretd,secretctl}` with frontdoors in `$META_ROOT/.local/bin` | Legacy compatibility paths are retained only until migrated and parity-proven. |
| Secrets config | `$META_ROOT/.config/env-ctl/secretd.toml` | Preserved unless the owning reset flow explicitly removes it; auth tokens are never stored here. |
| Secrets data/audit | `$META_ROOT/.local/share/env-ctl` and `$META_ROOT/.local/state/env-ctl` | Data paths are deleted only by the guarded `envctl reset env-ctl --purge --confirm --apply` flow. |
| Cognitum Seed Device CA pin | `$META_ROOT/.local/share/envctl/secrets/ca/cognitum-ca.crt` | `env-ctl.service` exports `ENVCTL_SEED_CA` to this path. |
| Local MITM/remote-client CAs | sealed in the encrypted vault store (`ca_key`/`certs` rows and `mitm.ca_cert_der` / `remote_clients.ca_cert_der` metadata) | Private keys never leave the daemon; public trust apply remains explicit/dry-run by default. |

## System-depth convergence principle

System-depth installs are not the envctl target. A host prerequisite such as the kernel, GPU driver,
or OS build floor may be detected and verified, but envctl-owned tools must be installed into
`$META_ROOT`. Existing system-depth or user-global tools are migration debt unless a component
explicitly classifies them as host prerequisites.
