# Distributed-compute audit — rusty-idd (axis: distributed-compute)

**Verdict.** rusty-idd is a single-binary, **Rust**-only CLI/TUI control plane with **no daemon, no
service routes, no scheduler, no message bus, and no embedded/networking runtime** (codemap
`reports/codemap-rusty-idd.md` lines 12-17, 68-69; `no_std|esp32|esp-hal|embassy|cortex-m|riscv`
grep over `crates/` returns **zero hits**). Therefore almost every *runtime* distributed-compute
execution surface is **genuine N/A** for rusty-idd-as-software. The axis is nonetheless
**convergence-relevant**: rusty-idd is the *why/what* (intent) plane that **mints
`handoff.task.v1` work-orders** (`crates/work-order/src/lib.rs:36-78`) which a downstream fabric
(weave/lane + the hardware fleet) would dispatch onto distributed compute. It also already
**encodes the distributed-compute target matrix as declarative knowledge** — including **Lua**,
**AI glasses**, and a "distributed device compute" fabric layer — inside `crates/knowledge`
(`crates/knowledge/src/lib.rs:3520-3746`). The plan target is to turn that declared intent into
issued, dispatchable work, not to build compute runtimes into this binary.

Every row below is grounded in a file path + line, the code graph
(`reports/codemap-rusty-idd.md`), the source ledger
(`research/sources-rusty-idd.jsonl`), or the reused distributed-compute research note
(`research/plan-architecture-loop-distributed-compute-2026-06.md`).

---

## 1. Hardware target matrix — N/A as runtime, present as declared intent

rusty-idd executes on the **workstation host** only (it is a host CLI/TUI binary `rusty-idd`,
`crates/cli/src/main.rs::main`). It does **not** run on, target, or talk to any other class of
device at runtime. But the hardware matrix the SKILL asks for is **literally enumerated as a
knowledge graph** inside the `knowledge` crate's North-Star/operating-fabric map.

| device class | rusty-idd runtime status | convergence relevance + evidence |
|---|---|---|
| **workstation / GPU** | **The only host it runs on.** Pure-Rust binary; no GPU, no inference code. | Host of the intent plane that mints work-orders. `crates/cli/src/main.rs`, codemap L12. |
| **local servers** | N/A — no daemon/service/route surface (`cross-service-impact` empty). | Declared as `layer:infrastructure-device-fabric` "Network control plus distributed device compute, storage, inference, and memory" — `crates/knowledge/src/lib.rs:3520-3525`; anchor "user devices for distributed compute storage inference memory" (`:3702`, repos `oh_my_pi`, `network_control`, `envctl`). |
| **phones / tablets (mobile)** | **mobile: N/A** — rusty-idd is a desktop host CLI, not a mobile app; no Android/iOS code (`android\|ios` grep over product crates: 0 hits). Convergence-relevant only as a **mobile** work-order *target* the fabric would dispatch to. | Implied under the device-fabric layer + `oh_my_pi`/`lifeos` repos in the knowledge map (`:3700-3746`). |
| **AI glasses / wearables** | **AI glasses: N/A as a runtime** — rusty-idd has no AR/HUD/wearables driver code. Convergence-relevant: it **declares** the AR-glasses capability. | `capability:lua-ar-interface` — "Supports AR-glasses coding and local automation with Rust-native Lua surfaces", anchors `"Lua required for AR glasses workflow"`, `"Brilliant Labs Noa style Rust-native agent UX"`, repos `lifeos`, `oh_my_pi`, `yazelix` — `crates/knowledge/src/lib.rs:3729-3737`; layer purpose `crates/knowledge/src/lib.rs:3531-3535`. |
| **Raspberry Pi / Pi Zero-class Linux** | **Raspberry Pi / Pi Zero: N/A** — no ARM-target build config, no Pi-specific code; the `oh_my_pi` repo named in the map is a *separate fleet member*, not rusty-idd. Convergence-relevant as a Linux work-order target. | `oh_my_pi` appears in device-fabric + Lua/AR + personal-automation capabilities (`crates/knowledge/src/lib.rs:3700,3734,3743`). No Pi code in `crates/`. |
| **ESP32 / ESP32-S3-class MCUs** | **ESP32: N/A — rusty-idd is a host CLI, not firmware; convergence-relevant as a work-order target only.** No `no_std`, no `esp-hal`/`embassy`, no `riscv`/`cortex-m` (grep over `crates/` = 0 hits). | Vendor path for this class (if the fabric ever targets it) is Espressif's Rust `esp-hal` — `research/plan-architecture-loop-distributed-compute-2026-06.md` L16-18. |
| **offline / degraded modes** | Inherently offline-capable: rusty-idd does **no network I/O** in the control path (only `crates/knowledge` does outbound `https`/git-clone URL synthesis for repomix ingest — `crates/knowledge/src/lib.rs:4932-4935`). The OpenSpec lifecycle + work-order minting are pure local filesystem ops. | codemap L68-77 (no weave/icm/grit/hf live deps); `crates/work-order/src/lib.rs` (in-process envelope). |

