---
name: plan-autoresearch-loop
description: >-
  Defines aggressive continuous auto-research for planning: code graph refresh, web recency refresh,
  source-ledger updates, contradiction checks, and stale-evidence invalidation on every loop cycle.
---

# plan-autoresearch-loop — constant code + web auto-research axis

The Planning Engineer loop should over-index on fresh evidence. Every cycle must refresh code graph
facts and web/tool facts, record what changed, and invalidate stale recommendations. Emit
`.handoff/loop/plan/findings/autoresearch-<T>.md`.

Required sections:
1. **Code auto-research** — exact `git-kb code` commands, graph snapshot/diff, entrypoint/public API,
   hotspots, dead code, unresolved calls, and cross-repo impact.
2. **Web auto-research** — 90-day recency window, official docs first, source ledger rows, advisories,
   vendor docs, and contradiction checks.
3. **Cadence** — per-cycle required refresh, batch-boundary deep refresh, resume refresh, and stale
   source invalidation rules.
4. **Upgrade rows** — `axis: autoresearch`, evidence, acceptance, risk, reversibility.
5. **Gate handoff** — tests/gates that prove missing stale-evidence checks fail closed.
