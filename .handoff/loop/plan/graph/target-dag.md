# Target dependency graph (TDP) — weave instance (cycle 4)

This is the **target dependency graph** for the parallel weave instance: weave is the active node;
the union organs (handoff, rusty-idd) are `done`; the rest are context. weave is a transport ROOT
(no upstream deps); `envctl` and `lane` depend on weave (they use the transport).

## Ready-set / topological order
- **ready-set now** (deps satisfied): weave (in-flight), harness_hub, icm, grit.
- **after weave done:** envctl, lane become ready.
- topological order: [handoff, rusty-idd] (done) -> weave -> [envctl, lane]; harness_hub/icm/grit independent.

## SELF-REVISION
A node may revise its own sub-plan (localized) without re-running the whole DAG. If weave's A2A-adapter
decision changes the transport contract, only the downstream `pending` subgraph (envctl, lane) is
marked for re-derivation — not the already-`done` union nodes. This instance touched only the `weave`
node; the union loop branch owns the full 63-node backlog DAG.
