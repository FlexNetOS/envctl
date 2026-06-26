# plan-synthesis — ASCII diagram legend, ADR template, ROADMAP + UPGRADE row formats

The exact conventions the `plan-architect` uses to render diagrams and promote the plan. Source for the
diagram conventions: envctl `docs/runbook/DIAGRAMS.md`.

## 1. ASCII diagram legend (from envctl `docs/runbook/DIAGRAMS.md`)

**Box-drawing characters** — use these, never ASCII-art `+---+`/`|`:

```
  corners/edges : ┌ ─ ┐  │  └ ─ ┘
  tees/cross    : ┬ ┤ ├ ┴ ┼
  arrows        : ▶  ◀  ▲  ▼     (and ──▶ for a directed edge)
  annotation    : * hotspot   ⚠ cycle/violation   (c=N) centrality
```

**Automation legend** — tag a box or arrow when the step it represents is automated or gated:

```
  [A]  AUTOMATED          — agent/loop runs it unattended; no human needed
  [A*] AUTOMATED-ELEVATED — runs sudo, but sudo -n is passwordless → still unattended
  [P]  PREVIEW-BY-DEFAULT — runs dry-run; a human (or an apply flag) must opt in to mutate
  [H]  HUMAN-GATED        — a human MUST trigger it (reboot, live migration, secret reveal)
  [!!] SUPERVISED/CRITICAL— the loop REFUSES to auto-run; writes NEEDS-HUMAN and stops
```

**Cite every diagram** with `Source: file:section` directly above or below it — the graph snapshot
(`graph/<T>.callgraph.json` / `graph/<T>.graph.md`), the codemap (`reports/codemap-<T>.md`), or a docs
section (`docs/ARCHITECTURE.md §N`, `docs/secrets/*.md §N`). A diagram with no source is a guess.

### Worked example (architecture + graph intelligence)

```
Source: graph/secrets-proto.callgraph.json @ a97c96a ; reports/codemap-secrets-proto.md

   entrypoints                    core (hotspots *)               leaves / store
   ┌────────────────────┐   ┌───────────────────────────┐   ┌────────────────────┐
   │ grpc:mint_github [A]│──▶│ SecretService (pub API)   │──▶│ vault_open          │
   │ grpc:vault_crud  [A]│   │ MintReq *      (c=8)       │   │ libSQL remote (noC) │
   └────────────────────┘   └────────────┬──────────────┘   └────────────────────┘
                                         │  ⚠ SCC [encode,decode]  (cycle, size 2)
                                         ▼
                            ┌───────────────────────────┐
                            │ legacy_encode   (dead)     │   blast: 0 dependents
                            └───────────────────────────┘
   [P] reveal/mint paths are preview-gated · [!!] key reveal is supervised
```

Render *behavior + connection*, not just names. Lift structure from `graph/<T>.graph.md`; add the
hotspot/cycle/layering annotations from `graph/<T>.metrics.json`; tag automation where a flow step is
`[A]/[P]/[H]/[!!]`.

## 2. UPGRADE row format (reused in the sequenced roadmap)

Each roadmap entry is the analyst's UPGRADE row that cleared the verifier, in this exact shape:

```
- UPGRADE: <change> | axis: quality|speed|accuracy | rationale: <why> | evidence: <path:line> | blast: <impact-scope> | risk: low|med|high | verdict: <ref> CONFIRMED|QUALIFIED(<cond>)
```

- `axis` — exactly one of quality / speed / accuracy.
- `blast` — from `metrics.blast_radius` for the touched file (how far the change reaches).
- `risk` — low/med/high, informed by blast + invariant-proximity.
- `verdict` — the `findings/verdicts.md` ref that confirmed/qualified it. **No verdict ⇒ not in the
  roadmap.**

Order the roadmap by value/risk: high-centrality (`metrics.hotspots`) + bounded-blast + low-risk first;
high-blast/high-risk later with the dependency noted.

