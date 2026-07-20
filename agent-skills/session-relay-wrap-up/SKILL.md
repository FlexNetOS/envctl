---
name: session-relay-wrap-up
description: >-
  Full end-of-session wrap-up + handoff for a harness loop (invoked as /session-relay-wrap-up, or
  /harness:session-relay-wrap-up). ALWAYS use to close a session at cycle budget, on STOP, at a
  forge-loop BATCH BOUNDARY (every `wrap_every` cycles, or when `.handoff/loop/WRAP-UP-OWED` exists),
  or when the owner says "wrap up", "wrap up the session", "hand off", "checkpoint and stop",
  "prep handoff", "close out". Runs the retro, persists durable memory to ICM, writes + commits the authoritative
  HANDOFF.md, broadcasts the weave heartbeat, arms a best-effort successor, then stops. The committed
  HANDOFF.md is the resume signal — weave is only the heartbeat.
---

# session-relay-wrap-up — the full wrap-up + handoff

## Two modes: HAND-OFF vs BATCH BOUNDARY
This skill runs in two modes that share steps 2–5b:
- **HAND-OFF** (cycle budget reached / STOP / owner "wrap up"): run **all** steps 1–8 — including the
  weave heartbeat, the successor cron, and **stop** the session.
- **BATCH BOUNDARY** (forge-loop hit `wrap_every`, or `.handoff/loop/WRAP-UP-OWED` exists, mid-session):
  run steps **2, 3, 3b, 4, 5, 5b** (retro → ICM → backlog reconcile → checkpoint → commit → reap),
  then **clear the marker, set `last_wrapup_total = cycles_total` in `loop_state.md`, commit that**,
  and **return to the loop** (do NOT do 6–8; the session is not ending). This is the periodic
  reaper + reconcile + retro that keeps a long batch run from drifting.

The clean way to end a loop session so the next one resumes cold with zero loss. It composes the
harness's continuity primitives into one ordered, idempotent sequence. Pairs with
`session-relay-resume`. (Generalizes the weave-loop `session-relay` HAND OFF entry point, adding the
**Phase E retro** and explicit **ICM persistence** so lessons and decisions survive the boundary, not
just the loop state.)

## Run this sequence (each step idempotent; stop on a terminal sentinel)

1. **Stop-checks first.** If `.handoff/loop/STOP` or `.handoff/loop/NEEDS-HUMAN` already exists, the
   run already terminated — log it and exit without re-handing-off.

2. **Phase E retro** — invoke `evolution-steward` (skill `harness-evolution`) for the lightweight
   retro: evaluate the session (friction / gate quality / coverage / human walls), append lessons to
   `LESSONS.md`, and write any `proposed-upgrades.md`. Capturing lessons *now* is why they survive the
   budget boundary. (Defer *applying* structural upgrades — wrap-up only records.)

3. **Persist durable memory to ICM** (the store half, symmetric to resume's recall — mirrors the
   `icm hook end` / `icm-memory` discipline). Store on the triggers that fired this session, before
   committing:
   ```bash
   icm store -t decisions-<harness> -c "<design decision + why>"            -i high   -k "kw1,kw2"
   icm store -t errors-resolved     -c "<what broke + the fix>"             -i high   -k "kw1,kw2"
   icm store -t context-<harness>   -c "<session summary: units done, next>" -i high   -k "kw1,kw2"
   ```
   Prefer the MCP tools (`mcp__icm__icm_memory_store`) when available. Do NOT store ephemeral state
   (build logs, git status) — that lives in `.handoff/loop/`. ICM holds the *why* and the lessons.

