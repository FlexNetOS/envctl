# weave — best-practices + latest trends (trend-research, cycle 4)

| field | value |
|---|---|
| target | **weave** — Rust agent-to-agent session mesh / transport; SQLite-mailbox broker + terminal-pane injection |
| target_root | /home/drdave/Desktop/meta/weave |
| researched | 2026-06-26 |
| recency window (90d) | **2026-03-28 → 2026-06-26** |
| frame | meta = ONE converging system. weave = the **A2A transport plane** (nervous system), DISTINCT from handoff's witnessed-receipts plane. Goal: converge weave toward **A2A v1.0** as a *strict-upgrade interop boundary* — keep the working SQLite-mailbox transport; ADD A2A interop without removing it. |
| reuse | cycle-1 `research/rusty-idd.trends.md` §D1 (A2A v1.0 LF) and cycle-2 `research/handoff.trends.md` §C1 (A2A transport↔state split) — cited, not duplicated. |
| findings | 11 material (8 in-window · 2 carried-in-window · 1 flagged-older); 1 advisory cluster (5 RUSTSEC ids, all scoped to optional `libsql` feature). |

Verified target pins (from weave `Cargo.toml` manifests + `deny.toml`, read 2026-06-26):
`tokio 1.52.3` · `libsql 0.9.30` (optional, default-features off: core/remote/tls) · `rusqlite 0.40.0`
(bundled) · `serde 1.0.228` · `serde_json 1.0.150` · `ed25519-dalek 2` + `sha2 0.10` (optional
`sign`) · `reqwest 0.12` rustls (optional `llm`/`surfaces`) · `clap 4.6.1` · `anyhow 1.0.102` ·
`criterion 0.7.0` (dev). Backends are mutually-exclusive features (`sqlite` default vs `libsql`).
There is **NO axum/hyper/tonic HTTP server** in weave's tree — `weave serve`/dashboard is served over
the SQLite mailbox + (optional) reqwest *client*; no inbound HTTP server crate is pinned (relevant to
the A2A-server question below).

---

## A. A2A protocol — state of standard (the interop boundary weave should converge toward)

### A1. A2A v1.0 is the current stable Linux Foundation standard; weave does NOT speak it yet. [HIGH · in-window]
- **A2A (Agent2Agent) v1.0** is the current stable release of the LF-governed cross-vendor interop
  standard (2026). One-year milestone announced **2026-04-09** (150+ orgs, 22k+ stars, SDKs in
  Python/JS/Java/Go/.NET; GA in Microsoft Copilot Studio / Azure AI Foundry / Amazon Bedrock
  AgentCore). Governance: Google donated A2A to the Linux Foundation **2025-06-23**.
- **On the wire:** JSON-RPC 2.0 over HTTP, **Server-Sent Events**, or **gRPC** bindings; clients
  consume task updates via **polling, streaming, or webhooks**; **version negotiation** lets one
  AgentCard advertise both v0.3 and v1.0 behavior (backward-compatible migration path).
- **Signed Agent Cards:** a cryptographic signature on the AgentCard lets a receiving agent verify the
  card was issued by the domain owner — trust *before* interaction across org boundaries.
- weave today uses its **OWN `Intent` struct** over a SQLite mailbox — no formal A2A AgentCard, no
  JSON-RPC/gRPC envelope, no version negotiation. So A2A is a **gap**, not a regression.
- Source: https://a2a-protocol.org/latest/announcing-1.0/ (v1.0 announcement, accessed 2026-06-26);
  https://www.linuxfoundation.org/press/a2a-protocol-surpasses-150-organizations-lands-in-major-cloud-platforms-and-sees-enterprise-production-use-in-first-year
  (LF press, 2026-04-09); cycle-1 `research/rusty-idd.trends.md` §D1 (A2A v1.0 LF / gRPC / signed cards, carried).
- Relevance: weave is meta's **local A2A-shaped substrate**. A2A v1.0 (+ signed cards, gRPC) is the
  strict-upgrade interop target. Architect direction: keep the SQLite-mailbox transport as the
  required local route; add an A2A **adapter** (emit/consume AgentCard + JSON-RPC task envelopes) as a
  strict upgrade — never replace the mailbox.
- **Refute attempt:** is A2A vaporware / a single-vendor pet? No — LF press + v1.0 spec site +
  multi-cloud GA + 150+ orgs across two cycles of corroboration. PASS (confirmed).

