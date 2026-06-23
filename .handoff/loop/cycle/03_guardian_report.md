# Verification report: TASK-0053 Route verified GitHub transport doctrine into envctl

## Verdict — PASS

Docs + one additive regression-test cycle. All real gates, all scoped cargo checks, the runtime
fail-closed surface, and every invariant-specific check pass. Zero dep/lock/manifest drift. No
blocking findings.

## Gate results (exit codes captured directly; rtk not in the path)
- `bash ci/gates/no-c.sh` → **PASS** exit=0. "resolved graph clean: rustls=['0.23.40'] on ring=['0.17.14']; zero aws-lc/openssl/C-SQLite; NO-C GATE PASS"
- `bash ci/gates/p7.sh` → **PASS** exit=0. "P7 GATE PASS" (backlog edit did not break .handoff conformance)
- `bash ci/gates/shape.sh` → **PASS** exit=0. "SHAPE GATE PASS"
- `bash ci/gates/loop-state.sh` → **PASS** exit=0 (loop_state.md untouched; confirmed)

## cargo
- `cargo test -p envctl-secretctl` → **PASS** exit=0. 16 passed / 0 failed (incl. new `policy_drift_permissions_scope_serializes`)
- `cargo test -p envctl-secrets-engine --features provider-github` → **PASS** exit=0. 171/4/6/22/15 passed across binaries, 0 failed (serializer cross-check green)
- `cargo test -p envctl-secretd --features provider-github` → **PASS** exit=0. 38/3/5 passed, 0 failed, 0 panics (byte-stable mint path green)
- `cargo clippy -p envctl-secretctl -p envctl-secretd -p envctl-secrets-engine --features envctl-secrets-engine/provider-github -- -D warnings` → **PASS** exit=0, no warnings. (Scoped to this repo's own secrets crates, mirroring the repo's own CI gate; workspace clippy in the meta tree lints siblings and is stricter than this repo's CI.)
- `cargo fmt -p envctl-secretctl --check` → **PASS** exit=0

## Invariant checks (1-10)
1. **No-C / no-dep** — PASS. `no-c.sh` proves resolved graph clean (one rustls 0.23.40 on ring; zero aws-lc/openssl/C-SQLite). `git status` shows ZERO changes to Cargo.toml/Cargo.lock/envctl.lock/agent-env.lock/manifest/*.toml. No new dependency.
2. **Code-shape** — PASS. `shape.sh` exit=0.
3. **secretd enable** — N/A (no systemd-unit surface touched); not affected by a docs+test cycle.
4. **Engine purity** — PASS. `git status` shows ZERO modifications to `crates/secrets-engine/` and `crates/secretd/`. New logic is a `#[cfg(test)]` test in `crates/secretctl/src/main.rs` only; no engine source touched, no print added.
5. **Front-end parity** — N/A. No new Engine method/Event/RPC/CLI flag. The test drives the EXISTING public engine surface (`GitHubAppMint::mint_scoped` via a capturing transport).
6. **Fail-closed + dry-run defaults** — PASS (verified at runtime, Phase 3.5). Mutating App ops remain `--apply`-gated (doc §3 cross-checked against `main.rs`); the mint path itself refuses without an unlocked/proven vault (observed live).
7. **Rust-native, no drift** — PASS. No non-Rust source/package files added; no banned dep; no dep at all.
8. **Lock honesty** — PASS. No components/deps changed; locks correctly untouched.
9. **Kasetto / agent-env** — N/A (no `crates/agent-env` change).
10. **Runtime behavior** — PASS. Plan declares `## Runtime surface` (CLI fail-closed path); driven and observed (see `## Runtime check`).

## Parity check
No new Engine method → no CLI/GUI parity surface to add. The new test reaches the real engine
serializer `build_token_request_body` *through* the public `GitHubAppMint::mint_scoped` path
(`crates/secrets-engine/src/mint_github.rs`) — the genuine code path, not a reimplementation.

## Unit ledger (per-unit present + wired)
| U# | present | wired | evidence file:line |
|----|---------|-------|--------------------|
| U1 (backlog doctrine subsection + TASK-0053 row) | YES | YES (p7 gate green; appended under existing anchor, no header dup) | `.handoff/loop/backlog.md` (+33) |
| U2 (regression test) | YES | YES (runs in `cargo test -p envctl-secretctl`, 16 passed) | `crates/secretctl/src/main.rs:1233 policy_drift_permissions_scope_serializes` |
| U3 (doctrine doc + README index) | YES | YES (tracked; README index entry +3) | `docs/secrets/GITHUB-TRANSPORT-DOCTRINE.md` (new, 116 lines); `docs/secrets/README.md` (+3) |
| U4 (gates + verify + lock clean) | YES | YES (this report) | gate logs + runtime check below |

## Runtime check
**PASS.** Built the real `target/debug/secretctl` and drove the declared fail-closed surface:
`secretctl mint-github --installation-id 140063898 --repository-ids 1 --permissions administration:write,metadata:read --ttl-secs 3600 --output json`.
A daemon was present but the vault was locked — an even stronger fail-closed demonstration than an
absent daemon. Observed: **exit=1**; **stdout EMPTY (no token emitted)**; **stderr** =
`Error: status: FailedPrecondition, message: "vault is locked"`. Token-leak scan over both streams:
NO-TOKEN-IN-OUTPUT. This is the required behavior — refuse to mint without a proven/unlocked vault,
emit no token, surface a clear error class, exit non-zero.

## Invariant-specific findings
- **AC4 token-never-logged** — PASS. The new test asserts ONLY on the captured request-body
  permission map (`body["permissions"] == {"administration":"write","metadata":"read"}`); it
  asserts on NO token value, prints nothing, and uses a clearly-labelled TEST-ONLY throwaway 1024-bit
  RSA key (`POLICY_DRIFT_TEST_KEY_PEM`, "weak BY DESIGN; NEVER a real credential") with a canned 201
  (no network, no real credential).
- **AC2/AC5 byte-stability** — PASS. main.rs diff is **0 deletions, +146 additive lines**. The
  TASK-0020 consumer-contract helpers/tests (`consumer_build_argv` :1090, `expires_at_unix`
  contract at :1131-1154) are present and UNCHANGED — the new test merely *reuses* `consumer_build_argv`.
  Consumer cross-check confirmed: `../flexnetos_github_app/crates/app-core/src/mint.rs::parse_mint_output`
  deserializes `struct Out { token: String, expires_at_unix: u64 }` — exactly the doc's cited
  two-field shape and what secretctl emits (`main.rs:442`).
- **AC7 continuity wording** — PASS. `grep -ni sqlite` over the diff + new doc: every hit is an
  explicit "NOT SQLite" negation (doc lines 107-108: "redb + deterministic JSONL export, never
  SQLite"); other hits are plan/log meta-text describing the negation rule. No bare-SQLite
  continuity claim introduced.
- **Doc citation integrity** — PASS (spot-checked 3 citations against live source):
  - `rotate-policy-drift-token.sh:37-39,90-95` — INSTALLATION_ID 140063898, TTL 3600,
    `PERMS="administration:write,metadata:read"`, `secretctl mint-github … --permissions "${PERMS}"`
    — matches the doc verbatim.
  - `mint.rs:131-143` parse contract — matches (`{token, expires_at_unix:u64}`).
  - `merge_gate.rs` — `ensure_armable` green-only + `UnwiredMergeGate` fails-closed (`NotWired`) —
    claims accurate. (See NOTE N1 on a cosmetic citation detail.)

## Notes (non-blocking)
- **N1 (citation cosmetic):** GITHUB-TRANSPORT-DOCTRINE.md §4 cites `merge_gate.rs:66-74`/`:81-88`
  and phrases the green check as `Conclusion::Success`; the live code checks
  `verdict.conclusion.is_green()` and `UnwiredMergeGate` sits at ~`:78-88`. The *claims* are faithful
  (green-only arm; fail-closed `NotWired`) — small line-offset / paraphrase, not a fabrication.
  Severity: trivial; no action required.
- **N2 (clippy axis classification):** scoped clippy on the 3 secrets crates is clean (exit 0). No
  inherited red in touched code; the broader workspace clippy is deliberately out of scope (lints
  meta siblings, stricter than this repo's own CI). No blocker.

## Re-test needed
None — PASS. To reproduce:
```
bash ci/gates/no-c.sh ; echo exit=$?
bash ci/gates/p7.sh ; echo exit=$?
cargo test -p envctl-secretctl
cargo test -p envctl-secrets-engine --features provider-github
cargo test -p envctl-secretd --features provider-github
cargo clippy -p envctl-secretctl -p envctl-secretd -p envctl-secrets-engine --features envctl-secrets-engine/provider-github -- -D warnings
cargo fmt -p envctl-secretctl --check
# runtime fail-closed (vault locked / daemon absent):
./target/debug/secretctl mint-github --installation-id 140063898 --repository-ids 1 --permissions administration:write,metadata:read --ttl-secs 3600 --output json ; echo exit=$?
```
