# HANDOFF — forge-loop

closed_utc: 2026-06-23T03:40:10Z
branch: task-0039-client-ca-lifecycle
worktree: /home/drdave/Desktop/meta/.worktrees/task-0039-client-ca-lifecycle/envctl
cycle_budget: 1
cycles_total: 28
cycles_this_session: 9
last_item: TASK-0039
next_item: TASK-0039
orchestrator_phase: PR_OPEN_WAIT_FOR_MERGE
last_agent: codex
gate_status: PASS-WITH-NOTES
pr_url: https://github.com/FlexNetOS/envctl/pull/162

## Landed This Session

- 882c111 secretd: add remote client CA lifecycle
- PR #159 merged earlier this session: handoff stale-card reconciliation.
- PR #162 is open and carries TASK-0039 implementation; do not run `hf done` until it is confirmed
  `MERGED`.

## Current State

TASK-0039 was reopened because PR #158 only delivered the verifier/revocation-file enforcement slice.
This branch implements the lifecycle slice:

- DEK-sealed remote-clients CA distinct from the MITM CA.
- Remote-clients CA rebuilds on unlock and zeroizes on lock.
- `control_plane_client` issuance defaults to 7 days and refuses `ttl_days > 7`.
- `Engine::ca_renew` reissues live client leaves and revokes superseded rows.
- `Engine::ca_revoke` is dry-run by default, requires `--apply --confirm`, revokes cert rows, and
  disables a matching remote-client registry row.
- `secretd` wires `Certs.Renew` and `Certs.Revoke`.
- `secretd` appends SHA-256 DER fingerprints to configured `client_revocations_path`, the same format
  PR #158's mTLS verifier reloads.
- libSQL cert storage now projects revocation state through an idempotent `cert_revocations` table.

## Decisions And Dead Ends

- Do not mark TASK-0039 done on local PASS or auto-merge armed. Tick-on-merged applies: only
  `gh pr view 162 --json state -q .state` returning `MERGED` permits `hf done TASK-0039 --pr 162`.
- Do not delete `task-0039-client-ca-lifecycle` branch or its worktree until PR #162 is confirmed
  merged. Reaper may clean older merged worktrees only.
- The existing `IssueLeafReq` stream does not return private key material. The implemented lifecycle
  closes the frozen Certs/secretctl surfaces; full device enrollment/export packet automation remains
  a future surface if the owner wants it.
- ICM correction: earlier local context had mistakenly treated TASK-0039 as done after PR #158. It is
  now correctly claimed/in-flight until PR #162 merges.

## Verification Completed Locally

- `cargo test -p envctl-secrets-engine ca_ -- --nocapture`
- `cargo test -p envctl-secretd append_revoked_client_fingerprints_writes_verifier_format -- --nocapture`
- `cargo test -p envctl-secrets-store-libsql bind_cert_row_shape -- --nocapture`
- `cargo check -p envctl-secretd --features relay-edge`
- `cargo check -p envctl-secretctl`
- `cargo build -p envctl-engine -p envctl`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace -- -D warnings`
- `bash ci/gates/no-c.sh`
- `bash ci/gates/shape.sh`
- `bash ci/gates/enable.sh`
- `bash ci/gates/p7.sh && bash ci/gates/loop-state.sh`
- `cargo test --workspace` (raw log: `/tmp/envctl-task0039-workspace-test.log`, exit=0)
- Post-rebase onto `origin/develop@e1e2726`: focused CA tests, relay-edge daemon check, p7, and
  loop-state gate passed.

## Verify On Resume

1. `gh pr view 162 --json state,mergeStateStatus,statusCheckRollup,url`
2. If checks are green and PR is not merged, run `gh pr merge 162 --auto --squash` or merge once
   branch protection permits.
3. Poll until `gh pr view 162 --json state -q .state` returns `MERGED`.
4. Only after merge:
   - `hf done TASK-0039 --pr 162`
   - `hf handoff`
   - `bash scripts/reap-worktrees.sh`
   - `bash scripts/reap-worktrees.sh --apply`
5. Re-run `hf resume --json`; expected remaining tasks after TASK-0039 merges are owner-blocked
   `TASK-0009`, owner-gated `TASK-0033`, and new buildable P0 `TASK-0053` (GitHub transport doctrine
   and scoped token path from PR #161's handoff work).

## ICM Stored

- `context-envctl`: `01KVS8ZH9DTW4C1KAFVZ9R1T2V`

## Resume Command

`/session-relay-resume from .handoff/loop/HANDOFF.md`
