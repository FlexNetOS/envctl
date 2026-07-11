---
name: harness-status
description: Show the live Codex permission profile and effective envctl harness capabilities for this chat thread.
---

# Harness Status

Inspect the current chat session without changing it.

1. Report the active model and permission profile visible to the parent chat;
   treat the binary's configured model and permission-profile environment value
   as signals, not as authoritative live state.
2. Run the repo-specific command from any current directory:
   `cargo run --quiet --manifest-path /home/flexnetos/meta/src/envctl/home/agent-env/codex-harness/Cargo.toml --bin codex-harness-policy -- session status`
3. Report every optional routing capability and any thread override.
4. Do not claim these switches change or mirror the Codex sandbox.
5. Remind the operator that `/permissions` is the authoritative in-session sandbox/approval selector, `/model` is the model selector, and `/agent` manages native subagent threads.
