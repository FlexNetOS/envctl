# Risk Register

- Task: `ART-125_RISK_REGISTER`
- Contract artifact: `artifact:09-governance-risk-register-md`
- Canonical path: `migration-artifacts/09-governance/risk-register.md`
- Generated at: `2026-07-04T23:19:47+00:00`
- Scope: envctl database, artifact registry, shared protocol, package execution framework, and target-filesystem artifact generation.

## Risks

| risk id | risk | owner | severity | status | mitigation |
|---|---|---|---|---|---|
| RISK-ART125-001 | Artifact path ambiguity between packet-local targets and shared contract paths could leave downstream validators looking at different files. | artifact-agent | medium | mitigating | Generate the canonical contract path and the task-local MD/JSON companion, then link all three as registry evidence. |
| RISK-ART125-002 | The envctl artifact registry currently proves behavior through an in-memory SQLite run, so durable deployment wiring may drift from package-local proof. | envctl-db-agent | high | open | Keep the artifact row, content hash, validation links, and proof record together; require the unit validation lane to replay registry checks before release. |
| RISK-ART125-003 | Proof ledger and generated status projections can lag new artifact work, making ART-125 look pending after the register is produced. | validation-agent | medium | mitigating | Append a task proof record and include this report in evidence so status rebuilds can consume a concrete completed task. |
| RISK-ART125-004 | Parallel artifact generation can create stale links if governance artifacts are produced before ownership, RACI, and readiness artifacts finish. | lane_d_filesystem | medium | monitoring | Record explicit graph links to adjacent governance artifacts and keep unresolved ownership references visible for later reconciliation. |
| RISK-ART125-005 | Security-sensitive source paths are intentionally blocked from registry evidence, which can hide context for IAM, secrets, and certificate risks. | security-reproducibility-agent | high | mitigating | Use redacted summaries and policy-safe artifact paths only; keep blocked path classes in the risk register instead of copying sensitive material. |
| RISK-ART125-006 | Nu plugin, shared protocol, and database schema surfaces can diverge if protocol validation is not rerun after artifact registry additions. | shared-protocol-agent | medium | monitoring | Retain dependency links to REQ-040 and block final unit validation until shared protocol schemas still pass. |
| RISK-ART125-007 | Rollback may remove only generated files while leaving registry proof or ledger references behind. | artifact-agent | medium | open | Use the packet rollback plan and remove proof, log, heartbeat, report, and generated artifact rows as one task-specific rollback set. |

## Validation Links

- Depends on `REQ-024_ENVCTL_ARTIFACT_REGISTRY` for registry hash persistence.
- Depends on `REQ-040_SHARED_PROTOCOL_SCHEMAS` for shared protocol compatibility.
- Blocks `VER-300_UNIT_VALIDATION` until this artifact is present, hashed, and linked.
