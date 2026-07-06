# UNION plan — handoff ⊕ rusty-idd (the unified continuity+intent control plane)

Author: plan-architect · Date: 2026-06-26 · Strategy: **MERGE** (fold rusty-idd's superset CLI into
handoff's kernel; **handoff = the north-star home @ `$META_ROOT + handoff`**, owner D1). Built ONLY
from CONFIRMED/QUALIFIED + feasible verdicts (`findings/verdicts.md` handoff cycle 2 + the cross-cycle
rusty-idd rows). Companion: `reports/handoff-plan.md`, `reports/rusty-idd-plan.md`,
`reports/north-star-DRAFT.md`, `reports/ADR-DRAFT-handoff-rusty-idd-union.md`.

---

## Union verdict (one line)

**MERGE: handoff is the production-hardened kernel with real-teeth gates and the witnessed ledger;
rusty-idd is the intent/OpenSpec superset with the 8 CLI commands handoff stripped — fold rusty-idd's
CLI UNDER handoff's gates into ONE workspace at the handoff north-star, but the union cannot build
until the RuVector path-dep is resolved (the non-standalone blocker).**

Evidence base (CONFIRMED/QUALIFIED): 95%+ shared lineage with handoff the hardened fork (union-1,
A-C9); identical `rusty-idd-*` pkg names = a Cargo collision on co-location (fs-2); `work-order`
originated in handoff and is mirrored (not depended on) in rusty-idd (A-C13); handoff's gates
`exit(1)` while rusty-idd's are advisory (rp-teeth); union=MERGE is **QUALIFIED-feasible** under
`{RuVector-resolve + pkg-dedup + no-C-scan + work-order-dep + filesystem-fallback}` (union-2).

## Three-way fit (owner D1/D3 convergence picture)

```
   user intent
       │
       ▼
   harness_hub  ── Front-Door INTERPRETER (D3): intent → model-ready language
       │            (non-deterministic; sits UPSTREAM of handoff's classifier)
       ▼
   handoff  ── WITNESSED CONTRACT + DETERMINISTIC classifier + LEDGER  ⇄  rusty-idd
   (continuity kernel; the UNION HOME)                                     (intent: OpenSpec why/what)
       │   one append-only witnessed log carries BOTH continuity events
       │   AND curated intent/decision records (ESAA pattern, trends §B1/§D1)
       ▼
   weave  ── A2A TRANSPORT (a DISTINCT plane; degrades to ledger-only offline; rp-A2A)
       ▼
   models / distributed compute ─▶ output ─▶ user
```

- **harness_hub** = the interpreter at the front door (intent→model language). It writes INTO handoff
  but does not perform handoff's deterministic classification (pa §6, D3).
- **handoff** = the continuity kernel + the deterministic intake classifier (`prompt_hub`/`intake`:
  vibe→`WorkOrder`, non-LLM, byte-identical card) + the witnessed ledger. **The union home.**
- **rusty-idd** = the intent control plane (OpenSpec why/what). Its spec/decision records become
  **curated events on handoff's ledger** (trends §D1), not a rival store.
- **weave** = transport, a distinct plane — never fused with the witnessed-receipts plane (rp-A2A).

## What each side brings (no overlap to rebuild)

| | handoff (KERNEL) | rusty-idd (INTENT superset) |
|---|---|---|
| Unique | hf · ledger (redb+SHAKE-256 witness) · work-order SoT · 13 handoff-* (policy/drift/intake/route/gates/fleet/lease/index) · real-teeth gates | codex/deploy/harness/knowledge/merge-tools/next/render/spec-plan-integration CLI cmds · config/external/codegraph crates |
| Shared (95%) | crates/{cli,core,runner,spec,tui} — STALE partial fork (A-C9) | crates/{cli,core,runner,spec,tui} — the SUPERSET, more developed |
| Seam today | reads `.handoff/tasks` work-order JSON | mirrors `task.schema.json` (copy, not dep) |

---

## The 5 sequenced union steps

Ordered by value/risk (graph centrality + blast); each carries blast + reversibility. Steps 1 and 2
are the gates — nothing else can build or dedup until they pass.

### Step 1 — Resolve the RuVector standalone blocker  · **SUPERVISED**
- **What:** move `hf` + `ledger` off the `../../RuVector/*` path deps — choose ONE of {vendor the
  `rvf-*`/`ruvector-*` crates under `vendor/ruvector/` + workspace `[patch]` (mirrors syntect);
  publish them and depend by version; git-pin `FlexNetOS/meta-ruvector` by rev}. Optionally make the
  `ledger` `v2`/RVF overlay default-off (couples to Step-of-record A-U2/mem-U2).
- **Why first:** EXP-1 — the path dep fails the WHOLE workspace at manifest-load (even leaf
  `work-order`); the union literally cannot build at `$META_ROOT + handoff` until this lands. It also
  gates the ledger read-API design (Step 4) and every below-leaf test.
- **Evidence:** EXP-1, A-U1/A-C5/A-C6, gov-003, fs V1/V2, DC-1, mem-U4. **Blast:** entire KERNEL
  (`Ledger.open` 120). **Acceptance:** `cargo build --workspace` green in a clone with NO sibling
  `RuVector/` (gate E1, RED today). **Reversibility:** HIGH — Cargo dep-source swap; vendored copy is
  additive/removable; witness chain unchanged (Integrity preserved). **No-C:** sha3 + ed25519-dalek +
  redb are all pure-Rust — the vendored surface stays C-free.

### Step 2 — Dedup the 95% shared crates/{cli,core,runner,spec,tui} (superset wins + re-apply HFTASK-0082)  · **SUPERVISED**
- **What:** converge the stale handoff fork to rusty-idd's superset as the single canonical set, then
  RE-APPLY handoff's HFTASK-0082 lint hardening (`#![cfg_attr(test, allow(...))]` + production
  unwrap/expect/panic discipline) onto the winner; restore rusty-idd's 8 stripped CLI commands on top
  of the kernel; fold rusty-idd's extras (config/knowledge/merge-tools/external/codegraph-*).
