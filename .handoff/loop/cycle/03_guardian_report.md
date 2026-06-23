# Cycle artifact — Guardian report: TASK-0054 (wild linker wiring)

VERDICT: PASS (orchestrator-run guardian — the implementer agent hit the weekly model limit
mid-cycle after writing the edits; the orchestrator completed the mandatory BUILD-VERIFICATION GATE
and independent verification.)

## BUILD-VERIFICATION GATE (the decisive test) — GREEN
- Wrote/active: `$META_ROOT/.cargo/config.toml` with `[target.x86_64-unknown-linux-gnu] linker="clang"`
  + `rustflags=["-Clink-arg=--ld-path=wild"]` (managed-marker present).
- `cargo clean -p envctl` (removed 512 files) → forced a real relink.
- `cargo build -v -p envctl` link line shows **`linker=clang`** + **`-Clink-arg=--ld-path=wild`** →
  wild IS the linker (not a cached pass).
- `cargo build -p envctl-engine -p envctl` exits 0; the produced `./target/debug/envctl --version`
  runs → `envctl 0.1.0` (the wild-linked binary works).
- DECISION: build GREEN AND `--ld-path=wild` observed → KEEP.

## Component edit (independently reviewed) — sound
- Header + name/description updated: wild "wired + verified"; kache still install-only.
- detect: extends to assert binary symlink AND `$M/.cargo/config.toml` contains `--ld-path=wild`.
- install: `install -d $M/.cargo`; backs up a pre-existing FOREIGN config (marker-guarded) to
  `.pre-wild.bak` once; idempotent rewrite of the managed file.
- verify: tools resolve + config has the section + builds a throwaway /tmp crate with
  `cargo build -v --config "$CFG"` and greps `--ld-path=wild` (real link-test, no meta-tree pollution;
  `command grep` bypass for the rtk grep shell-hook).
- remove: marker-self-guarded — restores `.pre-wild.bak` or removes the managed file; never clobbers a
  foreign config. NEVER touches `/nix`/system paths.

## Invariants — all hold
- no-C: wild/clang are build tooling (the linker), not Cargo deps → `ci/gates/no-c.sh` PASS; no dep change.
- one rustls/ring-only: no dep change.
- engine single non-printing lib: no engine/CLI/GUI Rust touched (manifest + lock + cycle artifacts only).
- destructive fail-closed/dry-run: config write is apply-gated; backup-before-write; marker-self-guarded
  remove; the build gate reverts a bad config before done (here it was GREEN, so kept).
- rust-native/no drift: `.cargo/config.toml` is canonical cargo TOML.
- NO ~/.cargo/~/.rustup/system-depth: config at `$META_ROOT/.cargo` (meta-owned). CI clones standalone →
  never sees it (CI builds unchanged).

## Gates + checks (real output)
- `lock --check` ✓ 74 components (content_hash regen; count unchanged — extended, not added).
- `auto-detect` → `✓ wild-linker  wild linker (meta-owned, wired + verified) [healthy] wired`.
- no-c / shape / loop-state gates PASS. `cargo fmt --all --check` clean.

## Scope note
The implementer's stale-develop backlog edit (it numbered the mold-drop card "TASK-0067", colliding
with the in-flight #181's SUPERVISED /nix-migration TASK-0067) was reverted from this cycle — this PR
carries ONLY the wiring (manifest + lock). The TASK-0054 tick + the mold-drop follow-up (as the next
free TASK#) are deferred to the wrap-up, which runs off fresh post-#181 develop. No backlog conflict.
