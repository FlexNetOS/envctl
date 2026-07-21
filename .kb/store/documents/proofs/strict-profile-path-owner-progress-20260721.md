---
id: 019f832b-1bdd-7201-a391-2e1aaa2b7dd9
slug: proofs/strict-profile-path-owner-progress-20260721
title: "Strict-profile path-owner implementation progress receipt 2026-07-21"
type: reference
status: completed
priority: high
tags: [nix-profile, path, envctl, yazelix, gitkb, proof]
---

## Scope

Progress receipt for [[tasks/strict-profile-path-owner-purge-20260721]]. The active task remains version 1 because GitKB 0.2.12 cannot regenerate the prior document_version row after a version-2 update; this separate reference preserves a reproducible projection.

## Source Identity

- Repository: /home/flexnetos/meta/.worktrees/codex-strict-profile-path-owner/envctl
- Branch: codex/strict-profile-path-owner-20260721
- SSH remote: git@github.com:FlexNetOS/envctl.git
- Branch HEAD before implementation commit: 3c31f3cc1a22ba5704e06f33c32fb855b41de4b7
- Final profile cutover has not run and remains restricted to a clean merged canonical /home/flexnetos/meta/src/yazelix checkout with HEAD equal to origin/main.

## Implemented Scope

- Retired tracked home Codex and Claude projections while preserving the migration receipt.
- Reworked PATH, component, lifecycle, manifest, documentation, catalog, and tests so installed command ownership is profile-only and envctl is validation-only for Yazelix-owned commands.
- Added the strict-profile gate, profile command lifecycle, catalog proof exporter, focused tests, and the profile-path Nushell source.
- Merged Yazelix dependency PR 97 and fast-forwarded only canonical Yazelix to 599deda8869219ab8bbbcece0e247afe0988a900; no cutover was executed.

## Verification Evidence

- All envctl shell gates passed, including strict-profile-owner.
- Per-member Rust formatting, cargo check --workspace --all-targets, clippy with warnings denied, and cargo test --workspace passed.
- Catalog projection snapshot: c3b6317394878147c86e987c84b201a022902cc4e46f350aa6ab610323e70a4c.
- Branch-scoped GitKB index: 6,994 symbols, 1,787 files, 18,876 resolved call edges, 70,105 call sites, and zero stale files.
- GitKB canonical projection: 23 documents, 17 commits, verify --full clean, fsck clean before this receipt.

## GitKB Limitation

A fresh projection rebuild after modifying an existing document fails with missing_version_rows=1 in GitKB 0.2.12 even though canonical lineage inspection is valid. The failure was reproduced in isolated copies. This receipt is created as a new version-1 document so repair projection remains reproducible without rewriting historical proof records.

## Code References

- [[code:crates/agent-env/src/runtime_contract.rs::function::validate_runtime_contract]]
- [[code:crates/engine/src/secrets.rs::function::resolve_secretctl]]
- [[code:crates/engine/src/runner.rs::function::enforced_meta_env]]
- [[code:crates/engine/src/catalog.rs::function::infer_file_kind]]