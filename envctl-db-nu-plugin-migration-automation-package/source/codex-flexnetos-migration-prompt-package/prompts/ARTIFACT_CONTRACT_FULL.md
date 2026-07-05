# Full Migration Artifact Contract

This file contains the complete artifact context that must not be reduced or omitted. It was sourced from the prior migration-artifact answer and preserved verbatim in `source/previous-migration-artifact-context.md`.

Generated: `2026-07-04T16:53:36Z`

## Non-reduction rule

Codex must build and link every artifact named below. If evidence is unavailable, create the artifact with `status: unknown` or `status: partial`, explain the evidence gap, and link it from the index.

---

For a migration project, the most useful artifacts are **maps, inventories, dependency graphs, validation evidence, and cutover controls**. The goal is to reduce unknowns: what exists, what depends on what, what breaks if moved, how data changes, how to prove parity, and how to roll back.

The best artifacts are **generated, version-controlled, diffable, and tied to real systems** rather than hand-drawn diagrams that rot.

## High-value migration artifacts

| Artifact                                      | What it answers                                                                                                 | Why it matters                                                                        |
| --------------------------------------------- | --------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| **System inventory**                          | What applications, services, jobs, databases, queues, APIs, reports, scripts, and schedulers exist?             | Prevents hidden systems from being missed.                                            |
| **Directory and subdirectory hierarchy tree** | Where is the code/config/data actually located?                                                                 | Useful for repo cleanup, ownership mapping, build discovery, and dead-code detection. |
| **Repository map**                            | Which repos exist, what they contain, who owns them, and how active they are?                                   | Helps split migration scope into manageable chunks.                                   |
| **Application/service dependency graph**      | Which services call which other services?                                                                       | Critical for sequencing migration waves.                                              |
| **Toolchain dependency tree**                 | What compilers, runtimes, package managers, SDKs, build tools, CI/CD tools, and deploy tools are required?      | Exposes version traps and platform incompatibilities.                                 |
| **Package/library dependency graph**          | Which packages depend on vulnerable, deprecated, or incompatible libraries?                                     | Useful for modernization, security, and upgrade planning.                             |
| **Runtime dependency map**                    | What does the app need at runtime: databases, env vars, secrets, filesystems, queues, third-party APIs, caches? | Prevents “works in build, fails in prod” problems.                                    |
| **Data flow graph**                           | How does data enter, move, transform, persist, and exit the system?                                             | Essential for data migration, compliance, and debugging.                              |
| **Database schema map**                       | What tables, columns, indexes, constraints, procedures, triggers, and views exist?                              | Needed for data migration and performance parity.                                     |
| **Data lineage map**                          | Where does each critical field originate, transform, and get consumed?                                          | Prevents silent data corruption.                                                      |
| **API contract catalog**                      | What endpoints/events exist, request/response schemas, auth, consumers, versions?                               | Prevents breaking integrations.                                                       |
| **Event/message contract map**                | What topics, queues, payloads, producers, consumers, retry semantics, and DLQs exist?                           | Critical for async architectures.                                                     |
| **Code ownership map**                        | Who owns each module/service/data pipeline?                                                                     | Speeds up decisions and incident resolution.                                          |
| **Code map for debugging**                    | Main execution paths, entry points, error paths, logging points, external calls.                                | Helps teams troubleshoot migrated workloads faster.                                   |
| **Environment matrix**                        | Dev/stage/prod differences: versions, configs, infra, secrets, endpoints, feature flags.                        | Finds environment drift before cutover.                                               |
| **Configuration inventory**                   | Env vars, config files, flags, secrets references, runtime parameters.                                          | Configuration is often where migrations fail.                                         |
| **Infrastructure topology map**               | Compute, networking, storage, DNS, load balancers, firewalls, certificates.                                     | Needed for cloud, data center, Kubernetes, or platform migrations.                    |
| **IAM/security access matrix**                | Users, roles, service accounts, permissions, secrets, certificates, tokens.                                     | Prevents broken auth and over-permissioned systems.                                   |
| **Observability map**                         | Logs, metrics, traces, dashboards, alerts, SLOs, runbooks.                                                      | Migration is dangerous without visibility.                                            |
| **Business process map**                      | Which business workflows depend on which systems?                                                               | Helps prioritize what truly matters.                                                  |
| **Migration wave plan**                       | What moves together, in what order, and why?                                                                    | Converts analysis into execution.                                                     |
| **Cutover checklist**                         | Exact steps for go-live.                                                                                        | Reduces human error during high-pressure migration windows.                           |
| **Rollback plan**                             | How to undo or fail back safely.                                                                                | Non-negotiable for serious migrations.                                                |
| **Validation and reconciliation reports**     | Did the migrated system produce the same outputs? Did all records move?                                         | The evidence layer. Without this, migration success is just a claim.                  |
| **Test coverage matrix**                      | What has unit, integration, regression, performance, security, and user acceptance coverage?                    | Shows where risk remains.                                                             |
| **Risk register**                             | Known risks, owners, mitigations, severity, status.                                                             | Keeps ugly truths visible.                                                            |
| **Decision log / ADRs**                       | What decisions were made, by whom, and why?                                                                     | Prevents re-litigating architecture choices later.                                    |

