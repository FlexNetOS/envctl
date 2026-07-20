---
description: Orchestrate strict-upgrade-only meta/envctl compliance recovery with background Opus agents.
argument-hint: "[GOAL_OR_RECOVERY_REQUEST]"
---

You are executing the envctl `/goal` recovery orchestrator.

Authoritative repo rules:
- Read `AGENTS.md`, `.codex/AGENTS.md`, and `/home/flexnetos/FlexNetOS/.kb/AGENTS.md` before changing code.
- Use repo-local skills as applicable: `.agents/skills/feature-forge/SKILL.md`, `.agents/skills/icm-memory/SKILL.md`, `.agents/skills/agent-env-config/SKILL.md`, `.agents/skills/cross-repo-health/SKILL.md`.
- Use `rtk`-prefixed shell commands.
- Use ICM recall before work and ICM store immediately for durable preferences, decisions, resolved errors, and significant completions.

Arguments supplied to this prompt: $ARGUMENTS

## Mission

Recover and harden the `meta` workspace by making `envctl` compliant with:
1. current `meta` CLI commands and plugin behavior,
2. `/home/flexnetos/FlexNetOS/.kb/AGENTS.md`,
3. envctl `AGENTS.md` invariants,
4. meta-local toolchain/dependency install policy,
5. GitHub workflows, runner assumptions, GitHub App/token surfaces, policy files, and loop workflows.

This is a recovery and compliance goal, not a destructive reset. The output must be verified code/config/docs, committed, pushed, PR-backed, and auto-merge armed for every coherent chunk.

## Non-negotiable owner rules

- ALWAYS STRICT UPGRADE ONLY. No downgrades.
- Do not remove a legacy working tool until the Rust/meta-native replacement is installed, configured, verified, and parity-proven.
- Do not repair by destructive reset, by weakening gates, or by moving dependencies into unmanaged system/user-global drift.
- Toolchains/dependencies for the meta environment must be owned by `envctl` manifests/engine policy or the meta workspace, not ad-hoc global shell state.
- Every committed chunk must immediately be pushed, opened as a PR, and auto-merge armed with squash. Never stop with committed-but-unpushed work.
- Start every envctl session with the local reaper preview and apply:
  - `rtk bash scripts/reap-worktrees.sh`
  - `rtk bash scripts/reap-worktrees.sh --apply`
- Treat all Claude/agent claims as untrusted until verified from source truth.
- For handoff/ledger/p7 claims, the required source truth is `meta/handoff` code and ADRs first; do not trust envctl harness prose or subagent summaries. Current verified contract: committed `.handoff/ledger.events.jsonl` plus rendered text; `.handoff/ledger.db`/RVF are gitignored per-worktree rebuild caches unless `meta/handoff` changes the contract.

## Preflight

1. Run targeted ICM recall for the supplied goal, including `envctl`, `meta`, `strict upgrade only`, `GitHub workflows`, `runner`, `GitHub App`, `policies`, and `loop workflows`.
2. Run the reaper preview and apply commands above.
3. Verify base state with `rtk git fetch --all --prune` and `rtk git status --short --branch`.
4. Create or move into a fresh isolated worktree/branch before mutation; do not edit a dirty main checkout.
5. If the work touches meta KB-governed work, obey `/home/flexnetos/FlexNetOS/.kb/AGENTS.md`:
   - detect KB state with `rtk git kb list --path context/`,
   - for bug/feature/fix requests, create the appropriate KB incident/task before code changes,
   - load required context; current CLI may use `rtk git kb checkout context/` rather than the older documented `--path` checkout form,
   - read the required context docs and active board before proceeding.

## Background Opus agent fan-out

Spawn the following background agents with maximum effort. In Claude Code, use `model: "opus"` (or explicit `claude-opus-4-8` where the runtime supports full model IDs), `run_in_background: true`, and isolated/read-only worktrees unless a subtask is explicitly assigned a disjoint write set. In Codex runtimes that inherit the current model, still name the intended Opus/max-effort role in the task prompt and keep roles read-only unless assigned a bounded patch.

