# Agent interop — fleet-convergence planning loop

How agents/organs talk across the fabric.

- **weave** — the A2A transport (nervous system): leases (`plan:claim:<target>`), inbox/reply/notify
  point-to-point, sessions/peers. This cycle: held `plan:claim:icm`; cycle-7 close notifies envctl.
- **mcp** — Model Context Protocol: icm ships `icm-mcp` (31 tools) injected into ~15 agent hosts; the
  fabric's tool-grant surface. icm's MCP/CLI seam is the SIDECAR contact point for the no-C kernel.
- **ACP** — Agent Control Protocol: reserved for backend agent control; not exercised by icm (local
  single-process). Candidate for icm-over-weave shared-memory bus (proposed upgrade U3).
- **A2A** — agent-to-agent: icm is currently NOT A2A-capable (local only; only RTK cloud-sync egress).
  Convergence recommendation: expose icm over weave for observable multi-agent shared memory.
- **GitHub cloud agent** — cloud code agents (ultrareview etc.): owner-triggered/billed; the loop
  ships PRs they can review, but does not spawn them.

Convergence note: icm binds into the union TODAY only by convention + a connected MCP server (ad-hoc).
The planned **bind-as-data** contract (a typed `memory` pointer in `handoff.context_capsule.v1`)
turns this MCP/CLI convention into referenced DATA.
