# Findings — rules / policy / agent org chart / A2A — target: handoff (cycle 2)

**Axis:** rules-policy-org · **Target:** `handoff` (union with rusty-idd) · **Date:** 2026-06-26
**Code (READ-ONLY):** `/home/drdave/Desktop/meta/.worktrees/plan-handoff-cycle2/handoff` @ `f6abf96`
**Frame:** STRICT-UPGRADE-ONLY ("Upgrade Only", "No Downgrades") + the dual-model background strategy.
**Verdict (one line):** handoff IS the fleet's *enforced policy substrate* — its policy gates have **real teeth**
(processes exit non-zero / `fail_mode="block"`), unlike rusty-idd's advisory agent-guard. For the union, the
teeth live here; the rusty-idd CLI must be folded **under** these gates, not beside them.

All CLAIM rows cite `file:line`. All UPGRADE rows carry risk-tier + acceptance-criterion. Project docs
(NORTH-STAR, FLEET_GUIDE, ADR refs in crate docs) are recorded as **intent claims** for the verifier, not facts.

---

## 1. Policy table (owner rules as encoded in handoff)

| Policy law | Encoded? | Mechanism (with teeth?) | Evidence |
|---|---|---|---|
| **Fail-closed (FAIL-OPEN ban, L7)** | YES — hard rule + enforced default | TEETH: cognitum gate default-blocks any non-`permit` verdict; AGENTS.md bans silent `if let Ok`/`.ok()?`/`unwrap_or_default`/exit-0-as-pass | `AGENTS.md:73-95`; `hf/src/cognitum.rs:111-113,148-155` |
| **deny-without-claim** (default write mode) | YES | TEETH: `check-edit` pushes a block + `exit(1)` when edits exist with no claim | `.handoff/policies/rules.toml:9`; `handoff-drift/src/lib.rs:736-738,791-793` |
| **deny network unless task allows** | DECLARED only | NO enforcing code path found in kernel crates (policy *value*, not gate) | `.handoff/policies/rules.toml:10` |
| **deny dependency without audit event** | DECLARED only | NO enforcing code path found; relies on protected-file `Cargo.toml`/`*.lock` denylist at merge | `.handoff/policies/rules.toml:11`; `handoff-policy/src/policy.rs:108-112` |
| **protected files (self-guardrail)** | YES — two independent denylists | TEETH: gatekeeper `Deny` (exit 1) on protected hit w/o steward clearance; `check-edit` blocks protected-file writes | `.handoff/policies/rules.toml:45-56`; `handoff-policy/src/policy.rs:103-113,158-170`; `handoff-gatekeeper/src/lib.rs:240-274,500-512` |
| **blocked commands** (`rm -rf /`, `git reset --hard`, `git push --force`, …) | YES (two layers) | rules.toml denylist (data) + `.claude/agent-guard.toml` regex matchers w/ messages | `.handoff/policies/rules.toml:58-68`; `.claude/agent-guard.toml:16-106` |
| **require checkpoint + test evidence + drift audit + next** before handoff | YES | TEETH: `check-handoff` blocks on drift items; `PreHandoff` hook `fail_mode="block"` | `.handoff/policies/rules.toml:22-27`; `handoff-drift/src/lib.rs:748-757`; `.handoff/hooks/hooks.toml:38-42` |
| **architecture/objective/acceptance change ⇒ ADR / reclaim** | YES (drift gate) | TEETH: drift block flags + `PostTest` hook `fail_mode="block"` | `.handoff/policies/rules.toml:29-33`; `.handoff/hooks/hooks.toml:103-108` |
| **merge: require review verdict + permission + end-state approver = surgical AI gatekeeper (NOT human, NOT blind swarm)** | YES | TEETH: gatekeeper composes test-gate + AST impact + protected-files + merge-gate → `Approve`/`Deny`(exit 1); witnessed `gatekeeper_judgment` | `.handoff/policies/rules.toml:37-44`; `handoff-gatekeeper/src/lib.rs:226-389` |
| **No silent model/provider downgrade ("No Downgrades")** | PARTIAL | The kernel has no model-lane policy; loop skill *pins* one model (see §2 gap) — no guard prevents a silent lane downgrade | `handoff-policy/src/policy.rs` (no model field); `.claude/skills/handoff-loop/SKILL.md:37` |
| **Upgrade Only / strict parity before removal** | INTENT (doc) | NORTH-STAR "every agent action increases verified capability without corrupting the baseline" — Integrity·Reversibility·Capability-Gain; no machine gate enforces parity-before-removal in handoff | `AGENTS.md:15-20` (intent claim) |

