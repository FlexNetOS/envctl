# Findings — rules / policy / agent org chart / A2A axis — rusty-idd

Target: **rusty-idd** (fabric AXIS: intent-driven control-plane organ of the FlexNetOS `meta` fleet).
Read-only audit. Every CLAIM cites `file:line`. Frame: owner's **Upgrade Only** / **No Downgrades**
law + the dual-model **background** strategy. Paths absolute.

Verdict (one line): rusty-idd has the owner's **Upgrade Only / No Downgrades** law *explicitly
codified* and a clean read-only-by-default **agent org chart** with a single-writer lock — but it
does **not** participate in **weave**/**A2A** background communication (continuity is filesystem-only),
has **no background-agent lane**, and its agent roster is missing commander/continuity/evolution
roles plus a working escalation artifact. Governance is strong on the merge-safety axis, thin on the
automate-the-bottleneck axis.

---

## 1. Policy table

| # | Policy (owner law) | Status in rusty-idd | Evidence (file:line) |
|---|---|---|---|
| P1 | **Upgrade Only** | CODIFIED verbatim | `/home/drdave/Desktop/meta/rusty-idd/AGENTS.md:42` — "Upgrade only. Never downgrade a working surface, dependency, action, model, agent, skill, or generated artifact to simplify a task." |
| P2 | **No Downgrades** (no silent model/provider/artifact downgrade) | CODIFIED (same rule, enumerates model + agent + skill + artifact) | `/home/drdave/Desktop/meta/rusty-idd/AGENTS.md:42`; reinforced by adopt-first `AGENTS.md:40-41` |
| P3 | Strict parity before removal | CODIFIED | `/home/drdave/Desktop/meta/rusty-idd/AGENTS.md:11` ("do not delete source code during migration; deprecate first, remove after parity tests pass"); `AGENTS.md:54` requires rollback path |
| P4 | Automate everything researchable | PARTIAL — workflow gates are automated (engine-computed `next`, pre/post-tool hooks) but no autonomous backlog runner in-repo | `/home/drdave/Desktop/meta/rusty-idd/.claude/settings.json:9` (SessionStart `rusty-idd next`); `/home/drdave/Desktop/meta/rusty-idd/.codex/hooks.json:15-40` (Pre/PostToolUse `codex workflow-check`) |
| P5 | Human only at explicit supervised / risk boundaries | CODIFIED — write pass requires explicit authorization; default harness read-only | `/home/drdave/Desktop/meta/rusty-idd/AGENTS.md:39`; `/home/drdave/Desktop/meta/rusty-idd/.codex/agents/rusty-idd-implementer.toml:3` (only `workspace-write` agent) |
| P6 | No silent model/provider downgrade | CODIFIED in P1; provider/credential config explicitly out-of-repo (not downgradable here) | `/home/drdave/Desktop/meta/rusty-idd/AGENTS.md:42`; `/home/drdave/Desktop/meta/rusty-idd/.codex/loops/rusty-idd-model-loop.toml:3-5` |
| P7 | Destructive-op guard / reviewable state | CODIFIED twice (rules + agent-guard) | `/home/drdave/Desktop/meta/rusty-idd/.claude/rules/meta-destructive-commands.md:7-13`; `/home/drdave/Desktop/meta/rusty-idd/.claude/agent-guard.toml:7-13` (`deny` list) |
| P8 | Single integration authority | CODIFIED | `/home/drdave/Desktop/meta/rusty-idd/AGENTS.md:18,60`; `/home/drdave/Desktop/meta/rusty-idd/.idd/LOCK.md:7` ("only one branch may hold integration authority at a time") |
| P9 | No host service/process mgmt / no user-global installs | CODIFIED | `/home/drdave/Desktop/meta/rusty-idd/AGENTS.md:44,46`; `/home/drdave/Desktop/meta/rusty-idd/.codex/rules/default.rules:6-68` (systemctl/kill/pkill `forbidden`; cargo/npm/pip install `prompt`) |
| P10 | Evidence-gated PRs | CODIFIED | `/home/drdave/Desktop/meta/rusty-idd/AGENTS.md:48-56`; `/home/drdave/Desktop/meta/rusty-idd/.claude/agent-guard.toml:15-23` (`required = build,test,lint,audit,validate,manifest`) |

CLAIM-1: The owner's `Upgrade Only` / `No Downgrades` law is a *first-class, literal* repo rule, not
inferred — `AGENTS.md:42`. CONFIRMED.

CLAIM-2: rusty-idd's policy plane lives in three layers, no `.handoff/policy` dir:
`/home/drdave/Desktop/meta/rusty-idd/.claude/agent-guard.toml` (Claude lane),
`/home/drdave/Desktop/meta/rusty-idd/.codex/rules/default.rules` (Codex lane), and `AGENTS.md`
(durable cross-tool rules). N/A — `.handoff/policy`: there is no `.handoff/policy` directory and no
`META-ORG-POLICY.md` in-repo; the handoff README *cites* fleet policy "META-ORG-POLICY.md P7" as an
external parent-fleet doc, not a local file (`/home/drdave/Desktop/meta/rusty-idd/.handoff/README.md:4`).

CLAIM-3 (agent-guard mode is advisory): `agent-guard.toml` `mode = "warn"`
(`/home/drdave/Desktop/meta/rusty-idd/.claude/agent-guard.toml:2`) — the destructive-command deny
list *warns*, it does not hard-block. The hard refusal lives in `.codex/rules/default.rules`
(`decision = "forbidden"`) but only for Codex, and only for host-process verbs — not for the
`git reset --hard` / `rm -rf` set, which is `warn`-only on the Claude side. CONFIRMED.

---

## 2. Agent org chart

Roster (control plane, not product symbols). Codex subagents at
`/home/drdave/Desktop/meta/rusty-idd/.codex/agents/`, loop at `.codex/loops/`, skills at `.agents/skills/`:

```
                 OWNER (human; explicit authorization boundary)
                          │  authorizes write pass (AGENTS.md:39)
                          ▼
        rusty-idd engine = de-facto "orchestrator"
        (computes next step; gates every tool call)
        .claude/settings.json:9  +  .codex/hooks.json:15-40
                          │
        ┌─────────────────┼───────────────────────────┐
        ▼ (read-only)     ▼ (read-only)                ▼ (WRITE — single writer)
   explorer           gap-hunter / verifier        implementer
   gpt-5.5-mini       gpt-5.5-mini / gpt-5.5       gpt-5.5
   (Scout/Atlas)      (Gap / Gauge)                (Builder/Mason/Forge)
        └──── rusty-idd-model-loop (3 read-only passes) ────┘
                 .codex/loops/rusty-idd-model-loop.toml
   Integration authority: .idd/LOCK.md (one branch at a time)
```

| Role | Agent file | Sandbox | Model / reasoning | Evidence |
|---|---|---|---|---|
| Explorer (orientation) | `.codex/agents/rusty-idd-explorer.toml` | `read-only` | medium | `:3,:4` |
| Gap-hunter (omission audit) | `.codex/agents/rusty-idd-gap-hunter.toml` | `read-only` | high | `:3,:4` |
| Verifier (gate/evidence) | `.codex/agents/rusty-idd-verifier.toml` | `read-only` | high | `:3,:4` |
| Implementer (the only writer) | `.codex/agents/rusty-idd-implementer.toml` | `workspace-write` | high | `:3,:6` ("single writer for a vertical slice") |
| Loop driver (explore→gap-hunt→verify) | `.codex/loops/rusty-idd-model-loop.toml` | all 3 passes `read-only` | mini→mini→full | `:14-50` |
| Skills (reusable workflows) | `.agents/skills/{rusty-idd-adopt-first,…-codex-rust-env,…-knowledge,…-verify}/SKILL.md` | n/a | n/a | dir listing |

CLAIM-4: rusty-idd implements a **read-only-by-default, single-writer** agent org chart — 3 of 4
subagents are `read-only`; only `rusty-idd-implementer` is `workspace-write` and is self-described as
"the single writer for a vertical slice" (`/home/drdave/Desktop/meta/rusty-idd/.codex/agents/rusty-idd-implementer.toml:6`).
Concurrency capped at `max_threads = 4`, `max_depth = 1`
(`/home/drdave/Desktop/meta/rusty-idd/.codex/config.toml:11-12`). AGENTS.md:45 confines parallel
subagents to "read-heavy exploration, verification, and gap hunting unless a single integration
branch/worktree owner coordinates writes." CONFIRMED — this is exactly the owner's "one writer,
many readers" safety model.

CLAIM-5 (dual-model tiering present, but it is NOT the owner's Claude background strategy): the loop
tiers models — `gpt-5.5-mini` for explore + gap-hunt, `gpt-5.5` (high reasoning) for verify
(`/home/drdave/Desktop/meta/rusty-idd/.codex/loops/rusty-idd-model-loop.toml:14,28,42`). This is a
cheap-scout / expensive-gate split — structurally the same *idea* as the owner's
opus-on-gates / sonnet-or-haiku-on-mechanical lanes — but it is a **Codex/GPT** roster, all passes
`sandbox = "read-only"`, run in the **foreground** as a design-first loop ("Default mode is read-only
and design-first", `:3`). There is no Claude opus/sonnet/haiku lane and no **background** execution
lane. CONFIRMED (org exists; background dual-model lane absent — see Gap-G1, UP-2).

CLAIM-6 (missing roles vs the owner's full org): the roster has explorer / gap-hunter / verifier /
implementer but **no** commander/orchestrator agent, no continuity-steward, no evolution-steward, and
no escalation agent — the "orchestrator" is the engine's computed `next` step, not an agent. Escalation
is by prose only ("ask the owner", `.codex/rules/default.rules:9`). CONFIRMED.

CLAIM-7 (escalation artifact is a dangling reference): `AGENTS.md:15` instructs "If two agents
conflict, stop and update `/AI_MERGE/05_conflict_risk_register.md` before continuing" — but that file
**does not exist** (`AI_MERGE/` has no `05_*` file; `ls` confirms absence). The conflict-resolution
escalation path points at a missing artifact. CONFIRMED (Gap-G2, UP-4).

---

## 3. A2A / weave communication map

| Channel | Does rusty-idd participate? | Evidence |
|---|---|---|
| **weave** / **A2A** runtime comms (send/receive, inbox scan, heartbeat) | **NO** — zero weave/A2A *dependency or wiring* in product or control plane | no `weave` in any `crates/*/Cargo.toml` (grep rc:1); no `a2a`/`agent-to-agent` anywhere |
| weave as a **mapped subject** (knowledge graph classifies the *fleet's* weave repo) | YES, but read-only classification only | `/home/drdave/Desktop/meta/rusty-idd/crates/knowledge/src/lib.rs:3593-3597` (`capability:agent-communication`, `repo_names: &["weave","atc","mcp_hub"]`, anchor "weave agent communication layer"); also `:3046,:3585,:3602` |
| Continuity / handoff (the real channel rusty-idd uses) | YES — **filesystem + JSON schema**, not live IPC | `/home/drdave/Desktop/meta/rusty-idd/.handoff/tasks/*.task.json` (3 cards); `/home/drdave/Desktop/meta/rusty-idd/.handoff/context/capsule.json` (`schema: handoff.context_capsule.v1`); README state precedence `Git > witnessed ledger > task cards` (`.handoff/README.md:6-7`) |
| Background-agent status ledger / inbox scan cadence | **NONE in-repo** — ledger is the *fleet* `meta/handoff/.handoff/ledger.db`, external; rusty-idd holds only git-committed text cards | `/home/drdave/Desktop/meta/rusty-idd/.handoff/README.md:7` ("The fleet ledger lives at `meta/handoff/.handoff/ledger.db` — no binary state in this directory") |
| Foreground-chat non-blocking rule | N/A — no async/background lane exists, so no blocking concern is engineered for or against | grep: no `run_in_background`, no "background agent" in `.md`/`.toml` |

CLAIM-8 (cartographer's "weave=0 product refs" is substantively right but needs a footnote): there
are 11 `weave` string hits in `crates/knowledge/src/lib.rs`, but **every one** is either a
repo-name/anchor literal in the knowledge-graph's *fleet capability registry*
(`:3585,:3593-3597,:3602-3605`), a classifier branch (`:3046` `if name == "weave"`), or a unit-test
fixture (`:6183-6252`). None is a comms call, IPC, or dependency. So rusty-idd **describes** weave
(it is the fleet's cartographer organ) without **using** weave. Net A2A participation = 0. CONFIRMED.

CLAIM-9: rusty-idd's only fleet-coupling is the `.handoff/` *file + schema* contract
(`handoff.task.v1` envelope in `crates/work-order`, per codemap `:64`) — read at
`crates/cli/src/commands/codex.rs:593` per the codemap. There is no inbox-scan cadence, no
background heartbeat emitter, no missed-message prevention because there is no live channel.
The harness's own rule (weave = "observable heartbeat", committed `HANDOFF.md` = authoritative) means
a non-participating member is *tolerable* for continuity — but it is invisible to fleet-level
background coordination. CONFIRMED (Gap-G1).

---

## 4. Replacement-of-human-bottleneck plan

North-star alignment is explicit: the handoff capsule states the owner law verbatim —
"**NO HUMAN IN THE LOOP**: witnessed work-orders, leases, fail-closed native auto-merge; Git >
ledger > cards" (`/home/drdave/Desktop/meta/rusty-idd/.handoff/context/capsule.json` `northstar`).
Yet the *current* in-repo automation stops at gate-computation, not autonomous execution.

| Manual action today | Category | Evidence / rationale |
|---|---|---|
| Compute the next workflow step | **Automated now** | SessionStart hook runs `rusty-idd next` (`.claude/settings.json:9`); engine, not human, decides |
| Enforce workflow order around each edit | **Automated now** | Pre/PostToolUse `codex workflow-check` (`.codex/hooks.json:15-40`) |
| Refresh deterministic artifacts (`.idd/knowledge`, MANIFEST, status) | **Automated now** (validation step) | `AGENTS.md:28`; `agent-guard.toml:18-22` evidence gate |
| Explore / gap-hunt / verify a change | **Automated, supervised** | read-only Codex loop (`.codex/loops/rusty-idd-model-loop.toml`); design-first, no writes |
| Implement a vertical slice | **Supervised** — agent capable, but write pass needs explicit owner authorization | `AGENTS.md:39`; implementer agent exists (`rusty-idd-implementer.toml`) |
| Run the loop continuously over a backlog (no human re-trigger) | **NOT automated — bottleneck** | no in-repo continuous/Ralph runner; `.codex/loops/*` is a 3-pass read-only design loop, not a self-restarting executor. AI_MERGE queue rows have unassigned owners (`Agent` = placeholder), `Status = queued`/`blocked` (`AI_MERGE/08_agent_queue.md:7-12`) — the queue waits on a human dispatcher |
| Resolve agent conflicts / escalate | **Owner-only, and broken** | `AGENTS.md:15` routes to a missing `05_conflict_risk_register.md` |
| Background fleet coordination via weave/A2A | **Owner-only / absent** | no channel (Section 3) |
| Manage host services, install tooling | **Owner-only by design** (correct) | `.codex/rules/default.rules`; AGENTS.md:44,46 — intentionally NOT automated here |
| Integration-branch authority assignment | **Owner-only** — `.idd/LOCK.md` fields all `unassigned` (`/home/drdave/Desktop/meta/rusty-idd/.idd/LOCK.md:5-9`) |

CLAIM-10: The biggest unreplaced bottleneck is **continuous autonomous execution** — rusty-idd
*plans* and *gates* automatically but still needs a human to dispatch the implementer and re-trigger
the loop; the AI_MERGE agent queue is human-serialized with unassigned (placeholder) owners
(`/home/drdave/Desktop/meta/rusty-idd/AI_MERGE/08_agent_queue.md:7-12`). CONFIRMED.

---

## 5. Upgrade rows (`axis: rules-policy-org`)

Each row: evidence → acceptance-criterion → risk-tier → reversibility. **Upgrade Only / No
Downgrades**: every row is additive; none weakens an existing guard.

### UP-1 — Promote the Claude destructive-command guard from `warn` to fail-closed `deny`
- axis: rules-policy-org
- evidence: `/home/drdave/Desktop/meta/rusty-idd/.claude/agent-guard.toml:2` (`mode = "warn"`) vs the
  Codex side's hard `decision = "forbidden"` (`.codex/rules/default.rules:8`). Asymmetric: Codex
  blocks host-process verbs, Claude only warns on `git reset --hard`/`rm -rf`.
- acceptance-criterion: `agent-guard.toml` enforces block (or a PreToolUse Bash matcher denies the
  five deny-list patterns) AND a deliberate `git reset --hard` in a scratch worktree is refused, not
  merely warned; existing allowed commands still pass.
- risk-tier: **LOW** (tightens an already-declared deny list; no new capability).
- reversibility: trivial — revert `mode` to `warn`.

### UP-2 — Add a dual-model **background** execution lane to the agent org chart
- axis: rules-policy-org
- evidence: org chart has read-only foreground passes only
  (`/home/drdave/Desktop/meta/rusty-idd/.codex/loops/rusty-idd-model-loop.toml:3,16,30,46` all
  `sandbox = "read-only"`, "design-first"); no `run_in_background` anywhere; capsule north-star
  demands "NO HUMAN IN THE LOOP" (`.handoff/context/capsule.json`). Owner dual-model background
  strategy = cheap model on mechanical/scout lanes, strong model on gates — partially present as
  `gpt-5.5-mini`→`gpt-5.5` tiering but not as a background lane.
- acceptance-criterion: a documented background lane (loop or agent def) runs the
  explore/gap-hunt passes unattended on the cheap tier and reserves the strong tier for the verify
  gate, emitting a heartbeat/status the owner can poll; the foreground chat is non-blocking while it
  runs; write actions still require the explicit-authorization boundary (P5 preserved).
- risk-tier: **MEDIUM** (introduces unattended execution; must keep the single-writer + explicit-auth
  invariants from `AGENTS.md:39,45`).
- reversibility: high — background lane is opt-in config; delete to revert to foreground-only.

### UP-3 — Make rusty-idd an observable **weave**/**A2A** fleet member (heartbeat-only first)
- axis: rules-policy-org
- evidence: zero weave/A2A wiring (Section 3, CLAIM-8); rusty-idd already *classifies* weave as the
  fleet comms layer (`/home/drdave/Desktop/meta/rusty-idd/crates/knowledge/src/lib.rs:3593-3597`) yet
  emits no heartbeat; continuity is filesystem cards only (`.handoff/tasks/*.task.json`).
