# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`envctl` is a **pure-Rust Cargo workspace** (8 crates) that declaratively manages a
dual-RTX-5090 Ubuntu workstation. Two halves share one engine:

- **env-manager** — `engine` + `cli` (`envctl`) + `gui` (`envctl-gui`). Brings the box to
  a declared state via TOML *components* whose lifecycle hooks wrap the proven bash in
  `assets/scripts/`. Verbs: `auto-detect`, `install`, `auto-fix`, `reset`, `add-repo`,
  `graph`, `lock`, `doctor` (see `README.md`).
- **secrets stack** — `secrets-engine` (pure-Rust crypto vault), `secrets-proto` (tonic/prost
  gRPC), `secretd` (async tokio daemon), `secretctl` (client), `secrets-store-libsql`
  (libSQL **remote** backend). Design corpus in `docs/secrets/`.

## Wired into the meta workspace (envctl is meta's env manager, not an island)

envctl is a first-class member of the `meta` workspace, reachable across every meta surface —
`meta exec`/`git`/`worktree`/`project` (via its `.meta.yaml` entry, `tags:[tools,env]`) — and now
the **plugin** surface:

- **`meta env <verb>` dispatches into envctl.** envctl ships the **`meta-env`** subprocess plugin
  (`crates/cli/src/bin/meta-env.rs`, native `meta_plugin_protocol::run_plugin`): `meta env doctor`,
  `meta env install`, `meta env auto-detect --json`, etc. It returns an `ExecutionPlan` that runs
  the `envctl` binary, so envctl's own rendering + fail-closed/dry-run-by-default semantics are
  reused verbatim. Distinct namespace from `meta dashboard` (which `meta_dashboard_cli` shells to).
- **The engine uses `loop_lib` as its command-construction substrate.** The hook runner
  (`crates/engine/src/runner.rs`) builds its `std::process::Command` via `loop_lib::build_command`
  (meta's shared builder) while keeping its own supervision (setsid reaping, per-phase timeout,
  streaming/tee) — loop_lib is a batch fan-out runner with no equivalent for those, so they stay
  in envctl. Parity is pinned by `crates/engine/tests/runner_parity.rs`.
- **Cargo:** envctl builds as its **own** workspace (it owns the no-C `[workspace.dependencies]`
  pins — ring-only rustls, libsql-remote-only, pure-Rust crypto — the security boundary it
  enforces). Cargo cannot nest a workspace as a `members` entry, so envctl is listed in the meta
  root `Cargo.toml` `exclude` **purely as a build mechanic** (like `weave`/`meta_dashboard_cli`) —
  this is NOT exclusion from meta. The `meta_plugin_protocol`/`loop_lib` deps are **path deps into
  the meta tree**, so envctl is a meta-tree-resident crate (it builds within `meta/`, not
  standalone) — exactly its role as meta's env manager. `ci/gates/no-c.sh` Gate 1.5 + Gate 4 prove
  these meta deps stay C-free.

## Session start: work in a fresh git worktree (mandatory)

This repo lives inside the `meta` workspace. **Begin every session by creating an isolated
worktree** rather than editing the checked-out tree directly. After verifying sync
(`git fetch && git status` — confirm clean and even with `origin/master`):

```bash
meta git worktree create <task-slug>     # preferred: meta-managed, multi-repo aware
# or, single-repo: git worktree add ../envctl-<task-slug> -b <task-slug>
```

Do all work in the worktree; never start coding on a stale or dirty `master`.

## Build / test / lint

```bash
cargo build -p envctl-engine -p envctl       # engine + CLI, zero system deps
cargo run  -p envctl -- auto-detect          # read-only, safe anytime (add --json for EnvReport)
cargo run  -p envctl-gui                      # needs system dev libs — see README "Native GUI"
cargo test --workspace                        # all crates
cargo test -p envctl-secrets-engine vault     # single crate / filter by test name
cargo test -p envctl-secretd --test e2e       # one integration test file (daemon e2e)
cargo fmt --all && cargo clippy --workspace -- -D warnings   # must be clean before commit
```

Tests are inline `#[cfg(test)] mod tests` beside the code, or `crates/<crate>/tests/*.rs`
integration tests (`#[tokio::test]` for the async daemon path). MSRV 1.88, stable toolchain
(`rust-toolchain.toml`).

## CI gates — run before pushing anything that touches deps or the trust boundary

