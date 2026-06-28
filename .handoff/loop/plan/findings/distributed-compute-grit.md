# Distributed-compute findings — target `grit`

axis: distributed-compute
target repo: /home/drdave/Desktop/meta/grit (grit 0.4.0, edition 2021)
verdict: grit is a single-host **merge-lock coordinator** whose only distribution
seam is its pluggable `LockStore` (local SQLite WAL | S3-compatible | Azure Blob).
It is a control-plane lock substrate for parallel AI coding agents, **not** a
compute scheduler. It realistically serves the workstation / local-server / cloud
tiers and full-Linux SBCs; it is N/A on `mobile`, `AI glasses`/`wearables`, and
`ESP32`. No `Lua`/`Luau` plane exists today (none warranted yet — rationale below).

---

## 1. What grit actually is (grounding)

- **Language/runtime.** Pure `Rust` CLI (`src/main.rs`, `clap`). Heavyweight native
  deps that fix the floor for any host: `rusqlite { bundled }` (vendored C SQLite),
  `tree-sitter 0.25` + 14 C grammars, `tokio { rt-multi-thread }`, `aws-sdk-s3`,
  `azure_storage*`, `futures` (`Cargo.toml`). A transitive `openssl-sys` (pulled by
  the aws/azure SDKs) is called out as the reason ARM64 Linux is unbuildable in CI
  (`.github/workflows/release.yml:41-43`).
- **Hard host requirements.**
  - **git CLI on PATH** — worktree/rebase/merge are shelled out via
    `Command::new("git")` (`src/git/mod.rs:35,52,63,100,221,243`). grit cannot
    coordinate where there is no `git` binary and a writable worktree tree
    (`.grit/worktrees/agent-N/`).
  - **Unix only.** The event room uses `std::os::unix::net` (`src/room/mod.rs:3`);
    CI excludes Windows for exactly this reason — "grit is Unix/macOS-only for now"
    (`.github/workflows/ci.yml:20-21`).
  - **A tokio runtime per cloud store.** `S3LockStore`/`AzureBlobLockStore` each
    construct `tokio::runtime::Runtime::new()` and `block_on` every call
    (`src/db/s3_store.rs:37`, `src/db/azure_store.rs:48`) — a full-OS, threaded,
    outbound-HTTPS profile, not an MCU/no_std profile.
- **The distribution seam.** `trait LockStore` (`src/db/lock_store.rs:28-43`):
  `try_lock / release / all_locks / gc_expired_locks / refresh_ttl`. Locks are
  TTL-leased (`LockEntry.ttl_seconds`, `lock_store.rs:11`), reclaimed on expiry
  (`is_entry_expired`, `s3_store.rs:280`), and renewed via heartbeat
  (`refresh_ttl`, `s3_store.rs:601`; `grit heartbeat`, README:231) — that TTL+GC
  lease model is what makes a remote bucket a safe **cross-machine** lock of truth.
- **Atomicity / vendor mesh.** Cross-host correctness rides on conditional-create:
  `If-None-Match: *` conditional PUT on AWS S3 / Cloudflare R2 and native Azure
  Blob (`s3_store.rs:343-360`, `azure_store.rs:11`); MinIO/no-conditional providers
  fall back to GET-then-PUT with an acknowledged TOCTOU window
  (`s3_store.rs:403-418`). Events: local Unix socket `room.sock`, Azure **Event
  Grid** (free, push), or `grit watch --poll N` for any other distributed backend
  (README:229-234, `azure_store.rs:14`).

---

## 2. Hardware target matrix (CLAIM rows)

Each row asks: can this tier (a) host a grit agent node, and/or (b) hold/serve the
shared merge-lock truth?

