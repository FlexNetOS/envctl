# DRAFT — Fleet North-Star artifact (for `$META_ROOT` + handoff)

**Status: DRAFT / PROPOSE — owner canon.** This is a proposed single artifact every repo can read
(bind-as-data). It is NOT written into the meta-root tree or handoff canon without owner approval
(owner-wall). Authored by plan-architect, 2026-06-26, in the plan dir only.

Binds owner verdicts: **D1** — the north-star lives @ `$META_ROOT` + handoff; the goal is the
`handoff + rusty-idd` UNION (a unified continuity+intent control plane). **D3** — harness_hub is the
Front-Door interpreter (intent → model-ready language). Source: `findings/resolved-decisions.md`,
`findings/fleet-north-star-map.md`. Supersedes the cycle-1 "two competing NORTH-STAR docs, neither
propagated" gap by converging on ONE source at the `$META_ROOT`+handoff location.

---

## 1. The organ map (one system mid-assembly)

```
                         user intent (owner: non-technical direction)
                                          │
                                          ▼
                          ┌──────────────────────────────┐
                          │ harness_hub  — FRONT-DOOR      │  D3: transforms intent
                          │ INTERPRETER (intent→model lang)│  → model-ready language
                          └───────────────┬───────────────┘
                                          │ interpreted intent
                                          ▼
   INTENT plane                 ┌──────────────────────────────────────┐   CONTINUITY plane
   ┌──────────────┐  curated    │  handoff  — THE UNION HOME             │   (who/when/proof)
   │  rusty-idd   │  spec/why   │  witnessed contract + deterministic    │
   │ (why / what) │ ⇄ events on │  classifier + ledger (redb + SHAKE-256 │
   │  OpenSpec    │  one log    │  witness, UNSIGNED) + real-teeth gates │
   └──────────────┘            └───────────────┬───────────────────────┘
                                                │ claims + leases
                                                ▼
   MEMORY axis (no unified recall yet)   ┌─────────────┐  TRANSPORT (distinct plane)
   ┌──────────┐ ┌──────────┐ ┌────────┐  │  weave A2A  │  degrades to ledger-only offline
   │   icm    │ │ ruvector │ │ git-kb │  └──────┬──────┘
   │ why/dec. │ │ vector/  │ │ code   │         ▼
   │ (absent  │ │ RVF (dead│ │ graph  │   models / DISTRIBUTED COMPUTE
   │ in prod) │ │ overlay) │ │ (.kb)  │   (workstation/server today; mobile/Pi/ESP32
   └──────────┘ └──────────┘ └────────┘    are leaf nodes a host PROXIES for — DC-2)
                                          envctl = environment + credential plane (installs INTO meta)
```

