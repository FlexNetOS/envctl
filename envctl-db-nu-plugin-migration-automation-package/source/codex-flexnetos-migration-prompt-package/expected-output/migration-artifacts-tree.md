# Expected Migration Artifact Tree

Codex must create this tree under the selected repository/workspace. Files may contain `status: unknown` or `status: partial` when evidence is insufficient, but the files must exist and must be linked from `index.md`.

```text
migration-artifacts/
├── MIGRATION_MEMORY.md
├── index.md
├── wiki-home.md
├── artifact-manifest.json
├── artifact-manifest.md
├── evidence-register.md
├── link-graph.md
├── _meta/
│   ├── run-context.md
│   ├── scan-runs.jsonl
│   └── artifact-status.tsv
├── _raw/
├── _spark/
│   ├── spark-filesystem-repo.md
│   ├── spark-filesystem-repo.json
│   ├── spark-toolchain-deps.md
│   ├── spark-toolchain-deps.json
│   ├── spark-code-runtime-debug.md
│   ├── spark-code-runtime-debug.json
│   ├── spark-data-schema-lineage.md
│   ├── spark-data-schema-lineage.json
│   ├── spark-infra-security-obs.md
│   ├── spark-infra-security-obs.json
│   ├── spark-integrations-contracts.md
│   ├── spark-integrations-contracts.json
│   ├── spark-migration-controls.md
│   ├── spark-migration-controls.json
│   ├── spark-flexnetos-investigator.md
│   └── spark-flexnetos-investigator.json
├── 00-executive-summary/
│   ├── executive-summary.md
│   ├── flexnetos-purpose-summary.md
│   ├── model-resolution-blocker.md
│   └── migration-readiness-summary.md
├── 01-current-state/
│   ├── system-inventory.md
│   ├── architecture-current.md
│   ├── dependency-graph.md
│   ├── data-flow-current.md
│   ├── risk-hotspots.md
│   ├── directory-tree.md
│   ├── repository-map.md
│   ├── application-service-dependency-graph.md
│   ├── runtime-dependency-map.md
│   ├── environment-matrix.md
│   ├── configuration-inventory.md
│   ├── business-process-map.md
│   ├── blast-radius-map.md
│   ├── exception-inventory.md
│   ├── flexnetos-vs-lifeos-evidence.md
│   ├── flexnetos-path-resolution.md
│   └── flexnetos-reference-index.md
├── 02-target-state/
│   ├── architecture-target.md
│   ├── platform-design.md
│   ├── security-model.md
│   ├── operating-model.md
│   ├── backward-compatibility-plan.md
│   └── deprecation-map.md
├── 03-code-analysis/
│   ├── repo-map.md
│   ├── directory-tree.md
│   ├── package-dependencies.md
│   ├── call-graph.md
│   ├── dead-code-report.md
│   ├── codebase-hierarchy-graph.md
│   ├── import-dependency-graph.md
│   ├── hotspot-map.md
│   ├── build-graph.md
│   ├── runtime-entrypoint-map.md
│   ├── compatibility-matrix.md
│   ├── code-map-for-debugging.md
│   ├── toolchain-dependency-tree.md
│   ├── package-library-dependency-graph.md
│   ├── technical-debt-ledger.md
│   └── flexnetos-entrypoints.md
├── 04-data-migration/
│   ├── schema-map.md
│   ├── source-target-mapping.md
│   ├── transformation-rules.md
│   ├── data-quality-report.md
│   ├── reconciliation-plan.md
│   ├── database-schema-map.md
│   ├── data-lineage-map.md
│   ├── data-flow-graph.md
│   ├── data-quality-profile.md
│   ├── reconciliation-report.md
│   ├── schema-diff-report.md
│   ├── critical-field-inventory.md
│   ├── backfill-plan.md
│   ├── incremental-sync-plan.md
│   ├── data-retention-compliance-map.md
│   └── golden-dataset.md
├── 05-integrations/
│   ├── api-catalog.md
│   ├── event-catalog.md
│   ├── third-party-dependencies.md
│   ├── auth-flows.md
│   ├── api-contract-catalog.md
│   ├── api-contract-map.md
│   ├── event-message-contract-map.md
│   ├── webhook-event-map.md
│   ├── integration-catalog.md
│   ├── third-party-dependency-register.md
│   ├── auth-flow-diagram.md
│   ├── failure-mode-map.md
│   ├── consumer-map.md
│   └── flexnetos-contracts.md
├── 06-testing-validation/
│   ├── test-strategy.md
│   ├── regression-matrix.md
│   ├── performance-baseline.md
│   ├── validation-evidence.md
│   ├── validation-reconciliation-reports.md
│   ├── test-coverage-matrix.md
│   ├── shadow-traffic-comparison-report.md
│   └── parity-dashboard.md
├── 07-cutover/
│   ├── wave-plan.md
│   ├── migration-wave-plan.md
│   ├── cutover-checklist.md
│   ├── rollback-plan.md
│   ├── communication-plan.md
│   └── decommission-plan.md
├── 08-operations/
│   ├── observability-map.md
│   ├── runbooks.md
│   ├── alerting.md
│   ├── incident-response.md
│   ├── infrastructure-topology-map.md
│   ├── network-dependency-map.md
│   ├── environment-parity-matrix.md
│   ├── resource-inventory.md
│   ├── iac-coverage-report.md
│   ├── secrets-certificates-inventory.md
│   ├── capacity-baseline.md
│   ├── cost-baseline-forecast.md
│   └── dr-backup-map.md
└── 09-governance/
    ├── decision-log.md
    ├── risk-register.md
    ├── ownership-matrix.md
    ├── code-ownership-map.md
    ├── iam-security-access-matrix.md
    ├── security-control-matrix.md
    ├── ownership-raci-matrix.md
    ├── migration-readiness-scorecard.md
    ├── business-capability-map.md
    └── flexnetos-open-questions.md
```
