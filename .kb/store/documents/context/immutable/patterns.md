---
id: 019f007a-0a64-7c22-83ce-deb92c5bb789
slug: context/immutable/patterns
title: "System Patterns — envctl"
type: context
status: active
priority: medium
tags: [context, immutable]
---

System patterns and design decisions envctl follows. These are stable; change them only with
an ADR.

## Engine-centric pattern (the core decision)

The **engine is the single shared library** (`crates/engine/src/lib.rs`): synchronous, pure-Rust,
**non-printing** (emits `Event`s, never `println!`), no UI, no clap. The CLI (`envctl`) and GUI
(`envctl-gui`) are thin front-ends that drive the **identical** `Engine` API, so they cannot
diverge. **Put logic in the engine, not in `main.rs` or the GUI.**

## Declarative components

State is expressed as TOML **components** (`manifest/*.toml`; drop-ins in
`manifest/components.d/`) whose lifecycle hooks wrap the proven bash in `assets/scripts/`. The
manifest dir defaults to `./manifest` (`ENVCTL_MANIFEST_DIR` overrides). Verbs: `auto-detect`,
`install`, `auto-fix`, `reset`, `add-repo`, `graph`, `lock`, `doctor`.

## Fail-closed safety

Destructive ops are **dry-run by default**; mutation needs `--apply`/`--build`. Guards
(`UuidResolves`, `NotLiveDevice`, `NotMounted`) **refuse** when they can't prove safety
(unit-test enforced).

## Reproducible locks

`envctl.lock` (content-hashed manifest-of-record) and `agent-env.lock` capture reproducible
state; CI verifies no drift. The **agent-env** engine (absorbed kasetto, `crates/agent-env`)
provisions `.claude/`+`.codex/` from `agent-skills/` via `agent-env.yaml` → `agent-env.lock`.

## Continuity kernel

`.handoff/` (the `hf` kernel) is the durable continuity surface: a single fleet-resident ledger
(`$META_ROOT`, never a per-repo `ledger.db`), packets rendered from the witnessed ledger, a
`handoff.context_capsule.v1` capsule (`role`/`northstar`/`plane`/`next_command`), p7-conformance.
`.handoff` is git-tracked TEXT in full; only binary rebuild caches (`ledger.db`) are ignored.

## Secrets stack

Pure-Rust vault (`secrets-engine`) + tonic/prost gRPC (`secrets-proto`) + async tokio daemon
(`secretd`) + client (`secretctl`) + libSQL **remote** store. Keys auto-inject into child tools
(bearer / base-url-repoint / HTTPS_PROXY MITM); the real key never leaves the daemon.

## KB durability (this is policy, not preference)

The git-kb document store (`.kb/store/`) is **git-tracked TEXT** — the source of truth — so the
KB survives clone/reclaim. Only `.kb/.cache/` (rebuildable index) and ephemeral surfaces are
ignored. This deliberately overrides `git-kb init`'s tool default (which ignores the whole
store). Same principle as `.handoff`: track text, ignore binary rebuild caches.