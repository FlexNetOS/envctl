# grit — decision-grade plan (cycle 5)

- Target: `grit` v0.4.0 — single Cargo binary crate, `/home/drdave/Desktop/meta/grit`.
- Built only from CONFIRMED/QUALIFIED + feasibility-passed rows in `findings/verdicts.md`.
- Graph: `graph/grit.graph.md` / `grit.metrics.json` (305 symbols, 548 intra-src edges, 74 `pub`).
- Currency: `research/grit.trends.md` (researched 2026-06-27, rolling 90-day window).
- Docs only. No grit production code is touched by this plan (owner-wall: grit's tree is not written).
- Automation legend (envctl `docs/runbook/DIAGRAMS.md`): `[A]` automated · `[A*]` elevated/sudo ·
  `[P]` preview/dry-run · `[H]` human-gated · `[!!]` supervised/critical.

---

## Verdict

**grit is an advisory symbol-LOCK + git-worktree coordinator — NOT a symbol-level reconciliation
engine — and is UNFIT *as-is* to BE the union merge engine for union step 2.** Its only content
merge is line-level `git merge --no-ff` after a rebase (src/git/mod.rs:221-253); its "no conflicts by
design" property comes from *partitioning* edits at symbol granularity, not from reconciling them. A
per-symbol content hash IS computed (src/parser/mod.rs:329 via `hash_str`, :420-424) and persisted
(src/db/mod.rs:156-161,171) but **is never read in any production path** — grep of `src/db` shows zero
`SELECT`/compare of the column outside `#[cfg(test)]`. grit computes a dedup primitive it never uses.

**The recommendation is convergence, not adoption-as-is:** make grit the *coordination substrate
AROUND* a separate symbol-level dedup/reconciler. The two primitives union step 2 needs already exist
in grit — deterministic `Symbol.hash` (proven by `test_symbol_hash_deterministic`, parser/mod.rs:908)
and `LockStore::try_lock` (db/lock_store.rs:29). The only missing piece is the cross-source
composition: a `Reconcile{a,b}` command that points at two roots, joins symbols by id, partitions by
hash into identical(auto-merge)/divergent(conflict), and routes conflicts to the locker. That gap is
**RED-proven** — `cargo test --test union_dedup_contract` runs 3 tests, all RED with the identical
correct reason `error: unrecognized subcommand 'reconcile'` (clap rejects the absent command; the
tests compile and run — capability-absence, not a compile error). This is the Feature-Forge GREEN
target.

**No-C routing (load-bearing, decides where the engine can live).** grit's own substrate is NOT no-C:
`rusqlite` uses `bundled` SQLite (C) and all 14 tree-sitter grammars are C. Therefore grit **cannot**
be the *in-trust-boundary* (no-C) union engine — that framing is REFUTED on feasibility. Route the
upgrade two explicit ways:

- **Route A — pure-Rust dedup/reconciler INSIDE handoff's no-C trust boundary**, using grit only for
  out-of-boundary coordination (locks/worktrees/room). This is the preferred long-run shape and is the
  draft-ADR candidate (`reports/adr-draft-grit-reconciler.md`).
- **Route B — `grit reconcile` built additively IN grit as an OUT-of-boundary coordination/dedup
  tool.** Feasible today (additive Rust, reuses `scan_all` + `Symbol.hash` + `try_lock`); it flips the
  3 RED tests but the reconciler then lives outside the trust boundary (grit's C deps stay out).

**Live convergence evidence (cite in framing).** This cycle a `weave` A2A round-trip occurred: the
envctl session asked rusty-idd (via weave) to verify the front-door plan, rusty-idd replied with
corrections, envctl folded them and shipped prompt_hub PR #182. grit's `room` (room.sock pub/sub of
`Claimed`/`Released`/`AgentDone`, src/room/mod.rs:22-154) is the **code-contention** coordination
plane; weave is the complementary **cross-session message** plane. The named upgrade is a grit→weave
room-event bridge (see Rules/policy).

---

## ASCII architecture

### A. Module call graph — layered DAG, no back-edges (Source: graph/grit.graph.md §Module call graph)

```
                          ┌──────────────┐
                          │  main.rs     │   main() → cli::run()           [A]
                          └──────┬───────┘
                                 │ x1
                          ┌──────▼────────────────────────────────┐
                          │  cli/mod.rs   (dispatch hub, god-mod)  │   run() ─match Command→ cmd_*
                          │  1702 LOC · out-deg run=23             │   [H] done/session pr = [!!]
                          └─┬───────┬───────┬────────┬───────┬─────┘
                  cli→db x58│  x23  │  x12  │  x10   │  x6   │
                            ▼       ▼       ▼        ▼       ▼
                     ┌──────────┐┌──────┐┌──────┐┌────────┐┌──────────┐
                     │ db/      ││ git/ ││room/ ││parser/ ││ config   │
                     │ Database ││GitRpo││ Room ││ Symbol ││ GritConfig│
                     │+LockStore││wktree││Notif ││ Index  ││ backend  │
                     └────┬─────┘└──────┘└──────┘└────────┘└────▲─────┘
                          │ db→config x1 ──────────────────────────┘ x6
```

Edge direction is strictly downward (`main → cli → {db, git, room, parser, config}`, plus
`db → config`). No back-edge exists. **Layering violations: 0; true architectural cycles: 0** (the
`Database::open ↔ SqliteLockStore::open` SCC is a git-kb ambiguous-name resolver artifact — both call
`rusqlite::Connection::open`, neither calls the other). Source: verdicts.md claim-5; graph/grit.graph.md §Cycles.

### B. LockStore polymorphism — the convergence-relevant core (Source: graph/grit.graph.md §LockStore)

```
                  trait LockStore (db/lock_store.rs:28)
                  try_lock · release · release_all · all_locks
                  locks_for_agent · gc_expired_locks · refresh_ttl
                        ▲                ▲                ▲
              impl ─────┘       impl ────┘       impl ────┘
        SqliteLockStore      S3LockStore        AzureLockStore
        sqlite_store.rs:47   s3_store.rs:520    azure_store.rs:383
        local .grit/         If-None-Match:*    Blob + Event Grid
        registry.db [A]      cond. PUT [A]      cond. write [A]
                  selected at runtime by GritConfig → resolve_lock_store (cli/mod.rs, in-deg 12)
                  catch-all `_ => SQLite` (cli/mod.rs:407) = SILENT DOWNGRADE on typo  [!!]
```

Blast note: changing the 7-method trait forces simultaneous edits to **all 3** backends (high,
fan-wide). Source: graph/grit.graph.md §Blast-radius; verdicts.md claim-3a.

### C. Proposed reconciler — OUTSIDE grit's C trust boundary (Route A preferred / Route B inline)

```
   ┌───────────────────── handoff NO-C TRUST BOUNDARY ──────────────────────┐
   │                                                                          │
   │   reconciler (PURE RUST, Route A)            stable Symbol.hash (no-C)   │
   │   join-by-id → partition-by-hash:            replace DefaultHasher       │
   │     identical (auto-merge) | divergent       with fixed algo  [A]        │
   │     | only_in_a | only_in_b                                              │
   │            │  conflicts → request lock                                   │
   └────────────┼─────────────────────────────────────────────────────────────┘
                │  (coordination call only — no C crosses the line)
                ▼
   ┌──────────────────────── grit (C substrate: OUTSIDE boundary) ───────────┐
   │  LockStore::try_lock (lock the divergent symbol)   [A]                    │
   │  git worktree per agent · merge.lock serialize     [H] merge = [!!]      │
   │  room.sock pub/sub  ──bridge──▶  weave A2A nudge (proposed)  [A]          │
   │  rusqlite(bundled C) · tree-sitter ×14 (C)  ← why grit stays OUT of TB    │
   └──────────────────────────────────────────────────────────────────────────┘
```

Route B builds the reconciler box *inside* grit instead (additive `Reconcile` clap variant +
`cmd_reconcile`); it flips the 3 RED tests but inherits grit's C substrate, so the engine remains an
out-of-boundary tool. Source: verdicts.md FEASIBILITY union-engine direction; test-strategy-grit.md §FF test-build spec.

---

## Sequenced upgrade roadmap

Ordered by graph centrality + blast-radius: high-centrality / contained-blast wins first; high-blast
changes are sequenced behind their prerequisites. Hottest modules (most depended-on) are **`parser/`**
(`SymbolIndex.new` in-deg 39, `find_sym`, `scan_all` 23) and the **`LockStore` trait** (changing it
forces all 3 backends). Tag = axis (quality / speed / accuracy / governance). Risk-tier = APPLY (low,
self-contained) / PROPOSE (touches a contract or central helper) / ADR (genuine architecture decision).

| # | Upgrade | Axis | Tier | Graph-grounded rationale (centrality / blast) | Evidence |
|---|---------|------|------|-----------------------------------------------|----------|
| 0a | Remove/relocate stray `.worktrees/Cargo.toml` (or add empty `[workspace]` to grit's manifest) | quality | APPLY | **Prerequisite for everything** — without it grit's standalone `cargo build`/CI/RED→GREEN fails (phantom-workspace wall). Zero src blast. | verdicts.md claim-7 |
| 0b | Stable version-independent `Symbol.hash` (replace `DefaultHasher`/SipHash with a fixed pure-Rust algo) | accuracy | APPLY | Single function in the **hottest module** (parser/mod.rs:420-424) but contained blast (init is the only writer today). **Prerequisite** for any persisted dedup key — `DefaultHasher` is not stable across toolchains. | verdicts.md FEASIBILITY stable-hash; architecture-grit.md U-hash |
| 1 | **`grit reconcile {a,b}` union-step-2 capability** (additive `Reconcile` clap variant + dispatch + `cmd_reconcile`; engine joins two roots by id, partitions by `Symbol.hash`; `--lock-conflicts` routes divergent symbols through `try_lock`) | accuracy | PROPOSE / **FF GREEN** | Reuses the two hottest leaves additively — `scan_all` (in-deg 23) + `try_lock` (in-deg 19) — so blast is contained (additive command, no signature changes). This is the headline value and the RED→GREEN target. | test-strategy-grit.md §FF spec; verdicts.md claim-2; parser/mod.rs:329 |
| 2 | Atomic multi-symbol `claim` — release the granted subset on terminal `bail!` (mirror the retry-path release at :626-628) | accuracy | APPLY | Single command (cli/mod.rs:541-616); contained blast but protects lock-ledger integrity a union depends on. Today a 2-symbol claim with 1 blocked leaks the granted lock. | verdicts.md claim-4b |
| 3 | Replace `backend: String` + catch-all `_ => SQLite` with `enum Backend{Local,S3,Azure}` deny-unknown (a typo becomes a hard error, not a silent local downgrade) | accuracy | PROPOSE | Touches config parse + `resolve_lock_store` (in-deg 12); medium blast. Fixes a strict-upgrade / No-Downgrade violation. | verdicts.md claim-3a; governance-config-grit.md gov-008 |
| 4 | Route lock-availability READS through the active `LockStore` (unify cloud locks into the queried view) | accuracy | PROPOSE | Touches central read helpers `available_symbols_in_files`/`list_symbols` (db/mod.rs:179-227) used by 3+ verbs — medium-high blast. Under S3/Azure, `symbols`/`plan`/`assign` currently report stale/empty lock state (reads `LEFT JOIN` the local table the cloud never populates). | verdicts.md claim-4a |
| 5 | Disambiguate symbol ids (add `kind`/positional discriminator so `struct Point` and `impl Point` don't collapse) | accuracy | ADR | `id` is the system-wide PK across locks/deps/queue/CLI args — **highest blast**, requires a schema/contract migration. Sequenced behind #1 (reconcile needs a precise per-symbol ledger). | architecture-grit.md U-id; verdicts.md FEASIBILITY disambiguate-ids |
| 6 | Caller-identity / token binding on lock ownership (`release`/`done` cannot act on another agent's locks) | governance | ADR | Changes the `LockStore` trust model across **all 3 backends** — high blast, security-relevant. Defer behind an ADR. | prompt-architecture-grit.md PA-U4 |
| 7 | Parse each file once in `scan_with_deps` (reuse the `scan_all` tree) + single tree glob dispatched by extension | speed | APPLY | Init-internal, low-medium blast; differential-output acceptance (identical symbols/deps). Init double-parses every function-bearing file today. | architecture-grit.md U-speed; verdicts.md FEASIBILITY parse-once |
| 8 | Call-edge scope resolution (file/import scope so `--with-deps` stops over-locking homonyms) | accuracy | PROPOSE | Affects the deps table + `--with-deps` claims; name-only edges link every `validate` in the repo. | architecture-grit.md U-callscope; verdicts.md FEASIBILITY call-edge-scope |
| 9 | Retire the socket `Room` for the working `--poll` path, OR run a real decoupled `grit serve` daemon | quality | PROPOSE | room module + watch/init only — low blast. The notify server dies with the `grit init` process, so socket `watch` never works (dead-on-arrival surface). | verdicts.md claim-4c |
| 10 | grit→`weave` A2A bridge: forward room `Released`/`AgentDone` events as weave nudges to the next queued agent's session | quality | PROPOSE | Additive, fail-open like the existing `notify`; low blast. Connects the code-contention plane to the cross-session plane (live convergence evidence). | rules-policy-org-grit.md rpo-A |
| 11 | `--json` output mode (or an MCP server) so agents consume a stable contract instead of scraping colored prose | accuracy | ADR | Every verb's print path in cli/mod.rs (medium-large surface); introduces a public machine interface. | prompt-architecture-grit.md PA-U2/ADR-cand-1 |
| 12 | Remove dead code (`NameExtractor::ChildKind` declared+matched but never constructed; audit `get_deps`/`count_deps` `#[allow(dead_code)]`) | quality | APPLY | Low blast; confirmed dead. | verdicts.md FEASIBILITY remove-dead-code |

Governance / currency items (docs/CI/config-additive, no trust-boundary conflict) are sequenced in the
Governance and Tool-evaluation sections below; each STRENGTHENS a gate, none weakens one.

---

## Tool-evaluation

What the graph shows grit imports/links (Cargo.toml resolved in Cargo.lock 2026-06-27), cross-referenced
with the researcher's 90-day currency + advisories. Recommendation per tool with cited date.
Source: research/grit.trends.md §Tool-currency.

| Crate (pin → resolved) | Latest | Recommendation | Reason (cited / dated) |
|---|---|---|---|
| `azure_storage_blobs` 0.21 | **`azure_storage_blob` 1.0.0** (renamed, GA **2026-05-14**) | **UPGRADE — highest priority** | Legacy crate "fully deprecated", source moved to `azure-sdk-for-rust/tree/legacy`, "no plans to update" — will not receive fixes (C10, accessed 2026-06-27). |
| `azure_core` / `azure_storage` 0.21 | 1.x GA line | **UPGRADE** | Pre-GA legacy lineage; fold the umbrella `azure_storage` into the GA blob crate (C10, 2026-05-14). |
| `rusqlite` 0.31 (bundled) | **0.40.0** (≈2026-06-17) | **UPGRADE (staged)** | 9 minor releases behind; multiple breaking minors → stage it. No RustSec advisory found (C9). |
| `tree-sitter` 0.25 | **0.26.8** (2026-03-31) | **UPGRADE** | 1 minor behind; re-pin the 14 grammar crates to match (C11). |
| `colored` 2.x | **3.0.0** | **HOLD (optional)** | 1 major behind, no advisory; cosmetic. Pair with control-char sanitization before TTY printing (RUSTSEC-2024-0364 pattern, C15). |
| `aws-sdk-s3` / `aws-config` 1.x | current 1.x | **HOLD** | Current; keep tracking 1.x (C9 table). |
| `tokio`/`serde`/`serde_json`/`anyhow`/`chrono`/`clap`/`glob`/`futures`/`tempfile`/`urlencoding` | current | **HOLD** | All current as of 2026-06-27; no advisories. |

**Toolchain advisory (affects the build, QUALIFIED — web-sourced, verify GA/CVE specifics at apply-time):**
pin the union build toolchain to **Rust ≥ 1.96.0** (released 2026-05-28) — fixes Cargo CVE-2026-5223 /
CVE-2026-5222 (symlink handling in third-party-registry tarballs; vendored/mirror/private-registry
flows exposed, crates.io unaffected). C13. grit has **no MSRV and no toolchain pin** today
(governance gov-005), so CI floats on `@stable`.

**Currency verdict:** grit's general-purpose deps are current; the two real actions are the deprecated
Azure lineage (migrate first) and the 9-minor-stale rusqlite (stage). Note: rusqlite-bundled + the C
grammars are exactly why grit stays OUTSIDE the no-C trust boundary — an Azure migration does not
change that.

---

## Governance

Source: governance-config-grit.md. grit has **no agent control plane at all** — no `.claude/rules`, no
`settings.json`, no `.handoff` policy, no agent-guard — despite being a registered meta member tagged
`orchestration` that ships cloud-credential code (gov-001). A missing expected surface is a finding,
not a pass.

- **AGENTS.md is an ICM-only stub** (26 lines): no mission, no hard rules, no fail-closed law, no
  destructive-command guard (gov-002). **CLAUDE.md** carries ~130 lines of irrelevant RTK boilerplate
  (pnpm/npm/vitest/tsc/next/prisma/docker/kubectl) for a single-crate Rust repo, ICM block duplicated
  verbatim across both files (gov-003) — instruction noise burned every session.
- **Destructive-command governance is not propagated** from parent meta into grit: an agent with grit
  as cwd inherits no guard against `git reset --hard`/`rm -rf`/force-push — a fail-OPEN drift across
  the governance boundary (gov-011).
- **No MSRV / toolchain pin** (gov-005); **clippy omits `--all-targets`** so test-code lints are
  ungated (gov-006); **no `cargo audit`/`deny`** despite a large cloud-SDK surface handling access keys
  (gov-007).
- **Backend catch-all silent downgrade** (gov-008) — see roadmap #3.
- **Azure access keys written plaintext** to `.grit/config.json` at default perms (gov-009) — see Risk
  policy; the S3 path correctly uses env, so the handling is asymmetric.
- **Release workflows hardcode `rtk-ai/grit` / `rtk-ai/homebrew-tap`** while the meta member is
  `FlexNetOS/grit` (a fork): a fork-side release targets the wrong org (gov-010).

Governance upgrades (all feasible, docs/CI/config-additive, each STRENGTHENS a gate): write AGENTS.md
Mission/Hard-Rules/Fail-closed + `.claude/rules/destructive-commands.md` [H]; trim RTK noise + dedup
ICM in CLAUDE.md [A]; add MSRV + `rust-toolchain.toml` [A]; clippy `--all-targets` + `cargo audit`/`deny`
job [A]; `enum Backend` deny-unknown [P]; parameterize release to `${{ github.repository }}` [P].
Owner-wall: confirm whether grit should carry a full harness or stay a thin member (the destructive
guard mirror is the minimum either way).

## Filesystem layout

Source: filesystem-layout-grit.md. grit's repo-local `.grit/` model (registry.db, worktrees,
room.sock, merge.lock, config.json — self-ignored at init, cli/mod.rs:430-444) is the correct
`.git`-style per-working-tree convention; **no FHS/XDG system or `$HOME` writes** are authored. Drift to
fix:

- `.worktrees/` at repo root — **untracked AND un-ignored**, empty, no owner (likely a meta/harness
  artifact leaked in; grit's own worktrees live under `.grit/worktrees/`). Ignore-with-attribution or
  remove (FL-1). Note: this is distinct from the stray `.worktrees/Cargo.toml` phantom-workspace wall
  (roadmap #0a) but adjacent.
- `tests/` holds `*.sh` + `gen_graph.py` in Cargo's reserved integration-test surface — move shell
  harnesses to `scripts/test/` and reserve `tests/*.rs` (FL-3); the new `tests/union_dedup_contract.rs`
  is the first real integration test.
- `examples/*.sh` in Cargo's `examples/` Rust-target dir → `docs/examples/` (FL-4, minor).
- `.fastembed_cache/` ignore rule with no producer in `Cargo.toml`/`src` — stale rule, remove (FL-2).
- `assets/benchmark.pdf` + `bench_data.json` committed generated artifacts → release assets (FL-6).
- `.grit/config.json` Azure key plaintext → secret-residency drift (FL-5; see Risk policy).

## Memory/vector

Source: memory-vector-intelligence-grit.md. grit ships **no ICM, no embeddings/RAG/vector store, no
handoff witnessed-ledger** — legitimate for a merge/lock substrate. But it owns its OWN persistent
code-intelligence store (`.grit/registry.db`: tree-sitter symbol index + `deps` call-graph + locks)
that functionally **overlaps git-kb** (duplication/convergence candidate, not a memory gap), and its
merge/claim decisions are neither recall-informed nor durably witnessed:

- Coordination events are **ephemeral and unwitnessed** — `Room::notify` drops the event if no socket
  peer; `locks` rows are deleted on release; no `RoomEvent` is persisted (CLAIM-M2). A merge leaves no
  durable audit row beyond the git commit.
- Symbol search is lexical `LIKE`, not vector (CLAIM-V1) — adequate (a lock needs exact identity).
- The index is **stale between `grit init` runs** — no file-watcher, unlike the git-kb daemon
  (CLAIM-V2); partially mitigated by the actionable "not in the registry" refusal on an unindexed
  claim (sqlite_store.rs:14-18).

Upgrades: emit a durable append-only decision ledger for claim/release/done (feeds the handoff ledger,
UPGRADE-1); export grit's symbol+deps graph in a git-kb-compatible shape to avoid two divergent graphs
(UPGRADE-2); index-freshness guard comparing working-tree hash vs `symbols.hash` (UPGRADE-4). The
recall-informed-merge hook (UPGRADE-3) must fail-open so grit stays standalone.

## Auto-research

The cycle's auto-research cadence (`findings/autoresearch-grit.md`, `research/sources-grit.jsonl`)
refreshed the code graph snapshot (`graph/grit.diff.md` vs previous), re-ran the 90-day web recency
gate (window since 2026-03-29), and updated the source ledger. Contradiction/stale-evidence checks:
the trend headline (entity/symbol-level structural merge now beats line-based by a measured margin —
`weave` 31/31 vs git 15/31, v0.3.x dated 2026-06-05, C1) externally **validates grit's symbol-level
thesis** for a 95%-shared union; flagged as single-vendor self-reported (corroborated for *approach*
via Mergiraf tree-sitter+GumTree C2 and the jj-vcs two-tier discussion C3, treated as signal not
audited fact). The actionable field inputs to carry into the build: object-store-native conditional
writes / leases are the standard distributed-lock primitive (S3 `If-None-Match` C5, Azure Lease Blob
C6 — grit already uses conditional PUT), and a **pre-merge dry-run conflict probe** (`git merge-tree`
spirit, AgenticFlict dataset C8) should be added so the union loop detects a contested symbol before
committing.

## Rules/policy

Source: rules-policy-org-grit.md. grit is the **arbiter** (not commander) of the agent org chart — it
gates writes (claim → work-in-isolated-worktree → done), it does not issue work; agents self-claim.
Owner standing rules hold: **Upgrade Only / No Downgrades** (pre-1.0 additive minor bumps,
CHANGELOG.md:3-9), fail-closed merge (a dirty main worktree makes `grit done` *refuse* to merge,
README.md:272-275), CI-gated commit/push/PR discipline with `develop→master` the only release path.

```
        OWNER (human — supervised/risk boundary only)  [H]
                          │
                  commander / orchestrator (spawns N parallel agents)
        ┌─────────────────┼─────────────────┐
   specialist         specialist        specialist     ← background lanes
   (.grit/worktrees/agent-1 … agent-N)
        ▼                 ▼                 ▼
   ┌──────────────────────────────────────────────┐
   │  grit lock substrate  (CONTENTION CONTROL)    │  ← arbiter, NOT commander
   │  AST symbol locks · queue · merge.lock(serial)│
   └──────────────────────────────────────────────┘
        │ room.sock pub/sub  ──proposed bridge──▶  weave A2A (cross-session) [A]
        ▼
   watchers / verifier / continuity (grit watch / --poll)
```

grit's `room` (code-contention plane) and `weave`'s A2A session mesh (cross-session message plane) are
distinct, complementary, both tagged `orchestration` in `.meta.yaml:184-200`; grit's event emission is
non-blocking/fail-open on the transport. The named upgrade is the grit→weave bridge (roadmap #10,
rpo-A) so a released symbol pings the next queued agent's *session*, not just its socket watcher — the
live convergence evidence (weave A2A round-trip → prompt_hub PR #182) is the proof this plane is real.
Replace-human-bottleneck: conflict resolution, free-symbol pick (`assign`), blocked-claim retry
(`--queue`), and stale-lock expiry (`gc`/`heartbeat`) are already automated; credential rotation and
force-push/hard-reset stay **owner-only** at the risk boundary.

## Distributed compute

Source: distributed-compute-grit.md. grit is a Rust, **Unix-only**, git-CLI-dependent merge-lock
*coordinator* (not a compute scheduler); its only distribution seam is the `LockStore` (local SQLite
WAL | S3/R2/MinIO | Azure Blob + Event Grid), using TTL-leased conditional-PUT locks as the
cross-machine source of truth. Hardware reach: **workstation / local-server / cloud-vendor** primary;
full-Linux **Raspberry Pi** feasible but must build from source (no ARM64 binary ships — `openssl-sys`
breaks the cross-build, release.yml:41-43); **Pi Zero** observer-only (heavy deps: bundled C SQLite +
14 grammars + tokio + 2 cloud SDKs); **mobile / AI glasses / wearables / ESP32** are N/A (no
git/worktree/std/tokio-HTTPS host; ESP32 is no_std-impossible). No `Lua`/`Luau` plane exists and none
is warranted yet — policy is fixed Rust; if ever wanted it must be a pure-Rust Luau (no-C per envctl),
filed as a gated speculative upgrade. Adjacent upgrades: swap `openssl-sys`→`rustls` to unlock ARM64
(U1); a feature-gated lock-client-only edge profile without tree-sitter (U2); document the MinIO
GET-then-PUT TOCTOU tier (U4).

## Test Strategy

Source: test-strategy-grit.md. grit is **unit-test-rich** (77 inline `#[cfg(test)]` tests) but had
**zero Rust integration tests** before this cycle (`tests/` held only shell/python harnesses).

- **Current coverage (by call-graph reachability):** parser symbol/dep extraction well-covered (25
  tests across 13 languages); `Database` CRUD/queue/deps covered (19); `SqliteLockStore` lock
  semantics incl. the cross-process `BEGIN IMMEDIATE` race covered (15, esp.
  `test_concurrent_access_separate_connections`); CLI **command bodies almost entirely uncovered** (11
  tests hit only `validate_identifier` + `is_entry_expired_local` + one `cmd_claim` path).
- **Ranked coverage gaps (contract-bearing symbols with no test caller):** (1) S3/Azure `LockStore`
  backends — **zero tests** on two runtime-selected contract impls; (2) the cross-backend read-skew bug
  (roadmap #4) has **zero coverage**; (3) `promote_queued` FIFO promotion (a hotspot called by
  `cmd_release` + `cmd_done`, hardcoded 600s TTL, no promoted-agent worktree); (4) the room
  notification server; (5) no init→claim→release→done binary e2e; (6) fail-closed FK-translation
  message on an unindexed claim.
- **Designed suite:** the authored additive RED suite `tests/union_dedup_contract.rs` (3 tests, binary-
  driven via `CARGO_BIN_EXE_grit`) pins the union-step-2 contract — reconcile subcommand exists; over
  two near-identical crates the 9 byte-identical helpers partition auto-mergeable and `checksum`
  reports a conflict; `--lock-conflicts` emits divergent id `src/core.rs::checksum`. Plus designed
  (not-yet-authored) rows: `promote_queued` FIFO tests, fail-closed FK message, init→done e2e, and
  feature-gated/mocked S3+Azure contract tests. Differential/golden: promote the two in-test crate
  fixtures to `tests/fixtures/union/{crate_a,crate_b}` and snapshot the reconcile stdout.

**FF test-build spec (carried verbatim into the Feature-Forge handoff; promoted as a test-build
backlog row — see reports/ROADMAP-grit.md):** planning-engineer authored + RED-ran the additive suite;
Feature Forge builds production code and GREEN-runs it. GREEN = (a) `grit reconcile --help` exits 0;
(b) `grit reconcile <A> <B>` exits 0 with stdout containing `parse`/`dedupe` (auto-merge set) AND
`checksum` with the word `conflict`; (c) `grit reconcile --lock-conflicts <A> <B>` exits 0 with stdout
containing `core.rs::checksum`. Engine-first (join two roots by id, partition by `Symbol.hash`), then
thin CLI wiring, then `--lock-conflicts` via `LockStore::try_lock`. CI note: builds grit standalone —
apply the phantom-workspace remediation (roadmap #0a) so `cargo test` resolves. Keep tests additive;
do not weaken assertions.

## Prompt-architecture

Source: prompt-architecture-grit.md. The tool built to orchestrate agents **does not expose itself to
agents through any authoritative surface** — CLAUDE.md/AGENTS.md are generic ICM+RTK boilerplate with
zero grit-specific guidance; the only place that tells an agent how to call grit is a `cat <<PROMPT`
heredoc buried in `examples/05-claude-code-integration.sh`, which literally says "Add this to your
CLAUDE.md" (per-consumer copy-paste, no canonical source, silent drift vs `src/cli/mod.rs`). The thin
prompt surface IS the headline finding and is a defect for a coordination substrate. Tool-grant gaps:
**no machine-readable output** (every verb prints ANSI-colored prose; agents must scrape stdout +
substring-match — string-fragile even internally, PA-G1); **destructive verbs** (`done` merges+deletes
branches, `session pr` pushes+opens PRs) ship **no permission profile** (PA-G2); **no caller-identity
authorization** — any process can `release`/`done` another agent's locks (PA-G3); **Azure key on
argv** while S3 uses env (PA-G4). Model lanes derived from verb risk: mechanical/haiku
(`heartbeat`/`status`/`gc`/`assign`), structured/sonnet (`claim`/`release`/`plan`/`session start`),
decision-gate/opus (`done`/`session pr`/`session end` — the irreversible merge/push/PR set, `[!!]`).
grit stays model-agnostic/LLM-free internally (explicit no-ADR). Upgrades: authoritative agent
operating contract (SKILL.md generated from / checked against the CLI, PA-U1); `--json`/MCP machine
surface (PA-U2, roadmap #11/ADR); recommended permission+lane profile drop-in (PA-U3); caller-identity
binding (PA-U4, roadmap #6/ADR); Azure key via env (PA-U5).

## Risk policy

See `risk-policy.md` `## grit` for the full policy. The SUPERVISED `[!!]`/`[H]` risk boundary covers:
(1) **secrets / trust-boundary** — plaintext Azure access key in `.grit/config.json` at default perms
(gov-009); harden via env-parity-with-S3 or 0600 + keyref, owner-gated; (2) **destructive verbs** —
`done` (rebase+merge+branch delete) and `session pr/end` (push/PR/checkout) require a destructive-
command guard mirroring parent meta and an explicit allowlist (gov-011, PA-G2); (3) **no-C trust
boundary** — grit's rusqlite-bundled + tree-sitter C substrate keeps it OUTSIDE handoff's no-C boundary;
any in-boundary reconciler must be pure-Rust (Route A). All three are SUPERVISED, fail-closed, never
auto-applied.

## Confidence

**Overall: HIGH on the verdict and the structural facts; MEDIUM on the union-fitness *routing* until
the FF GREEN run lands.** The headline (advisory lock + line-level git coordinator; hash computed but
never read; reconcile absent) is CONFIRMED by direct source reading AND a RED run (3 tests, all RED for
the right reason). The 12 material claims are CONFIRMED (1 QUALIFIED: the Azure GA date / Cargo CVE IDs
are web-sourced — verify at apply-time; 1 partial INCONCLUSIVE: the S3-secret-field sub-part, Azure
half confirmed). All upgrade rows are feasibility-passed as additive within grit's invariants; the
single REFUTED framing is "grit as-is = the in-boundary no-C union engine" (its C substrate forbids it).

What would raise confidence: (a) the Feature-Forge GREEN run flipping the 3 RED tests (proves Route B
buildable and the partition contract correct); (b) line-verifying the S3 `S3Config` secret field
(s3_store.rs:636) to close the one INCONCLUSIVE sub-part; (c) re-verifying the Azure GA date and Cargo
CVE IDs at apply-time; (d) a decision between Route A (pure-Rust in-boundary reconciler) and Route B
(grit-inline out-of-boundary) — the draft ADR frames it but the owner decides.

Named gaps / not examined: grit's cross-repo edges were deferred (self-contained crate this cycle —
the embed-into-union mapping belongs to the union-step plan); 1 `src/` file was skipped on index
(under-indexing gap); the `--with-deps` over-locking precision (#8) is plausible but its union impact
is unquantified. Notable refuted/infeasible findings reported as gaps: the "in-boundary engine"
framing (REFUTED on no-C); no cloud→local lock resync exists (so the read-skew is a real defect, not a
QUALIFIED-down).
