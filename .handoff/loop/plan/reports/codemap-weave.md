# Codemap — weave (A2A transport plane / "nervous system")

- Snapshot: `@4fe2419` · branch `plan/weave-red-tests` · cycle 4 (parallel weave instance)
- Read-only map of *what exists*. Project claims (README/ARCHITECTURE) are recorded as claims to verify.
- Built from `git-kb code` (2722 symbols / 9571 resolved edges / 36 deep source files).

## What weave is

weave is the fleet's **agent-to-agent (A2A) transport plane** — a session mesh + native terminal-
multiplexer injector. It is the "nervous system" that moves messages between agents. It is
**DISTINCT from handoff's witnessed-receipts plane**: handoff records *what happened* (ledger);
weave *carries the live traffic* (mailbox + inject). The substrate is a **SQLite mailbox** (the
broker), not a network protocol daemon.

## Crate roles & boundaries (4 workspace members, strictly layered)

| crate | role | key modules | public surface (sample) |
|---|---|---|---|
| **weave-core** | types · storage · config — *the spine* | `model` (schema), `store` (`SqliteStore` broker), `store_libsql` (`LibsqlStore`), `config`, `sign`, `webpolicy`, `memory`, `archive`, `export`, `session`, `llm`, `testenv` | `Intent`, `Message`, `Peer`, `Ask`, `Job`, `Lease`, `Store::{send,inbox,history,search,sessions,pull_from_store,commit_pulled,verify_pulled_intent}` |
| **weave-inject** | multi-mux injector (writes keystrokes into a live terminal pane) | `inject` | `Mux` (tmux·zellij·kitty·wezterm·screen·iterm2·none), `Target`, `inject`, `spawn`, `kill`, `detect_target` |
| **weave-mcp** | MCP stdio server + HTTP serve/dashboard | `mcp` (78 `weave_*` tools), `http` (serve/push/dashboard), `dashboard`, `obscura` | `call_tool` router, `tool_*` handlers, `handle_connection`, `handle_dashboard_connection` |
| **weave** | the binary / CLI (71 verbs) + host wiring | `main`, `setup`, `session`, `harness`, `provider_switch`, `telegram`, `slack`, `git`, `backup` | bin `main`; no library surface |

**Dependency edges (Cargo path deps — compiler-enforced, strictly downward):**
```
weave-core   : (zero internal weave deps — the root of the DAG)
weave-inject : weave-core
weave-mcp    : weave-core, weave-inject
weave        : weave-core, weave-inject, weave-mcp
```
No upward dependency exists. (Call-graph "upward-apparent" edges are name-ambiguity artifacts — see
`graph/weave.metrics.json` › layering.)

## Entry points

| entry | path | notes |
|---|---|---|
| `weave` bin `main` | `weave/src/main.rs:4489` | dispatches 71 top-level CLI verbs |
| MCP stdio server | `weave mcp` → `weave-mcp::mcp` | 78 `weave_*` tools over stdio |
| HTTP serve / dashboard / push | `weave-mcp/src/http.rs` (`handle_connection:309`, `handle_dashboard_connection:180`) | Tier-2 cross-machine PUSH (WL-056/ADR-0005) + live dashboard |
| daemon / scheduler | `weave daemon` / `weave tick` / `weave schedule` | background delivery + cron |
| host wiring | `weave setup` → `weave/src/setup.rs` | registers MCP server + lifecycle hooks into a coding-agent host (claude default), optional git pre-commit hook |

## A2A / transport surface — the verb inventory (71 top-level CLI verbs)

```
Mcp Setup Uninstall ProviderSwitch Harness Send Notify BroadcastNotify BroadcastAsk
Outbox Pull Reply Thread Summarize Receipts Delivery Watch Responder Inbox Search Peers
Sessions Tui Scan Gc Doctor Register Attach Connect Inject Spawn Kill Ask Answer Ack Asks
AskGet AskStatus AskMany AskManyResult Job Orchestrator Config Completions Man Key Audit
Describe Status PeerPolicy Schedule Schedules CancelSchedule Tick Hook Memory Daemon Review
Permission Lease Serve Graph Dashboard Push Telegram Slack Export Backup Restore Session Web
```
Core mesh primitives: **send/notify/reply/answer/ack** (message ops), **ask/ask-many/asks**
(request-response with tracked correlation ids), **spawn/attach/connect/inject/kill** (peer
lifecycle + keystroke injection), **register/peers/sessions/scan** (discovery), **lease
{reserve/release/list/sweep}** (file/resource reservation), **broadcast-notify/broadcast-ask**
(fan-out), **serve/dashboard/push** (Tier-2 HTTP), **daemon/tick/schedule** (background).
The MCP plane mirrors these as **78 `weave_*` tools** (e.g. `weave_send`, `weave_ask`,
`weave_lease_reserve`, `weave_spawn_peer`, `weave_job_create`, `weave_memory_write`).

