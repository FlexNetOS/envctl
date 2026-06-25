---
id: 019f0078-83f2-7493-8d62-df3c7672af55
slug: context/immutable/project-brief
title: "Project Brief — envctl as meta env-manager agent"
type: context
status: active
priority: medium
tags: [context, immutable, meta-env, envctl]
---

## What envctl is

**envctl is the environment-manager agent of the `meta` workspace.** `meta` is primary;
envctl is secondary — a subordinate agent whose single job is to **own and converge the meta
environment**, never to operate as a standalone box tool and never to exclude meta.

Concretely, envctl owns the **meta environment boundary**: `PATH`, dotfiles, `~/.local`, the
canonical `home/` overlay tree (`envctl/home`, ADR-0006) that `~/.claude/{settings.json,
CLAUDE.md,RTK.md}` and `meta/settings.json` symlink into, the toolchain prefixes, and the
`META_ROOT` export. Every FlexNetOS tool / dotfile / `.local/bin` should resolve **inside
meta**; user-global holds only symlinks into meta. envctl also holds secrets and auto-injects
them into child tools on demand.

It happens to be implemented as a **pure-Rust Cargo workspace (8 crates)** that declaratively
manages a dual-RTX-5090 Ubuntu workstation — but that is *how/where it runs*, not its identity.
Its identity is "meta's env-manager agent."

## Governing policy (meta is first)

envctl follows **meta policy first**. The authoritative sources are, in order:
1. `meta/.kb/AGENTS.md` — the FlexNetOS knowledge-base policy (PATH A/B/C; context-doc model).
2. `meta/META-ORG-POLICY.md` — workspace org policy; envctl is a **Tier-B** member ("meta env
   manager"), registered in `meta/.meta.yaml` (`provides: [envctl]`, `tags: [tools, env]`).
3. envctl's own `CLAUDE.md` + `.handoff/context/capsule.json` (`role`/`northstar`) — which must
   agree with the above (docs-are-traps, P5.22).

## Non-negotiable invariants (regressions if broken)

- **No C in the trust boundary.** No SQLite/OpenSSL/aws-lc linked; store is libSQL `remote`
  only; crypto is pure-Rust (ring, blake3, chacha20poly1305, argon2). `ci/gates/no-c.sh` proves
  it fail-closed.
- **Exactly one rustls, ring-only** (not aws-lc-rs).
- **The engine is the single shared library** (`crates/engine`): sync, pure-Rust, non-printing
  (emits `Event`s), no UI/clap. CLI and GUI both drive the identical `Engine` API.
- **Destructive ops are fail-closed and dry-run by default.** Guards refuse when they can't
  prove safety; mutation needs `--apply`/`--build`.
- **Pure-Rust, no language drift.** Detect and reverse drift toward non-Rust toolchains.
- **The KB store is git-durable.** `.kb/store/` is tracked TEXT (source of truth); only
  `.kb/.cache/` + ephemeral surfaces are ignored. (Fixes the `git-kb init` tool default that
  swept the durable store into `.gitignore`.)