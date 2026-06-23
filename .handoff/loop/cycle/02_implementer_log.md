# TASK-0076 implementer log
- Wrote manifest/cognitum-seed-autounlock.toml ([[component]] header; detect/install/verify/fix/remove).
- Schema-correct (kind=command command/args; kind=script needs_sudo script='''...''').
- Defect-2 fix: worker drops root→owner via `setpriv --reuid OWNER_UID --regid OWNER_UID
  --init-groups env XDG_RUNTIME_DIR=/run/user/UID SECRETCTL_SOCK=... secretctl unlock`.
- Runtime-verify FOUND A BUG: line-9 `HOME: unbound variable` (set -u; $HOME unset for systemd
  system service) → oneshot exited 1, breaking fail-closed exit-0. FIXED: guarded `${HOME:-}` in
  both worker copies + verify probe. Lock regenerated (78→79).
- Build OK; bash -n OK; 8 CI gates PASS; lock --check rc=0.