### A2. weave's optional `sign` feature (ed25519-dalek) is the local analogue of A2A's signed Agent Cards. [MED · in-window]
- A2A v1.0's headline trust primitive is the **signed AgentCard** (ed25519/EC signature proving card
  provenance). weave already ships an **optional `sign` feature** (`ed25519-dalek 2` + `sha2 0.10`) —
  the same primitive, applied to its Intent records rather than to an AgentCard.
- Relevance: the A2A adapter can REUSE weave's existing `sign` machinery to produce A2A-compatible
  signed cards/messages — minimal new crypto surface, on-trend with v1.0's security model.
- Source: A2A v1.0 announcement (signed Agent Cards), https://a2a-protocol.org/latest/announcing-1.0/
  (accessed 2026-06-26); weave `weave-core/Cargo.toml` `sign = ["dep:ed25519-dalek","dep:sha2"]` (read 2026-06-26).
- **Refute:** does weave's per-Intent signing actually map to card signing? It is the *primitive* match
  (same alg, same crate), not a wire-format match — QUALIFIED: reuse the crypto, still need card/JSON-RPC framing.

---

## B. Agent mesh / multi-agent transport patterns (2026)

### B1. Inter-agent comms decompose into transport × protocol × topology — weave owns transport+topology, A2A owns protocol. [HIGH · in-window]
- The 2026 field models inter-agent communication as **three independent decisions**: **transport**
  (how a message moves), **protocol** (A2A / MCP / ACP / ANP), and **topology** (orchestrated /
  direct-A2A / message-bus). Three patterns "survived production": **agent-flow** (assembly line),
  **orchestration** (hub-and-spoke), **bounded collaboration** (controlled peer mesh).
- Relevance: weave is a **message-bus transport** with a **controlled peer-mesh** topology (SQLite
  mailbox + pane injection). It is *protocol-agnostic today* (own Intent) — which is exactly why
  bolting A2A on as the **protocol** layer is the natural, non-destructive upgrade: weave keeps owning
  transport+topology; A2A becomes the swappable protocol envelope.
- Source: https://www.taskade.com/blog/inter-agent-communication-patterns (accessed 2026-06-26);
  https://niteagent.com/blog/multi-agent-production-2026/ ("3 patterns that survived", accessed 2026-06-26).
- **Refute:** are these blog taxonomies or real practice? Corroborated by Google ADK's "multi-agent as
  default, A2A for cross-agent comm" (B2) and the LF adoption data (A1) → not just blog vibes. PASS.

### B2. Cross-machine pattern: orchestrator/worker over A2A, with non-vendor agents reachable via the same protocol. [MED · in-window]
- Google's Agent Development Kit treats **multi-agent as the default deployment mode** and uses **A2A
  for cross-agent communication** — an ADK supervisor talks to ADK workers via A2A, and those workers
  reach **non-Google agents via the same A2A protocol**. This is the canonical cross-machine /
  cross-vendor transport pattern the field is standardizing on.
- Relevance: validates weave's role as the **local** orchestrator/worker bus and the case for an A2A
  edge adapter so weave-mediated agents can talk to *external* (non-meta) A2A agents over the same wire.
- Source: https://www.taskade.com/blog/inter-agent-communication-patterns (ADK / A2A cross-vendor,
  accessed 2026-06-26); A2A multi-cloud GA, LF press 2026-04-09 (A1).
- **Refute:** does this force weave onto gRPC/HTTP cross-machine? No — A2A is *one* edge; weave's local
  SQLite-mailbox transport stays primary. QUALIFIED (adapter at the edge, not a transport swap).

### B3. Each agent message = a traceable event in an event log; heartbeat/presence via a low-latency store. [MED · in-window]
- 2026 best-practice: treat **every agent message as a traceable event** (stable id + context) stored
  in an **event log / vector DB** for observability and replay; track **presence via heartbeats** in a
  low-latency ephemeral store (Redis cited as the common choice for frequent heartbeat updates).
- Relevance: weave's **SQLite mailbox already IS the event log** (durable, queryable, ordered) — on-
  trend for traceability/replay. The presence/heartbeat axis is where weave diverges: it uses the same
  durable SQLite store rather than an ephemeral one. That is a deliberate, *defensible* choice for a
  local single-node mesh (durability > sub-ms latency), but the architect should note the trade-off if
  weave ever scales to high-frequency multi-node heartbeats.
