# ADR-DRAFT — Typed convergence/adapter boundary for weave/icm/grit/hf (U7)

> **STATUS: DRAFT (proposed).** Authored by plan-architect in the plan dir; **not** written into
> rusty-idd's `adr/` tree (owner-wall — rusty-idd is read-only this run). The owner assigns the final
> ADR number on acceptance. Composes ADR-0010/ADR-0015 (thin-adapter / harness-control-plane) and the
> repo's Upgrade-Only law (`AGENTS.md:42`).

- **Decision area:** how rusty-idd (the intent/why-what control plane) binds to the fleet fabric organs —
  weave (comms/A2A), icm (memory), grit (merge), the `hf` continuity kernel.
- **Date:** 2026-06-26 · **Target SHA:** `5a55284` · **Axis:** governance / convergence · **Risk-tier:**
  SUPERVISED (see `../risk-policy.md`).

## Context (CONFIRMED evidence)

rusty-idd today has **zero library/IPC dependency** on weave/icm/grit/hf. The only in-code references are
descriptive data, not couplings: a repo catalog of `&str` arrays at `crates/knowledge/src/lib.rs:3585-3725`
and harness-contract text at `crates/cli/src/commands/harness.rs:208-265`
(`grep -rinE 'weave|icm|grit|\bhf\b' crates/*/Cargo.toml` → none — CONFIRMED C11). All fabric coupling is
**filesystem + JSON-schema only**: `.handoff/tasks` read at `crates/cli/src/commands/codex.rs:594`;
`_workspace/{backlog,loop_state,HANDOFF}.md` declared at `crates/merge-tools/src/lib.rs:110`; the
`handoff.task.v1` envelope is `crates/work-order` (CONFIRMED C12). The one seam shaped to the fabric —
`work-order` — is **unconsumed** (24 dead symbols, zero product callers; C7/F5).

The fleet is converging on cross-vendor standards: **A2A is a Linux-Foundation v1.0 standard** (signed
cards, gRPC; `research/rusty-idd.trends.md` D1, in-window 2026), and **weave** is meta's local A2A-shaped
substrate. The no-C trust boundary is intact and load-bearing: the only native surface is blake3 +
serde/serde_json/schemars; no FFI/C in the control path (CONFIRMED dc no-C).

The forces:
- The control plane must be able to **bind live** to the fabric (recall from icm, deliver via weave,
  coordinate merges via grit, witness via hf) to replace the human-dispatched, filesystem-only path.
- It must **not downgrade** the offline-by-construction guarantee (Upgrade-Only; `AGENTS.md:42`) — the
  filesystem `.handoff/` contract is the proven degraded path and must remain.
- It must **not** admit C into the trust path (no C TLS, no C-linked native vector lib, no `mlua`).

## Decision

Introduce a **typed convergence/adapter boundary** — a trait (a new `crates/interop` crate, or a `core`
trait module) that defines the **four fabric ports** (comms / memory / merge / continuity). The
**filesystem adapter is the first and required implementation**; weave, icm, grit, and an A2A adapter are
added behind it as **strict, opt-in upgrades**. The descriptive catalog at
`crates/knowledge/src/lib.rs:3585-3725` becomes a **typed registry** consumed by the boundary rather than
inert prose.

Binding rules (the QUALIFIED conditions, non-negotiable):
1. **weave stays the required local route** — the boundary never makes a remote adapter mandatory.
2. **every adapter is C-free in the trust path** — pure-Rust only (weave is redb/pure-Rust; A2A/gRPC via
   tonic is pure-Rust). No C TLS, no C-linked native vector lib, no `mlua`/`esp-hal`/`no_std` (see the
   DC-5 guardrail ADR-candidate).
3. **the filesystem `.handoff/` contract is retained as the offline/degraded fallback** behind a feature
   flag (this is the precondition the SUPERVISED transport upgrade DC-2 depends on).
4. **no-downgrade** — adding a live binding never removes or weakens the filesystem path.

## Acceptance (falsifiable gate — the P8 test)

- A trait defines the four fabric ports, with **at least the filesystem adapter** implementing it.
- A **contract test asserts the `handoff.task.v1` round-trip** through the boundary (mint → write →
  read → validate), reusing the fail-closed card consumer from FF-rusty-idd-001 (U6/DC-1).
- `cargo tree` shows **no C-linked crate** entering the trust path through any adapter.
- The filesystem adapter is reachable with all remote adapters disabled (offline path proven).

## Consequences

- **Positive:** closes the headline architectural gap (the path *into* the one fabric); lets weave/icm/grit/
  hf be added as strict upgrades; turns the inert fleet catalog into a typed registry; A2A v1.0 becomes an
  addressable interop target without coupling the control plane to any single transport.
- **Negative / risk:** once code depends on the boundary it is **hard to fully back out** (owner-walled,
  Reversibility = low). The first remote adapter (DC-2, weave/A2A) introduces the first live network/IPC
  dependency into an offline binary — SUPERVISED, behind a feature flag, filesystem fallback retained.
- **Sequencing:** the analyst's law applies — decompose the three god-files (U1/U2/U3, runner blast 803 /
  tui 248 / knowledge 105) **before** wiring fabric adapters through them, so the boundary lands on
  decomposed, testable surfaces rather than the current mega-files.

## Why this is ADR-worthy (and the routine upgrades are not)

This is a genuine, durable architecture decision about **where the control-plane↔fabric edge sits** and
under what invariants live binding is permitted. The routine upgrades (U8 serde_yaml, U9 config orphan,
U10 spec reachability, U1-U3 decomposition, U4 feature-gate, U5 de-dup) are mechanical or hygiene and get
ROADMAP rows, not ADRs. The prompt-architecture candidates (C1 root-bridge SoT, C2 model-lane policy, C3
hook execution contract) and the DC-5 no-firmware/no-Lua guardrail are recorded as ADR-*candidates* in the
ROADMAP, to be promoted only when actually decided.

## Alternatives considered

- **Keep filesystem-only (status quo).** Rejected: leaves the control plane unable to recall/deliver/
  coordinate live; the human-dispatch bottleneck (rules-policy CLAIM-10) persists.
- **Hard-wire weave directly into `work-order`.** Rejected: couples the control plane to one transport,
  risks the no-C boundary if a C TLS stack is pulled, and gives no offline fallback — a downgrade.
- **Adopt an external Rust agent framework (Rig/Swiftide/AutoAgents).** Rejected this cycle: pulling an
  external orchestrator into a no-C, self-owned control plane is a large blast-radius decision; the trends
  note (C1) recommends pattern-borrow over dependency-add. Reconsider only with an eval that proves it pays.

## Trace

CONFIRMED C11, C12, C7 (`findings/verdicts.md`); feasibility verdict U7 QUALIFIED (`verdicts.md:39`);
trends D1 A2A v1.0 (`research/rusty-idd.trends.md:156-167`); no-C boundary CONFIRMED dc
(`verdicts.md:54`); architecture UPGRADE U7 (`findings/architecture-rusty-idd.md:59`).
