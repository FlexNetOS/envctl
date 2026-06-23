# .handoff — continuity layer (full)

This repo is a member of the FlexNetOS meta workspace. This directory is its continuity layer
(META-ORG-POLICY.md **P7**; design: handoff ADR-0003 + ADR-0004).

- `context/capsule.json` — who this repo is and what's next (census-derived; keep accurate).
- State precedence: **Git > committed JSONL ledger export > local ledger cache > task cards**.
  `.handoff/ledger.events.jsonl` is the committed continuity truth (ADR-0018 D1). The local
  `.handoff/ledger.db` redb file and RVF sidecars are ignored rebuild caches; reconstruct them with
  `hf import` (or the source binary command in `loop/HANDOFF.md`) before running drift/dependency
  checks in a cold worktree. Never commit binary ledger files.
- `tasks/` — execution cards minted from kb planning tasks (`hf task mint --from-kb`, ADR-0003). Empty until
  kb task docs exist for envctl; the packet degrades to "(no open cards)".
- `packets/latest.md` — resume packet compiled by `hf handoff` from the local ledger cache rebuilt
  from the committed JSONL export. Rendered, never hand-written.
- `hooks/hooks.toml`, `policies/rules.toml`, `skills/` — OPTIONAL autonomous-loop descriptors (ADR-0004 §2);
  declarative text the kernel/harness reads. Ledger-mutating verbs they name run at `$META_ROOT`, never here.
- `loop/` — autonomous-loop state (the **active** agenticOS-consolidation forge-loop; Epics A–E). Migrated
  here from the deprecated `_workspace/` (HARNESS-UPGRADE-KIT v2 / ADR-0004); history preserved via `git mv`.
  Cold-start from `loop/HANDOFF.md` + `loop_state.md` + `backlog.md`.
- Planning lives on the kb board (`/kb-board`); cards here are derived views synced at checkpoint.
