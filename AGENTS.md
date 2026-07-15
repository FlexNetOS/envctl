# AGENTS.md

This file provides guidance to Codex (Codex.ai/code) when working with code in this repository.

## What this is

`envctl` is a first-class **meta** peer member — the **agentic environment manager for the
whole meta workspace**. It is a **pure-Rust Cargo workspace** (8 crates) that declaratively
brings every tool, dependency, provider, vendor, CLI, and config to a declared state and
installs it **into meta** (`$META_ROOT/{usr/bin,usr/lib,usr/share,etc,var/lib,var/cache,var/log,var/tmp,opt} plus XDG meta-home roots`,
legacy `meta/.toolchains/`, `$META_ROOT`), with **no system-depth or user-global installs** —
anything meta uses lives in meta. Its deployment target today is a dual-RTX-5090 Ubuntu 26.04
workstation. Two halves share one engine:

- **env-manager** — `engine` + `cli` (`envctl`) + `gui` (`envctl-gui`). Brings the box to
  a declared state via TOML *components* whose lifecycle hooks wrap the proven bash in
  `assets/scripts/`. Verbs: `auto-detect`, `install`, `auto-fix`, `reset`, `add-repo`,
  `graph`, `lock`, `doctor` (see `README.md`).
- **secrets stack** — `secrets-engine` (pure-Rust crypto vault), `secrets-proto` (tonic/prost
  gRPC), `secretd` (async tokio daemon), `secretctl` (client), `secrets-store-libsql`
  (libSQL **remote** backend). Design corpus in `docs/secrets/`.

## Session start: work in a fresh git worktree (mandatory)

This repo lives inside the `meta` workspace. **Begin every session by creating an isolated
worktree** rather than editing the checked-out tree directly. After verifying sync
(`git fetch && git status` — confirm clean and even with `origin/master`):

```bash
meta git worktree create <task-slug> origin/develop --repo envctl   # preferred: meta-managed
# (bare `meta git worktree create <slug>` BAILS: repos must be named via --repo/--all;
#  building in the set needs detached loop_lib/meta_plugin_protocol siblings from meta/src/*)
# or, single-repo: git worktree add ../envctl-<task-slug> -b <task-slug>
```

After entering a checkout, prove whether it is a main checkout or a meta-managed
worktree before using worktree-set commands:

```bash
bash scripts/reap-worktrees.sh --managed-worktree-slug "$(git rev-parse --show-toplevel)" envctl
```

This prints a slug only for `meta/.worktrees/<slug>/envctl`. The main checkout
(`/home/drdave/Desktop/meta/envctl`) has no managed slug. Never pass the repo
name `envctl` to `meta git worktree status` unless this helper derived it from
path shape (for example, the valid but uncommon `.worktrees/envctl/envctl`).

Do all work in the worktree; never start coding on a stale or dirty `master`.

## Build / test / lint

All command examples below are payloads for the profile-owned RTK frontdoor.
Use `/home/flexnetos/.nix-profile/bin/rtk <supported-command> ...` when its
summary is sufficient, or `/home/flexnetos/.nix-profile/bin/rtk proxy --
<command> ...` when exact stdout/stderr is required. Direct raw shell execution
is not an alternate path.

```bash
cargo build -p envctl-engine -p envctl       # engine + CLI, zero system deps
cargo run  -p envctl -- auto-detect          # read-only, safe anytime (add --json for EnvReport)
cargo run  -p envctl-gui                      # needs system dev libs — see README "Native GUI"
cargo test --workspace                        # all crates
cargo test -p envctl-secrets-engine vault     # single crate / filter by test name
cargo test -p envctl-secretd --test e2e       # one integration test file (daemon e2e)
cargo +1.88.0 check --workspace --locked      # MSRV floor, default feature graph
cargo fmt --all && cargo clippy --workspace -- -D warnings   # must be clean before commit
```

Tests are inline `#[cfg(test)] mod tests` beside the code, or `crates/<crate>/tests/*.rs`
integration tests (`#[tokio::test]` for the async daemon path). MSRV 1.88, stable toolchain
(`rust-toolchain.toml`).

