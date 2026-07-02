# Findings — rules-policy-org — TARGET `grit`

Axis: `rules-policy-org`. Read-only audit of owner standing rules, policy posture, the
**agent org chart** fit, and `weave`/`A2A` communication relationship for the `grit`
symbol-lock substrate. Every row cites `file:line` (paths absolute under
`/home/drdave/Desktop/meta/grit/`) or named source ledger rows.

`grit` is the AST symbol-lock + serialized-merge substrate for parallel AI agents; `weave`
is the agent-to-agent (`A2A`) session transport. They are complementary planes in the meta
`orchestration` tag set — `grit` arbitrates *write contention on code*, `weave` arbitrates
*messages between sessions*. Neither downgrades the other.

---

## 1. Policy table (owner standing rules)

| Policy | Stance for `grit` | Evidence |
|---|---|---|
| **Upgrade Only** | grit is the lock primitive that *makes* parallel multi-agent upgrades possible without losing work; SemVer is pre-1.0 minor-bump (`bump-minor-pre-major`), so feature growth is additive | `release-please-config.json:7-8`; `Cargo.toml:3` (`version = "0.4.0"`) |
| **No Downgrades** | `0.4.0` adds *real* shared read locks on S3/Azure (feature, not removal); CHANGELOG is additive Features/Bug-Fixes only — no capability rollback | `CHANGELOG.md:3-9` |
| Strict parity before removal | `grit done` *skips* the merge (refuses) when the main worktree is dirty rather than corrupting state — fail-closed, no silent data loss | `README.md:272-275` |
| Automate everything researchable | Whole tool exists to automate merge-conflict avoidance for 1–50 agents (0% work wasted vs 51–90% raw git); queue + auto-assign remove manual coordination | `README.md:25-37,198-213,296-308` |
| Human only at supervised/risk boundary | Destructive-command guard forbids `git reset --hard`, `git clean -fd`, force-push w/o lease unless owner-requested + worktree inspected | `/home/drdave/Desktop/meta/rusty-idd/.claude/rules/meta-destructive-commands.md` (workspace policy applies to grit operations) |
| No silent model/provider downgrade | grit holds no model/provider config; backend choice is explicit (`set-local`/`set-azure`/`set-s3`), never auto-switched | `README.md:96-144`; `src/config.rs` |
| Commit/push/PR discipline | CI gates every PR on build + test + `cargo fmt --check` + `clippy -D warnings`; `develop→master` is the only release path, other→master PRs flagged | `.github/workflows/ci.yml` (Build/Run tests/Check formatting/Clippy steps); `.github/workflows/pr-target-check.yml:13-15` |

**CLAIM rows**

- CLAIM rpo-1 — grit upholds **Upgrade Only**: pre-1.0 release config bumps minor for
  every feature and patches minors, so versions only move forward additively.
  `release-please-config.json:5-9`.
- CLAIM rpo-2 — grit upholds **No Downgrades**: `0.4.0` CHANGELOG is purely additive
  (Features: real shared read locks on S3/Azure; Bug Fixes) with no removed capability.
  `CHANGELOG.md:3-9`.
- CLAIM rpo-3 — Fail-closed merge: a dirty main worktree makes `grit done` *refuse* to
  merge and *keep* the agent branch intact rather than risk loss. `README.md:272-275`.
- CLAIM rpo-4 — Commit/push/PR discipline is machine-enforced: required CI checks are
  build, test, `cargo fmt -- --check`, `cargo clippy -- -D warnings`.
  `/home/drdave/Desktop/meta/grit/.github/workflows/ci.yml` (Clippy step `-D warnings`).
- CLAIM rpo-5 — Release-branch discipline: PRs targeting `master` not from `develop` are
  flagged; `develop→master` is the maintainer release path.
  `.github/workflows/pr-target-check.yml:13-15`.

---

## 2. Agent org chart — where grit's lock/merge substrate sits

grit is the **arbitration / contention-control layer** of the **agent org chart**: it does
not command agents, it *gates their writes*. Mapping onto the meta planning-engineer org:

```
              OWNER (human, supervised/risk boundary only)
                         │
                 commander / orchestrator
                  (spawns N parallel agents)
                         │
        ┌────────────────┼────────────────┐
   specialist        specialist        specialist     ← background lanes, each:
   (agent-1)          (agent-2)          (agent-N)       its own .grit/worktrees/agent-N
        │                │                │
        ▼                ▼                ▼
   ┌───────────────────────────────────────────────┐
   │  grit lock substrate  (CONTENTION CONTROL)     │
   │  claim → work(isolated worktree) → done        │
   │  AST symbol locks · queue · merge.lock(serial) │  ← arbiter, NOT commander
   └───────────────────────────────────────────────┘
        │   real-time events (room.sock pub/sub)
        ▼
   watchers / verifier / continuity  (grit watch)
```

