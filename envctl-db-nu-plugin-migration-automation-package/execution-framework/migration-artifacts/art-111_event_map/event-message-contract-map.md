# Event Message Contract Map

Task: `ART-111_EVENT_MAP`
Generated at: `2026-07-28T11:27:22+00:00`
Target root: `/home/flexnetos/FlexNetOS`

## Scope

This map records event and message contracts found in the target filesystem and envctl migration database artifacts. It covers topics or streams, queues, payload records, producers, consumers, retry semantics, and dead-letter or failure handling. Static source matches are evidence candidates; durable contract rows come from the envctl database and shared protocol manifests.

## Contract Summary

| topic | queue or stream | payload | producers | consumers | retry | DLQ or failure path |
|---|---|---|---|---|---|---|
| `envctl.migration.run_events` | append-only hash-chained event stream | `schemas/run_event.schema.json and shared_protocol.RunEvent` | envctl mutating migration commands with --emit-event | nu_plugin envctl migration events, nu_plugin status stream, live timeline view | idempotent command retry keyed by operation/run id; replay validates event and proof hashes | invalid or blocked events remain in operation/evidence failure surfaces; failed_ops/open_approvals views expose remediation |
| `envctl.migration.operations` | operation queue | `shared_protocol.Operation` | task packets, envctl command execution, artifact generators | envctl operation state machine, nu_plugin ops/status commands, artifact registry producer checks | idempotency_key plus command_hash prevents duplicate unsafe work | failed operation status, error_ref, rollback handles, and validation evidence become the remediation queue |
| `envctl.migration.artifacts` | artifact registry event surface | `shared_protocol.ArtifactRecord and EvidenceRecord` | artifact generation tasks including ART-111 | validation tasks, proof ledger merge, readiness scorecard | content hash recomputation and ON CONFLICT upserts keep artifact registration repeatable | blocked paths, mismatched hashes, and foreign producer operations are rejected fail-closed |
| `envctl.migration.approvals` | human approval queue | `shared_protocol.ApprovalRequest and ApprovalDecision` | risk-bearing envctl operations | nu_plugin approve/deny commands, operation state machine, run ledger | approval decision events are append-only and tied to operation id | denied/expired/blocked approvals halt execution until the operator records a decision |
| `envctl.migration.validation` | validation evidence ledger | `shared_protocol.ValidationResult` | verification commands and artifact registry validators | VER-300_UNIT_VALIDATION, readiness scorecard, proof ledger | validation commands are rerunnable and reference immutable evidence hashes | fail/warn/blocked statuses carry next_action through proof and evidence records |

## Signal Counts

| signal | count |
|---|---:|
| contract count | 5 |
| topic signal count | 180 |
| queue signal count | 180 |
| payload signal count | 180 |
| producer signal count | 180 |
| consumer signal count | 180 |
| retry signal count | 180 |
| dlq signal count | 180 |
| hotspot count | 100 |

## Hotspots

