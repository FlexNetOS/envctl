---
name: harness-full
description: Enable every optional envctl harness capability after Full Access is selected with the built-in permissions command.
---

# Harness Full

Enable all optional envctl harness capabilities for this chat thread without inventing permission authority.

1. Tell the operator that `/permissions` is the only authority for sandbox, approval, and network restrictions; select **Full Access** there when broad OS execution is intended.
2. Do not infer or block on `CODEX_PERMISSION_PROFILE`; current Codex child shells do not reliably export mutable `/permissions` state.
3. Run the repo-specific command from any current directory:
   `cargo run --quiet --manifest-path /home/flexnetos/meta/src/envctl/home/agent-env/codex-harness/Cargo.toml --bin codex-harness-policy -- session preset full`
4. Report the returned optional routing status without claiming the skill changed the Codex sandbox.
5. Hard safety remains non-toggleable: no secret reads/output, destructive user-data deletion, force-push, or direct ledger/archive mutation.
