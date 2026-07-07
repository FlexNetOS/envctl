# Toolchain ownership (nix-profile only)

Operator directive 2026-07-07: every toolchain and runtime on this
workstation is owned by the nix profile (`lifeos-foundation-yzx`).

- **Rust:** cargo/rustc come from the fenix toolchain in the nix profile —
  never rustup-in-place, never a system cargo.
- **Node/JS:** run with **bun / bunx** from the nix toolbin; avoid bare
  `npm`/`npx` invocations where bun works. Project-local `node_modules`
  are fine, but must be produced by nix-owned tools.
- **Binaries:** everything executable lives in
  `/nix/store/*-lifeos-foundation-yzx/toolbin/`. No ad-hoc global
  installs (`npm -g`, `cargo install` to `~/.cargo`, curl-to-bash).
- **Shell note:** the nu tool shell does not carry toolbin on PATH —
  wrap node/cargo tooling in `bash -lc` (login shell resolves the
  profile).

**Why:** one toolchain owner per repo — mixed npm/npx/global installs
recreate the 7-owner mess that the foundation unification collapsed
to 1.

Shared learning layers wired under this contract (2026-07-07):
- AgentDB: `$HARNESS_VAR/lib/agentdb/` (runtime installed with bun,
  ruvector backend, `AGENTDB_*` env in settings.json).
- Ruvector intelligence bridge: `~/.claude/hooks/ruvector-intel-bridge.sh`
  (bun with node fallback).
