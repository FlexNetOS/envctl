# ART-106 Runtime Dependency Map

Generated: `2026-07-04T23:29:22+00:00`
Status: `complete`

This map records runtime dependency evidence for databases, environment variables, secret references, queues, APIs, and caches. It is built from approved package artifacts, the envctl database model, the target descriptor, and a safe scan that excludes `.env`, secret directories, private keys, PEM files, and key files.

## Scope

- Target: `flexnetos-vs-lifeos`
- Target root: `/home/flexnetos/FlexNetOS`
- Files visited: `3000`
- Runtime live state confirmed: `False`
- Secret material read: `False`
- Envctl database backend: `sqlite`

## Coverage

| Category | Status | Evidence | Files | Top references |
| --- | --- | --- | --- | --- |
| databases | repo_evidence_found | 180 | 74 | `.DB` (102), `SQLite` (98), `sqlite3` (88), `postgres` (84), `Postgres` (75), `PostgreSQL` (74), `sqlite` (34), `postgresql` (31) |
| env_vars | repo_evidence_found | 180 | 16 | `HF_TOKEN` (130), `HOME` (69), `X` (48), `SKILL_DIR` (45), `SO_HOME` (36), `BINARY_NAME` (30), `PATH` (28), `XXX` (28) |
| secret_refs | repo_evidence_found | 180 | 26 | `HF_TOKEN` (276), `credential` (156), `NGC_API_KEY` (65), `CLOUDFLARE_API_TOKEN` (61), `hf_token` (54), `apiToken` (53), `redaction` (51), `redacted` (49) |
| queues | repo_evidence_found | 180 | 54 | `queue` (193), `Queue` (67), `QUEUE` (12), `MQTT` (12), `mqtt` (4), `NATs` (2), `kafka` (2), `Redis queue` (1) |
| apis | repo_evidence_found | 180 | 27 | `MCP` (785), `http` (393), `fetch(` (364), `mcp` (323), `HTTP` (321), `REST` (241), `rest` (197), `requests.` (151) |
| caches | repo_evidence_found | 180 | 81 | `Redis` (58), `Cache-Control` (47), `.cache` (36), `cache_dir` (17), `XDG_CACHE_HOME` (15), `sccache` (10), `.CACHE` (8), `redis` (7) |

## Envctl Runtime Nodes

| Node | Category | Evidence |
| --- | --- | --- |
| envctl migration SQLite model | databases | `generated/envctl_migration_db_model.json`<br>`sql/001_migration_automation_schema.sql` |
| artifact registry hash/evidence tables | databases | `generated/envctl_artifact_registry_report.json`<br>`scripts/artifact_registry.py` |
| envctl_migration_operations and run_events queue/event surfaces | queues | `generated/envctl_migration_db_model.json` |
| envctl shared protocol schema records | apis | `generated/shared_protocol_manifest.json`<br>`schemas/shared_protocol.schema.json` |

## Hotspot Files

