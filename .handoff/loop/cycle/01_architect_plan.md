# TASK-0076 architect plan — cognitum-seed-autounlock component

VERDICT: GO. New manifest/cognitum-seed-autounlock.toml (sibling to seed-trust/seed-net).
Three defects in the qwen candidate, all fixed:
1. invented `run =` schema → use kind="command" command/args + kind="script" script='''...'''.
2. root-oneshot can't pass SO_PEERCRED against the USER secretd (peercred.rs:50 gates
   cred.uid()==owner_uid; socket $XDG_RUNTIME_DIR/env-ctl/secretd.sock). FIX: worker drops to
   owner via `setpriv --reuid OWNER_UID --regid OWNER_UID --init-groups env XDG_RUNTIME_DIR=
   /run/user/UID SECRETCTL_SOCK=... secretctl unlock`. No-op if /run/user/UID/.../secretd.sock
   absent (no linger / pre-login). Verify probe runs `secretctl status` via same wrapper.
3. flat `id =` → MUST be `[[component]]` array-of-tables (else contributes nothing).
Artifacts: /usr/local/sbin/cognitum-seed-autounlock, /etc/systemd/system/...service,
/etc/udev/rules.d/99-...rules (sibling /usr/local + /etc convention; udev irreducibly system).
Unit: Type=oneshot, After=systemd-udevd.service, ExecStart, WantedBy=multi-user.target (drop
network-online + Wants=secretd — user unit invisible to system oneshot). worker prefers meta
secretctl. Lock regen: `cargo run -p envctl -- lock`. Gates: no-c/shape/enable/p7/kdf/agent-env/
loop-state/harness-scripts + lock --check. Runtime-verifiable NOW: manifest loads + component
discovered, lock round-trips, dry-run preview, detect=false clean, worker `bash -n`. Deferred:
live Seed-present reboot unlock (hardware-in-the-loop, owner-only vault).
