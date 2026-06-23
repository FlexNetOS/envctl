# envctl — The User Story (how / if / when the human is involved)

> Companion to [`DIAGRAMS.md`](DIAGRAMS.md) §16 and [`AGENTIC-STORY.md`](AGENTIC-STORY.md).
> The agentic story is about the box running itself. This is about **the human** — the single
> owner-admin of a no-human-in-the-loop workstation: when they touch it, how they touch it, what
> the box says back, and the exact enter/exit points (desktop app, CLI, sentinels).

The guiding principle (owner doctrine): **this box has one admin and runs unattended, so
involvement is never gated by *permission* — only by *work the loop must not do itself*.** "Needs
root" or "needs a policy change" is not a human wall (sudo is passwordless; envctl declares the
policy). The walls are exactly the irreversible / live-system / secret / judgment points. The user
is rare, high-trust, and mostly observes.

---

## 1. The two modes of the relationship

```
   MODE A — DIRECTOR (most of the time)            MODE B — OPERATOR (rare, deliberate)
   ────────────────────────────────────            ──────────────────────────────────────
   You write INTENT, the loop executes.            You take the wheel for one specific act.
     • a backlog item / a doctrine                   • a reboot
     • "keep building" / "run unattended"            • a /nix live migration
     • answer a direct question                      • a secret reveal / passphrase unlock
     • tune a gate (cycle_budget, RALPH_APPLY)       • an owner-sudo cleanup
   You are a WALL the loop respects,                 • an approval verdict
   not a driver it polls.                           Triggered by a sentinel or your own decision.
```

You spend almost all your time in **Mode A**. You drop into **Mode B** only at the five walls
(§4), and the box tells you when (§3).

---

## 2. Enter points — where you actually touch the box

| Surface | What it is | When you'd use it |
|---------|-----------|-------------------|
| **`envctl-gui`** (desktop app) | Native egui/eframe app over the *same shared Engine* as the CLI — Dashboard, Components, Add Repo, Live Logs, Settings. No web/WebView. | Eyeball state, watch a converge run live, drive a component by clicking. |
| **`envctl <verb>`** (CLI) | The env-manager: `auto-detect, install, doctor, auto-fix, reset, add-repo, graph, registry, lock, env, dashboard, agent, secret, self`. | Inspect (read-only) or converge (mutate gated by `--apply`). |
| **`secretctl …`** (CLI) | The vault/broker client: `status, unlock, lock, secret, run, relay, ca, mint-github, github-app, audit`. | Vault ops; run a tool with creds injected into the child only. |
| **zellij mission-control dashboard** | `envctl dashboard` lays out a pane per repo. **Panes default to a plain shell** — a Claude session starts only when you run `envctl-open-claude` (opt-in, `ENVCTL_DASHBOARD_AUTO_CLAUDE=1`). | Survey the whole meta workspace; deliberately open an agent. |
| **`! <command>`** (in-session) | Runs a shell command in the live session so its output lands in the conversation. | The secure path for an interactive owner action (e.g. `! secretctl unlock`). |
| **the backlog / a chat prompt** | Writing intent. | Mode A — the normal way you "use" the box. |

The desktop app and the CLI **cannot diverge** — both drive the identical `Engine` API
(`DIAGRAMS.md` §9). What you can do in one, you can do in the other.

---

## 3. The communication flow — how the box talks back

You are never left guessing. The box surfaces state through layered channels, loudest-first:

```
   LOUD (interrupts you)                          AMBIENT (you check when you want)
   ─────────────────────                          ──────────────────────────────────
   vox — spoken summary (piper, English)          PR descriptions (what shipped, why)
     after a significant task                     HANDOFF.md — the cold-start packet
   NEEDS-HUMAN / STOP sentinel files              .handoff/loop/ — backlog, loop_state, cycle/*
     (loop refused + stopped — your move)         GUI Live-Logs / dashboard panes
   a direct question in chat                      ICM memory (decisions/prefs/errors)
                                                  weave bus (cross-agent heartbeats, to:all)
```

