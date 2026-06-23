# TASK-0019 — implementation log · STATUS: GREEN

## Result

The current source already contains the real `RealUsbProbe` path the task asks for:

- `RealUsbProbe` delegates to `seed_factor::keyfile_for` under the `seed-factor` feature.
- The seed backend uses bounded, direct, pure-Rust HTTPS with pinned CA/ring rustls and returns a
  deterministic, PARTUUID-bound 64-byte Ed25519 signature as keyfile IKM.
- The daemon enroll/init path reads the USB keyfile via `read_usb_keyfile`, which forwards to
  `RealUsbProbe`.
- The daemon engine construction injects `RealUsbProbe` as the live USB seam.
- The `env-ctl` component install/fix hooks build `envctl-secretd --features seed-factor`.

The referenced `_done/secretd-provisioning-runbook.md` predates the seed-factor implementation and
is stale for TASK-0019.

While running the task's required build, `rtk cargo build -p envctl-engine -p envctl` failed because
`~/.cargo/bin/cargo` was a dangling shim to a missing `rustup`. The `rustup` manifest component was
upgraded so detect/verify execute real tools, install/fix use meta-owned
`$META_ROOT/.toolchains/{cargo,rustup}`, and `$HOME/.cargo/bin` only carries compatibility symlinks
back to that meta-owned rustup.

## Verification

- `cargo test -p envctl-secrets-engine --features seed-factor seam::seed_factor::tests -- --nocapture`
- `cargo build -p envctl-secretd --features seed-factor`
- `cargo test -p envctl-secrets-engine --test vault usb_keyslot_unlock_via_fake_probe -- --nocapture`
- `cargo build -p envctl-engine -p envctl`
- `./target/debug/envctl lock --check`
- `./target/debug/envctl doctor`
- `bash ci/gates/p7.sh`
- `bash ci/gates/no-c.sh`
- `bash ci/gates/shape.sh`
- `bash ci/gates/enable.sh`
- `bash ci/gates/kdf-feature-off.sh`
- `bash ci/gates/agent-env.sh`
- `bash ci/gates/loop-state.sh`
- `bash ci/gates/harness-scripts.sh`

All checks passed.