| Path | Categories | Score |
| --- | --- | --- |
| `oh-my-codex-main/.codex/.tmp/plugins/plugins/catalyst-by-zoho/skills/catalyst-by-zoho/references/cloud-scale.md` | apis, caches, env_vars, queues, secret_refs | 8 |
| `oh-my-codex-main/.codex/.tmp/plugins/plugins/catalyst-by-zoho/skills/catalyst-by-zoho/references/equivalents-azure.md` | apis, caches, env_vars, queues, secret_refs | 8 |
| `oh-my-codex-main/.codex/.tmp/plugins/plugins/notion/skills/notion-knowledge-capture/examples/conversation-to-faq.md` | apis, caches, databases, env_vars, secret_refs | 8 |
| `oh-my-codex-main/.codex/.tmp/plugins/plugins/nvidia/skills/physical-ai-infrastructure-setup-and-resilient-scaling/components/osmo-azure/reference.md` | apis, caches, databases, env_vars, secret_refs | 8 |
| `oh-my-codex-main/.codex/.tmp/plugins/plugins/nvidia/skills/physical-ai-infrastructure-setup-and-resilient-scaling/components/osmo-cli/reference.md` | apis, caches, databases, env_vars, secret_refs | 8 |
| `.codex/config.toml` | apis, databases, env_vars, secret_refs | 7 |
| `WORKLOG.md` | apis, databases, env_vars, secret_refs | 7 |
| `oh-my-codex-main/.codex/.tmp/plugins/plugins/cloudflare/skills/cloudflare/references/d1/api.md` | apis, databases, env_vars, secret_refs | 7 |
| `oh-my-codex-main/.codex/.tmp/plugins/plugins/cloudflare/skills/cloudflare/references/d1/configuration.md` | apis, databases, env_vars, secret_refs | 7 |
| `oh-my-codex-main/.codex/.tmp/plugins/plugins/cloudflare/skills/cloudflare/references/r2-data-catalog/configuration.md` | apis, databases, env_vars, secret_refs | 7 |
| `oh-my-codex-main/.codex/.tmp/plugins/plugins/cloudflare/skills/cloudflare/references/r2-data-catalog/patterns.md` | apis, databases, env_vars, secret_refs | 7 |
| `oh-my-codex-main/.codex/.tmp/plugins/plugins/cloudflare/skills/cloudflare/references/secrets-store/api.md` | apis, databases, env_vars, secret_refs | 7 |
| `oh-my-codex-main/.codex/.tmp/plugins/plugins/cloudflare/skills/cloudflare/references/workers-vpc/configuration.md` | apis, databases, env_vars, secret_refs | 7 |
| `oh-my-codex-main/.codex/.tmp/plugins/plugins/cloudflare/skills/cloudflare/references/workers/api.md` | apis, caches, databases, queues, secret_refs | 7 |
| `oh-my-codex-main/.codex/.tmp/plugins/plugins/expo/skills/expo-api-routes/SKILL.md` | apis, databases, env_vars, secret_refs | 7 |
| `oh-my-codex-main/.codex/.tmp/plugins/plugins/hugging-face/skills/jobs/SKILL.md` | apis, databases, env_vars, secret_refs | 7 |
| `oh-my-codex-main/.codex/.tmp/plugins/plugins/hugging-face/skills/llm-trainer/references/trackio_guide.md` | apis, databases, env_vars, secret_refs | 7 |
| `oh-my-codex-main/.codex/.tmp/plugins/plugins/nvidia/skills/aiq-deploy/SKILL.md` | apis, databases, env_vars, secret_refs | 7 |
| `oh-my-codex-main/.codex/.tmp/plugins/plugins/build-web-apps/skills/react-best-practices/AGENTS.md` | apis, caches, env_vars, secret_refs | 6 |
| `oh-my-codex-main/.codex/.tmp/plugins/plugins/catalyst-by-zoho/skills/catalyst-by-zoho/SKILL.md` | apis, caches, env_vars, secret_refs | 6 |
| `oh-my-codex-main/.codex/.tmp/plugins/plugins/catalyst-by-zoho/skills/catalyst-by-zoho/references/equivalents-aws.md` | apis, caches, env_vars, secret_refs | 6 |
| `oh-my-codex-main/.codex/.tmp/plugins/plugins/catalyst-by-zoho/skills/catalyst-by-zoho/references/functions-and-sdk.md` | apis, caches, env_vars, secret_refs | 6 |
| `oh-my-codex-main/.codex/.tmp/plugins/plugins/cloudflare/skills/cloudflare/references/cache-reserve/configuration.md` | apis, caches, env_vars, secret_refs | 6 |
| `oh-my-codex-main/.codex/.tmp/plugins/plugins/cloudflare/skills/cloudflare/references/queues/api.md` | apis, env_vars, queues, secret_refs | 6 |
| `oh-my-codex-main/.codex/.tmp/plugins/plugins/cloudflare/skills/cloudflare/references/web-analytics/configuration.md` | apis, caches, env_vars, secret_refs | 6 |

