# Findings — rules-policy-org — Target: prompt-hub

Axis: `rules-policy-org`. Target repo: `/home/drdave/Desktop/meta/prompt_hub`.
Read-only audit. Every CLAIM/UPGRADE row cites positive evidence (`file:line`, source-ledger
row, or dated doc). Fail-closed: an absent control is recorded as a gap, never a pass.

prompt_hub's role in the **agent org chart**: it is the **Front-Door intent STORE** — the durable,
governed half of the two-layer intent front door (the `harness_hub` *interpreter* is the other
half). Its RBAC + tamper-evident audit + lineage are not generic app security; they are
**governance of intent itself** — who may author/mutate a prompt (intent), proof that the mutation
happened, and the ancestry of how an intent evolved. It feeds `rusty-idd` via goal/prompt artifacts
(two-layered intent front door, owner decision D3; rusty-idd ADR-0007 boundary).

---

## 1. Policy table (owner standing rules)

| # | Policy | Status in prompt_hub | Evidence (file:line) |
|---|--------|----------------------|----------------------|
| P1 | **Upgrade Only** / strict-upgrade-only | Stated as repo law; the `.handoff/` migration was done as upgrade-only | `prompt_hub/CLAUDE.md:146` ("no downgrades, upgrade only"); `prompt_hub/docs/plans/lifeos-meta-front-door.md:24-25,162-163` |
| P2 | **No Downgrades** — no destructive reset, no legacy removal until replacement installed+configured+parity-proven | Stated repo-wide; also enforced by the destructive-command guard rule | `prompt_hub/docs/plans/lifeos-meta-front-door.md:162-166`; `.claude/rules/meta-destructive-commands.md` (forbids `git reset --hard`, `git clean -fd`, `rm -rf`, force-push w/o lease) |
| P3 | **No silent model/provider downgrade** | Dual-model accuracy strategy: background lanes run weave→Opus; fail closed if Opus worker unobtainable | `prompt_hub/prompts/fleet-convergence-first-run.md:111-114` |
| P4 | **Strict parity before removal** (Rust-native invariant) | Code is the contract; prose may be stale/foreign — transform to Rust-native, never honor drift; `#![forbid(unsafe_code)]` crate-wide | `prompt_hub/CLAUDE.md:9-25,83`; `prompt_hub/.agent.md:93` |
| P5 | **Automate everything researchable** | Self-evolving plan-loop + per-cycle self-eval/self-upgrade; audit-report → task-ledger automation | `prompt_hub/prompts/fleet-convergence-first-run.md:115-116`; `prompt_hub/README.md:161-167` (audit_sync.yml + audit_watcher.sh) |
| P6 | **Human only at explicit supervised/risk boundaries** | Owner walls → NEEDS-HUMAN (physical/account/irreversible/scope-expanding); first run capped `cycle_budget=1`, owner reviews before unattended | `prompt_hub/prompts/fleet-convergence-first-run.md:105-106,69-73,118-122` |
| P7 | **Commit/push/PR discipline** | Conventional Commits; feature branch from `main`; full validation before push; PR needs ≥1 approval; worktree-only commits enforced by pre-commit hook | `prompt_hub/CONTRIBUTING.md:114,164,172,180,186`; `:61-62` (pre-commit enforces worktree-only + lint/test gate) |
| P8 | **Worktree-per-task** (no work on shared `main`) | Stated project workflow; each named agent owns a dedicated worktree+branch | `prompt_hub/CLAUDE.md:91`; `prompt_hub/AGENTS.md:22,44,65,…` |
| P9 | **Fail-closed evidence rule** | Green exit / empty result / missing file = finding, never a pass; every claim cites positive evidence | `prompt_hub/prompts/fleet-convergence-first-run.md:101-104` |

CLAIM-P1 [CONFIRMED]: prompt_hub treats `Upgrade Only` / `No Downgrades` as a binding owner law, not
aspiration — the `_workspace/` → `.handoff/` state migration is recorded explicitly as
"no downgrades, upgrade only" (`prompt_hub/CLAUDE.md:146`) and the integration plan restates it as
governing "the whole integration" (`docs/plans/lifeos-meta-front-door.md:24-25`).

CLAIM-P7 [CONFIRMED]: commit/push/PR discipline is mechanically enforced, not just documented — a
version-controlled `pre-commit` hook (`core.hooksPath .githooks`) enforces worktree-only commits and
runs the lint/test gate (`CONTRIBUTING.md:61-62`); the live history shows the push→PR→squash-merge
pipeline in use (`#182`, `#181`, `#180` in `git log`).

---

## 2. Agent org chart