## CI gates — run before pushing anything that touches deps or the trust boundary

```bash
bash ci/gates/runner-routing.sh # GitHub Actions hybrid hosted/local runner policy
bash ci/gates/no-c.sh           # supply-chain: forbids C in the trust boundary (see below)
bash ci/gates/meta-substrates.sh # meta shared-substrate wiring: loop_lib + meta_plugin_protocol path deps
bash ci/gates/shape.sh          # code-shape invariants (native-roots, edge module)
bash ci/gates/enable.sh         # secretd systemd-unit enable invariant
bash ci/gates/p7.sh             # .handoff Tier-A p7-conformance: schema tags + ledger residency (ADR-0004 §3)
bash ci/gates/kdf-feature-off.sh # test-speed Argon2 floor must be off by default (TASK-0032)
bash ci/gates/agent-env.sh      # agent-env.yaml ↔ agent-env.lock no-drift (TASK-0040)
bash ci/gates/meta-local-policy.sh # active install sources target $META_ROOT FHS/XDG only; Yazelix real-home Nix profile preserved
bash ci/gates/yazelix-codex-runtime.sh # Yazelix/Codex runtime ownership: no retired MCPs, stale roots, or user-bin Codex shadows
bash ci/gates/cargo-audit.sh    # RustSec advisories; fails vulnerable tonic/hyper regressions
bash ci/gates/loop-state.sh     # forge-loop counter integrity: ints, cadence>=1, cycles_total monotonic & >= last_wrapup (TASK-0041)
bash ci/gates/harness-scripts.sh # Feature-Forge harness tooling safety (merge-driver + reaper + loop-state-gate invariants)
```

## NON-NEGOTIABLE invariants (a change that breaks these is a regression)

- **No C library in the trust boundary.** No SQLite/OpenSSL/aws-lc may be *linked*. The store
  uses libSQL `remote` only (`default-features = false`); crypto is pure-Rust (ring, blake3,
  chacha20poly1305, argon2). `ci/gates/no-c.sh` proves this fail-closed from the resolved
  `cargo metadata` graph — **never add a dependency that pulls one of the banned crates in.**
- **Exactly one rustls, ring-only** (not aws-lc-rs). All TLS/CA crates pin `features = ["ring"]`.
- **envctl is wired into meta, not special/excluded.** Keep sibling path dependencies on
  `loop_lib` and `meta_plugin_protocol`. `ProcessRunner` may own envctl-specific supervision
  (setsid, timeout, streaming, tee), but `runner.rs` must delegate `std::process::Command`
  construction to `loop_lib::build_command`. If the shared substrate lacks a needed API, upgrade
  that substrate first and consume it here; **never downgrade envctl by removing or bypassing it**.
  `ci/gates/meta-substrates.sh` proves this fail-closed in CI.
- **The engine is the single shared library** (`crates/engine/src/lib.rs`): sync, pure-Rust,
  **non-printing** (emits `Event`s, never `println!`), no UI, no clap. CLI and GUI both drive
  the *identical* `Engine` API so the front-ends can't diverge. Put logic in the engine, not in
  `main.rs` or the GUI.
- **Destructive ops are fail-closed and dry-run by default.** Guards (`UuidResolves`,
  `NotLiveDevice`, `NotMounted`) *refuse* when they can't prove safety (unit-test enforced).
  `auto-fix`/`reset`/`add-repo` default to preview; mutation needs `--apply`/`--build`.

## CRITICAL: keep everything rust-native — detect and reverse language drift

This is a **pure-Rust** workspace by design. Watch for and immediately correct any drift toward
another language or toolchain:

- **No new non-Rust source/package files** should appear in the workspace. If an external tool
  emits one — e.g. a stray `.omc` file, or **ECC auto-pushing a JS/Node package** — treat it as
  drift, not as intended state.
