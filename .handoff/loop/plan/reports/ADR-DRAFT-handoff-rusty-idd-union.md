# ADR-DRAFT — Merge the handoff + rusty-idd union into one workspace; resolve RuVector; keep the filesystem fallback; no downgrade

- **Status:** DRAFT / PROPOSE (owner-walled — not written into the target repos' trees this cycle).
- **Date:** 2026-06-26 · **Author:** plan-architect · **Cycle:** 2 (handoff, planned as the union).
- **Deciders:** owner (D1 resolved: north-star @ `$META_ROOT`+handoff; goal = the union).
- **Evidence:** ONLY CONFIRMED/QUALIFIED + feasible verdicts in `findings/verdicts.md` (handoff cycle 2).
- **Companions:** `reports/handoff-plan.md`, `reports/union-plan-handoff-rusty-idd.md`,
  `reports/north-star-DRAFT.md`. (Distinct from the cycle-1 `reports/ADR-DRAFT-rusty-idd-convergence-boundary.md`.)

## Context

handoff and rusty-idd are 95%+ shared lineage (A-C9, union-1): handoff is the production-hardened,
kernel-focused fork (real-teeth `exit(1)` gates, the redb+witness ledger, 13 `handoff-*` crates);
rusty-idd is the more-developed intent/OpenSpec superset with 8 CLI commands handoff stripped. They
share identical `rusty-idd-*` package names across two workspaces (fs-2), and rusty-idd attaches to
handoff today only by a **mirrored file copy** of the `handoff.task.v1` contract, not a dependency
(A-C13). Owner D1 resolves the goal: a unified continuity+intent control plane with handoff as the
north-star home @ `$META_ROOT`+handoff.

Three facts force this decision:
1. **The union is not standalone** — `hf`+`ledger` pull RuVector via `../../RuVector/*` path deps; the
   whole workspace fails at manifest-load even for `redb-store` (EXP-1, empirical). It cannot build at
   its own north-star location.
2. **Co-location is a hard Cargo collision** — two workspaces with identical `rusty-idd-*` pkg names
   cannot merge without dedup (fs-2/V3).
3. **The contract is duplicated across the boundary** — the exact fail-open class `validate_card` was
   built to kill, but only within handoff (A-C13/G2).

## Decision

1. **MERGE into ONE workspace at the handoff north-star.** Fold rusty-idd's superset CLI under
   handoff's kernel and gates; handoff = the authoritative source. Dedup `crates/{cli,core,runner,spec,
   tui}` to one canonical set with the **rusty-idd superset as winner**, re-applying handoff's
   HFTASK-0082 lint hardening (A-U4, codemap §4). (Not FEDERATE / not SHARED-CORE — the 95% identity +
   rusty-idd's lack of ledger support make those wasteful; union §4.)
2. **Resolve RuVector off the `../../` path deps** by ONE of: (a) **vendor** the `rvf-*`/`ruvector-*`
   crates under `vendor/ruvector/` + workspace `[patch]` (mirrors the syntect precedent, fs V4 — the
   recommended default); (b) **publish** the crates and depend by version; (c) **git-pin**
   `FlexNetOS/meta-ruvector` by rev. The CI sibling-clone becomes a documented fallback only (A-U1,
   EXP-1, gov-003).
3. **Replace the mirrored `work-order` with a real dependency** on handoff's `work-order` +
   `handoff-schema::validate_card` — one compiler-enforced `handoff.task.v1` source of truth (A-U3).
4. **Keep the filesystem `.handoff/` contract as the required fallback.** Every transport/binding
   upgrade preserves the `.handoff/tasks` JSON seam as the offline/degraded route (union-2 cond. e,
   rp-A2A degrade-to-ledger-only). The union never removes the file seam.
5. **NO C in the trust boundary.** Only pure-Rust native surface (sha3/blake3/ed25519-dalek + redb);
   `rusqlite` stays feature-gated migration-only (dc-4). The vendored RuVector surface and any future
   adapter must remain C-free. The at-land C-dep re-scan of rusty-idd `codex`/`knowledge`
   (syntect-onig/codegraph) is a precondition (union-2 cond. c).
6. **Strict-upgrade-only / no downgrade.** Superset wins; hardening re-applied; every step is additive
   (adds a gate, a dep, or a contract); none weakens an existing guard. A silent model/provider
   downgrade is a guard violation (UP-4 / No-Downgrades).

## Provenance correction recorded by this ADR

The witness chain is **SHAKE-256 hash-linked (SHA3-256 action hash), UNSIGNED** — NOT
"blake3+ed25519-signed" (EXP-3: `rvf-crypto/src/witness.rs:4`; `ledger/src/v1.rs:20` imports no
`sign`). `ed25519-dalek` is compiled into the default `ledger` build but the witness path never signs
(A-C7); blake3 is used only for `work-order::compute_intent_lock`. Any doc/seed/trends text saying
"blake3+ed25519 witness chain" must be corrected (mem-U3).

## Consequences

**Positive:** a provably standalone, portable kernel root; one maintainable toolkit with the stripped
CLI commands restored; a single fail-closed contract covering rusty-idd's writes too; the union builds
at its own north-star location.

**Costs / risks:** A-U1 and A-U4 are **SUPERVISED** (large blast — `Ledger.open` blast 120; 5-crate
reconcile + name collision; witness-crypto deps). core/cli are ~33%/~40% diverged (substantial
reconcile; A-C9 exact % is tool-derived/QUALIFIED). The **ledger read API (Seam 2) is unbuilt** and
must be designed before the intent plane can read witnessed state (ts-5, union-3). Below-leaf tests are
blocked until A-U1 (ts-U2, the standalone-build gate E1).

**Reversibility:** HIGH for the dep-source swap (Cargo) and the contract dep (re-vendor); MEDIUM for
the fork dedup (git history preserves both forks; core/cli are the risk).

## Alternatives rejected

- **FEDERATE (keep repos separate + thin adapter):** duplicate crates, no single ledger authority,
  drift-prone — NOT RECOMMENDED given 95% identity (union §4 Option B).
- **SHARED-CORE (extract a shared crate):** worth considering long-term but adds a coupling edge and
  CLI still can't call hf verbs directly; MERGE is simpler for the union (union §4 Option C).
- **Leave RuVector as path deps + CI clone:** keeps the union non-standalone; fails the portable-root
  mandate (fs §3 boundary verdict) — rejected.

## Acceptance (falsifiable gates)

- `cargo build --workspace` green in a clone with NO sibling `RuVector/` (gate E1; RED today).
- `cargo metadata` shows each `rusty-idd-*` package exactly once (gate E3).
- rusty-idd cards pass handoff `validate_card`; the two `task.schema.json` are byte-identical or the
  duplicate is deleted.
- `work-order/tests/union_failclosed.rs` flips GREEN when the fail-closed loader is implemented (ts-U1).
- No `../../` escaping path-dep remains in any member manifest (gate E2).