| file | score | categories |
|---|---:|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.claude/agents/continuity-steward.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.claude/agents/handoff-kernel-engineer.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.claude/agents/rust-port-cartographer.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.claude/skills/planning-engineer/SKILL.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.claude/skills/rust-port/SKILL.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.codex/agents/continuity-steward.toml` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.codex/agents/handoff-kernel-engineer.toml` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.codex/prompts/prompt:codex-gpt-harness-v3-full-access-no-sandbox.prompt.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.codex/prompts/prompt:codex-gpt-harness.prompt.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.codex/skills/agent-env-codex/references/source-prompt.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.handoff/loop/backlog.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.handoff/loop/loop_state.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.handoff/loop/plan/HANDOFF.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.handoff/loop/plan/findings/architecture-grit.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.handoff/loop/plan/findings/filesystem-layout-icm.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.handoff/loop/plan/findings/governance-config-weave.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.handoff/loop/plan/findings/memory-vector-intelligence-grit.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.handoff/loop/plan/findings/memory-vector-intelligence-rusty-idd.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.handoff/loop/plan/findings/memory-vector-intelligence-weave.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.handoff/loop/plan/findings/rules-policy-org-grit.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.handoff/loop/plan/findings/rules-policy-org-rusty-idd.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.handoff/loop/plan/findings/rules-policy-org-weave.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.handoff/loop/plan/graph/weave.symbols.json` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.handoff/loop/plan/reports/codemap-grit.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.handoff/loop/plan/reports/grit-plan.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.handoff/loop/plan/reports/prompt-hub-plan.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.handoff/loop/plan/reports/weave-plan.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.handoff/loop/plan/research/agentic-planning-trends-2026-06.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.handoff/loop/rust-port/parity-ledger.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |

## Topics And Streams

| file | line | signal | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-env.yaml` | 11 | `phase-channel` | # envctl agent sync --apply --color never # only after review/approval |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 8 | `phase-channel` | \| date \| harness \| lesson (class, generalized) \| evidence (cycle/finding) \| recurrence \| routed-to \| status \| |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 11 | `phase-channel` | \| date \| harness \| lesson (class, generalized) \| evidence \| recurrence \| routed-to \| status \| |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 13 | `phase-channel` | \| 2026-06-17 \| feature-forge \| Verify a triggering claim that asserts concrete code state against HEAD before designing — cross-session relay/handoff claims go stale; a plan built on a false premise wastes a cycle (no-fa |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 17 | `phase-channel` | \| 2026-06-17 \| feature-forge \| Phase-1.5 routing counts *independent* modules, not raw modules — a strict dependency chain (parallelism 0) stays sequential even when n>3; honor the architect's explicit routing recommenda |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 22 | `phase-channel` | \| 2026-06-18 \| feature-forge \| Prevent drift at PICK-time, not just at wrap-up: a frozen-consumer-contract check that runs only as a reactive end-of-session sweep fires a cycle too late. Grep for a frozen CLI/RPC/JSON co |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 28 | `phase-channel` | \| 2026-06-22 \| feature-forge \| RECURRENCE-CONFIRM (no new upgrade): the verify-against-HEAD-before-d... class (rows 2026-06-17 Phase-0 step 4 + 2026-06-18 Phase-0 step 5) generalizes beyond a *suspect stale relay claim*  |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 39 | `phase-channel` | \| 2026-06-28 \| forge-loop \| Owner-supervised migrations should climb a review ladder in separate PRs — status/precondition, validation, scaffold, explicit writer, and only then any live movement. Do not combine manifest  |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 25 | `phase-channel` | and tested — notably: `--connect` bypassed all spec validation (path traversal + |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 36 | `phase-channel` | ## Status (commits on `master`) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 38 | `phase-channel` | Phase 4+5 add-repo+telemetry · Phase 3 reset/auto-fix · GUI theme · PRD · |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 39 | `phase-channel` | Phase 2 streaming install · review fixes · Phase 0+1 scaffold+manifest+drift |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/README.md` | 136 | `phase-channel` | ## Status |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/README.md` | 138 | `phase-channel` | **Phase 0 + a working `auto-detect`.** The workspace compiles green on the latest nightly and |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 35 | `phase-channel` | (meta's shared builder) while keeping its own supervision (setsid reaping, per-phase timeout, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 51 | `phase-channel` | (`git fetch && git status` — confirm clean and even with `origin/master`): |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 69 | `phase-channel` | name `envctl` to `meta git worktree status` unless this helper derived it from |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 265 | `phase-channel` | \| 2026-06-05 \| Add component-research/audit phase (auto-append upgrades to backlog) \| skills/env-install-loop; skills/auto-provision (+scripts/ralph-provision.sh) \| Generalize the manual pytorch deep-dive (shallow gate,  |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 266 | `phase-channel` | \| 2026-06-05 \| Add A2 cross-repo parallel build (default-OFF, scale auto-trigger) \| skills/{feature-forge,forge-loop,session-relay}; agents/{rust-implementer,continuity-steward} \| Cross-repo parallelism via the three-own |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 269 | `phase-channel` | \| 2026-06-12 \| Migrate harness durable state `_workspace/`→`.handoff/loop/`; add kasetto-absorption capability + handoff-sync skill + hf-aware continuity \| skills/{forge-loop,feature-forge,session-relay,env-install-loop, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 273 | `phase-channel` | \| 2026-06-17 \| G2 retro: encode the instincts that made the run clean (evolution-steward, 5 low-risk APPLY) \| skills/feature-forge (Phase 0 verify-triggering-claim step; Phase 1.5 independent-modules routing); agents/fea |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 274 | `phase-channel` | \| 2026-06-17 \| Anti-drift: backlog reconcile is now a FAIL-CLOSED wrap-up gate (+ full drift sweep) \| skills/session-relay-wrap-up (new step 3b); .handoff/loop/backlog.md (reconcile + Epic F) \| Owner flag: *"during forge |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 275 | `phase-channel` | \| 2026-06-18 \| Worktree/branch reaper — keep worktrees ↔ branches ↔ origin consistent \| scripts/reap-worktrees.sh (NEW); skills/session-relay-wrap-up (new step 5b); skills/session-relay-resume (new step 4b); skills/forge |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 276 | `phase-channel` | \| 2026-06-18 \| Forge-loop audit upgrades U1/U3/U4/U6 — status integrity + merge safety + drift prevention \| skills/forge-loop (TICK-ON-MERGED gate in steps 4-5; "Worktree hygiene" already; auto-provision-for-unattended n |
| ... |  |  | 156 more entries in JSON artifact |

## Queues