- **When drift is found:** (1) verify it (don't act on a false positive — confirm the file/dep
  is actually language drift and not an accepted build-time artifact like the libSQL parser's
  `lemon.c` codegen, which emits Rust and links nothing); (2) **transform it to a rust-native
  equivalent** (a workspace crate, a TOML component, a pure-Rust dependency); (3) **sync it
  properly** into the codebase — add the crate to `Cargo.toml` `members`, wire it through the
  `Engine` API, and update `agent-env.lock`/`envctl.lock` so the reproducible state reflects it.
- The `add-repo --refactor=ai --goal port-to-rust` verb is the sanctioned path for porting an
  external repo into the workspace as a Rust crate. Use it (or its design as a template) rather
  than carrying foreign-language code as-is.

## Agent environment is agent-env-managed (absorbed kasetto) — do NOT hand-edit ECC files

The `.Codex/` and `.codex/` agent config (skills + MCP baseline) is **provisioned and locked
by the built-in agent-env engine** (`agent-env.yaml` → `agent-env.lock`, driven by `envctl agent`),
sourced from `./agent-skills`. (kasetto v3.2.0 was absorbed into `crates/agent-env` and the external
`kasetto` binary retired — TASK-0018/#98; the config/lock were renamed `kasetto.yaml`/`kasetto.lock`
→ `agent-env.yaml`/`agent-env.lock` — TASK-0040.) It supersedes the **ECC-auto-generated** files,
which were derived from a misread and assert **JavaScript** conventions (camelCase, `*.test.ts`,
JS imports) — those are **wrong for this repo**.

- **Source of truth for conventions:** the `agent-env-config` skill. Rust idiom: snake_case
  files/modules/functions, PascalCase types, SCREAMING_SNAKE_CASE consts, `#[cfg(test)]` tests,
  area-prefixed commit subjects (`engine:`, `secretd:`, `docs:`). Ignore any ECC instinct/skill
  that says otherwise.
- **To change the agent env:** edit `agent-skills/` + `agent-env.yaml`, then `envctl agent sync --apply`
  (the built-in agent-env engine; the external `kasetto` binary is retired — TASK-0018).
  Do **not** hand-maintain `.Codex/skills/*` or `.Codex/homunculus/instincts/*` — they're
  generated. CI enforces with `envctl agent lock --check` (read-only, zero-network, exits 1 on
  drift — `ci/gates/agent-env.sh`, TASK-0040).
- Keep the MCP baseline identical across Codex (`.mcp.json`) and Codex (`.codex/config.toml`):
  `github`, `context7`, `exa`, `memory`, `playwright`, `sequential-thinking`.
- Treat that envctl baseline as a repo-local MCP floor, not as authority to
  widen the active home/runtime plugin marketplace surface. Do not rehydrate
  removed plugin catalogs, temp marketplace caches, or not-installed plugin
  families such as `superhuman` or `digitalocean` into the active runtime just
  because they appear in cached bundles, old renders, or inventory output.
- Envctl catalog render is an audit/projection surface for repo outputs. It is
  not proof that the active home Codex runtime should preserve or restore every
  observed `mcp_servers` or plugin listing from an older `.codex/config.toml`.
- When envctl work touches Yazelix-integrated runtime behavior, follow the
  Yazelix ownership contract exactly: the user config tree is the editable
  input surface, the real-home data tree is generated runtime output, and the
  active launch frontdoor is the profile-owned `yzx`. Do not hand-edit
  generated runtime files or preserve stale user-bin wrappers or per-user
  desktop-entry shadows as parallel control paths.
