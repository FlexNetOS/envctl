# Dimension ledger (cartographer-owned, verifier-gated)

Per-target dimensions for cycle 1. `- [ ]` open · `- [~]` analysed-not-verified · `- [x]` verified · `- [!]` blocked.

## rusty-idd
- [x] rusty-idd/architecture
- [x] rusty-idd/code-quality (verified via verdicts.md: C3/C4/C5 god-files + C8 dead-code; architecture findings scope bundles code-quality)
- [x] rusty-idd/correctness (verified via verdicts.md: ts-25/ts-26 silent-accept defect + ts-28 no card validation)
- [~] rusty-idd/performance (NOT flipped — fail-closed: only perf-adjacent verdict U4 is QUALIFIED with "magnitude must be measured, not asserted"; no measured build-time/binary-size/runtime delta confirmed)
- [x] rusty-idd/test-coverage (verified via verdicts.md: ts-24..ts-28)
- [x] rusty-idd/governance-config (verified via verdicts.md: gov-001/gov-002/gov-003)
- [x] rusty-idd/filesystem-layout (verified via verdicts.md: C6 config-orphan + FL-3 + C3/C4/C5 LOC)
- [x] rusty-idd/memory-vector-intelligence (verified via verdicts.md: mem(.kb) absent + C11 no fabric lib deps)
- [~] rusty-idd/autoresearch (NOT flipped — fail-closed: no autoresearch CLAIM row gated; C14/C15 are trend-researcher-sourced currency facts, not the dimension's recency/source-ledger/contradiction claims)
- [~] rusty-idd/rules-policy-org (NOT flipped — fail-closed: only CLAIM-3 agent-guard advisory-mode overlaps via gov-002; CLAIM-1 Upgrade-Only law, CLAIM-4 org chart, CLAIM-5 model tiering, CLAIM-6 missing roles ungated)
- [x] rusty-idd/distributed-compute (verified via verdicts.md: dc no-C boundary + DC-2/DC-5 feasibility)
- [~] rusty-idd/prompt-architecture (NOT flipped — fail-closed: only harness hook-drift overlaps via gov-001; tool-grant/model-lane/hidden-coupling CLAIM rows ungated)

> Marks flip to `- [x]` after the verifier confirms each dimension's claims (findings/verdicts.md).
