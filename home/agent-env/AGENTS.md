# envctl home Codex harness

Harness root: `/home/flexnetos/meta/src/envctl/home/agent-env`.
Durable Rust harness source: `agent-env/codex-harness`.

Non-negotiable rules for this subtree:

- Never delete user data. Archive first under `agent-env/archive/<UTC>/`.
- Codex must resolve through the Nix profile and `/nix/store`.
- Use `codex-harness-runner` for background, parallel, child Codex, Claude, Ollama, or LM Studio work.
- Do not start write-capable parallel work. One foreground writer only.
- Do not use Codex subagents until containment tests pass.
- Do not read `auth.json`, private keys, `.env` values, tokens, or credential helper output.
- Do not hand-edit generated Yazelix runtime or envctl agent-env generated surfaces.
- Preserve symlinks and archive before replacing any existing path.
- Rust harness work must pass `rustfmt --check`, clippy-equivalent check, and tests.
- No unsafe Rust unless explicitly justified and reviewed.
- Terminal output and JSON ledgers are required proof. Prose alone is not acceptance.

New-session navigation:

- Start from a fresh envctl worktree at latest `origin/master`/`origin/develop`.
- Update agent config through envctl `agent-env.yaml`, `agent-env.lock`, and
  `agent-skills/`; preview with `envctl agent sync --json --color never` and
  only use `--apply` after review.
- Keep `/home/flexnetos/.codex/config.toml` as the active runtime config.
- Do not use retired mirrors `/home/flexnetos/lifeos/.codex` or
  `/home/flexnetos/FlexNetOS/.codex`.
- Toolchains resolve through the Nix/Yazelix foundation: nightly cargo/rustc
  from the profile, kache as the Rust cache wrapper, wild via clang as linker,
  and bun/bunx for Node.js package execution.

The root envctl `AGENTS.md` and runbook still apply. This file is a lean pointer and invariant layer only.
