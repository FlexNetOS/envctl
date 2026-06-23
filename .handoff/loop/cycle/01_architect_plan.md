# 01 — Architect Plan: TASK-0053 Route verified GitHub transport doctrine into envctl

**VERDICT: GO** — docs/doctrine-routing + verification-test cycle. Single-repo envctl, sequential.
No new Engine method, RPC, CLI flag, type, or crate dependency.

## Triggering-claim check (per acceptance criterion vs HEAD)
- **AC1 (doctrine in docs/backlog):** NET-NEW docs. Belongs in `.handoff/loop/backlog.md` + new `docs/secrets/GITHUB-TRANSPORT-DOCTRINE.md`.
- **AC2 (mint-github byte-stable):** HOLDS — `crates/secretctl/src/main.rs:1084-1429` verbatim-consumer contract tests; stdout pinned two-field `{token,expires_at_unix}` at `main.rs:440-445`. U2 extends.
- **AC3 (POLICY_DRIFT_TOKEN path):** SATISFIED by existing `mint-github --permissions administration:write,metadata:read --repository-ids <id>`; live consumer `.github_org/scripts/rotate-policy-drift-token.sh:90-93` (wired 2026-06-21). Engine `build_token_request_body` (`mint_github.rs:342`) already serializes arbitrary `name:access` perms. NO new surface — document + test.
- **AC4 (tokens broker-only/scoped/never logged):** HOLDS — `mint_github.rs` uses `Zeroizing`, token only in auth header, token-free error snippets, metadata-only audit.
- **AC5 (consumer cross-check):** HOLDS — `../flexnetos_github_app/crates/app-core/src/mint.rs::parse_mint_output` expects exactly `{token, expires_at_unix:u64}` (matches main.rs:440); `merge_gate.rs::ensure_armable` green-only + `UnwiredMergeGate` fail-closed.
- **AC6 (SSH-git + gh read-back verify):** verification step, U4.
- **AC7 (redb/JSONL continuity wording, NOT SQLite):** doc-wording invariant; reuse exact phrasing at `.handoff/loop/backlog.md:73,195-196`.

## Target repos
- **1 repo: envctl** (single-crew sequential DEFAULT). `flexnetos_github_app` and `.github_org` are **read-only cross-checks**, never edited.
- 4 units, near-linear doc→test→doc→verify — sequential, no pipeline/A2.

## Unit ledger
| U# | Goal | Lives (`file::symbol`) | Engine/FE | Test | AC |
|----|------|------------------------|-----------|------|-----|
| U1 | Doctrine subsection + status-truth + TASK-0053 row in backlog (append via handoff-reconcile; no header rewrite) | `.handoff/loop/backlog.md` | docs | p7 gate | AC1, AC7 |
| U2 | Additive permission-scoping regression test for the POLICY_DRIFT scope | `crates/secretctl/src/main.rs::tests::policy_drift_permissions_scope_serializes` (beside :1084 contract tests) | FE-test | `cargo test -p envctl-secretctl` | AC2,AC3,AC4 |
| U3 | Tracked doctrine note (SSH-truth/gh-advisory/read-back; POLICY_DRIFT path; merge-gate cross-check; redb/JSONL) | `docs/secrets/GITHUB-TRANSPORT-DOCTRINE.md` (new) + index entry in `docs/secrets/README.md` | docs | p7 | AC1,AC3,AC5,AC7 |
| U4 | Run gates + SSH-git/gh read-back verify; confirm lock/manifest clean | `ci/gates/no-c.sh`, cargo test/clippy | verify | gates | AC6, all |

## Runtime surface
**runtime_verifiable = YES (CLI verb, fail-closed path).** Guardian Phase 3.5 drives:
`cargo run -p envctl-secretctl -- mint-github --installation-id 140063898 --repository-ids 1 --permissions administration:write,metadata:read --ttl-secs 3600 --output json` against a **locked/absent** daemon ⇒ must fail-closed (never emit a token). Positive shape covered hermetically by `crates/secretd/tests/native_mint_e2e.rs` + U2. Doctrine read-back rule verified at workflow level: `git ls-remote --symref origin HEAD` (SSH) + `gh pr view <PR> --json state,mergeStateStatus`.

## Invariant risk
- **Continuity-wording trap (AC7):** every continuity sentence MUST say redb-backed ledger + deterministic JSONL export, NEVER SQLite. Guardian greps diff for "sqlite"/"SQLite" (absent except in a "NOT SQLite" negation).
- **Token-leak in new test (AC4):** U2 asserts on the request-body permission MAP only; never logs/prints a token; no real credential in fixtures.
- **Backlog merge-concatenation hazard:** U1 appends under existing anchors via `handoff-reconcile`; no duplicate headers.
- **No-C / no-dep:** zero deps (card forbids); `no-c.sh` hard regression gate in U4.

## Open questions
None. Fork (new POLICY_DRIFT surface vs document existing) resolves to **document + test the existing `mint-github` path** — production consumer already uses it with the named scope.
