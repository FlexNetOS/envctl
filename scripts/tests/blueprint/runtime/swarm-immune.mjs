// canonical: scripts/tests/blueprint/runtime/swarm-immune.mjs
// Swarm immune system — ruvector MinCut wired against the agent-coordination graph.
// The min-cut edge set is the isolation boundary: where to sever a hallucinating /
// failing agent before its corrupted logic cascades. Native ruvector-mincut
// (libruvector_mincut_node.so as .node).
//
// R1 fix (V4's 3 call-shape bugs against a proven native API):
//   1. the addon indexes nodes by u32 ids — the documented string-named agent
//      surface must be mapped names -> u32 before MinCut.fromEdges (the live
//      wrapper passed strings straight in => napi "NumberExpected").
//   2. cutEdges() / isConnected() are METHODS, not properties (the live wrapper
//      returned the bound functions instead of calling them).
//   3. edge weight change (degrade) is done by REBUILDING from a maintained edge
//      list, not deleteEdge()+insertEdge(). The native delete-then-reinsert of the
//      same vertex pair is nondeterministic here: insertEdge throws
//      "Edge already exists: (u, v)" for a just-deleted edge on a
//      process-hash-seed-dependent fraction of runs, and when it throws it leaves
//      the graph in an inconsistent state (cutEdges() shows the new edge but
//      minCut() reports value 0). fromEdges is the reliable primitive and yields a
//      correct, deterministic min cut every run — proven across repeated fresh
//      processes. The frozen surface immuneGraph(edges) ->
//      {g, isolationBoundary(), weakestCoupling(), degrade(u,v,w), connected()} is
//      preserved exactly; only degrade's internal mechanism changed.
import { createRequire } from 'module';
const require = createRequire(import.meta.url);
const { MinCut } = require('/home/flexnetos/lifeos/var/lib/ruvector/mincut/ruvector_mincut.node');

// edges: [srcAgent, dstAgent, couplingStrength] — srcAgent/dstAgent are string agent names.
export function immuneGraph(edges) {
  const ids = new Map();
  const names = [];
  const idOf = (name) => {
    if (!ids.has(name)) {
      ids.set(name, names.length);
      names.push(name);
    }
    return ids.get(name);
  };
  const nameOf = (id) => names[id] ?? id;

  // Maintain the numeric edge list so degrade() can rebuild deterministically.
  let numericEdges = edges.map(([u, v, w]) => [idOf(u), idOf(v), w]);
  let g = MinCut.fromEdges(numericEdges.map((e) => [...e]));

  return {
    // Always the live graph (degrade rebuilds it), exposed as a getter so a caller
    // that reads `.g` after a degrade sees the current graph, not a stale handle.
    get g() {
      return g;
    },
    isolationBoundary: () =>
      g.cutEdges().map((e) => ({ ...e, source: nameOf(e.source), target: nameOf(e.target) })),
    weakestCoupling: () => g.minCutValue,
    degrade: (u, v, newWeight) => {
      const a = idOf(u);
      const b = idOf(v);
      numericEdges = numericEdges.map(([x, y, w]) =>
        (x === a && y === b) || (x === b && y === a) ? [x, y, newWeight] : [x, y, w],
      );
      g = MinCut.fromEdges(numericEdges.map((e) => [...e]));
    },
    connected: () => g.isConnected(),
  };
}
