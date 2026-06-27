# prompt-architecture — icm (cycle 7)

TARGET: `icm` (persistent-memory organ) — repo `/home/drdave/Desktop/meta/icm`.
Frame: meta = ONE converging system; north-star @ $META_ROOT + handoff; goal = handoff + rusty-idd UNION.
icm reviewed AS ARCHITECTURE across four surfaces: instruction/prompt, tool grants, model lanes, governance + ADR.
Method: read-only source inspection; every CLAIM cites `file:line`. Verdicts: CONFIRMED unless marked.

---

## 1. prompt-architecture — instruction / prompt-injection surfaces

icm's primary prompt surface is the **wake-up pack** (`icm_wake_up` MCP tool + `recall-context`/`recall` CLI) that
emits LLM-system-prompt-injection text consumed at SessionStart, plus the **injected directive blocks** in
AGENTS.md / CLAUDE.md.

| # | CLAIM | Evidence | Verdict |
|---|-------|----------|---------|
| P1 | Wake-up builder is **well-typed at the build layer**: typed `WakeUpOptions`, a closed `Category` enum (6 variants) with fixed render order, deterministic ULID-tiebroken sort | `crates/icm-core/src/wake_up.rs:23-45` (opts), `:48-81` (Category), `:130-135` (sort) | CONFIRMED |
| P2 | The recall→prompt-injection **contract is an ad-hoc string render, not a versioned/typed envelope**: `render()`/`render_body()` assemble markdown by `push_str`; downstream agents must string-parse it; the ONLY stable contract token is `EMPTY_PACK_HEADER = "# ICM Wake-up (empty)"` | `wake_up.rs:313-389`, const at `:311` | CONFIRMED |
| P3 | Prompt-cache discipline is a deliberate, load-bearing architectural property: header kept byte-stable (no token-count prefix) so the Anthropic prompt-cache prefix match survives across runs; tie-break determinism documented as required for cache prefix | `wake_up.rs:127-135`, `:326-332` | CONFIRMED — a real (undocumented-as-ADR) design decision |
| P4 | `recall` output has a **typed JSON lane** (the real machine contract) alongside two string lanes: `Toon` (default, token-optimized), `Detail` (human), `Json` (serde) | `crates/icm-cli/src/recall_format.rs:15-37`, `render_json` `:125-141` | CONFIRMED |
| P5 | Summary sanitization hardens against prompt-structure injection: newlines flattened, runs collapsed so a stored memory cannot inject a fake `## header` into the pack | `wake_up.rs:287-305`, test `:638-653` | CONFIRMED — input-trust boundary handled |
| P6 | The injected **directive block** (canonical, idempotent via `<!-- icm:start/end -->` markers) tells agents ICM use is "MANDATORY" with 5 hard store-triggers; it speaks **CLI verbs** (`icm recall`/`icm store`) | `AGENTS.md` (whole file, icm:start/end block) | CONFIRMED |
| P7 | DRIFT: repo `CLAUDE.md` documents the **old, renamed** tool names (`icm_store`, `icm_recall`, `icm_forget`, `icm_consolidate`, `icm_list_topics`, `icm_stats`) — none of which are the actual MCP tool names (`icm_memory_*`). The prompt surface and the tool surface disagree | `CLAUDE.md:253-386` vs actual names §2 | CONFIRMED — stale instruction surface |

UPGRADE rows:

| ID | axis | upgrade | rationale | evidence | risk |
|----|------|---------|-----------|----------|------|
| U-P1 | accuracy | Define a **versioned wake-up envelope** (e.g. `{schema_version, project, sections[]}`) and make markdown a *renderer* of it, not the contract; expose envelope via `icm_wake_up` `format:"json"` | P2 — agents currently parse free markdown; any render change silently breaks consumers; mirrors the already-good `recall --json` lane (P4) | `wake_up.rs:313-389` | low (additive) |
| U-P2 | governance | Regenerate `CLAUDE.md` tool docs from the live `list_tools()` source so directive names never drift from dispatch | P7 — stale `icm_*` names will cause agents to call non-existent tools | `CLAUDE.md:253-386` | low |
| U-P3 | accuracy | Promote the prompt-cache byte-stability invariant (P3) into a property test + ADR so a future refactor can't silently regress the 100%→4% prefix-preservation cliff noted in-code | P3 | `wake_up.rs:326-332` | low |

