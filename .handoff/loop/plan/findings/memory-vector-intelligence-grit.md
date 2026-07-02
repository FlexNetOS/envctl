# memory-vector-intelligence — target: grit

Target repo: `/home/drdave/Desktop/meta/grit` (rtk-ai/grit) — an AST-level lock/merge
coordination substrate for parallel AI agents. Axis: `memory-vector-intelligence`.

## Verdict (3-line)

grit ships NO persistent agent-memory layer (no ICM), NO embeddings/RAG/vector store, and NO
handoff witnessed-ledger integration — that absence is legitimate for a merge/lock substrate.
However, grit DOES own a persistent code-intelligence surface of its own (`.grit/registry.db`:
a tree-sitter symbol index + call-graph + locks), which functionally overlaps git-kb; and its
merge/claim decisions are NOT recall-informed and NOT durably witnessed.
Convergence opportunity: feed grit's symbol/lock state into the fleet's git-kb / ICM / handoff
ledger so merge decisions become recall-informed and auditable.

## 1. Memory inventory

| Surface | Present in grit? | Evidence |
|---|---|---|
| ICM persistent memory (recall/store) | No (no integration) | `icm` named only as a sibling RTK ecosystem project, README.md:332; no `icm`/`recall`/`store` call sites in `src/` (grep: zero hits) |
| Vector / embeddings / RAG index | No | Cargo.toml has no `qdrant`/`faiss`/`embed`/`vector`/`reqwest`-to-embeddings dep; grep `embedding|vector|\brag\b` over `src/` = zero |
| git-kb code graph snapshot | No (not consumed) | grit builds its OWN graph instead (see below); no `git-kb`/`kb_` references in `src/` |
| `.handoff` witnessed ledger / source ledger | No | no `.handoff`/`.idd`/`handoff` references in `src/` or `docs/` (grep zero) |
| grit's OWN persistent store (`.grit/registry.db`) | Yes | SQLite WAL, schema `src/db/mod.rs:79-131`; architecture diagram README.md:247-268 |
| Symbol index (persistent) | Yes | `symbols(id,file,name,kind,start_line,end_line,hash)` `src/db/mod.rs:82-90`; populated by `upsert_symbols` `src/db/mod.rs:152` |
| Call-graph / dependency edges (persistent) | Yes | `deps(caller,callee,kind)` `src/db/mod.rs:102-107`; extracted via tree-sitter `Parser::scan_with_deps` `src/parser/mod.rs:101`, edge type `Dep{caller,callee}` `src/parser/mod.rs:18-22` |
| Lock / intent state (persistent) | Yes (no rationale beyond intent string) | `locks(symbol_id,agent_id,intent,mode,locked_at,ttl_seconds)` `src/db/mod.rs:92-100` |
| Event stream (ephemeral, NOT persisted) | Yes (ephemeral) | `RoomEvent{Claimed,Released,AgentDone}` `src/room/mod.rs:9-20`; broadcast over Unix socket, dropped if no listener `src/room/mod.rs:35-45` |
| Recall/store hooks for cross-session agent memory | No | N/A — grit has no agent-memory plane; lock TTL/heartbeat (`src/db/mod.rs:98`) is coordination state, not recoverable "why" memory |

CLAIM-M1: grit maintains a persistent, queryable code-intelligence store independent of git-kb.
Evidence: `symbols` table `src/db/mod.rs:82-90` + `deps` call-graph `src/db/mod.rs:102-107`,
queried by `get_deps`/`get_transitive_deps` (`src/db/mod.rs:355-378`) and `search_symbols`
(`src/db/mod.rs:260`). This is the same shape (symbol + caller/callee edges) the fleet's git-kb
code graph provides — a duplication/convergence candidate, not a memory gap per se.