- Codex binary/runtime ownership must mirror the Yazelix binary/runtime model
  with no drift. Yazelix is the normative model, not a comparison target or one
  possible runtime shape among several. Current Codex, binary, toolchain,
  launcher, plugin, MCP, or generated-output state that breaks the Yazelix
  input/output/frontdoor/shadow contract is proof of failed alignment and is
  required repair work, not neutral inventory and not a valid parallel
  ownership layer. Inventory current state only to locate violations, map each
  violation to its owning input/generator/package/profile path, and then prove
  the repaired state mirrors Yazelix. Preserve this explicit mapping and its
  source-backed proof:
  - Codex and every toolchain/runtime binary must use the same profile-owned
    binary and profile-owned runtime-config location model as Yazelix. This is
    not limited to `codex`: it includes `rtk`, `git-kb`, terminal helpers,
    package-manager shims, MCP/plugin launchers, and any other command exposed
    to the active runtime. User-bin shadows, repo-cache materializations, temp plugin bundles, marketplace caches, and generated-output files are never alternate active locations. No exceptions.
  - The user config tree is the main editable input surface, including
    `settings.jsonc` and managed overrides, per the Yazelix customization and
    POSIX XDG docs.
  - The real-home data tree is generated runtime output; edit config inputs,
    not generated runtime files, per the Yazelix customization docs and README.
  - The profile-owned `yzx` binary is the active install-owner frontdoor, per
    the Yazelix Home Manager docs.
  - Legacy user-bin `yzx` wrappers are stale shadows when they shadow the
    profile path, per the Yazelix Home Manager docs.
  - Per-user desktop entries are stale shadows when they shadow the active
    profile desktop entry, per the Yazelix Home Manager and troubleshooting
    docs.

## Agent navigation and retired mirror paths (2026-07-07)

New sessions must enter this repo through a fresh worktree at the latest
`origin/master`/`origin/develop`, then change agent configuration through the
owning envctl surfaces:

```bash
git fetch origin --prune
git worktree add ../envctl-<task-slug> -b <task-branch> origin/master
cd ../envctl-<task-slug>
envctl agent lock --check --color never
envctl agent sync --json --color never       # preview only
# after review/approval only:
envctl agent sync --apply --color never
```

Use these locations only:

| Purpose | Active location |
| --- | --- |
| Global Codex runtime config | `/home/flexnetos/.codex/config.toml` |
| Global Codex operating rules | `/home/flexnetos/.codex/RULES.md` |
| Home-level navigation entry | `/home/flexnetos/AGENTS.md` |
| RTK policy include | `/home/flexnetos/.codex/AGENTS.rtk.md` and `/home/flexnetos/AGENTS.rtk.md` |
| Repo-managed agent inputs | `agent-env.yaml`, `agent-env.lock`, `agent-skills/` |
| Repo home projection | `home/.codex/`, `home/.claude/`, `home/AGENTS.md` |
| Codex harness source | `home/agent-env/codex-harness/` |

Do **not** use `/home/flexnetos/lifeos/.codex` or
`/home/flexnetos/FlexNetOS/.codex` as active config, mirror, source, or
fallback. They were retired because `/home/flexnetos/FlexNetOS` is a symlink to
`/home/flexnetos/lifeos`, so those two paths were the same confusing control
surface. If either path reappears, archive it and route the change through
`/home/flexnetos/.codex` or envctl `agent-env.yaml` as appropriate.

## Pointers

- `docs/ARCHITECTURE.md`, `docs/ROADMAP.md`, `docs/DESIGN-NOTES.md` — env-manager design.
- `docs/secrets/SERVER-MODE.md`, `THREAT-MODEL.md`, `DESIGN-NOTES.md` — secrets-stack design;
  feature IDs (F12/F14/F15, OI-*, CF-*) referenced in commits and gate comments live here.
- `manifest/*.toml` — declarative components; drop-ins land in `manifest/components.d/`.
- The manifest dir defaults to `./manifest` (override with `ENVCTL_MANIFEST_DIR`).
- Logging: `RUST_LOG` (e.g. `RUST_LOG=envctl_engine=debug`).

## meta mission-control dashboard (zellij layout)

The `dashboard` component (`manifest/dashboard.toml`) installs two launchers on `$META_ROOT/usr/bin`:
- `envctl-dashboard-pane <repo>` — called by every pane in the generated zellij
  `mission-control.kdl` layout.
- `envctl-open-Codex` — run by a human inside a pane when they actually want a
  Codex session.

