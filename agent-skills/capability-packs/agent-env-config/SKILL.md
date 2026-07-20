---
name: agent-env-config
description: "The CORRECT conventions and agent-environment configuration for the envctl Rust workspace — supersedes the broken ECC-auto-generated skill/instincts that assert JavaScript conventions. Use whenever writing or reviewing envctl code, naming files/types, writing tests, composing commits, or configuring the .claude/.codex agent setup (skills, MCP servers, multi-agent roles). Triggers: 'what conventions', 'how do I name this', 'write a test', 'commit message', 'configure the agents', 'MCP setup', 'is camelCase right'."
---

# Agent Environment Config & Conventions (envctl)

envctl is a **pure-Rust** Cargo workspace (**9 crates**: engine, cli, gui, agent-env,
secrets-engine, secrets-proto, secretd, secretctl, secrets-store-libsql). The ECC-auto-generated
config (`.claude/skills/envctl/SKILL.md`, `.claude/homunculus/instincts/.../envctl-instincts.yaml`)
was derived from a misread and asserts **JavaScript** conventions. **This skill is the source of
truth; the ECC conventions are wrong — ignore them.**

## Corrections — ECC says X, the truth is Y

| ECC (WRONG) | Correct for envctl (Rust) |
|-------------|---------------------------|
| camelCase file names (`envManager.rs`) | **snake_case** files (`env_manager.rs`) |
| camelCase function names | **snake_case** functions (`load_env`, `relay_mint`) |
| "relative imports" JS-style (`import {x} from '../lib/x'`) | Rust `use` paths: `use crate::vault::store;`, `use super::*;` |
| named exports | `pub` items in modules; `mod` tree declares structure |
| test files named `*.test.ts` | `#[cfg(test)] mod tests { ... }` in the same `.rs`, or `crate/tests/*.rs` integration tests |
| (no lint guidance) | `cargo fmt` + `cargo clippy -- -D warnings`; **no C in the trust boundary** (`ci/gates/no-c.sh`) |

**Correct conventions ECC happened to get right (keep):** PascalCase for structs/enums
(`BearerRow`, `RemotePeer`), SCREAMING_SNAKE_CASE for consts, commit subjects prefixed by area
(`envctl:` / `engine:` / `secretd:`).

## Code Conventions

- **Naming:** snake_case modules/files/functions/vars; PascalCase types/traits/enum variants;
  SCREAMING_SNAKE_CASE consts/statics.
- **Module structure:** the `mod` tree is the API surface; expose with `pub`. Prefer
  `use crate::…` / `use super::…`.
- **Tests:** unit tests in `#[cfg(test)] mod tests` beside the code; integration tests in
  `crates/<crate>/tests/*.rs`; e2e where a daemon is involved (`secretd/tests/e2e.rs`). Run
  `cargo test -p <crate>` or `cargo test --workspace`.
- **Toolchain:** workspace support floor **MSRV 1.89** (`Cargo.toml` `rust-version` is the
  authoritative value). Developer toolchain is **nightly** (`rust-toolchain.toml`
  `channel = "nightly"` —
  the dev channel, not the support floor). Both come from the fenix toolchain in the nix
  profile; never rustup-in-place.
- **Lints & safety:** `cargo fmt` and `cargo clippy -- -D warnings` clean. The engine is sync,
  pure-Rust, **non-printing** (emits events). The supply-chain gate `ci/gates/no-c.sh` forbids
  linking any C library into the trust boundary — never add a dep that pulls one in.
- **Commits:** concise subject prefixed by area (`engine:`, `secretd:`, `agent-env:`,
  `secrets-store-libsql:`, `docs:`); body explains why.

## Agent Environment Layout

The environment targets two agent runtimes; keep them consistent (the built-in agent-env engine
manages this: `agent-skills/` + `agent-env.yaml` → `envctl agent sync --apply` →
`agent-env.lock`; zero-network drift gate `envctl agent lock --check --locked`. kasetto is retired — see
`env-stabilize`):

- **Claude Code** → `.claude/` : managed skills under `.claude/skills/<name>/SKILL.md`, plus
  `.claude/settings*.json`. Do NOT hand-maintain `.claude/homunculus/instincts/...` ECC files —
  superseded by the curated skills.
- **Codex** → `.codex/` : `config.toml` (MCP servers + multi-agent), `AGENTS.md`, role configs
  under `.codex/agents/`. Codex-facing skill mirror under `agent-skills/capability-packs/`.

### MCP baseline (Yazelix mirror only)
The generated agent environment may only install MCP entries that honor the Yazelix ownership
model: editable input under the user config tree, generated runtime as proof only, and
profile-owned frontdoors for local binaries. Today the only MCP entry in this repo baseline that
satisfies that rule is `exa`, because it is remote URL configuration and does not launch a local
Meta/toolchain binary.

Do not restore `github`, `context7`, `memory`, `playwright`, or `sequential-thinking` from
`agent-skills/mcps`, workspace mirrors, marketplace caches, or old locks. Those entries used
Meta `bunx` or repo-source scripts as active launchers. They can return only after their owning
package/profile path exposes a Yazelix-mirrored frontdoor and `agent-env.yaml` plus
`agent-env.lock` are regenerated from that source.

### Multi-agent roles (Codex)
Three read-only roles are the baseline: **explorer** (gather evidence before changes),
**reviewer** (correctness/security/tests), **docs-researcher** (verify APIs against primary
sources). Keep them read-only; mutation happens in the main thread.

## Why this matters
Agents act on whatever the environment tells them. If the config says "camelCase + *.test.ts",
an agent produces non-idiomatic Rust that fails fmt/clippy and confuses review. A correct,
curated environment is the precondition for finishing envctl.
