---
id: 019f8029-cffc-70c0-bfbe-e38ec4778467
slug: incidents/envctl-hook-profile-cleanup-20260720
title: "Envctl hook cleanup blocked by legacy Yazelix profile"
type: incident
status: investigating
priority: high
tags: [nix-profile, path, codex, claude, yazelix, runtime]
---

## Symptoms

The installed Yazelix foundation was cut over to the sole
`/home/flexnetos/.nix-profile` selector, but a login Nushell still inherited raw
store paths, a competing system Nix profile, and a retired home-local bin entry
from envctl's linked home projection. Codex and Claude therefore had a valid
profile frontdoor while `PATH` still exposed competing payload ownership.

## Impact

- `yazelix_profile_check` failed its `path_single_owner` clause.
- Raw store payloads could bypass the reviewed Codex/Claude frontdoors.
- The envctl home projection and its documentation contradicted the current
  owner-ratified single-profile contract.

## Investigation

The live file `/home/flexnetos/.config/nushell/meta-usr-path.nu` resolves into
`home/.config/nushell/meta-usr-path.nu` in envctl. Its sanitizer only removed
two narrow store patterns and still declared a retired home-local bin suffix.
The owning source, tests, gates, Claude/Codex projections, and repository
contracts must converge together; the generated host symlink must not be
edited directly.

## Resolution task

Tracked by [[tasks/strict-profile-path-owner-purge-20260721]].
