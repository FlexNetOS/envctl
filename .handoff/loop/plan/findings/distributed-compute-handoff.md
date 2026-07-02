# Distributed-compute audit — handoff (cycle 2, axis: distributed-compute)

**Target:** `handoff` — the continuity kernel (`hf` CLI + `hf-mcp` + the witnessed `.handoff`
ledger), planned as the union with rusty-idd.
**Code (read-only):** `/home/drdave/Desktop/meta/.worktrees/plan-handoff-cycle2/handoff`
@ `f6abf962413bafe164d56fa26b70b0a5fdacb8a2`.

**Verdict.** handoff is a **pure-Rust** (`edition = 2024`, `rust-version = 1.96`,
`unsafe_code = "deny"` — root `Cargo.toml:6,7,34`), **no-daemon** continuity kernel. It has **no
service runtime, no scheduler, no worker pool, no network stack** (zero `reqwest`/`hyper`/`tonic`/
`axum`/`grpc` deps; `tokio` is optional and used only by the `cognitum` gate, `hf/Cargo.toml:60,74`,
`hf/src/cognitum.rs:49-53`). Therefore the *runtime-execution* half of distributed compute (running
work on phones, AI glasses, Pi, ESP32) is **genuine N/A** for handoff-as-software. BUT handoff is the
**convergence-relevant continuity plane**: it is the surface that **distributes + witnesses** work
across nodes. Three crates make it so — **`handoff-fleet`** (git-as-sync multi-node rollup),
**`handoff-route`** (two-ledger residency), and **`handoff-lease`** (the weave-coordinated mesh claim
bridge) — and the `handoff.task.v1` `WorkOrder` (the `work-order` crate, schemars source-of-truth) is
the auditable work unit that a fleet of executors would carry. The distribution transport handoff
actually implements is **Git + CLI-subprocess bridges** (`weave`, `gh`), never an in-process network
daemon.

Every row cites a path/line, the code graph (`reports/codemap-handoff.md`), the union map
(`findings/union-handoff-rusty-idd.md`), or the distributed-compute research note
(`research/plan-architecture-loop-distributed-compute-2026-06.md`).

---

## 1. Hardware target matrix — N/A as a runtime; handoff is the cross-node continuity plane

handoff runs on the **workstation/server host** only (a host CLI binary `hf` +
`hf-mcp`, plus the `rusty-idd-cli` 3rd binary — codemap §6). It does not execute on, cross-compile
to, or contain drivers for any other device class. What it *does* provide is the substrate that lets
work be claimed/witnessed/handed-off **across many such hosts** via Git.

| device class | handoff runtime status | convergence relevance + evidence |
|---|---|---|
| **workstation / GPU** | **The only host it runs on.** Pure-Rust; no GPU/inference code. | Host of the continuity plane. codemap §6; root `Cargo.toml`. |
| **local servers** | N/A as a service — **no daemon** (`handoff-fleet` doc: "Git is the sync transport — **no daemons**", `handoff-fleet/src/lib.rs:12-13`). | **This is the multi-node seam.** Each server is a fleet *member* enumerated from `.meta.yaml` `projects:` (`handoff-fleet::find_meta_root`/`parse_members`, `handoff-fleet/src/lib.rs:36-55`); their `.handoff` git-text + the FLEET ledger join into one board. |
| **phones / tablets (mobile)** | **mobile: N/A** — handoff is a desktop/server host kernel; no Android/iOS target, no mobile build. | Convergence-relevant only as a **mobile** node that, if it ran `hf`, would commit/claim against the same witnessed ledger; or as a work-order *target* a fleet executor dispatches to. No mobile code in the tree. |
| **AI glasses / wearables** | **AI glasses / wearables: N/A** — no AR/HUD/wearables driver, no Lua-AR surface (the rusty-idd `capability:lua-ar-interface` declaration lives in `rusty-idd/crates/knowledge`, **not** in handoff). | Convergence-relevant: a wearable would be a leaf node whose work is witnessed through the same `handoff.task.v1` contract; handoff carries none of the AR runtime. |
| **Raspberry Pi / Pi Zero-class Linux** | **Raspberry Pi / Pi Zero: N/A** — no ARM cross-build config, no Pi code. handoff *could* compile for ARM Linux (it is dependency-light pure Rust) but nothing here targets it. | Convergence-relevant: a **Raspberry Pi** running `hf` would be a first-class fleet member (git-sync, ledger-only claim offline — see `handoff-lease` graceful degrade). Vendor build path for that class is upstream Rust/cross, research note L21. |
| **ESP32 / ESP32-S3-class MCUs** | **ESP32: N/A — handoff requires `std` and is not `no_std`/firmware.** Zero `no_std`, `esp-hal`, `embassy`, `cortex-m`, `riscv` hits in the whole repo (grep). The kernel depends on `redb` (an mmap/file store) and `std::process::Command` — neither exists on an **ESP32** MCU. | Convergence-relevant only as a deep-leaf executor a host node proxies for; firmware would live in a separate `no_std` repo. Vendor path for that class = Espressif `esp-hal`, research note L16-18. |
| **offline / degraded modes** | **First-class.** `handoff-lease` degrades to a **ledger-only** claim when `weave` is absent/old, "so the kernel still works offline (CI, air-gapped)" (`handoff-lease/src/lib.rs:16-19`, `Reserve::Unsupported`). Git is async-by-nature; no live connection required. | This is exactly the property a distributed edge fleet needs: every node operates offline and reconciles via Git when reconnected. |

