---
name: forge-loop
description: "Run the envctl Feature Forge crew CONTINUOUSLY over a backlog — the Ralph loop. ALWAYS use when asked to: work through a backlog/list of features autonomously, 'keep building', 'loop on the roadmap', run Feature Forge 'until done'/'on repeat'/'unattended', or 'resume the loop' from a handoff. Each iteration does the next undone backlog item via the full architect→implementer→guardian cycle, checkpoints, and self-paces. At the per-session cycle budget it triggers session-relay to hand off to a fresh session. Do NOT use for a single one-off feature (use feature-forge directly) or for environment/install tasks."
---

# Feature Forge Loop (Ralph)

You run the Feature Forge crew as a **self-perpetuating loop** over a durable backlog, instead of
one feature at a time. The design is deliberately simple — the *Ralph* pattern: durable state on
disk, each iteration reads it, does the next undone thing, writes the result back, and re-fires.
The loop's intelligence lives in the **backlog file and checkpoints**, not in conversation memory —
that is exactly what lets a fresh session pick the loop up with zero loss (see `session-relay`).

## Why this shape
Conversation context rots and token cost climbs the longer a single session runs. A loop that
keeps all its truth in durable files (backlog + checkpoints) can be carried across many short, cheap
sessions instead of one long, expensive, degrading one. So: **never hold loop state only in your
head — write it down every iteration.**

## State precedence (pin this — agents never re-rank it)
**Git > `.handoff/ledger.db` > `tasks/*.task.json` > `active.md` > packet.** The two markdown loop
surfaces (`.handoff/loop/HANDOFF.md`, `.handoff/loop/backlog.md`) rank **below** all of these —
never treat HANDOFF.md or backlog.md as higher precedence than Git or the witnessed ledger. When the
ledger (via `hf`) and a markdown view disagree, **the ledger wins** and the markdown is corrected.

## Durable state (the loop's memory)
All under the worktree's `.handoff/loop/` (the audit trail; preserve it):
- **`.handoff/loop/backlog.md`** — the human markdown VIEW. An ordered checklist of work items, each
  `- [ ] <TASK-####>: <one-line goal>` (→ `- [x]` when its cycle PASSES; `- [!]` blocked;
  `- [!!]` SUPERVISED/CRITICAL — never auto-run, see below). Sub-notes indented beneath carry
  dependency hints. **No dependency edges live here** — it is a view, not the ordering authority.
- **`handoff.task.v1` cards** (`.handoff/tasks/*.task.json`) — the structured surface with
  `dependencies`/`blocked_by` + a `status` enum (`backlog|active|claimed|blocked|checkpointed|review|done`).
  **After Epic-A TASK-0002 mints the cards, the CARDS own ordering** and `backlog.md` is just a view;
  **before** cards exist, parse deps from the markdown sub-notes. **Never tick a box in `backlog.md`
  that disagrees with the card's `status`** — the card (ledger-replayed) is authoritative.
- **`.handoff/loop/loop_state.md`** — the ledger: `cycles_this_session`, `cycles_total`,
  `cycle_budget`, `session_started` (UTC, passed in — never call Date.now), `last_item`, `status`.
- **Per-cycle artifacts** — `.handoff/loop/cycle/01_architect_plan.md` /
  `.handoff/loop/cycle/02_implementer_log.md` / `.handoff/loop/cycle/03_guardian_report.md`
  for the item currently in flight (same as a single feature-forge run).
- **Sentinels** under `.handoff/loop/`: `DONE`, `NEEDS-HUMAN`, `STOP` (read/write semantics below).

## `hf`-aware vs markdown picking & checkpointing
**IF `hf` is on PATH _and_ the ledger-residency guard holds** (ledger = `$META_ROOT/.handoff/ledger.db`;
run every ledger-touching verb from `$META_ROOT` — see the `handoff-sync` skill): delegate
next-item selection and checkpointing to the kernel (real verbs below). **ELSE** fall back to the
markdown-checkbox + sub-note dependency parsing path. Re-run the residency fail-closed check before
each hf call; on failure, drop to the markdown path for that cycle.

