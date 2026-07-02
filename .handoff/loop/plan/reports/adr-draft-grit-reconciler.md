# ADR-DRAFT — The union reconciler is a no-C pure-Rust component; grit provides coordination only

- Status: **DRAFT** (proposed by plan-architect, cycle 5; owner decision required)
- Date: 2026-06-27
- Target: grit / union step 2 (dedup the ~95%-shared handoff↔rusty-idd crates)
- Supersedes: nothing. Written in the plan dir only (`.handoff/loop/plan/reports/`) — NOT in grit's
  tree (owner-wall).

## Context

Union step 2 needs to dedup two near-identical crate trees at symbol granularity. grit was the
candidate engine. Verified evidence (`findings/verdicts.md` `## grit`):

- grit's only content merge is line-level `git merge --no-ff` (git/mod.rs:221-253); "no conflicts" is
  partitioning, not reconciliation.
- A per-symbol content hash IS computed (parser/mod.rs:329) and persisted but **never read** in any
  production path — grep of `src/db` shows zero `SELECT`/compare outside `#[cfg(test)]`.
- The two primitives a reconciler needs already exist: deterministic `Symbol.hash`
  (`test_symbol_hash_deterministic`, parser/mod.rs:908) and `LockStore::try_lock` (lock_store.rs:29).
  Only the cross-source composition (`Reconcile{a,b}`) is missing — RED-proven (3 tests, all RED:
  unrecognized `reconcile`).
- **Load-bearing constraint:** grit's substrate is NOT no-C — `rusqlite` uses `bundled` SQLite (C) and
  all 14 tree-sitter grammars are C. The envctl no-C trust boundary is NON-NEGOTIABLE.

This is a genuine architecture decision (where the reconciler lives relative to the trust boundary, and
what grit's role is), not a routine upgrade — hence an ADR rather than only a ROADMAP row.

## Decision (proposed)

**The union reconciler is a pure-Rust (no-C) component that lives INSIDE handoff's trust boundary
(Route A); grit provides coordination ONLY (locks, git worktrees, room events) from OUTSIDE that
boundary.** grit is NOT adopted as the in-boundary union engine — that framing is REFUTED on the no-C
invariant.

The reconciler:
- runs over two roots, joins symbols by id, partitions by a **stable pure-Rust hash** into
  identical(auto-merge) / divergent(conflict) / only_in_a / only_in_b;
- for divergent symbols, requests a lock from grit (coordination call only — no C crosses the boundary);
- never depends on rusqlite-bundled or tree-sitter directly inside the boundary (it brings its own
  no-C parse/hash, or consumes a pre-computed symbol export from grit's out-of-boundary side).

## Alternatives considered

- **Route B — `grit reconcile` built inline in grit.** Feasible today as additive Rust (reuses
  `scan_all` + `Symbol.hash` + `try_lock`); flips the 3 RED tests (this is the FF GREEN target,
  `grit-ff-1`). REJECTED as the *in-boundary* engine because grit's C substrate keeps the result
  OUTSIDE the no-C boundary. **Accepted as an interim/out-of-boundary tool** — Route B and Route A are
  not mutually exclusive: Route B proves the partition contract and serves out-of-boundary coordination
  use; Route A is the in-boundary destination.
- **grit as-is = the in-boundary no-C union engine.** REFUTED (rusqlite-bundled + tree-sitter are C).
- **CRDT-everywhere reconciliation layer.** REJECTED — CRDTs prevent conflicts by construction but do
  not resolve them post-hoc and remain weak for code/AST structure (research C4); grit's lock +
  structural-merge model is the better match.

## Consequences

- Positive: the in-boundary engine honors no-C; grit keeps its proven coordination role; the stable
  pure-Rust hash (roadmap grit-0b) becomes a shared prerequisite for both routes; the field validates
  the symbol-level thesis (weave 31/31 vs git 15/31, research C1).
- Negative / cost: two parse paths may exist transiently (grit's tree-sitter out-of-boundary vs the
  reconciler's no-C parse in-boundary) until a git-kb-compatible symbol export (memory-vector UPGRADE-2)
  unifies the source of symbol truth; symbol-id disambiguation (roadmap grit-5, schema migration) is a
  downstream prerequisite for a precise per-symbol ledger.
- Follow-ups gated by this ADR: grit-5 (id disambiguation), grit-6 (caller-identity binding),
  grit-11 (`--json`/MCP machine surface).

## Open question for the owner

Confirm Route A vs Route B-as-destination, and who owns the no-C reconciler crate (handoff vs a new
member). The draft does not decide ownership; it decides the boundary placement and grit's coordination-
only role.
