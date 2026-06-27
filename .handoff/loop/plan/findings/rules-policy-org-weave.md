# Findings — rules / policy / agent org chart / A2A · target: **weave** (cycle 4)

- Axis: `rules-policy-org`. Target: **weave** — the fleet's **A2A transport plane** (live mailbox + terminal-inject), DISTINCT from handoff's witnessed-receipts plane.
- Code read-only @ `/home/drdave/Desktop/meta/.worktrees/plan-weave-red/weave` (branch `plan/weave-red-tests`).
- Policy frame: **Upgrade Only**, **No Downgrades** (STRICT-UPGRADE-ONLY); automate every researchable bottleneck; humans only at explicit supervised/risk boundaries; no silent model/provider downgrade.
- Every CLAIM cites `file:line`. Every UPGRADE carries risk-tier + acceptance-criterion. Read-only except this artifact.

---

## 1. Policy table (owner rules → where enforced)

| Policy | Status in weave | Evidence (file:line) | Verdict |
|---|---|---|---|
| **Upgrade Only / No Downgrades** | Core invariant: "one dependency-light Rust binary"; the workspace split (`WL-001`) is an *interim* state, single-crate is the deferred goal, never add crates/heavyweight deps without justification | `CLAUDE.md:22-24`, `CLAUDE.md:47-59` | CONFIRMED — codified hard rule |
| **No silent structural downgrade** | `develop`→`master` sync uses a **no-downgrade ancestor guard**; refuses a diverged master (ff-only) | `CLAUDE.md:44`; `.handoff/policy.toml:13` (`develop_mirrors_trunk = true`) | CONFIRMED |
| **No-downgrade A2A convergence** | Converge weave toward A2A v1.0 as a **strict-upgrade interop boundary** — keep the SQLite mailbox, ADD an A2A adapter, never replace | `research/weave.trends.md:9`, `:42-44` | CONFIRMED (planned, not built) |
| **Strict parity before removal** | Dual storage backend (`sqlite`↔`libsql`) kept behavior-symmetric; `Store` change must mirror both backends; `compile_error!` guards enabling both | `CLAUDE.md:74-82`, `CLAUDE.md:135`; codemap `model.rs` Store trait | CONFIRMED |
| **Gates only strengthen, never weaken** | Retro rule: "Gate-touching changes may only ever *strengthen* a gate, never weaken one"; new gate ⇒ negative test (prove teeth) | `.handoff/loop/proposed-upgrades.md:5-6`, P-1 | CONFIRMED — explicit anti-downgrade gate law |
| **Fail-closed write/network/dependency** | `default_write_mode = deny_without_claim`, `default_network_mode = deny_unless_task_allows`, `default_dependency_mode = deny_without_audit_event` | `.handoff/policies/rules.toml:9-11` | CONFIRMED |
| **Protected-file self-guard** | A confused/compromised agent must not rewrite its own guardrails — PRs touching `.github/**`, `.handoff/policy.toml`, `.handoff/policies/**`, `.handoff/hooks/**`, ADRs, `CLAUDE.md`, `Cargo.toml`, lockfiles are blocked | `.handoff/policies/rules.toml:37-50` | CONFIRMED |
| **Destructive-command blocklist** | `rm -rf /`, `git reset --hard`, `git clean -fdx`, `git push --force` (use `--force-with-lease`), `curl|sh` blocked | `.handoff/policies/rules.toml:52-62` | CONFIRMED |
| **Human only at risk boundary** | Governance/security-config (branch protection, required checks, credentials) must STOP for owner approval even mid-approved-flow | `.handoff/loop/proposed-upgrades.md` P-2 | CONFIRMED (classifier-enforced; codification proposed) |

**CLAIM-P1:** weave's anti-downgrade posture is enforced at THREE layers — source invariant (`CLAUDE.md:47-59`), merge ancestor-guard (`CLAUDE.md:44`), and gate-strengthen-only retro law (`proposed-upgrades.md:5-6`). This is a genuine **No Downgrades** regime, not a slogan.

---

## 2. Agent org chart