```bash
bash ci/gates/no-c.sh           # supply-chain: forbids C in the trust boundary (see below)
bash ci/gates/shape.sh          # code-shape invariants (native-roots, edge module)
bash ci/gates/enable.sh         # secretd systemd-unit enable invariant
bash ci/gates/p7.sh             # .handoff Tier-A p7-conformance: schema tags + ledger residency (ADR-0004 §3)
bash ci/gates/kdf-feature-off.sh # test-speed Argon2 floor must be off by default (TASK-0032)
bash ci/gates/agent-env.sh      # agent-env.yaml ↔ agent-env.lock no-drift (TASK-0040)
bash ci/gates/loop-state.sh     # forge-loop counter integrity: ints, cadence>=1, cycles_total monotonic & >= last_wrapup (TASK-0041)
bash ci/gates/harness-scripts.sh # Feature-Forge harness tooling safety (merge-driver + reaper + loop-state-gate invariants)
```

## NON-NEGOTIABLE invariants (a change that breaks these is a regression)

- **No C library in the trust boundary.** No SQLite/OpenSSL/aws-lc may be *linked*. The store
  uses libSQL `remote` only (`default-features = false`); crypto is pure-Rust (ring, blake3,
  chacha20poly1305, argon2). `ci/gates/no-c.sh` proves this fail-closed from the resolved
  `cargo metadata` graph — **never add a dependency that pulls one of the banned crates in.**
- **Exactly one rustls, ring-only** (not aws-lc-rs). All TLS/CA crates pin `features = ["ring"]`.
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

The `.claude/` and `.codex/` agent config (skills + MCP baseline) is **provisioned and locked
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
  Do **not** hand-maintain `.claude/skills/*` or `.claude/homunculus/instincts/*` — they're
  generated. CI enforces with `envctl agent lock --check` (read-only, zero-network, exits 1 on
  drift — `ci/gates/agent-env.sh`, TASK-0040).
- Keep the MCP baseline identical across Claude (`.mcp.json`) and Codex (`.codex/config.toml`):
  `github`, `context7`, `exa`, `memory`, `playwright`, `sequential-thinking`.

## Pointers

- `docs/ARCHITECTURE.md`, `docs/ROADMAP.md`, `docs/DESIGN-NOTES.md` — env-manager design.
- `docs/secrets/SERVER-MODE.md`, `THREAT-MODEL.md`, `DESIGN-NOTES.md` — secrets-stack design;
  feature IDs (F12/F14/F15, OI-*, CF-*) referenced in commits and gate comments live here.
- `manifest/*.toml` — declarative components; drop-ins land in `manifest/components.d/`.
- The manifest dir defaults to `./manifest` (override with `ENVCTL_MANIFEST_DIR`).
- Logging: `RUST_LOG` (e.g. `RUST_LOG=envctl_engine=debug`).

## meta mission-control dashboard (zellij layout)

The `dashboard` component (`manifest/dashboard.toml`) installs two launchers on `~/.local/bin`:
- `envctl-dashboard-pane <repo>` — called by every pane in the generated zellij
  `mission-control.kdl` layout.
- `envctl-open-claude` — run by a human inside a pane when they actually want a
  Claude session.

