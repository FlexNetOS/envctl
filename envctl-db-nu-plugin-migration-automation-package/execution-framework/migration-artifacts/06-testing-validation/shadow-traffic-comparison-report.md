# Shadow Traffic Comparison Report

**Task:** ART-130_SHADOW_TRAFFIC  
**Target:** `flexnetos-vs-lifeos` — `/home/flexnetos/FlexNetOS` compared with `/home/flexnetos/lifeos`  
**Report status:** complete with capture gap  
**Runtime parity decision:** pending evidence (not certified)

## Result

No real mirrored request/response traffic capture was supplied in the mounted target descriptor, repository scan, or envctl database inputs. Consequently, this report does not claim that old and new behavior are equivalent. The comparison gate remains fail-closed until an approved capture is processed.

## Comparison Contract

For every approved shadow request, correlate the old and new executions using a redacted correlation ID and record:

| Dimension | Comparison rule | Gate |
|---|---|---|
| Route and method | Exact match after approved route normalization | mismatch fails sample |
| Response class | Exact match | mismatch fails sample |
| Business payload | Canonicalized, schema-aware equality after approved volatile-field masking | mismatch fails sample |
| Side effects | New path must be non-authoritative; old path remains the source of truth | any new-path write fails run |
| Latency | Record old/new duration and delta; evaluate against the approved SLO | threshold breach requires review |
| Errors | Compare normalized error class and stable code | mismatch fails sample |

Capture data must be redacted before storage. Never persist credentials, session tokens, authorization headers, private keys, or raw sensitive payload fields in this artifact.

## Evidence State

| Evidence | State |
|---|---|
| Target descriptor (`flexnetos-vs-lifeos`) | available |
| Repository scan | available |
| envctl migration database/model | available |
| Artifact-registry dependency (REQ-024) | passed |
| Shared-protocol dependency (REQ-040) | passed |
| Real shadow traffic capture | not present |
| Per-request comparison results | not started |

## Required Capture Before Certification

1. Mirror approved production-like traffic to the new path without letting it produce authoritative side effects.
2. Emit one redacted, correlated record per old/new pair with request fingerprint, normalized responses, errors, timings, and side-effect disposition.
3. Define and version the volatile-field masks, canonicalization rules, sample window, and SLO thresholds.
4. Re-run this report with nonzero sample counts and resolve every non-allowlisted mismatch.
5. Record final validation evidence in envctl before promoting the new path.

## Registration

The task-scoped Markdown and JSON artifacts are registered as `art-130-shadow-traffic-report-md` and `art-130-shadow-traffic-report-json`. The required contract artifact is `06-testing-validation-shadow-traffic-comparison-report-md`; its producer operation is `produce-06-testing-validation-shadow-traffic-comparison-report-md`. Registry linkage is evidenced by `REQ-024_ENVCTL_ARTIFACT_REGISTRY`; protocol compatibility is evidenced by `REQ-040_SHARED_PROTOCOL_SCHEMAS`.

