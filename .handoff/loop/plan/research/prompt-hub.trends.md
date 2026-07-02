# prompt-hub — best-practices + latest trends (trend-researcher, cycle 6)

- Target `T`: **prompt-hub** — Rust prompt/intent management store (core lib `prompt-hub` + CLI `prompthub` + axum HTTP server `prompthub-server`): prompt storage, search, lineage/versioning, RBAC/audit, goal-artifact emission. Role in the merged front door (prompt_hub#182, ADR-0007): the **durable, provenance-stamped intent STORE** that `rusty-idd` consumes; prompt_hub never owns rusty-idd lifecycle.
- `target_root`: /home/drdave/Desktop/meta/prompt_hub
- Today: **2026-06-27**. Recency window: **90 days → since ~2026-03-29.**
- Recency gate: every finding below is **cited + dated**. In-window sources preferred; older-but-still-current sources are flagged inline. New-but-unproven items are labelled **[trend / watch]** vs **[best-practice / safe to adopt]**.
- This note supersedes nothing target-specific (no prior `prompt-hub.trends.md` existed; the existing `agentic-planning-trends-2026-06.md` is about the planning *harness*, a different scope). Citation trail preserved in `sources-prompt-hub.jsonl`.

---

## Headline

1. The field has converged on **immutable, content-addressed prompt versions in a registry separated from application code**, with **alias-based promotion** (staging/production), **rollback by version id**, and **trace-id → prompt-version linkage** — this is now best-practice, and it maps directly onto prompt-hub's storage + lineage/versioning surface.
2. **"Intent is the source of truth"** spec-driven development (SDD) is mainstream in 2026 (GitHub Spec Kit, OpenSpec, Kiro, etc.): the durable artifact agents consume is a **specification/goal artifact**, not code — exactly prompt-hub's `intent → goal-artifact → rusty-idd` pipeline. Validates the architecture; raises the bar on provenance fidelity of the emitted artifact.
3. **Audit + RBAC are now a regulatory gate, not a nice-to-have.** EU AI Act high-risk obligations apply **2026-08-02**; Article 12 requires queryable, ≥6-month event logs and enforcement that is *inseparable from auditing*. prompt-hub's RBAC/audit must be a hard gate co-located with the mutating op, not an after-the-fact observer.
4. **Tool currency:** deps are mostly current; the standouts behind are **uuid** (1.18.0 → 1.23.4, ~5 minors), **rustls** (0.23.26 → 0.23.41, 15 patches on a TLS crate), and **tower_governor** (0.7 → 0.8.0). No *direct* advisory hits the pinned set, and the manifest's libsql/prometheus feature-trimming **provably dodges real 2026 rustls-webpki advisories + a protobuf advisory** — see below.

---

## Domain best-practices & trends

### A. Prompt/intent registries & versioning

- **[best-practice]** Prompt versions are **immutable** — once created, never mutated; a change makes a new version with a unique id. This is what makes distributed tracing reliable (a trace id maps to an exact prompt version). Corroborated across MLflow Prompt Registry docs (accessed 2026-06-27) and Braintrust "Best Prompt Versioning Tools for Production Teams" (**2026-06-21**, in-window).
- **[best-practice]** **Extract prompts out of application code into a dedicated registry** so a prompt can be hot-fixed or rolled back **without redeploying the binary**. (Braintrust, 2026-06-21; MLflow docs, accessed 2026-06-27.) → prompt-hub already *is* this registry; the lesson is to keep the store the single source and avoid in-binary prompt drift.
- **[best-practice]** **Alias-based environment promotion** — mutable named pointers (`production`, `staging`) onto immutable versions — is the standard promotion mechanism (MLflow Prompt Registry, "Manage Prompt Lifecycles with aliases", accessed 2026-06-27). → maps to prompt-hub's versioning/RBAC: promotion should be an alias re-point + audited RBAC action, not a content edit.
- **[best-practice]** **Tiered evaluation before promotion** — deterministic assertions (e.g. "output must be JSON") plus semantic checks — gate a version into `production`. (Braintrust 2026-06-21; MLflow 3.x docs accessed 2026-06-27.)
- Adversarial check: these four are each carried by ≥2 independent sources (MLflow primary docs + Braintrust), so they are facts, not single-blog claims. Git-like versioning with commit messages + rollback is explicitly an MLflow 3.x feature, not just vendor marketing.

### B. Prompt lineage / provenance

- **[best-practice]** Lineage = **provenance (origin) + transformations (what changed) + ownership (who decided)**, linked to experiments/eval results. (Atlan "Training Data Lineage for LLMs", accessed 2026-06-27; MLflow lineage docs.) → prompt-hub's lineage should stamp *who/what/when/from-which-version* on every emitted goal artifact, which is precisely what ADR-0007 provenance-stamping calls for.
- **[best-practice / now compliance-driven]** Documented provenance is **required for high-risk AI** under the EU AI Act, enforcement from **2026-08-02**. (Search synthesis over Raconteur "EU AI Act Compliance: technical audit guide for the 2026 deadline" and the prompt-versioning lineage sources, accessed 2026-06-27.) → provenance is no longer optional metadata; it's a gating artifact for any deployment that touches high-risk flows.

### C. RBAC / audit for prompt stores

- **[best-practice]** RBAC should operate at **org / project / object levels**, with SSO (OIDC/Okta/Entra/Google) and workspace separation for SOC 2. Audit must capture **all inputs, outputs, and metadata**, queryable, ≥6 months retention (EU AI Act Art. 12). (Search synthesis: Raconteur 2026 audit guide; Braintrust "Best AI governance platforms for LLM applications (2026)"; dev.to "AI agent audit trail before August 2", accessed 2026-06-27.)
- **[best-practice / 2026 sharpening]** **Enforcement and auditing cannot be separated** — a governance layer that only reads logs after the fact is already non-compliant for high-risk systems. The policy decision (allow/deny, hard-gate vs soft-gate) must be recorded *at the point of enforcement*. → for prompt-hub: the RBAC check and the audit-log write should be the **same transaction** as the mutating store op, not a downstream observer.
- Five overlapping frameworks now intersect on this: **EU AI Act (2024/1689), NIST AI RMF 1.0, ISO/IEC 42001:2023, SOC 2, GDPR** (Braintrust governance 2026; digitalapplied AI agent governance 2026, accessed 2026-06-27).

### D. Intent → spec/goal-artifact pipelines

- **[best-practice, mainstream by 2026]** Spec-Driven Development: **Specify → Plan → Tasks → Implement**, with the **specification as the durable artifact** agents consume. "We're moving from 'code is the source of truth' to 'intent is the source of truth.'" (GitHub Blog "Spec-driven development with AI", **2025-09-02** — *older than the window; flagged, but still current*: corroborated in-window by MarkTechPost "Meet GitHub Spec-Kit", **2026-05-08**, which reports Spec Kit at 90k+ stars and 30+ agent integrations by mid-2026.)
- **[trend / watch]** Every major coding-agent vendor shipped an SDD flavor by 2026 (Spec Kit, Kiro, OpenSpec, Claude Code, Cursor, Antigravity). Early-adopter reports claim **~3–10× higher first-pass success** on non-trivial tasks (GitHub/AWS, via MarkTechPost 2026-05-08 — *vendor-sourced metric, treat as signal not proof*).
- Tie-in: prompt-hub's `intent → goal-artifact → rusty-idd` is the local, governed instantiation of this industry pattern. The emitted goal artifact should carry the same spec-grade fidelity (clear intent, output contract, provenance) that SDD specs do. Note OpenSpec is already a first-class workflow in this repo (`crates/tui/.claude/skills` OpenSpec set, CLAUDE.md "bind the goal with OpenSpec artifacts") — the trend reinforces that binding.

### E. Prompt-management as agent front-door

- **[trend / watch]** The MCP ecosystem is standardizing the **front-door registry/gateway** pattern: a registry that federates servers, with **centralized discovery, guardrails, governance, and observability** (IBM ContextForge — "AI Gateway, registry, and proxy"; Official MCP Registry at registry.modelcontextprotocol.io; accessed 2026-06-27). Emerging **MCP Server Cards** (`.well-known` metadata) are on the 2026 roadmap for capability discovery without connecting.
- **[best-practice]** **Prompts are a first-class MCP context type** (alongside tools + resources) — pre-defined templates that guide workflows. But **MCP delivers integration; the prompt still does the work** (name the goal, enumerate tool applicability, define the output contract, plan recovery). (modelcontextprotocol.info prompts docs; WorkOS "Everything your team needs to know about MCP in 2026", accessed 2026-06-27; Red Hat "Building effective AI agents with MCP", **2026-01-08** — *older than window, flagged; foundational guidance still current*.)
- Tie-in: prompt-hub as the front-door intent store should treat its prompts/intents as first-class, discoverable, governed context — the harness_hub interpreter + prompt_hub store two-layer front door is the local realization of the "registry + governance + observability" gateway pattern.

---

## Tool-currency & advisories

Pinned versions from `prompt_hub/Cargo.toml` `[workspace.dependencies]`; latest stable from crates.io (accessed **2026-06-27**). Window = since ~2026-03-29.

| Crate | Pinned | Latest stable (date) | Gap | In-window release? | Assessment |
|---|---|---|---|---|---|
| axum | 0.8.8 (2025-12-20) | **0.8.9** (2026-04-14) | 1 patch | yes | Near-current; bump is low-risk. |
| tower | 0.5.2 (2024-12-11) | **0.5.3** (2026-01-12) | 1 patch | no (Jan) | One patch behind; 0.5.3 predates window — bump when convenient. |
| tower-http | 0.7.0 (2026-06-15) | **0.7.0** (2026-06-15) | none | yes | **Current — on the newest major**, released 2026-06-15. Verify the `cors/compression-full/trace/request-id` feature set still names correctly after the 0.6→0.7 jump. |
| tower_governor (`tower_governor`) | 0.7 (2025-03-17) | **0.8.0** (2025-08-14) | 1 minor | no (Aug 2025) | Behind a minor; rate-limit middleware — review 0.8 changelog for governor/key-extractor API changes before bump. |
| libsql | 0.9 (resolves 0.9.30, 2026-03-19) | **0.9.30** stable; 0.10.0-pre.4 (2026-06-02) | current in 0.9 line | yes (0.9.30) | Current within 0.9. `default-features=false, features=["core"]` (local-only) is correct and advisory-relevant (below). 0.10 is pre-release — do not chase. |
| rustls | 0.23.26 | **0.23.41** (2026-06-22) | 15 patches | yes | **Most material currency gap.** TLS crate 15 patch releases behind. No advisory found against the rustls crate proper at 0.23.x, but patch-currency on TLS is a standing risk — recommend bump to 0.23.41. |
| handlebars | 6.4.0 (2026-01-02) | **6.4.2** (2026-06-24) | 2 patches | yes | Minor patches behind; low-risk bump. (template engine actually wired in the workspace.) |
| uuid | 1.18.0 (~2025-08) | **1.23.4** (2026-06-24) | ~5 minors | yes | Notably behind (v4/v7/serde features). No advisory, but 5 minors of fixes/perf; recommend bump. |

### Advisories (RustSec / CVE)

- **No direct RUSTSEC advisory hits the pinned dependency set** as configured. The risk picture is dominated by **transitive chains the manifest already trims** — and those trims are validated by *real, recent* advisories:
  - **RUSTSEC-2026-0049** — `rustls-webpki`: CRLs not considered authoritative by Distribution Point (faulty matching). Announced **2026-03-20** (*~9 days before window; flagged older, still current*). Affected **0.102.0-alpha.0 through 0.103.9**; patched **0.103.10+**. The libsql default-feature chain the manifest drops pulls **rustls-webpki 0.102** — squarely in the affected range. The `libsql default-features=false` trim **avoids this advisory**.
  - **RUSTSEC-2026-0098** — `rustls-webpki`: name constraints for URI names incorrectly accepted. Announced **2026-04-15** (in-window). Affected **< 0.103.12** (+ some 0.104 alphas); patched **0.103.12+**. Same trimmed 0.102 chain is affected → trim **also avoids this**. (Low practical severity, but real.)
  - **RUSTSEC-2024-0437** — `protobuf`: crash via uncontrolled recursion (stack overflow on untrusted input). Announced **2024-12** (*older than window; flagged, still the live rationale*). The manifest's `prometheus default-features=false` (drops the protobuf exposition format) and the explicit decision **not** to use the discontinued `opentelemetry-prometheus` (final release 0.29) keep this unmaintained/vulnerable `protobuf` crate out of the graph. **Validated avoidance.**
- **libsql feature-trimming RUSTSEC context (as the prompt asked):** the manifest comment claims dropping `replication`/`remote`/`sync`/`tls` removes the bundled `hyper-rustls → rustls 0.22 → rustls-webpki 0.102` chain ("4 RUSTSEC advisories we don't use"). I **independently confirmed** that rustls-webpki 0.102 is in the affected range of *both* live 2026 advisories above — so the trim is not cosmetic; it removes genuinely-advised code paths the local-only store never exercises. This is a correctly-reasoned, advisory-avoiding configuration and should be **preserved** (regression-guard it: an accidental re-enable of libsql default features would re-import the advised chain).
- **Adversarial note / gap:** I could not load a single authoritative page enumerating "all 4" advisories in one place for the trimmed chain; I confirmed 2 of them (rustls-webpki) directly and the protobuf one separately. The manifest's "4 advisories" count is **plausible and partially corroborated** but not fully enumerated here — treat the exact count as the manifest author's claim, the *direction* (trim avoids real advisories) as **confirmed**. Low-confidence only on the precise number.

### Recommended currency actions (for the architect's R7 tool-eval)

1. **Bump rustls 0.23.26 → 0.23.41** (TLS patch-currency; highest-value low-risk).
2. **Bump uuid → 1.23.x** and **handlebars → 6.4.2** (low-risk patch/minor catch-up).
3. **Evaluate tower_governor 0.7 → 0.8.0** (review key-extractor/governor API changes first).
4. **Bump axum → 0.8.9, tower → 0.5.3** (trivial).
5. **Keep** `libsql default-features=false` + `prometheus default-features=false` + no `opentelemetry-prometheus`; add a deny/regression guard so the trimmed advised chains can't silently return.
6. **Re-verify tower-http 0.7.0 feature names** after the 0.6→0.7 major (just released 2026-06-15).

---

## Confidence

- Domain best-practices (A–C): **high** — each load-bearing claim is carried by ≥2 independent in-window-or-corroborated sources (MLflow primary docs + Braintrust + governance set).
- SDD / front-door trends (D–E): **medium-high** — mainstream and well-sourced; the 3–10× metric is vendor-sourced (signal).
- Tool currency: **high** — versions/dates from crates.io accessed 2026-06-27.
- Advisory avoidance: **high on direction, low on the exact "4" count.**

## Sources

| # | Source | Publisher | Published / accessed | In-window |
|---|---|---|---|---|
| S1 | crates.io API — axum | crates.io | 0.8.9 rel 2026-04-14; accessed 2026-06-27 | yes |
| S2 | crates.io API — tower | crates.io | 0.5.3 rel 2026-01-12; accessed 2026-06-27 | no (rel) |
| S3 | crates.io API — tower-http | crates.io | 0.7.0 rel 2026-06-15; accessed 2026-06-27 | yes |
| S4 | crates.io API — tower_governor | crates.io | 0.8.0 rel 2025-08-14; accessed 2026-06-27 | no (rel) |
| S5 | crates.io API — libsql | crates.io | 0.9.30 rel 2026-03-19; accessed 2026-06-27 | yes |
| S6 | crates.io API — rustls | crates.io | 0.23.41 rel 2026-06-22; accessed 2026-06-27 | yes |
| S7 | crates.io API — handlebars | crates.io | 6.4.2 rel 2026-06-24; accessed 2026-06-27 | yes |
| S8 | crates.io API — uuid | crates.io | 1.23.4 rel 2026-06-24; accessed 2026-06-27 | yes |
| S9 | RUSTSEC-2026-0098 (rustls-webpki URI name constraints) | RustSec | 2026-04-15 | yes |
| S10 | RUSTSEC-2026-0049 (rustls-webpki CRL distribution point) | RustSec | 2026-03-20 | no (flagged) |
| S11 | RUSTSEC-2024-0437 (protobuf uncontrolled recursion) | RustSec | 2024-12 | no (flagged) |
| S12 | Best Prompt Versioning Tools for Production Teams | Braintrust | 2026-06-21 | yes |
| S13 | Prompt Registry (immutability, aliases, lineage) | MLflow / Databricks docs | accessed 2026-06-27 | yes (living) |
| S14 | Spec-driven development with AI (intent is source of truth) | GitHub Blog | 2025-09-02 | no (flagged) |
| S15 | Meet GitHub Spec-Kit (90k stars, 30+ agents, mid-2026) | MarkTechPost | 2026-05-08 | yes |
| S16 | EU AI Act Compliance: technical audit guide for 2026 deadline | Raconteur | accessed 2026-06-27 (Aug-2 2026 deadline) | yes |
| S17 | Best AI governance platforms for LLM applications (2026) | Braintrust | 2026 / accessed 2026-06-27 | yes |
| S18 | AI agent audit trail before August 2 | dev.to (Ganapolsky) | accessed 2026-06-27 | yes |
| S19 | Training Data Lineage for LLMs (provenance/transforms/ownership) | Atlan | accessed 2026-06-27 | yes (living) |
| S20 | mcp-context-forge (front-door registry/gateway/governance) | IBM / GitHub | accessed 2026-06-27 | yes (living) |
| S21 | MCP prompts as first-class context; prompt still does the work | modelcontextprotocol.info / WorkOS | accessed 2026-06-27 | yes (living) |
| S22 | AI Agent Governance: Policy & Compliance 2026 (5 frameworks) | digitalapplied | 2026 / accessed 2026-06-27 | yes |

Full machine-readable ledger with URLs and claim ids: `research/sources-prompt-hub.jsonl`.