| file | line | signal | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 29 | `job-queue` | \| 2026-06-23 \| forge-loop \| Don't dodge an infra limit — AUTHENTICATE with the infra the owner already built. When automation hits a quota/permission wall (GitHub API 403), the fix is to use the available credential, not |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 37 | `operation-queue` | \| 2026-06-27 \| plan-loop \| Self-referential sentinel-token gate trap: an auditor that quotes the completeness-gate's own rejection vocabulary verbatim (the placeholder/uncertainty sentinel words the gate rejects case-ins |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 44 | `job-queue` | \| 2026-07-09 \| forge-loop \| Under a limited-runner local-first CI fleet with strict up-to-date protection, every develop merge re-triggers armed PRs' merge-ref runs and re-BEHINDs them (the "BEHIND treadmill"). Mitigate  |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/README.md` | 110 | `job-queue` | Add-Repo form · Live Logs · Settings. The engine runs on a worker thread; the UI never blocks. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/README.md` | 157 | `job-queue` | crates/engine/ # envctl_engine: Component model, Registry, the 5 verbs, detect, guards, GUI worker API |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 198 | `job-queue` | is set. This prevents accidental background Claude sessions and auto-spawn loops. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 266 | `job-queue` | \| 2026-06-05 \| Add A2 cross-repo parallel build (default-OFF, scale auto-trigger) \| skills/{feature-forge,forge-loop,session-relay}; agents/{rust-implementer,continuity-steward} \| Cross-repo parallelism via the three-own |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 267 | `job-queue` | \| 2026-06-08 \| Add grit-harness-parallel opt-in mode \| skills/{feature-forge,forge-loop} \| Adopt grit's claim→work→done AST git-lock coordination into the harness for parallel multi-repo implementations: `grit init` (ide |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 271 | `operation-queue` | \| 2026-06-13 \| Wire continuity auto-hooks (dormant until hf) + fix broken `.kb` hook \| .claude/settings.json (NEW project layer); .claude/hooks/hf-checkpoint.sh (NEW); .handoff/loop/backlog.md; (meta repo) .claude/settin |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 277 | `job-queue` | \| 2026-06-18 \| U2/U5 — complete the kasetto integration: migrate config filenames + wire the (claimed-but-absent) drift gate (TASK-0040) \| agent-env.yaml + agent-env.lock (git mv from kasetto.yaml/kasetto.lock; header re |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/AGENTS.md` | 247 | `job-queue` | is set. This prevents accidental background Codex sessions and auto-spawn loops. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/AGENTS.md` | 271 | `operation-queue` | git-kb status --json # Check for pending changes |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/context/immutable/project-brief.md` | 14 | `job-queue` | envctl is secondary — a subordinate agent whose single job is to **own and converge the meta |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/kb-status/SKILL.md` | 3 | `operation-queue` | description: Show workspace status and pending changes |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/kb-status/agents/openai.yaml` | 4 | `operation-queue` | default_prompt: "Use $kb-status to show workspace status and pending changes." |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/gitkb/SKILL.md` | 40 | `operation-queue` | git-kb status --json # Show pending changes |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/gitkb/SKILL.md` | 106 | `operation-queue` | \| `git-kb status --json` \| `kb_status` \| Show pending changes \| |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/kb-context/SKILL.md` | 53 | `operation-queue` | 1. Check `kb_status` for pending changes |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/kb-handoff/SKILL.md` | 3 | `operation-queue` | description: End-of-session handoff — update context, log progress, commit pending changes |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/kb-handoff/SKILL.md` | 14 | `operation-queue` | ### 1. Commit Pending Changes |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/kb-handoff/SKILL.md` | 71 | `operation-queue` | - Pending work: [what's next] |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/kb-commit/SKILL.md` | 6 | `operation-queue` | Review, validate, and commit pending workspace changes. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/kb-commit/SKILL.md` | 46 | `operation-queue` | > "kb_status shows changes to `<slug>` which I didn't modify. Excluding it from this commit. Another agent may have pending changes." |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/kb-commit/agents/openai.yaml` | 4 | `operation-queue` | default_prompt: "Use $kb-commit to review, validate, and commit pending workspace changes." |
| ... |  |  | 156 more entries in JSON artifact |

## Payloads And Schemas

| file | line | signal | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 43 | `json-payload` | \| 2026-07-09 \| forge-loop \| Hook-enforcement verify-at-pick must account for SESSION-SNAPSHOT semantics: Claude Code binds the hook set at session start, so a hook wired mid-session does NOT fire in the running session a |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 97 | `schema` | bash ci/gates/p7.sh # .handoff Tier-A p7-conformance: schema tags + ledger residency (ADR-0004 §3) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 266 | `schema` | \| 2026-06-05 \| Add A2 cross-repo parallel build (default-OFF, scale auto-trigger) \| skills/{feature-forge,forge-loop,session-relay}; agents/{rust-implementer,continuity-steward} \| Cross-repo parallelism via the three-own |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 270 | `schema` | \| 2026-06-13 \| Add `handoff-kernel-engineer` agent (Epic A) + seed loop_state to schema + reconcile backlog \| agents/handoff-kernel-engineer.md; skills/feature-forge (crew table + Epic-A Build routing); .handoff/loop/{lo |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 277 | `schema` | \| 2026-06-18 \| U2/U5 — complete the kasetto integration: migrate config filenames + wire the (claimed-but-absent) drift gate (TASK-0040) \| agent-env.yaml + agent-env.lock (git mv from kasetto.yaml/kasetto.lock; header re |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 281 | `schema` | \| 2026-06-18 \| P1/P2/P3 + batch wrap-up cadence — make the periodic reaper/wrap-up/retro hook-enforced, not skippable \| scripts/tests/{test-merge-driver,test-reaper}.sh (NEW); ci/gates/harness-scripts.sh (NEW) + ci.yml s |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 474 | `schema` | \| Tools, resources, schema reference \| `.claude/skills/gitnexus/gitne....md` \| |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/AGENTS.md` | 81 | `schema` | bash ci/gates/p7.sh # .handoff Tier-A p7-conformance: schema tags + ledger residency (ADR-0004 §3) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/AGENTS.md` | 352 | `schema` | \| Tools, resources, schema reference \| `.claude/skills/gitnexus/gitne....md` \| |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/retire-broken-pre-cleanroom-codex-hook-baseline.md` | 41 | `hash-chain` | sha256: 2f3122f94d847314886fe1999b8c... |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/tasks/catalog-runtime-closure-and-toolchain-proof.md` | 15 | `schema` | That is real progress, but it is not runtime closure yet. The live audit also surfaced hard gaps between the table story and the runtime story: `migration_evidence` is still empty, every `paths` row is `not_checked`, the |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/tasks/catalog-runtime-closure-and-toolchain-proof.md` | 44 | `schema` | - Distinguish declared/schema/layout env vars from observed/effective runtime env vars. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/tasks/codedb-nu-plugin-semantic-coverage.md` | 20 | `schema` | - `agent-skills/codedb-config-t....md` |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/tasks/codedb-nu-plugin-semantic-coverage.md` | 32 | `schema` | - `codedb schema` |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/tasks/codedb-nu-plugin-semantic-coverage.md` | 36 | `hash-chain` | - `content_hash`, `blob_ref`, `import_status`, `skip_reason` |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/tasks/codedb-content-blob-inventory.md` | 28 | `hash-chain` | - `content_hash` |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/tasks/codedb-metadata-only-inventory.md` | 29 | `hash-chain` | - empty `content_hash` / `blob_ref` |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/commits/019f24bb-82cb-7462-9bb1-ed51988c969e.json` | 2 | `hash-chain` | "commit": {"author":"drdave-flexnet <flexnetos@de-flex.net>","changes":[{"change_type":"created","content_hash":"65fb5c8d0cb97ffe0391aea1cb2a...","doc_type":"task","document_id":"019f24bb-6871-72d1-ad69-921e...","new_ver |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/commits/019f24bb-82cb-7462-9bb1-ed51988c969e.json` | 3 | `hash-chain` | "content_hash": "f60e3a28f4b2223078efb0b71bf0...", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/commits/019f0078-ad4d-7513-acd0-6b51e28f4507.json` | 2 | `hash-chain` | "commit": {"author":"drdave <revenaugh.david@gmail.com>","changes":[{"change_type":"created","content_hash":"8dd918d90abdba2359f8b0b10b1d...","doc_type":"context","document_id":"019f0078-83f2-7493-8d62-df3c...","new_vers |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/commits/019f0078-ad4d-7513-acd0-6b51e28f4507.json` | 3 | `hash-chain` | "content_hash": "1b210e773b9fd69b04ff0e009761...", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/commits/019f25be-1090-7090-a9dc-8b5c6bb10b64.json` | 2 | `hash-chain` | "commit": {"author":"drdave <flexnetos@de-flex.net>","changes":[{"change_type":"modified","content_hash":"9685f1e3f3c30ef7a44aedc54564...","doc_type":"task","document_id":"019f2588-5a5e-7420-b390-2204...","new_version":2 |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/commits/019f25be-1090-7090-a9dc-8b5c6bb10b64.json` | 3 | `hash-chain` | "content_hash": "8e572eb846293a05622b244cb93d...", |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/commits/019f2589-0827-7fa2-9674-f6b02f4f26cb.json` | 2 | `hash-chain` | "commit": {"author":"drdave <flexnetos@de-flex.net>","changes":[{"change_type":"created","content_hash":"9685f1e3f3c30ef7a44aedc54564...","doc_type":"task","document_id":"019f2588-5a5e-7420-b390-2204...","new_version":1, |
| ... |  |  | 156 more entries in JSON artifact |

## Producers

| file | line | signal | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 18 | `mutation-command` | \| 2026-06-18 \| forge-loop \| "Merged" ≠ "clean": workspace hygiene is a first-class loop OUTPUT, not a side effect. A loop that spawns a per-cycle resource (worktree/branch) MUST reap it at a settled boundary, or it accum |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 21 | `mutation-command` | \| 2026-06-18 \| forge-loop \| High-churn shared state committed in feature branches needs a merge DRIVER, not git's default 3-way — append-heavy files (loop_state/backlog) silently CONCATENATE across non-overlapping region |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 27 | `producer` | \| 2026-06-23 \| forge-loop \| Write-side complement of TICK-ON-MERGED: once `gh pr merge --auto` is armed, treat the PR branch as potentially-already-merged. A fast PR can merge (and origin can auto-delete the head, `delet |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 33 | `producer` | \| 2026-06-23 \| feature-forge \| When two components must write the SAME runtime config file, each owns a delimited `# >>> name >>> … # <<< name <<<` block written by a NON-CLOBBERING upsert (strip-my-block, append-fresh)  |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 37 | `producer` | \| 2026-06-27 \| plan-loop \| Self-referential sentinel-token gate trap: an auditor that quotes the completeness-gate's own rejection vocabulary verbatim (the placeholder/uncertainty sentinel words the gate rejects case-ins |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/README.md` | 28 | `mutation-command` | \| `db` \| index and query Meta/LifeOS roots and symbols; plan root-alias refactors and staged hook deployments \| read-only/plan (`--apply --confirm --approve` to mutate) \| |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 213 | `mutation-command` | the roadmap", "run unattended") use **`forge-loop`**; for **cross-session handoff/resume** ("transfer |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 214 | `mutation-command` | the session", "resume from handoff") use **`session-relay`** (checkpoints via `continuity-steward`, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 224 | `mutation-command` | "make .handoff tier-A", "resume handoff full-sync") use **`handoff-sync`** (Epic A; distinct from |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 259 | `producer` | \| 2026-06-04 \| Architect uses return-value (not Write) \| agents/feature-architect; skills/feature-forge \| Smoke test: `Plan` type is read-only and cannot Write its plan file — orchestrator persists the returned text \| |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 262 | `mutation-command` | \| 2026-06-05 \| Correct relay signal model after full smoke \| skills/session-relay \| Smoke test: `CronCreate{durable}` is session-only here (not persisted), and a self-identity weave message is invisible to the successor' |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 266 | `mutation-command` | \| 2026-06-05 \| Add A2 cross-repo parallel build (default-OFF, scale auto-trigger) \| skills/{feature-forge,forge-loop,session-relay}; agents/{rust-implementer,continuity-steward} \| Cross-repo parallelism via the three-own |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 272 | `mutation-command` | \| 2026-06-13 \| Eject `rust-port-merge` harness for the kasetto absorption (Epic C / TASK-0012) \| .claude/skills/{rust-port,rust-port-inventory,rust-port-translate,rust-port-parity,rust-port-merge,cross-repo-reference,icm |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 275 | `mutation-command` | \| 2026-06-18 \| Worktree/branch reaper — keep worktrees ↔ branches ↔ origin consistent \| scripts/reap-worktrees.sh (NEW); skills/session-relay-wrap-up (new step 5b); skills/session-relay-resume (new step 4b); skills/forge |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 276 | `mutation-command` | \| 2026-06-18 \| Forge-loop audit upgrades U1/U3/U4/U6 — status integrity + merge safety + drift prevention \| skills/forge-loop (TICK-ON-MERGED gate in steps 4-5; "Worktree hygiene" already; auto-provision-for-unattended n |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 281 | `mutation-command` | \| 2026-06-18 \| P1/P2/P3 + batch wrap-up cadence — make the periodic reaper/wrap-up/retro hook-enforced, not skippable \| scripts/tests/{test-merge-driver,test-reaper}.sh (NEW); ci/gates/harness-scripts.sh (NEW) + ci.yml s |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 283 | `mutation-command` | \| 2026-06-18 \| Epic G Tier-2/3 complete: TASK-0044 (hf-kernel cards) + TASK-0052 (harness_hub package) \| TASK-0044: `.handoff/tasks/TASK-*.task.json` (53 fleet-scoped cards via handoff-kernel-engineer) + `skills/forge-lo |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/AGENTS.md` | 262 | `mutation-command` | the roadmap", "run unattended") use **`forge-loop`**; for **cross-session handoff/resume** ("transfer |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/AGENTS.md` | 263 | `mutation-command` | the session", "resume from handoff") use **`session-relay`** (checkpoints via `continuity-steward`, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/AGENTS.md` | 273 | `mutation-command` | "make .handoff tier-A", "resume handoff full-sync") use **`handoff-sync`** (Epic A; distinct from |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/AGENTS.md` | 67 | `mutation-command` | \| Context already loaded this session \| **PATH C**: Quick resume \| |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/AGENTS.md` | 266 | `mutation-command` | ## PATH C: Returning Agent (Quick Resume) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/AGENTS.md` | 275 | `mutation-command` | If workspace is clean and context is still valid, resume work. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/AGENTS.md` | 788 | `mutation-command` | - **Continue**: Resume where you left off |
| ... |  |  | 156 more entries in JSON artifact |

## Consumers

| file | line | signal | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-env.yaml` | 9 | `consumer` | # envctl agent lock --check --locked --color never # read-only, zero-network |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-env.yaml` | 26 | `nu-plugin` | # The only active user runtime is the Yazelix-managed Nushell route. Agent sync/lock/doctor |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-env.yaml` | 29 | `nu-plugin` | runtime: yazelix-nushell |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 22 | `consumer` | \| 2026-06-18 \| feature-forge \| Prevent drift at PICK-time, not just at wrap-up: a frozen-consumer-contract check that runs only as a reactive end-of-session sweep fires a cycle too late. Grep for a frozen CLI/RPC/JSON co |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 27 | `consumer` | \| 2026-06-23 \| forge-loop \| Write-side complement of TICK-ON-MERGED: once `gh pr merge --auto` is armed, treat the PR branch as potentially-already-merged. A fast PR can merge (and origin can auto-delete the head, `delet |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 29 | `consumer` | \| 2026-06-23 \| forge-loop \| Don't dodge an infra limit — AUTHENTICATE with the infra the owner already built. When automation hits a quota/permission wall (GitHub API 403), the fix is to use the available credential, not |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 32 | `consumer` | \| 2026-06-23 \| feature-forge \| Build-affecting wiring (RUSTC_WRAPPER, cargo linker) belongs in the meta-root `.cargo/config.toml`, NOT the `envctl env` env-seam: cargo reads its config in EVERY context (interactive, logi |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 42 | `consumer` | \| 2026-07-09 \| feature-forge \| Treat an upstream ANN/HNSW result set as a MULTISET, not a set: a k-NN index can return the SAME node twice (identical id+score) in one result set, so a consumer needing k DISTINCT hits mus |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 44 | `consumer` | \| 2026-07-09 \| forge-loop \| Under a limited-runner local-first CI fleet with strict up-to-date protection, every develop merge re-triggers armed PRs' merge-ref runs and re-BEHINDs them (the "BEHIND treadmill"). Mitigate  |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 66 | `consumer` | # 2. auto-detect — read-only; should see 2x RTX 5090 (PCI floor), driver, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 83 | `consumer` | # 3c. doctor — read-only health (writability, toolchains, sudo, UEFI, GPU) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 136 | `consumer` | # Read-only list works without a config (uses cwd as project scope) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/README.md` | 19 | `consumer` | \| `auto-detect` \| read-only inventory: host, GPU (works pre-driver), tools, component drift \| — \| |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/README.md` | 26 | `consumer` | \| `doctor` \| read-only health: writability, toolchains, sudo, UEFI/Secure-Boot, GPU, last-op \| — \| |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/README.md` | 27 | `consumer` | \| `migrate` \| adopt legacy/global installs into the `$META_ROOT` FHS/XDG layout, preserve agent assets, protect shared meta substrates, and refuse unsafe purge \| read-only (`apply --apply` materializes dirs) \| |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/README.md` | 28 | `consumer` | \| `db` \| index and query Meta/LifeOS roots and symbols; plan root-alias refactors and staged hook deployments \| read-only/plan (`--apply --confirm --approve` to mutate) \| |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/README.md` | 37 | `consumer` | cargo run -p envctl -- auto-detect # read-only; safe to run anytime |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/README.md` | 58 | `consumer` | read-only root and query views: |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/README.md` | 80 | `consumer` | \| read-only shared data + generated drop-ins \| `$META_ROOT/usr/share` \| |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/README.md` | 140 | `consumer` | read-only verb is fully implemented and validated on the live dual-5090 box (PCI-floor GPU |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 78 | `consumer` | cargo run -p envctl -- auto-detect # read-only, safe anytime (add --json for EnvReport) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 124 | `consumer` | This is a **pure-Rust** workspace by design. Watch for and immediately correct any drift toward |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 157 | `consumer` | generated. CI enforces with `envctl agent lock --check --locked` (read-only, zero-network, exits 1 on |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 252 | `consumer` | > The loop is read-only on product code and writes planning artifacts under `.handoff/loop/plan/`. |
| ... |  |  | 156 more entries in JSON artifact |

## Retries And Replay

| file | line | signal | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 18 | `recovery` | \| 2026-06-18 \| forge-loop \| "Merged" ≠ "clean": workspace hygiene is a first-class loop OUTPUT, not a side effect. A loop that spawns a per-cycle resource (worktree/branch) MUST reap it at a settled boundary, or it accum |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 21 | `recovery` | \| 2026-06-18 \| forge-loop \| High-churn shared state committed in feature branches needs a merge DRIVER, not git's default 3-way — append-heavy files (loop_state/backlog) silently CONCATENATE across non-overlapping region |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 29 | `retry` | \| 2026-06-23 \| forge-loop \| Don't dodge an infra limit — AUTHENTICATE with the infra the owner already built. When automation hits a quota/permission wall (GitHub API 403), the fix is to use the available credential, not |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 35 | `retry` | (meta's shared builder) while keeping its own supervision (setsid reaping, per-phase timeout, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 213 | `recovery` | the roadmap", "run unattended") use **`forge-loop`**; for **cross-session handoff/resume** ("transfer |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 214 | `recovery` | the session", "resume from handoff") use **`session-relay`** (checkpoints via `continuity-steward`, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 224 | `recovery` | "make .handoff tier-A", "resume handoff full-sync") use **`handoff-sync`** (Epic A; distinct from |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 225 | `recovery` | `session-relay`, which is the per-loop checkpoint). Simple questions and |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 261 | `recovery` | \| 2026-06-04 \| Add continuity layer: Ralph loop + session handoff \| agents/continuity-steward; skills/{forge-loop,session-relay}; skills/feature-forge \| Run Feature Forge continuously over a backlog and survive context r |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 262 | `recovery` | \| 2026-06-05 \| Correct relay signal model after full smoke \| skills/session-relay \| Smoke test: `CronCreate{durable}` is session-only here (not persisted), and a self-identity weave message is invisible to the successor' |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 266 | `recovery` | \| 2026-06-05 \| Add A2 cross-repo parallel build (default-OFF, scale auto-trigger) \| skills/{feature-forge,forge-loop,session-relay}; agents/{rust-implementer,continuity-steward} \| Cross-repo parallelism via the three-own |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 271 | `recovery` | \| 2026-06-13 \| Wire continuity auto-hooks (dormant until hf) + fix broken `.kb` hook \| .claude/settings.json (NEW project layer); .claude/hooks/hf-checkpoint.sh (NEW); .handoff/loop/backlog.md; (meta repo) .claude/settin |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 272 | `recovery` | \| 2026-06-13 \| Eject `rust-port-merge` harness for the kasetto absorption (Epic C / TASK-0012) \| .claude/skills/{rust-port,rust-port-inventory,rust-port-translate,rust-port-parity,rust-port-merge,cross-repo-reference,icm |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 275 | `recovery` | \| 2026-06-18 \| Worktree/branch reaper — keep worktrees ↔ branches ↔ origin consistent \| scripts/reap-worktrees.sh (NEW); skills/session-relay-wrap-up (new step 5b); skills/session-relay-resume (new step 4b); skills/forge |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 276 | `recovery` | \| 2026-06-18 \| Forge-loop audit upgrades U1/U3/U4/U6 — status integrity + merge safety + drift prevention \| skills/forge-loop (TICK-ON-MERGED gate in steps 4-5; "Worktree hygiene" already; auto-provision-for-unattended n |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 281 | `recovery` | \| 2026-06-18 \| P1/P2/P3 + batch wrap-up cadence — make the periodic reaper/wrap-up/retro hook-enforced, not skippable \| scripts/tests/{test-merge-driver,test-reaper}.sh (NEW); ci/gates/harness-scripts.sh (NEW) + ci.yml s |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 283 | `recovery` | \| 2026-06-18 \| Epic G Tier-2/3 complete: TASK-0044 (hf-kernel cards) + TASK-0052 (harness_hub package) \| TASK-0044: `.handoff/tasks/TASK-*.task.json` (53 fleet-scoped cards via handoff-kernel-engineer) + `skills/forge-lo |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/AGENTS.md` | 100 | `retry` | (setsid, timeout, streaming, tee), but `runner.rs` must delegate `std::process::Command` |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/AGENTS.md` | 262 | `recovery` | the roadmap", "run unattended") use **`forge-loop`**; for **cross-session handoff/resume** ("transfer |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/AGENTS.md` | 263 | `recovery` | the session", "resume from handoff") use **`session-relay`** (checkpoints via `continuity-steward`, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/AGENTS.md` | 273 | `recovery` | "make .handoff tier-A", "resume handoff full-sync") use **`handoff-sync`** (Epic A; distinct from |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/AGENTS.md` | 274 | `recovery` | `session-relay`, which is the per-loop checkpoint). Simple questions and |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/Cargo.toml` | 44 | `recovery` | # --- OI-6 monotonic clock-rollback fence on the 24h relay window). All pure-Rust on Linux. --- |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/Cargo.toml` | 68 | `retry` | # Type=notify 90s timeout => restart storm). |
| ... |  |  | 156 more entries in JSON artifact |

## DLQs And Failure Paths

| file | line | signal | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 24 | `fail-closed` | \| 2026-06-18 \| forge-loop \| Destructive/irreversible automation must mirror the product's own invariants: dry-run-by-default, never `--force`, protect-and-skip (skip dirty, protect master/develop/current). This is also w |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 29 | `fail-closed` | \| 2026-06-23 \| forge-loop \| Don't dodge an infra limit — AUTHENTICATE with the infra the owner already built. When automation hits a quota/permission wall (GitHub API 403), the fix is to use the available credential, not |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 33 | `dead-letter` | \| 2026-06-23 \| feature-forge \| When two components must write the SAME runtime config file, each owns a delimited `# >>> name >>> … # <<< name <<<` block written by a NON-CLOBBERING upsert (strip-my-block, append-fresh)  |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 36 | `fail-closed` | \| 2026-06-27 \| plan-loop \| Phantom-workspace / sibling-escape environment hazard: a stray `Cargo.toml` in an ancestor dir makes Cargo walk UP and silently absorb the target into a foreign virtual workspace, breaking stan |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 142 | `fail-closed` | - Guards fail closed (`UuidResolves`/`NotLiveDevice`/`NotMounted` via blkid/findmnt; |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/AGENTS.md` | 488 | `fail-closed` | - **Organizational observability**: Anyone can see what's happening, what's blocked, what's done |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/AGENTS.md` | 759 | `fail-closed` | draft → backlog → active → blocked → completed |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/tasks/import-host-artifacts-yazelix-mission-control-20260703.md` | 49 | `dead-letter` | - flexnetos-quarantine-start.md — starts every FlexNetOS env trace at `_quarantine/20260630T234500Z....md`; names the authoritative source roots and warns off the generated-files trap; encodes the "do not dispatch agents |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/commits/019f2d99-0ab9-7940-9e86-7aa369ce278b.json` | 2 | `dead-letter` | "commit": {"author":"drdave <flexnetos@de-flex.net>","changes":[{"change_type":"modified","content_hash":"5f7c405cf62c29d8e6a6b0f0d4fd...","doc_type":"task","document_id":"019f2adb-59da-76c2-9164-1374...","hunks":[{"cont |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/kb-tasks/SKILL.md` | 10 | `fail-closed` | Optional filter: `active`, `draft`, `completed`, `blocked`, `all`, or a search term. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/kb-tasks/SKILL.md` | 22 | `fail-closed` | \| `blocked` \| `kb_list` with `type: "task"`, `status: "blocked"` \| |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/kb-tasks/SKILL.md` | 32 | `fail-closed` | - **Blocked by** (if the task has `blocked_by` relationships) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/kb-tasks/SKILL.md` | 45 | `fail-closed` | If any tasks are blocked, add a "Blocked Tasks" section explaining what's blocking each one. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/gitkb/SKILL.md` | 207 | `fail-closed` | `draft` → `backlog` → `active` → `blocked` → `completed` → `done` |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/kb-board/SKILL.md` | 46 | `fail-closed` | ### 2. Analyze Blocked Tasks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/kb-board/SKILL.md` | 48 | `fail-closed` | If any tasks are in the BLOCKED column (or have `blockedBy` relationships): |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/kb-board/SKILL.md` | 49 | `fail-closed` | - Use `kb_show` to load each blocked task |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/kb-board/SKILL.md` | 51 | `fail-closed` | - Summarize: "X is blocked by Y because Z" |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/kb-board/SKILL.md` | 69 | `fail-closed` | - Any blocked items with reasons |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/kb-create/SKILL.md` | 42 | `fail-closed` | 2. **Check the board:** use `kb_board`, or `git-kb board --json` if MCP is unavailable, to understand what's active, what's blocked, and where this new document fits in the current workstream. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/kb-create/SKILL.md` | 164 | `dead-letter` | [Other approaches and why they were rejected. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/kb-create/SKILL.md` | 210 | `fail-closed` | - If this task blocks or is blocked by another, mention it in the body with `[[wikilinks]]` |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/tests/relay.rs` | 707 | `fail-closed` | // Presenting the REMOTE bearer over the LOCAL egress path (req.remote == None) is DENIED |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/tests/relay.rs` | 712 | `fail-closed` | SwapOutcome::Denied(DenyReason::CrossKindPresentation) => {} |
| ... |  |  | 156 more entries in JSON artifact |

## Shared Protocol Context

- Protocol: `envctl_nu_plugin_migration_protocol` `1.0.0`
- Run event columns: `id, run_id, event_seq, event_type, phase, actor_type, actor_id, operation_id, payload_json, evidence_refs_json, previous_event_hash, event_hash, created_at_utc`
- Operation columns: `id, run_id, parent_operation_id, operation_type, phase, status, risk, idempotency_key, command_hash, command_redacted, input_json, output_ref, error_json, started_at_utc, completed_at_utc, created_at_utc`
- nu_plugin event command count: `12`

## Scan Limits

- Files scanned: `3000`
- Max files: `3000`
- Max file bytes: `700000`
- Truncated: `True`
- Skipped: `{"max_files_reached": 1, "too_large": 8, "unsupported_suffix": 399}`

## Validation

- Artifact registry persisted paths and content hashes for the canonical markdown, task markdown, and task JSON artifacts.
- Blocked path policy excluded `.env`, `secrets`, `private_keys`, `*.pem`, and `*.key` paths.
- Validation links include registry hash checks, shared protocol coverage, and required contract map sections.
