# Implementation log: TASK-0054 — wire wild as the local cargo linker

Status: **GREEN** — build-verify gate PASSED; wiring kept and committed.

## Changes
- `manifest/components.d/epic-h-toolchains.toml`: extended the existing `wild-linker` `[[component]]`
  (no new component — count stays 74). File header (lines ~6-12) updated to say wild is now fully
  wired + build-verified; kache note kept install-only. Per-unit:
  - U1 `[component.install]`: after the binary install + `~/.local/bin/wild` symlink, write
    `$M/.cargo/config.toml` (`M="${META_ROOT:-$HOME/Desktop/meta}"`) with the linker section
    (`linker="clang"` + `rustflags=["-Clink-arg=--ld-path=wild"]`). `install -d -m 755 "$M/.cargo"`
    first; back up any pre-existing FOREIGN config to `config.toml.pre-wild.bak` (only if our marker
    is absent); idempotent — always rewrites exactly our marker-stamped managed file.
  - U2 `[component.detect]`: now ALSO asserts `$M/.cargo/config.toml` exists and contains
    `--ld-path=wild`, in addition to the binary-symlink check.
  - U3 `[component.verify]`: changed from `command` to `script` — asserts wild+clang resolve, the
    config carries the section, AND builds a tiny throwaway crate in `/tmp` with
    `cargo build -v --config "$CFG"` (explicit meta-root config, avoids meta-workspace collision)
    and greps the verbose link line for `--ld-path=wild`. Uses `command grep` to bypass the
    interactive `grep`→ugrep shell-function wrapper present in login shells.
  - U4 `[component.remove]`: in addition to removing the binary/symlink, strips the wild section —
    restores `config.toml.pre-wild.bak` if present, else removes our managed file. Self-guarded:
    only touches a config carrying OUR marker. Component `name`/`description` updated to
    "wired + verified".
- `manifest/envctl.lock` (U5): regenerated via `envctl lock`. ONLY the `wild-linker` `content_hash`
  changed (`044b76e16bd48a7c` → `fe309a261aa484d2`); component count unchanged at **74**.
- `.handoff/loop/backlog.md` (U6): TASK-0054 card updated (install + wiring built + gate-passed,
  left `[~]` in-flight per TICK-ON-MERGED — orchestrator ticks `[x]` on merge). Appended deferred
  **TASK-0067 (H, EASY) — drop mold** card under Epic H (remove mold-linker apt component +
  `apt remove mold`; strip `-fuse-ld=mold` RUSTFLAGS from ai-clis codex hooks; drop mold-linker
  from ai-clis `requires`; re-lock).
- Runtime artifact (NOT committed, outside repo): `/home/drdave/Desktop/meta/.cargo/config.toml`
  written with the linker section (the live wiring the install hook produces).

## Engine API
None. Manifest-component + lock + backlog change only — no engine/CLI/GUI Rust touched, no Cargo
dependency added. (Parity contract unaffected.)

## Tests added
None (no Rust changed). Verification is the component's own detect/verify hooks (exercised live via
`envctl auto-detect`) + the build-verification gate below.

## Build/test status — exact commands run + PASS/FAIL
All run from the worktree with `PATH="$HOME/.local/bin:$PATH"` (so clang+wild resolve):

- **BUILD-VERIFY GATE (the decisive one):**
  - `cargo build -p envctl-engine -p envctl` → **PASS** (exit 0, Finished in 10.83s).
  - `cargo build --workspace` → **PASS** (exit 0, all 8 crates incl. gui + secretd, 16.80s).
  - `cargo build -v -p envctl 2>&1 | grep -- '--ld-path=wild'` → **MATCH** (exit 0). Real link line:
    ```
    rustc --crate-name envctl ... -C linker=clang ... -Clink-arg=--ld-path=wild -L native=.../ring-.../out
    ```
  - `readelf -p .comment target/debug/envctl | grep -i wild` → `Linker: Wild 0.9.0 non-git-build` —
    the produced binary is provably wild-linked. `./target/debug/envctl --version` → `envctl 0.1.0`
    (runs).
  - **DECISION: build GREEN + `--ld-path=wild` observed → wiring KEPT.**
- `envctl auto-detect` → `✓ wild-linker ... [healthy] wired` (extended detect + new verify both pass).
- `envctl lock` → `wrote manifest/envctl.lock (74 components)`; `envctl lock --check` → exit 0,
  `✓ envctl.lock matches the manifest (74 components)`.
- `ci/gates/no-c.sh` → **NO-C GATE PASS** (rustls=0.23.40 on ring=0.17.14; zero aws-lc/openssl/C-SQLite).
- `ci/gates/shape.sh` → **SHAPE GATE PASS**.
- `ci/gates/loop-state.sh` → **LOOP-STATE GATE PASS** (monotonic ok 35→35).
- `cargo fmt --all -- --check` → exit 0 (clean; no Rust touched).

## Deviations
1. **Verify-hook robustness (two fixes vs the architect's U3 sketch).** The architect suggested a
   `/tmp` probe build relying on the meta-root config applying. Two real-environment problems
   surfaced and were fixed so verify is reliable through `envctl`'s own runner (intent preserved):
   (a) a `/tmp` project does NOT inherit `$META_ROOT/.cargo/config.toml` (cargo only walks parents),
   and a probe placed UNDER `$M/.cargo/` gets pulled into the meta workspace (`current package
   believes it's in a workspace when it's not`). Solved by `cargo build --config "$CFG"` (explicit
   config, /tmp project — no workspace, config still applied; verified WILD FOUND).
   (b) `grep` is a shell **function** (rtk/ugrep wrapper) in login shells (`login_shell=true`), which
   corrupted the piped grep. Solved with `command grep` (via a small `g()` helper).
   This caused one extra lock regen (verify hash changed): final wild-linker hash `fe309a261aa484d2`.
2. None to scope: no new component, no Cargo dep, no engine/CLI/GUI Rust, count still 74.

## Handoff notes (for the invariant-guardian)
- **Build-verify gate is the cycle's pass condition and it PASSED** — re-confirm with
  `PATH="$HOME/.local/bin:$PATH" cargo build -v -p envctl 2>&1 | command grep -- '--ld-path=wild'`
  (must match) and `readelf -p .comment target/debug/envctl | command grep -i wild` (Wild 0.9.0).
  NOTE: ALL cargo builds in the meta tree now use wild because the config is at the shared
  `$META_ROOT/.cargo/config.toml` — if wild were broken, even the guardian's own build would fail.
- **Use `command grep`** when re-running any grep in a login shell here — bare `grep` is the
  ugrep-wrapper function and will mangle pipes (this is what produced a spurious first verify-fail).
- **Reversibility (destructive-op invariant):** install backs up a foreign config to
  `.pre-wild.bak` before taking over; remove restores `.bak` or deletes our marker-stamped file.
  To prove the revert path: run the component's `remove` script body, then
  `cargo build -p envctl-engine -p envctl` from the worktree must still succeed WITHOUT the override
  (the workspace was confirmed to build cleanly both with and without wild during this cycle).
- **no-C:** wild + clang are build TOOLING (the linker), never linked into the dep graph — `no-c.sh`
  is provably unaffected (still PASS) and no Cargo dependency was added.
- **CI safety:** the config lives at the meta-root, OUTSIDE the envctl repo; CI clones each repo
  standalone so it never sees this file — CI builds are unchanged. Only the manifest component def is
  committed.
- **Lock:** count unchanged at 74 (extended, not added); the only lock delta is the wild-linker hash.
