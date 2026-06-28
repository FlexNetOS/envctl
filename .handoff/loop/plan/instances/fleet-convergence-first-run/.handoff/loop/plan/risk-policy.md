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

---

# risk_policy — handoff (cycle 2) · the union with rusty-idd

Companion to `reports/handoff-plan.md` + `reports/union-plan-handoff-rusty-idd.md`. Classifies every
gated union/upgrade step by execution risk-tier and records the trust-boundary / secrets / destructive
/ provider / model risk rows. Built only from CONFIRMED/QUALIFIED + feasibility-passed verdicts
(`findings/verdicts.md`, handoff cycle 2). handoff + rusty-idd are read-only this run: APPLY means
"owner may apply behind the row's fail-closed gate"; it does NOT mean applied. Cycle-1 content above
is preserved unchanged.

risk_policy version: 2 · target: handoff (union) · SHA `f6abf96` (verifier empirical @ `d74ad4b`) ·
author: plan-architect · 2026-06-26.

## Tier definitions (carried from cycle 1; SUPERVISED restated for the union)

- **APPLY** — contained blast (gate-only / doc / additive test), reversible by one commit, no
  trust-boundary crossing, no new dependency, no destructive op.
- **PROPOSE** — structural or owner-walled (changes a high-blast public surface, adds a governance
  surface, adds a hook/permission). Needs owner review; reversible but not trivially.
- **SUPERVISED** — crosses an invariant (NO-C-in-trust-path), introduces the first live network/IPC
  dependency, alters the witness/continuity contract, OR has large blast and is owner-walled. A human
  is required at the boundary (Upgrade-Only / no silent downgrade).

## Classification of the union steps (the 5 of `union-plan-handoff-rusty-idd.md`)

| step | id | union action | axis | tier | trust-boundary? | destructive? | new dep? | gate (P8 acceptance) |
|---|---|---|---|---|---|---|---|---|
| 1 | A-U1 | Resolve RuVector path deps (vendor/publish/git-pin) | governance | **SUPERVISED** | protects it (C-free vendoring) | no (additive vendor) | dep-source swap (pure-Rust) | `cargo build --workspace` green w/ no sibling RuVector (E1, RED today) |
| 2 | A-U4 | MERGE/dedup the stale `crates/*` fork → superset; re-apply HFTASK-0082 | quality | **SUPERVISED** | no | **yes — converges/deletes a tracked fork** | no | one `rusty-idd-*` pkg per name (E3); spec/tui golden parity |
| 3 | A-U3 | rusty-idd deps handoff `work-order`+`validate_card` | accuracy | PROPOSE | no (pure-Rust serde path) | no | crate dep (pure-Rust) | cards pass `validate_card`; schemas byte-identical |
| 4 | ts-U3 + Seam-2 | Design + build the ledger read API | accuracy | PROPOSE | no | no | new read-only crate/module | read-only contract test green; reads never mutate witness chain (gated on A-U1) |
| 5 | gov-U1 + UP-1 | Bridge hooks→PreToolUse; fold rusty-idd CLI under the gates | governance/rules | PROPOSE | no | no | hook wiring | out-of-scope edit DENIED (live), native + rusty-idd identical (exit 1) |

**The MERGE (A-U4) and the RuVector resolution (A-U1) are SUPERVISED — large blast, owner-walled.**
A-U1 touches `Ledger.open` (blast 120) and the witness-crypto deps; A-U4 converges a tracked fork and
resolves a hard pkg-name collision across two workspaces. Both require a human at the boundary.

## Classification of the remaining gated upgrades

| upgrade | axis | tier | trust-boundary? | destructive? | new dep? | gate |
|---|---|---|---|---|---|---|
| ts-U1 — fail-closed loader tests (authored, RED) | accuracy | APPLY | no | no | no | 3 RED flip GREEN via `from_card_json` |
| ts-U4 — golden `task_schema_json` parity | quality | APPLY | no | no | no | mirror reproduces byte-for-byte |
| A-U5 — one-`Ledger`-per-feature test | quality | APPLY (gated A-U1) | no | no | no | single resolvable `ledger::Ledger` |
| A-U6 — manifest-cross-check graph gate | governance | APPLY | no | no | no | SCC/dead-code flagged vs Cargo DAG |
| mem-U3 — witness provenance doc fix | accuracy | APPLY | no | no | no | claim text == implementation (SHAKE-256) |
| gov-U9 — doc-sync 8 guard patterns | governance | APPLY | no | no | no | rule enumerates 8 ids |
| pa-U3/pa-U4 — trim catalog / explicit lane | prompt-arch | APPLY | no | no | no | scoped catalog; lane stated |
| ar-U1/ar-U2/ar-U4 — index gate / per-PR audit / one bot | autoresearch | APPLY | no | no | no | partial index / new advisory / one bot fails-closed |
| fs-U3/fs-U4/fs-U6 — untrack .idea / route catalog / mark schemas | filesystem | APPLY | no | trivial cache rm | no | git ls-files empty / regen byte-identical |
| A-U2 — ledger feature graph default=redb-store | quality | PROPOSE | no | no | no | default tree excludes rvf-runtime (gated A-U1) |
| gov-U1/U2/U4/U5/U6/U7 — hook/guard/toolchain/MCP/permission | governance | PROPOSE | no | no | hooks/config | each tightens; never relaxes |
| UP-1/UP-2/UP-3/UP-4 — fold CLI / enforce policies / steward / lane | rules-policy | PROPOSE | no | no | no | refusal/witness gates |
| mem-U3-sign / mem-U5 / mem-U6 — sign witness / why-memory / facade | memory | PROPOSE | no | no | additive (no new dep for ledger-curated) | sig verify / curated recall / organ-tagged |
| pa-U1/pa-U2 — front door / version-stamp | prompt-arch | PROPOSE | no | no | no | one Front Door / skew assertion |
| DC-2/DC-4/fs-U5/ar-U3 — proxy / egress / orphans / cadence | mixed | PROPOSE | no | no | no | witnessed proxy / refusal / routed root / schedule |
| **mem-U1 — wire query_by_intent w/ REAL embeddings OR delete v2-default** | memory-vector | **SUPERVISED** | **yes (a native embedder must be C-free)** | no | possible embedder | `hf recall` semantic OR ADR delegation; condition: C-free embedder |
| **DC-3 — native weave mesh binding** | distributed | **SUPERVISED** | **yes (first live network/IPC dep)** | no | weave/A2A (pure-Rust) | round-trips w/o spawning weave; offline byte-identical fallback |
| **ar-U5 — delete last C dep (legacy-sqlite)** | autoresearch | **SUPERVISED** | strengthens (removes the only C dep) | **yes — removes a feature+crates** | removes deps | `cargo tree -i rusqlite` empty all feature sets; CONDITION: all fleet legacy ledgers migrated first |
| DC-5 — guardrail: no embedded/Lua/in-kernel network | distributed (guardrail) | PROPOSE (ADR) | protects the boundary | no | **forbids** new deps | CI grep gate: no no_std/mlua/HTTP-client crate enters Cargo.toml |

