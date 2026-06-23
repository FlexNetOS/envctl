# Cycle artifact — Architect plan: TASK-0054 (wild linker wiring)

VERDICT: GO

## Summary
Wire **wild** as the local cargo linker (via `clang --ld-path=wild`) by writing a linker section to
the **meta-root** `$META_ROOT/.cargo/config.toml` (local-dev only; CI clones repos standalone so it
never sees this file). Delivered by EXTENDING the existing `wild-linker` component (NOT a new one) so
install/detect/verify/remove manage both the binary and the config as one reversible unit. The cycle
is gated by a **build-verification gate**: write config → full `cargo build` MUST pass AND use wild →
keep; else revert + mark blocked. mold-drop DEFERRED (sudo + separate concern → follow-up card).

## Config location (owner doctrine)
`$META_ROOT/.cargo/config.toml` = `/home/drdave/Desktop/meta/.cargo/config.toml` (absent today;
`~/.cargo/config.toml` absent). NOT `~/.cargo`/`~/.rustup`/system-depth. Runtime artifact at meta-root
(like `.toolchains/`), outside the envctl git repo — only the manifest component def is committed.

## Canonical syntax (confirmed vs wild docs; Wild 0.9.0 installed, clang meta-owned TASK-0061)
```
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-Clink-arg=--ld-path=wild"]
```

## Unit ledger (completeness contract)
- **U1** `epic-h-toolchains.toml` :: `wild-linker` `[component.install]` — after binary install+symlink,
  write `$M/.cargo/config.toml` with the linker section; back up any pre-existing file to
  `config.toml.pre-wild.bak`; idempotent (write-if-absent / overwrite-our-section).
- **U2** same :: `[component.detect]` — extend to ALSO assert `$M/.cargo/config.toml` contains the
  `--ld-path=wild` line (in addition to the binary-symlink check).
- **U3** same :: `[component.verify]` — prove wiring: clang+wild on PATH, config parses, a build links
  via wild (`cargo build -v` shows `--ld-path=wild`).
- **U4** same :: `[component.remove]` + file header (lines 6–9) + name/description — remove ALSO strips
  the linker section (or restores `.bak`) → no override; header updated to "wiring included + verified".
- **U5** `manifest/envctl.lock` :: `[components.wild-linker] content_hash` regen (from 044b76e16bd48a7c).
- **U6** `.handoff/loop/backlog.md` :: append deferred mold-drop follow-up (apt removal + ai-clis
  RUSTFLAGS strip + drop mold-linker from ai-clis requires + re-lock).
- **GATE** (process): write config → `cargo build` green AND `--ld-path=wild` observed → keep; red →
  revert (remove config section) + re-build to confirm healthy + mark TASK-0054 blocked, DON'T commit.

## Invariants (all PASS)
- no-C: wild/clang are build TOOLING (the linker), never linked into the dep graph → no-c.sh unaffected;
  no new crate dep.
- one rustls/ring-only: no dep change.
- engine single non-printing lib: no engine code; manifest component only.
- destructive fail-closed/dry-run: config write is apply-gated (only `envctl install`, not auto-detect/
  doctor); backup-before-write; reversible remove; the BUILD GATE reverts a bad config before done.
- rust-native/no drift: `.cargo/config.toml` is canonical cargo TOML, not foreign source.
- NO ~/.cargo/~/.rustup/system-depth: config at $META_ROOT/.cargo.
- CI safety: meta-root config is outside the per-repo CI clone → CI builds unchanged.

## mold-drop: DEFER
mold refs: `dev-tools.toml` mold-linker (apt), `ai-clis.toml` codex RUSTFLAGS (`-fuse-ld=mold`,
command -v guarded), `envctl.lock` ai-clis requires. Dropping apt = sudo; the codex RUSTFLAGS would
conflict with --ld-path=wild → strip in the follow-up. Defer all to keep this cycle scoped + honor
"verify builds first".

## Runtime surface
1. `$META_ROOT/.cargo/config.toml` exists with the linker section.
2. `cargo build -v -p envctl 2>&1 | grep -- '--ld-path=wild'` returns the clang link cmd; build exits 0.
3. `envctl auto-detect`/`doctor` shows wild-linker healthy (extended detect: binary + config).

## Open question (non-blocking → folded into mold-drop follow-up)
ai-clis codex hooks inject `-fuse-ld=mold` via RUSTFLAGS; with --ld-path=wild that's conflicting/wasteful
— strip in the deferred mold-drop follow-up. Harmless to this cycle's gate (gate builds are envctl).