**Per-cycle verb sequence (the REAL shipped `hf` verbs).** (The shipped binary also has `hf drift`,
`hf policy check-claim|check-edit|check-handoff`, `hf fleet`, `hf intake`, `hf dispatch`, `hf ship`,
`hf review` — verified 2026-06-18; an earlier note here wrongly said `drift`/`policy` did not exist.)
> **⚠ Member-scoped picking is BLOCKED on kernel HFTASK-0054 (verified 2026-06-18, TASK-0044).** The
> shipped `hf` is **CWD-relative for the ledger with no `--ledger`/`--member` override**. So the
> member-scoped *live picker* verbs (`hf resume --json` / `hf claim --next`) **cannot** run cleanly for
> envctl yet: run from the envctl member dir they **create a forbidden per-repo `.handoff/ledger.db`**
> (residency violation); run from `$META_ROOT` they read the **FLEET** ledger (the kernel's own
> `HFTASK-*` loop), **not** envctl's `TASK-*` cards. envctl's 53 dependency-DAG cards ARE minted
> (`.handoff/tasks/TASK-*.task.json`, per-member store, no contamination — TASK-0044), so the
> **dependency authority substrate exists**; only the live picker verb awaits HFTASK-0054.
> **Until HFTASK-0054 lands, pick like this:**
- **Pick / resume (TODAY):** read the authoritative open/DAG set with **`hf fleet render envctl`** from
  `$META_ROOT` — it renders envctl's packet from envctl's OWN cards (read-only on the FLEET ledger,
  creates **no** per-repo ledger, surfaces only `TASK-*`, zero `HFTASK-*` leak). Take the top unblocked
  `TASK-####` from that render. The markdown sub-note path remains the live-pick fallback. **Do NOT run
  `hf resume`/`hf claim --next` from the envctl member dir** (forbidden-ledger hazard) until HFTASK-0054
  adds a `--ledger`/`--member` override — at which point this reverts to `hf resume --json` +
  `hf claim --next` as the dep-DAG picker.
- **Cycle start:** `hf claim <TASK-####>` (witnessed claim; mesh-coordinated so two sessions can't
  grab the same task).
- **Mid-cycle:** `hf checkpoint --auto` (routine boundary) or `hf checkpoint --note "<reason>"`
  (notable state) — appends a witnessed ledger event. **Claim/checkpoint alone NEVER mark done.**
- **Cycle PASS (terminal Done only):** `hf done <TASK-####> --pr <N>` — the single verb that marks a
  task Done in the ledger; pass the merged PR number.
- After `hf done`, `hf handoff` re-renders `.handoff/packets/latest.md` + `.handoff/active.md`.
Markdown-fallback equivalents: pick = top unchecked unblocked `- [ ]` (deps from sub-notes);
"done" = tick `- [x]` in `backlog.md`.

If `.handoff/loop/backlog.md` does not exist, create it first from the user's request (a roadmap, a
doc, or an explicit list), then start the loop. Keep items small and independent — one Engine
capability or one component per item — so a cycle fits comfortably under the budget.

## One iteration (the loop body)
1. **Read state.** `.handoff/loop/backlog.md` + `.handoff/loop/loop_state.md`. Confirm the worktree is
   clean (`git status`) and on the loop branch.
