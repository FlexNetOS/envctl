# Agent interop map

How agents and tools talk across the prompt_hub front-door plane. Channels: weave · mcp · ACP · A2A ·
GitHub cloud agent. Grounded in cycle-6 findings (rules-policy §3, prompt-architecture §2, autoresearch).

## prompt-hub

| Channel | Role here | Where it lives | Evidence |
|---|---|---|---|
| **weave** | the fleet A2A nervous system / agent-to-agent transport — a plane distinct from handoff's witnessed receipts; loops USE it (resolved via `WEAVE_BIN`→PATH→`$META_ROOT/weave/...`), not host it. Lease-based duplicate-work prevention (`plan:claim:<target>`, TTL 1800) with degrade-visibly ledger-only fallback. | fleet plane AROUND prompt_hub; NOT in product code (`grep weave` over src = 0) | rules-policy CLAIM-A2A1/A2A2; distributed-compute CLAIM-9; loop_state.md lease |
| **A2A** | the abstract agent-to-agent contract weave implements; the cross-repo intent pipeline rides it (harness_hub→prompt_hub→rusty-idd→handoff→harness-hub). Background Opus lanes communicate over A2A. | conceptual transport; realized by weave | rules-policy §3; loop_state Frame |
| **mcp** | prompt_hub exposes NO MCP server in its OWN source (all mcp hits are vendored); as the STORE it grants tools via CLI + HTTP + library instead. The field trend is the MCP front-door registry/gateway pattern (centralized discovery+guardrails+observability) — prompt_hub is the local realization of that pattern, not an MCP server itself. | tool-grant surface (CLI/HTTP/lib); MCP is user-global, out of repo scope | prompt-architecture §2; trends §E; governance MCP-rot none-in-scope |
| **ACP** | agent-control-plane handoff for cross-session continuation and PR-opening automation (Feature Forge GREEN build, security remediation). The committed HANDOFF is authoritative; weave is heartbeat only. | `.handoff/` continuity kernel (Tier-A) + session-relay | rules-policy §3 (session-relay handoff); governance hooks.toml |
| **GitHub cloud agent** | the AI workflows (`external-ai-apis`, `ai-code-review`, `multi-model-evaluation`, `ai-safety-deployment`, `security_remediation`, `ai-test-doc-generation`) — opt-in cloud agents gated on `ENABLE_AI_WORKFLOWS`; output is currently ephemeral artifacts, not a fed-back loop. Reach Anthropic API + GitHub Models + Devin.ai. | `.github/workflows/*.yml` | autoresearch C-WEB-7/C-CODE-6; prompt-architecture §2 |

### Interop flow (the intent pipeline)

```
 harness_hub ──A2A(weave)──► prompt_hub ──goal-artifact(to build)──► rusty-idd
   interpreter                  STORE                                  lifecycle
                                  │                                       │
                                  ├─ tool grants: CLI · HTTP(111 routes) · lib (NOT mcp server)
                                  ├─ ACP: .handoff/ continuity + session-relay (authoritative)
                                  └─ GitHub cloud agents: AI workflows (opt-in, ephemeral)
```

### Interop gaps (from findings)

- The in-repo swarm-handoff helpers (`swarm.rs:179-228`) produce handoff TEMPLATES but are not wired to
  a live weave/A2A envelope — content generation, not messaging (rules-policy GAP-A2A3, U2).
- No prompt_hub-local background-agent status surface; status is delegated to meta/handoff + weave
  heartbeat (rules-policy U3).
- No ICM/handoff-ledger binding in store code — fleet recall is absent at the data layer
  (memory-vector U4).
