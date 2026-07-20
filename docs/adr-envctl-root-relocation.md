# ADR — Envctl root relocation: retire the `.toolchains` bridge (ARCHBP-030)

**Status:** accepted (2026-07-14, source-level; live cutover pending mission RESET) · **Owner:**
envctl · **Builds on:** [adr-install-locations-and-local-state](adr-install-locations-and-local-state.md),
[adr-meta-tool-location-and-portability](adr-meta-tool-location-and-portability.md).

## Context

The install-location ADR kept `$META_ROOT/.toolchains` as "Root 2 — compatibility toolchain
store … while manifests migrate", and the meta-home `.local/bin` plus real-home per-tool
user-bin entries as compatibility executable bridges. ARCHBP-030 completes the relocation:
the envctl-owned portable prefix (`$META_ROOT/{usr,etc,var,opt}` + meta-XDG roots) is the
**single canonical post-bootstrap runtime root**, and the compatibility surfaces are
**retired** — typed noncanonical, no longer emitted by any default runtime seam, and
relocatable/archivable only through receipted, reversible tooling.

The one Yazelix-owned Nix profile (`$ENVCTL_REAL_HOME/.nix-profile ->
$ENVCTL_REAL_HOME/.local/state/nix/profile`) remains the current install owner for
profile-provided frontdoors during the transition. Envctl never mutates the profile, its
generations, or profile-owned symlinks; there is exactly one canonical envctl root and one
profile owner — never a second active envctl root.

## Decision

