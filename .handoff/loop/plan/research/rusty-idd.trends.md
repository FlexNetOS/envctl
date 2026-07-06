# rusty-idd — best-practices + latest trends + tool currency (trends note)

Target: **rusty-idd** — intent-driven / spec-driven development control plane (Rust workspace).
Researcher: plan-trend-researcher. Date: **2026-06-26**.
Recency window (90 days): **2026-03-28 → 2026-06-26**. Findings outside the window are flagged.
Frame: meta is ONE converging system; rusty-idd = intent-driven control plane converging with
weave (comms/A2A), icm (memory), handoff (continuity). Findings tied to that convergence path.

Method: deep-research (fan-out search → fetch → adversarial refute → cited synthesis). Every
load-bearing claim carries a URL + date. Reuses prior cycle notes (cited inline) — not duplicated:
`research/agentic-planning-trends-2026-06.md`, `research/plan-architecture-loop-distributed-compute-2026-06.md`.

Verified target pins (from `Cargo.lock` + manifests, 2026-06-26): clap 4.6.1 · ratatui 0.30.2 ·
serde 1.0.228 · serde_json 1.0.150 · crossterm 0.29.0 · tokio 1.52.3 · anyhow 1.0.102 · toml 0.9.6 ·
serde_yaml 0.9.34+deprecated (residual, transitive) · serde_norway 0.9 (spec/runner) · comrak 0.52 ·
minijinja 2 · thiserror 2 (spec) / 1.0.69 (transitive).

Confidence legend: HIGH = corroborated by primary source (release/advisory/repo) + in-window;
MED = single strong source or in-window blog; LOW = single secondary source / could not corroborate.

---

## A. Tool-currency & advisories (architect R7 input)

### A1. clap 4.6.1 — CURRENT, no advisory. [HIGH · in-window]
- Latest published clap is **4.6.1** (latest on crates.io/docs.rs as of 2026-06-26), recent release,
  MSRV Rust 1.85.0+. rusty-idd `crates/cli` pins `clap = "4"` → resolves to 4.6.1 in lock = current.
  No RustSec advisory found for clap.
- Source: https://crates.io/crates/clap (accessed 2026-06-26); https://docs.rs/crate/clap/latest (accessed 2026-06-26).
- Action: none. Optional: pin `4.6` to lock in the MSRV-1.85 floor explicitly.
- Refute attempt: searched RustSec advisory DB for clap → no matching advisory. Cannot find a counter-source. PASS.

### A2. ratatui 0.30.2 — CURRENT (latest 0.30.x line). [HIGH · in-window]
- rusty-idd pins **0.30.2**; the 0.30 line is the latest major. 0.30.0 was a structural rework:
  modular multi-crate workspace, full `no_std`/embedded support, new `ratatui::run()` API; 0.30.1
  added `Block::shadow`, Canvas/Chart filled-area (`GraphType::Area`), `Cell::column_span`, `Fill` widget.
- Source: https://ratatui.rs/highlights/v030/ ; https://ratatui.rs/highlights/v0301/ ;
  https://github.com/ratatui/ratatui/releases (accessed 2026-06-26).
- Note: 0.30's modular split means a TUI crate can depend on only the sub-crates it uses (faster builds,
  better API stability for the OpenSpec TUI in `crates/tui`).
- Action: none for currency. Opportunity (not a gate): adopt new widgets if the TUI needs them.

### A3. serde 1.0.228 / serde_json 1.0.150 — CURRENT, no advisory. [HIGH]
- No RustSec advisory for serde/serde_json at these versions. Action: none.
- Refute: RustSec search returned no serde advisory. PASS.

