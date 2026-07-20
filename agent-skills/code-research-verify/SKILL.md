---
name: code-research-verify
description: >-
  How to adversarially verify a code-analysis claim — try to REFUTE it against the actual source and,
  where ambiguous, by running the code. ALWAYS use before a claim becomes a report fact, on "verify
  this claim", "is that actually true", "check it against the code", "prove/disprove". Find the
  counter-example; default-skeptical; fail-closed. Confirmed-by-evidence vs plausible-but-wrong.
---

# Code-Research Verify

The gate that keeps wrong conclusions out of the report. For each claim, **assume it's false and try
to prove it** from the code. Used by `plan-verifier` (and, in the source `code-research` harness, by
`code-research-verifier`). (The adversarial-verify pattern from deep web research, applied to code.)

## Method (refute, don't confirm)

1. **Open the cited evidence** and check it actually supports the claim — read the code, not the
   analyst's paraphrase.
2. **Hunt a counter-example:** a branch/early-return that contradicts it, a caller that uses it
   differently (`git-kb code callers`), a config/flag that disables it, a `todo!()`/stub where the
   claim implies a working feature, a test that asserts the opposite.
3. **Run it when static reading is ambiguous** — execute the function/CLI/endpoint over inputs that
   would expose the claim as false; the target's own tests are a fast oracle. "Exists/compiles" ≠
   "behaves as claimed."
4. **Verdict:** `CONFIRMED` · `REFUTED` (+counter-evidence) · `QUALIFIED` (true under a condition) ·
   `INCONCLUSIVE` (couldn't establish/run). Only CONFIRMED/QUALIFIED reach the report.

## Discipline
- **Cross-boundary, not existence** — compare the claim against the code's real behavior, both sides.
- **Counter-example beats argument** — one contradicting branch refutes a general claim.
- **Default skeptical / fail-closed** — uncertain ⇒ unconfirmed; never upgrade INCONCLUSIVE to fact.
- **Doc-vs-code overclaims are prize findings** — "README says it manages a fleet" + "no wiring" =
  REFUTED, reported as a finding.
