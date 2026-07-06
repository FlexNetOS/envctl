# distributed-compute — weave (cycle 4)

| field | value |
|---|---|
| target | **weave** — the A2A transport plane that CARRIES distributed work across agents/machines |
| code (read-only) | /home/drdave/Desktop/meta/.worktrees/plan-weave-red/weave |
| axis | distributed-compute |
| snapshot | codemap `@4fe2419` (cycle 4); manifests/source read 2026-06-26 |
| verdict | weave is the convergence-relevant **distribution substrate** (routes work-orders/messages across nodes) but its cross-machine reach is **host-class only** — a network seam (HTTP push / remote pull) that constrained nodes (Pi Zero / ESP32) cannot host. No Lua, no embedded, no mobile/wearable client today. |

Frame: weave = cross-agent/cross-MACHINE transport. Tier-2 = signed cross-store delivery + HTTP push
(WL-056/ADR-0005: `weave push --host <endpoint>` POSTs an `Intent` to a remote `weave serve`). So
weave IS the substrate that would route work onto distributed nodes — assessed below against the owner
hardware matrix.

---

## 1. Hardware target matrix (what weave can actually reach)

weave's distribution is **two network seams + one local seam**, all assuming a full host that can run
the `weave` binary and a writable SQLite/libsql store:

| target class | weave reach today | grounding |
|---|---|---|
| **Workstation / GPU host (local)** | FULL — runs the binary, the SQLite-mailbox broker, daemon/tick, inject, serve | `weave/src/main.rs:4489` dispatches 71 verbs; default build `default = ["sqlite"]` (`weave/Cargo.toml:18`) |
| **Local servers (same LAN / Tailscale)** | FULL — receive cross-machine push via `weave serve` (HTTP `POST /api`), or be a Tier-2 remote pull source via `libsql`/Turso | `weave-mcp/src/http.rs:51` `serve_http`; ARCHITECTURE.md:1840 (ADR-0005 push), :1563 (remote pull) |
| **mobile** (phones/tablets) | N/A — no mobile client, no Android/iOS target, no FFI/UniFFI surface; a phone could only reach weave as an external HTTP POSTer to `weave serve`, not run weave | docs grep for `mobile/phone/android/ios` over ARCHITECTURE.md+README.md returns **zero** matches |
| **AI glasses / wearables** | N/A — no wearable runtime, no BLE/companion transport; same as mobile (could only be an external HTTP client of `weave serve`) | grep `glasses/wearable` over docs = zero matches |
| **Raspberry Pi / Pi Zero class Linux** | PARTIAL/host-only — a Pi running 64-bit Linux *could* compile and run the `weave` binary (pure Rust, `edition 2021`, no platform `[target.'cfg(...)']` gates in any manifest), so it is a peer/relay node; **Pi Zero (armv6, 512 MB)** is impractical for the SQLite/libsql broker + tokio (libsql) footprint. No cross-compile target, no `aarch64`/`armv7` CI evidence | no `[target.*]` keys in any Cargo.toml; `aarch64/armv7/cross-compile` grep over `*.toml` = zero |
| **ESP32 / ESP32-S3 class MCU** | N/A — weave is `std`-only; **no `no_std`, no `esp-hal`, no embedded** anywhere in the tree. An ESP32 cannot host weave; it could at most be an HTTP client POSTing an `Intent` to a remote `weave serve` (if it implements the JSON-RPC `weave_push` envelope itself) | grep `no_std/esp32/esp-hal/embedded` (code) returns only unrelated "embedded NUL/embedded text" string matches — no embedded runtime |
| **Offline / degraded** | STRONG (local) — the SQLite mailbox is a durable embedded log; the broker, daemon, inject all work with **zero network**. Cross-machine push/pull degrade closed (fail if endpoint unreachable) but never block local delivery | codemap §"What weave is" (SQLite mailbox broker, not a network daemon); weave.trends.md §B3 (durable embedded log) |

**Finding (host-side only):** weave can carry work *between full hosts* (workstation ↔ local server ↔
Pi-class Linux), but it **cannot push work ONTO constrained nodes** (Pi Zero / ESP32 / mobile /
wearables) — those nodes can only participate as **external HTTP clients** that POST a `weave_push`
JSON-RPC `Intent` into a host-resident `weave serve`. The "minimal client on a constrained node" model
described in the frame is **not implemented**; it is a spec/adapter gap, not an existing capability.