Two distinct org charts coexist in this repo and must not be conflated.

### 2a. prompt_hub's product/build crew (the harness that BUILDS prompt_hub)

```
                         owner (escalation: NEEDS-HUMAN wall)
                                     │
                        prompt-loop orchestrator (commander)
                                     │
   ┌──────────────┬─────────────────┼───────────────┬────────────────┐
backlog-curator  feature-architect  rust-implementer  verification-gate  docs-scribe
  (bookend)        (design)          (build) ⇆ (gate, refute loop)       (record)
                                     │
                          continuity-steward (HANDOFF bookend)
                                     │
                          evolution-steward (per-run retro → LESSONS.md → harness upgrades, fail-closed)
```

Evidence: `prompt_hub/CLAUDE.md:121-132` (crew: `feature-architect` → `rust-implementer` ⇆
`verification-gate` → `docs-scribe`, with `backlog-curator` + `continuity-steward` bookends);
`prompt_hub/CLAUDE.md:147` + `.claude/agents/evolution-steward.md:84` (evolution-steward at
DONE/HAND OFF); `.claude/agents/verification-gate.md:53` (escalate to leader if refute loop > ~2–3
rounds without convergence — explicit escalation path).

### 2b. prompt_hub's PRODUCT model — the agents it governs intent FOR

prompt_hub ships an in-repo multi-agent dev model (Alpha…Theta), each owning a file slice + a
dedicated git worktree/branch, integrated by an orchestrator. This is the **org chart of the swarm
whose intent prompt_hub stores and governs**.

```
                         Orchestrator ("Junie" in-repo orchestrator hook)
                                     │
  Alpha(core types) → Beta(storage/config) → Gamma(security/audit) →
  ┌──────────────┬──────────────┬──────────────┐
  Delta(search/sync) Epsilon(automation) Zeta(advanced)   (parallel specialists)
                                     │
                              Eta(hub/metrics)  ── integrates all ──▶  Theta(server/CLI)
```

Evidence: `prompt_hub/AGENTS.md:7-199` (registry Alpha→Theta, each with worktree+branch);
`:204-227` (orchestrator owns setup/merge; dependency ordering); `prompt_hub/CLAUDE.md:87-89`
("Junie" in-repo orchestrator agent: `junie.rs`, `hooks.rs`, CLI `commands/junie.rs`).

CLAIM-ORG1 [CONFIRMED]: prompt_hub is the **Front-Door intent STORE** in the fleet **agent org
chart**, paired with `harness_hub` as the interpreter; the two-layer intent front door is an owner
decision (D3, 2026-06-26): "harness_hub is the front-door *interpreter*… and prompt_hub is the
durable intent *store + boundary* (ADR-0007). Together they feed rusty-idd"
(`prompt_hub/docs/plans/lifeos-meta-front-door.md:31-37`). The integration DAG places
`prompt_hub ──intent/prompt──▶ rusty-idd ──ready goal/spec──▶ planning_engineer`
(`:62`, ordering rule `:79-81`).

CLAIM-ORG2 [CONFIRMED]: prompt_hub's governance-of-intent is implemented, not nominal. Every
mutating Hub operation follows sanitize → RBAC authorize → storage → tamper-evident audit:
`register` runs `RbacAuthManager::authorize_action(identity, Action::Write)` then sanitizer then
`log_audit` (`prompt-hub/src/hub.rs:913-939`); `get`/`get_by_id` gate on `Action::Read`
(`:987,:1014`); `lock` gates on `Action::Lock` (`:1205`). RBAC capability mapping is explicit —
Read/Write/Admin with Admin-superset (`prompt-hub/src/auth.rs:86-109`), ownership transfer gated on
`Action::Transfer` (`:121-137`).

CLAIM-ORG3 [CONFIRMED]: the audit trail is tamper-evident (SHA-256 hash chain), supporting
governance/lineage of intent: `compute_diff_hash = SHA256(before_json || after_json || timestamp)`
(`prompt-hub/src/audit.rs:64-92`), `verify_entry_integrity` recomputes and flags mismatch
(`:97-115`), and GDPR right-to-erasure anonymizes without breaking the chain (`:116-137`). Lineage is
a first-class subsystem: `LineageTracker` with `get_lineage_ancestry`/`detect_lineage_forks`/
`get_lineage_descendants`/`build_lineage_tree` (`prompt-hub/src/hub.rs:1928-1979`).

---

## 3. A2A / weave communication map

prompt_hub itself is library/CLI/server (synchronous request→response over its Hub façade); it does
**not** embed a live `weave`/`A2A` transport in product code. The **A2A** layer is the harness/fleet
plane around it. The map:

