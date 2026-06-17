# Verification report: TASK-0036 — secretd in-process `mlockall` (FS-S4 process hardening)

## Verdict — PASS

The change is exactly what the plan/log claim: a small, secretd-local, Linux-gated `mlockall`
addition (best-effort default + `require_mlock` strict fail-closed), one new direct dep edge
(`libc`, already transitively resolved → no new lockfile crate), engine/CLI/GUI/secrets-engine
untouched. All gates + cargo checks green; all three named mlockall tests + both config tests pass.

(Supersedes the prior TASK-0030 cycle report that previously occupied this file.)

### Baseline note (important)
The correct review baseline is **`git diff HEAD`** (HEAD = `d2ddcc2`, the current tip of
`origin/develop`), NOT `git diff develop`. The local `develop` ref in this worktree is **stale**
(points at the pre-Epic-C #45 merge `3707680`), so a `develop` diff falsely attributes the entire
kasetto absorption (agent-env, baby-mimalloc, secrets stack rework) to this task. Verified against
`HEAD`, the TASK-0036 surface is exactly 5 source files + 2 handoff docs. No drift, no scope creep.

## Gate results
- `ci/gates/no-c.sh` : **PASS** (exit 0) — "resolved graph clean: rustls=['0.23.40'] on ring=['0.17.14']; zero aws-lc/openssl/C-SQLite" / "NO-C GATE PASS"
- `ci/gates/shape.sh` : **PASS** (exit 0) — "SHAPE GATE PASS"
- `ci/gates/enable.sh` : **PASS** (exit 0) — "ENABLE GATE PASS"
- `ci/gates/p7.sh` : **PASS** (exit 0) — "P7 GATE PASS"

## cargo
- `cargo fmt --all -- --check` : **PASS** (exit 0)
- `cargo clippy --workspace -- -D warnings` : **PASS** (exit 0, clean finish)
- `cargo test -p envctl-secretd` : **PASS** (exit 0) — lib 33 passed, bin `tests` 3 passed, integration suites all green (native_mint_e2e 11, proxy_swap_e2e 2, self_check 2, e2e/mitm_e2e green); 0 failed. Ran under EPERM (no CAP_IPC_LOCK) — tests tolerated it as designed.
  - Named mlockall tests (re-run, isolated): `tests::mlockall_best_effort_does_not_panic` ok · `tests::harden_process_best_effort_default` ok · `tests::require_mlock_strict_fatal_when_unlocked` ok
  - Config tests: `config::tests::require_mlock_defaults_false_and_threads_through` ok · `config::tests::env_bool_parsing` ok (part of the 33-test lib suite)

## Invariant checks
1. **No-C / dep hygiene** : **PASS**. Only `libc` added. `git diff HEAD -- Cargo.lock` = 1 insertion (`"libc"` under `envctl-secretd` deps); `grep -c '^+\[\[package\]\]'` on the lock diff = **0** new packages; `libc` package count in lock stays **1** (already resolved via tokio→signal-hook-registry→errno→libc). Cargo.toml edits limited to the libc edge: root `[workspace.dependencies] libc = "0.2"` + `crates/secretd/Cargo.toml libc = { workspace = true }`. libc is NOT on the no-c banned list (aws-lc/openssl/sqlite/mimalloc). secrets-engine `Cargo.toml` and `src/` untouched.
2. **Flags + cfg-gate** : **PASS**. `crates/secretd/src/main.rs` `libc::mlockall(libc::MCL_CURRENT | libc::MCL_FUTURE)` inside `mlock_all_pages()`, which is `#[cfg(target_os = "linux")]`; the `#[cfg(not(target_os = "linux"))]` fallback returns `NotApplicable` so non-Linux compiles. Call is placed AFTER the RLIMIT_MEMLOCK raise — `harden_process()` does setrlimit(Core)=0 → raise RLIMIT_MEMLOCK → then `mlock_all_pages()`.
3. **Fail-safe, never panics** : **PASS**. On `rc == -1`: errno captured via `std::io::Error::last_os_error().raw_os_error()`, a metadata-only `tracing::warn!(errno, error = %err, "...")` (fixed message; no secret bytes), returns `MlockOutcome::Failed { errno }`, and CONTINUES. No `.unwrap()`/`.expect()`/`panic!` on the syscall result. The `unsafe` block is narrowly scoped to the single syscall with a SAFETY comment.
4. **Strict mode applied AFTER config load** : **PASS**. `serve()` captures `let mlock = harden_process()` BEFORE config exists, then bails `if store_cfg.require_mlock && mlock.failed()` AFTER `StoreConfig::load()`, just before `build_engine`. `anyhow::bail!` = fail-closed refusal. `self_check()` does `let _ = harden_process();` — strict mode deliberately NOT honored, stays best-effort (confirmed: self_check/e2e suites pass under EPERM).
5. **No secret bytes in logs** : **PASS**. The only mlockall log is the WARN — fields `errno` + `error = %err` (strerror) + a fixed string. The word "secret" appears only in doc comments / the fixed message ("secret material may be swappable"); no secret value, DEK, token, passphrase, or vault byte is logged. Placement is pre-unlock, so no secret material exists in the address space at that point anyway.
6. **Engine purity** : **PASS**. `git diff HEAD -- crates/engine/` and `crates/secrets-engine/` both EMPTY. No Event, no `println!`/`eprint!`, no clap, no engine logic added. Change is entirely in `secretd::main` + `secretd::config`.
7. **Front-end parity** : **N/A (justified)**. No Engine method added — this is daemon-process hardening that runs only in `secretd serve`/`--self-check`. CLI/GUI don't run this path, so there is no parity surface to keep in sync (per the plan).
8. **Lock/manifest honesty** : **PASS**. No `[[package]]` added; manifest/*.toml, envctl.lock, kasetto.lock unchanged (secretd `--self-check` still passes under EPERM, confirmed by the self_check suite). Config tests prove `require_mlock` defaults false and threads through both backends; `env_bool` env-override precedence verified.

## Parity check
No Engine method introduced → no CLI/GUI caller required. Daemon-internal `harden_process()` →
called by `serve()` and `self_check()`, both within `crates/secretd/src/main.rs`.

## Findings
None blocking.

- (note, non-blocking) Stale local `develop` ref in the worktree — a `git diff develop` here is
  misleading (attributes all of Epic C to this task). Reviewers/orchestrator must diff against `HEAD`
  (`d2ddcc2`) or `origin/develop` to see the true TASK-0036 surface. Not a code issue; flagged so
  the change isn't mis-attributed or mis-merged.
- (note, non-blocking, in plan's "Out of scope") MADV_DONTDUMP — the deferred companion named
  alongside mlockall in THREAT-MODEL.md:8,77 — is correctly recorded as a follow-up, not widened here.

## Re-test needed
None for a PASS verdict. To reconfirm after any further edit, from the worktree root:
```
bash ci/gates/no-c.sh ; bash ci/gates/shape.sh ; bash ci/gates/enable.sh ; bash ci/gates/p7.sh
rtk proxy cargo fmt --all -- --check
rtk proxy cargo clippy --workspace -- -D warnings
rtk proxy cargo test -p envctl-secretd
```