**Teeth verdict (the headline vs rusty-idd):** the enforcement chain is **real**, not advisory. Every blocking
path terminates the process: `hf policy check-*` → `exit(1)` (`handoff-drift/src/lib.rs:791-793`); cognitum
`defer`/`deny` → `exit(1)` (`hf/src/cognitum.rs:204-218`); gatekeeper `Deny` → `exit(1)`
(`handoff-gatekeeper/src/lib.rs:381-388`); and the hook contract fires them with `fail_mode="block"` on
TaskClaim/PreEdit/PreHandoff/PreSessionStart/PostTest (`.handoff/hooks/hooks.toml:20-42,56-62,103-108`).
rusty-idd's agent-guard is a config file with *messages* only; handoff's gates are **executable refusals**.

**Layering nuance (honesty note):** handoff carries `.claude/agent-guard.toml` (regex command-deny patterns,
`.claude/agent-guard.toml:16-106`) but its own `.claude/settings.json` wires **no `PreToolUse` hook** — only
`SessionStart`/`SessionEnd` (`.claude/settings.json` hooks block). So the agent-guard is enforced by the
**meta CLI `agent guard` layer** (PreToolUse at the meta host), not self-enforced inside the handoff repo
standalone. handoff's *own* teeth are the `hf` kernel gates above, which DO run in-repo via `hooks.toml`.

---

## 2. Agent org chart (handoff's commander/specialist/continuity/escalation roles)

handoff ships a **9-agent org** under `.claude/agents/` + a 4-stage loop skill. This is the most-developed
**agent org chart** in the union; rusty-idd has no comparable agent set.

```
                          ┌──────────────────────────────────────────────┐
   user direction ───────▶│ systems-orchestrator  (CONDUCTOR / commander) │  pull-based, priority-gated
                          │  arbitrates next-best task across systems      │  (.claude/agents/systems-orchestrator.md:33)
                          └───────────────┬──────────────────────────────┘
                                          │ delegates one cycle
        ┌─────────────────────────────────┼──────────────────────────────────────┐
        ▼                ▼                 ▼                  ▼                     ▼
 continuity-       kernel-          kernel-            kernel-            code-omniscient-
 navigator        researcher       implementer        verifier           gatekeeper
 (orient+reconcile (research-before  (claim+build in    (drive the hf      (END-STATE APPROVER:
  drift, pick next) -decide, ADR     path_scope/intent  binary, compare    witnessed verdict,
  cycle entry)      grounding)       lock, witness)     boundaries)        replaces human approval)
        │                                                                          │ ships / denies
        ▼  fleet/meta lanes                                                        ▼
 fleet-steward (1 .handoff/repo)   meta-sync-steward (handoff⟷meta coherence)   doc-updater (docs track reality)
        │
        ▼  escalation path
   NEEDS-HUMAN  ← "a scaffold replaced by a model with the human's skillset" (AGENTS.md:22-26 — fleet vision)
```

