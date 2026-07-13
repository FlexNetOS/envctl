# 01 — Architect plan: Blueprint test suite T1–T6 + R1 swarm-immune wrapper fix
(persisted by orchestrator; architect agent a5ea0235dd76c76c0, 2026-07-09)

Intake corrections discovered by architect (material, resolved in-plan):
- Fact 7 STALE: R5 (codedb_store_pg) already MERGED to nu_plugin master (PR #16, 2026-07-09T13:13:34Z);
  nu_plugin-pgstore-wt no longer exists. T5 = GREEN opt-in parity regression-lock on master.
- Router CLI shape: no `route --json` subcommand — plain argv, always emits JSON; fixture uses
  `bun .claude/helpers/router.js "<prompt>"` and reads `modelTier`.
- meta-local-policy.sh verified gate-safe for the installer (flags only usr/bin frontdoor writes).

VERDICT: GO

Six tests are buildable now and R1 is a precise 3-line fix against a proven native API; nothing needs an owner decision. Sequential single-crew, envctl PR is script-only (all gates stay green), nu_plugin gets one `#[ignore]`d differential test.

## Target repos (routing: sequential single-crew — agreed)
| Repo | Path | Units | Change class | PR target |
|------|------|-------|--------------|-----------|
| envctl | /home/flexnetos/lifeos/src/envctl-ff-blueprint-tests-wt (branch ff-blueprint-tests off develop @ 7e74e5f) | U1 (R1 canonical+install), U2 (T1), U3 (T2), U4 (T3), U5 (T4), U6 (T6), U8 (installer) | scripts/mjs/json + installer only — no Cargo, no Rust, no dep change | develop |
| nu_plugin | fresh worktree off master (crates/codedb_store_pg now on master) | U7 (T5) | one #[ignore]d integration test + 2 dev-deps | per nu_plugin topology (confirm origin/HEAD) |
| var-runtime (NOT a git repo) | /home/flexnetos/lifeos/var/lib/ruvector/ | U1 live target + U8 install dest | in-place, Law-1 archived | n/a (untracked runtime) |

Routing rationale: two repos touched but U7 is one isolated additive test; below the A2 threshold. Sequential: U1→U6+U8 in envctl worktree, then U7 in nu_plugin.

## Placement / durability layout
Canonical-in-repo + install-to-runtime split. Canonical (committed, envctl): scripts/tests/blueprint/:
- t1_swarm_immune.mjs, t2_envctl_db_fresh.sh, t3_embedder_wiring.sh, t4_router_discrimination.mjs, t6_musl_static.sh
- fixtures/router_prompts.json (T4 10-prompt golden)
- runtime/swarm-immune.mjs (canonical fixed R1 wrapper — tracked for reproducibility/review)
- install.sh (Law-1 archive of live wrapper → install fix → copy bun/psql tests into var/lib/ruvector/tests/)

Sync rule: committed copy is source of truth; install.sh is idempotent, re-installs from canonical. Each script header carries `# canonical: scripts/tests/blueprint/<name>`. Tests reference production by absolute path — guardian may drive canonical copies directly; runtime copy satisfies the FF-spec home.

Gate-safety (verified): meta-local-policy.sh only flags ln -sf / install -m755 into $META_ROOT/usr/bin (lines 122–147). install.sh targets var/lib/ruvector/ and ~/.claude/archive/ only — never usr/bin.

T2/T6 non-wiring: placed in scripts/tests/blueprint/, NOT added to ci/gates/*.sh or ci.yml (harness-scripts.sh runs a fixed list — a new subdir is inert). Headers record flip-on conditions. RED only when run manually.

## Units
| U# | Unit | Lives at | Wired by / drives | Acceptance |
|----|------|----------|-------------------|------------|
| U1 | R1: fix swarm-immune.mjs | canonical scripts/tests/blueprint/runtime/swarm-immune.mjs; live var/lib/ruvector/swarm-immune.mjs (installed by U8) | immuneGraph required by T1 + coordination consumers | T1 GREEN post-install; frozen surface immuneGraph(edges)→{g,isolationBoundary(),weakestCoupling(),degrade(u,v,w),connected()} preserved |
| U2 | T1 bun integration | scripts/tests/blueprint/t1_swarm_immune.mjs | bun t1_swarm_immune.mjs | RED pre-R1 (NumberExpected); GREEN post-R1 |
| U3 | T2 smoke | scripts/tests/blueprint/t2_envctl_db_fresh.sh | /home/flexnetos/lifeos/usr/bin/envctl db --help | RED now (unrecognized subcommand, V9); flip-on: wire after R2 |
| U4 | T3 psql+json assertion | scripts/tests/blueprint/t3_embedder_wiring.sh | read-only psql SELECT + manifest read | RED now (minilm 0≠5157 AND manifest model "fallback"); GREEN after R3 |
| U5 | T4 bun golden fixture | scripts/tests/blueprint/t4_router_discrimination.mjs + fixtures/router_prompts.json | ROUTER_DIR router; parses modelTier | RED now (worktree constant-opus V7; main tier-absent); GREEN after R4 |
| U6 | T6 build smoke | scripts/tests/blueprint/t6_musl_static.sh | file(1) on musl-target binary | RED now (target absent, V10); flip-on: wire beside no-c.sh after R9 |
| U7 | T5 Rust differential | nu_plugin crates/codedb_store_pg/tests/blobstore_parity.rs + dev-deps (codedb-store-redb, tempfile) | cargo test -p codedb-store-pg --test blobstore_parity | GREEN opt-in (CODEDB_PG_TEST=1 … -- --ignored, live socket); #[ignore]d ⇒ default workspace test green |
| U8 | Installer / runtime home | scripts/tests/blueprint/install.sh | Law-1 archive live wrapper, install fix, copy tests to var/lib/ruvector/tests/ | idempotent; the RED→GREEN transition mechanism |

## R1 fix — exact wrapper content + T1 assertion math
Fixed swarm-immune.mjs (string-name surface preserved; names→u32 map; METHOD calls):

```js
// Swarm immune system — ruvector MinCut wired against the agent-coordination graph.
// The native addon indexes nodes by u32 ids and exposes cutEdges()/isConnected()/
// minCut() as METHODS; this wrapper preserves the documented string-named agent
// surface and maps names→u32 internally (fixes V4's 3 call-shape bugs).
import { createRequire } from 'module';
const require = createRequire(import.meta.url);
const { MinCut } = require('/home/flexnetos/lifeos/var/lib/ruvector/mincut/ruvector_mincut.node');

// edges: [srcAgent, dstAgent, couplingStrength] — srcAgent/dstAgent are string agent names.
export function immuneGraph(edges) {
  const ids = new Map();
  const names = [];
  const idOf = (name) => {
    if (!ids.has(name)) { ids.set(name, names.length); names.push(name); }
    return ids.get(name);
  };
  const numericEdges = edges.map(([u, v, w]) => [idOf(u), idOf(v), w]);
  const nameOf = (id) => names[id] ?? id;

  const g = MinCut.fromEdges(numericEdges);
  return {
    g,
    isolationBoundary: () =>
      g.cutEdges().map((e) => ({ ...e, source: nameOf(e.source), target: nameOf(e.target) })),
    weakestCoupling: () => g.minCutValue,
    degrade: (u, v, newWeight) => {
      g.deleteEdge(idOf(u), idOf(v));
      g.insertEdge(idOf(u), idOf(v), newWeight);
    },
    connected: () => g.isConnected(),
  };
}
```
Robustness: if g.minCutValue proves stale post-degrade on the GREEN drive, substitute g.minCut().value — only if observed.

T1 fixture graph (first-seen ids): coordinator=0, researcher=1, materializer=2, review-gate=3, merge-resolver=4 (the five real .rvf.db containers, V14):
  ['coordinator','researcher',2.0] ['coordinator','materializer',2.0] ['researcher','materializer',2.0]
  ['coordinator','review-gate',1.0]  // pendant → original global min cut
  ['researcher','merge-resolver',3.0] // pendant/bridge → degraded in the test
Pre-degrade GREEN: connected()===true; weakestCoupling()===1.0; isolationBoundary() = 1 edge, unordered endpoints {coordinator,review-gate}, weight 1.0.
degrade('researcher','merge-resolver',0.01) → post-degrade GREEN: connected()===true; weakestCoupling()===0.01 (±1e-9); boundary {researcher,merge-resolver} @0.01.
Why the cut genuinely shifts: merge-resolver is a PENDANT (isolating it costs only its lone edge). Degrade 3.0→0.01 < 1.0 moves the global min from review-gate's pendant to merge-resolver's. Unordered endpoint-set comparison + float epsilon.

## T4 fixture fixtures/router_prompts.json
{ "field": "modelTier", "stability_runs": 3, "cases": [
  {"prompt":"fix a typo in the README","expect_tier":"haiku"},
  {"prompt":"rename the local variable foo to bar","expect_tier":"haiku"},
  {"prompt":"add a trailing newline at the end of the file","expect_tier":"haiku"},
  {"prompt":"update the copyright year in the license header","expect_tier":"haiku"},
  {"prompt":"bump the patch version in Cargo.toml","expect_tier":"haiku"},
  {"prompt":"design a Byzantine fault-tolerant consensus protocol for the agent mesh","expect_tier":"opus"},
  {"prompt":"prove the min-cut isolation boundary is correct under dynamic edge updates","expect_tier":"opus"},
  {"prompt":"architect a two-plane model-routing system with a calibration merge gate","expect_tier":"opus"},
  {"prompt":"refactor the async daemon to remove the block_on/spawn_blocking deadlock across the tonic service boundary","expect_tier":"opus"},
  {"prompt":"derive the memory-safety invariants for the lock-free trajectory ring buffer","expect_tier":"opus"} ]}
Runner: ROUTER_DIR env (default /home/flexnetos/lifeos/src/meta-ruvector); invoke `bun .claude/helpers/router.js "<prompt>"`; parse JSON; require all 3 runs == expect_tier; absent modelTier counts FAIL; per-case PASS/FAIL + overall exit code.

## Runtime surface — runtime_verifiable? YES
- T1/R1: RED-before (bun T1 vs pre-fix live wrapper → NumberExpected), U8 install, GREEN-after (exit 0, boundary shift, connected()).
- T4 RED: ROUTER_DIR=meta-ruvector-router-wt → constant opus; default ROUTER_DIR → tier-absent.
- T3 RED: psql SELECT → 0/5157 + manifest fallback (read-only).
- T2 RED: usr/bin/envctl db --help → nonzero.
- T6 RED: musl binary absent.
- T5 GREEN opt-in: CODEDB_PG_TEST=1 cargo test -p codedb-store-pg --test blobstore_parity -- --ignored.

## Verification plan
- T1 RED→GREEN is the acceptance for U1+U2. T2/T3/T6 RED-now captures. T4 RED against both ROUTER_DIR values. T5 GREEN opt-in with dedicated temp table codedb_t5_<pid> (never codebase/codebase_codedb), dropped after; redb side tempfile::tempdir; compare list_source_files() → BTreeSet<(relative_path, sha256)> and captured_paths().
- envctl gates all green (script-only): no-c, shape, enable, p7, kdf-feature-off, agent-env, meta-local-policy, add-repo-policy, loop-state, harness-scripts, cargo-audit, runner-routing, meta-substrates + fmt/clippy/test (untouched Rust). Use rtk proxy for precise output.
- nu_plugin: workspace test green (T5 #[ignore]d); fmt/clippy clean on the new test.

## Invariants check
- No C in trust boundary: PASS — envctl dep graph untouched; nu_plugin dev-deps pure-Rust (codedb-store-redb=redb 2.6.1, tempfile; rust-postgres already present).
- One rustls ring-only: PASS — no TLS/dep change.
- Engine single shared non-printing lib: PASS — zero engine edits; R1 is var-runtime bun-lane JS (plan §12), not language drift.
- Destructive fail-closed: PASS/strengthened — tests read-only vs production (T3 SELECT; T5 temp table); U8 Law-1 archives before overwrite.
- Rust-native workspace: PASS — no non-Rust source enters crates; bun/psql harness mirrors scripts/tests/*.sh.
- Runtime surface: declared YES with drive paths.

## Lock/manifest sync
None. No envctl.lock/agent-env.lock/manifest change. nu_plugin Cargo.lock updates on its side only (dev-deps).

## Open corrections (not forks)
1. Fact 7 stale — R5 merged; T5 = GREEN regression-lock on fresh nu_plugin worktree off master. Fallback: compile-only #[ignore]d skeleton if live-socket drive infeasible in-cycle.
2. Router shape resolved against source: plain argv, no subcommand parser.
3. Judgment call: tracked canonical copy of the fixed wrapper (scripts/tests/blueprint/runtime/swarm-immune.mjs) added for reviewability/durability — purely additive; drop to var/lib-only if reviewer prefers.
