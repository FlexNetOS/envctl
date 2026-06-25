---
id: 019f007a-0aaa-78a3-8dc1-b40f7ede9f80
slug: context/overridable/active
title: "Active Context — envctl"
type: context
status: active
priority: medium
tags: [context, overridable]
---

Current focus (seeded 2026-06-25). Update when focus changes.

## In flight

**Register envctl as meta's env-manager agent + make the KB git-durable** (branch
`kb-durable-meta-register`). Triggered by the owner directive: *"envctl is a peer repo in meta;
meta policy is first; follow the proper register path; adopt meta policy (meta/.kb/AGENTS.md);
manage the meta env, do not exclude it."*

Work in this change:
- **Part A** — reframe `CLAUDE.md` + `README.md` so envctl reads as meta's subordinate
  env-manager agent (meta primary), consistent with the `.handoff` capsule northstar.
- **Part B (core fix)** — corrected `.gitignore` so `.kb/store/` is git-tracked (root cause: the
  `git-kb init` tool default ignored the durable store → KB was non-durable); seeded these 7
  context documents as tracked files.
- **Part C** — vendor the meta policy/KB-convention docs envctl inherits as tracked files;
  `docs/kb-sync-runbook.md` documents the local cross-KB sync (backup→pull→diff→reconcile).
- **Meta change** (separate repo) — fix `meta/.kb/.gitignore` (track meta's store) +
  `META-ORG-POLICY.md` workspace rule + conformance check.

## Next steps

- Land the envctl PR (Parts A/B/C) and the coordinated meta change.
- On the box: `git-kb reindex` to absorb the tracked store; wire the `meta` sync remote per the
  runbook.

## Provenance note (why this was needed)

The store-ignore came from `git-kb init`'s tool default, committed into meta by a Claude session
(`a9d4b93548`, #31, Co-Authored-By Claude) and dropped into envctl uncommitted today. It is a
tool default, not a deliberate "KB is non-durable" decision.