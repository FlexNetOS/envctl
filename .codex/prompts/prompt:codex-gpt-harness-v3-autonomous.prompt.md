# CODEX HARNESS V3 AUTONOMOUS RECOVERY WRAPPER

Purpose: make the Codex harness prompt executable instead of self-deadlocking.
This wrapper is additive. It does not delete or weaken the harness intent:
archive-first, Nix ownership, real proof, Rust durable logic, secrets safety,
and containment-before-capability remain mandatory.

## Operator intent

The operator wants the harness finished, not another loop at Phase 0. When the
old prompt conflicts with this wrapper, this wrapper resolves execution order so
work can continue safely.

## Phase -1: deadlock breaker

Enter Phase -1 before Phase 0 whenever any of these are true:

- local shell/sandbox cannot start;
- the prompt asks both for no human approval and for a human approval gate;
- the prompt requires subagents before the subagent fabric exists;
- Codex must be updated but Nix ownership cannot yet be proven;
- a read-only rule prevents the only narrow repair needed to make read-only
  proof possible.

Phase -1 is not a bypass. It is a constrained repair lane.

Allowed Phase -1 work:

- classify the deadlock with exact observed command/error text;
- repair the prompt/control contract additively;
- use official docs and read-only connector/subagent tools;
- create new additive prompt/control files;
- make the smallest archive-first host/runtime repair needed to restore shell
  bootstrap, when filesystem execution is otherwise impossible;
- record unresolved risks as build requirements rather than stopping the whole
  harness.

Forbidden Phase -1 work:

- read secrets;
- delete user data;
- use yolo/danger-full-access as normal operation;
- install Codex through npm, curl, pip, Homebrew, or ad hoc binary paths;
- claim Phase 0, Phase 1, or Phase 11 complete without actual proof.

## Autonomous mode

If the operator explicitly requests no human approval/no human review:

- replace approval questions with ledgered decision ids;
- keep dangerous actions blocked unless the decision id explicitly covers them;
- continue safe, non-destructive, additive work instead of stopping at approval
  scaffolds;
- do not ask `Approve Phase 1 build exactly as planned?` in autonomous mode.

Default autonomous decision ids:

- `AUTO-RECOVERY-PHASE-1-PLAN`: approve building containment artifacts only;
- `AUTO-DOCS-RESEARCH`: approve official/primary docs research;
- `AUTO-ADD-SAFE-FILES`: approve creating new additive harness files;
- `AUTO-NO-DESTRUCTIVE-GIT`: deny destructive git and force-push operations;
- `AUTO-NO-SECRETS`: deny secret reads, prints, hashes, or summaries;
- `AUTO-NO-YOLO`: deny yolo/danger/full-access.

## Operator permission grant reconciliation

Operator-granted filesystem, network, or sandbox permissions are not a failure
condition by themselves.

Do not report "blocked because too much access was granted." Treat an expanded
permission profile as execution context and continue with the narrowest safe
operation available.

Differentiate these cases:

1. `operator_grant`: the host or operator made more paths, network, or write
   roots available. This is allowed execution context. Use it to make progress.
2. `agent_bypass_request`: the agent asks to bypass approvals, sandboxing,
   hooks, policy, or secret controls. This remains forbidden unless a specific
   break-glass decision id covers that action.
3. `dangerous_command`: a concrete command would destroy data, read secrets,
   force-push, install Codex outside Nix ownership, or run uncontrolled
   background/model-provider sessions. Block or route it through the harness.

When broad access is available, the conductor must:

- keep secret-deny rules active;
- archive before modifying existing files;
- prefer additive new files over rewrites;
- run the requested work instead of stopping at a safety lecture;
- record any residual risk as a ledger row or final note, not as a reason to
  abandon the task.

A permission grant only becomes a block if the next required action is itself
forbidden, such as reading a denied secret path, deleting user data, or
installing Codex through a non-Nix owner.

## Bootstrap fallback

Subagent-mandatory execution starts after containment and the model-router gate
exist. Before that point, the conductor may directly perform:

- local bootstrap checks;
- docs research;
- repo inventory;
- prompt repair;
- minimal Rust workspace scaffolding;
- containment tests needed to prove subagent safety.

If orchestrator subagents are available before native Codex subagents are
verified, they may be used for read-only research. Native Codex subagent proof
remains a required acceptance item, not a Phase 0 deadlock.

## Shell/sandbox failure rule

If every shell command fails before startup because the sandbox builder cannot
scan an unreadable path, classify it as host-runtime blockage.

Do not repeatedly run the same failing command. Do one retest, then either:

1. make a narrow additive ignore/exclude repair if the environment permits
   writes; or
2. continue through connector/subagent/doc lanes and mark local terminal proof
   as blocked by exact error.

The known failure pattern this wrapper fixes is:

```text
error building bubblewrap command: Fatal error: ripgrep unreadable glob scan failed
```

## Codex update order

Do not update Codex before proving the active Codex path and Nix ownership.

If the operator says "update first", interpret it as:

1. prove current `codex` path and Nix ownership;
2. identify the Nix-owned update mechanism;
3. update through that owner only;
4. prove no non-Nix shadow precedes it in PATH.

If local proof is blocked, record update as blocked and continue with prompt and
harness work that does not require installing or upgrading Codex.

## Phase 0 revised gate

Phase 0 succeeds only when local bootstrap proof is available.

If local bootstrap proof is blocked, Phase 0 may produce a partial research
ledger and a blocker ledger, but it must not block additive prompt/harness
repair that is explicitly intended to unblock Phase 0.

## Acceptance rule

Do not print an acceptance matrix as passed unless commands actually ran.
When commands are blocked, print the matrix with `BLOCKED` rows and continue
with the next safe repair step instead of stopping indefinitely.

## Practical next step

When this wrapper is active on the current host:

1. stop trying to complete the old prompt unchanged;
2. keep the old prompt as historical input;
3. use this wrapper as the execution controller;
4. build the harness in small additive Rust-first slices;
5. run every available proof command when shell execution is restored.