- acceptance-criterion: rusty-idd emits a weave heartbeat/status at checkpoint (per the fleet rule
  "weave = observable heartbeat, committed HANDOFF.md = authoritative") AND scans an inbox at a stated
  cadence; absence of weave still degrades safely to the committed-card path (no hard dependency —
  preserves the no-new-coupling caution from the codemap).
- risk-tier: **MEDIUM** (new external coupling; must stay heartbeat/observability-only, not a
  continuity dependency, to avoid downgrading the Git>ledger>cards precedence).
- reversibility: high — heartbeat is fire-and-forget; remove emitter to revert.

### UP-4 — Repair the conflict-escalation artifact and complete the org escalation path
- axis: rules-policy-org
- evidence: `/home/drdave/Desktop/meta/rusty-idd/AGENTS.md:15` points at non-existent
  `AI_MERGE/05_conflict_risk_register.md` (confirmed absent); roster has no escalation/continuity/
  evolution agent (CLAIM-6); `.idd/LOCK.md` fields all `unassigned` (`:5-9`).
- acceptance-criterion: the referenced conflict register exists (or AGENTS.md:15 is corrected to a
  live target) AND a named escalation path (which agent/role + where conflicts are logged) is
  documented; a simulated two-agent conflict has a defined, file-backed landing spot.
