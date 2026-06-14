# HANDOFF — Epic C re-port DONE + KEPT (no backtrack); session 2026-06-14

closed_utc: 2026-06-14    branch: develop    worktree: create FRESH off origin/develop

## Status: Epic C is COMPLETE and STAYS
The kasetto v3.2.0 absorption (`crates/agent-env` + `Engine::agent_*` + `envctl agent` CLI/GUI) is
**done, merged (#71–#96 → master), and no-C-clean** (mimalloc dropped → system allocator, gate
PASS). **Do not re-port it, do not adopt-as-crate, do not throw it away.** It works. Keep it.

## Two things this session got WRONG (both corrected — learn, don't repeat)
1. **Fabricated a "no-downgrade bug."** Chased "github MCP dropped from the lock" as a regression.
   It was a **MISDIAGNOSIS**: kasetto's *own* bare `lock` → `sync --locked` fails identically
   (kasetto `src/commands/lock.rs:9-11` — MCP/command hashes are filled by `sync`, not `lock`, by
   design). envctl's port was faithful. **PR #99 (rebuild_mcp_assets/rebuild_command_assets) was a
   DIVERGENCE — correctly CLOSED.** Do not re-open. LESSON: when a "port gap" appears, run the
   rust-native source (the real `kasetto` binary) on the same input FIRST; differential vs the
   binary, never vs a reread of our reimplementation.
2. **Over-corrected into backtracking.** Then proposed ripping out the 11.4k-LOC re-port to "adopt
   kasetto as a crate." That throws away done, working, merged code — as wrong as the skip. RETRACTED
   (the pivot ADR was deleted). LESSON: the easier path (adopt kasetto's crate + drop its one C dep,
   `mimalloc`) existed — but the re-port already solved it; recognizing the cheaper path is NOT a
   license to redo finished work.

## What LANDED this session (legit, merged)
- PR #97 — `docs/KASETTO-FEATURES.md` v3.0.0 → v3.2.0 + absorbed-status (TASK-0011).
- PR #98 — retire external `kasetto`/`kst` binary symlinks; `manifest/agent-env.toml` drives the
  built-in `envctl agent`; localize agent-env config (kasetto.yaml mcps in-meta, shell hardcodes,
  Documentation URL) (TASK-0006 + TASK-0018).
- TASK-0023 verified done (sync-master.yml live + green). KBTASK-SEED-UNLOCK checkpointed
  code-complete (live-hardware unlock = the ONLY owner-gated remainder).
- PR #99 CLOSED (the misdiagnosis above).

## NEXT LOOP — the actual remaining backlog (NOT an Epic C redo)
Epic C is closed. Pick up the real open work:
- Epic B portability: TASK-0007 (doctor boundary-refusal + idempotent symlink regen), TASK-0008
  (relocate meta-mcp).
- Epic C tail (genuinely remaining): TASK-0016 (lock-unification *decision* — engine FNV-1a
  `envctl.lock` vs agent SHA-256 `agent-env.lock`; current implemented state = keep separate),
  TASK-0017 (adopt kasetto `extends` for component manifests).
- Epic D: TASK-0019 (secretd RealUsbProbe), TASK-0020 (github-app-mint P0), TASK-0021 (node-via-bun),
  TASK-0022 (agent-web-access phases 2-3).

## verify_on_resume
```
cd <fresh worktree off origin/develop>
bash ci/gates/no-c.sh && bash ci/gates/shape.sh && bash ci/gates/enable.sh   # all PASS
cargo build -p envctl-engine -p envctl && cargo test -p envctl-agent-env     # green
```
resume_command: /forge-loop  (pick the next open backlog item — Epic C is DONE)
