# Contract Manifest

Contract: `full-migration-artifact-contract`
Version: `1.0.0`
Hash: `9016f9a579812c51a10f1852279392b48521c31e958cddb8bf92361090ca9d07`
Source package: `codex-flexnetos-migration-prompt-package`

## Summary

- Required artifact rows: 136
- Automation rows: 16
- Recipe phases: 14
- Validation status: passed

## Required Sources

- `README.md`: `59cc1713a2f021dcd3b29cc005d1b86d57deb2527d07d2285c3e7ebd1c783720`
- `source/previous-migration-artifact-context.md`: `8db7e998ad79411c4ef577ff3bbf49ee3fcc357873c54efb372ddbaf7f076e94`
- `source/codex-flexnetos-migration-prompt-package/expected-output/migration-artifacts-tree.md`: `927c559ce4c5ac90f4d6f0a709d823fcdf68a676c63c2ea99af2a14df53e94cd`
- `expected-output/migration-automation-artifacts.md`: `9742609a0ef407f8ed5969481b67357ab080493f602458d4b352edb7f7755fbb`
- `prompts/UTILIZE_FLEXNETOS_PACKAGE.md`: `2a8f66bee3f441a29a789fb5c47a67067f9b07e0c29619518b6793006d653393`
- `prompts/STRATEGY_DECISION.md`: `c1846e51d42216f47e9c346d2301c9769adfb3a00f5595618d077ab9270c03bf`
- `prompts/MASTER_PROMPT_ENVCTL_DB_NU_PLUGIN.md`: `ac66a1ffd6f0ae3c7c67d7cde08593da60c3e5a3a0c30a4f5cc79ef2a8299830`
- `sql/003_seed_artifact_contract.sql`: `856caa810f5422921245d78d1d1de1201abf05372ca1f2dbe32ae7450c29a2c1`

## Contract Rows

