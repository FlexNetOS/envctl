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

## handoff (cycle 2 — union with rusty-idd)
- [x] handoff/architecture (verified via verdicts.md: A-C1..A-C15 all CONFIRMED/QUALIFIED; EXP-1/2/3 empirical)
- [x] handoff/code-quality (verified via verdicts.md: A-C2 cycles=artifact, A-C11 blast hotspots, A-C15 dead-code caveat, A-C10 unsafe-deny)
- [x] handoff/correctness (verified via verdicts.md: ts-2 fail-open work-order loader + ts-RED empirically RED; A-C12 validate_card fail-closed)
- [~] handoff/performance (NOT flipped — fail-closed: only perf-adjacent verdicts mem-U2/RVF write-amp are QUALIFIED with magnitude unmeasured; no measured build-time/binary-size/runtime delta confirmed)
- [x] handoff/test-coverage (verified via verdicts.md: ts-1..ts-6 + ts-RED re-run standalone 1-pass/3-fail; RED suite committed d74ad4b)
- [x] handoff/governance-config (verified via verdicts.md: gov-001..gov-009; gov-001 fail-OPEN seam + gov-003 RuVector empirical)
- [x] handoff/filesystem-layout (verified via verdicts.md: fs-1..fs-6 incl. two analyst corrections confirmed)
- [x] handoff/memory-vector-intelligence (verified via verdicts.md: mem-1..mem-7; SHAKE-256 correction EXP-3 + dead RVF recall mem-1 empirical)
- [x] handoff/autoresearch (verified via verdicts.md: ar-1..ar-4; handoff-drift exit(1) invalidation engine verified)
- [x] handoff/rules-policy-org (verified via verdicts.md: rp-teeth exit(1) chain empirical vs rusty-idd advisory; org-chart + A2A)
- [x] handoff/distributed-compute (verified via verdicts.md: dc-1..dc-4; no-C boundary + RuVector blocker EXP-1)
- [x] handoff/prompt-architecture (verified via verdicts.md: pa-dual-front-door + pa-fork-drift empirical via cli command set)
- [x] handoff/union-with-rusty-idd (verified via verdicts.md: union-1 lineage CONFIRMED — spec byte-identical, rusty-idd superset; union-2 MERGE QUALIFIED-feasible under RuVector/dedup/no-C conditions; union-3 ledger read-API gap)

> Marks flip to `- [x]` after the verifier confirms each dimension's claims (findings/verdicts.md).
