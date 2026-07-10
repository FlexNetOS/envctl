// canonical: scripts/tests/blueprint/t1_swarm_immune.mjs
// T1 — bun integration test for the swarm-immune MinCut wrapper (R1).
//   RED  (pre-R1): the live wrapper passes string agent names straight to the
//         native addon => "Failed to convert napi value String into rust type
//         u32" / code "NumberExpected" => this test throws at immuneGraph() and
//         exits non-zero.
//   GREEN (post-R1, after install.sh has installed the fixed wrapper): every
//         assertion below passes and this test exits 0.
//
// Drives PRODUCTION by absolute path. Override the wrapper under test with
// SWARM_IMMUNE=<path> (the guardian may point this at the canonical
// runtime/swarm-immune.mjs). Default is the live installed runtime wrapper so the
// install.sh RED->GREEN transition is observable end to end.
//
// Fixture graph (first-seen ids): coordinator=0, researcher=1, materializer=2,
// review-gate=3, merge-resolver=4 — the five real .rvf.db agent containers.
//   coordinator--researcher   2.0
//   coordinator--materializer 2.0
//   researcher--materializer  2.0
//   coordinator--review-gate  1.0   (pendant -> original global min cut)
//   researcher--merge-resolver 3.0  (pendant/bridge -> degraded in the test)
// Pre-degrade the global min cut is review-gate's pendant edge (1.0). Degrading
// merge-resolver's pendant 3.0 -> 0.01 (< 1.0) moves the global min cut to it,
// because isolating a pendant costs only its single edge.

const WRAPPER = process.env.SWARM_IMMUNE || '/home/flexnetos/lifeos/var/lib/ruvector/swarm-immune.mjs';

let failures = 0;
const check = (name, cond, detail) => {
  if (cond) {
    console.log(`PASS ${name}`);
  } else {
    console.log(`FAIL ${name}${detail !== undefined ? ` — ${detail}` : ''}`);
    failures += 1;
  }
};
const approx = (a, b, eps = 1e-9) => typeof a === 'number' && Math.abs(a - b) <= eps;
const endpointSet = (edge) => new Set([edge.source, edge.target]);
const setEq = (s, ...members) => s.size === members.length && members.every((m) => s.has(m));

const { immuneGraph } = await import(WRAPPER);

const edges = [
  ['coordinator', 'researcher', 2.0],
  ['coordinator', 'materializer', 2.0],
  ['researcher', 'materializer', 2.0],
  ['coordinator', 'review-gate', 1.0],
  ['researcher', 'merge-resolver', 3.0],
];

console.log(`T1 swarm-immune wrapper — under test: ${WRAPPER}`);
const G = immuneGraph(edges);

// --- pre-degrade: global min cut is the review-gate pendant @ 1.0 ---
check('pre.connected===true', G.connected() === true, `got ${G.connected()}`);
check('pre.weakestCoupling===1.0', approx(G.weakestCoupling(), 1.0), `got ${G.weakestCoupling()}`);
const preB = G.isolationBoundary();
check('pre.boundary has exactly 1 edge', Array.isArray(preB) && preB.length === 1, `got ${JSON.stringify(preB)}`);
if (Array.isArray(preB) && preB.length === 1) {
  check(
    'pre.boundary endpoints === {coordinator, review-gate}',
    setEq(endpointSet(preB[0]), 'coordinator', 'review-gate'),
    JSON.stringify([preB[0].source, preB[0].target]),
  );
  check('pre.boundary weight === 1.0', approx(preB[0].weight, 1.0), `got ${preB[0].weight}`);
}

// --- degrade merge-resolver's pendant 3.0 -> 0.01 ---
G.degrade('researcher', 'merge-resolver', 0.01);

// --- post-degrade: global min cut shifts to the merge-resolver pendant @ 0.01 ---
check('post.connected===true', G.connected() === true, `got ${G.connected()}`);
check('post.weakestCoupling===0.01', approx(G.weakestCoupling(), 0.01), `got ${G.weakestCoupling()}`);
const postB = G.isolationBoundary();
check('post.boundary has exactly 1 edge', Array.isArray(postB) && postB.length === 1, `got ${JSON.stringify(postB)}`);
if (Array.isArray(postB) && postB.length === 1) {
  check(
    'post.boundary endpoints === {researcher, merge-resolver}',
    setEq(endpointSet(postB[0]), 'researcher', 'merge-resolver'),
    JSON.stringify([postB[0].source, postB[0].target]),
  );
  check('post.boundary weight === 0.01', approx(postB[0].weight, 0.01), `got ${postB[0].weight}`);
}

if (failures === 0) {
  console.log('T1 GREEN: all assertions passed (isolation boundary shifted coordinator/review-gate@1.0 -> researcher/merge-resolver@0.01)');
  process.exit(0);
} else {
  console.log(`T1 RED: ${failures} assertion(s) failed`);
  process.exit(1);
}
