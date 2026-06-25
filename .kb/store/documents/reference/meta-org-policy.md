---
id: 019f007d-bed9-7373-86e7-34a622a7849d
slug: reference/meta-org-policy
title: "Meta org policy — envctl Tier-B obligations (inherited)"
type: reference
status: active
priority: medium
tags: [reference, meta, policy, org]
---

Vendored summary of envctl's obligations under **meta** org policy. Authoritative source:
`meta/META-ORG-POLICY.md` (POLICY v2) + `meta/ARCHITECTURE-TRUTH.md`. Durable in-repo pointer.

## envctl's classification

- **Tier B** ("meta env manager"), registered in `meta/.meta.yaml`
  (`provides: [envctl]`, `tags: [tools, env]`). Standalone semver (`0.1.0`), full P1–P7.
- Role (ARCHITECTURE-TRUTH): *meta env manager — env injection, secretd, agent-env seam, USB
  secret vault*. It **owns the meta environment boundary** (PATH, dotfiles, `~/.local`, the
  `home/` overlay, `META_ROOT`). meta is primary; envctl does not exclude it.

## Membership surfaces envctl must keep valid

- `.meta.yaml` entry + parent `.gitignore` (registered + child-ignored). ✓
- Standalone version strategy in `Cargo.toml`.
- CI gates green (`ci/gates/*`), preflight subset locally.
- `.handoff/` Tier-A continuity (capsule `role`/`northstar`/`plane`/`next_command`; p7-conformance).
- **`.kb/` knowledge base (P5.23):** REQUIRED for standalone-workable Tier-B repos. envctl
  maintains its own KB.

## The `.kb/store` durability rule (this change adds it to META-ORG-POLICY)

`.kb/store/` is git-tracked TEXT (the durable source of truth); only `.kb/.cache/` +
workspaces/worktrees/stashes are ignored. `git-kb init`'s tool default (ignore the whole store)
MUST be corrected in every member — otherwise the KB is non-durable (nothing committed/pushed;
docs lost on clone). A conformance check fails any member that ignores `.kb/store/`.

See [[reference/meta-kb-policy]], [[context/immutable/project-brief]].