```
                         OWNER (revenaugh.david) — risk/governance boundary only
                                     │ (approves STRUCTURAL + governance/security-config changes)
                                     ▼
            weave-orchestrator (skill, leader, model=opus)  ── the commander
            .claude/skills/weave-orchestrator/SKILL.md
                                     │  Producer–Reviewer + incremental QA; coordinates via SendMessage + shared task list
        ┌──────────────┬────────────┴───────────┬─────────────────────┐
        ▼              ▼                          ▼                     ▼
  weave-planner   weave-implementer        weave-verifier        weave-guardian
  (Plan, opus)    (general-purpose, opus)  (general-purpose,opus)(read-only review, opus)
  map→layers      write Rust; mirror       add test layers;      invariant audit +
  invariants,     both backends            run full gate on      Rust-native DRIFT GUARD
  test layers     01→02 handoff            sqlite+libsql; RED→   + docs sync; APPROVE/BLOCK
  01_planner      02_implementer           route back; 03_verif  04_guardian
        │              │                          │                     │
        └──────────────┴──────────────┬───────────┴─────────────────────┘
                                       ▼
                         continuity-steward (general-purpose)  ── continuity lane
                         writes cold-start .handoff/packets/latest.md at HAND OFF
                                       ▼
                         evolution-steward (harness retro)  ── self-upgrade lane
                         proposed-upgrades.md (classify STRUCTURAL vs LOW-RISK)

  Background / autonomous lane (weave-loop, ralph-weave.sh):
    Phase A plan+implement+verify = AGENT_CMD (claude, opus)   [foreground worker]
    Phase B GUARDIAN              = GUARDIAN_CMD / MiniMax (minimax-m3:cloud)  [DUAL-MODEL: external guardian]
    Phase C delivery             = AGENT_CMD (PR + gh auto-merge)
```

| Role | Definition | Evidence (file:line) |
|---|---|---|
| Commander / orchestrator | `weave-orchestrator` skill, leader, "Always call agents with `model: opus`" | `.claude/skills/weave-orchestrator/SKILL.md:10-19` |
| Planner (architect-lite) | `weave-planner`, Plan type, opus, read-only | `.claude/agents/weave-planner.md:1-6` |
| Implementer (builder) | `weave-implementer`, general-purpose, mirrors both backends | `weave-orchestrator/SKILL.md:18`, `:45-46` |
| Verifier (QA) | `weave-verifier`, runs full gate on sqlite+libsql, RED→route-back loop | `weave-orchestrator/SKILL.md:19`, `:49-50` |
| Guardian (gate) | `weave-guardian`, invariant + Rust-native drift + docs; APPROVE/BLOCK; guardian-block wins | `.claude/agents/weave-guardian.md:1-16`; `weave-orchestrator/SKILL.md:52-54`, `:93` |
| Continuity steward | `continuity-steward`, cold-start packet, state+pointers only | `.claude/agents/continuity-steward.md:1-12` |
| Evolution steward | retro classification (STRUCTURAL→owner / LOW-RISK→auto-PR) | `.handoff/loop/proposed-upgrades.md:3-6` |
| Codex side-org | explorer / reviewer / docs-researcher (read-only evidence, review, API verification) | `.codex/AGENTS.md:17-21` |

