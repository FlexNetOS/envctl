# Fleet North-Star Map — meta as ONE converging system

**Run:** Planning Engineer Loop FIRST RUN · fleet-mapper · 2026-06-26
**Frame:** meta is one system mid-assembly. Organs converge toward the AXES (persistent
memory/vector-intelligence · constant auto-research · rules/policy/agent-org + A2A · Rust+Lua
runtime · distributed compute · multi-vendor local+cloud mesh).
**Laws:** read-only · fail-closed (absent file = finding) · every UNCONFIRMED edge marked.

**Fleet index:** `/home/drdave/Desktop/meta/.meta.yaml` — **68 members** enumerated
(`grep -cE '^  [A-Za-z0-9_.-]+:$'` = 68). 2 are nested meta repos (`meta-yard` L166,
`mcp_hub` L245); 3 are paths inside the meta root repo (`claude-plugin`, `copilot-plugin`,
`github_org`). `ruvector` is **crates-only** adoption (.meta.yaml L292+, `adopt: all-crates`,
~200 crates listed in `provides:`).

---

## 1. Confirmed shared substrates (the fabric)

| Substrate | Owner repo | Plane | Evidence |
|---|---|---|---|
| **hf + `.handoff` Continuity Ledger Kernel** | `handoff/` | Execution (who/when/proof) | `handoff/FLEET_GUIDE.md:1-25` ("Continuity Ledger Kernel `hf` + `.handoff`"); **20 of 68 repos carry `.handoff/`** (enumerated below) |
| **git-kb (`.kb/`)** | meta root + per-repo | Planning (why/what/next) | `handoff/FLEET_GUIDE.md` §1 "Two planes" table; `.kb/AGENTS.md` |
| **weave** | `weave/` | Communication (A2A / heartbeat / leases) | `weave/README.md:1-8` "agent-to-agent session mesh"; root `CLAUDE.md` "weave is only an observable heartbeat" |
| **icm** | `icm/` | Persistent memory | `icm/README.md` "Permanent memory for AI agents… MCP native" |
| **grit** | `grit/` | Parallel-agent merge coordination | `grit/README.md` "Zero merge conflicts, any number of parallel agents" |
| **prompt_hub** | `prompt_hub/` | Front door (intent intake) | `prompt_hub/README.md` "prompt management for LLM agent swarms"; **seam confirmed** in `handoff/hf/src/prompt_hub.rs:1-15` (`hf prompt-hub "<vibe>"` → WorkOrder) |
| **envctl** | `envctl/` | Environment + credential injection | `envctl/README.md:1-13` "agentic environment manager… no system-depth installs" |

**Repos carrying `.handoff/` (hf-wired, 20):** network-control, handoff, loop_lib,
meta_git_lib, meta_git_cli, meta_mcp, meta_dashboard_cli, agent, envctl, flexnetos_runner,
ECC, prompt_hub, weave, lane, lifeos, harness_hub, harness-agent-rs, **rusty-idd**, teri,
meta-ruvector. → **48 of 68 members are NOT on the kernel** (fail-closed finding).

---

## 2. Organ table (north-star organs deep-read; rest tag-classified from .meta.yaml)

State legend: **core-substrate** = the fabric others ride · **partially-wired** = on kernel
or has a confirmed edge · **standalone** = no confirmed fleet edge yet.

