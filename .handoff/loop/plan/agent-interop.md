# agent-interop — weave (cycle 4)

Where **weave** sits in the fleet's agent-interop topology, and what the A2A-v1.0 convergence changes.
Built from CONFIRMED/QUALIFIED verdicts + `research/weave.trends.md` (A2A state-of-standard) +
prompt-architecture/rules-policy findings.

---

## North-star split (who interprets, who transports, who witnesses)

```
   harness_hub ──interprets──▶  weave ──transports──▶  handoff
   (Front-Door interpreter:     (A2A transport plane:   (witnessed-receipts plane:
    intent → model language)     moves messages/jobs/     durable signed ledger,
                                 leases/approvals)        proof work occurred)
```

- **weave = transport (this target).** Carries live A2A traffic over a SQLite-mailbox broker + pane
  injector. It does **not** interpret intent and carries **no model-routing logic** (prompt-arch
  CLAIM, transport-not-interpreter CONFIRMED).
- **weave ≠ handoff.** weave carries in-flight coordination traffic; **handoff** records *what
  happened* (tamper-evident receipts). The 2026 reliability field explicitly separates transport from
  durable verifiable state (research §C1) — **do not fuse the two planes**. Bridge them by emitting
  handoff witness records as A2A-compatible signed artifacts.
- **harness_hub = interpreter upstream.** It turns intent into model language; weave then transports
  the resulting messages/jobs between sessions.

---

## Interop surfaces today

| surface | protocol | status | evidence |
|---|---|---|---|
| **weave** mailbox + inject | own `Intent` schema over SQLite + HTTP push | the A2A **substrate** — fleet's live A2A-shaped transport | codemap §What weave is; `model.rs:216` |
| **mcp** (weave-mcp) | MCP / JSON-RPC 2.0 (`tools/call`) | carried — 72 dispatch arms / 76 catalog behind ONE byte-budget-gated `weave` meta-tool | ARCH-06 QUALIFIED; PA-TOOLS; `mcp.rs:434` |
| **A2A** (formal LF v1.0) | JSON-RPC 2.0 / SSE / gRPC + signed AgentCards | **NOT implemented** — convergence target | ARCH-09 CONFIRMED (grep-empty); research §A1 |
| **ACP** | Agent Communication Protocol | **N/A — not implemented, not chosen** | recorded non-choice (research §B1 transport×protocol×topology); A2A is the chosen protocol convergence |
| **GitHub cloud agent** | host coding-agent wiring | weave is the bus the host agent rides; `weave setup` registers weave's MCP server + lifecycle hooks into the host | codemap §Entry points; setup.rs |

A key distinction proven by the gate: the `jsonrpc:"2.0"` strings in weave's tree are all **MCP**
(`tools/call`, `notifications/initialized`), never A2A `message/send` — A2A and MCP are distinct
standards (test-strategy CLAIM, verdicts empirical bench).

---

## The A2A-v1.0 adapter = the interop convergence

`Source: reports/weave-plan.md R1; verdicts.md U-ARCH-2; research §A1/§A2/§E1`

- **Decomposition (research §B1):** inter-agent comms = transport × protocol × topology. weave owns
  **transport + topology** (message-bus + controlled peer-mesh); A2A becomes the swappable **protocol**
  envelope. This is why the adapter is non-destructive: keep the mailbox, add A2A as a strict upgrade.
- **What it adds:** `Intent ⇄ A2A Message` mapping (`to_a2a`/`from_a2a`), a JSON-RPC
  `{jsonrpc,method:"message/send",params.message}` envelope, and a **signed AgentCard** built on
  weave's existing default-off `sign` (ed25519) feature — the local analogue of A2A v1.0's signed
  Agent Cards (research §A2, distributed-compute DC-W2). Version negotiation lets one card advertise
  both v0.3 and v1.0 (research §A1).
- **Why it's the convergence:** it lets weave-mediated agents talk to **external, non-meta A2A agents**
  over the same wire (the canonical cross-vendor pattern — Google ADK uses A2A for cross-agent comms,
  research §B2) without abandoning the SQLite-mailbox transport. Additive, default-off `a2a` feature,
  no new dependency (rides serde_json + ed25519), no C in the trust boundary.
- **Acceptance contract:** the committed RED suite `weave-core/tests/a2a_interop.rs` (3 cases,
  tests-ran 3, all RED-on-assertion) is the GREEN target for Feature Forge.

---

## Cross-vendor model lane (interop at the model layer)

- The autonomous loop delegates the Phase-4 invariant/drift/docs **guardian to MiniMax
  `minimax-m3:cloud`** (non-Anthropic) while workers run on claude/opus — a genuine dual-model /
  cross-vendor split (CLAIM-P3 CONFIRMED; `ralph-weave.sh:18-21`, `weave-guardian.md:16`). weave
  itself carries no model-routing logic — MiniMax writes its verdict into the shared `.handoff/loop/`
  ledger that weave-transported sessions read.
- A non-Anthropic model gating auto-merge is **architecturally ADR-uncovered** → see
  `reports/ADR-DRAFT-weave-cross-vendor-model-lane.md`.

---

## Interop summary

- weave is the fleet's **A2A substrate**; the **A2A-v1.0 adapter is the interop convergence** (R1).
- weave = transport vs handoff = receipts — two planes, never fused.
- harness_hub interprets upstream; MiniMax is the cross-vendor model edge; A2A is the cross-vendor
  protocol edge. ACP is an explicit non-choice. GitHub cloud agents ride weave as the host bus.
