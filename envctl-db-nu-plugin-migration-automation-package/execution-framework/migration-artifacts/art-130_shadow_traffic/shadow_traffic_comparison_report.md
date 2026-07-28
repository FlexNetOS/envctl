# Shadow traffic comparison report

**Artifact ID:** ART-130_SHADOW_TRAFFIC  
**Status:** `not_started` — this worker received no mounted target descriptor, repository scan, envctl database, or captured shadow-traffic observations.  
**Generated:** 2026-07-28T09:54:55Z  
**Dependencies:** REQ-024_ENVCTL_ARTIFACT_REGISTRY, REQ-040_SHARED_PROTOCOL_SCHEMAS

## Purpose

Compare the legacy (old) and migration-target (new) implementations for mirrored production requests without allowing the new implementation's response to affect the client. This report is the evidence record for deciding whether shadow traffic can advance to a gated rollout.

## Scope and safety boundary

- Mirror only requests approved by the target descriptor and data-classification policy.
- The old path remains authoritative for every client response; new-path outputs are observation-only.
- Preserve request correlation through a stable `comparison_id`; redact or tokenize sensitive fields before storing evidence.
- Do not mirror non-idempotent writes unless the new path is explicitly configured for dry-run/sandbox execution.
- Exclude endpoints, tenants, and payload classes listed by the envctl registry policy.

## Required inputs before execution

| Input | Required evidence | State |
| --- | --- | --- |
| Target descriptor | old/new routes, eligible operations, sample window | unavailable |
| Repository scan | normalizers, comparators, known intentional deltas | unavailable |
| envctl database | environment, registry key, redaction and exclusion policy | unavailable |
| Shared protocol schemas | schema version and compatibility rules | unavailable |
| Captured comparisons | sampled records and aggregate counters | unavailable |

No behavioural conclusion is valid until these inputs are attached and the JSON report is updated with observed values.

## Comparison contract

Each eligible mirrored request produces one record containing:

- `comparison_id`, timestamp, route/operation, tenant cohort, and schema version;
- old and new outcome class, HTTP/protocol status, latency, and normalized response fingerprint;
- comparator result: `match`, `expected_delta`, `mismatch`, `old_error`, `new_error`, `timeout`, or `excluded`;
- a redacted evidence pointer (never a raw secret or unredacted payload);
- an approved-delta identifier whenever the normalizer intentionally accepts a difference.

Compare semantic response fields after protocol-schema normalization. Ignore only fields with a documented reason, such as server timestamps, generated IDs, trace IDs, response ordering where unordered by contract, or approved migration metadata. Treat authorization decisions, status/outcome class, required response fields, monetary values, persistence effects, and latency regressions as material.

## Gate criteria

| Gate | Pass condition | Current result |
| --- | --- | --- |
| Coverage | Eligible routes/cohorts and observation window meet descriptor minimums | pending |
| Correctness | Zero unapproved material mismatches | pending |
| Reliability | No new-path error or timeout regression beyond approved threshold | pending |
| Performance | New-path latency meets approved SLO/regression threshold | pending |
| Data safety | Redaction, exclusions, and write isolation validated | pending |
| Schema | Every observed record validates against the shared protocol schema | pending |

**Decision:** `pending_evidence`. Do not promote, route, or cut over traffic from this artifact alone.

## Execution and escalation

1. Resolve the eligible operation list and policy from envctl.
2. Mirror sampled real traffic asynchronously with the old path serving the client.
3. Normalize old/new outputs using the shared protocol schema, then classify each pair.
4. Aggregate by route, cohort, status family, and schema version; retain only redacted evidence references.
5. Open a defect for each unapproved material mismatch, new-only error, timeout, or SLO breach; link its ID in the JSON report.
6. Obtain owner approval only after all gate criteria pass for the required window.

## Rollback / stop conditions

Immediately stop mirroring the affected cohort and preserve redacted evidence if there is data exposure, a write-isolation failure, a critical authorization mismatch, or an error/timeout regression beyond the configured threshold. The legacy path remains client-serving throughout. Remove only this artifact's files to roll back the generated documentation, following `history/pre_execution_framework_manifest.json` when it is available.

