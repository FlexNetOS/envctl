# Codemap — prompt-hub (cycle 6)

- **Target:** `prompt-hub` (slug) — repo `/home/drdave/Desktop/meta/prompt_hub`
- **Snapshot:** git `f826ea33` on branch `plan/fleet-arch-integration-cycle1`; mapped 2026-06-27
- **Source of truth:** `git-kb code` 0.2.10 (AST/call-graph) + Cargo manifests + axum route table
- **Scale:** ~51,503 LOC across 3 src dirs; 3,589 indexed member symbols; 1,405 public src symbols
- **Edition:** 2024, rust-version 1.91.1; workspace resolver 3

> **Frame note (read first):** This repo is positioned as the *Front-Door intent STORE*
> that (per ADR-0007 / `docs/plans/lifeos-meta-front-door.md`) emits provenance-stamped
> **goal artifacts** to rusty-idd. **That emission seam does NOT exist in the Rust code.**
> The strings `rusty-idd` / `goal-artifact` / `provenance` appear in **`docs/plans/lifeos-meta-front-door.md` only**
> — zero hits in `prompt-hub/src`, `prompthub/src`, `prompthub-server/src`. The "goal" mentions
> in `hub.rs`/`junie.rs` flagged in the invocation are doc-comment / system-prompt **text**, not code.
> The actual front door is **`PromptHub::process_input` (`prompt-hub/src/hub.rs:1401`)** — a multimodal
> `UserInput → Intent` normalizer feeding the internal vibe path. See "Goal-artifact emission" below.
> This is a CLAIM for the verifier: *the rusty-idd seam is planned (doc), not built (code).*

---

## 1. Members & top-level modules

| Member | Crate | Role | src files | symbols |
|---|---|---|---|---|
| `prompt-hub/` | `prompt-hub` | **core library** — the intent/prompt STORE + engines | 70 | 3,095 |
| `prompthub/` | `prompthub` | **CLI** binary (clap) | 25 | 112 |
| `prompthub-server/` | `prompthub-server` | **HTTP API** (axum 0.8) | 7 | 382 |

Dependency direction (Cargo path deps, verified in manifests):
`prompthub (CLI) → prompt-hub` and `prompthub-server → prompt-hub`. **Neither front-end
depends on the other.** Strict 2-layer fan-in onto the core. Features in both front-ends
are pure re-exports of `prompt-hub/*` feature flags (`prompthub-server/Cargo.toml:19-48`,
`prompthub/Cargo.toml:19-60`).

### Core library modules (70 files, `prompt-hub/src/`)
The core is a **wide, flat module set** (`lib.rs:23-122` = 70 `pub mod`s). Grouped by concern:

- **Store / data model:** `hub.rs` (268 syms — the `PromptHub` facade), `models.rs` (155 — `Prompt`, `Intent`, `UserInput`, `Artifact`, `AgentIdentity`), `storage.rs` (72 — libsql-backed `Storage`), `config.rs`, `defaults.rs`, `error.rs`.
- **Search / retrieval:** `search.rs` (145 — `SearchEngine` trait + hybrid index), `qdrant.rs` (63), `tokens.rs`, `summarizer.rs`.
- **Intent / "vibe" front-door path:** `vibe.rs` (64 — `VibeEngine`, `IntentClassifier`, `SkillRecommender`, `SelfHealer`, `PromptGenerator`), `multimodal_input.rs`, `multimodal.rs`, `gather.rs` / `context_gatherer.rs`, `junie.rs` (agent identity).
- **Governance / safety:** `auth.rs` (RBAC: `AuthManager`/`RbacAuthManager`, `Capability`, `Action`), `audit.rs` (`AuditLogger`/`SqliteAuditLogger`, SOC2), `sanitize.rs`, `moderation.rs`, `malware_scan.rs`, `privacy.rs`, `quality_gate.rs` (`Linter`/`SecurityScanner`/`PerformanceChecker`/`AccessibilityChecker`), `sandbox.rs`.
- **Lifecycle / lineage:** `lineage.rs`, `diff.rs`, `rollback.rs`, `evolution.rs`, `gradual_rollout.rs`, `garbage_collector.rs`, `auto_purge.rs`, `retention.rs`, `lock.rs`.
- **Providers / cost / resilience:** `multi_provider.rs`, `provider_health.rs`, `load_balancer.rs`, `fallback.rs` (`FallbackChain`), `circuit_breaker.rs`, `cost.rs`/`cost_limits.rs`/`budget.rs`/`quota.rs`, `confidence.rs`.
- **Ops / telemetry / sync:** `metrics.rs`, `analytics.rs`, `health.rs`, `shutdown.rs`, `sync.rs`, `offline.rs`, `hooks.rs`, `plugins.rs` (inventory-based plugin registry), `swarm.rs`, `pollination.rs`.
- **Inputs / I18n / UX:** `voice.rs`/`voice_anonymize.rs`, `touch.rs`, `mobile.rs`, `accessibility.rs`, `i18n.rs`, `preview.rs`, `templates.rs`, `satisfaction.rs`, `beta_program.rs`, `learn.rs`, `chaos.rs`/`chaos_auto.rs`, `local_llm/` (engine + inference).

