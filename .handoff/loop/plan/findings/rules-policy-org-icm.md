# Rules / Policy / Org-Chart Findings — TARGET=icm (cycle 7)

Axis: `rules-policy-org`. Scope: owner standing rules, policy lanes, the multi-agent
**agent org chart**, and **A2A**/`weave` communication as they bind to icm — the owner-mandated
persistent-memory organ. Read-only audit; only this findings file written.

Repo: `/home/drdave/Desktop/meta/icm`. Surface: 4 crates (`icm-core`, `icm-store`, `icm-mcp`,
`icm-cli`, `Cargo.toml:3-8`), an MCP server exposing **31** agent-facing tools
(`crates/icm-mcp/src/tools.rs`, verified `grep -c '"name": "icm_'` = 31), 6 host injectors
(install path: CLAUDE.md / `~/.codex/AGENTS.md` / Cursor / etc., `crates/icm-cli/src/main.rs:293-313`),
and a single-file SQLite store (`crates/icm-store/src/store.rs`, WAL).

Verdict (one line): icm **honors Upgrade Only / No Downgrades by default** (additive migrations,
non-destructive default decay, critical/high never pruned) but is a **single-process local memory
organ with no `weave`/A2A fabric and no write-side RBAC** — every agent that holds the MCP grant can
`forget` any other agent's memory. The shared-memory bus is real but ungoverned.

---

## 1. Policy table (owner standing rules as lanes)

| Policy lane | icm posture | Evidence | Status |
|---|---|---|---|
| **Upgrade Only** (never remove legacy until Rust/meta-native replacement is installed + parity-proven) | Schema evolution is strictly additive: `CREATE TABLE IF NOT EXISTS` + idempotent `ALTER TABLE ADD COLUMN`; an explicit regression test guards against the pre-0.10.43 brick-on-upgrade. | `crates/icm-store/src/schema.rs:50,74,132,385,398`; test `test_migration_from_pre_0_10_43_schema` `schema.rs:531-610` ("Seed one row that pre-dates the migration — must survive intact") | PASS |
| **No Downgrades** (no destructive default forgetting) | Auto-decay on recall multiplies `weight` only (no row deletion) and **skips `critical`** entirely; `prune` (the only DELETE-by-weight) is **never auto-invoked** — it runs solely from explicit CLI/TUI/web with a confirm, and **never deletes `critical` or `high`**. | `apply_decay` `store.rs:1290-1305` (`WHERE importance != 'critical'`); `maybe_auto_decay` `store.rs:130-151` (decay only, no prune); `prune` `store.rs:1313-1334` (`NOT IN ('critical','high')`); prune callers are CLI `main.rs:3233`, web `web.rs:580`, TUI confirm `tui.rs:626` only | PASS |
| Strict parity before removal | N/A for icm internals (no legacy tool being retired inside this repo); applies at fabric level — icm is itself the mandated replacement memory tool, so removal of any predecessor memory store must be parity-proven against icm first. | CLAUDE.md "Persistent memory (ICM) — MANDATORY"; `AGENTS.md:1-30` | N/A — no in-repo legacy tool |
| Automate everything researchable | Strong: SessionStart wake-up inject, UserPromptSubmit recall inject, PostToolUse async extraction queue, auto-decay-on-recall (replaces an `icm decay` cron, stated `store.rs:129`). | `maybe_auto_decay` `store.rs:128-129`; hooks `main.rs:591-593,3075-3085`; extraction queue `store.rs:180-207` | PASS |
| Human only at supervised/risk boundaries | Destructive memory ops (`prune`, topic `forget`) are gated behind explicit human confirm in TUI/web; but agent-facing `icm_memory_forget`/`_forget_topic` are exposed to agents with **no confirm and no RBAC** (see §2). | TUI confirm `tui.rs:626`; web confirm svelte `+page.svelte:21`; vs MCP `tools.rs:728-729` | MIXED |
| No silent model/provider downgrade | icm does not select an LLM for inference; summarizer provider is explicit (`auto\|claude\|codex\|gemini\|ollama\|none`, `main.rs:251`) and embeddings are an optional local feature (fastembed). No covert provider switch. | `main.rs:251`; `Cargo.toml` `fastembed` optional | PASS |
| Always commit→push→PR→arm auto-merge per chunk | Process rule on the *operator*, not enforced by icm code. icm participates by being the memory each chunk's agent recalls/stores against. | meta CLAUDE.md auto-merge rule (workspace policy) | N/A — operator rule, not icm-enforced |

