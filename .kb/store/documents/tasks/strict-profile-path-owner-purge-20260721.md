---
id: 019f8275-fd99-7412-a8ca-8ae9026f0481
slug: tasks/strict-profile-path-owner-purge-20260721
title: "Purge competing runtime ownership and home-local references"
type: task
status: active
priority: high
tags: [nix-profile, path, codex, claude, yazelix, runtime]
---

## Overview
Converge envctl source and its linked host projection on /home/flexnetos/.nix-profile as the sole installed runtime/PATH owner. Remove retired home-local and home-root Codex/Claude ownership references, reject raw store and competing profile entries, and preserve profile-managed volatile state under /run/user/1001/yazelix/profile-runtime.

## Goals
- Harden the Nushell PATH sanitizer and its owning tests/gates.
- Update every applicable maintained envctl contract and Claude/Codex projection.
- Prove the linked host projection, profile checks, GitKB/ICM health, and recovery behavior.
- Commit, push, merge with green CI, then clean the worktree/branch.

## Acceptance Criteria
- [ ] No active or maintained-source home-local ownership reference remains.
- [ ] No home-root Codex or Claude path remains an active/runtime authority.
- [ ] PATH contains profile frontdoors and no raw store or competing Nix-profile command owner.
- [ ] Repository gates and focused runtime tests pass.
- [ ] GitKB code index, impact, graph links, integrity, and completion receipts are current.
- [ ] PR is merged with required checks and temporary branch/worktree removed.

## Context
Resolves [[incidents/envctl-hook-profile-cleanup-20260720]]. Owner authority supersedes stale tracked guidance that names retired ownership paths.