1. `META-CLI-RESEARCHER` — read-only
   - Research `meta --help` and every `meta <command> --help`.
   - Inspect `.meta.yaml`, `.meta/`, plugin registry/config, and command behavior relevant to envctl.
   - Return a command matrix: command, flags, intended use, envctl compliance requirement, verification command, and no-downgrade risks.

2. `KB-CONTEXT-RESEARCHER` — read-only
   - Read `/home/flexnetos/FlexNetOS/.kb/AGENTS.md`.
   - Verify current `git kb` command syntax from help/output.
   - Return the exact KB compliance checklist envctl prompts/agents must follow.

3. `ENVCTL-COMPLIANCE-AUDITOR` — read-only first
   - Audit envctl manifests, agent-env, dashboard, forge-loop/session-relay, auto-provision, CI gates, and docs.
   - Identify installs/configuration that are global/user-local/system-depth rather than meta/envctl-owned.
   - Return upgrade-only repair tasks and verification commands.

4. `GITHUB-WORKFLOW-AUDITOR` — read-only first
   - Audit `.github/workflows`, CI gates, runner assumptions, GitHub App/token surfaces, auto-merge flow, and policy files.
   - Return broken/weak surfaces, upgrade-only repairs, and verification commands.

5. `LOOP-HARNESS-AUDITOR` — read-only first
   - Audit `.handoff/loop`, forge-loop, session-relay, wrap-up/resume, reaper wiring, ICM hooks, and batch-boundary behavior.
   - Confirm no loop workflow can silently skip wrap-up, PR creation, worktree cleanup, backlog reconciliation, or downgrade protection.
   - Return repair tasks and verification commands.

## Synthesis and implementation

1. Merge agent findings into one backlog and classify:
   - P0: blocks green/safety/compliance,
   - P1: required hardening,
   - P2: docs/follow-up.
2. Reject any downgrade, destructive reset, or gate-weakening proposal.
3. Implement P0 first, one coherent chunk per PR.
4. Use Feature Forge / `rust-feature-impl` discipline for code changes:
   - engine-first,
   - Rust-native,
   - fail-closed destructive behavior,
   - no C trust-boundary regression,
   - CLI/GUI parity where relevant.
5. Keep logic in Rust/envctl engine or manifest-owned lifecycle hooks; do not introduce unmanaged shell drift.

## Verification

Run the narrowest sufficient checks for each chunk, and expand to full gates for code/trust-boundary/workflow changes:

```bash
rtk cargo test --workspace
rtk cargo +1.88.0 check --workspace --locked
rtk cargo fmt --all --check
rtk cargo clippy --workspace -- -D warnings
rtk bash ci/gates/no-c.sh
rtk bash ci/gates/shape.sh
rtk bash ci/gates/enable.sh
rtk bash ci/gates/p7.sh
rtk bash ci/gates/kdf-feature-off.sh
rtk bash ci/gates/agent-env.sh
rtk bash ci/gates/cargo-audit.sh
rtk bash ci/gates/loop-state.sh
rtk bash ci/gates/harness-scripts.sh
```

If a gate is red from pre-existing unrelated drift, prove it with baseline evidence, record it as a note or follow-up, and do not weaken the gate.

## Publish contract

For every coherent completed chunk:

```bash
rtk git status --short
rtk git add <intended-files-only>
rtk git commit -m "<area>: <summary>"
rtk git push -u origin HEAD
rtk gh pr create --fill
rtk gh pr merge <PR> --auto --squash
```

Do not ask whether to publish after committing. Publish immediately.

## Done criteria

- P0 compliance repairs are merged or have explicit owner-only blockers.
- `envctl` is compliant with current `meta` CLI and `meta/.kb/AGENTS.md` requirements.
- GitHub workflows, runner assumptions, GitHub App/token surfaces, policies, and loop workflows are verified green or have precise external blockers.
- Worktree/branch reaper has run after publication.
- ICM has a durable completion summary under `context-envctl`.
