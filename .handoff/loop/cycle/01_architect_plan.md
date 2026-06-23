# Cycle artifact — Architect plan: TASK-0062 (libgccjit for rustc_codegen_gcc)

VERDICT: GO

## Summary
9th Epic-H component (`libgccjit`): download the prebuilt CI `libgccjit.so` from `rust-lang/gcc`
(commit pinned by rustc_codegen_gcc's own `libgccjit.version`) into
`$META_ROOT/.toolchains/libgccjit/lib` — no system GCC build, no apt. Payload is a runtime `.so`
consumed by the external rustc_codegen_gcc backend, NOT a CLI binary → **no `~/.local/bin` symlink**;
exposed via a new `GCC_PATH` env seam in `run_env` (JSON + shell), mirroring `OLLAMA_LIBRARY_PATH`
(TASK-0060) / `LIBCLANG_PATH` (TASK-0061).

## Authoritative download (verified HTTP 200, commit 2f06e64…)
- `COMMIT=$(curl -fsSL https://raw.githubusercontent.com/rust-lang/rustc_codegen_gcc/master/libgccjit.version | tr -d '[:space:]')`
- `URL=https://github.com/rust-lang/gcc/releases/download/master-${COMMIT}/libgccjit.so`
- Upstream-authoritative pin (the backend's own `libgccjit.version`), not floating-latest.

## Unit ledger (completeness contract)
- **U1** `manifest/components.d/epic-h-toolchains.toml` :: new `[[component]] id="libgccjit"` —
  detect `[ -f "$M/.toolchains/libgccjit/lib/libgccjit.so" ]`; install (login_shell): resolve COMMIT
  from libgccjit.version, curl the `.so` → `$DEST/lib/libgccjit.so` + `ln -sfn ... libgccjit.so.0`
  (SONAME), `rm -rf "$DEST"` before extract (idempotent); verify file-exists + `file ... | grep -q
  'shared object'`; remove self-guarded `rm -rf "$M/.toolchains/libgccjit"`; NO `path_entries`.
- **U2** `crates/cli/src/main.rs` :: `run_env` JSON branch (after LIBCLANG_PATH) —
  `GCC_PATH = "{tc}/libgccjit/lib"`.
- **U3** `crates/cli/src/main.rs` :: `run_env` shell branch (after LIBCLANG_PATH) —
  `export GCC_PATH=...` + explanatory comment.
- **U4** `crates/cli/tests/env.rs` :: extend BOTH toolchains tests with GCC_PATH assertions (shell+json).
- **U5** `manifest/envctl.lock` :: regenerate; net-new id → count **72 → 73** (additive, NOT id-preserved).
- **U6** `docs/adr-install-locations-and-local-state.md` :: mark libgccjit row shipped + cite the
  `libgccjit.version`-pinned `rust-lang/gcc` release asset.

## Invariants (all PASS)
- no-C trust boundary: the `.so` is a runtime artifact under `.toolchains/`, NEVER a Cargo dep →
  `ci/gates/no-c.sh` (cargo-metadata-scoped) is provably unaffected. **Guardian: do not false-flag.**
- one rustls/ring-only: no new dep at all.
- engine single shared non-printing lib: no engine change; component is TOML hooks the engine runs;
  GCC_PATH is CLI-output only (GUI does not consume this seam → cannot diverge).
- destructive fail-closed/dry-run: install/remove gated by `--apply`; remove is self-guarded.
- rust-native / no drift: sanctioned `.toolchains/` delivery path replacing a system GCC build.

## Runtime surface
1. Component lifecycle: `auto-detect` lists libgccjit (absent fresh) → `install --apply` downloads →
   `doctor` verify green.
2. Env seam: `envctl env --toolchains [--json]` emits `GCC_PATH=<root>/.toolchains/libgccjit/lib`.
   Non-network observable (CI/sandbox): the `env --toolchains` GCC_PATH emission + `auto-detect` parse.
   Network install (GitHub download) may be deferred on a sandbox.

## Implementer judgment call (non-blocking)
Pin commit at install time by reading `libgccjit.version` (recommended; matches the backend's own
pin) vs hardcoding `2f06e64…` (fully frozen). Either satisfies "stable, pinnable, not floating-latest".
