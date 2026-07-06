# Autoresearch Findings: envctl Codebase

Date: 2026-07-02
Target: `envctl`
Worktree: `/home/flexnetos/FlexNetOS/src/envctl-plan-autoresearch-20260702`
Branch: `codex/plan-autoresearch-20260702`

## Verdict

envctl has a strong invariant culture: no-C gates, ring-only TLS pins, remote-only
libSQL, agent-env lock checks, harness gates, and explicit dry-run defaults. The
autoresearch gap is that those invariants are better guarded than the planning
evidence that decides what to build next. The code graph is useful, but noisy and
currently rooted to the main checkout from a sibling worktree; the source ledger
has the right fields, but the gate checks field presence more than date truth,
source freshness, contradiction handling, or stale recommendation invalidation.

## Code Auto-Research

Exact commands run:

- `rtk /home/flexnetos/FlexNetOS/usr/bin/git-kb code index --force --prune`
- `rtk /home/flexnetos/FlexNetOS/usr/bin/git-kb code stats --json`
- `rtk /home/flexnetos/FlexNetOS/usr/bin/git-kb code doctor --json`
- `rtk /home/flexnetos/FlexNetOS/usr/bin/git-kb code entrypoints --refresh --json`
- `rtk /home/flexnetos/FlexNetOS/usr/bin/git-kb code flows --refresh --json`
- `rtk /home/flexnetos/FlexNetOS/usr/bin/git-kb code dead --json`
- `rtk rg -n "stale|source ledger|autoresearch|contradiction|recency|advisory|vendor|git-kb code|entrypoints|flows|dead|unresolved" .handoff/loop/plan .agents/skills scripts ci crates`
- `rtk rg -n "tonic|hyper|rustls|ring|aws-lc|libsql|openssl|sqlite|mimalloc|argon2|chacha20poly1305|blake3" Cargo.toml Cargo.lock crates/*/Cargo.toml ci/gates/no-c.sh`
- `rtk wc -l crates/gui/src/main.rs crates/cli/src/main.rs crates/engine/src/catalog.rs crates/agent-env/tests/parity_vs_kasetto.rs crates/secrets-engine/src/lib.rs crates/engine/src/executor.rs`

Graph snapshot and diff:

- Snapshot: `.handoff/loop/plan/graph/envctl.graph.md`
- Diff: `.handoff/loop/plan/graph/envctl.diff.md`
- Indexed symbols: 4,297 from 925 files.
- Resolved call edges: 10,157.
- Unresolved calls: 27,251.
- Service-route/client facts: 0 routes, 0 client calls.
- Integrity caveat: `git-kb code stats --json` reported `kb_root` as
  `/home/flexnetos/FlexNetOS/src/envctl`, not the active sibling worktree root.

Entrypoints and public API:

- Runtime entrypoints are `crates/cli/src/main.rs`, `crates/cli/src/bin/meta-env.rs`,
  `crates/gui/src/main.rs`, `crates/secretctl/src/main.rs`, and
  `crates/secretd/src/main.rs`.
- `crates/engine/src/lib.rs` exposes the broad engine surface: add-repo, agent,
  catalog, command, component, dashboard, detect, drift, executor, graph, guard,
  install, lock, migration, model, peer, register, runner, runtime, secrets, and
  secrets edge modules.
- `crates/agent-env/src/lib.rs` exposes the absorbed agent-env surface: agent,
  command, config, config-edit, dirs, driver, extend, fsops, hash, lock, mcp,
  profile, report, runtime, source, sync, and util.
- `crates/secrets-engine/src/lib.rs` exposes the vault, broker, CA, guard, inject,
  keyslot, GitHub mint, path, seam, startup, and store-opening APIs.

Hotspots:

- `crates/secrets-engine/src/lib.rs`: 5,876 lines.
- `crates/cli/src/main.rs`: 4,998 lines.
- `crates/engine/src/catalog.rs`: 4,331 lines.
- `crates/gui/src/main.rs`: 3,466 lines.
- `crates/agent-env/tests/parity_vs_kasetto.rs`: 3,140 lines.
- `crates/engine/src/executor.rs`: 1,115 lines.

Dead code and unresolved calls:

- `git-kb code dead --json` returned mostly test-only functions. The planner should
  not treat those as product dead code without a `#[cfg(test)]` or test-file filter.
- Unresolved calls outnumber resolved edges. The dominant unresolved reasons were
  `no_match`, `skip_list`, and `ambiguous`, so impact analysis needs a confidence
  annotation until resolver coverage improves.
