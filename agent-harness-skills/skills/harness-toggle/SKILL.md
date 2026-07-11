---
name: harness-toggle
description: Toggle one optional envctl harness capability for the current chat thread.
---

# Harness Toggle

Toggle exactly one optional capability for this chat thread. Obtain `CAPABILITY` and `STATE` from the invocation or ask only if either is missing.

Accepted capabilities:

- `external_providers`
- `local_models`
- `network`
- `github_mutation`
- `browser_computer`
- `subagents`
- `background_jobs`

`STATE` must be `on` or `off`.

Run the repo-specific command from any current directory:
`cargo run --quiet --manifest-path /home/flexnetos/meta/src/envctl/home/agent-env/codex-harness/Cargo.toml --bin codex-harness-policy -- session set CAPABILITY STATE`

Report the effective optional routing result. Tell the operator to use `/permissions` for the actual Codex sandbox, approval, and network boundary. Never claim this skill changed or bypassed that boundary.

Hard safety is deliberately not toggleable: secret reads/output, destructive user-data deletion, force-push, and direct ledger/archive mutation.