**Finding 1 (HONEST FRAME):** the hardware matrix exists in rusty-idd **only as
`OperatingLayerDefinition` / `OperatingCapabilityDefinition` data** (`crates/knowledge/src/lib.rs`
≈ line 3500-3747) — i.e. rusty-idd *knows about* and *can render a North-Star map of* distributed
compute across workstation → mobile → AI glasses → Pi → device fabric, but **executes none of
it**. That is exactly the "intent/why-what plane" role; the gap is the dispatch bridge to real
compute.

---

## 2. Language / runtime map — Rust-only host; Lua/Luau is declared, not embedded

| runtime axis | status in rusty-idd | evidence |
|---|---|---|
| **Rust (std, host)** | The entire product is std Rust. Workspace `edition` mix (core 2021, tui/runner 2024), `rust-version = "1.88"`, `resolver = "3"`. | `Cargo.toml` (workspace.package + members); codemap L19-37, L80-85. |
| **Rust no_std / embedded** | **N/A** — no `no_std`, no embedded HAL crates anywhere in `crates/`. | grep `no_std\|esp-hal\|embassy\|cortex-m\|riscv` = 0 hits. |
| **Lua / Luau / mlua / Lune** | **Present only as KNOWLEDGE CONTENT, NOT as a runtime.** There is **no `mlua`/`lune`/`luau` dependency and no `.lua` interpreter** in rusty-idd. "**Lua**" appears as (a) a toolchain anchor for the Yazelix terminal surface (`capability:parser-runtime`, `crates/knowledge/src/lib.rs:3705-3718`), and (b) the AR-glasses automation capability (`capability:lua-ar-interface`, `:3729-3737`, "Rust-native **Lua** surfaces"). | grep `lua\|luau\|mlua\|lune` over `crates/` hits **only** `crates/knowledge/src/lib.rs` string data (lines 3529, 3715, 3730-3736, 4653, 6449-6480 tests). Cargo manifests: **zero** Lua deps. |
| **WASM sandboxing** | N/A — no `wasm`/`wasmtime`/`wasi` in product crates. | grep: 0 hits. |
| **no-C / no-downgrade trust boundary** | Honored by default: the only third-party native surface is `blake3` (work-order hashing) + serde/serde_json/schemars; no FFI/C bindings in the control path. | `crates/work-order/Cargo.toml` deps (serde, serde_json, blake3, schemars). |

**Finding 2 (Lua/Luau headline — this is the requested finding):** Lua/Luau has **no executable
presence** in rusty-idd. The candidate Rust-native embedding path *for the fleet* (not for this
binary) is **mlua** (Rust↔Lua/Luau bindings) or **Lune** (Rust-built async Luau runtime) —
`research/plan-architecture-loop-distributed-compute-2026-06.md` L19-20. For rusty-idd
specifically, introducing a Lua/Luau runtime would be **net-new scope** and must respect the
no-C/no-downgrade boundary (mlua links the C Lua lib unless the pure-Rust Luau path is used) —
flagged as a risk, not a recommendation.