| row | artifact | path | task |
|---|---|---|---|
| `artifact:migration-memory-md` | Migration Memory | `migration-artifacts/MIGRATION_MEMORY.md` | `` |
| `artifact:index-md` | Index | `migration-artifacts/index.md` | `` |
| `artifact:wiki-home-md` | Wiki Home | `migration-artifacts/wiki-home.md` | `` |
| `artifact:artifact-manifest-json` | Artifact Manifest | `migration-artifacts/artifact-manifest.json` | `` |
| `artifact:artifact-manifest-md` | Artifact Manifest | `migration-artifacts/artifact-manifest.md` | `` |
| `artifact:evidence-register-md` | Evidence Register | `migration-artifacts/evidence-register.md` | `` |
| `artifact:link-graph-md` | Link Graph | `migration-artifacts/link-graph.md` | `` |
| `artifact:meta-run-context-md` | Run Context | `migration-artifacts/_meta/run-context.md` | `` |
| `artifact:meta-scan-runs-jsonl` | Scan Runs | `migration-artifacts/_meta/scan-runs.jsonl` | `` |
| `artifact:meta-artifact-status-tsv` | Artifact Status | `migration-artifacts/_meta/artifact-status.tsv` | `` |
| `artifact:spark-spark-filesystem-repo-md` | Spark Filesystem Repo | `migration-artifacts/_spark/spark-filesystem-repo.md` | `` |
| `artifact:spark-spark-filesystem-repo-json` | Spark Filesystem Repo | `migration-artifacts/_spark/spark-filesystem-repo.json` | `` |
| `artifact:spark-spark-toolchain-deps-md` | Spark Toolchain Deps | `migration-artifacts/_spark/spark-toolchain-deps.md` | `` |
| `artifact:spark-spark-toolchain-deps-json` | Spark Toolchain Deps | `migration-artifacts/_spark/spark-toolchain-deps.json` | `` |
| `artifact:spark-spark-code-runtime-debug-md` | Spark Code Runtime Debug | `migration-artifacts/_spark/spark-code-runtime-debug.md` | `` |
| `artifact:spark-spark-code-runtime-debug-json` | Spark Code Runtime Debug | `migration-artifacts/_spark/spark-code-runtime-debug.json` | `` |
| `artifact:spark-spark-data-schema-lineage-md` | Spark Data Schema Lineage | `migration-artifacts/_spark/spark-data-schema-lineage.md` | `` |
| `artifact:spark-spark-data-schema-lineage-json` | Spark Data Schema Lineage | `migration-artifacts/_spark/spark-data-schema-lineage.json` | `` |
| `artifact:spark-spark-infra-security-obs-md` | Spark Infra Security Obs | `migration-artifacts/_spark/spark-infra-security-obs.md` | `` |
| `artifact:spark-spark-infra-security-obs-json` | Spark Infra Security Obs | `migration-artifacts/_spark/spark-infra-security-obs.json` | `` |
| `artifact:spark-spark-integrations-contracts-md` | Spark Integrations Contracts | `migration-artifacts/_spark/spark-integrations-contracts.md` | `` |
| `artifact:spark-spark-integrations-contracts-json` | Spark Integrations Contracts | `migration-artifacts/_spark/spark-integrations-contracts.json` | `` |
| `artifact:spark-spark-migration-controls-md` | Spark Migration Controls | `migration-artifacts/_spark/spark-migration-controls.md` | `` |
| `artifact:spark-spark-migration-controls-json` | Spark Migration Controls | `migration-artifacts/_spark/spark-migration-controls.json` | `` |
| `artifact:spark-spark-flexnetos-investigator-md` | Spark Flexnetos Investigator | `migration-artifacts/_spark/spark-flexnetos-investigator.md` | `` |
| `artifact:spark-spark-flexnetos-investigator-json` | Spark Flexnetos Investigator | `migration-artifacts/_spark/spark-flexnetos-investigator.json` | `` |
| `artifact:00-executive-summary-executive-summary-md` | Executive Summary | `migration-artifacts/00-executive-summary/executive-summary.md` | `` |
| `artifact:00-executive-summary-flexnetos-purpose-summary-md` | Flexnetos Purpose Summary | `migration-artifacts/00-executive-summary/flexnetos-purpose-summary.md` | `` |
| `artifact:00-executive-summary-model-resolution-blocker-md` | Model Resolution Blocker | `migration-artifacts/00-executive-summary/model-resolution-blocker.md` | `` |
| `artifact:00-executive-summary-migration-readiness-summary-md` | Migration Readiness Summary | `migration-artifacts/00-executive-summary/migration-readiness-summary.md` | `` |
| `artifact:01-current-state-system-inventory-md` | System Inventory | `migration-artifacts/01-current-state/system-inventory.md` | `ART-100_SYSTEM_INVENTORY` |
| `artifact:01-current-state-architecture-current-md` | Architecture Current | `migration-artifacts/01-current-state/architecture-current.md` | `` |
| `artifact:01-current-state-dependency-graph-md` | Dependency Graph | `migration-artifacts/01-current-state/dependency-graph.md` | `ART-103_SERVICE_DEP_GRAPH` |
| `artifact:01-current-state-data-flow-current-md` | Data Flow Current | `migration-artifacts/01-current-state/data-flow-current.md` | `ART-107_DATA_FLOW_GRAPH` |
| `artifact:01-current-state-risk-hotspots-md` | Risk Hotspots | `migration-artifacts/01-current-state/risk-hotspots.md` | `` |
| `artifact:01-current-state-directory-tree-md` | Directory Tree | `migration-artifacts/01-current-state/directory-tree.md` | `ART-101_DIRECTORY_TREE` |
| `artifact:01-current-state-repository-map-md` | Repository Map | `migration-artifacts/01-current-state/repository-map.md` | `ART-102_REPOSITORY_MAP` |
| `artifact:01-current-state-application-service-dependency-graph-md` | Application Service Dependency Graph | `migration-artifacts/01-current-state/application-service-dependency-graph.md` | `ART-103_SERVICE_DEP_GRAPH` |
| `artifact:01-current-state-runtime-dependency-map-md` | Runtime Dependency Map | `migration-artifacts/01-current-state/runtime-dependency-map.md` | `ART-106_RUNTIME_DEP_MAP` |
| `artifact:01-current-state-environment-matrix-md` | Environment Matrix | `migration-artifacts/01-current-state/environment-matrix.md` | `ART-114_ENV_CONFIG_MATRIX` |
| `artifact:01-current-state-configuration-inventory-md` | Configuration Inventory | `migration-artifacts/01-current-state/configuration-inventory.md` | `ART-115_CONFIG_INVENTORY` |
| `artifact:01-current-state-business-process-map-md` | Business Process Map | `migration-artifacts/01-current-state/business-process-map.md` | `ART-119_BUSINESS_PROCESS` |
| `artifact:01-current-state-blast-radius-map-md` | Blast Radius Map | `migration-artifacts/01-current-state/blast-radius-map.md` | `ART-127_BLAST_RADIUS` |
| `artifact:01-current-state-exception-inventory-md` | Exception Inventory | `migration-artifacts/01-current-state/exception-inventory.md` | `ART-134_EXCEPTION_INVENTORY` |
| `artifact:01-current-state-flexnetos-vs-lifeos-evidence-md` | Flexnetos Vs Lifeos Evidence | `migration-artifacts/01-current-state/flexnetos-vs-lifeos-evidence.md` | `` |
| `artifact:01-current-state-flexnetos-path-resolution-md` | Flexnetos Path Resolution | `migration-artifacts/01-current-state/flexnetos-path-resolution.md` | `` |
| `artifact:01-current-state-flexnetos-reference-index-md` | Flexnetos Reference Index | `migration-artifacts/01-current-state/flexnetos-reference-index.md` | `` |
| `artifact:02-target-state-architecture-target-md` | Architecture Target | `migration-artifacts/02-target-state/architecture-target.md` | `` |
| `artifact:02-target-state-platform-design-md` | Platform Design | `migration-artifacts/02-target-state/platform-design.md` | `` |
| `artifact:02-target-state-security-model-md` | Security Model | `migration-artifacts/02-target-state/security-model.md` | `` |
| `artifact:02-target-state-operating-model-md` | Operating Model | `migration-artifacts/02-target-state/operating-model.md` | `` |
| `artifact:02-target-state-backward-compatibility-plan-md` | Backward Compatibility Plan | `migration-artifacts/02-target-state/backward-compatibility-plan.md` | `` |
| `artifact:02-target-state-deprecation-map-md` | Deprecation Map | `migration-artifacts/02-target-state/deprecation-map.md` | `ART-133_DEPRECATION_MAP` |
| `artifact:03-code-analysis-repo-map-md` | Repo Map | `migration-artifacts/03-code-analysis/repo-map.md` | `ART-102_REPOSITORY_MAP` |
| `artifact:03-code-analysis-directory-tree-md` | Directory Tree | `migration-artifacts/03-code-analysis/directory-tree.md` | `ART-101_DIRECTORY_TREE` |
| `artifact:03-code-analysis-package-dependencies-md` | Package Dependencies | `migration-artifacts/03-code-analysis/package-dependencies.md` | `ART-105_PACKAGE_LIB_GRAPH` |
| `artifact:03-code-analysis-call-graph-md` | Call Graph | `migration-artifacts/03-code-analysis/call-graph.md` | `` |
| `artifact:03-code-analysis-dead-code-report-md` | Dead Code Report | `migration-artifacts/03-code-analysis/dead-code-report.md` | `` |
| `artifact:03-code-analysis-codebase-hierarchy-graph-md` | Codebase Hierarchy Graph | `migration-artifacts/03-code-analysis/codebase-hierarchy-graph.md` | `` |
| `artifact:03-code-analysis-import-dependency-graph-md` | Import Dependency Graph | `migration-artifacts/03-code-analysis/import-dependency-graph.md` | `ART-103_SERVICE_DEP_GRAPH` |
| `artifact:03-code-analysis-hotspot-map-md` | Hotspot Map | `migration-artifacts/03-code-analysis/hotspot-map.md` | `` |
| `artifact:03-code-analysis-build-graph-md` | Build Graph | `migration-artifacts/03-code-analysis/build-graph.md` | `` |
| `artifact:03-code-analysis-runtime-entrypoint-map-md` | Runtime Entrypoint Map | `migration-artifacts/03-code-analysis/runtime-entrypoint-map.md` | `` |
| `artifact:03-code-analysis-compatibility-matrix-md` | Compatibility Matrix | `migration-artifacts/03-code-analysis/compatibility-matrix.md` | `` |
| `artifact:03-code-analysis-code-map-for-debugging-md` | Code Map For Debugging | `migration-artifacts/03-code-analysis/code-map-for-debugging.md` | `ART-113_DEBUG_CODE_MAP` |
| `artifact:03-code-analysis-toolchain-dependency-tree-md` | Toolchain Dependency Tree | `migration-artifacts/03-code-analysis/toolchain-dependency-tree.md` | `ART-104_TOOLCHAIN_TREE` |
| `artifact:03-code-analysis-package-library-dependency-graph-md` | Package Library Dependency Graph | `migration-artifacts/03-code-analysis/package-library-dependency-graph.md` | `ART-103_SERVICE_DEP_GRAPH` |
| `artifact:03-code-analysis-technical-debt-ledger-md` | Technical Debt Ledger | `migration-artifacts/03-code-analysis/technical-debt-ledger.md` | `ART-136_TECH_DEBT_LEDGER` |
| `artifact:03-code-analysis-flexnetos-entrypoints-md` | Flexnetos Entrypoints | `migration-artifacts/03-code-analysis/flexnetos-entrypoints.md` | `` |
| `artifact:04-data-migration-schema-map-md` | Schema Map | `migration-artifacts/04-data-migration/schema-map.md` | `ART-108_DB_SCHEMA_MAP` |
| `artifact:04-data-migration-source-target-mapping-md` | Source Target Mapping | `migration-artifacts/04-data-migration/source-target-mapping.md` | `` |
| `artifact:04-data-migration-transformation-rules-md` | Transformation Rules | `migration-artifacts/04-data-migration/transformation-rules.md` | `` |
| `artifact:04-data-migration-data-quality-report-md` | Data Quality Report | `migration-artifacts/04-data-migration/data-quality-report.md` | `` |
| `artifact:04-data-migration-reconciliation-plan-md` | Reconciliation Plan | `migration-artifacts/04-data-migration/reconciliation-plan.md` | `` |
| `artifact:04-data-migration-database-schema-map-md` | Database Schema Map | `migration-artifacts/04-data-migration/database-schema-map.md` | `ART-108_DB_SCHEMA_MAP` |
| `artifact:04-data-migration-data-lineage-map-md` | Data Lineage Map | `migration-artifacts/04-data-migration/data-lineage-map.md` | `ART-109_DATA_LINEAGE` |
| `artifact:04-data-migration-data-flow-graph-md` | Data Flow Graph | `migration-artifacts/04-data-migration/data-flow-graph.md` | `ART-107_DATA_FLOW_GRAPH` |
| `artifact:04-data-migration-data-quality-profile-md` | Data Quality Profile | `migration-artifacts/04-data-migration/data-quality-profile.md` | `` |
| `artifact:04-data-migration-reconciliation-report-md` | Reconciliation Report | `migration-artifacts/04-data-migration/reconciliation-report.md` | `` |
| `artifact:04-data-migration-schema-diff-report-md` | Schema Diff Report | `migration-artifacts/04-data-migration/schema-diff-report.md` | `` |
| `artifact:04-data-migration-critical-field-inventory-md` | Critical Field Inventory | `migration-artifacts/04-data-migration/critical-field-inventory.md` | `` |
| `artifact:04-data-migration-backfill-plan-md` | Backfill Plan | `migration-artifacts/04-data-migration/backfill-plan.md` | `` |
| `artifact:04-data-migration-incremental-sync-plan-md` | Incremental Sync Plan | `migration-artifacts/04-data-migration/incremental-sync-plan.md` | `` |
| `artifact:04-data-migration-data-retention-compliance-map-md` | Data Retention Compliance Map | `migration-artifacts/04-data-migration/data-retention-compliance-map.md` | `` |
| `artifact:04-data-migration-golden-dataset-md` | Golden Dataset | `migration-artifacts/04-data-migration/golden-dataset.md` | `ART-131_GOLDEN_DATASET` |
| `artifact:05-integrations-api-catalog-md` | Api Catalog | `migration-artifacts/05-integrations/api-catalog.md` | `ART-110_API_CATALOG` |
| `artifact:05-integrations-event-catalog-md` | Event Catalog | `migration-artifacts/05-integrations/event-catalog.md` | `ART-111_EVENT_MAP` |
| `artifact:05-integrations-third-party-dependencies-md` | Third Party Dependencies | `migration-artifacts/05-integrations/third-party-dependencies.md` | `` |
| `artifact:05-integrations-auth-flows-md` | Auth Flows | `migration-artifacts/05-integrations/auth-flows.md` | `` |
| `artifact:05-integrations-api-contract-catalog-md` | Api Contract Catalog | `migration-artifacts/05-integrations/api-contract-catalog.md` | `ART-110_API_CATALOG` |
| `artifact:05-integrations-api-contract-map-md` | Api Contract Map | `migration-artifacts/05-integrations/api-contract-map.md` | `ART-110_API_CATALOG` |
| `artifact:05-integrations-event-message-contract-map-md` | Event Message Contract Map | `migration-artifacts/05-integrations/event-message-contract-map.md` | `ART-111_EVENT_MAP` |
| `artifact:05-integrations-webhook-event-map-md` | Webhook Event Map | `migration-artifacts/05-integrations/webhook-event-map.md` | `` |
| `artifact:05-integrations-integration-catalog-md` | Integration Catalog | `migration-artifacts/05-integrations/integration-catalog.md` | `` |
| `artifact:05-integrations-third-party-dependency-register-md` | Third Party Dependency Register | `migration-artifacts/05-integrations/third-party-dependency-register.md` | `` |
| `artifact:05-integrations-auth-flow-diagram-md` | Auth Flow Diagram | `migration-artifacts/05-integrations/auth-flow-diagram.md` | `` |
| `artifact:05-integrations-failure-mode-map-md` | Failure Mode Map | `migration-artifacts/05-integrations/failure-mode-map.md` | `` |
| `artifact:05-integrations-consumer-map-md` | Consumer Map | `migration-artifacts/05-integrations/consumer-map.md` | `` |
| `artifact:05-integrations-flexnetos-contracts-md` | Flexnetos Contracts | `migration-artifacts/05-integrations/flexnetos-contracts.md` | `` |
| `artifact:06-testing-validation-test-strategy-md` | Test Strategy | `migration-artifacts/06-testing-validation/test-strategy.md` | `` |
| `artifact:06-testing-validation-regression-matrix-md` | Regression Matrix | `migration-artifacts/06-testing-validation/regression-matrix.md` | `` |
| `artifact:06-testing-validation-performance-baseline-md` | Performance Baseline | `migration-artifacts/06-testing-validation/performance-baseline.md` | `` |
| `artifact:06-testing-validation-validation-evidence-md` | Validation Evidence | `migration-artifacts/06-testing-validation/validation-evidence.md` | `` |
| `artifact:06-testing-validation-validation-reconciliation-reports-md` | Validation Reconciliation Reports | `migration-artifacts/06-testing-validation/validation-reconciliation-reports.md` | `ART-123_VALIDATION_RECONCILIATION` |
| `artifact:06-testing-validation-test-coverage-matrix-md` | Test Coverage Matrix | `migration-artifacts/06-testing-validation/test-coverage-matrix.md` | `ART-124_TEST_COVERAGE` |
| `artifact:06-testing-validation-shadow-traffic-comparison-report-md` | Shadow Traffic Comparison Report | `migration-artifacts/06-testing-validation/shadow-traffic-comparison-report.md` | `ART-130_SHADOW_TRAFFIC` |
| `artifact:06-testing-validation-parity-dashboard-md` | Parity Dashboard | `migration-artifacts/06-testing-validation/parity-dashboard.md` | `ART-132_PARITY_DASHBOARD` |
| `artifact:07-cutover-wave-plan-md` | Wave Plan | `migration-artifacts/07-cutover/wave-plan.md` | `ART-120_WAVE_PLAN` |
| `artifact:07-cutover-migration-wave-plan-md` | Migration Wave Plan | `migration-artifacts/07-cutover/migration-wave-plan.md` | `ART-120_WAVE_PLAN` |
| `artifact:07-cutover-cutover-checklist-md` | Cutover Checklist | `migration-artifacts/07-cutover/cutover-checklist.md` | `ART-121_CUTOVER` |
| `artifact:07-cutover-rollback-plan-md` | Rollback Plan | `migration-artifacts/07-cutover/rollback-plan.md` | `ART-122_ROLLBACK` |
| `artifact:07-cutover-communication-plan-md` | Communication Plan | `migration-artifacts/07-cutover/communication-plan.md` | `` |
| `artifact:07-cutover-decommission-plan-md` | Decommission Plan | `migration-artifacts/07-cutover/decommission-plan.md` | `` |
| `artifact:08-operations-observability-map-md` | Observability Map | `migration-artifacts/08-operations/observability-map.md` | `ART-118_OBSERVABILITY` |
| `artifact:08-operations-runbooks-md` | Runbooks | `migration-artifacts/08-operations/runbooks.md` | `` |
| `artifact:08-operations-alerting-md` | Alerting | `migration-artifacts/08-operations/alerting.md` | `` |
| `artifact:08-operations-incident-response-md` | Incident Response | `migration-artifacts/08-operations/incident-response.md` | `` |
| `artifact:08-operations-infrastructure-topology-map-md` | Infrastructure Topology Map | `migration-artifacts/08-operations/infrastructure-topology-map.md` | `ART-116_INFRA_TOPOLOGY` |
| `artifact:08-operations-network-dependency-map-md` | Network Dependency Map | `migration-artifacts/08-operations/network-dependency-map.md` | `` |
| `artifact:08-operations-environment-parity-matrix-md` | Environment Parity Matrix | `migration-artifacts/08-operations/environment-parity-matrix.md` | `` |
| `artifact:08-operations-resource-inventory-md` | Resource Inventory | `migration-artifacts/08-operations/resource-inventory.md` | `` |
| `artifact:08-operations-iac-coverage-report-md` | Iac Coverage Report | `migration-artifacts/08-operations/iac-coverage-report.md` | `` |
| `artifact:08-operations-secrets-certificates-inventory-md` | Secrets Certificates Inventory | `migration-artifacts/08-operations/secrets-certificates-inventory.md` | `` |
| `artifact:08-operations-capacity-baseline-md` | Capacity Baseline | `migration-artifacts/08-operations/capacity-baseline.md` | `` |
| `artifact:08-operations-cost-baseline-forecast-md` | Cost Baseline Forecast | `migration-artifacts/08-operations/cost-baseline-forecast.md` | `` |
| `artifact:08-operations-dr-backup-map-md` | Dr Backup Map | `migration-artifacts/08-operations/dr-backup-map.md` | `` |
| `artifact:09-governance-decision-log-md` | Decision Log | `migration-artifacts/09-governance/decision-log.md` | `ART-126_DECISION_LOG` |
| `artifact:09-governance-risk-register-md` | Risk Register | `migration-artifacts/09-governance/risk-register.md` | `ART-125_RISK_REGISTER` |
| `artifact:09-governance-ownership-matrix-md` | Ownership Matrix | `migration-artifacts/09-governance/ownership-matrix.md` | `ART-112_CODE_OWNERSHIP` |
| `artifact:09-governance-code-ownership-map-md` | Code Ownership Map | `migration-artifacts/09-governance/code-ownership-map.md` | `ART-112_CODE_OWNERSHIP` |
| `artifact:09-governance-iam-security-access-matrix-md` | Iam Security Access Matrix | `migration-artifacts/09-governance/iam-security-access-matrix.md` | `ART-117_IAM_MATRIX` |
| `artifact:09-governance-security-control-matrix-md` | Security Control Matrix | `migration-artifacts/09-governance/security-control-matrix.md` | `` |
| `artifact:09-governance-ownership-raci-matrix-md` | Ownership Raci Matrix | `migration-artifacts/09-governance/ownership-raci-matrix.md` | `ART-135_RACI` |
| `artifact:09-governance-migration-readiness-scorecard-md` | Migration Readiness Scorecard | `migration-artifacts/09-governance/migration-readiness-scorecard.md` | `ART-128_READINESS_SCORECARD` |
| `artifact:09-governance-business-capability-map-md` | Business Capability Map | `migration-artifacts/09-governance/business-capability-map.md` | `ART-129_BUSINESS_CAPABILITY` |
| `artifact:09-governance-flexnetos-open-questions-md` | Flexnetos Open Questions | `migration-artifacts/09-governance/flexnetos-open-questions.md` | `` |
| `automation:automation-target-descriptor` | Target descriptor | `(database record)` | `` |
| `automation:automation-artifact-contract-version` | Artifact contract version | `(database record)` | `` |
| `automation:automation-migration-recipe-version` | Migration recipe version | `(database record)` | `` |
| `automation:automation-run-ledger` | Run ledger | `(database record)` | `` |
| `automation:automation-event-timeline` | Event timeline | `(database record)` | `` |
| `automation:automation-operation-queue` | Operation queue | `(database record)` | `` |
| `automation:automation-approval-ledger` | Approval ledger | `(database record)` | `` |
| `automation:automation-evidence-register` | Evidence register | `(database record)` | `` |
| `automation:automation-artifact-registry` | Artifact registry | `(database record)` | `` |
| `automation:automation-link-graph` | Link graph | `(database record)` | `` |
| `automation:automation-validation-scorecard` | Validation scorecard | `(database record)` | `` |
| `automation:automation-replay-readiness-report` | Replay readiness report | `(database record)` | `` |
| `automation:automation-rollback-readiness-report` | Rollback readiness report | `(database record)` | `` |
| `automation:automation-human-involvement-transcript` | Human involvement transcript | `(database record)` | `` |
| `automation:automation-agent-session-ledger` | Agent session ledger | `(database record)` | `` |
| `automation:automation-plugin-session-ledger` | Plugin session ledger | `(database record)` | `` |