## 3. ROADMAP row format (`docs/ROADMAP.md` promotion)

Append ONE row per planned target — an index pointing at the canonical plan under `reports/`:

```markdown
| <YYYY-MM-DD> | <T> | <verdict headline, 1 line> | <conf High/Med/Low> | quality:N speed:N accuracy:N | [plan](../.handoff/loop/plan/reports/<T>-plan.md) |
```

If `docs/ROADMAP.md` lacks a table, create one with this header first, then append:

```markdown
# Roadmap

| Date | Target | Recommendation | Confidence | Upgrades (by axis) | Plan |
|------|--------|----------------|------------|--------------------|------|
```

The plan under `reports/` is canonical; ROADMAP is the discoverable index. Never duplicate the full
plan into ROADMAP.

## 3b. Feature-Forge test-build row (the testing handoff)

The planning loop **designs** the test suite; **Feature Forge builds + runs it** (the loop is
read-only on production code — it never writes/runs tests). Append ONE test-build row to
`docs/ROADMAP.md` per target that needs new tests — an FF-ready backlog item indexing the plan's
`## Test Strategy & Coverage` section (which carries the full `## FF test-build spec`):

```markdown
| <YYYY-MM-DD> | <T> | test-build: <N> cases closing <M> coverage gaps (unit/integration/e2e/golden) | accuracy:N quality:N | → Feature Forge | [test plan](../.handoff/loop/plan/reports/<T>-plan.md#test-strategy--coverage) |
```

The row is shaped so Feature Forge's `feature-architect` reads the linked *Test Strategy & Coverage*
section straight into its **`## Verification plan`** (which tests to add + which CI gates) — then
`rust-implementer` writes them and `invariant-guardian` runs + gates them. This row **is** the
creation+implementation handoff: planning-engineer never writes test code. List only **verified**
coverage gaps + **feasibility-passed** test designs (the verifier dropped the rest).

## 4. ADR template (draft — ONLY for a genuine architecture decision)

Emit only when the plan settles a real architectural choice (a new boundary, a backend swap, a contract
change). A routine upgrade list is **not** an ADR — do not manufacture one. Pick the next free `####`
in `.handoff/decisions/` and write `.handoff/decisions/ADR-####-<slug>.md`:

```markdown
# ADR-####: <decision title>

- Status: draft        <!-- draft → accepted (owner) → superseded; never self-accept -->
- Date: <YYYY-MM-DD>
- Target: <T>
- Plan: ../loop/plan/reports/<T>-plan.md

## Context
<the forces — what the graph + research + verified findings show; cite verdict refs / path:line / research sources>

## Decision
<the single architectural choice being proposed>

## Consequences
<positive + negative; the blast-radius from metrics.json; what it commits us to; what it forecloses>

## Alternatives considered
<options weighed, incl. any feasibility-REFUTED option and *why* it was excluded — e.g. "in-process C cache: violates no-C-in-trust-boundary">

## Invariants check
<explicitly confirm the decision respects the NON-NEGOTIABLE invariants (no C in trust boundary, one rustls ring-only, engine single shared lib, destructive ops fail-closed) — or NEEDS-HUMAN>
```

Status stays `draft`; the **owner** accepts. The ADR is a docs artifact — committed with the plan, never
applied to production code by synthesis.

## 5. What synthesis writes (and only this)

- `.handoff/loop/plan/reports/<T>-plan.md` — the canonical plan (incl. the *Test Strategy & Coverage* section).
- `docs/ROADMAP.md` — one appended index row, plus a **test-build row** (the Feature-Forge handoff) when the target needs new tests.
- `.handoff/decisions/ADR-####-<slug>.md` — a draft ADR, only for a genuine architecture decision.

`git status` after the cycle should show only `.handoff/` + `docs/ROADMAP.md` (+ the draft ADR). Any
production-code change is a contract violation.
