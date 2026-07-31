# Yazelix-owned secrets runtime

Yazelix is the sole runtime owner. `yzx enter` and `yzx launch` run the packaged
`yazelix-stack-bootstrap` before the managed workspace opens. The bootstrap starts
the complete stack from pinned profile binaries, waits for dependencies in order,
opens the vault automatically when the Cognitum USB is present, and fails closed if
the required factor or service identity is wrong.

Durable service state is rooted at
`/home/flexnetos/meta/var/lib/yazelix/runtime`; durable databases remain beneath
`/home/flexnetos/meta/var/lib` or `/home/flexnetos/meta/var/xdg-data` as declared by
the Yazelix flake. No host or user service manager owns these processes, and no
service state belongs under a volatile login-session directory.

The executable contract is in Yazelix:

- `runtime/yzx_stack_bootstrap.rs` owns startup order, readiness, USB unlock,
  reconnect monitoring, process identity, and logs.
- `flake.nix` supplies every absolute path and pinned binary.
- `nushell/system/profile_environment_frontdoor.nu` exports the owned environment.

Envctl owns vault behavior and storage formats. It deliberately has no service-unit
schema or service-manager bridge. Legacy `systemd_user` manifest input is rejected.