### North-star organs (deep-read)
| Organ | Role (1 line, from README/AGENTS) | Fabric axis/layer | State | Gap to fabric |
|---|---|---|---|---|
| **rusty-idd** | Rust binary unifying IDD control plane + OpenSpec lifecycle + task TUI (`rusty-idd/README.md:3`) | Control plane (why/what) — the FRAME's intent organ | partially-wired | **No confirmed code edge to hf/weave.** Front-door intake is actually owned by prompt_hub→hf, not rusty-idd. Control plane is split 3 ways (see Gap #2). Refs to weave/hf appear only in `.idd/knowledge/*.json`, not in code. |
| **weave** | A2A session mesh + native tmux/zellij injector, one static Rust binary (`weave/README.md:1-8`) | Communication / A2A (nervous system) | core-substrate | Used as **heartbeat only** in handoff harness, not as the load-bearing transport for hf leases everywhere; A2A reach beyond Claude-Code sessions UNCONFIRMED |
| **handoff** | Continuity Ledger Kernel; `hf resume` is first command; CECCA/NOA executive, Gold-World baseline (`handoff/AGENTS.md`, `NORTH-STAR.md`) | Execution substrate + **owns FLEET_GUIDE.md** | core-substrate | Carries its OWN `NORTH-STAR.md` (kernel doctrine) distinct from meta-root `NORTH-STAR.md` (vision) — two fleet docs, neither propagated to members (Gap #1). `hf/src/prompt_hub.rs` front-door seam exists |
| **envctl** | Agentic env manager; brings every tool/provider/credential to declared state INTO meta (`envctl/README.md:1-13`) | Multi-vendor mesh + credential plane (Rust runtime) | core-substrate | Owns the "envctl injects credentials on demand" promise of NORTH-STAR; provider-mint seam present, but per-model credential routing across providers still maturing |
| **prompt_hub** | Prompt mgmt for LLM agent swarms; FTS5+ONNX semantic search, swarm bundles (`prompt_hub/README.md`) | Front door (intent) + memory(prompts) | partially-wired | Is the REAL front door per NORTH-STAR + `handoff/hf/src/prompt_hub.rs`, but overlaps rusty-idd's "control plane" claim → ambiguity |
| **icm** | Permanent memory for AI agents, single binary, MCP-native (`icm/README.md`) | **Memory axis** | core-substrate | One of THREE memory systems (icm/ruvector/git-kb) with no unifying recall (Gap #4) |
| **grit** | Coordination layer for parallel agents, zero merge conflicts (`grit/README.md`) | Rules/agent-org (merge governance) | partially-wired | Cross-repo adoption UNCONFIRMED — no `.grit` usage found across members in this pass |
| **lane** | Trusted HTTPS local domains + one-command public sharing (`lane/README.md`) | Network plane | partially-wired | Confirmed seam: `network-control` composes lane (.meta.yaml L5-7); `obscura↔lane` ADR-0001 (L201) |
| **shimmy** | Lightweight local OpenAI-API inference server, no deps (`shimmy/README.md`) | Local-model serving (multi-vendor mesh) | partially-wired | Named in NORTH-STAR as swarm-prediction w/ teri; wiring to envctl provider mesh UNCONFIRMED |
| **harness_hub** | Catalog of agent harnesses; registry.json SoT, Rust validator (`harness_hub/README.md`) | Rules/agent-org (harness catalog) | partially-wired | One of ~13 `*_hub` repos, most empty (Gap #5) |
| **ruvector** (`meta-ruvector/`) | ~200 vector/AI/OS/agent crates: vector compute, HNSW, RVF ledger, rvagent-a2a/acp/mcp, ruvix RustOS, Hailo/ESP32 (.meta.yaml `provides:`) | **Memory/vector-intelligence + Rust runtime + distributed compute** — "the agentic OS" (NORTH-STAR) | partially-wired (crates-only) | **Repo root README is the upstream Jujutsu README** (`meta-ruvector/README.md:3` = "Jujutsu—a version control system") — real identity lives only in `crates/` + .meta.yaml. Mis-identified root. crates-only: npm/ui/runtime out of scope |

### Tag-classified members (from .meta.yaml tags + README first line)
| Group (tag) | Members | Axis served | State |
|---|---|---|---|
| **canon meta CLI** | loop_lib, loop_cli, meta_core, meta_git_lib, meta_cli, meta_git_cli, meta_project_cli, meta_rust_cli, meta_plugin_protocol, meta_plugin_api | Rust runtime / orchestration backbone | core-substrate (explicit `depends_on` DAG in .meta.yaml) |
| **meta tooling** | meta_mcp, meta_dashboard_cli, meta-plugins | MCP + dashboard | partially-wired (dashboard→`envctl dashboard --json`, L74) |
| **ai/agent runtime** | agent, atc, hermes-agent, harness-agent-rs, teri, ruflo | Rules/agent-org + Rust runtime | standalone→partial (atc tagged orchestration) |
| **ai forks** | claude-code, codex, oh-my-claudecode, oh-my-pi, ECC, n8n, teri, shimmy, ruflo | dual-model lanes / automation | standalone (forks; codex=foreground lane, see Gap #3) |
| **network plane** | network-control, obscura, lane, network_hub | Network / web-egress | partially-wired (lane↔obscura↔ruvector seams) |
| **memory/knowledge** | icm, obsidian-mind, flexnetos_brain, flexnetos_wiki, my-wiki | Memory axis | mixed (icm substrate; rest docs) |
| **ops** | flexnetos_runner, flexnetos_github_app | CI/exec plane | partially-wired (runner on kernel) |
| **env/runtime** | kasetto, yazelix, ohmyzsh, agent-skills, vox | Env + voice I/O | partially (envctl-installed) |
| **hubs (catalog scaffolding)** | template_hub, flow_hub, harness_hub, network_hub, tool_hub, database_hub, mcp_hub, plugin_hub, hooks_hub, vault_hub, commands, assets, claude-plugins | Rules/org catalogs | mostly **empty placeholders** (Gap #5) |
| **hardware/distributed** | meta_hardware, ruvector(ruvix/esp32/Hailo crates) | Distributed compute (Pi/ESP32/NPU) | partial (esp32 reserved → meta-hardware) |
| **idd/docs** | rusty-idd, lifeos, prompt_hub | Control plane + front door | partial |

---

## 3. ASCII fleet architecture (CONFIRMED edges solid; UNCONFIRMED marked)

```
                         ┌───────────────────────────────────────────┐
                         │  DIRECTION IN (non-technical owner intent)  │
                         └───────────────────────┬───────────────────┘
                                                 │ vibe / what+why
                                                 v
                                       ╔═════════════════╗
                                       ║   prompt_hub    ║  FRONT DOOR
                                       ║ (intent intake) ║
                                       ╚════════╤════════╝
                  CONFIRMED: handoff/hf/src/prompt_hub.rs
                           "hf prompt-hub <vibe>" --mints--> WorkOrder
                                                 │
            rusty-idd ··· UNCONFIRMED control-plane edge ···┤  (FRAME says
          (intent/why/what)   no code link to hf/weave)     │   rusty-idd is
                                                            v   the control plane)
   PLANNING PLANE                              ╔═══════════════════════╗
   ┌──────────────┐   one-way seam (ADR-0003)  ║      hf + .handoff    ║  EXECUTION
   │  git-kb (.kb)│ ──mint card / hf sync──────▶║  Continuity Ledger    ║  KERNEL
   │ why/what/next│                             ║  (handoff/ owns it)   ║  who/when/proof
   └──────────────┘                             ╚═══╤═══════════╤═══════╝
                                                    │           │ claims+leases
                            governs (Gold World)    │           │  CONFIRMED via
                            CECCA/NOA               │           │  NORTH-STAR dest-state
                                                    │           v
                                                    │      ╔═════════╗ transports
                                                    │      ║  weave  ║ A2A / heartbeat
                                                    │      ╚════╤════╝ (CONFIRMED heartbeat;
                                                    │           │       load-bearing UNCONFIRMED)
   ┌─────────┐ remembers  ┌──────────┐             │           │
   │   icm   │◀───────────│ 20 repos │─────────────┘     ┌─────┴──────┐ coordinates
   │ memory  │  UNCONF    │ on kernel│                   │   grit     │ merge-locks
   └─────────┘  hf↔icm    └──────────┘                   │ (cross-repo│ UNCONFIRMED
        ▲                                                │  UNCONF)   │
        │ THREE memory systems, no unifying recall (Gap#4)└────────────┘
   ┌────┴──────────────────────────┐
   │ ruvector crates (vector intel, │  ── "the agentic OS" (NORTH-STAR) ──┐
   │ RVF ledger, rvagent-a2a/mcp,   │                                     │
   │ ruvix RustOS, Hailo/ESP32)     │                                     v
   └────────────────────────────────┘                          DISTRIBUTED COMPUTE
                                                                meta_hardware / Pi / ESP32 / NPU

   RUNTIME / CREDENTIAL PLANE                         NETWORK PLANE
   ┌────────────┐ installs INTO meta   ┌───────────┐ composes  ┌──────────┐
   │  envctl    │──provides creds────▶ │network-   │◀──────────│  lane    │
   │ + kasetto  │  to every model      │control    │  (.meta)  │ +obscura │ (ADR-0001)
   └─────┬──────┘                      └───────────┘           └──────────┘
         │ dashboard --json (CONFIRMED L74)
         v
   meta_dashboard_cli

   META CLI BACKBONE (explicit depends_on DAG, .meta.yaml):
   loop_lib → loop_cli ; {meta_core, plugin_protocol, loop_lib} → meta_cli
   → {meta_git_cli, meta_project_cli, meta_rust_cli} (all depend plugin_protocol)

   DUAL-MODEL (FRAME): codex=foreground / opus=background  ── NO governance doc found (Gap#3)
```

**Confirmed edges:** prompt_hub→hf (`handoff/hf/src/prompt_hub.rs`); git-kb→hf one-way
(`FLEET_GUIDE.md §1`); hf→weave leases (NORTH-STAR dest-state + root CLAUDE); network-control
composes lane+ruvector (.meta.yaml L5-7); obscura↔lane (L201); meta CLI depends_on DAG
(L44-61); meta_dashboard_cli→envctl (L74); envctl installs all tooling into meta (envctl README).

**UNCONFIRMED (do not assume):** rusty-idd↔hf/weave (only `.idd/knowledge/*.json` text, no code);
weave as load-bearing transport beyond heartbeat; grit cross-repo adoption; hf↔icm memory
integration; shimmy/teri↔envctl provider mesh.

---

## 4. Cross-cutting convergence gaps (top 5)

1. **No single fleet north-star every repo can read.** The vision lives ONLY in the meta
   ROOT repo `NORTH-STAR.md` — and since every member is an independent git repo, *no member
   can `cat` it*. Worse, `handoff/` carries a SECOND, different `NORTH-STAR.md` (kernel
   doctrine) + `FLEET_GUIDE.md` (the contract). Two competing fleet docs, neither propagated
   to the other 66 members. There is no per-repo "you are organ N of system M" pointer.

2. **The control plane is split 3 ways.** The FRAME names rusty-idd as the intent control
   plane (why/what), but the CONFIRMED front-door seam is `prompt_hub → handoff/hf WorkOrder`,
   and `handoff` itself owns intent intake (`work_order::Intent`). rusty-idd has **no confirmed
   code edge** to the execution substrate. Three organs claim "intent" with no single owner.

3. **Axes with no owning organ.** (a) **Constant auto-research** — owned only by planning
   *harness skills*, no repo/daemon. (b) **Dual-model (codex fg / opus bg)** — no governance
   doc in root `CLAUDE.md`/`NORTH-STAR.md`; `codex` is an un-wired fork. (c) **Multi-vendor
   mesh** — split across envctl(creds) + ruvector(`rvagent-backends`) + shimmy(local), no
   unifying router. (d) **Distributed compute** (mobile/wearables) — only Pi/ESP32/Hailo via
   ruvector+meta_hardware; no mobile/glasses organ.

4. **Three memory systems, no unified recall.** icm (agent memory) + ruvector (vector/HNSW +
   RVF ledger) + git-kb (code graph) each persist independently with no cross-index. The
   memory/vector-intelligence AXIS has no convergence layer.

5. **Hub sprawl + duplication.** ~13 `*_hub` repos (template/flow/network/tool/database/mcp/
   plugin/hooks/vault/commands/assets + harness_hub) — most are empty placeholders (.meta.yaml
   marks "new empty repos; content TBD"). Catalog scaffolding without content; risk of
   duplicating the one real catalog (harness_hub) across many shells. **Plus:** 48/68 members
   are not on the hf kernel, and `meta-ruvector`'s root README is still the upstream Jujutsu
   README (identity mis-set).

---

## Confidence
**Medium-high.** Substrates, the meta-CLI DAG, the prompt_hub→hf seam, and the dual fleet-doc
gap are CONFIRMED from files (cited). Edges through weave/grit/icm and rusty-idd's control-plane
role are explicitly marked UNCONFIRMED — they need code-graph verification (kb_callers across
repos) in a later cycle. Tag-classified rows lean on .meta.yaml tags + README first lines, not
deep reads.
