# Implementation log: TASK-0066 — meta-owned nix-portable (Epic H, ADDITIVE component only)

## Changes
- manifest/components.d/epic-h-toolchains.toml: appended a new `[[component]] id="nix-portable"`
  block AFTER the `libgccjit` block. Mirrors the **mise** single-static-binary idiom (curl direct to
  `$DEST/bin/nix-portable` → `~/.local/bin/nix-portable` symlink) with an `rm -rf "$DEST"` idempotence
  guard (ollama/llvm idiom). Tag pinned via the DavHau/nix-portable releases API
  (`grep -oE 'v[0-9]+'`), asset `nix-portable-x86_64`. detect = file-exists + executable + symlink
  resolves to DEST (mise readlink idiom). verify = NO mutation/network — file + executable +
  `file ... | grep -q 'ELF'` (libgccjit verify idiom). remove = symlink-ownership-guarded
  (`[ -L t ] && readlink t | grep -q "$M/.toolchains/nix-portable"`) + `rm -rf` DEST — NEVER touches
  host `/nix`. `[component.wiring] path_entries = ["~/.local/bin"]`. Comment documents bwrap runtime
  / `NP_RUNTIME=bwrap` / default store `~/.nix-portable` / additive-only + TASK-0067 deferral.
- manifest/envctl.lock: regenerated via `cargo run -p envctl -- lock` (NOT hand-edited). Net-new
  `[components.nix-portable]` (content_hash=`88e98a62e8fc4be2`, requires=[], resolved=""). +5 lines,
  one entry. Component count **73 → 74**.
- docs/adr-install-locations-and-local-state.md: (1) added a `nix` row to the §System-depth
  convergence-plan table marked **COMPONENT SHIPPED — ADDITIVE** (TASK-0066) with the destructive
  migration **DEFERRED to supervised TASK-0067**; (2) added a "Component SHIPPED (TASK-0066) —
  ADDITIVE only" bullet under the existing `nix-portable` prose, restating that it never touches host
  `/nix` and the destructive de-nix is supervised TASK-0067.

## Engine API
No Engine API change. No engine/CLI/GUI Rust touched, no `run_env`/`env.rs` change, no Cargo dep.
Purely a declarative manifest component the engine already knows how to run + the regenerated lock +
a doc. No front-end parity surface (nix-portable self-manages `~/.nix-portable`; no meta-owned env
seam like OLLAMA_LIBRARY_PATH/GCC_PATH was added, per the architect plan).

## Tests added
None — no Rust changed, so there is no new code path to unit-test. The component's behavior is
exercised by the existing manifest-driven detect/verify lifecycle (proven via `auto-detect`, below)
and the lock round-trip gate (`lock --check`).

## Build/test status (real output)
- `cargo build -p envctl-engine -p envctl` — PASS (Finished dev profile, 10.95s; no Rust changed).
- `cargo run -p envctl -- auto-detect | grep nix-portable` — PARSES + rostered:
  `[med] nix-portable  Missing: declared but not installed → envctl install nix-portable`
  (and `· nix-portable  nix-portable (meta-owned) wired` in the components roster). No parse error.
- `cargo run -p envctl -- lock` — `wrote manifest/envctl.lock (74 components)`.
- `cargo run -p envctl -- lock --check` — `✓ envctl.lock matches the manifest (74 components)` exit 0.
- `cargo fmt --all -- --check` — PASS (exit 0).
- `cargo clippy -p envctl-engine -p envctl -- -D warnings` — PASS (Finished, no issues).
- `bash ci/gates/no-c.sh` — PASS (`NO-C GATE PASS`; rustls 0.23.40 on ring 0.17.14, zero C-SQLite).
- `bash ci/gates/shape.sh` — PASS (`SHAPE GATE PASS`).
- `bash ci/gates/loop-state.sh` — PASS (`LOOP-STATE GATE PASS`).

