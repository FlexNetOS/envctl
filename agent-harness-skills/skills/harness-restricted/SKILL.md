---
name: harness-restricted
description: Disable every optional envctl harness capability for the current chat thread.
---

# Harness Restricted

Restrict the envctl harness immediately for this chat thread.

1. Run the repo-specific command from any current directory:
   `cargo run --quiet --manifest-path /home/flexnetos/meta/src/envctl/home/agent-env/codex-harness/Cargo.toml --bin codex-harness-policy -- session preset restricted`
2. Report the returned effective status.
3. Tell the operator to use `/permissions` to select Read Only, Ask for approval, Approve for me, or a named harness profile when an OS-enforced sandbox/approval change is also desired.
4. Do not claim that a skill or prompt changed the Codex sandbox.
