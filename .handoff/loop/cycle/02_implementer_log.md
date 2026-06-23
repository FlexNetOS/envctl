# Implementation log: TASK-0053 Route verified GitHub transport doctrine into envctl

Docs/doctrine + one additive regression test. NO new Engine method, RPC, CLI flag, type, or crate
dependency (card `allows_dependency_addition=false`).

## Changes
- `crates/secretctl/src/main.rs` — ADDED `#[cfg(test)] tests::policy_drift_permissions_scope_serializes`
  (and a TEST-ONLY throwaway `POLICY_DRIFT_TEST_KEY_PEM`), inserted beside the existing TASK-0020
  consumer-contract tests (new `// ===== TASK-0053` section immediately before the TASK-0026 block).
  Region: ~`main.rs:1164-1300`. No production code changed.
- `docs/secrets/GITHUB-TRANSPORT-DOCTRINE.md` — NEW tracked doctrine note (SSH-truth / gh-advisory /
  read-back; broker-only mint-github + POLICY_DRIFT path; merge-gate cross-check; redb/JSONL).
- `docs/secrets/README.md` — appended one index entry under "## Key entry points" (matches existing
  list format).
- `.handoff/loop/backlog.md` — appended a "### GitHub transport doctrine (TASK-0053…)" subsection +
  a `- [~]` status row, under the existing session-9 NEXT-PICK anchor (no header rewrite/duplication;
  handoff-reconcile merge-driver safe).

## Engine API
NONE — zero new/changed Engine methods or Events. The test calls only EXISTING public engine surface
(`GitHubAppMint::new`, `HttpTransport`/`HttpRequest`/`HttpResponse`/`TransportError`, `MintRequest`,
`ProviderMint`, `SystemClock`, `broker::Provider`) to drive the REAL private serializer
`build_token_request_body` (via `mint_scoped`) through a capturing transport. No reimplementation.

## Tests added
- `policy_drift_permissions_scope_serializes` (`crates/secretctl/src/main.rs`) — proves (1) the
  POLICY_DRIFT scope `administration:write,metadata:read` parses through the real `mint-github` clap
  surface into `MintGithubArgs.permissions` (via `consumer_build_argv`, installation 140063898), and
  (2) the ENGINE's real request-body serializer emits exactly the GitHub permission map
  `{"administration":"write","metadata":"read"}`. Asserts ONLY on the captured request body /
  parsed args; NEVER logs/prints a token; uses a throwaway 1024-bit RSA key + canned GitHub 201 (no
  network, no real credential). Covers AC2/AC3/AC4.

## Build/test status (all from worktree root; logs in /tmp)
- `bash ci/gates/no-c.sh` — **PASS** (exit 0). "NO-C GATE PASS; rustls=['0.23.40'] on ring; zero aws-lc/openssl/C-SQLite".
- `bash ci/gates/p7.sh` — **PASS** (exit 0). "P7 GATE PASS".
- `cargo test -p envctl-secretctl` — **PASS** (exit 0). 16 passed / 0 failed (new test included).
- `cargo test -p envctl-secrets-engine --features provider-github mint` — **PASS** (exit 0). serializer cross-check still green (35 + 4 passed).
- `cargo clippy -p envctl-secretctl -p envctl-secretd -p envctl-secrets-engine --features envctl-secrets-engine/provider-github -- -D warnings` — **PASS** (exit 0), no warnings.
  (Scoped to the 3 touched secrets crates per the plan note — workspace clippy in the meta tree lints siblings; this is an exact subset, never weaker, for the only crate with code changes.)
- `cargo fmt -p envctl-secretctl -- --check` — **PASS** (exit 0, after one auto-reformat of the new test; test re-run green after reformat).

## Zero-dep / zero-lock confirmation
`git status --short` shows ONLY: `.handoff/loop/backlog.md`, `.handoff/loop/cycle/01_architect_plan.md`
(orchestrator-written cycle plan, NOT edited by me), `.handoff/loop/cycle/02_implementer_log.md`,
`crates/secretctl/src/main.rs`, `docs/secrets/README.md`, and the new
`docs/secrets/GITHUB-TRANSPORT-DOCTRINE.md`. **NO** `Cargo.toml`, `Cargo.lock`, `envctl.lock`,
`agent-env.lock`, or `manifest/*.toml` changes — explicit guard grep returned
"ZERO lock/dep/manifest drift — clean".

Continuity-wording trap (AC7): `grep -ni sqlite` over the new/changed docs returns ONLY the two
explicit "NOT SQLite" negations in GITHUB-TRANSPORT-DOCTRINE.md (lines 107-108) and the pre-existing
backlog line 73. No bare "SQLite" claim introduced.

## Deviations
- The plan's U2 said "call the real serializer the engine uses." `build_token_request_body` is a
  module-private `fn` (not callable directly, and the card forbids changing the engine surface). To
  pin the ACTUAL shape without reimplementing it, the test drives the genuine serializer *through*
  the public `GitHubAppMint::mint_scoped` path with a capturing transport (the same technique the
  engine's own `mint_builds_correct_request_and_parses_token` test uses). This is the real engine
  code path, not a reimplementation — intent fully met, zero engine-surface change.
- Used `envctl_secrets::SystemClock` (public re-export) instead of a hand-rolled `FixedClock`, to
  avoid naming `chrono` (NOT a direct secretctl dep — adding it would violate the no-dep card). The
  JWT timestamp is irrelevant here (canned 201 ⇒ JWT never validated; only the request body is
  inspected). Otherwise implemented exactly as scoped.

## Handoff notes (for the invariant-guardian)
- **AC3/AC4 focus:** `policy_drift_permissions_scope_serializes` asserts on the request-body
  permission MAP only and uses a throwaway key + canned response — verify no token is ever printed
  and no real credential is present (it is parse-/fake-transport-level only).
- **Runtime surface (Phase 3.5):** `cargo run -p envctl-secretctl -- mint-github --installation-id
  140063898 --repository-ids 1 --permissions administration:write,metadata:read --ttl-secs 3600
  --output json` against a locked/absent daemon must fail-closed (never emit a token). Positive shape
  is covered hermetically by this test + `crates/secretd/tests/native_mint_e2e.rs`.
- **AC7 continuity wording:** confirm the only "sqlite" hits in the diff are "NOT SQLite" negations.
- **No-dep/no-lock:** confirm `git status` carries no `Cargo.*`/`*.lock`/`manifest/*.toml` change.
- The `01_architect_plan.md` modification in `git status` is the orchestrator's cycle-plan write
  (TASK-0039 → TASK-0053 content), NOT my edit.
- Doc citations were all read-back-verified from source (rotate-policy-drift-token.sh:37-39/90-95/116;
  merge_gate.rs:66-88; mint.rs:131-143; map:116/136/167/475; deep-review-plan:56).

## Status
**GREEN**
