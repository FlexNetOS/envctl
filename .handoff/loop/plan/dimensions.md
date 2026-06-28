# Dimension ledger (parallel instance: weave). [ ] open · [~] analysed · [x] verified · [!] blocked.

## weave
- [x] weave/architecture — verdicts.md "## weave (cycle 4)": 12 ARCH rows gated (9 CONFIRMED, 3 QUALIFIED), 4 UPGRADEs feasible
- [~] weave/code-quality — no dedicated `code-quality-weave.md`; only the `main.rs` god-file slice is covered (ARCH-07/U-ARCH-3). Missing: full quality analysis to gate.
- [~] weave/correctness — no dedicated `correctness-weave.md`; `send`/dedup/parity paths spot-verified inside ARCH-05/08/11 only. Missing: correctness-axis findings to gate.
- [~] weave/performance — no dedicated `performance-weave.md`; not analysed or gated this cycle.
- [x] weave/test-coverage — verdicts.md: A2A RED suite (U-TEST) + conformance-harness absence (ARCH-11/U-ARCH-1) gated; empirical bench run.
- [x] weave/governance-config — verdicts.md: GOV-001 (PreToolUse), GOV-003 (CI 6→7), GOV-004 (Python) CONFIRMED; gov UPGRADEs feasibility-gated.
- [~] weave/filesystem-layout — `filesystem-layout-weave.md` present but not gated this pass. Missing: verdicts for layout claims.
- [x] weave/memory-vector-intelligence — verdicts.md: MEM-1/2/3 CONFIRMED (empirical: memory.rs organ, no ICM ref, no vector deps); U-MEM-1 feasible, U-MEM-2 feasibility-qualified.
- [~] weave/autoresearch — `autoresearch-weave.md` present but not gated this pass. Missing: verdicts for autoresearch claims.
- [~] weave/rules-policy-org — `rules-policy-org-weave.md` present but not gated this pass. Missing: verdicts for org/A2A-comms claims.
- [~] weave/distributed-compute — `distributed-compute-weave.md` present but not gated this pass. Missing: verdicts for distributed-compute claims.
- [x] weave/prompt-architecture — verdicts.md: PA-TOOLS (token-safe meta-tool, QUALIFIED count) + PA-METACALL (gate not bypass) CONFIRMED; meta-tool budget/gate empirically verified.
