# Any-target extension specification

The FlexNetOS/lifeos case is only the first target. envctl must support arbitrary migration targets.

## Target descriptor required fields

```yaml
schema_version: 1
target_id: stable-human-readable-id
target_type: codebase | data | infrastructure | integration | mixed
primary_root: /absolute/or/repo-relative/path
compare_root: /optional/path
output_root: migration-artifacts
include:
  - "**/*"
exclude:
  - ".git/**"
  - "node_modules/**"
  - "target/**"
safety:
  default_mode: approval-gated
  max_auto_risk: R2
  allow_network: false
  allow_destructive: false
collectors:
  filesystem: true
  git: true
  package_managers: true
  databases: false
  infrastructure: true
  apis: true
artifact_contract:
  name: full-migration-artifact-contract
  version: 1
recipe:
  name: full-migration-discovery-and-execution
  version: 1
```

## Adapter requirements

A target adapter must define:

- what can be scanned
- what commands are safe
- how evidence is collected
- how secrets are redacted
- what artifact templates apply
- what validation rules apply
- how rollback/checkpoints are represented

## Collector capability map

```text
filesystem
repo/git
language/package manager
build/test
runtime entrypoints
config/secrets references
api contracts
event/message contracts
database schema
data lineage
infra/IaC
observability
security/IAM
business process metadata
```

## Generic migration applicability

envctl can apply the migration design to any target only when it has:

1. A target descriptor.
2. An artifact contract.
3. A migration recipe.
4. A safety policy.
5. Collector capability map.
6. Evidence redaction policy.
7. Validation policy.
8. Output/export policy.
9. Replay policy.
10. Human involvement mode.
