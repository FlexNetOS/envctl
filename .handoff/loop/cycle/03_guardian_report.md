# Verification report: TASK-0061 — meta-owned llvm-clang toolchain (Epic H)

Worktree: `/home/drdave/Desktop/meta/.worktrees/task-0061-llvm/envctl`. `$META_ROOT=/home/drdave/Desktop/meta`.

=========================================================================
## RE-VERIFY (round 2) — branch `task-0061-llvm` @ `f5c3ba9` (on top of `b793876`)

### FINAL VERDICT — PASS
Both round-1 blocking/note findings are fixed and confirmed at the real surface. All static
gates, the env test, the lock, the blast-radius chain, and the on-box runtime health re-pass.
No re-pull (DEST present; GitHub API rate-limited until ~04:56 — download path unchanged since
round 1's successful pull, only the symlink/verify logic changed). Orchestrator may open the PR.

**Authoritative health line:** `✓ llvm-clang  LLVM/clang (meta-owned) [healthy] wired`

### The fix (diff `b793876...f5c3ba9`, `manifest/components.d/epic-h-toolchains.toml`, +17/-5)
- Install symlink loop now probe-gates each candidate (`"$src" --version` exit 0) before
  `ln -sfn`, and self-prunes any stale symlink it owns (resolving into `$DEST`) when the binary
  no longer runs — so `lld`/`ld.lld` auto-drop on a `libxml2.so.16` box and the round-1 broken
  `lld` symlink is healed. `verify` hook: `clang --version && lld --version` → `clang --version
  && llvm-config --version`. `remove` loop covers the full candidate list (incl. `llc`/`lld`/`ld.lld`).
  `llc` restored to the curated set. No Rust source changed (manifest TOML + lock only).

### Static — all PASS
- `cargo build -p envctl-engine -p envctl` : PASS (no Rust change → cached)
- `cargo fmt --all --check` : PASS · `cargo clippy --workspace -- -D warnings` : PASS (clean)
- `cargo test -p envctl --test env` : PASS — 2/2 (LIBCLANG_PATH JSON+shell)
- `envctl lock --check` : PASS — `✓ matches the manifest (72 components)`; `[components.llvm-clang]`
  `content_hash de16a4e1f3ddf18b → b66d8854ad82aa99`, `requires = []` (tarball def still wins)
- `no-c.sh` : PASS (`rustls 0.23.40 on ring`, zero C) · `shape.sh` : PASS
- No new crate dep (Cargo.lock/toml untouched); no engine source change — invariants intact.

### Blast-radius — PASS (re-confirmed)
`cuda-oxide`/`gpu-stack` `requires` resolve (`group-gpu-stack 9 requires`); `llc` (named by the
replaced apt def for cuda-oxide bindgen) is back on PATH; id-preserved last-wins still holds.

### Runtime (Phase 3.5) — PASS — no re-pull, DEST intact
| Check | Result | Evidence |
|-------|--------|----------|
| `auto-detect` health | **PASS** | `✓ llvm-clang … [healthy] wired` (was `[unhealthy]` in round 1) |
| component verify hook (`clang && llvm-config`) | **PASS** | direct run exit 0 |
| `clang --version` | PASS | `clang version 21.1.8`, exit 0 |
| `llc --version` | PASS | `LLVM version 21.1.8`, exit 0 (Finding 2 fixed — restored) |
| `llvm-config --version` | PASS | `21.1.8`, exit 0 |
| `lld` / `ld.lld` | PASS | ABSENT — probe-pruned on this libxml2.so.16 box (Finding 1 fixed) |
| curated symlink set | PASS | exactly the 10: clang, clang++, clang-21, clang-cpp, llc, llvm-ar, llvm-config, llvm-nm, llvm-objcopy, llvm-objdump — all → `.toolchains/llvm/bin` |
| `env --toolchains --json` LIBCLANG_PATH | PASS | `…/.toolchains/llvm/lib` |
| idempotency re-run | PASS | `— skip llvm-clang (already present)`, exit 0; DEST intact (no re-extract) |

### Resolved from round 1
- **Finding 1 (BLOCKING) — lld/ld.lld shipped non-functional:** RESOLVED. Probe-gate drops them
  on this box; verify hook no longer depends on lld; component is `[healthy]`.
- **Finding 2 (NOTE) — llc dropped from PATH:** RESOLVED. `llc` restored to the symlink/remove sets and on PATH.

### Caveat
On-box state was already converged (the implementer re-ran the fixed install pre-handoff; `llc`
symlink stamped Jun 23 04:31, lld absent). The 2 GB download path is unchanged since round 1's
successful pull and was deliberately NOT re-exercised (API rate-limited); only the symlink/verify
logic — fully re-confirmed above — changed.

=========================================================================
## ROUND 1 (historical) — @ `b793876` — verdict FAIL (both findings now fixed above)

(Supersedes the stale TASK-0053 report previously at this path.)

## Verdict — FAIL (1 blocking runtime finding) — static layer + env seam are fully GREEN

The static gates, parity test, lock, and the LIBCLANG_PATH env seam all pass. The component
installs idempotently and clang converges correctly. **But the delivered component's own
`verify` hook FAILS on the live box** — `lld`/`ld.lld` are shipped non-functional (they link
the absent `libxml2.so.2`), so `auto-detect` reports the component `[unhealthy]`, not healthy.
The runtime surface does not converge to the state the checklist required (step 5: "now reports
installed/healthy"). Per invariant #10 a runtime FAIL is blocking and routes to the implementer.

## Gate results
- `bash ci/gates/no-c.sh` : **PASS** — `rustls=['0.23.40'] on ring=['0.17.14']; zero aws-lc/openssl/C-SQLite`
- `bash ci/gates/shape.sh` : **PASS** — `SHAPE GATE PASS`
- (enable.sh not relevant — no secretd surface in this manifest/CLI diff)

## cargo
- `cargo build -p envctl-engine -p envctl` : **PASS**
- `cargo fmt --all --check` : **PASS**
- `cargo clippy --workspace -- -D warnings` : **PASS** (clean; no inherited gui/main.rs lint at this snapshot)
- `cargo test -p envctl --test env` : **PASS** — 2/2 (`toolchains_shell_exports_rustup_home_with_cargo_home`, `toolchains_json_carries_rustup_home`)
- `./target/debug/envctl lock --check` : **PASS** — `✓ envctl.lock matches the manifest (72 components)`

## Invariant checks
1. No-C trust boundary — **PASS**. `Cargo.lock`/`Cargo.toml` untouched (no new crate dep); no-c gate green.
2. Code-shape — **PASS** (shape.sh green).
3. secretd enable — **N/A** (no secretd surface).
4. Engine purity — **PASS**. Zero `crates/engine/src` files changed; the new env-seam logic is CLI-only printing in `crates/cli/src/main.rs` (an existing print surface), not the engine library.
5. Front-end parity — **PASS/NOTE**. LIBCLANG_PATH seam is in `envctl env --toolchains`; both JSON (`run_env:1747`) and shell (`:1800-1803`) forms carry it. CLI env-export surface with no GUI counterpart — consistent with sibling OLLAMA/RUSTUP exports.
6. Fail-closed / dry-run — **PASS**. `install` is additive/idempotent; `--dry-run` previews ("would Install"); the `remove` hook is self-guarded — only unlinks `~/.local/bin/*` symlinks whose `readlink` resolves into `$M/.toolchains/llvm` before `rm -rf` of the prefix.
7. Rust-native, no drift — **PASS**. No non-Rust source/package files; component is a TOML manifest block; no banned dep.
8. Lock honesty — **PASS**. `[components.llvm-clang]` regenerated: `content_hash de16a4e1f3ddf18b`, `requires = []` (was `["nvidia-cuda-repo"]`) — tarball def won last-wins-on-id. `lock --check` clean.
9. Kasetto/agent-env — **N/A**.
10. Runtime behavior — **FAIL** (see Runtime check + Findings).

## Parity check (env seam)
- JSON: `crates/cli/src/main.rs:1747` → `map["LIBCLANG_PATH"] = "{tc}/llvm/lib"`.
- Shell: `crates/cli/src/main.rs:1800-1803` → `export LIBCLANG_PATH=…`.
- Both verified live: `env --toolchains --json` and shell form emit `…/.toolchains/llvm/lib`.

## Unit ledger (derived from diff — no `## Unit ledger` in plan packet at this path)
| U# | unit | present | wired | evidence |
|----|------|---------|-------|----------|
| U1 | `llvm-clang` tarball component | YES | YES (auto-detect parses + graph resolves) | `manifest/components.d/epic-h-toolchains.toml:345` |
| U2 | apt `llvm-clang` removed from gpu.toml | YES | YES (note left; id preserved) | `manifest/gpu.toml:105-110` |
| U3 | LIBCLANG_PATH seam (JSON+shell) | YES | YES (env tests + runtime) | `crates/cli/src/main.rs:1747,1800` |
| U4 | env.rs assertions | YES | YES (2/2 pass) | `crates/cli/tests/env.rs:68,96` |
| U5 | lock regen | YES | YES (lock --check clean) | `manifest/envctl.lock:243` |

## BLAST-RADIUS (gpu.toml apt→tarball swap) — PASS
- `requires = [… "llvm-clang" …]` intact: `cuda-oxide` (`gpu.toml:141`), `gpu-stack` (`gpu.toml:279`). `envctl graph` resolves the chain (`group-gpu-stack 9 requires`) — no dangling require.
- Last-wins confirmed: auto-detect shows tarball name "LLVM/clang (meta-owned)"; lock shows tarball `content_hash` + `requires = []`, NOT the old `requires = ["nvidia-cuda-repo"]`.
- No other component references the removed apt def's side effects.

## Runtime check — FAIL
Real install on the live box (`envctl install llvm-clang`, 36s, `✓ llvm-clang Install`).
Pre-state: apt clang 21.1.8 at `/usr/bin`; no `~/.local/bin/clang`; no `.toolchains/llvm`.

| Step | Result | Evidence |
|------|--------|----------|
| dry-run preview | PASS | `would Install llvm-clang` |
| install (real) | PASS | `✓ llvm-clang Install`, `wiring applied` |
| `clang --version` | PASS | `clang version 21.1.8 (…llvm-project 2078da…)`, target `x86_64-unknown-linux-gnu` |
| `clang -print-resource-dir` | PASS | `…/.toolchains/llvm/lib/clang/21` (realpath into the prefix — symlink reasoning holds) |
| `command -v clang` shadows apt | PASS | `/home/drdave/.local/bin/clang` (meta ahead of `/usr/bin`) |
| `libclang.so*` present | PASS | `libclang.so → .so.21.1 → .so.21.1.8` (199 MB) under `.toolchains/llvm/lib` |
| `env --toolchains` LIBCLANG_PATH | PASS | JSON + shell emit `…/.toolchains/llvm/lib` |
| **`lld --version`** | **FAIL** | `lld: error while loading shared libraries: libxml2.so.2: cannot open shared object file` — **exit 127** |
| **verify hook** (`clang --version && lld --version`) | **FAIL** | chain **exit 127** (lld short-circuits) |
| `auto-detect` post-install health | **FAIL** | `✓ llvm-clang … [unhealthy] wired` → `[high] Unhealthy: installed but verify failed` |
| idempotency re-run | PASS | `— skip llvm-clang (already present)`, exit 0 |

## Findings
1. **BLOCKING — `lld`/`ld.lld` shipped non-functional (libxml2 soname mismatch).**
   `~/.local/bin/lld` and `ld.lld` link `libxml2.so.2`, which is absent — the box only has
   `libxml2.so.16` (newer Ubuntu soname); the tarball bundles no libxml2. The component's own
   `[component.verify]` is `clang --version && lld --version`, so a clean install is permanently
   `[unhealthy]` and the LLD linker (a flagship deliverable, named in the component) does not run.
   Evidence: `ldd ~/.local/bin/lld → libxml2.so.2 => not found`; `lld --version → exit 127`;
   `auto-detect → [high] Unhealthy: installed but verify failed`.
   Fix options: (a) make the install hook provide/symlink a compatible libxml2 ABI only when
   genuinely ABI-compatible (a soname bump may not be); (b) declare the system `libxml2`(`.so.2`)
   dependency / `requires`; (c) use an LLVM tarball whose lld is statically linked or links the
   current soname; (d) if lld isn't needed downstream, drop `lld`/`ld.lld` from BOTH the symlink
   set AND the verify hook. The verify hook and the shipped surface must agree — today they don't.

2. **NOTE / possible downgrade — `llc` dropped from the PATH symlink set.**
   The replaced apt def's description explicitly said *"Provides **llc** + libclang resource-dir
   headers cuda-oxide's bindgen needs"* and detected `command -v llc-21`. The tarball ships `llc`
   (`.toolchains/llvm/bin/llc`) but the new symlink loop omits it, so `llc` is not on PATH. If
   cuda-oxide's build reaches `llc` by PATH name this is a regression for the named consumer; if it
   reaches it via `llvm-config`/resource-dir it's harmless. Implementer should confirm cuda-oxide's
   lookup and, if PATH-based, add `llc` to the symlink + remove sets.

3. **NOTE — additive coexistence correct.** apt `clang-21`/`llvm-21`/`libclang-21-dev` remain
   installed (removal is a separate sudo step per plan); `~/.local/bin` correctly shadows `/usr/bin`.

## Re-test needed (after fix)
```
cd /home/drdave/Desktop/meta/.worktrees/task-0061-llvm/envctl
cargo build -p envctl-engine -p envctl
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
cargo test -p envctl --test env
./target/debug/envctl lock --check                # regen first if the component block changes
bash ci/gates/no-c.sh && bash ci/gates/shape.sh
# Runtime — the gate that currently fails:
./target/debug/envctl install llvm-clang
PATH="$HOME/.local/bin:$PATH" lld --version                       # must exit 0
./target/debug/envctl auto-detect 2>/dev/null | grep -i llvm     # must report healthy, not [unhealthy]
command -v llc                                                     # if Finding 2 accepted: ~/.local/bin/llc
```
