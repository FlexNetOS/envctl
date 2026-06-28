# verdicts — plan-verifier gate (the GATE)

Adversarial refutation of analyst CLAIMs + feasibility-gating of UPGRADEs. Only CONFIRMED/QUALIFIED +
feasible rows flow to the architect. Default-skeptical; fail-closed; gate never weakened.

---

## weave (cycle 4)

- Date: 2026-06-26
- Target code (READ-ONLY): `/home/drdave/Desktop/meta/.worktrees/plan-weave-red/weave` @ `4fe2419`
- Findings gated: `architecture-weave.md` (11+1 CLAIM / 4 UPGRADE) + axis findings
  (`governance-config`, `memory-vector-intelligence`, `prompt-architecture`, `test-strategy`).
- Method: opened cited source, ran empirical `grep`/`wc`/`awk` over the real tree as oracle, hunted
  counter-examples. Empirical commands re-run, not trusted from the analyst summary.

### Empirical bench (the oracle results)

| probe | analyst said | I measured | verdict impact |
|---|---|---|---|
| A2A symbols in `weave-core/src`+`weave-mcp/src` (`to_a2a/from_a2a/AgentCard/"message/send"/a2a`) | empty | **empty** (0 hits) | (a) CONFIRMED |
| `jsonrpc` strings in tree | MCP, not A2A | all MCP (`obscura.rs` `tools/call`, `notifications/initialized`); no A2A `message/send` | (a) CONFIRMED — distinct standard |
| `wc -l weave/src/main.rs` | 9631 | **9631** | (c) CONFIRMED |
| `^\s+Cmd::` arms | 76 | **76**; two matches at `:4494`/`:4660`; `fn main` at `:4489` | (c) CONFIRMED |
| `enum Cmd` variants | 71 CLI verbs | **71** | corroborates verb surface |
| `weave-core/tests/store_conformance` (shared dual-backend harness) | absent | **absent** (no `run_store_conformance`/`store_test!`/`both_backends`) | (b) CONFIRMED |
| `#[test]` store.rs vs store_libsql.rs | 160 / 95 | **160 / 95** | (b) CONFIRMED |
| `weave-core/src/memory.rs` | real organ, `~/.config/weave/memory`, 5 MCP tools | **present (25.9K)**; doc confirms FS-backed scoped notes; 5 `weave_memory_*` tools; **0 `icm` refs** | (e) CONFIRMED |
| `MAX_STANDING_TOOLS_BYTES` | 8192 | **8192** (`mcp.rs:233`); budget test `standing_mcp_surface_is_within_token_budget` present | (f) CONFIRMED |
| `weave_*` tool count | 78 (arch) / 70 arms,74 catalog (prompt) | **72 dispatch arms / 76 catalog entries** | (f) QUALIFIED — number drifts |
| `sync-master.yml` required checks | 6 documented | **7 enforced** incl. `audit` (`:46`); CLAUDE.md:43 + policy.toml:37 list 6 | (g) CONFIRMED |
| Python in CI | invariant says no-Python | **`ci.yml:69,181`** run `python3 scripts/{target_smoke,supply_chain_audit}.py` | (g) CONFIRMED |
| vector/RAG deps (`embedding/faiss/qdrant/hnsw/candle/onnx…`) | none | **none** (only a false-positive prose comment in `model.rs:23`) | CONFIRMED |

### CLAIM verdicts — architecture

