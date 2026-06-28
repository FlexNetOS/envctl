# Agent interop map — how the loop's agents talk, and the convergence interop boundary

The transports/protocols by which agents in this loop (and the fleet they plan) communicate, with
what is CONFIRMED live this run vs. what is the convergence target.

## Transports used / available this run
| Surface | Role | Status this run |
|---|---|---|
| **Agent tool + SendMessage** (in-runtime) | orchestrator ↔ sub-agent dispatch + resume; structured return values | LIVE — all 13 sub-agents dispatched this way; reconcile via SendMessage to resumed agents |
| **weave** | fleet A2A / background transport (the nervous system); `WEAVE_BIN`→PATH→`$META_ROOT/weave/target/{release,debug}/weave` | RESOLVED (binary present, PATH + both targets) but NOT exercised — foreground is Claude, so Opus lanes ran as direct sub-agents, not weave→Opus |
| **mcp** | tool/data servers (icm, context7, Hugging Face, meta, vox, claude-in-chrome) | available via ToolSearch; trend-researcher used context7/HF MCP |
| **filesystem + JSON-schema contracts** | rusty-idd's ACTUAL fabric attachment (`.handoff/` envelopes, `handoff.task.v1` work-orders, `_workspace/*.md`) | LIVE in rusty-idd — this is the gap finding: rusty-idd interops by FILES, not live IPC |

## Convergence interop boundary (the target)
- **ACP** (Agent Client Protocol) — the editor/agent client boundary; target for IDE/foreground interop.
- **A2A** (Agent-to-Agent) — now a Linux Foundation standard at **v1.0** (v0.3 added gRPC + signed
  security cards; 150+ orgs; see `research/rusty-idd.trends.md`). This is the cross-vendor interop
  target **weave should converge toward** as a strict-upgrade boundary — adopt the A2A interface
  without removing weave's working transport until parity is proven.
- **GitHub cloud agent** — PR-mediated cloud execution lane (e.g. ultrareview); interop is the PR +
  checks contract, not live IPC.
- **MCP** — keep as the tool/data-plane standard; prune MCP rot (dead/duplicate servers) per the
  governance-config findings.

## The headline interop gap (rusty-idd)
rusty-idd attaches to the fabric via **filesystem + JSON schema** only — `weave`, `icm`, `grit`, and
the `hf` kernel have **0 live product references**. Convergence path: keep the file/schema contract
(reversible, integrity-preserving) AND add a live **weave**/A2A binding so the control plane can
dispatch work-orders as messages, not just files — without removing the file contract until the live
path is parity-proven.
