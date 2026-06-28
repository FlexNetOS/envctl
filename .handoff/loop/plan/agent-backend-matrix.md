# agent-backend matrix — weave (cycle 4)

How agent work executes against weave, by backend/isolation lane — and where weave sits in each.
**weave IS the transport that carries these lanes** (the A2A substrate moving messages/jobs/leases/
approvals between sessions); it is not itself a sandbox. Built from CONFIRMED/QUALIFIED verdicts +
the distributed-compute / rules-policy findings.

Legend: `[A]` automated · `[A*]` elevated · `[P]` preview/dry-run · `[H]` human-gated ·
`[!!]` supervised/critical.

---

## Backend lanes

| backend lane | weave's role | reach (CONFIRMED) | isolation | risk gate | evidence |
|---|---|---|---|---|---|
| **read-only-local** | weave runs read-only Tier-1 federation: a process reads peers/sessions/messages aggregated across local stores, **no cross-store writes** | FULL (workstation/GPU host) | local process, no mutation | none needed (read-only) | `store::federated_peers/federated_sessions/federation_status` (`store.rs:23`); codemap §Federation |
| **isolated-worktree** | weave's **lease primitive** is the mutual-exclusion the parallel plan-loop reuses to keep concurrent worktrees from colliding on write-scopes | FULL — each worktree is a full host running the binary | git worktree + `require_disjoint_write_scopes` lease | `reserve_lease` returns Err naming holder on conflict `[A]` | `model::Lease` (`model.rs:1302`), `reserve_lease` (`store.rs:750`), `lease_path_conflicts` (`model.rs:1359`), `rules.toml:13-18` |
| **container** | host-class only — a container that runs the `weave` binary + a writable SQLite/libsql store is a peer/relay node; weave has no container-native packaging of its own | PARTIAL — works if the container is a full std host (pure Rust, no `[target.*]` gates) | OS container | inherits host gates | distributed-compute §1-2 (no embedded/no_std; runs on any 64-bit Linux host) |
| **remote-vm** | Tier-2 cross-machine: sender appends a delivery `Intent` to its `outbox`; the **recipient's own process pulls** it (read-only) and commits via owner-only-write `send`; cross-machine rides HTTP PUSH | FULL for full-host VMs (LAN/Tailscale); **opt-in** — `#[cfg(feature="surfaces")]`, default build is single-host | per-VM process; loopback-only bind by default, non-loopback refused without bearer token | fail-CLOSED bind (checked before `TcpListener::bind`); SSRF-guarded explicit `--host` only `[!!]` | `push_to_remote` (`main.rs:1886`), `serve_http` (`http.rs:51`), bind guard (`http.rs:21-72`); ADR-0005 |
| **cloud-agent** | weave is the A2A bus the host coding-agent rides; `weave setup` registers weave's MCP server + lifecycle hooks into the host (claude default). Optional `libsql`/Turso = cloud remote-pull source; optional Telegram/Slack egress bridges | host-wiring + opt-in cloud edges; **no automatic multi-vendor failover** (transport, not router) | host settings.json (written by setup); cloud edges degrade closed | PreToolUse gate (opt-in, deny-by-default) governs dangerous cloud-agent tool calls `[!!]` | codemap §Entry points (`weave setup`); distributed-compute §3; GOV-001 |
| **constrained node** (Pi Zero / ESP32 / mobile / wearable) | **cannot host weave** (std-only, no `no_std`/embedded/Lua/WASM); can only be an **external HTTP client** POSTing a `weave_push` JSON-RPC `Intent` into a host `weave serve` | N/A as a node runtime — adapter/spec gap, not a capability | external-client only | bearer-gated `serve` accepts the posted Intent; `sign` (ed25519) as the cross-vendor trust primitive | distributed-compute §1 (DC-W1/DC-W2); grep: zero embedded/no_std/mlua |

---

## Protocol lanes carried by weave (ACP / A2A)

weave is the **A2A** substrate. The matrix below maps the agent-comm protocols weave does/doesn't carry.

| protocol | weave status | note |
|---|---|---|
| **A2A** (Agent2Agent, LF v1.0) | **substrate today via its own `Intent` schema**; formal A2A v1.0 is the **convergence target** (additive adapter R1) | weave owns transport+topology; A2A becomes the swappable protocol envelope (research §B1). RED suite `a2a_interop.rs` is the acceptance contract |
| **ACP** (Agent Communication Protocol) | **N/A — not implemented**; no ACP surface in tree | the field decomposes comms into transport × protocol × topology (research §B1); weave carries the transport, and A2A (not ACP) is the chosen protocol convergence — recorded so the ACP lane is an explicit non-choice, not a silent gap |
| **MCP** | **carried** — weave-mcp exposes the fleet's largest tool grant (72 arms / 76 catalog) behind ONE byte-budget-gated `weave` meta-tool | distinct standard from A2A; the only `jsonrpc` strings in tree are MCP, never A2A `message/send` (test-strategy CLAIM) |

---

## Isolation × risk summary

- **read-only-local / isolated-worktree** are the safe, automated lanes `[A]` — read-only federation
  and the disjoint-write-scope lease make concurrent agents safe by construction.
- **remote-vm / cloud-agent** are opt-in and fail-closed `[!!]` — cross-machine surfaces are
  feature-gated (`surfaces`/`libsql`), bind is loopback-only by default, and the PreToolUse gate
  (deny-by-default) governs dangerous tool execution when armed.
- **container** rides host-class isolation; **constrained nodes** are external clients only.
- No lane grants weave a destructive auto-write across machines: every cross-store commit is
  **owner-only-write** by the recipient's own process (ARCH-05/ARCH-08 CONFIRMED).