## Evidence Samples

| Category | Signal | Path | Line | Snippet |
| --- | --- | --- | --- | --- |
| databases | postgres | `LOCAL_WORKAROUNDS.md` | 273 | codex plugin add neon-postgres@meta-plugins-codex --json |
| databases | postgres | `WORKLOG.md` | 475 | neon-postgres |
| databases | sqlite | `WORKLOG.md` | 1298 | That repaired the SQLite projection and restored `head`, but |
| databases | sqlite | `WORKSPACE_LAYOUT.md` | 50 | \| `/home/flexnetos/FlexNetO...` \| Untracked Codex SQLite state candidate. \| |
| databases | postgres | `.codex/config.toml` | 144 | [plugins."neon-postgres@meta-plugins-codex"] |
| databases | sqlite | `artifacts/generated/T036/bootstrap.nu` | 20 | $env.CODEX_SQLITE_HOME = "/home/flexnetos/FlexNetO..." |
| databases | sqlite | `artifacts/generated/T036/bootstrap.sh` | 21 | export CODEX_SQLITE_HOME='/home/flexnetos/FlexNetO...' |
| databases | sqlite | `artifacts/generated/T036-repeat/bootstrap.nu` | 20 | $env.CODEX_SQLITE_HOME = "/home/flexnetos/FlexNetO..." |
| databases | sqlite | `artifacts/generated/T036-repeat/bootstrap.sh` | 21 | export CODEX_SQLITE_HOME='/home/flexnetos/FlexNetO...' |
| databases | sqlite | `artifacts/recovery/old-pack-context/flexnetos_production_execution_pack/WORKLOG.md` | 773 | - Classified `auth.json`, config, history, SQLite DBs and sidecars, sessions, caches, package cache, plugins, skills, temp dirs, model cache, and metadata as envctl seed rows. |
| env_vars | shell-env | `LOCAL_WORKAROUNDS.md` | 378 | PATH=/home/flexnetos/FlexNetO...:$PATH meta exec -- /home/flexnetos/FlexNetO... verify -> 17 commands complete |
| env_vars | shell-env | `LOCAL_WORKAROUNDS.md` | 379 | PATH=/home/flexnetos/FlexNetO...:$PATH meta exec -- /home/flexnetos/FlexNetO... status --json -> 17 commands complete |
| env_vars | shell-env | `LOCAL_WORKAROUNDS.md` | 424 | PATH=/home/flexnetos/FlexNetO...:$PATH /home/flexnetos/FlexNetO... doctor --json -> 16 repos discovered |
| env_vars | shell-env | `LOCAL_WORKAROUNDS.md` | 635 | and place the workspace tool surface, `${HOME}/.nix-profile/bin`, and the system |
| env_vars | nu-env | `LOCAL_WORKAROUNDS.md` | 945 | yzx run nu -c 'codex --version; $env.PATH \| to json --raw' |
| env_vars | shell-env | `WORKLOG.md` | 178 | env -i ... scripts/build-local-ubun....sh --help -> FXRUN_RELEASE_DIR default is $FXRUN_WORKSPACE_ROOT/release |
| env_vars | shell-env | `WORKLOG.md` | 683 | PATH=/home/flexnetos/FlexNetO...:$PATH meta exec -- /home/flexnetos/FlexNetO... verify -> 17 commands complete |
| env_vars | shell-env | `WORKLOG.md` | 684 | PATH=/home/flexnetos/FlexNetO...:$PATH meta exec -- /home/flexnetos/FlexNetO... status --json -> 17 commands complete |
| env_vars | shell-env | `WORKLOG.md` | 730 | PATH=/home/flexnetos/FlexNetO...:$PATH git-kb doctor --json -> repos.discovery ok, 16 repos discovered |
| env_vars | shell-env | `WORKLOG.md` | 1258 | path, avoiding the installer default of `$HOME/.local/bin` and placing the |
| secret_refs | secret-store | `WORKLOG.md` | 2126 | The local GitHub CLI account `drdave-flexnetos` is authenticated in the keyring |
| secret_refs | secret-name | `.codex/config.toml` | 15 | tool_output_token_limit = 12000 |
| secret_refs | secret-name | `.codex/config.toml` | 183 | 'ROOT="${LIFEOS_ROOT:-/home/flexnetos/FlexNetOS}"; export LIFEOS_ROOT="$ROOT"; export PATH="$ROOT/usr/bin:$ROOT/.toolchains/.bun/bin:$ROOT/.toolchains/node/bin:$ROOT/.local/bin:$PATH"; export N8N_API_URL="${N8N_API_URL:- |
| secret_refs | secret-name | `artifacts/logs/T036/header-and-secret-predicates.json` | 4 | "nu_secret_refs": false, |
| secret_refs | secret-name | `artifacts/logs/T036/header-and-secret-predicates.json` | 5 | "sh_secret_refs": false, |
| secret_refs | secret-name | `artifacts/logs/T036/header-and-secret-predicates.json` | 8 | "nu_github_token": false, |
| secret_refs | secret-name | `artifacts/logs/T036/header-and-secret-predicates.json` | 9 | "sh_github_token": false |
| secret_refs | secret-name | `artifacts/logs/T037/negative-validation-summary.json` | 12 | "validation_id": "raw_secret_pattern_scan", |
| secret_refs | secret-name | `artifacts/logs/T037/negative-validation-summary.json` | 13 | "check_name": "raw_secret_patterns", |
| secret_refs | secret-name | `artifacts/logs/T037/validate-env-tables-negative.json` | 201 | "validation_id": "required_table_secrets_exists", |
| queues | task-queue | `artifacts/recovery/old-pack-context/flexnetos_production_execution_pack/logs/T013-runner-audit.md` | 36 | - `fxrun forge-loop` surfaces include `run`, `eval`, `research`, `self-upgrade`, `doctor`, runner health/flow/fleet/queue audits, agentic-system audit, docs drift, components audit, target mining audit, and output schema |
| queues | task-queue | `oh-my-codex-main/CHANGELOG.md` | 1803 | - Team operator docs now clarify Claude-pane Enter (`C-m`) can queue while busy and document state-first/safe manual intervention guidance for `$team`. |
| queues | task-queue | `oh-my-codex-main/DEMO.md` | 192 | │ │ Shared Task Queue │ │ |
| queues | task-queue | `oh-my-codex-main/DEMO.md` | 451 | [3/8] Create task → Creates a test task in the shared queue |
| queues | task-queue | `oh-my-codex-main/package-lock.json` | 1437 | "yocto-queue": "^0.1.0" |
| queues | task-queue | `oh-my-codex-main/package-lock.json` | 2008 | "node_modules/yocto-queue": { |
| queues | task-queue | `oh-my-codex-main/package-lock.json` | 2010 | "resolved": "https://registry.npmjs.org/yocto-queue/-/yocto-queue-0.1.0.tgz", |
| queues | task-queue | `oh-my-codex-main/.codex/.tmp/plugins/plugins/atlassian-rovo/skills/search-company-knowledge/SKILL.md` | 211 | According to the Confluence documentation, they process jobs from the queue and |
| queues | message-broker | `oh-my-codex-main/.codex/.tmp/plugins/plugins/atlassian-rovo/skills/search-company-knowledge/SKILL.md` | 421 | - Process jobs from a Redis queue |
| queues | task-queue | `oh-my-codex-main/.codex/.tmp/plugins/plugins/atlassian-rovo/skills/search-company-knowledge/SKILL.md` | 427 | worker-queue pattern where: |
| apis | protocol | `LOCAL_WORKAROUNDS.md` | 122 | ## Codex GitKB MCP Route Dedup - 2026-07-03 |
| apis | protocol | `LOCAL_WORKAROUNDS.md` | 124 | The active Codex config had two GitKB MCP server registrations: |
| apis | protocol | `LOCAL_WORKAROUNDS.md` | 127 | gitkb -> /home/flexnetos/FlexNetO... |
| apis | protocol | `LOCAL_WORKAROUNDS.md` | 128 | gitkb-yazelix -> /home/flexnetos/FlexNetO... |
| apis | protocol | `LOCAL_WORKAROUNDS.md` | 136 | /home/flexnetos/FlexNetO....tar.gz |
| apis | protocol | `LOCAL_WORKAROUNDS.md` | 143 | Codex MCP: gitkb only |
| apis | protocol | `LOCAL_WORKAROUNDS.md` | 144 | Wrapper: /home/flexnetos/FlexNetO... |
| apis | protocol | `LOCAL_WORKAROUNDS.md` | 149 | manual use, but it should not be registered as a second top-level Codex MCP |
| apis | protocol | `LOCAL_WORKAROUNDS.md` | 467 | The installed GitKB Codex plugin payload has a portable MCP declaration using |
| apis | protocol | `LOCAL_WORKAROUNDS.md` | 468 | `git-kb mcp`. This host's live MCP front door remains pinned through the |
| caches | build-cache | `LOCAL_WORKAROUNDS.md` | 119 | the packaged `0.143.0-alpha.35` Nix-store binary until a patched Codex package, |
| caches | cache-api | `artifacts/generated/T036/bootstrap.nu` | 34 | $env.XDG_CACHE_HOME = "/home/flexnetos/.cache" |
| caches | cache-api | `artifacts/generated/T036/bootstrap.sh` | 35 | export XDG_CACHE_HOME='/home/flexnetos/.cache' |
| caches | cache-api | `artifacts/generated/T036-repeat/bootstrap.nu` | 34 | $env.XDG_CACHE_HOME = "/home/flexnetos/.cache" |
| caches | cache-api | `artifacts/generated/T036-repeat/bootstrap.sh` | 35 | export XDG_CACHE_HOME='/home/flexnetos/.cache' |
| caches | build-cache | `artifacts/recovery/old-pack-context/flexnetos_production_execution_pack/execution_artifacts/blocked_decisions.md` | 22 | \| Kache \| required, replaces sccache \| |
| caches | redis | `artifacts/recovery/old-pack-context/flexnetos_production_execution_pack/execution_artifacts/blocked_decisions.md` | 42 | 10. **Non-v0.1 database services.** Postgres/Neon, Redis/Valkey, Qdrant, Chroma, DuckDB, LanceDB, RocksDB, sled/redb, DataFusion/Arrow stay deferred unless source scan proves a v0.1 dependency. |
| caches | redis | `artifacts/recovery/old-pack-context/flexnetos_production_execution_pack/execution_artifacts/database_env_audit.md` | 30 | - Redis / Valkey. |
| caches | cache-api | `artifacts/recovery/old-pack-context/flexnetos_production_execution_pack/execution_artifacts/yazelix_comprehensive_setup.md` | 64 | `yazelix-zellij-bar` can be used standalone; its optional provider widgets need tokenusage on `PATH` for Codex/Claude and default or explicit SQLite DB paths for OpenCode Go. Its cache files live under `$XDG_CACHE_HOME/y |
| caches | build-cache | `artifacts/recovery/old-pack-context/flexnetos_production_execution_pack/logs/T024-kache-readme-main.md` | 162 | verdict. For sccache comparison runs it isolates the sccache daemon/cache, |

## Gaps

No empty runtime dependency categories in the safe scan.

## Evidence Boundary

- Environment variables and secret references are recorded by reference name only when names are visible in non-secret files.
- Secret values, `.env` files, private key material, PEM files, and blocked directories are excluded.
- Queue/API/cache rows are evidence categories from repository files and envctl database reports, not a claim of deployed live services unless later runtime inventory confirms them.