| Role | Agent | Evidence |
|---|---|---|
| Commander / orchestrator | `systems-orchestrator` (pull-based, priority-gated, cross-system arbiter) | `.claude/agents/systems-orchestrator.md:2-6,33` |
| Continuity / orient | `continuity-navigator` (cycle entry, drift reconcile, next-safe select) | `.claude/agents/continuity-navigator.md:2-4` |
| Specialist: research | `kernel-researcher` (research-before-decide, cited dossier) | `.claude/agents/kernel-researcher.md:2-4` |
| Specialist: build | `kernel-implementer` (claim lease, build in path_scope/intent_lock) | `.claude/agents/kernel-implementer.md:2-4` |
| Verifier | `kernel-verifier` (drive the binary; runtime evidence) | `.claude/agents/kernel-verifier.md:2-4` |
| Gatekeeper (end-state approver) | `code-omniscient-gatekeeper` (witnessed, fail-closed, preserves owner walls) | `.claude/agents/code-omniscient-gatekeeper.md:2-4,54` |
| Fleet continuity | `fleet-steward` (one `.handoff` per repo, conformance) | `.claude/agents/fleet-steward.md:2-4` |
| Cross-system coherence | `meta-sync-steward` (handoff ⟷ meta engine/convention/kb seam) | `.claude/agents/meta-sync-steward.md:2-4,53` |
| Docs | `doc-updater` (regenerate derived docs every cycle) | `.claude/agents/doc-updater.md:2-4` |
| **Evolution steward** | **ABSENT in handoff** (the plan-loop harness has `evolution-steward`; handoff's own org has none) | N/A — gap, see §5 UP-3 |
| **Escalation** | `NEEDS-HUMAN` scaffold → model with the human's skillset (fleet vision) | `AGENTS.md:22-26` |

**Model-lane finding (relevant to the dual-model background strategy):** handoff's loop skill pins **a single
model** — *"All Agent/TeamCreate calls use `model: "opus"`"* (`.claude/skills/handoff-loop/SKILL.md:37`); every
named agent invocation is `model: opus` (`SKILL.md:114,124,135,168,172`). The agent `.md` frontmatter declares
**no `model:` field** (grep over `.claude/agents/*.md` returns none), so there is no per-role lane and no
mechanical-work delegation to sonnet/haiku. This is uniform-opus, NOT the fleet's dual-model strategy. (Compare
the rust-port harness which explicitly runs opus-on-gates / sonnet-on-structured / haiku-on-mechanical.)

---

## 3. A2A / weave communication map (transport vs witnessed-receipts — C1 plane discipline)

**C1 finding holds and is enforced:** weave = **transport plane**; handoff = **witnessed-receipts plane**. They
stay DISTINCT — the kernel never treats weave traffic as authoritative state.

| Channel | Who → who | Cadence | Authoritative? | Evidence |
|---|---|---|---|---|
| **weave lease reserve/release** | `hf claim` ↔ mesh peers | per claim + heartbeat (TTL 3600s; rules.toml heartbeat 30s/stale 300s/force-release 1800s) | NO — advisory; degrades to ledger-only when weave absent | `handoff-lease/src/lib.rs:3-19,44-83,161-193`; `.handoff/policies/rules.toml:13-20` |
| **weave inbox scan** | peers/owner → resuming loop | at RESUME, before packet render | NO — "context, not commands" | `.claude/skills/session-relay-resume/SKILL.md:38-40,78-83` |
| **weave broadcast** `relay:resumed` / heartbeat | loop → all | on resume / on wrap-up | NO — "weave is only the heartbeat" | `.claude/skills/session-relay-resume/SKILL.md:67-68`; `.claude/skills/session-relay-wrap-up/SKILL.md:8-9` |
| **witnessed `hf` packet / ledger** | loop → successor (committed) | every checkpoint/handoff | **YES — the authoritative resume signal** | `.claude/skills/session-relay-resume/SKILL.md:40,83` |
| **binding chain** (traceability) | kb slug → card `correlation_id` → weave job → PR → merge | per task | ledger/card side authoritative | `.claude/agents/meta-sync-steward.md:53`; `.claude/skills/meta-kb-sync/SKILL.md:57` |

**Missed-message prevention + non-blocking rule:** the loop *scans* the inbox at resume but **never blocks** the
foreground chat on it; the witnessed packet (rendered from ledger replay) is the single source of truth, so a
dropped weave message degrades to "missing context," never "lost state" (`session-relay-resume/SKILL.md:38-40,
78-83`). The lease bridge degrades gracefully — `Reserve::Unsupported` → `ProceedDegraded` (ledger-only) when
weave is absent, so an offline/air-gapped run is not walled (`handoff-lease/src/lib.rs:30-33,77-83`).
**Plane-keep guard:** `preflight.refuse_legacy_weave = true` blocks a stale weave transport from seeding a
session (`handoff-policy/src/policy.rs:124-131`) — an explicit DISTINCT-PLANES enforcement.

