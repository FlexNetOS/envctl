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

Project projections are `.codex/skills/agent-env-codex/` and `.claude/skills/agent-env-codex/`. The active materialization is `${CODEX_HOME:-/home/flexnetos/meta/var/lib/codex}/skills/agent-env-codex/`. All must remain byte-identical to the durable source.

Read `references/coverage-map.md` to navigate every controller and Phase 0-11 contract, `references/ownership-map.md` before editing, `references/runbook-cli-contract.md` when changing initialization, routing, automation, shell behavior, or hardware-aware validation, `references/yazelix-cli-plugin-policy.md` before any Yazelix/toolchain/plugin/add-on work, and `references/github-execution-policy.md` before any branch, commit, worktree, PR, CI, merge, or cleanup operation. Read `references/github-org-and-ccboard.md` before GitHub organization administration or ccboard/Codex/Claude integration. Read `references/bunx-and-github-ssh.md` before executing a JavaScript package or claiming personal/organization GitHub SSH access. For shell behavior, treat `~yazelix/nushell/config` and `~yazelix/nushell/scripts` as the configured owner surfaces; use Nu scripts when possible; Bash is already configured inside that Nushell/Yazelix runtime, so do not add separate bash wrappers, shell launchers, or parallel shell control paths. The coverage map never replaces the complete snapshot.

## Execute the rebuild or edit

1. **Re-anchor.** Read the active `AGENTS.md` chain and `/home/flexnetos/meta/var/lib/codex/RULES.md`. Fetch through RTK/Meta, verify the SSH remote, and work from a clean current Meta-managed envctl worktree based on current `origin/develop` or the repository's protected trunk.
2. **Archive.** Archive every existing target before modifying or replacing it. Never treat git history alone as the requested archive.
3. **Load the whole specification.** Read `references/source-prompt.md` completely, then verify it against both repo prompt entrypoints. Read every file under `docs/runbook`, the relevant Yazelix docs, and the current `home/agent-env/codex-harness` tree. Build a compact source ledger.
4. **Map the requested change across the whole harness.** Check Rust code, binaries, agents, teams, policies, rules, model catalog, prompt review, tests, agent-env inputs, repo projections, and profile/runtime frontdoors. Do not patch only the first visible file.
5. **Edit owning source surfaces.** Durable harness implementation belongs in the envctl repo. Do not hand-edit generated Yazelix runtime under `/home/flexnetos/var/lib/yazelix` or treat active home projection as the durable owner.
6. **Route the skill and managed assets through agent-env.** Keep `agent-skills/agent-env-codex/` byte-identical to the active skill, edit `agent-env.yaml` and other owning inputs when needed, run `envctl agent lock` to refresh the lock, then run `envctl agent sync --json --color never` for preview. Use `--apply` only when the user requested materialization, then verify `agent-env.lock`.
7. **Preserve session toggles without making work optional.** `/permissions` is the live Codex sandbox/approval/network authority. Harness capability states and model lanes are session-scoped, never hard-coded as permanent lockouts. A toggle may be off; its task, capability, integration, and verification remain mandatory.
8. **Use the latest profile frontdoors.** Yazelix/Nix owns binaries and runtime delivery. Resolve current versions from profile commands at execution time, use the newest available owner-provided toolchain as the primary lane, and keep older floors only as added compatibility tests. Use `rtk meta git <adapted-command>` for fleet Git and `rtk meta exec --include <repo> -- git <unlisted-command>` only when Meta has no adapted command. Use `bun` instead of `npm` and `bunx` instead of `npx` in every executable skill recipe. Never invoke raw `git`, never cherry-pick, and never bypass Meta worktree ownership.
9. **Execute GitHub work to completion.** Follow `references/github-execution-policy.md`. Preserve all capabilities, reconcile every surfaced stale/orphaned commit or worktree, use Linux-only workflows, commit and push every intended change, open/update a PR, enable auto-merge, wait for merge, and remove merged task branches/worktrees. Protect only `main`, `master`, and `develop` from lifecycle cleanup.
10. **Administer GitHub deliberately.** When the task touches FlexNetOS organization governance, inventory every surface in `references/github-org-and-ccboard.md`, compare it with declared policy, apply only requested drift through `gh`/REST/GraphQL, and verify without reading secret values. Prove `drdave-flexnetos` as the personal GitHub SSH identity and separately prove active FlexNetOS organization authorization plus SSH repository access. SSH is mandatory for Git transport; organization settings are API/web administration and must not be misrepresented as SSH operations.
11. **Trace ccboard before wiring.** Preserve ccboard's existing Claude and Codex ingestion. Extend the ccboard source owner, not generated Yazelix runtime. Follow the Claude hook/data endpoints and the Codex parser/store/watcher/live-session path in `references/github-org-and-ccboard.md`, then prove startup indexing, incremental updates, TUI/API visibility, and installed profile delivery.
12. **Converge Yazelix after Yazelix changes.** Follow `references/yazelix-cli-plugin-policy.md`: select one install owner, run its current `yzx update` route, prove generated-state convergence, and verify every plugin/add-on class is packaged, materialized, permissioned, and connected. Treat `yazelix-yazi-assets` as the consolidation owner and every competing plugin source as migration work that must preserve behavior until represented.
13. **Validate and finish with proof.** Run `scripts/validate.sh <envctl-root>`, `scripts/check-yazelix-contract.py --root <envctl-root> --live` for Yazelix-related work, focused tests, complete relevant gates, and live frontdoor probes. Report archives, changed owners, tests, prompt hashes, skill identity, merged PR state, protected-trunk sync, and task branch/worktree cleanup. A runtime receipt, auto-merge request, or green-but-unmerged PR is not completion.