**Finding 1.** handoff is **not** a multi-device runtime — it is the **multi-node continuity
substrate**. "Distribution" here = N hosts each running `hf`, each owning a per-repo `.handoff`
ledger, reconciled through **Git** with `handoff-fleet` rolling them up and `handoff-lease` providing
cross-node mutual exclusion. The gap for the architect: that substrate today assumes **Git-reachable,
`std`-capable Linux/macOS hosts** — it does not yet model mobile/Pi-Zero/ESP32 leaf nodes that cannot
run `redb`+`git` directly (they would need a proxy node).

---

## 2. Language / runtime map — pure-Rust `std`; Lua/Luau entirely absent

| runtime axis | status in handoff | evidence |
|---|---|---|
| **Rust (std, host)** | The entire kernel is `std` **Rust**, `edition 2024`, `rust-version 1.96`, `resolver 3`, `unsafe_code = "deny"` (one audited FFI exception: `ledger::v2::pid_is_alive`/`OpenProcess`). | root `Cargo.toml:2,6,7,16-18,34`; codemap §1. |
| **Rust no_std / embedded** | **N/A** — no `no_std`, no embedded HAL, no MCU target anywhere (incl. vendored `syntect`). Hard-blocked by `redb` + `std::process::Command` dependence (`handoff-lease`, `handoff-fleet`, `hf-mcp` all spawn subprocesses). | grep `no_std\|esp-hal\|embassy\|cortex-m\|riscv` = **0 hits** repo-wide. |
| **Lua / Luau / mlua / Lune** | **GENUINE N/A — zero presence.** There is **no `mlua`, `lua`, `luau`, or `lune` dependency in any `Cargo.toml`, and no `.lua`/interpreter and no Lua/Luau string content in any `.rs`** in the handoff tree. Unlike rusty-idd (which carries Lua as *knowledge content* in `crates/knowledge`), handoff does not even reference Lua/Luau. | grep `mlua\|\blua\b\|luau\|lune` over `--include=Cargo.toml` and `--include=*.rs` repo-wide = **0 hits**. |
| **WASM sandboxing** | N/A — no `wasm`/`wasmtime`/`wasi` in the tree. | grep: 0 hits. |
| **no-C / no-downgrade trust boundary** | Honored and enforced as policy. Default build is C-free: the only `bundled`-C crate, `rusqlite` (`ledger/Cargo.toml:23`), is **optional and migration-import-only**, never in the default graph. `handoff-fleet` explicitly refuses to add a YAML crate "for the pure-Rust/no-C trust-boundary gate" (`handoff-fleet/src/lib.rs:49-52`). | `ledger/Cargo.toml:21-23,29-31`; `handoff-fleet/src/lib.rs:49-52`. |