**Escalation path:** verifier RED → implementer (loop until GREEN); guardian BLOCK → implementer → re-verify+re-guard (retry once, then escalate to leader; **guardian's block wins for invariants/drift**, `weave-orchestrator/SKILL.md:93`); STRUCTURAL/governance change → OWNER (`proposed-upgrades.md` P-2).

**CLAIM-P2:** The 4-agent team is a real, file-defined **agent org chart** (`.claude/agents/weave-{planner,implementer,verifier,guardian}.md` + orchestrator skill), with a deliberate decision to keep the guardian SEPARATE from the verifier for invariant+drift review (`CLAUDE.md:16`).

**CLAIM-P3 (dual-model background strategy):** In autonomous loop mode the guardian is delegated to **MiniMax** (`minimax-m3:cloud`) as an *external* guardian while workers run on `claude/opus` — a genuine dual-model split (independent reviewer model ≠ builder model). Evidence: `weave-guardian.md:16`, `weave-orchestrator/SKILL.md:56`, `ralph-weave.sh:18-19` (`MODEL=minimax-m3:cloud`, separate `GUARDIAN_CMD`), `ralph-weave.sh:199-200` (Phase B invokes `GUARDIAN_CMD_ARY`).

---

## 3. A2A / weave communication map (weave IS the A2A substrate)

weave is itself the A2A transport — so its A2A map is its own product surface, not an external dependency.

| A2A primitive | Surface (CLI verb / MCP tool) | Evidence |
|---|---|---|
| **send / notify / reply / answer / ack** | message ops; live nudge-inject on send | codemap `weave/src/main.rs:4489` (71 verbs); `store.rs` `Store::{send,inbox}` |
| **ask / ask-many / asks** (request-response, tracked correlation ids) | `weave_ask`, `weave_answer`, `weave_ack` | codemap §A2A surface; `store.ask(...)` used by the gate (`main.rs:8948`) |
| **spawn / attach / connect / inject / kill** (peer lifecycle + keystroke inject) | `weave-inject` `spawn`/`inject` | `weave-inject/src/inject.rs:556,714` |
| **register / peers / sessions / scan** (discovery) | presence seam | `weave-core/src/store.rs` peers table |
| **inbox** (per-reader read tracking; broadcast delivered once *per reader*) | `Store::inbox`, `inbox_since` | `weave-core/src/store.rs:83,105` |
| **heartbeat** (presence/liveness ledger) | `Store::heartbeat`, presence TTL | `weave-core/src/store.rs:647,650`, `:40-42` (`PRESENCE_TTL_SECS`) |
| **lease {reserve/release/list/sweep}** (mesh mutual-exclusion) | `reserve_lease`/`release_lease`, path-conflict detect | `weave-core/src/store.rs:750-765`; `model::Lease` `model.rs:1302-1309`; `lease_path_conflicts` `model.rs:1359` |
| **broadcast-notify / broadcast-ask** (fan-out) | `--to all` | `weave-loop/SKILL.md:51-52` |
| **serve / dashboard / push** (Tier-2 cross-machine HTTP) | `weave-mcp/src/http.rs` | codemap entry points |

**CLAIM-A1 (the lease = the mesh mutual-exclusion the parallel plan loop reuses):** weave provides a real distributed lease primitive — `model::Lease{resource,holder,acquired,expires}` (`model.rs:1302`), `reserve_lease` which "On conflict, returns `Err` naming the current holder and expiry" (`store.rs:750-752`), and path-conflict detection over exact/parent/child paths (`lease_path_conflicts`, `model.rs:1359`). The handoff continuity policy adopts the SAME shape — `[leases] heartbeat_seconds=30, stale_after_seconds=300, force_release_after_seconds=1800, require_disjoint_write_scopes=true` (`.handoff/policies/rules.toml:13-18`). This `require_disjoint_write_scopes` lease is exactly the **mutual-exclusion the parallel plan loop now reuses** to keep concurrent worktrees from colliding on write-scopes. CONFIRMED as a primitive; the plan-loop's *consumption* of it is policy-declared (`rules.toml:13-18`), not yet a verified code call from the plan loop.

**A2A standard gap (No Downgrades framing):** weave does NOT speak formal A2A v1.0 — it uses its own `Intent` wire schema (`model.rs:216`, codemap §Message schema) over SQLite-mailbox + HTTP-push. This is a **gap, not a regression** (`weave.trends.md:36-38`). Convergence is a strict-upgrade adapter job (emit/consume AgentCard + JSON-RPC envelope), reusing the existing default-off `sign` (ed25519) feature as the local analogue of A2A signed AgentCards (`weave.trends.md:48-54`). weave=transport/topology; A2A=swappable protocol envelope (`weave.trends.md:63-71`).

**Inbox scan cadence / missed-message prevention / foreground non-blocking rule:**
- Background-agent status ledger = the **heartbeat** presence rows (`store.rs:647`, TTL `:40-42`) + the committed `.handoff/packets/latest.md` checkpoint (`continuity-steward.md:33-63`).
- **Foreground-chat non-blocking rule (CONFIRMED):** the loop's authoritative resume signal is the FILE, not the inbox — "a self-addressed weave message does **not** land in your own inbox. Don't rely on your inbox for the resume signal — the file is." (`weave-loop/SKILL.md:133-134`). weave heartbeat/broadcast is an *observable* signal only (`weave-loop/SKILL.md:15`), so the live chat is never blocked waiting on a message.
- Missed-message prevention: per-reader `reads(message_id, reader)` tracking ⇒ broadcast delivered exactly once per reader; two delivery channels compose (instant inject + next-hook drain; graceful degradation for non-injectable peers) (`CLAUDE.md:108-110`).

---

## 4. The weave PreToolUse approval gate — REAL TEETH (the central security finding)

**CLAIM-G1 (real teeth, fail-closed):** The PreToolUse gate is enforcing, not decorative. `weave hook pretooluse` (`main.rs::handle_pretooluse_hook` / `pretooluse_decision`, `main.rs:8857,8871`) reads Claude's PreToolUse JSON and for a dangerous tool raises a blocking approval ask on the existing `store.ask`/`permission_verdict` machinery (`main.rs:8948-8966`), then blocks on weave's OWN short timeout (`cfg.pretooluse_timeout()`, default 30s clamped `[1,300]`, `config.rs:919-921`). Verdict mapping (`main.rs:8990-9011`):
- Approved ⇒ `allow`.
- Denied / Timeout / still-Pending ⇒ `deny` ("DENY-BY-DEFAULT … Claude's own timeout would have failed OPEN, so we MUST emit deny ourselves", `main.rs:8999-9009`).
- No approver configured ⇒ `deny` (`main.rs:8901-8913`); broadcast approver ⇒ `deny` (a tracked ask is point-to-point, `main.rs:8914-8921`); ask-open failure ⇒ `deny` (`main.rs:8958-8965`).
- Only a *safe* tool or an *unparseable* payload ⇒ `defer` (fail-OPEN is deliberately confined to "we cannot identify the tool", `main.rs:8877-8898`).

**CLAIM-G2 (it closes a once-empty gap):** WL-021 shipped the approval *primitive* but installed NO hook, so nothing actually blocked a tool; WL-055 added the real enforcing hook (`.handoff/loop/backlog.md:99`; `docs/REPOWIRE-PARITY.md:180`). This is the textbook "green gate proves nothing — only a negative test proves teeth" lesson (`proposed-upgrades.md` P-1) applied.

**CLAIM-G3 (opt-in, never surprise-blocks — anti-footgun, not anti-teeth):** Wiring is opt-in via `weave setup --pretooluse` (Claude-only, matcher `Bash|Edit|Write`, never-clobber-foreign + atomic + read-back-verify); default setup does NOT install it (`setup.rs:181-196`; `backlog.md:99`). When ENABLED it is fully fail-closed; OFF by default so it can't silently block a session that never asked for it. No new standing MCP tool (token-light invariant preserved, `CLAUDE.md:119`).

> Distinct from handoff: weave's PreToolUse gate governs **live tool execution** (transport-plane authorization); handoff's hooks (`hf policy check-edit/check-claim/check-handoff`, `.handoff/hooks/hooks.toml:23-39`, `fail_mode=block`) govern the **witnessed-receipts ledger**. Two planes, two gates — keep distinct.

---

## 5. Replacement-of-human-bottleneck plan

| Manual action today | Classification | Evidence / acceptance |
|---|---|---|
| Plan→implement→verify→guard a change | **Automate now** (done) | 4-agent team auto-runs; loop delivers via PR+auto-merge `weave-orchestrator/SKILL.md:66-74` |
| Guardian/invariant+drift review | **Automate now** (done, dual-model) | MiniMax external guardian `ralph-weave.sh:199-200`; canonical spec `weave-guardian.md` |
| Session handoff / cold-start resume | **Automate now** (done) | `continuity-steward` + `.handoff/packets/latest.md`; SessionStart `hf resume --compact` `.handoff/hooks/hooks.toml:11-16` |
| Out-of-scope write / claim policy | **Automate now** (done, fail-closed) | PreEdit/TaskClaim `hf policy check-*` `fail_mode=block` `hooks.toml:17-27` |
| Dangerous tool authorization (Bash/Edit/Write) | **Supervised** (gate routes to a human/peer approver) | PreToolUse ask → approver verdict `main.rs:8948-9011`; deny-by-default |
| PR merge | **Automate now** (done, on green) | gates-green auto-merge `CLAUDE.md:43`; `.handoff/policy.toml:22` (`auto_merge=on_approve`) |
| `develop`→`master` propagation | **Automate now** (in build-out) | `sync-master.yml` ff with ancestor guard `CLAUDE.md:44` |
| Governance / security-config (branch protection, required checks, credentials) | **Owner-only** | STOP for owner approval `proposed-upgrades.md` P-2 |
| A2A v1.0 interop adapter | **Supervised** (architecture decision, not yet built) | strict-upgrade adapter `weave.trends.md:42-44` |
| `permission_gate` (Tier-2 merge authorization) | **Supervised → transitional to AI gatekeeper** | `.handoff/policy.toml:23` (`permission_gate=true   # transitional → AI gatekeeper`) |

---

## 6. Upgrade rows (`axis: rules-policy-org`)

| ID | UPGRADE | Evidence | Acceptance criterion | Risk-tier | Reversibility |
|---|---|---|---|---|---|
| U-1 | **Codify the gate-strengthen-only + negative-test law into the verifier/guardian agent defs** (P-1) so every new gate must ship a negative test (introduce the violation → confirm exit≠0) | `proposed-upgrades.md` P-1; `main.rs` PreToolUse already exemplifies | A gate-bearing PR is BLOCKED unless `03_verifier_report.md` records a passing negative test; CI red on the deliberate violation | Low (strengthens only) | High (doc/agent-def edit) |
| U-2 | **Make the PreToolUse approver default to a real peer in loop mode** (today OFF-by-default ⇒ if enabled-but-unconfigured it denies everything) — wire `pretooluse_approver` to the orchestrator/owner peer in the loop preflight | `setup.rs:181-196`; `config.rs:907-921`; `main.rs:8901-8913` | In loop mode, a dangerous tool raises an ask that resolves to a live approver within `pretooluse_timeout`; no blanket deny-storm and no fail-open | Med (security-config; owner-gated per P-2) | High (config flag) |
| U-3 | **A2A v1.0 strict-upgrade adapter**: emit/consume AgentCard + JSON-RPC task envelope over the existing mailbox, reusing the `sign` (ed25519) feature for signed cards — ADD, never replace the `Intent` mailbox | `weave.trends.md:42-54,63-71`; `model.rs:216` `Intent` | weave can exchange a signed AgentCard + JSON-RPC task with an external A2A agent while the SQLite-mailbox route stays the required local path (No Downgrades) | Med (new surface, behind feature) | High (feature-flagged) |
| U-4 | **Promote `permission_gate` from transitional to an AI gatekeeper** (`.handoff/policy.toml:23`) — automate the Tier-2 merge-authorization human bottleneck behind a witnessed AI verdict | `.handoff/policy.toml:20-23` | Merge authorization is decided by a recorded AI verdict with deny-by-default; owner override path preserved | Med (governance boundary; owner-gated) | Med |
| U-5 | **Make the plan-loop's lease consumption explicit in code** — the policy declares `require_disjoint_write_scopes` (`rules.toml:18`) and weave provides the lease primitive (`store.rs:750`), but the plan-loop's reuse is policy-declared, not a verified call path; wire/verify the loop actually `reserve_lease`s its write-scope | `rules.toml:13-18`; `store.rs:750-765`; `model.rs:1359` | Two concurrent plan worktrees with overlapping write-scope: the second's `reserve_lease` returns Err naming the holder; no double-write | Low-Med | High |

---

## 7. Markers / laws / N/A

- Required literal markers present: **Upgrade Only** (§1, §6), **No Downgrades** (§1, §3, U-3), **agent org chart** (§2), **weave**/**A2A** (throughout), **background** (§2 background lane, §3, U-5 context).
- Laws honored: read-only on target code; fail-closed reasoning; every CLAIM cites `file:line`.
- N/A — formal A2A v1.0 transport conformance: **N/A — weave intentionally runs its own `Intent` mailbox schema (`model.rs:216`); A2A is a planned strict-upgrade adapter, not a current surface.**
- N/A — plan-loop→lease verified call path: tracked as U-5 (policy-declared `rules.toml:18`, code-side consumption unverified this cycle).
