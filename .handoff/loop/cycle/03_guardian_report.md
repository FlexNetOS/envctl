# TASK-0019 — guardian report

## Verdict: PASS

TASK-0019 is satisfied at HEAD. The earlier runbook claim that `RealUsbProbe` was unimplemented is
stale.

## Findings

- PASS: Default `RealUsbProbe` returns `None` when `seed-factor` is not compiled, preserving the
  fail-closed stock build.
- PASS: `seed-factor` compiles a real Cognitum Seed backend for `RealUsbProbe::keyfile_for`, deriving
  keyfile material from a PARTUUID-bound Ed25519 signature over bounded, pure-Rust HTTPS.
- PASS: `secretd` forwards USB enrollment through `RealUsbProbe` and injects that probe into the live
  daemon engine seams.
- PASS: `manifest/env-ctl.toml` builds/rebuilds `envctl-secretd --features seed-factor`, so the
  installed daemon has the real USB factor compiled in.
- PASS: The runner/toolchain break found during verification is fixed at the envctl manifest layer:
  rustup now detects executable cargo/rustc/rustup under meta-owned `$META_ROOT/.toolchains`, and
  home cargo shims point back into that owned toolchain.
- PASS: No new dependency or trust-boundary drift was introduced by this closeout.

## Gate Results

All local gates passed:

- seed-factor unit tests, seed-factor daemon build, fake-probe USB keyslot unlock test
- engine+CLI build, envctl lock check, envctl doctor
- p7, no-c, shape, enable, kdf-feature-off, agent-env, loop-state, harness-scripts

No live Seed/network probe was run because TASK-0019 has `allows_network=false`.