- Specialists = the parallel agents (`agent-1..agent-N`), each isolated in
  `.grit/worktrees/agent-N` for true parallelism. `README.md:163,255-259`;
  `src/cli/mod.rs:459-469` (`cmd_claim` per-agent).
- Arbiter role = grit itself: AST function-level locks block conflicting writers while
  letting non-overlapping symbols in the *same file* proceed. `README.md:41-51,71-73`.
- Serialization gate = `merge.lock` file lock serializes git merges to prevent
  `index.lock` races — the org's single-writer-at-merge rule. `README.md:73,258-259`.
- Dependency-aware claims auto-lock callees as **read** (`--with-deps`), expanding the
  blast-radius lock set from the dep graph. `src/cli/mod.rs:482-499`; `README.md:191-195`.
- Queue = escalation/back-pressure path: blocked agents join a queue and are auto-granted
  on release instead of failing (no busy human reassignment). `README.md:198-206`;
  `src/cli/mod.rs:577-591`.

**CLAIM rpo-6** — grit is an *arbiter*, not a commander, in the **agent org chart**: it
holds locks/queue/merge serialization but issues no work; agents self-claim.
`src/cli/mod.rs:459-475,569-575`.

---

## 3. A2A / weave communication map

Two distinct event/transport planes; grit owns the *code-contention* plane, `weave` owns
the *session-message* (`A2A`) plane.

| Concern | grit (this target) | weave (`A2A` transport) |
|---|---|---|
| Unit of coordination | AST symbol lock claim/release/done | session-to-session message / nudge |
| Channel | `room.sock` Unix-socket pub/sub (local) | tmux/zellij pane injection + hook delivery |
| Background mechanism | **background** thread broadcasts events | **background** heartbeat daemon |
| Distributed mode | Azure Event Grid / S3 notifications (poll) | libsql pull sources w/ per-source timeout |
| meta tag | `orchestration` | `orchestration` |

- meta registry places both under `orchestration`: grit `tags: [ai, rust, orchestration]`,
  weave `tags: [mcp, orchestration]`. `/home/drdave/Desktop/meta/.meta.yaml:184-200`.
- weave is the **A2A** mesh — "Rust-native agent-to-agent session mesh", pushes into a
  running session's pane, runs a **background** heartbeat daemon.
  `/home/drdave/Desktop/meta/weave/README.md:1-18,151`.
- grit's own comms is the **room** (see §4): `room.sock` newline-JSON pub/sub of
  `Claimed`/`Released`/`AgentDone`. `src/room/mod.rs:8-20,48-54`.

**Relationship & background/async coordination:** grit emits lock-lifecycle events; a
peer or watcher can react. Locally this is the room socket; on Azure/S3 the same events are
free/native (`BlobCreated`/`BlobDeleted`, S3 notifications) so agents coordinate **without
polling**. `README.md:106-107,227-234`. weave can sit *above* this as the A2A transport
carrying "agent-N released login()" type nudges between sessions, while grit remains the
authority on whether a claim is granted. The two are non-blocking on each other — grit's
room broadcast is fire-and-forget (`notify` returns silently if no socket / on timeout),
mirroring the foreground-chat non-blocking rule. `src/room/mod.rs:35-45`.

**CLAIM rpo-7** — grit and `weave`/`A2A` are separate planes joined by tag
`orchestration`; grit = code-lock authority, weave = session message transport.
`/home/drdave/Desktop/meta/.meta.yaml:184-200`; `weave/README.md:1-18`.

**CLAIM rpo-8** — grit's event emission is non-blocking/**background** and fail-open on the
*transport* (silent return if `room.sock` absent or write times out), so it never stalls a
foreground agent. `src/room/mod.rs:35-45`.

---

## 4. The "room" coordination primitive — assessment

`grit`'s **room** IS a coordination primitive: a local Unix-socket publish/subscribe bus
for lock-lifecycle events, served by a **background** thread.

- `RoomEvent { event_type, agent, symbols }` with `EventType::{Claimed, Released,
  AgentDone}` — the coordination vocabulary. `src/room/mod.rs:8-20`.
- `NotificationServer::start` binds `room.sock` and spawns a **background** thread
  (`thread::spawn`) that accepts connections; returns immediately, runs until process exit.
  `src/room/mod.rs:66-96`.
- Producer/watcher disambiguation by a 200ms read timeout: a connection that sends within
  200ms is a **producer** (`grit claim/release/done`) and its line is broadcast; one that
  stays silent is a **watcher** (`grit watch`) parked in the list. `src/room/mod.rs:48-54,99-133`.
- DoS guard: max 128 watchers; dead watchers pruned on broadcast failure.
  `src/room/mod.rs:124-130,135-154`.
- Wired into the workflow: `grit init` starts the server (`src/cli/mod.rs:447-449`);
  `grit claim` fires `Claimed` after a grant (`src/cli/mod.rs:569-575`); `grit watch`
  streams it (`src/cli/mod.rs:108-109,322`).