| Sender → Receiver | Channel | Cadence / rule | Evidence |
|---|---|---|---|
| envctl-plan-orchestrator → background Opus workers | `weave` → Opus (dual-model) | heavy research/code-mapping/governance scans run as **background** lanes; fail-closed if Opus unobtainable | `prompt_hub/prompts/fleet-convergence-first-run.md:111-114`; `prompts/plan-loop-parallel-run.md:18,61` |
| plan-loop instance ↔ plan-loop instance | `weave lease reserve/release` (resource `plan:claim:<target>`, TTL 1800) | prevents duplicate work; degrades to **ledger-only** with visible warning when weave absent (never silent) | `prompts/plan-loop-parallel-run.md:47-53,110,138` |
| loop end → successor session | weave session event + `session-relay` handoff | session-handoff wired at end-of-loops; committed HANDOFF is authoritative, weave is heartbeat only | `.kb/.../tasks/lane-loop-handoff.md:12-20`; `prompt_hub/CLAUDE.md:124-125` |
| transport resolution | `WEAVE_BIN` → PATH → `$META_ROOT/weave/target/{release,debug}/weave` | cwd-independent; weave is the transport the loop USES, not where it lives | `prompts/fleet-convergence-first-run.md:36-39`; `prompts/plan-loop-parallel-run.md:31-32` |
| fleet north-star | `weave` = the communication layer / **A2A** nervous system | A2A / background transport, a plane *distinct* from handoff's witnessed receipts | `prompts/fleet-convergence-first-run.md:15,19`; `prompts/plan-loop-parallel-run.md:15-18` |

CLAIM-A2A1 [CONFIRMED]: weave is the fleet's `A2A` / agent-to-agent nervous system and is a plane
distinct from handoff's witnessed receipts (`prompts/plan-loop-parallel-run.md:15-18`); prompt_hub's
loops USE it as transport (resolved via `WEAVE_BIN`/PATH) rather than hosting it
(`prompts/fleet-convergence-first-run.md:36-39`).

CLAIM-A2A2 [CONFIRMED]: missed-message / duplicate-work prevention is a weave lease keyed on a
slash-free resource (`plan:claim:<target>`) for exact-match detection, with explicit degrade-visibly
(never-silent) fallback to ledger-only when weave is unavailable
(`prompts/plan-loop-parallel-run.md:48-53`). Foreground-chat non-blocking rule: heavy work is pushed
to **background** lanes so the foreground stays responsive
(`prompts/fleet-convergence-first-run.md:111-114`).

GAP-A2A3 [CONFIRMED gap]: prompt_hub has **no background-agent status ledger of its own** for its
product runtime; status/heartbeat is delegated to the meta/handoff ledger + weave heartbeat
(`prompt_hub/CLAUDE.md:108-119` — federated per-repo `.handoff/ledger.db` feeds central
`meta/.handoff`). prompt_hub also has an internal swarm-handoff helper set
(`generate_swarm_bundle`/`generate_handoff_template`/`generate_full_handoff_chain`,
`prompt-hub/src/swarm.rs:122-228`) that produces handoff *templates* but is **not** wired to a live
A2A transport — it is content generation, not messaging.

---

## 4. Replacement-of-human-bottleneck plan

| Manual action | Category | Evidence / rationale |
|---|---|---|
| Sync audit reports → repo task ledger | **Automate now** (done) | `audit_sync.yml` CI + `scripts/audit_watcher.sh` local (`README.md:161-167`) |
| Per-cycle plan retro / lessons capture | **Automate now** (done) | `evolution-steward` runs harness-evolution at DONE/HAND OFF → `LESSONS.md` (`CLAUDE.md:147`) |
| Background research / code-mapping / governance scans | **Automate now** (done) | dual-model **background** lanes via weave→Opus (`prompts/fleet-convergence-first-run.md:111-114`) |
| Duplicate-work avoidance across parallel loops | **Automate now** (done) | weave lease claim (`prompts/plan-loop-parallel-run.md:47-53`) |
| Lint/test gate before commit | **Automate now** (done) | pre-commit hook runs `scripts/code_review.sh` (`CONTRIBUTING.md:61-62`) |
| Verification of implementer output | **Supervised** | `verification-gate` agent refutes; escalates to leader after ~2–3 non-converging rounds (`.claude/agents/verification-gate.md:53`) |
| Unattended continuation past cycle 1 | **Owner-only** | first run capped `cycle_budget=1`; owner reviews before loop runs free (`prompts/fleet-convergence-first-run.md:69-73,118-122`) |
| PR merge approval | **Owner/supervised** | PRs require ≥1 approval (`CONTRIBUTING.md:186`); `/prompt-loop` auto-merges only on green DONE-gates, fail-closed to NEEDS-HUMAN (`CLAUDE.md:135-139`) |
| Physical / account / irreversible / scope-expanding actions | **Owner-only** | owner-wall → NEEDS-HUMAN (`prompts/fleet-convergence-first-run.md:105-106`) |
| Legacy-tool removal | **Owner-only / gated** | No removal until replacement installed+configured+parity-proven (`prompts/fleet-convergence-first-run.md:107-109`) |