### CLI modules (`prompthub/src/`)
`main.rs:30` (entry) → `cli.rs` (clap `Commands` enum) → `commands/` (21 handlers) + `tui.rs`, `fuzzy.rs`, `identity.rs`.

### Server modules (`prompthub-server/src/`)
`main.rs:46` (entry) → `server.rs:36` (`Router::new()` — 111 routes) → `routes.rs` (handlers) + `state.rs` (`AppState`), `middleware.rs`, `responses.rs` (`ErrorResponse`/`success`/`error`), `openapi.rs`.

---

## 2. Entry points

| Kind | Symbol | Location |
|---|---|---|
| CLI binary `main` | `prompthub::main` | `prompthub/src/main.rs:30` |
| HTTP server `main` | `prompthub-server::main` | `prompthub-server/src/main.rs:46` |
| Library root | `prompt-hub` crate | `prompt-hub/src/lib.rs` (70 `pub mod`, re-exports `PromptHub`, `HubError`, `models::*`) |
| Library facade | `PromptHub` | `prompt-hub/src/hub.rs` (struct + 183 pub methods) |
| Build scripts | `build.rs` | `prompt-hub/build.rs:1`, `prompthub-server/build.rs:5` |

`git-kb code flows --refresh` returned **no traced flows** at this index scope (recorded in callgraph.json).

### CLI verbs (`prompthub/src/cli.rs`, clap `Commands`)
`init`, `add`, `get`, `list`, `search`, `update`, `rollback`, `diff`, `lock`, `unlock`, `audit`,
`export`, `import`, `lineage`, `completions`, `tui`, `metrics`, `server`, `cache`, `restore`,
`evolve`, `tokens`, `lint`, `plugin`, `vibe`, `magic`, `gather`, `preview`, `cost`, `scan`,
`deploy`, `summarize`, `feedback`, `junie`, `budget`, `voice`, `onboard`, `heal`, `suggest`,
`quota`, `chat`, … (search-mode enum: `Fast`/`Smart`/`Hybrid`). Subcommand groups: `cache`
(`Clear`/`Status`/`Evict`), `plugin` (`List`/`Install`/`Uninstall`/`Enable`/`Disable`),
`budget`/`quota` (`Set`/`Check`/`History`/`Alerts`).

### Server routes — 111 total (`prompthub-server/src/server.rs:36`)
All under `/api/v1/*` plus ops endpoints. The verb→handler→core mapping is the external
contract surface. Counts by domain:

| Domain | # routes | Sample |
|---|---|---|
| prompts CRUD + lifecycle | 17 | `POST /api/v1/prompts`, `PATCH /api/v1/prompts/{id}`, `POST .../evolve`, `.../rollback`, `.../render`, `.../lock`, `.../transfer`, `.../audit` |
| lineage | 7 | `GET /api/v1/lineage/ancestry/{version_id}`, `.../descendants`, `.../tree`, `.../forks`, `.../roots` |
| audit / SOC2 | 5 | `POST /api/v1/audit/verify`, `.../hash`, `.../soc2/summary`, `.../soc2/validate`, `.../anonymize` |
| providers / multi-provider / lb | 14 | `POST /api/v1/providers/register`, `/api/v1/multi-provider/route`, `/api/v1/lb/select` |
| budget / cost / cost-limits / quota | 14 | `PUT /api/v1/budget/budget`, `POST /api/v1/cost/estimate`, `/api/v1/quota/consume` |
| rollouts / rollback / deploy | 8 | `POST /api/v1/rollouts/advance`, `.../evaluate-rollback`, `/api/v1/deploy` |
| gc / auto-purge / retention | 14 | `POST /api/v1/gc/run`, `/api/v1/auto-purge/daemon/start`, `/api/v1/retention/cleanup` |
| safety: moderation / privacy / confidence / fallback / learn | 8 | `POST /api/v1/moderation/check`, `/api/v1/privacy/scan`, `/api/v1/confidence`, `/api/v1/fallback` |
| context / diff / template / vibe / input | 11 | `POST /api/v1/context/gather/smart`, `/api/v1/diff/compute`, `/api/v1/vibe/code`, **`POST /api/v1/input/process`** (the front-door normalizer), `/api/v1/swarm/bundle` |
| satisfaction / beta | 8 | `POST /api/v1/satisfaction/csat`, `/api/v1/beta/cohorts` |
| ops | 6 | `GET /health`, `/ready`, `/live`, `/metrics`, `/openapi.json`, `/docs` |

Full method+path+handler table: `graph/prompt-hub.callgraph.json` is edge-level; the route
list is in `scratchpad/routes_clean.json` and rendered in `graph/prompt-hub.graph.md`.

---

## 3. Goal-artifact emission surface (the ADR-0007 seam) — **NOT FOUND IN CODE**

The frame asks specifically to map the goal-artifact emission path + the rusty-idd seam.
Result of the mapping (cite-checked):