- Source: https://www.taskade.com/blog/inter-agent-communication-patterns (message-as-event +
  heartbeat/Redis, accessed 2026-06-26); https://niteagent.com/blog/multi-agent-production-2026/ (accessed 2026-06-26).
- **Refute:** is SQLite "wrong" for a mailbox? No — for a local broker, a durable embedded log is a
  feature (no external service, survives restart). The Redis pattern targets cloud-scale presence fan-
  out; not weave's deployment. QUALIFIED — no change required; note the scaling boundary.

---

## C. weave = transport vs handoff = witnessed-receipts — the field validates the plane split

### C1. The field explicitly SEPARATES the A2A transport/mesh from durable verifiable state. [HIGH · in-window · carried + refreshed]
- The 2026 reliability literature treats **transport** (A2A: discover + exchange tasks + coordinate)
  as **distinct** from **durable verifiable receipts** that prove work occurred (the audit trail), and
  makes **observability/verifiable execution a first-class plane**. This is the exact meta design
  invariant: **weave = transport plane** (A2A-shaped: discover/route/exchange) vs **handoff = durable
  witnessed-state plane** (signed receipt/ledger). A2A standardizes *transport/discovery*; it does
  **not** subsume the need for a separate verifiable-state plane.
- Relevance: do **not** fuse weave and handoff. Adapter direction (from cycle-2): emit handoff witness
  records as A2A-compatible signed artifacts; let weave/A2A carry them. weave's optional `sign`
  feature (A2) is the transport-side analogue of handoff's signed witness chain.
- Source: https://dev.to/chunxiaoxx/building-multi-agent-ai-systems-in-2026-a2a-observability-and-verifiable-execution-10gn
  (2026-04-10, in-window); cycle-2 `research/handoff.trends.md` §C1 (transport↔state split, carried);
  cycle-1 `research/rusty-idd.trends.md` §D1 (A2A v1.0 LF, carried).
- **Refute:** does A2A's task-state make handoff redundant? No — A2A task state is *in-flight* coordination
  state, not a tamper-evident historical receipt chain. Distinct planes. PASS (confirmed across 2 cycles).

### C2. EMERGING (signal, not yet best-practice): topology-independent agent naming / capability discovery. [LOW · in-window · watch]
- Recent arXiv work proposes **topology-independent agent naming + capability-based discovery** (Agent
  Identity URI schemes; "Internet of Agents" / NANDA index + verified AgentFacts layered above A2A).
  This is **new-but-unproven** — research-stage, not a ratified standard.
- Relevance: a *future* watch item for weave's addressing/discovery layer (how a weave agent is named
  across machines). Mark as **trend/pilot**, NOT a current best-practice; do not plan adoption yet.
- Source: https://arxiv.org/pdf/2601.14567 (Agent Identity URI Scheme, 2026, in-window);
  https://arxiv.org/pdf/2511.19699 (Layered Protocol Architecture for the Internet of Agents).
- **Refute:** is this actionable now? No — pre-standard research. Correctly labelled emerging/watch.

---

## D. Tool-currency & advisories (weave's stack vs latest stable, 2026-06-26)

All versions verified against the crates.io API on 2026-06-26. Window = 2026-03-28 → 2026-06-26.

| crate (weave pin) | latest stable | published | verdict |
|---|---|---|---|
| **tokio 1.52.3** | 1.52.3 | 2026-05-08 | **CURRENT** (exact match; in-window) |
| **libsql 0.9.30** (opt) | 0.9.30 (stable); 0.10.0-pre.4 (pre, 2026-06-02) | 2026-03-19 | **CURRENT stable** — 0.10 is pre-release only; do not chase |
| **rusqlite 0.40.0** | 0.40.1 | 2026-06-06 | **1 patch behind** (trivial; in-window bump — bump to 0.40.1) |
| **clap 4.6.1** | 4.6.1 | 2026-04-15 | **CURRENT** (exact; in-window) |
| **serde 1.0.228** | 1.0.228 | 2025-09-27 | **CURRENT** (latest is this; release older-than-window but unsuperseded) |
| **serde_json 1.0.150** | 1.0.x line | — | current 1.0 line (no breaking successor) |
| **ed25519-dalek 2** (opt `sign`) | 2.2.0 | 2025-07-09 | **CURRENT** (v2 line; release older-than-window but unsuperseded) |
| **sha2 0.10** (opt `sign`) | 0.10.x (RustCrypto) | — | current 0.10 line |
| **reqwest 0.12** (opt `llm`/`surfaces`) | **0.13.4** | 2026-05-25 | **1 minor behind** — 0.13 is a new minor w/ changes; OPTIONAL features only; evaluate, not urgent |
| **anyhow 1.0.102** | 1.0.x line | — | current 1.0 line |
| **criterion 0.7.0** (dev) | 0.7.x line | — | current 0.7 line |

