# Guardian report — TASK-0075 cognitum-seed-trust (cycle: seed-ca-trust)

## Verdict: PASS

## Invariants
- No-C trust boundary: PASS (pure manifest, no Cargo dep; `ci/gates/no-c.sh` PASS). Verify compares by raw bytes (`cmp -s`), no openssl.
- Engine single shared/sync/non-printing: PASS (no Rust touched; `ENVCTL_SEED_CA` knob already in `seam.rs:113-116`).
- Fail-closed + additive + absent-Seed no-op: PASS (every hook exits 0 when anchor absent; cp is additive; never removes other CAs; never scripts the passphrase; never reveals a secret). needs_sudo on install/fix/remove.
- No-system-depth: pins to `/usr/local` (the path secretd reads today); meta-path relocate deferred to TASK-0075b (carded).

## CI gates (all PASS)
no-c, shape, enable, kdf-feature-off, agent-env, loop-state, harness-scripts. `enable.sh` is scoped to `manifest/env-ctl.toml` — untouched.

## Runtime verification
1. `cargo run -p envctl -- auto-detect` -> `cognitum-seed-trust  Cognitum Seed Device-CA auto-refresh  wired` (detect: Missing/declared-not-installed, correct). No parse error.
2. `cargo run -p envctl -- lock --check` -> matches manifest (78 components), exit 0.
3. Worker (extracted from TOML, `bash -n` OK):
   - STALE pin -> re-pinned from `/run/media/drdave/COGNITUM/trust`; re-pinned content == anchor (YES); exit 0.
   - idempotent re-run -> "pin already current (no-op)"; exit 0.
   - glob fallback found the real mounted Seed (proves the `*` search path).
4. verify hook (`bash -n` OK): hard artifact predicate exits 1 pre-install (correct fail-closed); non-fatal byte-compare logic is identical to the verified worker path.

## Notes
- Truly-absent-Seed no-op path is logic-verified (read) but not live-exercised (the Seed is currently plugged in; the glob found it). Boot oneshot + hotplug udev both re-pin.
- udev trigger reuses the sibling's cdc_ncm NIC event; the mass-storage mount may lag the NIC slightly — the boot oneshot catches any miss. Acceptable for PR-1.
