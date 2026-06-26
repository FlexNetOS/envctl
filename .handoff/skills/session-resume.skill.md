# Skill: session-resume

## Purpose

Rehydrate current project state for a new agent session with minimal context load.

## Trigger phrases

- resume
- continue
- pick up
- what next
- recover session

## Steps

1. Run `hf import` if `.handoff/ledger.db` is absent, then `hf resume --json` from this member repo.
2. Read `.handoff/context/capsule.json` (who this repo is + next_command).
3. Read `.handoff/packets/latest.md` (compiled by `hf handoff` from this repo's imported local ledger cache).
4. Read `.handoff/loop/HANDOFF.md` + `loop_state.md` + `backlog.md` (the forge-loop cold-start package).
5. Check the latest drift report.
6. Print the exact next command.

## Hard rule

Do not edit files during this skill. Ledger-mutating verbs run in the member repo, followed by `hf export`; binary `.handoff/ledger.db` remains ignored while `.handoff/ledger.events.jsonl` is committed (P7 / ADR-0004 §3 / ADR-0018 D1).
