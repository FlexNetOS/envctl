# risk_policy — rusty-idd convergence plan (cycle 1)

Companion to `reports/rusty-idd-plan.md`. Classifies every gated upgrade by execution risk-tier and
records the trust-boundary / secrets / destructive / provider/model risk rows. Built only from
CONFIRMED/QUALIFIED + feasibility-passed verdicts (`findings/verdicts.md`). rusty-idd is read-only this
run: APPLY here means "owner may apply with the stated fail-closed gate"; it does **not** mean applied.

risk_policy version: 1 · target: rusty-idd · SHA `5a55284` · author: plan-architect · 2026-06-26.

## Tier definitions

- **APPLY** — contained blast (graph blast 0 or gate-only), reversible by a single commit, no trust-boundary
  crossing, no new dependency, no destructive op. Owner may land behind the row's RED gate.
- **PROPOSE** — structural or owner-walled (deletes tracked trees, changes public API across a high-blast
  surface, or adds a governance surface `validation.rs` tracks). Needs owner review; reversible but not
  trivially.
- **SUPERVISED** — crosses an invariant (the no-C-in-trust-path boundary), introduces the first live
  network/IPC dependency, or alters the security/continuity contract. Owner-gated **and** must satisfy a
  QUALIFIED condition before it may proceed; a human is required at the boundary (Upgrade-Only / no
  silent downgrade; AGENTS.md:42).

## Classification of the gated upgrades

| upgrade | axis | tier | trust-boundary crossed? | destructive? | new dep? | gate (P8 acceptance test) |
|---|---|---|---|---|---|---|
| U9 — config orphan + member-guard | governance | **APPLY** | no | no | no | RED member-guard test (fails today on `crates/config/`) |
| U10 — wire/mark 30 dead `spec` symbols | accuracy | **APPLY** | no | no | no | no *undocumented* dead public symbol in spec |
| U8 — serde_yaml → serde_norway (vendored) | governance | **PROPOSE** | no | no | swap (pure-Rust, no-C) | `cargo tree -i serde_yaml` empty |
| U6/DC-1 — fail-closed card consumer + consume work-order | accuracy / dist-compute | **PROPOSE** | no (pure-Rust serde path) | no | no | 3 RED tests GREEN; baseline stays GREEN; work-order dead → ~0 |
| U1 — decompose runner.rs | quality | **PROPOSE** | no | no (mechanical move) | no | runner public-API diff = ∅; tests green |
| U2 — split tui app.rs | quality | **PROPOSE** | no | no | no | tui public-API diff = ∅; extracted-module unit test |
| U3 — split knowledge lib.rs; catalog → data | quality | **PROPOSE** | no | no | no | catalog round-trip == prior set; public-API diff = ∅ |
| FL-3 — `no src/*.rs > 1500 LOC` gate | filesystem-layout | **PROPOSE** | no | no | no | gate RED today on knowledge/tui/runner |
| U4 — feature-gate 182 dead vendored codegraph | speed | **PROPOSE** | no | no (gates existing) | no | slim build green; `code dead` drops ≥100 + measured before/after |
| U5 — de-dup vendored upstreams (handoff 3×) | governance | **PROPOSE** | no | **yes — deletes tracked trees** | no | one tracked path per upstream; product build unaffected |
| **U7 — typed convergence/adapter boundary** | governance | **SUPERVISED** | **yes (invariant: NO C in trust path)** | no | new `crates/interop` trait + filesystem adapter | trait + filesystem adapter + handoff.task.v1 round-trip test; **condition: weave required local route, every adapter C-free** |
| **DC-2 — bind work-orders to weave/A2A transport** | distributed-compute | **SUPERVISED** | **yes (first live network/IPC dep)** | no | weave/A2A transport (pure-Rust tonic) | weave job keyed by correlation_id; stub executor ACKs; **condition: behind a transport feature flag, filesystem `.handoff/` remains the offline fallback** |
| DC-5 — guardrail: no mlua/esp-hal/no_std | dist-compute (guardrail) | **PROPOSE** (ADR-candidate) | protects the boundary | no | **forbids** new deps | CI grep gate: no embedded/Lua-runtime crate enters Cargo.toml |