## Internal harness capabilities

Keep these capabilities inside this one skill and its references/scripts:

| Capability | Required behavior |
| --- | --- |
| Initialize | Non-mutating probes for Yazelix, Nushell, GitKB, Grit, ICM, Meta, RTK, Weave, and envctl. |
| Synchronize | Prompt-to-implementation and agent-env preview/apply with drift checks. |
| Status | Report live permission profile, model lane, capability state, frontdoors, and exact gaps. |
| Full/restricted/toggle | Change session routing state only; never make the underlying capability/task optional or claim to change the Codex OS boundary. |
| Rebuild | Edit the complete `home/agent-env/codex-harness` owner surface and all affected projections/tests. |

## Mandatory constraints

- Never read, print, paste, or commit secrets.
- Never classify a task, requirement, integration, validation, or surfaced unfinished item as optional. Only live session state is toggleable; off does not mean removed or complete.
- Never block or downgrade a capability or requirement to avoid repairing it. Record the exact affected gap, continue every unblocked requirement, and finish the owner-level repair.
- Never silently initialize GitKB, Grit, ICM, Meta, RTK, or Weave because a session started.
- Never create a weaker replacement prompt, split this one-skill product into competing top-level skills, or create a second harness owner.
- Never cherry-pick, force-push, discard, comment out, or remove an existing capability to make reconciliation easier. Preserve the stronger behavior and manually integrate every required delta into the current source.
- Never change `/permissions`, approval policy, sandbox mode, network policy, or an `Allow`/approval setting to bypass a failing test, policy, warning, or integration problem. Fix the owning source or report the exact external blocker.
- Never leave a task commit only local or only pushed. Commit all intended changes, push them, merge the PR after required checks pass, and remove merged task branches/worktrees only after archiving and proving their work is represented.
- Never invoke raw `git` or bypass the Meta worktree policy. Use RTK/Meta Git routing for every repository operation.
- Never execute `npm` or `npx` from a skill. Use the profile-owned `bun`/`bunx` frontdoors; for example, `bunx ruv-swarm/claude-flow@alpha`.
- Never target an older toolchain because it is easier. Use the latest available Nix/Yazelix/fenix/Bun-owned binaries as the primary lane and run MSRV or pinned versions only as additional compatibility gates.
- Never call a Yazelix change complete before running the correct owner-specific `yzx update` route and proving `status`, `inspect`, `doctor`, generated-state convergence, and plugin connectivity.
- Never establish a second durable plugin/add-on owner. Consolidate Yazelix plugin and add-on authority in `/home/flexnetos/meta/src/yazelix-yazi-assets`, preserving every existing behavior until migration proof permits owner cleanup.
- Never leave completed or idle subagents running. Capture the deliverable, close the worker, and finish with an empty harness-owned roster; respawn a fresh bounded worker when later work needs one.
- Never claim FlexNetOS organization SSH access from a personal greeting alone. Prove the `drdave-flexnetos` SSH identity, active organization membership, and an SSH read against a FlexNetOS repository without reading private keys.
- Never add macOS or Windows GitHub Actions jobs. Use Ubuntu/Linux hosted or FlexNetOS Linux self-hosted runners.
- Never remove a working capability. Required post-merge branch/worktree cleanup is lifecycle cleanup, not capability removal.
- Never restore GPT-5.5 planning routes or tracked model-cache authority. Use Sol for high-stakes work, Terra for professional workhorse tasks, and Luna for simple high-volume tasks.
- Never claim success without running the relevant tests and live frontdoor probes.