- **No rusty-idd seam exists in source.** `grep -riE 'rusty.?idd|goal.?artifact|provenance'` over all three members' `src` = **0 hits**; only `docs/plans/lifeos-meta-front-door.md` mentions them.
- **Closest code analogs (what actually exists):**
  - `PromptHub::process_input` (`prompt-hub/src/hub.rs:1401`) — "front-door normalizer that turns any input modality into the `Intent` consumed by the vibe/orchestration path" (its own doc). Delegates to `MultiModalInput::process` (`multimodal_input.rs:25`). Exposed as `POST /api/v1/input/process`.
  - `Intent` (`prompt-hub/src/models.rs:558`) and `UserInput` (`:599`) — the front-door data types.
  - `Artifact` enum (`prompt-hub/src/models.rs:654`) — variants `Prompt`/`Code`/`Config`/`Test`/`Migration`/`Documentation`. This is the **swarm/agent execution output** model, **not** a provenance-stamped goal artifact bound to rusty-idd. No `provenance` field.
  - `generate_bundle` (`prompthub-server/src/routes.rs:879`, `GET /api/v1/swarm/bundle`) — emits a swarm bundle; closest "emit to a downstream consumer" endpoint, but not rusty-idd-aware.
  - `Junie` (`prompt-hub/src/junie.rs`) — an agent identity / `system_prompt` provider; no emission logic.
- **Verdict for downstream agents:** the "two-layer front door (harness_hub interpreter + prompt_hub store → rusty-idd)" is at the **store + intent-normalization** layer in code; the **goal-artifact emission + rusty-idd handshake is unimplemented** (a planning ROADMAP item, not a present capability). The analyst/architect must treat any "prompt_hub emits to rusty-idd" assumption as a **gap to build**, not a seam to extend.

---

## 4. External interfaces

- **Persistence:** **libsql** (`libsql = { default-features = false, features=["core"] }`, workspace Cargo.toml) — local-only SQLite-compatible store. `Storage` (`prompt-hub/src/storage.rs`) is the DB facade; `Storage::acquire` (`:146`, fan-in 41) and `Storage::insert_prompt` (`:283`) are the hottest DB methods. `SandboxStore` (`sandbox.rs`) is an in-memory rate/sandbox store. Workspace deliberately drops libsql's `replication`/`remote`/`sync`/`tls` to avoid 4 RUSTSEC advisories (manifest comment).
- **HTTP server:** axum 0.8.8 + tower / tower-http (cors, compression, trace, request-id) + tower_governor (rate limiting). 111 routes, OpenAPI/Swagger at `/openapi.json` + `/docs`. Prometheus at `/metrics` (`MetricsCollector::prometheus_text`, the top fan-out at 21).
- **Templating:** handlebars (default) / tera (feature) — `TemplateEngine` trait (`templates.rs:8`).
- **Tokenization / embeddings:** tiktoken-rs, tokenizers, optional ONNX (`smart-ort`), qdrant client (feature `qdrant`).
- **Auth/crypto:** argon2 (password hashing; see the careful `getrandom`/`password-hash` version-pinning note in the workspace manifest).
- **Plugin system:** `inventory` (compile-time registry) + `libloading` (dynamic) via `plugins.rs` — `Plugin` trait is a top fan-in by implementor count.

---

## 5. Build / run surface

- **Build:** Cargo workspace, 3 members, `resolver = "3"`, edition 2024, MSRV 1.91.1. Per-repo CI clippy gate (see meta CLAUDE.md). `benches/` (criterion) + `docker/` + `examples/` present (excluded from the member graph).
- **Run:** `cargo run -p prompthub -- <verb>` (CLI) or `cargo run -p prompthub-server` (HTTP API). CLI `server` verb (`cli.rs:138`) can also launch the server in-process.
- **Heavy feature matrix:** ~35+ features per front-end, all forwarding to `prompt-hub/*`. Default = `handlebars` (+ `vibe`,`budget` on the server). This wide optional-feature surface is a notable build-complexity/coupling signal for the analyst (governance + perf dimensions).

---

## 6. How to navigate fast (for analysts)
- The **core facade** is `PromptHub` (`hub.rs`); almost every CLI command and server route resolves into a `PromptHub` method. Start there.
- **Hottest intra-repo callees** (real coupling): `PromptHub::lock` (76), `Storage::acquire` (41), `HubConfig::load` (45), `PromptSanitizer::sanitize` (26), `Storage::insert_prompt` (24), `FallbackChain::execute` (29). Changing any of these has the widest blast radius. (server-side `ErrorResponse::into_response`/`success`/`error` are the top fan-in but are mechanical response helpers.)
- **Trait seams** (extension points, ranked by implementor count): `Plugin`, `SearchEngine`, `FallbackStrategy`, `TemplateEngine`, `Linter`/`SecurityScanner`/`PerformanceChecker`/`AccessibilityChecker`, `Hook`, `Embedder`.
- See `graph/prompt-hub.graph.md` for the ASCII module/call graph + metrics, and `graph/prompt-hub.metrics.json` for the full ranked lists.