## Risk rows

### trust-boundary (NO C in the trust path)
- **Invariant (CONFIRMED dc):** the only third-party native surface is pure-Rust sha3 (the actual
  witness — SHAKE-256/SHA3-256) + blake3 (work-order intent_lock) + ed25519-dalek (compiled in but the
  witness path never signs — A-C7) + redb; no FFI/C in the control path. `rusqlite` is the lone C dep,
  feature-gated migration-only (`legacy-sqlite`, never default — dc-4). This invariant gates the union.
- **A-U1 (SUPERVISED):** vendoring/publishing the RuVector crates keeps the boundary C-free (sha3 +
  ed25519-dalek + redb are pure-Rust). It is owner-walled for blast (Ledger.open 120) + witness-crypto
  deps, not for a C crossing.
- **mem-U1 (SUPERVISED):** any native embedder wired for real recall MUST be C-free (no C vector lib);
  the delete/delegate path is unconditionally feasible.
- **DC-3 (SUPERVISED):** weave is pure-Rust, so transport stays inside the boundary — but it is the
  first live network/IPC dep in a no-daemon kernel; feature-gated, byte-identical offline fallback
  preserved (the filesystem `.handoff/` contract).
- **DC-5 (guardrail):** forbids `mlua` (links C Lua), `esp-hal`, `no_std`, in-kernel HTTP — firmware +
  Lua/Luau belong to executor repos. Zero dep cost; protects the boundary.
- **The filesystem `.handoff/` contract is preserved as the offline fallback in every transport/binding
  row** (union-2 cond. e; rp-A2A `Reserve::Unsupported`→`ProceedDegraded`).

### secrets
- handoff handles secrets only through the optional, feature-gated `handoff-secrets` envctl seam
  (default build is envctl-free — dc §4). No union step adds an inline secret surface; any credential
  routes through envctl, never inline (a SUPERVISED precondition for DC-3, not a capability added here).

### destructive
- **A-U4 (the MERGE) is the destructive-class union step** — it converges/deletes a tracked
  `rusty-idd-*` fork (reversible via git history but owner-walled). Per
  `.claude/rules/meta-destructive-commands.md` it requires explicit owner request + worktree inspection
  before any deletion. **ar-U5** removes the `legacy-sqlite` feature + crates (SUPERVISED, conditioned
  on fleet migration). The in-repo agent-guard `deny[]` is NOT wired as project PreToolUse (gov-002),
  so neither may rely on the guard — the owner-wall is the control (gov-U1/U2 close that gap).

### provider/model
- No provider/model is invoked from handoff product code (dc §3: no Ollama/OpenAI/Anthropic/Cloudflare/
  HF client; the one cloud touch is `gh` CLI + optional `api.github.com` merge-gate behind the secrets
  feature). The loop pins uniform opus with NO per-role `model:` lane (rp-org-chart, pa-single-opus-
  lane) and NO guard against a silent downgrade — a No-Downgrades risk. Mitigation: **UP-4 / pa-U4**
  (witnessed dual-model lane: gates/ADR/verifier stay opus; mechanical work routes cheaper; a witnessed
  guard BLOCKS a silent downgrade of a gate-tier action). `WorkOrder.allows_network`
  (`work-order/src/lib.rs:80`) is the per-order egress control a future executor binding must enforce
  fail-closed (DC-4).

## Laws honored (cycle 2)
- Upgrade-Only / No-Downgrades: every row is additive; the MERGE winner is the superset + re-applied
  hardening; none weakens a guard, gate, rule, or permission.
- Owner-wall: handoff + rusty-idd code untouched; all rows are owner-applies-with-gate; the MERGE +
  RuVector resolution are SUPERVISED (human at the boundary).
- Fail-closed: every tier carries a falsifiable RED/acceptance gate that fails on drift; the
  filesystem `.handoff/` contract remains the offline fallback throughout.