---

## The most useful artifacts by migration type

### 1. **Codebase migration**

Useful when moving languages, frameworks, monoliths to services, legacy apps to modern platforms, or repos to new standards.

Key artifacts:

| Artifact                     | Use                                                                                                         |
| ---------------------------- | ----------------------------------------------------------------------------------------------------------- |
| **Codebase hierarchy graph** | Shows folders, modules, packages, entry points, generated code, tests, configs.                             |
| **Import/dependency graph**  | Shows module-to-module dependencies. Great for identifying circular dependencies and extraction boundaries. |
| **Call graph**               | Shows function/method invocation paths. Useful for debugging and refactoring.                               |
| **Dead-code report**         | Identifies unused modules, functions, routes, jobs, and feature flags.                                      |
| **Hotspot map**              | Combines code churn, bug history, complexity, and ownership. Shows risky areas.                             |
| **Build graph**              | Shows how source code becomes artifacts: binaries, containers, packages, deployables.                       |
| **Runtime entry-point map**  | Shows app start commands, workers, jobs, handlers, APIs, and scheduled tasks.                               |
| **Compatibility matrix**     | Shows supported runtime versions, framework versions, OS versions, and package constraints.                 |

**Especially useful:**
A **code map for debugging** that includes:

```text
Request/Event Entry Point
  → Controller/Handler
    → Service Layer
      → Business Logic
        → Repository/Data Access
          → Database/External API/Queue
    → Error Handling
    → Logging/Tracing
    → Response/Event Output
```

This is more useful than a pretty architecture diagram because it shows where failures actually happen.

---

### 2. **Data migration**

Useful when moving databases, warehouses, lakes, ETL/ELT pipelines, schemas, or reporting layers.

Key artifacts:

| Artifact                          | Use                                                                                        |
| --------------------------------- | ------------------------------------------------------------------------------------------ |
| **Source-to-target mapping**      | Maps old fields/tables to new fields/tables.                                               |
| **Transformation rules catalog**  | Defines type conversions, normalization, joins, defaults, deduplication, enrichment.       |
| **Data lineage graph**            | Shows where data originates and where it flows.                                            |
| **Data quality profile**          | Null rates, duplicates, invalid values, outliers, referential integrity.                   |
| **Reconciliation report**         | Compares source vs. target record counts, checksums, aggregates, business totals.          |
| **Schema diff report**            | Shows changes between old and new schema.                                                  |
| **Critical field inventory**      | Identifies fields that drive billing, reporting, compliance, auth, or customer experience. |
| **Backfill plan**                 | Defines historical data load strategy.                                                     |
| **Incremental sync plan**         | Defines CDC, dual writes, event replay, or batch sync strategy.                            |
| **Data retention/compliance map** | Shows PII, PHI, PCI, retention rules, deletion obligations.                                |

The artifact that usually saves the project is the **reconciliation report**. A migration is not done because the pipeline ran. It is done when the numbers prove it.

---

### 3. **Cloud or infrastructure migration**

Useful when moving from on-prem to cloud, cloud-to-cloud, VMs to containers, or legacy hosting to Kubernetes/serverless.

Key artifacts:

| Artifact                               | Use                                                                        |
| -------------------------------------- | -------------------------------------------------------------------------- |
| **Infrastructure topology map**        | Shows compute, networking, storage, ingress, DNS, security boundaries.     |
| **Network dependency map**             | Ports, protocols, firewalls, allowlists, private links, VPNs, VPCs/VNETs.  |
| **Environment parity matrix**          | Compares old vs. new dev/stage/prod environments.                          |
| **Resource inventory**                 | Servers, VMs, containers, databases, buckets, volumes, certificates.       |
| **IaC coverage report**                | Shows what is codified vs. manually configured.                            |
| **Secrets and certificates inventory** | Expirations, owners, rotation requirements, consuming services.            |
| **Capacity baseline**                  | CPU, memory, storage, IOPS, throughput, latency, peak load.                |
| **Cost baseline and forecast**         | Current cost vs. expected target cost.                                     |
| **DR/backup map**                      | Recovery point objective, recovery time objective, backups, restore tests. |
| **Security control matrix**            | Encryption, network policies, IAM, audit logs, vulnerability controls.     |

