# Code graph — prompt-hub (ASCII view + intelligence)

Snapshot git `f826ea33` (`plan/fleet-arch-integration-cycle1`), 2026-06-27.
Built from `git-kb code` 0.2.10. Symbols: 3,589 (members). Intra-repo call edges: 4,006 (non-test).
Diagram-ready for the architect (box-drawing).

> **Graph-fidelity caveat (load-bearing):** under the vendor-inclusive full index
> `git-kb` resolved **0** call edges (`call_edge_count=0`, vendor = 1.42M of 1.43M symbols).
> Edges only resolved after **re-indexing each member's `src/` in isolation**. The Rust
> resolver under-resolves method/trait-dispatch + generic calls, so **degree counts are a
> LOWER BOUND** and a few SCCs are same-name artifacts (flagged). Layering + module structure
> below are corroborated by Cargo manifests and are high-confidence.

## 1. Member / layer graph (verified from Cargo path deps)

```
                         ┌──────────────────────────────────────────┐
   front-ends (thin)     │                                          │
                         ▼                                          ▼
        ┌─────────────────────────┐                 ┌─────────────────────────────┐
        │  prompthub (CLI)        │                 │  prompthub-server (HTTP API)│
        │  bin: main.rs:30        │                 │  bin: main.rs:46            │
        │  clap Commands (~41)    │                 │  axum Router 111 routes     │
        │  commands/ (21) + tui   │                 │  routes.rs + state.rs       │
        └───────────┬─────────────┘                 └──────────────┬──────────────┘
                    │  13 call edges                  177 call edges │
                    │  (path dep)                       (path dep)   │
                    └───────────────┬──────────────────────────────┘
                                    ▼
                 ┌───────────────────────────────────────────────────┐
                 │  prompt-hub (core library) — lib.rs: 70 pub mod    │
                 │  ┌───────────────────────────────────────────────┐│
                 │  │  PromptHub facade (hub.rs, 183 pub methods)    ││
                 │  └───────────────────────────────────────────────┘│
                 │  Store  Search  Vibe/Intent  Governance  Lifecycle │
                 │  Providers/Cost  Ops/Sync  Inputs/UX  Plugins      │
                 └───────────────────────────────┬───────────────────┘
                                                 ▼
                        ┌──────────────────────────────────────┐
                        │ external: libsql (local SQLite)       │
                        │ tower/axum, handlebars/tera, argon2,  │
                        │ tiktoken/tokenizers, qdrant, inventory│
                        └──────────────────────────────────────┘
```

**Cross-member call edges (resolved, non-test):** `prompthub-server → prompt-hub` = 177,
`prompthub → prompt-hub` = 13. The apparent `prompthub-server → prompthub` (14) is a
**name-resolution artifact** (server tests → `FuzzyPromptFinder::new`); server has no Cargo
dep on the CLI. **Layering is strict/clean.**

## 2. The front-door / intent path (what exists)

```
  UserInput (models.rs:599)                 ┌─ NOT IMPLEMENTED ─────────────┐
        │                                   │  goal-artifact emission        │
        ▼   POST /api/v1/input/process      │  → rusty-idd (ADR-0007)        │
  PromptHub::process_input (hub.rs:1401) ─X─┤  only in docs/plans/           │
        │                                   │  lifeos-meta-front-door.md     │
        ▼                                   └────────────────────────────────┘
  MultiModalInput::process (multimodal_input.rs:25)
        │
        ▼
  Intent (models.rs:558) ──► vibe path: VibeEngine / IntentClassifier /
                                        SkillRecommender / PromptGenerator (vibe.rs)
                                        │
                                        ▼  Artifact enum (models.rs:654)
                              {Prompt|Code|Config|Test|Migration|Documentation}
                              (swarm execution output — NOT provenance-stamped, NOT rusty-idd bound)
```

The `X` marks the missing seam: the code stops at `Intent` → internal vibe/swarm output.
There is **no provenance stamping and no rusty-idd handshake** in source.

## 3. Hotspots — top fan-in (most-called intra-repo; LOWER BOUND)

