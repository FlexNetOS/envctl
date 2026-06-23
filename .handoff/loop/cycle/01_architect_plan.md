VERDICT: GO

## Task
TASK-0039: finish the remote-clients CA lifecycle that remained after PR #158. PR #158 delivered
revocation-set reload/enforcement in the mTLS verifier; this cycle delivers the management
lifecycle surfaces that create, renew, and revoke short-lived client leaves.

## Target Repos
- envctl: sequential single-crew. Touched modules are linearly dependent:
  secrets-engine CA/store -> secretd Certs RPC -> secretctl surface/tests.

## Frozen Contract
- Existing proto and CLI already define `Certs.Issue`, `Certs.Renew`, `Certs.Revoke`, and
  `secretctl ca issue|renew|revoke|list`.
- Keep default dry-run/apply semantics from the proto: `RenewLeafReq.apply`; `RevokeLeafReq.apply`
  plus `confirm`.

## Design
- Add a DEK-sealed remote-clients CA distinct from the MITM CA; rebuild/zeroize it on unlock/lock
  alongside the existing local CA.
- Extend persisted cert rows with `revoked` so `ca list` reports real status.
- Clamp control-plane client leaves to a hardened remote-client max TTL of 7 days.
- Add engine methods:
  - `ca_renew(cn, apply, sink)`: dry-run logs the candidate renewal; apply reissues a fresh
    `control_plane_client` leaf for the same CN/SANs with `not_after <= now+7d`, revokes the
    superseded public row, and persists the new public cert row.
  - `ca_revoke(cn, apply, confirm, sink)`: dry-run logs candidate rows; apply requires confirm,
    marks cert rows revoked, and disables the matching remote-client row when one exists.
- Wire daemon `Certs.Renew`/`Certs.Revoke` to those engine methods. If `secretd` has
  `client_revocations_path` configured, append SHA-256 DER fingerprints for newly revoked certs
  after engine revoke succeeds so the PR #158 verifier consumes the lifecycle result.

## Verification
- Engine unit tests for issue <=7d, renew dry-run/apply, revoke apply+confirm and remote-client
  disablement.
- Daemon/CLI compile surface via existing generated proto.
- Run focused crate tests, p7/loop-state gates, and then workspace CI subset before PR.
