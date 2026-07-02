# Distributed-Compute Findings — TARGET: prompt-hub

- axis: `distributed-compute`
- target dir: `/home/drdave/Desktop/meta/prompt_hub`
- date: 2026-06-27
- posture (one line): a `Rust` 2024 workspace — core lib `prompt-hub`, CLI `prompthub`, axum HTTP
  server `prompthub-server` — over a `local` libsql store, fanning out to multi-`vendor` AI APIs.
  It serves the server/workstation and CLI tiers today; it is a control/coordination plane for
  distributed inference, NOT itself an on-device inference runtime for the deep edge.

All claims cite repo paths/lines (verified read-only this cycle). Three crate manifests:
`prompt-hub/Cargo.toml`, `prompthub/Cargo.toml`, `prompthub-server/Cargo.toml`; core source under
`prompt-hub/src/`.

---

## 1. Hardware target matrix

| Tier | Verdict | Evidence |
|------|---------|----------|
| Workstation / local server (Linux/x86_64) | **SERVED** | axum server `prompthub-server/src/main.rs` (`#[bin] prompthub-server`, default port 8080, `TcpListener`/`axum::serve`); CLI bin `prompthub/src/main.rs`; libsql `local`-only store (`Cargo.toml:29-32` `Builder::new_local`, replication/remote/sync/tls features dropped). This is the home tier. |
| GPU box (heavy local inference) | **PARTIAL / delegated** | The repo does NOT run model weights itself — `prompt-hub/src/local_llm/mod.rs:5` "No model weights are embedded"; it is an HTTP client to a co-located inference server (Ollama/llamafile/whisper.cpp). Optional SMART search runs ONNX locally via `ort` (`search.rs:313` `ort::session::Session`, `smart-ort` feature) — CPU/GPU embedding, downloads `.onnx` (`search.rs:455-456 download_model`). |
| Phones / tablets (`mobile`) | **DESIGNED, not deployed** | First-class `mobile.rs` module (`prompt-hub/src/mobile.rs:1-5` "Mobile-first prompt management layer … offline-first CRUD with SQLite-on-device storage, bandwidth-aware sync"), `NetworkCondition`/`CellularGeneration`/`SyncStrategy`/`MobileConfig`. Pure-Rust + libsql; gated behind `mobile` feature. No iOS/Android FFI/UniFFI binding or build target exists — it is a portable library layer, not a shipped app. |
| `AI glasses` / `wearables` | **N/A — no target** | No wearable/glasses/companion-protocol code or build target anywhere (`grep -riE 'wearable|glasses' → 0 hits`). The `mobile` offline-sync + bandwidth-aware client (`mobile.rs`) is the nearest reusable substrate a future thin `wearables` companion could lean on, but nothing targets `AI glasses` today. |
| `Raspberry Pi` / `Pi Zero` (ARM Linux) | **PLAUSIBLE, unproven** | Pure-`Rust` + libsql + rustls (no system OpenSSL pinned in runtime crates) is cross-compilable to `aarch64`/`armv7` Linux in principle, and a CLI-only / lib-only build drops the heavy `ort`/`tokenizers`/`ratatui` optionals. But there is zero ARM/`musl`/cross-compile evidence (`grep -riE 'aarch64|armv7|raspberry|musl' → 0 hits`), no CI matrix for it. A `Pi Zero` (single-core ARMv6, 512MB) would struggle with the full feature set; a `Raspberry Pi` 4/5 running the CLI or a thin fetch client is realistic. |
| `ESP32` / `ESP32-S3` class MCUs | **N/A — out of class** | The crate is `std`-only and `tokio`-`full` async (`Cargo.toml:18`), libsql, axum, reqwest — none of which fit a no_std/no-alloc microcontroller. No `#![no_std]`, `esp-idf`, `embassy`, `cortex`, or `riscv` anywhere (`grep → 0`). `ESP32` could only ever be a dumb HTTP sensor/client POSTing to the server, not a host for this code. |
| Offline / degraded modes | **SERVED (design strength)** | `offline.rs` (`prompt-hub/src/offline.rs:1-6` in-memory store mirroring full CRUD with change tracking + replay-on-reconnect), `SyncStatus`, `ConflictEntry`; `mobile.rs` `NetworkCondition::Offline`; `offline` + `fallback` + `circuit-breaker` features. Connectivity loss is a designed-for state, not a crash. |

