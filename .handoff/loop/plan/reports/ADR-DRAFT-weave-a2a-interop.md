# ADR-DRAFT — weave: A2A v1.0 interop adapter (additive, default-off)

- Status: **DRAFT — proposed** (owner-wall; not written into weave's `.handoff/decisions/`)
- Date: 2026-06-26 · cycle 4 · target: weave (A2A transport plane)
- Supersedes/relates: extends ADR-0005 (cross-machine push); does NOT change ADR-0004 (Rust-native) or
  the SQLite-mailbox transport. Existing ADRs cover surfaces/transport; none covers A2A-standard interop.
- Traces to: ARCH-09 (CONFIRMED, no A2A surface), U-ARCH-2 (FEASIBLE), research §A1/§A2/§B1/§E1,
  test-strategy RED suite `weave-core/tests/a2a_interop.rs`.

## Context

weave is the fleet's A2A transport plane but speaks its **own `Intent` schema** over a SQLite-mailbox +
HTTP-push substrate. The gate confirmed there is **no A2A v1.0 / gRPC / AgentCard / JSON-RPC-A2A
adapter anywhere** in weave-core or weave-mcp (grep-empty across both crates; the only `jsonrpc`
strings are MCP). Meanwhile A2A (Agent2Agent) v1.0 is the current stable Linux-Foundation cross-vendor
interop standard (GA 2026-04-09; JSON-RPC 2.0 / SSE / gRPC bindings; signed AgentCards; version
negotiation) — the canonical cross-vendor agent-comm wire (Google ADK uses A2A for cross-agent comms).

The field decomposes inter-agent comms into **transport × protocol × topology** (research §B1). weave
owns transport + topology (message-bus + controlled peer-mesh) and is protocol-agnostic today — which
is exactly why A2A can be added as the **protocol** layer without disturbing the transport.

## Decision

Add an **A2A v1.0 interop adapter** as an **additive, default-off** capability attached at the
`Store`/`Intent` seam:

1. New `to_a2a`/`from_a2a` mapping (`Intent ⇄ A2A Message`) in a new `weave-core/src/a2a.rs` (or as
   functions on `model.rs`) plus an A2A surface on weave-mcp, behind a default-off `a2a` feature.
2. A JSON-RPC 2.0 `{jsonrpc,method:"message/send",params.message}` envelope distinct from the existing
   MCP envelope; SSE/polling/webhook task consumption per A2A v1.0; gRPC optional.
3. **Signed AgentCards** built on weave's existing default-off `sign` (ed25519-dalek) feature — the
   local analogue of A2A v1.0's signed Agent Cards (research §A2).
4. Version negotiation so one AgentCard can advertise v0.3 and v1.0.

The adapter **rides the already-present `serde_json`** (no new dependency) and pure-Rust
`ed25519-dalek` (no C enters the trust boundary). The committed RED suite
`weave-core/tests/a2a_interop.rs` (3 cases, tests-ran 3, all RED-on-assertion) is the acceptance
contract; Feature Forge implements until GREEN.

## Constraints (No Downgrades — strict upgrade)

- **Additive only.** Never mutate `Intent`'s existing serde; the native Tier-2 goldens
  (`integration.rs:3541/3646`) and the SQLite-mailbox transport stay intact and remain the required
  local route. A2A is an edge adapter, not a transport swap.
- **No new dep, no C.** serde_json + ed25519 only. If a gRPC binding is ever added it MUST use
  pure-Rust `tonic`/`prost`, never a C protobuf (re-decide at that point).
- **Default-off.** The `a2a` feature is off by default; enabling it must not add standing MCP tokens
  beyond the byte-budget invariant (ADR-0003).

## Consequences

- **Positive:** weave-mediated agents can interoperate with external, non-meta A2A v1.0 agents over the
  industry-standard wire, with cryptographic trust-before-interaction via signed AgentCards — without
  abandoning the mailbox. The `model.rs` blast (1238) is contained because the change is additive.
- **Cost / risk:** new public protocol surface at the highest-blast schema (PROPOSE tier; see
  `risk-policy.md`); must be kept additive or it ripples fleet-wide. The adapter does not subsume
  handoff's witnessed-receipts plane — keep the two planes separate (research §C1).
- **Reversibility:** feature-gate off; integrity-preserving.

## Alternatives considered

- **Replace the `Intent` mailbox with A2A wire** — rejected (violates No Downgrades; the local mailbox
  is the durable, offline-capable required route).
- **Wait for topology-independent agent naming / Internet-of-Agents discovery** — rejected as a basis
  for action now: pre-standard arXiv research (research §C2), a watch item, not adoptable.
- **Do nothing** — rejected: A2A v1.0 is the ratified cross-vendor standard with multi-cloud GA; staying
  on a proprietary schema forecloses interop.