---

## 2. Agent org chart — icm's role in the multi-agent system

icm is the **shared persistent-memory organ** of the meta agent org, but it is a passive store, not a
coordinator. Its position in the org chart:

```
            OWNER (human, supervised/risk boundary)
                     |
          north-star: $META_ROOT + handoff (hf kernel)  ──> continuity / org control plane
                     |
   ┌─────────────────┼───────────────────────────────────────────┐
   | commander/orchestrator (plan-loop / forge-loop / Ralph loops)|
   └─────────────────┼───────────────────────────────────────────┘
       writes(store)  |  reads(recall)            ALL share ONE icm db
   ┌──────────┬───────┴────────┬──────────────┬─────────────────┐
   | planning | feature-forge  | rust-port    | background/async|
   | crew     | crew           | crew         | loop agents     |
   └────┬─────┴───────┬────────┴──────┬───────┴────────┬────────┘
        |  store/recall via MCP (31 tools) OR `icm` CLI (auto-allowed)
        v
  ┌───────────────────────────────────────────────────────────┐
  | icm-mcp server (stdio)  ──>  icm-store (single SQLite WAL)  |
  | host injectors (6): SessionStart wake-up + UserPromptSubmit |
  | recall  ──auto-inject──> every agent session's context      |
  └───────────────────────────────────────────────────────────┘
```

- **Who writes:** any agent/loop holding the MCP grant or the `icm` CLI — planning loop stores
  decisions, feature-forge stores task-complete, error-resolved triggers, etc. (the owner's MANDATORY
  store triggers, CLAUDE.md). Hooks also write autonomously (PostToolUse extraction `store.rs:191-207`).
- **Who reads:** every agent via `icm_memory_recall` (`tools.rs:727`) and, crucially, **passively**:
  SessionStart wake-up + UserPromptSubmit recall **auto-inject** stored memory into *every* session's
  context (`main.rs:591-593,3075-3085`). This makes icm a de-facto **shared memory bus** — one agent's
  stored memory reaches another agent's prompt without that agent issuing a recall.
- **RBAC / auth on memory — ABSENT.** Unlike `prompt_hub` (which has RBAC), icm has **no role/ACL/
  tenant gate on writes or deletes**. `grep -niE 'rbac|authoriz|permission|role|acl|tenant'` over
  `crates/` returns only: (a) transcript message `role` (user/assistant/system/tool — a content field,
  not authz, `main.rs:870,2074-2077`), and (b) the Claude-Code PreToolUse permission *hook* that
  auto-allows `icm` commands (`main.rs:574,2291-2292`). There is no per-agent identity on a memory row.
  Sessions carry an `agent` string (`store.rs:2410,2443-2458`) but it is descriptive metadata, not an
  authorization principal — any caller can `forget` any topic regardless of which agent stored it.
