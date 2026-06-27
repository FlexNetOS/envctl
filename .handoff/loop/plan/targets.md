# Planning backlog (auto-derived from meta/.meta.yaml; owner-overridable)
#
# DONE scope = every active target planned + verified. First run is CAPPED to rusty-idd, then HAND OFF.
# Active rows below use the gate's strict one-slug-per-row, kebab-case form. The full fleet backlog
# (incl. snake_case repo names the gate's slug regex cannot encode as rows) is the commented list at
# the bottom — informational; promote a repo to an active row when it becomes a cycle target.
#
# Legend: [ ] todo  [~] in-flight / planned-with-gaps  [x] planned+verified  [!] blocked  [!!] SUPERVISED

## Cycle 1 (this run)
- [~] rusty-idd: intent-driven control plane — planned-with-gaps (8/12 dimensions verified; perf/autoresearch/rules-policy-org/prompt-architecture analysed-not-verified — see dimensions.md + verdicts.md)

## Next (convergence priority — kebab-valid organs)
- [ ] weave: communication layer (A2A / background transport; the nervous system) — cycle-2 recommended pick (unblocks rusty-idd, envctl, harness, the agents)
- [ ] envctl: fleet environment manager + this loop's run-from
- [ ] icm: persistent memory (memory-vector-intelligence axis)
- [ ] grit: symbol-level merge/lock substrate
- [ ] handoff: continuity kernel (hf)
- [ ] lane: distributed-compute lane substrate
- [ ] shimmy: local LLM serving (ollama replacement track)
- [ ] ruvector: vector intelligence (ollama replacement track)

# --- Full fleet backlog (informational; snake_case names cannot be active rows under the gate slug regex) ---
# north-star organs (snake_case): harness_hub, prompt_hub
# meta core: meta_cli, meta_core, meta_git_lib, meta_git_cli, meta_project_cli, meta_rust_cli,
#            meta_mcp, meta_dashboard_cli, meta_plugin_protocol, meta_plugin_api, loop_lib, loop_cli
# network/agents: network-control, network_hub, atc, agent, hermes-agent, harness-agent-rs
# hubs: tool_hub, database_hub, mcp_hub, plugin_hub, hooks_hub, vault_hub, flow_hub, template_hub
# flexnetos: flexnetos_runner, flexnetos_github_app, flexnetos_wiki, flexnetos_brain, github_org
# plugins/clients: claude-plugins, claude-plugin, copilot-plugin, meta-plugins, claude-code, codex,
#                  oh-my-claudecode, oh-my-pi
# apps/tools: rtk-tokenkill, vox, n8n, obscura, obsidian-mind, lifeos, kasetto, my-wiki, ECC,
#             meta-yard, teri, assets, commands
#
# NOTE (cycle-2 evolution item): the artifact-gate slug regex ^[a-z0-9][a-z0-9-]*$ rejects the fleet's
# snake_case repo names. Either relax the regex to allow `_` or canonicalize target slugs — see
# proposed-upgrades.md. target-dag.json already carries nodes for the snake_case slugs (64 nodes).