- **Sentinels are the contract.** `STOP` = human kill switch (halt now). `NEEDS-HUMAN` = the loop
  hit a `[!!]` or an unroutable verdict and stopped for you. `WRAP-UP-OWED` = a maintenance
  boundary is due (handled automatically at next resume). `DONE` = backlog complete.
- **vox** gives you a 1–2 sentence spoken result so you don't have to read to know "it's done" or
  "it's blocked on you."
- **ICM** means you rarely have to re-explain anything — corrections and preferences persist and
  are recalled.

---

## 4. The five human walls (exit points the loop will not cross)

These are the only places the box *requires* you. Each is irreversible, touches a live system,
exposes a secret, or is a judgment call. The loop writes a sentinel and stops — it never
improvises around them.

| Wall | What you do | Why it's yours |
|------|-------------|----------------|
| **Reboot** | Trigger the 595→610 driver bump + reboot when ready. | A kernel-module reload needs a reboot; held to the very end so it never interrupts a build run. |
| **`/nix` live migration** (`[!!]` TASK-0067) | Run the supervised de-nix + yazelix repoint in a window you choose ("i will tell you when"). | It re-provisions your **running** interactive shell; autonomous execution could break your terminal. |
| **Secret reveal / passphrase unlock** | `secretctl secret get --reveal --apply --confirm`; `! secretctl unlock` for the passphrase. | Revealing plaintext and the passphrase factor are owner-only and audited. (USB possession auto-unlocks — no human needed when the Cognitum Seed is plugged in.) |
| **Owner-sudo cleanup** | `sudo apt remove cuda-toolkit-13-3 / mold / gh`. | Pure cleanup of shadowed system packages; meta already wins on PATH, so it's safe and unhurried. |
| **Approval verdict** | Decide a queued `[!!]` decision the steward surfaces. | A judgment call the loop is designed not to self-authorize. |

Note what is **not** a wall here: routine sudo installs (passwordless → `[A*]` automated), policy
knobs (envctl declares them), and "needs root." Those are work, not permission.

---

## 5. A day in the life (three representative threads)

**Thread 1 — "keep building" (pure Mode A).**
You type `/forge-loop resume`. You walk away. The loop picks items, builds them, merges PRs, reaps
worktrees, hands off at its budget, and the successor continues. You hear an occasional vox line.
You touch nothing until a `NEEDS-HUMAN` sentinel or a question appears.

**Thread 2 — a direct question (Mode A, synchronous).**
You ask "why nvidia-open? where does cuda-oxide fit?" The agent verifies on the box, pulls current
facts, answers with sources, records the decision to ICM, and ties it to the relevant Epic-H
cards. No build happens unless you asked for one.

**Thread 3 — finishing the env (Mode A → brief Mode B).**
The autonomous convergence has done everything it can; the backlog now shows only the five walls.
The box tells you (vox + sentinel): "provisioning is done except the reboot, the `/nix` migration,
and three sudo cleanups." You pick a window, say "do the `/nix` close-out with me," and drive that
one supervised step together. Then you reboot for the driver bump. The env is set.

---

## 6. Enter ⇄ exit, at a glance

```
                ┌──────────────────────── THE BOX (self-converging) ───────────────────────┐
   YOU ─intent─▶│  agent loop  ──▶  Engine converge  ──▶  PRs merged  ──▶  state on disk    │
   (Mode A)     │      │                                                      │             │
                │      │ hits a wall (reboot / live / secret / sudo / verdict)│             │
                │      ▼                                                      │             │
                │  writes SENTINEL + vox  ──────────────────────────────────▶│             │
                └──────┼───────────────────────────────────────────────────────────────────┘
                       ▼
   YOU ◀─notified──  NEEDS-HUMAN / STOP / a question / a vox line
   (Mode B)           │
                      ▼  you act at the surface: envctl-gui · envctl/secretctl CLI · ! cmd
                      └─▶ then hand control back: clear the sentinel / `/forge-loop resume`
```

**Enter** the box through the desktop app, the CLI, the dashboard, or a chat prompt. **Exit** the
loop only at a sentinel — and you re-enter just by clearing it or saying "resume." The contract is
symmetric and explicit: the box runs itself, surfaces exactly when it needs you, and never strands
you in an unknown state.
