# Harness lessons ledger (Feature Forge + ejected harnesses)

Append-only across runs. Recurrence is the point — never truncate. The evolution-steward appends
one row per mined lesson at every run boundary (DONE / HAND OFF), per the `harness-evolution`
method. Row format:

```
| date | harness | lesson (class, generalized) | evidence (cycle/finding) | recurrence | routed-to | status |
```

| date | harness | lesson (class, generalized) | evidence | recurrence | routed-to | status |
|------|---------|------------------------------|----------|------------|-----------|--------|
| 2026-06-17 | feature-forge | Verify a triggering claim that asserts concrete code state against HEAD before designing — cross-session relay/handoff claims go stale; a plan built on a false premise wastes a cycle (no-fabricate applied to inputs). | G2 #116 asserted `inject.rs`/`run_child = todo!()`, false at HEAD; orchestrator verified first (Friction-low note) | 1 | skills/feature-forge (Phase 0 step 4) | applied |
| 2026-06-17 | feature-forge | A stated gap usually implies an adjacent unstated gap — the architect must trace the full call path and fold any missing seam (missing field/hardcoded branch/unreachable variant) into the plan, not discover it at build time. | G2: request named native minting but `MintReq` lacked `mode` ⇒ `NativeSubtoken` unreachable via `Mint`; architect folded fix into U4 (01_architect_plan.md U4) | 1 | agents/feature-architect (working principles) | applied |
| 2026-06-17 | feature-forge | Sync-engine→async-daemon I/O has a fixed envctl idiom (off-reactor `block_on` via captured `Handle` inside `spawn_blocking`; reuse the frozen `build_upstream_client`, add no dep; key-free error strings) — capture it so it isn't re-invented per feature. | G2 R1 (load-bearing), implementer `DaemonHttpTransport` (02_implementer_log.md; transport.rs) | 1 | skills/rust-feature-impl (new "daemon seam idiom" section) | applied |
| 2026-06-17 | feature-forge | The guardian must classify clippy findings by BOTH axis (gate form vs `--all-targets`) and origin (touched vs untouched file): `--all-targets`-only in an untouched file = inherited red (NOTE, with `git diff` proof, never silently "fixed"); same axis in a touched file = a real finding to fix. Never relax the gate to clear red. | G2: `gui/main.rs:1997` fired only under `--all-targets` in an untouched file → correctly NOTE not blocker (03_guardian_report.md Finding #1) | 1 | skills/rust-feature-impl/references/verification.md (§0) | applied |
| 2026-06-17 | feature-forge | Phase-1.5 routing counts *independent* modules, not raw modules — a strict dependency chain (parallelism 0) stays sequential even when n>3; honor the architect's explicit routing recommendation over the raw count. | G2: 6 modules but strict U1→U6 chain ⇒ correctly sequential (Phase 1.5; 01_architect_plan.md `## Target repos`) | 1 | skills/feature-forge (Phase 1.5) | applied |
