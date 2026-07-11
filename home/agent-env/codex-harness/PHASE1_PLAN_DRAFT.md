# Codex Harness Phase 1 Plan Draft

Status: operator-approved direction, saved before implementation.
Saved from host shell because attached Codex tool sandbox failed with:
`bwrap: Can't mkdir /.git: Permission denied`

## Scope

PROJECT_ROOT=/home/flexnetos/meta/src/envctl/home
HARNESS_ROOT=/home/flexnetos/meta/src/envctl/home/agent-env
HARNESS_WORKSPACE=/home/flexnetos/meta/src/envctl/home/agent-env/codex-harness

## Approval

Phase 1 is approved for the runbook-reconciled containment plan.

## Runbook correction

Do not default to local SQLite.

Use:
- JSONL ledgers as canonical source of truth.
- Pure-Rust redacted index backend, preferably redb.
- SQLite only behind later explicit exception.

## envctl ownership boundary

envctl agent owns:
- skills
- commands
- MCP packs
- agent-env.yaml
- agent-env.lock

envctl agent does not own SOUL/AGENTS instructions in this absorbed v3.2.0 surface.
Do not use `instructions:` in agent-env.yaml.

Codex/harness owns:
- AGENTS/SOUL layering
- rules
- hooks
- policy
- profiles
- provider routing
- model catalog
- status overlay
- containment gates

## MCP reconciliation gate

Runbook baseline mentions:
- github
- context7
- exa
- memory
- playwright
- sequential-thinking

Phase 0 observed active Codex MCP list was empty.
Do not blindly widen MCP scope.
Use active /home/flexnetos/.codex/config.toml as runtime authority.
Use envctl preview and lock checks first.
Additive merge only.
No mutating MCP tools without explicit approval.

## First Phase 1 actions when Codex tool execution works

1. Re-run bootstrap proof:
   - date -u
   - pwd
   - whoami
   - command -v codex
   - readlink -f "$(command -v codex)"
   - codex --version
   - codex features list
   - git status

2. Verify Nix/Yazelix profile-frontdoor ownership.

3. Archive every existing path before modification.

4. Save archive events into ledger/archive.jsonl.

5. Implement containment before capability:
   - deny yolo/danger/full-access without decision id
   - deny secret reads
   - deny unmanaged network
   - deny nested Codex/Claude/model-provider launches outside runner
   - deny uncontrolled background jobs
   - deny GitHub mutation without guard
   - deny browser/computer use without approved profile
   - require model-router before subagent spawn
   - require archive before writes

## Required Rust binaries

Existing:
- codex-harness-hook
- codex-harness-runner
- codex-harness-status
- codex-harness-halt
- codex-harness-audit
- codex-harness-nix-verify
- codex-harness-jsonl

Add or complete:
- codex-harness-model-router
- codex-harness-policy
- codex-harness-index
- codex-harness-github-guard
- codex-harness-memory-audit
- codex-harness-memory-export-redacted
- codex-harness-memory-disable-plan

Conditional only after compatibility and approval:
- codex-harness-openrouter-shim
- codex-harness-claude-bridge

## Acceptance commands

- envctl agent lock --check --color never
- envctl agent sync --json --color never
- codex mcp list
- codex plugin list
- codex execpolicy check --pretty
- cargo fmt --check
- cargo clippy --all-targets --all-features -- -D warnings
- cargo test --all-features
- codex-harness-audit
- codex-harness-nix-verify
- codex-harness-status
- codex-harness-model-router sample tasks
- codex-harness-index integrity check

## Stop condition

Do not claim Phase 1 complete until real terminal proof exists for archive-first behavior,
containment gates, Rust tests, envctl preview gates, Codex rules/hooks, model routing,
bad-behavior counters, kill switch, and redacted index integrity.
