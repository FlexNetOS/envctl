<!-- BEGIN prompts/EXECUTION_STYLE.md -->
# Execution Style Contract

## Required local execution model

- Orchestrator: `Codex 5.4` through the local Codex Rust CLI.
- Helpers/subagents: multiple `Spark 5.3` helpers.
- Shell strategy: run background shell helpers first to collect raw evidence, then have Spark helpers independently analyze specific domains, then have Codex 5.4 merge, link, and validate all artifacts.

## Model verification

Before generating any migration finding, Codex must verify the active model/provider state. If `codex-5.4` or `spark-5.3` cannot be resolved by the local setup, Codex must create:

- `migration-artifacts/00-executive-summary/model-resolution-blocker.md`

and stop before creating factual analysis artifacts. It may still create empty scaffolding with `status: blocked`.

## Background shell helper requirements

Run `helpers/background_scan.sh` in the background scan phase. The helper must write raw evidence under:

```text
migration-artifacts/_raw/YYYYMMDDTHHMMSSZ/
```

The raw scan must include:

- OS and command availability
- path resolution for FlexNetOS and lifeos candidates
- filesystem tree and file inventory
- git status, remotes, branches, tags, and recent history
- manifest inventory
- package/build/toolchain files
- config/environment key inventory with values redacted
- dependency references
- API/event/webhook references
- database/schema/migration references
- infrastructure/IaC references
- observability references
- test/CI references
- FlexNetOS-vs-lifeos reference search
- checksums and job status

## Spark 5.3 helper allocation

Spawn at least these eight helpers:

1. `spark-filesystem-repo`
2. `spark-toolchain-deps`
3. `spark-code-runtime-debug`
4. `spark-data-schema-lineage`
5. `spark-infra-security-obs`
6. `spark-integrations-contracts`
7. `spark-migration-controls`
8. `spark-flexnetos-investigator`

Each helper must write a domain report under:

```text
migration-artifacts/_spark/<helper-name>.md
migration-artifacts/_spark/<helper-name>.json
```

Each helper must include:

- Scope
- Commands or raw scan files used
- Proven findings
- Unknowns
- Artifact files it recommends creating/updating
- Evidence links

## Orchestrator merge requirements

Codex 5.4 must consolidate Spark helper reports into final artifacts. It must not merely paste helper reports. It must reconcile contradictions, mark unresolved conflicts, link evidence, and produce a coherent wiki-style artifact graph.

<!-- END prompts/EXECUTION_STYLE.md -->


<!-- BEGIN prompts/FLEXNETOS_INVESTIGATION_PROMPT.md -->
# FlexNetOS Investigation Prompt

## Investigation question

What was `FlexNetOS` used for at `~/home/flexnetos/FlexNetOS` instead of `~/home/flexnetos/lifeos`?

## Path resolution rules

Treat the user-provided paths as candidates, not guaranteed truths. Resolve and document:

```text
~/home/flexnetos/FlexNetOS
~/home/flexnetos/lifeos
/home/flexnetos/FlexNetOS
/home/flexnetos/lifeos
```

For each candidate, record:

- exists / missing
- realpath
- symlink target
- mountpoint
- owner/group/mode
- git repository root
- remote URLs redacted only if credential-bearing
- active branch, tags, and recent commits
- top-level tree
- manifests and service definitions

## Evidence classes to inspect

Use real local evidence only:

1. README, docs, markdown, design notes, ADRs
2. package manifests: `package.json`, `Cargo.toml`, `pyproject.toml`, `requirements.txt`, `go.mod`, `pom.xml`, `build.gradle`, Dockerfiles, Compose files, Helm charts, Terraform
3. service definitions: `*.service`, `systemd`, init scripts, supervisord, PM2, cron specs, GitHub/GitLab/CI workflow files
4. code identifiers: namespaces, package names, app names, CLI names, API routes, event topics, database names, table prefixes
5. configuration keys: env var names only, config section names, feature flags, domain names, ports, hosts, queue/topic names
6. git history: commit messages, renames, branch names, tags, merge messages, file-level history
7. test fixtures and golden data
8. generated artifacts or old migration notes
9. references to `FlexNetOS`, `flexnetos`, `FLEXNETOS`, `lifeos`, `LifeOS`, `LIFEOS`, `FlexNet`, `Life OS`

## Required analysis questions

Answer these with proof or `UNKNOWN`:

- Was FlexNetOS a fork, rename, wrapper, replacement, migration target, prototype, deployment packaging, or unrelated repo?
- Was lifeos present as a predecessor, dependency, submodule, namespace, archived copy, or comparison target?
- What runtime/application role did FlexNetOS have?
- What services, jobs, APIs, UI apps, databases, queues, schedulers, or infrastructure did it define or depend on?
- What business or operational capability did it appear to serve?
- Why might the path `FlexNetOS` have been used instead of `lifeos`?
- What evidence contradicts the leading explanation?
- What evidence is missing?

## Required outputs