## Network install — NOT exercised end-to-end (sandbox limitation; documented)
- The GitHub **releases API** (`api.github.com/repos/DavHau/nix-portable/releases/latest`, used by
  the install hook to PIN the tag) returns **HTTP 403** from this sandbox (unauthenticated
  rate-limit), so the full install hook (tag resolve → curl → chmod → symlink → verify) could NOT be
  run here.
- The **asset URL itself** IS reachable: `curl -fsSLI` on
  `https://github.com/DavHau/nix-portable/releases/download/v012/nix-portable-x86_64` → **HTTP 200**.
  This confirms the architect's pinned tag (`v012`) and asset name (`nix-portable-x86_64`) are valid.
- ACTION FOR GUARDIAN/ORCHESTRATOR: run the on-box `envctl install --apply` scoped to `nix-portable`
  (or the install hook body) where the GitHub API is authenticated/reachable. Expect: tag resolves to
  `v012`, ~68 MB download to `.toolchains/nix-portable/bin/nix-portable`,
  `~/.local/bin/nix-portable` → DEST symlink, `file` reports ELF, verify hook PASS, auto-detect
  `[healthy] wired`. Do NOT run `nix-portable nix ...` — first real run bwrap-bootstraps a store.

## Deviations
- **Install hook does NOT use `mktemp -d`+`trap`.** The plan/U1 said "match whatever the mise block
  does — if mise curls direct to DEST, do the same." The mise block curls straight to
  `$DEST/bin/mise` with no tmp dir; since nix-portable is the same single-binary shape and we
  `rm -rf "$DEST"` first, I curled direct to `$DEST/bin/nix-portable` (no tmp). This matches the
  mise idiom exactly and is simpler/atomic-enough for a single file. (The architect's U1 listed
  `mktemp -d`+`trap` as one option but explicitly made it optional and deferred to the mise idiom.)
- Otherwise implemented exactly per the U1/U2/U3 ledger. U4 (the TASK-0067 backlog card) was NOT
  filed — that is the orchestrator's job per the brief.

## Lock count delta
73 → 74 (additive, net-new id `nix-portable`).

## Pre-existing drift seen
None. `cargo fmt --check` and `cargo clippy -p envctl-engine -p envctl -- -D warnings` were both
clean. I touched only TOML + lock + a doc, so no Rust-line drift was possible; no `--workspace`
clippy run (GUI crate may carry inherited lints per prior cycles, unrelated to this change).

## Handoff notes (guardian)
- **no-C invariant is safe — do NOT false-flag.** The nix-portable binary is a runtime artifact
  downloaded into `.toolchains/` at install time; it is NEVER a Cargo dependency and never linked
  into a workspace crate. `ci/gates/no-c.sh` (cargo-metadata-scoped) ran PASS and is provably
  unaffected.
- **Additive-only / NEVER touches host `/nix`.** detect/install/verify/remove all operate solely on
  `$M/.toolchains/nix-portable` + the `~/.local/bin/nix-portable` symlink. The remove hook is
  symlink-ownership-guarded (only removes `~/.local/bin/nix-portable` if it resolves into our DEST).
  The destructive `/nix` migration is explicitly DEFERRED to supervised TASK-0067 (not in this PR).
- **verify hook is read-only by design** (file + executable + ELF check) — the binary forwards all
  args to nix and has no native `--version`/`--help`; first real run bwrap-bootstraps a store, so
  there is intentionally NO functional `nix-portable nix --version` in verify. This is the correct
  shape, mirroring libgccjit's read-only `file`-based verify.
- **Network install was NOT exercised on-box** (GitHub API 403 in sandbox); the asset URL was
  confirmed HTTP 200 and the tag (`v012`) validated. The guardian should run the install where the
  API is reachable to confirm tag-pin + verify-pass before merge.
- **Lock is machine-generated** (`cargo run -p envctl -- lock`), not hand-edited; `lock --check`
  exits 0 at 74 components.
- Sequential single-implementer; no grit/parallel mode, no symbols claimed.