CLAIM-M2: grit's coordination *events* are ephemeral and unwitnessed. Evidence: `Room::notify`
returns early when no socket peer exists (`src/room/mod.rs:36-38`) and writes JSON to a transient
`UnixStream` (`src/room/mod.rs:39-45`); no table persists `RoomEvent`. A merge/claim/release
therefore leaves no durable audit row beyond the live `locks` row (deleted on release). Contrast
the fleet's handoff witnessed ledger, where decisions are appended and rendered, never lost.

## 2. Vector intelligence map

| Index | Exists | Freshness mechanism | Owner | Update command | Failure behavior |
|---|---|---|---|---|---|
| Embedding / RAG / vector DB | No | N/A — grit does no semantic retrieval; symbol search is keyword `LIKE` (`src/db/mod.rs:260`), not vector similarity | N/A — none | N/A — none | N/A — none |
| grit symbol+deps graph (`.grit/registry.db`) | Yes | Rebuilt by `grit init` (`scan_with_deps` `src/parser/mod.rs:101`); STALE between runs — README.md:279 instructs re-run `grit init` "if the codebase changed" | grit CLI (the repo owner who runs `grit init`) | `grit init` (README.md:156, `src/cli/mod.rs`) | Stale-closed-ish: `grit claim` on an unindexed symbol fails with an actionable error ("not in the registry … re-run grit init") `src/db/sqlite_store.rs:14-18` — a claim cannot silently succeed on an unknown symbol |

CLAIM-V1: There is no vector/RAG layer; the only "intelligence" is the lexical symbol search +
the AST call-graph. Evidence: `search_symbols` builds `name LIKE ?` clauses (`src/db/mod.rs:260`),
no embedding/ANN path anywhere in `src/`. For a lock substrate this is adequate (exact symbol
identity, not fuzzy recall, is what a lock needs).

CLAIM-V2: grit's graph freshness is manual and event-driven only at `grit init`; unlike the
fleet's git-kb daemon (file-watch re-index per the code-intelligence rule), grit has no watcher —
the index drifts from HEAD until the next `grit init`. Evidence: no file-watch/notify-on-save
code in `src/`; README.md:279 documents the manual re-index remedy.

## 3. Recall guarantees

No plan should depend on chat memory; here is grit's cold-start posture.

- Session-start recall: N/A — grit has no agent-memory recall step; `grit session start` only
  creates a git branch (`grit/<name>`, README.md:219, sessions table `src/db/mod.rs:109-115`). It
  does not `recall` prior decisions or rationale.
- Background-agent recall: N/A — agents coordinate via lock state + ephemeral socket events
  (`src/room/mod.rs:9-45`), not via a shared recall store. An agent that restarts re-reads live
  locks (`available_symbols_in_files` `src/db/mod.rs:179`) but cannot recall *why* a prior agent
  claimed a symbol beyond the free-text `intent` column (`src/db/mod.rs:95`).
- Wrap-up store: Partial/coordination-only — `grit done`/release deletes the lock row; the only
  surviving artifact is the git merge commit itself (README.md:73). No decision/rationale is
  stored for later `recall`.
- Cold-start resume proof: The durable truth that survives a process restart is `.grit/registry.db`
  (symbols, deps, locks, queue, sessions — `src/db/mod.rs:82-124`) plus git history. That is enough
  to resume *coordination* (who holds what), but NOT enough to resume *intent/why* — there is no
  ICM/handoff memory to recall.

CONVERGENCE CLAIM (recall-informed merge): grit merge decisions are currently mechanical (serialize
via `merge.lock`, rebase, merge — README.md:73, `src/git/mod.rs`) and are NOT informed by any
recall of past conflicts, prior rationale, or fleet memory. A recall-informed merge would query
ICM/handoff before granting a contested claim or resolving a rebase. grit has no such hook today.

## 4. Upgrade rows (axis: memory-vector-intelligence)

