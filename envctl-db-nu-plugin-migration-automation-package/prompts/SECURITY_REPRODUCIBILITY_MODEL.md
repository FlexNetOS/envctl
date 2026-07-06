# Security and reproducibility model

## Security requirements

- Redact secrets before artifact persistence.
- Store command strings in redacted form when they may contain credentials.
- Store raw evidence only in approved evidence locations.
- Mark evidence as redacted/unredacted.
- Hash every evidence file and generated artifact.
- Approval-gate risky operations.
- Never bypass sandbox/approval controls.
- Keep plugin mutating commands auditable.
- Prevent agents from writing directly to production targets without policy.

## Reproducibility identity

A migration run identity must include:

```text
target_descriptor_hash
artifact_contract_hash
recipe_hash
package_hashes
collector_versions
tool_versions
operation_input_hashes
evidence_hashes
artifact_hashes
approval_decision_hashes
```

## Replay modes

| Mode | Behavior |
|---|---|
| `verify-only` | Recompute hashes and confirm artifacts/evidence still match. |
| `dry-run-plan` | Reconstruct operation plan without executing target-affecting commands. |
| `execute-safe` | Re-run R0-R2 operations. |
| `execute-full` | Requires explicit approval for R3+. |

## Chain integrity

Run events should be chained:

```text
event_hash = sha256(previous_event_hash + canonical_event_json)
```

If the repo already uses a different integrity model, preserve it and map this requirement to the native model.