| Tier | Verdict | Evidence | Rationale |
|------|---------|----------|-----------|
| **CLAIM** Workstation / GPU box | **Primary, supported** | `Cargo.toml`; `release.yml` ships `x86_64-unknown-linux-gnu`, `x86_64/aarch64-apple-darwin` | This is grit's home: full OS + `git` + writable worktrees + outbound HTTPS. Agents *and* the local SQLite WAL lock store run here. GPU is irrelevant to grit itself (no model inference in-repo) — it coordinates the agents that may use it. |
| **CLAIM** Local server / self-hosted lock host | **Supported** | `set-local` SQLite WAL (README:99); MinIO S3 fallback (`s3_store.rs:403`); Azure/S3 native (README:141) | A Linux server is the natural place to host the shared truth: either a self-hosted **MinIO** bucket (accept the GET-then-PUT TOCTOU caveat) or a colocated Azure/S3 endpoint. Same Unix/git profile as workstation. |
| **CLAIM** `Raspberry Pi` (Pi 4/5, aarch64 Linux) | **Feasible, must build from source** | no ARM64 binary shipped (`release.yml:41-43`); `cargo install --git` (README install) | Full Linux + `git` satisfies the host contract, so a Pi can be a thin **agent node or `grit watch --poll` observer** against a remote S3/Azure bucket, or a small local coordinator. But there is **no released ARM64 Linux asset** and CI flags the `openssl-sys` cross-build as broken, so it requires a local source build (and likely a `rustls`/vendored-OpenSSL swap) — not turnkey. |
| **CLAIM** `Pi Zero` (armv6, ~512 MB RAM) | **Marginal — QUALIFIED** | bundled C SQLite + 14 tree-sitter grammars + tokio + 2 cloud SDKs (`Cargo.toml`) | The dependency surface is large to compile and RAM-hungry; `grit init` AST-parses the whole tree (`src/parser/mod.rs`) which is the heavy step. A Pi Zero might act as a poll-only lock *client* against a remote bucket, but local indexing of a real repo is impractical. Treat as observer-tier only. |
| **mobile** (phones / tablets) | **N/A** — no supported, sandboxed-OS target | CI is Unix/macOS-only (`ci.yml:20-21`); host needs `git` + worktree tree + tokio HTTPS | iOS/Android app sandboxes provide neither a `git` CLI nor a free worktree filesystem, and grit ships no mobile target. A rooted Android via Termux could *in theory* run a source build against the same S3/Azure bucket, but that is unsupported and untested — so `mobile` is not a coordinator tier today. The realistic mobile role is a passive Event-Grid/webhook **viewer** of lock state, which is outside grit's binary. |
| **`AI glasses`/`wearables`** | **N/A** — no general-purpose compute / no git | host contract above; nothing in-repo targets RTOS/embedded | Wearables and `AI glasses` have no `git`, no worktree filesystem, and no role editing an AST symbol graph. The only conceivable touch is receiving an Event Grid push as a notification surface — that is a downstream consumer of grit's events, not a grit node. |
| **`ESP32` / ESP32-S3 (no_std MCU)** | **N/A** — architecturally impossible | no `no_std`/embedded/`esp32` anywhere (repo-wide grep returns nothing); deps require `std` + threads + filesystem + TLS | grit needs `std`, a multi-thread tokio runtime, a real filesystem (SQLite file, git worktrees), and outbound HTTPS — none exist on an `ESP32`-class microcontroller. It cannot be an agent node or a lock host. The furthest edge role is an MQTT/Event-Grid-bridged dumb display of lock state, which requires zero grit code on the device. |
| **`vendor`/`cloud`/`local` providers (the lock substrate)** | **This IS grit's distribution axis** | `GritConfig.backend` = `local`\|`s3`\|`azure` (`src/config.rs:9-18`); provider table README:141-143 | `local` = SQLite WAL (single machine, zero setup). `cloud`/`vendor` = AWS S3, Cloudflare R2, MinIO (S3 path) and native Azure Blob + Event Grid. Multi-`vendor` is abstracted behind `LockStore`; failover/liveness is the TTL+GC lease (`gc_expired_locks`, expired-holder reclaim `s3_store.rs:188`). Atomicity grade varies by `vendor`: native conditional PUT (S3/R2/Azure) vs GET-then-PUT TOCTOU (MinIO). |
| **Offline / degraded mode** | **Local-only fallback** | SQLite WAL default (`config.rs:23`); Unix-socket events (`room/mod.rs`) | With no network grit runs fully on the `local` SQLite backend with real-time Unix-socket events; it simply loses cross-machine coordination. Cloud backends degrade to `grit watch --poll N` when push events are unavailable (README:229). |

---

## 3. Language / runtime map — where `Lua`/`Luau` could fit (and why it is N/A today)

- **Today: `Rust`-only, no scripting plane.** A repo-wide search for
  `lua|luau|mlua|lune|wasm|no_std|embedded` returns **zero hits**. All policy is
  hard-coded `Rust`: lock modes write/read (`lock_store.rs:default_mode`),
  dependency-aware `--with-deps` auto-read-locking (README:120), queueing
  (`grit queue`, README:202), and the serialized merge gate (`.grit/merge.lock`,
  README:255). There is no embedded interpreter and no policy hook surface.
- **Why `Lua`/`Luau` is N/A right now.** There is no extension point that an
  operator currently needs to script: the claim/queue/merge-gate decisions are
  fixed and well-defined, and adding an interpreter would enlarge the trust
  boundary for no present requirement. Under envctl's **no-C** trust rule, the
  common `mlua` (C Lua) binding would be **disallowed**; only a pure-Rust Luau
  (`full_moon`/`piccolo`-style) would be admissible — another reason not to add it
  speculatively.
