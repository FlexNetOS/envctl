# Agent backend matrix — fleet-convergence planning loop

Where each agent class may run, by isolation/trust tier. The planning loop uses **read-only-local**
for analysis and **isolated-worktree** for the additive RED-test lane; heavier backends are available
for execution cycles, not planning.

| backend | isolation | used for | notes |
|---------|-----------|----------|-------|
| **read-only-local** | process, no writes to product code | cartographer, researcher, all axis auditors, analyst, verifier | the default planning lane; cites file:line, runs build/probe read-only |
| **isolated-worktree** | own git worktree + branch | plan-test-strategist (additive RED tests); execution cycles | e.g. `plan-icm-red/icm` on `plan/icm-red-tests`; no product-code edits |
| **container** | OS container | future FF build/run of GREEN implementations | not used this cycle |
| **remote-vm** | remote host | heavy parallel fan-out / long builds | not used this cycle |
| **cloud-agent** | provider cloud | ultrareview / cloud code agents | owner-triggered, billed; not loop-spawned |

Interop transports (weave/mcp/ACP/A2A, GitHub cloud agent) are detailed in `agent-interop.md`.
Cycle-7 lanes used: read-only-local (11 agents) + isolated-worktree (1 test-strategist). Cycle 8
swaps the model lane to **Codex** (foreground) per owner directive.