### Advisory cluster (5 RUSTSEC ids) — ALL scoped to the OPTIONAL `libsql` `tls` feature; DEFAULT build is clean.
weave's `deny.toml` already enumerates these as explicit, scoped, time-bounded ignores (WL-044b);
`cargo tree -i rustls-webpki` on **default** features matches nothing — they only compile under
`--features libsql` (remote-TLS). Independently re-verified one id against RUSTSEC below.

| advisory | crate / issue | fixed in | published | scope in weave |
|---|---|---|---|---|
| **RUSTSEC-2026-0104** | rustls-webpki — reachable **panic** parsing CRL `IssuingDistributionPoint` empty BIT STRING (DoS, pre-sig-validation) | **>=0.103.13** (and 0.104.0-alpha.7+) | **2026-04-22** (in-window) | optional libsql remote-TLS only |
| RUSTSEC-2026-0098 | rustls-webpki — name-constraints (URI) | >=0.103 | 2026 | optional libsql remote-TLS only |
| RUSTSEC-2026-0099 | rustls-webpki — wildcard name-constraints | >=0.103 | 2026 | optional libsql remote-TLS only |
| RUSTSEC-2026-0049 | rustls-webpki — CRL Distribution Point matching | >=0.103 | 2026 | optional libsql remote-TLS only |
| RUSTSEC-2025-0134 | rustls-pemfile — **unmaintained** (pulled by libsql's rustls-native-certs) | (no fix; migrate off) | 2025 | optional libsql remote-TLS only |

- **Why still ignored (upstream-blocked):** the patched `rustls-webpki >=0.103` requires `rustls 0.23`
  / `hyper-rustls 0.27`, which **libsql still pins to 0.25** even on the latest stable (0.9.30) — so
  weave cannot pull the fix without libsql upgrading its TLS stack first. The ignore is **scoped,
  documented, and CI-gated** (any *new* advisory not listed fails `cargo deny check advisories`).
- **Note:** the bincode advisory **RUSTSEC-2025-0141 was eliminated** by WL-044b's libsql feature trim
  (no longer in the tree) — already resolved, not a live item.
- **Architect action items (tool-evaluation R7):** (1) bump `rusqlite 0.40.0 → 0.40.1` (trivial,
  in-window); (2) track libsql for a rustls-0.23 TLS-stack upgrade — that is the single upstream
  unblock that clears all four rustls-webpki ids + the rustls-pemfile unmaintained flag; (3) the
  `reqwest 0.12 → 0.13` minor is OPTIONAL-feature-only — schedule, not urgent; (4) default/`sqlite`
  build carries **zero** known advisories — the recommendation is to keep `sqlite` the default route
  and treat `libsql`/remote-TLS as the opt-in surface that owns the advisory budget.
- Sources: weave `deny.toml` (WL-044b advisory ignores, read 2026-06-26); crates.io API (versions,
  accessed 2026-06-26); https://rustsec.org/advisories/RUSTSEC-2026-0104.html (verified, 2026-04-22).
- **Refute:** are these advisories a real risk to weave's default build? No — re-verified they are
  confined to the optional `libsql`+`tls` path; the default `sqlite`/rusqlite build does not compile
  rustls-webpki at all. Risk is real ONLY when `--features libsql` is enabled. CONFIRMED.

---

## E. Synthesis for the architect

1. **Converge weave → A2A v1.0 as a strict upgrade.** Keep the SQLite-mailbox transport as the
   required local route (B1/B3); add an A2A **adapter** layer — AgentCard + JSON-RPC-2.0 task
   envelopes (HTTP/SSE; gRPC optional) with v0.3↔v1.0 version negotiation (A1). Reuse weave's existing
   `sign` (ed25519-dalek) for **signed Agent Cards** (A2). This is the cross-vendor interop boundary.
2. **Do NOT fuse weave with handoff.** The field separates transport (weave/A2A) from durable
   verifiable receipts (handoff) — keep two planes; bridge them by emitting handoff witness records as
   A2A-compatible signed artifacts (C1).
3. **Tool currency is healthy.** Only one trivial bump owed (rusqlite 0.40.1). The whole advisory
   budget lives behind the optional `libsql` feature and is upstream-blocked on libsql's TLS stack —
   already documented + CI-gated. Default build is advisory-clean.
4. **Watch, don't adopt:** topology-independent agent naming / Internet-of-Agents discovery (C2) is
   pre-standard research — a future addressing-layer signal, not a current best-practice.

Confidence: **HIGH** on tool-currency/advisories (versions from crates.io API + repo's own deny.toml +
one RUSTSEC re-verify) and on A2A state-of-standard (LF press + spec site, corroborated across 3
cycles). **MEDIUM** on mesh-pattern findings (reputable secondary/blog + ADK corroboration, primary
spec for the protocol layer). No fabricated dates or versions; older-than-window crate *releases*
(serde, ed25519-dalek) are flagged as "older release, still current because unsuperseded."

---

## Sources

| id | claim | url | publisher | published | in-window |
|---|---|---|---|---|---|
| A1 | A2A v1.0 announcement (gRPC/SSE/signed cards/version-negotiation) | https://a2a-protocol.org/latest/announcing-1.0/ | A2A Protocol (LF) | 2026 (accessed 2026-06-26) | yes (accessed) |
| A1 | A2A 1-year / 150+ orgs / multi-cloud GA | https://www.linuxfoundation.org/press/a2a-protocol-surpasses-150-organizations-lands-in-major-cloud-platforms-and-sees-enterprise-production-use-in-first-year | Linux Foundation | 2026-04-09 | yes |
| A1/A2 | A2A v1.0 LF / gRPC / signed cards (carried) | research/rusty-idd.trends.md §D1 | cycle-1 note | 2026-03-14 | carried (in-window) |
| B1 | transport×protocol×topology; comms patterns | https://www.taskade.com/blog/inter-agent-communication-patterns | Taskade | accessed 2026-06-26 | yes (accessed) |
| B1/B3 | 3 production patterns that survived | https://niteagent.com/blog/multi-agent-production-2026/ | NiteAgent | accessed 2026-06-26 | yes (accessed) |
| C1 | A2A transport ↔ verifiable-state split | https://dev.to/chunxiaoxx/building-multi-agent-ai-systems-in-2026-a2a-observability-and-verifiable-execution-10gn | DEV Community | 2026-04-10 | yes |
| C1 | transport/state split (carried) | research/handoff.trends.md §C1 | cycle-2 note | 2026-04-10 | carried (in-window) |
| C2 | topology-independent agent naming (emerging) | https://arxiv.org/pdf/2601.14567 | arXiv | 2026 | yes |
| C2 | layered Internet-of-Agents protocol (emerging) | https://arxiv.org/pdf/2511.19699 | arXiv | 2025 | flagged-older (watch) |
| D | rusqlite latest 0.40.1 | https://crates.io/api/v1/crates/rusqlite | crates.io | 2026-06-06 | yes |
| D | tokio latest 1.52.3 | https://crates.io/api/v1/crates/tokio | crates.io | 2026-05-08 | yes |
| D | libsql 0.9.30 stable / 0.10.0-pre.4 | https://crates.io/api/v1/crates/libsql | crates.io | 2026-03-19 | edge (stable older; pre in-window) |
| D | clap latest 4.6.1 | https://crates.io/api/v1/crates/clap | crates.io | 2026-04-15 | yes |
| D | reqwest latest 0.13.4 | https://crates.io/api/v1/crates/reqwest | crates.io | 2026-05-25 | yes |
| D | serde latest 1.0.228 | https://crates.io/api/v1/crates/serde | crates.io | 2025-09-27 | flagged-older (unsuperseded) |
| D | ed25519-dalek latest 2.2.0 | https://crates.io/api/v1/crates/ed25519-dalek | crates.io | 2025-07-09 | flagged-older (unsuperseded) |
| D | RUSTSEC-2026-0104 rustls-webpki CRL panic (fix >=0.103.13) | https://rustsec.org/advisories/RUSTSEC-2026-0104.html | RustSec | 2026-04-22 | yes |
| D | advisory cluster scoping + ignores (WL-044b) | weave/deny.toml | weave repo | read 2026-06-26 | yes (repo state) |