1. **Canonical toolchain store: `$META_ROOT/opt/toolchains/<manager>`.** New `MetaLayout`
   surface: `toolchains_root()` (Canonical layout entry `toolchains`) and
   `toolchain_home(manager)`. Manager homes map 1:1 from the retired bridge:

   | export | retired (`.toolchains/…`) | canonical (`opt/toolchains/…`) |
   |---|---|---|
   | `BUN_INSTALL` | `.toolchains/.bun` | `opt/toolchains/.bun` (keeps the `.bun` segment Codex's updater keys on) |
   | `MISE_DATA_DIR` | `.toolchains/mise` | `opt/toolchains/mise` |
   | `CARGO_HOME` | `.toolchains/cargo` | `opt/toolchains/cargo` |
   | `RUSTUP_HOME` | `.toolchains/rustup` | `opt/toolchains/rustup` |
   | `UV_TOOL_DIR` / `UV_PYTHON_INSTALL_DIR` | `.toolchains/uv/{tools,python}` | `opt/toolchains/uv/{tools,python}` |
   | `OLLAMA_LIBRARY_PATH` | `.toolchains/ollama/lib/ollama` | `opt/toolchains/ollama/lib/ollama` |
   | `LIBCLANG_PATH` | `.toolchains/llvm/lib` | `opt/toolchains/llvm/lib` |
   | `GCC_PATH` | `.toolchains/libgccjit/lib` | `opt/toolchains/libgccjit/lib` |
   | `HELIX_RUNTIME` | `.toolchains/helix/runtime` | `opt/toolchains/helix/runtime` |

   `OLLAMA_MODELS` stays at `var/lib/ollama/models` (state, already canonical).

2. **Default runtime seams are canonical-only.** `envctl env --toolchains` and the managed
   hook environment (`enforced_meta_env`) emit only the portable-prefix paths: PATH is
   `usr/bin : usr/sbin : usr/local/bin : usr/local/sbin : opt/toolchains/{.bun,cargo,uv/tools}/bin`.
   No `.local/bin`, no `.toolchains`, no `ENVCTL_LOCAL_BIN`/`ENVCTL_LEGACY_TOOLCHAINS` in the
   canonical export set (`MetaLayout::env_exports`).

3. **The compatibility bridge survives only as an explicit, noncanonical rollback surface.**
   `envctl env --toolchains --legacy-bridge` (and `ENVCTL_LEGACY_BRIDGE=1` for hook runs)
   re-emits the recorded prior frontdoor exports, marked with `ENVCTL_LEGACY_BRIDGE=1` and a
   `# envctl legacy bridge` banner. `MetaLayout::legacy_env_exports()` carries the retired
   variables. Nothing selects the bridge by default.

4. **Receipted, reversible relocation tooling** (`envctl migrate relocate …`, engine module
   `crates/engine/src/relocation.rs`), dry-run by default:
   - `plan` — stages the source-to-prefix migration: every active child of
     `$META_ROOT/.toolchains` relocates to `opt/toolchains/<name>`; meta `.local/bin`
     entries whose targets live inside meta promote to `usr/bin`; real-home `.local/bin`
     shadows of canonically-provided commands are archived in place with a dated
     `.archived-<UTC>` suffix (never deleted). Yazelix-profile-owned symlinks (targets under
     `/nix/store`, `~/.nix-profile`, or `~/.local/state/nix`) are always `preserve`.
     The plan also reports **split ownership**: a command name resolvable from both a
     canonical and a retired bin dir (binary split) or a manager home present under both
     roots (config split).
   - `apply --apply` — performs the moves (refusing any move whose destination already
     exists), verifies SHA-256 content digests across the move, and writes a receipt JSON
     (`envctl.relocation.receipt.v1`) under `$META_ROOT/var/lib/envctl/relocations/`.
   - `rollback --receipt <path> --apply` — reverses a receipt move-for-move (checksum
     verified, refuses clobbers) and writes a rollback receipt pointing at the original.
   - `status` — retirement criteria: ready only when the retired roots hold no active
     entries, no split ownership remains, and no unarchived real-home shadow of a canonical
     command exists. Until then the bridge stays, explicitly noncanonical.

5. **Boundary reporting follows the single active owner.** `meta_boundary_report` scans the
   canonical `opt/toolchains/cargo/bin` when it exists, else the retired location, so the
   report never legitimizes two active cargo-bin owners at once.

## Staged migration / cutover order (mission RESET, orchestrator-owned)

Nothing below runs automatically; the tooling is dry-run by default and this wave changes
source only. At cutover, with checksum-backed backups:

1. `envctl migrate relocate plan --json` — review entries + split-ownership report.
2. `envctl migrate relocate apply --apply` — move `.toolchains/*` → `opt/toolchains/*`,
   promote meta `.local/bin` meta-owned frontdoors → `usr/bin`, archive real-home shadows;
   keep the receipt.
3. Install the new envctl binary; regenerate every consumer of the env seam (shell-rc block,
   `home/.config/nushell/meta-usr-path.nu`, `environment.d/10-meta.conf`).
4. Re-run component installs (or update manifests) so hooks land in `opt/toolchains` — the
   migration engine already tracks every manifest `.toolchains`/`ENVCTL_LEGACY_TOOLCHAINS`
   reference as `needs_migration` debt.
5. `envctl migrate relocate status --json` — retirement criteria must report ready.
6. Rollback path: `envctl migrate relocate rollback --receipt <receipt> --apply` plus
   `envctl env --toolchains --legacy-bridge` restores the recorded prior frontdoor without
   creating a second canonical root; the Yazelix profile generation is never part of the
   receipt, so no reachable rollback generation is ever deleted.

## Consequences

- Red tests reject compatibility paths as canonical (`crates/cli/tests/env.rs`,
  `crates/engine/src/runner.rs`, `crates/engine/src/layout.rs`).
- `crates/engine/tests/relocation.rs` proves staged migration, command resolution, data
  preservation, upgrade idempotence, rollback, and retirement criteria on staging roots.
- `crates/engine/tests/source_retirement.rs` pins the source-level retirement: the literal
  `.toolchains` may appear only in the typed definition/migration/relocation machinery.
- The secrets seam keeps its read-only legacy CA fallback (`seam.rs`,
  `engine/src/secrets.rs`) — explicitly noncanonical, used only when the legacy file still
  exists, retired with the same cutover.