---

## 2. tool grants (tools granted) — the 31-tool MCP capability surface

| # | CLAIM | Evidence | Verdict |
|---|-------|----------|---------|
| T1 | icm-mcp grants **exactly 31 tools** (`icm_memory_*` ×11, `icm_memoir_*` ×10, `icm_learn`, `icm_feedback_*` ×3, `icm_transcript_*` ×5, `icm_wake_up`); `icm_memory_embed_all` is conditionally added only when an embedder is present | dispatch table `crates/icm-mcp/src/tools.rs:724-762`; `embed_all` gate `:694-708`; count = 31 | CONFIRMED |
| T2 | **No capability gating / MCP annotations exist**: grep for `annotation\|readOnly\|destructive\|confirm\|consent\|auth\|dangerous` across `crates/icm-mcp/src/` returns **zero hits**. Read tools and **destructive mutators are granted on equal footing** | grep (empty) over `crates/icm-mcp/src/`; tool JSON `tools.rs:660-711` carries no `annotations` block | CONFIRMED |
| T3 | Destructive/mutating tools are open and ungated: `icm_memory_forget`, `icm_memory_forget_topic` (bulk delete), `icm_memory_update`, `icm_memory_consolidate`, `icm_memoir_refine`, `icm_memoir_link` dispatch straight to the store with no confirmation arg, no dry-run, no `destructiveHint` | `tools.rs:728-731`, `:741-745` | CONFIRMED |
| T4 | `call_tool` has no auth/identity/scope parameter — any connected host may call any of the 31 tools; the server is a thin trusted-local dispatcher | signature `tools.rs:717-723`; `crates/icm-mcp/src/server.rs` (189 lines, no auth) | CONFIRMED |
| T5 | The **same full 31-tool grant is injected identically into every agent host** — there is no per-host capability scoping; a read-only host gets the bulk-forget tool too | single `icm_server_entry` reused for all targets, `crates/icm-cli/src/main.rs:3433-3447` | CONFIRMED |
| T6 | Host-injector fan-out is **~15 hosts, not 6**: the JSON-`mcpServers` loop covers Claude Code, Claude Desktop, Cursor, Windsurf, VS Code, Gemini, Amp, Amazon Q, Cline, Roo Code, Kilo Code; Zed, Codex CLI, OpenCode, Copilot CLI, and Continue.dev (YAML) are injected separately | `main.rs:3383-3431` (loop list), `:3449-3508` (Zed/Codex/OpenCode/Copilot); `install_manifest.rs:60-76` enumerates the 7 injection shapes incl. `YamlContinue` | CONFIRMED — brief's "6 injectors" is a structural undercount |
| T7 | The grant surface is **auditable + reversible**: every injection is recorded (pre-mutation sha256 + bytes) in a versioned `install-manifest.json` so `icm uninstall` can revert exactly what was granted | `install_manifest.rs:26-162`, `entry_from_disk` `:141-162` | CONFIRMED — strong governance control |

UPGRADE rows:

| ID | axis | upgrade | rationale | evidence | risk |
|----|------|---------|-----------|----------|------|
| U-T1 | governance | Add MCP `annotations` (`readOnlyHint`/`destructiveHint`/`idempotentHint`) to every tool definition so hosts can surface confirm prompts for `forget*`/`consolidate`/`update` | T2/T3 — bulk-delete tools are indistinguishable from reads to the host | `tools.rs:660-711` | low (additive metadata) |
| U-T2 | governance | Offer a **read-only / curated grant profile** for injection (e.g. `icm init --profile=read-only` → recall/wake_up/stats only) so a host that only needs hydration isn't granted bulk-forget | T5 — uniform max grant violates least-privilege across 15 hosts | `main.rs:3433-3447` | medium |
| U-T3 | accuracy | Update install docs/brief to the real ~15-host matrix and the 7 injection shapes; treat host list as data, not prose | T6 | `main.rs:3383-3431` | low |

