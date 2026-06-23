# Implementation log: TASK-0075b — relocate Seed Device-CA pin off /usr/local to a meta path

PURE-MANIFEST change. Zero Rust source edits (the `ENVCTL_SEED_CA` override seam already exists at
`crates/secrets-engine/src/seam.rs:113-116`).

## Changes
- `manifest/env-ctl.toml`: secretd `[Service]` — added `Environment=ENVCTL_SEED_CA=%h/Desktop/meta/.toolchains/secrets/ca/cognitum-ca.crt` (U1) and `ReadOnlyPaths=%h/Desktop/meta/.toolchains/secrets/ca` (U2).
- `manifest/cognitum-seed-trust.toml`: relocated the DST default from `/usr/local/share/ca-certificates/cognitum-ca.crt` to `${META_ROOT:-$HOME/Desktop/meta}/.toolchains/secrets/ca/cognitum-ca.crt` in all 5 plan-listed places (install worker DST, fix worker DST, verify probe DST, `[[component]]` description, header comment block incl. NO-SYSTEM-DEPTH NOTE now marked DONE) + the 2 REMOVE-block comments (U3).
- `manifest/envctl.lock`: regenerated (78 components; env-ctl + cognitum-seed-trust content_hash updated) (U4).
- `.handoff/loop/cycle/01_architect_plan.md`: architect's TASK-0075b plan (this cycle's planning output, overwrote the prior committed plan).

## Engine API
None changed. `seam.rs::ca_path()` (U5) still reads `ENVCTL_SEED_CA`, defaulting to the /usr/local path only when the env var is unset — and the secretd unit now sets it (U1). Read-only confirmed: `git diff -- crates/secrets-engine/src/seam.rs` is empty.

## Tests added
None (pure manifest/config; no Rust). Verification is by parse + lock + sandbox-reachability + CI gates below.

## Build/test status — exact commands run + PASS/FAIL
1. `cargo run -p envctl -- auto-detect | grep -iE 'cognitum-seed-trust|env-ctl|parse|error'` — PASS. Both TOMLs parse; `env-ctl … wired`, `cognitum-seed-trust … wired`. No parse error/panic.
2. `cargo run -p envctl -- lock` then `cargo run -p envctl -- lock --check` — PASS, exit 0. "✓ envctl.lock matches the manifest (78 components)".
3. Sandbox reachability (crux): seeded a dummy PEM at `~/Desktop/meta/.toolchains/secrets/ca/cognitum-ca.crt`, then `systemd-run --user --pty --wait -p ProtectHome=read-only -p ProtectSystem=strict cat <meta CA path>` — PASS, RC=0, printed the PEM. Proves the sandboxed daemon (the exact unit hardening) can read the meta CA. `systemd-run --user` WAS available — no fallback needed.
4. `bash ci/gates/no-c.sh && bash ci/gates/shape.sh && bash ci/gates/enable.sh` — all PASS.
   - `NO-C GATE PASS` (rustls=['0.23.40'] on ring=['0.17.14']; zero aws-lc/openssl/C-SQLite)
   - `SHAPE GATE PASS`
   - `ENABLE GATE PASS`

## Deviations
- Plan U3 enumerated 5 path-reference locations. I ALSO updated the 2 REMOVE-block doc comments
  (`[component.remove]` header comment + inline NOTE) that still named the old
  `/usr/local/share/ca-certificates/cognitum-ca.crt` as the "pinned CA left in place". Leaving them
  stale would contradict the relocation (the trust root they describe moved). These are comment-only,
  zero-behavior edits within the spirit of U3 (all path references → meta path). No logic changed.
- The `cmp -s` idempotence, `install -d -m755 "$(dirname "$DST")"` (auto-creates …/ca/), and
  absent-Seed `exit 0` no-op are all preserved unchanged in both worker heredocs.

## Handoff notes (for the guardian)
- This is config-only; seam.rs is provably untouched (empty diff) — invariant "engine is the single
  shared library, non-printing" is not engaged.
- Crux to re-verify: the systemd sandbox can read the meta CA. Already proven with `systemd-run
  --user -p ProtectHome=read-only -p ProtectSystem=strict cat <meta path>` → PEM printed, RC=0. The
  `ReadOnlyPaths=` (U2) is belt-and-suspenders; `ProtectHome=read-only` already makes `%h/...` readable.
- Trust-root safety: U2 is `ReadOnlyPaths`, NOT `ReadWritePaths` — the daemon must never overwrite
  its trust root. The worker (which writes the pin) runs as root via sudo outside the daemon sandbox,
  not inside secretd.
- Frozen-roots invariant intact: only the file LOCATION of the Cognitum CA changed; the daemon still
  pins ONLY that one CA explicitly (not the OS store).
- `%h` systemd specifier used in env-ctl.toml (matching the ExecStart idiom); the worker uses
  `${META_ROOT:-$HOME/Desktop/meta}` since it runs as a shell script, not under systemd specifier
  expansion. The two resolve to the same path on this box.

Status: GREEN
