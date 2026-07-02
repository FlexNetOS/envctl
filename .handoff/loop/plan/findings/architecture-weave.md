# findings — weave / architecture (cycle 4, parallel weave instance)

- Dimension: **weave/architecture** — "what is the structural shape of the A2A transport plane, where are
  the load-bearing seams, and what must change to converge on A2A v1.0 interop without losing the
  SQLite-mailbox transport?"
- Source read-only: `/home/drdave/Desktop/meta/.worktrees/plan-weave-red/weave` @4fe2419
- Graph/metrics: `graph/weave.{graph.md,metrics.json}`, `reports/codemap-weave.md`
- Verdict (1 line): **Cleanly layered 4-crate spine with one real broker abstraction (`Store` trait,
  29 methods) — architecturally sound and adapter-ready; the structural debt is a 9.6k-line `main.rs`
  god-file, an unenforced dual-backend parity invariant, and the absence of any A2A-standard adapter.**

---

## CLAIM rows

- CLAIM: Crate layering is strictly downward and compiler-enforced — `weave-core` has **zero** internal weave deps; the path-dep DAG is `weave-core <- weave-inject <- weave-mcp <- weave` | evidence: `weave-core/Cargo.toml` (deps block has no `path="../"` entries) + `weave/Cargo.toml:46-48` (`weave-core`/`weave-inject`/`weave-mcp` path deps) + metrics `layering.manifest_dag` | confidence: high
- CLAIM: All "upward-apparent" call edges (`weave-core`→`weave-mcp` ×98, →`weave` ×36) are name-ambiguity resolver artifacts, not real deps — they cannot exist given the Cargo DAG | evidence: `graph/weave.metrics.json` `layering.cross_crate_edges` (classification "upward-apparent") + 774 `ambiguous` resolutions in `unresolved` | confidence: high
- CLAIM: The `Store` trait (`weave-core/src/store.rs:73`, **29 methods**) is the single broker abstraction; `open_store` returns `Box<dyn Store>` and the whole CLI dispatches through `&dyn Store` — this trait is the natural seam for an A2A adapter | evidence: `weave-core/src/store.rs:73` (`pub trait Store: Send`), `weave/src/main.rs:1657` (`fn open_store(...) -> Result<Box<dyn Store>>`), `:4657` (`let store = open_store(&cfg)?`) | confidence: high
- CLAIM: `weave-core/src/model.rs` is the single highest-blast file (1238 transitive dependents) because `Intent` (`model.rs:216`) is the cross-store wire schema — fields `id,ts,to,to_host,from,subject,body,sig,idempotency_key,trace_id,priority,ttl` | evidence: `graph/weave.metrics.json` `blast_radius.by_file` (model.rs:1238) + `weave-core/src/model.rs:216` struct def | confidence: high
- CLAIM: `SqliteStore.send` (`store.rs:3153`) is the load-bearing deliver verb — owner-only INSERT into `messages`, idempotent on `idempotency_key` (returns existing id), guarded by `check_ident`/`check_body`/`idempotency_key_valid`/`trace_id_valid` before write | evidence: `weave-core/src/store.rs:3153-3189` (42 in-degree callers per metrics `top_in_degree`) | confidence: high
- CLAIM: The MCP plane is a **flat string-match router** — `call_tool` (`mcp.rs:434`, out-degree 50) is a single `match name { "weave_send" => tool_send(...), ... }` over the 78 `weave_*` tools, mirroring the CLI verbs by hand | evidence: `weave-mcp/src/mcp.rs:434-520+` | confidence: high
- CLAIM: The CLI is a god-file dispatch — `weave/src/main.rs` is **9631 lines** (not 4489; line 4489 is only where `fn main` begins), with **two** large sequential matches: a pre-store `match &cli.cmd` (`:4494`) and a post-store `match cli.cmd` (`:4660`) over **76 `Cmd::` arms** plus inline business logic | evidence: `wc -l weave/src/main.rs`=9631; `main.rs:4489` (`fn main`), `:4494`, `:4660`; `grep -cE '^\s+Cmd::'`=76 | confidence: high *(corrects the anchor's "4489 lines")*
- CLAIM: Tier-2 cross-store delivery is owner-pull, not sender-push-write — `pull_from_store` (`store.rs:2808`) + `commit_pulled` (`:2870`) + `verify_pulled_intent` (`:2967`) let the recipient's own process pull an `Intent` read-only and commit it through the normal `send` path; per-source `pull_cursor` high-water mark dedups on `(source,id)` | evidence: `weave-core/src/store.rs:2808/2870/2967`, `model.rs:216` doc | confidence: high
- CLAIM: **No A2A v1.0 / gRPC / AgentCard / JSON-RPC adapter exists** anywhere in weave-core or weave-mcp — convergence onto an industry A2A standard is schema-mapping work over the `Store`/`Intent` seam, not a drop-in | evidence: `grep -rln 'a2a|A2A|AgentCard|agent_card|jsonrpc 2.0|json-rpc' weave-core/src weave-mcp/src` → empty; codemap "No HTTP routes/gRPC services detected" | confidence: high
- CLAIM: The #1 reported hotspot `Harness.new` (302 callers, `obscura.rs:414`) is **test-only** — it lives inside `#[cfg(test)] mod tests` (`obscura.rs:396`); the real production hubs are `now` (model.rs:98), `send` (store.rs:3153), `check_ident` (store.rs:1235) | evidence: `weave-mcp/src/obscura.rs:396` (`mod tests`), `:413` (`impl Harness`) | confidence: high *(refines metric: test code inflates centrality)*
- CLAIM: The dual backends (`SqliteStore`, `LibsqlStore`) implement the 29-method `Store` trait **independently**, and parity is asserted in comments only — there is **no** shared conformance/differential test harness; SqliteStore carries 160 `#[test]` vs LibsqlStore 95, so the libsql impl is materially less exercised | evidence: `store.rs:80` comment "splitting keeps both backends identical"; `grep -c '#\[test\]'` store.rs=160 / store_libsql.rs=95; `grep 'macro_rules.*store_test|store_conformance|both_backends'` → empty | confidence: high
- CLAIM: The 3 Tarjan SCCs are resolver back-edges, not real recursion — `open→open_conn` (`store.rs:2469/2499`) and `call_tool→tool_meta` (`mcp.rs:434/4854`) are real forward edges whose reverse edge is an ambiguous resolve | evidence: `graph/weave.metrics.json` `cycles.verdict`; forward edges confirmed in `call_tool` body (`mcp.rs:437` calls `tool_meta`) | confidence: medium *(verifier: confirm no reverse edge in source)*

---

## Gaps (architecture-revealed)

1. **Unenforced dual-backend invariant.** The "both backends identical" contract is the strongest
   architectural promise in weave-core (it's what makes `libsql` a safe opt-in) but nothing tests it
   across the 29-method surface; libsql is also 40% less test-covered. Backend drift is silent until a
   production libsql user hits it. *(Cross-dim hook: test-coverage, correctness.)*
2. **`main.rs` god-file (9631 lines, 76 arms, blast 427).** Dispatch is interleaved with verb business
   logic in two giant matches → each verb is hard to unit-test in isolation, and any verb edit touches
   the highest-blast bin file. Only a few verb-groups (`dispatch_memory`/`dispatch_lease`/`dispatch_job`
   at `:7090/:7305/:7410`) are already extracted; most are inline.
3. **Hand-mirrored CLI↔MCP verb surface (71↔78).** Two independent flat matches (`Cmd::` in main.rs,
   `match name` in mcp.rs) must be kept in sync by hand — a verb added to one and not the other is a
   silent capability gap with no compile-time or test guard. *(Cross-dim hook: governance-config.)*
4. **No A2A-standard interop surface.** The convergence target (A2A v1.0 interop while keeping the
   SQLite-mailbox transport) has no landing seam yet; it must be built as an adapter over the existing
   `Store` trait + an `Intent`↔A2A-Message schema map.

---

## UPGRADE rows

- UPGRADE: Add a backend-conformance test harness — a single parametrized suite (`fn run_store_conformance(s: &dyn Store)` or a `macro_rules!`) exercising all 29 `Store` methods, invoked once against `SqliteStore` and once against `LibsqlStore`, so any behavioral divergence fails RED | axis: quality | target-surface: `weave-core/src/store.rs` (trait) + new `weave-core/tests/store_conformance.rs` | rationale: the "both backends identical" invariant (store.rs:80) is comment-asserted only; libsql is 40% less tested (95 vs 160) so drift is silent | evidence: `store.rs:73` (29-method trait), `grep -c '#[test]'` 160 vs 95, `graph` blast store.rs=462 / store_libsql.rs=488 | blast: 462+488 (both store impls) | effort: M | risk-tier: APPLY (additive test module; touches no production code) | acceptance: one shared conformance suite runs against both `dyn Store` impls and fails if any method (send/inbox/history/search/pull/commit/…) returns divergent results for identical inputs | reversibility: Integrity-preserving (tests only) · fully Reversible (delete the file) · Capability-Gain = enforced dual-backend parity
- UPGRADE: Build an A2A v1.0 interop adapter as a default-off feature over the `Store`/`Intent` seam — map `Intent`↔A2A Message + expose an AgentCard, keeping SQLite-mailbox as the transport underneath | axis: accuracy | target-surface: new `weave-mcp/src/a2a.rs` (adapter) + `Intent` schema map on `weave-core/src/model.rs` | rationale: convergence target requires interop; no A2A surface exists today so this is the primary structural add | evidence: `grep a2a|AgentCard|json-rpc` empty; `model.rs:216` Intent is the wire schema (blast 1238); `Store` trait is the stable seam (`store.rs:73`) | blast: 1238 (Intent schema map) — design as additive fields/mapping, never mutate existing Intent fields | effort: L | risk-tier: PROPOSE (new public protocol surface + touches the highest-blast schema) | acceptance: an A2A v1.0 client round-trips a message through weave via the adapter — every `Intent` field (id,ts,to,from,body,priority,ttl,idempotency_key,trace_id) maps to/from an A2A Message with no field loss | reversibility: Integrity-preserving (default-off, additive) · Reversible (feature-gate off) · Capability-Gain = industry A2A interop without abandoning the mailbox
- UPGRADE: Extract the post-store CLI dispatch (`main.rs:4660+`) into a `dispatch` module of per-verb handler fns (extending the existing `dispatch_memory`/`dispatch_lease`/`dispatch_job` pattern), shrinking `main.rs` and making each verb independently unit-testable | axis: quality | target-surface: `weave/src/main.rs` (→ new `weave/src/dispatch/*.rs`) | rationale: 9631-line god-file, 76 arms, highest-blast bin (427); business logic interleaved with dispatch blocks isolation testing | evidence: `wc -l main.rs`=9631, `grep -cE '^\s+Cmd::'`=76, blast main.rs=427, prior extractions at `main.rs:7090/7305/7410` | blast: 427 | effort: L | risk-tier: PROPOSE (large structural refactor on a high-blast file) | acceptance: `main.rs` drops below a stated line threshold and each mesh verb dispatches through a named handler fn that has at least one direct unit test | reversibility: Integrity-preserving (pure move, behavior-identical) · Reversible (revert the module split) · Capability-Gain = per-verb testability + lower edit-blast
- UPGRADE: Single-source the CLI↔MCP verb surface — derive both the `Cmd` dispatch and the `call_tool` tool list from one declarative verb registry (or add a test that asserts every mesh CLI verb has a matching `weave_*` MCP tool and vice versa), so the 71↔78 mirror cannot silently drift | axis: governance+settings+config | target-surface: `weave/src/main.rs` (Cmd enum) + `weave-mcp/src/mcp.rs:434` (call_tool router) | rationale: two hand-maintained flat matches with no cross-guard; a verb added to one plane and not the other is an undetected capability gap | evidence: `mcp.rs:434` flat `match name`, `main.rs:4660` flat `match cli.cmd`, codemap "71 CLI verbs / 78 MCP tools" | blast: 124 (mcp.rs) + 427 (main.rs) | effort: M | risk-tier: PROPOSE (touches the control-plane verb surface of two crates) | acceptance: a test enumerates CLI mesh verbs and MCP tools and fails RED on any orphan in either direction (modulo an explicit allowlist) | reversibility: Integrity-preserving · Reversible · Capability-Gain = drift-proof CLI/MCP parity

---

## Open questions for the verifier

- Confirm the 3 SCC reverse edges (`open_conn→open`, `tool_meta→call_tool`, inject `spawn/kill` cluster)
  do **not** exist in source — i.e. they are genuinely resolver artifacts (read `mcp.rs:4854`,
  `store.rs:2499`, `inject.rs:714/816`).
- Confirm `LibsqlStore.send` (`store_libsql.rs:1499`) reproduces the same idempotency-on-key short-circuit
  and `check_ident`/`check_body` ordering as `SqliteStore.send` (store.rs:3153) — this is the parity
  invariant the conformance-harness upgrade would lock down; a divergence here is itself a correctness gap.
- Feasibility-gate the A2A adapter against the no-C / trust-boundary invariant: the adapter must be pure
  Rust (it can ride the already-present `serde_json`); confirm no new C-linking dep is required.

## Counts
- CLAIM rows: 11 (high ×10, medium ×1)
- UPGRADE rows: 4 (quality ×2, accuracy ×1, governance+settings+config ×1 · APPLY ×1, PROPOSE ×3)
