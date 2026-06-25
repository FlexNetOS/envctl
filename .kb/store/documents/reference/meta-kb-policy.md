---
id: 019f007d-bec7-76e3-9dec-63aae2a24022
slug: reference/meta-kb-policy
title: "Meta KB policy (inherited)"
type: reference
status: active
priority: medium
tags: [reference, meta, policy, kb]
---

Vendored summary of the **meta** knowledge-base policy that envctl inherits. Authoritative
source: `meta/.kb/AGENTS.md` (read it for the full text). This doc is the durable, in-repo
pointer so the inheritance survives clone/reclaim even without the parent meta checkout present.

## The policy envctl follows

- **Database-first KB with a git-like CLI** (`git-kb` / `git kb`). Context, tasks, and docs are
  **documents** in `.kb/`.
- **Session start = detect KB state** (PATH A/B/C):
  - PATH A (empty) → bootstrap the context documents.
  - PATH B (populated) → load + validate context before work.
  - PATH C (returning) → `git-kb status` + refresh `context/overridable/active`.
- **Context-document model** (the seven docs envctl maintains):
  - `context/immutable/{project-brief,patterns,architecture}` — core truths (change via ADR only).
  - `context/extensible/{product,tech}` — evolving.
  - `context/overridable/{active,progress}` — current state (update as work proceeds).
- **Document-before-implement**, link everything (`[[wikilinks]]`), complete the body before
  flipping status to done.

## Durability rule (envctl-enforced, see META-ORG-POLICY)

`.kb/store/` is **git-tracked TEXT** (source of truth); only `.kb/.cache/` (rebuildable index)
and ephemeral `workspaces/`/`stashes/` are `.gitignore`d. `git-kb init`'s default ignores the
whole store — that is overridden here so the KB is durable. After pulling tracked store changes,
`git-kb reindex` rebuilds the local index; `git-kb verify` checks file-store integrity.

## Relationship

meta is primary; envctl is meta's env-manager agent. envctl's KB adopts this policy locally and
syncs with meta's KB per `docs/kb-sync-runbook.md`. See also [[context/immutable/project-brief]],
[[reference/meta-org-policy]].