# prompt-architecture — prompt-hub (TARGET; the STORE half of the two-layer intent front door)

Axis: **prompt-architecture**. Scope: prompt_hub as a prompt-architecture system and its seam to
harness_hub (interpreter) and rusty-idd (lifecycle engine). Read-only review.
(Materialized by the orchestrator from the prompt-architecture auditor's returned findings — the
sub-agent returned text instead of writing this file; content is its evidence verbatim-in-substance.)

## 0. Headline
prompt_hub is a real prompt-architecture store (typed prompt/lineage/RBAC/audit model + CLI/HTTP/plugin/
library **tool grants**) — but the interpreter↔store↔rusty-idd seam it is assigned in the LifeOS
front-door plan exists ONLY as prose: there is **no typed goal-artifact contract in code**, and the
plan's "(ADR-0007)" citation mis-resolves (prompt_hub's ADR-0007 is "Plugin System"; the boundary ADR
lives in rusty-idd, not here). **model lanes** are split three ways with no governing policy and a
drifting model id.

## 1. Instruction surfaces
- CLAIM: Rust workspace is the declared single source of truth; prose harness files are advisory + drift-flagged | CLAUDE.md:9-25 | high
- CLAIM: Multi-tool instruction surface — Copilot (.instructions.md/.prompt.md), generic agent (.agent.md:4-13,24-33), Gemini (GEMINI.md), Junie (.junie/), worktree-per-agent registry Alpha–Theta (AGENTS.md) | high
- CLAIM: Executable prompts = prompts/*.prompt.yml (GitHub Models format: model/messages/testData/evaluators) | prompts/code-review-rust.prompt.yml:1-93 | high
- CLAIM: In-product instruction surface — stored prompts/templates seeded from defaults.rs/templates.rs into a libsql store | prompt-hub/src/defaults.rs | high
- CLAIM: Harness control plane under .claude/ (skills prompt-loop/feature-build/session-relay/harness-evolution + evolution-steward), defaults to APPLY (push→PR→auto-merge) | CLAUDE.md:104-147 | high
- CLAIM: ADR set is 8 records (0001-why-sqlite … 0008-vibe-coding-architecture); NONE document the cross-repo intent seam | docs/adr/ | high

## 2. Tools granted / tool grants
- CLAIM: prompt_hub exposes NO MCP server in its own source (all mcp hits are vendored); as the STORE it grants tools via CLI + HTTP + library; A2A/MCP is routed through weave, not prompt_hub | grep mcp over prompt-hub/src → vendor only | high
- CLAIM: CLI grant = mutating set (add/import/export/deploy/evolve/rollback/plugin/vibe/gather/budget/cost…) | prompthub/src/commands/ | high
- CLAIM: HTTP grant = very large Axum route table (routes.rs ≈194 KB) + 67 KB OpenAPI — wide mutating network surface for a "store" | prompthub-server/src/routes.rs | high
- CLAIM: Plugin grant = dynamic native-code loading via libloading + inventory (catch_unwind isolation) — arbitrary-.so execution beside the crate's #![forbid(unsafe_code)] (the guarantee does NOT extend to loaded objects) | docs/adr/0007-plugin-system.md, prompt-hub/src/plugins.rs | high
- CLAIM: Outbound grants = local_llm HTTP client (Ollama/llamafile/whisper.cpp) + multi_provider egress (OpenAI/Anthropic) | prompt-hub/src/local_llm/mod.rs:1-12, multi_provider.rs:14-15 | high
- CLAIM: CI grants external-API egress with secrets — Anthropic (ANTHROPIC_API_KEY), Devin.ai (DEVIN_API_KEY) over curl, GitHub Models gated by models:read + GITHUB_TOKEN | .github/workflows/external-ai-apis.yml:57,129 | high

## 3. Model lanes
- CLAIM: Two divergent prompt.yml model lanes — task prompts pin openai/gpt-4o; fleet planning loops pin anthropic/claude-opus-4-8 | prompts/code-review-rust.prompt.yml:5, prompts/planning-engineer-loop.prompt.yml:22 | high
- CLAIM: CI multi-model-evaluation = 4-model fan-out (gpt-4o, claude/claude-opus, gpt-4o "codex", deepseek/deepseek-coder) via GitHub Models | .github/workflows/multi-model-evaluation.yml:71,101,131,161 | high
- CLAIM: Runtime lane = MultiProviderRouter over Vendor::{OpenAi,Anthropic} with health failover + RoutingStrategy::ProviderOverride | prompt-hub/src/multi_provider.rs:14-15,128-144 | high
- CLAIM: Model-id drift — claude/claude-opus (CI) vs anthropic/claude-opus-4-8 (prompt.yml) vs runtime free-text anthropic; no canonical model-id registry; CI id likely stale for GitHub Models | medium
- CLAIM: No-downgrade rule asserted at plan level, not encoded in any model-routing config | docs/plans/lifeos-meta-front-door.md:24-25 | high

## 4. Hidden architectural couplings (the convergence core)
- CLAIM: process_input produces an internal `Intent` from multimodal input — it is NOT the rusty-idd goal/spec artifact; prompt_hub's typed Intent and rusty-idd's OpenSpec goal are UNBRIDGED in code | prompt-hub/src/hub.rs:1385-1404 | high
- CLAIM: The interpreter↔store↔engine seam and the goal-artifact "envelope (source citations + test traceability + resume packet)" exist only as prose + a seed prompt.yml — no serializer, schema, or typed contract in prompt-hub/src | docs/plans/lifeos-meta-front-door.md:30-37,123,147; absence in src | high
- CLAIM: Plan's "(ADR-0007)" authority for "prompt_hub = durable intent store/boundary" is a dangling/mis-resolved citation — prompt_hub ADR-0007 is "Plugin System"; the boundary ADR lives in rusty-idd | docs/plans/lifeos-meta-front-door.md:36 vs docs/adr/0007-plugin-system.md | high
- CLAIM: swarm.rs (bundle/handoff/consistency) + sync.rs (WebSocket/file-watch/split-brain) imply distributed multi-client state beyond a single-store boundary | medium

## 5. Governance controls
- CLAIM: RBAC = Capability::{Read,Write,Admin,SwarmOnly} + authorize_action; Admin subsumes all | prompt-hub/src/auth.rs:18,86-110 | high
- CLAIM: Versioning/lineage = ancestry graph w/ fork detection (LineageTracker/LineageNode/Fork) | prompt-hub/src/lineage.rs:7-44 | high
- CLAIM: Audit/safety = tamper-evident audit.rs, sanitize.rs (prompt-injection), privacy.rs (PII) on mutate path (sanitize→authorize→mutate→audit→sync→metrics) | CLAUDE.md:77 | high

## 6. UPGRADE rows (prompt-architecture axis)
- UPGRADE: Add a typed goal-artifact contract + serializer (harness_hub→prompt_hub→rusty-idd envelope: intent, source citations, test-traceability, resume-packet pointer) in prompt-hub/src so the seam is code, not prose | axis: prompt-architecture | rationale: the store's core assigned job has no executable contract | evidence: hub.rs:1385-1404; plan:123,147 | blast: new module + hub method + CLI/HTTP; cross-repo (rusty-idd) | risk: med
- UPGRADE: Fix the dangling "(ADR-0007)" seam citation and write a real ADR for prompt_hub-as-intent-store/boundary | axis: prompt-architecture | rationale: a cross-repo plan cites a governing decision that does not exist here | evidence: plan:36 | risk: low
- UPGRADE: Canonical model-lane policy + model-id registry (reconcile claude/claude-opus ↔ anthropic/claude-opus-4-8 ↔ runtime anthropic; encode no-downgrade) | axis: prompt-architecture | evidence: multi-model-evaluation.yml:101; multi_provider.rs:14-15 | risk: low
- UPGRADE: Document the plugin native-code trust boundary vs #![forbid(unsafe_code)] (loaded .so are outside the guarantee) | axis: prompt-architecture | evidence: docs/adr/0007-plugin-system.md; CLAUDE.md:83 | risk: low

## 7. ADR candidates / no-ADR rationale
- ADR-CANDIDATE: Goal-artifact emission format (typed envelope schema + version) prompt_hub → rusty-idd — a cross-repo runtime contract between two authoritative engines, currently prose-only.
- ADR-CANDIDATE: Two-layer intent front-door seam (harness_hub interpreter ↔ prompt_hub store) — defines cross-repo ownership; asserted by owner D3 but no ADR records it in prompt_hub.
- ADR-CANDIDATE: Model-lane routing policy (provider-per-lane, model-id canonicalization, no-downgrade enforcement).
- ADR-CANDIDATE: Plugin native-code trust boundary vs unsafe-code guarantee (amend ADR-0007).
- NO-ADR: individual prompts/*.prompt.yml task prompts; new CLI subcommands + sequential migrations; multi-runtime advisory instruction files (CLAUDE.md governs them as advisory).