**Default behavior:** dashboard panes open a plain shell, not an idle Claude session.
`envctl-dashboard-pane` only starts `claude` when `ENVCTL_DASHBOARD_AUTO_CLAUDE=1`
is set. This prevents accidental background Claude sessions and auto-spawn loops.
To start Claude in a pane, run `envctl-open-claude` (which sets the opt-in env var
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
fresh `claude -p` per cycle (the `/new` effect) wrapping `env-install-loop`. To **build/install the
`hf` continuity kernel and bring `.handoff` to Tier-A** ("build hf", "sync the handoff layer",
"make .handoff tier-A", "resume handoff full-sync") use **`handoff-sync`** (Epic A; distinct from
`session-relay`, which is the per-loop checkpoint). Simple questions and
trivial edits may be answered/done directly. (A SINGLE component install → `env-toolchain-install`;
drift/lock/doctor → `env-stabilize`; conventions → `agent-env-config`.)

**Placement:** the harness is **hand-authored and git-tracked**, intentionally *outside* the
kasetto/agent-env pipeline. Agent definitions live in `.claude/agents/*.md` and the harness skills
(`feature-forge`, `rust-feature-impl`, `forge-loop`, `session-relay`, `env-install-loop`,
`auto-provision`, `handoff-sync`) live directly in `.claude/skills/` — edit those files in place and commit them. They are **not** sourced from `agent-skills/`, not in `agent-env.yaml` /
`agent-env.lock`, and not produced by `envctl agent sync`. (Note: this is a deliberate exception to the
general "`.claude/skills/*` are kasetto-generated" rule above — the kasetto-managed skills remain
`agent-env-config`, `env-stabilize`, `env-toolchain-install`.)

> **Packaged upstream (TASK-0052, owner-locked 2026-06-18):** the generic construction-crew core —
> `feature-forge` + `forge-loop` + `rust-feature-impl` + the architect/implementer/guardian/
> kernel-engineer specialists — is now ALSO a **registered, ejectable packaged harness in
> `harness_hub`** (`/harness:feature-forge`, `harness_hub/harness/skills/feature-forge/` + prefixed
> `harness/agents/feature-forge-*`, `registry.json`/`entries/feature-forge.md`; harness_hub PR #38).
> This **supersedes the "hand-authored, never packaged" stance for that core**: the hub package is the
> reusable source-of-truth (the envctl `.claude/` copies are an ejected instance that may be
> re-synced via the package's `eject.sh`). The **envctl-specific loops** (`env-install-loop`,
> `auto-provision`, `handoff-sync`) are NOT generically reusable and remain hand-authored in envctl
> only — they are deliberately out of the hub package's scope.

**Change history:**
| Date | Change | Target | Reason |
|------|--------|--------|--------|
| 2026-06-04 | Initial harness build | agents/{feature-architect,rust-implementer,invariant-guardian}; skills/{feature-forge,rust-feature-impl} | Build a feature-delivery construction crew (design/implement/verify) that upholds the non-negotiable invariants |
| 2026-06-04 | Architect uses return-value (not Write) | agents/feature-architect; skills/feature-forge | Smoke test: `Plan` type is read-only and cannot Write its plan file — orchestrator persists the returned text |
| 2026-06-04 | Add rtk-proxy + baseline-stash guidance | skills/rust-feature-impl/references/verification; skills/feature-forge | Smoke test: rtk summarizes cargo/git output (corrupts fmt/clippy diagnostics); floating `stable`=1.96 causes pre-existing workspace fmt/clippy drift to be mis-attributed to the change |
| 2026-06-04 | Add continuity layer: Ralph loop + session handoff | agents/continuity-steward; skills/{forge-loop,session-relay}; skills/feature-forge | Run Feature Forge continuously over a backlog and survive context rot / token burn — cycle-budget handoff writes a durable checkpoint, coordinates over weave, and schedules a durable-cron successor session |
| 2026-06-05 | Correct relay signal model after full smoke | skills/session-relay | Smoke test: `CronCreate{durable}` is session-only here (not persisted), and a self-identity weave message is invisible to the successor's own inbox. Authoritative resume signal = committed `HANDOFF.md` + cron prompt; weave is a cross-identity (`to:all`) observable heartbeat |
| 2026-06-05 | Add env-install-loop (whole-box provisioning loop) | skills/env-install-loop; agents/continuity-steward; skills/session-relay | First-class loop to drive the workstation to fully-installed/healthy/drift-free via envctl doctor/install/auto-fix + env-toolchain-install, reusing the loop+relay continuity. Generalized continuity-steward + session-relay to serve both the feature and env loops |
| 2026-06-05 | Add auto-provision (self-restarting fresh-context Ralph runner) | skills/auto-provision (+scripts/ralph-provision.sh); skills/env-install-loop | Fully-unattended provisioning that restarts with a fresh context each cycle (the `/new` effect) by spawning a fresh `claude -p` per iteration, wrapping env-install-loop; added install↔reset remediation rung + DONE/NEEDS-HUMAN/STOP sentinels. Safe-by-default (RALPH_APPLY opt-in for unattended apply) |
| 2026-06-05 | Add component-research/audit phase (auto-append upgrades to backlog) | skills/env-install-loop; skills/auto-provision (+scripts/ralph-provision.sh) | Generalize the manual pytorch deep-dive (shallow gate, no-CUDA-assert, verify side-effect, toolkit↔driver skew) into a loop phase: subagents deep-probe each component past detect/verify (real exercise, gate quality, version currency+advisories, cross-component skew, hook hygiene, wiring reach) and append evidence-based, owner-classified items (`harden:`/`fix:`/`upgrade:` loop-fixable; `feature:` routed to feature-forge). Two-tier DONE (Tier-1 provisioned vs Tier-2 upgrades-resolved/routed). `research=` arg + `RALPH_RESEARCH` toggle (default on) |
| 2026-06-05 | Add A2 cross-repo parallel build (default-OFF, scale auto-trigger) | skills/{feature-forge,forge-loop,session-relay}; agents/{rust-implementer,continuity-steward} | Cross-repo parallelism via the three-owner split — **meta** owns the coordinated worktree set (one independent branch per repo → no cross-repo conflict by construction) + aggregation (`meta --json git worktree exec --parallel`), **grit** owns intra-repo `file::symbol` locks only (Option X: `init/claim/release/heartbeat/gc/status/queue`, never `done`/`session`/`worktree`), the **orchestrator** owns the guardian gate (only it commits/merges/PRs, only after that repo's guardian PASSes). Auto-trigger by scale (1 repo ≤3 mod → sequential DEFAULT; 1 repo >3 mod → `Workflow.pipeline`; >1 repo → A2) with `FORGE_PARALLEL=0` escape hatch; sequential single-crew unchanged when no >1-repo trigger fires. PR-1 = minimal-coherent foundation (envctl-style gate scope + schema/2-repo continuity demo); per-repo gate contracts, grit-lifecycle inversion, full N-branch resume, dep-ordered fan-out staged to PR-2..5 |
| 2026-06-08 | Add grit-harness-parallel opt-in mode | skills/{feature-forge,forge-loop} | Adopt grit's claim→work→done AST git-lock coordination into the harness for parallel multi-repo implementations: `grit init` (idempotent), file::symbol claims, --queue for contested symbols, --with-deps for dependency-aware locks. Opt-in via USE_GRIT=1; default single-implementer path unchanged. |
| 2026-06-12 | Dashboard panes default to shell; require human opt-in for Claude | assets/scripts/envctl-dashboard-pane; assets/scripts/envctl-open-claude; manifest/dashboard.toml | Prevent auto-spawn of idle Claude sessions in every zellij mission-control pane. A human must run `envctl-open-claude` to start a session. See incident audit `CLAUDE-SESSION-AUDIT-2026-06-12.md` §10.4. |
| 2026-06-12 | Migrate harness durable state `_workspace/`→`.handoff/loop/`; add kasetto-absorption capability + handoff-sync skill + hf-aware continuity | skills/{forge-loop,feature-forge,session-relay,env-install-loop,auto-provision,handoff-sync,rust-feature-impl}; agents/{feature-architect,rust-implementer,invariant-guardian,continuity-steward}; ci/gates/no-c.sh; .gitignore | Wire the harness to the real `.handoff/loop/` continuity surface (ADR-0004), carry the no-downgrade kasetto absorption playbook (Epic C, references/kasetto-absorption.md), and make checkpoints hf-kernel-aware (Epic A). P0 safeguards: legacy `_workspace/` read-only fallback for in-flight successors; hf ledger-residency guard ($META_ROOT, no per-repo ledger.db); `hf done` terminal verb; `- [!!]` SUPERVISED auto-run refusal; no-c gate extended for mimalloc. Meta-CLI fixes: `meta project list --json` (not `list-projects --names`), `meta git worktree status <slug>`. See `.handoff/decisions/ADR-0001`. |
| 2026-06-13 | Add `handoff-kernel-engineer` agent (Epic A) + seed loop_state to schema + reconcile backlog | agents/handoff-kernel-engineer.md; skills/feature-forge (crew table + Epic-A Build routing); .handoff/loop/{loop_state.md,backlog.md} | `/verify` finding: Epic A (build hf / handoff full-sync) is cross-repo (`meta/handoff`↔envctl) with kernel invariants (ledger-residency, packets-rendered, p7) that don't fit the envctl-engine-first `rust-implementer`/guardian — dedicate an agent. Also seeded `loop_state.md` with the forge-loop counter schema (budget guard was inert) and reconciled the markdown backlog (TASK-0010 rtk-relocate DONE; TASK-0005 settings-heal in review, envctl#37). |
| 2026-06-13 | Wire continuity auto-hooks (dormant until hf) + fix broken `.kb` hook | .claude/settings.json (NEW project layer); .claude/hooks/hf-checkpoint.sh (NEW); .handoff/loop/backlog.md; (meta repo) .claude/settings.json | `/verify` finding: of "auto-inject ICM / auto-sync .handoff+.kb / update .handoff per task", only ICM was live. Wire the `.handoff` auto-checkpoint hook (Stop+PreCompact, fleet-ledger-resident via `$META_ROOT`, no-op until `hf checkpoint --auto` lands) into the envctl project settings layer; fix the broken `.kb` SessionStart hook in meta (`git kb service`→guarded background `git kb serve`; code-intel was already independent). Go-live queued under TASK-0001/0002. |
| 2026-06-13 | Eject `rust-port-merge` harness for the kasetto absorption (Epic C / TASK-0012) | .claude/skills/{rust-port,rust-port-inventory,rust-port-translate,rust-port-parity,rust-port-merge,cross-repo-reference,icm-memory,session-relay-{wrap-up,resume},cross-repo-health,harness-loop-init,harness-evolution}; .claude/agents/{rust-port-*,build-health-auditor,evolution-steward}; .handoff/loop/rust-port/{loop_state,parity-ledger,merge-ledger,reports/research}.md | Owner: "rerun the kasetto integration via `/harness:rust-port-merge`; full feature, nothing overlooked or left behind." Ejected the harness into envctl (12 skills + 10 agents; FF `continuity-steward` preserved, not clobbered) and ran it in **verify/merge mode** (X=kasetto v3.2.0, Y=envctl). Researcher confirmed **0 duplications** (agent-env SHA-256 agent-asset lock ≠ engine FNV-1a component lock) and the left-behind sweep caught **3 unrowed units** — the `src/mcps/*` additive-never-clobber MCP merge (#1 no-downgrade risk: preserve global broker/repowire/weave) + `src/prompts/*` 5 command-format transforms — that the original port ledger referenced as deps but never rowed. State namespaced under `.handoff/loop/rust-port/` (avoids the forge-loop's flat `.handoff/loop/` collision). |
| 2026-06-17 | G2 retro: encode the instincts that made the run clean (evolution-steward, 5 low-risk APPLY) | skills/feature-forge (Phase 0 verify-triggering-claim step; Phase 1.5 independent-modules routing); agents/feature-architect (stated-gap-implies-adjacent-gap principle); skills/rust-feature-impl (sync-engine→async-daemon seam idiom + verification.md §0 clippy-axis classification); LESSONS.md (new durable ledger) | G2 (PR #102, native GitHub-App token minting) ran GO→GREEN→PASS-WITH-NOTES with 0 blocking findings; the steward mined 5 generalizable lessons from behaviors that were correct-by-instinct but unencoded: (1) verify a stale cross-session claim (#116 `inject.rs=todo!()` was false at HEAD) against source before designing — no-fabricate on inputs; (2) architect should trace the full call path so a stated gap surfaces its adjacent unstated gap (`MintReq.mode` missing ⇒ `NativeSubtoken` unreachable, folded into U4); (3) the sync-engine→async-daemon block_on/spawn_blocking + reuse-frozen-client idiom; (4) guardian classifies clippy by axis (gate vs `--all-targets`) × origin (touched vs untouched) so inherited red (`gui/main.rs:1997`) is a NOTE not a blocker and never silently "fixed"; (5) Phase-1.5 counts *independent* modules — a linear U1→U6 chain stays sequential despite n>3. No gate weakened; no agent added/removed; no phase reordered. |
| 2026-06-17 | Anti-drift: backlog reconcile is now a FAIL-CLOSED wrap-up gate (+ full drift sweep) | skills/session-relay-wrap-up (new step 3b); .handoff/loop/backlog.md (reconcile + Epic F) | Owner flag: *"during forge loop we drift and forget the remaining parts; this repo is holding up the rest of the project."* A 3-axis completeness sweep found the backlog stale (Epic C #71/#90-98 done but `[ ]`; TASK-0023 done) AND missing the biggest real blocker (Phase 8 SERVER-MODE edge F2/F5/F6 — `secretd/src/edge` absent — UNTRACKED, added as Epic F), and the G2 cycle's 3 follow-ups lived ONLY in PR #102's body. **Sharpest finding: G2/PR #102 built `relay mint --mode native` but TASK-0020 froze a different consumer contract** (`secretctl mint-github → {token,expires_at_unix}` + `MintGithub` RPC) that `flexnetos_github_app` shells — so #102 does NOT unblock the App (still 404s); the frozen surface was then built as TASK-0020-COMPLETE (PR #105). Root cause: the loop never surfaced TASK-0020 when G2 was designed. Fix: wrap-up step 3b makes the backlog a *written-back* artifact — append every discovered follow-up with origin, build-to-the-frozen-contract check (grep backlog for an existing TASK pinning a frozen CLI/RPC/JSON shape before marking done), status-truth reconcile vs merged PRs, promote cross-namespace residuals. |
| 2026-06-18 | Worktree/branch reaper — keep worktrees ↔ branches ↔ origin consistent | scripts/reap-worktrees.sh (NEW); skills/session-relay-wrap-up (new step 5b); skills/session-relay-resume (new step 4b); skills/forge-loop (new "Worktree hygiene" section); .gitignore (.handoff fully tracked, ledger.db guard kept) | Owner: *"this repo was left in a mess and you told me it was clean … way too many worktrees and branches."* The forge-loop creates a fresh `meta/.worktrees/<slug>/envctl` per cycle and **never reaped them** after PR auto-merge — 46 worktrees / 85 local branches / 17 stale remotes had accumulated (code WAS merged, but *workspace hygiene* was not; "merged" ≠ "clean"). origin already self-cleans (`delete_branch_on_merge=true`) so the gap is purely local. Fix: `scripts/reap-worktrees.sh` mirrors origin's cleanup locally — dry-run by default (envctl fail-closed ethos), never `--force`, protects `master`/`develop`/current-worktree, **skips dirty worktrees** (never destroys uncommitted work), reaps a branch only when its upstream is `[gone]` (merged) or it is an ancestor of `origin/master`, prunes dangling tracking refs, and best-effort `meta git worktree prune`. Wired into the safe boundaries where merge status is settled: **resume** (start clean) and **wrap-up** (after the handoff commit) — NOT mid-cycle (a PR may still be merging). Also: `.handoff` is now git-tracked in full (sentinel/log ignores dropped; the `.handoff/**/ledger.db` ADR-0004 residency guard retained — required by `ci/gates/p7.sh §3c`). |
| 2026-06-18 | Forge-loop audit upgrades U1/U3/U4/U6 — status integrity + merge safety + drift prevention | skills/forge-loop (TICK-ON-MERGED gate in steps 4-5; "Worktree hygiene" already; auto-provision-for-unattended note); skills/session-relay-wrap-up (step 3b status-truth now MERGED-gated); skills/session-relay-resume (registers merge guard at start); skills/feature-forge (Phase 0 step 5 frozen-contract pick-time check); .gitattributes + scripts/handoff-merge-guard.sh + scripts/install-handoff-merge-driver.sh (NEW); .handoff/loop/backlog.md (legend +`- [~]`) | Forge-loop audit. **U1 (tick-on-merged):** a cycle reaches terminal Done only after `gh pr view <N>` returns `MERGED`, not on "guardian PASS + auto-merge armed" — armed-not-merged stays `- [~]` in-flight and is re-polled next session. Fixes the #125 drift (TASK-0027 ticked before merge → retired as superseded). **U3 (merge guard):** `loop_state.md`/`backlog.md` map to a `handoff-reconcile` merge driver that forces a visible conflict instead of silently concatenating (fixes the cycle-5 triplicated-header / duplicated-card hazard); registered per-clone at resume (idempotent). **U4:** feature-forge greps for a frozen consumer contract at pick-time, before designing (preventive form of the G2/TASK-0020 wrong-surface miss). **U6:** documents that truly-unattended runs use the external `auto-provision` runner (the in-session cron is session-only), so `cycle_budget>1` is real. |
| 2026-06-18 | U2/U5 — complete the kasetto integration: migrate config filenames + wire the (claimed-but-absent) drift gate (TASK-0040) | agent-env.yaml + agent-env.lock (git mv from kasetto.yaml/kasetto.lock; header refreshed); ci/gates/agent-env.sh (NEW); .github/workflows/ci.yml (+agent-env gate step); CLAUDE.md (kasetto→agent-env doc reconcile, U5) | Root cause of the missing gate: kasetto v3.2.0 was absorbed into `crates/agent-env` and the binary retired (TASK-0018/#98), and the crate renamed its config resolution `kasetto.yaml`→`agent-env.yaml` / `kasetto.lock`→`agent-env.lock` (`KASETTO_CONFIG`→`ENVCTL_AGENT_CONFIG`) — but **the actual repo files were never migrated**, so the absorbed CLI failed `config not found: agent-env.yaml` and CLAUDE.md's claimed `CI enforces with envctl agent ... --locked` had no gate (it would always fail). TASK-0040 `git mv`s the files (schema-compatible; `agent lock --check` verifies up-to-date, no regen needed), adds `ci/gates/agent-env.sh` (`envctl agent lock --check` — read-only, zero-network, exits 1 on config↔lock drift), wires it into the CI gates job, and reconciles the CLAUDE.md/config-header references. Absorption was already complete (parity 102/0/13); this closes the last *operational* loose end so the agent-env is reproducibly gated. |
| 2026-06-18 | Reaper polish: FF-sync the protected trunk branches (close the /verify finding) | scripts/reap-worktrees.sh (new step 1b); skills/forge-loop ("Worktree hygiene" note) | `/verify` found the local main checkout's `master` lagging origin by 1 (direct develop pushes FF *origin*/master via the sync-master workflow, but the **local** checkout doesn't auto-FF). The reaper now fast-forwards `master` + `develop` to origin as step 1b — **FF-only** (only when the local branch is a strict ancestor of its origin ref; ahead/diverged → skip) and **clean-only** (a dirty worktree is skipped, never FF'd over uncommitted work), FF-ing a branch in whichever worktree holds it (incl. the main checkout, run from the develop worktree). Verified both paths: clean+behind → FF `0627008..8876c87`; dirty+behind → SKIP, branch untouched. Keeps `develop ↔ master ↔ origin` consistent locally with no manual merge. |
| 2026-06-18 | Retro audit (evolution-steward): recurrence fix + 2 new lessons + irreversible-remote-delete discipline | LESSONS.md (frozen-contract recurrence 1→2; +2 rows); scripts/reap-worktrees.sh (`[gone]`≠merged CAVEAT comment); skills/session-relay-wrap-up (step 5b "local reap vs remote delete" note); .handoff/loop/{evaluation,proposed-upgrades}.md | Audited the 7 manually-appended 2026-06-18 LESSONS rows as a draft. All 6 routed edits verified present at HEAD (0 false-applied). Corrected one recurrence: the frozen-contract lesson is the **2nd** occurrence of its class (1st = 2026-06-17 wrap-up step 3b reactive form, same G2/TASK-0020 evidence, CLAUDE.md row 188, never rowed) — the textbook once→noted / 2nd→upgrade-now escalation that correctly fired (reactive sweep → proactive pick-time check). Mined 2 genuinely-new lessons this session didn't capture: (1) the squash-robust merge **oracle** is the GitHub merged-PR head-ref, NOT local `[gone]`/ancestor — `[gone]` is identical for merged vs closed-unmerged (PR #99 this session), so confirm `gh pr ... state==MERGED` before any irreversible **remote** delete (reaper CAVEAT comment added; logic unchanged); (2) irreversible **external** mutations (remote deletes, advisory dismissals) are a correct owner-authz human wall + recovery-manifest/in-tree-rationale trail, distinct from in-repo dry-run gating (wrap-up 5b note). Escalated (proposed-upgrades.md, NOT applied): no test for the merge driver (P1) or reaper invariants (P2), and reaper-invocation-cadence is loop-boundary-only by design (P3, recommend accept). No gate weakened; no agent/phase changed. |
| 2026-06-18 | P1/P2/P3 + batch wrap-up cadence — make the periodic reaper/wrap-up/retro hook-enforced, not skippable | scripts/tests/{test-merge-driver,test-reaper}.sh (NEW); ci/gates/harness-scripts.sh (NEW) + ci.yml step; .claude/hooks/hf-checkpoint.sh (boundary marker); .handoff/loop/loop_state.md (`wrap_every`/`last_wrapup_total`); skills/{forge-loop,session-relay-wrap-up,session-relay-resume} | Owner: keep 1-task-per-PR but **remove the per-task pause/summary**, run tasks back-to-back, and move the pause to a **batch boundary every N tasks** that auto-runs reaper + wrap-up + evolution-steward — plus "a post hook trigger for wrap-up to ensure properly done; it gets skipped." Closed the steward's escalations and implemented the cadence. **P1/P2:** the merge driver (cycle-5 anti-concatenation guard) and the reaper (anti-pileup guard) were destructive/merge-affecting but **untested** — added hermetic, network-free tests (driver: clean-merge→forced-conflict; reaper: reap-merged / skip-dirty / protect-trunk / FF-sync) wired into a new `harness-scripts` CI gate. **P3 + cadence:** new `wrap_every` (default 5) / `last_wrapup_total` schema; forge-loop runs cycles with no per-task narration and fires a **batch boundary** at `cycles_total - last_wrapup_total >= wrap_every` (reaper→wrap-up reconcile→steward retro), distinct from `cycle_budget` (hand-off). **Enforcement (the "ensure properly done"):** the Stop/PreCompact hook drops `.handoff/loop/WRAP-UP-OWED` when a boundary comes due (cheap file I/O, no git — the reaper does NOT run from a per-turn hook), wrap-up gained a BATCH-BOUNDARY mode that clears the marker + sets `last_wrapup_total`, and resume is **fail-closed** on the marker (runs the owed wrap-up before any new work). A skipped boundary is now impossible to lose — caught at the next resume, bounded to one inter-session gap. |
| 2026-06-18 | Epic G deep audit + Tier-1 hardening (TASK-0041/0042/0043) | `.handoff/loop/backlog.md` (Epic G plan, TASK-0041..0052, Tier-2/3 decisions LOCKED); ci/gates/loop-state.sh + scripts/tests/test-loop-state-gate.sh (NEW) + harness-scripts.sh + ci.yml; skills/session-relay-wrap-up (step 3b proposed-upgrades drain) + .handoff/loop/proposed-upgrades.md (drained); skills/feature-forge (Phase 3.5 runtime-verify); agents/{feature-architect (`## Runtime surface`),invariant-guardian (invariant #10 + Runtime verification + `## Runtime check`)}; skills/rust-feature-impl/references/verification.md (§4.5) | Owner-requested deep audit of the forge-loop harness (provenance + gaps + adoptable rust-port/harness_hub patterns). **Provenance:** hand-authored bespoke in envctl (`5dcc4b2`/`00237ca`, 2026-06-04), NOT from a "forge" repo — it is the *source pattern* harness_hub later abstracted. Planned all findings as tiered Epic G; owner LOCKED the Tier-2/3 forks (TASK-0044 pick-deps via the **hf kernel**, gated on Epic A; TASK-0048 A2 **all-green barrier**, not OS-matrix; TASK-0052 **full eject/package** into harness_hub — overrides "hand-authored outside pipeline" doctrine, CLAUDE.md reconcile owed). **Tier 1 shipped:** (0041) `loop-state.sh` counter-integrity gate (ints/cadence≥1/`cycles_total`≥`last_wrapup`/monotonic) + hermetic test in the harness-scripts gate; (0042) wrap-up step 3b now drains `proposed-upgrades.md` fail-closed to tracked `- [?]` items (drained the stale 49-line file: P1/P2 already-resolved, P3 declined); (0043, P0) **Phase 3.5 runtime-verify** — the guardian now drives an architect-declared observable surface with the `verify` skill (run the app, capture evidence) before a clean PASS, closing the "compiles + gates green but doesn't work at runtime" gap (TASK-0028 GUI screen marked done with no `secretctl` call / no GUI launch). No gate weakened; each change is additive/strictly-stronger. |
| 2026-06-18 | Epic G Tier-2/3 complete: TASK-0044 (hf-kernel cards) + TASK-0052 (harness_hub package) | TASK-0044: `.handoff/tasks/TASK-*.task.json` (53 fleet-scoped cards via handoff-kernel-engineer) + `skills/forge-loop` pick-path correction; TASK-0052: **harness_hub** `harness/skills/feature-forge/` + `forge-loop`/`rust-feature-impl` + `harness/agents/feature-forge-{architect,implementer,guardian,kernel-engineer}.md` + `registry.json`/`entries/feature-forge.md` + plugin 1.11.0 (PR #38); this CLAUDE.md Placement reconcile | Closed Epic G (11/12 → all picked up). **0044:** minted envctl's backlog into the shared fleet ledger as per-member `handoff.task.v1` cards (no contamination — handoff 55/prompt_hub 71/fleet 840 unchanged; HFTASK-0026-done routing); the dependency-authority substrate now exists. Found kernel **HFTASK-0054** (shipped hf is CWD-relative for the ledger, no `--ledger`/`--member` override → live picker `hf resume`/`claim --next` can't serve a member's cards without contamination/forbidden per-repo ledger), so the safe read-only authority TODAY is `hf fleet render envctl`; forge-loop SKILL corrected, markdown fallback retained. **0052 (owner-locked full eject/package):** packaged the generic construction-crew core into harness_hub as `/harness:feature-forge` (prefixed specialists, reuses shared evolution/continuity/integration-qa; ejectable; `hub-validate` PASS 8 entries) — supersedes the never-packaged stance for that core; envctl-specific loops (env-install-loop/auto-provision/handoff-sync) stay envctl-only. |
