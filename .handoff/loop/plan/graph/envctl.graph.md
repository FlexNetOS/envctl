# envctl Code Graph Snapshot

Date: 2026-07-02
Worktree: `/home/flexnetos/FlexNetOS/src/envctl-plan-autoresearch-20260702`
Branch: `codex/plan-autoresearch-20260702`
Baseline head: `origin/master` at `5210061 catalog: import Yazelix CodeDB file inventory (#410)`

## Commands

- `rtk /home/flexnetos/FlexNetOS/usr/bin/git-kb code index --force --prune`
- `rtk /home/flexnetos/FlexNetOS/usr/bin/git-kb code stats --json`
- `rtk /home/flexnetos/FlexNetOS/usr/bin/git-kb code doctor --json`
- `rtk /home/flexnetos/FlexNetOS/usr/bin/git-kb code entrypoints --refresh --json`
- `rtk /home/flexnetos/FlexNetOS/usr/bin/git-kb code flows --refresh --json`
- `rtk /home/flexnetos/FlexNetOS/usr/bin/git-kb code dead --json`

## Snapshot

- Indexed symbols: 4,297 from 925 files.
- Language split: 4,279 Rust symbols, 18 Python symbols.
- Call sites extracted: 37,408.
- Resolved call edges: 10,157.
- Unresolved calls: 27,251.
- Import facts: 500.
- Symbol-forwarding facts: 402.
- Stale files: 0.
- Service-route/client facts: 0 routes, 0 client calls, 0 matches.

## Graph Integrity Caveat

`git-kb code stats --json` reported `kb_root` as `/home/flexnetos/FlexNetOS/src/envctl`,
even though this snapshot was requested from the sibling worktree
`/home/flexnetos/FlexNetOS/src/envctl-plan-autoresearch-20260702`.
This is acceptable as a baseline smell-finding, but it is not acceptable as an
unqualified proof for branch-specific planning. Future autoresearch must fail closed when
`kb_root` and `git rev-parse --show-toplevel` disagree.

## Entrypoint Clusters

- Runtime binaries: `envctl`, `meta-env`, `envctl-gui`, `secretctl`, `secretd`.
- Build script: `crates/secrets-proto/build.rs`.
- Maintenance script: `scripts/meta-fleet-sync.py`.
- High-confidence test entrypoints are numerous, especially under `crates/agent-env/tests`.

## Hotspots

- `crates/secrets-engine/src/lib.rs`: 5,876 lines.
- `crates/cli/src/main.rs`: 4,998 lines.
- `crates/engine/src/catalog.rs`: 4,331 lines.
- `crates/gui/src/main.rs`: 3,466 lines.
- `crates/agent-env/tests/parity_vs_kasetto.rs`: 3,140 lines.
- `crates/engine/src/executor.rs`: 1,115 lines.

`git-kb code doctor --json` also listed the largest symbol-bearing files as
`crates/agent-env/tests/parity_vs_kasetto.rs`, `crates/engine/src/catalog.rs`,
`crates/gui/src/main.rs`, `crates/secrets-engine/src/lib.rs`, and
`crates/cli/src/main.rs`.

## Dead-Code Query Shape

`git-kb code dead --json` returned 100 entries, mostly test-only functions under
`#[cfg(test)]` or integration test files. Treat the current dead-code report as a
noise detector until the planner either excludes test-only symbols or labels them
separately from product dead code.

## Unresolved Breakdown

- `no_match`: 12,687.
- `skip_list`: 12,276.
- `ambiguous`: 1,488.
- `stdlib_allowlist`: 718.
- `symbol_forwarding_ambiguous_star`: 82.

Resolved edges are materially lower than unresolved calls, so call-graph-derived
impact analysis should carry a confidence note until resolver provenance improves.