---

## 3. Vendor mesh — N/A in product code; rusty-idd carries vendor *intent*, dispatches none

rusty-idd makes **no LLM/vendor API calls** in product code. There is **no Ollama, OpenAI,
Anthropic/Claude, Cloudflare, or Hugging Face client** in any product crate. The only places a
provider name appears are: (a) inert config knobs in vendored `codegraph-core`
(`evaluation_provider`/`evaluation_model`, e.g. `gpt-5.1`, `crates/external/codegraph-core/src/
config_manager.rs:260-263,818-823` and the commented `crates/config/example.toml:95-96`), and (b)
knowledge-map anchors (`rtk-ai`, etc., `crates/knowledge/src/lib.rs:3721-3727`).

| vendor | rusty-idd local/cloud role | status + evidence |
|---|---|---|
| **local models / Ollama** | none | no Ollama client; `local` inference lives in the fleet's `knowledge-runtime` layer concept (`crates/knowledge/src/lib.rs:3501-3504`), not here. |
| **OpenAI** | none (config string only) | `gpt-5.1` is an inert default in vendored codegraph config (`config_manager.rs:262`, `config/example.toml:96`). |
| **Anthropic / Claude via weave** | **none in product code** — weave/A2A absent. | codemap L68 ("zero weave/a2a references in product code"). |
| **Cloudflare Workers / Workers AI** | N/A | candidate **cloud**/serverless plane per `research/plan-architecture-loop-distributed-compute-2026-06.md` L13-15. |
| **Hugging Face** | N/A | not referenced in product code. |
| **GitHub / Copilot cloud agent** | adjacent only — `commands/codex.rs` scaffolds `.idd`/`openspec` for agent harnesses; no cloud agent call. | codemap L53-56. |