Create or update:

```text
migration-artifacts/00-executive-summary/flexnetos-purpose-summary.md
migration-artifacts/01-current-state/flexnetos-vs-lifeos-evidence.md
migration-artifacts/01-current-state/flexnetos-path-resolution.md
migration-artifacts/01-current-state/flexnetos-reference-index.md
migration-artifacts/03-code-analysis/flexnetos-entrypoints.md
migration-artifacts/05-integrations/flexnetos-contracts.md
migration-artifacts/09-governance/flexnetos-open-questions.md
```

The executive summary must contain:

- Bottom-line answer
- Confidence level: High / Medium / Low
- Top 10 evidence items
- What would disprove the conclusion
- Next follow-up task recommendation

<!-- END prompts/FLEXNETOS_INVESTIGATION_PROMPT.md -->


<!-- BEGIN prompts/LINKING_AND_MEMORY_PROMPT.md -->
# Linking and Persistent Memory Prompt

## Goal

Build a persistent memory layer so future Codex migration tasks can understand the repository without rediscovery.

## Required files

```text
migration-artifacts/MIGRATION_MEMORY.md
migration-artifacts/index.md
migration-artifacts/artifact-manifest.json
migration-artifacts/artifact-manifest.md
migration-artifacts/evidence-register.md
migration-artifacts/link-graph.md
migration-artifacts/wiki-home.md
migration-artifacts/_meta/scan-runs.jsonl
migration-artifacts/_meta/artifact-status.tsv
```

## Wiki-style linking rules

- Every artifact must have a stable relative Markdown link from `migration-artifacts/index.md`.
- Every artifact must include a backlink to `../index.md` or `../../index.md`.
- Use relative links only; no absolute local file links in final Markdown.
- Every graph source file must link to its rendered SVG/PNG if generated.
- Every finding must link to evidence in `evidence-register.md` or cite a raw scan file path.

## Persistent memory sections

`MIGRATION_MEMORY.md` must include:

1. Project identity
2. Proven purpose of FlexNetOS
3. Relationship to lifeos
4. Current-state architecture summary
5. Runtime and deployment summary
6. Data and integration summary
7. Toolchain summary
8. Known risks and blockers
9. Artifact map
10. Open questions
11. Commands used to create memory
12. Last verified timestamp

## Link validation

Run the link/index helper after artifacts are written:

```bash
python3 <prompt-package>/helpers/artifact_manifest.py migration-artifacts
python3 <prompt-package>/helpers/make_wiki_index.py migration-artifacts
```

If links break, fix the links before finalizing.

<!-- END prompts/LINKING_AND_MEMORY_PROMPT.md -->


<!-- BEGIN prompts/ARTIFACT_CONTRACT_FULL.md -->
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

<!-- END prompts/ARTIFACT_CONTRACT_FULL.md -->


<!-- BEGIN prompts/MASTER_PROMPT.md -->
# MASTER PROMPT — Codex 5.4 Local Migration Artifact Builder for FlexNetOS

You are Codex 5.4 running locally through the Codex Rust CLI on Ubuntu 26.04+.

Your mission is to build a complete, linked, persistent migration-artifact knowledge base for the repository/workspace rooted at the resolved FlexNetOS path, then answer the follow-up investigation question:

> What was `FlexNetOS` used for at `~/home/flexnetos/FlexNetOS` instead of `~/home/flexnetos/lifeos`?

## Input context

The runner may provide these variables at the top of stdin:

```text
RUN_CONTEXT_FILE=<path>
PRIMARY_ROOT=<path>
COMPARE_ROOT=<path>
PROMPT_PACKAGE_DIR=<path>
```

Read the run context if present. Use these default candidates if missing:

```text
PRIMARY_ROOT_CANDIDATES:
  - ~/home/flexnetos/FlexNetOS
  - /home/flexnetos/FlexNetOS
COMPARE_ROOT_CANDIDATES:
  - ~/home/flexnetos/lifeos
  - /home/flexnetos/lifeos
```

## Absolute non-negotiables

- No simulation.
- No demo artifacts.
- No invented facts.
- No fabricated ownership, services, dependencies, business process, data stores, routes, or histories.
- Use real shell commands and real local files.
- Redact secrets.
- Do not delete files.
- Do not mutate production systems.
- Do not write to databases.
- Do not install packages without explicit approval.
- If a required artifact cannot be populated from evidence, create it anyway with `status: unknown` or `status: partial` and document the evidence gap.

## Model and execution requirements

1. Verify that you are using the requested orchestrator label: `codex-5.4`.
2. Verify that helper/subagent role configs resolve to `spark-5.3`.
3. Use multiple Spark 5.3 helpers/subagents. Spawn at least eight helpers:
   - `spark-filesystem-repo`
   - `spark-toolchain-deps`
   - `spark-code-runtime-debug`
   - `spark-data-schema-lineage`
   - `spark-infra-security-obs`
   - `spark-integrations-contracts`
   - `spark-migration-controls`
   - `spark-flexnetos-investigator`
