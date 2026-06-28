# Distributed-Compute Findings — TARGET=icm (cycle 7)

Axis: `distributed-compute` (Rust/Lua multi-vendor edge↔cloud fabric).
Frame: meta = one converging system; goal = handoff + rusty-idd UNION; icm = the
persistent-memory organ. Read-only audit. All paths absolute; all claims cited to
`file:line` in `/home/drdave/Desktop/meta/icm`.

Verdict up front: icm is a **portable-but-native Rust memory store**. Its binary
cross-compiles to the desktop/server/64-bit-ARM-Linux class, but it carries a
**hard, non-optional C dependency** (rusqlite `bundled` SQLite + sqlite-vec) plus
an **optional heavy native ML stack** (fastembed → ONNX Runtime). The C floor is
the load-bearing distributed-compute + convergence finding: **icm cannot live
inside handoff's no-C (redb) trust boundary as-is** — they meet over a wire/IPC
seam, not in one process.

---

## Hardware target matrix (where icm's binary actually runs)

Release matrix is desktop/server-class only — five 64-bit targets, no MCU/mobile/iOS/Android:
`x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`,
`aarch64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`
(`/home/drdave/Desktop/meta/icm/.github/workflows/release.yml:42-54`). The aarch64
Linux build uses a cross GCC toolchain + `vendored-openssl` to compile OpenSSL from
source for the target arch (`release.yml:68-77`), which is itself proof that the
build pulls native C/C++ that must be cross-linked per target.

---

## Axis findings (CLAIM rows)

### Rust
- **CLAIM (CONFIRMED):** icm is pure-Rust *source* — a 4-crate Cargo workspace
  (`icm-core`, `icm-store`, `icm-mcp`, `icm-cli`)
  (`/home/drdave/Desktop/meta/icm/Cargo.toml:3-8`) tuned for a real shipping binary:
  `opt-level=3, lto=true, codegen-units=1, panic="abort", strip=true`
  (`Cargo.toml:11-15`). Cross-platform plumbing is genuine: `directories`/`walkdir`
  for paths, `libc` gated `cfg(unix)` only
  (`/home/drdave/Desktop/meta/icm/crates/icm-cli/Cargo.toml`, `[target.'cfg(unix)'.dependencies]`).
- **CLAIM (QUALIFIED):** "pure Rust" is true at the `.rs` level but **false at the
  link level** — `icm-store` *unconditionally* depends on `rusqlite { features =
  ["bundled","modern_sqlite"] }` and `sqlite-vec`
  (`/home/drdave/Desktop/meta/icm/crates/icm-store/Cargo.toml:7-9`;
  workspace pin `Cargo.toml:18-19`). `bundled` compiles the SQLite **C** amalgamation
  at build time; `sqlite-vec` is a C extension loaded via FFI
  (`crates/icm-store/src/store.rs:8` imports `rusqlite::ffi::sqlite3_auto_extension`;
  `store.rs:81-88` registers `sqlite_vec::sqlite3_vec_init` through an `unsafe`
  `transmute`). This C floor exists in **every** build, even with all features off.

### Lua / Luau
- **CLAIM (N/A — confirmed absent):** icm embeds **no Lua/Luau** scripting plane.
  A workspace-wide search for `mlua`/`rlua`/`lune`/`luau`/`lua` over `crates/` and
  `Cargo.toml` returns zero engine matches. icm has no policy/scripting DSL — its
  "policy" is hard-coded Rust (temporal decay, topic matching, importance levels;
  e.g. `crates/icm-core/src/lib.rs:48-60` topic/keyword matching;
  `crates/icm-store/src/store.rs:128-151` auto-decay). For the Rust+Lua north-star
  this is a **gap, not a defect**: a small Lua/Luau plane could externalize
  decay/scoring/recall-ranking policy without touching the C store, but none exists today.

### mobile
- **CLAIM (REFUTED for shipping; QUALIFIED feasible):** icm does **not** ship a
  mobile build — no `*-android`/`*-ios` targets in the release matrix
  (`release.yml:42-54`). Feasibility split:
  - The **store** half is portable: rusqlite `bundled` is a well-trodden
    cross-compile to `aarch64-linux-android`/`aarch64-apple-ios` (SQLite C compiles
    on those NDK/SDK toolchains); sqlite-vec is the same FFI registration path
    (`store.rs:81-88`) and would come along.
  - The **embeddings** half is the blocker: the `embeddings` feature pulls
    `fastembed` (`crates/icm-core/Cargo.toml:8-13`), which links the **ONNX Runtime**
    native lib and downloads ~100–400MB model weights at first use
    (`crates/icm-core/src/fastembed_embedder.rs:112-117` calls `TextEmbedding::try_new`
    with `with_show_download_progress(true)` into a `~/.cache/icm/models` dir
    `fastembed_embedder.rs:14-26`). On mobile, ONNX Runtime cross-builds are fragile
    and the weight download/storage is hostile to the platform.
  - **Mitigation already in the code:** embeddings are *optional at every layer* —
    cargo feature `embeddings` (default-on, `crates/icm-cli/Cargo.toml` `[features]`),
    `--no-embeddings` CLI flag (`crates/icm-cli/src/main.rs:53`), `ICM_NO_EMBEDDINGS`
    env, and config `embeddings.enabled` (`main.rs:1080`). A
    `--no-default-features` build yields a mobile-plausible store-only icm (SQLite C
    only, no ONNX), at the cost of vector recall (lexical/decay recall remains;
    `DEFAULT_EMBEDDING_DIMS=384` placeholder vectors, `crates/icm-core/src/lib.rs:17`).

