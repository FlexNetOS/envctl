# harness-ops

Use when operating the envctl Codex harness under `/home/flexnetos/meta/src/envctl/home/agent-env`.

Commands:

Use the canonical source manifest from any current directory:

```bash
HARNESS_MANIFEST=/home/flexnetos/meta/src/envctl/home/agent-env/codex-harness/Cargo.toml
```

- Status: `cargo run --quiet --manifest-path "$HARNESS_MANIFEST" --bin codex-harness-status`
- Nix verification: `cargo run --quiet --manifest-path "$HARNESS_MANIFEST" --bin codex-harness-nix-verify`
- Audit: `cargo run --quiet --manifest-path "$HARNESS_MANIFEST" --bin codex-harness-audit`
- Policy check: `cargo run --quiet --manifest-path "$HARNESS_MANIFEST" --bin codex-harness-runner -- policy-check -- <command...>`
- Foreground supervised run: `cargo run --quiet --manifest-path "$HARNESS_MANIFEST" --bin codex-harness-runner -- run --cwd <dir> -- <command...>`
- Background supervised spawn: `cargo run --quiet --manifest-path "$HARNESS_MANIFEST" --bin codex-harness-runner -- spawn --cwd <dir> -- <command...>`
- Halt harness-owned jobs: `cargo run --quiet --manifest-path "$HARNESS_MANIFEST" --bin codex-harness-halt`

Do not invoke stale copied binaries under `codex-harness/bin/`; source and the
Nix-profile-owned Cargo toolchain are the operational owner until the harness
binaries are packaged into the profile.

Operational rules:

- Archive existing files before modification.
- Never read secrets.
- Use localhost-only local model lanes.
- Never pull local models without approval.
- Do not install plugins or MCP mutation tools without approval.
- If hooks misfire, restore from `agent-env/archive` and disable only through an approved archive-first change.
