# Agent control protocol

## Principle

Agents and humans use the same operation queue and event ledger. The difference is authority, not state.

## Actors

```text
system      envctl internal scheduler/runner
agent       Codex or other automation agent
human       shell user/operator
plugin      nu_plugin session acting on behalf of a human
external    imported package/helper process
```

## Authority levels

```text
read_only          can inspect targets, events, artifacts, and plans
safe_execute       can run read-only scanners and artifact generation
approval_request  can request approval for risky operations
operator           can approve, deny, pause, resume, or rollback
admin              can alter policies/contracts/schemas
```

## Operation risk classes

```text
R0 read-only inspection
R1 writes only to envctl artifact/evidence store
R2 modifies local working copy artifacts
R3 modifies target repository code/config
R4 modifies databases/infrastructure/secrets
R5 destructive or production-impacting operation
```

R3+ requires explicit approval unless policy has a pre-approved scoped rule. R4/R5 must default to approval-gated.

## Event append contract

All actions append events. No hidden side effects.

## Idempotency

Operations must include an idempotency key derived from:

```text
run_id + operation_type + target_descriptor_hash + recipe_step_id + input_hash
```

## Concurrency

Multiple agents may work in parallel only if operations have non-conflicting target scopes or the database obtains a lease/lock. Locks must be visible in live status.