**Finding 3:** the vendor mesh is **genuine N/A as live integration**. The convergence design
point: rusty-idd's `work-order` envelope carries `allows_network: bool`
(`crates/work-order/src/lib.rs:64-65`) — i.e. a minted order can **declare** whether the
downstream **vendor**/**cloud** executor is permitted network egress, but rusty-idd itself never
performs the call. Vendor routing/failover is a fabric (weave) responsibility, not a rusty-idd
one. Standardization targets for that bridge: A2A v1.0 (Linux-Foundation-governed cross-vendor
agent protocol) + MCP for the tool/data layer — `research/sources-rusty-idd.jsonl` claim D1/D2.

---

## 4. Control / data plane — the one real convergence seam: `handoff.task.v1` work-orders

This is where rusty-idd is **convergence-relevant rather than N/A**. It owns the *intent* half of
the control plane and emits a provable hand-off contract; it owns **no** data-plane execution,
scheduling, telemetry, or transport.

| plane concern | rusty-idd status | evidence |
|---|---|---|
| **scheduling / dispatch** | **N/A** — no scheduler, no worker pool, no dispatch loop. `dispatch()` in the CLI is just clap subcommand routing (`crates/cli/src/lib.rs:113`), not compute dispatch. | grep; codemap L14. |
| **work-order issuance (the seam)** | **PRESENT (spike).** `work-order` converts a prompt_hub `SwarmBundle` into one+ provable `WorkOrder`s (`handoff.task.v1`), carrying `correlation_id` (= weave `Job.correlation_id`), `path_scope`, `acceptance_criteria`, `test_commands`, `allows_network`, and a blake3 `intent_lock`. | `crates/work-order/src/lib.rs:1-78`, `intake.rs`. |
| **discovery** | N/A | no discovery/registry code. |
| **telemetry** | N/A — no metrics/tracing export. | grep. |
| **secrets** | N/A — handled by envctl in the fleet, not rusty-idd. | codemap (no secret surface). |
| **model routing** | N/A (see §3). | — |
| **OTA / update** | **N/A** — no update channel; rusty-idd is a build-from-source host binary. | `Cargo.toml`; codemap L83. |
| **message bus / A2A** | **N/A in product code** — weave/A2A absent; coupling to the fabric is purely the **filesystem + JSON schema** of `.handoff/`, `.idd/`, `openspec/`, `_workspace/`. | codemap L60-77. |
| **bandwidth / power constraints** | N/A — desktop host, no constraint modeling. | — |
| **privacy / data-residency** | partial-by-construction: offline control path, `allows_network` opt-in per order (§3). | `crates/work-order/src/lib.rs:64-65`. |

**Finding 4 (the headline gap for the architect):** rusty-idd converges with the fleet's compute
fabric through **one seam** — the `handoff.task.v1` work-order — and that seam is **shaped but not
wired**: `work-order` has **24 dead (unconsumed) symbols** (codemap L30, L106). It mirrors the
on-disk schema by file path (`crates/work-order/src/lib.rs:36`) rather than importing the live
`hf` kernel, and there is **no live weave/A2A binding** to actually deliver the order to a remote
executor (codemap L68-69). Closing the distributed-compute axis = (1) consume `work-order`, (2)
bind the issued order to weave/A2A transport, (3) let `allows_network`/`path_scope` flow through
to a vendor/edge executor.

---

## 5. Upgrade rows

Each: `axis: distributed-compute`. Evidence cited; acceptance falsifiable; risk + reversibility
stated. These are **convergence** upgrades — rusty-idd stays the intent plane; none add a compute
runtime to this binary.

| # | upgrade | evidence | acceptance | risk | reversibility |
|---|---|---|---|---|---|
| DC-1 | **Consume the `work-order` crate** (close the 24 dead symbols) — wire a CLI verb that mints a `handoff.task.v1` `WorkOrder` from a bound OpenSpec goal. | `crates/work-order/src/lib.rs:36-78` (envelope exists); codemap L30,L106 (24 dead/unconsumed). | A `rusty-idd` subcommand emits a schema-valid `handoff.task.v1` JSON whose `intent_lock` blake3 matches the source spec; `work-order` dead-symbol count drops to ~0. | Low — additive; the envelope+intake already exist and are tested (`crates/knowledge` test pattern shows the style). | High — new command behind a flag; revert = drop the command. |
| DC-2 | **Bind issued work-orders to weave/A2A transport** so an order can be *delivered* to a remote executor, carrying `correlation_id` (= weave `Job.correlation_id`). | `crates/work-order/src/lib.rs:67-69` (correlation handle); codemap L68 (weave absent); A2A v1.0 stable, LF-governed — `sources-rusty-idd.jsonl` D1. | An emitted order appears as a weave job keyed by `correlation_id`; a stub remote executor ACKs it. | **Medium** — introduces the first live network/IPC dep into a currently-offline binary; must keep an offline/degraded path. | Medium — gate behind a transport feature flag; filesystem `.handoff/` contract remains the fallback. |
| DC-3 | **Honor `allows_network` + `path_scope` as the egress/residency policy** carried to vendor/cloud/edge executors (privacy & data-residency). | `crates/work-order/src/lib.rs:62-65` (`allows_network`, `path_scope`). | An order with `allows_network=false` is rejected by the (future) executor binding if it would egress; covered by a fail-closed unit test. | Low — policy is data already in the envelope; enforcement lives at the executor. | High — pure validation rule; revert = remove the check. |
| DC-4 | **Render the declared hardware/Lua/AR matrix as a live target registry**, so the North-Star map (workstation→mobile→AI glasses→Pi→device-fabric) drives where orders *can* be dispatched, instead of being inert prose. | `crates/knowledge/src/lib.rs:3500-3746` (layer/capability defs incl. distributed-device-fabric, Lua-AR). | `rusty-idd knowledge` output enumerates the device-fabric capabilities as a machine-readable target list consumable by DC-2. | Low — reads existing knowledge data; no new compute. | High — additive render path. |
| DC-5 | **(Deferred / flag only) Embedded & Lua/Luau evaluation** — do NOT add `mlua`/`esp-hal`/`no_std` to rusty-idd; record that ESP32/Pi-Zero firmware and a Lua/Luau policy runtime belong to *fleet executor* repos (`oh_my_pi`, `lifeos`), not this binary, per the no-C/no-downgrade boundary. | grep (0 embedded/Lua-runtime hits in `crates/`); mlua/Lune/esp-hal vendor paths — `research/plan-architecture-loop-distributed-compute-2026-06.md` L16-20. | Decision recorded as an ADR-candidate; no embedded/Lua-runtime crate enters rusty-idd's `Cargo.toml`. | N/A (a guardrail). | N/A. |

---

## Required-marker disposition (genuine N/A vs convergence-relevant)

- **Rust** — the entire rusty-idd product is std **Rust** (`Cargo.toml`); **relevant** (host control plane).
- **Lua/Luau** — **N/A as a runtime** (no `mlua`/`lune`/`.lua` interpreter; zero Lua deps); **present only as knowledge content** (`crates/knowledge/src/lib.rs:3715,3730-3736`); convergence-relevant via the declared AR-glasses capability and as a fleet-executor (not rusty-idd) runtime choice.
- **mobile** — **mobile: N/A** — rusty-idd is a desktop host CLI, not a mobile app (no Android/iOS code); convergence-relevant only as a work-order *target* the fabric dispatches to.
- **AI glasses / wearables** — **AI glasses: N/A as a runtime** (no AR/HUD driver code); convergence-relevant — declared as `capability:lua-ar-interface` (`crates/knowledge/src/lib.rs:3729-3737`).
- **Raspberry Pi / Pi Zero** — **Raspberry Pi / Pi Zero: N/A** — no ARM target/Pi code in `crates/`; convergence-relevant as a Linux work-order target (`oh_my_pi` fleet member, not rusty-idd).
- **ESP32** — **ESP32: N/A — rusty-idd is a host CLI, not firmware; convergence-relevant as a work-order target only** (no `no_std`/`esp-hal`; vendor path = Espressif `esp-hal`, research note L16-18).
- **vendor / cloud / local** — **vendor/cloud: N/A as live integration** (no Ollama/OpenAI/Anthropic/Cloudflare/HF client); the **local** control path is offline-by-construction; convergence-relevant via `allows_network` egress policy carried in the work-order (`crates/work-order/src/lib.rs:64-65`).

## Sources
- `reports/codemap-rusty-idd.md` (code graph: crates, blast radius, convergence-surface table, work-order dead symbols).
- `research/plan-architecture-loop-distributed-compute-2026-06.md` (vendor/runtime references: mlua, Lune, esp-hal, Raspberry Pi, Cloudflare, A2A, MCP).
- `research/sources-rusty-idd.jsonl` (claim D1 A2A v1.0 / LF governance; D2 MCP).
- `crates/work-order/src/lib.rs`, `crates/work-order/Cargo.toml` (handoff.task.v1 envelope + deps).
- `crates/knowledge/src/lib.rs:3500-3747` (operating layer/capability North-Star map: device fabric, Lua-AR).
- `crates/external/codegraph-core/src/config_manager.rs:260-263,818-823`; `crates/config/example.toml:95-96` (inert provider config).
- `Cargo.toml` (workspace members, edition/rust-version), `crates/cli/src/main.rs`/`lib.rs` (single entrypoint, no dispatch loop).
- Grep evidence (read-only): `no_std|esp32|esp-hal|embassy|cortex-m|riscv` = 0 hits in `crates/`; `lua|luau|mlua|lune` hits only `crates/knowledge` string data; no networking deps beyond `tokio` rt-multi-thread (repomix runtime) in `crates/knowledge/Cargo.toml`.
