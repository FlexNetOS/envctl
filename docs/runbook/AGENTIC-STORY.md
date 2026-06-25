# envctl — The Agentic Story (full agent-automation narrative)

> Companion to [`DIAGRAMS.md`](DIAGRAMS.md) §14–§16 and [`USER-STORY.md`](USER-STORY.md).
> This is the story of how envctl — meta's agentic environment manager — **builds, provisions, and
> maintains the whole meta workspace with no human in the loop**, on its deployment target today (a
> dual-RTX-5090 Ubuntu 26.04 workstation) — and where, by deliberate design, it stops and asks for one.

The thesis: **the box is a declarative target, and a swarm of agents converges reality onto that
target on a loop.** Nothing here is "an AI helping a human run commands." The human writes intent
(a backlog item, a doctrine) and tunes a few gates; the agents do the building, verifying,
merging, remembering, and self-correcting. The human is a *wall the loop respects*, not a driver
it waits on.

---

## 1. The substrate: a declared state + a convergence engine

Everything the box should be is declared as data:

- **`manifest/*.toml`** — 79 components (§11), each a Detect→Install→Verify→Fix→Remove lifecycle
  wrapping proven bash. The desired state of every tool, daemon, font, driver, and symlink.
- **`agent-env.yaml` → `agent-env.lock`** — the desired state of the agent environment itself
  (skills / MCP servers / slash-commands), content-hashed.
- **`.handoff/loop/backlog.md` + `tasks/*.task.json`** — the desired state of the *work*: the
  ordered list of features/fixes/upgrades still to build.

The **`Engine`** (one shared, sync, non-printing Rust library) is the convergence primitive: it
reads reality (`auto-detect` → `EnvReport`), diffs against the manifest, and converges (`install`
acts additively; `auto-fix`/`reset` mutate only with `--apply`). `envctl.lock` is the
content-hash that proves a converged box is reproducible. This is the inner loop — a single
`envctl` invocation. The agentic story is what wraps it.

---

## 2. The crew (who does what)

The harness is a **construction crew**, each agent a specialist with one job. The orchestrator
(opus, the main loop) routes between them and owns every gate.

| Agent | Role | Mutates? |
|-------|------|----------|
| **feature-architect** | Turns a backlog item into an invariant-aware plan + unit ledger | read-only |
| **rust-implementer** | Writes the Rust (engine first, then thin CLI/GUI), in a worktree | yes |
| **invariant-guardian** | Runs the CI gates + cargo + **runtime-verify**; PASS/FAIL verdict | runs scripts |
| **handoff-kernel-engineer** | Builds/relocates the `hf` kernel; upholds ledger invariants | yes |
| **continuity-steward** | Writes the cold-start `HANDOFF.md` at a budget boundary | yes (state) |
| **evolution-steward** | Post-run retro: mines lessons → harness upgrades (PR) | yes (harness) |
| **build-health-auditor** | "Is it green?" baseline across the workspace | runs scripts |
| **rust-port-\*** crew | Port-and-merge of an external repo into Rust (X→Y) | yes |

Plus **parallel qwen3.6 background sessions** (`ollama launch claude --model qwen3.6`): cheap
legwork drafters. They sketch future units, draft candidate TOML, pre-read files — but **opus
gates everything they produce** (qwen has, e.g., invented a manifest schema and missed a security
design flaw; its output is always a sketch, never a merge).

---

## 3. The cycle (one backlog item, end to end, unattended)

This is the heartbeat. `forge-loop` runs it once per item:

```
 pick item ─▶ architect plan ─▶ implementer builds ─▶ guardian verifies ─▶ PR + auto-merge
                                                                              │
                          tick item ◀── TICK-ON-MERGED (gh pr view == MERGED) ┘
```

Key disciplines that make it safe to leave alone:

- **One PR per cycle.** Small, reviewable, revertible units.
- **TICK-ON-MERGED, not tick-on-armed.** Arming `gh pr merge --auto` is not done — a required
  check (usually Format) can still block. An item stays `- [~]` (in-flight) until
  `gh pr view <N>` returns `MERGED`; the next session re-polls and promotes it. This is what
  stopped the historic "ticked before merge" drift.
- **Runtime-verify gate.** The guardian doesn't just compile and run tests — it *runs the app at
  its real surface* (the CLI command, the daemon socket, the GUI) and captures evidence. "Compiles
  + gates green" is not PASS; "I drove it and saw it work" is.
- **Isolated worktrees.** Each cycle runs in `meta/.worktrees/<slug>/envctl`, so parallel cycles
  (and parallel repos, the A2 mode) never collide. The reaper cleans them at safe boundaries.
- **Fail-closed routing.** An unroutable guardian FAIL marks the item `- [!]` blocked and moves
  on — the loop does not thrash on one item.

---

## 4. The convergence loops (same shape, different target)

The cycle generalizes into three named loops:

- **`forge-loop`** — over a *feature/upgrade backlog*. Builds envctl capabilities, components,
  Epic-H convergence items, secrets-stack features.
- **`env-install-loop`** — over *provisioning*. Drives `doctor` → `install` → `auto-fix` until
  `doctor` is green and the box is fully set up and drift-free. Includes a component-audit phase
  that deep-probes each component (real exercise, version currency, advisories, cross-component
  skew) and appends evidence-based upgrade items back to the backlog.
- **`auto-provision`** — the *external runner*. Because an in-session cron is session-only in this
  runtime, true set-and-forget operation spawns a **fresh `claude -p` per cycle** (the `/new`
  effect) wrapping `env-install-loop`. This is how the box provisions overnight with a clean
  context every iteration and a real `cycle_budget > 1`.