- `git-kb code doctor --json` recommended reviewing unresolved-call provenance,
  symbol-forwarding ambiguity, and missing route/client fact storage.

Cross-repo impact:

- envctl is a meta peer and environment manager; breakage can affect `$META_ROOT`
  tool installation, generated `.codex`/`.Codex` agent surfaces, dashboard launchers,
  and secrets consumers.
- The codebase also depends on shared substrates (`loop_lib`,
  `meta_plugin_protocol`). Planning must preserve the existing substrate direction:
  upgrade the shared substrate when needed, then consume it here.
- Service-edge graph facts are currently empty, so cross-repo impact cannot be
  trusted from route/client extraction alone.

## Web Auto-Research

Recency window: 2026-04-03 through 2026-07-02.
Source ledger: `.handoff/loop/plan/research/sources-envctl.jsonl`

Claims:

- W1: The latest stable Rust signal is newer than envctl's declared MSRV floor.
  This is not a demand to bump MSRV. It means the planner needs separate lanes:
  one gate for MSRV compatibility and one recurring lane for current-stable
  warnings, clippy drift, and dependency compatibility.
- W2: rustls currently defaults to `aws-lc-rs`; envctl's
  `default-features = false` plus `ring` features are not cosmetic. They are part
  of the no-C trust boundary.
- W3: libSQL's `core`/replication features include C-backed code, while `remote`
  is the HTTP-only path envctl intends to use. Feature unification is the primary
  regression risk.
- W4: The pinned tonic floor is tied to an older RustSec advisory and remains a
  valid floor check, but current advisory state must come from a fresh RustSec or
  `cargo audit` run each cycle.
- W5: GitHub Actions now has Ubuntu 26.04 runner images in public preview. Because
  envctl targets an Ubuntu 26.04 workstation, runner-routing and doctor coverage
  should track the preview/stable transition instead of assuming the older hosted
  runner set is enough.

Contradiction and stale-source handling:

- Rust current-stable versus MSRV is a policy distinction, not a contradiction.
  Treating latest stable as an automatic MSRV bump would be a planning bug.
- The rustls docs contradict any default-feature shortcut: the default provider is
  not envctl's desired provider.
- The libSQL docs contradict any generic "libsql is Rust-native" shorthand:
  remote-only is the safe contract; core/replication are outside the trust boundary.
- The tonic advisory is outside the 90-day window and must be labeled as a stale
  but still relevant floor check. It must not satisfy current advisory freshness by
  itself.

## Cadence And Invalidation

Per-cycle refresh:

- Run the exact `git-kb code` commands above.
- Record `git rev-parse --show-toplevel`, branch, HEAD, `kb_root`, symbol count,
  file count, resolved edges, unresolved calls, route/client facts, and top
  hotspots.
- Run source-ledger checks against a 90-day window and mark every stale row with a
  reason: floor check, historical context, superseded, or invalidated.

Batch-boundary deep refresh:

- Re-run a full code index from the true target root.
- Compare graph metrics with the prior snapshot.
- Re-read official vendor docs for Rust, rustls, libSQL, GitHub Actions runners,
  and RustSec.
- Reconcile recommendations whose evidence source changed or expired.

Resume refresh:

- Refuse to continue from a graph snapshot when `kb_root` differs from
  `git rev-parse --show-toplevel`.
- Refuse to promote a recommendation when all supporting source rows are stale
  without a stale-ok reason.
- Re-open unresolved recommendations whose acceptance criteria rely on "latest"
  toolchain, runner-image, or advisory facts.

Current gate gap:

- `.agents/skills/planning-engineer/scripts/plan-artifact-gate.sh` validates that
  source ledger rows contain required keys and claim IDs. It does not currently
  prove date parseability, `in_recency_window` correctness, source authority,
  stale-source invalidation, or contradiction resolution.

## Upgrade Rows

### U1: Fail Closed On Code-Graph Root Drift

axis: autoresearch

Evidence: `git-kb code stats --json` reported `kb_root` as the main checkout while
the active worktree was the isolated sibling checkout.

Acceptance: Add a planning gate that compares `git rev-parse --show-toplevel` with
the graph `kb_root` before a code graph can support recommendations.

Risk: Low. The likely failure mode is revealing existing ambiguous graph state.

Reversibility: Remove the gate or downgrade it to warning if GitKB gains explicit
multi-worktree scoping and emits a different trusted root field.

