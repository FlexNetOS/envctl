---
id: 019f007a-0a88-7160-bdf7-150a7cea8744
slug: context/extensible/product
title: "Product Context — envctl"
type: context
status: active
priority: medium
tags: [context, extensible]
---

Why envctl exists and who depends on it.

## Problem

The dual-RTX-5090 Ubuntu workstation that the **entire `meta` fleet** runs on must be brought to
a declared, reproducible, drift-free state — toolchains, `PATH`, dotfiles, `~/.local`, secrets,
agent config — **without system-depth drift** (no unmanaged apt/`/usr/local`/global installs;
converge each to a meta-owned `.toolchains` prefix or declare an irreducible `system:` exception).

## Who it's for

- The **owner**, who declares desired state once and converges to it.
- **Every agent / loop in the meta workspace** — they depend on envctl having provisioned the
  environment they run inside. envctl is meta's env-manager *agent*: it serves meta, it does not
  stand apart from it.

## Product principles

1. **Meta is primary.** envctl converges *the meta environment*; it never excludes or forks away
   from meta.
2. **Declarative + reproducible.** Desired state is TOML components + locks, not ad-hoc scripts.
3. **Fail-closed.** Preview by default; mutation is explicit.
4. **Pure-Rust native.** No language/toolchain drift; port foreign tools into Rust crates.
5. **Durable memory.** Knowledge (this KB), continuity (`.handoff`), and locks are git-tracked so
   they survive clone/reclaim.