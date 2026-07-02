# Agent → execution-backend matrix

How planning/merge agents that drive `grit` map to execution backends. grit itself is a
control-plane lock substrate (Unix-only, git-CLI-dependent); these rows say which backend each agent
*lane* runs on and what grit reach it has from there.

---

## grit

Backends (lanes): `read-only-local` · `isolated-worktree` · `container` · `remote-vm` · `cloud-agent`.
Coordination plane between agents: grit's `room` (code-contention) bridged to weave (`A2A`,
cross-session) — the grit→weave bridge (roadmap #10) is the named upgrade.

| Backend | Who runs here | grit role / verbs | Why this backend | Risk |
|---|---|---|---|---|
| **read-only-local** | planning crew (cartographer, analysts, verifier, this architect), `grit status`/`symbols`/`plan`/`watch --poll` | observe only — no lock acquisition, no merge | grit's read verbs are side-effect-free; the planning loop is read-only on grit's tree | none (no writes) |
| **isolated-worktree** | each parallel build/merge agent (`agent-1..agent-N`) | `claim` → work in `.grit/worktrees/agent-N` → `done` (rebase+merge) | this IS grit's native model — one git worktree per agent over a shared `.git` object store; AST symbol locks gate the shared hotspots worktree isolation alone cannot | merge/`done` = `[!!]` SUPERVISED |
| **container** | CI / sandboxed Feature-Forge GREEN runs of `tests/union_dedup_contract.rs` | `cargo test` driving the `grit` binary; reconcile build | full-OS profile (std + tokio + filesystem + git) fits a container; isolates the phantom-workspace remediation (roadmap #0a) | low (ephemeral) |
| **remote-vm** | a self-hosted lock host / shared coordinator | hosts the SQLite WAL or a colocated S3/MinIO endpoint; serves `LockStore` truth cross-machine | grit's cross-machine source of truth is the configured backend; a Linux VM is the natural home (Unix + git + HTTPS) | medium (shared state; backend creds) |
| **cloud-agent** | GitHub-hosted / cloud background agents opening PRs | `session pr` (push + PR), or consuming grit lock state over S3/Azure Event Grid | the cloud-vendor lock backend (S3/R2/Azure Blob, conditional-PUT/lease) lets a cloud agent coordinate without a local coordinator; push/PR is the destructive seam | push/PR = `[!!]`; plaintext-key residency if Azure local config used |

### Edge / unsupported

- Full-Linux **Raspberry Pi** can be an `isolated-worktree` agent or a `read-only-local` poll observer
  but must build from source (no ARM64 binary ships; `openssl-sys` cross-build broken). Pi Zero is
  observer-only.
- **mobile / AI glasses / wearables / ESP32**: N/A as any backend — no git/worktree/std/tokio-HTTPS
  host. At most a downstream Event-Grid notification *viewer*, which runs zero grit code.

### Inter-agent coordination (A2A)

Agents on these backends do not message each other directly — they observe each other through the
shared lock truth (the `LockStore` bucket) plus grit's `room` event stream. The cross-session **A2A**
plane is `weave`: the proposed grit→weave bridge forwards room `Released`/`AgentDone` events as weave
nudges so a queued agent's *session* (on any backend above) is pinged, not just its socket watcher.
Live proof this plane works: the weave A2A round-trip this cycle (envctl ↔ rusty-idd) that produced
prompt_hub PR #182.
