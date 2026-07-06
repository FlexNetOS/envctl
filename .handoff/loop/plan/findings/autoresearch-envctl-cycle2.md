# envctl autoresearch cycle 2

Date: 2026-07-02
Repo: /home/flexnetos/FlexNetOS/src/envctl-plan-autoresearch-20260702
Branch: codex/plan-autoresearch-20260702
Head: 52100614ab4666e6abab52a3292d0149351a9453
Recency window: 2026-04-03 through 2026-07-02

This cycle refreshes code graph evidence, checks current upstream facts for the trust-boundary and runner/toolchain claims, and surfaces planning gaps. It is an evidence pass, not an implementation pass.

## Code auto-research

Refresh commands:

- `/home/flexnetos/FlexNetOS/usr/bin/git-kb code index --force --prune`
- `/home/flexnetos/FlexNetOS/usr/bin/git-kb code stats --json`
- `/home/flexnetos/FlexNetOS/usr/bin/git-kb code doctor --json`
- `/home/flexnetos/FlexNetOS/usr/bin/git-kb code entrypoints --refresh --json`
- `/home/flexnetos/FlexNetOS/usr/bin/git-kb code flows --refresh --json`
- `/home/flexnetos/FlexNetOS/usr/bin/git-kb code dead --json`
- `/home/flexnetos/FlexNetOS/usr/bin/git-kb code query public-api --json`
- `/home/flexnetos/FlexNetOS/usr/bin/git-kb code impact crates/secretd/src/main.rs --depth 3 --json`

Observed graph shape:

- Symbols: 4297 total, 4279 Rust, 18 Python.
- Indexed files: stats reported 925 detected files; doctor reported 149 symbolized files.
- Call sites: 37408 extracted, 10157 resolved call edges, 27251 unresolved calls.
- Stale files: 0.
- Unresolved breakdown: 12687 no_match, 12276 skip_list, 1488 ambiguous, 718 stdlib_allowlist, 82 symbol_forwarding_ambiguous_star.
- Highest fan-out flows include CLI main, GUI main/test_app, and many agent-env / engine / secretd test entrypoints.
- Service edge health is all zero: routes, clients, cross-service edges, inbound, outbound, and files_with_service_facts.

## Patterns

1. The engine and CLI remain the main product spine. The largest product flow is `crates/cli/src/main.rs::function::main`, spanning 725 nodes and 68 files.

2. Agent-env is a dominant public API and test surface. Public API queries are heavily populated by `envctl-agent-env` config, source, MCP, and command target types, and the top flows include `crates/agent-env/tests/parity_vs_kasetto.rs`.

3. GUI is a high-fan-out surface. `crates/gui/src/main.rs::function::main` and `test_app` both traverse 470+ nodes and 56 files. GUI runtime verification should be kept separate from compile-only evidence.

4. The secrets stack is broad but better modularized than the CLI/GUI surfaces. Secret daemon impact on `crates/secretd/src/main.rs` showed only 8 local symbols at depth 3, while integration tests cover larger flows.

5. Planning evidence is strongly dependent on GitKB metadata correctness. The graph looks internally fresh, but root and branch identity are inconsistent with the active worktree.

## Surfaced gaps

G1. GitKB root and branch identity drift.

- Shell branch: `codex/plan-autoresearch-20260702`.
- GitKB stats `kb_root`: `/home/flexnetos/FlexNetOS/src/envctl`.
- GitKB doctor and flows branch: `master`.
- Risk: a terminal plan could cite fresh graph evidence from the main checkout while the agent is actually in a worktree.

G2. File-count semantics are unclear.

- `stats.file_count` reported 925.
- `doctor.file_count` reported 149.
- Likely explanation: stats counts detected files while doctor reports symbolized deep files. Plans must label which metric they use.

G3. Flow ranking is not product/test aware.

- Test entrypoints are ranked beside product entrypoints with the same 0.92 criticality.
- Risk: planners may over-prioritize test helper flows or understate product runtime blast radius.

G4. Dead-code reports are polluted by tests.

- Top dead symbols are mostly `#[cfg(test)]` functions and fixtures.
- Risk: dead-code cleanup plans can become noisy unless reports are filtered or labeled by product/test/source kind.

G5. Service edge extraction is a blind spot.