**Finding 2 (Lua/Luau headline — the requested finding).** **Lua/Luau has no executable or even
declarative presence in handoff** — it is a clean N/A, stronger than rusty-idd's (where Lua at least
appears as knowledge-map data). If a Lua/Luau **policy/scripting plane** is ever wanted on the
continuity kernel, the Rust-native embedding path is **mlua** (Rust↔Lua/Luau) or **Lune** (Rust-built
async Luau runtime) — research note L19-20 — but mlua's default backend links the **C** Lua lib, so
it would have to use the pure-Rust Luau feature to respect handoff's `no-C` boundary; that is net-new
scope and a **risk**, not a recommendation. handoff's actual "policy plane" today is
**`handoff-policy`** (a pure-Rust leaf engine, codemap §2), not a scripting runtime.

---

## 3. Vendor mesh — no LLM/vendor client in handoff; coordination is via CLI-subprocess bridges

handoff makes **no** in-process LLM or cloud-provider API calls. There is **no Ollama, OpenAI,
Anthropic/Claude, Cloudflare, or Hugging Face HTTP client** anywhere in the kernel. Every external
interaction is a **subprocess bridge** to a sibling CLI, or **Git**.

| vendor / plane | handoff local/cloud role | status + evidence |
|---|---|---|
| **local models / Ollama** | none | No Ollama client; handoff runs no inference. **local**-first by construction (offline ledger). |
| **OpenAI** | none | No client; not referenced in handoff `Cargo.toml`s. |
| **Anthropic / Claude via weave** | **indirect only** — handoff never calls Claude; it shells to the **`weave`** CLI for mesh coordination (`handoff-lease::WeaveCli`, `HF_WEAVE_BIN`, `handoff-lease/src/lib.rs:148-181`). Any A2A/Claude routing is weave's job, not handoff's. | `handoff-lease/src/lib.rs:3-19,148-181`. |
| **Cloudflare Workers / Workers AI** | N/A | Candidate **cloud**/serverless executor plane per research note L13-15; nothing in handoff touches it. |
| **Hugging Face** | N/A | Not referenced. |
| **GitHub / cloud agent** | **the one real cloud touch** — `handoff-gatekeeper` projects `gh pr view --json` / `gh pr diff` via the **`gh`** CLI (`GhPrView`, `handoff-gatekeeper/src/lib.rs:29-32,186`) and an **optional** `handoff_secrets::github_merge_gate` against `api.github.com` (behind the `secrets` feature; default build is GitHub-API-free and "relies on required GitHub check + branch protection", `handoff-gatekeeper/src/lib.rs:204-258`). | `handoff-gatekeeper/Cargo.toml`; `handoff-gatekeeper/src/lib.rs:186,204-258`. |
| **MCP tool/data plane** | **PRESENT.** `hf-mcp` is an MCP server (protocol `2024-11-05`, JSON-RPC over stdin/stdout) exposing `hf` verbs (`hf_status`, `hf_claim`, `hf_ship`, …) as tools to chat/rvAgent clients; each tool shells to `hf` (`hf/src/bin/hf-mcp.rs:3-20`). This is the **T11 universal control seam**. | `hf/src/bin/hf-mcp.rs:1-20`. |
| **RuVector (vector compute / witness)** | **PRESENT as path-deps** — handoff's only "compute vendor" coupling. `hf` pulls `ruvector-verified`, `ruvector-domain-expansion`, and optional `cognitum-gate-tilezero`; `ledger` pulls `rvf-runtime`/`rvf-index`/`rvf-types`/`rvf-crypto` via `../../RuVector/*`, and **`ledger`'s default feature `v2` enables the RVF recall overlay by default** (`ledger/Cargo.toml:16-20,29-31`). | `hf/Cargo.toml:43-59`; `ledger/Cargo.toml:16-35`; codemap §5 (standalone blocker). |

**Finding 3.** The vendor mesh is **genuine N/A as live LLM integration** but **convergence-relevant
as a control fabric**: handoff exposes itself *to* the agent mesh through **MCP** (`hf-mcp`) and
coordinates *with* the mesh through the **`weave` CLI** (lease) and **`gh` CLI** (gatekeeper) —
never via an embedded HTTP/RPC client. The `WorkOrder.allows_network: bool` field
(`work-order/src/lib.rs:80`) is the per-order **egress policy** a downstream **vendor/cloud/local**
executor would honor — handoff mints/witnesses it but performs the egress itself **only** through the
`gh`/`weave` subprocess seams above. The **RuVector path-dep is the real distributed-compute coupling
to plan**: it is a vector/witness compute dependency wired by relative path, and (per codemap §5) the
**standalone-ization blocker** for a union @ `$META_ROOT` — it must be vendored, path-pinned, or
published before handoff is distributable as a standalone node.

