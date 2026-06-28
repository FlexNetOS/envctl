# Archived planning-instance shared artifacts — grit

This directory preserves the branch-local `plan/loop-grit` versions of shared
planning-loop artifacts that collided after earlier planning instances merged
first on `master`.

Conflict policy: keep the current `origin/master` shared artifacts in their
canonical `.handoff/loop/plan/*` locations, and archive this branch's local
versions here instead of silently clobbering or downgrading already-merged plan
docs. This makes the sync additive and reviewable while preserving the grit
planning evidence for follow-up consolidation.

Additional sync note: the branch-local `.handoff/loop/proposed-upgrades.md`
(global plan-loop escalation ledger content for cycle 5 / grit) is also archived
under `./.handoff/loop/proposed-upgrades.md`; the canonical file stays at the
latest merged `origin/master` version so open proposals remain monotonic.