### AI glasses / wearables
- **CLAIM (N/A — aspirational):** A full icm store on **AI glasses / wearables** is
  not feasible today and not targeted. Wearable class compute is MCU/lightweight-SoC
  with tight RAM/flash/power budgets; icm assumes a writable filesystem (creates
  `db dir` + WAL journaling, `store.rs:111-118` `journal_mode=WAL`, `busy_timeout=30000`),
  an LRU hot cache (`MEMORY_CACHE_CAP=256`, `store.rs:96`), and — by default — the
  ONNX/model footprint above. Even the store-only build wants a real SQLite C runtime
  + filesystem. **Correct architecture for wearables = thin client:** the glasses
  hold no store and call icm over the network (the MCP server `icm-mcp` /
  `icm serve`, or the RTK cloud sync endpoint below). The memory organ lives on the
  phone/workstation/cloud; the wearable is a sensor/recall surface only.

### Pi Zero / Raspberry Pi
- **CLAIM (QUALIFIED):** 64-bit Raspberry Pi (Pi 3/4/5, Pi Zero 2 W in 64-bit mode)
  is **directly covered** — the `aarch64-unknown-linux-gnu` release artifact runs on
  them (`release.yml:51`, `release.yml:265` publishes `icm-aarch64-unknown-linux-gnu.tar.gz`).
  SQLite C is trivial on ARM Linux. **Caveats:**
  - The original **Pi Zero / Zero W is armv6 (32-bit)** and has **no release
    artifact** — `armv7`/`arm-unknown-linux-gnueabihf` are absent from the matrix
    (`release.yml:42-54`); it would need a from-source cross build.
  - **fastembed/ONNX is the practical Pi blocker**: ONNX Runtime on ARM + a
    100–400MB model in `~/.cache/icm/models` (`fastembed_embedder.rs:14-26,112-117`)
    is heavy for Pi-Zero-class RAM. Run Pi deployments **store-only**
    (`--no-embeddings` / `ICM_NO_EMBEDDINGS`, `main.rs:53,1080`), or point recall at a
    remote embedder. A Pi 4/5 with embeddings on works but is slow on first model load.

### ESP32
- **CLAIM (N/A — not feasible, confirmed by dep graph):** icm is **impossible** on
  ESP32/ESP32-S3 class MCUs. icm is std-only and rests on rusqlite `bundled`, which
  needs a hosted libc + filesystem + heap to compile and run the SQLite C amalgamation
  (`crates/icm-store/Cargo.toml:7`; `store.rs:115` `Connection::open(path)` opens a
  real file DB). ESP32 is `no_std` (Xtensa/RISC-V, no OS filesystem, ~520KB SRAM) —
  there is no path to compiling SQLite C + ONNX there, and no `no_std` shim in the
  codebase. The ESP32 role in the fabric is, like wearables, a **pure remote client**
  of `icm-mcp` / the cloud endpoint — never a store host.

### vendor / cloud / local
- **CLAIM (CONFIRMED — local-first):** Default operation is **local**: an on-disk
  SQLite file under platform dirs, opened locally (`store.rs:104-126`), with the
  embedding model also **local** (fastembed runs ONNX in-process; weights cached
  locally `fastembed_embedder.rs:14-26`). No network is required for store/recall.
- **CLAIM (CONFIRMED — optional cloud sync, single vendor):** A **cloud** path exists
  in `crates/icm-cli/src/cloud.rs` — an RTK Cloud client (OAuth browser login to
  `cloud.rtk-ai.app`, `cloud.rs:1-6,125`) that pushes project/org-scoped memories via
  `ureq::post`/`ureq::get` (`cloud.rs:235,326,367`) for team context sharing.
  Credentials live in the platform config dir and **fall back to reusing rtk-pro
  credentials** (`cloud.rs:35-60`). Sync payloads use the `tar`+`flate2` (gzip) +
  `sha2` stack (workspace `Cargo.toml`, `icm-cli/Cargo.toml`). This is a **single
  vendor** (RTK Cloud), not a multi-vendor mesh — no Ollama/OpenAI/Anthropic/CF
  Workers routing in icm itself.
