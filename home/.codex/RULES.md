# Codex Operating Rules

Source: LifeOS Planning Spine v0 - Executable Spec,
Google Doc `1G2wBHIl906eszJckWCPoP79B5OixcJhcpy4pGfO55DU`, tab
`t.6xt8vmxo8u5c`.

## Main Rule

Always anchor the north star, then think small, act small.

## LifeOS Operating Rules

This file is the executable specification for the LifeOS Planning Spine. It
defines the logic, constraints, and protocols required to maintain system
integrity.

### 0. Foundational Architecture

Nix profile via Yazelix + Nix is the foundation binary and runtime model that
everything must adapt to. Always use the latest toolchain. `bun` and `bunx` run
Node.js. The Cargo toolchain is managed via Nix, felix, kache, and the wild
linker.

### 0.1 Agent Navigation And Runtime Ownership

- New envctl sessions start in a fresh worktree created from the latest
  `origin/master` or `origin/develop`; never continue implementation on a stale
  or dirty checkout.
- Agent configuration changes route through envctl `agent-env.yaml`,
  `agent-env.lock`, and `agent-skills/`. Use `envctl agent lock --check` and
  `envctl agent sync --json --color never` before any mutation; use
  `envctl agent sync --apply` only after review.
- `/home/flexnetos/.codex/config.toml` is the active Codex runtime config.
  `/home/flexnetos/lifeos/.codex` and `/home/flexnetos/FlexNetOS/.codex` are
  retired mirror paths and must not be used as active config, source, fallback,
  plugin, MCP, hook, or instruction surfaces.
- Codex/Yazelix toolchain ownership is single-path: profile-owned cargo/rustc
  nightly via Nix/fenix (the owner shorthand has also appeared as "felix"),
  kache for Rust compiler caching, wild through clang linker flags, and
  bun/bunx for Node.js package execution. Avoid global npm/npx/cargo installs.

### 1. System Governance And Cadence

LifeOS is a living system. Its reliability depends on consistent review cycles.

- The Daily Pulse: review `Active Tasks` and clear the `Inbox`.
- The Weekly Sync: review the `Planning Spine` to ensure all Projects align
  with current Priorities.
- The Quarterly Refactor: re-evaluate `Priorities` and `Outcomes`. Archive any
  Project that does not contribute to the current 90-day horizon.

### 2. Structural Integrity Rules

Every node in the Spine must remain executable.

| Node Level | Rule | Constraint |
| --- | --- | --- |
| Vision/Values | Immutability | Modified only during Annual Review. |
| Priorities | The Rule of Three | Maximum of 3 active Priorities at any given time. |
| Projects | Direct Parentage | Every Project must link to at least one Priority. |
| Actions | Atomicity | Actions must be discrete, non-decomposable tasks. |

### 3. Data And Syntax Standards

All entries must remain queryable and clear.

- Date Format: all deadlines and milestones must use `YYYY-MM-DD` (ISO 8601).
- Naming Convention: Projects should be named as completed states, for example
  "Kitchen Remodel Finished" rather than "Remodeling Kitchen".
- Status Logic: use only the defined states below.

| Status | Meaning |
| --- | --- |
| `[BACKLOG]` | Not yet started. |
| `[ACTIVE]` | Current focus. |
| `[BLOCKED]` | Waiting on external dependency or Person. |
| `[DONE]` | Verification required. |

### 4. Execution Logic

- Rule of Stale Data: if a Project has no updated Action for more than 14 days,
  move it to `[BACKLOG]` or delete it.
- Rule of Capture: no commitment is made verbally. If it is not in the `Inbox`
  or `Spine`, it does not exist for execution.
- Rule of Alignment: if a new opportunity does not serve an existing
  `Priority`, it requires a one-in, one-out trade-off or must be rejected.

### 5. Metadata And Ownership

Each durable planning item must preserve:

- Owner: Person.
- Last System Audit: Date.
- Version Reference: File.

### 6. GitHub Protocol And Branch Hygiene

- Commit Management: all commits must be pushed and PRs merged.
- Branching Strategy: use only `develop` and `main` or `master` branches unless
  a repo owner explicitly requires a different branch model.
- Origin Management: `origin` must always track the primary remote repository.
  Fetch and pull regularly from `origin` so local branches reflect the latest
  state. Prune stale remote-tracking branches with `git fetch -p`.
- Git Worktree Workflow: use `git worktree add <path> <branch>` for isolated
  feature development. Each worktree is disposable; remove it immediately after
  merge with `git worktree remove <path>`.
- Workflow: worktrees and feature branches must be removed after merging.
- Sync: `develop` and `main` or `master` must remain in sync.
- Forking: forks auto-sync with upstream.
- Upgrade Policy: always perform an upgrade on all edits or errors. Do not use
  comment-outs, and do not use an approval variable when a warning indicates
  unconnected endpoints.
- Branch Hygiene: never leave a branch dirty, and ensure there are no unmerged
  PRs.

GitHub, PR, merge, publish, and repo-cleanup work must finish with a clean
target repo. Do not end a turn with a dirty branch, an open PR that can be
merged, a stale merged PR branch, an unresolved temp worktree, or a stash created
for the task. Before the final response, prove `git status --short --branch`,
open PR inventory, touched PR merge/check state, and local/remote branch cleanup.

If self-hosted runners or generated repo state dirty the tree after checks or
merge, settle the state through the repo-owned command or policy, archive or
ignore generated artifacts deliberately, commit and push the resulting state when
appropriate, then re-run the clean-status and open-PR proof. A direct state-only
`[skip ci]` push may be used only when it is the deliberate way to break a runner
self-dirty loop, and it must be reported explicitly with proof.

### 7. Infrastructure And Pipeline Integrity

- Build Priority:
  - Rule of Build Priority: The Ubuntu 26.04+ build is the primary gatekeeper.
    No macOS or Windows builds shall execute until the full system has
    successfully built on Ubuntu 26.04+.
- Environment And Dependency Rules:
  - Rule of Runtime Environment: the `nu` (Nushell) binary must be present in
    the system `PATH`. Without it, the environment control tables cannot
    regenerate.
  - Rule of Canonical Tooling: the `bun` runtime is the exclusive executor for
    the planning-spine verification pipeline.
- Pipeline Integrity (Source Of Truth):
  - Rule of Source Provenance: the system of record for the Task Graph is the
    local committed CSV, `generated/task_graph.source.csv`. Google Sheets are
    for visual planning only; they are not the canonical source for execution.
  - Rule of Proof: every entry in the Task Graph requires a `proof_uri`. If a
    row exists without a `proof_uri`, the build must fail.
- Verification Protocol:
  - Rule of Pre-Flight: before any commit or merge,
    `bun run planning-spine:verify` must execute without error.
  - Rule of Manifest Freshness: `PACKAGE_MANIFEST.json` must be regenerated
    immediately after any change to the underlying table structure to prevent
    stale audit states.
