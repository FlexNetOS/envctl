---
name: agent-env-codex
description: Rebuild, edit, repair, or upgrade the complete envctl Codex agent harness under home/agent-env/codex-harness from the polished CODEX-GPT-HARNESS prompt, including prompts, Rust harness code, agents, teams, policies, rules, model routing, session capabilities, agent-env projections, tests, and active-skill materialization. Use when the user says /agent-env-codex, rebuild the Codex harness, edit the entire Codex agent environment, update the agent-env Codex harness, convert the harness prompt into the implementation, repair harness drift, or upgrade all Codex harness surfaces.
---

# Agent Env Codex

Use this single skill to rebuild or edit the complete Codex harness. Do not split this workflow into separate top-level init, sync, status, full, restricted, or toggle skills unless the owner explicitly requests separate products. Treat those as internal capabilities of this skill.

## Source of truth

Load `references/source-prompt.md` completely before planning or editing. It is the bundled, byte-identical snapshot of:

```text
/home/flexnetos/meta/src/envctl/.codex/prompts/prompt:codex-gpt-harness-v3-full-access-no-sandbox.prompt.md
```

Keep its canonical entrypoint byte-identical when either prompt is touched:

```text
/home/flexnetos/meta/src/envctl/.codex/prompts/prompt:codex-gpt-harness.prompt.md
```

The prompt is the specification. This skill is the compact execution controller. Its durable skill source and complete harness implementation owner are:

```text
/home/flexnetos/meta/src/envctl/agent-skills/agent-env-codex/
/home/flexnetos/meta/src/envctl/home/agent-env/codex-harness/
```

Project projections are `.codex/skills/agent-env-codex/` and `.claude/skills/agent-env-codex/`. The active materialization is `${CODEX_HOME:-/home/flexnetos/.codex}/skills/agent-env-codex/`. All must remain byte-identical to the durable source.

Read `references/coverage-map.md` to navigate every controller and Phase 0-11 contract, `references/ownership-map.md` before editing, `references/runbook-cli-contract.md` when changing initialization, routing, automation, shell behavior, or hardware-aware validation, and `references/github-execution-policy.md` before any branch, commit, worktree, PR, CI, merge, or cleanup operation. The coverage map never replaces the complete snapshot.

## Execute the rebuild or edit

1. **Re-anchor.** Read the active `AGENTS.md` chain and `/home/flexnetos/.codex/RULES.md`. Work from a clean current envctl worktree.
2. **Archive.** Archive every existing target before modifying or replacing it. Never treat git history alone as the requested archive.
3. **Load the whole specification.** Read `references/source-prompt.md` completely, then verify it against both repo prompt entrypoints. Read every file under `docs/runbook`, the relevant Yazelix docs, and the current `home/agent-env/codex-harness` tree. Build a compact source ledger.
4. **Map the requested change across the whole harness.** Check Rust code, binaries, agents, teams, policies, rules, model catalog, prompt review, tests, agent-env inputs, repo projections, and profile/runtime frontdoors. Do not patch only the first visible file.
5. **Edit owning source surfaces.** Durable harness implementation belongs in the envctl repo. Do not hand-edit generated Yazelix runtime under `/home/flexnetos/.local/share/yazelix` or treat active home projection as the durable owner.
6. **Route the skill and managed assets through agent-env.** Keep `agent-skills/agent-env-codex/` byte-identical to the active skill, edit `agent-env.yaml` and other owning inputs when needed, run `envctl agent lock` to refresh the lock, then run `envctl agent sync --json --color never` for preview. Use `--apply` only when the user requested materialization, then verify `agent-env.lock`.
7. **Preserve session toggles.** `/permissions` is the live Codex sandbox/approval/network authority. Harness capability states and model lanes are session-scoped, never hard-coded as permanent lockouts.
8. **Use the profile frontdoors.** Yazelix/Nix owns binaries and runtime delivery. Route fleet git through `rtk meta git`; use `rtk meta exec -- git <command>` for unlisted fleet git operations.
9. **Validate the real implementation.** Run `scripts/validate.sh <envctl-root>`. Add focused tests for changed behavior and run live CLI probes for any frontdoor or automation claim.
10. **Execute GitHub work to completion.** Follow `references/github-execution-policy.md`. Reconcile commits by inspecting and re-implementing their changes in the current branch; never cherry-pick. Treat every surfaced stale branch, orphaned worktree, unpushed commit, open superseded PR, failed check, or unmerged change as unfinished work that remains in scope until integrated or conclusively proven unrelated.
11. **Finish with proof.** Report archives, exact changed owners, tests, prompt hashes, skill validation, pushed commits, merged PR state, branch/worktree cleanup, open PR inventory, and final git status. A runtime receipt, local commit, pushed branch, or auto-merge request alone is not completion.

## Internal harness capabilities

Keep these capabilities inside this one skill and its references/scripts:

| Capability | Required behavior |
| --- | --- |
| Initialize | Non-mutating probes for Yazelix, Nushell, GitKB, Grit, ICM, Meta, RTK, Weave, and envctl. |
| Synchronize | Prompt-to-implementation and agent-env preview/apply with drift checks. |
| Status | Report live permission profile, model lane, capability state, frontdoors, and exact gaps. |
| Full/restricted/toggle | Change optional session routing only; never claim to change the Codex OS boundary. |
| Rebuild | Edit the complete `home/agent-env/codex-harness` owner surface and all affected projections/tests. |

## Mandatory constraints

- Never read, print, paste, or commit secrets.
- Never silently initialize GitKB, Grit, ICM, Meta, RTK, or Weave because a session started.
- Never create a weaker replacement prompt, split this one-skill product into competing top-level skills, or create a second harness owner.
- Never cherry-pick, force-push, discard, comment out, or remove an existing capability to make reconciliation easier. Preserve the stronger behavior and manually integrate every required delta into the current source.
- Never change `/permissions`, approval policy, sandbox mode, network policy, or an `Allow`/approval setting to bypass a failing test, policy, warning, or integration problem. Fix the owning source or report the exact external blocker.
- Never leave a task commit only local or only pushed. Commit all intended changes, push them, merge the PR after required checks pass, and remove merged task branches/worktrees only after archiving and proving their work is represented.
- Never restore GPT-5.5 planning routes or tracked model-cache authority. Use Sol for high-stakes work, Terra for professional workhorse tasks, and Luna for simple high-volume tasks.
- Never claim success without running the relevant tests and live frontdoor probes.
