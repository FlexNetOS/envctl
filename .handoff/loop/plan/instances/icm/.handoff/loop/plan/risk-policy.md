# risk_policy — fleet-convergence planning loop

risk_policy: each planning cycle is read-only on product code (sole mutation = additive RED tests in
an isolated target worktree). SUPERVISED items require an explicit owner decision before execution.
Trust-boundary, secrets, destructive, and provider/model risks are enumerated per target.

## icm (cycle 7)
- **trust-boundary** — icm carries an UNCONDITIONAL C floor (rusqlite{bundled} + sqlite-vec; ONNX
  optional). handoff's union kernel is **no-C** (redb). Risk: folding icm into the kernel would import
  C into the trust boundary. Policy: **SIDECAR only** — icm stays a memory service reached over the
  existing MCP/CLI seam; add a fail-closed CI dep-gate denying rusqlite/sqlite-vec/onnx in kernel
  crates so the boundary is mechanical, not aspirational.
- **secrets** — icm stores `~/.config/icm/credentials` (RTK cloud-sync) in user-global XDG, plaintext
  path. SUPERVISED: any residency migration or credential handling is envctl-owned (preview/apply/
  lock/rollback/parity); never auto-move secrets.
- **destructive** — 31 MCP tools expose ungated destructive mutators (`forget`/`forget_topic`/
  `consolidate`/`update`) to ~15 hosts with no write-side RBAC. SUPERVISED: gate destructive memory
  ops; default decay is weight-only and spares `critical`, `prune` is opt-in (currently safe).
- **provider/model** — embedding lane default e5-base (768d, local fastembed); summarization lane
  shells to host CLIs (claude-haiku-4-5/gpt-5-mini/claude-sonnet-4-5). No canonical model registry;
  no-downgrade not encoded. Risk: model drift. Policy: pin + register the embedding/summarization
  lanes; honor Upgrade-Only.
- **SUPERVISED (owner-wall)** — meta-owned data-residency migration; bundled-SQLite version bump (CI
  must prove parity); stale-CLAUDE.md replacement (large auto-loaded doc — verify before overwrite).
