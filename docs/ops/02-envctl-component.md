# Envctl runtime boundary

Envctl installs and reconciles artifacts but does not supervise the FlexNetOS
runtime. Yazelix is the sole owner of long-running processes. Its pinned stack
bootstrap receives absolute paths from `flake.nix`, starts dependencies in order,
opens the USB-bound vault, verifies readiness and process identity, and only then
starts agents and the runner.

The Envctl `Wiring` schema has no service-unit field. Unknown input is rejected so a
manifest cannot recreate the retired control plane. See
`docs/secrets/ops/01-yazelix-runtime.md` for the operational contract.