## Federation model — Tier-1 / Tier-2

- **Tier-1 (local, read-only federation):** a process reads peers/sessions/messages aggregated
  across local stores read-only (`store::federated_peers`, `federated_sessions`, `federation_status`
  — `weave-core/src/store.rs:23`). No cross-store writes.
- **Tier-2 (signed cross-store + HTTP push, WL-056/ADR-0005):** a sender appends a cross-store
  **delivery intent** to its own `outbox`; the **recipient's own process pulls** the intent
  (read-only) and commits it into its inbox via the owner-only-writes `Store::send` path
  (`pull_from_store` / `commit_pulled` / `verify_pulled_intent`, `weave-core/src/store.rs`).
  Cross-machine delivery rides HTTP PUSH (`weave-mcp/src/http.rs`; loopback-only by default,
  cross-machine bind gated per ADR-0005). Optional ed25519 signing makes `from` unforgeable.

## Message schema — the `Intent` struct (CONVERGENCE FINDING)

`weave-core/src/model.rs:216` — `Intent` is weave's cross-store wire schema:
`id:i64` (sender's monotonic outbox id), `ts:i64`, `to:String`, `to_host:String`, `from:String`,
`subject:Option<String>`, `body:String`, `sig:String` (ed25519, populated only by `sign` feature),
`idempotency_key:Option<String>`, `trace_id:Option<String>`, `priority:String`, `ttl:i64` (WL-038
ephemeral TTL). The receiver dedups on `(source, id)` via a per-source `pull_cursor` high-water mark
and re-stamps `ts` locally on commit.

**Gap vs A2A v1.0:** weave does **NOT** implement a formal A2A v1.0 / gRPC transport. It uses its
**own `Intent` schema** over a SQLite-mailbox + HTTP-push substrate. Any convergence onto an industry
A2A standard (or onto handoff's receipt schema) is a schema-mapping job, not a drop-in — this is the
primary cross-plane convergence finding for the architect.

## Dependency hygiene (POSITIVE FINDING)

- **No repo-escaping path deps.** All internal deps are `path = "../<crate>"` (within the weave
  repo). No `../../` or absolute path deps. Verified across all 4 `Cargo.toml`. This is a clean
  contrast to handoff's RuVector/envctl path escapes.
- External workspace-internal crates (`fnx-classes`, `fnx-algorithms`, `fnx-runtime`) are pulled by
  **git**, not path (`weave/Cargo.toml:49-51`).
- **Dual storage backend:** `sqlite` (default, via `rusqlite`) and `libsql` (opt-in, adds `tokio`).
  Backends are kept behavior-symmetric (`SqliteStore` ↔ `LibsqlStore`).
- **`sign` feature** (ed25519-dalek + sha2) is **default-off**; signing is the Tier-2 unforgeable-
  identity upgrade path, reserved in the `Intent.sig` field now so enabling it needs no migration.

## Build / run surface

- Workspace v0.2.0, edition 2021, `resolver = "2"`, license MIT OR Apache-2.0.
- Features: `weave` bin `default = ["sqlite"]`; `libsql` and `sign` propagate to `weave-core`/`weave-mcp`.
- Supply-chain gates: `deny.toml`, `scripts/supply_chain_audit.py`, `scripts/target_smoke.py`.
- No HTTP routes/gRPC services detected by the indexer (`query routes`/`route-clients`/
  `handler-routes` all empty) — consistent with the custom mailbox+inject transport, not a REST/gRPC API.

## Notes / claims to verify (for the analyst + verifier)

- ARCHITECTURE.md (126 KB) and CHANGELOG.md (100 KB) state design intent (Tier model, WL-* workitems,
  ADRs) — treat as claims; the verifier checks them against `store.rs`/`http.rs`/`model.rs`.
- The 3 Tarjan SCCs and the "upward-apparent" cross-crate edges are flagged as resolver artifacts
  (774 ambiguous resolutions); the verifier should confirm the suspect back-edges don't exist.