2. **Phase-0 stop checks (read ALL THREE sentinels first, in order):**
   - `.handoff/loop/STOP` present → **halt immediately**, no re-fire (human kill switch; takes
     priority over everything).
   - `.handoff/loop/NEEDS-HUMAN` present → stop and surface for a human; do not auto-pick around it.
   - `.handoff/loop/DONE` present **OR** completion confirmed (see below) → **DONE**: report, no re-fire.
     *Completion is confirmed when* `hf resume --json` reports `next_command: "done"` (hf present) **or**
     all cards are `status: done` / all `backlog.md` items are `- [x]`/`- [!]` (hf absent).
   - `.handoff/loop/WRAP-UP-OWED` present → a batch boundary came due (the Stop/PreCompact hook
     dropped it) and the owed wrap-up has not run → **run the batch boundary now** (see "Batch
     wrap-up cadence" below) BEFORE picking another item; do not pick around an owed wrap-up.
   - **Batch boundary due** — `cycles_total - last_wrapup_total >= wrap_every` → run the batch
     boundary (reaper + wrap-up reconcile + evolution-steward), then continue if still under
     `cycle_budget`. This fires *inside* a session; it is NOT a hand-off.
   - `cycles_this_session >= cycle_budget` → **HAND OFF**: invoke the `session-relay` skill and stop
     (do not re-fire from this session). This is the cycle-budget trigger (always also runs the
     boundary work as part of wrap-up, so a hand-off never skips a reap/retro).
3. **Pick** the next item:
   - **hf present:** take `next_task_id` from `hf resume --json` (the `next_safe` DAG picker) and
     `hf claim <TASK-####>`.
   - **hf absent:** the top unchecked unblocked `- [ ]` item, honoring deps parsed from sub-notes.
   - **`- [!!]` SUPERVISED/CRITICAL refusal:** if the picked item is marked `- [!!]` (e.g. the
     rtk-hook install, a live n8n/smoke test), the loop **REFUSES to auto-run it** — write
     `.handoff/loop/NEEDS-HUMAN` (with the item id + why it needs a human), do **not** claim/build it,
     and stop. Never auto-run a supervised item.
4. **Run one Feature Forge cycle** on it via the `feature-forge` orchestrator: architect → implementer
   → guardian, with the same routing/loop caps and `.handoff/loop/cycle/` artifacts. Mid-cycle, when
   hf is present, emit `hf checkpoint --auto` (or `--note`) at notable boundaries (claim/checkpoint
   **never** mark done). Commit on PASS / PASS-WITH-NOTES (area-prefixed subject).

   > **TICK-ON-MERGED, not tick-on-armed (status-integrity gate — non-negotiable).** Guardian PASS +
   > `gh pr merge --auto` *arms* the merge; it does NOT complete it (a required check — usually Format
   > — can still block it). So a cycle reaches **terminal Done** ONLY after the PR is confirmed
   > merged: `gh pr view <N> --json state,mergeStateStatus -q .state` returns `MERGED`. Until then the
   > item stays **in-flight `- [~]`** (PASS, armed, not merged) — never `- [x]`, never `hf done`.
   > - **Merged** → mark terminal Done: `hf done <TASK-####> --pr <N>` (hf present; the only verb that
   >   marks done) then `hf handoff` to re-render the packet; markdown fallback → tick `- [x]`.
   > - **Armed-not-merged at the cycle boundary** → leave `- [~]`, record `pr=<N> state=<status>` in
   >   `loop_state.md`, and make the **next session's FIRST action** re-poll `gh pr view <N>` and
   >   promote `- [~]`→`- [x]` once `MERGED` (this is what stops the #125 tick-before-merge drift).
   >   Never tick a sibling/superseding reconcile box for a PR that has not merged.

   On an **unrecoverable guardian FAIL or NEEDS-DECISION the loop can't route around**, write
   `.handoff/loop/NEEDS-HUMAN` and mark the item `- [!]` blocked with a one-line reason, then move to
   the next item (don't thrash).

   > **Multi-repo cycle (A2):** if the architect's plan for this item lists **>1 target repo**,
   > step 4's build runs the A2 shape (feature-forge **Phase 1.5 → Phase 2-A2**): one coordinated
   > meta worktree set, N implementers, per-repo guardian gates. The loop itself is unchanged —
   > still **one backlog item per cycle**, A2 is just the internal shape of that cycle's build.
   > Run `grit gc` per repo before each wave and keep heartbeat hygiene (the implementers refresh
   > their own TTLs). The cycle completes only when **all** target repos reach guardian PASS (or
   > are marked `- [!]` blocked). Cycle-budget counting and the `session-relay` handoff are
   > unchanged — an A2 cycle is still one cycle against the per-session budget.
5. **Write state back:** tick the markdown VIEW (`- [x]` done **only when MERGED-confirmed per the
   gate above** / `- [~]` in-flight if armed-not-merged / `- [!]` blocked) **only if it agrees
   with the card status** — when hf is present the card (ledger-replayed via `hf done`) is
   authoritative; reconcile the box to the card, never the reverse (use `hf sync-cards` to re-derive
   cards if they look stale). Increment `cycles_this_session` and `cycles_total`, update `last_item`
   and `status` in `loop_state.md`, and append a one-line progress note. Commit the `.handoff/` update
   (text only — never a `ledger.db`).
6. **Re-fire silently** to continue the loop (see Self-pacing). **No per-task pause/summary** — write
   the result to `.handoff/loop/` and go straight to the next item. A consolidated, user-facing
   summary is produced ONLY at the batch boundary (every `wrap_every` cycles) and at HAND OFF — this
   is what keeps per-task context lean enough to fit 5–8 cycles in one session. (One PR per cycle is
   unchanged; tick-on-merged still gates terminal Done. Removing the *narration* between tasks is the
   only change — the durable state is still written every cycle.)

## Worktree hygiene (keep worktrees ↔ branches ↔ origin in sync)
Each cycle runs in a fresh `meta/.worktrees/<slug>/envctl` worktree off develop; on PR auto-merge,
origin deletes the head branch (`delete_branch_on_merge=true`) but the local worktree/branch/tracking
ref do **not** self-clean — left unmanaged they pile up (this is what produced a 46-worktree /
85-branch mess). The loop does **not** reap mid-cycle (a PR may still be merging asynchronously);
instead the reap runs at the safe boundaries where merge status is settled:
- **`session-relay-resume`** reaps at session start (clean slate), and
- **`session-relay-wrap-up`** reaps after the handoff commit.
Both call `scripts/reap-worktrees.sh` — dry-run by default, never `--force`, protects
`master`/`develop`/the current worktree, skips any dirty worktree, and only reaps a branch whose
upstream is `[gone]` (merged) or that is an ancestor of `origin/master`. You may also run
`bash scripts/reap-worktrees.sh` anytime to preview, or `--apply` to reap on demand. The reaper
also **fast-forwards the protected trunk branches** (`master`, `develop`) to origin — FF-only and
clean-only (an ahead/diverged/dirty branch is left untouched) — so the main checkout's `master`
mirror and `develop` stay in lockstep with origin without a manual merge after each develop push.

## Batch wrap-up cadence (the periodic boundary — keeps long sessions from drifting)
The loop runs tasks **back-to-back with no per-task pause** (one PR each, tick-on-merged), and batches
the heavy continuity work to a **boundary every `wrap_every` completed cycles** (default 5; set in
`loop_state.md`). At the boundary — `cycles_total - last_wrapup_total >= wrap_every`, or a
`WRAP-UP-OWED` marker is present — run, in order:
1. **Reaper** — `bash scripts/reap-worktrees.sh --apply` (clears the merged per-cycle worktrees/branches
   that accrued over the batch; safe-by-construction, skips dirty, FF-syncs trunk).
2. **Wrap-up reconcile** — `session-relay-wrap-up` steps 3b (backlog status-truth, MERGED-gated) + the
   ICM store; this is also where the consolidated user-facing summary of the batch is produced.
3. **Evolution-steward** — the retro over the batch (mine lessons → `LESSONS.md` / `proposed-upgrades.md`).

Then **clear the marker and set `last_wrapup_total = cycles_total`** so the next boundary is measured
from here. The boundary is *in-session* — it is NOT a hand-off; after it, continue picking items if
still under `cycle_budget`.

**Why a marker + hook (don't rely on remembering):** the boundary is an agentic step, and skipping it
is precisely what let 46 worktrees pile up. The Stop/PreCompact hook (`.claude/hooks/hf-checkpoint.sh`)
drops `.handoff/loop/WRAP-UP-OWED` the moment a boundary comes due, and `session-relay-resume` is
**fail-closed** on that marker (it runs the owed wrap-up before any new work). So even a session that
forgets the boundary cannot silently skip it — the next resume catches it, bounded to one inter-session
gap. The hook itself does only cheap file I/O (it does **not** run the reaper from a per-turn hook — git
fetch + worktree removal interleaving with the agent mid-cycle is unsafe; the reaper runs at the
agentic boundary and at resume, the settled points where merge status is known).

`wrap_every` (in-session continuity cadence) and `cycle_budget` (when to hand off to a fresh session)
are independent: with `wrap_every=5, cycle_budget=8` a session does up to 8 cycles with a wrap-up at
cycle 5 and again at hand-off; with the heavy-context `cycle_budget=1`, every cycle hands off (wrap-up
runs anyway), so the boundary only changes behavior once `cycle_budget>1` (the unattended `auto-provision`
runner, or attended batch runs).

## Parallel mode (opt-in grit git-lock coordination)

When looping over items that span multiple meta repos, activate with `USE_GRIT=1`:

1. Before the first implementer: `for repo in $(meta project list --json | jq -r '.[].name'); do cd /home/drdave/Desktop/meta/$repo && grit init -y; done` (idempotent).
2. Each implementer claims symbols via `grit claim file::symbol --with-deps` before writing, `grit done` after commit.
3. Contested symbols auto-queue (`grit claim --queue`).

Parallel mode is **opt-in** — the default single-implementer path is unchanged. See `feature-forge/SKILL.md` for full details on the parallel protocol (claim→work→done, `--queue`, `--with-deps`, CLI-only constraints).

## Self-pacing (how the loop re-fires)
- Default: **dynamic /loop** — use `ScheduleWakeup` to re-enter this skill for the next iteration,
  passing the same `/forge-loop …` prompt verbatim so the next firing repeats the body. Pick the
  delay by what you're waiting on; for back-to-back build iterations a short warm-cache delay
  (≤270s) is fine. When you HAND OFF or finish, **omit** the ScheduleWakeup call to end the loop.
- Alternative: a fixed interval (`/loop <interval> /forge-loop …`) when the user wants paced runs.
- A cycle counts only when a Feature Forge cycle **completes** (PASS/PASS-WITH-NOTES/blocked) — a
  re-fire that does no work (e.g. waiting) does not increment the ledger.

## Cycle budget (the handoff trigger)
The per-session budget is **cycles-only** (no token-meter guessing): default **3** completed cycles
per session unless the user sets another (`/forge-loop budget=N …`). Record it in `loop_state.md`.
When `cycles_this_session` reaches it, you do **not** start another cycle — you invoke
`session-relay`, which checkpoints + announces + schedules the successor, then you stop. The
successor resets `cycles_this_session` to 0 and continues where the backlog left off. This keeps
every session short, cheap, and well below context rot — by construction, not by measurement.

> **`cycle_budget` (hand-off) vs `wrap_every` (in-session boundary) — distinct knobs.** `cycle_budget`
> ends the *session*; `wrap_every` (see "Batch wrap-up cadence") triggers the periodic reaper + wrap-up
> + retro *within* a session without ending it. The hand-off at `cycle_budget` always runs the boundary
> work too (it's part of wrap-up), so a session never ends without a reap/retro. For attended bounded
> runs keep `cycle_budget` small; for unattended batch runs (via `auto-provision`) set `cycle_budget`
> to the 5–8 the context comfortably holds and let `wrap_every` keep the workspace clean mid-session.

> **Truly-unattended runs: use the external runner, don't rely on the in-session cron.** The
> successor cron scheduled by `session-relay` is **session-only** in this runtime (it does not
> survive process exit), so an in-session `/forge-loop` cannot actually self-perpetuate past one
> handoff — which is why heavy-context sessions end up pinned at `cycle_budget: 1`. For real
> set-and-forget operation (fresh context every cycle, budget>1 honored), launch via the
> **`auto-provision`** runner (the bundled Ralph loop that spawns a fresh `claude -p` per cycle).
> In-session `/forge-loop` remains correct for attended, bounded runs.

## Resume (entering mid-loop from a handoff)
If invoked to **resume** (a `.handoff/loop/HANDOFF.md` exists, or weave inbox / the successor cron
prompt says so): follow `session-relay`'s resume protocol first (read HANDOFF + ack via weave +
reset `cycles_this_session`), then run the iteration body normally. **When hf is present, the
authoritative resume read is `hf resume --json`** (`next_task_id`/`next_command`) from `$META_ROOT`,
not the markdown — HANDOFF.md is the companion. When hf is absent, resume from the backlog's current
item per the markdown.

## Stop conditions & sentinel write semantics (end the loop — no re-fire)
Sentinels live under `.handoff/loop/`. **Phase-0 reads all three (STOP, NEEDS-HUMAN, DONE) before
picking;** write them as follows:
- **DONE** — write `.handoff/loop/DONE` only when completion is *confirmed* AND the left-behind
  sweep clears:
  1. *Completion confirmed:* `hf resume --json` reports `next_command: "done"` (hf present) **or** all
     cards are `done` / all backlog items `- [x]`/`- [!]` (hf absent).
  2. **Pre-DONE left-behind sweep (independent re-derivation — fail-closed).** Before writing DONE,
     spawn an independent completeness critic (an `invariant-guardian` pass scoped to "what's missing")
     that **re-derives the expected surface from the plans/specs** (`.handoff/loop/cycle/*/01_architect_
     plan.md` `## Unit ledger` rows for the completed items, plus the backlog goals + `docs/`) and
     **diffs it against the delivered code** — NOT against the backlog's own `- [x]` marks (the backlog
     trusting itself is exactly the drift). Every ledger unit must be present+wired in HEAD. If the
     re-derivation surfaces an un-built/unwired unit, or it cannot re-derive a scope at all (zero/partial
     harvest), the result is **INCONCLUSIVE → write `.handoff/loop/NEEDS-HUMAN`** with the gap, NOT DONE.
     "Clean" requires a positive re-derivation that matches, not the mere absence of open `- [ ]`.
  Only when BOTH hold: write DONE, report the summary, no re-fire.
- **NEEDS-HUMAN** — write `.handoff/loop/NEEDS-HUMAN` on (a) an unroutable guardian **FAIL** /
  NEEDS-DECISION, **or** (b) encountering any `- [!!]` SUPERVISED/CRITICAL item (rtk-hook, live
  smoke). Stop and surface for a human; do not auto-pick around it.
- **STOP** — `.handoff/loop/STOP` is the human kill switch: when present, **halt re-fire
  immediately**, ahead of all other checks.
- **Cycle budget reached** → hand off (session-relay), then stop.
- **A hard blocker the loop can't route around** (dirty/ambiguous worktree, repeated guardian FAIL on
  the same item) → write NEEDS-HUMAN, stop and report; don't burn cycles spinning.
- The user interrupts.

## Test Scenarios
**Happy path:** `/forge-loop budget=3` with a 7-item backlog. Iterations 1-3 each complete a feature
(architect→implementer→guardian PASS, committed), ticking items and incrementing the ledger. After
cycle 3, the stop check trips the budget → `session-relay` writes HANDOFF, weave-announces, schedules
a durable-cron successor, and this session stops. The successor fires, resets the session counter,
and continues at item 4.

**Error path:** Iteration 2's item needs a banned C dep (guardian FAIL the architect can't route
around). The loop marks item `- [!] blocked: needs C SQLite — out of bounds`, commits the backlog
update, and proceeds to item 3 rather than thrashing. The blocked item surfaces in the DONE/HANDOFF
summary for a human decision.
