# Architect plan

VERDICT: GO.

1. Remove remote cache actions and add a Nushell policy gate.
2. Port automatically invoked gate/setup entrypoints to Nushell or keep affected jobs disabled.
3. Ban `actions/cache`, `Swatinem/rust-cache`, Magic Nix Cache, Cachix, `type=gha`, non-Kache Rust wrappers, and automatic Bash/sh/zsh.
4. Do not re-enable Actions until the cross-repository all-green barrier passes.