- Distributed analogue: on Azure/S3 the room is replaced by Event Grid / S3 notifications
  so the same claim/release coordination works cross-machine. `README.md:106-107,227-234`.

**CLAIM rpo-9** — the room is a real coordination primitive (socket pub/sub of
claim/release/done) run on a **background** thread, with a distributed Event-Grid/S3
equivalent — it is grit's local heartbeat/observability bus, complementary to weave's
**A2A** session transport. `src/room/mod.rs:22-154`; `README.md:227-234`.

---

## 5. Replacement-of-human-bottleneck plan

| Manual action | Category | Evidence |
|---|---|---|
| Resolving merge conflicts between parallel agents | **automate now** (done) | `README.md:25-51` — AST locks → 0% work wasted |
| Picking a free symbol to work on | **automate now** (done) | `grit assign` auto-picks a free symbol `README.md:208-213` |
| Re-trying a blocked claim | **automate now** (done) | `--queue` auto-grants on release `src/cli/mod.rs:577-591` |
| Watching who holds what / when released | **automate now** (done) | room pub/sub + `grit watch` `src/room/mod.rs:48-54`; `src/cli/mod.rs:108-109` |
| Expiring stale locks | **automate now** (done) | `grit gc` + `heartbeat --ttl` `README.md:229-231` |
| Cross-session A2A nudge ("symbol freed") | **automate (supervised)** — wire grit room events → weave transport | gap: no direct grit→weave bridge in `src/`; `weave/README.md:151` daemon exists |
| Backend credential rotation (Azure/S3 keys) | **owner-only** (risk boundary) | `README.md:109-114,132-137` explicit key flags |
| Force-push / hard-reset during merge recovery | **owner-only** | `/home/drdave/Desktop/meta/rusty-idd/.claude/rules/meta-destructive-commands.md` |

---

## 6. Upgrade rows (`axis: rules-policy-org`)

| id | Upgrade | Evidence | Acceptance | Risk | Reversibility |
|---|---|---|---|---|---|
| UPGRADE rpo-A | Add an explicit grit→`weave` (`A2A`) bridge: forward room `Released`/`AgentDone` events as weave nudges so a queued agent's *session* is pinged, not just its socket watcher | `src/room/mod.rs:35-54` (room events exist); `weave/README.md:151` (background daemon exists); no bridge in `src/` | A released symbol triggers a weave nudge to the next queued agent's pane within one heartbeat; foreground non-blocking preserved | Low — additive, fail-open like existing `notify` | High — feature-flag; drop bridge, room/weave unaffected |
| UPGRADE rpo-B | Document the **agent org chart** + grit-as-arbiter contract in `AGENTS.md` (currently ICM-only, 27 lines) so the lock/merge authority and No-Downgrade/fail-closed rules are stated where agents read them | `/home/drdave/Desktop/meta/grit/AGENTS.md:1-27` (no org/policy content) | AGENTS.md states claim→work→done contract, arbiter (not commander) role, and merge-refuse-on-dirty rule | Low — docs only | High — revert doc |
| UPGRADE rpo-C | Persist a **background**-agent status ledger (last-seen heartbeat per agent) alongside locks so missed-message / dead-agent detection is explicit, feeding `grit gc` and weave presence | `README.md:229-231` (`heartbeat`/`gc` exist but no surfaced ledger); `src/room/mod.rs` (events ephemeral) | `grit status` shows per-agent last-heartbeat + stale flag; gc uses it | Med — touches db schema (additive column) | Med — additive table/column, droppable |
| UPGRADE rpo-D | Make CI clippy/fmt the documented required-checks contract in repo policy (mirror meta preflight) so PR discipline is owner-visible, not just CI-implicit | `.github/workflows/ci.yml` (gates exist); no policy doc citing them | A policy doc lists build/test/fmt/clippy as required + `develop→master` rule | Low — docs only | High — revert doc |

---

## Summary (3 lines)
grit is the symbol-lock/serialized-merge **arbiter** of the agent org chart (claim→work→done, AST locks, queue, merge.lock) and enforces owner rules — `Upgrade Only`/`No Downgrades` (additive pre-1.0 releases, CHANGELOG.md:3-9) plus fail-closed merge refusal and CI-gated commit/push/PR discipline (ci.yml, pr-target-check.yml).
Its "room" is a genuine coordination primitive — a `background`-thread `room.sock` pub/sub of Claimed/Released/AgentDone (src/room/mod.rs:22-154) with an Azure/S3 distributed equivalent — complementary to `weave`, the `A2A` session transport (both tagged `orchestration` in .meta.yaml:184-200); grit is the code-contention authority, weave the cross-session message plane.
Top upgrades: bridge grit room events → weave A2A nudges (rpo-A), document the agent org chart + arbiter contract in AGENTS.md (rpo-B), and add a background-agent heartbeat status ledger (rpo-C); the main gap is no direct grit→weave bridge today.