The dangerous gaps are usually **DNS, certificates, firewall rules, secrets, and undocumented cron jobs**.

---

### 4. **Application integration migration**

Useful when replacing SaaS systems, ERP/CRM platforms, identity providers, payment systems, messaging layers, or API gateways.

Key artifacts:

| Artifact                            | Use                                                                              |
| ----------------------------------- | -------------------------------------------------------------------------------- |
| **Integration catalog**             | Lists every inbound/outbound integration.                                        |
| **API contract map**                | Shows endpoints, methods, schemas, auth, rate limits, consumers.                 |
| **Webhook/event map**               | Shows event producers, consumers, payloads, retries, delivery guarantees.        |
| **Third-party dependency register** | Vendors, credentials, SLAs, rate limits, support contacts.                       |
| **Auth flow diagram**               | OAuth, SAML, OIDC, API keys, service accounts, token lifetimes.                  |
| **Failure-mode map**                | What happens on timeout, retry, partial failure, duplicate message, bad payload. |
| **Backward compatibility plan**     | Versioning, adapter layers, proxies, shims, redirects.                           |

For integrations, the best artifact is often a **consumer map**:

```text
API / Event / File Feed
  → Producer
  → Consumers
  → Payload Contract
  → Auth Method
  → Frequency
  → Failure Behavior
  → Business Owner
  → Technical Owner
  → Migration Status
```

---

## Artifacts specifically matching your examples

### Directory and subdirectory hierarchy graph

Useful for:

* Discovering project structure.
* Finding duplicated modules.
* Identifying generated files.
* Separating source, tests, configs, docs, scripts, and build outputs.
* Planning repo splits or consolidation.

Example shape:

```text
repo-root/
├── apps/
│   ├── web/
│   ├── api/
│   └── worker/
├── packages/
│   ├── shared-models/
│   ├── auth/
│   └── logging/
├── infra/
│   ├── terraform/
│   └── k8s/
├── scripts/
├── tests/
└── docs/
```

Better version: a graph that tags each folder with:

```text
owner
language
runtime
build tool
deployment target
test coverage
last modified
dependency count
risk score
```

---

### Toolchain dependency tree

Useful for identifying what must exist before the app can build, test, deploy, and run.

Example:

```text
Application
├── Runtime
│   ├── Node 20
│   ├── Python 3.12
│   └── Java 21
├── Build
│   ├── pnpm
│   ├── Maven
│   └── Docker
├── CI/CD
│   ├── GitHub Actions
│   ├── Artifact Registry
│   └── Deployment Runner
├── Infrastructure
│   ├── Terraform
│   ├── Kubernetes
│   └── Helm
└── Security
    ├── SAST
    ├── Dependency scanning
    └── Container scanning
```

For a migration, this should include **version constraints** and **upgrade blockers**.

---

### Code map for debugging

Useful for understanding how to troubleshoot the migrated system.

A good debugging map includes:

| Layer          | What to document                                                  |
| -------------- | ----------------------------------------------------------------- |
| Entry points   | HTTP routes, CLI commands, jobs, queue consumers, event handlers. |
| Control flow   | Main execution path through controllers/services/modules.         |
| External calls | Databases, APIs, queues, caches, filesystems.                     |
| Failure points | Known exception sources, retry paths, timeout risks.              |
| Logs           | Log statements, correlation IDs, trace IDs, severity levels.      |
| Metrics        | Counters, latency, errors, saturation, business KPIs.             |
| Alerts         | What fires, who receives it, what action to take.                 |
| Runbooks       | How to diagnose and recover.                                      |

The useful version is not just “how the code is organized.” It is **how failure moves through the system**.

---

### Data flow graph

Useful for seeing how data actually behaves across systems.

Example:

```text
Customer submits form
  → Web App
  → API Gateway
  → Customer Service
  → Validation Layer
  → PostgreSQL
  → Change Data Capture
  → Event Bus
  → Analytics Pipeline
  → Data Warehouse
  → BI Dashboard
```

Enhance it with:

```text
data owner
system owner
schema
PII classification
latency expectation
retention policy
transformation logic
validation rule
downstream consumers
```

---

## Minimum viable artifact set

For a serious migration, I would not start execution without these:

| Priority | Artifact                             |
| -------: | ------------------------------------ |
|        1 | **System inventory**                 |
|        2 | **Dependency graph**                 |
|        3 | **Data flow graph**                  |
|        4 | **Source-to-target mapping**         |
|        5 | **Environment/configuration matrix** |
|        6 | **Toolchain dependency tree**        |
|        7 | **API/event contract catalog**       |
|        8 | **Test and validation matrix**       |
|        9 | **Cutover checklist**                |
|       10 | **Rollback plan**                    |
|       11 | **Risk register**                    |
|       12 | **Decision log**                     |

