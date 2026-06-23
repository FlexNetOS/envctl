# Implementation log: TASK-0062 — meta-owned libgccjit for rustc_codegen_gcc (Epic H, 9th component)

## Changes
- manifest/components.d/epic-h-toolchains.toml: appended a new `[[component]] id="libgccjit"` AFTER the `llvm-clang` block — detect/install/verify/remove mirroring the llvm-clang/ollama hook shape (`M="${META_ROOT:-$HOME/Desktop/meta}"` resolution, `mktemp -d`+`trap`, idempotent `rm -rf "$DEST"` before install, self-guarded remove). Install reads the pinned commit from rustc_codegen_gcc's `libgccjit.version`, downloads `rust-lang/gcc` release asset `master-${COMMIT}/libgccjit.so`, installs to `$DEST/lib/libgccjit.so` + `ln -sfn ... libgccjit.so.0` SONAME. NO `[component.wiring]` / `path_entries` (runtime `.so`, not a CLI binary).
- crates/cli/src/main.rs (run_env): added `GCC_PATH = "{tc}/libgccjit/lib"` to BOTH the JSON map (after `LIBCLANG_PATH`) and the shell-export form (after `LIBCLANG_PATH`), copying the exact `tc` prefix + `sh_single_quote` idiom; one-line explanatory comment on the shell export.
- crates/cli/tests/env.rs: extended BOTH toolchains tests (shell + json) with a `GCC_PATH` assertion matching the existing `LIBCLANG_PATH` style (path `<root>/.toolchains/libgccjit/lib`).
- manifest/envctl.lock: regenerated via `envctl lock` — net-new id, component count **72 → 73**. `envctl lock --check` exits 0.
- docs/adr-install-locations-and-local-state.md: updated the libgccjit row to **SHIPPED** (TASK-0062), citing the `libgccjit.version`-pinned `rust-lang/gcc` release asset as the reproducible source and the GCC_PATH seam.

## Engine API
No Engine API change. CLI-only env-seam addition (`GCC_PATH`) + a manifest TOML component the engine runs. GUI does not consume this seam, so there is no front-end parity surface to diverge — consistent with TASK-0060/0061.

## Tests added
Extended (not new) in crates/cli/tests/env.rs:
- `toolchains_shell_exports_rustup_home_with_cargo_home`: now also asserts `export GCC_PATH='<root>/.toolchains/libgccjit/lib'`.
- `toolchains_json_carries_rustup_home`: now also asserts json `GCC_PATH == <root>/.toolchains/libgccjit/lib`.
Both prove the env seam emits the libgccjit lib dir in both output modes.

## Build/test status (real output)
- `cargo build -p envctl-engine -p envctl` — PASS (Finished dev profile, 10.71s).
- `cargo test -p envctl --test env` — PASS (2 passed).
- `envctl auto-detect | grep -i libgccjit` — PARSES; before install: `[med] libgccjit Missing: declared but not installed`; after install: `✓ libgccjit (meta-owned) [healthy] wired`.
- `envctl env --toolchains --json | grep GCC_PATH` — `"GCC_PATH": "/home/drdave/Desktop/meta/.toolchains/libgccjit/lib"`.
- `envctl env --toolchains | grep GCC_PATH` — `export GCC_PATH='/home/drdave/Desktop/meta/.toolchains/libgccjit/lib'`.
- `envctl lock` — `wrote manifest/envctl.lock (73 components)`; `envctl lock --check` — `✓ envctl.lock matches the manifest (73 components)`.
- `cargo fmt --all -- --check` — PASS (fmt collapsed the multi-line GCC_PATH assert to one line; clean).
- `cargo clippy -p envctl-engine -p envctl -- -D warnings` — PASS (No issues found).
- `bash ci/gates/no-c.sh` — PASS. `bash ci/gates/shape.sh` — PASS. `bash ci/gates/loop-state.sh` — PASS.

## Network install — EXERCISED (network available in sandbox)
- `libgccjit.version` resolved `COMMIT=2f06e64df0dc15f861f77595b77bfc2ba5deb59d` (matches the architect's verified pin).
- Asset URL `https://github.com/rust-lang/gcc/releases/download/master-2f06e64.../libgccjit.so` — HEAD HTTP 200.
- `envctl install libgccjit` — PASS (exit 0). Artifact landed: `.toolchains/libgccjit/lib/libgccjit.so` (426.3M, mode 644) + `libgccjit.so.0 -> .../libgccjit.so` SONAME symlink.
- verify hook: `file ... | grep -q 'shared object'` — PASS. auto-detect after install: `[healthy] wired`.
- The component is left INSTALLED on the box (the desired end state); the artifact is real, not a stub.

## Deviations
None. Implemented exactly per the U1..U6 ledger. Implementer judgment call (architect's non-blocking note): chose to pin the commit at install time by reading `libgccjit.version` (matches the backend's own pin) rather than hardcoding the commit — keeps the component reproducible AND auto-tracking the backend's pin.

## Lock count delta
72 → 73 (additive, net-new id `libgccjit`; this is the standard Epic-H additive case, unlike TASK-0061 where a duplicate apt id was collapsed).

## Pre-existing drift seen
None observed. `cargo fmt --check` and `cargo clippy -p envctl-engine -p envctl -- -D warnings` were both clean (no untouched-line lints surfaced this run). I did NOT run a `--workspace` clippy (the GUI crate can carry inherited lints per prior cycles); engine+CLI are clean.

## Handoff notes (guardian)
- **no-C invariant is safe — do NOT false-flag.** The 426MB `libgccjit.so` is a runtime artifact under `.toolchains/`, downloaded at install time; it is NEVER a Cargo dependency and never linked into a workspace crate. `ci/gates/no-c.sh` (cargo-metadata-scoped) is provably unaffected and ran PASS.
- **Network install was fully exercised on-box** (download → SONAME symlink → verify → `[healthy]`), so the guardian does NOT need to re-run the install to confirm the URL resolves. The artifact remains installed.
- **remove hook** is self-guarded (`rm -rf "$M/.toolchains/libgccjit"`, scoped to our DEST only). I did NOT run remove (would delete the 426MB artifact and force a re-download); the path mirrors the proven ollama/llvm remove idiom.
- **No `path_entries`** by design — `.so`, not a CLI binary, so there is no `~/.local/bin` symlink and no PATH wiring. Confirmed the TOML parses without a `[component.wiring]` table (auto-detect shows `wired`).
- Sequential single-implementer; no grit/parallel mode, no symbols claimed.
