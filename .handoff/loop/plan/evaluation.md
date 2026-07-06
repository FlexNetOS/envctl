# evaluation.md — self-eval scorecard (evolution-steward)

Run: `plan-loop-parallel / weave` · cycle **4** · target **weave** · 2026-06-26 (UTC).
Marker: **scorecard / self-eval / evolution**. Mode: PARALLEL isolated instance
(`cycle_budget=1`), first run of the new parallel prompt `prompt_hub/prompts/plan-loop-parallel-run.md`
(#181). This steward **evaluates + mines + routes + PROPOSES only** — no harness edit is auto-applied
or PR'd this cycle (PROPOSE-only per the run brief). Every routed upgrade is recorded as a proposal in
`proposed-upgrades.md`.

Evidence base: `loop_state.md`, `findings/verdicts.md` (## weave cycle 4), `findings/architecture-weave.md`,
`findings/*-weave.md` (axis findings), `reports/weave-plan.md`, `reports/agent-run-ledger-weave.md`,
`dimensions.md`, `graph/weave.{symbols,metrics}.json`.

---

## Scorecard — four axes

| axis | grade | one-line |
|---|---|---|
| Friction | **A-** | First run of the new PARALLEL prompt; gate-named artifacts emitted directly, lease claimed first try, own-worktree isolation clean. Only rough edge: a transient concurrency-timing read (a peer artifact briefly "not yet present"), no false finding shipped. |
| Gate quality | **A** | Verifier re-ran every empirical probe as its own oracle, CONFIRMED 16 / QUALIFIED 4 / REFUTED 0, corrected three undercounts the analyst had carried, and found a real backend asymmetry. Strong adversarial reading; 0 false-blocks; no gate weakened. |
| Coverage | **B** | 5/12 dims `[x]`; 3 axes (code-quality/correctness/performance) folded into architecture with no dedicated file, 4 axis findings present-but-not-gated this pass. All gaps recorded honestly in `dimensions.md`, but the un-gated debt is real. |
| Human-walls | **A** | Both walls are genuine: the `main.rs` extraction is correctly SUPERVISED (highest-blast bin file), and the unmanaged `~/.config/weave/memory` user-global writes need an exemption ADR, not an auto-fix. Neither is avoidable. |

### Friction (A-) — first run of the parallel prompt
- **The parallel design worked on first contact.** This instance ran entirely in its own worktree
  (`meta/.worktrees/plan-weave/envctl` on `plan/loop-weave`) under the weave lease `plan:claim:weave`
  (`HF_LEASE_HOLDER=plan-weave-20260626`), with a separate RED worktree
  (`.worktrees/plan-weave-red/weave`). Zero edits to the union loop branch
  (`reports/agent-run-ledger-weave.md:27-30` parallel-isolation proof). Lease claim succeeded first try.
- **Gate-named artifacts emitted directly** — the 13-lane fan-out produced every gate-named graph split,
  trends headers, findings file and report with **no SendMessage reconcile round-trips** recorded in the
  agent-run ledger. This is the L1 (artifact-name contract) behavior holding under parallelism — the
  drift class that cost two round-trips in the union cycle 1 did **not** recur here.
- **Only rough edge (harmless):** under the parallel fan-out, an axis agent can read a sibling artifact
  before a peer lane has finished writing it and momentarily observe it as "not yet present". This was a
  transient read timing artifact, not a fail-closed finding — **no false "missing" finding shipped** to
  `verdicts.md` (the orchestrator gates analysis on orientation completion). Recorded as L9 below.
- No items bounced `- [~]`→`- [ ]`; no retried/abandoned agents; one limit-free cycle.

### Gate quality (A) — verifier tally 16 CONFIRMED / 4 QUALIFIED / 0 REFUTED (20 verdicts)
- **Empirical oracle, not summary-trust.** The verifier re-ran `grep`/`wc`/`awk` over the real tree as
  the oracle rather than trusting the analyst's numbers (`verdicts.md:14-15` "Empirical commands re-run,
  not trusted from the analyst summary"). The empirical bench (`verdicts.md:19-33`) independently
  measured: A2A symbols **absent** (0 hits across both crates), `main.rs` = **9631** lines / 76 `Cmd::`
  arms, the dual-backend conformance harness **absent**, the `memory.rs` organ **present** with **0 ICM
  refs**, and **no** vector/RAG deps.
- **Caught real undercounts and routed them back.** Three analyst figures were QUALIFIED against source
  and corrected: `Store` trait **29 → ~90 methods** (ARCH-03), MCP tool surface **78 → 72 arms / 76
  catalog** (ARCH-06/PA-TOOLS), and one Tarjan SCC re-labelled — `call_tool↔tool_meta` is **genuine
  bounded recursion** (guarded by the `want=="weave"` self-target check), **not** a resolver artifact
  (ARCH-12). These are routed back to the analyst as required corrections before they become plan facts
  (`verdicts.md:84-88`), and the dependent UPGRADE acceptance (U-ARCH-1 "all 29 methods") was re-scoped
  to the real surface so the conformance harness can't silently under-cover.
- **Found a real defect the harness would lock down.** Adversarial probing surfaced a genuine backend
  asymmetry: `LibsqlStore.send` (`store_libsql.rs:1499`) calls `self.guard_writable()?` before the
  `check_ident` block; `SqliteStore.send` (`store.rs:3153`) has no such call (ARCH-11). The impls are
  not byte-identical — exactly the silent drift the proposed conformance harness (U-ARCH-1) would catch,
  and a ready first divergence target.
- **0 false-blocks, no gate weakened.** Every UPGRADE passed the no-C / strict-upgrade feasibility gate
  (9 FEASIBLE, 1 FEASIBILITY-QUALIFIED U-MEM-2, 0 INFEASIBLE); the conditions attached (re-scope
  U-ARCH-1 acceptance, additive default-off A2A adapter, pure-Rust `tonic`/`prost` if gRPC ever added)
  are **strengthenings**, not blocks (`verdicts.md:64-75`).

### Coverage (B) — 5/12 dims verified, all gaps recorded
- **Verified `[x]` (5):** architecture, test-coverage, governance-config, memory-vector-intelligence,
  prompt-architecture (`dimensions.md:4,8,9,11,15`).
- **Folded, no dedicated file (3):** code-quality, correctness, performance — covered only as slices
  inside the architecture findings (`main.rs` god-file = ARCH-07/U-ARCH-3; `send`/dedup/parity =
  ARCH-05/08/11), not as standalone gated axes (`dimensions.md:5-7`). Honestly marked `[~]`.
- **Present-but-not-gated (4):** filesystem-layout, autoresearch, rules-policy-org, distributed-compute
  each have a findings file but no verdicts this pass (`dimensions.md:10,12,13,14`). The debt is real and
  named — a future cycle must run the verifier over these four before any weave DONE that needs them.
- Nothing was silently capped; every un-gated axis is enumerated with its missing artifact.

### Human-walls (A) — both walls genuine
- **`main.rs` dispatch extraction — SUPERVISED (correct).** U-ARCH-3 extracts the post-store CLI dispatch
  out of the 9631-line, highest-blast bin file into a `dispatch/*` module; rated PROPOSE/SUPERVISED
  because it is a large structural move on the top-blast file (`verdicts.md:68`,
  `architecture-weave.md:54`). A behavior-identical move on the riskiest file is a correct human wall, not
  an avoidable stop.
- **Unmanaged user-global writes → exemption ADR (correct).** `memory.rs` writes scoped notes under
  `~/.config/weave/memory` (MEM-2, `verdicts.md:61`) — an unmanaged user-global write that conflicts with
  the meta/envctl "everything lives in meta, globals hold only symlinks" invariant. The plan routes this
  to a docs/ADR (U-MEM-1 classify/quarantine as a bounded send-time cache; U-MEM-2 reconcile-or-document
  vs ICM), not an auto-deletion. Deciding a cross-plane residency exemption is genuinely an owner call.

---

## Net read
A clean, well-isolated first run of the parallel prompt with a strong adversarial gate. The headline
value this cycle is the **gate catching three analyst undercounts and a real backend asymmetry** before
they reached the plan. The headline debt is **coverage** (7/12 dims un-gated) and one new low-severity
parallel-mode artifact (peer-pending-vs-missing reads). Both are routed below; nothing is auto-applied.
