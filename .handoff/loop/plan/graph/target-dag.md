# Target dependency graph (TDP) — fleet convergence run

Task-Decoupled Planning (TDP) DAG for the fleet-convergence planning loop. Each member repo is an
organ converging into one fabric; this graph schedules *which organ to plan next* by dependency, not
by first-unchecked-line. Built from `.handoff/loop/plan/targets.md` (63 target slugs) with convergence
edges from the north-star FRAME and real `meta/.meta.yaml` `depends_on`/`provides` data.

- Nodes: **63** (one per targets.md slug — fail-closed: a slug with no node fails the gate)
- Edges: **30** (only where convergence genuinely requires it; isolated repos have empty deps)
- Machine-readable companion: `.handoff/loop/plan/graph/target-dag.json`
- Node status this run: `rusty-idd` = **in-flight** (cycle 1, owner-capped); all others **pending**

## FRAME — substrate organs are upstream

The north-star organs are the shared substrates many repos depend on:

| organ | role | downstream (who depends on it) |
|---|---|---|
| `weave` | Communication layer - A2A/background transport; the fabric's nervous system | `rusty-idd`, `envctl`, `atc`, `agent`, `hermes-agent`, `harness-agent-rs` |
| `icm` | Persistent memory substrate (memory-vector-intelligence axis) | `rusty-idd`, `ruvector` |
| `handoff` | Continuity kernel (hf) - witnessed ledger + rendered handoff packets | `rusty-idd`, `harness_hub`, `envctl`, `harness-agent-rs` |
| `grit` | Symbol-level merge/lock substrate | (none yet) |
| `prompt_hub` | Prompt / north-star intent store (upstream intent source) | `rusty-idd` |
| `lane` | Distributed-compute lane substrate (the spine) | `network-control` |

`rusty-idd` (control plane) depends on `weave`+`handoff`+`icm` (+`prompt_hub` for intent).
`envctl`/`harness_hub` relate to the loop itself (continuity kernel + comms).
`shimmy`/`ruvector` = local-LLM + vector-intelligence track. The `meta_*`/`loop_*` family and
`network-control` carry their real declared `depends_on` from `.meta.yaml`.

## ASCII overview (substrate -> dependents)

```
           [ substrate organs — convergence roots ]
   weave        handoff        icm        prompt_hub     grit   lane
     |   \        |  \          |  \          |            .      .
     |    \       |   \         |   \         |          (merge) (spine)
     v     v      v    v        v    v        v
  agent  atc   rusty-idd <------+----+--------+   ruvector<--icm
  hermes-agent     ^  ^                          ^
  harness-agent-rs/|  |                          |
  envctl<--weave+handoff   harness_hub<--handoff  network-control<--lane,ruvector

           [ meta plugin family — declared depends_on ]
   meta_plugin_protocol --> meta_cli --> {meta_git_cli, meta_project_cli, meta_rust_cli, meta_mcp}
   meta_core ------------/      ^
   loop_lib ------------/       |  meta_git_lib --> meta_git_cli/meta_project_cli
   loop_lib --> loop_cli        meta_plugin_protocol --> meta_dashboard_cli

   [ isolated catalogs/apps — empty deps ]  *_hub, flexnetos_*, claude-*, codex,
   vox, n8n, obscura, lifeos, kasetto, my-wiki, ECC, meta-yard, teri, assets, commands, ...
```

## ready-set scheduling (topological)

A node is **ready** when every dep is `done` (or `blocked` with a qualified gap that does not
invalidate it). The supervisor picks from the ready set in targets.md priority order, runs one
node per cycle with **node-scoped context** (each node reads only `targets.md`, `loop_state.md`,
and its own `graph/<id>.graph.md` — see `context_paths` in the JSON), then re-derives the ready set.

**Cycle 1 (this run):** owner-capped to `rusty-idd` (the why/what entry point). It is force-picked
ahead of its substrate deps by the documented first-run cap (`cycle_budget=1`), then HAND OFF.
After cycle 1, `rusty-idd` = `done`.

**Cycle 2 ready-set** (47 nodes — every node whose deps are now satisfied). Highest-
priority ready organ first (north-star ordering from targets.md):

