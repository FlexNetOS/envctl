# harness-ops

Use when operating the envctl Codex harness under `/home/flexnetos/lifeos/src/envctl/home/agent-env`.

Commands:

- Status: `agent-env/codex-harness/bin/codex-harness-status`
- Nix verification: `agent-env/codex-harness/bin/codex-harness-nix-verify`
- Audit: `agent-env/codex-harness/bin/codex-harness-audit`
- Policy check: `agent-env/codex-harness/bin/codex-harness-runner policy-check -- <command...>`
- Foreground supervised run: `agent-env/codex-harness/bin/codex-harness-runner run --cwd <dir> -- <command...>`
- Background supervised spawn: `agent-env/codex-harness/bin/codex-harness-runner spawn --cwd <dir> -- <command...>`
- Halt harness-owned jobs: `agent-env/codex-harness/bin/codex-harness-halt`

Operational rules:

- Archive existing files before modification.
- Never read secrets.
- Use localhost-only local model lanes.
- Never pull local models without approval.
- Do not install plugins or MCP mutation tools without approval.
- If hooks misfire, restore from `agent-env/archive` and disable only through an approved archive-first change.
