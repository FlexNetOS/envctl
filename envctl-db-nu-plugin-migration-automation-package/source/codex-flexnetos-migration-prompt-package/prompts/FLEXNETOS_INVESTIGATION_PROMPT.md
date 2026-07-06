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