CLAIM-1: prompt-hub is a 3-crate `Rust` workspace (lib + CLI + axum server) over a `local` libsql
store — server/CLI tiers are first-class; deep-edge tiers are not. Evidence: `Cargo.toml:1-2`
members; `local_llm/mod.rs:5`; `mobile.rs:1`. Confidence: HIGH.

CLAIM-2: It is a coordination/control plane for inference, not an inference runtime — model weights
live in external servers (Ollama/llamafile/whisper.cpp) reached over HTTP. Evidence:
`local_llm/mod.rs:5`, `local_llm/inference.rs:47-53,129` (`reqwest` POST to local endpoint),
`models.rs:875-897` `LocalProviderKind::{Ollama,Llamafile,WhisperCPP}` + `localhost:11434`.
Confidence: HIGH.

---

## 2. Language / runtime map

CLAIM-3 (`Rust`): Entirely `Rust` 2024, `rust-version 1.91.1`, `#![forbid(unsafe_code)]` at the top
of every module read (`lib.rs:1`, `mobile.rs:1`, `offline.rs:1`, `sync.rs:1`, `swarm.rs:1`,
`multi_provider.rs:6`, `load_balancer.rs:1`, `local_llm/*`). Async on `tokio` `full`. `std`-only —
no `no_std`/embedded support (the only "embedded" / "no_std" grep hits are false positives:
`search.rs:337` "embedded models.json", embedding vectors, `accessibility.rs`/`test_chaos.rs`
prose — confirmed by reading each). Confidence: HIGH.

CLAIM-4 (`Lua`/`Luau`): **N/A — no `Lua`/`Luau` (and no mlua/rlua/Lune) anywhere in the repo**
(`grep -riE 'mlua|luau|\blua\b|rlua' over *.rs/*.toml → 0 hits`). The repo's policy/scripting needs
are met natively in `Rust`: `hooks.rs` (lifecycle hooks), `templates.rs` + Handlebars/Tera template
engines (`Cargo.toml:44-45`, compile-time pick), and feature-gated `Rust` modules. A `Lua`/`Luau`
embedded policy plane is therefore **not applicable** to prompt-hub as built — there is no
user-supplied-script surface that would justify embedding an interpreter, and adding one would
introduce a sandbox/trust-boundary cost the project currently avoids. Confidence: HIGH.

CLAIM-5 (WASM): No WASM runtime (`wasmtime`/`wasmer`/`wasm32`) present (`grep → 0`). The `sandbox`
feature (`sandbox.rs`) is a pure-`Rust` logical sandbox, not a WASM/OS isolation boundary.
Confidence: HIGH.

CLAIM-6 (no-C trust boundary): Runtime HTTP uses pure-`Rust` TLS — `reqwest` is pinned
`default-features = false, features = ["rustls-tls", …]` (`Cargo.toml:83`) and libsql's bundled
hyper-rustls/native-TLS chain is deliberately dropped (`Cargo.toml:29-32`). `openssl`/`native-tls`
appear in `Cargo.lock` only transitively via tooling/dev deps (the tokenizers/monostate and
changelog/`git-cliff` chains), NOT via the runtime workspace crates. So the production trust boundary
is no-C-by-intent, with a tooling-only OpenSSL presence to confirm and lock out for true edge builds.
Confidence: MEDIUM-HIGH (manifest pins verified; full transitive proof is a lock-graph audit task).

---

## 3. Vendor mesh (multi-`vendor` fan-out)

CLAIM-7: A real multi-`vendor` routing layer exists. `multi_provider.rs:11-37` defines
`enum Vendor { OpenAi, Anthropic, Google, Custom(String) }`, `ProviderConfig {vendor, endpoint,
priority, max_retries}`, `TrackedProvider` with health/failover (`record_failure`/`accepts_traffic`/
`HealthStatus`). `load_balancer.rs:7-17` adds round-robin / weighted / least-latency strategies over
`ProviderEntry {url, weight, latency_ms, healthy}`. `provider_health.rs` + `fallback.rs` +
`circuit_breaker.rs` complete the failover mesh. Confidence: HIGH.