### A4. serde_yaml is DEPRECATED/UNMAINTAINED — residual transitive presence in rusty-idd. [HIGH · status older-but-current]
- `serde_yaml` last release is **0.9.34+deprecated (March 2024)**; upstream repo (dtolnay/serde-yaml)
  archived; RustSec tracks it unmaintained (advisory-db issue #2132).
- Source: https://github.com/rustsec/advisory-db/issues/2132 (issue 2024, status still current 2026-06-26);
  https://users.rust-lang.org/t/serde-and-yaml-support-status/125684 (community, 2026).
- rusty-idd state (verified from manifests): the *first-party* crates already migrated correctly —
  `crates/spec` and `crates/runner` use **serde_norway** (the maintained fork), with explicit comments
  "NOT serde_yml". GOOD. BUT the deprecated `serde_yaml 0.9` is still pulled in transitively by:
  - `crates/external/codegraph-core/Cargo.toml:40` → `serde_yaml = "0.9"`
  - `imports/prompt_hub/Cargo.toml:27` / `prompthub` → `serde_yaml = "0.9.34"`
  This keeps `serde_yaml 0.9.34+deprecated` in `Cargo.lock`.
- Action (P1, axis: governance/accuracy): migrate the external/import crates off `serde_yaml` to
  `serde_norway` (or `serde-yaml-ng`) to clear the deprecated crate from the lock; or, if those are
  vendored upstreams not owned here, document the residual as an accepted-known item.

### A5. Pick the RIGHT serde_yaml successor — `serde_norway` is reasonable but not bulletproof. [MED · in-window]
- The maintained drop-in landscape (2026): **serde_norway** (mdBook/Norway ecosystem; active, ~30 commits,
  but maintainers explicitly *not* committed long-term), **serde-yaml-ng** (acatton; independent
  continuation of dtolnay's serde-yaml), **serde-saphyr** (modern parser, no Value DOM), **yaml-rust2**
  (pure-Rust primitives). AVOID **serde_yml** (Sebastien Rousseau fork) — it is itself unmaintained;
  rusty-idd's comments already correctly say "NOT serde_yml".
- Source: https://users.rust-lang.org/t/serde-yaml-deprecation-alternatives/108868 (2026 thread);
  https://github.com/acatton/serde-yaml-ng (accessed 2026-06-26).
- Implication for plan: serde_norway is a sound current choice (no action needed now) but is a
  *watch* item — track its maintenance; `serde-yaml-ng` is the fallback if it stalls.
- Refute: confirmed serde_yml ≠ serde_norway; the "unmaintained" warning applies to serde_yml, not the
  Norway fork. The earlier-cycle note conflated these is NOT present — distinction holds. PASS.

### A6. tokio 1.52.3 — CURRENT; the 2026 advisories do NOT apply. [HIGH · in-window]
- 2026 RustSec hits for "tokio" are **RUSTSEC-2026-0057 (tokio-reactor unmaintained)** and
  **RUSTSEC-2026-0060 (tokio-timer unmaintained)** — both are the legacy **tokio 0.1** ecosystem, not
  tokio 1.x. The broadcast-clone unsoundness (RUSTSEC-2025-0023) is fixed in current 1.x. tokio 1.52.3
  is unaffected.
- Source: https://rustsec.org/advisories/RUSTSEC-2026-0057 ; https://rustsec.org/advisories/RUSTSEC-2026-0060.html ;
  https://rustsec.org/packages/tokio.html (accessed 2026-06-26).
- Action: none. Confirm `cargo deny`/`cargo audit` is in CI so future advisories surface automatically.

### A7. crossterm 0.29.0 / toml 0.9.6 / anyhow 1.0.102 / comrak 0.52 / minijinja 2 — current, no advisory found. [MED]
- No advisories surfaced for these at pinned versions. Lower-confidence (absence-of-evidence, not a
  dedicated per-crate audit). Action: rely on CI `cargo audit`/`cargo deny` for ongoing coverage.

---

## B. Spec-driven / intent-driven development tooling (state of the art, in-window)

### B1. OpenSpec is the most-active SDD framework; 1.x line + intent-driven template. [HIGH · mixed window]
- OpenSpec model: a single **living unified spec** as the authoritative reference (vs spec-scattered
  approaches). 1.0 (released **2026-01-26**, *out of window — flagged older, still the current baseline*)
  introduced custom schemas via `config.yaml`; **profiles** landed in 1.2; ~52.1k GitHub stars (June 2026);
  supports 21 AI tools. The **intent-driven template** (in-window, **2026-05-10**) wires OpenSpec +
  openspec-git-discipline + grill-me proposals + C4 diagrams + ADRs + a custom intent-driven schema into
  one SDD workflow.
- Source: https://github.com/Fission-AI/OpenSpec (accessed 2026-06-26);
  https://intent-driven.dev/blog/2026/05/10/spec-driven-development-openspec-opencode/ (2026-05-10);
  https://openspec.dev/ (accessed 2026-06-26).
- Relevance: rusty-idd IS an intent-driven control plane and already embeds OpenSpec workflow skills
  (`crates/tui/.claude/skills/openspec-*`). The intent-driven *custom-schema + profiles* path is the
  current best-practice for "one living spec + ADR + C4" — directly aligns with rusty-idd's binding of
  goals to OpenSpec artifacts. Best-practice to adopt: profiles/custom-schema for per-goal spec shape.

### B2. GitHub spec-kit v0.11.0 — competing/complementary SDD standard, 30+ agents. [HIGH · in-window]
- spec-kit codifies **Spec → Plan → Tasks → Implement**, each phase a Markdown artifact feeding the next;
  docs last updated **2026-05-27**; **v0.11.0 (June 2026)** supports 30+ agents (Claude Code, Copilot,
  Cursor, Gemini/Codex/Qwen CLI, Goose, Windsurf…); Claude Code is a native skill since v0.4.5.
- Source: https://github.com/github/spec-kit (accessed 2026-06-26);
  https://github.github.com/spec-kit/ (updated 2026-05-27);
  https://www.marktechpost.com/2026/05/08/meet-github-spec-kit-... (2026-05-08).
- Relevance: SDD has become an industry standard in-window (AWS **Kiro**, **Tessl** also cited). For
  rusty-idd's convergence: the field is consolidating on phase-gated Markdown artifacts + multi-agent
  fan-out — rusty-idd's OpenSpec-binding + Rusty-IDD gates are on-trend, but it should track spec-kit's
  cross-artifact-analysis + quality-checklist pattern as a best-practice baseline to compare against.
- Refute: is SDD just hype? Two independent primary toolchains (Fission-AI OpenSpec, github/spec-kit)
  with active 2026 releases + Microsoft/GitHub blog coverage → corroborated as real, not single-blog. PASS.

---

## C. Rust agentic control planes / orchestration crates + patterns (in-window)

### C1. Rust-native agent framework ecosystem solidified in 2026. [HIGH · in-window]
- By Q1–Q2 2026 a distinct Rust agent ecosystem exists: **Rig** (0xPlaygrounds — modular LLM apps,
  graph-workflow primitives, *no built-in orchestrator* — primitives compose), **Swiftide** (bosun-ai —
  workspace: swiftide-core / swiftide-agents / swiftide-indexing / swiftide-query), **AutoAgents**
  (liquidos-ai — Ractor **actor-model** multi-agent, described as "most complete"), **rs-graph-llm**
  (high-perf interactive multi-agent workflow), and **OpenFANG** ("Agent OS"). Reported gains vs Python:
  ~5x memory reduction, 25–44% latency improvement, far better cold start.
- Source: https://zylos.ai/research/2026-04-01-rust-native-ai-agent-frameworks-ecosystem-2026/ (2026-04-01);
  https://github.com/0xplaygrounds/rig ; https://github.com/bosun-ai/swiftide ;
  https://github.com/liquidos-ai/autoagents ; https://github.com/a-agmon/rs-graph-llm (accessed 2026-06-26).
- Relevance: rusty-idd already owns its control plane (clap CLI + ratatui TUI + crates/runner + weave).
  These are NOT drop-in replacements — but the **patterns** are the on-trend best-practice to mirror:
  (a) graph-based workflow scheduling (Rig/rs-graph-llm) ↔ the TDP target-DAG already queued in the
  planning loop; (b) actor-model isolation (AutoAgents/Ractor) ↔ rusty-idd's lane/worker isolation;
  (c) workspace-of-specialized-crates (Swiftide) ↔ rusty-idd's existing crate split.
- Treat as **trend/watch**, not adopt: pulling an external agent framework into a no-C, self-owned
  control plane is a large blast-radius decision — pattern-borrow over dependency-add.
- Refute (mini-swe-agent caution, prior note): strong results can come from a tiny harness; do not add
  framework complexity without an eval proving it pays. Carries forward — keep these as reference patterns.

### C2. Graph-orchestration / TDP is the dominant long-horizon planning pattern. [HIGH · carried + in-window]
- Reaffirms prior-cycle findings (Task-Decoupled Planning, Microsoft Agent Framework graph orchestration,
  Code-as-Agent-Harness). Rust frameworks above independently converge on graph/DAG scheduling →
  corroborates the planning loop's `plan-dependency-graph` (target-DAG, ready-set, localized self-revision).
- Source (prior, retained): TDP https://arxiv.org/abs/2601.07577 ; Microsoft Agent Framework
  https://github.com/microsoft/agent-framework — see `agentic-planning-trends-2026-06.md`. No supersession.

---

## D. Continuity / ledger kernels + A2A comms (weave/handoff convergence)

### D1. A2A is now a Linux Foundation standard at v1.0 — the cross-vendor target for weave. [HIGH · edge/in-window]
- A2A (Agent2Agent) is governed by the **Linux Foundation** (launched 2025-06-23); **v0.3** added
  **gRPC** support, **signed agent/security cards**, extended client support, with version negotiation
  guaranteeing backward-compatible v0.3→v1.0 migration; **v1.0 is the current stable (2026)**; 150+
  organizations, enterprise production use, in major cloud platforms.
- Source: https://www.linuxfoundation.org/press/a2a-protocol-surpasses-150-organizations-... (2026);
  https://dev.to/eclaw/a2a-protocol-tech-update-20260314-5emf (2026-03-14, in-window);
  https://a2a-protocol.org/latest/ ; https://github.com/a2aproject/A2A (accessed 2026-06-26).
- Relevance: meta's **weave** is the local A2A-shaped substrate. A2A v1.0 (+ signed cards, gRPC) is the
  external standard to converge toward as an interop boundary — matches the prior-cycle recommendation
  to add an `agent_interop` registry (weave | MCP | ACP | A2A) and keep weave the required local route
  while adding A2A/MCP adapters as strict upgrades (see `agentic-planning-trends-2026-06.md` §P2/ACP).
- Refute: is A2A vaporware? Two LF press releases + dated tech-update + active spec repo → corroborated. PASS.

### D2. MCP remains the tool/data protocol layer (conformance suites maturing). [MED · carried]
- MCP (modelcontextprotocol) is the tool/data protocol; roadmap emphasizes conformance test suites + SDK
  tiers. Complements (does not replace) A2A's agent-to-agent layer. Retained from prior cycle —
  `plan-architecture-loop-distributed-compute-2026-06.md`. Source:
  https://github.com/modelcontextprotocol/modelcontextprotocol (accessed 2026-06-26). No supersession.

---

## E. Recency ledger

| # | Finding | Best source date | Window |
|---|---------|------------------|--------|
| A1 clap 4.6.1 current | latest on crates.io | 2026-06-26 access | in-window |
| A2 ratatui 0.30.x current | ratatui.rs highlights | 2026-06-26 access | in-window |
| A4 serde_yaml deprecated/residual | advisory-db #2132 | 2024 (status current) | older-flagged |
| A5 serde_norway watch | rust-lang forum thread | 2026 | in-window |
| A6 tokio 1.52.3 unaffected | RustSec advisories | 2026 | in-window |
| B1 OpenSpec intent-driven template | intent-driven.dev | 2026-05-10 | in-window |
| B1 OpenSpec 1.0 baseline | OpenSpec release | 2026-01-26 | older-flagged (baseline) |
| B2 spec-kit v0.11.0 | spec-kit docs/release | 2026-05-27 | in-window |
| C1 Rust agent ecosystem | Zylos research | 2026-04-01 | in-window |
| D1 A2A v1.0 / v0.3 | LF press + DEV update | 2026-03-14 | in-window (edge) |

Counts: **in-window: 8** · **flagged-older (still current): 2** (serde_yaml unmaintained status;
OpenSpec 1.0 baseline). Carried-forward from prior cycles (not re-dated as new): C2 (TDP/MS Agent
Framework), D2 (MCP) — see prior notes, no supersession.

## F. Gaps / could-not-corroborate
- Exact GA dates for ratatui 0.30.0 and clap 4.6.0/4.6.1 were not pinned to a single primary
  changelog date (RustSec clap package page 404'd; crates.io version page not fetched). Currency
  (these ARE the latest published) is HIGH; precise release dates are LOW — not load-bearing for the plan.
- A7 crates (crossterm/toml/anyhow/comrak/minijinja): "no advisory" is absence-of-evidence from a
  general RustSec search, not a per-crate audit — rely on CI `cargo audit`/`cargo deny` for authority.

---

## G. Sources

Machine-readable ledger: `research/sources-rusty-idd.jsonl` (one JSON object per cited source, with
url / title / publisher / accessed_at / published_at / in_recency_window / why_used / claim_ids).
Summary of the load-bearing sources below (claim ids reference §A–§D and §E):

| Claim | Source URL | Publisher | Published / accessed | In-window |
|-------|-----------|-----------|----------------------|-----------|
| A1 clap current | https://crates.io/crates/clap | crates.io | accessed 2026-06-26 | yes |
| A2 ratatui current | https://ratatui.rs/highlights/v030/ , /v0301/ | Ratatui | accessed 2026-06-26 | yes |
| A4 serde_yaml deprecated | https://github.com/rustsec/advisory-db/issues/2132 | RustSec | 2024 (status current) | no (flagged) |
| A5 yaml successors | https://users.rust-lang.org/t/serde-yaml-deprecation-alternatives/108868 | rust-lang forum | 2026 | yes |
| A6 tokio unaffected | https://rustsec.org/advisories/RUSTSEC-2026-0057 ; /RUSTSEC-2026-0060.html | RustSec | 2026 | yes |
| B1 OpenSpec | https://intent-driven.dev/blog/2026/05/10/spec-driven-development-openspec-opencode/ ; https://github.com/Fission-AI/OpenSpec | intent-driven.dev / Fission-AI | 2026-05-10 / 2026-01-26 | yes / no(baseline) |
| B2 spec-kit | https://github.com/github/spec-kit ; https://github.github.com/spec-kit/ | GitHub | 2026-05-27 | yes |
| C1 Rust agent frameworks | https://zylos.ai/research/2026-04-01-rust-native-ai-agent-frameworks-ecosystem-2026/ | Zylos Research | 2026-04-01 | yes |
| D1 A2A v1.0 | https://dev.to/eclaw/a2a-protocol-tech-update-20260314-5emf ; https://github.com/a2aproject/A2A | DEV / a2aproject (LF) | 2026-03-14 | yes |
| D2 MCP | https://github.com/modelcontextprotocol/modelcontextprotocol | Anthropic/MCP | accessed 2026-06-26 | carried |