```
 in  out  symbol                              location
─── ───── ─────────────────────────────────  ────────────────────────────────────
111   0   ErrorResponse::into_response        prompthub-server/responses.rs:59   (mechanical)
110   2   evolve_test_state                   prompthub-server/routes.rs:3846    (test helper)
109   0   success                             prompthub-server/responses.rs:66   (mechanical)
 76   5   PromptHub::lock          ◄ core     prompt-hub/hub.rs:1199
 61   1   error                               prompthub-server/responses.rs:81   (mechanical)
 45   0   HubConfig::load          ◄ core     prompt-hub/config.rs:82
 44   0   SandboxStore::insert     ◄ core     prompt-hub/sandbox.rs:67
 41   0   Storage::acquire         ◄ core     prompt-hub/storage.rs:146  (DB checkout)
 33   0   HubConfig::default       ◄ core     prompt-hub/config.rs:55
 29   2   FallbackChain::execute   ◄ core     prompt-hub/fallback.rs:216
 26   8   PromptSanitizer::sanitize◄ core     prompt-hub/sanitize.rs:109
 25   2   map_hub_error                       prompthub-server/routes.rs:2424
 24   2   Storage::insert_prompt   ◄ core     prompt-hub/storage.rs:283
 24   1   PromptSanitizer::default ◄ core     prompt-hub/sanitize.rs:92
 24   1   LockManager::new         ◄ core     prompt-hub/hub.rs:115
```

**Architectural hotspots** (filtering out response/test helpers): `PromptHub::lock`,
`Storage::acquire`/`insert_prompt`, `HubConfig::load`, `PromptSanitizer::sanitize`,
`FallbackChain::execute`. These are the highest-blast-radius change points.

### Top fan-out (orchestrators)
```
 out in  symbol                              location
─── ── ──────────────────────────────────  ──────────────────────────────
 21  3  MetricsCollector::prometheus_text   prompt-hub/metrics.rs:328
 15  2  MetricsCollector::summary           prompt-hub/metrics.rs:272
 11  1  structured_json                     prompt-hub/accessibility.rs:308
  9  7  PromptHub::evolve_prompt            prompt-hub/hub.rs:1801
```

### Trait seams (extension points; ranked by implementor count, git-kb global hotspots)
`Plugin` (5) · `SearchEngine` (5) · `FallbackStrategy` (4) · `Linter` (3) · `TemplateEngine` (3) ·
`SecurityScanner`/`PerformanceChecker`/`AccessibilityChecker` (2 each) · `Hook` (2) · `Embedder` (2).

## 4. Cycles (Tarjan SCC, in-process over the edge list)

```
SCC count (size>1): 4   |   self-recursive fns: 0   |   large cycles: NONE → effectively a DAG
  • shutdown ↔ stop_purge_daemon        (plausible genuine daemon-lifecycle 2-cycle)
  • subscribe ↔ subscribe   (artifact)  ┐ same-name resolution collisions:
  • clear ↔ clear           (artifact)  ├ two distinct symbols share a name; discount
  • search ↔ search         (artifact)  ┘
```
No multi-node cycles. The module graph is acyclic in practice — a clean structural signal.

## 5. Layering

```
declared:   CLI ─┐                          actual edges:  server→core 177 ✓
                 ├──► core (prompt-hub)                     cli→core      13 ✓
         server ─┘                                          server→cli    14  ✗artifact
verdict:    STRICT / CLEAN.  0 real violations (the 14 are name-resolution artifacts;
            prompthub-server has no Cargo dep on prompthub).
```

## 6. Public API & dead code (summary; full lists in metrics.json)
- **Public src symbols:** 1,405 (core 1,236 · server ~110 · cli ~59). Core `PromptHub` exposes 183 pub methods — a very wide facade.
- **Dead-code candidates (git-kb NoCallers, src-scoped):** 416 — **candidates, not confirmed**. Inflated by the empty-edge-table problem + Rust trait-object / test-only usage. Needs manual confirmation per symbol before any removal.

## 7. Server route map (111 routes) — see codemap §2 for the domain table
Counts: prompts CRUD/lifecycle 17 · gc/auto-purge/retention 14 · providers/lb 14 ·
budget/cost/quota 14 · context/diff/template/vibe/input 11 · rollouts/deploy 8 ·
moderation/privacy/confidence/fallback/learn 8 · satisfaction/beta 8 · lineage 7 ·
audit/SOC2 5 · ops 6. Front-door normalizer = `POST /api/v1/input/process`.
