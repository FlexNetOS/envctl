# TASK-0019 — RealUsbProbe verification · VERDICT: GO

## Trigger Check

TASK-0019 says the USB-unlock path still needs a real `RealUsbProbe`, pointing at the old
`secretd-provisioning-runbook.md` note. That premise is stale at HEAD.

Source truth:

- `crates/secrets-engine/src/seam.rs` implements `RealUsbProbe::keyfile_for`.
- Default builds return `None` fail-closed.
- With `--features seed-factor`, `RealUsbProbe` resolves a Cognitum Seed-backed, PARTUUID-bound
  Ed25519 signature via the pure-Rust HTTPS `seed_factor` backend and returns the 64 bytes as the
  USB keyfile material.
- `crates/secretd/src/grpc.rs` forwards USB enrollment through that same seam.
- `crates/secretd/src/main.rs` injects `RealUsbProbe` into the daemon engine seams.
- `manifest/env-ctl.toml` builds and rebuilds `envctl-secretd` with `--features seed-factor`, so the
  installed daemon is USB-unlock-capable while stock Cargo builds remain fail-closed.

## Design

No Rust implementation is required. Close this cycle by proving the existing implementation and
marking the stale task done with evidence.

## Target Repos

Single repo: envctl. Sequential single-crew path.

Touched surfaces:

- `.handoff/loop/backlog.md`
- `.handoff/loop/loop_state.md`
- `.handoff/loop/cycle/*`

## Non-Goals

- Do not enable `seed-factor` as a default Cargo feature; the manifest install path owns production
  enablement and the default fail-closed build is intentional.
- Do not run a live Seed/network probe; TASK-0019 has `allows_network=false`.
- Do not mix unrelated manifest/toolchain edits into this closeout.