- Doctor reported service-capable files but zero route/client facts.
- Risk: RPC, CLI-to-daemon, and secretctl/secretd impact plans cannot rely on current service-edge graphs.

G6. Public API output is broad but not grouped by stable surface.

- Agent-env public symbols dominate the top of the output.
- Risk: planning cannot distinguish exported crate APIs, binary-only helpers, generated proto surface, and internal modules without extra grouping.

G7. Source-ledger validation is structural, not truth-checking.

- The gate checks required JSON keys but does not prove parseable dates, recency truth, contradiction handling, source authority, or claim expiry.
- Risk: stale or malformed sources can pass as long as the row has the right keys.

## Web auto-research

Current facts checked this cycle:

- Rust latest stable is 1.96.1, published 2026-06-30. envctl's MSRV 1.88 is fine as a floor, but any plan calling 1.88 "current stable" would be stale.
- rustls latest docs still show `aws-lc-rs` in default features. envctl's ring-only `default-features = false` setup remains a required trust-boundary pattern, not accidental complexity.
- libsql docs still state defaults enable all features, including core C code. envctl's `default-features = false` plus `remote` feature is still the right no-C posture.
- RustSec advisory RUSTSEC-2024-0376 says tonic is patched at `>=0.12.3`; envctl's tonic 0.12.3 pin matches the documented floor.
- RustSec currently has 2026 rustls-webpki advisories in the 90-day window. Cargo audit freshness remains a live planning requirement for TLS work.
- GitHub Actions Ubuntu 26.04 runner images entered public preview on 2026-06-11. Treat Ubuntu 26.04 runner parity as preview evidence, not a stable hosted-runner guarantee.

## Upgrade-only backlog rows

U8. Add a graph identity gate.

- Trigger: G1.
- Requirement: terminal plans must fail or warn loudly when `git rev-parse --show-toplevel`, current branch, GitKB `kb_root`, and GitKB branch metadata disagree.

U9. Add product/test labels to graph reports.

- Trigger: G3 and G4.
- Requirement: entrypoint, flow, and dead-code reports must classify symbols as binary, library API, integration test, unit test, build script, example, or script.

U10. Add service-edge health expectations.

- Trigger: G5.
- Requirement: plans touching `secretctl`, `secretd`, proto, daemon server mode, relay minting, or edge hardening must include an explicit note when route/client graph facts are zero.

U11. Strengthen source-ledger truth checks.

- Trigger: G7.
- Requirement: validate ISO dates, compute `in_recency_window`, reject unknown dates unless marked durable-reference, and require contradiction rows to invalidate affected claim IDs.

U12. Add trust-boundary dependency watchlist rows.

- Trigger: web checks for rustls, libsql, tonic, rustls-webpki.
- Requirement: planning for TLS, libSQL, gRPC, or audit gates must cite current crate/advisory state and no-C gate expectations.

U13. Add runner-preview classification.

- Trigger: Ubuntu 26.04 runner public preview.
- Requirement: runner-routing plans must distinguish workstation target, local runner, hosted runner preview, and stable hosted runner.

## Gate handoff

Recommended tests for planning gates:

- A source row with `published_at: not-a-date` must fail.
- A source row outside the 90-day window with `in_recency_window: true` must fail unless marked as a durable reference.
- A contradictory source row must invalidate at least one active claim ID.
- A graph artifact with mismatched shell root and GitKB `kb_root` must fail terminal-plan mode.
- Flow/dead-code artifacts without product/test labels must fail terminal-plan mode.
- A service-edge-zero graph must force an explicit blind-spot note for daemon/RPC/CLI boundary plans.

## Cadence

Per-cycle refresh:

- Re-run GitKB index, stats, doctor, entrypoints, flows, dead-code, and target-specific impact.
- Refresh source ledger for any claim that is current, versioned, advisory-driven, or runner/toolchain-driven.
- Append graph diff and source rows rather than overwriting prior cycle evidence.

Batch-boundary deep refresh:

- Re-run `cargo metadata`, `ci/gates/no-c.sh`, `ci/gates/cargo-audit.sh`, `ci/gates/runner-routing.sh`, and the planning artifact gate.
- Compare top flow and unresolved-call deltas against the previous cycle.

Resume invalidation:

- Invalidate non-durable web claims older than 90 days.
- Re-check GitKB root and branch identity before accepting any stale graph artifact.
- Re-check active target rows before declaring a plan terminal.
