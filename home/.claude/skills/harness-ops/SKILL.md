---
name: harness-ops
description: FlexNetOS harness runbooks - team spawn/cleanup, kill switch, budget check, recovery, sandbox enablement. Use when operating the parallel-execution fabric or diagnosing harness state.
---

# harness-ops — operational runbooks

## Budget check (before any fan-out)
```bash
cat $HARNESS_VAR/lib/claude-harness/rate-limits.json   # cached by statusline
cat $HARNESS_VAR/lib/claude-harness/active-agents.count
ls $HARNESS_VAR/lib/claude-harness/budget-block.flag 2>/dev/null && echo BLOCKED
```
Ceiling: 80% of five-hour or seven-day window → spawns hook-denied. Warns at each 25% step.
Print a cost line (statusline cost + rate %) at every team spawn and every 25% step.

## Team spawn (Phase-3 discipline)
1. Budget check first (above). 2. 3–5 fable teammates max, 5–6 tasks each, explicit names.
2. Strict file-ownership partitioning — every task carries an owner (TaskCreated hook rejects ownerless tasks while a team is live).
3. Plan-approval required for any teammate that writes. TeammateIdle gate blocks idling with in_progress tasks.
4. Teammates cannot spawn agents (hook + platform). One team per session.

## Team cleanup / verification
```bash
tmux ls                                    # team panes gone?
ls ~/.claude/teams/ ~/.claude/tasks/       # config auto-removed on exit; tasks may persist
```
Orphans: `tmux kill-session -t <name>`; archive stray team/task state (never rm).

## Kill switch
```bash
~/FlexNetOS/src/envctl/home/bin/harness-halt.sh
```
Stops team tmux sessions, dispatched supervisor jobs (~/.claude/jobs), background bash children, prunes worktrees, ledgers the halt.

## STOP-for-decision
```bash
echo "<the one question>" > $HARNESS_VAR/lib/claude-harness/decisions/<slug>.pending
```
Stop hook blocks completion until it is renamed `.answered`. Ask via AskUserQuestion; never loop.

## Recovery after a crash / runaway
1. `harness-halt.sh` 2. Check ledger tail: `tail -50 $HARNESS_VAR/log/claude-harness/ledger.jsonl`
3. Reset counter if stale: `printf '0' > $HARNESS_VAR/lib/claude-harness/active-agents.count`
4. Clear budget flag ONLY on operator instruction: `rm $HARNESS_VAR/lib/claude-harness/budget-block.flag`

## Sandbox enablement (deferred 2026-07-07, operator decision)
Blocked by AppArmor userns restriction. To enable later:
1. Install `/etc/apparmor.d/bwrap` profile targeting the Nix bwrap path glob (`/nix/store/*-bubblewrap-*/bin/bwrap`), `sudo systemctl reload apparmor`.
2. Set `sandbox.enabled=true` in envctl home/.claude/settings.json.tmpl (worktree flow), add `excludedCommands` for nvidia/cuda/nix builds.
3. Re-render, re-link, re-run the P1.6 containment drill.

## Model routing
Everything on Fable. Reroute-to-Opus → statusline shows ⚠ REROUTED + desktop alert → ask the operator (options: continue on Opus for flagged work / return via /model fable). Routing changes are never silent.
