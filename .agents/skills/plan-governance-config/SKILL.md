---
name: plan-governance-config
description: >-
  Audit the planning target's control plane plus settings/config plane: rules, instructions, hooks,
  policy, CLAUDE.md/AGENTS.md, .claude/.codex settings, .meta.yaml, Cargo/toolchain, .kb, .handoff,
  CI, bun, MCP rot, skill overload, token burn, permission/config drift. Produces cited governance+
  settings+config CLAIM/UPGRADE rows for planning-engineer.
---

# plan-governance-config — control-plane + settings/config axis

Use this method for the planning prompt's governance/settings/config scan. It turns non-code control
surfaces into evidence-backed planning inputs, with the same fail-closed discipline as code findings.

## Required surfaces

Scan and cite the live paths that exist; record missing expected surfaces as findings.

- Governance: `.claude/rules/*.md`, `CLAUDE.md`, `AGENTS.md`, `.kb/AGENTS.md`, `.handoff/hooks/*`,
  `.handoff/policies/*`, `.handoff/policy.toml`, ADRs, `.githooks/*`, preflight scripts.
- Settings/runtime: `.claude/settings.json`, `.codex/config.toml`, `.codex/hooks.json`,
  `.claude/agent-guard.toml`, permissions/env/model/effort/MCP/agents/features/hook wiring.
- Project/build/tool config: `.meta.yaml`, Cargo files, `rust-toolchain.toml`, `.cargo/config.toml`,
  `.kb/config.toml`, `.github/workflows/*.yml`, `package.json`/`bun.lock`/`bunfig.toml`, manifests,
  envctl locks, `.env*`/`.envrc`.

## Required detectors

1. **MCP rot** — enumerate MCP servers, identify dead/duplicate/unused/stale/broken optional servers.
2. **Skill overload** — count skills, compare with listing budget and actual trigger value.
3. **Token burn** — model/effort/listing/context/cadence settings that waste budget or stall loops.
4. **Permission drift** — broad allows, missing denies, guard/policy mismatch, fail-open paths.
5. **Config drift** — toolchain/CI/deps/lockfile/meta/manifest drift; JS must use bun, not pnpm/node.

## Output schema

Write `findings/governance-config-<T>.md` with:

- surfaces scanned table,
- hygiene detectors table,
- `CLAIM[...] axis: governance+settings+config` rows,
- `UPGRADE[...] axis: governance+settings+config` rows carrying `target_surface`, `evidence`,
  `expected_impact`, `effort`, `risk_tier: APPLY|PROPOSE|REGENERATE`, `acceptance`, and
  `reversibility`,
- gaps / owner walls.

## Routing law

APPLY only low-risk documentation/checklist/additive-test improvements. PROPOSE hooks, policies,
permissions, MCP/settings, agent runtime, build/toolchain/CI, loop cadence, destructive guards, and
owner-canon changes. REGENERATE lockfiles/generated files with their owning tools. Never weaken gates.
