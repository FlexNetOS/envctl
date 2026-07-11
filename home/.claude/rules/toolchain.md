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
  profile). Non-login `bash -c` also carries toolbin on this box and
  skips the profile re-source; prefer it for POSIX one-liners, and
  `nu -l -c` when structured output or nu plugins are the point.

**Why:** one toolchain owner per repo — mixed npm/npx/global installs
recreate the 7-owner mess that the foundation unification collapsed
to 1.

Shared learning layers wired under this contract (2026-07-07):
- AgentDB: `$HARNESS_VAR/lib/agentdb/` (runtime installed with bun,
  ruvector backend, `AGENTDB_*` env in settings.json).
- Ruvector intelligence bridge: `~/.claude/hooks/ruvector-intel-bridge.sh`
  (bun with node fallback).

## Nushell-primary shell doctrine (agent-env prompt, 2026-07-11)

- **Nushell is the primary shell** for humans and agents: yazelix sets
  `"shell": {"default_shell": "nu"}`; bash/zsh/nu all come from the same
  `lifeos-foundation-yzx` runtime (toolbin symlinks -> libexec -> real
  packages). Compatibility is by construction; nu does NOT parse bash syntax.
- **rtk routing**: `~/.config/nushell/rtk-wrappers.nu` (nu), aliases in
  `~/.config/yazelix/shell_bash.sh` (bash), `rtk hook claude` +
  `hooks/bash-to-nu.py` (Claude Bash tool). Escape hatches: `^git` (nu),
  `\git` (bash), `rtk proxy <cmd>`.
- **Bash-tool routing**: Claude Code's Bash tool is bash/zsh/sh-only; the
  PreToolUse hook `~/.claude/hooks/bash-to-nu.py` routes every Bash-tool
  command through nu supervision (`nu -l -c "^bash <scratch-file>"`), with
  rtk composed internally. Disable per-session with `BASH_NU_ROUTE=0`.
- **Symlink contract**: `~/.config/nushell/{config.nu,rtk-wrappers.nu,meta-usr-path.nu}`
  and `~/.config/yazelix/{shell_nu.nu,shell_bash.sh}` are symlinks into
  `meta/src/envctl/home/.config/` — never copies (a partial copy hard-broke
  every nu login on 2026-07-10).
- **Terminal chain**: kitty packaged default, host ghostty backup, mars removed.