```
  RECOMMENDED next pick : weave   (nervous system — unblocks rusty-idd path, envctl, harness_hub, all agents)
  ready (substrate/axis): weave, prompt_hub, icm, grit, handoff, lane, shimmy
  ready (fleet roots)   : meta_core, meta_git_lib, meta_plugin_protocol, meta_plugin_api, loop_lib, network_hub, tool_hub, database_hub, mcp_hub, plugin_hub, hooks_hub, vault_hub
                          flow_hub, template_hub, flexnetos_runner, flexnetos_github_app, flexnetos_wiki, flexnetos_brain, github_org, claude-plugins, claude-plugin, copilot-plugin, meta-plugins, claude-code
                          codex, oh-my-claudecode, oh-my-pi, rtk-tokenkill, vox, n8n, obscura, obsidian-mind, lifeos, kasetto, my-wiki, ECC, meta-yard, teri, assets, commands
```

**Cycle 3+ unlocks** (become ready once their deps are done): once `weave`+`handoff`+`icm`+`prompt_hub`
are done -> `rusty-idd` was cycle1; `envctl`,`harness_hub`,`ruvector`, the agents (`agent`,`atc`,
`hermes-agent`,`harness-agent-rs`) become ready. Once `meta_plugin_protocol`+`meta_core`+`loop_lib`
done -> `meta_cli`; then the `meta_*_cli` plugins; `meta_cli`+`lane`+`ruvector` -> `network-control`.

## Topological order (Kahn, deterministic)

```
L0: weave, prompt_hub, icm, grit, handoff, lane, shimmy, meta_core, meta_git_lib, meta_plugin_protocol, meta_plugin_api, loop_lib, network_hub, tool_hub, database_hub, mcp_hub, plugin_hub, hooks_hub, vault_hub, flow_hub, template_hub, flexnetos_runner, flexnetos_github_app, flexnetos_wiki, flexnetos_brain, github_org, claude-plugins, claude-plugin, copilot-plugin, meta-plugins, claude-code, codex, oh-my-claudecode, oh-my-pi, rtk-tokenkill, vox, n8n, obscura, obsidian-mind, lifeos, kasetto, my-wiki, ECC, meta-yard, teri, assets, commands
L1: rusty-idd, harness_hub, envctl, ruvector, meta_dashboard_cli, meta_cli, loop_cli, atc, agent, hermes-agent, harness-agent-rs
L2: meta_git_cli, meta_project_cli, meta_rust_cli, meta_mcp, network-control
```

## SELF-REVISION — localized replanning rule

When a verifier **refutes or qualifies** an upstream claim, the supervisor appends a `SELF-REVISION`
row and marks **only the impacted downstream subgraph** `pending` again — a node may revise its own
sub-plan without re-running the whole DAG. Verified, unrelated nodes are preserved (never reset).
If a graph query (e.g. `git-kb code query entrypoints`) returns empty, that node records an
`INCONCLUSIVE` finding and self-revises only its declared `downstream`.

| id | trigger | affected (downstream only) | action |
|---|---|---|---|
| sr-001 | cycle-1 rusty-idd finding (loop_state.md OPEN: no fleet-level NORTH-STAR.md every repo reads) | `prompt_hub`, `envctl`, `harness_hub` | revise only these downstream specs to bind to the shared north-star artifact as data; do not reset unrelated nodes |

`sr-001` is grounded in `loop_state.md` (the cycle-1 OPEN finding: no fleet-level NORTH-STAR.md that
every repo reads). That changes the *intent-binding* spec for the loop trio only — `prompt_hub`
(where the shared north-star artifact lives), `envctl` and `harness_hub` (how repos bind to it as
data) — so those three downstream specs are revised; all other 60 nodes are untouched.

## Node-scoped context (the TDP discipline)

Each node may read **only**: `.handoff/loop/plan/targets.md`, `.handoff/loop/plan/loop_state.md`,
and its own `.handoff/loop/plan/graph/<id>.graph.md` snapshot (see each node's `context_paths` in
`target-dag.json`). This keeps planning of one organ decoupled from the rest of the fleet and makes
self-revision strictly local.

