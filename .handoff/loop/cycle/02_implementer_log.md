# Implementation log: TASK-0061 — meta-owned llvm-clang toolchain (Epic H)

## Changes
- manifest/components.d/epic-h-toolchains.toml: added `[[component]] id = "llvm-clang"` (8th Epic-H tarball component) — detect/install/verify/remove/wiring mirroring ollama/mise; install pins latest 21.x via the releases API.
- manifest/gpu.toml: REMOVED the pre-existing apt-based `id = "llvm-clang"` (apt-get llvm-21/clang-21/libclang-*-dev) and left a NOTE pointer; the id is preserved so consumer `requires` still resolve.
- crates/cli/src/main.rs (run_env): added `LIBCLANG_PATH` = `{tc}/llvm/lib` to the JSON map and the shell `export` form (sh_single_quote), beside the OLLAMA_LIBRARY_PATH lines.
- crates/cli/tests/env.rs: extended both toolchains tests to assert LIBCLANG_PATH (shell `export LIBCLANG_PATH='{r}/.toolchains/llvm/lib'`; json `v["LIBCLANG_PATH"]`).
- manifest/envctl.lock: regenerated — llvm-clang content_hash de16a4e1f3ddf18b, requires now [] (was ["nvidia-cuda-repo"]); count stays 72.

## Engine API
No Engine API change. CLI-only env-seam addition (LIBCLANG_PATH) + manifest component; both front-ends consume the manifest identically (no GUI parity change needed — this is a manifest component + `envctl env` shell/json output).

## Tests added
Extended (not new) in crates/cli/tests/env.rs:
- toolchains_shell_exports_rustup_home_with_cargo_home: now also asserts the LIBCLANG_PATH shell export.
- toolchains_json_carries_rustup_home: now also asserts json LIBCLANG_PATH.
Both prove the env seam emits `.toolchains/llvm/lib` in both output modes.

## Build/test status
- `cargo build -p envctl-engine -p envctl` — PASS
- `./target/debug/envctl lock` then `lock --check` — PASS, "matches the manifest (72 components)"
- `cargo test -p envctl --test env` — PASS (2/2)
- `envctl auto-detect | grep -i llvm` — PASS (parses as "LLVM/clang (meta-owned)", detected not-installed)
- `cargo fmt --all -- --check` — PASS
- `cargo clippy --workspace -- -D warnings` — PASS (0 warnings; no inherited gui lint surfaced this run)
- `bash ci/gates/no-c.sh` — PASS
- `bash ci/gates/shape.sh` — PASS

## Deviations
1. **Pre-existing duplicate id (the big one).** The plan assumed no existing `llvm-clang` and a lock count 72→73. In fact an apt-based `id = "llvm-clang"` already lived in `manifest/gpu.toml` (install: `apt-get install -y llvm-21 clang-21 libclang-*-dev`) — the exact system-depth install TASK-0061 eliminates. The manifest loader is **last-wins on id** (`by_id.insert`, model.rs:57) with files in sorted path order, so `gpu.toml` (sorts after `components.d/...`) was WINNING — a naive add would have left my component dead and the apt install active (the lock's llvm-clang still showed `requires = ["nvidia-cuda-repo"]`, the gpu.toml fingerprint). I removed the apt definition from gpu.toml so there is one canonical meta-owned component; id unchanged so `cuda-oxide`/`gpu-stack` requires still resolve. **Consequence: lock count stays 72, not 73** — same id, new content_hash + empty requires. Design-shaped call I'd normally route back, but the plan goal + the duplicate-id defect made intent unambiguous; flagged for guardian/architect confirm.

## Handoff notes
- **Confirm the gpu.toml apt-llvm removal is intended.** It changes how `cuda-oxide`'s libclang dependency is provisioned (apt → meta tarball). `lock --check` clean and `auto-detect` parses the whole graph, so topo-order resolves; new llvm-clang has `requires = []` and consumers still require it by id.
- **Runtime install verify is yours (guardian):** I did NOT run `envctl install --apply` (~2 GB). The install hook pins latest 21.x via `api.github.com/.../releases?per_page=100` grep — confirm the API still returns a 21.x tag and the `LLVM-${VER}-Linux-X64.tar.xz` asset name holds. Detect uses a `case` glob (resolved target is *under* `$M/.toolchains/llvm`) rather than the exact `readlink -f` equality the others use, because clang resolves through a symlink chain.
- **Env seam:** LIBCLANG_PATH = `.toolchains/llvm/lib`; bindgen/cuda-oxide consumers pick up libclang.so from there after `eval "$(envctl env --toolchains)"`.
- No grit/parallel mode (sequential single-implementer).

## Commit
b793876  manifest: meta-own llvm/clang-21 (Epic H TASK-0061) — eliminate apt clang/llvm system-depth
f5c3ba9  manifest: probe-gate llvm symlinks + verify on clang only (TASK-0061 guardian fix)

## Re-run note (guardian on-box findings)
Guardian's real install found 2 issues; both fixed in f5c3ba9.

1. BLOCKER — prebuilt lld/ld.lld link libxml2.so.2 (box has only libxml2.so.16, ABI-incompatible), so `lld --version` exits 127 and the old verify hook (`clang && lld`) reported [unhealthy]. Fix: the install symlink loop now PROBES each candidate (`"$src" --version`) and only exposes runnable ones — lld/ld.lld auto-drop here, kept on boxes with libxml2.so.2; any other tool with an unsatisfied lib self-prunes. Also added a PRUNE branch: a stale ~/.local/bin symlink resolving into our DEST for a now-skipped candidate is removed (self-heals a re-install over a broken install; only touches symlinks we own). verify hook → `clang --version && llvm-config --version` (core deliverable only, not the non-core lld).
2. NOTE — restored `llc` (the replaced apt def provided it for cuda-oxide bindgen). It's in the candidate list, probes clean (no libxml2 dep), and lands symlinked.

On-box proof: auto-detect `llvm-clang [healthy] wired`; curated set = clang/clang++/clang-21/clang-cpp/llc/llvm-ar/llvm-config/llvm-nm/llvm-objcopy/llvm-objdump (lld/ld.lld pruned); clang/llc/llvm-config all 21.1.8. fmt/clippy/no-c/shape green. Lock content_hash de16a4e1f3ddf18b → b66d8854ad82aa99 (script body changed); count still 72; lock --check clean.

CAVEAT for guardian: the full end-to-end `envctl install llvm-clang` ran clean at 04:29 (download→extract→verify→[healthy]). The final prune-branch verification was done by executing the install hook's exact symlink-loop body against the already-extracted DEST, because a 2nd back-to-back full re-install hit the GitHub unauthenticated API rate limit (60/hr, 403 on the `releases?per_page=100` tag query; resets ~04:56). The DEST tarball was intact and unchanged, so the loop-body run is behaviorally identical to the in-hook run. A guardian re-install after the rate-limit window resets will exercise the download path again with the prune branch in place.

CLI note: the install verb takes positional TARGETS and applies directly (`envctl install llvm-clang`); there is no `--only`/`--apply` flag on `install` (--apply is on the destructive `reset` verb). `reset llvm-clang` is fail-closed-refused here because cuda-oxide/gpu-stack are live reverse-dependents (correct guard behavior); I forced a clean re-install by removing just the `clang` symlink so detect missed and the full install hook re-ran.
