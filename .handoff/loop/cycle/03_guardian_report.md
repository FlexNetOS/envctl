# TASK-0039 — guardian report

## Verdict: PASS-WITH-NOTES

TASK-0039's reopened implementation gap is closed on the existing frozen Certs/secretctl surfaces:
remote client leaves now use a separate envctl-owned CA, are capped to seven days, can be renewed,
can be revoked, disable matching remote-client rows, and feed the mTLS verifier's revocation file.

## Findings

- PASS: `control_plane_client` issuance is capped at `ttl_days <= 7`; the default is 7 days.
- PASS: client leaves are signed by `env-ctl remote clients CA`, not the MITM CA.
- PASS: `Certs.Renew` and `Certs.Revoke` no longer return `Unimplemented`.
- PASS: revoke is fail-closed: dry-run by default, `--apply` requires `--confirm`.
- PASS: revoke persists cert state and disables a matching remote-client registry row.
- PASS: `secretd` appends lowercase SHA-256 DER fingerprints in the verifier format consumed by
  the PR #158 mTLS revocation loader.
- NOTE: the existing `IssueLeafReq` stream does not return private key material, so device
  enrollment packet/export remains a separate future surface if the owner wants full provisioning
  automation.

## Gate Results

All local gates passed:

- focused engine CA lifecycle tests
- focused daemon revocation-file writer test
- libSQL cert-row shape test
- relay-edge daemon check
- secretctl check
- engine+CLI build
- fmt
- clippy
- no-c / shape / enable / p7 / loop-state gates
- full workspace test