3b. **Backlog reconcile — FAIL-CLOSED (the anti-drift gate).** The backlog (`.handoff/loop/backlog.md`)
   is a **written-back artifact, not read-only input**. Before the checkpoint, reconcile it or the
   handoff is INCOMPLETE (this is the mechanism that stops "follow-ups discovered mid-cycle live only
   in a PR body / cycle artifact / ICM and get forgotten"):
   - **Append every discovered follow-up** as a new `- [ ]`/`- [?]`/`- [!]` item **with its origin**
     (PR#, cycle file, ICM id). Sources to drain THIS cycle: the guardian's `PASS-WITH-NOTES` notes,
     the implementer log `## Deviations`, the PR body's "Follow-ups / out-of-scope / deferred" section,
     and any `icm store` you just wrote. If a follow-up isn't in the backlog after this step, you have
     NOT finished wrap-up.
   - **Build-to-the-frozen-contract check.** Before treating a feature as done, grep the backlog for an
     existing TASK describing the SAME capability — if it pins a *frozen consumer contract* (a CLI
     flag/JSON shape, an RPC name, a downstream caller), verify the delivered surface matches THAT
     contract, not a parallel one. A mismatch = the TASK is **PARTIAL**, recorded as such with the
     exact gap (e.g. G2/TASK-0020: built `relay mint --mode native`, but the frozen contract was
     `secretctl mint-github → {token,expires_at_unix}` — App still 404s). Never mark a frozen-contract
     TASK done on a near-miss surface.
   - **Status-truth reconcile (TICK-ON-MERGED).** Diff backlog `[ ]` items against **merged** PRs /
     live code; flip a stale `[ ]` to `[x]` ONLY when `gh pr view <N> --json state -q .state` returns
     `MERGED` — cite the PR. A PR that is merely *armed* (guardian PASS + `gh pr merge --auto`, not yet
     merged) is recorded `- [~]` in-flight, NOT `- [x]`; never tick a reconcile/superseding box for an
     unmerged PR (this is the #125 failure: TASK-0027 was ticked before its PR merged, then had to be
     retired as superseded). A stale `[ ]` on done work pollutes the next pick; a premature `[x]` on
     unmerged work hides a not-actually-landed change.
   - **Promote cross-namespace residuals.** Any non-loop-local open item in a namespaced sub-loop
     (`.handoff/loop/<sub>/HANDOFF.md`, e.g. rust-port) gets promoted into the flat backlog.
   - **Drain `proposed-upgrades.md` — FAIL-CLOSED (the self-improvement closure gate).** The
     evolution-steward (Phase E, step 2) writes structural harness proposals to
     `.handoff/loop/proposed-upgrades.md`; they are ESCALATED, not auto-applied. Nothing else tracks
     them to a decision, so they silently accumulate (the audit found 49 undrained lines). Before the
     checkpoint, **drain every entry** in that file to a tracked disposition:
     - **Still open** → append a `- [?]` harness-upgrade item to the backlog citing the proposal, so it
       enters the normal pick flow with an owner-decision status.
     - **Already addressed** (a prior cycle shipped it — verify against HEAD, don't assume) → record it
       resolved with the commit/PR, do NOT re-open.
     - **Declined / accept-as-is** (owner chose an option, or "do nothing" is the recommendation) →
       record the disposition.
     Then **reset `proposed-upgrades.md` to its drained header** (empty body) so a non-empty file always
     means "undrained proposals exist." A non-empty `proposed-upgrades.md` at the end of wrap-up means
     wrap-up is **INCOMPLETE** — same fail-closed shape as the follow-up drain above.
   - Commit these backlog edits as part of step 5 (they ARE handoff state).

4. **Write the checkpoint** — spawn `continuity-steward` with the worktree, the in-flight cycle, and
   the orchestrator pipeline state. It writes the cold-start `.handoff/loop/HANDOFF.md` (layout below)
   in one pass, keeping the orchestrator's context lean. Overwrite — the steward body is authoritative.
   If the meta handoff kernel (`hf`) is reachable, prefer `hf checkpoint` / `hf handoff` to render the
   packet from the witnessed ledger; the file-based form is the fallback.

5. **Commit** — `chore(<harness>): handoff (at <item>)`, including `HANDOFF.md` + `.handoff/loop/`
   state + any wrap-up edits. **A fresh process must resume from this commit alone** — this is the
   real payload.

5a. **Satisfy the batch boundary (both modes).** Set `last_wrapup_total = cycles_total` in
   `loop_state.md` and remove `.handoff/loop/WRAP-UP-OWED` if present (`rm -f`), so the cadence is
   measured from here and the fail-closed resume check is cleared. Commit this with step 5 (BATCH
   BOUNDARY: this is the commit that lets the loop continue; HAND-OFF: it rides along with the handoff
   commit). Skipping this would re-trigger the boundary every turn via the hook.

5b. **Reap merged worktrees/branches (keep the workspace in sync).** The loop creates a fresh
   `meta/.worktrees/<slug>/envctl` per cycle; `<slug>` is the managed meta worktree-set slug and
   `envctl` is only the repo checkout name. Once a cycle's PR auto-merges, origin deletes the head
   (`delete_branch_on_merge`) but the *local* worktree/branch/tracking-ref linger and pile up. After
   the handoff commit, run the reaper to mirror origin's cleanup locally:
   ```bash
   bash scripts/reap-worktrees.sh            # preview first
   bash scripts/reap-worktrees.sh --apply    # reap merged/clean worktrees + branches + prune refs
   ```
   If you need to check a meta worktree set before/after reaping, derive it from path shape first:
   `bash scripts/reap-worktrees.sh --managed-worktree-slug <worktree-dir> envctl`; never pass the
   repo name `envctl` as a guessed slug for the main checkout.

   It is **safe by construction**: dry-run by default, never `--force`, protects `master`/`develop`/the
   current worktree/branch, **skips any dirty worktree** (uncommitted work is never destroyed), and
   never touches remotes. A branch is reaped only when its patches are already represented on
   `origin/master` (ancestor or squash/patch-equivalent). `[gone]` is only a diagnostic that the
   temporary remote branch disappeared, not proof of merge. This is the step that stops the
   46-worktree / 85-branch pileup.
   > **Local reap vs. remote delete (irreversible-action discipline).** The reaper is LOCAL-only by
   > design; origin self-cleans merged heads. If you ever delete **origin** branches manually (an
   > irreversible off-box action), that is a human wall: get explicit owner authorization, write a
   > recovery manifest of the refs+SHAs first, and confirm each PR actually `MERGED` via the GitHub
   > oracle (`gh pr view <ref> --json state`) — `[gone]`/ancestor are NOT merge proof (a
   > closed-unmerged head reads identically locally; cf. PR #99, 2026-06-18).

6. **Weave heartbeat (best-effort)** — broadcast `to:"all"`:
   `weave send --to all --subject "relay:handoff" --body "worktree=<abs> item=<next> reason=<budget|stop>"`.
   Bootstrap-hazard guard: if *this harness's own messaging code* is in the diff this cycle, skip the
   heartbeat and log the skip — the committed file is the truth.

7. **Best-effort one-shot successor** — `CronCreate {recurring:false}` ~3 min out, self-describing:
   `"/session-relay-resume from .handoff/loop/HANDOFF.md (worktree=<abs>, model=opus)"`. Session-only
   in this runtime; the committed HANDOFF.md is the survives-restart signal (a human or the external
   runner resumes from it).

8. **Stop** — no `ScheduleWakeup`. The next runner iteration spawns a fresh `Codex -p` (the `/new`
   effect) which enters `session-relay-resume`.

## What `HANDOFF.md` must contain (cold-start test)

A successor given ONLY this file + the repo must resume correctly. Required:
```markdown
# HANDOFF — <harness>
closed_utc: <UTC>           branch: <branch>      worktree: <abs path>
cycle_budget: <n>           cycles_total: <N>     cycles_this_session: <n>
last_item: <id>             next_item: <id>       orchestrator_phase: <phase>
last_agent: <name>          gate_status: <PASS|FAIL|n/a>   pr_url: <url|(none)>
landed_this_session:
  - <sha> <subject>
findings: <pointers into .handoff/loop/findings/*.md — do not inline>
decisions_and_dead_ends: <what would otherwise be re-litigated/re-tried>
icm_stored: <topics written this session, so resume recalls them>
verify_on_resume: <the exact commands the successor runs FIRST to confirm green>
resume_command: /session-relay-resume from .handoff/loop/HANDOFF.md
```

## Non-negotiables
- **Write state down, then commit** — never hold the plan only in context.
- **The committed HANDOFF.md (or `hf` packet) is authoritative** — not the weave inbox (a self-
  addressed message doesn't land in your own inbox; a same-machine successor shares your identity).
- **Capture lessons + store memory before stopping** — the retro and ICM store are part of wrap-up,
  not optional afterthoughts; they're what makes the *next* session smarter, not just unblocked.