- **Privilege-grade attack surface:** the PreToolUse auto-allow is "privilege-grade" by the code's own
  comment and was hardened against prompt-injection chaining (`main.rs:2343-2400`, rejects
  substitution/redirection so `icm $(rm -rf /)` cannot ride the allow). Because recalled memory is
  auto-injected into other agents (`main.rs:3084` "auto-injected into the session without user
  confirmation"), a poisoned memory written by one agent is a cross-agent prompt-injection vector;
  the only mitigation present is `wake_up::sanitize_summary` newline-flattening (`main.rs:3085`).

---

## 3. weave / A2A communication map

icm has **no `weave` integration and no A2A transport** — it is purely local, single-process, and
coordinates only via direct CLI/MCP stdio against one SQLite file.

| Question | Finding | Evidence |
|---|---|---|
| Does icm coordinate across agents via `weave`? | **No.** `grep -rniE 'weave\|a2a\|agent-to-agent\|broadcast\|inbox'` over all `*.rs` returns **only bench-fixture prose** about a fictional consensus protocol (`icm-cli/src/bench_knowledge.rs:23,45`) — not a real transport. No `weave` import, no inbox scan, no status ledger. | `grep` result: `bench_knowledge.rs:23,45` only; no `mcp__weave` or weave crate dep in `Cargo.toml` |
| Any A2A / message bus? | **No peer A2A.** The only network egress is **RTK Cloud sync** (`icm cloud login/sync/status`, `main.rs:518-619`, `ureq`/`cloud.rs`) — a **centralized hub** for one user's machines, not peer agent-to-agent messaging. | `main.rs:518-619`; `Cargo.toml` `ureq` |
| Inbox scan cadence / background status ledger / missed-message prevention | N/A — icm exposes none. The fabric's A2A/heartbeat layer is `weave` + the `hf` handoff kernel, which icm does **not** touch (`grep` for `weave\|grit\|hf\|handoff` in `crates/` hits only ICM bench/test/example strings, not integration). | `grep` result: `store.rs:2540`, `tools.rs:3317-3334` are test fixtures only |
| Foreground-chat non-blocking rule | N/A — icm has no agent-messaging surface to block on. | — |

**Assessment / gap:** the converging fabric (handoff + weave) provides A2A; icm provides shared *state*
but reaches agents only by **passive auto-inject of last-writer-wins memory**, not by addressed
messages. For true multi-agent shared memory there is **no `weave`-mediated coordination** — concurrent
writers rely solely on SQLite isolation (§4), and there is no notification when agent A's store should
invalidate agent B's in-flight recall. An **icm-over-weave** (or icm events on the weave bus) is a
candidate fabric upgrade (UPGRADE-3) so memory mutations become observable A2A events rather than silent
DB writes.

---

## 4. Background / concurrency under parallel agent writes

icm is explicitly invoked by **background**/async agents (the planning loop stores in the background;
PostToolUse hooks enqueue extraction asynchronously, `store.rs:180-207`). Concurrency posture on the
shared SQLite store:

- **WAL + 30s busy_timeout + foreign_keys** set at open: `PRAGMA journal_mode=WAL; PRAGMA
  foreign_keys=ON; PRAGMA busy_timeout=30000;` (`store.rs:118`, also `:451`), with a regression test
  pinning the 30s timeout ("previous 5s … Bumping to 30s covers realistic" — `store.rs:5999-6009`). WAL
  permits concurrent readers during a write; the 30s busy_timeout absorbs writer contention from many
  background agents instead of erroring immediately.
- **In-process safety:** an `LruCache` behind a `Mutex` (`store.rs:100,124`); decay/prune/delete clear
  the cache to avoid stale reads (`store.rs:1309,1331`).
- **Atomic counters/slots** use `INSERT … ON CONFLICT DO UPDATE … RETURNING` so concurrent callers
  don't double-apply: auto-decay slot claim ("only one caller wins the race", `store.rs:134-148`) and
  the hook counter (`store.rs:154-166`). Consolidation uses `BEGIN IMMEDIATE` for write exclusivity
  (`store.rs:1378-1393`).
- **Gap — cross-process write amplification under heavy parallel agents:** WAL has a single-writer
  constraint; N background agents each opening their own `SqliteStore` connection serialize writes and
  lean on the 30s busy_timeout. No connection pool, no write queue, no per-agent backpressure signal.
  Under a wide fan-out (many loop agents storing simultaneously) this is correctness-safe but a
  latency/throughput hotspot, and a `BUSY` after 30s surfaces as a hard error to the agent.

---

## 5. CLAIM rows (each cited; verdict-bearing)

| ID | CLAIM | Evidence | Verdict |
|---|---|---|---|
| C1 | Schema migrations are additive-only; legacy rows survive upgrade (honors **Upgrade Only**). | `schema.rs:50,74,132,385,531-610` | CONFIRMED |
| C2 | Default behavior never destroys memory; decay is weight-only and skips `critical`; prune is opt-in and spares `critical`+`high` (honors **No Downgrades**). | `store.rs:130-151,1290-1305,1313-1334` | CONFIRMED |
| C3 | icm is a shared memory bus: stored memory auto-injects into every agent session without recall or confirmation. | `main.rs:591-593,3075-3085,3084` | CONFIRMED |
| C4 | No write-side RBAC/auth: any MCP/CLI caller can `forget` any topic; `agent`/`role` are metadata, not principals (contrast prompt_hub RBAC). | `tools.rs:728-729`; `main.rs:870,2074-2077`; grep result | CONFIRMED |
| C5 | No `weave`/A2A: icm is local single-process; only egress is centralized RTK Cloud sync, not peer A2A. | `bench_knowledge.rs:23,45` (fixtures only); `main.rs:518-619` | CONFIRMED |
| C6 | Background/parallel-agent writes are correctness-safe via WAL + 30s busy_timeout + atomic upserts, but unpooled/unqueued (latency hotspot at wide fan-out). | `store.rs:118,134-166,1378-1393,5999-6009` | CONFIRMED |
| C7 | Agent-facing destructive ops bypass the human confirm that the TUI/web enforce (asymmetric risk gate). | MCP `tools.rs:728-729` vs `tui.rs:626`, `web.rs:580` | CONFIRMED |

---

## 6. UPGRADE rows

| ID | axis | Upgrade | Evidence/rationale | Acceptance | Risk | Reversibility |
|---|---|---|---|---|---|---|
| U1 | rules-policy-org | Add a **write/delete authorization lane** (per-agent or per-topic ACL) so `icm_memory_forget`/`_forget_topic` cannot cross-delete another agent's memory; mirror prompt_hub's RBAC model. Default-deny destructive MCP ops; require a capability or topic-owner match. | C4, C7: today any grant-holder deletes anything; no principal on a row. | A non-owner agent's `forget` on a foreign topic is refused; owner/CLI path unchanged. | Med — must not break the owner's legitimate `icm forget`. | High — additive ACL table + checked at delete; feature-flag off = current behavior. |
| U2 | rules-policy-org | Gate agent-facing destructive MCP ops behind the same **confirm/dry-run** the TUI/web require (or a `--yes`-equivalent capability), closing the asymmetric risk boundary. | C7: TUI/web confirm, MCP does not. | `icm_memory_forget_topic` from an agent returns a preview/refusal unless explicitly confirmed. | Low. | High — wrapper at `tools.rs:728-729`. |
| U3 | rules-policy-org / fabric | **icm-over-weave**: emit memory-mutation events (store/forget/consolidate) onto the `weave` bus so multi-agent shared memory becomes observable A2A, enabling cross-agent recall-invalidation and a background-write status ledger. | §3 gap, C5: no A2A; passive last-writer-wins only. | A store by agent A produces a weave event other agents/loops can observe; opt-in, no-op when weave absent (keep portability). | Med — new optional dep; must stay graceful-degradation like ICM-absent no-op. | High — additive optional feature. |
| U4 | speed/scale | Add a **write queue / connection strategy** for wide background fan-out so parallel agent stores don't serialize into 30s busy_timeout errors; surface backpressure instead of hard `BUSY`. | C6: unpooled WAL single-writer hotspot. | Under N concurrent storing agents, writes complete or backpressure cleanly; no spurious `BUSY` error to agents. | Med. | High — internal to icm-store. |
| U5 | rules-policy-org / security | Strengthen **poisoned-memory defense** on the auto-inject path (provenance tagging + stronger sanitization than newline-flatten) since recalled memory is a cross-agent prompt-injection vector. | C3 + `main.rs:2343-2400,3084-3085`. | Injected memory carries provenance; suspicious content is quarantined, not silently injected. | Med. | High — additive at inject site. |

---

## 7. Replacement-of-human-bottleneck plan

| Manual action | Category | Evidence |
|---|---|---|
| Recall before work / store on triggers | **Automate now** — already automated via hooks (SessionStart, UserPromptSubmit, PostToolUse) and the MANDATORY-store discipline. | `main.rs:591-593,3075-3085`; CLAUDE.md triggers |
| `icm decay` cron | **Automate now** — replaced by auto-decay-on-recall. | `store.rs:128-151` |
| Destructive prune / topic-forget | **Supervised** today (TUI/web confirm) but agent path is ungated — recommend keep human-supervised AND add U1/U2 so agents are default-deny. | `tui.rs:626`, `web.rs:580` vs `tools.rs:728-729` |
| Cross-agent memory coordination / invalidation | **Owner-gap → automate via U3** — currently no automation; relies on passive inject. | §3 |
| Retiring any predecessor memory tool | **Owner-only** under Upgrade Only — must be parity-proven against icm before removal (no in-repo legacy exists to remove). | CLAUDE.md "MANDATORY"; AGENTS.md |

---

## 8. Confidence

High on §1, §2, §3, §4 (direct source citations; greps exhaustive over `crates/`). Medium on the
fabric-integration recommendations (U3/U5) — they assume the workspace `weave`/`hf` fabric is the
intended A2A layer (per the cycle frame), which is true at the meta level but not encoded in icm itself.
N/A items are explicitly marked with rationale; no sentinel placeholder tokens used (all claims cite file:line).
