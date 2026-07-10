# ADR-0005 — ruvector extension ↔ client crate pairing (pin 0.3.0 server / 2.0.5 client)

- **Status:** accepted (forge loop TASK-0092 / blueprint R8; plan PR #460 §3 row 8, verdict V20)
- **Date:** 2026-07-09
- **Owners:** envctl forge loop; operator ratifies by merge

## Context

The global-brain PostgreSQL cluster (declared as the `postgres-ruvector` component, TASK-0091)
runs the `ruvector` extension at **0.3.0**, while the envctl workspace pins the
**`ruvector-postgres` client crate at 2.0.5** (`Cargo.toml:153`, registries-not-repo directive).
A two-major skew looks like drift, so R8 required either an in-place reconcile or a pinned,
reasoned pairing.

Measured facts (2026-07-09, live cluster):

- `pg_available_extension_versions`: `0.1.0 (f) · 0.3.0 (installed, default) · 2.0.0 (f)`.
- `ruvector.control`: `default_version = '0.3.0'`.
- The ext ships **only a downgrade path** `ruvector--2.0.0--0.3.0.sql`; there is **no
  `0.3.0--2.0.0` upgrade script**, so `ALTER EXTENSION ruvector UPDATE TO '2.0.0'` has no
  migration chain.
- The presence of the downgrade script means a prior 2.0.0 install was **deliberately walked
  back** to 0.3.0 on this box ("this bit someone already", V20).
- The only in-place route to 2.0.0 is `DROP EXTENSION ... CASCADE` + recreate — which would drop
  every `ruvector(…)`-typed column: `codebase.embedding_minilm` (5157 MiniLM vectors, R3),
  `codebase.semantic_embedding`, and the `episodes` lane. That is a data-destroying migration,
  forbidden as a side effect of a "version tidy" (Law 1).

## Decision

**Pin the pairing instead of forcing a reconcile:**

1. **Server extension stays 0.3.0** — the installed default, the only version with a tested
   on-box history, and the version every live lane was created under.
2. **Client crate stays 2.0.5** — the crates.io pin under the registries-not-repo directive.
3. **The skew is safe today by construction:** envctl's `ruvector` cargo feature is
   **default-OFF** (V23), so no envctl client code speaks to the extension in any shipping
   build. The only live consumers of the cluster are psql/bun runtime scripts that use plain
   SQL + the `<=>` operator, which 0.3.0 serves.
4. **Reconcile trigger (when this ADR must be revisited):** the first change that makes a
   `ruvector-postgres`-linked code path touch the live cluster — e.g. R10's consumer feature
   flipping default-ON, or an episodes-lane migration. That change's cycle MUST first ship an
   authored `ruvector--0.3.0--2.0.0.sql` upgrade script (or a dump/reload migration plan) with
   a rollback test against a scratch database, as its own gated task.

## Consequences

- No live mutation of the brain store; all 5157+ vectors and both lanes survive untouched.
- The `postgres-ruvector` component's verify hook asserts `ext=0.3.0` implicitly (non-empty
  extversion); if someone upgrades the extension out-of-band, `envctl auto-detect` still reads
  healthy — version drift detection beyond presence is deferred to the reconcile trigger.
- R10 (first envctl ruvector consumer) remains buildable: it is feature-gated OFF and tests
  against its own fixtures, not the live cluster, so it does not trip the trigger by itself.