- **CLAIM (CONFIRMED — embedding vendor is local-by-default, model-pluggable):** The
  embedding "vendor" is local fastembed, model-selectable across ~30 models
  (`fastembed_embedder.rs:46-83`), default `intfloat/multilingual-e5-base` (768d)
  (`fastembed_embedder.rs:36`). There is no remote-embedding-API client — embeddings
  are either local-ONNX or off. The `Embedder` trait (`crates/icm-core/src/embedder.rs`)
  is the clean seam where a remote/API embedder *could* be added (relevant for
  mobile/Pi/wearable offload).

### The convergence crux (C-dependency vs handoff no-C boundary)
- **CLAIM (CONFIRMED — this is the key distributed-compute + convergence finding):**
  handoff's continuity kernel (`hf`) is deliberately **no-C / pure-Rust (redb)** per
  the meta harness contract (CLAUDE.md "redb (no-C) `hf`" / handoff PR #114 migration
  off legacy SQLite). icm is the **opposite**: an unconditional SQLite-C +
  sqlite-vec-C store (`crates/icm-store/Cargo.toml:7-9`; `store.rs:8,81-88`) plus
  optional ONNX. Therefore icm and handoff **cannot be merged into one no-C
  process/binary** — the meta UNION can only join them across a **wire/IPC seam**:
  handoff stays the no-C witnessed-ledger kernel; icm stays the native memory organ
  reached via its MCP server (`icm-mcp`) or cloud endpoint. Any plan that assumes
  icm folds *inside* handoff's trust boundary is infeasible as-is.

---

## UPGRADE rows (`axis: distributed-compute`)

| # | Upgrade | Evidence (file:line) | Acceptance | Risk | Reversibility |
|---|---------|----------------------|------------|------|---------------|
| U1 | **Ship a store-only profile as a first-class target** (no `embeddings`, no `tui`) for Pi/mobile/edge — document `--no-default-features` build, add `aarch64-unknown-linux-gnu` store-only artifact. | embeddings already optional at every layer (`crates/icm-cli/Cargo.toml [features]`; `main.rs:53,1080`; `crates/icm-core/Cargo.toml:6-13`) | edge build links SQLite C only, no ONNX; recall degrades to lexical/decay; binary materially smaller | Low — features already exist | High — additive build profile |
| U2 | **Add a remote `Embedder` impl** (HTTP API behind the `Embedder` trait) so Pi/mobile/wearable/ESP32 clients offload embedding to a workstation/cloud instead of running ONNX locally. | trait seam `crates/icm-core/src/embedder.rs:3-7`; local-only impl today `fastembed_embedder.rs:129-154` | a thin client computes vectors over the network; no ONNX on the edge device | Med — new network/auth surface, latency | High — new impl, default stays local |
| U3 | **Formalize the icm↔handoff wire seam** (icm as a network/MCP memory service handoff calls) rather than in-process linkage, recorded as an ADR, because of the C-vs-no-C boundary. | C floor `crates/icm-store/Cargo.toml:7-9`, `store.rs:8,81-88` vs handoff redb no-C kernel (meta CLAUDE.md) | UNION plan documents icm-as-service; no attempt to link SQLite-C into the `hf` binary | Low (doc/architecture) | High — reversible ADR |
| U4 | **Add `armv7`/`arm-unknown-linux-gnueabihf` (Pi Zero / 32-bit) to the build matrix**, store-only, for the smallest Pi class. | matrix lacks 32-bit ARM (`release.yml:42-54`); aarch64 cross pattern already proven (`release.yml:68-77`) | a 32-bit ARM artifact builds + runs the store-only profile | Med — extra cross toolchain, OpenSSL-vendored complexity | High — additive matrix row |
| U5 | **(Optional, north-star fit) Introduce a tiny Lua/Luau policy plane** for recall ranking / decay / scope rules, sandboxed via `mlua`, kept *out of* the C trust boundary. | no scripting plane today (zero `lua/mlua/luau` matches); policy hard-coded `lib.rs:48-60`, `store.rs:128-151` | operators tune ranking/decay without recompiling Rust; default behavior unchanged when no script present | Med — new embedded runtime, sandbox surface | High — feature-gated, opt-in |

---

## Confidence
High on the structural facts (crate graph, C dependency, feature gating, release
matrix, cloud path, absence of Lua) — all cited to source. Medium on mobile/Pi-Zero
*feasibility specifics* (ONNX-on-ARM behavior is inferred from the fastembed/ONNX
dependency, not a build attempt in this repo). N/A axes (Lua/Luau, AI glasses/
wearables, ESP32) are confirmed-absent or architecturally-excluded, not unknowns.