- **Why second:** identical `rusty-idd-*` pkg names are a hard Cargo workspace name-collision on
  co-location — dedup is mandatory, not optional (fs-2). Per-crate effort (codemap §4): tui ~0.1% /
  spec ~1.2% trivial; runner ~11% moderate; core ~33% / cli ~40% substantial.
- **Evidence:** A-U4/A-C9, fs V3/§4, union §4, codemap §4. **Blast:** 5 crates + 3rd binary; ~0
  KERNEL (1 real call edge, A-C8). **Acceptance:** one `rusty-idd-*` pkg per name (`cargo metadata`);
  spec/tui differential-golden parity green (gate E3). **Reversibility:** MEDIUM — git history
  preserves both forks; core/cli are the risk. **Strict-upgrade-only / no-downgrade:** winner = the
  superset; the hardening lint is re-applied so no production safety is lost.

### Step 3 — rusty-idd depends on handoff `work-order` + `validate_card` (kill the mirrored schema)  · **PROPOSE**
- **What:** replace rusty-idd's mirrored `work-order` copy with a real crate dependency on handoff's
  `work-order` + `handoff-schema::validate_card`, making `handoff.task.v1` a single compiler-enforced
  source-of-truth across the union.
- **Why third:** the contract is duplicated across the boundary (A-C13/G2) — the exact fail-open class
  `validate_card` was built to kill, but only *within* handoff. After Step 2 the workspace is one
  tree, so the dep is direct. Pure-Rust crate dep — stays inside the no-C boundary.
- **Evidence:** A-U3, A-C13, codemap §5.1, ts-3. **Blast:** rusty-idd work-order consumers + schema
  generation. **Acceptance:** rusty-idd cards pass handoff `validate_card`; the two `task.schema.json`
  are byte-identical or the duplicate is deleted (golden gate, RED/absent today). **Reversibility:**
  HIGH — re-vendor the copy. **Integrity strengthened:** one fail-closed gate covers rusty-idd's
  writes too.

### Step 4 — Design the MISSING ledger read API (Seam 2)  · **PROPOSE**
- **What:** design + build a public, read-only ledger surface (`get_claimed()` / `latest_checkpoint()`
  / `witness_chain()`) — minimal, thread-safe, immutable (no writes from outside `hf`) — so the intent
  plane (rusty-idd commands like `harness`/`deploy`) can read witnessed state instead of file IO. Then
  author the contract test (`ledger/tests/read_api.rs`, ts-U3).
