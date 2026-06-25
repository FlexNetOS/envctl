---
id: 019f007a-0a76-7f83-b728-301d525f402c
slug: context/immutable/architecture
title: "Architecture — envctl (8 crates + secrets)"
type: context
status: active
priority: medium
tags: [context, immutable]
---

Structure of the envctl workspace and how it owns the meta environment.

## Two halves, one engine

envctl is a pure-Rust Cargo workspace of **8 crates**, split into two halves that share one
engine:

**env-manager**
- `engine` — the shared, sync, non-printing `Engine` library (all logic lives here).
- `cli` (`envctl`) — thin clap front-end over `Engine`.
- `gui` (`envctl-gui`) — thin native GUI over the same `Engine`.

**secrets stack**
- `secrets-engine` — pure-Rust crypto vault (ring, blake3, chacha20poly1305, argon2).
- `secrets-proto` — tonic/prost gRPC contract.
- `secretd` — async tokio daemon (holds keys; auto-injects).
- `secretctl` — client.
- `secrets-store-libsql` — libSQL **remote** backend (`default-features = false`; no C linked).

## What it owns (the meta environment boundary)

envctl owns `META_ROOT`, `PATH`, `~/.local`, dotfiles, toolchain prefixes, and the canonical
`home/` overlay (`envctl/home`, ADR-0006) that `~/.claude/{settings.json,CLAUDE.md,RTK.md}` and
`meta/settings.json` symlink into. User-global holds only symlinks into meta. The reproducible
source of truth = `home/` overlay + `agent-env.lock` + `manifest/*.toml` + `envctl.lock`.

## Data flow

- **Provisioning:** `auto-detect` → `Engine` reads manifest components → `install`/`auto-fix`
  run lifecycle hooks (bash in `assets/scripts/`) → `doctor` verifies → `lock` records state.
- **Secrets:** `secretctl` → gRPC → `secretd` (vault) → key auto-injected into child tool; the
  real key never leaves the daemon.

## CI gates (trust-boundary + shape invariants)

`ci/gates/`: `no-c.sh` (no C in trust boundary), `shape.sh`, `enable.sh` (secretd systemd unit),
`p7.sh` (.handoff Tier-A conformance), `kdf-feature-off.sh`, `agent-env.sh` (config↔lock
no-drift), `loop-state.sh`, `harness-scripts.sh`.

## Design docs

`docs/ARCHITECTURE.md`, `docs/ROADMAP.md`, `docs/DESIGN-NOTES.md` (env-manager);
`docs/secrets/{SERVER-MODE,THREAT-MODEL,DESIGN-NOTES}.md` (secrets stack).