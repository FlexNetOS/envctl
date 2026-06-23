# Verification report: TASK-0062 — meta-owned libgccjit for rustc_codegen_gcc (Epic H, 9th component)

Worktree: `/home/drdave/Desktop/meta/.worktrees/task-0062-libgccjit/envctl` · branch `task-0062-libgccjit` @ `c64f539` (off `develop`, on top of `46eb908`). `$META_ROOT=/home/drdave/Desktop/meta`.
(Prior on-disk report was the stale TASK-0061 cycle — no open TASK-0062 finding carried forward; overwritten.)

## Verdict — PASS

All five non-negotiable invariants hold, all 8 CI gates + every cargo check pass, and the
architect-declared runtime surface was driven on-box with captured evidence (auto-detect
`[healthy] wired`, GCC_PATH seam in both JSON+shell, the 426 MB `.so` artifact present, verify
hook green, no drift/boundary flag). No blocking findings. Orchestrator may open the PR.

## Gate results
| Gate | Result | First line |
|------|--------|-----------|
| `no-c.sh` | **PASS** | `resolved graph clean: rustls=['0.23.40'] on ring=['0.17.14']; zero aws-lc/openssl/C-SQLite` |
| `shape.sh` | **PASS** | `SHAPE GATE PASS` |
| `enable.sh` | **PASS** | `ENABLE GATE PASS` |
| `p7.sh` | **PASS** | `P7 GATE PASS` |
| `agent-env.sh` | **PASS** | `✓ agent-env.lock is up to date` |
| `loop-state.sh` | **PASS** | `monotonic ok (34 -> 34)` |
| `harness-scripts.sh` | **PASS** | driver + reaper + loop-state-gate tests all PASS |
| `kdf-feature-off.sh` | **PASS** | `low-cost-kdf-tests correctly OFF by default` |

## cargo
| Check | Result | Evidence |
|-------|--------|----------|
| `cargo build -p envctl-engine -p envctl` | **PASS** | finished, exit 0 |
| `cargo test -p envctl --test env` | **PASS** | 2/2 — `toolchains_json_carries_rustup_home`, `toolchains_shell_exports_rustup_home_with_cargo_home` (both carry the new GCC_PATH assertions) |
| `cargo run -p envctl -- lock --check` | **PASS** | `✓ envctl.lock matches the manifest (73 components)` (72→73 confirmed) |
| `cargo fmt --all -- --check` | **PASS** | exit 0, no diff |
| `cargo clippy -p envctl-engine -p envctl -- -D warnings` | **PASS** | exit 0, clean |

### Clippy axis classification
Zero clippy findings on the gate-scope (`-p envctl-engine -p envctl`), touched crates — nothing
to classify. Did NOT run/fix any `--all-targets` or untouched-crate (gui) lint; out of scope and
not introduced by this change.

## Invariant checks
1. **No C in the trust boundary — PASS.** `no-c.sh` green (rustls 0.23.40 on ring, zero
   aws-lc/openssl/C-SQLite). **Independently confirmed NO new dependency:** `git diff
   develop...HEAD` over `Cargo.toml`/`Cargo.lock` is EMPTY — no crate added, no TLS/dep churn.
   **The downloaded `libgccjit.so` is correctly NOT a Cargo dependency:** it is a runtime shared
   object under `$META_ROOT/.toolchains/libgccjit/lib/`, consumed by the EXTERNAL
   `rustc_codegen_gcc` backend, never linked into any envctl crate. The no-c gate is
   cargo-metadata-scoped, so a file under `.toolchains/` is invisible to it by design — NOT
   false-flagged, and correctly so.
2. **Exactly one rustls, ring-only — PASS.** `Cargo.lock` diff empty; no rustls/ring/aws-lc
   churn. no-c gate reports the single `rustls 0.23.40 on ring 0.17.14`.
3. **Engine is the single shared, non-printing library — PASS.** NO engine code changed
   (diff touches only `manifest/components.d/epic-h-toolchains.toml`, `crates/cli/src/main.rs`
   `run_env`, `crates/cli/tests/env.rs`, `manifest/envctl.lock`, the ADR doc, and the two handoff
   cycle files). No `println!` added to the engine path; GCC_PATH is emitted by the CLI's
   `run_env` only (existing print site, mirrors the other toolchain exports). New logic is in the
   manifest TOML (data) + the thin CLI seam — correct placement.
4. **Destructive ops fail-closed / dry-run by default — PASS.** install/remove run only through
   envctl's standard apply harness (no bespoke guard needed for a manifest component). Read the
   actual `remove` hook (TOML 449-456): `set -u` then `rm -rf "$M/.toolchains/libgccjit"` — scoped
   to its OWN `.toolchains/libgccjit` dir only, never a foreign path; `set -u` aborts on an unbound
   `META_ROOT` (default `$HOME/Desktop/meta`). Self-guarded as claimed.
