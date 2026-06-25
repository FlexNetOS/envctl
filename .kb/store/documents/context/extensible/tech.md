---
id: 019f007a-0a99-7422-aa53-e093269fb834
slug: context/extensible/tech
title: "Tech Context — envctl"
type: context
status: active
priority: medium
tags: [context, extensible]
---

How envctl is built and operated.

## Stack

Pure-Rust Cargo workspace (8 crates). MSRV **1.88**, stable toolchain (`rust-toolchain.toml`).
Zero system deps for engine+CLI; GUI needs native dev libs (see README "Native GUI").

## Build / test / lint

```bash
cargo build -p envctl-engine -p envctl       # engine + CLI
cargo run  -p envctl -- auto-detect          # read-only, safe (add --json for EnvReport)
cargo test --workspace                        # all crates
cargo fmt --all && cargo clippy --workspace -- -D warnings   # must be clean before commit
```

Tests are inline `#[cfg(test)] mod tests` or `crates/<crate>/tests/*.rs` (`#[tokio::test]` for
the async daemon path).

## CI gates (run before pushing dep / trust-boundary changes)

`ci/gates/{no-c,shape,enable,p7,kdf-feature-off,agent-env,loop-state,harness-scripts}.sh`.
`scripts/preflight.sh` mirrors the fast required CI checks locally (per-repo clippy mirror).

## Agent environment

`.claude/` + `.codex/` are provisioned by the built-in **agent-env** engine (absorbed kasetto):
edit `agent-skills/` + `agent-env.yaml`, then `envctl agent sync --apply`; CI enforces with
`envctl agent lock --check`. Do **not** hand-edit generated `.claude/skills/*` — except the
hand-authored Feature-Forge harness skills, which are git-tracked outside that pipeline.

## Knowledge base (this KB)

git-kb document store; `.kb/store/` is git-tracked text (durable). After pulling tracked store
changes, run `git-kb reindex` to rebuild the local `.cache/` index; `git-kb verify` checks file
store integrity. Cross-KB sync with meta: see `docs/kb-sync-runbook.md`.

## Logging

`RUST_LOG` (e.g. `RUST_LOG=envctl_engine=debug`).