# Event Message Contract Map

Task: `ART-111_EVENT_MAP`
Generated at: `2026-07-04T23:29:32+00:00`
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
| `LOCAL_WORKAROUNDS.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `WORKLOG.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/scripts/workbench_db.py` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/envctl/.codex/plugins/cache/meta-plugins-codex/codex-security/0.1.10/skills/deep-security-scan/SKILL.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/envctl/crates/agent-env/tests/parity_vs_kasetto.rs` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/envctl/crates/cli/src/main.rs` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/envctl/crates/secretd/src/edge/authorizer.rs` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/teri/.handoff/loop/findings/parity.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/teri/.handoff/loop/findings/u026-architecture.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/teri/.handoff/loop/findings/u026-d-architecture.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/teri/.handoff/loop/findings/u028-architecture.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/teri/.handoff/loop/loop_state.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/teri/.handoff/loop/merge-ledger.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/teri/.handoff/loop/symbol-map.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/teri/.handoff/loop/target-architecture.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/teri/.worktrees/issue-86-source-wires/.handoff/loop/findings/parity.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/teri/.worktrees/issue-86-source-wires/.handoff/loop/findings/u026-architecture.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/teri/.worktrees/issue-86-source-wires/.handoff/loop/findings/u026-d-architecture.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/teri/.worktrees/issue-86-source-wires/.handoff/loop/findings/u028-architecture.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/teri/.worktrees/issue-86-source-wires/.handoff/loop/loop_state.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/teri/.worktrees/issue-86-source-wires/.handoff/loop/merge-ledger.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/teri/.worktrees/issue-86-source-wires/.handoff/loop/symbol-map.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/teri/.worktrees/issue-86-source-wires/.handoff/loop/target-architecture.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/cluaiz/README.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/cluaiz/docs/reference/api.md` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/inferrs/inferrs/src/server.rs` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/teri/.worktrees/issue-86-source-wires/frontend/locales/en.json` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/teri/.worktrees/issue-86-source-wires/src/agent/mod.rs` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/teri/.worktrees/issue-86-source-wires/src/api/graph.rs` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |
| `src/teri/.worktrees/issue-86-source-wires/src/i18n/locales/en.json` | 11 | consumers, dlqs, payloads, producers, queues, retries, topics |

## Topics And Streams

| file | line | signal | evidence |
|---|---:|---|---|
| `LOCAL_WORKAROUNDS.md` | 43 | `phase-channel` | ## Yazelix Codex Status-Bar Live Config Repair - 2026-07-03 |
| `LOCAL_WORKAROUNDS.md` | 74 | `phase-channel` | /home/flexnetos/.nix-profile/bin/yzx status -> Generated runtime state up to date, repair needed no |
| `LOCAL_WORKAROUNDS.md` | 79 | `phase-channel` | window restart to consume the regenerated status-bar command. Its active session |
| `LOCAL_WORKAROUNDS.md` | 324 | `phase-channel` | clean-env yzx status --json runtime_dir -> /nix/store/p45lnz6nsvjzvhjlb... |
| `LOCAL_WORKAROUNDS.md` | 325 | `phase-channel` | clean-env yzx status --json generated_state_materializat... -> noop |
| `LOCAL_WORKAROUNDS.md` | 379 | `phase-channel` | PATH=/home/flexnetos/FlexNetOS/us...:$PATH meta exec -- /home/flexnetos/FlexNetOS/us... status --json -> 17 commands complete |
| `LOCAL_WORKAROUNDS.md` | 504 | `phase-channel` | clean-env yzx status --json runtime_dir -> /nix/store/hk4m00bfnqddhlymf... |
| `LOCAL_WORKAROUNDS.md` | 505 | `phase-channel` | clean-env yzx status --json generated_state_materializat... -> noop |
| `LOCAL_WORKAROUNDS.md` | 668 | `phase-channel` | src/meta git-kb status --json -> clean at 019f1f89-ddf0-72a2-89c1-5147... |
| `LOCAL_WORKAROUNDS.md` | 712 | `phase-channel` | git-kb verify/status # from src/yazelix and src/meta |
| `LOCAL_WORKAROUNDS.md` | 714 | `phase-channel` | meta exec -- git-kb verify/status # from src/meta, across the project set |
| `LOCAL_WORKAROUNDS.md` | 727 | `phase-channel` | health, and `meta exec -- git-kb verify/status` passing across all 16 projects. |
| `LOCAL_WORKAROUNDS.md` | 729 | `phase-channel` | Hook self-validation on 2026-07-01 passed for both `session-start` and `stop` |
| `LOCAL_WORKAROUNDS.md` | 918 | `phase-channel` | rendered bottom bar: native zellij:status-bar |
| `LOCAL_WORKAROUNDS.md` | 937 | `phase-channel` | top status bar: generated Yazelix zjstatus bar |
| `LOCAL_WORKAROUNDS.md` | 938 | `phase-channel` | bottom status bar: native zellij:status-bar |
| `LOCAL_WORKAROUNDS.md` | 984 | `phase-channel` | `yzx status --versions` from `/home/flexnetos/.nix-profile/bin/yzx` reports an |
| `LOCAL_WORKAROUNDS.md` | 998 | `phase-channel` | Use Yazelix status, its Nix profile, or the Yazelix dev shell for package |
| `LOCAL_WORKAROUNDS.md` | 1211 | `phase-channel` | It loads Yazelix-managed plugins including `sidebar-status`, `auto-layout`, |
| `WORKLOG.md` | 47 | `phase-channel` | ## 2026-07-03 - Yazelix Codex Status-Bar Live Repair |
| `WORKLOG.md` | 49 | `phase-channel` | Archived live Yazelix host state, then repaired the active Codex status-bar |
| `WORKLOG.md` | 59 | `phase-channel` | `yzx_side.kdl` status widget commands use `--display both --periods 5h,week`, |
| `WORKLOG.md` | 60 | `phase-channel` | and both `yzx status` and `yzx doctor` report healthy generated runtime state. |
| `WORKLOG.md` | 76 | `phase-channel` | regenerated layout and must restart before its in-memory status plugin consumes |
| ... |  |  | 156 more entries in JSON artifact |

## Queues

| file | line | signal | evidence |
|---|---:|---|---|
| `LOCAL_WORKAROUNDS.md` | 679 | `job-queue` | Known caveat: `git-kb daemon start --background` reaches readiness but does not |
| `WORKLOG.md` | 126 | `operation-queue` | `18 ok · 1 notes · 0 warn · 0 fail`. Rust format/test validation is pending |
| `WORKLOG.md` | 2020 | `job-queue` | unless `--background` is passed. `daemon start --background` reaches readiness |
| `WORKLOG.md` | 2045 | `job-queue` | incidents/gitkb-daemon-backg... |
| `.kb/AGENTS.md` | 269 | `operation-queue` | git-kb status --json # Check for pending changes |
| `.kb/skills/kb-status/SKILL.md` | 3 | `operation-queue` | description: Show workspace status and pending changes |
| `.kb/skills/kb-status/agents/openai.yaml` | 4 | `operation-queue` | default_prompt: "Use $kb-status to show workspace status and pending changes." |
| `.kb/skills/gitkb/SKILL.md` | 40 | `operation-queue` | git-kb status --json # Show pending changes |
| `.kb/skills/gitkb/SKILL.md` | 106 | `operation-queue` | \| `git-kb status --json` \| `kb_status` \| Show pending changes \| |
| `.kb/skills/kb-context/SKILL.md` | 53 | `operation-queue` | 1. Check `kb_status` for pending changes |
| `.kb/skills/kb-handoff/SKILL.md` | 3 | `operation-queue` | description: End-of-session handoff — update context, log progress, commit pending changes |
| `.kb/skills/kb-handoff/SKILL.md` | 14 | `operation-queue` | ### 1. Commit Pending Changes |
| `.kb/skills/kb-handoff/SKILL.md` | 71 | `operation-queue` | - Pending work: [what's next] |
| `.kb/skills/kb-commit/SKILL.md` | 6 | `operation-queue` | Review, validate, and commit pending workspace changes. |
| `.kb/skills/kb-commit/SKILL.md` | 46 | `operation-queue` | > "kb_status shows changes to `<slug>` which I didn't modify. Excluding it from this commit. Another agent may have pending changes." |
| `.kb/skills/kb-commit/agents/openai.yaml` | 4 | `operation-queue` | default_prompt: "Use $kb-commit to review, validate, and commit pending workspace changes." |
| `src/teri/SPRINT.md` | 58 | `operation-queue` | `TODO.md` is dated 2026-06-12 and still says "pipeline pending" (false). Rewrite it to point at |
| `src/teri/FEATURE-PARITY.md` | 254 | `job-queue` | unit tests (debounce / one-job / budget-cap / persist-reload continuity / per-domain isolation / |
| `src/teri/FEATURE-PARITY.md` | 353 | `operation-queue` | - ☑ **TASK-DOC-1** — refreshed the stale `TODO.md` (was dated 2026-06-12, claimed "pipeline pending"): |
| `src/teri/README.md` | 160 | `job-queue` | \| `LLM_MAX_CONCURRENT_REQUESTS` \| `1` for local inferrs, otherwise unset \| In-flight OpenAI-compatible request cap; prevents local CUDA queue timeouts \| |
| `src/teri/RUNBOOK.md` | 6 | `operation-queue` | > "Known gaps" section marks what is wired vs. pending so you never operate on a false premise. |
| `src/teri/RUNBOOK.md` | 151 | `job-queue` | \| `LLM_MAX_CONCURRENT_REQUESTS` \| `1` for local inferrs, otherwise unset \| In-flight OpenAI-compatible request cap; prevents local CUDA queue timeouts \| |
| `src/teri/tests/social_world_e2e_test.rs` | 117 | `job-queue` | background: "bg".to_string(), |
| `src/teri/tests/codex_harness.rs` | 158 | `operation-queue` | "wiring pending", |
| ... |  |  | 156 more entries in JSON artifact |

## Payloads And Schemas

| file | line | signal | evidence |
|---|---:|---|---|
| `LOCAL_WORKAROUNDS.md` | 26 | `hash-chain` | sha256: 8eac967312222bad4805b929db29... |
| `LOCAL_WORKAROUNDS.md` | 115 | `hash-chain` | sha256: 8884e5a881cfb5a4e028a37cec13... |
| `LOCAL_WORKAROUNDS.md` | 137 | `hash-chain` | sha256: b7b5d0a6b38f6e53713df85dc181... |
| `LOCAL_WORKAROUNDS.md` | 183 | `hash-chain` | sha256: 1f39f9fdf1eec4b830eca1ea86e5... |
| `LOCAL_WORKAROUNDS.md` | 185 | `hash-chain` | sha256: d1e088fca402c482e6fe0c3f27b0... |
| `LOCAL_WORKAROUNDS.md` | 187 | `hash-chain` | sha256: fcfd97c24468a5d6bd17e8d6793e... |
| `LOCAL_WORKAROUNDS.md` | 299 | `hash-chain` | sha256: 4648832c3718caf39aba99a8d4fe... |
| `LOCAL_WORKAROUNDS.md` | 397 | `hash-chain` | Only its stored commit `content_hash` was changed, to match the full hash |
| `LOCAL_WORKAROUNDS.md` | 467 | `json-payload` | The installed GitKB Codex plugin payload has a portable MCP declaration using |
| `LOCAL_WORKAROUNDS.md` | 721 | `hash-chain` | sha256: f649d213c082f8eb86413b01495b... |
| `LOCAL_WORKAROUNDS.md` | 768 | `hash-chain` | sha256: 1c25ffb663137d0fba4a2fd083e7... |
| `LOCAL_WORKAROUNDS.md` | 798 | `hash-chain` | sha256: 81d7be43085e3689915fc70ef3e7... |
| `LOCAL_WORKAROUNDS.md` | 803 | `hash-chain` | sha256: d3de843759bc1d9f31e0d5cce90e... |
| `LOCAL_WORKAROUNDS.md` | 813 | `hash-chain` | sha256: e0e6b5a183ea09a30489149972d8... |
| `LOCAL_WORKAROUNDS.md` | 838 | `hash-chain` | isolated install proof sha256: 1bf8f56b6afaa06ae2c7b2deac94... |
| `LOCAL_WORKAROUNDS.md` | 852 | `hash-chain` | sha256: ca2735e05b4bffefec8de2b46119... |
| `LOCAL_WORKAROUNDS.md` | 914 | `hash-chain` | source layout sha256: 082556812a8b893c9b862178958b... |
| `LOCAL_WORKAROUNDS.md` | 916 | `hash-chain` | proof log sha256: 375563e5104f0cc57e36ff6f60aa... |
| `LOCAL_WORKAROUNDS.md` | 935 | `hash-chain` | sha256: 082556812a8b893c9b862178958b... |
| `LOCAL_WORKAROUNDS.md` | 1046 | `hash-chain` | profile upgrade sha256: b815e894e4926aeeffcaf1739cc5... |
| `LOCAL_WORKAROUNDS.md` | 1048 | `hash-chain` | direct launch sha256: 495e0f1efff8f060ce106ad7b301... |
| `LOCAL_WORKAROUNDS.md` | 1050 | `hash-chain` | plugin proof sha256: 50b1e99611ae809bf6151faa9b79... |
| `LOCAL_WORKAROUNDS.md` | 1169 | `json-payload` | package was changed to ship an empty hook payload. This is host/session control |
| `LOCAL_WORKAROUNDS.md` | 1178 | `json-payload` | Archive of the pre-change hook payload: |
| ... |  |  | 156 more entries in JSON artifact |

## Producers

| file | line | signal | evidence |
|---|---:|---|---|
| `LOCAL_WORKAROUNDS.md` | 865 | `producer` | upstream-runtime Mars/Zellij session and write a non-empty launch log. That |
| `LOCAL_WORKAROUNDS.md` | 898 | `producer` | nix build .#runtime_mars --no-link --no-write-lock-file -> /nix/store/88fhkzjap6pg64zwl... |
| `LOCAL_WORKAROUNDS.md` | 1234 | `producer` | alive under the Codex exec wrapper after the producer exits. Confirmed with a |
| `WORKLOG.md` | 213 | `producer` | edits, build the canonical release bundle, write provenance and checksums, then |
| `WORKLOG.md` | 624 | `producer` | nix build .#runtime_mars --no-link --no-write-lock-file --print-out-paths -> /nix/store/65vi6r5brsxmdv38h... |
| `WORKLOG.md` | 625 | `producer` | nix build .#yazelix .#yazelix_mars --no-link --no-write-lock-file --print-out-paths -> /nix/store/sm87nrpjl89i9l8n7... |
| `WORKLOG.md` | 947 | `producer` | nix build .#runtime_mars --no-link --no-write-lock-file -> /nix/store/88fhkzjap6pg64zwl... |
| `WORKLOG.md` | 990 | `producer` | nix build .#runtime_mars --no-link --no-write-lock-file |
| `WORKLOG.md` | 1714 | `producer` | reviewed GitKB files in the repo, then publish with `git push` when appropriate. |
| `WORKLOG.md` | 2134 | `producer` | Token scopes: admin:public_key, gist, repo, write:org |
| `WORKLOG.md` | 2137 | `producer` | `write:org` covers organization write authorization, while the SSH key surface |
| `WORKLOG.md` | 2310 | `producer` | exec wrapper after their producer exits. Reproduced with `find ... \| rg ...`; |
| `.kb/AGENTS.md` | 67 | `mutation-command` | \| Context already loaded this session \| **PATH C**: Quick resume \| |
| `.kb/AGENTS.md` | 264 | `mutation-command` | ## PATH C: Returning Agent (Quick Resume) |
| `.kb/AGENTS.md` | 273 | `mutation-command` | If workspace is clean and context is still valid, resume work. |
| `.kb/AGENTS.md` | 786 | `mutation-command` | - **Continue**: Resume where you left off |
| `.kb/skills/kb-context/SKILL.md` | 51 | `mutation-command` | **If context was already loaded this session (PATH C — Quick Resume):** |
| `.kb/skills/kb-context/SKILL.md` | 55 | `mutation-command` | 3. Resume work |
| `.kb/skills/kb-handoff/SKILL.md` | 33 | `producer` | If the user provided a session summary in `$ARGUMENTS`, use it to write better progress entries. |
| `.kb/skills/kb-handoff/SKILL.md` | 76 | `mutation-command` | - Then: /kb-start <suggested-task> to resume work |
| `.kb/skills/kb-create/SKILL.md` | 56 | `producer` | If the description mentions specific code (files, functions, modules), use `kb_symbols` or `kb_impact` to understand the relevant code and write more precise goals and acceptance criteria. |
| `.kb/skills/kb-create/SKILL.md` | 88 | `producer` | Build the document body based on type. Write **substantive, project-aware content** — not placeholder text. Use what you learned from discovery and context loading to write content that a cold-starting agent could pick u |
| `src/teri/SPRINT.md` | 39 | `producer` | \| S8 \| Social-DB producer + `sqlite`-default serve \| TASK-SIM-3 \| high \| cargo test \| |
| `src/teri/SPRINT.md` | 103 | `producer` | ### S8 · TASK-SIM-3 — social-DB producer + sqlite-default serve |
| ... |  |  | 156 more entries in JSON artifact |

## Consumers

| file | line | signal | evidence |
|---|---:|---|---|
| `LOCAL_WORKAROUNDS.md` | 208 | `consumer` | consumer is the default Nix profile entry: |
| `LOCAL_WORKAROUNDS.md` | 235 | `consumer` | codex exec --ephemeral --skip-git-repo-check -s read-only -> CLEAN_ENV_CODEX_EXEC_OK |
| `LOCAL_WORKAROUNDS.md` | 433 | `consumer` | Codex plugin marketplace/plugin installation state is currently read from the |
| `LOCAL_WORKAROUNDS.md` | 991 | `nu-plugin` | nushell 0.112.2 |
| `LOCAL_WORKAROUNDS.md` | 1005 | `consumer` | consumer-side artifact only. It must not be used as proof that the source package |
| `WORKLOG.md` | 287 | `consumer` | /nix/store/q4yikx94871lmcfxg... exec --ephemeral --skip-git-repo-check -s read-only 'print exactly FOUNDATION_CODEX_EXEC_CROSS_...' |
| `WORKLOG.md` | 305 | `consumer` | Codex app-server consumer state. The active branch |
| `WORKLOG.md` | 402 | `consumer` | ./result/bin/codex exec --ephemeral --skip-git-repo-check -s read-only -> CODEX_EXEC_FULL_RUNTIME_OK |
| `WORKLOG.md` | 426 | `consumer` | Installed consumer proof: |
| `WORKLOG.md` | 878 | `consumer` | read from the user Codex config under `/home/flexnetos/.codex/config.toml`, not |
| `WORKLOG.md` | 1090 | `consumer` | generations, and desktop entries are consumer surfaces. Build and validate source |
| `WORKLOG.md` | 1257 | `nu-plugin` | Installed the pinned GitKB release binary through the Yazelix runtime/Nushell |
| `WORKLOG.md` | 1268 | `nu-plugin` | was completed through the same Yazelix/Nushell route by downloading the pinned |
| `WORKLOG.md` | 1414 | `nu-plugin` | Installed upstream `gitkb/meta` release `v0.2.22` through the Yazelix/Nushell |
| `AGENTS.md` | 3 | `consumer` | This is the first file to read when working under `/home/flexnetos/FlexNetOS`. |
| `AGENTS.md` | 39 | `consumer` | 1. Read this file, `WORKSPACE_LAYOUT.md`, `WORKLOG.md`, and |
| `AGENTS.md` | 41 | `consumer` | 2. Read the target repo's `AGENTS.md`, `README.md`, and relevant subsystem docs. |
| `AGENTS.md` | 62 | `consumer` | Read `LOCAL_WORKAROUNDS.md` before running `yzx desktop install`, editing |
| `.kb/AGENTS.md` | 97 | `consumer` | Treat `.kb/store/**` as read-only implementation state. Never edit it directly. |
| `.kb/AGENTS.md` | 221 | `consumer` | 1. [ ] AGENTS.md fully read |
| `.kb/AGENTS.md` | 226 | `consumer` | 3. [ ] All required context read: |
| `.kb/AGENTS.md` | 241 | `consumer` | Lock --> Read[Read AGENTS.md] |
| `.kb/AGENTS.md` | 242 | `consumer` | Read --> LoadContext[Load Context via GitKB] |
| `.kb/AGENTS.md` | 630 | `consumer` | 2. **Read active context**: Understand current focus |
| ... |  |  | 156 more entries in JSON artifact |

## Retries And Replay

| file | line | signal | evidence |
|---|---:|---|---|
| `LOCAL_WORKAROUNDS.md` | 1241 | `retry` | FILE`), `find -exec`, or wrap probes with `timeout` when a pipe is unavoidable. |
| `WORKLOG.md` | 1244 | `retry` | Idempotency proof: a second `envctl install node-via-bun` skipped both `bun` |
| `WORKLOG.md` | 1270 | `retry` | timeouts/retries, verifying `sha256sum -c`, and installing the extracted |
| `WORKLOG.md` | 1736 | `retry` | Later retry: |
| `WORKLOG.md` | 2314 | `retry` | or bounded `timeout` commands. |
| `.kb/AGENTS.md` | 67 | `recovery` | \| Context already loaded this session \| **PATH C**: Quick resume \| |
| `.kb/AGENTS.md` | 264 | `recovery` | ## PATH C: Returning Agent (Quick Resume) |
| `.kb/AGENTS.md` | 273 | `recovery` | If workspace is clean and context is still valid, resume work. |
| `.kb/AGENTS.md` | 453 | `retry` | git commit -m "fix: resolve auth timeout issue |
| `.kb/AGENTS.md` | 720 | `retry` | fix: resolve auth timeout issue |
| `.kb/AGENTS.md` | 728 | `retry` | resolves: incidents/inc-009-auth-timeout |
| `.kb/AGENTS.md` | 786 | `recovery` | - **Continue**: Resume where you left off |
| `.kb/skills/gitkb/SKILL.md` | 180 | `retry` | fix: resolve auth timeout issue |
| `.kb/skills/gitkb/SKILL.md` | 214 | `retry` | \| Incident \| `incidents/inc-{NNN}-{slug}` \| `incidents/inc-001-auth-timeout` \| |
| `.kb/skills/kb-context/SKILL.md` | 51 | `recovery` | **If context was already loaded this session (PATH C — Quick Resume):** |
| `.kb/skills/kb-context/SKILL.md` | 55 | `recovery` | 3. Resume work |
| `.kb/skills/kb-handoff/SKILL.md` | 76 | `recovery` | - Then: /kb-start <suggested-task> to resume work |
| `.kb/skills/kb-create/SKILL.md` | 62 | `retry` | - For **numbered types** (task, epic, incident): complete when the last path segment ends with a digit (e.g. `tasks/my-project-28`, `incidents/inc-001-auth-timeout`). |
| `src/teri/SPRINT.md` | 19 | `recovery` | - Witness milestones (`/checkpoint`), close segments (`/handoff`). |
| `src/teri/SPRINT.md` | 126 | `recovery` | `pipeline::run_pipeline` runs under a compute budget; continuity/resume + witnessed audit trail. |
| `src/teri/FEATURE-PARITY.md` | 95 | `retry` | `_wait_for_episodes`/`generate_python_code`/pagination/backoff — Zep-server artifacts inapplicable to |
| `src/teri/FEATURE-PARITY.md` | 173 | `retry` | - `HistoryDatabase.vue` (history card gallery + replay modal), |
| `src/teri/FEATURE-PARITY.md` | 250 | `recovery` | `JsonFileStateStore` atomic-write checkpoint for prod) — a restart on an unchanged signal does not |
| `src/teri/FEATURE-PARITY.md` | 255 | `retry` | retry / policy-query / report-shape / budget normalization). **Left for S13 (TASK-AUTO-2):** the L3 |
| ... |  |  | 156 more entries in JSON artifact |

## DLQs And Failure Paths

| file | line | signal | evidence |
|---|---:|---|---|
| `LOCAL_WORKAROUNDS.md` | 384 | `fail-closed` | The Codex stop gate denied shutdown after Meta GitKB verification failed. Before |
| `LOCAL_WORKAROUNDS.md` | 707 | `fail-closed` | Session start and stop now fail closed unless the foundation context is |
| `WORKLOG.md` | 104 | `dead-letter` | is correct for writable directories, but invalid for writable files such as |
| `WORKLOG.md` | 106 | `dead-letter` | `config.toml/.codex`, and cleanup later panicked while inspecting the invalid |
| `WORKLOG.md` | 495 | `dead-letter` | `codex-security` payload was left intact after the scaffold validator rejected a |
| `WORKLOG.md` | 691 | `fail-closed` | Recovered the Codex stop hook after the runtime gate denied shutdown with |
| `WORKLOG.md` | 846 | `fail-closed` | against those repos are denied unless the same operation first creates a |
| `WORKLOG.md` | 2106 | `fail-closed` | Session start and stop now fail closed unless `br ready`, `git-kb |
| `WORKLOG.md` | 2367 | `fail-closed` | Rust format/test proof was blocked in the active shell because `cargo`, |
| `.kb/AGENTS.md` | 486 | `fail-closed` | - **Organizational observability**: Anyone can see what's happening, what's blocked, what's done |
| `.kb/AGENTS.md` | 757 | `fail-closed` | draft → backlog → active → blocked → completed |
| `.kb/skills/kb-tasks/SKILL.md` | 10 | `fail-closed` | Optional filter: `active`, `draft`, `completed`, `blocked`, `all`, or a search term. |
| `.kb/skills/kb-tasks/SKILL.md` | 22 | `fail-closed` | \| `blocked` \| `kb_list` with `type: "task"`, `status: "blocked"` \| |
| `.kb/skills/kb-tasks/SKILL.md` | 32 | `fail-closed` | - **Blocked by** (if the task has `blocked_by` relationships) |
| `.kb/skills/kb-tasks/SKILL.md` | 45 | `fail-closed` | If any tasks are blocked, add a "Blocked Tasks" section explaining what's blocking each one. |
| `.kb/skills/gitkb/SKILL.md` | 207 | `fail-closed` | `draft` → `backlog` → `active` → `blocked` → `completed` → `done` |
| `.kb/skills/kb-board/SKILL.md` | 46 | `fail-closed` | ### 2. Analyze Blocked Tasks |
| `.kb/skills/kb-board/SKILL.md` | 48 | `fail-closed` | If any tasks are in the BLOCKED column (or have `blockedBy` relationships): |
| `.kb/skills/kb-board/SKILL.md` | 49 | `fail-closed` | - Use `kb_show` to load each blocked task |
| `.kb/skills/kb-board/SKILL.md` | 51 | `fail-closed` | - Summarize: "X is blocked by Y because Z" |
| `.kb/skills/kb-board/SKILL.md` | 69 | `fail-closed` | - Any blocked items with reasons |
| `.kb/skills/kb-create/SKILL.md` | 42 | `fail-closed` | 2. **Check the board:** use `kb_board`, or `git-kb board --json` if MCP is unavailable, to understand what's active, what's blocked, and where this new document fits in the current workstream. |
| `.kb/skills/kb-create/SKILL.md` | 164 | `dead-letter` | [Other approaches and why they were rejected. |
| `.kb/skills/kb-create/SKILL.md` | 210 | `fail-closed` | - If this task blocks or is blocked by another, mention it in the body with `[[wikilinks]]` |
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
- Skipped: `{"max_files_reached": 1, "too_large": 8, "unsupported_suffix": 644}`

## Validation

- Artifact registry persisted paths and content hashes for the canonical markdown, task markdown, and task JSON artifacts.
- Blocked path policy excluded `.env`, `secrets`, `private_keys`, `*.pem`, and `*.key` paths.
- Validation links include registry hash checks, shared protocol coverage, and required contract map sections.