5. **Rust-native / no language drift — PASS.** No foreign-language SOURCE added to the workspace
   (no `.c`/`.cpp`/`.js`/etc. tracked). `git status --short` is clean — the 426 MB `.so` is NOT a
   tracked add. `.toolchains/` is gitignored at the meta root (`/home/drdave/Desktop/meta/.gitignore:85
   → .toolchains/`), so the runtime artifact lives entirely outside version control.

## Parity check
GCC_PATH reaches BOTH machine consumers via the single `run_env` site (no front-end divergence):
- JSON form: `crates/cli/src/main.rs:1748` — `map["GCC_PATH"] = format!("{tc}/libgccjit/lib")`
- shell form: `crates/cli/src/main.rs:1807-1810` — `export GCC_PATH=…libgccjit/lib`
- both asserted: `crates/cli/tests/env.rs` JSON (`v["GCC_PATH"]`) + shell (`export GCC_PATH='…'`).
(This is a CLI env-seam surface; no GUI/Engine method involved — GUI parity N/A for an env export.)

## Unit ledger (derived from plan — Engine API delta + Work breakdown)
| U# | Unit | Present | Wired | Evidence |
|----|------|---------|-------|----------|
| U1 | `[[component]] id="libgccjit"` (detect/install/verify/remove) | ✓ | ✓ | `epic-h-toolchains.toml:417-456`; loaded → `auto-detect` lists it `[healthy] wired` |
| U2 | install: pin COMMIT from `libgccjit.version`, download `master-${COMMIT}/libgccjit.so` → `.so` + `.so.0` symlink, no `~/.local/bin` link | ✓ | ✓ | `toml:430-442`; on-box `.so` (426M) + `.so.0` SONAME present, no bin symlink |
| U3 | GCC_PATH in JSON env seam | ✓ | ✓ | `main.rs:1748`; runtime json emits `…/.toolchains/libgccjit/lib` |
| U4 | GCC_PATH in shell env seam | ✓ | ✓ | `main.rs:1807`; runtime shell emits `export GCC_PATH='…'` |
| U5 | env test GCC_PATH assertions (both forms) | ✓ | ✓ | `tests/env.rs`; 2/2 pass |
| U6 | `envctl.lock` regen 72→73 (`[components.libgccjit]`) | ✓ | ✓ | `envctl.lock` (+5); `lock --check` ✓ 73 components |
| U7 | ADR row marked SHIPPED | ✓ | ✓ | `docs/adr-install-locations-and-local-state.md:78` |
All ledger rows present AND wired (reached at runtime / referenced). No unwired stubs.

## Runtime check — PASS (architect declared a `## Runtime surface`; driven on-box)
Drove the real surfaces; install was NOT re-exercised (already converged on-box; the network
GitHub download is the deferred-on-sandbox path the architect noted — verified the resulting state
instead, which is the observable evidence).
| # | Surface driven | Result | Evidence (captured) |
|---|---------------|--------|---------------------|
| 1 | `envctl auto-detect` | **PASS** | `✓ libgccjit  libgccjit (meta-owned) [healthy] wired` |
| 2a | `env --toolchains --json \| grep GCC_PATH` | **PASS** | `"GCC_PATH": "/home/drdave/Desktop/meta/.toolchains/libgccjit/lib"` |
| 2b | `env --toolchains \| grep GCC_PATH` | **PASS** | `export GCC_PATH='/home/drdave/Desktop/meta/.toolchains/libgccjit/lib'` |
| 3 | component verify hook (`[ -f …libgccjit.so ] && file … \| grep 'shared object'`) | **PASS** | `VERIFY_HOOK_GREEN`, exit 0; `doctor` shows no libgccjit FAIL |
| 4 | installed artifact | **PASS** | `libgccjit.so` 426.3M + `.so.0 -> …libgccjit.so`; `file` → `ELF 64-bit LSB shared object, x86-64 … dynamically linked` |
| 5a | off-happy-path: `remove` hook self-guard | **PASS (read)** | deletes only `$M/.toolchains/libgccjit`; never a foreign path |
| 5b | off-happy-path: drift/boundary probe | **PASS** | libgccjit absent from ALL `auto-detect` drift entries; the 5 listed drift items are pre-existing (`weave` `~/.cargo/bin` boundary etc.), unrelated to this change — the meta-owned `.so` under `.toolchains/` is correctly NOT a boundary violation |

## Findings
None blocking. None non-blocking.

NOTE (informational, not a finding): `auto-detect`/`doctor` report 5 pre-existing drift items
(`weave` BoundaryViolation resolving to `~/.cargo/bin/weave` outside META_ROOT, etc.). These are
unrelated to TASK-0062 — present on `develop`, owned by other components — and do NOT involve
libgccjit. Out of scope for this cycle.

## Re-test needed
None — clean PASS. Should the orchestrator wish to re-confirm after rebase:
```
cd /home/drdave/Desktop/meta/.worktrees/task-0062-libgccjit/envctl
bash ci/gates/no-c.sh && bash ci/gates/shape.sh
cargo test -p envctl --test env
cargo run -p envctl -- lock --check        # expect: 73 components
META_ROOT=/home/drdave/Desktop/meta cargo run -p envctl -- auto-detect | grep libgccjit
```
