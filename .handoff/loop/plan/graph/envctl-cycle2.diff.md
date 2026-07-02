# envctl graph diff - cycle 2

Compared evidence: cycle 1 artifacts in this directory and cycle 2 live refresh on 2026-07-02.

## Stable findings

- Symbol count stayed at 4297.
- Rust/Python split stayed at 4279/18.
- Resolved and unresolved call counts stayed at 10157 and 27251.
- Stale files stayed at 0.
- The main product hotspot remains `crates/cli/src/main.rs::function::main`.
- Test flows continue to dominate the top flow list beside product flows.

## New or sharpened findings

1. Worktree identity mismatch is now directly proven.

- Active shell branch is `codex/plan-autoresearch-20260702`.
- GitKB doctor and flows report `master`.
- GitKB stats report the main checkout root, not the worktree root.

2. Source-ledger weakness is now tied to concrete gate behavior.

- The planning artifact gate checks key presence but not date parsing, computed recency, contradiction invalidation, or source authority.

3. Trust-boundary source facts were refreshed from official/current sources.

- Rust latest stable: 1.96.1 on 2026-06-30.
- rustls latest defaults still include `aws-lc-rs`; ring-only configuration remains intentional.
- libsql defaults still include C core paths; remote-only no-default configuration remains intentional.
- tonic 0.12.3 remains the documented fix floor for RUSTSEC-2024-0376.
- rustls-webpki advisories in 2026 keep TLS audit freshness relevant.

4. Runner planning needs preview/stable classification.

- GitHub Actions Ubuntu 26.04 images are public preview as of 2026-06-11, not a stable hosted-runner baseline.

## Actionable deltas

- U8: graph identity gate.
- U9: product/test labels for entrypoint, flow, and dead-code output.
- U10: explicit service-edge-zero blind-spot note.
- U11: source-ledger truth checks.
- U12: dependency trust-boundary watchlist.
- U13: hosted runner preview classification.

No product code changed in this research cycle.