---

## 3. model lanes

icm has **two distinct model lanes** plus a non-LLM cloud lane (brief's open question resolved).

| # | CLAIM | Evidence | Verdict |
|---|-------|----------|---------|
| M1 | **Embedding lane** = local fastembed; default `intfloat/multilingual-e5-base`; resolver maps name→`(EmbeddingModel, dims)`; models cached under per-OS dirs; lazy `OnceLock` init | `crates/icm-core/src/fastembed_embedder.rs:36` (default), `:39-83` (resolver+dims), `:102-120` (lazy init) | CONFIRMED |
| M2 | DRIFT in the embedding lane: the default is `multilingual-e5-base` which `model_dimensions` maps to **768d**, but the doc comments assert "multilingual-e5-small (384d)" in two places. Canonical dim is ambiguous in-source | constant + comment `:35-36`; `MultilingualE5Base => 768` `:58-65`; `Self::new` comment "(multilingual-e5-small)" `:86` | CONFIRMED — comment vs code mismatch |
| M3 | Silent-fallback hazard: `with_model` falls back to **`384`** dims when a model name fails to parse (`unwrap_or(384)`), which can mismatch a base/large model's true dimensionality and corrupt vector search if a bad name is ever passed | `fastembed_embedder.rs:93` | CONFIRMED — fail-open, not fail-closed |
| M4 | **Summarization lane** = a *delegated* LLM lane: icm shells out to whichever host agent CLI is present rather than calling a provider SDK. Defaults: `claude-haiku-4-5` (Claude provider), `gpt-5-mini` (Codex provider); provider auto-detected; a third default `claude-sonnet-4-5` for a CLI command | `crates/icm-cli/src/summarizer.rs:215` (haiku), `:231` (gpt-5-mini), `:74-115` (detect_provider), `crates/icm-cli/src/main.rs:496` (sonnet default) | CONFIRMED |
| M5 | Self-recursion guard in the summarization lane: when running *inside* Claude Code, the Claude provider is disabled (returns `None`) to avoid an agent summarizing via itself | `summarizer.rs:76-77` | CONFIRMED — thoughtful coupling control |
| M6 | **Cloud sync is NOT an LLM lane** (brief's open question): `cloud.rs` is plain memory CRUD (push/pull) over HTTP (`ureq`) to `{endpoint}/api/icm/memories` with bearer-token auth to `cloud.rtk-ai.app`; no completion/chat/inference call | `crates/icm-cli/src/cloud.rs:309-368` (CRUD), `:489` (endpoint); grep for `chat/completions\|inference` empty | CONFIRMED |
| M7 | The model lanes are **not centrally pinned**: embedding default and the three summarizer model strings are hardcoded literals scattered across 3 files, not a single canonical model-policy config | `fastembed_embedder.rs:36`, `summarizer.rs:215,231`, `main.rs:496` | CONFIRMED |

UPGRADE rows:

| ID | axis | upgrade | rationale | evidence | risk |
|----|------|---------|-----------|----------|------|
| U-M1 | accuracy | Fix the embed-lane drift: make `multilingual-e5-base`/768d the single sourced truth (or switch default to the documented `-small`/384d), delete the contradictory comments, and assert stored-vector dims == embedder dims at open | M2/M3 — a dim mismatch silently breaks hybrid search | `fastembed_embedder.rs:35-93` | low |
| U-M2 | governance | Replace the `384` parse-failure fallback with a hard error (fail-closed) so an unknown model name never silently downgrades dimensionality | M3 | `:93` | low |
| U-M3 | governance | Centralize all model-lane selections (embed model + summarizer models) into one pinned model-policy surface (config + ADR) instead of scattered literals | M7 | M4/M7 cites | medium |

---

## 4. ADR

| # | CLAIM | Evidence | Verdict |
|---|-------|----------|---------|
| A1 | icm has **no ADR set**: no `docs/adr*`, no `*adr*` file anywhere in-tree (`docs/` = architecture/features/guide/integrations/product only) | `find . -iname '*adr*'` → empty; `ls docs/` | CONFIRMED |
| A2 | Several genuine, load-bearing architecture decisions are **undocumented as ADRs**: the recall→prompt-injection envelope contract, the prompt-cache byte-stability invariant, the open/ungated 31-tool grant model, the embedding-model lane + dim policy, the delegated (host-CLI) summarization lane, and the icm↔handoff↔rusty-idd memory-ownership boundary | P2/P3, T2, M1/M4 above; boundary: icm is the only `icm_*` MCP organ in the union | CONFIRMED |

### ADR-CANDIDATES

1. **ADR — recall→prompt-injection envelope contract.** Decide whether the wake-up/recall output is a versioned typed envelope (rendered to markdown/TOON) or remains an ad-hoc string. Captures the prompt-cache byte-stability invariant as a first-class constraint. (Drivers: P2, P3, U-P1.)
2. **ADR — MCP tool-grant model & least-privilege.** Record the decision to grant all 31 tools ungated to ~15 hosts, vs. annotation-gated / profile-scoped grants. Must state the trust boundary (trusted-local, no auth in `call_tool`). (Drivers: T2–T7, U-T1/U-T2.)
3. **ADR — embedding-model lane policy.** Pin the canonical embedding model + dimensionality, the fail-closed-on-unknown-model rule, and the migration story when the default changes (re-embed). Resolves the base/768 vs small/384 drift. (Drivers: M1–M3, U-M1/U-M2.)
4. **ADR — summarization lane: delegate-to-host-CLI.** Record that icm intentionally has no first-party LLM SDK and instead shells out to the host agent CLI (with the in-Claude self-recursion guard), and pin the default models. (Drivers: M4, M5, M7.)
5. **ADR — icm↔handoff↔rusty-idd memory boundary (UNION-relevant).** Define icm as the persistent-memory organ of the converging system: who owns durable cross-session memory vs. handoff's witnessed ledger vs. rusty-idd's spec/control-plane state, and the non-overlap contract. This is the highest-leverage ADR for the union goal. (Driver: A2 + frame.)

### NO-ADR rationale (surfaces that do NOT warrant an ADR)
- **install-manifest schema / uninstall reversibility** — already self-documenting, versioned (`schema_version`), and tested; an ADR would add no constraint. `install_manifest.rs:24-162`. N/A — already a recorded, enforced contract.
- **recall output formats (TOON/Detail/JSON)** — a settled, tested presentation choice with a typed JSON lane; no cross-cutting architectural tension. N/A — local, low-blast-radius.
- **cloud sync transport** — plain authenticated CRUD, no model/architecture decision of union significance. N/A — no LLM lane, conventional client.

---

## Summary verdict
icm's prompt architecture is **stronger at the build layer than at the contract layer**: typed builders, deterministic
prompt-cache-aware rendering, and an auditable/reversible install manifest are genuine strengths, but the agent-facing
contracts are under-specified — the wake-up pack is free-form markdown (no versioned envelope), the 31-tool grant is
ungated and injected at max privilege into ~15 hosts, the embedding lane carries a base/768-vs-small/384 drift with a
fail-open fallback, and there are zero ADRs recording any of it. For the handoff + rusty-idd UNION, the single
highest-leverage gap is the **unwritten icm↔handoff↔rusty-idd memory-ownership boundary** (ADR-CANDIDATE 5).
Confidence: HIGH (all claims cite source read this cycle; no INCONCLUSIVE rows).
