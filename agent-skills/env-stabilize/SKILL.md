---
name: env-stabilize
description: "How to keep the environment reproducible and drift-free — the detect/drift, doctor-diagnostic, and content-hashed lock discipline envctl uses, plus how the built-in agent-env engine (envctl agent) provisions and locks the agent config. Use when checking environment health, diagnosing drift, regenerating or verifying a lock, or making the agent environment reproducible. Triggers: 'is the environment stable', 'check for drift', 'run doctor', 'the lock is stale', 'make this reproducible', 'env is inconsistent', 'sync the agents'."
---

# Environment Stabilization (drift · doctor · lock)

A stable environment is **fully declared and reproducible**: every tool and every agent-config
file is described by a manifest/source and pinned by a content hash, so drift is detectable and
repair is mechanical. Three layers own it: the **nix profile** (`lifeos-foundation-yzx`, built
from the local yazelix checkout) owns every toolchain binary; **envctl** owns meta-local
components (`manifest/*.toml`, `envctl.lock`); the built-in **agent-env engine**
(`crates/agent-env`, driven by `envctl agent`) owns the `.claude`/`.codex` toolkit
(`agent-env.yaml` → `agent-env.lock`). kasetto was absorbed into `crates/agent-env` and the
external binary retired (TASK-0018/#98; files renamed `kasetto.*` → `agent-env.*`, TASK-0040) —
every `kasetto` verb is dead.

## The Three Disciplines

### 1. Drift detection
Drift = live state diverges from declared state. For the agent environment, drift = installed
`.claude`/`.codex` skills/MCPs differ from `agent-skills/` + `agent-env.yaml`. Detect before
trusting; never "fix" by editing live synced files (that hides drift) — fix the source and
re-sync. Drift gate: `envctl agent lock --check --locked` (read-only, zero-network, exit 1 on drift;
CI: `ci/gates/agent-env.sh`).

### 2. Doctor diagnostics
A doctor pass is a read-only health report: present / broken (detects-present, fails verify) /
missing / drifted. Run before and after any environment change: `envctl doctor` (box),
`envctl agent lock --check --locked` (agent layer), `yzx doctor` (yazelix runtime). Non-green doctor =
blocker, not warning.

### 3. Content-hashed lock
The lock is the manifest-of-record, committed and CI-gated. envctl commits `envctl.lock`
(components, content-hashed; gate `lock --check`). The agent layer commits `agent-env.lock`
(skills + MCP assets, hashes OS-normalized). The lock is authoritative: a plain
`envctl agent sync` installs exactly what the lock pins; only an explicit update re-resolves
moving sources.

## envctl agent as the Stabilizer

- **Source of truth:** `agent-skills/` + `agent-env.yaml` (project scope), targeting
  `claude-code` + `codex`. Config override: `ENVCTL_AGENT_CONFIG`.
- **`envctl agent sync --apply`** writes skills into each agent dir, merges MCP servers
  additively into each agent's native format (Claude JSON, Codex TOML), records everything in
  `agent-env.lock`. Preview (no `--apply`) is the default — fail-closed, dry-run ethos.
- **Reproducibility test:** a second sync is a no-op. If it is not, something drifted —
  investigate before proceeding.
- **CI gate:** `ci/gates/agent-env.sh` runs `envctl agent lock --check --locked`.

## Layer table — compose, don't duplicate

| | **nix profile** | **envctl** | **envctl agent (agent-env)** |
|---|---|---|---|
| Manages | toolchain binaries and runtimes | meta-local components, wiring, daemons | the agent toolkit (skills · MCPs) |
| Targets | `~/.nix-profile` / toolbin | `$META_ROOT`, `manifest/*.toml` surfaces | `.claude/` + `.codex/` |
| Lock / verbs | `flake.lock`; `yzx update local_source`, `yzx doctor` | `envctl.lock`; `doctor/install/auto-fix/reset/lock --check` | `agent-env.lock`; `agent sync [--apply]`, `agent lock --check --locked` |

**Rules that make it pay off:**
1. **One source of truth — never hand-edit what the sync owns.** Change `agent-skills/` +
   `agent-env.yaml`, then `envctl agent sync --apply`; never touch the generated
   `.claude/skills/<managed>/*`, `.mcp.json`, `.codex/config.toml`.
2. **Multi-agent parity for free.** One config keeps `claude-code` and `codex` consistent.
3. **Know what stays OUT of the sync.** The hand-authored harness skills named in envctl
   `CLAUDE.md` (the feature-forge family, `env-install-loop`, `auto-provision`,
   `session-relay*`) are git-tracked and edited in place — bespoke,
   fast-iterating orchestration. The sync is for the stable, locked baseline only.
4. **Toolchain requests route to the owner.** A missing binary is a nix-profile item
   (yazelix flake → `yzx update local_source`), not an ad-hoc install and usually not an
   envctl component.

## Stabilization Workflow

1. `envctl doctor` + `envctl agent lock --check --locked` + `yzx doctor` → baseline health.
2. Reconcile: `.claude`/`.codex` managed surfaces come from `agent-skills/`, nothing else.
3. Edit source → `envctl agent sync --apply` → commit `agent-env.yaml` + `agent-env.lock`.
4. Re-run sync → confirm no-op (proves reproducibility).
5. From now on, changes go through the source + sync, never by hand-editing live agent files.

## Why
Without lock + doctor + drift discipline the environment silently rots: someone edits a live
`.claude` file, a regen overwrites it, conventions drift, and agents act on stale config.
Locking makes the environment a committed, verifiable artifact.
