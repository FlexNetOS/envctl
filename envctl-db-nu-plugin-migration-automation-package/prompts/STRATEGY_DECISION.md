# Strategy decision — package first, tooling first, or parallel?

## Decision

Use **contract-first parallel implementation**.

This is not a pure sequence where Codex fully implements the FlexNetOS package first. It is also not a pure tooling-first build where envctl is designed in the abstract. The correct path is:

1. **Lock the artifact contract first.** Extract the prior migration-artifact requirements and FlexNetOS package shape into versioned envctl database records/schemas.
2. **Run the prior package through an adapter early.** The existing package becomes the first executable migration bundle and acceptance fixture.
3. **Build envctl database execution in parallel.** envctl must persist targets, recipes, operations, evidence, approvals, artifacts, graph links, validations, checkpoints, rollbacks, and replay metadata.
4. **Build nu_plugin in parallel after shared protocol lock.** The plugin reads the envctl event/materialized views and appends controlled operator decisions; it does not own state.
5. **Gradually replace external shell helpers with native envctl collectors.** Native collectors are better long-term, but the adapter gives immediate continuity with the prior package.

## Why not implement the prior package first?

That would produce useful artifacts but still leave migration execution outside the database. It would fail the bigger requirement: envctl must reproduce migration operations anytime.

## Why not build envctl tooling first?

That risks building a generic orchestration database that cannot run the actual artifact contract. The FlexNetOS package is the concrete truth test.

## Why parallel?

Because the hard boundary is not code generation; it is state ownership. envctl must own state and replay. `nu_plugin` must expose control/visuals. The prior package must become an executable artifact contract. These three surfaces have to converge around the same event and artifact schema.

## Required first PR sequence

1. Shared target descriptor + artifact contract schema.
2. envctl database migration automation tables and persistence layer.
3. envctl adapter for `codex-flexnetos-migration-prompt-package`.
4. nu_plugin status/events/artifacts read-only commands.
5. Approval/pause/resume/replay mutating commands.
6. Native collectors and richer visualizations.
