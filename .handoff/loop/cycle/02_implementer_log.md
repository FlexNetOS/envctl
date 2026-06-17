# Implementation log: TASK-0036 — secretd in-process `mlockall` (FS-S4 process hardening)

**Status: GREEN.** All builds, clippy `-D warnings`, secretd tests (33+3+5+1), and the load-bearing
`no-c.sh` + `shape.sh` gates pass. Changes left uncommitted in the worktree for the guardian.

## Changes
- `Cargo.toml` (root): added `libc = "0.2"` under `[workspace.dependencies]` (matches resolved
  0.2.186) with a no-c rationale comment (pure-Rust FFI, already transitively resolved, not banned).
- `crates/secretd/Cargo.toml`: added `libc = { workspace = true }` to `[dependencies]` with a comment
  (FS-S4 mlockall only, pure-Rust FFI, no new lockfile crate, no-c-clean).
- `crates/secretd/src/config.rs`: added `require_mlock: bool` to `StoreConfig` (+ Debug field); new
  `[security].require_mlock` TOML table (`FileSecurity`, `#[serde(default)]` → false); new
  `SECRETD_REQUIRE_MLOCK` env override (`ENV_REQUIRE_MLOCK` const) parsed by a new `env_bool` helper
  (env wins over file); threaded `require_mlock` through `load()` → `resolve()` (signature +1 arg);
  updated all 7 existing `resolve(...)` test calls to the new arity; added 2 config tests.
- `crates/secretd/src/main.rs`: added `MlockOutcome` enum (`Locked` / `Failed{errno}` /
  `NotApplicable`) with `failed()`; rewrote `harden_process()` to return `MlockOutcome` and call the
  new Linux-gated `mlock_all_pages()` (non-Linux fallback returns `NotApplicable`); `serve` captures
  the outcome and applies the `require_mlock && mlock.failed()` fail-closed bail AFTER config load;
  `self_check` discards the outcome (best-effort, strict mode deliberately NOT honored); rewrote the
  deferral prose (module doc + NOTE block + the serve/self_check inline comments); added 3 tests.

## Engine API delta
NONE. Entirely in secretd's `main`/`config`. The engine, CLI, GUI, and proto are untouched — they
don't run this daemon-process path, so there is no parity surface to keep in sync (per the plan).