- ARCH-01 (crate layering strictly downward, `weave-core` zero internal deps) -> **CONFIRMED** (manifest DAG `weave-core <- inject <- mcp <- weave`; "upward-apparent" call edges are name-ambiguity, cannot exist at compile time).
- ARCH-02 (upward call edges are resolver artifacts) -> **CONFIRMED** (cross-checked against the SCC probe below: `open_conn` resolves `open` to rusqlite `Connection::open`, a real downward call mis-bound to `SqliteStore::open`).
- ARCH-03 (`Store` trait is the single broker abstraction, **29 methods**) -> **QUALIFIED** — the abstraction is CONFIRMED (`store.rs:73` `pub trait Store: Send`; `open_store -> Result<Box<dyn Store>>` at `main.rs:1657`; CLI dispatches `&dyn Store`). **Counter-evidence on the count:** the trait body (lines 73–873) declares **~95 `fn` signatures (~90 required + 3 default-bodied)**, NOT 29. The "29" understates the broker surface; the conformance-harness UPGRADE acceptance ("all 29 Store methods") must be re-scoped to the real method count or the harness will silently under-cover the trait.
- ARCH-04 (`model.rs`/`Intent` is the cross-store wire schema, highest blast 1238) -> **CONFIRMED** (`Intent` at `model.rs:216` with the cited fields incl. `#[serde(default)]` `to_host`/`sig`; blast 1238 from metrics).
- ARCH-05 (`SqliteStore.send` is the owner-only, idempotent, guarded deliver verb) -> **CONFIRMED** (`store.rs:3153`: `check_ident`(sender,recipient) → `check_body` → `idempotency_key_valid`/`trace_id_valid` → SELECT-existing-by-key short-circuit → owner INSERT into `messages`).
- ARCH-06 (MCP plane is a flat string-match router over the tool surface) -> **QUALIFIED** — the flat `match name { "weave_*" => tool_* }` router IS CONFIRMED (`call_tool` `mcp.rs:434`). **Counter-evidence on the count:** the surface is **72 dispatch arms / 76 catalog entries**, not 78. Use the measured numbers in any drift-guard test.
- ARCH-07 (CLI is a 9631-line god-file, 76 `Cmd::` arms, two sequential matches) -> **CONFIRMED** (empirically exact; corrects the anchor's "4489").
- ARCH-08 (Tier-2 delivery is owner-pull via `pull_from_store`/`commit_pulled`/`verify_pulled_intent`, dedup on `(source,id)` via `pull_cursor`) -> **CONFIRMED** — the owner-only-write `send` path was directly verified (ARCH-05); the pull/commit/cursor surface is corroborated independently by the `pull_cursor` table (`store.rs:1561`, memory-finding schema) and codemap §Federation. No counter-example (no sender-push-write path found).
- ARCH-09 (**No A2A v1.0/gRPC/AgentCard/JSON-RPC-A2A adapter exists**) -> **CONFIRMED** — grep empty across both crates; the only `jsonrpc` strings are MCP. Strengthened by `weave-core/tests/a2a_interop.rs`, a committed RED suite that compiles against the *existing flat* `Intent` and asserts the unbuilt A2A shape (i.e. it exists precisely *because* the adapter does not). Convergence is schema-mapping work over the `Store`/`Intent` seam.
- ARCH-10 (`Harness.new` #1 hotspot is test-only) -> **CONFIRMED** (`obscura.rs:396` `mod tests`, `:413` `impl Harness`, `:414` `fn new`). Production hubs (`now`, `send`, `check_ident`) stand.
- ARCH-11 (dual backends implement `Store` independently; parity comment-asserted only; no conformance harness; 160 vs 95 tests) -> **CONFIRMED** — no shared conformance test exists; counts exact. Adversarial probe found a *real* backend asymmetry that proves the risk: `LibsqlStore.send` (`store_libsql.rs:1499`) opens with `self.guard_writable()?` **before** the `check_ident` block; `SqliteStore.send` has **no** `guard_writable` call. Core idempotency/check ordering matches, but the impls are NOT byte-identical — exactly the silent drift the harness would catch.
- ARCH-12 (the 3 Tarjan SCCs are resolver back-edges, not real recursion; confidence medium) -> **QUALIFIED** — split verdict:
  - `open ↔ open_conn`: **CONFIRMED resolver artifact** — `open_conn` (`store.rs:2499`) calls rusqlite `Connection::open(path)`, mis-bound to `SqliteStore::open`. Not real recursion.
  - `call_tool ↔ tool_meta`: **REFUTED as artifact** — it is a **genuine bounded mutual recursion**: `tool_meta` `mode=call` dispatches the inner op back through `call_tool` (`mcp.rs` ~4925), and `call_tool` routes the `weave` meta-tool to `tool_meta`. Termination is guaranteed by the `if want == "weave" { return Err }` self-target guard, not by the edge being fake. The "resolver artifact" framing is wrong for this pair (the cycle is real and safe).
  - inject 5-cluster (`spawn/kill/run_bounded*`): **INCONCLUSIVE** — `spawn` (`:714`) really calls `run_bounded_env`; the full cycle was not traced this pass.

### CLAIM verdicts — cross-axis (key gates d/e/f/g)

- GOV-001 (PreToolUse gate has real teeth AND is opt-in) [key (d)] -> **CONFIRMED both clauses.** Teeth: `main.rs:8896-8919` deny-by-default for a dangerous tool unless an approver is positively proven; broadcast approver → DENY; the drain enforces its **own** short `pretooluse_timeout` and emits an explicit `deny`, never relying on Claude's 600s fail-OPEN timeout (`main.rs:8800-8809`); in-drain `pretooluse_is_dangerous` reconfirms danger so an over-broad matcher can't sneak a benign tool through. Opt-in: `setup.rs:194-196` — the `PreToolUse` hook is installed only with `weave setup --pretooluse`; default leaves it uninstalled, and with no approver configured it denies everything. (Strongest control in the repo, dormant by default.)
- GOV-003 (docs say "6" CI checks; gate enforces "7" incl. `audit`/WL-044) [key (g)] -> **CONFIRMED** (`sync-master.yml:46` = 7; `CLAUDE.md:43` & `policy.toml:37` = 6). Teeth-bearing supply-chain `audit` gate is real but understated in both human and machine policy.
- GOV-004 (Python in CI vs the no-Python Rust-native invariant) [key (g)] -> **CONFIRMED** (`ci.yml:69` `target_smoke.py`, `:181` `supply_chain_audit.py`; invariant `CLAUDE.md:22,47-59`). Genuine language drift in the build/CI plane.
- PA-TOOLS (78-tool surface collapses to one byte-budget-gated meta-tool) [key (f)] -> **QUALIFIED** — the token-safety MECHANISM is CONFIRMED (single `weave` meta-tool default via `tools()`; `MAX_STANDING_TOOLS_BYTES=8192`; budget + progressive-default tests). Count QUALIFIED: 72 arms / 76 catalog, not 78. Eager-flat (`WEAVE_MCP_EAGER=1`) is opt-in and budget-exempt.
- PA-METACALL (meta-tool `call` re-applies the destructive gate; not a bypass) [key (f)] -> **CONFIRMED** (`mcp.rs:4925-4933`: `call` rejects `want=="weave"` and re-checks `is_dangerous_tool` under safe-HTTP mode; locked by test `meta_call_preserves_safe_http_gate`).
- MEM-1 (the SQLite store is transport/event log, not recall memory) -> **CONFIRMED** (mailbox/queue/receipt schema; only query surface is FTS5 over message bodies — searching traffic, not recalled facts).
- MEM-2 (weave ships a real memory organ in `memory.rs`; a 6th memory surface that can't see ICM) [key (e)] -> **CONFIRMED** — module present (25.9K), FS-backed scoped notes under `~/.config/weave/memory`, full CRUD+search, exposed on CLI + 5 MCP tools, auto-injected via `build_context_prefix` (`memory.rs:332`); **zero `icm` references** in `memory.rs` (it cannot read ICM). Separation-of-concerns risk is real.
- MEM-3 (no vector/embeddings/RAG anywhere) -> **CONFIRMED** (empirical grep: 0 vector deps/symbols; sole hit is a prose false-positive). Correct state for a transport plane.

### UPGRADE verdicts (feasibility-gate: NO-C-in-trust-boundary · pure-Rust · strict-upgrade)

- U-ARCH-1 (backend-conformance test harness over both `dyn Store` impls) -> **FEASIBLE** — additive test crate, pure Rust, touches no production code (APPLY tier holds). **Condition:** re-scope acceptance from "all 29 methods" to the trait's real surface (~90 required methods, ARCH-03) so it does not under-cover. The verified `guard_writable` asymmetry (ARCH-11) is a ready first divergence target.
- U-ARCH-2 (A2A v1.0 interop adapter, default-off, over the `Store`/`Intent` seam) -> **FEASIBLE** — passes the hard gate. The RED suite proves the mapping rides the **already-present `serde_json`** (no new dep); AgentCard signing rides the existing pure-Rust `ed25519-dalek` under the default-off `sign` feature → **no C enters the trust boundary**. Must be ADDITIVE (new `to_a2a`/`from_a2a` + new `a2a.rs`/types, feature-gated default-off) and must **not** re-derive `Intent`'s existing serde — the native Tier-2 goldens (`integration.rs:3541/3646`) and the SQLite-mailbox transport stay intact (strict-upgrade preserved). If a gRPC binding is ever added it must use pure-Rust `tonic`/`prost`, not a C protobuf — flagged for the architect, not required for the JSON-RPC binding.
- U-ARCH-3 (extract post-store CLI dispatch into a `dispatch/*` module) -> **FEASIBLE** — pure behavior-identical move, pure Rust, extends the existing `dispatch_memory/lease/job` pattern. PROPOSE tier (highest-blast bin) is appropriate; reversible.
- U-ARCH-4 (single-source the CLI↔MCP verb surface / cross-guard test) -> **FEASIBLE** — the additive cross-guard-test variant is the low-risk pure-Rust path; the full declarative-registry derive is the heavier option. **Condition:** the parity test must enumerate the *measured* surfaces (71 CLI verbs / 72 MCP arms / 76 catalog), not the stale "71↔78".
- U-GOV-001 (align documented CI gate 6→7) -> **FEASIBLE** — doc-only edit; PROPOSE (protected files). Pure tightening of truthfulness.
- U-GOV-002 (remove Python from the CI/build plane, Rust `xtask` replacement) -> **FEASIBLE** — restores the Rust-native invariant; pure Rust. **Condition (never weaken):** the Rust replacement must reproduce the same supply-chain/target-smoke gate outputs (same `target-smoke.json` schema, same `cargo deny` posture) — a port, not a relaxation.
- U-GOV-009 (document + optionally arm the PreToolUse gate) -> **FEASIBLE** — security TIGHTENING only; must not weaken the deny-by-default semantics (`main.rs:8896-8919`). Reversible to current opt-in default.
- U-MEM-1 (ADR classifying/quarantining `memory.rs` as a bounded send-time cache) -> **FEASIBLE** — docs/ADR + doc-gate; no code deletion required. Pure governance.
- U-MEM-2 (reconcile weave's send-time recall with ICM, or document it as a local cache) -> **FEASIBILITY: QUALIFIED** — option (b) doc-contract (weave memory is an explicit local augmentation cache, durable recall stays ICM+handoff) is **feasible** and preferred. Option (a) wiring `build_context_prefix` to read ICM is **feasibility-constrained**: it adds a cross-binary coupling from `weave-core` to ICM that the transport plane should not own; pursue only if (a)'s coupling is itself ADR'd. Default to (b).
- U-TEST (A2A interop RED tests: to_a2a / from_a2a / JSON-RPC envelope) -> **FEASIBLE** — already authored and committed RED (`weave-core/tests/a2a_interop.rs`, 3 cases, tests-ran=3 all-RED-on-assertion); pure Rust over `serde_json`. These are the GREEN target for U-ARCH-2.

### Counts (weave, cycle 4)

- CLAIMS: **CONFIRMED 16 · QUALIFIED 4 · REFUTED 0 · INCONCLUSIVE 0** (20 verdicts).
  - QUALIFIED: ARCH-03 (29→~90 methods), ARCH-06 (78→72/76 tools), ARCH-12 (one SCC pair is real bounded recursion, not an artifact), PA-TOOLS (count).
  - No fully-REFUTED claim; the only refutation is *internal* to ARCH-12 (the `call_tool↔tool_meta` "resolver artifact" sub-claim is refuted — it is genuine guarded recursion).
- UPGRADES: **FEASIBLE 9 · FEASIBILITY-QUALIFIED 1 (U-MEM-2) · INFEASIBLE 0.** No-C / strict-upgrade gate held on every row (notably U-ARCH-2 A2A adapter is feasible because it rides existing pure-Rust serde_json + ed25519, additive/default-off).

### Routed back to analyst (corrections required before plan facts)

1. `Store` method count 29 → real surface ~90 (fix U-ARCH-1 acceptance).
2. MCP tool count 78 (and 70/74) → 72 arms / 76 catalog (fix U-ARCH-4 test + ARCH-06).
3. ARCH-12: re-label the `call_tool↔tool_meta` SCC as a real bounded recursion (self-target guard), not a resolver artifact; inject 5-cluster left INCONCLUSIVE pending a trace.
