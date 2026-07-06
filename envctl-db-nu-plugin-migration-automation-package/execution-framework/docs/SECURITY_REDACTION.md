# Security Redaction And Reproducibility Controls

Control: `envctl_nu_plugin_security_redaction`
Schema version: `1.0`
Validation status: `passed`

## Capture Rules

- Redact command strings, log lines, proof fields, scan payloads, and packet payloads before persistence.
- Persist raw evidence only under approved evidence locations and only when the evidence record marks redacted=false.
- Mark every evidence record with a redacted boolean and sha256 hash.
- Hash generated artifacts and evidence files used by proof records.
- Require approval for execute-full replay or any R3+ operation.
- Do not scan or persist blocked secret-bearing paths.

## Scan Surfaces

- `execution-framework/logs/**/*.log`
- `execution-framework/proof_records/**/*.json`
- `execution-framework/generated/execution_packets/**/*.json`
- `execution-framework/generated/**/*scan*.json`
- `execution-framework/generated/**/*manifest*.json`
- `execution-framework/generated/**/*validation*.json`
- `execution-framework/state/**/*.json`

## Blocked Paths

- `**/.env`
- `**/secrets/**`
- `**/private_keys/**`
- `**/*.pem`
- `**/*.key`

## Reproducibility Identity

- `target_descriptor_hash`
- `artifact_contract_hash`
- `recipe_hash`
- `package_hashes`
- `collector_versions`
- `tool_versions`
- `operation_input_hashes`
- `evidence_hashes`
- `artifact_hashes`
- `approval_decision_hashes`

## Replay Modes

| mode | behavior |
|---|---|
| `verify-only` | Recompute hashes and confirm artifacts/evidence still match. |
| `dry-run-plan` | Reconstruct operation plan without executing target-affecting commands. |
| `execute-safe` | Re-run R0-R2 operations. |
| `execute-full` | Requires explicit approval for R3+. |

## Verification

- Files scanned: `244`
- Pre-redaction findings: `90`
- Sanitized files: `1`
- Redaction fixtures passed: `5`
- Secret findings: `0`
- Blocked path probes passed: `5`
- Chain hash sample: `d32d9f0a6cf3b892b7936dfe68936cf4d14be91834fdbc52c9fcc9f6a8ce0858`
