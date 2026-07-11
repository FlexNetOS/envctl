---
name: harness-session
description: Continue or repair the envctl Codex harness session from the former CODEX-GPT-HARNESS prompt as a reusable skill. Use when the user says session prompt, Codex harness prompt, full-access/no-sandbox harness, v3 autonomous recovery, or asks to turn/update the prompt into a skill.
---

# Harness Session

Use this skill as the compact execution controller for the old
`CODEX-GPT-HARNESS` prompt. The long prompt files remain provenance and
compatibility shims:

- `.codex/prompts/prompt:codex-gpt-harness.prompt.md`
- `.codex/prompts/prompt:codex-gpt-harness-v3-full-access-no-sandbox.prompt.md`
- `.codex/prompts/prompt:codex-gpt-harness-v3-autonomous.prompt.md`

Do not keep growing those prompt files as the primary control plane. Put durable
session procedure here or in focused harness skills/rules, then leave a short
prompt pointer when prompt compatibility still matters.

## Start or resume

1. Read the active `AGENTS.md` chain and `/home/flexnetos/.codex/RULES.md`.
2. Work from a clean envctl worktree. If a PR branch already exists for the
   harness work, continue from that branch instead of editing a dirty main
   checkout.
3. Run `$harness-init` for read-only context: Yazelix/Nix profile frontdoors,
   GitKB, Grit-if-present, ICM, Meta, RTK, and target repo instructions.
4. Run `$harness-status` when the live capability state matters. Use
   `$harness-full`, `$harness-restricted`, or `$harness-toggle` only for
   optional harness routing.
5. Treat `/permissions` as the only live Codex sandbox, approval, and network
   authority. Never claim a skill, prompt, env var, or ledger changed the Codex
   OS boundary.

## Non-negotiable harness rules

- Yazelix/Nix profile-owned frontdoors are authoritative for `yzx`, `rtk`,
  `codex`, `claude`, `git-kb`, `grit`, `icm`, `meta`, `git`, `cargo`, and `nu`.
  Do not preserve user-bin, repo-cache, generated-runtime, or temp-bundle
  shadows as active control paths.
- Archive before changing existing files or state. Do not hand-edit generated
  Yazelix runtime under `/home/flexnetos/.local/share/yazelix`.
- Never read, print, paste, or commit secrets. Secret metadata and redacted
  proof are allowed; secret values stay in vault/runtime owners.
- Do not run `git-kb init`, `grit init`, `icm init`, `meta init`, or mutating
  `rtk init` because a session started. Those are explicit writable tasks.
- Sol/Terra/Luna are the routeable Codex model lanes for this harness family.
  Do not restore GPT-5.5 planning-agent routes or tracked model cache authority.
- External Claude, OpenRouter, local models, browser/computer-use, GitHub
  mutation, subagents, and background jobs require the matching harness
  capability and current proof. Route external Claude through the supervised
  tool-free harness bridge; do not represent it as a filesystem implementer.
- GitHub mutation uses the harness GitHub guard and final branch/PR hygiene
  proof. No force-push unless the operator explicitly asks for it.

## Work pattern

```text
anchor last session/PR
  -> verify branch, checks, worktree cleanliness, and runtime owner paths
  -> run read-only bootstrap/status
  -> make the smallest archive-first source change
  -> validate with focused tests and prompt review
  -> commit/push or report exact blocker
  -> prove clean status, touched PR state, and open PR inventory
```

When continuing from a prior Codex session, search local session JSONL by commit
SHA, PR number, prompt filename, or harness keyword. Report the session id,
rollout path, final checkpoint, and any open blockers before editing.

## Validation commands

From the envctl worktree, prefer focused checks before broad gates:

```bash
export CODEX_HARNESS_ROOT="$PWD/home/agent-env/codex-harness"
export CODEX_HARNESS_PROJECT_ROOT="$PWD/home"
unset CODEX_HARNESS_FULL_ACCESS

cargo test --manifest-path home/agent-env/codex-harness/Cargo.toml \
  --test full_access_contract
cargo run --quiet --manifest-path home/agent-env/codex-harness/Cargo.toml \
  --bin codex-harness-prompt-review -- \
  .codex/prompts/prompt:codex-gpt-harness.prompt.md
git diff --check
```

Before final reporting after GitHub/PR work, also run:

```bash
git status --short --branch
gh pr view <touched-pr> --json state,mergeStateStatus,statusCheckRollup,autoMergeRequest
gh pr list --state open
```

If a command is blocked, report the exact command and error. Do not turn blocked
or ignored runtime receipts into success.
