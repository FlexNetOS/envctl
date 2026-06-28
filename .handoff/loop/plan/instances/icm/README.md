# Archived planning-instance shared artifacts — icm

This directory preserves the branch-local `plan/loop-icm` versions of shared
planning-loop artifacts that collided after earlier planning instances merged
first on `master`.

Conflict policy: keep the current `origin/master` shared artifacts in their
canonical `.handoff/loop/plan/*` locations, and archive this branch's local
versions here instead of silently clobbering or downgrading already-merged plan
docs. This makes the sync additive and reviewable while preserving the ICM
planning evidence for follow-up consolidation.