## The exact mlockall call + flags
`crates/secretd/src/main.rs::mlock_all_pages()` (Linux-gated `#[cfg(target_os = "linux")]`):
```rust
let rc = unsafe { libc::mlockall(libc::MCL_CURRENT | libc::MCL_FUTURE) };
```
On `rc == -1`: errno captured via `std::io::Error::last_os_error()` (`raw_os_error()`), a
**metadata-only** `tracing::warn!` (errno + strerror + fixed "secrets may be swappable, relying on
RLIMIT_CORE=0 + systemd LimitMEMLOCK" message — NO secret bytes), then returns `Failed{errno}` and
CONTINUES (never panics). `MCL_FUTURE` is load-bearing: it covers the DEK/vault-plaintext/PEM allocs
that happen post-`Lock.Unlock`. Strict-mode fatality (`anyhow::bail!`) is applied in `serve` after
config load, never inside `harden_process` (which runs before config is available).

## Tests added (+ pass counts)
- `tests::mlockall_best_effort_does_not_panic` — calls `mlock_all_pages()`; asserts a handled value
  (Failed/Locked/NotApplicable) with no panic; tolerates EPERM (Failed must carry a nonzero errno).
- `tests::harden_process_best_effort_default` — `harden_process()` returns a known variant normally
  even when mlockall EPERMs (startup doesn't crash).
- `tests::require_mlock_strict_fatal_when_unlocked` — pure-logic drive of the exact
  `require_mlock && outcome.failed()` predicate `serve` bails on (deterministic; constructed
  outcomes, no real lock needed).
- `config::tests::require_mlock_defaults_false_and_threads_through` — default false, threads through
  both backends when true.
- `config::tests::env_bool_parsing` — truthy/falsy/unrecognized/unset mapping for `env_bool`.

Test run: `cargo test -p envctl-secretd` → all suites pass:
`33 passed` (lib, incl. the 2 config tests) · `3 passed` (main bin `tests`) · `5 passed` ·
`1 passed` (e2e) · `1 ignored` (pre-existing). Total 0 failed. Ran under EPERM (no CAP_IPC_LOCK).

## No new lockfile crate (confirmed)
`git diff Cargo.lock` = **exactly 1 insertion**: `"libc"` added to the `envctl-secretd` dependency
list. NO new `[[package]]` entry — `libc` count in the lock stays 1 (it was already resolved via
tokio → signal-hook-registry → errno → libc). `no-c.sh` PASS: `rustls=['0.23.40']` on `ring` only,
zero aws-lc/openssl/C-SQLite. `shape.sh` PASS.

## Build/test status (exact commands)
- `cargo build -p envctl-secretd` → exit 0 (1 transient dead_code warning on `NotApplicable` fixed
  with a narrow `#[cfg_attr(target_os = "linux", allow(dead_code))]` on that variant only — see
  Deviations; NOT a broad crate/function allow).
- `cargo fmt --all` → exit 0 (reformatted the new `env_bool` match; no other drift).
- `cargo clippy --workspace -- -D warnings` → exit 0 (clean; no pre-existing baseline noise).
- `cargo test -p envctl-secretd` → exit 0 (counts above).
- `bash ci/gates/no-c.sh` → exit 0, "NO-C GATE PASS".
- `bash ci/gates/shape.sh` → exit 0, "SHAPE GATE PASS".

## Deviations
1. **Config placement** — the plan said "find where StoreConfig is parsed". `require_mlock` is a
   process-hardening flag, not a store flag, so I exposed it as a field ON `StoreConfig` (what `serve`
   already loads) but sourced it from a NEW top-level `[security]` TOML table rather than `[store]`,
   keeping the store/security concerns separate. Also added a `SECRETD_REQUIRE_MLOCK` env override
   (env-wins, matching the existing env > file > default precedence) — a small, in-spirit extension
   beyond the plan's TOML-only mention, so the flag is operable without a config file.
2. **`NotApplicable` dead-code allow** — on a Linux build the variant is only constructed by the
   `#[cfg(not(target_os = "linux"))]` fn, so clippy flags it. Scoped a single-variant
   `#[cfg_attr(target_os = "linux", allow(dead_code))]` (NOT a broad allow, NOT a lint silencer for
   real findings) so the strict-mode logic stays target-uniform. Rationale documented inline.

## Handoff notes (targeted checks for the guardian)
- **Fail-safe**: confirm `mlock_all_pages()` never panics on `-1` (it returns `Failed{errno}` +
  WARN). The `require_mlock_strict_fatal_when_unlocked` test proves the bail predicate; the
  `mlockall_best_effort_does_not_panic` test proves the EPERM path is handled (CI has no
  CAP_IPC_LOCK, so it exercises Failed, NOT Locked — do not expect a real lock).
- **Fail-closed strict mode**: the `anyhow::bail!` lives in `serve` AFTER config load (main.rs, just
  before `build_engine`), guarded by `store_cfg.require_mlock && mlock.failed()`. Verify `self_check`
  does NOT bail (it discards the outcome — `--self-check` must still exit 0 under EPERM; confirmed by
  the self-check/e2e suite passing).
- **No secret bytes in logs**: the only mlockall log is metadata-only (errno/strerror + a fixed
  message). Grep the WARN to confirm.
- **No-c**: `libc` is pure-Rust FFI, NOT on the banned list; Cargo.lock added exactly one line
  (`"libc"` under envctl-secretd), no new package. no-c.sh + shape.sh both green.
- **Linux-cfg**: the `mlockall` call is `#[cfg(target_os = "linux")]`; non-Linux gets the
  `NotApplicable` fallback so dev builds compile.

## Follow-ups
- **MADV_DONTDUMP** (the deferred companion to mlockall, named alongside it in
  `docs/secrets/THREAT-MODEL.md:8,77`): apply `madvise(MADV_DONTDUMP)` to secret-bearing regions so
  they're excluded from any core dump even if `RLIMIT_CORE` is somehow nonzero. Out of scope here;
  record as a new follow-up task — NOT widened into this change.