4. Run background shell helpers before final artifact synthesis.
5. The shell helper to run is:

```bash
bash "$PROMPT_PACKAGE_DIR/helpers/background_scan.sh" "$PRIMARY_ROOT" "$COMPARE_ROOT"
```

If `PROMPT_PACKAGE_DIR` is absent, locate this prompt package by searching upward/current known paths and record the search commands.

## Required source files from the prompt package

Read and apply all of the following. Do not reduce them:

```text
prompts/EXECUTION_STYLE.md
prompts/ARTIFACT_CONTRACT_FULL.md
prompts/FLEXNETOS_INVESTIGATION_PROMPT.md
prompts/LINKING_AND_MEMORY_PROMPT.md
prompts/spark_helpers/*.md
source/previous-migration-artifact-context.md
expected-output/migration-artifacts-tree.md
```

## Work plan

### Phase 0 — Resolve paths and safety

- Resolve `PRIMARY_ROOT` and `COMPARE_ROOT` with `realpath`.
- Verify which candidate paths exist.
- Confirm git repository state.
- Create `migration-artifacts/` if absent.
- Create a run ID using UTC timestamp.
- Write `migration-artifacts/_meta/run-context.md`.

### Phase 1 — Background evidence scan

Run the background scanner. It must collect raw evidence in parallel and write a run directory under:

```text
migration-artifacts/_raw/<UTC_RUN_ID>/
```

Do not skip this phase unless the shell cannot run. If it cannot run, write a blocker artifact.

### Phase 2 — Spark 5.3 helper analysis

Spawn the eight Spark helpers. Give each helper the relevant prompt from `prompts/spark_helpers/` plus the raw scan directory path.

Each helper must produce:

```text
migration-artifacts/_spark/<helper-name>.md
migration-artifacts/_spark/<helper-name>.json
```

Wait for all helpers. If a helper fails, record the failure and continue with explicit gap markers.

### Phase 3 — Build every migration artifact

Create the artifact tree defined in `expected-output/migration-artifacts-tree.md` and the full artifact contract.

Every artifact must include this front matter:

```yaml
---
artifact_id: <stable-kebab-id>
title: <title>
status: complete | partial | unknown | blocked
generated_at_utc: <timestamp>
source_root: <resolved PRIMARY_ROOT>
compare_root: <resolved COMPARE_ROOT or UNKNOWN>
evidence_paths:
  - <relative evidence path or UNKNOWN>
last_verified_utc: <timestamp>
links:
  parent_index: <relative link>
  related: []
---
```

Every artifact must contain:

```text
## Verified findings
## Evidence ledger
## Unknowns / gaps
## Commands run
## Related artifacts
## Next actions
```

### Phase 4 — FlexNetOS purpose answer

Create the explicit answer artifacts:

```text
migration-artifacts/00-executive-summary/flexnetos-purpose-summary.md
migration-artifacts/01-current-state/flexnetos-vs-lifeos-evidence.md
migration-artifacts/01-current-state/flexnetos-path-resolution.md
migration-artifacts/01-current-state/flexnetos-reference-index.md
migration-artifacts/03-code-analysis/flexnetos-entrypoints.md
migration-artifacts/05-integrations/flexnetos-contracts.md
migration-artifacts/09-governance/flexnetos-open-questions.md
```

The summary must answer the question directly using evidence only. Use confidence levels and explain uncertainty.

### Phase 5 — Link, validate, and write persistent memory

Generate:

```text
migration-artifacts/MIGRATION_MEMORY.md
migration-artifacts/index.md
migration-artifacts/wiki-home.md
migration-artifacts/evidence-register.md
migration-artifacts/link-graph.md
migration-artifacts/artifact-manifest.json
migration-artifacts/artifact-manifest.md
migration-artifacts/_meta/artifact-status.tsv
```

Run:

```bash
python3 "$PROMPT_PACKAGE_DIR/helpers/artifact_manifest.py" migration-artifacts
python3 "$PROMPT_PACKAGE_DIR/helpers/make_wiki_index.py" migration-artifacts
```

Fix broken links or missing artifacts.

## Required artifacts to build and link

Build all artifacts from the full contract, including all high-value artifacts, all migration-type artifacts, all example-specific artifacts, the minimum viable set, all advanced artifacts, and the recommended repository structure.

At minimum, this means all files listed in `expected-output/migration-artifacts-tree.md` must exist and be linked.

## Final response required from Codex

At the end, report:

1. Whether the run completed, partially completed, or blocked.
2. The resolved FlexNetOS path.
3. The resolved lifeos comparison path.
4. The bottom-line answer to what FlexNetOS was used for.
5. Confidence level.
6. Top evidence links.
7. Artifact index path.
8. Persistent memory path.
9. Remaining blockers.

Do not include secrets. Do not claim completion if required artifacts are missing.

<!-- END prompts/MASTER_PROMPT.md -->
