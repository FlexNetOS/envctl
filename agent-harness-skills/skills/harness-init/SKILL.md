---
name: harness-init
description: Initialize an envctl Codex chat session with read-only profile, GitKB, Grit, ICM, Meta, and RTK context probes.
---

# Harness Init

Initialize session context without silently initializing or mutating tool state.

1. Prove the Yazelix/Nix profile frontdoors with raw `readlink -f`, checking
   `/home/flexnetos/.nix-profile/bin/<tool>` and then
   `/home/flexnetos/.nix-profile/toolbin/<tool>` for `yzx`, `rtk`, `git-kb`,
   `grit`, `icm`, `meta`, `git`, `codex`, `claude`, and `nu`.
2. Read the target repository's `AGENTS.md` and `.kb/AGENTS.md` when present.
3. Run the existing GitKB context probe through RTK:
   `/home/flexnetos/.nix-profile/bin/rtk git-kb list --path context/ --json`.
   GitKB MCP is primary when registered; `git-kb` is the profile-owned fallback.
4. If `.grit/` exists, run
   `/home/flexnetos/.nix-profile/bin/rtk grit status`; otherwise report Grit as
   inactive for this read-only bootstrap.
5. Run
   `/home/flexnetos/.nix-profile/bin/rtk icm --read-only wake-up --max-tokens 200`
   to load session context without changing ICM integration files.
   A missing existing ICM database means context is unavailable, not that the
   harness should run `icm init` or block the rest of the session.
6. From `/home/flexnetos/meta`, run
   `/home/flexnetos/.nix-profile/bin/rtk meta git status`.
7. Run `/home/flexnetos/.nix-profile/bin/rtk init --show` to prove the current
   RTK instruction integration.
8. Report exact failures and continue with available read-only context. Do not
   convert a missing optional context source into a sandbox or permission block.

Never run `git-kb init`, `grit init`, `icm init`, `meta init`, or a mutating
`rtk init` merely because a session started. Those commands create or replace
state and require an explicit writable task, archive-first handling for existing
targets, and the live `/permissions` profile selected by the operator.
Never run mutating `rtk init` automatically.

Git routing:

- Meta plugin-owned Git commands: `rtk meta git <command>`.
- Unlisted fleet Git commands: `rtk meta exec -- git <command>`.
- One-repository fleet intent: add `--include <repo>`.
- Single-checkout Git: `rtk git <command>`.
- Raw gate/root-cause evidence: follow `AGENTS.rtk.md` and preserve raw output.

`$harness-full`, `$harness-restricted`, and `$harness-toggle` control optional
routing only. They never initialize GitKB, Grit, ICM, Meta, or RTK state.
