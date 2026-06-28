# Dimension ledger (verifier-gated)

Legend: [ ] todo  [~] analysed-not-verified  [x] verified  [!] blocked

## prompt-hub

Seeded by plan-cartographer cycle 6 (graph snapshot @f826ea33). Dependency-ordered: architecture
first (frames the rest), then the front-door/goal-artifact seam (the cycle's load-bearing question),
then data-flow/contracts, then quality/perf/governance. Each row is an analyst work item.

- [x] architecture — 3-member layering (CLI/server → core), the 70-module flat core + `PromptHub` 183-method facade; is the wide-facade/wide-feature-matrix structure sound or a god-object risk? (graph: hub.rs centrality, clean DAG, strict layering) [verified: verdicts.md VERDICTs 1,5]
- [x] front-door-and-goal-artifact-seam — VERIFY the headline gap: the ADR-0007 goal-artifact emission to rusty-idd exists only in docs (`docs/plans/lifeos-meta-front-door.md`), not code; map what `process_input`/`Intent`/`Artifact`/`generate_bundle`/`junie` actually do vs. what ADR-0007 requires; scope the build-gap. (the cycle's central question) [verified: verdicts.md VERDICT 1 — CONFIRMED doc-only + RED probe]
- [ ] public-api-contracts — 1,405 public src symbols incl. 183 `PromptHub` methods + 111 HTTP routes + ~41 CLI verbs; is the surface intentional/stable, or over-exposed? OpenAPI ↔ route parity.
- [x] data-flow — UserInput→Intent→vibe→Artifact path; libsql `Storage` (acquire/insert_prompt hotspots), lineage/diff/rollback lifecycle, audit/SOC2 trail; where does provenance need to attach for the rusty-idd seam? [verified: verdicts.md VERDICTs 1,5]
- [ ] hotspots-coupling — blast-radius of `PromptHub::lock`(76), `Storage::acquire`(41)/`insert_prompt`, `HubConfig::load`(45), `PromptSanitizer::sanitize`, `FallbackChain::execute`; trait seams (Plugin/SearchEngine/FallbackStrategy/TemplateEngine) as extension points.
- [x] governance-security — RBAC (`auth.rs` Capability/Action), audit/SOC2, sanitize/moderation/malware/privacy/sandbox; argon2 + libsql RUSTSEC-avoidance feature trimming; the ~35-feature matrix per front-end as a config/governance risk. [verified: verdicts.md VERDICT 4 — PR-diff injection + .db tracking reconciled]
- [~] perf — libsql local single-writer contention (`Storage::acquire`/`lock`), hybrid search (`search.rs`/qdrant), tokenization (tiktoken/tokenizers), metrics collector fan-out; async/tokio surface. [partial: verdicts.md VERDICT 5 — shared-conn/serialization CONFIRMED; no contention benchmark]
- [ ] dead-code — 416 git-kb NoCallers candidates (LOWER-BOUND-inflated by the empty-edge-table + trait-object usage); triage real dead vs. resolver false-positives before any removal.
- [~] tooling-currency — axum 0.8.8 / tower / libsql 0.9 / clap 4.6 / argon2 0.5.3 / tiktoken / qdrant currency + advisories (feeds architect tool-eval; pairs with trend-researcher). [partial: verdicts.md VERDICT 7 — rustls QUALIFIED (lock=0.23.40, not 15-behind) + libsql/prometheus trims; axum/tower/clap/argon2/tiktoken/qdrant NOT checked]
- [x] test-coverage — findings/test-strategy-prompt-hub.md; RED suite authored+run (tests-ran: 7, all RED for contract-absent), commit 6fa3462 on plan/prompt-hub-red-tests [verified: verdicts.md VERDICT 3 — 7-RED probe + orphaned-root CONFIRMED]
