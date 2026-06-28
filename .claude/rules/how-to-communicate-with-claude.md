# How to Communicate with Claude

This policy exists to prevent completion-by-narration, scope inflation, and retry spirals. It defines how a human or coordinating agent should ask Claude to do work and how Claude must prove completion.

## 1. Demand raw evidence, not summaries

Make Claude show raw evidence, never only Claude's summary.

Required prompts:

- "Paste the actual output."
- "Prove it ran."
- "Show the artifact, not your narration."

A claim only counts when backed by the actual command output, message id, PR URL, diff, receipt, log line, screenshot, or other directly observable artifact.

## 2. Define done as something observable

Do not accept vague completion criteria.

Examples:

- Not: "send the message."
- Done: "the recipient replies" or "the delivery receipt/message id is shown."
- Not: "it is fixed."
- Done: "this command exits 0 and the raw output is shown."

If the success criterion is checkable, Claude must check it and show the evidence.

## 3. Use one literal action with tight scope

When the task must stay narrow, say exactly that:

- "Run exactly this."
- "Touch nothing else."
- "No upgrades."
- "Do not improve adjacent systems."

This blocks the reflex to interpret a simple request as a larger excellence pass.

## 4. Make "I do not know" the expected answer

If Claude is not sure, the correct answer is:

> I do not know yet.

Guessing is the failure. Fabricated certainty, invented status, and confident extrapolation are worse than saying what is unknown and naming the next evidence-gathering step.

## 5. On failure: stop, do not retry

If a requested action fails once, stop and show the raw error. Do not silently retry, send duplicates, widen the task, or explain around the failure.

Required behavior:

1. Stop on the first failure.
2. Paste the actual error/output.
3. State what did and did not happen.
4. Wait for the next instruction unless the original instruction explicitly allowed retries.

## 6. Encode enforcement in rules and hooks, not goodwill

Do not rely on Claude remembering this policy. Durable enforcement belongs in checked-in rules, harness policy, and hooks.

Enforceable rule:

> Any claim of completion must include raw evidence for the observable done criterion.

Settings/hooks should prefer fail-closed checks where possible: block or flag final responses that claim work is done without a raw artifact, command output, receipt, or explicit uncertainty.
