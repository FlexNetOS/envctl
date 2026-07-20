---
name: plan-test-strategy
description: >-
  Turn a target's code graph into a TEST STRATEGY — map existing tests (by call-graph reachability,
  not file presence), compute coverage gaps (public-API / hotspots / data-flows / error-paths with no
  test caller), and DESIGN the suite that closes them (unit / integration / #[tokio::test] daemon e2e /
  differential-golden / property / fail-closed-refusal), ending in a Feature-Forge-ready test-build
  spec. ALWAYS use for the `test-coverage` dimension, on "test strategy", "what's untested", "coverage
  gaps", "design the test suite", "how do we test this upgrade", AND follow-ups — "re-run the test
  strategy", "update coverage", "redo the test plan for <target>". Read-only: it PLANS tests; Feature
  Forge builds + runs them. Used by `plan-test-strategist`.
---

# plan-test-strategy — coverage gaps + suite design (the testing component)

Produce a test strategy for a planning target that is **implementable, not aspirational**: every
"untested" finding cites the symbol that lacks a test caller, every proposed test is a concrete case,
and the output ends in a spec Feature Forge can build verbatim. The harness is **read-only** — this
method *designs* the suite; it writes and RED-runs additive test code; Feature Forge implements the production change and turns the suite GREEN, reached by
the handoff at the end).

> The exact CLAIM/UPGRADE row formats and the `## FF test-build spec` shape live in
> `planning-engineer/references/state-contract.md` and the `plan-test-strategist` agent definition —
> this file is the method.

## Why reachability, not file count

A `tests/` file existing proves nothing about what runs. Coverage here means **a test function
transitively reaches the symbol on the call graph.** This is why the method is graph-based: the
cartographer's `graph/<T>.callgraph.json` tells you which production symbols each test actually
exercises; a symbol with no test on any inbound path is a real gap regardless of how many test files
the crate has.

## Method — 4 steps

### 1. MAP existing tests (from the graph)
- `git-kb code symbols --json` over `<target_root>` → the symbol set. Test functions surface inside
  `#[cfg(test)] mod tests`, as `#[test]` / `#[tokio::test]`, and in `crates/<c>/tests/*.rs`.
- For each test fn, `git-kb code callees <test-sym> --json` (and follow transitively) → the set of
  production symbols it reaches. Union across all tests = the **covered set**.
- Prefer `git-kb code` over grep; the AST knows `mod tests` boundaries and macro-expanded calls better
  than text. (rtk gotcha applies: `rtk proxy git-kb …` or redirect for clean JSON.)

### 2. COMPUTE coverage gaps (rank by the metrics' risk signals)
Subtract the covered set from the contract-bearing symbols, ranked:
| Gap class | Source | Why it ranks high |
|-----------|--------|-------------------|
| **public-API untested** | `metrics.json` public-api ∖ covered | an unguarded external contract |
| **hotspot untested** | high centrality ∖ covered | most-depended-on, least-tested |
| **data-flow untested** | `entrypoints`/`flows` with no e2e test | the integration path is unverified |
| **error-path untested** | `Result`/`Err`/guard branches with no failure-driving test | fail-closed guards MUST be proven |
| **high-blast untested** | `impact --depth` leaders ∖ covered | risky-to-change because untested |

### 3. DESIGN the suite (right type per case)
Close each gap, and cover each CONFIRMED upgrade the other analysts proposed, with the correct type:
- **unit** (`#[cfg(test)] mod tests`) — pure logic beside the code.
- **integration** (`crates/<crate>/tests/*.rs`) — public-API / cross-module behavior.
- **daemon e2e** (`#[tokio::test]`) — the async `secretd` path.
- **differential / golden** — for any behavior-preserving upgrade (refactor, `axis: speed`): capture
  current output as a golden fixture, assert the upgrade reproduces it (the `rust-port-parity`
  discipline — reuse it, don't reinvent).
- **property** — round-trip / idempotence / monotonicity invariants.
- **fail-closed refusal** — every destructive op / guard gets a test asserting it REFUSES without
  proof (envctl `UuidResolves`/`NotLiveDevice`/`NotMounted`).

Each designed test names the exact symbol/flow it covers and the assertion — never "add coverage."

### 4. EMIT the Feature-Forge test-build spec
End `findings/test-strategy-<T>.md` with a `## FF test-build spec` block (the handoff): test surface
(files/modules), one bullet per concrete case (symbol/flow + assertion + test type), golden fixtures to
capture, the coverage target, and which CI gate(s) the new tests touch. Shape it to Feature Forge's
`feature-architect` `## Verification plan` intake so the architect can promote it directly. **This is
the creation+implementation path: you write + RED-run the additive tests; Feature Forge writes production code + GREEN-runs.**


## RED authoring and count verification (prompt P8)

For every accepted plan item or UPGRADE row, create at least one additive test that encodes its
acceptance criterion — a **full suite, per the repo's real runners**, never a single test:
- **unit + integration** — `tests/` + `#[cfg(test)]`, run with `cargo test` / `cargo nextest` for Rust
  or `bun test` for JS (**bun**, never pnpm/node);
- a **differential-drive live case** for any item touching a binary/CLI's observable behavior — author
  cases in `scripts/differential-drive.cases.sh` and drive them with the harness's
  `scripts/differential-drive.sh` (drives the REAL binary and diffs its output against a golden;
  fail-closed when 0 cases run). Green unit tests are not proof (HFTASK-0078);
- **property / golden** fixtures where behavior warrants.
Never modify production code (tests are additive-only). Run the narrow command — `hf test <id>` or the
card's `test_commands` where available — and record:

- command,
- expected RED failure reason,
- **tests-ran count** (`> 0` required — an exit-0 that ran **zero** tests is a FAIL, the
  `parse_tests_ran` / fail-open ban),
- traceability: plan item ↔ acceptance criterion ↔ test path/name ↔ RED|GREEN.

A pre-implementation PASS is a bad test and must be rewritten. An exit-0 with zero tests is FAIL.
If the environment cannot run the test, record the owner wall and the exact command; do not mark the
item verifiable.

## Discipline
- **Coverage = reachability** (call graph), never file presence.
- **Evidence or it's not a gap** — cite the symbol; the verifier kills uncited claims.
- **Implementable cases only** — a builder could write each test verbatim from your bullet.
- **Read-only / invariant-safe** — never write or run tests; never design a test needing a banned C
  dep or breaching the trust boundary (the verifier feasibility-gates, but design clean).
- **Cover the upgrades** — an upgrade with no test to prove it is an incomplete plan.

## References
- `planning-engineer/references/state-contract.md` — the CLAIM/UPGRADE/FF-test-build-spec row formats +
  the `test-coverage` dimension.
- The `rust-port-parity` skill — the differential/golden-testing discipline reused for
  behavior-preserving upgrades.
