# Impact map

This phase changes CI policy and workflow entrypoints. The immediate blast radius is all envctl CI jobs and every gate they invoke. The policy is fail-closed: a non-Kache cache directive or an automatic non-Nushell shell is a hard failure. Rust engine changes are deferred until their GitNexus upstream impact is recorded.