### U2: Add Graph-Quality Thresholds To Autoresearch

axis: autoresearch

Evidence: Unresolved calls were 27,251 versus 10,157 resolved edges, and service
route/client facts were all zero.

Acceptance: Autoresearch artifacts must report unresolved ratio, route/client fact
counts, and resolver confidence. Impact recommendations must be blocked or marked
low-confidence when unresolved calls materially exceed resolved calls.

Risk: Medium. It may initially classify useful graph results as low-confidence.

Reversibility: Relax thresholds once resolver provenance improves and has tests.

### U3: Harden Source-Ledger Freshness Checks

axis: autoresearch

Evidence: The artifact gate checks for required source-ledger fields, but not that
dates parse, the 90-day boolean is true, stale rows have a stale-ok reason, or stale
evidence invalidates recommendations.

Acceptance: Add RED fixtures where an old `published_at` row marked
`in_recency_window: true` fails, a stale row without a stale-ok reason fails, and a
recommendation supported only by stale rows fails.

Risk: Low. Existing artifacts may need explicit stale classifications.

Reversibility: Keep the strict parser but allow an emergency override field with a
required owner note.

### U4: Track Latest Rust Separately From MSRV

axis: autoresearch

Evidence: envctl declares MSRV 1.88.0 while the official latest-stable release is
newer. Those facts serve different planning needs.

Acceptance: CI or planning artifacts report both MSRV gate status and
current-stable observation status without conflating either lane with a required
MSRV bump.

Risk: Low. This is observational unless a later policy makes current-stable a
blocking gate.

Reversibility: Remove the latest-stable lane if it generates noise without finding
actionable compatibility issues.

### U5: Add Ubuntu 26.04 Runner-Routing Watch

axis: autoresearch

Evidence: GitHub Actions has Ubuntu 26.04 runner images in public preview, and
envctl targets an Ubuntu 26.04 workstation.

Acceptance: `ci/gates/runner-routing.sh` or its planning companion records whether
26.04 hosted/local coverage is intentionally adopted, deferred, or blocked on
preview maturity.

Risk: Medium. Runner images in preview can introduce transient tool-version drift.

Reversibility: Keep the current runner policy and carry a dated deferral note until
26.04 images leave preview.

### U6: Split Planning Attention Around Giant Runtime Flows

axis: autoresearch

Evidence: `secrets-engine/src/lib.rs`, `cli/src/main.rs`, `engine/src/catalog.rs`,
and `gui/src/main.rs` dominate line count and flow size.

Acceptance: Planning cards touching these files must name the owning flow, expected
entrypoint, and focused runtime proof before implementation starts.

Risk: Low. This adds planning friction only around high-blast-radius surfaces.

Reversibility: Lower the requirement to warning-only once module boundaries shrink.

### U7: Preserve No-C Feature-Unification Proofs

axis: autoresearch

Evidence: rustls and libSQL official docs both show defaults/features that would
violate envctl's trust boundary if enabled casually. Existing no-C gates are
therefore load-bearing.

Acceptance: Autoresearch snapshots cite the no-C gate result and dependency feature
intent whenever recommendations touch TLS, SQL storage, HTTP, crypto, or allocator
crates.

Risk: Low. The project already treats this as non-negotiable.

Reversibility: None without changing the threat model.

## Gate Handoff

Tests or gates that should be added:

- Root drift fixture: graph root differs from repository root, and the planning
  gate fails closed.
- Ledger recency fixture: `published_at` is older than the 90-day window while
  `in_recency_window` is true, and the gate fails.
- Stale-only recommendation fixture: an upgrade row supported only by stale rows
  without stale-ok reasons fails.
- Dead-code classification fixture: test-only dead-code entries are labeled as
  test-only and cannot be counted as product dead code.
- Graph-quality fixture: unresolved calls materially exceed resolved calls, and
  impact recommendations must carry low-confidence status or fail.
- Runner watch fixture: Ubuntu runner policy has a dated decision for 26.04 when
  the workstation target is Ubuntu 26.04.

## Open Gaps To Surface Next

- Confirm whether GitKB can be pointed at the sibling worktree root, or whether
  envctl autoresearch should run only in the main checkout with an explicit dirty
  branch guard.
- Decide whether source-ledger hardening belongs in
  `plan-artifact-gate.sh`, a new helper, or the `plan-autoresearch-loop` skill.
- Add a compact machine-readable graph metrics row so future cycles can diff
  without scraping markdown.
