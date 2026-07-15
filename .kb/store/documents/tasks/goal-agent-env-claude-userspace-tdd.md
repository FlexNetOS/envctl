---
id: 019f5432-95ed-7332-a135-49dbbc14c0c3
slug: tasks/goal-agent-env-claude-userspace-tdd
title: "Goal: TDD loops upgrading all Claude-related user-space files via agent-env"
type: task
status: active
priority: high
tags: [agent-env, goal, tdd, claude-userspace]
---

## Overview
/goal orchestration (2026-07-12): run /agent-env-codex in TDD loops fixing every found issue,
upgrade the agent-env-codex skill at each loop boundary, repeat until agent-env has upgraded all
Claude-related files/folders in user space. Strict upgrade only; PR-per-chunk with auto-merge.

## Goals
- Loop 1 (pivoted): orphaned codex yazelix WIP proven ALREADY LANDED at HEAD (reference identical, checker newer, validate.sh wired, test present) -> close ICM note; loop 1 becomes the live settings superset reconciliation.
- Reconcile live ~/.claude/settings.json with envctl home/.claude source (superset: repo prefs + live-only weave/icm hooks) with a regression test for the divergence class.
- Loops 2+: audit-driven fixes from the 5-agent fan-out (meta-cli, kb, envctl-compliance, github-workflows, loop-harness).
- P1 queue: lifeos stale Stop hook (/home/drdave path), yazelix `br` hooks (binary absent), meta guard binary unbuilt, .kb/AGENTS.md documents .kb/workspace/ but live layout is .kb/workspaces/main/.

## Acceptance Criteria
- [ ] Yazelix orphan proven green at HEAD (test run output) and ICM note corrected.
- [ ] Settings superset reconciliation landed with regression test, PR MERGED.
- [ ] Audit backlog triaged P0/P1/P2; every P0 merged or owner-blocked.
- [ ] agent-env-codex skill upgraded at each loop boundary.
- [ ] Reaper clean after each merge; ICM completion summary stored.

## Progress Log
### 2026-07-12 (loops 5-6 + activations)
- Loop 5 MERGED (#496): ratified defaultMode:auto encoded (TDD RED-first); Tier-B ratification contract in codex runbook.
- Loop 6 armed (#497): runner-routing derives jobs from ci.yml (+floor guard + hermetic test w/ probe-job and floor-removal fixtures); dead LOOP_LIB_REF pin dropped; actionlint gate wired.
- yazelix #52 MERGED + profile update: actionlint 1.7.12 live on toolbin -> the new gate PASSes live (not skip).
- br chain complete: br 0.2.16 nix-owned, br ready serves triage, beads JSONL deduped (#51), doctor --repair green.
- Recurring self-inflicted bug hit 3rd time: piping build/check commands through tail masks exit codes — capture exit BEFORE piping.
### 2026-07-12 (br research + rulings enforcement + meta KB fix)
- br-hook research workflow (3 lenses + synthesis, 545k tokens): br = beads_rust v0.2.16 sole bin (FlexNetOS fork; agent issue tracker, fsqlite+JSONL); yazelix flake ALREADY packages it (packages.br) but lifeos_foundation_yzx extraRuntimePackages never listed it; only yazelix wires the hooks; develop already carries fail-soft guards (the bare hooks were the STALE feat/agent-harness-init checkout).
- yazelix #50 MERGED: beads_rust + "br" added to the foundation runtime; checkout moved feat/agent-harness-init (merged #36) -> develop b71238c3; profile rebuild (yzx update local_source) running for activation.
- meta #106 MERGED: .kb/AGENTS.md workspace path corrected to .kb/workspaces/<name>/ (runtime-verified the other documented forms are VALID - auditor drift table overstated without a shell).
- PR #495 incident: first push violated the codified no-root-var owner ruling (crates/cli/src/main.rs:3164 gate) - ${META_ROOT} tmpl parameterization REVERTED, effortLevel kept, test+runbook rewritten to encode the ruling; env_cmd_tests 3 passed; fix 36449de pushed, auto-merge armed. Lesson ICM'd: grep codified rulings before config-shape changes; run workspace tests locally.
### 2026-07-12 (loops 3-4 + operator rulings)
- Loop 3 PR #495 armed: effortLevel adopted (settings+tmpl), tmpl fully ${META_ROOT}-parameterized (8 paths), render round-trip proven byte-exact; codex runbook gained settings-template ownership contract.
- Loop 4 MERGED: lifeos #33 — Stop session-log hook repointed /home/drdave path -> $CLAUDE_PROJECT_DIR (smoke exit 0); worktree+branch reaped.
- Operator rulings (AskUserQuestion, markers .answered): meta/src sibling remotes repointed to org SSH (done, ssh fetch proven); rust-analyzer-lsp TRUE wins (repo already true); defaultMode:"auto" RATIFIED -> encode in loop 5.
- Meta guard restored: agent crate built to meta/target/debug/agent; denies git reset --hard, passes benign.
### 2026-07-12 (loop 2 + synthesis)
- 5-auditor fan-out returned; agents released (roster empty). Backlog: P0s all in loops 1-2; P1 remaining: lifeos stale Stop hook (/home/drdave path), yazelix `br` hooks (binary not on PATH), settings residuals (effortLevel; 2 operator decision markers pending), meta/src sibling HTTPS remotes (decision marker: classifier-gated repoint). P2: LOOP_LIB_REF dead pin, runner-routing literal job list, actionlint unwired, ${META_ROOT} tmpl parameterization, meta-root AGENTS.md git-kb syntax drift, envctl-lock-0.2.25-vs-tree-0.2.22 provenance (CI empirically green; local lock churn understood — never commit the downgrade).
- Loop 2 landed as PR #494 (armed): HANDOFF-parity gate check 5 (TDD RED-first), DONE sentinel archived+removed, HANDOFF re-rendered (handoff_cycles_total:66), escalations drained to TASK-0097/0098, yazelix-codex-runtime gate wired into CI, worktree-ritual + .kb workspaces doc truth, forge-loop prior-loop sentinel exception, codex runbook lock-driven sync order.
- Restored meta destructive-command guard: built agent crate to meta/target/debug/agent (hook path); guard blocks `git reset --hard`, passes `git status` (smoke below in terminal log).
### 2026-07-12 (loop 1)
- Yazelix orphan: proven landed+green at HEAD (cargo test 1 passed); ICM note corrected.
- Loop 1 TDD landed as PR #493 (auto-merge armed): RED substrate-hook-parity test -> ten weave/icm hooks adopted PATH-resolved into settings.json+tmpl; codex runbook contract upgraded (lock regen + sync).
- Found en route: envctl worktree sets need loop_lib/meta_plugin_protocol siblings (skipped by meta worktree create: no origin/develop); meta root workspace 0.2.22 vs envctl lock 0.2.25 (local tree likely behind origin) -> P1; validate.sh bun scan red at baseline on vendored private-codex-state cache -> scanner-precision fix queued.

### 2026-07-12
- /goal preflight done: ICM recall, reaper clean, master==origin, KB live. 5 background auditors spawned (opus).
- Orphan-archive diff vs HEAD: references IDENTICAL, check-yazelix-contract.py landed newer (310 lines), validate.sh wired, yazelix test present at tests/full_access_contract.rs:419.