**Organs (from the deep-read fleet map + this cycle's verdicts):**

| Organ | Role | Axis / plane | State (evidence) |
|---|---|---|---|
| **rusty-idd** | intent control plane (why/what; OpenSpec lifecycle) | intent | partially-wired — no confirmed code edge to hf/weave; folds into the union (A-C8, union-1) |
| **handoff** | **the union** — continuity kernel: witnessed contract + deterministic classifier + ledger; real-teeth gates | continuity / execution / governance substrate | core-substrate; **non-standalone** until RuVector resolved (EXP-1) |
| **harness_hub** | Front-Door interpreter (intent→model language) | front door | the interpreter that writes INTO handoff (D3, pa §6) |
| **weave** | A2A transport / heartbeat / leases | communication | core-substrate; transport plane, distinct from witnessed-receipts (rp-A2A) |
| **icm** | decision / "why" / error / preference memory | memory | absent from product code (mem-4); to be introduced (mem-U5) |
| **ruvector** | vector compute / RVF ledger overlay / formal verification | memory-vector + runtime | crates-only; in handoff the RVF overlay is DEAD (0 callers, SHA3 pseudo-embeddings — mem-1/2) |
| **git-kb** | code intelligence + embeddings (`.kb/`) | memory-code | handoff IS a first-class member (committed `.kb/`); rusty-idd is not (mem-5) |
| **envctl** | environment + credential injection (no system-depth installs) | runtime/credential | core-substrate (run-from per D2) |

**The axes** (the dimensions the fleet converges toward): persistent memory/vector-intelligence ·
constant auto-research · rules/policy/agent-org + A2A · Rust(+Lua-in-executor-repos-only) runtime ·
distributed compute · multi-vendor local+cloud mesh.

## 2. The triad — Integrity · Reversibility · Capability-Gain

Every agent action and every plan row is judged against this triad (handoff AGENTS.md:15-20 intent;
made the explicit acceptance shape of every UPGRADE row this cycle):

- **Integrity** — the witnessed baseline is never corrupted. Fail-closed gates (`exit(1)` under
  `fail_mode="block"`, rp-teeth), the committed-JSONL cold-start that re-verifies the witness chain
  (mem cold-start), NO-C-in-the-trust-path. An action that cannot prove it preserved integrity is
  refused.
- **Reversibility** — every change is revertible (git history, feature flags, additive-only). The
  filesystem `.handoff/` contract is the always-available offline fallback under every transport
  upgrade.
- **Capability-Gain** — strict-upgrade-only / no-downgrade: every action increases verified
  capability. The superset wins on merge; the hardening lint is re-applied; a silent model/provider
  downgrade is a guard violation (UP-4 / No-Downgrades).

This triad is the machine-checkable restatement of "Upgrade Only" — currently north-star *intent*
with no enforcing gate (rp-upgrade-only-is-intent); the plan proposes turning it into witnessed gates
(UP-2, UP-4, the standalone/golden/loader gates).

## 3. HOW repos bind to it as data (proposed mechanism)

The cycle-1 failure was that the vision lived only in the meta-root `NORTH-STAR.md`, which no member
(each its own git repo) can `cat`, and handoff carried a *second*, different one (Gap #1,
fleet-north-star-map §4). The proposed binding mechanism — **bind-as-data, not hardcode-as-prose:**

1. **Single source @ `$META_ROOT` + handoff (D1).** The canonical north-star artifact is co-located
   with the kernel every repo can already reach (`hf` / `.handoff`) and the meta root. ONE source,
   not per-skill prose.
2. **Carried in the witnessed capsule, not a string literal.** handoff already derives a packet's
   north-star from `.handoff/context/capsule.json` (`northstar`), NOT a hardcoded string (ADR-0006,
   pa §1). The proposal: this artifact is the data that capsule field points at, so every rendered
   packet binds the live north-star revision.
3. **Drift-invalidated when doctrine changes.** `handoff-drift` already treats the `northstar` surface
   as a 5th intent-lock surface and re-mints a card minted against a superseded doctrine revision
   (`current_northstar_revision()`, ar C13). The proposal: bumping this artifact's revision
   fail-closed-invalidates stale intent fleet-wide — the north-star is enforced, not advisory.
4. **Per-repo "you are organ N of system M" pointer.** Each member's `.handoff` capsule names its
   organ + axis from §1 (resolving Gap #1's "no per-repo pointer"), read at session start by the
   continuity-navigator; harness_hub (the interpreter) reads the same artifact to interpret intent
   against the live organ map (D3 — "binds in a meta-level layer the skill reads," not inside one
   skill).
5. **A2A-shareable.** Because it is data in the witnessed capsule, weave/A2A can carry the north-star
   revision as a signed artifact between agents (trends §C1 signed-card direction) without fusing the
   transport and receipts planes.

## 4. Open / unresolved for owner canon (do not assume)

- The **ledger read-API** (the witnessed-read seam every organ needs) is unbuilt (Step 4 of the union
  plan) — the binding mechanism's read path depends on it.
- **Three memory systems, no unified recall** (icm / ruvector / git-kb + rusty-idd `.idd/knowledge`) —
  mem-U6 facade is PROPOSE; the memory axis has no convergence layer yet.
- **48 of 68 fleet members are not on the hf kernel** (fleet-north-star-map §1) — propagation of this
  artifact beyond the 20 hf-wired repos is a fleet-rollout decision, owner-gated.
- This artifact is **DRAFT/PROPOSE** — it is owner canon and is not written into meta-root or handoff
  canon without approval.