**Default behavior:** dashboard panes open a plain shell, not an idle Codex session.
`envctl-dashboard-pane` only starts `Codex` when `ENVCTL_DASHBOARD_AUTO_CLAUDE=1`
is set. This prevents accidental background Codex sessions and auto-spawn loops.
To start Codex in a pane, run `envctl-open-Codex` (which sets the opt-in env var
and preserves the pane's mesh identity: `META_REPO`, `MESH_IDENTITY`, `WEAVE_*`,
`REPOWIRE_*`).

## Harness: Feature Forge (the construction crew)

**Goal:** turn a feature / upgrade / design request into invariant-verified working Rust, fast —
a design → implement → verify crew. The crew *builds* the feature; it is not the building.

**Trigger:** for any request to add / build / implement / design / upgrade / extend / refactor an
envctl feature, Engine method, CLI/GUI surface, secrets-stack capability, or manifest component
(and follow-ups like "re-run", "fix the guardian's findings", "revise the design"), use the
**`feature-forge`** skill. It drives `feature-architect` → `rust-implementer` →
`invariant-guardian`. For **continuous/autonomous** runs over a backlog ("keep building", "loop on
the roadmap", "run unattended") use **`forge-loop`**; for **cross-session handoff/resume** ("transfer
the session", "resume from handoff") use **`session-relay`** (checkpoints via `continuity-steward`,
coordinates over **weave**, schedules a best-effort successor cron at a per-session cycle budget).
To **provision the whole box / install all toolchains, PATH, and env vars in a loop until
`doctor` is green** ("install everything", "set up the box", "loop until installed"), use
**`env-install-loop`** (the same loop+relay continuity, driving envctl's `doctor`/`install`/
`auto-fix` verbs + `env-toolchain-install`). For **fully unattended, self-restarting** provisioning
with a fresh context every cycle ("run it overnight / set-and-forget", "auto-provision", "cycle
install and reset until done") use **`auto-provision`** — the external Ralph runner that spawns a
fresh `Codex -p` per cycle (the `/new` effect) wrapping `env-install-loop`. To **build/install the
`hf` continuity kernel and bring `.handoff` to Tier-A** ("build hf", "sync the handoff layer",
"make .handoff tier-A", "resume handoff full-sync") use **`handoff-sync`** (Epic A; distinct from
`session-relay`, which is the per-loop checkpoint). Simple questions and
trivial edits may be answered/done directly. (A SINGLE component install → `env-toolchain-install`;
drift/lock/doctor → `env-stabilize`; conventions → `agent-env-config`.)

**Placement:** the harness is **hand-authored and git-tracked**, intentionally *outside* the
kasetto/agent-env pipeline. Agent definitions live in `.Codex/agents/*.md` and the harness skills
(`feature-forge`, `rust-feature-impl`, `forge-loop`, `session-relay`, `env-install-loop`,
`auto-provision`, `handoff-sync`) live directly in `.Codex/skills/` — edit those files in place and commit them. They are **not** sourced from `agent-skills/`, not in `agent-env.yaml` /
`agent-env.lock`, and not produced by `envctl agent sync`. (Note: this is a deliberate exception to the
general "`.Codex/skills/*` are kasetto-generated" rule above — the kasetto-managed skills remain
`agent-env-config`, `env-stabilize`, `env-toolchain-install`.)

> **Packaged upstream (TASK-0052, owner-locked 2026-06-18):** the generic construction-crew core —
> `feature-forge` + `forge-loop` + `rust-feature-impl` + the architect/implementer/guardian/
> kernel-engineer specialists — is now ALSO a **registered, ejectable packaged harness in
> `harness_hub`** (`/harness:feature-forge`, `harness_hub/harness/skills/feature-forge/` + prefixed
> `harness/agents/feature-forge-*`, `registry.json`/`entries/feature-forge.md`; harness_hub PR #38).
> This **supersedes the "hand-authored, never packaged" stance for that core**: the hub package is the
> reusable source-of-truth (the envctl `.Codex/` copies are an ejected instance that may be
> re-synced via the package's `eject.sh`). The **envctl-specific loops** (`env-install-loop`,
> `auto-provision`, `handoff-sync`) are NOT generically reusable and remain hand-authored in envctl
> only — they are deliberately out of the hub package's scope.

> **Packaged planning upstream (2026-06-26):** the recovered **planning-engineer** harness is now a
> registered, ejectable packaged harness in `harness_hub` (`/harness:planning-engineer` for one
> evidence-backed planning cycle and `/harness:plan-loop` for the continuous Ralph loop). The envctl
> `.claude/`/`.agents/` copies are ejected mirrors; the hub package is the reusable source-of-truth.
> The loop is read-only on product code and writes planning artifacts under `.handoff/loop/plan/`.

**Change history:** The archival table moved to
[`docs/AGENTS-CHANGE-HISTORY.md`](docs/AGENTS-CHANGE-HISTORY.md). It is kept
outside this always-loaded instruction surface to reduce repeated agent context;
consult it only when historical provenance is relevant.

<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **envctl** (14492 symbols, 30909 relationships, 300 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

> Index stale? Run `node .gitnexus/run.cjs analyze` from the project root. No `.gitnexus/run.cjs` yet? Regenerate it with profile-owned `bunx gitnexus@latest analyze`; never use a global package-manager install.

## Always Do

- **MUST run impact analysis before editing any symbol.** Before modifying a function, class, or method, run `impact({target: "symbolName", direction: "upstream"})` and report the blast radius (direct callers, affected processes, risk level) to the user.
- **MUST run `detect_changes()` before committing** to verify your changes only affect expected symbols and execution flows. For regression review, compare against the default branch: `detect_changes({scope: "compare", base_ref: "master"})`.
- **MUST warn the user** if impact analysis returns HIGH or CRITICAL risk before proceeding with edits.
- When exploring unfamiliar code, use `query({search_query: "concept"})` to find execution flows instead of grepping. It returns process-grouped results ranked by relevance.
- When you need full context on a specific symbol — callers, callees, which execution flows it participates in — use `context({name: "symbolName"})`.
- For security review, `explain({target: "fileOrSymbol"})` lists taint findings (source→sink flows; needs `analyze --pdg`).

## Never Do

- NEVER edit a function, class, or method without first running `impact` on it.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis.
- NEVER rename symbols with find-and-replace — use `rename` which understands the call graph.
- NEVER commit changes without running `detect_changes()` to check affected scope.

## Resources

| Resource | Use for |
|----------|---------|
| `gitnexus://repo/envctl/context` | Codebase overview, check index freshness |
| `gitnexus://repo/envctl/clusters` | All functional areas |
| `gitnexus://repo/envctl/processes` | All execution flows |
| `gitnexus://repo/envctl/process/{name}` | Step-by-step execution trace |

## Cross-Repo Groups

This repository is listed under GitNexus **group(s): envctl-migration** (see `~/.gitnexus/groups/`). For cross-repo analysis, use MCP tools `impact`, `query`, and `context` with `repo` set to `@<groupName>` or `@<groupName>/<memberPath>` (paths match keys in that group’s `group.yaml`). Use `group_list` / `group_sync` for membership and sync. From the project root: `node .gitnexus/run.cjs group list`, `node .gitnexus/run.cjs group sync <name>`, `node .gitnexus/run.cjs group impact <name> --target <symbol> --repo <group-path>` (the `.gitnexus/run.cjs` path is repo-root-relative).

## CLI

| Task | Read this skill file |
|------|---------------------|
| Understand architecture / "How does X work?" | `.claude/skills/gitnexus/gitnexus-exploring/SKILL.md` |
| Blast radius / "What breaks if I change X?" | `.claude/skills/gitnexus/gitnexus-impact-analysis/SKILL.md` |
| Trace bugs / "Why is X failing?" | `.claude/skills/gitnexus/gitnexus-debugging/SKILL.md` |
| Rename / extract / split / refactor | `.claude/skills/gitnexus/gitnexus-refactoring/SKILL.md` |
| Tools, resources, schema reference | `.claude/skills/gitnexus/gitnexus-guide/SKILL.md` |
| Index, status, clean, wiki CLI commands | `.claude/skills/gitnexus/gitnexus-cli/SKILL.md` |

<!-- gitnexus:end -->
