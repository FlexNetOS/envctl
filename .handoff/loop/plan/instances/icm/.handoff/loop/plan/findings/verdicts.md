# Plan-verifier verdicts (THE GATE)

Default-skeptical, fail-closed. Only CONFIRMED/QUALIFIED + feasible rows flow to the architect.
Verdict format: `<ref> -> CONFIRMED | REFUTED (<counter>) | QUALIFIED (<cond>) | INCONCLUSIVE (<why>)`.

## icm

Date: 2026-06-27 · Verifier pass cycle 7 · target=icm · source verified at /home/drdave/Desktop/meta/icm
Build probe: `cargo build -p icm-store` -> **EXIT 0** (icm-core + rusqlite 0.34.0 bundled + icm-store
compiled clean, 5.98s). The C-floor and std facts below are real in THIS env.

### Material claims

- **Claim 1 (embedding-dim drift, HIGH)** -> **QUALIFIED**. The *documentation/const drift* is CONFIRMED;
  the *"latent insert-failure/silent-truncation bug"* hypothesis is **REFUTED**.
  - CONFIRMED facts: `fastembed_embedder.rs:36` `DEFAULT_MODEL = "intfloat/multilingual-e5-base"`, but the
    doc comment `:35` says "multilingual-e5-small (384d)"; `model_dimensions` `:58-65` maps
    `MultilingualE5Base => 768`; the fail-open `unwrap_or(384)` at `:93` fires ONLY when `resolve_model`
    fails to parse the name (not for the valid default). So `FastEmbedder::dimensions()` returns **768** for
    the default model — comment + the `DEFAULT_EMBEDDING_DIMS` const name (`lib.rs:18`, doc'd "used when no
    embedder is configured") are misleading.
  - REFUTED (counter-example): the vec0 table is created at the **runtime embedder dim**, not 384.
    `main.rs:1087-1093` → `embedding_dims = embedder.dimensions()` (768 with embeddings on) and only
    `unwrap_or(DEFAULT_EMBEDDING_DIMS=384)` with embeddings off; `create_vec_table` (schema.rs:17-37) emits
    `embedding float[{embedding_dims}]` at that dim. Storage is dimension-consistent; on a model/dim change
    the vec table is dropped + embeddings NULLed + self-healed (schema.rs:416-464, issue #200). Net: a
    cosmetic doc/const-naming defect, **not** a data-corruption bug. The "384d" must be corrected to
    "768d (e5-base); 384 = no-embeddings fallback" before any plan fact uses the number.

- **Claim 2 (C-floor unconditional)** -> **CONFIRMED**. `icm-store/Cargo.toml:7-9` always depends on
  `rusqlite` + `sqlite-vec` (no feature gate); workspace `Cargo.toml:19` `rusqlite { features=["bundled",
  "modern_sqlite"] }`; `store.rs:8` imports `rusqlite::ffi::sqlite3_auto_extension` and `:81-88`
  unconditionally registers `sqlite_vec::sqlite3_vec_init` via `Once`. ONNX/fastembed are the *optional*
  layer (`icm-core/src/lib.rs:4` `#[cfg(feature="embeddings")]`, `ICM_NO_EMBEDDINGS`/`--no-embeddings`
  at `main.rs:1080`) — they sit in icm-core, NOT icm-store. Build probe linked the bundled C path clean.

- **Claim 3 (no recency decay — the RED contract)** -> **CONFIRMED**. `apply_decay` (store.rs:1267-1311)
  SQL `:1293-1302` multiplies `weight` by a factor derived ONLY from `importance` (CASE high/low/else) and
  `MIN(access_count,5)` — it never references `last_accessed`/`created_at`. The 5 RED tests
  (`recency_decay_red.rs`) seed rows identical except `last_accessed` and drive `apply_decay(0.95)`; they
  fail for the RIGHT reason (time-blindness, not a compile/API error): e.g. CONTRACT 1 asserts
  `stale.weight < fresh.weight` but the flat factor yields equal weights; CONTRACT 4 asserts `w > 0.99`
  but flat 0.95 floors a Medium row at ~0.95. Genuine absent capability.

- **Claim 4 (no write-side RBAC)** -> **CONFIRMED**. `tools.rs:717-764` `call_tool(store, embedder, name,
  args, compact)` dispatches on `name` with no principal/auth/caller-identity parameter and no gate; all
  **31** `icm_*` tools (11 memory + 10 memoir + 1 learn + 3 feedback + 5 transcript + 1 wake_up) are
  reachable ungated, including destructive `icm_memory_forget` (`:728`) and `icm_memory_forget_topic`
  (`:729`). Any MCP caller can forget any topic.

- **Claim 5 (bundled-SQLite CVE blind spot)** -> **QUALIFIED**. Version facts CONFIRMED: `Cargo.lock`
  `libsqlite3-sys 0.32.0`, and the vendored amalgamation header defines `SQLITE_VERSION "3.49.1"` (read
  from `libsqlite3-sys-0.32*/sqlite3.h`); `rusqlite 0.34.0`, `sqlite-vec 0.1.6`, `fastembed 4.9.1`,
  `ort 2.0.0-rc.9`. QUALIFIED because: (a) cargo-audit operates on the crate graph and CANNOT see a CVE in
  C source vendored inside `libsqlite3-sys` — that "blind spot" is a real CI-coverage gap and stands; but
  (b) the *specific* "in-window CVE-2026-11822 (FTS5)" is a research-supplied advisory I cannot adjudicate
  from the repo — it does not become a plan fact on this gate. The mechanism (bundled C invisible to
  cargo-audit) is the confirmable, plan-worthy part.

- **Claim 6 (stale CLAUDE.md)** -> **CONFIRMED**. `icm/CLAUDE.md` is 26.2K and references the abandoned
  stack: "Turso" (lines 8,16,37,119), "libsql" (16,37,41), "1536" (96,134), with 42 French/persistence
  keyword hits — while the repo actually ships `rusqlite`+`sqlite-vec`+`fastembed` (e5-base 768d). Doc
  describes architecture the code no longer has = high-value doc-vs-code gap.

- **Claim 7 (XDG residency)** -> **CONFIRMED**. Runtime state lives under XDG/`ProjectDirs`, none
  meta-owned: data `main.rs:1042-1043` (`ProjectDirs`/`data_dir`), config `config.rs:4,292-301`
  (`ICM_CONFIG`, `XDG_`, `ProjectDirs`, `.config`), cache `fastembed_embedder.rs:14-26`
  (`ProjectDirs ... cache_dir().join("models")`, `~/.cache/icm/models`), `main.rs:5183-5185` (`XDG_`,
  `.cache`). Redirectable via `ICM_CONFIG` / `[store] path` / `--db` / `XDG_CACHE_HOME`. Confirmed.

- **Claim 8 (convergence verdicts)** -> **CONFIRMED (with one corrected sub-fact)**.
  - icm = canonical memory plane + PEER of handoff/git-kb, 3 disjoint corpora -> CONFIRMED: distinct
    corpora cited (`memory.rs:6-30` agent knowledge vs handoff redb event ledger vs git-kb AST graph);
    zero code coupling (grep icm in handoff = none, memory-vector §3).
  - NOT bound as data (no `memory` field in `handoff.context_capsule.v1`) -> CONFIRMED: capsule key list
    `[schema,project_name,role,plane,tier,northstar,next_command,source]`; `init_capsule`
    (`handoff/hf/src/main.rs:237-253`) writes exactly those — no memory pointer. Binding is CLAUDE.md/
    AGENTS.md convention + connected MCP, not a typed contract.
  - SIDECAR (C-floor vs no-C kernel) -> CONFIRMED: icm links bundled C (claim 2, build probe) while the
    union kernel is no-C (handoff redb/RVF, ADR-0001/0017) → icm must sit OUTSIDE as a sidecar.
  - icm vs prompt_hub distinct planes / duplicate substrate -> CONFIRMED, but the convergence-analysis
    sub-claim "two 384-dim spaces" is **QUALIFIED/corrected**: icm's real default is **768d** (e5-base),
    prompt_hub is 384d (MiniLM-L6) — they share neither model NOR dimensionality under the real default,
    which *strengthens* "not mergeable as-is" but refutes the "both 384" phrasing. Planes are distinct
    (agent memory vs intent store).
  - CORRECTION routed to analyst: `convergence-analysis-icm.md` A3 (line 18) and A5 (line 45-46) state the
    default embedding is "384-dim" — factually wrong; real default-with-embeddings is 768d. The downstream
    peer/sidecar/distinct-plane verdicts do NOT depend on the number and stand, but the figure must be
    fixed before becoming a plan fact.

- **Claim 9 (empirical build probe)** -> **CONFIRMED**. `cargo build -p icm-store` succeeds (EXIT 0) in
  this env; rusqlite 0.34.0 (bundled C) + icm-store compiled. C-floor + std facts are real here.

### Upgrade feasibility gate

All upgrade rows below pass the no-C trust-boundary check: every icm-internal upgrade stays in the C-bearing
**sidecar** (never linked into the no-C union kernel), and the one cross-repo upgrade touches handoff only
with a string/metadata pointer (no C deps imported). The gate is NOT weakened.

- UPGRADE *outcome-aware reinforcement + true time-decay* (memory-vector §1) -> **feasible**. `last_accessed`
  already exists on `Memory` and the RED suite drives it; touches `apply_decay`; serves axis:accuracy.
  This is the GREEN target the RED suite gates. feasible.
- UPGRADE *bump sqlite-vec ≥0.1.9 + switch blend to RRF* (memory-vector §2) -> **feasible**. Version bump +
  ranking change, additive, sidecar-internal; serves accuracy/quality. feasibility OK.
- UPGRADE *fix default-model doc/const drift* (memory-vector §2) -> **feasible** (trivial; comment+naming);
  directly closes claim 1's confirmed defect.
- UPGRADE *provenance-aware recall + admission policy* (memory-vector §4) -> **feasible**. `MemorySource`
  already persisted (`memory.rs:130-151`); add ranking + store-time policy hook; serves accuracy/governance.
- UPGRADE *configurable hybrid fusion weight 0.3/0.7* (convergence C) -> **feasible**. Default-preserving;
  `search_hybrid` (store.rs:1153-1212); serves accuracy.
- UPGRADE *add `memory` pointer block to `handoff.context_capsule.v1`* (convergence C) -> **feasible**.
  Feasibility gate: this touches the no-C kernel repo, but it adds an additive **string pointer**
  (endpoint/scope/recall-contract), NOT a link to icm's rusqlite/sqlite-vec/ort — so it does NOT breach the
  no-C trust boundary. feasible (PROPOSE; fleet-wide capsule schema = high blast).
- UPGRADE *ADR "memory/vector plane ownership"* (convergence C) -> **feasible** (doc-only).
- UPGRADE *fail-closed no-C BUILD gate on the union kernel* (convergence C) -> **feasible** and
  **STRENGTHENS** the invariant (cargo-deny/dep-graph assert kernel links none of rusqlite/sqlite-vec/ort/
  fastembed). Asserting the gate, not weakening it. feasible.
- UPGRADE *single embedding/recall contract across icm+prompt_hub+handoff* (convergence C) -> **feasible**
  as metadata descriptors (high blast, 3 repos, PROPOSE/long-horizon). feasibility OK.
- Hypothetical INFEASIBLE check (not proposed, recorded for the gate): consolidating the 3 vector engines by
  linking icm's sqlite-vec/ONNX *in-process into the no-C kernel* would be **infeasible** (breaches the no-C
  boundary). Correctly, no upgrade proposes this — consolidation is routed to handoff's pure-Rust RVF.

### Tally (icm)
- CONFIRMED: 6 (claims 2,3,4,6,7,9) + claim 8 cluster CONFIRMED
- QUALIFIED: 2 (claim 1 doc-drift-yes/data-bug-no; claim 5 mechanism-yes/specific-CVE-unadjudicable)
- REFUTED: 1 sub-hypothesis (claim 1 "latent insert-failure/truncation bug"); 1 corrected sub-fact
  (convergence "default 384d" → 768d)
- INCONCLUSIVE: 0
- Upgrades: 9 feasible / 0 infeasible (1 hypothetical recorded as infeasible to hold the gate)