- **Why fourth:** Seam 2 is CONFIRMED MISSING (ts-5, union-3) — `Ledger.open` callers are all
  in-kernel (blast 120). It is design-only today: a compiling RED cannot be authored until the API
  exists AND Step 1 lands (a call to a non-existent `Ledger::query_claimed` would fail to COMPILE,
  which is forbidden). It depends on Step 1 (the ledger can't build standalone) and benefits from
  Step 2 (one workspace).
- **Evidence:** ts-5/ts-U3, union-3, Seam 2 (codemap §5). **Blast:** `Ledger.open` (120) — the
  kernel's widest surface; the read API must not change the write/witness path. **Acceptance:**
  read-only contract test green; reads never mutate the witness chain. **Reversibility:** MEDIUM — the
  new crate/module is additive; the API shape is a stickier commitment (hence design-first + ADR).

### Step 5 — Bridge `hooks.toml` block-gates to the Claude PreToolUse lifecycle (close the fail-OPEN seam for BOTH)  · **PROPOSE**
- **What:** wire `.claude/settings.json` `PreToolUse(Edit|Write|Bash)` → `hf hook run` (PreEdit /
  PreCommand) + `PostToolUse` → PostEdit, so the 5 `fail_mode="block"` gates fire at the agent
  tool-call boundary; fold rusty-idd's CLI commands UNDER `hf policy check-edit`/gatekeeper (UP-1) so
  the same teeth cover them. Self-enforce agent-guard via PreToolUse (gov-U2/UP-5).
- **Why fifth (and last):** gov-001 (CONFIRMED empirical) — the block gates are NOT bridged to Claude
  (`settings.json` has only SessionStart/SessionEnd), so an edit can go out-of-scope without tripping
  the gate: the kernel's own L7 fail-OPEN class. Folding rusty-idd's now-merged CLI under the gates
  (Step 2 prerequisite) closes the seam for the whole union in one move.
- **Evidence:** gov-001/gov-U1, UP-1/UP-5, rp-teeth, gov-002. **Blast:** every Claude session edit
  (enforcement, not logic). **Acceptance:** an out-of-scope edit (native `hf` OR a rusty-idd command)
  is DENIED by the PreToolUse hook in a live attempt, identical refusal (exit 1). **Reversibility:**
  HIGH — revert the settings hunk / remove the hook wiring; commands still run standalone. **Never
  weakens a guard** (additive enforcement only).

---

## Union invariants held across all 5 steps

- **Keep the filesystem `.handoff/` contract as the FALLBACK.** Every binding/transport upgrade
  preserves the `.handoff/tasks` JSON contract as the offline/degraded route (union-2 condition e,
  rp-A2A `Reserve::Unsupported`→`ProceedDegraded`). The union never *removes* the file seam.
- **NO C in the trust boundary.** The only native surface is pure-Rust sha3/blake3/ed25519-dalek +
  redb; `rusqlite` is feature-gated migration-only (dc-4). The at-land C-dep re-scan of rusty-idd's
  `codex`/`knowledge` (syntect-onig / codegraph) is a union-2 precondition (passed cycle-1, re-check
  at merge).
- **Strict-upgrade-only / no downgrade.** Superset wins (Step 2); hardening lint re-applied; every
  step adds a gate, a dep, or a contract — none weakens an existing guard.
- **Owner-walled MERGE.** Steps 1 + 2 are SUPERVISED (large blast, name collision, witness-crypto
  deps); Steps 3-5 are PROPOSE. Classification in `risk-policy.md` (`## handoff (cycle 2)`).

## Open union gaps (honest)

- **Ledger read-API shape is undesigned** (Step 4) — the largest open design item; needs an ADR.
- **Below-leaf tests blocked** until Step 1 (ts-U2 intake refusal, the standalone-build gate E1).
- **Exact fork divergence % is tool-derived** (A-C9 QUALIFIED) — core/cli reconcile effort is
  estimated, not measured to the line.
- **Dual front door** (`hf` loop-entry vs `rusty-idd next`) is not yet reconciled to ONE canonical
  entry (pa-U1) — a union control-plane decision, recorded as an ADR-candidate.