| Provider | Role | local/cloud | Evidence |
|----------|------|-------------|----------|
| Ollama / llamafile / whisper.cpp | on-box inference + STT | `local` | `models.rs:875-897` `LocalProviderKind`, `local_llm/inference.rs:129` POST; `local_llm` feature |
| OpenAI (and OpenAI-compatible) | chat + voice STT/TTS | `cloud` `vendor` | `models.rs:1210,1480` `https://api.openai.com`; `voice.rs:90-105,152-162` real `reqwest` POST STT/TTS |
| Anthropic / Claude | chat `vendor` | `cloud` | `multi_provider.rs:357-394` `Vendor::Anthropic` + `https://api.anthropic.com` (routing/config; dispatch is the generic provider path) |
| Google | chat `vendor` | `cloud` | `multi_provider.rs:11` `Vendor::Google` |
| Custom / self-hosted | arbitrary endpoint | local or cloud | `Vendor::Custom(String)` + free-form `endpoint` |
| Qdrant | remote vector store for SMART search | local or `cloud` | `qdrant.rs:30,106-114,276-307` `reqwest` to `:6333`; `qdrant` feature |
| Hugging Face | ONNX model artifact fetch | `cloud` (fetch-once) | `search.rs:455-456 download_model`; `hf-hub` dev/opt dep `Cargo.toml:98` |