- risk-tier: **LOW** (doc + artifact creation; no behavior change).
- reversibility: trivial — additive doc/artifact.

### UP-5 — Bound `.idd` backup-file accumulation (audit-state hygiene)
- axis: rules-policy-org
- evidence: 20 `.idd/*.idd-bak-*` files including `MANIFEST.tsv.idd-bak-1`…`-19` and
  `LOCK.md.idd-bak-1` (`ls` count = 20); MANIFEST is the declared audit baseline
  (`/home/drdave/Desktop/meta/rusty-idd/AGENTS.md:16`). Unbounded backups dilute the reviewable-state
  guarantee that `.claude/rules/meta-destructive-commands.md:3` rests on.
- acceptance-criterion: a documented retention policy (or rotation cap) for `*.idd-bak-*` such that
  the audit baseline stays singular and reviewable; deletion of superseded backups still honors the
  destructive-command guard (explicit request + worktree inspection, `meta-destructive-commands.md:5`).
- risk-tier: **LOW** (hygiene; the deletion itself is the only guarded action).
- reversibility: high — policy is documentation; backups are regenerated by the engine.

---

## Markers (gate)
Upgrade Only ✓ · No Downgrades ✓ · agent org chart ✓ (Section 2) · weave/A2A ✓ (Section 3) ·
background ✓ (UP-2, CLAIM-5).

## Confidence
HIGH on policy + org-chart + A2A-absence (all from primary files with line cites; weave-dep absence
cross-checked by grep rc:1 on Cargo manifests). MEDIUM on the bottleneck/automation gap (inferred
from absence of a continuous runner + unassigned queue owners — absence-of-evidence, not contradiction).
