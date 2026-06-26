---
name: plan-memory-vector-intelligence
description: >-
  Plans and gates persistent memory plus vector/code intelligence for Planning Engineer: ICM recall/store,
  GitKB graph snapshots, vector/RAG indexes, source ledgers, and cross-session recall guarantees.
---

# plan-memory-vector-intelligence — persistent memory + vector intelligence axis

Every plan must survive context loss and every agent must be able to recover the why, graph, and
research evidence without trusting chat history. This skill maps memory/vector surfaces and emits
`.handoff/loop/plan/findings/memory-vector-intelligence-<T>.md`.

Required sections:
1. **Memory inventory** — ICM topics used, HANDOFF pointers, `.handoff` ledgers, source ledgers,
   vector indexes, GitKB code graph snapshots, and recall/store hooks.
2. **Vector intelligence map** — which indexes exist (`git-kb code`, embeddings/RAG/vector DB if
   present), freshness, owner, update command, and failure behavior.
3. **Recall guarantees** — session start recall, background-agent recall, wrap-up store, and cold-start
   resume proof; no plan may depend on conversation memory alone.
4. **Upgrade rows** — `axis: memory-vector-intelligence`, evidence, acceptance, risk, reversibility.
5. **Gate handoff** — artifact/test additions needed so missing memory/vector surfaces fail closed.
