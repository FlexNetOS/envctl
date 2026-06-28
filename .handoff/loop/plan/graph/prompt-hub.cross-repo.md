# Cross-repo edges — prompt-hub (Front-Door STORE → rusty-idd)

Snapshot git `f826ea33`, 2026-06-27. The frame places prompt_hub as the intent STORE that
(per ADR-0007) emits provenance-stamped goal artifacts to **rusty-idd**. This file maps the
intended cross-repo edge and its actual status.

## Cross-repo edge table

| From (prompt_hub) | To (other repo) | Edge kind | Status in code |
|---|---|---|---|
| `PromptHub::process_input` → `Intent`/`Artifact` (hub.rs / models.rs) | **rusty-idd** | goal-artifact emission (ADR-0007) | **ABSENT** — no `rusty-idd`/`goal-artifact`/`provenance` symbol or call in any of the 3 members' `src/`. Defined only in `docs/plans/lifeos-meta-front-door.md`. |
| harness_hub interpreter | prompt_hub store | "two-layer front door" upstream | **Not present in this repo's code** — no harness_hub import/dependency in prompt_hub manifests. |

## Method (`cross-repo-reference`)
- `git-kb code` callers/callees were run scoped to prompt_hub member src; **no edge crosses
  the repo boundary** into rusty-idd or harness_hub (no such crates are dependencies).
- `grep -riE 'rusty.?idd|goal.?artifact|provenance'` over `prompt-hub/src prompthub/src
  prompthub-server/src` = **0 hits**; over `docs/` = 1 file (the plan).
- Closest in-repo emission analog: `generate_bundle` (`prompthub-server/src/routes.rs:879`,
  `GET /api/v1/swarm/bundle`) and the `Artifact` enum (`models.rs:654`) — neither is
  rusty-idd-aware nor provenance-stamped.

## Blast radius of the (future) boundary
Because the seam is unbuilt, there is **no current cross-repo blast radius**. When built, the
emission point will most naturally hang off `process_input`/`Intent` (the front door) and a new
provenance field on `Artifact`; the architect should treat this as a **new boundary to design**,
sequenced against rusty-idd's consumption contract — not an existing edge to refactor.

## Verdict
prompt-hub is, in code, a **self-contained 3-member workspace** with no live cross-repo edges.
The rusty-idd / harness_hub fleet edges are **architectural intent (docs), not implemented**.
This is the cycle's headline finding for the architect's fleet-level diagram.
