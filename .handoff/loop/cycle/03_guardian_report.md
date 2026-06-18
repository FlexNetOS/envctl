# Verification report: TASK-0027 — GitHub installation-token early-revoke (DELETE /installation/token)

## Verdict — **PASS**

Independent cross-boundary verification of branch `task-0027-early-revoke` (off `develop`) in worktree
`/home/drdave/Desktop/meta/.worktrees/task-0027-early-revoke/envctl`. The additive `DELETE
/installation/token` early-revoke lands clean as a new engine method + `RevokeGithubToken` RPC +
`secretctl github-app revoke-token` verb + best-effort `relay_revoke` native tie-in. Every NON-NEGOTIABLE
invariant holds with source + test evidence; all 4 CI gates + fmt + clippy + the engine/secretd/secretctl
suites are green from raw `rtk proxy` passthrough (verified exit codes, not the implementer's word).
ZERO new dependencies / ZERO Cargo.lock delta.

(Note: the pre-existing `03_guardian_report.md` was a STALE report for TASK-0031-PR2 — overwritten.)

## Changeset scope
`git diff origin/develop --stat` = 11 files, +1201/-268 (incl. the two handoff `.md` artifacts):
`crates/secrets-engine/src/{mint_github.rs,event.rs,lib.rs}`,
`crates/secrets-proto/proto/control.proto`, `crates/secretd/src/{grpc.rs,conv.rs}`,
`crates/secretd/tests/native_mint_e2e.rs`, `crates/secretctl/src/{cli.rs,main.rs}`.
**No Cargo.toml change. No Cargo.lock change.**

## Gate results — exit codes captured (raw passthrough)
| Gate | Command | Exit | Result |
|------|---------|------|--------|
| no-c | `bash ci/gates/no-c.sh` | **0** | PASS — `rustls=['0.23.40'] on ring=['0.17.14']; zero aws-lc/openssl/C-SQLite` |
| shape | `bash ci/gates/shape.sh` | **0** | PASS — `SHAPE GATE PASS` |
| enable | `bash ci/gates/enable.sh` | **0** | PASS — `ENABLE GATE PASS` |
| p7 | `bash ci/gates/p7.sh` | **0** | PASS — `P7 GATE PASS` |

## cargo — exit codes captured
| Check | Command | Exit | Result |
|-------|---------|------|--------|
| fmt | `cargo fmt --all -- --check` | **0** | PASS |
| clippy (ws) | `cargo clippy --workspace --all-targets -- -D warnings` | **0** | PASS |
| test engine (provider-github) | `cargo test -p envctl-secrets-engine --features provider-github` | **0** | PASS — lib (incl. 5 mint_github + 8 lib revoke units), relay 22, vault 15; **0 failed** |
| test engine (default) | `cargo test -p envctl-secrets-engine` | **0** | PASS |
| test secretd | `cargo test -p envctl-secretd` | **0** | PASS — native_mint_e2e **14** (incl. 3 new revoke + `mint_github_returns_frozen_two_field_response`), proxy_swap 2, self_check 2; **0 failed** |
| test secretctl | `cargo test -p envctl-secretctl` | **0** | PASS — **13** (incl. 3 new revoke clap + the 2 frozen mint-github round-trips); **0 failed** |

New revoke tests observed green: engine `revoke_github_token_{dry_run_no_egress, apply_204_succeeds_metadata_only,
non_204_is_err_no_false_success, locked_vault_fails_closed}`, `relay_revoke_native_tie_in_best_effort_{success,
failure_still_returns}`, `relay_revoke_dry_run_no_native_egress`, `lock_clears_native_token_cache`; mint_github
`revoke_{builds_correct_delete_request, 204_is_success, non_204_is_failure_without_token,
transport_error_is_failure, token_only_in_auth_header_not_in_error}`; e2e
`revoke_github_token_{over_wire_204_succeeds, dry_run_contacts_nothing, locked_vault_fails_precondition}`;
clap `github_app_revoke_token_{parses_token_installation_and_apply, defaults_to_dry_run_and_accepts_stdin_dash,
requires_token}`.

## Invariant checks (1–6 from the brief)

1. **No C / one ring-only rustls — PASS.** no-c.sh exit=0. `git diff origin/develop --stat Cargo.lock` EMPTY
   (zero new crates). `git diff origin/develop -- '**/Cargo.toml' 'Cargo.toml'` EMPTY (ZERO added dep lines).
   Revoke REUSES the existing seam: `revoke_installation_token<T: HttpTransport + ?Sized>` runs over
   `self.inner.github_transport` (engine) / the daemon's `DaemonHttpTransport` (e2e) — same reqwest/rustls-
   on-ring transport as the mint path. Resolved graph unchanged ⇒ unaffected.

2. **Engine = single sync NON-PRINTING library — PASS.** Grep of the entire `crates/secrets-engine/` diff
   (added lines) for `println!/eprintln!/eprint!/print!/std::io::stdout`: NONE. The request shaping
   (`build_revoke_request`), 204/non-204/transport handling (`revoke_installation_token`), the dry-run vs
   apply policy + auth floor (`Engine::revoke_github_token`), and the relay tie-in policy (swallow + clear)
   all live in the engine via the `HttpTransport` seam (env-free). secretd is thin (Zeroizing wrap, empty
   reject, env read of `ENVCTL_GITHUB_API_BASE`, `spawn_blocking`, `map_mint_github_err`, RevokeResp drain);
   secretctl is thin (read token, dry-run preview, RPC drive, bool JSON). Engine emits
   `SecretEvent::GithubTokenRevoked`.

3. **Fail-closed / dry-run by default — PASS.** `apply` defaults false in BOTH proto3 (`bool apply = 2`) and
   clap (`#[arg(long)] apply: bool`). Dry-run does NO egress: engine returns early after a metadata-only
   audit+event (no transport call); CLI prints to stderr and never `connect()`s; e2e
   `revoke_github_token_dry_run_contacts_nothing` points at an UNROUTED base (`http://127.0.0.1:1`, no mock)
   and still returns `dry_run:true`. Non-204 ⇒ `Err` (`revoke_github_token_non_204_is_err_no_false_success`,
   401 mock); transport error ⇒ `Err` (`revoke_transport_error_is_failure`) — never a false success. The
   `relay_revoke` tie-in SWALLOWS its best-effort error (`relay_revoke_native_tie_in_best_effort_failure_still_returns`:
   500 DELETE ⇒ relay_revoke still `Ok`, emits `best_effort_failed`), while the EXPLICIT verb propagates the
   failure. Grep of the revoke request path in `mint_github.rs` (the two new fns) for
   `unwrap(`/`expect(`/`panic!`: NONE. (The engine's `revoke_github_token`/tie-in use `.lock().expect(..)` /
   `.read().expect(..)` only for lock-poison recovery — the standard engine-wide pattern, not in the
   network-shaping path.)

4. **No secret bytes in logs/audit — PASS (the critical one).** The installation token is `Zeroizing` end to
   end (CLI `read_token`, RPC `Zeroizing::new(req.token)`, engine arg `Zeroizing<Vec<u8>>`, cache values
   `Zeroizing<Vec<u8>>`). It appears ONLY as `format!("Bearer {tok}")` in the `Authorization` header of
   `build_revoke_request`; body is empty; url carries no token. The `GithubTokenRevoked` event +
   `github_token_revoked` audit rows are metadata-only (`installation_id: Option<u64>`, `outcome` ∈
   `revoked`/`dry_run`/`best_effort_failed`) — token never carried. `revoke_installation_token` builds its
   error from `resp.status` + a ≤200-char **response-body** snippet (the request token is not in the response
   body); the engine wraps it as `"github revoke failed: {e}"` — no token. `map_mint_github_err` echoes no
   secret. The request is NEVER `{:?}`/Debug-logged (explicit doc-comment in `build_revoke_request`). The
   zeroize unit (`revoke_token_only_in_auth_header_not_in_error`) asserts the token is absent from the
   MintError Display, the captured request url, and the body; the engine units scan **every emitted event
   JSON** for the token bytes; the e2e scans the event-stream wire. Verified: token nowhere but the header.

5. **Frozen-contract safety — PASS.** `control.proto` diff touches only the additive `rpc RevokeGithubToken`
   line + the new `RevokeGithubTokenReq` message; `MintGithubReq`/`MintGithubResp`/`RevokeResp` are
   UNCHANGED (the only MintGithub line in the diff is a context anchor). The `mint-github` clap shape is
   untouched (cli.rs diff only ADDS the `RevokeToken` variant). Wire/round-trip guards stay green:
   `mint_github_argv_round_trips_through_clap` + `..._without_optional_scopes` (secretctl),
   `mint_github_returns_frozen_two_field_response` (e2e) all pass.

6. **GHES api-base parity — PASS.** The revoke handler reads `ENVCTL_GITHUB_API_BASE` at grpc.rs:487
   BYTE-FOR-BYTE identically to the mint handler at grpc.rs:419
   (`std::env::var("ENVCTL_GITHUB_API_BASE").ok().filter(|b| !b.trim().is_empty())`), threading the same base
   into `revoke_github_token`. e2e drives both 204 and dry-run via the same `ENVCTL_GITHUB_API_BASE` loopback
   harness. (The relay tie-in plane has no request-level base and targets the public default
   `GITHUB_API_BASE_DEFAULT` — a documented best-effort limitation per Deviation 2, not a parity break, since
   the explicit verb is the GHES-correct kill-switch.)

## Parity check (front-end reach)
New engine method `revoke_github_token` → CLI: `crates/secretctl/src/main.rs` `github_app_revoke_token`
(via `Vault.RevokeGithubToken`) reaches it. GUI: NOT wired — explicitly Out-of-scope in the plan
(mint-github itself is not yet in the GUI); the engine method is the single shared entry so later GUI parity
won't diverge. Justified CLI-only surface for this cycle. Event reaches CLI+GUI identically via the
`conv.rs` no-proto-twin `return None` funnel (same path as `RelayRevoked`).

## Findings
None blocking. Notes:
- **N1 (doc nit, non-blocking).** `cli.rs` `--token` help mentions an `@path` form, but `read_token` treats
  any non-`-` value as a literal file path (no `@` prefix stripping). File paths and `-`/stdin both work as
  documented elsewhere; only the `@`-prefix phrasing is slightly off. Cosmetic; → implementer at leisure.
- **N2 (consistency, not a defect).** The grpc revoke handler emits to `EventSink::null()`, so the explicit
  verb's `GithubTokenRevoked` event is not surfaced on the daemon event stream. This is the SAME pattern as
  ALL 11 Vault RPC handlers (mint included) — consistent, not a regression. The relay tie-in (driven on the
  live engine sink) does surface its event.
- **N3 (deviation, accepted — matches plan).** `revoke_installation_token` bound is `T: HttpTransport +
  ?Sized` so the engine can pass `&dyn HttpTransport`; call surface unchanged, no behavior change.
- **N4 (deviation, accepted — matches plan §Deviations).** relay tie-in targets the public default base; a
  GHES relay's NATIVE early-revoke is best-effort while its policy+bearer revoke stays authoritative and the
  explicit verb threads the GHES base. Consistent with the plan's "best-effort, native-plane only" framing.

## Re-test needed
None — all gates + all suites green on this changeset. If N1 (the `@path` help phrasing) is fixed, re-run
`cargo test -p envctl-secretctl`.