## Risk rows

### trust-boundary (NO C in the trust path)
- **Invariant (CONFIRMED dc no-C):** the only third-party native surface is blake3 (pure-Rust intrinsics)
  + serde/serde_json/schemars; no FFI/C, no `mlua`/`rusqlite`/`openssl-sys`/`-sys`/`cc`/`bindgen` in
  `crates/*` (excl. external). This invariant is the single most important constraint on the convergence
  upgrades.
- **U7 (SUPERVISED):** the adapter boundary is buildable C-free — weave is redb/pure-Rust; A2A/gRPC via
  tonic is pure-Rust. The QUALIFIED gate: any future adapter must remain C-free in the trust path (no C
  TLS, no C-linked native vector lib). The filesystem adapter (first impl) is the required offline route.
- **DC-2 (SUPERVISED):** weave is pure-Rust, so the transport stays inside the no-C boundary — but it is
  the first live network/IPC dependency in an offline-by-construction binary; gated behind a feature flag,
  filesystem `.handoff/` contract retained as the degraded path.
- **DC-5 (guardrail):** explicitly forbids `mlua` (links the C Lua lib), `esp-hal`, `no_std` from
  rusty-idd's `Cargo.toml` — firmware + Lua/Luau runtime belong to fleet-executor repos. Recording it
  protects the boundary at zero dep cost.

### secrets
- rusty-idd handles **no secrets** in product code (CONFIRMED distributed-compute audit §4: "secrets — N/A,
  handled by envctl in the fleet"). No secret surface is added by any gated upgrade. DC-2's transport must
  route any future credential through envctl, never inline — a SUPERVISED precondition, not a secrets
  capability added here.

### destructive
- **U5 is the only destructive-class upgrade** — it deletes tracked vendored trees (handoff 3×). It is
  reversible via git history but owner-walled; per `.claude/rules/meta-destructive-commands.md` it requires
  explicit owner request + worktree inspection before any deletion. The Claude agent-guard `deny[]` for
  `git reset --hard`/`git clean -fd`/`rm -rf` is currently **decorative** (`mode="warn"`, never parsed —
  CONFIRMED gov-002), so U5 must not rely on the guard for protection; the owner-wall is the control.
- gov-007 / FL-6 (`*.idd-bak-*` pruning, `git rm --cached`) touch only gitignored/regenerable litter —
  destructive-class but trivially reversible; not gated this cycle.

### provider/model
- No provider/model is invoked from product code (CONFIRMED distributed-compute §3: no Ollama/OpenAI/
  Anthropic/Cloudflare/HF client; provider strings are inert config in vendored codegraph-core). The model
  lanes (Codex `gpt-5.5*` vs Claude `opus`) are declared in scattered TOML + test fixtures with **no
  governing decision record** (CONFIRMED prompt-architecture M6). Risk: a silent model downgrade would
  violate Upgrade-Only (AGENTS.md:42) yet nothing pins the lane→model mapping. Mitigation is a model-lane
  policy-of-record ADR-candidate (prompt-architecture C2) — recorded, not promoted as an ADR this cycle.
  DC-2's `allows_network` egress policy (`work-order/src/lib.rs:64-65`) is the provider-egress control a
  future executor binding must enforce fail-closed.

## Laws honored
- Upgrade-Only / No-Downgrades: every row is additive; none weakens a guard, rule, gate, or permission
  (U7/U10/gov-class candidates STRENGTHEN). 
- Owner-wall: rusty-idd code untouched; all rows are owner-applies-with-gate.
- Fail-closed: every tier carries a falsifiable RED gate that fails on drift.