CLAIM-8 (multi-model evaluation): The "evaluation/selection" axis is metric-driven, not a live
N-way model bake-off. `evolution.rs:98-108` scores prompts by `success_rate*0.4 + usage_score*0.3 +
token_efficiency*0.2 + recency*0.1`; `models.rs:128-184` carries `specialization_score`;
`confidence.rs`/`quality_gate.rs`/`satisfaction.rs` provide per-result scoring; `swarm.rs` provides
the agent-swarm DAG substrate (`DiGraph`). Model *routing* across vendors is `multi_provider.rs` +
`load_balancer.rs`. Confidence: MEDIUM-HIGH (routing + scoring confirmed; "differential
multi-model eval harness" is not a distinct subsystem).

CLAIM-9 (Anthropic/Claude via weave): No `weave`/A2A integration in this repo (`grep weave → 0` in
source). Claude is reached as a generic cloud `vendor` endpoint, not through the meta `weave` bus.
Confidence: HIGH.

---

## 4. Control / data plane

- **Scheduling / routing:** `load_balancer.rs` (round-robin/weighted/least-latency),
  `multi_provider.rs` priority + retry, `circuit_breaker.rs`, `gradual_rollout.rs`. (control plane)
- **Discovery / health:** `provider_health.rs`, `health.rs`, `local_llm` `refresh_health`,
  `TrackedProvider` health transitions. (control plane)
- **Telemetry:** `metrics.rs`, `analytics.rs`, optional Prometheus text-exposition (`otel` feature,
  `Cargo.toml:74-80` — protobuf exposition dropped to stay no-C/no-vuln), `tracing` everywhere.
- **Secrets:** `auth.rs` (argon2, `Cargo.toml:52-61` `password-hash` `getrandom`); provider API keys
  are config/env-driven (`config.rs`, clap `env` feature `Cargo.toml:42`). No vault/KMS integration.
- **Data plane / sync:** `sync.rs` (`SyncManager` + `tokio::broadcast`, `SplitBrainResolution`),
  `offline.rs` (change-tracking + replay), `mobile.rs` (bandwidth-aware, `max_push_size_bytes`,
  metered/`Offline` conditions), `pollination.rs`. The data plane is built for intermittent
  connectivity and conflict resolution — its strongest distributed-systems surface.
- **Bandwidth / power:** `mobile.rs` `estimated_bandwidth_bytes_per_sec`, `SyncStrategy`,
  `CellularGeneration` — explicit constraint modeling (relevant to any future `mobile`/`wearables`
  edge tier). No battery/thermal governor.
- **OTA / update:** none for the edge (no agent-update channel); model artifacts self-fetch
  (`search.rs download_model`) but app/binary OTA is N/A.
- **Privacy / residency:** `privacy.rs`, `voice_anonymize.rs`, `retention.rs`/`auto_purge.rs`,
  `sanitize.rs`, `moderation.rs` — `local`-first storage keeps data on-box by default; cloud
  `vendor` calls are opt-in per provider config.

---

## 5. Upgrade rows

UPGRADE-1 — Prove + pin the no-C edge build (Pi/ARM)
- axis: `distributed-compute`
- evidence: `Cargo.toml:83` rustls-tls pin; `Cargo.toml:29-32` libsql local-only; transitive
  `openssl`/`native-tls` in `Cargo.lock` (tooling-only); zero ARM/`musl`/cross-compile artifacts.
- action: add an `aarch64-unknown-linux-musl` (or gnu) cross-build of a lib/CLI-only feature set
  (no `ort`/`tokenizers`/`tui`); assert via `cargo deny`/lockfile that no C-TLS reaches the runtime
  graph; add a CI smoke target. Lands the `Raspberry Pi` (Pi 4/5) and thin-client tiers concretely.
- acceptance: CI produces an ARM artifact; deny-check confirms rustls-only runtime; binary runs
  `prompthub search` against a remote server on a Pi-class host.
- risk: LOW (additive build target; no runtime code change). reversibility: trivial (drop the target).

UPGRADE-2 — Thin edge fetch/cache client for `mobile` / `wearables`
- axis: `distributed-compute`
- evidence: `mobile.rs` (offline-first, bandwidth-aware) + `offline.rs` (replay) already exist as a
  portable `Rust` library; no FFI binding ships them to a device.
- action: expose `mobile`+`offline` as a UniFFI/`cdylib` surface (or a minimal read-only HTTP client
  crate) so phones — and, downstream, a `wearables`/`AI glasses` companion — can fetch+cache prompts
  and sync deltas. Keep it read-mostly to bound the trust surface.
- acceptance: a sample Android/iOS (or headless ARM) consumer fetches a prompt offline and syncs on
  reconnect using only `mobile`+`offline`.
- risk: MEDIUM (new public binding surface). reversibility: HIGH (feature-gated, removable).

UPGRADE-3 — Differential multi-model evaluation harness over the existing `vendor` mesh
- axis: `distributed-compute`
- evidence: `multi_provider.rs` routing + `evolution.rs:98-108` / `confidence.rs` / `quality_gate.rs`
  scoring exist, but there is no side-by-side N-`vendor` comparison run.
- action: add an eval mode that fans one prompt to multiple `Vendor`s (OpenAI/Anthropic/`local`
  Ollama) and ranks responses by the existing score formulas; record per-vendor latency/cost from
  `cost.rs`/`metrics.rs`.
- acceptance: one command yields a ranked, scored comparison across ≥2 vendors with cost/latency.
- risk: MEDIUM (live API calls, cost). reversibility: HIGH (opt-in subcommand).

UPGRADE-4 — `ESP32` / sensor ingestion boundary (explicit non-goal, documented)
- axis: `distributed-compute`
- evidence: `std`+`tokio`-full+libsql+axum stack is structurally incompatible with no_std MCUs
  (CLAIM-3/CLAIM... §1 row).
- action: do NOT port the host to `ESP32`; instead document the server's HTTP ingest path as the
  supported way for an `ESP32` device to POST data — keep the MCU as a dumb client.
- acceptance: a doc note + an example `POST /…` contract an MCU can target.
- risk: LOW. reversibility: N/A (documentation).

UPGRADE-5 — Decide `Lua`/`Luau` policy plane: confirm out-of-scope (recommended)
- axis: `distributed-compute`
- evidence: zero `Lua`/`Luau`/mlua usage; policy/scripting already met by `hooks.rs` + Handlebars/
  Tera + feature-gated `Rust` modules (CLAIM-4).
- action: record an explicit "no embedded scripting interpreter" decision (keeps the no-C /
  no-untrusted-script trust boundary clean). Revisit only if a user-supplied dynamic-policy surface
  is ever required, in which case `Luau` (sandboxed) over `Lua` would be the safer choice.
- acceptance: a one-line ADR/decision note in the plan.
- risk: NONE. reversibility: full (decision-only).

---

## N/A tiers (explicit)

- `AI glasses` / `wearables`: **N/A — no wearable/glasses code or build target exists** (0 grep
  hits); only the reusable `mobile`/`offline` substrate could seed a future companion.
- `ESP32`: **N/A — `std`+tokio-full+libsql+axum stack cannot run on a no_std/no-alloc MCU**; MCUs
  can only act as HTTP clients to the server.
- `Lua`/`Luau`: **N/A — no scripting-interpreter surface in the repo**; policy needs are met
  natively in `Rust` (hooks + template engines).
- `Pi Zero`: **N/A for the full feature set** (ARMv6/512MB vs heavy `ort`/`tokenizers`); a
  `Raspberry Pi` 4/5 CLI/thin-client build is the realistic ARM target (UPGRADE-1).