**A2A status ledger:** there is no dedicated background-agent status ledger crate; cross-agent status is the
**weave job + the witnessed ledger events** (`gatekeeper_judgment`, `cognitum_decision`, claim/checkpoint).
For the union's background lanes (§5 UP-4) a status-ledger surface is the named gap.

---

## 4. Replacement-of-human-bottleneck plan (automate / supervised / owner-only)

| Manual action | Disposition | Mechanism / gap | Evidence |
|---|---|---|---|
| Approve a PR / merge | **AUTOMATE NOW (done)** — surgical AI gatekeeper replaces human approval | `code-omniscient-gatekeeper` + `hf gatekeeper check` witnessed `Approve`/`Deny` | `handoff-gatekeeper/src/lib.rs:226-389`; `.claude/agents/code-omniscient-gatekeeper.md:2-4` |
| Claim coordination across agents | **AUTOMATE NOW (done)** — weave lease + in-ledger CAS | `handoff-lease/src/lib.rs:38-83`; `.handoff/policies/rules.toml:13-20` |
| Decide next task | **AUTOMATE NOW (done)** — pull-based priority-gated orchestrator + `next_safe` | `.claude/agents/systems-orchestrator.md:33`; `handoff-drift/src/lib.rs:720` |
| In-loop action permit (e.g. `hf ship`) | **AUTOMATE NOW (done, fail-closed)** — cognitum gate `permit/defer/deny`, defer⇒"human review required" | `hf/src/cognitum.rs:122-156,204-210` |
| Checkpoint / handoff / drift audit | **AUTOMATE NOW (done)** — `SessionEnd`/`PreHandoff` hooks + blocking gates | `.handoff/hooks/hooks.toml:38-52`; `.claude/settings.json` |
| Credential / egress decisions | **SUPERVISED** — envctl secrets broker seam (optional feature, experimental) | `handoff-gatekeeper/src/lib.rs:202-224`; codemap §2 (`handoff-secrets`) |
| Protected-file / guardrail change (`.github`, policies, ADRs, Cargo) | **OWNER-ONLY / supervised** — blocked unless explicit steward task clearance | `handoff-gatekeeper/src/lib.rs:240-274`; `.handoff/policies/rules.toml:45-56` |
| Architecture decision (ADR) | **OWNER-ONLY** — drift gate blocks objective/architecture change without ADR | `.handoff/policies/rules.toml:29-33` |
| Final escalation when a model cannot proceed | **OWNER-ONLY (shrinking)** — `NEEDS-HUMAN` scaffold to be replaced by a model with the human's skillset | `AGENTS.md:22-26` (intent claim) |
| Mechanical/structured sub-work (low-risk edits, doc regen) | **AUTOMATE — but UNOPTIMIZED** — currently uniform-opus; no sonnet/haiku **background** lane | `.claude/skills/handoff-loop/SKILL.md:37` → UP-4 |

---

## 5. Upgrade rows (`axis: rules-policy-org`)

> Union frame: STRICT-UPGRADE-ONLY ("Upgrade Only", "No Downgrades"). Every row is additive — it adds a guard,
> a lane, or an enforcement seam; **none weakens an existing gate**. The verifier must feasibility-gate each.

**UP-1 — Fold rusty-idd's CLI UNDER handoff's policy gates (replace the toothless guard).**
- **Evidence:** rusty-idd attaches by file only (codemap §5); its agent-guard is advisory. handoff's gates
  block at process boundary (`handoff-drift/src/lib.rs:791-793`; `hf/src/cognitum.rs:148-155`).
- **Risk-tier:** MEDIUM (wiring, not logic change; commands are independent modules — union §4 blast radius LOW).
- **Acceptance-criterion:** every rusty-idd CLI command that mutates the tree runs **through** `hf policy
  check-edit` (PreEdit `fail_mode="block"`) and merge-touching ops through the gatekeeper; a differential test
  shows an out-of-scope or protected-file write from a rusty-idd command is REFUSED (exit 1), identical to a
  native `hf` edit. **Reversibility:** HIGH (remove the hook wiring; commands still run standalone).

