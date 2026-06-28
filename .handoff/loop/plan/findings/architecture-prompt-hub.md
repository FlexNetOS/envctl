# Findings — Dimension: architecture / correctness / store→rusty-idd convergence (TARGET: prompt_hub)

Target: `/home/drdave/Desktop/meta/prompt_hub` — Rust workspace, 3 members (`prompt-hub` core lib,
`prompthub` CLI, `prompthub-server` axum API). Read-only analysis.

Convergence lens verdict (up front): **The ADR-0007 "prompt_hub emits a provenance-stamped GOAL
ARTIFACT that rusty-idd consumes" contract is ASPIRATIONAL / DOC-ONLY.** No `GoalArtifact` type, no
emission code, no rusty-idd-addressed serialization, no schema version exists in the codebase. Every
`rusty-idd` / goal-artifact reference lives in `docs/` and `prompts/` markdown, not in `.rs`. The
nearest real primitives (generic prompt `export`, the SHA-256 audit hash chain, the `Intent` struct)
exist but are unconnected to any goal-artifact contract.

---

## CLAIMS

- CLAIM: There is no goal-artifact type, struct, or emission function anywhere in the codebase; the
  string `goal` in `junie.rs` is a word inside a system-prompt literal and in `hub.rs` refers to the
  unrelated `Artifact` (produced output) type. | evidence: `prompt-hub/src/junie.rs:33` ("Your goal
  is to coordinate tasks…"); `grep -rni "GoalArtifact|goal_artifact|goal-artifact" --include=*.rs`
  over the workspace returns zero hits; `prompt-hub/src/hub.rs:1857-1873` `Artifact` is codegen
  output, not an intent/goal | confidence: high
- CLAIM: Every `rusty-idd` and `ADR-0007 goal-artifact` reference is documentation, not code — they
  appear only in `docs/plans/lifeos-meta-front-door.md`, `prompts/fleet-convergence-first-run.md`,
  `prompts/plan-loop-parallel-run.md`. | evidence: `grep -rni "rusty-idd|adr-0007 goal"` hits only
  `.md` under `docs/` and `prompts/`; `docs/plans/lifeos-meta-front-door.md:62` draws the intended
  flow `prompt_hub ──intent/prompt──▶ rusty-idd ──ready goal/spec──▶ planning_engineer` as a *plan*,
  not an implemented interface | confidence: high
- CLAIM: ADR-0007 in this repo is "Plugin System", unrelated to the convergence goal-artifact
  contract — the ADR number referenced by the convergence plan collides with an existing local ADR.
  | evidence: `docs/adr/0007-plugin-system.md:1` `# ADR-0007: Plugin System` | confidence: high
- CLAIM: The CLI `export` command produces a generic full-DB dump of `Prompt` records (JSONL/YAML/
  Markdown) with no provenance stamp, no schema/format version, no audit-hash, and no rusty-idd
  framing — it is `serde_json::to_string(prompt)` per row. | evidence:
  `prompthub/src/commands/export.rs:42-52` (`export_jsonl` serializes raw `Prompt`),
  `export.rs:22` (`hub.list(...)` of up to 10 000 prompts) — no goal/intent selection, no signature
  | confidence: high
- CLAIM: The serialized `Prompt` carries author identity but no integrity signature; `AgentIdentity`
  has a `token_hash` field but the prompt body itself is unsigned, so an exported prompt is not
  tamper-evident. | evidence: `prompt-hub/src/models.rs:388-408` (`Prompt` fields), `models.rs`
  `AgentIdentity { id, name, capabilities, token_hash, specialization_score }` — no per-record hash |
  confidence: high
- CLAIM: The only tamper-evident provenance primitive that exists is the audit hash chain
  (`SHA256(before_json || after_json || timestamp)`), and it is confined to the audit log — it is
  NOT included in `export` output nor exposed as a goal-artifact envelope. | evidence:
  `prompt-hub/src/audit.rs:46-74` (`compute_diff_hash`, "tamper-evident SHA-256 hash chain");
  `export.rs` never references `audit` | confidence: high
- CLAIM: Prompt lineage/provenance is non-functional in production paths. `LineageTracker` is an
  in-memory `HashMap` field on `PromptHub`, is never loaded from or written to storage, and
  `register_version` is called only from unit tests. | evidence: `prompt-hub/src/lineage.rs:13-18`
  (in-mem `nodes`/`roots`); `grep register_version` → only `prompt-hub/src/hub.rs:3906,3913` (both in
  `#[cfg(test)]`); `prompt-hub/src/storage.rs` has zero `LineageTracker`/`lineage` references |
  confidence: high
- CLAIM: The CLI `lineage` subcommand is a stub that prints a deferral message instead of computing
  lineage. | evidence: `prompthub/src/main.rs:169-172` prints
  `"(Lineage tracking requires version history — use 'audit' for now)"` | confidence: high
- CLAIM: `LineageNode.created_at` is a hardcoded placeholder string `"now"`, so any lineage that *is*
  built carries no real timestamp — a correctness defect in the provenance/ancestry data. | evidence:
  `prompt-hub/src/lineage.rs:27` (`pub created_at: String`), `lineage.rs:86` (`created_at:
  "now".to_string()`) | confidence: high
- CLAIM: The libsql "connection pool" is a single shared `Connection` gated by a `Semaphore`; it is
  not a real pool — every `acquire()` hands out a clone of the same connection handle, so the
  `max_connections` (defaulting to `available_parallelism()`) does not buy parallel DB I/O and all
  writes serialize through one handle. | evidence: `prompt-hub/src/storage.rs:40-58` (`Storage { conn:
  Connection, semaphore }` + the doc comment "Reusing one connection — rather than opening a fresh
  one per acquire"), `storage.rs:29-33` (`max_connections` from `available_parallelism`) | confidence:
  high
- CLAIM: The database path is hardcoded to the relative literal `"prompthub.db"` in every CLI command
  except `init`; `HubConfig::load()` is loaded but its (potential) DB path is ignored, so
  `init --path /elsewhere.db` then `add` operate on different databases (CWD-dependent). | evidence:
  `prompthub/src/commands/{add,budget,cache,cost,evolve,export,feedback,gather,import,list,metrics,
  rollback,search,vibe}.rs` all `PromptHub::new(Path::new("prompthub.db"), config)`;
  `prompthub/src/commands/init.rs:10-11` is the only one honoring `--path` | confidence: high
- CLAIM: `PromptHub` is a God-object — 186 `pub fn`/`pub async fn`, ~30+ fields (most feature-gated),
  4 748 lines — making it the central coupling hotspot through which CLI, server, and lib all flow.
  | evidence: `prompt-hub/src/hub.rs` `wc -l` = 4748; `grep -c "pub fn|pub async fn"` = 186; struct
  `PromptHub` field list `hub.rs` (storage, search_engine, sanitizer, auth, lock_manager, metrics,
  sync, shutdown_coordinator, hooks, junie, quality_gate, lineage, swarm_registry, pollination,
  satisfaction_tracker, health_monitor, load_balancer, + ~15 `#[cfg(feature)]` fields) | confidence:
  high
- CLAIM: The server's `routes.rs` (193.9 KB) and generated `openapi.rs` (66.6 KB) are God-files; the
  axum router wires ~40+ routes from a single module. | evidence: `prompthub-server/src/routes.rs`
  193.9 KB, `prompthub-server/src/openapi.rs` 66.6 KB; `prompthub-server/src/server.rs:36-118`
  (single `Router::new()` chain of ~40 `.route(...)`) | confidence: high
- CLAIM: The core intent→prompt path (`PromptHub::get`) is "simplified" relevance search that returns
  the single top-ranked stored prompt for a role+intent — it does not synthesize, validate, or
  provenance-stamp a goal; it is a retrieval, not a goal-artifact producer. | evidence:
  `prompt-hub/src/hub.rs:981-999` (comment `// Simplified: use search engine…`; returns
  `results.items.into_iter().next().map(|sp| sp.prompt)`) | confidence: high
- CLAIM: The `Intent` struct (the natural candidate payload for a goal artifact) is a transient
  in-memory classification (`raw_text`, `domain`, `role`, `task_type`, `complexity`, `urgency`,
  `extracted_entities`) with no id, no timestamp, no author, and no persistence — it cannot serve as
  a durable, addressable goal artifact as-is. | evidence: `prompt-hub/src/models.rs:558-566` |
  confidence: high

## GAPS (named, vs the convergence baseline the plan asserts)

1. **No goal-artifact schema.** Nothing defines what a rusty-idd-consumable goal artifact *is* —
   fields, versioning, envelope. The contract is drawn in `docs/plans/lifeos-meta-front-door.md`
   (lines 62, 81, 91-92, 123, 147) but never typed in Rust. The store→rusty-idd pipeline has no
   wire format.
2. **No emission surface.** No CLI subcommand, no server route, and no library API emits an artifact
   *addressed to* rusty-idd. `export` is a generic backup dump; the server exposes prompt CRUD +
   search, not a goal hand-off endpoint (`prompthub-server/src/server.rs:36-118`).
3. **Provenance is fragmented.** Three half-primitives exist but none compose into the
   provenance-stamp the contract names: (a) the audit SHA-256 chain (`audit.rs`) is log-only; (b)
   lineage (`lineage.rs`) is in-memory, test-only, with a fake timestamp; (c) `Prompt.author` is
   unsigned. There is no single "provenance-stamped" object.
4. **Lineage is effectively dead code** in production: never persisted, never loaded, CLI stubbed.
   This directly undercuts any "lineage/provenance" claim in the convergence narrative.
5. **No rusty-idd consumer contract / version negotiation.** Even if prompt_hub emitted something,
   nothing pins a schema version or compatibility guarantee, so rusty-idd cannot safely consume it.
6. **Store identity ambiguity** (hardcoded `prompthub.db`) means the "durable intent store" has no
   single canonical location — a fail-open governance gap for a control-plane store.
7. **God-object coupling** (`hub.rs` 186 pub fns; `routes.rs` 193 KB) means any goal-artifact feature
   added today inherits maximal blast radius — every consumer touches `PromptHub`.

## UPGRADE ROWS

- UPGRADE: Define a versioned `GoalArtifact` envelope type (`schema_version`, `id`, `created_at`,
  `source: "prompt_hub"`, `intent: Intent`, `selected_prompt: PromptRef`, `provenance: {audit_hash,
  author, lineage_path}`) in a new `prompt-hub/src/goal_artifact.rs`, behind a `goal-artifact`
  feature. | axis: accuracy | target-surface: `prompt-hub/src/goal_artifact.rs` (new module) + `lib.rs`
  export | rationale: turns the doc-only ADR-0007 contract into a typed, falsifiable wire format that
  rusty-idd can consume; composes the three existing provenance fragments into one stamped object |
  evidence: `models.rs:558` (`Intent` exists, reusable), `audit.rs:69` (`compute_diff_hash`
  reusable), `docs/plans/lifeos-meta-front-door.md:62,81` (the required flow) | blast: low — new
  module, additive; no existing symbol changes (graph: zero current callers) | effort: M | risk-tier:
  PROPOSE | acceptance: `serde_json` round-trip of a `GoalArtifact` preserves all fields AND
  `schema_version` is asserted non-empty; a golden fixture validates the envelope shape | reversibility:
  Integrity: adds a signed envelope (raises integrity) · Reversibility: feature-gated, deletable ·
  Capability-Gain: makes the store→rusty-idd pipeline real
- UPGRADE: Add a `prompthub emit-goal --role --intent --out <file>` CLI subcommand + a
  `POST /api/v1/goal/emit` server route that builds a `GoalArtifact` from the `get` selection path and
  stamps it with the current audit hash. | axis: accuracy | target-surface:
  `prompthub/src/commands/emit_goal.rs` (new) + `prompthub-server/src/routes.rs` (new route) +
  `server.rs:36-118` router wiring | rationale: provides the missing emission surface so the pipeline
  is end-to-end exercisable, not just a type | evidence: `hub.rs:981-999` (`get` is the existing
  selection path to wrap), `server.rs:41` (`/api/v1/prompts/get` shows the route pattern) | blast:
  medium — touches the 193 KB `routes.rs` God-file and the router chain | effort: M | risk-tier:
  PROPOSE | acceptance: a RED test invokes `emit-goal` and asserts the output file deserializes to a
  `GoalArtifact` with a non-empty `provenance.audit_hash` matching the audit log entry | reversibility:
  Integrity: emission is read-only on the store · Reversibility: additive command/route ·
  Capability-Gain: the actual hand-off to rusty-idd
- UPGRADE: Persist lineage — write `LineageNode` rows to a `lineage` table on every version create and
  rebuild `LineageTracker` from storage in `PromptHub::new`; replace `created_at: "now"` with the real
  `DateTime<Utc>`. | axis: correctness (quality) | target-surface: `prompt-hub/src/lineage.rs`,
  `prompt-hub/src/storage.rs` (new table + load), `prompt-hub/migrations/` (new migration) | rationale:
  converts dead/in-memory lineage into the durable provenance the contract requires; fixes the fake
  timestamp defect | evidence: `lineage.rs:86` (`"now"` placeholder), `main.rs:169-172` (stubbed CLI),
  `storage.rs` (no lineage refs), `hub.rs:3906,3913` (only test callers) | blast: medium — adds a
  migration (REGENERATE-adjacent) and touches `hub.rs` construction | effort: L | risk-tier: PROPOSE
  | acceptance: a RED test registers two versions across two `PromptHub` instances over the same DB
  and asserts the ancestry path survives reconstruction with real (non-"now") timestamps | reversibility:
  Integrity: durable lineage raises integrity · Reversibility: migration is forward-additive, table
  droppable · Capability-Gain: real provenance lineage
- UPGRADE: Replace the hardcoded `Path::new("prompthub.db")` in every CLI command with a resolved
  store path from `HubConfig` / `$PROMPTHUB_DB` / `--db` global flag, defaulting to an XDG data dir.
  | axis: governance+settings+config | target-surface: `prompthub/src/cli.rs` (global `--db` arg) +
  the 14 `prompthub/src/commands/*.rs` call sites + `HubConfig` | rationale: a single canonical store
  location is a precondition for the "durable intent store + boundary" role; current behavior silently
  forks the store by CWD | evidence: `commands/add.rs:11` (+ 13 sibling files all hardcode
  `"prompthub.db"`), `commands/init.rs:10-11` (only path-aware command) | blast: medium — 14 call
  sites, all leaf CLI handlers (graph: no further dependents) | effort: M | risk-tier: APPLY |
  acceptance: a RED test sets `--db /tmp/x.db`, runs `init` then `add` then `list`, and asserts the
  added prompt is listed (proving both commands hit the same store) | reversibility: Integrity: removes
  a fail-open store-fork hazard · Reversibility: flag is additive, default preserves current path ·
  Capability-Gain: configurable canonical store
- UPGRADE: Include the audit hash chain head in `export` output (per-record `provenance` block) so the
  generic dump becomes verifiable. | axis: accuracy | target-surface:
  `prompthub/src/commands/export.rs` | rationale: cheap step toward provenance-stamped output reusing
  the existing audit primitive; closes the "export is unsigned" gap | evidence: `export.rs:42-52`
  (raw serialize), `audit.rs:46-74` (hash chain available) | blast: low — single command, leaf |
  effort: S | risk-tier: APPLY | acceptance: a RED test exports a prompt with audit history and
  asserts the JSONL line contains a non-empty `provenance.audit_hash` field | reversibility: Integrity:
  adds verifiability · Reversibility: additive field · Capability-Gain: tamper-evident export
- UPGRADE: Decompose `PromptHub` God-object by extracting cohesive sub-facades (e.g. a `provenance`
  facade owning audit+lineage+goal-artifact) to bound the blast radius of the convergence work. |
  axis: quality | target-surface: `prompt-hub/src/hub.rs` (facade extraction) | rationale: 186 pub
  fns + 30 fields make every new feature maximal-blast; a provenance facade localizes the
  goal-artifact additions | evidence: `hub.rs` 4748 LOC / 186 pub fns; struct field list `hub.rs`
  | blast: high — `PromptHub` is the central type used by CLI + server (graph: top centrality) |
  effort: L | risk-tier: PROPOSE | acceptance: a RED test asserts the new `hub.provenance()` facade
  exposes audit + lineage + goal-artifact ops and the pre-existing public `get`/`list`/`audit`
  signatures are unchanged (no consumer breakage) | reversibility: Integrity: pure refactor, behavior
  preserved · Reversibility: internal, re-inlinable · Capability-Gain: bounded blast for future
  convergence work
- UPGRADE: Document the libsql "pool" honestly and either make it a real multi-connection pool or
  rename to `SharedConnection` + drop the misleading `max_connections`/semaphore-as-pool framing.
  | axis: speed | target-surface: `prompt-hub/src/storage.rs` | rationale: current code advertises a
  connection pool but funnels all I/O through one handle; either fix (real pool for read concurrency
  on file DBs) or stop implying parallelism that doesn't exist | evidence: `storage.rs:40-58` (single
  `conn` reused, semaphore-gated), `storage.rs:29-33` (`max_connections` from `available_parallelism`)
  | blast: medium — `Storage` is used throughout the lib via `acquire()` | effort: M | risk-tier:
  PROPOSE | acceptance: a RED benchmark/test asserts the documented concurrency behavior matches the
  implementation (either N concurrent file connections OR a renamed single-conn type with no pool
  claim) | reversibility: Integrity: removes a misleading contract · Reversibility: framing change ·
  Capability-Gain: honest concurrency model (or real read parallelism)

## OPEN QUESTIONS (for the verifier / architect)

- The convergence ADR-0007 referenced by the plan is NOT the local `docs/adr/0007-plugin-system.md`.
  Where is the authoritative goal-artifact ADR — in `rusty-idd` or at `META_ROOT`? The number
  collision should be resolved before any "implements ADR-0007" claim is made. (cross-dimension hook
  → governance-config dimension)
- Does rusty-idd already define a goal/spec input schema the `GoalArtifact` envelope must match? If
  so, the envelope above must be derived from rusty-idd's consumer contract, not invented here.
  (cross-dimension hook → cross-repo contract; needs the rusty-idd reports under
  `rusty-idd/.handoff/loop/plan/`)
- Whether `:memory:` single-connection reuse (the documented reason for the shared-conn design,
  `storage.rs:47-55`) blocks a real pool for file DBs, or whether a per-mode strategy (shared for
  `:memory:`, pooled for files) is feasible — affects the speed upgrade's design.