---

## 2. Language / runtime map

| runtime | status in weave | grounding |
|---|---|---|
| **Rust `std`** | the entire substrate — 4 crates, `edition 2021`, `resolver = "2"`, pure-Rust, no C trust-boundary escape; all internal deps are in-repo `path = "../<crate>"` | codemap §"Crate roles"; `Cargo.toml` manifests |
| **Rust `no_std` / embedded** | N/A — absent. No `no_std`, no `esp-hal`, no `#![no_main]`; weave cannot target ESP32-class MCUs | grep over `*.rs`/`*.toml` = zero embedded markers |
| **Lua / Luau (mlua / Lune / Xedge)** | N/A — **none present**. No `mlua`, `lune`, `rlua`, or any Lua/Luau scripting plane. weave has no policy/script runtime; its "policy" is compiled Rust (`weave-core/src/webpolicy.rs`, `sign`, `peerpolicy`). (The grep hits for "lua" were false positives inside the word *evaluate*.) | grep `lua\|luau\|mlua\|lune` over `*.rs`/`*.toml` = zero real matches |
| **WASM sandboxing** | N/A — no `wasm`, no `wasmtime`/`wasmer`; injected work is native pane keystrokes, not a sandboxed module | grep `wasm` = zero |
| **no-C / no-downgrade trust boundary** | UPHELD — default `sqlite` route is `rusqlite` (bundled C is the one sanctioned exception inside the SQLite engine itself, not weave's code); optional `libsql` adds tokio + a rustls TLS stack that owns the entire advisory budget (5 RUSTSEC ids, all behind `--features libsql`). Default build is advisory-clean. | codemap §"Dependency hygiene"; weave.trends.md §D advisory cluster |

**Finding (Lua absent by design):** the Rust+Lua north star (`plan-architecture-loop-distributed-compute-2026-06.md`
decision 4) has **no Lua leg in weave**. weave is a single-language (Rust) substrate. If a portable
policy/script plane is ever wanted on constrained relay nodes, it would be a *new* `mlua`/`Lune`
addition — currently it does not exist and is not stubbed.

---

## 3. Vendor mesh (local / cloud responsibility + failover)

weave is **protocol-agnostic transport**, not a model router — but it touches vendor/cloud surfaces:

| surface | local/cloud | role in weave | grounding |
|---|---|---|---|
| **SQLite mailbox (local broker)** | local | the required, always-on transport; survives offline; the event log | codemap §"What weave is" |
| **libsql / Turso (remote pull)** | cloud/remote | optional Tier-2 v2 cross-machine **read-only** source (`--features libsql`); async tokio client | ARCHITECTURE.md:1563, :469 |
| **HTTP push → `weave serve`** | local-or-cloud | the A-initiated cross-machine delivery seam; gated behind the **`surfaces`** feature (NOT default) | `weave/src/main.rs:1886` `#[cfg(feature = "surfaces")] fn push_to_remote`; `weave/Cargo.toml:27` |
| **LLM provider client** | cloud | optional `llm` feature pulls a blocking+rustls `reqwest` client (`weave-core/src/llm`); a vendor model call surface, not a routing fabric | `weave-core/Cargo.toml:13` `llm = ["dep:reqwest"]` |
| **Telegram / Slack bots** | cloud | optional `surfaces` egress bridges (`weave telegram`, `weave slack`) — carry weave traffic to external vendor chat | codemap §crate roles (`telegram`, `slack` modules); `weave/Cargo.toml:27` |
| **Anthropic/Claude via weave** | cloud (host wiring) | `weave setup` registers weave's MCP server + lifecycle hooks into a coding-agent host (claude default) — weave is the A2A bus the agents ride, not the model itself | codemap §"Entry points" (`weave setup` → `setup.rs`) |
| **Ollama / local models** | local | N/A — no Ollama/local-model integration in weave; model choice is the host agent's concern, not weave's | no `ollama` match in tree |
| **Cloudflare Workers / Workers AI** | cloud | N/A — no Cloudflare/Workers binding; weave's cloud reach is HTTP push + libsql only | no match |
| **Hugging Face / GitHub Copilot agent** | cloud | N/A — not integrated | no match |

**Cross-vendor trust primitive:** the optional **`sign`** feature (`ed25519-dalek 2` + `sha2 0.10`,
`weave-core/Cargo.toml`, default-OFF) signs the canonical `(from,to,body)` of each `Intent`/push so
`from` is unforgeable across stores/machines. This is weave's local analogue of A2A v1.0's **signed
AgentCard** (weave.trends.md §A2) and is the natural cross-**vendor** trust anchor if weave converges
toward A2A. `push_to_remote` reuses the SAME `sign_intent_if_keyed` signer (`weave/src/main.rs:1934-1935`).

**Failover:** local SQLite is primary and never depends on a vendor; cloud/remote seams (push, libsql
pull, llm, bots) all **degrade closed** — failure surfaces as a CLI/feature error and local delivery
is unaffected. There is **no automatic multi-vendor failover/model-routing** in weave (that is not its
job; it is the transport, not the router).

---

## 4. Control / data plane

| concern | weave mechanism | grounding |
|---|---|---|
| **discovery** | `register/register_peer_full`, `peers`, `sessions`, `scan` over the local store (+ Tier-1 read-only federation across local stores) | `weave-core/src/store.rs:141,167`; codemap §Federation |
| **presence / heartbeat daemon** | a real presence plane: `heartbeat(name,host,pid)` writes a presence row (`PRESENCE_TTL_SECS = 30`), `peer_liveness` is a 3-tier resolver (fresh daemon heartbeat → `Live`; else TTL heuristic `ONLINE_TTL_SECS = 900` → `Likely`/`Offline`). `weave daemon`/`weave tick` drive it on an `--interval` | `weave-core/src/store.rs:40-45,643-677`; `weave/src/main.rs:423,440` |
| **cross-machine liveness** | **TTL-only** for remote hosts — the ONLY liveness probe is a same-host `/proc/<pid>` check gated to the local arm; **no cross-machine pid/network/ssh/ping probe exists**; remote presence fails OPEN | ARCHITECTURE.md:1079-1088, :1132; README.md:229-269 |
| **scheduling / OTA** | `schedule`/`schedules`/`cancel-schedule` + cron evaluator (`model.rs:2770`); `tick` evaluates due schedules. **No OTA/firmware-update plane** (consistent with no embedded-node support) | codemap §verb inventory; `weave-core/src/model.rs:2770` |
| **message bus / A2A** | the SQLite mailbox IS the bus: `send/notify/reply/answer/ack`, `ask/ask-many/asks` (correlated request-response), `broadcast-notify/broadcast-ask`, `job`/`orchestrator`, `lease` (resource reservation). Mirrored as 78 `weave_*` MCP tools | codemap §A2A surface |
| **work injection into panes (the "carry work" primitive)** | `weave-inject` writes keystrokes into a LIVE local terminal pane via `Mux::{Tmux,Zellij,Kitty,Wezterm,Screen,ITerm2,None}` | `weave-inject/src/inject.rs:84-99` |
| **secrets / tokens** | push bearer token resolution `--token > $WEAVE_PUSH_TOKEN`; serve/dashboard bearer is CLI/random (no config key) | `weave/src/main.rs:1909-1915`; `http.rs:66` |
| **bandwidth / power** | SQLite-local first (zero network in the common path); cross-machine is opt-in HTTP — appropriate for low-bandwidth, but the broker+tokio footprint is **not** power/RAM-tuned for MCU/wearable nodes | weave.trends.md §B3; manifest footprint |
| **privacy / data-residency** | fail-CLOSED routable bind: `serve_http`/`serve_dashboard` refuse a non-loopback `--bind` without a bearer token, checked BEFORE `TcpListener::bind` (never opens an open network socket); default bind `127.0.0.1`; SSRF avoidance — `push --host` is EXPLICIT-ONLY, never auto-resolved from message content | `weave-mcp/src/http.rs:21-72,121-131`; `weave/src/main.rs:1882-1885` |

**Finding — multi-mux injection does NOT reach remote panes.** `weave-inject` runs a multiplexer found
by **ABSOLUTE path on the LOCAL host** (`inject.rs:1347`); the "remote" tokens in that file refer to a
mux's own remote-control SOCKET (kitty `KITTY_LISTEN_ON`) or an MCP-network-*triggered* spawn, NOT
cross-machine injection. Cross-machine work delivery is push → B's `weave serve` → **B's own process**
verifies/commits into B's inbox and lights **B's own** pane (owner-only-writes). So a workstation
cannot keystroke-inject directly into a pane on another machine; it can only deliver an `Intent` that
B's resident weave acts on. This bounds the "carry work onto nodes" claim to **nodes already running a
weave process**.

**Finding — the cross-machine push surface is feature-gated, not default.** `push_to_remote` is
`#[cfg(feature = "surfaces")]` and `serve` write-receive depends on the optional HTTP/`--write` seam;
the default `default = ["sqlite"]` build is **single-host** (local mailbox + inject only). Distributed
cross-machine compute is an opt-in (`surfaces`/`libsql`) posture, which is the correct fail-closed
default but means "distributed" is not on out of the box.

---

## 5. Upgrade rows

| id | axis | upgrade | evidence | acceptance | risk | reversibility |
|---|---|---|---|---|---|---|
| DC-W1 | distributed-compute | Specify a **constrained-node minimal client contract**: a tiny external HTTP poster that emits the `weave_push` JSON-RPC `Intent` envelope so a Pi Zero / **ESP32** / **mobile** companion can DELIVER to a host `weave serve` (host-side only confirmed — no node runtime). Document it as adapter, not weave-resident. | `http.rs:51` serve seam; `main.rs:1886` push (host-only); no embedded runtime in tree | a documented Intent-over-HTTP client contract + one reference minimal poster; host `weave serve` accepts it bearer-gated | LOW (additive; no change to weave core) | full — doc/adapter only |
| DC-W2 | distributed-compute | Reuse the optional **`sign`** (ed25519) feature as the **cross-vendor** trust primitive for any external/node poster (signed `from`), aligning with A2A v1.0 signed AgentCards. | `weave-core/Cargo.toml` `sign`; `main.rs:1934` reuses `sign_intent_if_keyed`; weave.trends.md §A2 | a node-posted Intent with a valid ed25519 sig is accepted; an unsigned/forged `from` is rejected when keyed | LOW (feature already exists, default-off) | full — toggle the feature |
| DC-W3 | distributed-compute | Decide explicitly whether a **Rust `no_std` / Lua/Luau policy plane** is in scope for relay nodes. Current state: **neither exists**; the Rust+Lua north star has no Lua leg in weave. Record a no-ADR rationale OR an ADR to add `mlua`/`Lune` only if a node-script need is proven. | grep: zero `no_std`/`mlua`/`lune`/`wasm`; decision-4 of distributed-compute research | a written decision (ADR or no-ADR) referencing the absence; no silent scope drift | LOW (decision doc) | full — doc only |
| DC-W4 | distributed-compute | Validate weave on **Raspberry Pi**-class aarch64 Linux as a relay/peer node (no `[target.*]` gates → should compile) and document the floor (Pi Zero/armv6 + libsql tokio footprint = impractical). | no `[target.*]` in manifests; `aarch64/armv7` CI = absent | a CI or manual aarch64 build + a documented "min node = 64-bit Linux host" floor | MED (new build target; libsql footprint on small RAM) | full — CI lane is additive |
| DC-W5 | distributed-compute | Note the **cross-machine liveness gap**: remote presence is TTL-only (fails open), no network probe. For distributed work routing, add an optional remote heartbeat-over-push so a router can tell a remote node is actually `Live`, not just `Likely`. | `store.rs:40-45,662-677`; ARCHITECTURE.md:1079-1088; README.md:229 | a remote node's heartbeat reaches the router; stale remote nodes resolve `Offline` instead of `Likely` | MED (new heartbeat path; must stay fail-closed) | full — opt-in path |
| DC-W6 | distributed-compute | Keep `sqlite` the default; treat `libsql`/`surfaces`/remote-TLS as the opt-in distribution surface that owns the advisory budget (5 RUSTSEC, all behind `libsql`+tls, upstream-blocked on libsql's rustls 0.23). Bump `rusqlite 0.40.0 → 0.40.1`. | weave.trends.md §D; `deny.toml` WL-044b | default build stays advisory-clean; rusqlite patched | LOW | full — version pin |

---

## Confidence

**HIGH** that weave is host-class-only for cross-machine reach (push is `#[cfg(feature="surfaces")]`,
inject is local-absolute-path, no embedded/no_std, no mobile/wearable runtime — all grounded in source
+ manifests + docs). **HIGH** that Lua/Luau and ESP32/Pi-Zero runtimes are absent (grep over code +
manifests). **MEDIUM** on the Pi aarch64 "would compile" claim (inferred from no `[target.*]` gates +
pure-Rust manifests; not yet built on-device — captured as DC-W4). No fabricated versions or paths.