**UP-2 — Enforce the DECLARED-but-unenforced policy values (network + dependency-audit).**
- **Evidence:** `default_network_mode`/`default_dependency_mode` exist as data with **no kernel code path**
  (`.handoff/policies/rules.toml:10-11`; no consumer found in `hf/src`,`handoff-*`).
- **Risk-tier:** MEDIUM (new enforcement can false-block; must default-warn then promote to block).
- **Acceptance-criterion:** a `check-network`/`check-dependency` gate (or extension of `check-edit`) blocks a
  dependency add (`Cargo.toml`/`*.lock` change) lacking a witnessed `dependency_audit` ledger event; covered by
  a RED test asserting exit 1 on un-audited dep add. **Reversibility:** HIGH (config flag to warn-only).

**UP-3 — Add an `evolution-steward` to handoff's own org chart (self-upgrade after each cycle).**
- **Evidence:** handoff's 9-agent org has researcher/implementer/verifier/gatekeeper but **no evolution role**
  (`ls .claude/agents` — none); the plan-loop harness proves the pattern.
- **Risk-tier:** LOW (additive agent + skill; propose-by-default, fail-closed, never weakens a guard).
- **Acceptance-criterion:** after a loop cycle the steward emits a witnessed retro + routes low-risk in-scope
  edits via PR only (never mid-cycle, never touching a protected guard); a test shows it cannot auto-edit a
  protected file. **Reversibility:** HIGH (remove agent def).

**UP-4 — Introduce the dual-model **background** lane into handoff's loop (No Downgrades guard).**
- **Evidence:** loop pins uniform opus (`.claude/skills/handoff-loop/SKILL.md:37`); no per-role `model:` lane;
  no background/run-in-background delegation of mechanical work; no guard prevents a *silent* model downgrade.
- **Risk-tier:** MEDIUM (model routing affects quality; must gate hard work on opus).
- **Acceptance-criterion:** gates/ADR/verifier stay opus (asserted), mechanical sub-work (doc regen, lint
  fixes) routes to a cheaper lane and may run as a **background** agent; a policy guard records the chosen lane
  per action and **blocks a silent downgrade of a gate-tier action** (witnessed, like `cognitum_decision`).
  **Reversibility:** HIGH (revert to uniform-opus). Honors "No Downgrades": the lane choice is witnessed, not silent.

**UP-5 — Self-enforce the agent-guard in handoff's own `settings.json` (PreToolUse), not only via meta.**
- **Evidence:** `.claude/agent-guard.toml` has 9 command-deny patterns but `.claude/settings.json` wires no
  `PreToolUse` hook (only SessionStart/SessionEnd) — the guard is enforced only at the meta host layer.
- **Risk-tier:** LOW (additive hook; patterns already authored).
- **Acceptance-criterion:** a `PreToolUse` hook denies `git push --force` / `git reset --hard` / `rm -rf`
  on dangerous paths **inside a standalone handoff clone** (proven by a denied-command test), so the guard is
  not lost when the repo runs outside the meta host. **Reversibility:** HIGH (remove the hook block).

---

## 6. Honesty notes / N/A

- **N/A — handoff background-agent status-ledger crate:** none exists; A2A status today = weave jobs + witnessed
  ledger events (see §3). Recorded as the named gap feeding UP-4, not fabricated.
- **N/A — per-agent `model:` frontmatter:** grep over `.claude/agents/*.md` returns no `model:` field; the lane
  is set only in the loop skill prose (`SKILL.md:37`). Stated as found, not inferred.
- **Intent vs fact:** "Upgrade Only / capability-gain-without-baseline-corruption" and the NEEDS-HUMAN→model
  end-state are NORTH-STAR/AGENTS intent (`AGENTS.md:15-26`), recorded as claims for the verifier — no machine
  gate in handoff *enforces* parity-before-removal today (this is why STRICT-UPGRADE-ONLY is a frame, not yet a gate).
- All enforcement claims were read in source; the `exit(1)`/`fail_mode="block"` chain is the load-bearing
  difference vs rusty-idd's advisory guard and was verified across `handoff-drift`, `cognitum`, `handoff-gatekeeper`, and `hooks.toml`.
```