All three share the same continuity layer (§5) and the same gates (§6).

---

## 5. Continuity: why it never loses its place

The loop's intelligence is on disk, not in chat — that's what lets cheap, short sessions chain
into long campaigns without context rot.

- **`hf` kernel** witnesses every `claim` / `checkpoint` / `done` into a ledger (state precedence:
  Git > ledger > task cards > active.md > markdown views).
- **`.handoff/loop/`** holds the human-readable views + the sentinels.
- At a **`cycle_budget`** boundary, `continuity-steward` writes a cold-start `HANDOFF.md`,
  announces over the **weave** bus, and the successor resumes with zero loss.
- A **`wrap_every`** (default 5) in-session boundary batches the heavy work — reaper, status-truth
  reconcile, evolution-steward retro — without ending the session.
- **Hooks** (Stop / PreCompact) auto-checkpoint and drop a `WRAP-UP-OWED` marker, and resume is
  **fail-closed** on it: a missed boundary is caught at the next resume, bounded to one gap.
- **ICM** is the cross-session semantic memory: decisions, owner preferences, resolved errors —
  recalled at the start of work, stored the moment a trigger fires.

---

## 6. The gates (why "autonomous" isn't "unsafe")

Autonomy is bounded by non-negotiable, machine-checked invariants. The guardian (and CI) enforce
them every cycle; a change that breaks one is a regression, not a feature:

- **No C in the trust boundary** (`ci/gates/no-c.sh`) — libSQL `remote` only, pure-Rust crypto.
- **Exactly one rustls, ring-only**; the engine stays sync + non-printing; CLI and GUI share it.
- **Destructive ops fail-closed + dry-run by default** — guards refuse when they can't prove
  safety.
- **Loop-state integrity** (`ci/gates/loop-state.sh`) — counters are ints, monotonic, cadence ≥ 1.
- **agent-env no-drift** (`ci/gates/agent-env.sh`) — `agent-env.yaml ↔ agent-env.lock`.
- **p7-conformance** (`ci/gates/p7.sh`) — `.handoff` Tier-A schema + ledger residency.

The loop can build anything *except* something that trips a gate. That's the whole safety model:
move fast inside a fence that cannot be moved by the thing inside it.

---

## 7. Self-improvement (the loop edits the loop)

After each run, the **evolution-steward** runs a retro: it mines generalizable lessons from what
happened (what was correct-by-instinct but unencoded, what drifted, what a pattern from a sister
harness would have caught) and turns them into **harness upgrades** — edits to skills, agent
defs, the orchestrator, or bundled scripts. It is propose-by-default and fail-closed: it
auto-applies only low-risk, in-scope edits via PR, **never weakens a gate**, and escalates
structural changes for owner approval. Lessons land in `LESSONS.md`; escalations in
`proposed-upgrades.md` (drained fail-closed at wrap-up). The harness that built the box this week
is measurably better than the one that built it last week.

---

## 8. What is fully automated, end to end

With the owner's doctrine and gates in place, the following run **with no human**:

- Detecting drift and converging every meta-prefix toolchain (Epic H): gh, nushell, zellij, mise,
  ollama, llvm-clang, libgccjit, CUDA toolkit, yazi, helix, huggingface-cli, wild-linker, kache,
  nix-portable, the secrets stack — all to `.toolchains` + `~/.local/bin`, verified on-box.
- Building features through architect→implementer→guardian→PR→merge, with runtime proof.
- GitHub fetches at 5000/hr via authenticated `gh` (a meta-owned component), so rate limits stop
  blocking installs.
- Worktree/branch hygiene (the reaper), trunk fast-forward, lock regeneration, CI-gate execution.
- Cross-session handoff, resume, checkpointing, and memory.
- Sudo steps — because `sudo -n` is passwordless here, `needs_sudo` components still run unattended
  (`[A*]`).

---

## 9. Where the loop deliberately stops (the five human walls)

Autonomy is *not* permission to cross irreversible or live-system lines. The loop refuses these,
writes a sentinel, and stops — it never improvises around them (see [`USER-STORY.md`](USER-STORY.md)):

1. **A reboot** — the 595→610 driver bump (kernel module reload). Held to the very end.
2. **A live-shell migration** — `/nix` removal + yazelix repoint (TASK-0067, `[!!]`): it touches
   the owner's running terminal; needs a human window.
3. **A secret reveal / passphrase unlock** — `--reveal --apply --confirm`; the passphrase path is
   owner-only (USB possession auto-unlocks; passphrase does not).
4. **An owner-sudo cleanup** — `apt remove cuda-toolkit-13-3 / mold / gh` (pure cleanup; meta
   already shadows them on PATH).
5. **An owner approval verdict** — a queued `[!!]` decision the steward surfaces; the owner
   decides, the loop records the witnessed verdict.

---

## 10. The one-paragraph version

A declarative manifest says what the box should be; a convergence engine makes reality match; an
agent loop picks the next undone thing, plans it, builds it in an isolated worktree, verifies it
by actually running it, ships it as one merged PR, remembers what it learned, hands off cleanly
when its budget runs out, and improves its own harness afterward — all inside a fence of
machine-checked invariants it cannot move, and stopping cold at the five points where only a human
should act. That is the agentic story: **the box maintains itself, and asks for a human only when
the next step is irreversible, live, secret, or a judgment call.**