---

## 5. Upgrade rows (`axis: rules-policy-org`)

| ID | Upgrade | Evidence (current gap) | Acceptance | Risk | Reversibility |
|----|---------|------------------------|------------|------|---------------|
| U1 | Publish prompt_hub's **role in the agent org chart** as repo-local data (not only in envctl skill prose) — a committed `docs/ORG-CHART.md` / front-door-store doc stating "intent STORE half, harness_hub = interpreter, feeds rusty-idd (ADR-0007)" | The two-layer front-door framing lives in a *plan* doc (`docs/plans/lifeos-meta-front-door.md:31-37`) + envctl skills, not in a normative prompt_hub policy file; `prompt_hub/CLAUDE.md:87-91` describes only the in-repo Alpha…Theta crew, not the fleet role | `docs/` file states prompt_hub's fleet role + binds to north-star as data; referenced from `CLAUDE.md` | low | delete doc |
| U2 | Wire the in-repo swarm-handoff helpers (`generate_handoff_template`/`generate_full_handoff_chain`, `prompt-hub/src/swarm.rs:179-228`) to emit/consume a **weave**/`A2A`-compatible envelope so handoff *content* can ride the live transport | helpers produce markdown templates only; no transport binding (GAP-A2A3) | a Hub API or CLI verb renders a handoff bundle into the weave message/lease envelope schema; round-trip test | med | feature-flag the bridge |
| U3 | Add a prompt_hub-local **background-agent status surface** (project handoff-ledger state into a queryable Hub/server endpoint) so the intent STORE exposes its own background lane status, not only via meta/handoff | status delegated entirely to meta/.handoff + weave heartbeat (`CLAUDE.md:108-119`); no local projection | `prompthub-server` route exposes background/loop status JSON; matches the LifeOS "handoff status projection" missing seam (`docs/plans/lifeos-meta-front-door.md:125,151`) | med | drop the route |
| U4 | Encode the **Upgrade Only** / **No Downgrades** + commit/PR discipline as machine-checkable policy (extend the pre-commit/CI gate to reject downgrade-shaped diffs: dependency downgrades, deleted-without-replacement modules) | rules are prose (`CLAUDE.md:146`, `lifeos-...:162-163`) + a destructive-command guard; no positive downgrade-detection in CI | CI job fails on a crafted downgrade diff; passes on a clean upgrade | med | revert CI job |
| U5 | Make the **A2A**/weave degrade-visibly contract a tested invariant in prompt_hub's loop scripts (assert ledger-only fallback emits a warning, never silent) | the rule is documented (`prompts/plan-loop-parallel-run.md:52-53`) but not asserted by a prompt_hub-owned test | a script/contract test forces weave-absent and asserts a visible warning + ledger-only claim | low | remove test |

All upgrades preserve `Upgrade Only` / `No Downgrades`: each is additive (new docs, new feature-gated
bridge, new endpoint, new CI/test gate) and reversible; none removes an existing capability or
weakens a gate.

---

## Verification notes

- RBAC/audit/lineage claims were read directly from source, not inferred: `prompt-hub/src/auth.rs`,
  `prompt-hub/src/audit.rs`, `prompt-hub/src/hub.rs` (line ranges cited per row).
- Convergence-seam claims (two-layer front door, ADR-0007 boundary, intent→rusty-idd flow) are
  grounded in `docs/plans/lifeos-meta-front-door.md` (synthesized from the committed rusty-idd
  plan-loop artifacts; source keys `[P1][P2][R1]` resolve to `prompt_hub/README.md` and
  `prompts/README.md` ranges, `lifeos-meta-front-door.md:189-190`).
- `weave`/`A2A` claims are grounded in `prompts/fleet-convergence-first-run.md` and
  `prompts/plan-loop-parallel-run.md`; prompt_hub product code contains no live weave client (the
  transport is the fleet plane around it, used by the loop scripts).
- Source-ledger context: target/dimension state under
  `.handoff/loop/plan/{targets.md,dimensions.md,loop_state.md}` (worktree `plan-prompt-hub/envctl`).