---

## 4. Control / data plane — handoff IS the witnessed control plane for distributed work

This is where handoff is **convergence-relevant rather than N/A**: it owns the continuity control
plane that makes distributed work-orders **auditable across nodes**.

| plane concern | handoff status | evidence |
|---|---|---|
| **scheduling / dispatch** | **Partial — one witnessed task per cycle, not a parallel scheduler.** `hf` claims/works/checkpoints/hands-off ONE `handoff.task.v1` per cycle; there is no worker pool or compute dispatcher. | codemap §1,§6. |
| **work-order issuance (the distributable unit)** | **PRESENT (source-of-truth).** `work-order::WorkOrder` is the schemars SoT for `schemas/task.schema.json`; carries `path_scope`, `acceptance_criteria`, `test_commands`, `allows_network`, `correlation_id` (= weave `Job.correlation_id`), and the 5-field blake3 `intent_lock`. | `work-order/src/lib.rs:57-91,108-134`. |
| **discovery (node enumeration)** | **PRESENT.** Fleet members are discovered from `.meta.yaml` `projects:` by walking up to the meta root (`handoff-fleet::find_meta_root`/`parse_members`). | `handoff-fleet/src/lib.rs:36-55`. |
| **residency / routing** | **PRESENT (fail-closed).** `handoff-route` resolves a task to its KERNEL (local repo) vs FLEET (meta-root) ledger by where the card lives, and **fails closed** if neither — never conjures a ledger (anti-contamination). | `handoff-route/src/lib.rs:3-44`. |
| **mutual exclusion / mesh lease** | **PRESENT.** `handoff-lease` (WL-024) turns `hf claim` into a mesh-coordinated claim: reserve an advisory lease via `weave lease reserve`; conflict → refuse; same holder → heartbeat-extend; absent weave → ledger-only fallback. Zero external deps; no shell (explicit argv). | `handoff-lease/src/lib.rs:3-19,35-55,148-181`. |
| **telemetry** | N/A — no metrics/tracing export. | grep. |
| **secrets** | **Seam only (optional).** `handoff-secrets` wraps `envctl`'s `secrets-engine` (path dep) behind the `secrets` feature; default build is envctl-free. | `handoff-secrets/Cargo.toml`; `handoff-gatekeeper/Cargo.toml` features. |
| **model routing** | N/A (see §3 — MCP exposes verbs; routing is the client/weave's job). | `hf/src/bin/hf-mcp.rs`. |
| **OTA / update** | **N/A** — `hf` is a build-from-source binary; no update channel. | codemap §6. |
| **message bus / A2A** | **N/A in-process; Git + weave-subprocess is the transport.** No A2A/bus library; cross-node sync is committed `.handoff/ledger.events.jsonl` text over Git, with weave the optional mesh-lease coordinator. | `handoff-fleet/src/lib.rs:10-26`; `handoff-lease/src/lib.rs`. |
| **bandwidth / power constraints** | N/A — no constraint modeling (host kernel). | — |
| **privacy / data-residency** | **Partial-by-construction.** Per-repo-first residency (ADR-0004/0018): committed truth is the deterministic JSONL text export, the binary `.handoff/ledger.db` is a gitignored local rebuild cache; `WorkOrder.allows_network` gates egress per order; offline-by-default. | `handoff-fleet/src/lib.rs:14-26`; `work-order/src/lib.rs:80`. |

**Finding 4 (the headline for the architect).** handoff already implements the **hard part** of
distributed work — **witnessed, fail-closed, offline-capable coordination across nodes** — using the
cheapest possible transport (**Git** + advisory **weave** leases + a stdin/stdout **MCP** control
seam), with **zero in-process network surface**. What is *missing* for the distributed-compute north
star is the **executor leg**: (a) no model of non-`std` leaf nodes (mobile/Pi-Zero/ESP32 cannot run
`redb`+`git`+`Command`, so they need a proxy-node contract); (b) the `weave` mesh is reached only via
CLI subprocess + degrades to local — there is **no native A2A/transport binding**; (c) the
**RuVector** vector-compute coupling is path-dep-bound, blocking standalone node deployment.

---

## 5. Upgrade rows

Each: `axis: distributed-compute`. Evidence cited; acceptance falsifiable; risk + reversibility
stated. These keep handoff as the **continuity/coordination plane** — none add a compute runtime,
embedded target, or Lua interpreter to the kernel.

| # | upgrade | evidence | acceptance | risk | reversibility |
|---|---|---|---|---|---|
| DC-1 | **Resolve the RuVector standalone blocker** — choose vendor / path-pin / publish for `ruvector-*` + `rvf-*` so a handoff node builds outside `$META_ROOT`. Make the `ledger` `v2`/RVF overlay an *opt-in* default-off feature if it must stay path-dep. | `hf/Cargo.toml:43-59`; `ledger/Cargo.toml:16-35` (`default=["v2"]`); codemap §5 (blocker). | `cargo build -p hf` succeeds from a clone with no sibling `../../RuVector/` (or with a documented vendored copy); no `../../RuVector/*` path dep remains in the default graph. | **Medium** — touches the witness chain (`rvf-crypto`) and recall overlay; must preserve tamper-evidence parity. | Medium — revert = restore the path deps. |
| DC-2 | **Define the leaf-node proxy contract** for non-`std` devices (**mobile** / **Raspberry Pi Zero** / **ESP32**) that cannot run `redb`+`git`: a host node accepts their `handoff.task.v1` results over the existing `hf-mcp`/`gh`/file seam and witnesses them into the ledger on their behalf. | `hf/src/bin/hf-mcp.rs:1-20`; `work-order/src/lib.rs:57-91`; §1 (non-std blockers). | A documented ADR + a stub: an off-host result (JSON `WorkOrder`) submitted via `hf-mcp` is witnessed into the proxy node's ledger with a correct `correlation_id` and intact `intent_lock`. | Low — additive, reuses MCP + work-order; no kernel network code. | High — new tool/verb behind a flag; revert = drop it. |
| DC-3 | **Native weave mesh binding (beyond the CLI subprocess)** — optional feature so `handoff-lease` can talk to a weave/A2A mesh as a peer for lease + work delivery, keeping the ledger-only offline fallback. | `handoff-lease/src/lib.rs:3-19,148-181` (subprocess bridge today); research note L9 (A2A cross-vendor). | With the feature on, a lease reserve/conflict round-trips without spawning `weave`; with it off, behavior is byte-identical to today (offline fallback preserved by test). | **Medium** — first live network/IPC dep in a currently no-daemon kernel; must stay behind a feature and not weaken the no-C/offline guarantees. | Medium — feature-gated; CLI-subprocess path remains the default. |
| DC-4 | **Enforce `allows_network` + `path_scope` as the cross-node egress/residency policy** at the gatekeeper/route seam, so a witnessed order that forbids egress is refused before any `gh`/network subprocess runs. | `work-order/src/lib.rs:72,80`; `handoff-gatekeeper/src/lib.rs:204-258` (gh/secrets gate); `handoff-route/src/lib.rs` (residency). | A `WorkOrder{allows_network:false}` that would trigger a `gh`/`api.github.com` call is rejected by a fail-closed unit test; `path_scope` violations are likewise refused. | Low — policy is data already in the envelope; enforcement is local. | High — pure validation; revert = remove the check. |
| DC-5 | **(Guardrail / ADR-only) No embedded, no Lua runtime, no in-kernel network stack in handoff.** Record that **ESP32**/**Pi-Zero** firmware and any **Lua/Luau** policy runtime belong to *executor* repos, and that handoff stays no-daemon (Git + subprocess transport), per the no-C/no-downgrade boundary. | grep (0 `no_std`/`esp-hal`/`mlua`/`lua`/`luau` hits repo-wide); `handoff-fleet/src/lib.rs:49-52` (no-YAML-crate precedent); root `Cargo.toml:34` (`unsafe_code=deny`). | Decision recorded as an ADR-candidate; no embedded/Lua/HTTP-client crate enters handoff's default `Cargo.toml` graph. | N/A (a guardrail). | N/A. |

---

## Required-marker disposition (genuine N/A vs convergence-relevant)

- **Rust** — the entire handoff kernel is `std` **Rust** (`edition 2024`, `rust-version 1.96`,
  `unsafe_code=deny`; root `Cargo.toml:6,7,34`); **relevant** (it is the safety/control plane).
- **Lua / Luau** — **GENUINE N/A — zero presence.** No `mlua`/`lua`/`luau`/`lune` dep and no Lua
  string/interpreter anywhere in the tree (grep = 0 hits). handoff's policy plane is the pure-Rust
  `handoff-policy` crate, not a scripting runtime. (Embedding path *if ever wanted* = mlua/Lune,
  research note L19-20 — net-new, must avoid C Lua to keep the no-C boundary.)
- **mobile** — **mobile: N/A as a runtime** — no Android/iOS target; handoff requires `std`+`redb`+
  `git`. Convergence-relevant only as a leaf node a host proxies for (DC-2).
- **AI glasses / wearables** — **AI glasses / wearables: N/A** — no AR/HUD/wearables code, and (unlike
  rusty-idd) not even a Lua-AR knowledge declaration. Convergence-relevant only via the proxy-node
  contract (DC-2).
- **Raspberry Pi / Pi Zero** — **Raspberry Pi / Pi Zero: N/A** — no ARM build config; a full Pi
  (with `std`) *could* run `hf` as a fleet member (offline ledger-only claim degrade), but **Pi Zero**
  class as a constrained leaf needs the DC-2 proxy. No Pi code in the tree.
- **ESP32** — **ESP32: N/A — handoff is `std` host software, not `no_std` firmware** (hard-blocked by
  `redb` + `std::process::Command`); zero `esp-hal`/`no_std` hits. Convergence-relevant only as a deep
  leaf a host node proxies for; vendor path = Espressif `esp-hal` (research note L16-18).
- **vendor / cloud / local** — **vendor/cloud: N/A as live LLM integration** (no Ollama/OpenAI/
  Anthropic/Cloudflare/HF client). The one **cloud** touch is GitHub via the `gh` CLI + optional
  `api.github.com` merge-gate (`handoff-gatekeeper/src/lib.rs:186,204-258`). The **local** control
  path is offline-by-construction (Git sync, ledger-only lease fallback). Convergence-relevant via the
  MCP control seam (`hf-mcp`), the `weave`-subprocess mesh lease, the `WorkOrder.allows_network`
  egress policy, and the **RuVector** vector-compute path-dep (the standalone blocker, DC-1).

## Sources
- `reports/codemap-handoff.md` (§1 kernel identity, §2 crate roles, §5 union seams + RuVector
  standalone blocker, §6 entrypoints).
- `findings/union-handoff-rusty-idd.md` (lineage; handoff = kernel-focused fork; live seams).
- `research/plan-architecture-loop-distributed-compute-2026-06.md` (vendor/runtime references: mlua,
  Lune, esp-hal/Espressif, Raspberry Pi, Cloudflare, A2A, MCP).
- Handoff code (read-only @ `f6abf962`):
  - root `Cargo.toml:2,6,7,16-18,34` (pure-Rust, edition/rust-version, unsafe deny, no-C note).
  - `handoff-fleet/src/lib.rs:10-26,36-55,49-52` (Git-as-sync rollup, member discovery, no-YAML/no-C).
  - `handoff-route/src/lib.rs:3-44` (two-ledger residency, fail-closed).
  - `handoff-lease/src/lib.rs:3-19,35-55,148-181` (weave mesh lease bridge, offline degrade, no-shell).
  - `hf/src/bin/hf-mcp.rs:1-20` (MCP `2024-11-05` control seam, subprocess-to-`hf`).
  - `handoff-gatekeeper/Cargo.toml` + `src/lib.rs:29-32,186,204-258` (gh/cloud touch, optional secrets gate).
  - `handoff-secrets/Cargo.toml` (envctl secrets-engine seam, feature-gated).
  - `work-order/src/lib.rs:57-91,108-134` (`handoff.task.v1` envelope: path_scope/allows_network/
    correlation_id/intent_lock — the distributable, witnessed work unit).
  - `hf/Cargo.toml:43-60,74`; `ledger/Cargo.toml:16-35` (RuVector/RVF path-deps, optional tokio,
    optional migration-only rusqlite, `default=["v2"]`).
- Grep evidence (read-only, repo-wide): `mlua|lua|luau|lune` = **0 hits**;
  `no_std|esp-hal|embassy|cortex-m|riscv` = **0 hits**; `reqwest|hyper|tonic|axum|grpc` = **0**
  (the only `reqwest` mention is `handoff-schema/Cargo.toml:11` *dropping* it).
