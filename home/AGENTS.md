# envctl home Codex harness

Harness root: `/home/flexnetos/lifeos/src/envctl/home/agent-env`.
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

The root envctl `AGENTS.md` and runbook still apply. This file is a lean pointer and invariant layer only.