That is the core set. Everything else is useful, but those are the artifacts that directly reduce migration failure risk.

---

## Advanced artifacts that are especially valuable

These are higher-effort, but they tend to expose the problems nobody wants to talk about.

| Artifact                             | Why it is powerful                                                               |
| ------------------------------------ | -------------------------------------------------------------------------------- |
| **Blast-radius map**                 | Shows what breaks if a service, table, queue, API, or credential fails.          |
| **Migration readiness scorecard**    | Scores each app/service/data domain against readiness criteria.                  |
| **Business capability map**          | Connects technical systems to actual business functions.                         |
| **Shadow traffic comparison report** | Compares old vs. new system behavior under real traffic.                         |
| **Golden dataset**                   | Known-good sample data used to validate transformations and outputs.             |
| **Parity dashboard**                 | Real-time comparison of old and new system metrics.                              |
| **Deprecation map**                  | Shows what gets retired, replaced, wrapped, or preserved.                        |
| **Exception inventory**              | Captures special cases, one-off scripts, manual processes, and tribal knowledge. |
| **Ownership/RACI matrix**            | Defines who approves, builds, validates, cuts over, and supports.                |
| **Technical debt ledger**            | Separates “must fix before migration” from “carry temporarily” from “delete.”    |

---

## Recommended artifact structure

A practical migration artifact repository might look like this:

```text
migration-artifacts/
├── 00-executive-summary/
├── 01-current-state/
│   ├── system-inventory.md
│   ├── architecture-current.md
│   ├── dependency-graph.md
│   ├── data-flow-current.md
│   └── risk-hotspots.md
├── 02-target-state/
│   ├── architecture-target.md
│   ├── platform-design.md
│   ├── security-model.md
│   └── operating-model.md
├── 03-code-analysis/
│   ├── repo-map.md
│   ├── directory-tree.md
│   ├── package-dependencies.md
│   ├── call-graph.md
│   └── dead-code-report.md
├── 04-data-migration/
│   ├── schema-map.md
│   ├── source-target-mapping.md
│   ├── transformation-rules.md
│   ├── data-quality-report.md
│   └── reconciliation-plan.md
├── 05-integrations/
│   ├── api-catalog.md
│   ├── event-catalog.md
│   ├── third-party-dependencies.md
│   └── auth-flows.md
├── 06-testing-validation/
│   ├── test-strategy.md
│   ├── regression-matrix.md
│   ├── performance-baseline.md
│   └── validation-evidence.md
├── 07-cutover/
│   ├── wave-plan.md
│   ├── cutover-checklist.md
│   ├── rollback-plan.md
│   └── communication-plan.md
├── 08-operations/
│   ├── observability-map.md
│   ├── runbooks.md
│   ├── alerting.md
│   └── incident-response.md
└── 09-governance/
    ├── decision-log.md
    ├── risk-register.md
    ├── ownership-matrix.md
    └── status-dashboard.md
```

---

## Best formats

| Artifact type         | Good formats                                        |
| --------------------- | --------------------------------------------------- |
| Trees and hierarchies | Markdown, Mermaid, Graphviz DOT, JSON               |
| Dependency graphs     | Graphviz, Mermaid, Neo4j, D3, Cytoscape             |
| Inventories           | CSV, spreadsheet, database table                    |
| API contracts         | OpenAPI, AsyncAPI, Postman collections              |
| Data mappings         | Spreadsheet, dbt docs, YAML, SQL metadata           |
| Architecture          | C4 diagrams, Mermaid, PlantUML, Structurizr         |
| Security/IAM          | Matrix tables, policy-as-code, JSON/YAML exports    |
| Test evidence         | CI reports, JUnit XML, coverage reports, dashboards |
| Cutover controls      | Markdown checklist, runbook, project board          |

---

## The artifact hierarchy I would use

For a migration, artifacts should ladder from **discovery** to **execution**:

```text
1. Inventory
   ↓
2. Dependency Map
   ↓
3. Risk Map
   ↓
4. Target Design
   ↓
5. Migration Plan
   ↓
6. Validation Plan
   ↓
7. Cutover Plan
   ↓
8. Rollback Plan
   ↓
9. Operational Runbook
   ↓
10. Decommission Plan
```

The most common mistake is overbuilding architecture diagrams and underbuilding validation artifacts. Pretty diagrams do not prove the migration worked. **Reconciliation, test evidence, parity dashboards, and rollback plans do.**
