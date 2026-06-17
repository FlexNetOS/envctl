# TASK-0036 — secretd in-process `mlockall` (FS-S4 process hardening) · VERDICT: GO

Small, secretd-local, Linux-gated. No engine/CLI/GUI/proto surface. Lock the daemon's memory so the
DEK/vault plaintext/PEMs never reach swap. mlockall was deferred because pinned rustix lacks the `mm` module.

## Target repos
1 — envctl. One crate: crates/secretd. No lock/manifest churn.

## Dep decision (no-C proof)
Add `libc` as a DIRECT dep of crates/secretd ONLY; call `libc::mlockall(libc::MCL_CURRENT | libc::MCL_FUTURE)`.
- Chosen over rustix `mm` feature: rustix pin (root Cargo.toml:37) is a SHARED workspace pin (already a
  contested cross-crate negotiation per the CONFLICT-1 comment); adding `mm` widens it for every consumer.
  libc is already resolved in secretd's tree (cargo tree -p envctl-secretd -i libc → tokio→signal-hook-
  registry→errno→libc), so promoting to a direct dep adds ZERO new lockfile crates and is secretd-scoped.
- no-C proof: libc is pure-Rust FFI bindings (extern "C" decls; compiles no C, links no new C lib) and is
  NOT on the no-c banned list (aws-lc/openssl/sqlite/mimalloc — no-c.sh:26,33,42,57,74). Expect no-c.sh green.

## Placement
crates/secretd/src/main.rs::harden_process() (body ~294-313): mlockall call AFTER the RLIMIT_MEMLOCK raise
(~312), before return (~313). harden_process is called at ~120 (serve) and ~90 (self_check), BOTH before
Engine::open/build_engine (~134) and before any Lock.Unlock puts plaintext in the address space → MCL_FUTURE
covers DEK/vault/PEM allocs that happen later. Update deferral prose: module doc ~6-8, NOTE block ~15-21,
inline comments ~118-119 (serve) and ~87-89 (self_check).

## Flags: MCL_CURRENT | MCL_FUTURE
FS-S4 requires secrets never swapped; DEK/vault are allocated AFTER startup (post-unlock) → MCL_FUTURE is
load-bearing. Spec names this combo literally (THREAT-MODEL.md:8). RLIMIT_MEMLOCK already raised to infinity
(:307) + systemd LimitMEMLOCK=infinity mitigates the ENOMEM-on-future-alloc trade-off. NOT MCL_ONFAULT (lazy,
weakens the never-swapped guarantee).

## Best-effort + fail-safe (CRITICAL)
mlockall can fail EPERM (no CAP_IPC_LOCK / low RLIMIT_MEMLOCK), ENOMEM, EAGAIN.
- DEFAULT = best-effort, NEVER fatal: on -1, capture errno via std::io::Error::last_os_error(), emit a
  structured WARN (metadata only — errno/strerror + fixed message; no secret bytes — none in scope pre-unlock
  anyway), then CONTINUE. RLIMIT_CORE=0 (:295-303) independently mitigates core-dump leakage.
- STRICT opt-in `require_mlock: bool` (serde default false) on the secretd config: when true, mlockall failure
  is FATAL — serve returns anyhow::Err (fail-closed, daemon refuses to start). Honors THREAT-MODEL.md:8
  (refuse-on-fail) as an operator-elected hardened mode while keeping a safe default (THREAT-MODEL.md:77
  best-effort). In self_check(), mlockall stays best-effort REGARDLESS of require_mlock (non-serving pre-flight).
- cfg-gate the mlockall call `#[cfg(target_os = "linux")]` so non-Linux dev builds compile (MCL_* are Linux).

## Engine API delta
NONE. Daemon process hardening in secretd::main; engine/CLI/GUI untouched (can't diverge — they don't run this path).

## Lock/manifest sync
crates/secretd/Cargo.toml: add `libc = { workspace = true }` ([dependencies]); add `libc = "0.2"` to root
[workspace.dependencies] if missing (match resolved 0.2.186). No new lockfile crate. manifest/*.toml,
envctl.lock, kasetto.lock: NO change (secretd verify predicate --self-check still passes; mlockall best-effort there).

## Sequencing (leaf-first)
1. root Cargo.toml: ensure libc under [workspace.dependencies]. 2. crates/secretd/Cargo.toml: add libc dep
(comment: pure-Rust FFI, already resolved, no-c-clean, for mlockall only). 3. config: add require_mlock:bool
(serde default false) + doc. 4. harden_process: add cfg(linux) mlockall(CURRENT|FUTURE) returning a MlockOutcome;
WARN metadata-only on fail. NOTE: harden_process is called (:120/:90) BEFORE config load (:132/:102) — keep the
SYSCALL best-effort there and apply require_mlock fatality in serve AFTER config load (if require_mlock &&
outcome.failed → bail!). 5. rewrite deferral prose (:6-8, :15-21, :118-119, :87-89). 6. tests. 7. fmt/clippy
--workspace -D warnings + cargo test -p envctl-secretd + no-c.sh + shape.sh.

## Tests (tolerate EPERM in CI — daemon usually lacks CAP_IPC_LOCK)
- mlockall_best_effort_does_not_panic: wrapper returns Ok/locked OR handled Err/not-locked WITHOUT panic; must
  tolerate EPERM (assert clean not-locked path, NOT success).
- harden_process_best_effort_default: returns normally even when mlockall EPERMs (startup doesn't crash).
- require_mlock_strict_fatal_when_unlocked: pure-logic check that outcome.failed + require_mlock → Err
  (deterministic, doesn't depend on actually locking in CI).
- secretd --self-check still exits 0 under EPERM.
Gates: no-c.sh (load-bearing — must stay green), shape.sh. enable.sh/p7.sh unaffected.

## Invariants (each checkable)
- no-C: libc pure-Rust FFI, already resolved, not banned → no-c.sh green.
- one rustls ring-only: no TLS dep touched.
- engine single non-printing lib: change entirely in secretd::main; no engine code/Event/println!/clap.
- fail-safe: mlockall failure handled (WARN+continue), never panics; require_mlock strict = fail-closed refusal.
- Linux-cfg: non-Linux compiles (mlockall call cfg-gated).

## Risks
harden_process called before config load → apply require_mlock fatality in serve after config (step 4). MCL_FUTURE
+ memlock pressure → ENOMEM later (mitigated by infinity RLIMIT_MEMLOCK + systemd). CI lacks CAP_IPC_LOCK → EPERM
(handled; tests tolerate).

## Out of scope (follow-up)
MADV_DONTDUMP (named alongside mlockall in THREAT-MODEL.md:8,77) — record as a new follow-up task, NOT widened here.