- **If a policy plane is ever wanted** (future, not now): a sandboxed pure-Rust
  `Luau` evaluator could express custom claim-admission / queue-priority /
  merge-gate policy without recompiling grit. That is captured as the speculative
  UPGRADE row below, explicitly gated on a real requirement first.

---

## 4. Control / data plane summary

- **Scheduling / discovery:** none built-in. grit decides *who may edit which AST
  symbol*, not *which host runs the agent*. Fleet scheduling is the caller's job;
  grit is the contention gate beneath it.
- **Telemetry / message bus / A2A:** local Unix-socket event stream (`room.sock`),
  Azure Event Grid push, or polling (`watch --poll`). Cross-host A2A is implicit —
  agents observe each other through the shared lock bucket, not a direct bus.
- **Secrets:** S3 creds via standard AWS chain (`aws_config::defaults`,
  `s3_store.rs:38`); Azure account + access key passed at `config set-azure`
  (README:110) and persisted to `.grit/config.json` (`config.rs:51`) — a plaintext
  secret-at-rest concern flagged for the governance/secrets axis, not this one.
- **Bandwidth / power:** each cloud lock op is a synchronous HTTPS round-trip
  (`block_on`, `s3_store.rs:144`); high TTL-refresh/poll cadence on a metered or
  battery-bound edge node is costly — reinforces why low-power tiers (`Pi Zero`,
  `mobile`) are observer-only at best.
- **Data residency:** the lock truth physically lives wherever the chosen `vendor`
  bucket/region is provisioned (`S3Config.region`, `AzureConfig` in `config.rs`).

---

## 5. UPGRADE rows

| # | UPGRADE | Evidence / driver | Acceptance | Risk | Reversibility |
|---|---------|-------------------|------------|------|---------------|
| U1 | Swap `openssl-sys` for `rustls` across aws/azure SDK features and re-add the `aaarch64-unknown-linux-gnu` release target | `release.yml:41-43` (ARM64 unbuildable due to openssl-sys); no ARM64 asset ships | `release.yml` matrix builds + uploads an ARM64 Linux asset; `Raspberry Pi` runs the released binary with no source build | Low-med: TLS backend swap may shift cloud-SDK behavior; needs the cloud integration tests re-run | High — revert the matrix entry + dependency feature flags |
| U2 | Add a pure-Rust pollerd/agent-light profile (no tree-sitter, lock-client only) behind a cargo feature for edge observers | heavy deps gate `Pi Zero`/edge (`Cargo.toml`); poll mode already exists (README:229) | A feature build with no `tree-sitter`/grammars produces a small binary that can `status`/`watch --poll`/`heartbeat` against a remote bucket on a `Raspberry Pi`/`Pi Zero` | Med: must cleanly cfg-gate the parser out of `init`-less paths | High — feature flag, default off |
| U3 | (Speculative, gated) Optional sandboxed pure-Rust `Luau` policy plane for claim-admission / queue-priority / merge-gate hooks | no scripting today (repo grep empty); fixed-`Rust` policy (§3) | Only pursued once a concrete operator policy need exists; if built, uses a no-C `Luau` engine and a capability-restricted host API; default-off | Med-high: enlarges trust boundary; must honor envctl no-C rule | High — opt-in, removable |
| U4 | Document/guard the MinIO GET-then-PUT TOCTOU so self-hosted `local`/`vendor` deployments understand the weaker atomicity tier | `s3_store.rs:403-418` (acknowledged TOCTOU); table README:142 marks MinIO "fallback" | A backend-capability note + a startup warning when a non-conditional-PUT provider is selected | Low | High — doc + warning only |

---

## 3-line summary
grit is a `Rust`, Unix-only, git-CLI-dependent merge-lock **coordinator** whose only distribution seam is its `LockStore` (local SQLite WAL | S3/R2/MinIO | Azure Blob+Event Grid), using TTL-leased, conditional-PUT locks as the cross-machine source of truth — so it realistically serves workstation, local-server, and `cloud`/`vendor` tiers, and full-Linux `Raspberry Pi` from source, while `Pi Zero` is observer-only.
`mobile`, `AI glasses`/`wearables`, and `ESP32` are N/A — no `git`/worktree/std/tokio-HTTPS host (ESP32 is no_std-impossible; mobile/wearables can at most consume Event Grid notifications), and no ARM64 Linux binary ships today (openssl-sys, `release.yml:41-43`).
There is no `Lua`/`Luau` plane and none is warranted yet — policy is fixed `Rust`; a future pure-Rust `Luau` hook (no-C per envctl) is filed as a gated UPGRADE, alongside an `openssl→rustls` swap to unlock ARM64 and an edge lock-client profile.