| ID | Upgrade | Evidence (file:line) | Acceptance | Risk | Reversibility |
|---|---|---|---|---|---|
| UPGRADE-1 | Emit a durable, append-only decision ledger for claim/release/done (witness the events that `src/room/mod.rs:35-45` currently drops) so merge decisions are auditable and feed the fleet handoff ledger | `src/room/mod.rs:9-45` (ephemeral today); `locks` deleted on release `src/db/mod.rs:92-100` | A new `events`/`decisions` table (or JSONL) records every Claimed/Released/AgentDone with agent, symbols, intent, timestamp; survives restart; readable by an external handoff/git-kb importer | Low — additive table, no change to lock semantics | High — drop the table/file; existing flows unaffected |
| UPGRADE-2 | Expose grit's symbol+deps graph as a git-kb-compatible export (avoid two divergent code graphs in the fleet) | symbols `src/db/mod.rs:82-90`, deps `src/db/mod.rs:102-107`, `scan_with_deps` `src/parser/mod.rs:101` | `grit symbols`/a new `grit graph --json` emits nodes+edges importable into the fleet git-kb code graph; one source of symbol truth | Medium — schema/identity mapping (grit `file::symbol` id `src/parser/mod.rs:21` vs git-kb ids) | High — export-only, read path |
| UPGRADE-3 | Recall-informed contested-claim policy: before granting/queuing a contested symbol (`lock_queue` `src/db/mod.rs:117-124`), allow an optional ICM/handoff `recall` of prior conflict outcomes for that symbol | queue path `src/db/mod.rs:387-433`; merge serialize README.md:73 | Optional hook (off by default) consults a recall source; when absent, behavior is byte-identical to today | Medium — must fail-open so grit stays standalone | High — feature-flagged; default off |
| UPGRADE-4 | Index freshness guard: detect HEAD drift since last `grit init` and warn/refuse-stale, closing the manual-reindex gap | manual remedy only README.md:279; no watcher in `src/` | `grit claim`/`status` compares working-tree hash vs `symbols.hash` (`src/db/mod.rs:89`) and flags drift | Low | High — advisory check |

## 5. Gate handoff (fail-closed additions)

So missing memory/vector surfaces fail closed rather than silently:

- Convergence gate (fleet-side, not grit-internal): a planning gate should assert that if grit is
  adopted as the fleet merge substrate, a decision-ledger exporter (UPGRADE-1) exists — otherwise
  merge decisions are unwitnessed (CLAIM-M2) and the handoff ledger has a blind spot. Acceptance:
  an artifact test that fails when `.grit/` produces merges with no corresponding durable
  ledger/event row.
- Graph-duplication gate: a test/assertion that grit's symbol export (UPGRADE-2) and the fleet
  git-kb graph do not silently diverge for the same repo HEAD (compare symbol-id sets). Until then,
  treat grit's `deps` graph as authoritative only for lock-scoping, not as the fleet code graph.
- Staleness gate: CI/precommit check that `.grit/registry.db` symbol hashes match HEAD (UPGRADE-4)
  so a stale index cannot grant a claim on a moved/deleted symbol — partially mitigated today by
  the actionable "not in the registry" refusal `src/db/sqlite_store.rs:14-18`.
- ICM/RAG aspects: N/A — grit intentionally has no agent-memory or RAG plane, so there is nothing
  to fail-closed *inside* grit; the gate lives at the fleet boundary where grit's lock/merge state
  is (or is not) bridged into ICM/handoff/git-kb.

## Source ledger

- grit source (read-only): `src/db/mod.rs`, `src/db/sqlite_store.rs`, `src/parser/mod.rs`,
  `src/room/mod.rs`, `src/cli/mod.rs`, `README.md`, `docs/RELEASE_FLOW.md`, `Cargo.toml`.
- Tooling: `tree-sitter 0.25` + 13 grammars (Cargo.toml), `rusqlite 0.31` bundled (Cargo.toml),
  `azure_storage*`/S3 backends for distributed lock store (README.md:96-145).
- Fleet convergence references (named, not integrated): `icm` README.md:332; handoff witnessed
  ledger and git-kb code graph are fleet substrates external to this repo.
