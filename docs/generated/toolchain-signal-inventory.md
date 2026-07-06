# envctl Toolchain Signal Inventory

This report highlights catalog rows that appear relevant to Rust and adjacent toolchain wiring, including Cargo, rustup, nix, Felix, Wild, and kache signals.

## Summary

- toolchain signals: `1163`
- env var rows scanned: `106`
- settings rows scanned: `4925`
- path rows scanned: `49`
- codedb import rows scanned: `3549`

## Signal Kinds

### Kinds

- `codedb_import`: `699`
- `env_var`: `1`
- `path`: `3`
- `setting`: `460`

## Rows

| kind | key | source | detail | value |
| --- | --- | --- | --- | --- |
| `codedb_import` | `envctl_repo_Cargo_lock` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `envctl_repo:Cargo.lock` |
| `codedb_import` | `envctl_repo_Cargo_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=93` | `envctl_repo:Cargo.toml` |
| `codedb_import` | `envctl_repo__agents_rusty-idd-adapter_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=7` | `envctl_repo:.agents/rusty-idd-adapter.md` |
| `codedb_import` | `envctl_repo__agents_skills_env-toolchain-install_SKILL_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=48` | `envctl_repo:.agents/skills/env-toolchain-install/SKILL.md` |
| `codedb_import` | `envctl_repo__agents_skills_rust-feature-impl_SKILL_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=98` | `envctl_repo:.agents/skills/rust-feature-impl/SKILL.md` |
| `codedb_import` | `envctl_repo__agents_skills_rust-feature-impl_references_kasetto-absorption_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=220` | `envctl_repo:.agents/skills/rust-feature-impl/references/kasetto-absorption.md` |
| `codedb_import` | `envctl_repo__agents_skills_rust-feature-impl_references_verification_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=148` | `envctl_repo:.agents/skills/rust-feature-impl/references/verification.md` |
| `codedb_import` | `envctl_repo__agents_skills_rust-port-inventory_SKILL_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=56` | `envctl_repo:.agents/skills/rust-port-inventory/SKILL.md` |
| `codedb_import` | `envctl_repo__agents_skills_rust-port-merge_SKILL_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=108` | `envctl_repo:.agents/skills/rust-port-merge/SKILL.md` |
| `codedb_import` | `envctl_repo__agents_skills_rust-port-parity_SKILL_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=44` | `envctl_repo:.agents/skills/rust-port-parity/SKILL.md` |
| `codedb_import` | `envctl_repo__agents_skills_rust-port-translate_SKILL_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=57` | `envctl_repo:.agents/skills/rust-port-translate/SKILL.md` |
| `codedb_import` | `envctl_repo__agents_skills_rust-port_SKILL_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=271` | `envctl_repo:.agents/skills/rust-port/SKILL.md` |
| `codedb_import` | `envctl_repo__agents_skills_rust-port_references_eject_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=30` | `envctl_repo:.agents/skills/rust-port/references/eject.md` |
| `codedb_import` | `envctl_repo__agents_skills_rust-port_references_merge-ledger_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=86` | `envctl_repo:.agents/skills/rust-port/references/merge-ledger.md` |
| `codedb_import` | `envctl_repo__agents_skills_rust-port_references_parity-ledger_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=49` | `envctl_repo:.agents/skills/rust-port/references/parity-ledger.md` |
| `codedb_import` | `envctl_repo__agents_skills_rust-port_references_runtime-constructs_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=51` | `envctl_repo:.agents/skills/rust-port/references/runtime-constructs.md` |
| `codedb_import` | `envctl_repo__agents_skills_rust-port_references_symbol-map_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=136` | `envctl_repo:.agents/skills/rust-port/references/symbol-map.md` |
| `codedb_import` | `envctl_repo__agents_skills_rust-port_scripts_eject_sh` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=shell structured_rows=56` | `envctl_repo:.agents/skills/rust-port/scripts/eject.sh` |
| `codedb_import` | `envctl_repo__agents_skills_rust-port_scripts_loop_state_template_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=22` | `envctl_repo:.agents/skills/rust-port/scripts/loop_state.template.md` |
| `codedb_import` | `envctl_repo__agents_skills_rust-port_scripts_ralph-rust-port_sh` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=shell structured_rows=37` | `envctl_repo:.agents/skills/rust-port/scripts/ralph-rust-port.sh` |
| `codedb_import` | `envctl_repo__claude_agents_rust-implementer_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=104` | `envctl_repo:.claude/agents/rust-implementer.md` |
| `codedb_import` | `envctl_repo__claude_agents_rust-port-architect_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=53` | `envctl_repo:.claude/agents/rust-port-architect.md` |
| `codedb_import` | `envctl_repo__claude_agents_rust-port-cartographer_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=75` | `envctl_repo:.claude/agents/rust-port-cartographer.md` |
| `codedb_import` | `envctl_repo__claude_agents_rust-port-cross-repo-referencer_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=55` | `envctl_repo:.claude/agents/rust-port-cross-repo-referencer.md` |
| `codedb_import` | `envctl_repo__claude_agents_rust-port-merge-integrator_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=82` | `envctl_repo:.claude/agents/rust-port-merge-integrator.md` |
| `codedb_import` | `envctl_repo__claude_agents_rust-port-parity-verifier_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=55` | `envctl_repo:.claude/agents/rust-port-parity-verifier.md` |
| `codedb_import` | `envctl_repo__claude_agents_rust-port-porter_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=57` | `envctl_repo:.claude/agents/rust-port-porter.md` |
| `codedb_import` | `envctl_repo__claude_agents_rust-port-researcher_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=62` | `envctl_repo:.claude/agents/rust-port-researcher.md` |
| `codedb_import` | `envctl_repo__claude_rusty-idd-adapter_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=7` | `envctl_repo:.claude/rusty-idd-adapter.md` |
| `codedb_import` | `envctl_repo__claude_skills_env-toolchain-install_SKILL_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=48` | `envctl_repo:.claude/skills/env-toolchain-install/SKILL.md` |
| `codedb_import` | `envctl_repo__claude_skills_rust-feature-impl_SKILL_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=98` | `envctl_repo:.claude/skills/rust-feature-impl/SKILL.md` |
| `codedb_import` | `envctl_repo__claude_skills_rust-feature-impl_references_kasetto-absorption_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=220` | `envctl_repo:.claude/skills/rust-feature-impl/references/kasetto-absorption.md` |
| `codedb_import` | `envctl_repo__claude_skills_rust-feature-impl_references_verification_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=148` | `envctl_repo:.claude/skills/rust-feature-impl/references/verification.md` |
| `codedb_import` | `envctl_repo__claude_skills_rust-port-inventory_SKILL_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=56` | `envctl_repo:.claude/skills/rust-port-inventory/SKILL.md` |
| `codedb_import` | `envctl_repo__claude_skills_rust-port-merge_SKILL_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=108` | `envctl_repo:.claude/skills/rust-port-merge/SKILL.md` |
| `codedb_import` | `envctl_repo__claude_skills_rust-port-parity_SKILL_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=44` | `envctl_repo:.claude/skills/rust-port-parity/SKILL.md` |
| `codedb_import` | `envctl_repo__claude_skills_rust-port-translate_SKILL_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=57` | `envctl_repo:.claude/skills/rust-port-translate/SKILL.md` |
| `codedb_import` | `envctl_repo__claude_skills_rust-port_SKILL_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=271` | `envctl_repo:.claude/skills/rust-port/SKILL.md` |
| `codedb_import` | `envctl_repo__claude_skills_rust-port_references_eject_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=30` | `envctl_repo:.claude/skills/rust-port/references/eject.md` |
| `codedb_import` | `envctl_repo__claude_skills_rust-port_references_merge-ledger_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=86` | `envctl_repo:.claude/skills/rust-port/references/merge-ledger.md` |
| `codedb_import` | `envctl_repo__claude_skills_rust-port_references_parity-ledger_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=49` | `envctl_repo:.claude/skills/rust-port/references/parity-ledger.md` |
| `codedb_import` | `envctl_repo__claude_skills_rust-port_references_runtime-constructs_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=51` | `envctl_repo:.claude/skills/rust-port/references/runtime-constructs.md` |
| `codedb_import` | `envctl_repo__claude_skills_rust-port_references_symbol-map_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=136` | `envctl_repo:.claude/skills/rust-port/references/symbol-map.md` |
| `codedb_import` | `envctl_repo__claude_skills_rust-port_scripts_eject_sh` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=shell structured_rows=56` | `envctl_repo:.claude/skills/rust-port/scripts/eject.sh` |
| `codedb_import` | `envctl_repo__claude_skills_rust-port_scripts_loop_state_template_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=22` | `envctl_repo:.claude/skills/rust-port/scripts/loop_state.template.md` |
| `codedb_import` | `envctl_repo__claude_skills_rust-port_scripts_ralph-rust-port_sh` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=shell structured_rows=37` | `envctl_repo:.claude/skills/rust-port/scripts/ralph-rust-port.sh` |
| `codedb_import` | `envctl_repo__codex_agents_plan-opus-bg-rusty-idd-north-star_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=5` | `envctl_repo:.codex/agents/plan-opus-bg-rusty-idd-north-star.toml` |
| `codedb_import` | `envctl_repo__codex_agents_rust-implementer_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=3` | `envctl_repo:.codex/agents/rust-implementer.toml` |
| `codedb_import` | `envctl_repo__codex_agents_rust-port-architect_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=3` | `envctl_repo:.codex/agents/rust-port-architect.toml` |
| `codedb_import` | `envctl_repo__codex_agents_rust-port-merge-integrator_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=3` | `envctl_repo:.codex/agents/rust-port-merge-integrator.toml` |
| `codedb_import` | `envctl_repo__codex_agents_rust-port-parity-verifier_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=3` | `envctl_repo:.codex/agents/rust-port-parity-verifier.toml` |
| `codedb_import` | `envctl_repo__codex_agents_rust-port-porter_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=3` | `envctl_repo:.codex/agents/rust-port-porter.toml` |
| `codedb_import` | `envctl_repo__codex_agents_rust-port-researcher_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=3` | `envctl_repo:.codex/agents/rust-port-researcher.toml` |
| `codedb_import` | `envctl_repo__codex_rusty-idd-adapter_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=7` | `envctl_repo:.codex/rusty-idd-adapter.md` |
| `codedb_import` | `envctl_repo__codex_skills_env-toolchain-install_SKILL_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=48` | `envctl_repo:.codex/skills/env-toolchain-install/SKILL.md` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_findings_architecture-rusty-idd_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=59` | `envctl_repo:.handoff/loop/plan/findings/architecture-rusty-idd.md` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_findings_autoresearch-rusty-idd_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=85` | `envctl_repo:.handoff/loop/plan/findings/autoresearch-rusty-idd.md` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_findings_distributed-compute-rusty-idd_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=132` | `envctl_repo:.handoff/loop/plan/findings/distributed-compute-rusty-idd.md` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_findings_filesystem-layout-rusty-idd_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=177` | `envctl_repo:.handoff/loop/plan/findings/filesystem-layout-rusty-idd.md` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_findings_governance-config-rusty-idd_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=63` | `envctl_repo:.handoff/loop/plan/findings/governance-config-rusty-idd.md` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_findings_memory-vector-intelligence-rusty-idd_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=91` | `envctl_repo:.handoff/loop/plan/findings/memory-vector-intelligence-rusty-idd.md` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_findings_prompt-architecture-rusty-idd_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=119` | `envctl_repo:.handoff/loop/plan/findings/prompt-architecture-rusty-idd.md` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_findings_rules-policy-org-rusty-idd_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=204` | `envctl_repo:.handoff/loop/plan/findings/rules-policy-org-rusty-idd.md` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_findings_test-strategy-rusty-idd_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=74` | `envctl_repo:.handoff/loop/plan/findings/test-strategy-rusty-idd.md` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_findings_union-handoff-rusty-idd_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=261` | `envctl_repo:.handoff/loop/plan/findings/union-handoff-rusty-idd.md` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_graph_rusty-idd_callgraph_json` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=json structured_rows=79` | `envctl_repo:.handoff/loop/plan/graph/rusty-idd.callgraph.json` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_graph_rusty-idd_diff_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=33` | `envctl_repo:.handoff/loop/plan/graph/rusty-idd.diff.md` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_graph_rusty-idd_graph_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=113` | `envctl_repo:.handoff/loop/plan/graph/rusty-idd.graph.md` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_graph_rusty-idd_json` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=json structured_rows=110` | `envctl_repo:.handoff/loop/plan/graph/rusty-idd.json` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_graph_rusty-idd_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=113` | `envctl_repo:.handoff/loop/plan/graph/rusty-idd.md` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_graph_rusty-idd_metrics_json` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=json structured_rows=83` | `envctl_repo:.handoff/loop/plan/graph/rusty-idd.metrics.json` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_graph_rusty-idd_symbols_json` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=json structured_rows=3507` | `envctl_repo:.handoff/loop/plan/graph/rusty-idd.symbols.json` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_reports_ADR-DRAFT-handoff-rusty-idd-union_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=77` | `envctl_repo:.handoff/loop/plan/reports/ADR-DRAFT-handoff-rusty-idd-union.md` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_reports_ADR-DRAFT-rusty-idd-convergence-boundary_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=78` | `envctl_repo:.handoff/loop/plan/reports/ADR-DRAFT-rusty-idd-convergence-boundary.md` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_reports_ROADMAP-rusty-idd_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=63` | `envctl_repo:.handoff/loop/plan/reports/ROADMAP-rusty-idd.md` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_reports_agent-run-ledger-rusty-idd_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=30` | `envctl_repo:.handoff/loop/plan/reports/agent-run-ledger-rusty-idd.md` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_reports_codemap-rusty-idd_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=92` | `envctl_repo:.handoff/loop/plan/reports/codemap-rusty-idd.md` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_reports_rusty-idd-plan_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=385` | `envctl_repo:.handoff/loop/plan/reports/rusty-idd-plan.md` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_reports_union-plan-handoff-rusty-idd_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=137` | `envctl_repo:.handoff/loop/plan/reports/union-plan-handoff-rusty-idd.md` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_research_rusty-idd_trends_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=189` | `envctl_repo:.handoff/loop/plan/research/rusty-idd.trends.md` |
| `codedb_import` | `envctl_repo__handoff_loop_plan_research_sources-rusty-idd_jsonl` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=jsonl structured_rows=0` | `envctl_repo:.handoff/loop/plan/research/sources-rusty-idd.jsonl` |
| `codedb_import` | `envctl_repo__handoff_loop_rust-port_HANDOFF_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=70` | `envctl_repo:.handoff/loop/rust-port/HANDOFF.md` |
| `codedb_import` | `envctl_repo__handoff_loop_rust-port_baseline_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=14` | `envctl_repo:.handoff/loop/rust-port/baseline.md` |
| `codedb_import` | `envctl_repo__handoff_loop_rust-port_loop_state_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=103` | `envctl_repo:.handoff/loop/rust-port/loop_state.md` |
| `codedb_import` | `envctl_repo__handoff_loop_rust-port_merge-ledger_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=156` | `envctl_repo:.handoff/loop/rust-port/merge-ledger.md` |
| `codedb_import` | `envctl_repo__handoff_loop_rust-port_parity-ledger_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=340` | `envctl_repo:.handoff/loop/rust-port/parity-ledger.md` |
| `codedb_import` | `envctl_repo__handoff_loop_rust-port_reports_inventory_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=52` | `envctl_repo:.handoff/loop/rust-port/reports/inventory.md` |
| `codedb_import` | `envctl_repo__handoff_loop_rust-port_reports_research_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=115` | `envctl_repo:.handoff/loop/rust-port/reports/research.md` |
| `codedb_import` | `envctl_repo__handoff_loop_rust-port_target-architecture_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=32` | `envctl_repo:.handoff/loop/rust-port/target-architecture.md` |
| `codedb_import` | `envctl_repo_agent-skills_env-toolchain-install_SKILL_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=48` | `envctl_repo:agent-skills/env-toolchain-install/SKILL.md` |
| `codedb_import` | `envctl_repo_ci_gates_cargo-audit_sh` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=shell structured_rows=41` | `envctl_repo:ci/gates/cargo-audit.sh` |
| `codedb_import` | `envctl_repo_crates_agent-env_Cargo_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=17` | `envctl_repo:crates/agent-env/Cargo.toml` |
| `codedb_import` | `envctl_repo_crates_cli_Cargo_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=19` | `envctl_repo:crates/cli/Cargo.toml` |
| `codedb_import` | `envctl_repo_crates_engine_Cargo_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=25` | `envctl_repo:crates/engine/Cargo.toml` |
| `codedb_import` | `envctl_repo_crates_gui_Cargo_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=10` | `envctl_repo:crates/gui/Cargo.toml` |
| `codedb_import` | `envctl_repo_crates_secretctl_Cargo_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=25` | `envctl_repo:crates/secretctl/Cargo.toml` |
| `codedb_import` | `envctl_repo_crates_secretd_Cargo_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=62` | `envctl_repo:crates/secretd/Cargo.toml` |
| `codedb_import` | `envctl_repo_crates_secrets-engine_Cargo_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=51` | `envctl_repo:crates/secrets-engine/Cargo.toml` |
| `codedb_import` | `envctl_repo_crates_secrets-proto_Cargo_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=12` | `envctl_repo:crates/secrets-proto/Cargo.toml` |
| `codedb_import` | `envctl_repo_crates_secrets-store-libsql_Cargo_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=19` | `envctl_repo:crates/secrets-store-libsql/Cargo.toml` |
| `codedb_import` | `envctl_repo_docs_secrets_research_07-rustls-mitm-ca_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=212` | `envctl_repo:docs/secrets/research/07-rustls-mitm-ca.md` |
| `codedb_import` | `envctl_repo_docs_secrets_research_11-ca-trust-wiring_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=195` | `envctl_repo:docs/secrets/research/11-ca-trust-wiring.md` |
| `codedb_import` | `envctl_repo_manifest_cognitum-seed-trust_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=20` | `envctl_repo:manifest/cognitum-seed-trust.toml` |
| `codedb_import` | `envctl_repo_manifest_components_d_epic-h-toolchains_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=207` | `envctl_repo:manifest/components.d/epic-h-toolchains.toml` |
| `codedb_import` | `envctl_repo_manifest_nix-yazelix_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=126` | `envctl_repo:manifest/nix-yazelix.toml` |
| `codedb_import` | `envctl_repo_manifest_rusty-idd_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=19` | `envctl_repo:manifest/rusty-idd.toml` |
| `codedb_import` | `envctl_repo_rust-toolchain_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=2` | `envctl_repo:rust-toolchain.toml` |
| `codedb_import` | `nix_store_03x27r1ylbdway3z1ifilvxfrkc7ip96-yazelix-runtime-release-contracts` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=plain_or_binary structured_rows=0` | `nix_store:03x27r1ylbdway3z1ifilvxfrkc7ip96-yazelix-runtime-release-contracts` |
| `codedb_import` | `nix_store_07xq44ak3r71sycsc70c7br4ypa742gd-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:07xq44ak3r71sycsc70c7br4ypa742gd-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_0aanqld8d96mpahzkh4n131c3i0q1b74-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:0aanqld8d96mpahzkh4n131c3i0q1b74-yzx.drv` |
| `codedb_import` | `nix_store_0gccq4z4ik2zn853vcmq6918ml018p0j-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:0gccq4z4ik2zn853vcmq6918ml018p0j-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_0hzzk6za4s1n3qrxm1wpbgyk2w2cmjzv-yazelix-runtime-release-contracts` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=plain_or_binary structured_rows=0` | `nix_store:0hzzk6za4s1n3qrxm1wpbgyk2w2cmjzv-yazelix-runtime-release-contracts` |
| `codedb_import` | `nix_store_0mcrx7bxrdjndjyr5l1y1kyk1p2zcr46-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:0mcrx7bxrdjndjyr5l1y1kyk1p2zcr46-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_0r27lydayjd4qwvdqsl89xsdbnpi397b-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:0r27lydayjd4qwvdqsl89xsdbnpi397b-yazelix-runtime` |
| `codedb_import` | `nix_store_0ralas458g47m1ykqkq7fmckk8qxpfvm-yazelix-zellij-popup-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:0ralas458g47m1ykqkq7fmckk8qxpfvm-yazelix-zellij-popup-0.1.0.drv` |
| `codedb_import` | `nix_store_0xsv47ii61g9pk9rl0k6nai2v58mk3mx-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:0xsv47ii61g9pk9rl0k6nai2v58mk3mx-yzx` |
| `codedb_import` | `nix_store_15z8xpqxk3fy2pwv396vr6pgys5xpp3i-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:15z8xpqxk3fy2pwv396vr6pgys5xpp3i-yzx.drv` |
| `codedb_import` | `nix_store_16xp6y8mh7kkwmrf7yvqcchldx491bnx-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:16xp6y8mh7kkwmrf7yvqcchldx491bnx-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_17hn3klllllxrs3axlmabn5v54p294wg-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:17hn3klllllxrs3axlmabn5v54p294wg-yzx` |
| `codedb_import` | `nix_store_1cxn0j0ax1qfmvpjgl8gzdh3y6qv7hmz-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:1cxn0j0ax1qfmvpjgl8gzdh3y6qv7hmz-yazelix-runtime` |
| `codedb_import` | `nix_store_1fkkhldzf7d0y2xc0dcd74axq07m6dc5-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:1fkkhldzf7d0y2xc0dcd74axq07m6dc5-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_1j8nf8diqfavvqg9y5qcrzrl4s1ipll2-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:1j8nf8diqfavvqg9y5qcrzrl4s1ipll2-yzx` |
| `codedb_import` | `nix_store_1nsb6s6gpxz81gi7wj8jpa1l7ha5jwh8-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:1nsb6s6gpxz81gi7wj8jpa1l7ha5jwh8-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_1q375sbl1yyivl3lhhi6w0vm6q53wbw9-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:1q375sbl1yyivl3lhhi6w0vm6q53wbw9-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_1rvmrraiwn7yb3q696ffywi0kbh6swic-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:1rvmrraiwn7yb3q696ffywi0kbh6swic-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_1sml5pbacn2figrnnr6h41fqq91ikg9b-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:1sml5pbacn2figrnnr6h41fqq91ikg9b-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_1v1mg9rgcl28960kwy8xw6vqlcbw82ri-yazelix_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:1v1mg9rgcl28960kwy8xw6vqlcbw82ri-yazelix.drv` |
| `codedb_import` | `nix_store_1yzx64fbbgn3slprk3biq4zwjs23x7yi-Glob-0_10_2_tar_gz_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:1yzx64fbbgn3slprk3biq4zwjs23x7yi-Glob-0.10.2.tar.gz.drv` |
| `codedb_import` | `nix_store_202qhvbmazcvffl19xkrn9y9sf5g51r4-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:202qhvbmazcvffl19xkrn9y9sf5g51r4-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_247rlvqaizfi6jdvq4cv62wjlrrj4hbs-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:247rlvqaizfi6jdvq4cv62wjlrrj4hbs-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_27f7kd4w88xl525gx67jpfr32lf7nahg-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:27f7kd4w88xl525gx67jpfr32lf7nahg-yazelix-runtime` |
| `codedb_import` | `nix_store_283s0abvn212qhdpvj5m76b7i1snl9hn-yazelix-helix-25_7_1` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:283s0abvn212qhdpvj5m76b7i1snl9hn-yazelix-helix-25.7.1` |
| `codedb_import` | `nix_store_2ap005gv70gmmxx2cjy0948dl1lc0sx8-yazelix-package-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:2ap005gv70gmmxx2cjy0948dl1lc0sx8-yazelix-package-source` |
| `codedb_import` | `nix_store_2b0jxqhb58wph27ix86hhdimb4isaxvy-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:2b0jxqhb58wph27ix86hhdimb4isaxvy-yazelix-runtime` |
| `codedb_import` | `nix_store_2fhrs93782sizj8qhj5467x3yiyv647k-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:2fhrs93782sizj8qhj5467x3yiyv647k-yazelix-runtime` |
| `codedb_import` | `nix_store_2qa7irnhc1xdi77yhdv0a0gy2bii7g4p-yazelix-package-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:2qa7irnhc1xdi77yhdv0a0gy2bii7g4p-yazelix-package-source` |
| `codedb_import` | `nix_store_2wg1qd1q8kaic3nz252w4vddvncs5slh-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:2wg1qd1q8kaic3nz252w4vddvncs5slh-yzx.drv` |
| `codedb_import` | `nix_store_2y3vib6nm7y7da7vmmx69p81v509dmv2-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:2y3vib6nm7y7da7vmmx69p81v509dmv2-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_2z6s2p95glwv204fjyqd4c61czrh066m-yazelix-runtime-release-contracts` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=plain_or_binary structured_rows=0` | `nix_store:2z6s2p95glwv204fjyqd4c61czrh066m-yazelix-runtime-release-contracts` |
| `codedb_import` | `nix_store_2zp6hfp9fs4ms98j9v7jviqifga95773-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:2zp6hfp9fs4ms98j9v7jviqifga95773-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_33gj16j3wrdxfzp6pv08mj5sx9x9c6vx-yazelix_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:33gj16j3wrdxfzp6pv08mj5sx9x9c6vx-yazelix.drv` |
| `codedb_import` | `nix_store_34fy0s65zxb0771kg1im75594ij4130z-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:34fy0s65zxb0771kg1im75594ij4130z-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_35lhra7hy9rn4991zklfv047f6sqvmc4-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:35lhra7hy9rn4991zklfv047f6sqvmc4-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_36fifkgyadm5rk4kqmcw3x8w4cy8kz88-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:36fifkgyadm5rk4kqmcw3x8w4cy8kz88-yzx` |
| `codedb_import` | `nix_store_3iwv9bz1i4p7bigxxbr8kj394hvj3lv9-yazelix-kgp-package-contracts` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=plain_or_binary structured_rows=0` | `nix_store:3iwv9bz1i4p7bigxxbr8kj394hvj3lv9-yazelix-kgp-package-contracts` |
| `codedb_import` | `nix_store_3kij26lf47y5b4lc22fwpajiqyr6n1ik-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:3kij26lf47y5b4lc22fwpajiqyr6n1ik-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_3qhsakkqdinrpqj00fl6vkw62nla2riv-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:3qhsakkqdinrpqj00fl6vkw62nla2riv-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_3x713lncshym5j185azqkb6jk1pvncaa-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:3x713lncshym5j185azqkb6jk1pvncaa-yzx.drv` |
| `codedb_import` | `nix_store_3x86inzgnn2087q3ja4jhhan8xqgzis7-yazelix-helix-25_7_1_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:3x86inzgnn2087q3ja4jhhan8xqgzis7-yazelix-helix-25.7.1.drv` |
| `codedb_import` | `nix_store_44c5xz1kjn7kjc3z3d9bvg6jzbrq0ny4-yazelix-zellij-pane-orchestrator-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:44c5xz1kjn7kjc3z3d9bvg6jzbrq0ny4-yazelix-zellij-pane-orchestrator-0.1.0` |
| `codedb_import` | `nix_store_48mldwb2x29nlq3qqzf55hgqkl3bjjq4-yazelix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:48mldwb2x29nlq3qqzf55hgqkl3bjjq4-yazelix` |
| `codedb_import` | `nix_store_4dgcfp9j2059r8733b290g1yq05pyllq-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:4dgcfp9j2059r8733b290g1yq05pyllq-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_4dvlvkil2jj8b6q7g2qsd8kxwj1s2qn5-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:4dvlvkil2jj8b6q7g2qsd8kxwj1s2qn5-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_4gdrwyn4ll5albw3r83zdwlz70rsjjd4-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:4gdrwyn4ll5albw3r83zdwlz70rsjjd4-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_4hbbphhhji7b0qzpy4gx8snigc8kyzxm-crate-munge_macro-0_4_7_tar_gz_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:4hbbphhhji7b0qzpy4gx8snigc8kyzxm-crate-munge_macro-0.4.7.tar.gz.drv` |
| `codedb_import` | `nix_store_4l2mqn6qxg911xadvyfyv9msy9h35nyh-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:4l2mqn6qxg911xadvyfyv9msy9h35nyh-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_4m2j04p463rkpzy3cn59wg8pgjjbgfb4-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:4m2j04p463rkpzy3cn59wg8pgjjbgfb4-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_4xad76rrsx45js3ga1ibm0f4wp6y0mq4-yazelix-package-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:4xad76rrsx45js3ga1ibm0f4wp6y0mq4-yazelix-package-source` |
| `codedb_import` | `nix_store_56dmrmq28m142af3rl1l0w7xfsnkvp3c-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:56dmrmq28m142af3rl1l0w7xfsnkvp3c-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_5b4v60fh58wqc85fis9gri6mbbdkgc9a-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:5b4v60fh58wqc85fis9gri6mbbdkgc9a-yzx` |
| `codedb_import` | `nix_store_5dali2krp6pfbbhrg0bzsjrzvsxcacxh-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:5dali2krp6pfbbhrg0bzsjrzvsxcacxh-yzx` |
| `codedb_import` | `nix_store_5k5nb8i9ccr5c0k6q396sbyzxvhlczi0-crate-ryu-1_0_20_tar_gz_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:5k5nb8i9ccr5c0k6q396sbyzxvhlczi0-crate-ryu-1.0.20.tar.gz.drv` |
| `codedb_import` | `nix_store_5ndxjy5r3gjcxiw8qgw4lac7mnyh5biz-yazelix_zellij_bar-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:5ndxjy5r3gjcxiw8qgw4lac7mnyh5biz-yazelix_zellij_bar-0.1.0.drv` |
| `codedb_import` | `nix_store_5wxvz2pj0qa5zhhgpspsdfk4zy4bx4vw-yazelix_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:5wxvz2pj0qa5zhhgpspsdfk4zy4bx4vw-yazelix.drv` |
| `codedb_import` | `nix_store_63l7a91iprf777i0yk2f4c59256z7w34-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:63l7a91iprf777i0yk2f4c59256z7w34-yzx` |
| `codedb_import` | `nix_store_63wmj5ay1a94acllavh1mwl5lhpr2shw-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:63wmj5ay1a94acllavh1mwl5lhpr2shw-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_64avnkrkiw9sm92kzmdmlhpnc3czanfb-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:64avnkrkiw9sm92kzmdmlhpnc3czanfb-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_6549gy7dw9nc8b9zvzivyni9hfkb9xll-yazelix-cursors-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:6549gy7dw9nc8b9zvzivyni9hfkb9xll-yazelix-cursors-0.1.0` |
| `codedb_import` | `nix_store_65vi6r5brsxmdv38hnsmbl198bn883yc-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:65vi6r5brsxmdv38hnsmbl198bn883yc-yazelix-runtime` |
| `codedb_import` | `nix_store_67957ghd0v09pmjqhjvcjl2m21schigj-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:67957ghd0v09pmjqhjvcjl2m21schigj-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_67ccqyvg3vmxs608r76rpl6x8msii3bj-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:67ccqyvg3vmxs608r76rpl6x8msii3bj-yzx` |
| `codedb_import` | `nix_store_67dl5ayk18fr02fqi0k3c4cv8wi3xdq9-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:67dl5ayk18fr02fqi0k3c4cv8wi3xdq9-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_68z2sirwwnh2yg2la76l1yvkh05ya3g9-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:68z2sirwwnh2yg2la76l1yvkh05ya3g9-yzx.drv` |
| `codedb_import` | `nix_store_6cma40gdhj1dlmdbsak5x7a94d99fs8l-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:6cma40gdhj1dlmdbsak5x7a94d99fs8l-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_6fqsd285q53nzsf8zmf2k6wldy6jjd93-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:6fqsd285q53nzsf8zmf2k6wldy6jjd93-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_6i0w8d5p0j5f2km3xiqd40jrpji1iw2l-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:6i0w8d5p0j5f2km3xiqd40jrpji1iw2l-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_6l1rzvbmy6v6468f9g76fchb7p5d8k51-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:6l1rzvbmy6v6468f9g76fchb7p5d8k51-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_6nq1qgndd2yp6v435zgjmwhwiqs4426c-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:6nq1qgndd2yp6v435zgjmwhwiqs4426c-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_6r1yvldclwaiqp9l7r423bwvlqvh9mfq-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:6r1yvldclwaiqp9l7r423bwvlqvh9mfq-yzx` |
| `codedb_import` | `nix_store_6vxkq286r99dg69mpvi2ssc9qr8id2d4-yazelix-package-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:6vxkq286r99dg69mpvi2ssc9qr8id2d4-yazelix-package-source` |
| `codedb_import` | `nix_store_6yyzf6svcnynng4dpyci8r4d2wrxzaci-yazelix-zellij-pane-orchestrator-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:6yyzf6svcnynng4dpyci8r4d2wrxzaci-yazelix-zellij-pane-orchestrator-0.1.0.drv` |
| `codedb_import` | `nix_store_712hg3pcnigca8acfij7d8qgszc8qs81-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:712hg3pcnigca8acfij7d8qgszc8qs81-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_73b344k6rryv921nrgllf01jmcj2zi94-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:73b344k6rryv921nrgllf01jmcj2zi94-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_77cjyq6yanfsdv4msr2xgzj4gpmhjpn9-yazelix-rust-core-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:77cjyq6yanfsdv4msr2xgzj4gpmhjpn9-yazelix-rust-core-source` |
| `codedb_import` | `nix_store_7ac9pmrq6lcfqna4sc06p3qyl9m3k2h8-yazelix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:7ac9pmrq6lcfqna4sc06p3qyl9m3k2h8-yazelix` |
| `codedb_import` | `nix_store_7bsvmxdw620jk3dabnpa69q2pryzxd43-docbook-xsl-nons-1_79_2_tar_bz2_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:7bsvmxdw620jk3dabnpa69q2pryzxd43-docbook-xsl-nons-1.79.2.tar.bz2.drv` |
| `codedb_import` | `nix_store_7cjmzaik61fb0pzp3h0nx2f88l20yxdn-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:7cjmzaik61fb0pzp3h0nx2f88l20yxdn-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_7djmy1dyny3di29zwsscqyyvy4gylzhd-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:7djmy1dyny3di29zwsscqyyvy4gylzhd-yazelix-runtime` |
| `codedb_import` | `nix_store_7jf9j6anhprwjablhpwlcdwv4dfbjvcl-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:7jf9j6anhprwjablhpwlcdwv4dfbjvcl-yazelix-runtime` |
| `codedb_import` | `nix_store_7lnf05n5xcdy090wxhbz3la0k9wn0rd6-yazelix-helix-25_7_1_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:7lnf05n5xcdy090wxhbz3la0k9wn0rd6-yazelix-helix-25.7.1.drv` |
| `codedb_import` | `nix_store_7p5m0pgssiwwyxzp25s5wbnlc0s1kqwq-yazelix_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:7p5m0pgssiwwyxzp25s5wbnlc0s1kqwq-yazelix.drv` |
| `codedb_import` | `nix_store_7qfqxhf210ki5fqr55fqvrr9bgjb31lk-yazelix-package-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:7qfqxhf210ki5fqr55fqvrr9bgjb31lk-yazelix-package-source` |
| `codedb_import` | `nix_store_7x64g38x65h4qcv6m2apb9dgxinpd9sh-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:7x64g38x65h4qcv6m2apb9dgxinpd9sh-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_7y5l85v4zi1d1aiip471i4jgbjlxsg9s-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:7y5l85v4zi1d1aiip471i4jgbjlxsg9s-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_8315fskl6ypfs5lsma7wwhs9x95gy9hg-yazelix-zellij-bar-assets` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:8315fskl6ypfs5lsma7wwhs9x95gy9hg-yazelix-zellij-bar-assets` |
| `codedb_import` | `nix_store_83dfhjd9l6b0vb09m62kwb3v4plpcyk1-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:83dfhjd9l6b0vb09m62kwb3v4plpcyk1-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_840fn2mz5fi2wdwgiwg80kcvxr1rzjqs-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:840fn2mz5fi2wdwgiwg80kcvxr1rzjqs-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_86cs3qzvjpqchpf91qxinxr254cxzs5g-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:86cs3qzvjpqchpf91qxinxr254cxzs5g-yzx` |
| `codedb_import` | `nix_store_88fhkzjap6pg64zwl92gb8xhfirlfnc1-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:88fhkzjap6pg64zwl92gb8xhfirlfnc1-yazelix-runtime` |
| `codedb_import` | `nix_store_896fmibv1xy202zgzqav6v8zgh96qypf-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:896fmibv1xy202zgzqav6v8zgh96qypf-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_8awz7x4icjwpcvanrhk77158k64f6yrg-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:8awz7x4icjwpcvanrhk77158k64f6yrg-yazelix-runtime` |
| `codedb_import` | `nix_store_8azlf5jnj0936fbnd6sf349sga640mg0-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:8azlf5jnj0936fbnd6sf349sga640mg0-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_8brxli9c6ln1jfqgl88gybkar3z80s06-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:8brxli9c6ln1jfqgl88gybkar3z80s06-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_8d4731rrdnxmj7nlvhsz8hrfkp59c78k-yazelix-helix-25_7_1_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:8d4731rrdnxmj7nlvhsz8hrfkp59c78k-yazelix-helix-25.7.1.drv` |
| `codedb_import` | `nix_store_8f6jp9mnl3qmps0z91j2csds6d3dcb8j-yazelix_screen-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:8f6jp9mnl3qmps0z91j2csds6d3dcb8j-yazelix_screen-0.1.0` |
| `codedb_import` | `nix_store_8k1jnwal910br7dp26cgirzvv4hnrbvm-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:8k1jnwal910br7dp26cgirzvv4hnrbvm-yzx` |
| `codedb_import` | `nix_store_8x7bidafx5map8jdxjhyzx93rbm4y2r2-textwrap-0_16_2_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:8x7bidafx5map8jdxjhyzx93rbm4y2r2-textwrap-0.16.2.drv` |
| `codedb_import` | `nix_store_8zrs8knz4pb9q3fcvybx026q7bdnh4x3-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:8zrs8knz4pb9q3fcvybx026q7bdnh4x3-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_8zw0q4i8a3d8dwb9n439nn67wrj8qknj-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:8zw0q4i8a3d8dwb9n439nn67wrj8qknj-yzx` |
| `codedb_import` | `nix_store_91vim48pbp7pj4brw5pzl2db6jlls7si-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:91vim48pbp7pj4brw5pzl2db6jlls7si-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_95ydl615bk2hf1nlwqwqbkbpybfwq9kv-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:95ydl615bk2hf1nlwqwqbkbpybfwq9kv-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_980fscfyw32mw7cql73jjrrpbxcdxfap-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:980fscfyw32mw7cql73jjrrpbxcdxfap-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_9bidwzjb7yvls151qlj18zzfshhyzv05-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:9bidwzjb7yvls151qlj18zzfshhyzv05-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_9d27y24jahhdn1gpz8c78bvj3siplfp7-yazelix-helix-25_7_1_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:9d27y24jahhdn1gpz8c78bvj3siplfp7-yazelix-helix-25.7.1.drv` |
| `codedb_import` | `nix_store_9h8bcrnspvxzzakf3c1drq21ld936h4c-yazelix-package-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:9h8bcrnspvxzzakf3c1drq21ld936h4c-yazelix-package-source` |
| `codedb_import` | `nix_store_9hklffnwz1vyc1ikh3gfizq55a8jgfcn-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:9hklffnwz1vyc1ikh3gfizq55a8jgfcn-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_9mzihvnlwbd22kxyjwz4p0qjh6mi9wr1-yazelix-rust-core-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:9mzihvnlwbd22kxyjwz4p0qjh6mi9wr1-yazelix-rust-core-source` |
| `codedb_import` | `nix_store_9pwigjynp65pb5m21fjxqivn1vs1zpfi-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:9pwigjynp65pb5m21fjxqivn1vs1zpfi-yzx` |
| `codedb_import` | `nix_store_9s9rvr4msflbpgwx2m77gk1zwkjcfy4h-yazelix-rust-core-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:9s9rvr4msflbpgwx2m77gk1zwkjcfy4h-yazelix-rust-core-source` |
| `codedb_import` | `nix_store_9v4n4zqqbngxdfcdzlgval7d371yf3s2-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:9v4n4zqqbngxdfcdzlgval7d371yf3s2-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_9vyd5s285k4656grwagg4fplsfraasv3-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:9vyd5s285k4656grwagg4fplsfraasv3-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_9xzm9gmraha3df310hj2xdh2vkaj32a0-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:9xzm9gmraha3df310hj2xdh2vkaj32a0-yazelix-runtime` |
| `codedb_import` | `nix_store_a2xjkjid1cq1qxia53a9d0r4pyfbdbkf-yazelix-cursors-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:a2xjkjid1cq1qxia53a9d0r4pyfbdbkf-yazelix-cursors-0.1.0.drv` |
| `codedb_import` | `nix_store_a6qbn4n42rd1wyxbz54hl6sf8abn5yzx-libxfont_2-2_0_7_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:a6qbn4n42rd1wyxbz54hl6sf8abn5yzx-libxfont_2-2.0.7.drv` |
| `codedb_import` | `nix_store_a7gdh9p3kgrawws2w1946bdba9p7myqd-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:a7gdh9p3kgrawws2w1946bdba9p7myqd-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_a8p3g37l62nw2rhx2sc11gi1kcqfj7v4-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:a8p3g37l62nw2rhx2sc11gi1kcqfj7v4-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_aax4vjpym40kacf7qnxcx8ylgnmxr39k-yazelix-rust-core-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:aax4vjpym40kacf7qnxcx8ylgnmxr39k-yazelix-rust-core-source` |
| `codedb_import` | `nix_store_achw4cqmmmnbhpr6536ybw0wi1fbh55k-yazelix-helix-25_7_1_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:achw4cqmmmnbhpr6536ybw0wi1fbh55k-yazelix-helix-25.7.1.drv` |
| `codedb_import` | `nix_store_ahr7zqms4mrqcx8xwc626ajmbx6r84i1-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:ahr7zqms4mrqcx8xwc626ajmbx6r84i1-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_aprwmzmag3xvyrkkcv129b31sy1gpdjg-yazelix-rust-core-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:aprwmzmag3xvyrkkcv129b31sy1gpdjg-yazelix-rust-core-source` |
| `codedb_import` | `nix_store_as0hvyscffj2rb781sbvszdchskf9adc-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:as0hvyscffj2rb781sbvszdchskf9adc-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_avlcijz4v5ay3gsjci1zhpiwjk2dhcpq-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:avlcijz4v5ay3gsjci1zhpiwjk2dhcpq-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_awk53pghp7hvaikxqzvrswrgy53cnz2z-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:awk53pghp7hvaikxqzvrswrgy53cnz2z-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_b5yissg46sfdbsk92d3np0jgah7a40g7-yazelix_cursors-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:b5yissg46sfdbsk92d3np0jgah7a40g7-yazelix_cursors-0.1.0.drv` |
| `codedb_import` | `nix_store_bgw9cik568f7ljfa3yrkg6pqpdsn3qaw-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:bgw9cik568f7ljfa3yrkg6pqpdsn3qaw-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_bj090nm7ggjzny5xmwaim09dvad3msi9-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:bj090nm7ggjzny5xmwaim09dvad3msi9-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_bn9jx9298m1h5b54r4kzmm0xpxzqic17-yazelix-rust-core-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:bn9jx9298m1h5b54r4kzmm0xpxzqic17-yazelix-rust-core-source` |
| `codedb_import` | `nix_store_bpdyiylayymawp5idn8lxgj386y46r1y-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:bpdyiylayymawp5idn8lxgj386y46r1y-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_bpzq0y3hcmxxhg8d6lgjksfvay4k1bik-yazelix-runtime-release-contracts_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:bpzq0y3hcmxxhg8d6lgjksfvay4k1bik-yazelix-runtime-release-contracts.drv` |
| `codedb_import` | `nix_store_br8pj9j2zkl2s5fwdqraj338kcbba39j-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:br8pj9j2zkl2s5fwdqraj338kcbba39j-yazelix-runtime` |
| `codedb_import` | `nix_store_bw8jk128kfc8nvxw8ls95vyhpvd6q3d1-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:bw8jk128kfc8nvxw8ls95vyhpvd6q3d1-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_bwv2n2a8lqphkwkqxai0y2zbm2pg2j1h-yazelix_yazi_assets-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:bwv2n2a8lqphkwkqxai0y2zbm2pg2j1h-yazelix_yazi_assets-0.1.0` |
| `codedb_import` | `nix_store_bxarm7ci7a13vcdcpxw1ahhryizcrh0g-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:bxarm7ci7a13vcdcpxw1ahhryizcrh0g-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_c1dcjank76wfa14hkgbsz51jpip7i92s-yazelix_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:c1dcjank76wfa14hkgbsz51jpip7i92s-yazelix.drv` |
| `codedb_import` | `nix_store_c4qsp7ch1lkp25rrhjjkqnmrih25w2jc-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:c4qsp7ch1lkp25rrhjjkqnmrih25w2jc-yzx` |
| `codedb_import` | `nix_store_c52apmsbs0n3zcwj8h81dm4mfwdmkpha-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:c52apmsbs0n3zcwj8h81dm4mfwdmkpha-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_c6hkksy5mbj0brm93l2ks511fy4ld5l8-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:c6hkksy5mbj0brm93l2ks511fy4ld5l8-yazelix-runtime` |
| `codedb_import` | `nix_store_ca2yi9nw6ynrpay56nxp6wgz5w4yi8zm-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:ca2yi9nw6ynrpay56nxp6wgz5w4yi8zm-yzx.drv` |
| `codedb_import` | `nix_store_ca8z75sdvjry645f14drccn9qi0hax1g-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:ca8z75sdvjry645f14drccn9qi0hax1g-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_ca9nnmzpwa35xrqib9mqj6myd202cjfi-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:ca9nnmzpwa35xrqib9mqj6myd202cjfi-yzx.drv` |
| `codedb_import` | `nix_store_cai61mkpidrw0rdkk9brspc0dnfn85vy-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:cai61mkpidrw0rdkk9brspc0dnfn85vy-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_cg7xmfy4b812a70lw68ym6mawawp8kps-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:cg7xmfy4b812a70lw68ym6mawawp8kps-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_cmzw9kd2hmn7q3cbdayshxg8kdhnchvh-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:cmzw9kd2hmn7q3cbdayshxg8kdhnchvh-yzx` |
| `codedb_import` | `nix_store_cqrz2wvqd3k128qwmw8gjvhv2am61xxy-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:cqrz2wvqd3k128qwmw8gjvhv2am61xxy-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_cxnls7rhazsnax2ayakgmiq716pq6z6i-yazelix_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:cxnls7rhazsnax2ayakgmiq716pq6z6i-yazelix.drv` |
| `codedb_import` | `nix_store_czid0ghi1sbnz7x5g251m79hlsp3vshv-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:czid0ghi1sbnz7x5g251m79hlsp3vshv-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_d0a4jcw6i8ms0kpidf8f5i028v48cqn7-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:d0a4jcw6i8ms0kpidf8f5i028v48cqn7-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_d2shxfy1f0hqa63nds9vf732z2j6964w-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:d2shxfy1f0hqa63nds9vf732z2j6964w-yzx` |
| `codedb_import` | `nix_store_d7s8yzxx4lkp4pnyj35i12v5kdg6hwiv-python3_13-wcwidth-0_6_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:d7s8yzxx4lkp4pnyj35i12v5kdg6hwiv-python3.13-wcwidth-0.6.0.drv` |
| `codedb_import` | `nix_store_d8jv1hn8d6bkccfbr9x47j38lin9vg4x-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:d8jv1hn8d6bkccfbr9x47j38lin9vg4x-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_d9bmhsva5cm7879qz6mg4zs8h5jgx1j9-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:d9bmhsva5cm7879qz6mg4zs8h5jgx1j9-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_da1k4991mvhkm5jvklwa52mrrfdnm2cy-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:da1k4991mvhkm5jvklwa52mrrfdnm2cy-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_dalp6jj108n8dq14cjqd5xxampvzh5kq-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:dalp6jj108n8dq14cjqd5xxampvzh5kq-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_dmw7nrwz8lg6rlj5p550di6fx5wsqcki-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:dmw7nrwz8lg6rlj5p550di6fx5wsqcki-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_dpnb2g5d4xvl28h447jj32w2l8i4lwlb-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:dpnb2g5d4xvl28h447jj32w2l8i4lwlb-yazelix-runtime` |
| `codedb_import` | `nix_store_dqp5v3yw6pqi08n2ivjd1fqsjwy59iy8-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:dqp5v3yw6pqi08n2ivjd1fqsjwy59iy8-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_dxfq94flwny6xhkwvihvk31qzgm3xmz9-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:dxfq94flwny6xhkwvihvk31qzgm3xmz9-yzx.drv` |
| `codedb_import` | `nix_store_f2j7vk7pbxca4hj9371yalp5b47kbh9s-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:f2j7vk7pbxca4hj9371yalp5b47kbh9s-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_f2jsbqpjk14cpfc21df3lvyp99nwjkf6-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:f2jsbqpjk14cpfc21df3lvyp99nwjkf6-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_f3arq12p0laiy0i44pi10csqs0zk8bf0-yazelix_yazi_assets-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:f3arq12p0laiy0i44pi10csqs0zk8bf0-yazelix_yazi_assets-0.1.0.drv` |
| `codedb_import` | `nix_store_f5zp3xra1f11mslk986ds0vaviy6l0cy-yazelix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:f5zp3xra1f11mslk986ds0vaviy6l0cy-yazelix` |
| `codedb_import` | `nix_store_ffg20pvjgjvfxz20grmgnybmgy5ir7c7-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:ffg20pvjgjvfxz20grmgnybmgy5ir7c7-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_fgkz89q3czrp5p9iik1nfscgnqmln8an-yazelix_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:fgkz89q3czrp5p9iik1nfscgnqmln8an-yazelix.drv` |
| `codedb_import` | `nix_store_fhcs1c9f1ksrks1y56jgsr6g772fxk1h-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:fhcs1c9f1ksrks1y56jgsr6g772fxk1h-yzx` |
| `codedb_import` | `nix_store_fmflhp52g862wh13zj02w7rq8hf8qfn4-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:fmflhp52g862wh13zj02w7rq8hf8qfn4-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_fmkbp695jblj1yzx8sh85nlg6g3phk6p-source_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:fmkbp695jblj1yzx8sh85nlg6g3phk6p-source.drv` |
| `codedb_import` | `nix_store_frp1inf0zwj7v4jykjg3hwg7ylghgjz9-yazelix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:frp1inf0zwj7v4jykjg3hwg7ylghgjz9-yazelix` |
| `codedb_import` | `nix_store_fxxwmsajgm85wz2khmwi2aj0ggs188r8-yazelix-screen-9dc52f7_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:fxxwmsajgm85wz2khmwi2aj0ggs188r8-yazelix-screen-9dc52f7.drv` |
| `codedb_import` | `nix_store_g2f3l3pfwfgf9iv2myzxmmqxw4j3xqg5-source_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:g2f3l3pfwfgf9iv2myzxmmqxw4j3xqg5-source.drv` |
| `codedb_import` | `nix_store_g3lylm0mvax5gkjp4n4y8chp41wcfxg0-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:g3lylm0mvax5gkjp4n4y8chp41wcfxg0-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_g65zywhp44h2fxwhk13nnclrb9khr5kv-yazelix-kgp-package-contracts_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:g65zywhp44h2fxwhk13nnclrb9khr5kv-yazelix-kgp-package-contracts.drv` |
| `codedb_import` | `nix_store_gas80z260j54bvjhknaj461fc9phfia3-yazelix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:gas80z260j54bvjhknaj461fc9phfia3-yazelix` |
| `codedb_import` | `nix_store_gazlkgssv4hjg0zb8ilx9w66cdfjargg-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:gazlkgssv4hjg0zb8ilx9w66cdfjargg-yzx.drv` |
| `codedb_import` | `nix_store_gcrywhvrym473acnz0nxz7wp9x9j26h2-yazelix-rust-core-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:gcrywhvrym473acnz0nxz7wp9x9j26h2-yazelix-rust-core-source` |
| `codedb_import` | `nix_store_gczbfnryyg2wps78ds1xq5xng8wsjb06-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:gczbfnryyg2wps78ds1xq5xng8wsjb06-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_ggdkkjvirkl4r3ymshkvkkjrr0ykxmj7-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:ggdkkjvirkl4r3ymshkvkkjrr0ykxmj7-yzx` |
| `codedb_import` | `nix_store_gjpds0cyn98k9j6m1h4xxk6nv13rh2nn-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:gjpds0cyn98k9j6m1h4xxk6nv13rh2nn-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_gka60vxd9qgvpz7n3y9lhsbnpyvpjsj5-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:gka60vxd9qgvpz7n3y9lhsbnpyvpjsj5-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_gmfyzxz6qrvkfa3dcp5jncc08xxah59v-0008-Provide-mach-compatibility-headers-based-on-LLVM-s-h_patch` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:gmfyzxz6qrvkfa3dcp5jncc08xxah59v-0008-Provide-mach-compatibility-headers-based-on-LLVM-s-h.patch` |
| `codedb_import` | `nix_store_gmhk0vywy1sj5mnr3ks6l3ip9dbffs54-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:gmhk0vywy1sj5mnr3ks6l3ip9dbffs54-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_grhi21s6f8japiy32y0q6bxqqvlz71wz-yazelix_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:grhi21s6f8japiy32y0q6bxqqvlz71wz-yazelix.drv` |
| `codedb_import` | `nix_store_gs7nh103xhs2xh5647yxwpbgkbwij0k6-yazelix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:gs7nh103xhs2xh5647yxwpbgkbwij0k6-yazelix` |
| `codedb_import` | `nix_store_gwl7c83iby74f15hddi7xd4xgqwijd2b-yazelix-cursors-98ddd8b_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:gwl7c83iby74f15hddi7xd4xgqwijd2b-yazelix-cursors-98ddd8b.drv` |
| `codedb_import` | `nix_store_h1mflz4dvkny32i0jn8hw003g1600z64-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:h1mflz4dvkny32i0jn8hw003g1600z64-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_h8d06npb9bbqcscdshv2wpvhj1jkdfwn-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:h8d06npb9bbqcscdshv2wpvhj1jkdfwn-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_hcnw419i2h7lnk22idh0j13y14i6a1p2-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:hcnw419i2h7lnk22idh0j13y14i6a1p2-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_hd0hwv5bw4fb49acgcz8dk1bkcsf123m-yazelix_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:hd0hwv5bw4fb49acgcz8dk1bkcsf123m-yazelix.drv` |
| `codedb_import` | `nix_store_hgxb6r9s01j2fv1picy9lgi1nmm5j3aj-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:hgxb6r9s01j2fv1picy9lgi1nmm5j3aj-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_hgyhrs9sa39lxfz7l2m6pi7wgvj3fjn3-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:hgyhrs9sa39lxfz7l2m6pi7wgvj3fjn3-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_hi99gm0447n0x3nmpbr7r1wsa0fijg3k-yazelix-runtime-release-contracts` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=plain_or_binary structured_rows=0` | `nix_store:hi99gm0447n0x3nmpbr7r1wsa0fijg3k-yazelix-runtime-release-contracts` |
| `codedb_import` | `nix_store_hk4m00bfnqddhlymfy8s6afrr8fd93kk-yazelix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:hk4m00bfnqddhlymfy8s6afrr8fd93kk-yazelix` |
| `codedb_import` | `nix_store_hwflz3l71m3b53ylslwi00k5qdkq635c-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:hwflz3l71m3b53ylslwi00k5qdkq635c-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_i79q5n25rfkysa1nfmqp03nfnm1jdsa8-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:i79q5n25rfkysa1nfmqp03nfnm1jdsa8-yazelix-runtime` |
| `codedb_import` | `nix_store_i7q7nf6blv32y48zc8mncckh7fpaa6sb-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:i7q7nf6blv32y48zc8mncckh7fpaa6sb-yzx` |
| `codedb_import` | `nix_store_icq8v6fb52brn90qmpawil1wgif98g6y-yazelix-dark_toml_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:icq8v6fb52brn90qmpawil1wgif98g6y-yazelix-dark.toml.drv` |
| `codedb_import` | `nix_store_id44kjf198bq0sd7bb52fgc8xgari1ac-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:id44kjf198bq0sd7bb52fgc8xgari1ac-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_idi61hx65jadw4zr0afc1gig5n0bj6x9-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:idi61hx65jadw4zr0afc1gig5n0bj6x9-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_idqmyg21bmzr0ib13hjfbl72zlgdjrvg-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:idqmyg21bmzr0ib13hjfbl72zlgdjrvg-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_igi162gp0190am52imdmdlqb6s6k2mdx-yazelix_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:igi162gp0190am52imdmdlqb6s6k2mdx-yazelix.drv` |
| `codedb_import` | `nix_store_ihpr0n6hd9kjmb56fsra35b93x7cyyi3-yazelix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:ihpr0n6hd9kjmb56fsra35b93x7cyyi3-yazelix` |
| `codedb_import` | `nix_store_ihw971vw3jqgpbg2gfh17dhr1dv9dsn6-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:ihw971vw3jqgpbg2gfh17dhr1dv9dsn6-yazelix-runtime` |
| `codedb_import` | `nix_store_iikfn8cpspsqln8yqrgaj8mxzmbx020s-yazelix-rust-core-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:iikfn8cpspsqln8yqrgaj8mxzmbx020s-yazelix-rust-core-source` |
| `codedb_import` | `nix_store_iiwvfsk6k37rmg7p6rax7xzy477yzxsh-cargo-src-zerovec-derive-0_11_3_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:iiwvfsk6k37rmg7p6rax7xzy477yzxsh-cargo-src-zerovec-derive-0.11.3.drv` |
| `codedb_import` | `nix_store_irj27gry9a5a4nlks66cgpdb796l7w15-yazelix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:irj27gry9a5a4nlks66cgpdb796l7w15-yazelix` |
| `codedb_import` | `nix_store_ivkrnyg9f8xgb83mivd6qyg3yh28yqry-yazelix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:ivkrnyg9f8xgb83mivd6qyg3yh28yqry-yazelix` |
| `codedb_import` | `nix_store_iwyzri2ixpv8vv74v9ja97fm7965lz3w-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:iwyzri2ixpv8vv74v9ja97fm7965lz3w-yazelix-runtime` |
| `codedb_import` | `nix_store_iy4nsvf8yp7hvki5ldf141p87g1vinv5-yazelix_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:iy4nsvf8yp7hvki5ldf141p87g1vinv5-yazelix.drv` |
| `codedb_import` | `nix_store_j101js2d63qh0hm5jxms5k7x7d63rack-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:j101js2d63qh0hm5jxms5k7x7d63rack-yzx` |
| `codedb_import` | `nix_store_j2x4x72skbdzik18bh0qx8366ja41x6f-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:j2x4x72skbdzik18bh0qx8366ja41x6f-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_j3rfw1yzx0b7y93ccdscim5kzn9rq2vd-quickcheck-instances-0_3_33_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:j3rfw1yzx0b7y93ccdscim5kzn9rq2vd-quickcheck-instances-0.3.33.drv` |
| `codedb_import` | `nix_store_j4l5i8zb4l0ls0r1kvkln9kvz94h094h-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:j4l5i8zb4l0ls0r1kvkln9kvz94h094h-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_j78ii2lsd1af8245ic98vf3bndphbk3p-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:j78ii2lsd1af8245ic98vf3bndphbk3p-yzx.drv` |
| `codedb_import` | `nix_store_jacs0vqlyalq5dnha4yzrsrb23n3m967-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:jacs0vqlyalq5dnha4yzrsrb23n3m967-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_jajkhnap76g6pa65y5q37q42qgxncp0l-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:jajkhnap76g6pa65y5q37q42qgxncp0l-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_jd4g3p9grc0ccz9k7nwcql1pvn029c68-yazelix-runtime-release-contracts` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=plain_or_binary structured_rows=0` | `nix_store:jd4g3p9grc0ccz9k7nwcql1pvn029c68-yazelix-runtime-release-contracts` |
| `codedb_import` | `nix_store_jkyw23cdk75g7iwbwrwy6s83sm53myzx-libc-0_2_178_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:jkyw23cdk75g7iwbwrwy6s83sm53myzx-libc-0.2.178.drv` |
| `codedb_import` | `nix_store_jnf3rn723b4f081xcvxybwjx0gqcx5yb-yazelix_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:jnf3rn723b4f081xcvxybwjx0gqcx5yb-yazelix.drv` |
| `codedb_import` | `nix_store_jpkiynq35lgl524wwc078azp0xpjrk3a-yazelix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:jpkiynq35lgl524wwc078azp0xpjrk3a-yazelix` |
| `codedb_import` | `nix_store_jpq7q7bilyxcc7bchq9i47qkmw1hpxp7-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:jpq7q7bilyxcc7bchq9i47qkmw1hpxp7-yazelix-runtime` |
| `codedb_import` | `nix_store_jxcxab0iqpj0aisqwjrii02i66yxl0a1-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:jxcxab0iqpj0aisqwjrii02i66yxl0a1-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_jz1qriq2kcbz98nlfnahw9n27hhzsj25-yazelix-helix-25_7_1` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:jz1qriq2kcbz98nlfnahw9n27hhzsj25-yazelix-helix-25.7.1` |
| `codedb_import` | `nix_store_k0b4d2zg19gkwlmpgfr4hlnnijb65gd9-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:k0b4d2zg19gkwlmpgfr4hlnnijb65gd9-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_kic1i908l4xzi2bkmv4x29mggh4xk44r-yazelix-rust-core-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:kic1i908l4xzi2bkmv4x29mggh4xk44r-yazelix-rust-core-source` |
| `codedb_import` | `nix_store_kmmwvdpri9n3skpkrqmxkah6i2c78h41-yazelix_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:kmmwvdpri9n3skpkrqmxkah6i2c78h41-yazelix.drv` |
| `codedb_import` | `nix_store_knjw0nsjk1pm8b2nhripvx6z9q50lhd8-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:knjw0nsjk1pm8b2nhripvx6z9q50lhd8-yzx` |
| `codedb_import` | `nix_store_kw9w3n4w2fhp3xwgz6icdq4vw5gw9n66-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:kw9w3n4w2fhp3xwgz6icdq4vw5gw9n66-yazelix-runtime` |
| `codedb_import` | `nix_store_l119g9bisvgrchnpnkabb5hps8qmvl4d-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:l119g9bisvgrchnpnkabb5hps8qmvl4d-yzx.drv` |
| `codedb_import` | `nix_store_l410zv1xb5cfhjf26iknv4v62bbpzbhs-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:l410zv1xb5cfhjf26iknv4v62bbpzbhs-yzx.drv` |
| `codedb_import` | `nix_store_l471v51sv3gg729iagjzz5ib491ls673-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:l471v51sv3gg729iagjzz5ib491ls673-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_l5xy223hvzhgyqz8nmg3iyhd5ryvmp1m-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:l5xy223hvzhgyqz8nmg3iyhd5ryvmp1m-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_lcikp367kzi3s8cplcij6in2347hxvbb-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:lcikp367kzi3s8cplcij6in2347hxvbb-yazelix-runtime` |
| `codedb_import` | `nix_store_lhqywzyys15m2q7z48bhi3gha74fjkfp-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:lhqywzyys15m2q7z48bhi3gha74fjkfp-yzx.drv` |
| `codedb_import` | `nix_store_li9n767mpnbc9rcivbq9ysxl9h0m7c51-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:li9n767mpnbc9rcivbq9ysxl9h0m7c51-yazelix-runtime` |
| `codedb_import` | `nix_store_lm01z5vsdky3q5az4b53h1qmxgj5kyhr-yazelix-rust-core-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:lm01z5vsdky3q5az4b53h1qmxgj5kyhr-yazelix-rust-core-source` |
| `codedb_import` | `nix_store_lprwb26crdizhcj5122kq722skslp8vb-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:lprwb26crdizhcj5122kq722skslp8vb-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_lqmwi3h40kb970k4yrf2szkx03p9qhcw-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:lqmwi3h40kb970k4yrf2szkx03p9qhcw-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_lv4qmj5gjkn27mbn3pmnib6idfwgl0af-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:lv4qmj5gjkn27mbn3pmnib6idfwgl0af-yzx.drv` |
| `codedb_import` | `nix_store_lwfyx0pp2hmw5z41g0c2kw3lsc81kdl4-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:lwfyx0pp2hmw5z41g0c2kw3lsc81kdl4-yazelix-runtime` |
| `codedb_import` | `nix_store_m4qgr0a9f9v44n7drik6pd6l0mi369kz-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:m4qgr0a9f9v44n7drik6pd6l0mi369kz-yzx.drv` |
| `codedb_import` | `nix_store_m71qb122axf2a4xcsabd3bg4azyzxsll-contravariant-1_5_6_tar_gz_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:m71qb122axf2a4xcsabd3bg4azyzxsll-contravariant-1.5.6.tar.gz.drv` |
| `codedb_import` | `nix_store_m96h8x4dxsxdyi6fjimnmz2yv644hll6-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:m96h8x4dxsxdyi6fjimnmz2yv644hll6-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_mdjsqsadzvcynnb0is2wwrv6xbmzxmsm-yazelix-rust-core-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:mdjsqsadzvcynnb0is2wwrv6xbmzxmsm-yazelix-rust-core-source` |
| `codedb_import` | `nix_store_mdv6m0z2s8b73l89sbs083r97db7m4ib-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:mdv6m0z2s8b73l89sbs083r97db7m4ib-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_mhrz04q5x3wla17s0rrlwc71ra2ywxbz-yazelix-runtime-release-contracts_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:mhrz04q5x3wla17s0rrlwc71ra2ywxbz-yazelix-runtime-release-contracts.drv` |
| `codedb_import` | `nix_store_mi12fcr97xk6h5lrx8b3ff98ra4rvp3m-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:mi12fcr97xk6h5lrx8b3ff98ra4rvp3m-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_mj7xq29k1kaidk91c09s5zywssrhbm0l-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:mj7xq29k1kaidk91c09s5zywssrhbm0l-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_mk2y654kzwwvvld1m6v7iq1y504p9k16-yazelix_cursors-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:mk2y654kzwwvvld1m6v7iq1y504p9k16-yazelix_cursors-0.1.0` |
| `codedb_import` | `nix_store_mkbvcwk465ykahhzblz0yzzlr760qaq1-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:mkbvcwk465ykahhzblz0yzzlr760qaq1-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_mlrc5bi8sg1sp81bnjnd26z4wmv3sb09-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:mlrc5bi8sg1sp81bnjnd26z4wmv3sb09-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_mlwq1i3assys5lhv0caj6a9fdrwr0bxp-yazelix_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:mlwq1i3assys5lhv0caj6a9fdrwr0bxp-yazelix.drv` |
| `codedb_import` | `nix_store_mp6gqcp8k3zvmanib0vzkdl22d9hf8aj-yazelix_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:mp6gqcp8k3zvmanib0vzkdl22d9hf8aj-yazelix.drv` |
| `codedb_import` | `nix_store_mpnlqf6k38glp9h3pw3mg4c7ihlcsd5j-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:mpnlqf6k38glp9h3pw3mg4c7ihlcsd5j-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_mqw8vjiayywbak4394w5iwnhlp1rnq98-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:mqw8vjiayywbak4394w5iwnhlp1rnq98-yzx.drv` |
| `codedb_import` | `nix_store_mvrfq0hqiandy1wdbfsxxkzv40n9gpjq-yazelix-install-check-0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:mvrfq0hqiandy1wdbfsxxkzv40n9gpjq-yazelix-install-check-0.drv` |
| `codedb_import` | `nix_store_n94wy3ybyklxr28zn0zgd56kpf5njqlk-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:n94wy3ybyklxr28zn0zgd56kpf5njqlk-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_nbcp9d10aqcn38hw9w8808r26n1dwvzz-yazelix-rust-core-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:nbcp9d10aqcn38hw9w8808r26n1dwvzz-yazelix-rust-core-source` |
| `codedb_import` | `nix_store_ncsrnp48xribl61gwhvyjbk0b1pqk47k-yazelix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:ncsrnp48xribl61gwhvyjbk0b1pqk47k-yazelix` |
| `codedb_import` | `nix_store_nf5gslikap20kq6mwhrhvwk3iahp2sri-yazelix-cursors-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:nf5gslikap20kq6mwhrhvwk3iahp2sri-yazelix-cursors-source` |
| `codedb_import` | `nix_store_nk36gm7ppg3knkqyb00zl9jb6fhvrkma-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:nk36gm7ppg3knkqyb00zl9jb6fhvrkma-yazelix-runtime` |
| `codedb_import` | `nix_store_nm8kryi2cpqwf7v6469bdjrvv5dmjd15-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:nm8kryi2cpqwf7v6469bdjrvv5dmjd15-yzx.drv` |
| `codedb_import` | `nix_store_nmd7hrb9sx01gfad8gh5xj94a869jncm-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:nmd7hrb9sx01gfad8gh5xj94a869jncm-yzx.drv` |
| `codedb_import` | `nix_store_nwi5misl6jyx4zsc2243pacwld5akhpd-yazelix_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:nwi5misl6jyx4zsc2243pacwld5akhpd-yazelix.drv` |
| `codedb_import` | `nix_store_nydq5j3mmhyz2nps2b7msjf8m2r48hd8-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:nydq5j3mmhyz2nps2b7msjf8m2r48hd8-yzx` |
| `codedb_import` | `nix_store_p45lnz6nsvjzvhjlbaifqncmb21vgwdy-yazelix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:p45lnz6nsvjzvhjlbaifqncmb21vgwdy-yazelix` |
| `codedb_import` | `nix_store_p5h2b61x535hr98c0hkzbxv4scgqrs0l-yazelix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:p5h2b61x535hr98c0hkzbxv4scgqrs0l-yazelix` |
| `codedb_import` | `nix_store_p6dzh0bjzy8panc7xni7n1fdizjv77x9-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:p6dzh0bjzy8panc7xni7n1fdizjv77x9-yazelix-runtime` |
| `codedb_import` | `nix_store_pjsrlvpa3rgvja12xskbk8czbfr9sj62-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:pjsrlvpa3rgvja12xskbk8czbfr9sj62-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_prbjddm3004f4xmms36j3mnf097izrxi-yazelix-zellij-bar-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:prbjddm3004f4xmms36j3mnf097izrxi-yazelix-zellij-bar-source` |
| `codedb_import` | `nix_store_pvbrn7ikyar4m58f28zminqr99xrrnpd-yazelix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:pvbrn7ikyar4m58f28zminqr99xrrnpd-yazelix` |
| `codedb_import` | `nix_store_pw27gxmhg1kpxy41rkkbh2f2m94gz6f8-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:pw27gxmhg1kpxy41rkkbh2f2m94gz6f8-yzx.drv` |
| `codedb_import` | `nix_store_pxfrkpikc7p8hbrq6gv0zk1dzc32z5lg-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:pxfrkpikc7p8hbrq6gv0zk1dzc32z5lg-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_q0j6qq35hnnj3s9zwirc0794rzqmzbxm-yazelix_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:q0j6qq35hnnj3s9zwirc0794rzqmzbxm-yazelix.drv` |
| `codedb_import` | `nix_store_q11y5in8g47a80vhynfs8rm4wpbbwwxy-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:q11y5in8g47a80vhynfs8rm4wpbbwwxy-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_q3m7z5nky63y6p59ym4dk81a90y18f2g-yazelix_screen-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:q3m7z5nky63y6p59ym4dk81a90y18f2g-yazelix_screen-0.1.0.drv` |
| `codedb_import` | `nix_store_q9gzlpxvx7bnwndd8bcm4wbrk78qz5pa-yazelix-runtime-release-contracts_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:q9gzlpxvx7bnwndd8bcm4wbrk78qz5pa-yazelix-runtime-release-contracts.drv` |
| `codedb_import` | `nix_store_qa8cjnjm211j4a7cyjshmzw27ljj461z-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:qa8cjnjm211j4a7cyjshmzw27ljj461z-yzx` |
| `codedb_import` | `nix_store_qcwz0ib2znyglw3bm90nyls105ss1h4m-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:qcwz0ib2znyglw3bm90nyls105ss1h4m-yzx.drv` |
| `codedb_import` | `nix_store_qf3l8rbxr1gjwwgrx64fddd8ghgyzyxb-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:qf3l8rbxr1gjwwgrx64fddd8ghgyzyxb-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_qia8p5ccnh1hih7air912zr496nq20h1-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:qia8p5ccnh1hih7air912zr496nq20h1-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_qnhdvgw4ailkrzzfsr861y58zrybvkj8-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:qnhdvgw4ailkrzzfsr861y58zrybvkj8-yazelix-runtime` |
| `codedb_import` | `nix_store_qs8vkxfva7fjin4rdf7984m5hfxwwzlq-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:qs8vkxfva7fjin4rdf7984m5hfxwwzlq-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_qv2i62yn4fyz3qb1y4x7q47qjvndnfg5-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:qv2i62yn4fyz3qb1y4x7q47qjvndnfg5-yzx` |
| `codedb_import` | `nix_store_qw97jm3044fk3xly7w8p33278j8vfz28-yazelix-package-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:qw97jm3044fk3xly7w8p33278j8vfz28-yazelix-package-source` |
| `codedb_import` | `nix_store_qyl7bjmbqn3p9qmgzsxz84r285dw3i2a-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:qyl7bjmbqn3p9qmgzsxz84r285dw3i2a-yzx.drv` |
| `codedb_import` | `nix_store_r00bigyvbjlkv4msw9kh8imlh77zrb6a-yazelix_yazi_assets-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:r00bigyvbjlkv4msw9kh8imlh77zrb6a-yazelix_yazi_assets-0.1.0.drv` |
| `codedb_import` | `nix_store_r2zwc8m1py4ds8yvhkf3jplirjrsiffh-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:r2zwc8m1py4ds8yvhkf3jplirjrsiffh-yazelix-runtime` |
| `codedb_import` | `nix_store_r36cabmrw4lcw1impyaj8armys2lb0d4-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:r36cabmrw4lcw1impyaj8armys2lb0d4-yzx.drv` |
| `codedb_import` | `nix_store_r8az73rjkgnmzdfjldqx2cwx5amy6zca-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:r8az73rjkgnmzdfjldqx2cwx5amy6zca-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_ri6a5idmf83wsqr7vx62haycgiri1zy8-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:ri6a5idmf83wsqr7vx62haycgiri1zy8-yzx.drv` |
| `codedb_import` | `nix_store_rkqzrzp07s90p9wgnhl52fxbz2xr72fs-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:rkqzrzp07s90p9wgnhl52fxbz2xr72fs-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_rli7zs8n455fp5ffgmpawjplc750vl0p-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:rli7zs8n455fp5ffgmpawjplc750vl0p-yzx` |
| `codedb_import` | `nix_store_rmxrvlr5v9pnjk3z8q31x8328dgcf2bp-yazelix_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:rmxrvlr5v9pnjk3z8q31x8328dgcf2bp-yazelix.drv` |
| `codedb_import` | `nix_store_rssf6plj39an36s2yww357gvbs97aj6z-yazelix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:rssf6plj39an36s2yww357gvbs97aj6z-yazelix` |
| `codedb_import` | `nix_store_rvg33wclzwvrkycccfxya5hv3q22bm3p-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:rvg33wclzwvrkycccfxya5hv3q22bm3p-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_rw41pca9j2bqmww8nb8zpszkw94pds8z-yazelix-yazi-assets-e9a936a_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:rw41pca9j2bqmww8nb8zpszkw94pds8z-yazelix-yazi-assets-e9a936a.drv` |
| `codedb_import` | `nix_store_rx8bz1x2y87njpwp6acs8fcrliglq2c3-yazelix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:rx8bz1x2y87njpwp6acs8fcrliglq2c3-yazelix` |
| `codedb_import` | `nix_store_rydkcrx3yrcxkvval4wapgcbj5hiymlr-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:rydkcrx3yrcxkvval4wapgcbj5hiymlr-yzx.drv` |
| `codedb_import` | `nix_store_rynfsa12agwi7wvg8pjpjayy4xh6w9rn-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:rynfsa12agwi7wvg8pjpjayy4xh6w9rn-yzx` |
| `codedb_import` | `nix_store_s0idvs3f1xncmi7rg6mviyklkahkw74f-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:s0idvs3f1xncmi7rg6mviyklkahkw74f-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_s0qcrjdl15zg8jr5qm78ij2ywp1vmjiz-yazelix-rust-core-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:s0qcrjdl15zg8jr5qm78ij2ywp1vmjiz-yazelix-rust-core-source` |
| `codedb_import` | `nix_store_s43did42mx51sw0magm7f16n4gy9xg43-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:s43did42mx51sw0magm7f16n4gy9xg43-yzx` |
| `codedb_import` | `nix_store_s8v0yqhk2vjp4gspm65qhn4gs23hsfh2-yazelix-runtime-release-contracts` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=plain_or_binary structured_rows=0` | `nix_store:s8v0yqhk2vjp4gspm65qhn4gs23hsfh2-yazelix-runtime-release-contracts` |
| `codedb_import` | `nix_store_sm87nrpjl89i9l8n7724am8lxmxcbar0-yazelix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:sm87nrpjl89i9l8n7724am8lxmxcbar0-yazelix` |
| `codedb_import` | `nix_store_smgkvfkyn9hb99vqf5k0kcr3zy5w48x8-yazelix-package-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:smgkvfkyn9hb99vqf5k0kcr3zy5w48x8-yazelix-package-source` |
| `codedb_import` | `nix_store_snjbgkf7yslaia8prs3p5pzsiigp5cy2-yazelix-light_toml_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:snjbgkf7yslaia8prs3p5pzsiigp5cy2-yazelix-light.toml.drv` |
| `codedb_import` | `nix_store_sq6zkgarcmj4z13555n17pkfm3d263mm-yazelix_zellij_bar-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:sq6zkgarcmj4z13555n17pkfm3d263mm-yazelix_zellij_bar-0.1.0` |
| `codedb_import` | `nix_store_sr4mq2m6da38qwswynh9wqshhfzzd1fc-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:sr4mq2m6da38qwswynh9wqshhfzzd1fc-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_sz4xbfci2i6jjhwbcmnwfg1m2lw6y8fm-yazelix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:sz4xbfci2i6jjhwbcmnwfg1m2lw6y8fm-yazelix` |
| `codedb_import` | `nix_store_v080m4w24q1v29ka43bn47qq1rqjac9v-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:v080m4w24q1v29ka43bn47qq1rqjac9v-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_v0s5xh687qmv5j53vilz92rl4a5bi1p5-yazelix-runtime-release-contracts_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:v0s5xh687qmv5j53vilz92rl4a5bi1p5-yazelix-runtime-release-contracts.drv` |
| `codedb_import` | `nix_store_v4icrk1n00iics1prj8b10chrw5cp3h5-yazelix_zellij_bar_tools-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:v4icrk1n00iics1prj8b10chrw5cp3h5-yazelix_zellij_bar_tools-0.1.0.drv` |
| `codedb_import` | `nix_store_v5wlpb7n8mwnzg468cdagkwjgs893zrm-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:v5wlpb7n8mwnzg468cdagkwjgs893zrm-yzx` |
| `codedb_import` | `nix_store_v654gb5kxkyzxvl85fs7h1518yh7lp37-parking_lot_core-0_9_12` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:v654gb5kxkyzxvl85fs7h1518yh7lp37-parking_lot_core-0.9.12` |
| `codedb_import` | `nix_store_vbx939b91rfn60mlcjkhwsx7ssfapm7z-yazelix-rust-core-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:vbx939b91rfn60mlcjkhwsx7ssfapm7z-yazelix-rust-core-source` |
| `codedb_import` | `nix_store_vfd172nza7f14z79ymvhl9fqf5h363pl-yazelix-rust-core-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:vfd172nza7f14z79ymvhl9fqf5h363pl-yazelix-rust-core-source` |
| `codedb_import` | `nix_store_vfgsw0jqx7dhfdjsbwgnby8idn196qig-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:vfgsw0jqx7dhfdjsbwgnby8idn196qig-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_vfnxhgx2g0vpxj3h2kgviss3rxsp1dw2-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:vfnxhgx2g0vpxj3h2kgviss3rxsp1dw2-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_vigb48b5nc6k8ccgb6qzzaczypapscmg-yazelix-runtime-release-contracts_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:vigb48b5nc6k8ccgb6qzzaczypapscmg-yazelix-runtime-release-contracts.drv` |
| `codedb_import` | `nix_store_vjjymwb7crddn8344g3b3v6zipz2wsys-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:vjjymwb7crddn8344g3b3v6zipz2wsys-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_vnnr6h3vvf17dwilp3bvrksc735wan9z-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:vnnr6h3vvf17dwilp3bvrksc735wan9z-yzx.drv` |
| `codedb_import` | `nix_store_vr4pi9jbv804yjih0vsm0r81ijlkimk2-yazelix_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:vr4pi9jbv804yjih0vsm0r81ijlkimk2-yazelix.drv` |
| `codedb_import` | `nix_store_w4d20i85sfqxnbnii7pr9w5dcy40ys7n-yazelix-rust-core-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:w4d20i85sfqxnbnii7pr9w5dcy40ys7n-yazelix-rust-core-source` |
| `codedb_import` | `nix_store_w4ny193vrz5b144w7w83xzpj29jc0ki8-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:w4ny193vrz5b144w7w83xzpj29jc0ki8-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_w9dy1m3f7azkwv2bsa68p4k5kpa13xym-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:w9dy1m3f7azkwv2bsa68p4k5kpa13xym-yazelix-runtime` |
| `codedb_import` | `nix_store_w9jk0ralya607byak2j0652qnwqi2b86-yazelix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:w9jk0ralya607byak2j0652qnwqi2b86-yazelix` |
| `codedb_import` | `nix_store_wfsy9v5vw886v7yzxf7xjyb5i3cj5h3w-sphinxcontrib_qthelp-2_0_0_tar_gz_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:wfsy9v5vw886v7yzxf7xjyb5i3cj5h3w-sphinxcontrib_qthelp-2.0.0.tar.gz.drv` |
| `codedb_import` | `nix_store_wfzmjlbqd4fazv5fryqsw3arbqkvm6d1-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:wfzmjlbqd4fazv5fryqsw3arbqkvm6d1-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_whaj2niz1wd3grgrhsn0vxpqbhrwz6m0-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:whaj2niz1wd3grgrhsn0vxpqbhrwz6m0-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_whid870sjs02lna7c5wjzkabzqsj6509-yazelix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:whid870sjs02lna7c5wjzkabzqsj6509-yazelix` |
| `codedb_import` | `nix_store_wl0rv6v2gh0x4cq311wl0cxnwi6qbkn9-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:wl0rv6v2gh0x4cq311wl0cxnwi6qbkn9-yzx.drv` |
| `codedb_import` | `nix_store_wl49kffz26r3siwn1mxs9s4dmk6rw8gp-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:wl49kffz26r3siwn1mxs9s4dmk6rw8gp-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_wlvmyf4z5blhp2cjv5vpkxd2jq7xdd32-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:wlvmyf4z5blhp2cjv5vpkxd2jq7xdd32-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_wm5haawkk3l4y4mczraljadcx59dyz6r-yazelix_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:wm5haawkk3l4y4mczraljadcx59dyz6r-yazelix.drv` |
| `codedb_import` | `nix_store_wqsifiawzkhgc3kdzkk7ll5v9g7pia19-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:wqsifiawzkhgc3kdzkk7ll5v9g7pia19-yzx.drv` |
| `codedb_import` | `nix_store_wvyw1wl0brvf5ql95my06mdjaafw3djy-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:wvyw1wl0brvf5ql95my06mdjaafw3djy-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_wx53ghr67rx5mszw4wqvq3l4l1a67gkn-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:wx53ghr67rx5mszw4wqvq3l4l1a67gkn-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_wxil0nyk2w32az8m1q8kipi6y3gd7x71-yazelix-runtime` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:wxil0nyk2w32az8m1q8kipi6y3gd7x71-yazelix-runtime` |
| `codedb_import` | `nix_store_x11r84jw66kb2g78hqy1q7rrsqa2avc8-yazelix_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:x11r84jw66kb2g78hqy1q7rrsqa2avc8-yazelix.drv` |
| `codedb_import` | `nix_store_x49a7rk2g6p163x9vyaqjc0xd8vwlg5s-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:x49a7rk2g6p163x9vyaqjc0xd8vwlg5s-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_x4hwn0qmp5cgig334ijj8hdvxdivq0pm-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:x4hwn0qmp5cgig334ijj8hdvxdivq0pm-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_x74dymzhmrr0vc4v82zflyf9kpdsnb1r-yazelix-rust-core-source` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:x74dymzhmrr0vc4v82zflyf9kpdsnb1r-yazelix-rust-core-source` |
| `codedb_import` | `nix_store_x7r33wi0ng47mvvd2nm72z0ljrbmv33s-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:x7r33wi0ng47mvvd2nm72z0ljrbmv33s-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_xdwcllf2sj4x4d9855s7nqp73af7bn0d-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:xdwcllf2sj4x4d9855s7nqp73af7bn0d-yzx.drv` |
| `codedb_import` | `nix_store_xg63vwv5cp2f67y0pffz6r4na0qxdxr3-yazelix_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:xg63vwv5cp2f67y0pffz6r4na0qxdxr3-yazelix.drv` |
| `codedb_import` | `nix_store_xlm3mniz9s0wanmbvdnzvqc89z9q2pfr-yazelix-runtime-release-contracts_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:xlm3mniz9s0wanmbvdnzvqc89z9q2pfr-yazelix-runtime-release-contracts.drv` |
| `codedb_import` | `nix_store_xn5ihvqz969akg1vpw6q0lhnamhilrlr-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:xn5ihvqz969akg1vpw6q0lhnamhilrlr-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_xqjqnwbcjwpyf47d7xphn6295cjzykv7-yazelix_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:xqjqnwbcjwpyf47d7xphn6295cjzykv7-yazelix.drv` |
| `codedb_import` | `nix_store_xrqqzzm9w5hzggl293k5shjgim1f6lv7-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:xrqqzzm9w5hzggl293k5shjgim1f6lv7-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_y1bi19rs2na61jl8821kg9f1plfl1bfb-yazelix-zellij-popup-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:y1bi19rs2na61jl8821kg9f1plfl1bfb-yazelix-zellij-popup-0.1.0` |
| `codedb_import` | `nix_store_y4348qn4afprxic8m6rxais40qnh00cl-yazelix_yazi_assets-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:y4348qn4afprxic8m6rxais40qnh00cl-yazelix_yazi_assets-0.1.0` |
| `codedb_import` | `nix_store_yaz2aqbwf5d0pr50blh3kg4qkyzjj71x-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:yaz2aqbwf5d0pr50blh3kg4qkyzjj71x-yzx.drv` |
| `codedb_import` | `nix_store_ydfw36nlq4slzk5pmgkarcmn6iifsj7y-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:ydfw36nlq4slzk5pmgkarcmn6iifsj7y-yzx` |
| `codedb_import` | `nix_store_ygsh6fsxz69vhv3fnsha7c0ycrskllhx-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:ygsh6fsxz69vhv3fnsha7c0ycrskllhx-yzx` |
| `codedb_import` | `nix_store_yz2223sp1d1wz2a483im1128nmbqwxh6-yazelix-core-0_1_0` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:yz2223sp1d1wz2a483im1128nmbqwxh6-yazelix-core-0.1.0` |
| `codedb_import` | `nix_store_yzfak6xm72jamqvm6ca1ayzxqv8l43jw-either-1_16_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:yzfak6xm72jamqvm6ca1ayzxqv8l43jw-either-1.16.0.drv` |
| `codedb_import` | `nix_store_yzx1bykapxdvq1g81ygk5n7vp95gxhqd-crate-ignore-0_4_25_tar_gz_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:yzx1bykapxdvq1g81ygk5n7vp95gxhqd-crate-ignore-0.4.25.tar.gz.drv` |
| `codedb_import` | `nix_store_z5182rdkv3j5418wxad8kr3fvhm8sk2d-yazelix-helix-25_7_1` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:z5182rdkv3j5418wxad8kr3fvhm8sk2d-yazelix-helix-25.7.1` |
| `codedb_import` | `nix_store_zidmq37id2yrvijpys0k9qzrklsijgmk-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:zidmq37id2yrvijpys0k9qzrklsijgmk-yzx.drv` |
| `codedb_import` | `nix_store_zpji39d116xr3q8nwxxw9lnbrpadk7si-yzx_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:zpji39d116xr3q8nwxxw9lnbrpadk7si-yzx.drv` |
| `codedb_import` | `nix_store_zql7cj6j12kpcmiyivv5jhw5w36drfmy-yazelix-core-0_1_0_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:zql7cj6j12kpcmiyivv5jhw5w36drfmy-yazelix-core-0.1.0.drv` |
| `codedb_import` | `nix_store_zrgy6ha5gnysj9x39gdf2xvyxiva8ka2-yazelix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:zrgy6ha5gnysj9x39gdf2xvyxiva8ka2-yazelix` |
| `codedb_import` | `nix_store_zrlwffhgz0c431lmalzgc1nk1rs0nbzk-yazelix-runtime_drv` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `nix_store:zrlwffhgz0c431lmalzgc1nk1rs0nbzk-yazelix-runtime.drv` |
| `codedb_import` | `nix_store_zy97ziv6isrrg34s8ry2q3rllwg4h5m0-yzx` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=directory parser_hint=directory structured_rows=0` | `nix_store:zy97ziv6isrrg34s8ry2q3rllwg4h5m0-yzx` |
| `codedb_import` | `yazelix_repo__github_actions_setup_nix_ubuntu_action_yml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=yaml structured_rows=30` | `yazelix_repo:.github/actions/setup_nix_ubuntu/action.yml` |
| `codedb_import` | `yazelix_repo__github_workflows_publish_nix_cache_yml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=yaml structured_rows=38` | `yazelix_repo:.github/workflows/publish_nix_cache.yml` |
| `codedb_import` | `yazelix_repo__kb_store_documents_tasks_envctl-pr409-cargo-audit-advisories_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=75` | `yazelix_repo:.kb/store/documents/tasks/envctl-pr409-cargo-audit-advisories.md` |
| `codedb_import` | `yazelix_repo__kb_store_documents_tasks_nu-plugin-codedb-cargo-metadata_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=38` | `yazelix_repo:.kb/store/documents/tasks/nu-plugin-codedb-cargo-metadata.md` |
| `codedb_import` | `yazelix_repo__kb_store_documents_tasks_nu-plugin-codedb-cargo-sources_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=39` | `yazelix_repo:.kb/store/documents/tasks/nu-plugin-codedb-cargo-sources.md` |
| `codedb_import` | `yazelix_repo__kb_store_documents_tasks_nu-plugin-codedb-rust-items_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=44` | `yazelix_repo:.kb/store/documents/tasks/nu-plugin-codedb-rust-items.md` |
| `codedb_import` | `yazelix_repo__kb_store_documents_tasks_nu-plugin-codedb-rust-workspace-skeleton_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=48` | `yazelix_repo:.kb/store/documents/tasks/nu-plugin-codedb-rust-workspace-skeleton.md` |
| `codedb_import` | `yazelix_repo__kb_store_documents_tasks_yazelix-ci-beads-rust-absolute-path_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=35` | `yazelix_repo:.kb/store/documents/tasks/yazelix-ci-beads-rust-absolute-path.md` |
| `codedb_import` | `yazelix_repo__kb_workspaces_main_tasks_envctl-pr409-cargo-audit-advisories_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=75` | `yazelix_repo:.kb/workspaces/main/tasks/envctl-pr409-cargo-audit-advisories.md` |
| `codedb_import` | `yazelix_repo__kb_workspaces_main_tasks_nu-plugin-codedb-cargo-metadata_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=38` | `yazelix_repo:.kb/workspaces/main/tasks/nu-plugin-codedb-cargo-metadata.md` |
| `codedb_import` | `yazelix_repo__kb_workspaces_main_tasks_nu-plugin-codedb-cargo-sources_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=39` | `yazelix_repo:.kb/workspaces/main/tasks/nu-plugin-codedb-cargo-sources.md` |
| `codedb_import` | `yazelix_repo__kb_workspaces_main_tasks_nu-plugin-codedb-rust-items_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=44` | `yazelix_repo:.kb/workspaces/main/tasks/nu-plugin-codedb-rust-items.md` |
| `codedb_import` | `yazelix_repo__kb_workspaces_main_tasks_nu-plugin-codedb-rust-workspace-skeleton_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=48` | `yazelix_repo:.kb/workspaces/main/tasks/nu-plugin-codedb-rust-workspace-skeleton.md` |
| `codedb_import` | `yazelix_repo__kb_workspaces_main_tasks_yazelix-ci-beads-rust-absolute-path_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=35` | `yazelix_repo:.kb/workspaces/main/tasks/yazelix-ci-beads-rust-absolute-path.md` |
| `codedb_import` | `yazelix_repo_config_metadata_rust_ownership_budget_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=68` | `yazelix_repo:config_metadata/rust_ownership_budget.toml` |
| `codedb_import` | `yazelix_repo_docs_contracts_nix_control_plane_boundary_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=98` | `yazelix_repo:docs/contracts/nix_control_plane_boundary.md` |
| `codedb_import` | `yazelix_repo_docs_contracts_nix_customization_surfaces_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=106` | `yazelix_repo:docs/contracts/nix_customization_surfaces.md` |
| `codedb_import` | `yazelix_repo_docs_contracts_nixpkgs_package_contract_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=132` | `yazelix_repo:docs/contracts/nixpkgs_package_contract.md` |
| `codedb_import` | `yazelix_repo_docs_contracts_rust_nextest_harness_boundary_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=110` | `yazelix_repo:docs/contracts/rust_nextest_harness_boundary.md` |
| `codedb_import` | `yazelix_repo_docs_contracts_rust_nushell_bridge_contract_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=218` | `yazelix_repo:docs/contracts/rust_nushell_bridge_contract.md` |
| `codedb_import` | `yazelix_repo_docs_rust_code_inventory_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=119` | `yazelix_repo:docs/rust_code_inventory.md` |
| `codedb_import` | `yazelix_repo_docs_rust_maintainer_tooling_boundary_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=137` | `yazelix_repo:docs/rust_maintainer_tooling_boundary.md` |
| `codedb_import` | `yazelix_repo_flake_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=350` | `yazelix_repo:flake.nix` |
| `codedb_import` | `yazelix_repo_home_manager_examples_example_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=80` | `yazelix_repo:home_manager/examples/example.nix` |
| `codedb_import` | `yazelix_repo_home_manager_examples_minimal_flake_flake_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=32` | `yazelix_repo:home_manager/examples/minimal_flake/flake.nix` |
| `codedb_import` | `yazelix_repo_home_manager_examples_minimal_flake_home_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=9` | `yazelix_repo:home_manager/examples/minimal_flake/home.nix` |
| `codedb_import` | `yazelix_repo_home_manager_module_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=103` | `yazelix_repo:home_manager/module.nix` |
| `codedb_import` | `yazelix_repo_home_manager_options_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=406` | `yazelix_repo:home_manager/options.nix` |
| `codedb_import` | `yazelix_repo_home_manager_runtime_integration_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=192` | `yazelix_repo:home_manager/runtime_integration.nix` |
| `codedb_import` | `yazelix_repo_home_manager_settings_contract_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=251` | `yazelix_repo:home_manager/settings_contract.nix` |
| `codedb_import` | `yazelix_repo_maintainer_shell_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=182` | `yazelix_repo:maintainer_shell.nix` |
| `codedb_import` | `yazelix_repo_packaging_beads_rust_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=23` | `yazelix_repo:packaging/beads_rust.nix` |
| `codedb_import` | `yazelix_repo_packaging_cargo_crap_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=23` | `yazelix_repo:packaging/cargo_crap.nix` |
| `codedb_import` | `yazelix_repo_packaging_flake_outputs_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=112` | `yazelix_repo:packaging/flake_outputs.nix` |
| `codedb_import` | `yazelix_repo_packaging_install_check_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=20` | `yazelix_repo:packaging/install_check.nix` |
| `codedb_import` | `yazelix_repo_packaging_kgp_package_contracts_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=85` | `yazelix_repo:packaging/kgp_package_contracts.nix` |
| `codedb_import` | `yazelix_repo_packaging_kgp_packages_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=24` | `yazelix_repo:packaging/kgp_packages.nix` |
| `codedb_import` | `yazelix_repo_packaging_mk_runtime_tree_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=279` | `yazelix_repo:packaging/mk_runtime_tree.nix` |
| `codedb_import` | `yazelix_repo_packaging_mk_yazelix_package_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=88` | `yazelix_repo:packaging/mk_yazelix_package.nix` |
| `codedb_import` | `yazelix_repo_packaging_nixpkgs_default_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=17` | `yazelix_repo:packaging/nixpkgs/default.nix` |
| `codedb_import` | `yazelix_repo_packaging_nixpkgs_submission_notes_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=39` | `yazelix_repo:packaging/nixpkgs/submission_notes.md` |
| `codedb_import` | `yazelix_repo_packaging_nixpkgs_yazelix_package_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=30` | `yazelix_repo:packaging/nixpkgs/yazelix_package.nix` |
| `codedb_import` | `yazelix_repo_packaging_repo_source_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=52` | `yazelix_repo:packaging/repo_source.nix` |
| `codedb_import` | `yazelix_repo_packaging_runtime_component_registry_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=44` | `yazelix_repo:packaging/runtime_component_registry.nix` |
| `codedb_import` | `yazelix_repo_packaging_runtime_deps_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=13` | `yazelix_repo:packaging/runtime_deps.nix` |
| `codedb_import` | `yazelix_repo_packaging_runtime_release_contracts_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=28` | `yazelix_repo:packaging/runtime_release_contracts.nix` |
| `codedb_import` | `yazelix_repo_packaging_runtime_tool_registry_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=411` | `yazelix_repo:packaging/runtime_tool_registry.nix` |
| `codedb_import` | `yazelix_repo_packaging_rust_core_helper_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=68` | `yazelix_repo:packaging/rust_core_helper.nix` |
| `codedb_import` | `yazelix_repo_packaging_tokenusage_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=27` | `yazelix_repo:packaging/tokenusage.nix` |
| `codedb_import` | `yazelix_repo_packaging_yazelix_kgp_zellij_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=36` | `yazelix_repo:packaging/yazelix_kgp_zellij.nix` |
| `codedb_import` | `yazelix_repo_packaging_yazelix_zellij_config_pack_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=100` | `yazelix_repo:packaging/yazelix_zellij_config_pack.nix` |
| `codedb_import` | `yazelix_repo_rust_core_Cargo_lock` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/Cargo.lock` |
| `codedb_import` | `yazelix_repo_rust_core_Cargo_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=4` | `yazelix_repo:rust_core/Cargo.toml` |
| `codedb_import` | `yazelix_repo_rust_core__config_nextest_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=6` | `yazelix_repo:rust_core/.config/nextest.toml` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_Cargo_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=46` | `yazelix_repo:rust_core/yazelix_core/Cargo.toml` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_assets_ascii_art_data_json` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=json structured_rows=68` | `yazelix_repo:rust_core/yazelix_core/assets/ascii_art_data.json` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_action_registry_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/action_registry.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_active_config_surface_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/active_config_surface.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_agent_commands_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/agent_commands.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_appearance_mode_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/appearance_mode.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_atomic_fs_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/atomic_fs.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_backup_timestamp_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/backup_timestamp.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_bin_yzx_control_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/bin/yzx_control.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_bin_yzx_core_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/bin/yzx_core.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_bin_yzx_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/bin/yzx.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_bridge_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/bridge.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_cli_render_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/cli_render.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_command_metadata_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/command_metadata.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_config_apply_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/config_apply.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_config_commands_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/config_commands.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_config_normalize_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/config_normalize.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_config_state_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/config_state.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_config_ui_app_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/config_ui/app.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_config_ui_apply_adapter_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/config_ui/apply_adapter.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_config_ui_custom_popups_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/config_ui/custom_popups.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_config_ui_details_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/config_ui/details.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_config_ui_keybindings_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/config_ui/keybindings.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_config_ui_model_builder_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/config_ui/model_builder.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_config_ui_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/config_ui.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_config_ui_tests_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/config_ui/tests.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_control_plane_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/control_plane.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_cursor_commands_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/cursor_commands.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_desktop_exec_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/desktop_exec.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_doctor_commands_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/doctor_commands.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_doctor_config_report_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/doctor_config_report.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_doctor_helix_report_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/doctor_helix_report.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_doctor_runtime_report_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/doctor_runtime_report.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_doctor_zellij_plugin_health_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/doctor_zellij_plugin_health.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_edit_commands_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/edit_commands.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_front_door_commands_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/front_door_commands.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_front_door_render_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/front_door_render.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_ghostty_cursor_registry_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/ghostty_cursor_registry.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_helix_bridge_client_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/helix_bridge_client.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_helix_external_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/helix_external.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_helix_materialization_helix_config_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/helix_materialization/helix_config.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_helix_materialization_import_notice_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/helix_materialization/import_notice.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_helix_materialization_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/helix_materialization.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_helix_materialization_steel_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/helix_materialization/steel.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_helix_materialization_tests_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/helix_materialization/tests.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_helix_steel_plugins_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/helix_steel_plugins.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_home_manager_commands_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/home_manager_commands.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_import_commands_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/import_commands.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_initializer_commands_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/initializer_commands.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_install_ownership_env_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/install_ownership_env.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_install_ownership_report_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/install_ownership_report.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_keys_commands_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/keys_commands.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_launch_commands_config_override_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/launch_commands/config_override.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_launch_commands_desktop_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/launch_commands/desktop.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_launch_commands_enter_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/launch_commands/enter.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_launch_commands_launch_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/launch_commands/launch.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_launch_commands_process_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/launch_commands/process.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_launch_commands_restart_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/launch_commands/restart.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_launch_commands_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/launch_commands.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_launch_commands_terminal_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/launch_commands/terminal.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_launch_materialization_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/launch_materialization.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_lib_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/lib.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_managed_user_config_stubs_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/managed_user_config_stubs.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_menu_commands_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/menu_commands.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_native_config_status_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/native_config_status.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_onboard_commands_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/onboard_commands.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_pane_orchestrator_client_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/pane_orchestrator_client.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_popup_runtime_command_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/popup_runtime_command.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_popup_session_facts_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/popup_session_facts.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_profile_commands_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/profile_commands.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_public_command_surface_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/public_command_surface.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_reset_commands_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/reset_commands.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_runtime_apply_mode_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/runtime_apply_mode.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_runtime_components_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/runtime_components.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_runtime_contract_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/runtime_contract.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_runtime_env_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/runtime_env.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_runtime_materialization_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/runtime_materialization.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_runtime_ownership_graph_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/runtime_ownership_graph.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_session_config_snapshot_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/session_config_snapshot.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_session_facts_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/session_facts.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_settings_contract_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/settings_contract.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_settings_jsonc_patch_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/settings_jsonc_patch.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_settings_surface_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/settings_surface.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_sidebar_bootstrap_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/sidebar_bootstrap.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_startup_facts_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/startup_facts.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_startup_handoff_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/startup_handoff.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_status_report_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/status_report.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_support_commands_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/support_commands.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_terminal_control_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/terminal_control.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_terminal_cursor_materialization_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/terminal_cursor_materialization.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_terminal_materialization_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/terminal_materialization.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_terminal_variant_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/terminal_variant.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_tutor_document_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/tutor_document.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_update_commands_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/update_commands.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_upgrade_summary_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/upgrade_summary.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_user_config_paths_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/user_config_paths.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_workspace_asset_contract_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/workspace_asset_contract.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_workspace_commands_popup_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/workspace_commands/popup.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_workspace_commands_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/workspace_commands.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_workspace_commands_yazi_sidebar_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/workspace_commands/yazi_sidebar.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_workspace_session_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/workspace_session.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_yazi_materialization_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/yazi_materialization.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_yazi_materialization_writer_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/yazi_materialization/writer.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_zellij_commands_pipe_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/zellij_commands/pipe.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_zellij_commands_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/zellij_commands.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_zellij_commands_status_agent_usage_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/zellij_commands/status/agent_usage.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_zellij_commands_status_cache_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/zellij_commands/status/cache.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_zellij_commands_status_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/zellij_commands/status.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_zellij_commands_status_tests_cache_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/zellij_commands/status/tests/cache.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_zellij_commands_status_tests_mod_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/zellij_commands/status/tests/mod.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_zellij_commands_status_tests_widgets_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/zellij_commands/status/tests/widgets.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_zellij_commands_status_widgets_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/zellij_commands/status/widgets.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_zellij_commands_workspace_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/zellij_commands/workspace.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_zellij_materialization_io_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/zellij_materialization_io.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_zellij_materialization_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/zellij_materialization.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_src_zellij_plugin_materialization_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/src/zellij_plugin_materialization.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_tests_support_commands_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/tests/support/commands.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_tests_support_envelopes_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/tests/support/envelopes.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_tests_support_fixtures_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/tests/support/fixtures.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_tests_support_mod_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/tests/support/mod.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_tests_yzx_control_helix_bridge_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/tests/yzx_control_helix_bridge.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_tests_yzx_control_public_commands_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/tests/yzx_control_public_commands.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_tests_yzx_control_runtime_surface_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/tests/yzx_control_runtime_surface.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_tests_yzx_control_workspace_surface_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/tests/yzx_control_workspace_surface.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_tests_yzx_core_config_edit_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/tests/yzx_core_config_edit.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_tests_yzx_core_config_ui_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/tests/yzx_core_config_ui.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_tests_yzx_core_runtime_env_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/tests/yzx_core_runtime_env.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_tests_yzx_core_settings_jsonc_patch_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/tests/yzx_core_settings_jsonc_patch.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_core_tests_yzx_core_yazi_materialization_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_core/tests/yzx_core_yazi_materialization.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_Cargo_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=14` | `yazelix_repo:rust_core/yazelix_maintainer/Cargo.toml` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_bin_yzx_repo_maintainer_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/bin/yzx_repo_maintainer.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_bin_yzx_repo_validator_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/bin/yzx_repo_validator.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_lib_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/lib.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_repo_canary_session_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/repo_canary_session.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_repo_child_release_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/repo_child_release.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_repo_contract_validation_config_surface_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/repo_contract_validation/config_surface.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_repo_contract_validation_helpers_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/repo_contract_validation/helpers.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_repo_contract_validation_installed_runtime_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/repo_contract_validation/installed_runtime.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_repo_contract_validation_nix_interface_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/repo_contract_validation/nix_interface.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_repo_contract_validation_nix_package_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/repo_contract_validation/nix_package.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_repo_contract_validation_readme_surface_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/repo_contract_validation/readme_surface.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_repo_contract_validation_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/repo_contract_validation.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_repo_contract_validation_upgrade_contract_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/repo_contract_validation/upgrade_contract.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_repo_docs_validation_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/repo_docs_validation.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_repo_issue_sync_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/repo_issue_sync.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_repo_nu_lint_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/repo_nu_lint.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_repo_release_workflow_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/repo_release_workflow.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_repo_rust_budget_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/repo_rust_budget.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_repo_rust_commands_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/repo_rust_commands.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_repo_sweep_runner_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/repo_sweep_runner.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_repo_test_runner_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/repo_test_runner.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_repo_update_workflow_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/repo_update_workflow.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_repo_validation_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/repo_validation.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_repo_version_bump_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/repo_version_bump.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_repo_yazelix_file_inventory_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/repo_yazelix_file_inventory.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_src_workspace_session_contract_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/src/workspace_session_contract.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_tests_repo_upgrade_contract_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/tests/repo_upgrade_contract.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_maintainer_tests_yazelix_file_target_inventory_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_maintainer/tests/yazelix_file_target_inventory.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_zellij_config_pack_Cargo_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=15` | `yazelix_repo:rust_core/yazelix_zellij_config_pack/Cargo.toml` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_zellij_config_pack_LICENSE` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=plain_or_binary structured_rows=0` | `yazelix_repo:rust_core/yazelix_zellij_config_pack/LICENSE` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_zellij_config_pack_README_md` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=markdown structured_rows=21` | `yazelix_repo:rust_core/yazelix_zellij_config_pack/README.md` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_zellij_config_pack_config_metadata_zellij_layout_families_toml` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=toml structured_rows=7` | `yazelix_repo:rust_core/yazelix_zellij_config_pack/config_metadata/zellij_layout_families.toml` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_zellij_config_pack_layouts_fragments_swap_agent_closed_kdl` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=kdl structured_rows=13` | `yazelix_repo:rust_core/yazelix_zellij_config_pack/layouts/fragments/swap_agent_closed.kdl` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_zellij_config_pack_layouts_fragments_swap_agent_open_kdl` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=kdl structured_rows=13` | `yazelix_repo:rust_core/yazelix_zellij_config_pack/layouts/fragments/swap_agent_open.kdl` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_zellij_config_pack_layouts_fragments_swap_sidebar_closed_kdl` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=kdl structured_rows=13` | `yazelix_repo:rust_core/yazelix_zellij_config_pack/layouts/fragments/swap_sidebar_closed.kdl` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_zellij_config_pack_layouts_fragments_swap_sidebar_open_kdl` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=kdl structured_rows=13` | `yazelix_repo:rust_core/yazelix_zellij_config_pack/layouts/fragments/swap_sidebar_open.kdl` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_zellij_config_pack_layouts_yzx_side_kdl` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=kdl structured_rows=13` | `yazelix_repo:rust_core/yazelix_zellij_config_pack/layouts/yzx_side.kdl` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_zellij_config_pack_layouts_yzx_side_swap_kdl` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=kdl structured_rows=13` | `yazelix_repo:rust_core/yazelix_zellij_config_pack/layouts/yzx_side.swap.kdl` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_zellij_config_pack_src_bin_yazelix_zellij_config_pack_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_zellij_config_pack/src/bin/yazelix_zellij_config_pack.rs` |
| `codedb_import` | `yazelix_repo_rust_core_yazelix_zellij_config_pack_src_lib_rs` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=opaque structured_rows=0` | `yazelix_repo:rust_core/yazelix_zellij_config_pack/src/lib.rs` |
| `codedb_import` | `yazelix_repo_yazelix_package_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=66` | `yazelix_repo:yazelix_package.nix` |
| `codedb_import` | `yazelix_repo_yazelix_runtime_package_nix` | `home/flexnetos/FlexNetOS/src/yazelix/docs/generated/yazelix_file_target_inventory.json` | `file_kind=regular_file parser_hint=nix structured_rows=56` | `yazelix_repo:yazelix_runtime_package.nix` |
| `env_var` | `ENVCTL_LEGACY_TOOLCHAINS` | `crates/engine/src/layout.rs` | `producer=layout scope=layout sensitive=false` | `/home/flexnetos/FlexNetOS/.toolchains` |
| `path` | `binary` | `crates/engine/src/layout.rs` | `artifact_kind=compatibility secretd/secretctl binary prefix canonical=false legacy=true bridge=true verification=missing` | `/home/flexnetos/FlexNetOS/.toolchains/secrets/bin` |
| `path` | `path` | `crates/engine/src/layout.rs` | `artifact_kind=shared secrets-stack data such as trust roots canonical=true legacy=false bridge=false verification=missing` | `/home/flexnetos/FlexNetOS/usr/share/envctl/secrets` |
| `path` | `toolchain_root` | `crates/engine/src/layout.rs` | `artifact_kind=compatibility prefix for manifests not yet migrated to the FHS layout canonical=false legacy=true bridge=true verification=dir_exists` | `/home/flexnetos/FlexNetOS/.toolchains` |
| `setting` | `component[0].description` | `manifest/cognitum-seed-trust.toml` | `scope=component source_kind=manifest` | `Auto-refreshes the pinned Cognitum Device CA (the meta path $META_ROOT/etc/envctl/secrets/ca/cognitum-ca.crt that secretd reads via ENVCTL_SEED_CA) from the USB trust anchor (COGNITUM/trust/cognitum-ca.pem) on boot + Seed hotplug, so the vault USB-unlock factor keeps working across Seed CA rotation. Additive; never removes other CAs; absent Seed = no-op.` |
| `setting` | `component[0].description` | `manifest/components.d/codex-global-baseline.toml` | `scope=component source_kind=manifest` | `Owns the Codex global surface for meta: ~/.codex symlink, Rust CLI env vars, supported feature flags, MCP baseline, FlexNetOS Codex marketplace/plugin, hooks, and trust entries for every meta repo.` |
| `setting` | `component[0].description` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `gh from upstream release tarball into .toolchains/gh + a regular $META_ROOT/usr/bin frontdoor wrapper. Replaces the apt \`gh\`. Repo: cli/cli.` |
| `setting` | `component[0].description` | `manifest/components.d/handoff-hf.toml` | `scope=component source_kind=manifest` | `Builds the meta/handoff Rust continuity kernel and exposes the canonical hf and hf-mcp front doors under $META_ROOT/usr/bin; never links hf to real-home Cargo state.` |
| `setting` | `component[0].description` | `manifest/components.d/just.toml` | `scope=component source_kind=manifest` | `Built from source via envctl add-repo (build-from-source; only runs when named).
strategy=as-is source=https://github.com/casey/just ref=(default branch) sha=5097d64c8b765f8f6bf0f19d13be199bb1d1769c build_system=cargo
artifacts=$META_ROOT/usr/bin/just
` |
| `setting` | `component[0].description` | `manifest/components.d/meta-core-clis.toml` | `scope=component source_kind=manifest` | `Git 2.54.0 copied into $META_ROOT/.toolchains/git as a real binary tree. $META_ROOT/usr/bin/git is a regular executable frontdoor wrapper for the meta-hosted git executable.` |
| `setting` | `component[0].description` | `manifest/components.d/secretd.toml` | `scope=component source_kind=manifest` | `Pure-Rust gRPC secrets vault + credential broker, served as a systemd user service.` |
| `setting` | `component[0].description` | `manifest/components.d/zsh-migration-launcher.toml` | `scope=component source_kind=manifest` | `One-click desktop app + icon launching the meta zsh (.toolchains/zsh) with the ohmyzsh ZDOTDIR. Drives the env migration; no login-shell change.` |
| `setting` | `component[0].description` | `manifest/components.d/zsh.toml` | `scope=component source_kind=manifest` | `Latest stable zsh built from source (zsh-users/zsh) into .toolchains/zsh + $META_ROOT/usr/bin/zsh. Meta-owned shell outside /nix; the migration terminal. Unpinned: resolves newest tag at install.` |
| `setting` | `component[0].description` | `manifest/env-ctl.toml` | `scope=component source_kind=manifest` | `Pure-Rust gRPC secrets vault + credential broker (secretd/secretctl). AEAD-at-rest, <=24h peer-bound relay bearers, in-process TLS for the relay edge. Control = UDS + SO_PEERCRED owner-only; data = loopback.` |
| `setting` | `component[0].description` | `manifest/grit.toml` | `scope=component source_kind=manifest` | `Function-level (tree-sitter) git locking so N AI agents edit the same files without merge conflicts. External tool binary on $META_ROOT/.toolchains/cargo/bin; NOT linked into any envctl crate.` |
| `setting` | `component[0].description` | `manifest/prompt_hub.toml` | `scope=component source_kind=manifest` | `Builds + installs the prompthub CLI and prompthub-server binaries from the meta-cloned prompt_hub checkout. External product binary on $META_ROOT/.toolchains/cargo/bin; NOT linked into any envctl crate. Declares prompt_hub's build needs (rustup + a C compiler + OpenSSL headers) so the repo is provisioned, not assumed-broken.` |
| `setting` | `component[0].description` | `manifest/rusty-idd.toml` | `scope=component source_kind=manifest` | `Single-binary IDD engine + fleet-deploy control plane (ADR-0015). The thin-adapter SessionStart hooks deployed across the fleet call \`rusty-idd next\`, so the binary must be on PATH box-wide. External tool binary installed under $META_ROOT/.toolchains/rusty-idd and exposed through a regular $META_ROOT/usr/bin frontdoor wrapper; NOT linked into any envctl crate.` |
| `setting` | `component[0].description` | `manifest/sqld.toml` | `scope=component source_kind=manifest` | `Prebuilt libSQL server (sqld) bound to 127.0.0.1:8080 as secretd's durable store backend. External binary OUTSIDE the trust boundary (installed from a release tarball, NOT built from source) so no C-SQLite is linked into the workspace.` |
| `setting` | `component[0].detect.args` | `manifest/ai-clis.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"\nR=\"${ENVCTL_REAL_HOME:-$HOME}\"\ncur=\"$M/.toolchains/claude/current/bin/claude\"\nlink=\"$M/usr/bin/claude\"\nmeta_compat=\"$M/.local/bin/claude\"\nreal_compat=\"$R/.local/bin/claude\"\n[ -x \"$cur\" ]\n[ -x \"$link\" ] && [ ! -L \"$link\" ] && grep -q \"envctl claude wrapper\" \"$link\"\n[ -x \"$meta_compat\" ] && grep -q \"envctl claude wrapper\" \"$meta_compat\"\n[ -x \"$real_compat\" ] && grep -q \"envctl claude wrapper\" \"$real_compat\"\n[ \"$(readlink -f \"$R/.claude\" 2>/dev/null)\" = \"$(readlink -f \"$M/.local/share/claude\" 2>/dev/null)\" ]\ntimeout --kill-after=2s 8s \"$link\" --version >/dev/null\nCLEANUP=\"${ENVCTL_CLAUDE_CLEANUP:-$M/envctl/assets/scripts/envctl-claude-cleanup.sh}\"\n[ -x \"$CLEANUP\" ] || CLEANUP=\"$PWD/assets/scripts/envctl-claude-cleanup.sh\"\n\"$CLEANUP\" verify\n"]` |
| `setting` | `component[0].detect.args` | `manifest/cognitum-seed-trust.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; test -x \"$M/usr/libexec/envctl/cognitum-seed-trust-refresh\" && test -f /etc/systemd/system/cognitum-seed-trust.service && test -f /etc/udev/rules.d/99-cognitum-seed-trust.rules"]` |
| `setting` | `component[0].detect.args` | `manifest/components.d/codex-global-baseline.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"\nR=\"${ENVCTL_REAL_HOME:-$HOME}\"\nC=\"$M/.local/share/codex/config.toml\"\nH=\"$M/.local/share/codex/hooks.json\"\n[ \"$(readlink -f \"$R/.codex\" 2>/dev/null)\" = \"$(readlink -f \"$M/.local/share/codex\" 2>/dev/null)\" ] || exit 1\n[ \"$(readlink -f \"$R/.local/bin/codex\" 2>/dev/null)\" = \"$(readlink -f \"$M/usr/bin/codex\" 2>/dev/null)\" ] || exit 1\n[ -f \"$C\" ] && [ -f \"$H\" ] || exit 1\ngrep -q '^model = \"gpt-5.5\"' \"$C\" || exit 1\ngrep -q '^service_tier = \"fast\"' \"$C\" || exit 1\ngrep -q '^model_auto_compact_token_limit = 3000000' \"$C\" || exit 1\ngrep -q '^web_search = \"live\"' \"$C\" || exit 1\ngrep -q '^background_terminal_max_timeout = 300000' \"$C\" || exit 1\ngrep -q '^tool_output_token_limit = 12000' \"$C\" || exit 1\nfor f in apps auto_compaction browser_use browser_use_external computer_use enable_request_compression fast_mode goals guardian_approval hooks image_generation in_app_browser mentions_v2 multi_agent personality plugin_sharing plugins prevent_idle_sleep remote_compaction_v2 secret_auth_storage shell_snapshot shell_tool skill_mcp_dependency_install tool_call_mcp_elicitation tool_suggest unified_exec workspace_dependencies memories network_proxy; do\n  grep -q \"^$f = true\" \"$C\" || { echo \"missing feature: $f\"; exit 1; }\ndone\nfor s in meta gitkb icm vox context7 weave openaiDeveloperDocs; do\n  grep -q \"^\\[mcp_servers\\.$s\\]\" \"$C\" || { echo \"missing mcp: $s\"; exit 1; }\ndone\ngrep -q '^\\[marketplaces\\.flexnetos-codex\\]' \"$C\" || exit 1\ngrep -q '^\\[plugins\\.\"meta@flexnetos-codex\"\\]' \"$C\" || exit 1\ngrep -q '^\\[plugins\\.\"harness@harness-marketplace\"\\]' \"$C\" || exit 1\ngrep -q '^enabled = true' \"$C\" || exit 1\ngrep -q '^default_permissions = \"meta-workspace\"' \"$C\" || exit 1\nfor a in meta-worker pr-explorer reviewer docs-researcher codex-baseline-researcher; do [ -f \"$M/.local/share/codex/agents/$a.toml\" ] || { echo \"missing agent: $a\"; exit 1; }; done\nfor slug in gpt-5.5 gpt-5.4-mini gpt-5.4-nano gpt-5.4 gpt-5.3-codex-spark gpt-5.6-sol gpt-5.6-terra gpt-5.6-luna; do grep -q '\"slug\": '\"\\\"$slug\\\"\" \"$M/.local/share/codex/model-catalog.json\" || { echo \"missing model catalog slug: $slug\"; exit 1; }; done\ngrep -q 'Official OpenAI GPT-5.6 Sol limited-preview' \"$M/.local/share/codex/model-catalog.json\" || { echo \"stale GPT-5.6 Sol model catalog description\"; exit 1; }\nfor var in FXN_AGENT_COMMUNICATION FXN_AGENT_TEAM_SWARM FXN_AGENT_TEAM_SWARM_MODEL_TAGS FXN_CODEX_MODEL_SOL FXN_CODEX_MODEL_TERRA FXN_CODEX_MODEL_LUNA FXN_CODEX_REMOTE_CONTROL; do grep -q \"export $var=\" \"$M/.codex/hooks/with-meta-env.sh\" || { echo \"missing hook env var: $var\"; exit 1; }; done\nCODEX_HOME=\"$M/.local/share/codex\" CODEX_SQLITE_HOME=\"$M/.local/state/codex\" \"$M/usr/bin/codex\" plugin list | grep -q 'meta@flexnetos-codex  installed, enabled' || exit 1\nCODEX_HOME=\"$M/.local/share/codex\" CODEX_SQLITE_HOME=\"$M/.local/state/codex\" \"$M/usr/bin/codex\" plugin list | grep -q 'harness@harness-marketplace  installed, enabled' || exit 1\nCODEX_HOME=\"$M/.local/share/codex\" CODEX_SQLITE_HOME=\"$M/.local/state/codex\" \"$M/usr/bin/codex\" mcp list >/dev/null || exit 1\n! find \"$M/.toolchains/bun\" \"$M/.toolchains/.bun\" -path '*/@openai/codex*' -print -quit 2>/dev/null | grep -q .\n\"$M/envctl/assets/scripts/envctl-codex-cleanup.sh\" verify\n"]` |
| `setting` | `component[0].detect.args` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/gh/bin/gh\" ] && [ -x \"$META_ROOT/usr/bin/gh\" ] && [ ! -L \"$META_ROOT/usr/bin/gh\" ] && grep -Fqx \"exec \\\"$M/.toolchains/gh/bin/gh\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/gh\""]` |
| `setting` | `component[0].detect.args` | `manifest/components.d/just.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/src/just/target/release/just\" ] && [ -x \"$M/usr/bin/just\" ] && [ ! -L \"$M/usr/bin/just\" ] && grep -Fqx \"exec \\\"$M/.toolchains/src/just/target/release/just\\\" \\\"\\$@\\\"\" \"$M/usr/bin/just\""]` |
| `setting` | `component[0].detect.args` | `manifest/components.d/meta-core-clis.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/git/bin/git\" ] && [ -x \"$META_ROOT/usr/bin/git\" ] && [ ! -L \"$META_ROOT/usr/bin/git\" ] && grep -Fqx \"exec \\\"$M/.toolchains/git/bin/git\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/git\""]` |
| `setting` | `component[0].detect.args` | `manifest/components.d/ohmyzsh.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; C=\"$M/.toolchains/zsh-config\"; [ -f \"$M/ohmyzsh/oh-my-zsh.sh\" ] && [ -f \"$C/.zshrc\" ] && [ -f \"$C/custom/plugins/zsh-autosuggestions/zsh-autosuggestions.zsh\" ] && [ -f \"$C/custom/plugins/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh\" ] && [ -d \"$C/custom/plugins/zsh-completions\" ] && [ -f \"$C/custom/themes/powerlevel10k/powerlevel10k.zsh-theme\" ]"]` |
| `setting` | `component[0].detect.args` | `manifest/components.d/zsh-migration-launcher.toml` | `scope=component source_kind=manifest` | `["-lc","[ -f \"$META_ROOT/.local/share/applications/meta-zsh-migration.desktop\" ] && [ -x \"${META_ROOT:?META_ROOT required}/.toolchains/zsh-config/launch.sh\" ]"]` |
| `setting` | `component[0].detect.args` | `manifest/components.d/zsh.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/zsh/bin/zsh\" ] && [ -x \"$META_ROOT/usr/bin/zsh\" ] && [ ! -L \"$META_ROOT/usr/bin/zsh\" ] && grep -Fqx \"exec \\\"$M/.toolchains/zsh/bin/zsh\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/zsh\""]` |
| `setting` | `component[0].detect.args` | `manifest/grit.toml` | `scope=component source_kind=manifest` | `["-lc","export CARGO_HOME=\"$META_ROOT/.toolchains/cargo\"; export PATH=\"$CARGO_HOME/bin:$PATH\"; command -v grit >/dev/null || [ -x \"$META_ROOT/.toolchains/cargo/bin/grit\" ]"]` |
| `setting` | `component[0].detect.args` | `manifest/n8n-mcp.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export PATH=\"$M/usr/bin:$M/.toolchains/.bun/bin:$M/.toolchains/node/bin:$PATH\"; [ -x \"$M/usr/bin/bunx\" ]"]` |
| `setting` | `component[0].detect.args` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/nix/bin/nix\" ] && [ -x \"$META_ROOT/usr/bin/nix\" ] && [ ! -L \"$META_ROOT/usr/bin/nix\" ] && grep -Fqx \"exec \\\"$M/.toolchains/nix/bin/nix\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/nix\" && [ -e /nix ]"]` |
| `setting` | `component[0].detect.args` | `manifest/odysseus.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; S=\"$M/.local/share/odysseus/src\"; P=\"$(command -v podman 2>/dev/null || echo \"$M/.toolchains/podman/usr/local/bin/podman\")\"; [ -d \"$S/.git\" ] && [ \"$(git -C \"$S\" rev-parse HEAD 2>/dev/null)\" = \"${ODYSSEUS_REF:-ebead8083e84f58f7e1012f22c9a9266a13fa1ee}\" ] && \"$P\" ps --format '{{.Names}}' 2>/dev/null | grep -q odysseus && curl -fsS -o /dev/null --max-time 5 http://127.0.0.1:7000"]` |
| `setting` | `component[0].detect.args` | `manifest/prompt_hub.toml` | `scope=component source_kind=manifest` | `["-lc","export CARGO_HOME=\"$META_ROOT/.toolchains/cargo\"; export PATH=\"$CARGO_HOME/bin:$PATH\"; command -v prompthub >/dev/null || [ -x \"$META_ROOT/.toolchains/cargo/bin/prompthub\" ]"]` |
| `setting` | `component[0].detect.args` | `manifest/rusty-idd.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; B=\"${ENVCTL_BIN_DIR:-$M/usr/bin}\"; [ -x \"$M/.toolchains/rusty-idd/bin/rusty-idd\" ] && [ -x \"$B/rusty-idd\" ] && [ ! -L \"$B/rusty-idd\" ] && grep -Fqx \"exec \\\"$M/.toolchains/rusty-idd/bin/rusty-idd\\\" \\\"\\$@\\\"\" \"$B/rusty-idd\""]` |
| `setting` | `component[0].detect.args` | `manifest/sqld.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; test -x \"$M/.toolchains/sqld/bin/sqld\" && test -x \"$M/usr/bin/sqld\" && [ ! -L \"$M/usr/bin/sqld\" ] && grep -Fqx \"exec \\\"$M/.toolchains/sqld/bin/sqld\\\" \\\"\\$@\\\"\" \"$M/usr/bin/sqld\" && test -f \"$M/.config/systemd/user/sqld.service\""]` |
| `setting` | `component[0].fix.args` | `manifest/n8n-mcp.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export PATH=\"$M/usr/bin:$M/.toolchains/.bun/bin:$M/.toolchains/node/bin:$PATH\"; [ -x \"$M/usr/bin/bunx\" ]"]` |
| `setting` | `component[0].fix.script` | `manifest/agent-env.toml` | `scope=component source_kind=manifest` | `export PATH="$META_ROOT/.toolchains/cargo/bin:$META_ROOT/usr/bin:$PATH"
envctl agent sync --config agent-env.yaml --apply
` |
| `setting` | `component[0].fix.script` | `manifest/ai-clis.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
R="${ENVCTL_REAL_HOME:-$HOME}"
BASE="$M/.toolchains/claude"
STATE="$M/.local/share/claude"
LINK="$M/usr/bin/claude"
META_COMPAT="$M/.local/bin/claude"
REAL_COMPAT="$R/.local/bin/claude"

write_wrapper() {
  dst="$1"
  install -d -m 755 "$(dirname "$dst")"
  cat > "$dst" <<'WRAPPER'
#!/usr/bin/env bash
# envctl claude wrapper
set -euo pipefail
self="${BASH_SOURCE[0]}"
while [ -L "$self" ]; do
  dir="$(cd -P "$(dirname "$self")" && pwd)"
  target="$(readlink "$self")"
  case "$target" in
    /*) self="$target" ;;
    *) self="$dir/$target" ;;
  esac
done
bindir="$(cd -P "$(dirname "$self")" && pwd)"
META_ROOT="$(cd "$bindir/../.." && pwd)"
export META_ROOT
exec "$META_ROOT/.toolchains/claude/current/bin/claude" "$@"
WRAPPER
  chmod 755 "$dst"
}
latest_claude_source() {
  for dir in "$STATE/versions" "$R/.local/share/claude/versions" "$BASE"/*/bin; do
    [ -d "$dir" ] || continue
    find "$dir" -maxdepth 1 -mindepth 1 -type f -name claude -perm -111 -printf '%f	%p
' 2>/dev/null
    find "$dir" -maxdepth 1 -mindepth 1 -type f -perm -111 -printf '%f	%p
' 2>/dev/null
  done | sort -V | tail -1 | cut -f2-
}
install -d -m 700 "$STATE"
[ -L "$R/.claude" ] || ln -sfn "$STATE" "$R/.claude"
src="$(latest_claude_source)"
[ -n "$src" ] && [ -x "$src" ] || { echo "FATAL: no local Claude Code ELF found; run envctl install claude-code-cli" >&2; exit 1; }
ver="$(basename "$src")"
case "$src" in */bin/claude) ver="$(basename "$(dirname "$(dirname "$src")")")" ;; esac
case "$ver" in *[!0-9.]*|""|claude ) ver="local-$(sha256sum "$src" | awk '{print substr($1,1,12)}')" ;; esac
VDIR="$BASE/$ver"
install -d -m 755 "$VDIR/bin"
if [ "$(readlink -f "$src")" != "$(readlink -f "$VDIR/bin/claude" 2>/dev/null || true)" ]; then
  install -m 755 "$src" "$VDIR/bin/claude"
fi
ln -sfn "$ver" "$BASE/current"
write_wrapper "$LINK"
write_wrapper "$META_COMPAT"
write_wrapper "$REAL_COMPAT"
timeout --kill-after=2s 8s "$LINK" --version >/dev/null
CLEANUP="${ENVCTL_CLAUDE_CLEANUP:-$M/envctl/assets/scripts/envctl-claude-cleanup.sh}"
[ -x "$CLEANUP" ] || CLEANUP="$PWD/assets/scripts/envctl-claude-cleanup.sh"
"$CLEANUP" clean
"$CLEANUP" verify
` |
| `setting` | `component[0].fix.script` | `manifest/cognitum-seed-autounlock.toml` | `scope=component source_kind=manifest` | `set -euo pipefail

META_ROOT="${META_ROOT:?META_ROOT required by envctl hook}"
WORKER_PATH="$META_ROOT/usr/libexec/envctl/cognitum-seed-autounlock"
install -d -m755 "$(dirname "$WORKER_PATH")"
cat > "$WORKER_PATH" <<'WORKER'
#!/usr/bin/env bash
# cognitum-seed-autounlock — auto-unlock the secretd vault via the Cognitum Seed USB possession
# factor. USB-first (no passphrase ever scripted). Fail-closed: absent Seed/daemon => vault stays
# LOCKED, exit 0 (never errors the boot path).
set -euo pipefail

SECRETCTL=""
META_ROOT="${META_ROOT:?META_ROOT not set by envctl unit}"
for c in "$META_ROOT/usr/bin/secretctl" "$META_ROOT/usr/libexec/envctl/secrets/bin/secretctl" "$META_ROOT/.toolchains/secrets/bin/secretctl"; do
  if command -v "$c" >/dev/null 2>&1; then SECRETCTL="$c"; break; fi
done
[ -n "${SECRETCTL:-}" ] || { echo "autounlock: secretctl not found (no-op)"; exit 0; }

OWNER="drdave"
OWNER_UID="$(id -u "$OWNER" 2>/dev/null || echo "")"
[ -n "${OWNER_UID:-}" ] || { echo "autounlock: owner $OWNER not resolvable (no-op)"; exit 0; }
RT="/run/user/${OWNER_UID}"
SOCK="${RT}/env-ctl/secretd.sock"

for _i in $(seq 1 10); do
  [ -S "$SOCK" ] && break
  sleep 1
done
[ -S "$SOCK" ] || { echo "autounlock: owner secretd socket absent (no-op)"; exit 0; }

if setpriv --reuid "$OWNER_UID" --regid "$OWNER_UID" --init-groups \
     env XDG_RUNTIME_DIR="$RT" SECRETCTL_SOCK="$SOCK" \
     "$SECRETCTL" unlock >/dev/null 2>&1; then
  echo "autounlock: vault unlocked via USB possession factor"
else
  echo "autounlock: Seed not present/possessed (vault stays locked; no-op)"
fi
exit 0
WORKER
chmod 0755 "$WORKER_PATH"

META_ROOT_LITERAL="$META_ROOT"
cat > /etc/systemd/system/cognitum-seed-autounlock.service <<UNIT
[Unit]
Description=envctl secretd auto-unlock via Cognitum Seed USB possession factor
Documentation=https://github.com/FlexNetOS/envctl/blob/main/manifest/cognitum-seed-autounlock.toml
After=systemd-udevd.service

[Service]
Type=oneshot
Environment=META_ROOT=${META_ROOT_LITERAL}
ExecStart=${META_ROOT_LITERAL}/usr/libexec/envctl/cognitum-seed-autounlock

[Install]
WantedBy=multi-user.target
UNIT

cat > /etc/udev/rules.d/99-cognitum-seed-autounlock.rules <<'RULE'
ACTION=="add", SUBSYSTEM=="net", ENV{ID_NET_DRIVER}=="cdc_ncm", ENV{ID_MODEL}=="Cognitum_Seed", TAG+="systemd", ENV{SYSTEMD_WANTS}+="cognitum-seed-autounlock.service"
RULE

udevadm control --reload-rules || true
systemctl daemon-reload || true
systemctl enable cognitum-seed-autounlock.service || true
systemctl restart cognitum-seed-autounlock.service || true
` |
| `setting` | `component[0].fix.script` | `manifest/cognitum-seed-trust.toml` | `scope=component source_kind=manifest` | `set -euo pipefail

META_ROOT="${META_ROOT:?META_ROOT required by envctl hook}"
WORKER_PATH="$META_ROOT/usr/libexec/envctl/cognitum-seed-trust-refresh"
install -d -m755 "$(dirname "$WORKER_PATH")"
cat > "$WORKER_PATH" <<'WORKER'
#!/usr/bin/env bash
set -euo pipefail
if [ -n "${COGNITUM_TRUST_DIR:-}" ] && [ -d "$COGNITUM_TRUST_DIR" ]; then
  TRUST_DIR="$COGNITUM_TRUST_DIR"
else
  TRUST_DIR=""
  for d in /run/media/"${USER:-}"/COGNITUM/trust /run/media/*/COGNITUM/trust /media/*/COGNITUM/trust; do
    if [ -d "$d" ] && [ -f "$d/cognitum-ca.pem" ]; then TRUST_DIR="$d"; break; fi
  done
fi
if [ -z "${TRUST_DIR:-}" ]; then
  echo "cognitum-seed-trust: no Cognitum trust anchor present (no-op)"; exit 0
fi
SRC="$TRUST_DIR/cognitum-ca.pem"
META_ROOT="${META_ROOT:?META_ROOT not set by envctl unit}"
DST="${ENVCTL_SEED_CA:-$META_ROOT/etc/envctl/secrets/ca/cognitum-ca.crt}"
if cmp -s "$SRC" "$DST"; then
  echo "cognitum-seed-trust: pin already current (no-op)"; exit 0
fi
install -d -m755 "$(dirname "$DST")"
cp "$SRC" "$DST"
command -v update-ca-certificates >/dev/null 2>&1 && update-ca-certificates >/dev/null 2>&1 || true
echo "cognitum-seed-trust: re-pinned Device CA from $TRUST_DIR"
exit 0
WORKER
chmod 0755 "$WORKER_PATH"

META_ROOT_LITERAL="$META_ROOT"
cat > /etc/systemd/system/cognitum-seed-trust.service <<UNIT
[Unit]
Description=Cognitum Seed Device CA auto-refresh (envctl secretd USB-unlock factor)
Documentation=https://github.com/FlexNetOS/envctl/blob/main/manifest/cognitum-seed-trust.toml
After=systemd-udevd.service

[Service]
Type=oneshot
Environment=META_ROOT=${META_ROOT_LITERAL}
Environment=ENVCTL_SEED_CA=${META_ROOT_LITERAL}/etc/envctl/secrets/ca/cognitum-ca.crt
ExecStart=${META_ROOT_LITERAL}/usr/libexec/envctl/cognitum-seed-trust-refresh

[Install]
WantedBy=multi-user.target
UNIT

cat > /etc/udev/rules.d/99-cognitum-seed-trust.rules <<'RULE'
ACTION=="add", SUBSYSTEM=="net", ENV{ID_NET_DRIVER}=="cdc_ncm", ENV{ID_MODEL}=="Cognitum_Seed", TAG+="systemd", ENV{SYSTEMD_WANTS}+="cognitum-seed-trust.service"
RULE

udevadm control --reload-rules || true
systemctl daemon-reload || true
systemctl enable cognitum-seed-trust.service || true
systemctl restart cognitum-seed-trust.service || true
` |
| `setting` | `component[0].fix.script` | `manifest/components.d/handoff-hf.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
repo="${HANDOFF_REPO:-$M/handoff}"
[ -f "$repo/Cargo.toml" ] || { echo "handoff-hf: missing handoff repo at $repo" >&2; exit 1; }
export CARGO_HOME="$M/.toolchains/cargo"
export CARGO_TARGET_DIR="$M/var/cache/envctl/cargo-target/handoff"
export PATH="$M/usr/bin:$CARGO_HOME/bin:$PATH"
cargo build --release -p hf --manifest-path "$repo/Cargo.toml" --locked --target-dir "$CARGO_TARGET_DIR"
install -d -m 755 "$M/usr/bin" "$M/.local/bin"
envctl_frontdoor "$CARGO_TARGET_DIR/release/hf" "$M/usr/bin/hf"
envctl_frontdoor "$CARGO_TARGET_DIR/release/hf-mcp" "$M/usr/bin/hf-mcp"
ln -sfn "$M/usr/bin/hf" "$M/.local/bin/hf"


` |
| `setting` | `component[0].fix.script` | `manifest/components.d/meta-core-clis.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
DEST="$M/.toolchains/git"
install -d -m 755 "$META_ROOT/usr/bin"
PATH="$(printf '%s' "$PATH" | tr ':' '
' | grep -v -E "^$META_ROOT/usr/bin$" | paste -sd: -)"
src_bin="$(readlink -f "$(command -v git)")"
case "$src_bin" in "$DEST"/*) src_bin="$(find /nix/store -path '*/bin/git' -type f 2>/dev/null | grep 'git-2.54.0' | head -n1)";; esac
[ -n "$src_bin" ] || { echo "FATAL: no Git 2.54.0 source binary found"; exit 1; }
src_root="$(cd "$(dirname "$src_bin")/.." && pwd)"
rm -rf "$DEST"
mkdir -p "$DEST"
cp -a "$src_root/." "$DEST/"
envctl_frontdoor "$DEST/bin/git" "$META_ROOT/usr/bin/git"

` |
| `setting` | `component[0].fix.script` | `manifest/components.d/meta-env-plugin.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
DEST="$M/usr/libexec/envctl/meta-env"
LINK="$M/usr/bin/meta-env"
export CARGO_HOME="$M/.toolchains/cargo"
export PATH="$M/usr/bin:$CARGO_HOME/bin:$PATH"
cargo build --release -p envctl --bin meta-env --manifest-path "$M/envctl/Cargo.toml"
install -d -m 755 "$DEST/bin" "$M/usr/bin"
install -m 755 "$M/envctl/target/release/meta-env" "$DEST/bin/meta-env"
envctl_frontdoor "$DEST/bin/meta-env" "$LINK"

` |
| `setting` | `component[0].fix.script` | `manifest/env-ctl.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
# Meta-owned toolchain + prefix (Epic H TASK-0069). See install.
M="${META_ROOT:?META_ROOT required}"
export CARGO_HOME="$M/.toolchains/cargo"
META_BIN="$M/usr/bin"
export PATH="$M/.toolchains/cargo/bin:$META_BIN:$META_ROOT/usr/bin:$PATH"
DEST="$M/usr/libexec/envctl/secrets/bin"
# Resolve the workspace root (override -> meta layout; sentinel-validated). See install.
repo="${ENV_CTL_REPO:-$META_ROOT/envctl}"
test -d "${repo:-/nonexistent}/crates/secretd" || { echo "FATAL: envctl workspace not found (set ENV_CTL_REPO)"; exit 1; }
# Rebuild WITH seed-factor (see install note): a plain rebuild would drop the USB unlock factor.
cargo build --release --manifest-path "$repo/Cargo.toml" -p envctl-secretd --features seed-factor
cargo build --release --manifest-path "$repo/Cargo.toml" -p envctl-secretctl
install -Dm755 "$repo/target/release/secretd"   "$DEST/secretd"
install -Dm755 "$repo/target/release/secretctl" "$DEST/secretctl"
install -d -m755 "$META_BIN"
envctl_frontdoor "$DEST/secretd"   "$META_BIN/secretd"
envctl_frontdoor "$DEST/secretctl" "$META_BIN/secretctl"

cleanup_legacy_cargo_bin() {
  b="$1"
  legacy="$META_ROOT/.toolchains/cargo/bin/$b"
  owned="$DEST/$b"
  [ -e "$legacy" ] || return 0
  # If an old symlink already points at the new meta-local binary, remove the duplicate link.
  if [ -L "$legacy" ] && [ "$(readlink -f "$legacy" 2>/dev/null)" = "$(readlink -f "$owned" 2>/dev/null)" ]; then
    rm -f "$legacy"
    echo "removed legacy cargo-bin symlink: $legacy"
    return 0
  fi
  # If the old regular file is byte-identical to the just-installed meta binary, archive it.
  # Different/foreign binaries are left in place and surfaced by verify; strict upgrade-only means
  # never deleting an unproven executable.
  if [ -f "$legacy" ] && [ -f "$owned" ] && cmp -s "$legacy" "$owned"; then
    arch="$META_ROOT/var/lib/envctl/legacy-archives/$(date -u +%Y%m%dT%H%M%SZ)/.cargo/bin"
    install -d -m700 "$arch"
    mv "$legacy" "$arch/$b"
    echo "archived legacy cargo-bin copy: $legacy -> $arch/$b"
  elif [ -e "$legacy" ]; then
    echo "WARNING: legacy cargo-bin $legacy differs from $owned; left in place (strict upgrade-only)"
  fi
}
cleanup_legacy_cargo_bin secretd
cleanup_legacy_cargo_bin secretctl

# Re-assert the durable store config idempotently (write only if absent; operator edits preserved).
install -d -m700 "$META_ROOT/.config/env-ctl"
cfg="$META_ROOT/.config/env-ctl/secretd.toml"
if [ ! -f "$cfg" ]; then
  cat > "$cfg" <<'TOML'
[store]
backend = "libsql"
url = "http://127.0.0.1:8080"
TOML
  chmod 0600 "$cfg"
fi

systemctl --user daemon-reload || true
systemctl --user try-restart env-ctl.service || true


# REMOVE: disable + drop the unit and bins. The vault is data_paths and is touched ONLY on --purge.
` |
| `setting` | `component[0].fix.script` | `manifest/grit.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
export CARGO_HOME="$META_ROOT/.toolchains/cargo"
export PATH="$CARGO_HOME/bin:$PATH"
repo="${GRIT_REPO:-$META_ROOT/grit}"
test -d "$repo" || { echo "FATAL: grit checkout not found at $repo (set GRIT_REPO)"; exit 1; }
cargo install --path "$repo" --force --locked
` |
| `setting` | `component[0].fix.script` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
sudo systemctl restart nix-daemon 2>/dev/null || true
install -d -m 755 "$M/.toolchains/nix/bin" "$META_ROOT/usr/bin"
src="$(readlink -f /nix/var/nix/profiles/default/bin/nix)"
install -m 755 "$src" "$M/.toolchains/nix/bin/nix"
envctl_frontdoor "$M/.toolchains/nix/bin/nix" "$META_ROOT/usr/bin/nix"
"$M/.toolchains/nix/bin/nix" --version

` |
| `setting` | `component[0].fix.script` | `manifest/odysseus.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
M="${META_ROOT:?META_ROOT required}"; rt="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
export PATH="$M/.local/bin:$M/.toolchains/podman/usr/local/bin:$M/usr/bin:$PATH"
ROOT="$M/.local/share/odysseus"; SRC="$ROOT/src"; DATA="$ROOT/data"; LOGS="$ROOT/logs"
command -v podman >/dev/null 2>&1 || { echo "odysseus needs the meta-local podman engine; run: envctl install podman"; exit 1; }
systemctl --user enable --now podman.socket 2>/dev/null || true
[ -d "$SRC/.git" ] || { echo "odysseus not installed; run install"; exit 1; }

odysseus_ensure_state_dir() {
  d="$1"
  if [ -d "$d" ]; then
    return 0
  fi

  if mkdir -p "$d" 2>/dev/null; then
    chmod 700 "$d" 2>/dev/null || true
    return 0
  fi

  podman unshare mkdir -p "$d"
  podman unshare chmod 700 "$d" 2>/dev/null || true
}

odysseus_ensure_state_dir "$DATA"
odysseus_ensure_state_dir "$LOGS"
odysseus_ensure_state_dir "$DATA/ssh"
odysseus_ensure_state_dir "$DATA/huggingface"
odysseus_ensure_state_dir "$DATA/local"

cd "$SRC"
podman compose -f docker-compose.yml -f compose.meta.yml up -d
` |
| `setting` | `component[0].fix.script` | `manifest/prompt_hub.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
export CARGO_HOME="$META_ROOT/.toolchains/cargo"
export PATH="$CARGO_HOME/bin:$PATH"
repo="${PROMPT_HUB_REPO:-$META_ROOT/prompt_hub}"
test -d "$repo" || { echo "FATAL: prompt_hub checkout not found at $repo (set PROMPT_HUB_REPO)"; exit 1; }
cargo install --path "$repo/prompthub" --force --locked
cargo install --path "$repo/prompthub-server" --force --locked
` |
| `setting` | `component[0].fix.script` | `manifest/rusty-idd.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
export CARGO_HOME="$M/.toolchains/cargo"
export RUSTUP_HOME="$M/.toolchains/rustup"
BIN="${ENVCTL_BIN_DIR:-$M/usr/bin}"
export PATH="$CARGO_HOME/bin:$BIN:$PATH"
repo="${RUSTY_IDD_REPO:-$M/rusty-idd}"
test -d "$repo" || { echo "FATAL: rusty-idd checkout not found at $repo (set RUSTY_IDD_REPO)"; exit 1; }
ROOT="$M/.toolchains/rusty-idd"
cargo install --path "$repo/crates/cli" --force --locked --root "$ROOT"
mkdir -p "$BIN"
envctl_frontdoor "$ROOT/bin/rusty-idd" "$BIN/rusty-idd"


# REMOVE: uninstall the binary by PACKAGE name (rusty-idd-cli), not bin name. \`|| true\` keeps reset
# idempotent if rusty-idd was never installed.
` |
| `setting` | `component[0].fix.script` | `manifest/sqld.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
ver="${SQLD_VERSION:-v0.24.32}"
arch="$(uname -m)"
case "$arch" in
  x86_64)  target="x86_64-unknown-linux-gnu" ;;
  aarch64) target="aarch64-unknown-linux-gnu" ;;
  *) echo "FATAL: unsupported arch $arch"; exit 1 ;;
esac
envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
M="${META_ROOT:?META_ROOT required}"
DEST="$M/.toolchains/sqld/bin"
install -d -m755 "$DEST" "$M/usr/bin"
install -d -m700 "$M/.local/share/sqld"
if [ ! -x "$DEST/sqld" ]; then
  tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
  url="https://github.com/tursodatabase/libsql/releases/download/libsql-server-${ver}/libsql-server-${target}.tar.xz"
  curl -fsSL "$url" -o "$tmp/sqld.tar.xz" || { echo "FATAL: download failed $url"; exit 1; }
  tar -xJf "$tmp/sqld.tar.xz" -C "$tmp"
  bin="$(find "$tmp" -type f -name sqld | head -n1)"
  install -Dm755 "$bin" "$DEST/sqld"
  envctl_frontdoor "$DEST/sqld" "$M/usr/bin/sqld"
fi
systemctl --user daemon-reload || true
systemctl --user enable --now sqld.service || true
` |
| `setting` | `component[0].id` | `manifest/cognitum-seed-trust.toml` | `scope=component source_kind=manifest` | `cognitum-seed-trust` |
| `setting` | `component[0].id` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `nix` |
| `setting` | `component[0].id` | `manifest/rusty-idd.toml` | `scope=component source_kind=manifest` | `rusty-idd` |
| `setting` | `component[0].install.args` | `manifest/n8n-mcp.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export PATH=\"$M/usr/bin:$M/.toolchains/.bun/bin:$M/.toolchains/node/bin:$PATH\"; [ -x \"$M/usr/bin/bunx\" ]"]` |
| `setting` | `component[0].install.script` | `manifest/agent-env.toml` | `scope=component source_kind=manifest` | `export PATH="$META_ROOT/.toolchains/cargo/bin:$META_ROOT/usr/bin:$PATH"
envctl agent sync --config agent-env.yaml --apply
` |
| `setting` | `component[0].install.script` | `manifest/ai-clis.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
R="${ENVCTL_REAL_HOME:-$HOME}"
BASE="$M/.toolchains/claude"
STATE="$M/.local/share/claude"
LINK="$M/usr/bin/claude"
META_COMPAT="$M/.local/bin/claude"
REAL_COMPAT="$R/.local/bin/claude"

write_wrapper() {
  dst="$1"
  install -d -m 755 "$(dirname "$dst")"
  cat > "$dst" <<'WRAPPER'
#!/usr/bin/env bash
# envctl claude wrapper
set -euo pipefail
self="${BASH_SOURCE[0]}"
while [ -L "$self" ]; do
  dir="$(cd -P "$(dirname "$self")" && pwd)"
  target="$(readlink "$self")"
  case "$target" in
    /*) self="$target" ;;
    *) self="$dir/$target" ;;
  esac
done
bindir="$(cd -P "$(dirname "$self")" && pwd)"
META_ROOT="$(cd "$bindir/../.." && pwd)"
export META_ROOT
exec "$META_ROOT/.toolchains/claude/current/bin/claude" "$@"
WRAPPER
  chmod 755 "$dst"
}

latest_claude_source() {
  for dir in "$STATE/versions" "$R/.local/share/claude/versions"; do
    [ -d "$dir" ] || continue
    find "$dir" -maxdepth 1 -mindepth 1 -type f -perm -111 -printf '%f	%p
' 2>/dev/null | sort -V | tail -1 | cut -f2-
  done | tail -1
}

install -d -m 700 "$STATE"
if [ -e "$R/.claude" ] && [ "$(readlink -f "$R/.claude" 2>/dev/null)" != "$(readlink -f "$STATE" 2>/dev/null)" ]; then
  ARCH="$M/var/lib/envctl/legacy-archives/claude-home-$(date -u +%Y%m%d-%H%M%S)"
  install -d -m 700 "$ARCH"
  mv "$R/.claude" "$ARCH/.claude"
  echo "archived previous real-home .claude: $ARCH/.claude"
fi
[ -L "$R/.claude" ] || ln -sfn "$STATE" "$R/.claude"

# Run the official updater with the old self-recursing wrappers hidden from PATH. It should leave
# a versioned ELF under $STATE/versions; if the network/updater is unavailable but an existing
# versioned ELF is present, we still repair the envctl toolchain from that local version.
PATH_CLEAN="$M/.toolchains/node/bin:$M/.toolchains/.bun/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
PATH="$PATH_CLEAN" curl -fsSL https://claude.ai/install.sh | PATH="$PATH_CLEAN" bash || true
src="$(latest_claude_source)"
[ -n "$src" ] && [ -x "$src" ] || { echo "FATAL: no Claude Code versioned ELF found under $STATE/versions" >&2; exit 1; }
ver="$(basename "$src")"
case "$ver" in
  *[!0-9.]*|"" ) ver="local-$(sha256sum "$src" | awk '{print substr($1,1,12)}')" ;;
esac
VDIR="$BASE/$ver"
install -d -m 755 "$VDIR/bin"
install -m 755 "$src" "$VDIR/bin/claude"
ln -sfn "$ver" "$BASE/current"

if timeout --kill-after=2s 8s "$VDIR/bin/claude" --version >/dev/null; then
  for old in "$LINK" "$META_COMPAT" "$REAL_COMPAT"; do
    if [ -e "$old" ] && { [ -L "$old" ] || ! grep -q "envctl claude wrapper" "$old" 2>/dev/null; }; then
      ARCH="$M/var/lib/envctl/legacy-archives/claude-bin-$(date -u +%Y%m%d-%H%M%S)/$(dirname "${old#/}")"
      install -d -m 755 "$ARCH"
      mv "$old" "$ARCH/claude"
      echo "archived previous Claude front door: $ARCH/claude"
    fi
  done
  write_wrapper "$LINK"
  write_wrapper "$META_COMPAT"
  write_wrapper "$REAL_COMPAT"
  timeout --kill-after=2s 8s "$LINK" --version >/dev/null
  CLEANUP="${ENVCTL_CLAUDE_CLEANUP:-$M/envctl/assets/scripts/envctl-claude-cleanup.sh}"
  [ -x "$CLEANUP" ] || CLEANUP="$PWD/assets/scripts/envctl-claude-cleanup.sh"
  "$CLEANUP" clean
  "$CLEANUP" verify
else
  echo "FATAL: envctl Claude binary failed verification; legacy front doors left in place" >&2
  exit 1
fi
` |
| `setting` | `component[0].install.script` | `manifest/cognitum-seed-autounlock.toml` | `scope=component source_kind=manifest` | `set -euo pipefail

META_ROOT="${META_ROOT:?META_ROOT required by envctl hook}"
WORKER_PATH="$META_ROOT/usr/libexec/envctl/cognitum-seed-autounlock"
install -d -m755 "$(dirname "$WORKER_PATH")"
cat > "$WORKER_PATH" <<'WORKER'
#!/usr/bin/env bash
# cognitum-seed-autounlock — auto-unlock the secretd vault via the Cognitum Seed USB possession
# factor. USB-first (no passphrase ever scripted). Fail-closed: absent Seed/daemon => vault stays
# LOCKED, exit 0 (never errors the boot path).
set -euo pipefail

# Resolve secretctl: meta-prefix only. The unit injects META_ROOT because this is a system service
# and must not rediscover binaries from the operator's real HOME.
SECRETCTL=""
META_ROOT="${META_ROOT:?META_ROOT not set by envctl unit}"
for c in "$META_ROOT/usr/bin/secretctl" "$META_ROOT/usr/libexec/envctl/secrets/bin/secretctl" "$META_ROOT/.toolchains/secrets/bin/secretctl"; do
  if command -v "$c" >/dev/null 2>&1; then SECRETCTL="$c"; break; fi
done
# No meta-hosted secretctl => nothing to do.
[ -n "${SECRETCTL:-}" ] || { echo "autounlock: secretctl not found (no-op)"; exit 0; }

# Owner identity — single-user box convention.
OWNER="drdave"
OWNER_UID="$(id -u "$OWNER" 2>/dev/null || echo "")"
[ -n "${OWNER_UID:-}" ] || { echo "autounlock: owner $OWNER not resolvable (no-op)"; exit 0; }
RT="/run/user/${OWNER_UID}"
SOCK="${RT}/env-ctl/secretd.sock"

# Bounded wait for the owner's USER secretd socket (pre-login boot / no linger => clean no-op).
for _i in $(seq 1 10); do
  [ -S "$SOCK" ] && break
  sleep 1
done
# Socket never appeared => the owner daemon is not up; nothing to unlock.
[ -S "$SOCK" ] || { echo "autounlock: owner secretd socket absent (no-op)"; exit 0; }

# Drop to the owner uid/gid + owner XDG_RUNTIME_DIR so the secretctl connection's SO_PEERCRED uid
# == owner_uid (secretd peercred gate) AND the socket path resolves to the owner's daemon. The
# unlock is USB-first possession — NO passphrase is passed.
if setpriv --reuid "$OWNER_UID" --regid "$OWNER_UID" --init-groups \
     env XDG_RUNTIME_DIR="$RT" SECRETCTL_SOCK="$SOCK" \
     "$SECRETCTL" unlock >/dev/null 2>&1; then
  echo "autounlock: vault unlocked via USB possession factor"
else
  echo "autounlock: Seed not present/possessed (vault stays locked; no-op)"
fi
# Always exit 0 — fail-closed; never error the boot path.
exit 0
WORKER
chmod 0755 "$WORKER_PATH"

META_ROOT_LITERAL="$META_ROOT"
cat > /etc/systemd/system/cognitum-seed-autounlock.service <<UNIT
[Unit]
Description=envctl secretd auto-unlock via Cognitum Seed USB possession factor
Documentation=https://github.com/FlexNetOS/envctl/blob/main/manifest/cognitum-seed-autounlock.toml
# UDS-only (no network ordering). secretd is a USER unit, invisible to this system oneshot's dep
# graph, so readiness is handled by the worker's bounded socket-wait, not unit ordering.
After=systemd-udevd.service

[Service]
Type=oneshot
Environment=META_ROOT=${META_ROOT_LITERAL}
ExecStart=${META_ROOT_LITERAL}/usr/libexec/envctl/cognitum-seed-autounlock

[Install]
WantedBy=multi-user.target
UNIT

cat > /etc/udev/rules.d/99-cognitum-seed-autounlock.rules <<'RULE'
ACTION=="add", SUBSYSTEM=="net", ENV{ID_NET_DRIVER}=="cdc_ncm", ENV{ID_MODEL}=="Cognitum_Seed", TAG+="systemd", ENV{SYSTEMD_WANTS}+="cognitum-seed-autounlock.service"
RULE

udevadm control --reload-rules || true
systemctl daemon-reload || true
systemctl enable cognitum-seed-autounlock.service || true
systemctl start cognitum-seed-autounlock.service || true
` |
| `setting` | `component[0].install.script` | `manifest/cognitum-seed-trust.toml` | `scope=component source_kind=manifest` | `set -euo pipefail

META_ROOT="${META_ROOT:?META_ROOT required by envctl hook}"
WORKER_PATH="$META_ROOT/usr/libexec/envctl/cognitum-seed-trust-refresh"
install -d -m755 "$(dirname "$WORKER_PATH")"
cat > "$WORKER_PATH" <<'WORKER'
#!/usr/bin/env bash
# cognitum-seed-trust-refresh — re-pin the secretd Device CA from the COGNITUM USB trust anchor.
# Idempotent (re-pins only on diff); exits 0 when the Seed/anchor is absent.
set -euo pipefail

# Locate the Cognitum trust anchor (honor COGNITUM_TRUST_DIR override first).
if [ -n "${COGNITUM_TRUST_DIR:-}" ] && [ -d "$COGNITUM_TRUST_DIR" ]; then
  TRUST_DIR="$COGNITUM_TRUST_DIR"
else
  TRUST_DIR=""
  for d in /run/media/"${USER:-}"/COGNITUM/trust /run/media/*/COGNITUM/trust /media/*/COGNITUM/trust; do
    if [ -d "$d" ] && [ -f "$d/cognitum-ca.pem" ]; then TRUST_DIR="$d"; break; fi
  done
fi

if [ -z "${TRUST_DIR:-}" ]; then
  echo "cognitum-seed-trust: no Cognitum trust anchor present (no-op)"; exit 0
fi

SRC="$TRUST_DIR/cognitum-ca.pem"
META_ROOT="${META_ROOT:?META_ROOT not set by envctl unit}"
DST="${ENVCTL_SEED_CA:-$META_ROOT/etc/envctl/secrets/ca/cognitum-ca.crt}"

if cmp -s "$SRC" "$DST"; then
  echo "cognitum-seed-trust: pin already current (no-op)"; exit 0
fi

install -d -m755 "$(dirname "$DST")"
cp "$SRC" "$DST"
command -v update-ca-certificates >/dev/null 2>&1 && update-ca-certificates >/dev/null 2>&1 || true
echo "cognitum-seed-trust: re-pinned Device CA from $TRUST_DIR"
exit 0
WORKER
chmod 0755 "$WORKER_PATH"

META_ROOT_LITERAL="$META_ROOT"
cat > /etc/systemd/system/cognitum-seed-trust.service <<UNIT
[Unit]
Description=Cognitum Seed Device CA auto-refresh (envctl secretd USB-unlock factor)
Documentation=https://github.com/FlexNetOS/envctl/blob/main/manifest/cognitum-seed-trust.toml
After=systemd-udevd.service

[Service]
Type=oneshot
Environment=META_ROOT=${META_ROOT_LITERAL}
Environment=ENVCTL_SEED_CA=${META_ROOT_LITERAL}/etc/envctl/secrets/ca/cognitum-ca.crt
ExecStart=${META_ROOT_LITERAL}/usr/libexec/envctl/cognitum-seed-trust-refresh

[Install]
WantedBy=multi-user.target
UNIT

cat > /etc/udev/rules.d/99-cognitum-seed-trust.rules <<'RULE'
# When the Cognitum Seed USB-Ethernet (cdc_ncm) NIC appears, start the oneshot that re-pins the
# Device CA from the USB trust anchor (same trigger as cognitum-seed-linklocal).
ACTION=="add", SUBSYSTEM=="net", ENV{ID_NET_DRIVER}=="cdc_ncm", ENV{ID_MODEL}=="Cognitum_Seed", TAG+="systemd", ENV{SYSTEMD_WANTS}+="cognitum-seed-trust.service"
RULE

udevadm control --reload-rules || true
systemctl daemon-reload || true
systemctl enable cognitum-seed-trust.service || true
systemctl start cognitum-seed-trust.service || true
` |
| `setting` | `component[0].install.script` | `manifest/components.d/codex-global-baseline.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
R="${ENVCTL_REAL_HOME:-$HOME}"
CODEX_HOME_DIR="$M/.local/share/codex"
CODEX_STATE_DIR="$M/.local/state/codex"
install -d -m 700 "$CODEX_HOME_DIR" "$CODEX_STATE_DIR" "$CODEX_HOME_DIR/agents"
install -d -m 755 "$R/.local/bin"
if [ -e "$R/.codex" ] && [ "$(readlink -f "$R/.codex" 2>/dev/null)" != "$(readlink -f "$CODEX_HOME_DIR" 2>/dev/null)" ]; then
  ARCH="$M/var/lib/envctl/legacy-archives/codex-home-$(date -u +%Y%m%d-%H%M%S)"
  install -d -m 700 "$ARCH"
  mv "$R/.codex" "$ARCH/.codex"
  echo "archived previous real-home .codex: $ARCH/.codex"
fi
[ -L "$R/.codex" ] || ln -s "$CODEX_HOME_DIR" "$R/.codex"
if [ -e "$R/.local/bin/codex" ] && [ "$(readlink -f "$R/.local/bin/codex" 2>/dev/null)" != "$(readlink -f "$M/usr/bin/codex" 2>/dev/null)" ]; then
  ARCH="$M/var/lib/envctl/legacy-archives/codex-bin-$(date -u +%Y%m%d-%H%M%S)/.local/bin"
  install -d -m 755 "$ARCH"
  mv "$R/.local/bin/codex" "$ARCH/codex"
  echo "archived previous real-home codex: $ARCH/codex"
fi
ln -sfn "$M/usr/bin/codex" "$R/.local/bin/codex"
python3 - <<'PY'
import json, subprocess
from pathlib import Path
M = Path(__import__('os').environ['META_ROOT'])
home = M / '.local/share/codex'
config = home / 'config.toml'
hooks = home / 'hooks.json'
projects = [M]
try:
    out = subprocess.check_output(['rtk','meta','project','list','--json'], cwd=M, text=True, stderr=subprocess.DEVNULL)
    data = json.loads(out)
    projects = [Path(data['root'])] + [Path(data['root']) / p['path'] for p in data.get('projects', [])]
except Exception:
    pass
features = [
    'apps', 'auto_compaction', 'browser_use', 'browser_use_external', 'computer_use',
    'enable_request_compression', 'fast_mode', 'goals', 'guardian_approval', 'hooks',
    'image_generation', 'in_app_browser', 'mentions_v2', 'multi_agent', 'personality',
    'plugin_sharing', 'plugins', 'prevent_idle_sleep', 'remote_compaction_v2',
    'secret_auth_storage', 'shell_snapshot', 'shell_tool', 'skill_mcp_dependency_install',
    'tool_call_mcp_elicitation', 'tool_suggest', 'unified_exec', 'workspace_dependencies',
    'memories', 'network_proxy'
]
lines = [
    'model = "gpt-5.5"',
    'model_reasoning_effort = "medium"',
    'approvals_reviewer = "user"',
    'personality = "friendly"',
    'approval_policy = "on-request"',
    'model_context_window = 4000000',
    'model_auto_compact_token_limit = 3000000',
    'model_auto_compact_token_limit_scope = "total"',
    'web_search = "live"',
    'background_terminal_max_timeout = 300000',
    'tool_output_token_limit = 12000',
    f'model_catalog_json = "{home}/model-catalog.json"',
    'notify = ["weave", "hook", "wake"]',
    'service_tier = "fast"',
    'default_permissions = "meta-workspace"',
    '',
    '[permissions.meta-read-only]',
    'description = "Read-only meta workspace profile with limited network for documentation and source lookup."',
    'extends = ":read-only"',
    '',
    '[permissions.meta-read-only.workspace_roots]',
    f'"{M}" = true',
    '',
    '[permissions.meta-read-only.network]',
    'enabled = true',
    '',
    '[permissions.meta-read-only.network.domains]',
    '"developers.openai.com" = "allow"',
    '"api.openai.com" = "allow"',
    '"github.com" = "allow"',
    '"*.github.com" = "allow"',
    '"objects.githubusercontent.com" = "allow"',
    '',
    '[permissions.meta-workspace]',
    'description = "Default FlexNetOS meta workspace profile: write inside workspace roots, deny env/secrets, allow vetted developer network endpoints."',
    'extends = ":workspace"',
    '',
    '[permissions.meta-workspace.workspace_roots]',
    f'"{M}" = true',
    '',
    '[permissions.meta-workspace.filesystem.":workspace_roots"]',
    '"**/.env" = "deny"',
    '"**/.env.*" = "deny"',
    '"**/*secret*" = "deny"',
    '"**/*token*" = "deny"',
    '".codex" = "read"',
    '".codex/tmp" = "write"',
    '".local/share/codex" = "read"',
    '".local/state/codex" = "write"',
    '',
    '[permissions.meta-workspace.network]',
    'enabled = true',
    '',
    '[permissions.meta-workspace.network.domains]',
    '"api.openai.com" = "allow"',
    '"developers.openai.com" = "allow"',
    '"github.com" = "allow"',
    '"*.github.com" = "allow"',
    '"objects.githubusercontent.com" = "allow"',
    '"registry.npmjs.org" = "allow"',
    '"crates.io" = "allow"',
    '"static.crates.io" = "allow"',
    '',
    '[permissions.meta-full-trusted]',
    'description = "Explicit high-trust escape hatch for owner-approved meta maintenance only."',
    'extends = ":danger-full-access"',
    '',
    '[features]',
]
lines += [f'{f} = true' for f in features]
lines += [
    '',
    '[mcp_servers.meta]', 'command = "meta-mcp"', 'args = []',
    '',
    '[mcp_servers.gitkb]', 'command = "git"', 'args = ["kb", "mcp"]',
    '',
    '[mcp_servers.icm]', 'command = "icm"', 'args = ["serve"]',
    '',
    '[mcp_servers.vox]', 'command = "vox"', 'args = ["serve"]',
    '',
    '[mcp_servers.context7]', 'command = "bunx"', 'args = ["-y", "@upstash/context7-mcp"]',
    '',
    '[mcp_servers.weave]', 'command = "weave"', 'args = ["mcp"]',
    '',
    '[mcp_servers.openaiDeveloperDocs]', 'url = "https://developers.openai.com/mcp"',
    '',
    '[agents]', 'max_depth = 1', 'job_max_runtime_seconds = 1800',
    '',
    '[tui]',
    'status_line = ["model-with-reasoning", "context-used", "used-tokens", "five-hour-usage", "weekly-usage", "fast-mode"]',
    'status_line_use_colors = true',
    '',
    '[marketplaces.flexnetos-codex]',
    'source_type = "local"',
    f'source = "{M}/codex-plugins"',
    '',
    '[marketplaces.gitkb]',
    'source_type = "local"',
    f'source = "{M}/claude-plugins"',
    '',
    '[marketplaces.harness-marketplace]',
    'source_type = "local"',
    f'source = "{M}/harness_hub/harness"',
    '',
    '[plugins."meta@flexnetos-codex"]', 'enabled = true',
    '',
    '[plugins."gitkb@gitkb"]', 'enabled = true',
    '',
    '[plugins."meta@gitkb"]', 'enabled = true',
    '',
    '[plugins."harness@harness-marketplace"]', 'enabled = true',
    '',
    '[skills]', 'include_instructions = true',
    '',
    '[memories]', 'use_memories = true', 'generate_memories = true',
    '',
    '[hooks.state]',
]
for p in projects:
    lines += ['', f'[projects."{p}"]', 'trust_level = "trusted"']
config.write_text('\n'.join(lines) + '\n')


def model_entry(slug, display_name, description, context_window=4000000, priority=0, reasoning=('low','medium','high','xhigh'), fast=False, visibility='list'):
    return {
        'additional_speed_tiers': ['fast'] if fast else [],
        'apply_patch_tool_type': 'freeform',
        'auto_compact_token_limit': None,
        'availability_nux': {'message': f'{display_name} is configured by the FlexNetOS/meta Codex baseline.'},
        'base_instructions': '',
        'context_window': context_window,
        'default_reasoning_level': 'medium',
        'default_reasoning_summary': 'none',
        'default_service_tier': None,
        'default_verbosity': 'low',
        'description': description,
        'display_name': display_name,
        'effective_context_window_percent': 95,
        'experimental_supported_tools': [],
        'input_modalities': ['text', 'image'],
        'max_context_window': context_window,
        'priority': priority,
        'service_tiers': [{'description': '1.5x speed, increased usage', 'id': 'priority', 'name': 'Fast'}] if fast else [],
        'shell_type': 'shell_command',
        'slug': slug,
        'support_verbosity': True,
        'supported_in_api': True,
        'supported_reasoning_levels': [
            {'effort': e, 'description': {
                'low': 'Fast responses with lighter reasoning',
                'medium': 'Balances speed and reasoning depth for everyday tasks',
                'high': 'Greater reasoning depth for complex problems',
                'xhigh': 'Extra high reasoning depth for complex problems',
                'max': 'Maximum reasoning depth for GPT-5.6 Sol preview workloads',
            }.get(e, e)} for e in reasoning
        ],
        'supports_image_detail_original': True,
        'supports_parallel_tool_calls': True,
        'supports_reasoning_summaries': True,
        'supports_search_tool': True,
        'truncation_policy': {'limit': 10000, 'mode': 'tokens'},
        'upgrade': None,
        'visibility': visibility,
        'web_search_tool_type': 'text_and_image',
    }
(home / 'model-catalog.json').write_text(json.dumps({'models': [
    model_entry('gpt-5.5', 'GPT-5.5', 'Current recommended OpenAI Codex/API model for complex coding, research, and tool-heavy workflows.', priority=0, fast=True),
    model_entry('gpt-5.4-mini', 'GPT-5.4 mini', 'Fast OpenAI model for lighter coding tasks and subagents.', context_window=400000, priority=1, reasoning=('low','medium','high','xhigh')),
    model_entry('gpt-5.4-nano', 'GPT-5.4 nano', 'Smallest GPT-5.4 frontier variant for the lowest-latency, lowest-cost text/image tasks when available.', context_window=400000, priority=2, reasoning=('low','medium','high')),
    model_entry('gpt-5.4', 'GPT-5.4', 'OpenAI GPT-5 family model retained for compatibility and explicit routing.', priority=3),
    model_entry('gpt-5.3-codex-spark', 'GPT-5.3 Codex Spark', 'Research-preview Codex model optimized for near-instant coding iteration when the account has access.', context_window=1000000, priority=4, reasoning=('low','medium','high')),
    model_entry('gpt-5.6-sol', 'GPT-5.6 Sol', 'Official OpenAI GPT-5.6 Sol limited-preview routing name; available through API and Codex only for accounts/organizations with preview access.', priority=10, reasoning=('low','medium','high','xhigh','max')),
    model_entry('gpt-5.6-terra', 'GPT-5.6 Terra', 'Official OpenAI GPT-5.6 Terra limited-preview routing name; available through API and Codex only for accounts/organizations with preview access.', priority=11),
    model_entry('gpt-5.6-luna', 'GPT-5.6 Luna', 'Official OpenAI GPT-5.6 Luna limited-preview routing name; available through API and Codex only for accounts/organizations with preview access.', priority=12),
]}, indent=2) + '\n')
agent_dir = home / 'agents'
agent_dir.mkdir(parents=True, exist_ok=True)
agent_src = M / '.codex/agents'
if agent_src.exists():
    for src in agent_src.glob('*.toml'):
        (agent_dir / src.name).write_text(src.read_text())
else:
    raise SystemExit(f'missing project Codex agent source: {agent_src}')
def meta_env_command(*parts):
    quoted = ' '.join(__import__('shlex').quote(str(part)) for part in parts)
    return 'rtk bash -lc ' + __import__('shlex').quote(f'exec "{M}/.codex/hooks/with-meta-env.sh" {quoted}')
hooks.write_text(json.dumps({
  'hooks': {
    'SessionStart': [{'hooks': [{'type': 'command', 'command': meta_env_command('icm', 'hook', 'start')}]}],
    'PreToolUse': [{'matcher': 'Bash', 'hooks': [{'type': 'command', 'command': meta_env_command('icm', 'hook', 'pre')}]}],
    'PostToolUse': [{'hooks': [{'type': 'command', 'command': meta_env_command('icm', 'hook', 'post')}]}],
    'UserPromptSubmit': [{'hooks': [{'type': 'command', 'command': meta_env_command('icm', 'hook', 'prompt')}]}],
    'PreCompact': [{'hooks': [{'type': 'command', 'command': meta_env_command('icm', 'hook', 'compact')}]}],
    'Stop': [{'hooks': [{'type': 'command', 'command': meta_env_command('weave', 'hook', 'wake')}]}],
    'SubagentStop': [{'hooks': [{'type': 'command', 'command': meta_env_command('weave', 'hook', 'wake')}]}]
  }
}, indent=2) + '\n')
PY
export CODEX_HOME="$CODEX_HOME_DIR" CODEX_SQLITE_HOME="$CODEX_STATE_DIR"
"$M/usr/bin/codex" plugin marketplace add "$M/codex-plugins" --json >/dev/null || true
"$M/usr/bin/codex" plugin add meta@flexnetos-codex >/dev/null || true
"$M/usr/bin/codex" plugin add gitkb@gitkb >/dev/null || true
"$M/usr/bin/codex" plugin add meta@gitkb >/dev/null || true
"$M/usr/bin/codex" plugin add harness@harness-marketplace >/dev/null || true
"$M/usr/bin/codex" mcp list >/dev/null
"$M/envctl/assets/scripts/envctl-codex-cleanup.sh" clean
"$M/envctl/assets/scripts/envctl-codex-cleanup.sh" verify
` |
| `setting` | `component[0].install.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
ROOT="$M/envctl"; source "$ROOT/assets/scripts/envctl-gh-fetch.sh"
DEST="$M/.toolchains/gh"
# Bootstrapping gh itself: the redirect-resolve is a github.com HEAD (not the JSON API) so it
# stays plain curl; the asset download goes through the resolver (bearer when a token is already
# available, identical bytes otherwise).
TAG="$(curl -fsSLI -o /dev/null -w '%{url_effective}' https://github.com/cli/cli/releases/latest | sed 's#.*/tag/##')"
VER="${TAG#v}"
URL="https://github.com/cli/cli/releases/download/${TAG}/gh_${VER}_linux_amd64.tar.gz"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
envctl_gh_curl "$URL" -o "$tmp/gh.tgz"
install -d -m 755 "$DEST"
tar -xzf "$tmp/gh.tgz" -C "$DEST" --strip-components=1
install -d -m 755 "$META_ROOT/usr/bin"
envctl_frontdoor "$DEST/bin/gh" "$META_ROOT/usr/bin/gh"


` |
| `setting` | `component[0].install.script` | `manifest/components.d/handoff-hf.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
repo="${HANDOFF_REPO:-$M/handoff}"
[ -f "$repo/Cargo.toml" ] || { echo "handoff-hf: missing handoff repo at $repo" >&2; exit 1; }
export CARGO_HOME="$M/.toolchains/cargo"
export CARGO_TARGET_DIR="$M/var/cache/envctl/cargo-target/handoff"
export PATH="$M/usr/bin:$CARGO_HOME/bin:$PATH"
cargo build --release -p hf --manifest-path "$repo/Cargo.toml" --locked --target-dir "$CARGO_TARGET_DIR"
install -d -m 755 "$M/usr/bin" "$M/.local/bin"
for bin in hf hf-mcp; do
  src="$CARGO_TARGET_DIR/release/$bin"
  [ -x "$src" ] || { echo "handoff-hf: missing built binary $src" >&2; exit 1; }
  link="$M/usr/bin/$bin"
  envctl_frontdoor "$src" "$link"
done
ln -sfn "$M/usr/bin/hf" "$M/.local/bin/hf"
` |
| `setting` | `component[0].install.script` | `manifest/components.d/just.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
install -d -m 700 "$M/.toolchains/src"
SRC="$M/.toolchains/src/just"
if [ -d "$SRC/.git" ]; then git -C "$SRC" remote set-url origin 'https://github.com/casey/just' || true; else git clone 'https://github.com/casey/just' "$SRC"; fi
git -C "$SRC" fetch --depth 1 origin 5097d64c8b765f8f6bf0f19d13be199bb1d1769c && git -C "$SRC" checkout --detach 5097d64c8b765f8f6bf0f19d13be199bb1d1769c
cd "$SRC"
cargo build --release --locked || cargo build --release
install -d -m 755 "$M/usr/bin"
envctl_frontdoor "$SRC/target/release/just" "$M/usr/bin/just"


` |
| `setting` | `component[0].install.script` | `manifest/components.d/meta-core-clis.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
DEST="$M/.toolchains/git"
install -d -m 755 "$META_ROOT/usr/bin"
PATH="$(printf '%s' "$PATH" | tr ':' '
' | grep -v -E "^$META_ROOT/usr/bin$" | paste -sd: -)"
src_bin="$(readlink -f "$(command -v git)")"
case "$src_bin" in "$DEST"/*) src_bin="$(find /nix/store -path '*/bin/git' -type f 2>/dev/null | grep 'git-2.54.0' | head -n1)";; esac
[ -n "$src_bin" ] || { echo "FATAL: no Git 2.54.0 source binary found"; exit 1; }
src_root="$(cd "$(dirname "$src_bin")/.." && pwd)"
rm -rf "$DEST"
mkdir -p "$DEST"
cp -a "$src_root/." "$DEST/"
envctl_frontdoor "$DEST/bin/git" "$META_ROOT/usr/bin/git"

` |
| `setting` | `component[0].install.script` | `manifest/components.d/meta-env-plugin.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
DEST="$M/usr/libexec/envctl/meta-env"
LINK="$M/usr/bin/meta-env"
export CARGO_HOME="$M/.toolchains/cargo"
export PATH="$M/usr/bin:$CARGO_HOME/bin:$PATH"
# Build only the meta-env autobin from the envctl CLI crate (no GUI → no system libs). wild-linker
# acceleration is inherited via the meta-root $M/.cargo/config.toml.
cargo build --release -p envctl --bin meta-env --manifest-path "$M/envctl/Cargo.toml"
install -d -m 755 "$DEST/bin" "$M/usr/bin"
install -m 755 "$M/envctl/target/release/meta-env" "$DEST/bin/meta-env"
envctl_frontdoor "$DEST/bin/meta-env" "$LINK"

` |
| `setting` | `component[0].install.script` | `manifest/components.d/ohmyzsh.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
OMZ="$M/ohmyzsh"
ZDD="$M/.toolchains/zsh-config"
CUSTOM="$ZDD/custom"

# 1. Ensure the FlexNetOS/ohmyzsh fork is present (registered meta project; clone
#    as a fallback). FULL clone — never shallow (keeps tags/branches for omz update).
if [ ! -f "$OMZ/oh-my-zsh.sh" ]; then
  git clone git@github.com:FlexNetOS/ohmyzsh.git "$OMZ"
fi

# 2. Clone the canonical add-ons (FlexNetOS forks) into the meta ZSH_CUSTOM.
#    FULL clones so \`git pull\` updates work. Idempotent.
install -d -m 755 "$CUSTOM/plugins"
for p in zsh-autosuggestions zsh-syntax-highlighting zsh-completions; do
  if [ ! -d "$CUSTOM/plugins/$p/.git" ]; then
    git clone "git@github.com:FlexNetOS/${p}.git" "$CUSTOM/plugins/$p"
  fi
done

# 2b. Powerlevel10k theme (FlexNetOS fork, full clone). Nerd Font already present
#     (JetBrainsMono/FiraCode in $META_ROOT/.local/share/fonts). The p10k config WIZARD is
#     left enabled — it runs on first interactive open; its output (.p10k.zsh) is
#     sourced by the .zshrc below, so the managed .zshrc never needs hand-edits.
install -d -m 755 "$CUSTOM/themes"
if [ ! -d "$CUSTOM/themes/powerlevel10k/.git" ]; then
  git clone "git@github.com:FlexNetOS/powerlevel10k.git" "$CUSTOM/themes/powerlevel10k"
fi

# 3. Meta-owned ZDOTDIR with the FULL plugin set. $OMZ/$CUSTOM baked absolute;
#    runtime $ZSH escaped literal. zsh-syntax-highlighting MUST be the last plugin.
install -d -m 755 "$ZDD"
cat > "$ZDD/.zshrc" <<EOF
# Meta-owned zsh config — generated by the envctl ohmyzsh component. Do not edit.
# Full-feature ohmyzsh for the meta zsh. Launch: ZDOTDIR="$ZDD" zsh
export ZSH="$OMZ"
export ZSH_CUSTOM="$CUSTOM"
ZSH_THEME="powerlevel10k/powerlevel10k"
plugins=(
  git sudo z extract colored-man-pages command-not-found
  copypath copyfile dirhistory history
  zsh-completions
  zsh-autosuggestions
  zsh-syntax-highlighting
)
source "\$ZSH/oh-my-zsh.sh"
# Powerlevel10k: the configuration wizard runs on first interactive open (no
# ~/.p10k.zsh yet). It writes \${ZDOTDIR}/.p10k.zsh, which is sourced here — so the
# wizard's output is honored without editing this managed file.
[[ ! -f "\${ZDOTDIR:-\$HOME}/.p10k.zsh" ]] || source "\${ZDOTDIR:-\$HOME}/.p10k.zsh"
EOF
` |
| `setting` | `component[0].install.script` | `manifest/components.d/zsh-migration-launcher.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
ZDD="$M/.toolchains/zsh-config"
LAUNCH="$ZDD/launch.sh"
ICON="$ZDD/meta-zsh.svg"
APPS="$META_ROOT/.local/share/applications"
DESKTOP="$APPS/meta-zsh-migration.desktop"
install -d -m 755 "$ZDD" "$APPS"

# 1. Launcher script — picks a present terminal emulator and runs the meta zsh
#    with the meta ZDOTDIR. Literal heredoc: all $vars resolve at click time.
cat > "$LAUNCH" <<'SH'
#!/usr/bin/env bash
# Meta zsh migration terminal — generated by the envctl zsh-migration-launcher component.
set -e
M="${META_ROOT:?META_ROOT required}"
export META_ROOT="$M"
export ZDOTDIR="$M/.toolchains/zsh-config"
ZSH_BIN="$META_ROOT/usr/bin/zsh"
export PATH="$META_ROOT/usr/bin:$PATH"
# Prefer the on-box, NON-/nix terminals (survive the system-nix teardown) by
# ABSOLUTE path — never yazelix's /nix ghostty. Ptyxis is the Ubuntu 26.04 /
# GNOME default; /usr/bin/ghostty is the system ghostty.
[ -x /usr/bin/ptyxis ]  && exec /usr/bin/ptyxis  -- "$ZSH_BIN"
[ -x /usr/bin/ghostty ] && exec /usr/bin/ghostty -e "$ZSH_BIN"
# Fallback: search PATH (last resort, may include the /nix ghostty).
for term in ptyxis ghostty kgx gnome-terminal konsole xterm; do
  if command -v "$term" >/dev/null 2>&1; then
    case "$term" in
      gnome-terminal|ptyxis) exec "$term" -- "$ZSH_BIN" ;;
      *)                     exec "$term" -e "$ZSH_BIN" ;;
    esac
  fi
done
echo "meta-zsh: no terminal emulator found" >&2
exit 1
SH
chmod +x "$LAUNCH"

# 2. Icon (SVG, terminal with a green zsh prompt). Literal heredoc.
cat > "$ICON" <<'SVG'
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256">
  <rect width="256" height="256" rx="40" fill="#1b1f2a"/>
  <rect x="28" y="40" width="200" height="176" rx="16" fill="#0d1017" stroke="#2c3242" stroke-width="3"/>
  <circle cx="52" cy="62" r="6" fill="#ff5f56"/>
  <circle cx="74" cy="62" r="6" fill="#ffbd2e"/>
  <circle cx="96" cy="62" r="6" fill="#27c93f"/>
  <text x="52" y="150" font-family="monospace" font-size="64" font-weight="bold" fill="#27c93f">%</text>
  <text x="96" y="150" font-family="monospace" font-size="64" font-weight="bold" fill="#d6deeb">z</text>
  <rect x="150" y="120" width="36" height="10" fill="#d6deeb"/>
</svg>
SVG

# 3. Desktop entry. $LAUNCH / $ICON baked to absolute meta paths.
cat > "$DESKTOP" <<EOF
[Desktop Entry]
Type=Application
Version=1.0
Name=Meta zsh (migration terminal)
Comment=Meta-owned zsh (.toolchains/zsh) + ohmyzsh — drives the env migration
Exec=$LAUNCH
Icon=$ICON
Terminal=false
Categories=Utility;TerminalEmulator;
Keywords=zsh;shell;meta;migration;envctl;
EOF
chmod +x "$DESKTOP"

command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$APPS" >/dev/null 2>&1 || true
` |
| `setting` | `component[0].install.script` | `manifest/components.d/zsh.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
ROOT="$M/envctl"; source "$ROOT/assets/scripts/envctl-gh-fetch.sh"
DEST="$M/.toolchains/zsh"

# 1. Resolve the NEWEST stable tag (exclude *-test pre-releases). No hardcoded
#    version → auto-latest. Token-aware listing to dodge anonymous rate limits.
TAG="$(envctl_gh_api repos/zsh-users/zsh/tags?per_page=100 \
        | grep -oE '"name": *"zsh-[0-9]+\.[0-9]+(\.[0-9]+)?"' \
        | sed -E 's/.*"(zsh-[0-9.]+)"/\1/' \
        | sort -V | tail -1)"
[ -n "$TAG" ] || { echo "envctl[zsh]: could not resolve latest zsh tag" >&2; exit 1; }
VER="${TAG#zsh-}"
echo "envctl[zsh]: building zsh $VER (latest stable, unpinned)" >&2

# 2. Fetch the official release tarball (ships a prebuilt ./configure → no
#    autotools needed). Plain curl: these mirrors are not GitHub. zsh.org primary,
#    SourceForge fallback.
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
TARBALL="zsh-${VER}.tar.xz"
fetched=""
for url in \
  "https://www.zsh.org/pub/${TARBALL}" \
  "https://downloads.sourceforge.net/project/zsh/zsh/${VER}/${TARBALL}"; do
  if curl -fsSL --max-time 180 -L -o "$tmp/$TARBALL" "$url"; then fetched="$url"; break; fi
done
[ -n "$fetched" ] || { echo "envctl[zsh]: failed to download $TARBALL from any mirror" >&2; exit 1; }

# 3. Build into the meta prefix vs system libc/ncurses (NOT /nix). Default cc
#    (system gcc) — do not force the meta clang link-driver here.
tar -xJf "$tmp/$TARBALL" -C "$tmp"
src="$tmp/zsh-${VER}"
( cd "$src" \
  && ./configure --prefix="$DEST" --enable-multibyte >/dev/null \
  && make -j"$(nproc)" >/dev/null \
  && rm -rf "$DEST" && make install >/dev/null )

# 4. Wire into $META_ROOT/usr/bin (already on PATH via run_env()).
install -d -m 755 "$META_ROOT/usr/bin"
envctl_frontdoor "$DEST/bin/zsh" "$META_ROOT/usr/bin/zsh"


` |
| `setting` | `component[0].install.script` | `manifest/env-ctl.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
# Meta-owned toolchain (Epic H TASK-0069): build with the meta cargo, NOT $META_ROOT/.toolchains/cargo/bin.
M="${META_ROOT:?META_ROOT required}"
export CARGO_HOME="$M/.toolchains/cargo"
META_BIN="$M/usr/bin"
export PATH="$M/.toolchains/cargo/bin:$META_BIN:$META_ROOT/usr/bin:$PATH"
DEST="$M/usr/libexec/envctl/secrets/bin"

# MSRV gate (rust-version = 1.80 in the workspace). Fail closed, do not silently upgrade.
# Pass iff cargo >= 1.80.0: with the MSRV line FIRST, \`sort -V -C\` succeeds only when the
# pair is ascending (1.80.0 <= $ver). (Operands were reversed, which rejected every cargo
# newer than 1.80 — i.e. always — failing the install on a healthy toolchain.)
ver="$(cargo --version | awk '{print $2}')"
printf '1.80.0\n%s\n' "$ver" | sort -V -C || { echo "FATAL: cargo $ver < MSRV 1.80"; exit 1; }

# Resolve the envctl workspace root. Explicit override wins; otherwise use the meta-hosted
# checkout under $META_ROOT. Do not fall back to a user-home checkout: envctl is a first-class
# meta member and builds from the meta workspace only.
repo="${ENV_CTL_REPO:-$META_ROOT/envctl}"
test -d "${repo:-/nonexistent}/crates/secretd" || { echo "FATAL: envctl workspace not found (set ENV_CTL_REPO to the checkout root)"; exit 1; }

# secretd is built WITH \`--features seed-factor\` so the Cognitum Seed USB unlock factor
# (Profile S) is compiled in. Without it, \`RealUsbProbe::keyfile_for\` is a no-op stub that
# always returns \`None\`, so the USB unlock factor can NEVER succeed (the daemon falls back to
# passphrase only) — i.e. a stock build cannot reproduce a USB-unlock-capable daemon. seed-factor
# only pulls pure-Rust ring/rustls (already in the resolved graph, so the no-C trust-boundary gate
# stays green) and degrades cleanly to passphrase-only when no Seed is present, so it is always
# safe to enable. secretctl has no such feature and is built plain.
cargo build --release --manifest-path "$repo/Cargo.toml" -p envctl-secretd --features seed-factor
cargo build --release --manifest-path "$repo/Cargo.toml" -p envctl-secretctl

# Meta-prefix install (Epic H TASK-0069): private binaries live under
# $M/usr/libexec/envctl/secrets/bin, exposed through regular executable wrappers in $M/usr/bin.
# Never install directly into $META_ROOT/.toolchains/cargo/bin (system-depth).
install -Dm755 "$repo/target/release/secretd"   "$DEST/secretd"
install -Dm755 "$repo/target/release/secretctl" "$DEST/secretctl"
install -d -m755 "$META_BIN"
envctl_frontdoor "$DEST/secretd"   "$META_BIN/secretd"
envctl_frontdoor "$DEST/secretctl" "$META_BIN/secretctl"

cleanup_legacy_cargo_bin() {
  b="$1"
  legacy="$META_ROOT/.toolchains/cargo/bin/$b"
  owned="$DEST/$b"
  [ -e "$legacy" ] || return 0
  # If an old symlink already points at the new meta-local binary, remove the duplicate link.
  if [ -L "$legacy" ] && [ "$(readlink -f "$legacy" 2>/dev/null)" = "$(readlink -f "$owned" 2>/dev/null)" ]; then
    rm -f "$legacy"
    echo "removed legacy cargo-bin symlink: $legacy"
    return 0
  fi
  # If the old regular file is byte-identical to the just-installed meta binary, archive it.
  # Different/foreign binaries are left in place and surfaced by verify; strict upgrade-only means
  # never deleting an unproven executable.
  if [ -f "$legacy" ] && [ -f "$owned" ] && cmp -s "$legacy" "$owned"; then
    arch="$META_ROOT/var/lib/envctl/legacy-archives/$(date -u +%Y%m%dT%H%M%SZ)/.cargo/bin"
    install -d -m700 "$arch"
    mv "$legacy" "$arch/$b"
    echo "archived legacy cargo-bin copy: $legacy -> $arch/$b"
  elif [ -e "$legacy" ]; then
    echo "WARNING: legacy cargo-bin $legacy differs from $owned; left in place (strict upgrade-only)"
  fi
}
cleanup_legacy_cargo_bin secretd
cleanup_legacy_cargo_bin secretctl

# XDG dirs, fail-closed perms (ARCHITECTURE layout). RUNTIME dir is created by the unit at start.
install -d -m700 "$META_ROOT/.config/env-ctl"
install -d -m700 "$META_ROOT/.local/share/env-ctl"
install -d -m700 "$META_ROOT/.local/state/env-ctl"

# Durable store config (docs/ops/08): point secretd at the loopback sqld (libSQL remote backend).
# Idempotent: written only if absent (operator edits are preserved). The AUTH TOKEN is NEVER written
# here — it comes from SECRETD_LIBSQL_AUTH_TOKEN[_FILE] only (config.rs refuses a token in this file).
cfg="$META_ROOT/.config/env-ctl/secretd.toml"
if [ ! -f "$cfg" ]; then
  cat > "$cfg" <<'TOML'
# secretd store backend (OI-1 (a)). libSQL \`remote\` -> a LOOPBACK sqld (see manifest/sqld.toml).
# The auth token is NEVER stored here (credentials come from SECRETD_LIBSQL_AUTH_TOKEN[_FILE]).
[store]
backend = "libsql"
url = "http://127.0.0.1:8080"
TOML
  chmod 0600 "$cfg"
fi

# TASK-0033 (F7 / FS-S21): VPS Profile B install-time fail-closed gate. A \`[profile] topology =
# "remote"\` (VPS) deployment MUST configure an operator_authorizer_url (the substitute presence
# factor); without it, secretd would SILENTLY downgrade to ungated egress. Refuse the install FATALLY
# rather than ship that downgrade (secretd ALSO refuses to start — this is the earlier, install-time
# guard). On-box (the default, or no [profile] block) is unaffected.
if grep -Eq '^\s*topology\s*=\s*"(remote|vps)"' "$cfg" 2>/dev/null; then
  grep -Eq '^\s*operator_authorizer_url\s*=' "$cfg" \
    || { echo "FATAL: secretd.toml has [profile] topology=remote (VPS) but no operator_authorizer_url (FS-S21 substitute presence factor); refusing to install a silently ungated VPS"; exit 1; }
fi


# VERIFY: bins answer, AND secretd's own non-serving self-check passes. \`secretd --self-check\` runs
# the daemon's startup pre-flight (ring crypto provider, XDG paths, store-config validation) and
# EXITS — it never binds the socket, connects the store, or serves, so verify cannot block and is safe
# to run while the daemon is live. A misconfigured/un-built daemon reports verify=false (drift
# visible) WITHOUT triggering a destructive fix — detect, not verify, is the install predicate.
#
# SERVING PROBE (non-fatal): additionally, IF the unit is active, confirm a live \`secretctl status\`
# round-trip — this is what proves the sd-notify READY + durable-store fix actually serves. It is
# strictly best-effort: a not-yet-started/locked daemon must NOT fail verify (locked is HEALTHY; the
# vault is simply unprovisioned), and verify never asserts "unlocked" and never triggers a fix. The
# hard predicate stays the two binary/self-check checks; the serving probe only logs.
` |
| `setting` | `component[0].install.script` | `manifest/grit.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
export CARGO_HOME="$META_ROOT/.toolchains/cargo"
export PATH="$CARGO_HOME/bin:$PATH"

# MSRV gate: pass iff cargo >= 1.80.0. With the MSRV line FIRST, \`sort -V -C\` succeeds only when
# the pair is ascending (1.80.0 <= $ver). (Same ordering fix as env-ctl.toml — do NOT reverse.)
ver="$(cargo --version | awk '{print $2}')"
printf '1.80.0\n%s\n' "$ver" | sort -V -C || { echo "FATAL: cargo $ver < MSRV 1.80"; exit 1; }

# grit lives in the meta workspace (registered in ../.meta.yaml, .gitignore'd, its own repo).
repo="${GRIT_REPO:-$META_ROOT/grit}"
test -d "$repo" || { echo "FATAL: grit checkout not found at $repo (set GRIT_REPO)"; exit 1; }

cargo install --path "$repo" --locked
` |
| `setting` | `component[0].install.script` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
if [ ! -e /nix ]; then
  curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install --no-confirm
fi
install -d -m 755 "$M/.toolchains/nix/bin" "$META_ROOT/usr/bin"
src="$(readlink -f /nix/var/nix/profiles/default/bin/nix)"
install -m 755 "$src" "$M/.toolchains/nix/bin/nix"
envctl_frontdoor "$M/.toolchains/nix/bin/nix" "$META_ROOT/usr/bin/nix"

` |
| `setting` | `component[0].install.script` | `manifest/odysseus.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
rt="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
# Prepend the meta podman bridges so a bare \`podman\` resolves even in a stripped engine env.
export PATH="$M/.local/bin:$M/.toolchains/podman/usr/local/bin:$M/usr/bin:$PATH"
ROOT="$M/.local/share/odysseus"; SRC="$ROOT/src"; DATA="$ROOT/data"; LOGS="$ROOT/logs"
REPO="${ODYSSEUS_REPO:-https://github.com/pewdiepie-archdaemon/odysseus.git}"
REF="${ODYSSEUS_REF:-ebead8083e84f58f7e1012f22c9a9266a13fa1ee}"
SOCK="$rt/podman/podman.sock"

# 0) fail-closed precondition: the meta-local podman engine must be present (we do NOT declare it
#    as a hard \`requires\`, to avoid the destructive podman-reinstall footgun — see the note above).
command -v podman >/dev/null 2>&1 || { echo "odysseus needs the meta-local podman engine; run: envctl install podman"; exit 1; }

# 1) Docker-API endpoint on the meta-local Podman engine (rootless; no sudo) + reboot persistence.
systemctl --user enable --now podman.socket 2>/dev/null || true
loginctl enable-linger "$(id -un)" 2>/dev/null || true
# the compose services declare \`restart: unless-stopped\`; podman-restart.service (+ linger) brings
# them back after a reboot. Without it the stack stays down post-reboot.
systemctl --user enable podman-restart.service 2>/dev/null || true

# 2) meta-local state dirs.
#
# Rootless Podman can leave bind-mounted volume trees owned by shifted IDs after a run. Re-running
# install must not downgrade the service by chmod/chowning existing data/log paths back from the
# namespace owner (or failing when the host user cannot chmod them). Create missing dirs only; leave
# existing state untouched. The podman-unshare fallback lets rootless Podman create a missing child
# below a shifted-ID parent without any system-depth privilege escalation.
install -d -m 700 "$ROOT"

odysseus_ensure_state_dir() {
  d="$1"
  if [ -d "$d" ]; then
    return 0
  fi

  if mkdir -p "$d" 2>/dev/null; then
    chmod 700 "$d" 2>/dev/null || true
    return 0
  fi

  podman unshare mkdir -p "$d"
  podman unshare chmod 700 "$d" 2>/dev/null || true
}

odysseus_ensure_state_dir "$DATA"
odysseus_ensure_state_dir "$LOGS"
odysseus_ensure_state_dir "$DATA/ssh"
odysseus_ensure_state_dir "$DATA/huggingface"
odysseus_ensure_state_dir "$DATA/local"

# 3) pinned clone (idempotent; detached at the exact SHA — never floating).
if [ -d "$SRC/.git" ]; then git -C "$SRC" remote set-url origin "$REPO" || true; else git clone "$REPO" "$SRC"; fi
git -C "$SRC" fetch --depth 1 origin "$REF" 2>/dev/null || git -C "$SRC" fetch origin
git -C "$SRC" checkout --detach "$REF"

# 4) generate the meta .env (gitignored upstream; secrets are NOT written here — AUTH stays on
#    with an auto-generated admin password printed in the logs; provider keys come from secretctl
#    at runtime if/when wired). Loopback + meta data dirs + safe defaults.
cat > "$SRC/.env" <<EOF
# generated by envctl manifest/odysseus.toml — do not commit (upstream .gitignore covers .env)
APP_BIND=127.0.0.1
APP_PORT=7000
APP_DATA_DIR=$DATA
APP_LOGS_DIR=$LOGS
AUTH_ENABLED=true
LOCALHOST_BYPASS=false
SECURE_COOKIES=false
PUID=$(id -u)
PGID=$(id -g)
OLLAMA_BASE_URL=http://host.docker.internal:11434/v1
EOF
chmod 600 "$SRC/.env"

# 5) compose override: replace the odysseus volume list (compose \`!override\`) so the hardcoded
#    /var/run/docker.sock becomes the rootless PODMAN socket (never the root daemon), drop the
#    docker-GID group_add (no docker group under rootless podman), and make SearXNG/ntfy
#    internal-only (no host port → avoids the sqld:8080 clash; SECURITY.md wants them internal).
#    \`logging: json-file\` on every service: the meta-local static conmon has NO journald support,
#    and the podman socket service defaults to journald → conmon exits 1; json-file maps to
#    podman's k8s-file driver and makes the stack start. (Set per-service so it reaches the socket
#    service that actually creates the containers — a CONTAINERS_CONF_OVERRIDE env would not.)
cat > "$SRC/compose.meta.yml" <<EOF
services:
  odysseus:
    logging: { driver: json-file }
    volumes: !override
      - "$DATA:/app/data:z"
      - "$LOGS:/app/logs:z"
      - "$DATA/ssh:/app/.ssh:z"
      - "$DATA/huggingface:/app/.cache/huggingface:z"
      - "$DATA/local:/app/.local:z"
      - "$SOCK:/var/run/docker.sock"
    group_add: !reset null
  searxng:
    logging: { driver: json-file }
    ports: !reset null
  ntfy:
    logging: { driver: json-file }
    ports: !reset null
  chromadb:
    logging: { driver: json-file }
EOF

# 6) bring the stack up on the meta-local Podman engine.
cd "$SRC"
podman compose -f docker-compose.yml -f compose.meta.yml up -d --build

# 7) observability: best-effort health event to the planning loops (weave is on PATH or skip).
command -v weave >/dev/null 2>&1 && weave notify --from envctl --to all --subject "odysseus-ready" \
  --body "odysseus up via podman compose @127.0.0.1:7000 (sandboxed, AGPL, QUALIFY)" 2>/dev/null || true
echo "odysseus stack started; admin password in: podman logs \$(podman ps --filter name=odysseus -q | head -1)"
` |
| `setting` | `component[0].install.script` | `manifest/prompt_hub.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
export CARGO_HOME="$META_ROOT/.toolchains/cargo"
export PATH="$CARGO_HOME/bin:$PATH"

# MSRV gate: pass iff cargo >= 1.91.1 (prompt_hub workspace rust-version). MSRV line FIRST so
# \`sort -V -C\` succeeds only when the pair is ascending (1.91.1 <= $ver). (Same ordering idiom as
# grit.toml / rusty-idd.toml / env-ctl.toml — do NOT reverse.)
ver="$(cargo --version | awk '{print $2}')"
printf '1.91.1\n%s\n' "$ver" | sort -V -C || { echo "FATAL: cargo $ver < prompt_hub MSRV 1.91.1"; exit 1; }

# prompt_hub lives in the meta workspace (registered in ../.meta.yaml, .gitignore'd, its own repo).
repo="${PROMPT_HUB_REPO:-$META_ROOT/prompt_hub}"
test -d "$repo" || { echo "FATAL: prompt_hub checkout not found at $repo (set PROMPT_HUB_REPO)"; exit 1; }

# Front-door CLI (package \`prompthub\`, bin \`prompthub\`) + the HTTP server (\`prompthub-server\`).
# Default features: the heavy optional backends (smart-ort/ONNX, tui, otel, …) are opt-in and NOT
# needed for the box-wide binaries; a feature build is a usage choice, out of scope for install.
cargo install --path "$repo/prompthub" --locked
cargo install --path "$repo/prompthub-server" --locked
` |
| `setting` | `component[0].install.script` | `manifest/rusty-idd.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
export CARGO_HOME="$M/.toolchains/cargo"
export RUSTUP_HOME="$M/.toolchains/rustup"
BIN="${ENVCTL_BIN_DIR:-$M/usr/bin}"
export PATH="$CARGO_HOME/bin:$BIN:$PATH"

# MSRV gate: pass iff cargo >= 1.88.0. MSRV line FIRST so \`sort -V -C\` succeeds only when the pair is
# ascending (1.88.0 <= $ver). (Same ordering idiom as grit.toml/env-ctl.toml — do NOT reverse.)
ver="$(cargo --version | awk '{print $2}')"
printf '1.88.0\n%s\n' "$ver" | sort -V -C || { echo "FATAL: cargo $ver < MSRV 1.88"; exit 1; }

# rusty-idd lives in the meta workspace (registered in ../.meta.yaml, .gitignore'd, its own repo).
repo="${RUSTY_IDD_REPO:-$M/rusty-idd}"
test -d "$repo" || { echo "FATAL: rusty-idd checkout not found at $repo (set RUSTY_IDD_REPO)"; exit 1; }

# Install only the front-door binary's package (crates/cli = rusty-idd-cli, bin name \`rusty-idd\`)
# into a meta-owned root, then expose a regular frontdoor wrapper on $META_ROOT/usr/bin.
ROOT="$M/.toolchains/rusty-idd"
cargo install --path "$repo/crates/cli" --locked --root "$ROOT"
mkdir -p "$BIN"
envctl_frontdoor "$ROOT/bin/rusty-idd" "$BIN/rusty-idd"


# VERIFY: the installed binary answers. Read-only; a missing/broken rusty-idd reports verify=false
# (drift visible) WITHOUT triggering a destructive fix — detect, not verify, is the install gate.
` |
| `setting` | `component[0].install.script` | `manifest/sqld.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
ver="${SQLD_VERSION:-v0.24.32}"
arch="$(uname -m)"
case "$arch" in
  x86_64)  target="x86_64-unknown-linux-gnu" ;;
  aarch64) target="aarch64-unknown-linux-gnu" ;;
  *) echo "FATAL: unsupported arch $arch for sqld release binary"; exit 1 ;;
esac

envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
M="${META_ROOT:?META_ROOT required}"
DEST="$M/.toolchains/sqld/bin"
install -d -m755 "$DEST" "$M/usr/bin"
install -d -m700 "$M/.local/share/sqld"   # loopback data dir (the durable libSQL database)

if [ ! -x "$DEST/sqld" ]; then
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  # Release asset naming: tag = libsql-server-<ver>; asset = libsql-server-<target>.tar.xz.
  url="https://github.com/tursodatabase/libsql/releases/download/libsql-server-${ver}/libsql-server-${target}.tar.xz"
  echo "fetching sqld ${ver} for ${target}"
  curl -fsSL "$url" -o "$tmp/sqld.tar.xz" \
    || { echo "FATAL: could not download sqld release from $url"; exit 1; }
  tar -xJf "$tmp/sqld.tar.xz" -C "$tmp"
  # The tarball contains the \`sqld\` binary (under a versioned dir); place just the binary.
  bin="$(find "$tmp" -type f -name sqld | head -n1)"
  test -n "$bin" || { echo "FATAL: sqld binary not found in tarball"; exit 1; }
  install -Dm755 "$bin" "$DEST/sqld"
  envctl_frontdoor "$DEST/sqld" "$M/usr/bin/sqld"
fi
` |
| `setting` | `component[0].name` | `manifest/ai-clis.toml` | `scope=component source_kind=manifest` | `Claude Code CLI (meta toolchain; versioned binary, meta-owned state wrapper)` |
| `setting` | `component[0].name` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `Nix (Determinate, flakes)` |
| `setting` | `component[0].name` | `manifest/rusty-idd.toml` | `scope=component source_kind=manifest` | `rusty-idd — harness control-plane front door` |
| `setting` | `component[0].remove.args` | `manifest/agent-env.toml` | `scope=component source_kind=manifest` | `["-lc","export PATH=\"$META_ROOT/.toolchains/cargo/bin:$META_ROOT/usr/bin:$PATH\"; envctl agent clean --scope project --apply"]` |
| `setting` | `component[0].remove.args` | `manifest/components.d/handoff-hf.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; frontdoor_is_managed(){ front=\"$1\"; private=\"$2\"; if [ -L \"$front\" ] && [ \"$(readlink -f \"$front\" 2>/dev/null || true)\" = \"$(readlink -f \"$private\" 2>/dev/null || true)\" ]; then return 0; fi; [ -f \"$front\" ] && grep -Fqx \"exec \\\"$private\\\" \\\"\\$@\\\"\" \"$front\"; }; for bin in hf hf-mcp; do private=\"$M/var/cache/envctl/cargo-target/handoff/release/$bin\"; front=\"$M/usr/bin/$bin\"; if frontdoor_is_managed \"$front\" \"$private\"; then rm -f \"$front\"; fi; done; if [ \"$(readlink \"$M/.local/bin/hf\" 2>/dev/null || true)\" = \"$M/usr/bin/hf\" ]; then rm -f \"$M/.local/bin/hf\"; fi"]` |
| `setting` | `component[0].remove.args` | `manifest/components.d/meta-core-clis.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; t=\"$META_ROOT/usr/bin/git\"; src=\"$M/.toolchains/git/bin/git\"; if { [ -L \"$t\" ] && [ \"$(readlink -f \"$t\" 2>/dev/null || true)\" = \"$(readlink -f \"$src\" 2>/dev/null || true)\" ]; } || { [ -f \"$t\" ] && grep -Fqx \"exec \\\"$src\\\" \\\"\\$@\\\"\" \"$t\"; }; then rm -f \"$t\"; fi; rm -rf \"$M/.toolchains/git\""]` |
| `setting` | `component[0].remove.args` | `manifest/grit.toml` | `scope=component source_kind=manifest` | `["-lc","export CARGO_HOME=\"$META_ROOT/.toolchains/cargo\"; export PATH=\"$CARGO_HOME/bin:$PATH\"; cargo uninstall grit || true"]` |
| `setting` | `component[0].remove.args` | `manifest/prompt_hub.toml` | `scope=component source_kind=manifest` | `["-lc","export CARGO_HOME=\"$META_ROOT/.toolchains/cargo\"; export PATH=\"$CARGO_HOME/bin:$PATH\"; cargo uninstall prompthub || true; cargo uninstall prompthub-server || true"]` |
| `setting` | `component[0].remove.args` | `manifest/rusty-idd.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; B=\"${ENVCTL_BIN_DIR:-$M/usr/bin}\"; t=\"$B/rusty-idd\"; src=\"$M/.toolchains/rusty-idd/bin/rusty-idd\"; if { [ -L \"$t\" ] && [ \"$(readlink -f \"$t\" 2>/dev/null || true)\" = \"$(readlink -f \"$src\" 2>/dev/null || true)\" ]; } || { [ -f \"$t\" ] && grep -Fqx \"exec \\\"$src\\\" \\\"\\$@\\\"\" \"$t\"; }; then rm -f \"$t\"; fi; rm -rf \"$M/.toolchains/rusty-idd\""]` |
| `setting` | `component[0].remove.script` | `manifest/cognitum-seed-trust.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
systemctl disable --now cognitum-seed-trust.service 2>/dev/null || true
M="${META_ROOT:?META_ROOT required by envctl hook}"
rm -f "$M/usr/libexec/envctl/cognitum-seed-trust-refresh" \
      /etc/systemd/system/cognitum-seed-trust.service \
      /etc/udev/rules.d/99-cognitum-seed-trust.rules
# NOTE: the pinned CA at $META_ROOT/etc/envctl/secrets/ca/cognitum-ca.crt is
# intentionally LEFT in place — removing it could break a working USB unlock. Only the refresh
# mechanism is removed here.
udevadm control --reload-rules || true
systemctl daemon-reload || true
` |
| `setting` | `component[0].remove.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `set -u
M="${META_ROOT:?META_ROOT required}"
t="$META_ROOT/usr/bin/gh"; [ -L "$t" ] && readlink "$t" | grep -q "$M/.toolchains/gh" && rm -f "$t"
rm -rf "$M/.toolchains/gh"
` |
| `setting` | `component[0].remove.script` | `manifest/components.d/just.toml` | `scope=component source_kind=manifest` | `set -u
M="${META_ROOT:?META_ROOT required}"
STORE="$M/.toolchains/src"
for t in "$M/usr/bin/just"; do if [ -L "$t" ] && readlink "$t" | grep -q "$STORE/just"; then rm -f "$t"; fi; done
rm -rf "$STORE/just"
` |
| `setting` | `component[0].remove.script` | `manifest/components.d/ohmyzsh.toml` | `scope=component source_kind=manifest` | `set -u
M="${META_ROOT:?META_ROOT required}"
# Removes the generated ZDOTDIR incl. the cloned add-ons under custom/. The
# ohmyzsh repo is a meta project and is NOT deleted here (managed via meta git).
rm -rf "$M/.toolchains/zsh-config"
` |
| `setting` | `component[0].remove.script` | `manifest/components.d/zsh-migration-launcher.toml` | `scope=component source_kind=manifest` | `set -u
M="${META_ROOT:?META_ROOT required}"
APPS="$META_ROOT/.local/share/applications"
rm -f "$APPS/meta-zsh-migration.desktop" "$M/.toolchains/zsh-config/launch.sh" "$M/.toolchains/zsh-config/meta-zsh.svg"
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$APPS" >/dev/null 2>&1 || true
` |
| `setting` | `component[0].remove.script` | `manifest/components.d/zsh.toml` | `scope=component source_kind=manifest` | `set -u
M="${META_ROOT:?META_ROOT required}"
t="$META_ROOT/usr/bin/zsh"; [ -L "$t" ] && readlink "$t" | grep -q "$M/.toolchains/zsh" && rm -f "$t"
rm -rf "$M/.toolchains/zsh"
` |
| `setting` | `component[0].remove.script` | `manifest/env-ctl.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
systemctl --user disable --now env-ctl.service 2>/dev/null || true
M="${META_ROOT:?META_ROOT required}"
# Self-guarded: drop only managed regular wrappers or legacy symlinks that resolve to the
# canonical meta-local install, then remove the canonical private binaries. Legacy
# .toolchains binaries are strict-upgrade fallbacks and are never removed here.
meta_frontdoor_is_managed() {
  front="$1"
  private="$2"
  private_real="$(readlink -f "$private" 2>/dev/null || true)"
  if [ -L "$front" ] && [ -n "$private_real" ] && [ "$(readlink -f "$front" 2>/dev/null || true)" = "$private_real" ]; then
    return 0
  fi
  [ -f "$front" ] && grep -Fqx "exec \"$private\" \"\$@\"" "$front"
}
for b in secretd secretctl; do
  private="$M/usr/libexec/envctl/secrets/bin/$b"
  meta_link="$M/usr/bin/$b"
  host_link="$META_ROOT/usr/bin/$b"
  if meta_frontdoor_is_managed "$host_link" "$private"; then rm -f "$host_link"; fi
  if [ "$meta_link" != "$host_link" ] && meta_frontdoor_is_managed "$meta_link" "$private"; then rm -f "$meta_link"; fi
done
rm -f "$M/usr/libexec/envctl/secrets/bin/secretd" "$M/usr/libexec/envctl/secrets/bin/secretctl"
systemctl --user daemon-reload || true
# vault.db / ca/ / audit are data_paths+config_paths below — envctl deletes them ONLY with --purge,
# after a UUID re-verify. remove() never touches user data (THREAT-MODEL: user data is never touched).
` |
| `setting` | `component[0].remove.script` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `M="${META_ROOT:?META_ROOT required}"; t="$META_ROOT/usr/bin/nix"; src="$M/.toolchains/nix/bin/nix"; if { [ -L "$t" ] && [ "$(readlink -f "$t" 2>/dev/null || true)" = "$(readlink -f "$src" 2>/dev/null || true)" ]; } || { [ -f "$t" ] && grep -Fqx "exec \"$src\" \"\$@\"" "$t"; }; then rm -f "$t"; fi; rm -rf "$M/.toolchains/nix"` |
| `setting` | `component[0].remove.script` | `manifest/odysseus.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
export PATH="$M/.local/bin:$M/.toolchains/podman/usr/local/bin:$M/usr/bin:$PATH"
SRC="$M/.local/share/odysseus/src"
if [ -d "$SRC" ] && command -v podman >/dev/null 2>&1; then
  ( cd "$SRC" && podman compose -f docker-compose.yml -f compose.meta.yml down 2>/dev/null ) || true
fi
rm -rf "$SRC"
# data/ + logs/ are data_paths — intentionally preserved here (reversible).
echo "odysseus stack removed; data preserved under $M/.local/share/odysseus (use reset --purge to delete)"
` |
| `setting` | `component[0].remove.script` | `manifest/sqld.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
systemctl --user disable --now sqld.service 2>/dev/null || true
M="${META_ROOT:?META_ROOT required}"; t="$M/usr/bin/sqld"; src="$M/.toolchains/sqld/bin/sqld"; { [ -L "$t" ] && [ "$(readlink -f "$t" 2>/dev/null)" = "$(readlink -f "$src" 2>/dev/null)" ]; } || { [ -f "$t" ] && grep -Fqx "exec \"$src\" \"\$@\"" "$t"; } && rm -f "$t" || true; rm -rf "$M/.toolchains/sqld/bin"
systemctl --user daemon-reload || true
# $META_ROOT/.local/share/sqld is data_paths below — never deleted by remove(); --purge only (user data rule).
` |
| `setting` | `component[0].requires` | `manifest/components.d/handoff-hf.toml` | `scope=component source_kind=manifest` | `["rustup"]` |
| `setting` | `component[0].requires` | `manifest/components.d/meta-env-plugin.toml` | `scope=component source_kind=manifest` | `["rustup"]` |
| `setting` | `component[0].requires` | `manifest/desktop-app.toml` | `scope=component source_kind=manifest` | `["rustup"]` |
| `setting` | `component[0].requires` | `manifest/env-ctl.toml` | `scope=component source_kind=manifest` | `["rustup","sqld"]` |
| `setting` | `component[0].requires` | `manifest/grit.toml` | `scope=component source_kind=manifest` | `["rustup","libssl-dev"]` |
| `setting` | `component[0].requires` | `manifest/prompt_hub.toml` | `scope=component source_kind=manifest` | `["rustup","llvm-clang","libssl-dev"]` |
| `setting` | `component[0].requires` | `manifest/rusty-idd.toml` | `scope=component source_kind=manifest` | `["rustup"]` |
| `setting` | `component[0].verify.args` | `manifest/agent-env.toml` | `scope=component source_kind=manifest` | `["-lc","export PATH=\"$META_ROOT/.toolchains/cargo/bin:$META_ROOT/usr/bin:$PATH\"; envctl agent lock --config agent-env.yaml --check --locked"]` |
| `setting` | `component[0].verify.args` | `manifest/ai-clis.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"\nR=\"${ENVCTL_REAL_HOME:-$HOME}\"\ncur=\"$M/.toolchains/claude/current/bin/claude\"\nlink=\"$M/usr/bin/claude\"\nmeta_compat=\"$M/.local/bin/claude\"\nreal_compat=\"$R/.local/bin/claude\"\n[ -x \"$cur\" ]\n[ -x \"$link\" ] && [ ! -L \"$link\" ] && grep -q \"envctl claude wrapper\" \"$link\"\n[ -x \"$meta_compat\" ] && grep -q \"envctl claude wrapper\" \"$meta_compat\"\n[ -x \"$real_compat\" ] && grep -q \"envctl claude wrapper\" \"$real_compat\"\n[ \"$(readlink -f \"$R/.claude\" 2>/dev/null)\" = \"$(readlink -f \"$M/.local/share/claude\" 2>/dev/null)\" ]\ntimeout --kill-after=2s 8s \"$link\" --version >/dev/null\nCLEANUP=\"${ENVCTL_CLAUDE_CLEANUP:-$M/envctl/assets/scripts/envctl-claude-cleanup.sh}\"\n[ -x \"$CLEANUP\" ] || CLEANUP=\"$PWD/assets/scripts/envctl-claude-cleanup.sh\"\n\"$CLEANUP\" verify\n"]` |
| `setting` | `component[0].verify.args` | `manifest/cognitum-seed-autounlock.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required by envctl hook}\"\ntest -x \"$M/usr/libexec/envctl/cognitum-seed-autounlock\" \\\n  && test -f /etc/systemd/system/cognitum-seed-autounlock.service \\\n  && test -f /etc/udev/rules.d/99-cognitum-seed-autounlock.rules || exit 1\n\n# Non-fatal probe (runs as the invoking user; verify carries no sudo so it cannot setpriv).\nSECRETCTL=\"\"\nMETA_ROOT=\"${META_ROOT:?META_ROOT required by envctl hook}\"\nfor c in \"$META_ROOT/usr/bin/secretctl\" \"$META_ROOT/usr/libexec/envctl/secrets/bin/secretctl\" \"$META_ROOT/.toolchains/secrets/bin/secretctl\"; do\n  if command -v \"$c\" >/dev/null 2>&1; then SECRETCTL=\"$c\"; break; fi\ndone\nif [ -n \"${SECRETCTL:-}\" ] && \"$SECRETCTL\" status >/dev/null 2>&1; then\n  echo \"autounlock probe: secretd reachable as $(id -un) (vault status queried; non-fatal)\"\nelse\n  echo \"autounlock probe: secretd not reachable / secretctl absent (non-fatal; unlock applies on Seed hotplug)\"\nfi\nexit 0\n"]` |
| `setting` | `component[0].verify.args` | `manifest/cognitum-seed-trust.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required by envctl hook}\"\ntest -x \"$M/usr/libexec/envctl/cognitum-seed-trust-refresh\" \\\n  && test -f /etc/systemd/system/cognitum-seed-trust.service \\\n  && test -f /etc/udev/rules.d/99-cognitum-seed-trust.rules || exit 1\n\nif [ -n \"${COGNITUM_TRUST_DIR:-}\" ] && [ -d \"$COGNITUM_TRUST_DIR\" ]; then\n  TRUST_DIR=\"$COGNITUM_TRUST_DIR\"\nelse\n  TRUST_DIR=\"\"\n  for d in /run/media/\"${USER:-}\"/COGNITUM/trust /run/media/*/COGNITUM/trust /media/*/COGNITUM/trust; do\n    if [ -d \"$d\" ] && [ -f \"$d/cognitum-ca.pem\" ]; then TRUST_DIR=\"$d\"; break; fi\n  done\nfi\n\nif [ -z \"${TRUST_DIR:-}\" ]; then\n  echo \"pin probe: no Cognitum Seed anchor present (non-fatal; re-pin applies on hotplug)\"; exit 0\nfi\n\nMETA_ROOT=\"${META_ROOT:?META_ROOT required by envctl hook}\"\nDST=\"${ENVCTL_SEED_CA:-$META_ROOT/etc/envctl/secrets/ca/cognitum-ca.crt}\"\nif cmp -s \"$TRUST_DIR/cognitum-ca.pem\" \"$DST\"; then\n  echo \"pin probe: Device CA pin MATCHES USB anchor (healthy)\"\nelse\n  echo \"pin probe: Device CA pin STALE vs USB anchor (non-fatal; run fix to re-pin)\"\nfi\n# Always exit 0 — the probe never fails verify.\nexit 0\n"]` |
| `setting` | `component[0].verify.args` | `manifest/components.d/codex-global-baseline.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"\nR=\"${ENVCTL_REAL_HOME:-$HOME}\"\nC=\"$M/.local/share/codex/config.toml\"\nH=\"$M/.local/share/codex/hooks.json\"\n[ \"$(readlink -f \"$R/.codex\" 2>/dev/null)\" = \"$(readlink -f \"$M/.local/share/codex\" 2>/dev/null)\" ]\n[ \"$(readlink -f \"$R/.local/bin/codex\" 2>/dev/null)\" = \"$(readlink -f \"$M/usr/bin/codex\" 2>/dev/null)\" ]\n[ -f \"$C\" ] && [ -f \"$H\" ]\ngrep -q '^background_terminal_max_timeout = 300000' \"$C\"\ngrep -q '^tool_output_token_limit = 12000' \"$C\"\nfor f in apps auto_compaction browser_use browser_use_external computer_use enable_request_compression fast_mode goals guardian_approval hooks image_generation in_app_browser mentions_v2 multi_agent personality plugin_sharing plugins prevent_idle_sleep remote_compaction_v2 secret_auth_storage shell_snapshot shell_tool skill_mcp_dependency_install tool_call_mcp_elicitation tool_suggest unified_exec workspace_dependencies memories network_proxy; do grep -q \"^$f = true\" \"$C\"; done\nfor s in meta gitkb icm vox context7 weave openaiDeveloperDocs; do grep -q \"^\\[mcp_servers\\.$s\\]\" \"$C\"; done\ngrep -q '^\\[marketplaces\\.flexnetos-codex\\]' \"$C\"\ngrep -q '^\\[plugins\\.\"meta@flexnetos-codex\"\\]' \"$C\"\ngrep -q '^\\[plugins\\.\"harness@harness-marketplace\"\\]' \"$C\"\ngrep -q '^default_permissions = \"meta-workspace\"' \"$C\"\nfor a in meta-worker pr-explorer reviewer docs-researcher codex-baseline-researcher; do [ -f \"$M/.local/share/codex/agents/$a.toml\" ]; done\nfor slug in gpt-5.5 gpt-5.4-mini gpt-5.4-nano gpt-5.4 gpt-5.3-codex-spark gpt-5.6-sol gpt-5.6-terra gpt-5.6-luna; do grep -q '\"slug\": '\"\\\"$slug\\\"\" \"$M/.local/share/codex/model-catalog.json\"; done\ngrep -q 'Official OpenAI GPT-5.6 Sol limited-preview' \"$M/.local/share/codex/model-catalog.json\"\nfor var in FXN_AGENT_COMMUNICATION FXN_AGENT_TEAM_SWARM FXN_AGENT_TEAM_SWARM_MODEL_TAGS FXN_CODEX_MODEL_SOL FXN_CODEX_MODEL_TERRA FXN_CODEX_MODEL_LUNA FXN_CODEX_REMOTE_CONTROL; do grep -q \"export $var=\" \"$M/.codex/hooks/with-meta-env.sh\"; done\nCODEX_HOME=\"$M/.local/share/codex\" CODEX_SQLITE_HOME=\"$M/.local/state/codex\" \"$M/usr/bin/codex\" plugin list | grep -q 'meta@flexnetos-codex  installed, enabled'\nCODEX_HOME=\"$M/.local/share/codex\" CODEX_SQLITE_HOME=\"$M/.local/state/codex\" \"$M/usr/bin/codex\" plugin list | grep -q 'harness@harness-marketplace  installed, enabled'\nCODEX_HOME=\"$M/.local/share/codex\" CODEX_SQLITE_HOME=\"$M/.local/state/codex\" \"$M/usr/bin/codex\" mcp list >/dev/null\n! find \"$M/.toolchains/bun\" \"$M/.toolchains/.bun\" -path '*/@openai/codex*' -print -quit 2>/dev/null | grep -q .\n\"$M/envctl/assets/scripts/envctl-codex-cleanup.sh\" verify\n"]` |
| `setting` | `component[0].verify.args` | `manifest/components.d/meta-core-clis.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export PATH=\"$M/usr/bin:$PATH\"; [ -x \"$M/usr/bin/git\" ] && [ ! -L \"$M/usr/bin/git\" ] && grep -Fqx \"exec \\\"$M/.toolchains/git/bin/git\\\" \\\"\\$@\\\"\" \"$M/usr/bin/git\" && git --version | grep -q '2\\.54\\.0'"]` |
| `setting` | `component[0].verify.args` | `manifest/components.d/ohmyzsh.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; o=$(POWERLEVEL9K_DISABLE_CONFIGURATION_WIZARD=true ZDOTDIR=\"$M/.toolchains/zsh-config\" \"$META_ROOT/usr/bin/zsh\" -ic 'print -r -- \"OMZ=$ZSH P10K=${+functions[p10k]} AS=${+functions[_zsh_autosuggest_start]} HL=${+ZSH_HIGHLIGHT_VERSION}\"' 2>/dev/null); echo \"$o\"; echo \"$o\" | grep -q \"OMZ=$M/ohmyzsh\" && echo \"$o\" | grep -q 'P10K=1' && echo \"$o\" | grep -q 'AS=1' && echo \"$o\" | grep -q 'HL=1'"]` |
| `setting` | `component[0].verify.args` | `manifest/components.d/zsh-migration-launcher.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; desktop-file-validate \"$META_ROOT/.local/share/applications/meta-zsh-migration.desktop\" && [ -x \"$M/.toolchains/zsh-config/launch.sh\" ] && [ -f \"$M/.toolchains/zsh-config/meta-zsh.svg\" ]"]` |
| `setting` | `component[0].verify.args` | `manifest/components.d/zsh.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; PATH=\"$META_ROOT/usr/bin:$PATH\"; zsh --version && ! ldd \"$M/.toolchains/zsh/bin/zsh\" | grep -q /nix/store"]` |
| `setting` | `component[0].verify.args` | `manifest/env-ctl.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"\nexport PATH=\"$M/usr/bin:$META_ROOT/usr/bin:$PATH\"\n# Hard predicate (fail-closed): bins answer + the daemon's offline self-check passes.\nsecretctl --help >/dev/null && secretd --self-check >/dev/null || exit 1\n# Legacy cargo-bin copies must not shadow the meta-local install. They are cleaned by install/fix\n# when byte-identical or symlinked to the meta prefix; different foreign binaries are surfaced here\n# rather than removed destructively.\nfor b in secretd secretctl; do\n  legacy=\"$META_ROOT/.toolchains/cargo/bin/$b\"\n  owned=\"$M/usr/libexec/envctl/secrets/bin/$b\"\n  if [ -e \"$legacy\" ] && [ \"$(readlink -f \"$legacy\" 2>/dev/null)\" != \"$(readlink -f \"$owned\" 2>/dev/null)\" ]; then\n    echo \"legacy cargo-bin $b shadows meta-local install: $legacy\"\n    exit 1\n  fi\ndone\n# TASK-0033 (F7 / FS-S21): a VPS Profile B config MUST carry operator_authorizer_url. verify fails\n# closed on the same downgrade the install gate refuses (so drift introduced after install is caught).\ncfg=\"$META_ROOT/.config/env-ctl/secretd.toml\"\nif grep -Eq '^\\s*topology\\s*=\\s*\"(remote|vps)\"' \"$cfg\" 2>/dev/null; then\n  grep -Eq '^\\s*operator_authorizer_url\\s*=' \"$cfg\" \\\n    || { echo \"FATAL: VPS profile (topology=remote) without operator_authorizer_url (FS-S21)\"; exit 1; }\nfi\n# Best-effort serving probe — NEVER fails verify (locked is healthy; daemon may be stopped).\nif systemctl --user is-active --quiet env-ctl.service; then\n  if secretctl status >/dev/null 2>&1; then\n    echo \"serving probe: env-ctl.service active and secretctl status round-trips (healthy)\"\n  else\n    echo \"serving probe: env-ctl.service active but secretctl status did not round-trip (non-fatal)\"\n  fi\nelse\n  echo \"serving probe: env-ctl.service not active (non-fatal; start it to serve)\"\nfi\nexit 0\n"]` |
| `setting` | `component[0].verify.args` | `manifest/grit.toml` | `scope=component source_kind=manifest` | `["-lc","export CARGO_HOME=\"$META_ROOT/.toolchains/cargo\"; export PATH=\"$CARGO_HOME/bin:$PATH\"; grit --version >/dev/null"]` |
| `setting` | `component[0].verify.args` | `manifest/n8n-mcp.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export PATH=\"$M/usr/bin:$M/.toolchains/.bun/bin:$M/.toolchains/node/bin:$PATH\"; printf '%s\\n' '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2024-11-05\",\"capabilities\":{},\"clientInfo\":{\"name\":\"envctl-verify\",\"version\":\"0\"}}}' | timeout 60 \"$M/usr/bin/bunx\" n8n-mcp 2>/dev/null | grep -q 'n8n-documentation-mcp'"]` |
| `setting` | `component[0].verify.args` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export PATH=\"$M/usr/bin:$PATH\"; [ -x \"$M/usr/bin/nix\" ] && [ ! -L \"$M/usr/bin/nix\" ] && grep -Fqx \"exec \\\"$M/.toolchains/nix/bin/nix\\\" \\\"\\$@\\\"\" \"$M/usr/bin/nix\" && nix --version"]` |
| `setting` | `component[0].verify.args` | `manifest/odysseus.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; P=\"$(command -v podman 2>/dev/null || echo \"$M/.toolchains/podman/usr/local/bin/podman\")\"; \"$P\" ps --format '{{.Names}} {{.Ports}}' 2>/dev/null | grep -q odysseus && curl -fsS -o /dev/null --max-time 8 http://127.0.0.1:7000 && ! (\"$P\" ps --format '{{.Ports}}' 2>/dev/null | grep -qE '0\\.0\\.0\\.0:|\\[::\\]:')"]` |
| `setting` | `component[0].verify.args` | `manifest/prompt_hub.toml` | `scope=component source_kind=manifest` | `["-lc","export CARGO_HOME=\"$META_ROOT/.toolchains/cargo\"; export PATH=\"$CARGO_HOME/bin:$PATH\"; prompthub --version >/dev/null && prompthub-server --help >/dev/null"]` |
| `setting` | `component[0].verify.args` | `manifest/rusty-idd.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; B=\"${ENVCTL_BIN_DIR:-$M/usr/bin}\"; export PATH=\"$B:$PATH\"; [ -x \"$B/rusty-idd\" ] && [ ! -L \"$B/rusty-idd\" ] && grep -Fqx \"exec \\\"$M/.toolchains/rusty-idd/bin/rusty-idd\\\" \\\"\\$@\\\"\" \"$B/rusty-idd\" && rusty-idd --version >/dev/null"]` |
| `setting` | `component[0].verify.args` | `manifest/sqld.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"\nexport PATH=\"$M/usr/bin:$PATH\"\n[ -x \"$M/usr/bin/sqld\" ] && [ ! -L \"$M/usr/bin/sqld\" ] && grep -Fqx \"exec \\\"$M/.toolchains/sqld/bin/sqld\\\" \\\"\\$@\\\"\" \"$M/usr/bin/sqld\"\nsqld --version >/dev/null 2>&1 || sqld --help >/dev/null 2>&1 || exit 1\nif systemctl --user is-active --quiet sqld.service; then\n  if curl -fsS http://127.0.0.1:8080/health >/dev/null 2>&1 \\\n     || curl -fsS http://127.0.0.1:8080/ >/dev/null 2>&1; then\n    echo \"sqld serving on 127.0.0.1:8080 (healthy)\"\n  else\n    echo \"sqld.service active but loopback port not answering yet (non-fatal)\"\n  fi\nelse\n  echo \"sqld.service not active (non-fatal; start it to serve)\"\nfi\nexit 0\n"]` |
| `setting` | `component[0].wiring.path_entries` | `manifest/grit.toml` | `scope=component source_kind=manifest` | `["$META_ROOT/.toolchains/cargo/bin"]` |
| `setting` | `component[0].wiring.path_entries` | `manifest/prompt_hub.toml` | `scope=component source_kind=manifest` | `["$META_ROOT/.toolchains/cargo/bin"]` |
| `setting` | `component[0].wiring.systemd_user[0].content` | `manifest/env-ctl.toml` | `scope=component source_kind=manifest` | `[Unit]
Description=env-ctl secrets vault + credential broker
Documentation=https://github.com/FlexNetOS/envctl/blob/master/docs/ARCHITECTURE.md
# No network ordering for the control/data planes (UDS + loopback). Only the relay HTTPS
# edge needs the network, and that is operator-enabled, not a startup dependency.
After=default.target
# Durable store ordering: the loopback sqld must be up first (libSQL remote backend on
# http://127.0.0.1:8080). Wants (not Requires) so a transient sqld blip auto-relocks rather than
# hard-failing the daemon; the libSQL store retries the connection on use.
Wants=sqld.service
After=sqld.service

[Service]
Type=notify
NotifyAccess=main
# Meta-local binary (Epic H TASK-0069). %h=home; META_ROOT is %h/Desktop/meta on this box.
ExecStart=%h/Desktop/meta/usr/libexec/envctl/secrets/bin/secretd
# Cognitum Seed Device-CA pin lives under the meta-local share prefix, NOT /usr/local (TASK-0075b,
# no-system-depth doctrine). seam.rs \`ca_path()\` reads ENVCTL_SEED_CA; cognitum-seed-trust pins here.
Environment=ENVCTL_SEED_CA=%h/Desktop/meta/etc/envctl/secrets/ca/cognitum-ca.crt
Environment=META_ROOT=%h/Desktop/meta
Environment=XDG_CONFIG_HOME=%h/Desktop/meta/.config
Environment=XDG_DATA_HOME=%h/Desktop/meta/.local/share
Environment=XDG_STATE_HOME=%h/Desktop/meta/.local/state
Restart=on-failure
RestartSec=10
# Fail-closed memory hygiene is enforced IN secretd (mlockall + RLIMIT_CORE=0 + MADV_DONTDUMP;
# the daemon refuses to start if mlockall fails — FS-S4). The unit only RAISES the ceiling the
# daemon needs; it never substitutes for the in-process refusal.
LimitMEMLOCK=infinity
LimitCORE=0
# Defense-in-depth sandbox (does not weaken the in-process TCB story):
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=%h/Desktop/meta/.local/share/env-ctl %h/Desktop/meta/.local/state/env-ctl %t/env-ctl
# Belt-and-suspenders read access to the Seed Device-CA dir (TASK-0075b). NOT ReadWritePaths — the
# daemon must never overwrite its trust root; survives any future ProtectHome=tmpfs hardening.
ReadOnlyPaths=%h/Desktop/meta/etc/envctl/secrets/ca
RuntimeDirectory=env-ctl
RuntimeDirectoryMode=0700
PrivateTmp=true
ProtectKernelTunables=true
ProtectControlGroups=true
RestrictSUIDSGID=true
# Pulling the enrolled USB auto-relocks within the drain grace; stop must allow the daemon to
# drain in-flight relays/streams.
TimeoutStopSec=30
KillMode=mixed

[Install]
WantedBy=default.target
` |
| `setting` | `component[10].description` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `yazi terminal file manager (yazi + ya) from the upstream musl release zip into .toolchains/yazi + $META_ROOT/usr/bin/{yazi,ya}. Removes nix as the delivery path for interactive yazi. Repo: sxyazi/yazi.` |
| `setting` | `component[10].detect.args` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/yazi/yazi\" ] && [ -x \"$M/.toolchains/yazi/ya\" ] && [ -x \"$META_ROOT/usr/bin/yazi\" ] && [ ! -L \"$META_ROOT/usr/bin/yazi\" ] && grep -Fqx \"exec \\\"$M/.toolchains/yazi/yazi\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/yazi\" && [ -x \"$META_ROOT/usr/bin/ya\" ] && [ ! -L \"$META_ROOT/usr/bin/ya\" ] && grep -Fqx \"exec \\\"$M/.toolchains/yazi/ya\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/ya\""]` |
| `setting` | `component[10].install.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
ROOT="$M/envctl"; source "$ROOT/assets/scripts/envctl-gh-fetch.sh"
DEST="$M/.toolchains/yazi"
ASSET="yazi-x86_64-unknown-linux-musl.zip"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
# Authenticated fetch via the shared resolver (gh tier = 5000/hr or the vault-sealed mint token;
# gh is meta-owned Epic-H TASK-0057) to dodge the 60/hr unauth GitHub API 403. When gh is authed
# the resolver uses \`gh release download\`; otherwise we keep this component's OWN /releases/latest
# redirect + curl fallback, now bearer-tokened via envctl_gh_curl when a token is available.
if _envctl_gh_authed; then
  envctl_gh_release_download --repo sxyazi/yazi --pattern "$ASSET" --output "$tmp/yazi.zip"
else
  TAG="$(curl -fsSLI -o /dev/null -w '%{url_effective}' 'https://github.com/sxyazi/yazi/releases/latest' | sed 's#.*/tag/##')"
  envctl_gh_curl "https://github.com/sxyazi/yazi/releases/download/${TAG}/${ASSET}" -o "$tmp/yazi.zip"
fi
# zip extraction: unzip if present, else python3's stdlib zipfile (no system dep). Exec bit is
# re-applied by \`install -m 755\` below regardless of what the extractor preserves.
if command -v unzip >/dev/null 2>&1; then unzip -q -o "$tmp/yazi.zip" -d "$tmp"; else python3 -m zipfile -e "$tmp/yazi.zip" "$tmp"; fi
SRC="$(dirname "$(find "$tmp" -type f -name yazi | head -1)")"
install -d -m 755 "$DEST"
install -m 755 "$SRC/yazi" "$DEST/yazi"
install -m 755 "$SRC/ya" "$DEST/ya"
install -d -m 755 "$META_ROOT/usr/bin"
envctl_frontdoor "$DEST/yazi" "$META_ROOT/usr/bin/yazi"
envctl_frontdoor "$DEST/ya" "$META_ROOT/usr/bin/ya"


` |
| `setting` | `component[10].remove.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `set -u
M="${META_ROOT:?META_ROOT required}"
for b in yazi ya; do t="$META_ROOT/usr/bin/$b"; [ -L "$t" ] && readlink "$t" | grep -q "$M/.toolchains/yazi" && rm -f "$t"; done
rm -rf "$M/.toolchains/yazi"
` |
| `setting` | `component[11].description` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `helix modal editor (hx) + bundled tree-sitter runtime from the upstream release tarball into .toolchains/helix + $META_ROOT/usr/bin/hx. Removes nix as the delivery path for interactive helix. Repo: helix-editor/helix.` |
| `setting` | `component[11].detect.args` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/helix/hx\" ] && [ -d \"$M/.toolchains/helix/runtime\" ] && [ -x \"$META_ROOT/usr/bin/hx\" ] && [ ! -L \"$META_ROOT/usr/bin/hx\" ] && grep -Fqx \"exec \\\"$M/.toolchains/helix/hx\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/hx\""]` |
| `setting` | `component[11].install.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
ROOT="$M/envctl"; source "$ROOT/assets/scripts/envctl-gh-fetch.sh"
DEST="$M/.toolchains/helix"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
# Authenticated fetch via the shared resolver (gh tier = 5000/hr or the vault-sealed mint token)
# to dodge the 60/hr unauth GitHub API 403. The gh --pattern is a version-agnostic GLOB (the
# asset name embeds the version), so this branch keeps \`gh release download\` directly through the
# resolver; the unauth fallback builds the concrete URL and is bearer-tokened via envctl_gh_curl.
if _envctl_gh_authed; then
  envctl_gh_release_download --repo helix-editor/helix --pattern 'helix-*-x86_64-linux.tar.xz' --output "$tmp/helix.tar.xz"
else
  TAG="$(curl -fsSLI -o /dev/null -w '%{url_effective}' 'https://github.com/helix-editor/helix/releases/latest' | sed 's#.*/tag/##')"
  envctl_gh_curl "https://github.com/helix-editor/helix/releases/download/${TAG}/helix-${TAG}-x86_64-linux.tar.xz" -o "$tmp/helix.tar.xz"
fi
tar -xJf "$tmp/helix.tar.xz" -C "$tmp"
SRC="$(dirname "$(find "$tmp" -type f -name hx | head -1)")"
rm -rf "$DEST"; install -d -m 755 "$DEST"
install -m 755 "$SRC/hx" "$DEST/hx"
cp -a "$SRC/runtime" "$DEST/runtime"
install -d -m 755 "$META_ROOT/usr/bin"
envctl_frontdoor "$DEST/hx" "$META_ROOT/usr/bin/hx"


` |
| `setting` | `component[11].remove.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `set -u
M="${META_ROOT:?META_ROOT required}"
t="$META_ROOT/usr/bin/hx"; [ -L "$t" ] && readlink "$t" | grep -q "$M/.toolchains/helix" && rm -f "$t"
rm -rf "$M/.toolchains/helix"
` |
| `setting` | `component[11].verify.args` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; PATH=\"$META_ROOT/usr/bin:$PATH\"; export HELIX_RUNTIME=\"$M/.toolchains/helix/runtime\"; hx --version"]` |
| `setting` | `component[12].description` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `huggingface_hub CLI in an isolated venv at .toolchains/huggingface, exposed as $META_ROOT/usr/bin/huggingface-cli (wrapped to the WORKING \`hf\` entry point; never \`hf\`, which collides with the handoff kernel). Repo: huggingface/huggingface_hub.` |
| `setting` | `component[12].detect.args` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/huggingface/bin/hf\" ] && [ -x \"$META_ROOT/usr/bin/huggingface-cli\" ] && [ ! -L \"$META_ROOT/usr/bin/huggingface-cli\" ] && grep -Fqx \"exec \\\"$M/.toolchains/huggingface/bin/hf\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/huggingface-cli\""]` |
| `setting` | `component[12].install.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
DEST="$M/.toolchains/huggingface"
python3 -m venv "$DEST"
"$DEST/bin/pip" install --quiet --upgrade pip
"$DEST/bin/pip" install --quiet --upgrade huggingface_hub
install -d -m 755 "$META_ROOT/usr/bin"
# Expose the WORKING \`hf\` entry point under the classic, non-colliding name. The classic
# \`huggingface-cli\` binary is a dead deprecation stub in huggingface_hub 1.x; \`hf\` itself is
# NEVER symlinked (it collides with $META_ROOT/usr/bin/hf, the handoff kernel).
envctl_frontdoor "$DEST/bin/hf" "$META_ROOT/usr/bin/huggingface-cli"


` |
| `setting` | `component[12].remove.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `set -u
M="${META_ROOT:?META_ROOT required}"
DEST="$M/.toolchains/huggingface"
t="$META_ROOT/usr/bin/huggingface-cli"; [ -L "$t" ] && readlink "$t" | grep -q "$M/.toolchains/huggingface" && rm -f "$t"
rm -rf "$DEST"
` |
| `setting` | `component[1].description` | `manifest/apt-base.toml` | `scope=component source_kind=manifest` | `Rootless container stack from the static podman release, installed under $META_ROOT/.toolchains/podman. $META_ROOT/usr/bin/podman is a regular executable frontdoor wrapper for the meta-hosted podman binary; helper binaries and containers config are also meta-hosted.` |
| `setting` | `component[1].description` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `Fast Rust linker (davidlattimore/wild) via \`cargo install --locked wild-linker\` into .toolchains/wild, wired as the local cargo linker via a delimited block in the meta-root .cargo/config.toml (clang + --ld-path=wild). Owner-preferred over mold. Wiring is verified — a full cargo build links through wild before it lands; the config is co-managed with kache (block-upsert) and remove strips only the wild block.` |
| `setting` | `component[1].description` | `manifest/components.d/meta-core-clis.toml` | `scope=component source_kind=manifest` | `Static curl binary installed under $META_ROOT/.toolchains/curl. $META_ROOT/usr/bin/curl is a regular executable frontdoor wrapper for the meta-hosted curl binary.` |
| `setting` | `component[1].description` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `CUDA toolkit (nvcc, cuBLAS/cuDNN, nsys/ncu) from NVIDIA's runfile into .toolchains/cuda (toolkit-only, no-sudo, no-driver). Replaces the apt cuda-toolkit-13-3 system-depth install. Owns the cuda env shell block (meta-prefix first, apt fallback).` |
| `setting` | `component[1].description` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `Appends eval-cores + yazelix substituter to /etc/nix/nix.custom.conf.` |
| `setting` | `component[1].detect.args` | `manifest/ai-clis.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; VER=\"${CODEX_VERSION:-0.142.3}\"; vdir=\"$M/.toolchains/openai-codex/${VER}\"; dest=\"$vdir/bin/codex\"; cur=\"$M/.toolchains/openai-codex/current/bin/codex\"; link=\"$M/usr/bin/codex\"; meta_compat=\"$M/.local/bin/codex\"; real_compat=\"${ENVCTL_REAL_HOME:-}/.local/bin/codex\"; config=\"$M/.local/share/codex/config.toml\"; [ -x \"$dest\" ] && [ \"$(readlink -f \"$cur\" 2>/dev/null)\" = \"$(readlink -f \"$dest\" 2>/dev/null)\" ] && [ -x \"$link\" ] && grep -q \"envctl codex wrapper\" \"$link\" && [ \"$(readlink -f \"$meta_compat\" 2>/dev/null)\" = \"$(readlink -f \"$link\" 2>/dev/null)\" ] && { [ -z \"${ENVCTL_REAL_HOME:-}\" ] || [ \"$(readlink -f \"$real_compat\" 2>/dev/null)\" = \"$(readlink -f \"$link\" 2>/dev/null)\" ]; } && CODEX_HOME= CODEX_SQLITE_HOME= \"$link\" --version >/dev/null && CODEX_HOME= CODEX_SQLITE_HOME= \"$link\" mcp list >/dev/null && grep -q 'CODEX_BIN_PATH=.*openai-codex/current/bin/codex' \"$link\" && [ ! -e \"$M/.toolchains/bun/bin/codex\" ] && [ ! -L \"$M/.toolchains/bun/bin/codex\" ] && [ ! -e \"$M/.toolchains/.bun/bin/codex\" ] && [ ! -L \"$M/.toolchains/.bun/bin/codex\" ] && [ ! -e \"$M/.toolchains/bun/install/global/node_modules/.bin/codex\" ] && [ ! -L \"$M/.toolchains/bun/install/global/node_modules/.bin/codex\" ] && [ ! -e \"$M/.toolchains/.bun/install/global/node_modules/.bin/codex\" ] && [ ! -L \"$M/.toolchains/.bun/install/global/node_modules/.bin/codex\" ] && ! grep -Rqs '\"@openai/codex\"' \"$M/.toolchains/bun/install/global/package.json\" \"$M/.toolchains/bun/install/global/bun.lock\" \"$M/.toolchains/.bun/install/global/package.json\" \"$M/.toolchains/.bun/install/global/bun.lock\" 2>/dev/null && ! find \"$M/.toolchains/bun\" \"$M/.toolchains/.bun\" -path '*/@openai/codex*' -print -quit 2>/dev/null | grep -q . && { [ ! -f \"$config\" ] || ! awk 'BEGIN{s=0;bad=0} /^\\[/{s=($0==\"[hooks.state]\")} s && /^[[:space:]]*enabled[[:space:]]*=/{bad=1} END{exit bad?0:1}' \"$config\"; } && ! grep -RqsE '/home/[^[:space:]]+/(\\.local/bin/icm|\\.cargo/bin/weave)|/usr/local/bin/vox' \"$M/.local/share/codex/config.toml\" \"$M/.local/share/codex/hooks.json\" 2>/dev/null && \"$M/envctl/assets/scripts/envctl-codex-cleanup.sh\" verify"]` |
| `setting` | `component[1].detect.args` | `manifest/apt-base.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; B=\"$M/.toolchains/podman/usr/local/bin/podman\"; [ -x \"$B\" ] || exit 1; for c in \"$M/usr/bin/podman\" \"$M/.local/bin/podman\"; do [ -x \"$c\" ] && \"$c\" --version >/dev/null 2>&1 && exit 0; done; exit 1"]` |
| `setting` | `component[1].detect.args` | `manifest/base.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/.bun/bin/bun\" ] && [ -x \"$META_ROOT/usr/bin/bun\" ] && [ ! -L \"$META_ROOT/usr/bin/bun\" ] && grep -Fqx \"exec \\\"$M/.toolchains/.bun/bin/bun\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/bun\" && [ -x \"$META_ROOT/usr/bin/bunx\" ] && grep -q 'envctl bunx wrapper' \"$META_ROOT/usr/bin/bunx\""]` |
| `setting` | `component[1].detect.args` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/wild/bin/wild\" ] && [ -x \"$META_ROOT/usr/bin/wild\" ] && [ ! -L \"$META_ROOT/usr/bin/wild\" ] && grep -Fqx \"exec \\\"$M/.toolchains/wild/bin/wild\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/wild\" && [ -f \"$M/.cargo/config.toml\" ] && grep -q -- '--ld-path=wild' \"$M/.cargo/config.toml\""]` |
| `setting` | `component[1].detect.args` | `manifest/components.d/meta-core-clis.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/curl/bin/curl\" ] && [ -x \"$META_ROOT/usr/bin/curl\" ] && [ ! -L \"$META_ROOT/usr/bin/curl\" ] && grep -Fqx \"exec \\\"$M/.toolchains/curl/bin/curl\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/curl\""]` |
| `setting` | `component[1].detect.args` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/.bun/bin/vite\" ] && [ -x \"$META_ROOT/usr/bin/vite\" ] && [ ! -L \"$META_ROOT/usr/bin/vite\" ] && grep -Fqx \"exec \\\"$M/.toolchains/.bun/bin/vite\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/vite\""]` |
| `setting` | `component[1].detect.args` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/cuda/bin/nvcc\" ]"]` |
| `setting` | `component[1].detect.args` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `["-lc","grep -q 'yazelix.cachix.org' /etc/nix/nix.custom.conf 2>/dev/null"]` |
| `setting` | `component[1].fix.script` | `manifest/ai-clis.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
ROOT="$M/envctl"; source "$ROOT/assets/scripts/envctl-gh-fetch.sh"
VER="${CODEX_VERSION:-0.142.3}"
BASE="$M/.toolchains/openai-codex"
CUR="$BASE/current"
LINK="$M/usr/bin/codex"
if [ ! -x "$CUR/bin/codex" ]; then
  tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
  URL="https://github.com/openai/codex/releases/download/rust-v${VER}/codex-x86_64-unknown-linux-musl.tar.gz"
  envctl_gh_curl "$URL" -o "$tmp/codex.tgz"
  tar -xzf "$tmp/codex.tgz" -C "$tmp"
  src="$(find "$tmp" -type f \( -name codex -o -name 'codex-*-linux-musl' \) | head -1)"
  [ -n "$src" ] && [ -f "$src" ] || { echo "codex: no binary found in release tarball" >&2; exit 1; }
  VDIR="$BASE/${VER}"
  install -d -m 755 "$VDIR/bin"
  install -m 755 "$src" "$VDIR/bin/codex"
  ln -sfn "$VER" "$CUR"
fi
install -d -m 755 "$M/usr/bin"
if [ -e "$LINK" ] && ! grep -q "envctl codex wrapper" "$LINK" 2>/dev/null; then
  ARCH="$M/var/lib/envctl/legacy-archives/usr-bin-$(date -u +%Y-%m-%d)/usr/bin"
  install -d -m 755 "$ARCH"; mv "$LINK" "$ARCH/codex"; echo "archived previous codex front door: $ARCH/codex"
fi
cat >"$LINK" <<'WRAPPER'
#!/usr/bin/env bash
# envctl codex wrapper
set -euo pipefail
self="${BASH_SOURCE[0]}"
while [ -L "$self" ]; do
  dir="$(cd -P "$(dirname "$self")" && pwd)"
  target="$(readlink "$self")"
  case "$target" in /*) self="$target" ;; *) self="$dir/$target" ;; esac
done
bindir="$(cd -P "$(dirname "$self")" && pwd)"
META_ROOT="$(cd "$bindir/../.." && pwd)"
export META_ROOT
export CODEX_HOME="${CODEX_HOME:-$META_ROOT/.local/share/codex}"
export CODEX_SQLITE_HOME="${CODEX_SQLITE_HOME:-$META_ROOT/.local/state/codex}"
# Codex must resolve to the Rust CLI, never the legacy Bun-installed JS shim.
export CODEX_BIN_PATH="${CODEX_BIN_PATH:-$META_ROOT/.toolchains/openai-codex/current/bin/codex}"
export CODEX_CLI_BIN="$CODEX_BIN_PATH"
# Keep meta shims first. The Bun tool-bin directory remains available for non-Codex
# tools such as gemini/vite, but the old @openai/codex Bun package is not on this path.
export PATH="$META_ROOT/usr/bin:$META_ROOT/.local/bin:$META_ROOT/.toolchains/cargo/bin:$META_ROOT/.toolchains/.bun/bin:$META_ROOT/.toolchains/node/bin:$PATH"
umask 077
mkdir -p "$CODEX_HOME" "$CODEX_SQLITE_HOME"
exec "$META_ROOT/.toolchains/openai-codex/current/bin/codex" "$@"
WRAPPER
chmod 755 "$LINK"
install -d -m 755 "$M/.local/bin"
ln -sfn "$LINK" "$M/.local/bin/codex"
if [ -n "${ENVCTL_REAL_HOME:-}" ]; then
  real_link="$ENVCTL_REAL_HOME/.local/bin/codex"
  install -d -m 755 "$(dirname "$real_link")"
  if [ -e "$real_link" ] && [ ! -L "$real_link" ]; then
    ARCH="$M/var/lib/envctl/legacy-archives/real-home-bin-$(date -u +%Y-%m-%d)/.local/bin"
    install -d -m 755 "$ARCH"
    mv "$real_link" "$ARCH/codex"
    echo "archived previous real-home codex front door: $ARCH/codex"
  fi
  ln -sfn "$LINK" "$real_link"
fi
install -d -m 700 "$M/.local/share/codex" "$M/.local/state/codex"
if [ -n "${ENVCTL_REAL_HOME:-}" ] && [ -d "$ENVCTL_REAL_HOME/.codex" ]; then
  tar -C "$ENVCTL_REAL_HOME/.codex" -cf - . | tar -C "$M/.local/share/codex" --keep-old-files -xf - 2>/dev/null || true
fi
python3 - <<PYCONF
from pathlib import Path
import json, re
home = Path("$M/.local/share/codex")
config = home / "config.toml"
if config.exists():
    text = config.read_text()
    text = text.replace("/home/drdave/.cargo/bin/weave", "weave")
    text = text.replace("/home/drdave/.local/bin/icm", "icm")
    text = text.replace("/usr/local/bin/vox", "vox")
    text = text.replace('command = "npx"', 'command = "bunx"')
    text = re.sub(r'model_catalog_json = ".*?/model-catalog\.json"', f'model_catalog_json = "{home}/model-catalog.json"', text)
    text = re.sub(r'\n\[hooks\.state\](?:\n\n?\[hooks\.state\."[^"]+"\]\ntrusted_hash = "[^"]*")+', '\n[hooks.state]\n', text)
    lines = []
    in_hooks_state = False
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            in_hooks_state = stripped == "[hooks.state]"
        if in_hooks_state and re.match(r"^\s*enabled\s*=", line):
            continue
        lines.append(line)
    text = "\n".join(lines) + ("\n" if text.endswith("\n") else "")
    config.write_text(text)
hooks = home / "hooks.json"
if hooks.exists():
    data = json.loads(hooks.read_text())
    for entries in data.get("hooks", {}).values():
        for entry in entries:
            for hook in entry.get("hooks", []):
                if isinstance(hook.get("command"), str):
                    hook["command"] = hook["command"].replace("/home/drdave/.local/bin/icm", "icm")
    hooks.write_text(json.dumps(data, indent=2) + "\n")
PYCONF
"$LINK" --version >/dev/null
CODEX_HOME= CODEX_SQLITE_HOME= "$LINK" mcp list >/dev/null
"$M/envctl/assets/scripts/envctl-codex-cleanup.sh" clean
"$M/envctl/assets/scripts/envctl-codex-cleanup.sh" verify
` |
| `setting` | `component[1].fix.script` | `manifest/apt-base.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
DEST="$M/.toolchains/podman"
VER="${PODMAN_STATIC_VERSION:-v5.8.3}"
ARCH="$(uname -m)"; case "$ARCH" in x86_64) A=amd64;; aarch64) A=arm64;; *) echo "unsupported arch: $ARCH"; exit 1;; esac
URL="https://github.com/mgoltzsche/podman-static/releases/download/$VER/podman-linux-$A.tar.gz"
cache="$M/.cache/podman"; install -d -m 755 "$cache"
tarball="$cache/podman-linux-$A-$VER.tar.gz"
[ -s "$tarball" ] || curl -fSL "$URL" -o "$tarball"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
tar -xzf "$tarball" -C "$tmp"
# Re-extract the binary/config tree ONLY. Do NOT \`rm -rf "$DEST"\` — the container store
# (graphroot/runroot/rootless-storage, set under $DEST in storage.conf below) lives here; wiping
# it on a reinstall destroys every image/container (it once nuked a live stack). Removing just
# usr/ + etc/ refreshes the payload while preserving the store; cp -a overlays the new binaries.
rm -rf "$DEST/usr" "$DEST/etc"; install -d -m 755 "$DEST"
cp -a "$tmp/podman-linux-$A/." "$DEST/"
install -d -m 755 "$DEST/etc/containers" "$META_ROOT/.config/containers" "$META_ROOT/usr/bin"
cat > "$DEST/etc/containers/containers.conf" <<EOF
[engine]
cgroup_manager = "cgroupfs"
events_logger = "file"
conmon_path = ["$DEST/usr/local/lib/podman/conmon"]
helper_binaries_dir = ["$DEST/usr/local/lib/podman", "$DEST/usr/local/libexec/podman", "$DEST/usr/local/bin"]
network_cmd_path = "$DEST/usr/local/bin/pasta"
runtime = "crun"

[engine.runtimes]
crun = ["$DEST/usr/local/bin/crun"]
runc = ["$DEST/usr/local/bin/runc"]
EOF
cat > "$DEST/etc/containers/storage.conf" <<EOF
[storage]
driver = "overlay"
runroot = "$M/.toolchains/podman/runroot"
graphroot = "$M/.toolchains/podman/graphroot"
rootless_storage_path = "$M/.toolchains/podman/rootless-storage"

[storage.options.overlay]
ignore_chown_errors = "true"
mount_program = "$DEST/usr/local/bin/fuse-overlayfs"
mountopt = "nodev,fsync=0"
EOF
for f in containers.conf storage.conf registries.conf policy.json; do
  [ -f "$DEST/etc/containers/$f" ] && ln -sfn "$DEST/etc/containers/$f" "$META_ROOT/.config/containers/$f"
done
envctl_frontdoor "$DEST/usr/local/bin/podman" "$META_ROOT/usr/bin/podman"
for b in crun runc fuse-overlayfs fusermount3 pasta; do
  [ -x "$DEST/usr/local/bin/$b" ] && envctl_frontdoor "$DEST/usr/local/bin/$b" "$META_ROOT/usr/bin/$b"
done

` |
| `setting` | `component[1].fix.script` | `manifest/base.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -e
M="${META_ROOT:?META_ROOT required}"
export BUN_INSTALL="$M/.toolchains/.bun"
mkdir -p "$BUN_INSTALL" "$META_ROOT/usr/bin"
curl -fsSL https://bun.sh/install | bash
envctl_frontdoor "$BUN_INSTALL/bin/bun" "$META_ROOT/usr/bin/bun"
cat >"$META_ROOT/usr/bin/bunx" <<'BUNX'
#!/usr/bin/env bash
# envctl bunx wrapper
set -euo pipefail
self="${BASH_SOURCE[0]}"
while [ -L "$self" ]; do
  dir="$(cd -P "$(dirname "$self")" && pwd)"
  target="$(readlink "$self")"
  case "$target" in
    /*) self="$target" ;;
    *) self="$dir/$target" ;;
  esac
done
bindir="$(cd -P "$(dirname "$self")" && pwd)"
META_ROOT="$(cd "$bindir/../.." && pwd)"
export META_ROOT
export BUN_INSTALL="${BUN_INSTALL:-$META_ROOT/.toolchains/.bun}"
export HOME="${HOME:-$META_ROOT/.local}"
export XDG_CACHE_HOME="${XDG_CACHE_HOME:-$META_ROOT/.local/cache}"
exec "$BUN_INSTALL/bin/bun" x "$@"
BUNX
chmod 755 "$META_ROOT/usr/bin/bunx"

` |
| `setting` | `component[1].fix.script` | `manifest/components.d/meta-core-clis.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
VER="${CURL_VERSION:-8.21.0}"
DEST="$M/.toolchains/curl"
ARCH="$(uname -m)"; case "$ARCH" in x86_64) A=x86_64;; aarch64) A=aarch64;; *) echo "unsupported arch: $ARCH"; exit 1;; esac
URL="https://github.com/stunnel/static-curl/releases/download/$VER/curl-linux-$A-musl-$VER.tar.xz"
cache="$M/.cache/curl"; install -d -m 755 "$cache" "$DEST/bin" "$META_ROOT/usr/bin"
tarball="$cache/curl-linux-$A-musl-$VER.tar.xz"
[ -s "$tarball" ] || /usr/bin/curl -fSL "$URL" -o "$tarball"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
tar -xJf "$tarball" -C "$tmp"
src="$(find "$tmp" -type f -name curl -perm -111 | head -n1)"
install -m 755 "$src" "$DEST/bin/curl"
envctl_frontdoor "$DEST/bin/curl" "$META_ROOT/usr/bin/curl"

` |
| `setting` | `component[1].fix.script` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -e
M="${META_ROOT:?META_ROOT required}"
export BUN_INSTALL="$M/.toolchains/.bun"
export PATH="$BUN_INSTALL/bin:$PATH"
bun add -g vite
mkdir -p "$META_ROOT/usr/bin"
envctl_frontdoor "$BUN_INSTALL/bin/vite" "$META_ROOT/usr/bin/vite"

` |
| `setting` | `component[1].fix.script` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
DEST="$M/.toolchains/cuda"
VER="13.3.0"; DRV="610.43.02"
MD5="16d68669cf659157777d2e7adaff179d"
RUN="cuda_${VER}_${DRV}_linux.run"
URL="https://developer.download.nvidia.com/compute/cuda/${VER}/local_installers/${RUN}"
cache="$M/.cache/cuda"; install -d -m 755 "$cache"
if ! ( [ -s "$cache/$RUN" ] && echo "${MD5}  $cache/$RUN" | md5sum -c - >/dev/null 2>&1 ); then
  curl -fSL "$URL" -o "$cache/$RUN"
  echo "${MD5}  $cache/$RUN" | md5sum -c -
fi
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
rm -rf "$DEST"; install -d -m 755 "$DEST"
sh "$cache/$RUN" --silent --toolkit --toolkitpath="$DEST" --override --no-opengl-libs --no-man-page --nox11 --tmpdir="$tmp" </dev/null
install -d -m 755 "$META_ROOT/usr/bin"
for t in nvcc nsys ncu cuda-gdb compute-sanitizer cuobjdump; do
  [ -x "$DEST/bin/$t" ] && envctl_frontdoor "$DEST/bin/$t" "$META_ROOT/usr/bin/$t"
done
if [ -x /usr/bin/nvidia-smi ]; then
  install -m 755 /usr/bin/nvidia-smi "$DEST/bin/nvidia-smi"
  envctl_frontdoor "$DEST/bin/nvidia-smi" "$META_ROOT/usr/bin/nvidia-smi"
fi

` |
| `setting` | `component[1].fix.script` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `set -e
add(){ grep -qF -- "$1" /etc/nix/nix.custom.conf || echo "$1" | sudo tee -a /etc/nix/nix.custom.conf >/dev/null; }
add "extra-substituters = https://yazelix.cachix.org"
sudo systemctl restart nix-daemon 2>/dev/null || true
` |
| `setting` | `component[1].id` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `wild-linker` |
| `setting` | `component[1].id` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `nix-yazelix-cache` |
| `setting` | `component[1].install.script` | `manifest/ai-clis.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
ROOT="$M/envctl"; source "$ROOT/assets/scripts/envctl-gh-fetch.sh"
VER="${CODEX_VERSION:-0.142.3}"
BASE="$M/.toolchains/openai-codex"
CUR="$BASE/current"
LINK="$M/usr/bin/codex"
if [ "${CODEX_BUILD_FROM_SOURCE:-0}" = "1" ] && [ -d "$M/codex/codex-rs/cli" ]; then
  export CARGO_HOME="$M/.toolchains/cargo"
  export PATH="$CARGO_HOME/bin:$M/.toolchains/cargo/bin:$PATH"
  jobs="${CODEX_CARGO_JOBS:-$(nproc --all)}"
  export CARGO_BUILD_JOBS="$jobs"
  export CARGO_PROFILE_RELEASE_LTO="${CARGO_PROFILE_RELEASE_LTO:-false}"
  export CARGO_PROFILE_RELEASE_CODEGEN_UNITS="${CARGO_PROFILE_RELEASE_CODEGEN_UNITS:-$jobs}"
  export CARGO_PROFILE_RELEASE_INCREMENTAL="${CARGO_PROFILE_RELEASE_INCREMENTAL:-true}"
  cargo build --release -p codex-cli --jobs "$jobs" --timings --manifest-path "$M/codex/codex-rs/Cargo.toml"
  VER="src-$(git -C "$M/codex" rev-parse --short HEAD 2>/dev/null || echo unknown)"
  src="$M/codex/codex-rs/target/release/codex"
else
  tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
  URL="https://github.com/openai/codex/releases/download/rust-v${VER}/codex-x86_64-unknown-linux-musl.tar.gz"
  envctl_gh_curl "$URL" -o "$tmp/codex.tgz"
  tar -xzf "$tmp/codex.tgz" -C "$tmp"
  src="$(find "$tmp" -type f \( -name codex -o -name 'codex-*-linux-musl' \) | head -1)"
  [ -n "$src" ] && [ -f "$src" ] || { echo "codex: no binary found in release tarball" >&2; exit 1; }
fi
VDIR="$BASE/${VER}"
install -d -m 755 "$VDIR/bin"
install -m 755 "$src" "$VDIR/bin/codex"
ln -sfn "$VER" "$CUR"
install -d -m 755 "$M/usr/bin"
if [ -e "$LINK" ] && ! grep -q "envctl codex wrapper" "$LINK" 2>/dev/null; then
  ARCH="$M/var/lib/envctl/legacy-archives/usr-bin-$(date -u +%Y-%m-%d)/usr/bin"
  install -d -m 755 "$ARCH"
  mv "$LINK" "$ARCH/codex"
  echo "archived previous codex front door: $ARCH/codex"
fi
cat >"$LINK" <<'WRAPPER'
#!/usr/bin/env bash
# envctl codex wrapper
set -euo pipefail
self="${BASH_SOURCE[0]}"
while [ -L "$self" ]; do
  dir="$(cd -P "$(dirname "$self")" && pwd)"
  target="$(readlink "$self")"
  case "$target" in
    /*) self="$target" ;;
    *) self="$dir/$target" ;;
  esac
done
bindir="$(cd -P "$(dirname "$self")" && pwd)"
META_ROOT="$(cd "$bindir/../.." && pwd)"
export META_ROOT
export CODEX_HOME="${CODEX_HOME:-$META_ROOT/.local/share/codex}"
export CODEX_SQLITE_HOME="${CODEX_SQLITE_HOME:-$META_ROOT/.local/state/codex}"
# Codex must resolve to the Rust CLI, never the legacy Bun-installed JS shim.
export CODEX_BIN_PATH="${CODEX_BIN_PATH:-$META_ROOT/.toolchains/openai-codex/current/bin/codex}"
export CODEX_CLI_BIN="$CODEX_BIN_PATH"
# Keep meta shims first. The Bun tool-bin directory remains available for non-Codex
# tools such as gemini/vite, but the old @openai/codex Bun package is not on this path.
export PATH="$META_ROOT/usr/bin:$META_ROOT/.local/bin:$META_ROOT/.toolchains/cargo/bin:$META_ROOT/.toolchains/.bun/bin:$META_ROOT/.toolchains/node/bin:$PATH"
umask 077
mkdir -p "$CODEX_HOME" "$CODEX_SQLITE_HOME"
exec "$META_ROOT/.toolchains/openai-codex/current/bin/codex" "$@"
WRAPPER
chmod 755 "$LINK"
install -d -m 755 "$M/.local/bin"
ln -sfn "$LINK" "$M/.local/bin/codex"
if [ -n "${ENVCTL_REAL_HOME:-}" ]; then
  real_link="$ENVCTL_REAL_HOME/.local/bin/codex"
  install -d -m 755 "$(dirname "$real_link")"
  if [ -e "$real_link" ] && [ ! -L "$real_link" ]; then
    ARCH="$M/var/lib/envctl/legacy-archives/real-home-bin-$(date -u +%Y-%m-%d)/.local/bin"
    install -d -m 755 "$ARCH"
    mv "$real_link" "$ARCH/codex"
    echo "archived previous real-home codex front door: $ARCH/codex"
  fi
  ln -sfn "$LINK" "$real_link"
fi
install -d -m 700 "$M/.local/share/codex" "$M/.local/state/codex"
if [ -n "${ENVCTL_REAL_HOME:-}" ] && [ -d "$ENVCTL_REAL_HOME/.codex" ]; then
  tar -C "$ENVCTL_REAL_HOME/.codex" -cf - . | tar -C "$M/.local/share/codex" --keep-old-files -xf - 2>/dev/null || true
fi
python3 - <<PYCONF
from pathlib import Path
import json, re
home = Path("$M/.local/share/codex")
config = home / "config.toml"
if config.exists():
    text = config.read_text()
    text = text.replace("/home/drdave/.cargo/bin/weave", "weave")
    text = text.replace("/home/drdave/.local/bin/icm", "icm")
    text = text.replace("/usr/local/bin/vox", "vox")
    text = text.replace('command = "npx"', 'command = "bunx"')
    text = re.sub(r'model_catalog_json = ".*?/model-catalog\.json"', f'model_catalog_json = "{home}/model-catalog.json"', text)
    text = re.sub(r'\n\[hooks\.state\](?:\n\n?\[hooks\.state\."[^"]+"\]\ntrusted_hash = "[^"]*")+', '\n[hooks.state]\n', text)
    lines = []
    in_hooks_state = False
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            in_hooks_state = stripped == "[hooks.state]"
        if in_hooks_state and re.match(r"^\s*enabled\s*=", line):
            continue
        lines.append(line)
    text = "\n".join(lines) + ("\n" if text.endswith("\n") else "")
    config.write_text(text)
hooks = home / "hooks.json"
if hooks.exists():
    data = json.loads(hooks.read_text())
    for entries in data.get("hooks", {}).values():
        for entry in entries:
            for hook in entry.get("hooks", []):
                if isinstance(hook.get("command"), str):
                    hook["command"] = hook["command"].replace("/home/drdave/.local/bin/icm", "icm")
    hooks.write_text(json.dumps(data, indent=2) + "\n")
PYCONF
if "$VDIR/bin/codex" --version >/dev/null 2>&1 && "$LINK" --version >/dev/null 2>&1 && CODEX_HOME= CODEX_SQLITE_HOME= "$LINK" mcp list >/dev/null 2>&1; then
  if [ -d "$M/.toolchains/codex" ]; then
    FLATARCH="$M/var/lib/envctl/legacy-archives/codex-flat-$(date -u +%Y-%m-%d)"
    install -d -m 755 "$FLATARCH"
    mv "$M/.toolchains/codex" "$FLATARCH/codex"
    echo "archived legacy flat install: .toolchains/codex -> $FLATARCH/codex"
  fi
  for BUN_INSTALL in "$M/.toolchains/.bun" "$M/.toolchains/bun"; do
    [ -x "$BUN_INSTALL/bin/bun" ] && PATH="$BUN_INSTALL/bin:$PATH" timeout --kill-after=2s 20s bun remove -g @openai/codex >/dev/null 2>&1 || true
    rm -f "$BUN_INSTALL/bin/codex" "$BUN_INSTALL/install/global/node_modules/.bin/codex"
    rm -rf "$BUN_INSTALL/install/cache/@openai/codex" "$BUN_INSTALL/install/cache/@openai/codex"* \
      "$BUN_INSTALL/install/global/node_modules/@openai/codex" "$BUN_INSTALL/install/global/node_modules/@openai/codex"*
  done
  "$M/envctl/assets/scripts/envctl-codex-cleanup.sh" clean
  "$M/envctl/assets/scripts/envctl-codex-cleanup.sh" verify
else
  echo "codex: new pinned binary, wrapper, or MCP config failed verification; legacy install left in place" >&2
  exit 1
fi
` |
| `setting` | `component[1].install.script` | `manifest/apt-base.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
DEST="$M/.toolchains/podman"
VER="${PODMAN_STATIC_VERSION:-v5.8.3}"
ARCH="$(uname -m)"; case "$ARCH" in x86_64) A=amd64;; aarch64) A=arm64;; *) echo "unsupported arch: $ARCH"; exit 1;; esac
URL="https://github.com/mgoltzsche/podman-static/releases/download/$VER/podman-linux-$A.tar.gz"
cache="$M/.cache/podman"; install -d -m 755 "$cache"
tarball="$cache/podman-linux-$A-$VER.tar.gz"
[ -s "$tarball" ] || curl -fSL "$URL" -o "$tarball"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
tar -xzf "$tarball" -C "$tmp"
# Re-extract the binary/config tree ONLY. Do NOT \`rm -rf "$DEST"\` — the container store
# (graphroot/runroot/rootless-storage, set under $DEST in storage.conf below) lives here; wiping
# it on a reinstall destroys every image/container (it once nuked a live stack). Removing just
# usr/ + etc/ refreshes the payload while preserving the store; cp -a overlays the new binaries.
rm -rf "$DEST/usr" "$DEST/etc"; install -d -m 755 "$DEST"
cp -a "$tmp/podman-linux-$A/." "$DEST/"

# Relocated, meta-hosted config. The podman binary is direct; config is read via
# normal XDG/user config symlinks, not a launcher wrapper.
install -d -m 755 "$DEST/etc/containers" "$META_ROOT/.config/containers" "$META_ROOT/usr/bin"
cat > "$DEST/etc/containers/containers.conf" <<EOF
[engine]
cgroup_manager = "cgroupfs"
events_logger = "file"
conmon_path = ["$DEST/usr/local/lib/podman/conmon"]
helper_binaries_dir = ["$DEST/usr/local/lib/podman", "$DEST/usr/local/libexec/podman", "$DEST/usr/local/bin"]
network_cmd_path = "$DEST/usr/local/bin/pasta"
runtime = "crun"

[engine.runtimes]
crun = ["$DEST/usr/local/bin/crun"]
runc = ["$DEST/usr/local/bin/runc"]
EOF
cat > "$DEST/etc/containers/storage.conf" <<EOF
[storage]
driver = "overlay"
runroot = "$M/.toolchains/podman/runroot"
graphroot = "$M/.toolchains/podman/graphroot"
rootless_storage_path = "$M/.toolchains/podman/rootless-storage"

[storage.options.overlay]
ignore_chown_errors = "true"
mount_program = "$DEST/usr/local/bin/fuse-overlayfs"
mountopt = "nodev,fsync=0"
EOF
for f in containers.conf storage.conf registries.conf policy.json; do
  [ -f "$DEST/etc/containers/$f" ] && ln -sfn "$DEST/etc/containers/$f" "$META_ROOT/.config/containers/$f"
done
envctl_frontdoor "$DEST/usr/local/bin/podman" "$META_ROOT/usr/bin/podman"
for b in crun runc fuse-overlayfs fusermount3 pasta; do
  [ -x "$DEST/usr/local/bin/$b" ] && envctl_frontdoor "$DEST/usr/local/bin/$b" "$META_ROOT/usr/bin/$b"
done

` |
| `setting` | `component[1].install.script` | `manifest/base.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -e
M="${META_ROOT:?META_ROOT required}"
export BUN_INSTALL="$M/.toolchains/.bun"
mkdir -p "$BUN_INSTALL" "$META_ROOT/usr/bin"
curl -fsSL https://bun.sh/install | bash
envctl_frontdoor "$BUN_INSTALL/bin/bun" "$META_ROOT/usr/bin/bun"
cat >"$META_ROOT/usr/bin/bunx" <<'BUNX'
#!/usr/bin/env bash
# envctl bunx wrapper
set -euo pipefail
self="${BASH_SOURCE[0]}"
while [ -L "$self" ]; do
  dir="$(cd -P "$(dirname "$self")" && pwd)"
  target="$(readlink "$self")"
  case "$target" in
    /*) self="$target" ;;
    *) self="$dir/$target" ;;
  esac
done
bindir="$(cd -P "$(dirname "$self")" && pwd)"
META_ROOT="$(cd "$bindir/../.." && pwd)"
export META_ROOT
export BUN_INSTALL="${BUN_INSTALL:-$META_ROOT/.toolchains/.bun}"
export HOME="${HOME:-$META_ROOT/.local}"
export XDG_CACHE_HOME="${XDG_CACHE_HOME:-$META_ROOT/.local/cache}"
exec "$BUN_INSTALL/bin/bun" x "$@"
BUNX
chmod 755 "$META_ROOT/usr/bin/bunx"
` |
| `setting` | `component[1].install.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
DEST="$M/.toolchains/wild"
cargo install --locked wild-linker --root "$DEST"
install -d -m 755 "$META_ROOT/usr/bin"
envctl_frontdoor "$DEST/bin/wild" "$META_ROOT/usr/bin/wild"
# Wire wild as the local cargo linker via the meta-root .cargo/config.toml. This file
# lives OUTSIDE the envctl repo (a runtime artifact at $META_ROOT, like .toolchains/); only
# this component def is committed. CI clones each repo standalone so it never sees it. The
# file is CO-MANAGED with the kache component (Epic H TASK-0055): each component owns a
# delimited block written by a NON-CLOBBERING upsert (strip my old block, append fresh),
# so the two never overwrite each other and any foreign content is preserved.
install -d -m 755 "$M/.cargo"
CFG="$M/.cargo/config.toml"
touch "$CFG"
# One-time migration off the legacy wholesale format (pre-TASK-0055: a single marker line +
# a [target...] section written via \`cat >\`). Strip exactly those legacy lines, preserving
# anything else (e.g. a kache [build] block written first). Stop the legacy skip at the next
# [section] OR the next comment line (so a following block's \`# >>> ... >>>\` marker survives).
if grep -q '^# managed by envctl wild-linker component' "$CFG"; then
  awk '/^# managed by envctl wild-linker component/{leg=1;next} leg&&((/^\[/&&$0!~/^\[target\.x86_64-unknown-linux-gnu\]/)||/^#/){leg=0} leg{next} {print}' "$CFG" > "$CFG.mig" && mv "$CFG.mig" "$CFG"
fi
# Upsert our delimited block idempotently (replace any prior wild block, keep the rest).
BEG="# >>> envctl wild-linker (Epic H TASK-0054) >>>"
END="# <<< envctl wild-linker (Epic H TASK-0054) <<<"
awk -v b="$BEG" -v e="$END" '$0==b{s=1} s&&$0==e{s=0;next} !s{print}' "$CFG" > "$CFG.tmp" && mv "$CFG.tmp" "$CFG"
printf '%s\n[target.x86_64-unknown-linux-gnu]\nlinker = "clang"\nrustflags = ["-Clink-arg=--ld-path=wild"]\n%s\n' "$BEG" "$END" >> "$CFG"


` |
| `setting` | `component[1].install.script` | `manifest/components.d/meta-core-clis.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
VER="${CURL_VERSION:-8.21.0}"
DEST="$M/.toolchains/curl"
ARCH="$(uname -m)"; case "$ARCH" in x86_64) A=x86_64;; aarch64) A=aarch64;; *) echo "unsupported arch: $ARCH"; exit 1;; esac
URL="https://github.com/stunnel/static-curl/releases/download/$VER/curl-linux-$A-musl-$VER.tar.xz"
cache="$M/.cache/curl"; install -d -m 755 "$cache" "$DEST/bin" "$META_ROOT/usr/bin"
tarball="$cache/curl-linux-$A-musl-$VER.tar.xz"
[ -s "$tarball" ] || /usr/bin/curl -fSL "$URL" -o "$tarball"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
tar -xJf "$tarball" -C "$tmp"
src="$(find "$tmp" -type f -name curl -perm -111 | head -n1)"
install -m 755 "$src" "$DEST/bin/curl"
envctl_frontdoor "$DEST/bin/curl" "$META_ROOT/usr/bin/curl"

` |
| `setting` | `component[1].install.script` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -e
M="${META_ROOT:?META_ROOT required}"
export BUN_INSTALL="$M/.toolchains/.bun"
export PATH="$BUN_INSTALL/bin:$PATH"
bun add -g vite
mkdir -p "$META_ROOT/usr/bin"
envctl_frontdoor "$BUN_INSTALL/bin/vite" "$META_ROOT/usr/bin/vite"
` |
| `setting` | `component[1].install.script` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
DEST="$M/.toolchains/cuda"
VER="13.3.0"; DRV="610.43.02"
MD5="16d68669cf659157777d2e7adaff179d"
RUN="cuda_${VER}_${DRV}_linux.run"
URL="https://developer.download.nvidia.com/compute/cuda/${VER}/local_installers/${RUN}"
# Cache the ~4GB runfile under $M/.cache so a retry/fix never re-downloads. Verify md5 each
# time; re-fetch only if missing or corrupt. The download + integrity check happen BEFORE we
# touch the existing prefix, so a bad download is fail-closed and never destroys a good install.
cache="$M/.cache/cuda"; install -d -m 755 "$cache"
if ! ( [ -s "$cache/$RUN" ] && echo "${MD5}  $cache/$RUN" | md5sum -c - >/dev/null 2>&1 ); then
  curl -fSL "$URL" -o "$cache/$RUN"
  echo "${MD5}  $cache/$RUN" | md5sum -c -
fi
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
# Toolkit-only, user-prefix, no-sudo install. --toolkit (NOT --driver/--samples) so the apt
# nvidia-open kernel driver (system floor) is never touched; --override skips the host
# compiler/distro checks; --nox11 + </dev/null stop the makeself wrapper from trying to spawn
# an xterm when stdout is not a TTY (headless/CI); --tmpdir keeps the ~7GB self-extraction
# off a small /tmp. NOTE: invoked with \`sh\`; --nox11 is the load-bearing headless flag.
rm -rf "$DEST"; install -d -m 755 "$DEST"
sh "$cache/$RUN" --silent --toolkit --toolkitpath="$DEST" --override --no-opengl-libs --no-man-page --nox11 --tmpdir="$tmp" </dev/null
# Curated bin tools → $META_ROOT/usr/bin (nvcc resolves its sibling nvvm/targets via realpath).
install -d -m 755 "$META_ROOT/usr/bin"
for t in nvcc nsys ncu cuda-gdb compute-sanitizer cuobjdump; do
  [ -x "$DEST/bin/$t" ] && envctl_frontdoor "$DEST/bin/$t" "$META_ROOT/usr/bin/$t"
done
if [ -x /usr/bin/nvidia-smi ]; then
  install -m 755 /usr/bin/nvidia-smi "$DEST/bin/nvidia-smi"
  envctl_frontdoor "$DEST/bin/nvidia-smi" "$META_ROOT/usr/bin/nvidia-smi"
fi

` |
| `setting` | `component[1].install.script` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `set -e
sudo install -dm 755 /etc/nix
sudo touch /etc/nix/nix.custom.conf
add(){ grep -qF -- "$1" /etc/nix/nix.custom.conf || echo "$1" | sudo tee -a /etc/nix/nix.custom.conf >/dev/null; }
add "eval-cores = 0"
add "extra-substituters = https://yazelix.cachix.org"
add "extra-trusted-public-keys = yazelix.cachix.org-1:ZgxIjQvaP0VTWL8Racx27mpUNzDJ97xC2y7QWYjmGNM="
sudo systemctl restart nix-daemon 2>/dev/null || true
` |
| `setting` | `component[1].name` | `manifest/ai-clis.toml` | `scope=component source_kind=manifest` | `Codex CLI (meta toolchain; pinned upstream release rust-v0.142.3, meta-owned CODEX_HOME wrapper)` |
| `setting` | `component[1].name` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `wild linker (meta-owned, wired + verified)` |
| `setting` | `component[1].remove.args` | `manifest/ai-clis.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; for BUN_INSTALL in \"$M/.toolchains/.bun\" \"$M/.toolchains/bun\"; do [ -x \"$BUN_INSTALL/bin/bun\" ] && PATH=\"$BUN_INSTALL/bin:$PATH\" timeout --kill-after=2s 20s bun remove -g @openai/codex >/dev/null 2>&1 || true; rm -f \"$BUN_INSTALL/bin/codex\" \"$BUN_INSTALL/install/global/node_modules/.bin/codex\"; rm -rf \"$BUN_INSTALL/install/cache/@openai/codex\" \"$BUN_INSTALL/install/cache/@openai/codex\"* \"$BUN_INSTALL/install/global/node_modules/@openai/codex\" \"$BUN_INSTALL/install/global/node_modules/@openai/codex\"*; done; for t in \"$M/usr/bin/codex\" \"$M/.local/bin/codex\"; do if { [ -L \"$t\" ] || grep -q \"envctl codex wrapper\" \"$t\" 2>/dev/null; }; then rm -f \"$t\"; fi; done; if [ -n \"${ENVCTL_REAL_HOME:-}\" ]; then t=\"$ENVCTL_REAL_HOME/.local/bin/codex\"; if { [ -L \"$t\" ] || grep -q \"envctl codex wrapper\" \"$t\" 2>/dev/null; }; then rm -f \"$t\"; fi; fi; rm -rf \"$M/.toolchains/openai-codex\""]` |
| `setting` | `component[1].remove.args` | `manifest/apt-base.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; for b in podman crun runc fuse-overlayfs fusermount3 pasta; do t=\"$META_ROOT/usr/bin/$b\"; [ -L \"$t\" ] && readlink \"$t\" | grep -q \"$M/.toolchains/podman\" && rm -f \"$t\"; done; for f in containers.conf storage.conf registries.conf policy.json; do t=\"$META_ROOT/.config/containers/$f\"; [ -L \"$t\" ] && readlink \"$t\" | grep -q \"$M/.toolchains/podman\" && rm -f \"$t\"; done; rm -rf \"$M/.toolchains/podman\""]` |
| `setting` | `component[1].remove.args` | `manifest/base.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; t=\"$M/usr/bin/bun\"; src=\"$M/.toolchains/.bun/bin/bun\"; { [ -L \"$t\" ] && [ \"$(readlink -f \"$t\" 2>/dev/null)\" = \"$(readlink -f \"$src\" 2>/dev/null)\" ]; } || { [ -f \"$t\" ] && grep -Fqx \"exec \\\"$src\\\" \\\"\\$@\\\"\" \"$t\"; } && rm -f \"$t\" || true; if grep -q 'envctl bunx wrapper' \"$M/usr/bin/bunx\" 2>/dev/null; then rm -f \"$M/usr/bin/bunx\"; fi; rm -rf \"$M/.toolchains/.bun\""]` |
| `setting` | `component[1].remove.args` | `manifest/components.d/meta-core-clis.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; t=\"$META_ROOT/usr/bin/curl\"; src=\"$M/.toolchains/curl/bin/curl\"; if { [ -L \"$t\" ] && [ \"$(readlink -f \"$t\" 2>/dev/null || true)\" = \"$(readlink -f \"$src\" 2>/dev/null || true)\" ]; } || { [ -f \"$t\" ] && grep -Fqx \"exec \\\"$src\\\" \\\"\\$@\\\"\" \"$t\"; }; then rm -f \"$t\"; fi; rm -rf \"$M/.toolchains/curl\""]` |
| `setting` | `component[1].remove.args` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export BUN_INSTALL=\"$M/.toolchains/.bun\"; export PATH=\"$BUN_INSTALL/bin:$PATH\"; bun remove -g vite || true; t=\"$M/usr/bin/vite\"; src=\"$BUN_INSTALL/bin/vite\"; { [ -L \"$t\" ] && [ \"$(readlink -f \"$t\" 2>/dev/null)\" = \"$(readlink -f \"$src\" 2>/dev/null)\" ]; } || { [ -f \"$t\" ] && grep -Fqx \"exec \\\"$src\\\" \\\"\\$@\\\"\" \"$t\"; } && rm -f \"$t\" || true"]` |
| `setting` | `component[1].remove.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `set -u
M="${META_ROOT:?META_ROOT required}"
t="$META_ROOT/usr/bin/wild"; [ -L "$t" ] && readlink "$t" | grep -q "$M/.toolchains/wild" && rm -f "$t"
rm -rf "$M/.toolchains/wild"
# Strip the wild linker block from the CO-MANAGED config (non-destructive to other blocks,
# e.g. kache, and to foreign content). Self-guarded by OUR delimiters; also strips the legacy
# wholesale format if present. If nothing is left, remove the file. Drop any obsolete
# pre-wild backup left by the legacy format.
CFG="$M/.cargo/config.toml"
BEG="# >>> envctl wild-linker (Epic H TASK-0054) >>>"
END="# <<< envctl wild-linker (Epic H TASK-0054) <<<"
if [ -f "$CFG" ]; then
  awk -v b="$BEG" -v e="$END" '$0==b{s=1} s&&$0==e{s=0;next} !s{print}' "$CFG" > "$CFG.tmp" && mv "$CFG.tmp" "$CFG"
  if grep -q '^# managed by envctl wild-linker component' "$CFG" 2>/dev/null; then
    awk '/^# managed by envctl wild-linker component/{leg=1;next} leg&&((/^\[/&&$0!~/^\[target\.x86_64-unknown-linux-gnu\]/)||/^#/){leg=0} leg{next} {print}' "$CFG" > "$CFG.mig" && mv "$CFG.mig" "$CFG"
  fi
  grep -q '[^[:space:]]' "$CFG" || rm -f "$CFG"
fi
rm -f "$CFG.pre-wild.bak"
` |
| `setting` | `component[1].remove.script` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `set -u
M="${META_ROOT:?META_ROOT required}"
for t in nvcc nsys ncu cuda-gdb compute-sanitizer cuobjdump nvidia-smi; do
  tt="$META_ROOT/usr/bin/$t"; [ -L "$tt" ] && readlink "$tt" | grep -q "$M/.toolchains/cuda" && rm -f "$tt"
done
rm -rf "$M/.toolchains/cuda"
` |
| `setting` | `component[1].requires` | `manifest/ai-clis.toml` | `scope=component source_kind=manifest` | `["rustup","bun","wild-linker"]` |
| `setting` | `component[1].requires` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `["rustup"]` |
| `setting` | `component[1].requires` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `["nix"]` |
| `setting` | `component[1].verify.args` | `manifest/ai-clis.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export PATH=\"$M/usr/bin:$M/.local/bin:$M/.toolchains/node/bin:$PATH\"; link=\"$M/usr/bin/codex\"; cur=\"$M/.toolchains/openai-codex/current/bin/codex\"; [ -x \"$cur\" ] && [ -x \"$link\" ] && grep -q \"envctl codex wrapper\" \"$link\" && [ \"$(readlink -f \"$M/.local/bin/codex\" 2>/dev/null)\" = \"$(readlink -f \"$link\" 2>/dev/null)\" ] && CODEX_HOME= CODEX_SQLITE_HOME= codex --version >/dev/null && CODEX_HOME= CODEX_SQLITE_HOME= codex mcp list >/dev/null && grep -q 'CODEX_BIN_PATH=.*openai-codex/current/bin/codex' \"$link\" && [ ! -e \"$M/.toolchains/bun/bin/codex\" ] && [ ! -L \"$M/.toolchains/bun/bin/codex\" ] && [ ! -e \"$M/.toolchains/.bun/bin/codex\" ] && [ ! -L \"$M/.toolchains/.bun/bin/codex\" ] && [ ! -e \"$M/.toolchains/bun/install/global/node_modules/.bin/codex\" ] && [ ! -L \"$M/.toolchains/bun/install/global/node_modules/.bin/codex\" ] && [ ! -e \"$M/.toolchains/.bun/install/global/node_modules/.bin/codex\" ] && [ ! -L \"$M/.toolchains/.bun/install/global/node_modules/.bin/codex\" ] && ! grep -Rqs '\"@openai/codex\"' \"$M/.toolchains/bun/install/global/package.json\" \"$M/.toolchains/bun/install/global/bun.lock\" \"$M/.toolchains/.bun/install/global/package.json\" \"$M/.toolchains/.bun/install/global/bun.lock\" 2>/dev/null && ! find \"$M/.toolchains/bun\" \"$M/.toolchains/.bun\" -path '*/@openai/codex*' -print -quit 2>/dev/null | grep -q . && \"$M/envctl/assets/scripts/envctl-codex-cleanup.sh\" verify"]` |
| `setting` | `component[1].verify.args` | `manifest/apt-base.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; P=\"$(command -v podman 2>/dev/null || echo \"$M/.toolchains/podman/usr/local/bin/podman\")\"; \"$P\" --version 2>/dev/null | grep -q '5\\.8\\.3' && \"$P\" info --format '{{.Host.Conmon.Path}} {{.Host.OCIRuntime.Path}}' 2>/dev/null | grep -q \"$M/.toolchains/podman\""]` |
| `setting` | `component[1].verify.args` | `manifest/base.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export BUN_INSTALL=\"$M/.toolchains/.bun\"; export PATH=\"$M/usr/bin:$BUN_INSTALL/bin:$PATH\"; bun --version >/dev/null && [ -x \"$M/usr/bin/bunx\" ] && grep -q 'envctl bunx wrapper' \"$M/usr/bin/bunx\""]` |
| `setting` | `component[1].verify.args` | `manifest/components.d/meta-core-clis.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export PATH=\"$M/usr/bin:$PATH\"; [ -x \"$M/usr/bin/curl\" ] && [ ! -L \"$M/usr/bin/curl\" ] && grep -Fqx \"exec \\\"$M/.toolchains/curl/bin/curl\\\" \\\"\\$@\\\"\" \"$M/usr/bin/curl\" && curl --version | grep -q '8\\.21\\.0'"]` |
| `setting` | `component[1].verify.args` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export BUN_INSTALL=\"$M/.toolchains/.bun\"; export PATH=\"$BUN_INSTALL/bin:$PATH\"; vite --version"]` |
| `setting` | `component[1].verify.args` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; D=\"$M/.toolchains/cuda\"; \"$D/bin/nvcc\" --version | command grep -q 'release 13.3' && { ls \"$D\"/targets/x86_64-linux/lib/libcublas.so* >/dev/null 2>&1 || ls \"$D\"/lib64/libcublas.so* >/dev/null 2>&1; } && { [ ! -x /usr/bin/nvidia-smi ] || [ -x \"$META_ROOT/usr/bin/nvidia-smi\" ] && [ ! -L \"$META_ROOT/usr/bin/nvidia-smi\" ] && grep -Fqx \"exec \\\"$D/bin/nvidia-smi\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/nvidia-smi\"; }"]` |
| `setting` | `component[1].verify.args` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `["-lc",". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh 2>/dev/null; nix config show 2>/dev/null | grep -qE 'yazelix\\.cachix\\.org'"]` |
| `setting` | `component[1].verify.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
export PATH="$META_ROOT/usr/bin:$PATH"
# Bypass any interactive \`grep\` shell-function wrapper (rtk/ugrep hook in login shells):
# \`command grep\` always invokes the real binary so piped greps in this hook are robust.
g() { command grep "$@"; }
# 1) tools resolve
wild --version
clang --version >/dev/null
# 2) the managed config exists and carries the wild linker section
CFG="$M/.cargo/config.toml"
[ -f "$CFG" ] && g -q -- '--ld-path=wild' "$CFG"
# 3) prove the wiring actually links via wild. Build a tiny throwaway crate in a /tmp dir
#    (outside the meta tree so it never gets pulled into the meta workspace) and apply the
#    meta-root linker config EXPLICITLY via \`cargo --config <path>\`. Confirm the verbose link
#    line carries \`--ld-path=wild\`. Self-contained target-dir; cleaned on exit.
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/proj/src"
cat > "$tmp/proj/Cargo.toml" <<EOF
[package]
name = "wild-wire-probe"
version = "0.0.0"
edition = "2021"
[[bin]]
name = "wild-wire-probe"
path = "src/main.rs"
EOF
echo 'fn main() {}' > "$tmp/proj/src/main.rs"
( cd "$tmp/proj" && cargo build -v --config "$CFG" --target-dir "$tmp/td" 2>&1 ) | g -q -- '--ld-path=wild'
` |
| `setting` | `component[1].wiring.nix_conf_lines[0].line` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `eval-cores = 0` |
| `setting` | `component[1].wiring.nix_conf_lines[1].line` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `extra-substituters = https://yazelix.cachix.org` |
| `setting` | `component[1].wiring.nix_conf_lines[2].line` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `extra-trusted-public-keys = yazelix.cachix.org-1:ZgxIjQvaP0VTWL8Racx27mpUNzDJ97xC2y7QWYjmGNM=` |
| `setting` | `component[1].wiring.path_entries` | `manifest/base.toml` | `scope=component source_kind=manifest` | `["$META_ROOT/.toolchains/.bun/bin","$META_ROOT/usr/bin"]` |
| `setting` | `component[1].wiring.shell_rc[0].content` | `manifest/base.toml` | `scope=component source_kind=manifest` | `eval "$(envctl env --toolchains)"` |
| `setting` | `component[1].wiring.shell_rc[0].content` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `M="${META_ROOT:?META_ROOT required}"
CUDA_HOME="$M/.toolchains/cuda"; [ -d "$CUDA_HOME/bin" ] || CUDA_HOME=/usr/local/cuda; [ -d "$CUDA_HOME/bin" ] || CUDA_HOME="$(ls -d /usr/local/cuda-* 2>/dev/null | sort -V | tail -n1)"
export CUDA_HOME
export PATH="$CUDA_HOME/bin:$PATH"
export LD_LIBRARY_PATH="$CUDA_HOME/lib64:$CUDA_HOME/targets/x86_64-linux/lib:${LD_LIBRARY_PATH:-}"
export CUDA_OXIDE_LLC="$(command -v llc-22 || command -v llc-21 || command -v llc || ls /usr/lib/llvm-2*/bin/llc 2>/dev/null | sort -V | tail -n1)"
` |
| `setting` | `component[1].wiring.shell_rc[0].marker` | `manifest/base.toml` | `scope=component source_kind=manifest` | `meta toolchain path` |
| `setting` | `component[2].description` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `Content-addressed Rust compiler cache (kunobi-ninja/kache) via \`cargo install kache\` into .toolchains/kache, exposed through a $META_ROOT/usr/bin/kache XDG wrapper, and wired as RUSTC_WRAPPER via an absolute delimited [build] block in the meta-root .cargo/config.toml (co-managed with wild via block-upsert). Wiring is verified by a throwaway cargo build that must write $META_ROOT/.cache/kache, not ~/.cache/kache. sccache only as last-resort fallback.` |
| `setting` | `component[2].description` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `Repo-required Rust test runner for Codex and other Rust workspaces; installed through meta-owned CARGO_HOME.` |
| `setting` | `component[2].detect.args` | `manifest/ai-clis.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; VER=\"${CODEX_ALPHA_VERSION:-0.143.0-alpha.29}\"; dest=\"$M/.toolchains/openai-codex/${VER}/bin/codex\"; link=\"$M/usr/bin/codex-alpha\"; meta_compat=\"$M/.local/bin/codex-alpha\"; [ -x \"$dest\" ] && [ -x \"$link\" ] && grep -q \"envctl codex alpha wrapper\" \"$link\" && [ \"$(readlink -f \"$meta_compat\" 2>/dev/null)\" = \"$(readlink -f \"$link\" 2>/dev/null)\" ] && \"$link\" --version | grep -Fq \"$VER\""]` |
| `setting` | `component[2].detect.args` | `manifest/base.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; { [ -x \"$M/.toolchains/.bun/bin/node\" ] && readlink -f \"$M/.toolchains/.bun/bin/node\" 2>/dev/null | grep -q bun; } || command -v node >/dev/null"]` |
| `setting` | `component[2].detect.args` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/kache/bin/kache\" ] && [ -x \"$META_ROOT/usr/bin/kache\" ] && [ ! -L \"$META_ROOT/usr/bin/kache\" ] && grep -Fqx \"exec \\\"$M/.toolchains/kache/bin/kache\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/kache\" && [ -f \"$M/.cargo/config.toml\" ] && grep -q 'rustc-wrapper' \"$M/.cargo/config.toml\""]` |
| `setting` | `component[2].detect.args` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/cargo/bin/cargo-nextest\" ] && [ -x \"$META_ROOT/usr/bin/cargo-nextest\" ] && [ ! -L \"$META_ROOT/usr/bin/cargo-nextest\" ] && grep -Fqx \"exec \\\"$M/.toolchains/cargo/bin/cargo-nextest\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/cargo-nextest\""]` |
| `setting` | `component[2].detect.args` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `["-lc",". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh 2>/dev/null; export PATH=\"$META_ROOT/.nix-profile/bin:$PATH\"; command -v home-manager"]` |
| `setting` | `component[2].fix.args` | `manifest/base.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; ln -sf \"$M/.toolchains/.bun/bin/bun\" \"$M/.toolchains/.bun/bin/node\""]` |
| `setting` | `component[2].fix.script` | `manifest/ai-clis.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
ROOT="$M/envctl"; source "$ROOT/assets/scripts/envctl-gh-fetch.sh"
VER="${CODEX_ALPHA_VERSION:-0.143.0-alpha.29}"
BASE="$M/.toolchains/openai-codex"
VDIR="$BASE/${VER}"
LINK="$M/usr/bin/codex-alpha"
if [ ! -x "$VDIR/bin/codex" ]; then
  tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
  URL="https://github.com/openai/codex/releases/download/rust-v${VER}/codex-x86_64-unknown-linux-musl.tar.gz"
  envctl_gh_curl "$URL" -o "$tmp/codex.tgz"
  tar -xzf "$tmp/codex.tgz" -C "$tmp"
  src="$(find "$tmp" -type f \( -name codex -o -name 'codex-*-linux-musl' \) | head -1)"
  [ -n "$src" ] && [ -f "$src" ] || { echo "codex-alpha: no binary found in release tarball" >&2; exit 1; }
  install -d -m 755 "$VDIR/bin"
  install -m 755 "$src" "$VDIR/bin/codex"
fi
install -d -m 755 "$M/usr/bin"
cat >"$LINK" <<'WRAPPER'
#!/usr/bin/env bash
# envctl codex alpha wrapper
set -euo pipefail
self="${BASH_SOURCE[0]}"
while [ -L "$self" ]; do
  dir="$(cd -P "$(dirname "$self")" && pwd)"
  target="$(readlink "$self")"
  case "$target" in /*) self="$target" ;; *) self="$dir/$target" ;; esac
done
bindir="$(cd -P "$(dirname "$self")" && pwd)"
META_ROOT="$(cd "$bindir/../.." && pwd)"
VER="${CODEX_ALPHA_VERSION:-0.143.0-alpha.29}"
export META_ROOT
export CODEX_HOME="${CODEX_HOME:-$META_ROOT/.local/share/codex}"
export CODEX_SQLITE_HOME="${CODEX_SQLITE_HOME:-$META_ROOT/.local/state/codex}"
export CODEX_BIN_PATH="$META_ROOT/.toolchains/openai-codex/${VER}/bin/codex"
export CODEX_CLI_BIN="$CODEX_BIN_PATH"
export PATH="$META_ROOT/usr/bin:$META_ROOT/.local/bin:$META_ROOT/.toolchains/cargo/bin:$META_ROOT/.toolchains/.bun/bin:$META_ROOT/.toolchains/node/bin:$PATH"
umask 077
mkdir -p "$CODEX_HOME" "$CODEX_SQLITE_HOME"
exec "$CODEX_BIN_PATH" "$@"
WRAPPER
chmod 755 "$LINK"
install -d -m 755 "$M/.local/bin"
ln -sfn "$LINK" "$M/.local/bin/codex-alpha"
"$LINK" --version | grep -Fq "$VER"
CODEX_HOME= CODEX_SQLITE_HOME= "$LINK" mcp list >/dev/null
` |
| `setting` | `component[2].fix.script` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
export CARGO_HOME="$M/.toolchains/cargo"
export PATH="$CARGO_HOME/bin:$META_ROOT/.toolchains/cargo/bin:$PATH"
cargo install cargo-nextest --version 0.9.137 --force --locked
mkdir -p "$META_ROOT/usr/bin"
envctl_frontdoor "$CARGO_HOME/bin/cargo-nextest" "$META_ROOT/usr/bin/cargo-nextest"

` |
| `setting` | `component[2].fix.script` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `. /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh 2>/dev/null; nix profile upgrade home-manager || nix profile install nixpkgs#home-manager` |
| `setting` | `component[2].id` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `kache` |
| `setting` | `component[2].id` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `cargo-nextest` |
| `setting` | `component[2].install.args` | `manifest/base.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; ln -sf \"$M/.toolchains/.bun/bin/bun\" \"$M/.toolchains/.bun/bin/node\""]` |
| `setting` | `component[2].install.script` | `manifest/ai-clis.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
ROOT="$M/envctl"; source "$ROOT/assets/scripts/envctl-gh-fetch.sh"
VER="${CODEX_ALPHA_VERSION:-0.143.0-alpha.29}"
BASE="$M/.toolchains/openai-codex"
VDIR="$BASE/${VER}"
LINK="$M/usr/bin/codex-alpha"
if [ ! -x "$VDIR/bin/codex" ]; then
  tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
  URL="https://github.com/openai/codex/releases/download/rust-v${VER}/codex-x86_64-unknown-linux-musl.tar.gz"
  envctl_gh_curl "$URL" -o "$tmp/codex.tgz"
  tar -xzf "$tmp/codex.tgz" -C "$tmp"
  src="$(find "$tmp" -type f \( -name codex -o -name 'codex-*-linux-musl' \) | head -1)"
  [ -n "$src" ] && [ -f "$src" ] || { echo "codex-alpha: no binary found in release tarball" >&2; exit 1; }
  install -d -m 755 "$VDIR/bin"
  install -m 755 "$src" "$VDIR/bin/codex"
fi
install -d -m 755 "$M/usr/bin"
if [ -e "$LINK" ] && ! grep -q "envctl codex alpha wrapper" "$LINK" 2>/dev/null; then
  ARCH="$M/var/lib/envctl/legacy-archives/usr-bin-$(date -u +%Y-%m-%d)/usr/bin"
  install -d -m 755 "$ARCH"
  mv "$LINK" "$ARCH/codex-alpha"
  echo "archived previous codex-alpha front door: $ARCH/codex-alpha"
fi
cat >"$LINK" <<'WRAPPER'
#!/usr/bin/env bash
# envctl codex alpha wrapper
set -euo pipefail
self="${BASH_SOURCE[0]}"
while [ -L "$self" ]; do
  dir="$(cd -P "$(dirname "$self")" && pwd)"
  target="$(readlink "$self")"
  case "$target" in
    /*) self="$target" ;;
    *) self="$dir/$target" ;;
  esac
done
bindir="$(cd -P "$(dirname "$self")" && pwd)"
META_ROOT="$(cd "$bindir/../.." && pwd)"
VER="${CODEX_ALPHA_VERSION:-0.143.0-alpha.29}"
export META_ROOT
export CODEX_HOME="${CODEX_HOME:-$META_ROOT/.local/share/codex}"
export CODEX_SQLITE_HOME="${CODEX_SQLITE_HOME:-$META_ROOT/.local/state/codex}"
export CODEX_BIN_PATH="$META_ROOT/.toolchains/openai-codex/${VER}/bin/codex"
export CODEX_CLI_BIN="$CODEX_BIN_PATH"
export PATH="$META_ROOT/usr/bin:$META_ROOT/.local/bin:$META_ROOT/.toolchains/cargo/bin:$META_ROOT/.toolchains/.bun/bin:$META_ROOT/.toolchains/node/bin:$PATH"
umask 077
mkdir -p "$CODEX_HOME" "$CODEX_SQLITE_HOME"
exec "$CODEX_BIN_PATH" "$@"
WRAPPER
chmod 755 "$LINK"
install -d -m 755 "$M/.local/bin"
ln -sfn "$LINK" "$M/.local/bin/codex-alpha"
if [ -n "${ENVCTL_REAL_HOME:-}" ]; then
  real_link="$ENVCTL_REAL_HOME/.local/bin/codex-alpha"
  install -d -m 755 "$(dirname "$real_link")"
  if [ -e "$real_link" ] && [ ! -L "$real_link" ]; then
    ARCH="$M/var/lib/envctl/legacy-archives/real-home-bin-$(date -u +%Y-%m-%d)/.local/bin"
    install -d -m 755 "$ARCH"
    mv "$real_link" "$ARCH/codex-alpha"
    echo "archived previous real-home codex-alpha front door: $ARCH/codex-alpha"
  fi
  ln -sfn "$LINK" "$real_link"
fi
"$LINK" --version | grep -Fq "$VER"
CODEX_HOME= CODEX_SQLITE_HOME= "$LINK" mcp list >/dev/null
` |
| `setting` | `component[2].install.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
DEST="$M/.toolchains/kache"
cargo install kache --root "$DEST"
install -d -m 755 "$META_ROOT/usr/bin"
envctl_frontdoor "$DEST/bin/kache" "$META_ROOT/usr/bin/kache"
# Wire kache as RUSTC_WRAPPER via the meta-root .cargo/config.toml [build] section. Same
# CO-MANAGED file as the wild-linker component (TASK-0054): each owns a delimited block,
# written by a non-clobbering upsert so the two never overwrite each other and foreign
# content is preserved. Use an ABSOLUTE wrapper path so Cargo never falls through to an
# older host-local kache or any PATH-shadowed binary.
install -d -m 755 "$M/.cargo"
CFG="$M/.cargo/config.toml"
touch "$CFG"
BEG="# >>> envctl kache (Epic H TASK-0055) >>>"
END="# <<< envctl kache (Epic H TASK-0055) <<<"
awk -v b="$BEG" -v e="$END" '$0==b{s=1} s&&$0==e{s=0;next} !s{print}' "$CFG" > "$CFG.tmp" && mv "$CFG.tmp" "$CFG"
printf '%s\n[build]\nrustc-wrapper = "kache"\n%s\n' "$BEG" "$END" >> "$CFG"


` |
| `setting` | `component[2].install.script` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
export CARGO_HOME="$M/.toolchains/cargo"
export PATH="$CARGO_HOME/bin:$META_ROOT/.toolchains/cargo/bin:$PATH"
cargo install cargo-nextest --version 0.9.137 --locked
mkdir -p "$META_ROOT/usr/bin"
envctl_frontdoor "$CARGO_HOME/bin/cargo-nextest" "$META_ROOT/usr/bin/cargo-nextest"
` |
| `setting` | `component[2].install.script` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `. /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh 2>/dev/null; nix profile install nixpkgs#home-manager` |
| `setting` | `component[2].name` | `manifest/ai-clis.toml` | `scope=component source_kind=manifest` | `Codex CLI alpha candidate (parallel Rust release lane; does not repoint codex/current)` |
| `setting` | `component[2].name` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `kache compiler cache (meta-owned, wired + verified)` |
| `setting` | `component[2].name` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `cargo-nextest (Rust test runner)` |
| `setting` | `component[2].remove.args` | `manifest/base.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; rm -f \"$M/.toolchains/.bun/bin/node\""]` |
| `setting` | `component[2].remove.args` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export CARGO_HOME=\"$M/.toolchains/cargo\"; export PATH=\"$CARGO_HOME/bin:$PATH\"; cargo uninstall cargo-nextest || true; t=\"$M/usr/bin/cargo-nextest\"; src=\"$CARGO_HOME/bin/cargo-nextest\"; { [ -L \"$t\" ] && [ \"$(readlink -f \"$t\" 2>/dev/null)\" = \"$(readlink -f \"$src\" 2>/dev/null)\" ]; } || { [ -f \"$t\" ] && grep -Fqx \"exec \\\"$src\\\" \\\"\\$@\\\"\" \"$t\"; } && rm -f \"$t\" || true"]` |
| `setting` | `component[2].remove.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `set -u
M="${META_ROOT:?META_ROOT required}"
t="$M/usr/bin/kache"
if [ -e "$t" ] || [ -L "$t" ]; then
  if { [ -f "$t" ] && grep -q 'envctl kache wrapper' "$t"; } || { [ -L "$t" ] && readlink "$t" | grep -q "$M/.toolchains/kache"; }; then
    rm -f "$t"
  fi
fi
rm -rf "$M/.toolchains/kache"
# Strip ONLY the kache block from the co-managed config (leaves wild's block + foreign
# content intact). Self-guarded by OUR delimiters. Remove the file if nothing remains.
CFG="$M/.cargo/config.toml"
BEG="# >>> envctl kache (Epic H TASK-0055) >>>"
END="# <<< envctl kache (Epic H TASK-0055) <<<"
if [ -f "$CFG" ]; then
  awk -v b="$BEG" -v e="$END" '$0==b{s=1} s&&$0==e{s=0;next} !s{print}' "$CFG" > "$CFG.tmp" && mv "$CFG.tmp" "$CFG"
  grep -q '[^[:space:]]' "$CFG" || rm -f "$CFG"
fi
if command -v systemctl >/dev/null 2>&1; then
  systemctl --user disable --now kache.service >/dev/null 2>&1 || true
  systemctl --user daemon-reload >/dev/null 2>&1 || true
fi
UNIT_BRIDGE="$HOME/.config/systemd/user/kache.service"
UNIT_SRC="$M/etc/systemd/user/kache.service"
[ -L "$UNIT_BRIDGE" ] && [ "$(readlink "$UNIT_BRIDGE")" = "$UNIT_SRC" ] && rm -f "$UNIT_BRIDGE"
rm -f "$UNIT_SRC"
` |
| `setting` | `component[2].remove.script` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `. /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh 2>/dev/null; nix profile remove home-manager` |
| `setting` | `component[2].requires` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `["rustup"]` |
| `setting` | `component[2].requires` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `["nix"]` |
| `setting` | `component[2].verify.args` | `manifest/ai-clis.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; VER=\"${CODEX_ALPHA_VERSION:-0.143.0-alpha.29}\"; link=\"$M/usr/bin/codex-alpha\"; dest=\"$M/.toolchains/openai-codex/${VER}/bin/codex\"; [ -x \"$dest\" ] && [ -x \"$link\" ] && grep -q \"envctl codex alpha wrapper\" \"$link\" && [ \"$(readlink -f \"$M/.local/bin/codex-alpha\" 2>/dev/null)\" = \"$(readlink -f \"$link\" 2>/dev/null)\" ] && \"$link\" --version | grep -Fq \"$VER\" && CODEX_HOME= CODEX_SQLITE_HOME= \"$link\" mcp list >/dev/null"]` |
| `setting` | `component[2].verify.args` | `manifest/base.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export PATH=\"$M/.toolchains/.bun/bin:$PATH\"; node -e 'process.exit(0)'"]` |
| `setting` | `component[2].verify.args` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export CARGO_HOME=\"$M/.toolchains/cargo\"; export PATH=\"$CARGO_HOME/bin:$PATH\"; cargo nextest --version"]` |
| `setting` | `component[2].verify.args` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `["-lc",". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh 2>/dev/null; export PATH=\"$META_ROOT/.nix-profile/bin:$PATH\"; home-manager --version"]` |
| `setting` | `component[2].verify.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
export PATH="$M/usr/bin:$PATH"
# Bypass any interactive \`grep\` shell-function wrapper (rtk/ugrep hook in login shells).
g() { command grep "$@"; }
# 1) tools resolve to the meta wrapper + real binary exists.
[ "$(command -v kache)" = "$M/usr/bin/kache" ]
[ -x "$M/usr/bin/kache" ] && g -q 'envctl kache wrapper' "$M/usr/bin/kache"
[ -x "$M/.toolchains/kache/bin/kache" ]
kache --version
# 2) the co-managed config carries the absolute rustc-wrapper wiring (not bare PATH lookup).
CFG="$M/.cargo/config.toml"
[ -f "$CFG" ] && g -q "rustc-wrapper = \"$M/usr/bin/kache\"" "$CFG"
! awk '/^# >>> envctl kache /{s=1} s&&/^# <<< envctl kache /{s=0} s&&/rustc-wrapper = "kache"/{bad=1} END{exit bad?0:1}' "$CFG"
# 3) prove kache actually wraps a real build and that the active cache lands in meta.
#    The throwaway HOME catches regressions where kache falls back to ~/.cache/kache.
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/home" "$M/.cache"
rm -f "$tmp/home/.cache/kache/index.db"
mkdir -p "$tmp/proj/src"
cat > "$tmp/proj/Cargo.toml" <<EOF
[package]
name = "kache-wire-probe"
version = "0.0.0"
edition = "2021"
[[bin]]
name = "kache-wire-probe"
path = "src/main.rs"
EOF
echo 'fn main() {}' > "$tmp/proj/src/main.rs"
( cd "$tmp/proj" && env -u XDG_CACHE_HOME HOME="$tmp/home" PATH="$M/usr/bin:$PATH" cargo build --config "$CFG" --target-dir "$tmp/td" 2>&1 )
[ -s "$M/.cache/kache/index.db" ] || { echo "kache did not record the wrapped build under $M/.cache/kache"; exit 1; }
[ ! -e "$tmp/home/.cache/kache/index.db" ] || { echo "kache leaked cache state to HOME/.cache/kache"; exit 1; }
# 4) daemon service restarts must also stay meta-owned.
if command -v systemctl >/dev/null 2>&1; then
  UNIT_BRIDGE="$HOME/.config/systemd/user/kache.service"
  UNIT_SRC="$M/etc/systemd/user/kache.service"
  [ -L "$UNIT_BRIDGE" ] && [ "$(readlink "$UNIT_BRIDGE")" = "$UNIT_SRC" ]
  g -q "ExecStart=$M/.toolchains/kache/bin/kache daemon run" "$UNIT_SRC"
  g -q "Environment=XDG_CACHE_HOME=$M/.cache" "$UNIT_SRC"
fi
kache doctor
` |
| `setting` | `component[3].description` | `manifest/base.toml` | `scope=component source_kind=manifest` | `Real Node 20-24 for tools that need V8 (n8n's isolated-vm), installed into $META_ROOT/.toolchains/node with regular $META_ROOT/usr/bin frontdoor wrappers. Bun (JSC) cannot satisfy these; this is the narrow non-bun carve-out. Bun remains the default JS runtime.` |
| `setting` | `component[3].description` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `nu from upstream musl release tarball into .toolchains/nushell + $META_ROOT/usr/bin/nu. Removes nix as the delivery path for interactive nu. Repo: nushell/nushell.` |
| `setting` | `component[3].description` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `Wasmer runtime mirrored into $META_ROOT/.toolchains/wasmer with a $META_ROOT/usr/bin frontdoor.` |
| `setting` | `component[3].description` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `Pinned nightly-2026-04-03 + rust-src/rustc-dev/llvm-tools.` |
| `setting` | `component[3].description` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `nix profile github:FlexNetOS/yazelix#yazelix; bundles nushell+mise.` |
| `setting` | `component[3].detect.args` | `manifest/ai-clis.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/.bun/bin/gemini\" ] && [ -x \"$META_ROOT/usr/bin/gemini\" ] && [ ! -L \"$META_ROOT/usr/bin/gemini\" ] && grep -Fqx \"exec \\\"$M/.toolchains/.bun/bin/gemini\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/gemini\""]` |
| `setting` | `component[3].detect.args` | `manifest/base.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/node/bin/node\" ] && [ -x \"$META_ROOT/usr/bin/node\" ] && [ ! -L \"$META_ROOT/usr/bin/node\" ] && grep -Fqx \"exec \\\"$M/.toolchains/node/bin/node\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/node\" && \"$META_ROOT/usr/bin/node\" -e 'const m=+process.versions.node.split(\".\")[0]; process.exit(m>=20 && m<=24 ? 0 : 1)'"]` |
| `setting` | `component[3].detect.args` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/nushell/nu\" ] && [ -x \"$META_ROOT/usr/bin/nu\" ] && [ ! -L \"$META_ROOT/usr/bin/nu\" ] && grep -Fqx \"exec \\\"$M/.toolchains/nushell/nu\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/nu\""]` |
| `setting` | `component[3].detect.args` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/wasmer/bin/wasmer\" ] && [ -x \"$META_ROOT/usr/bin/wasmer\" ] && [ ! -L \"$META_ROOT/usr/bin/wasmer\" ] && grep -Fqx \"exec \\\"$M/.toolchains/wasmer/bin/wasmer\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/wasmer\""]` |
| `setting` | `component[3].detect.args` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export CARGO_HOME=\"$M/.toolchains/cargo\" RUSTUP_HOME=\"$M/.toolchains/rustup\" PATH=\"$CARGO_HOME/bin:$META_ROOT/.toolchains/cargo/bin:$PATH\"; rustup toolchain list 2>/dev/null | grep -q nightly-2026-04-03"]` |
| `setting` | `component[3].detect.args` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `["-lc",". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh 2>/dev/null; export PATH=\"$META_ROOT/.nix-profile/bin:$PATH\"; command -v yzx"]` |
| `setting` | `component[3].fix.script` | `manifest/ai-clis.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -e
M="${META_ROOT:?META_ROOT required}"
export BUN_INSTALL="$M/.toolchains/.bun"
export PATH="$BUN_INSTALL/bin:$PATH"
bun install -g @google/gemini-cli
mkdir -p "$META_ROOT/usr/bin"
envctl_frontdoor "$BUN_INSTALL/bin/gemini" "$META_ROOT/usr/bin/gemini"

` |
| `setting` | `component[3].fix.script` | `manifest/base.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -e
VER=v22.22.3
ARCH="$(uname -m)"; case "$ARCH" in x86_64) A=x64;; aarch64) A=arm64;; *) A="$ARCH";; esac
M="${META_ROOT:?META_ROOT required}"
DEST="$M/.toolchains/node"
cd "$(mktemp -d)"
curl -fsSL -o node.tar.xz "https://nodejs.org/dist/$VER/node-$VER-linux-$A.tar.xz"
rm -rf "$DEST"
mkdir -p "$DEST" "$META_ROOT/usr/bin"
tar -xJf node.tar.xz --strip-components=1 -C "$DEST"
for b in node npm npx corepack; do
  [ -x "$DEST/bin/$b" ] && envctl_frontdoor "$DEST/bin/$b" "$META_ROOT/usr/bin/$b"
done

` |
| `setting` | `component[3].fix.script` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
DEST="$M/.toolchains/wasmer"
export HOME="$M"
curl -fsSL https://get.wasmer.io | sh
rm -rf "$DEST"
install -d -m 755 "$DEST" "$META_ROOT/usr/bin"
cp -a "$M/.wasmer/." "$DEST/"
envctl_frontdoor "$DEST/bin/wasmer" "$META_ROOT/usr/bin/wasmer"

` |
| `setting` | `component[3].fix.script` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `M="${META_ROOT:?META_ROOT required}"; export CARGO_HOME="$M/.toolchains/cargo" RUSTUP_HOME="$M/.toolchains/rustup" PATH="$CARGO_HOME/bin:$META_ROOT/.toolchains/cargo/bin:$PATH"; rustup component add rust-src rustc-dev llvm-tools --toolchain nightly-2026-04-03` |
| `setting` | `component[3].fix.script` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `. /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh 2>/dev/null; export PATH="$META_ROOT/.nix-profile/bin:$PATH"; nix profile upgrade yazelix 2>/dev/null || nix profile add --refresh github:FlexNetOS/yazelix#yazelix; yzx doctor || true` |
| `setting` | `component[3].id` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `rust-nightly-cuda-oxide` |
| `setting` | `component[3].install.script` | `manifest/ai-clis.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -e
M="${META_ROOT:?META_ROOT required}"
export BUN_INSTALL="$M/.toolchains/.bun"
export PATH="$BUN_INSTALL/bin:$PATH"
bun install -g @google/gemini-cli
mkdir -p "$META_ROOT/usr/bin"
envctl_frontdoor "$BUN_INSTALL/bin/gemini" "$META_ROOT/usr/bin/gemini"
` |
| `setting` | `component[3].install.script` | `manifest/base.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -e
VER=v22.22.3
ARCH="$(uname -m)"; case "$ARCH" in x86_64) A=x64;; aarch64) A=arm64;; *) A="$ARCH";; esac
M="${META_ROOT:?META_ROOT required}"
DEST="$M/.toolchains/node"
cd "$(mktemp -d)"
curl -fsSL -o node.tar.xz "https://nodejs.org/dist/$VER/node-$VER-linux-$A.tar.xz"
rm -rf "$DEST"
mkdir -p "$DEST" "$META_ROOT/usr/bin"
tar -xJf node.tar.xz --strip-components=1 -C "$DEST"
for b in node npm npx corepack; do
  [ -x "$DEST/bin/$b" ] && envctl_frontdoor "$DEST/bin/$b" "$META_ROOT/usr/bin/$b"
done

` |
| `setting` | `component[3].install.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
ROOT="$M/envctl"; source "$ROOT/assets/scripts/envctl-gh-fetch.sh"
DEST="$M/.toolchains/nushell"
TAG="$(curl -fsSLI -o /dev/null -w '%{url_effective}' https://github.com/nushell/nushell/releases/latest | sed 's#.*/tag/##')"
URL="https://github.com/nushell/nushell/releases/download/${TAG}/nu-${TAG}-x86_64-unknown-linux-musl.tar.gz"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
envctl_gh_curl "$URL" -o "$tmp/nu.tgz"
install -d -m 755 "$DEST"
tar -xzf "$tmp/nu.tgz" -C "$DEST" --strip-components=1
install -d -m 755 "$META_ROOT/usr/bin"
envctl_frontdoor "$DEST/nu" "$META_ROOT/usr/bin/nu"


` |
| `setting` | `component[3].install.script` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
DEST="$M/.toolchains/wasmer"
export HOME="$M"
curl -fsSL https://get.wasmer.io | sh
rm -rf "$DEST"
install -d -m 755 "$DEST" "$META_ROOT/usr/bin"
cp -a "$M/.wasmer/." "$DEST/"
envctl_frontdoor "$DEST/bin/wasmer" "$META_ROOT/usr/bin/wasmer"

` |
| `setting` | `component[3].install.script` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `M="${META_ROOT:?META_ROOT required}"; export CARGO_HOME="$M/.toolchains/cargo" RUSTUP_HOME="$M/.toolchains/rustup" PATH="$CARGO_HOME/bin:$META_ROOT/.toolchains/cargo/bin:$PATH"; rustup toolchain install nightly-2026-04-03; rustup component add rust-src rustc-dev llvm-tools --toolchain nightly-2026-04-03` |
| `setting` | `component[3].install.script` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `. /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh 2>/dev/null; nix profile add --refresh github:FlexNetOS/yazelix#yazelix` |
| `setting` | `component[3].name` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `Rust nightly + components (cuda-oxide)` |
| `setting` | `component[3].remove.args` | `manifest/ai-clis.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export BUN_INSTALL=\"$M/.toolchains/.bun\"; export PATH=\"$BUN_INSTALL/bin:$PATH\"; bun remove -g @google/gemini-cli; [ -L \"$META_ROOT/usr/bin/gemini\" ] && rm \"$META_ROOT/usr/bin/gemini\" || true"]` |
| `setting` | `component[3].remove.args` | `manifest/base.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; for b in node npm npx corepack; do t=\"$M/usr/bin/$b\"; src=\"$M/.toolchains/node/bin/$b\"; { [ -L \"$t\" ] && [ \"$(readlink -f \"$t\" 2>/dev/null)\" = \"$(readlink -f \"$src\" 2>/dev/null)\" ]; } || { [ -f \"$t\" ] && grep -Fqx \"exec \\\"$src\\\" \\\"\\$@\\\"\" \"$t\"; } && rm -f \"$t\" || true; done; rm -rf \"$M/.toolchains/node\""]` |
| `setting` | `component[3].remove.args` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; t=\"$M/usr/bin/wasmer\"; src=\"$M/.toolchains/wasmer/bin/wasmer\"; { [ -L \"$t\" ] && [ \"$(readlink -f \"$t\" 2>/dev/null)\" = \"$(readlink -f \"$src\" 2>/dev/null)\" ]; } || { [ -f \"$t\" ] && grep -Fqx \"exec \\\"$src\\\" \\\"\\$@\\\"\" \"$t\"; } && rm -f \"$t\" || true; rm -rf \"$M/.toolchains/wasmer\""]` |
| `setting` | `component[3].remove.args` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export CARGO_HOME=\"$M/.toolchains/cargo\" RUSTUP_HOME=\"$M/.toolchains/rustup\" PATH=\"$CARGO_HOME/bin:$META_ROOT/.toolchains/cargo/bin:$PATH\"; rustup toolchain uninstall nightly-2026-04-03"]` |
| `setting` | `component[3].remove.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `set -u
M="${META_ROOT:?META_ROOT required}"
t="$META_ROOT/usr/bin/nu"; [ -L "$t" ] && readlink "$t" | grep -q "$M/.toolchains/nushell" && rm -f "$t"
rm -rf "$M/.toolchains/nushell"
` |
| `setting` | `component[3].remove.script` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `. /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh 2>/dev/null; nix profile remove yazelix` |
| `setting` | `component[3].requires` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `["rustup"]` |
| `setting` | `component[3].requires` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `["nix","nix-yazelix-cache","home-manager","ghostty"]` |
| `setting` | `component[3].verify.args` | `manifest/ai-clis.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export BUN_INSTALL=\"$M/.toolchains/.bun\"; export PATH=\"$META_ROOT/usr/bin:$BUN_INSTALL/bin:$PATH\"; gemini --version"]` |
| `setting` | `component[3].verify.args` | `manifest/base.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export PATH=\"$M/usr/bin:$PATH\"; [ -x \"$M/usr/bin/node\" ] && [ ! -L \"$M/usr/bin/node\" ] && grep -Fqx \"exec \\\"$M/.toolchains/node/bin/node\\\" \\\"\\$@\\\"\" \"$M/usr/bin/node\" && node -e 'const m=+process.versions.node.split(\".\")[0]; process.exit(m>=20 && m<=24 ? 0 : 1)'"]` |
| `setting` | `component[3].verify.args` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export PATH=\"$M/usr/bin:$PATH\"; [ -x \"$M/usr/bin/wasmer\" ] && [ ! -L \"$M/usr/bin/wasmer\" ] && grep -Fqx \"exec \\\"$M/.toolchains/wasmer/bin/wasmer\\\" \\\"\\$@\\\"\" \"$M/usr/bin/wasmer\" && wasmer --version"]` |
| `setting` | `component[3].verify.args` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export CARGO_HOME=\"$M/.toolchains/cargo\" RUSTUP_HOME=\"$M/.toolchains/rustup\" PATH=\"$CARGO_HOME/bin:$META_ROOT/.toolchains/cargo/bin:$PATH\"; rustup component list --toolchain nightly-2026-04-03 --installed | grep -q rust-src"]` |
| `setting` | `component[3].verify.args` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `["-lc",". /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh 2>/dev/null; export PATH=\"$META_ROOT/.nix-profile/bin:$PATH\"; yzx --version 2>/dev/null || command -v yzx"]` |
| `setting` | `component[4].description` | `manifest/apt-base.toml` | `scope=component source_kind=manifest` | `OpenSSL development headers + pkg-config metadata, so external tools that link system OpenSSL (grit's aws/azure SDKs → openssl-sys) can build. System package; not in the envctl trust boundary.` |
| `setting` | `component[4].description` | `manifest/base.toml` | `scope=component source_kind=manifest` | `Rust toolchain manager (cargo/rustc). Foundation for rtk + cuda-oxide.` |
| `setting` | `component[4].description` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `zellij from upstream musl release tarball into .toolchains/zellij + $META_ROOT/usr/bin/zellij. Removes nix as the delivery path for interactive zellij. Repo: zellij-org/zellij.` |
| `setting` | `component[4].description` | `manifest/components.d/portability-links.toml` | `scope=component source_kind=manifest` | `Compatibility id only. The old per-tool real-home bin/cargo link farm is retired; binaries must be installed directly in META_ROOT-owned prefixes.` |
| `setting` | `component[4].detect.args` | `manifest/base.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export CARGO_HOME=\"$M/.toolchains/cargo\" RUSTUP_HOME=\"$M/.toolchains/rustup\" PATH=\"$CARGO_HOME/bin:$META_ROOT/.toolchains/cargo/bin:$PATH\"; rustup --version >/dev/null 2>&1 && cargo --version >/dev/null 2>&1 && rustc --version >/dev/null 2>&1 && rustup default 2>/dev/null | grep -q '^nightly'"]` |
| `setting` | `component[4].detect.args` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/zellij/zellij\" ] && [ -x \"$META_ROOT/usr/bin/zellij\" ] && [ ! -L \"$META_ROOT/usr/bin/zellij\" ] && grep -Fqx \"exec \\\"$M/.toolchains/zellij/zellij\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/zellij\""]` |
| `setting` | `component[4].detect.args` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/uv/bin/uv\" ] && [ -x \"$M/.toolchains/uv/bin/uvx\" ] && [ -x \"$M/usr/bin/uv\" ] && [ ! -L \"$M/usr/bin/uv\" ] && grep -Fqx \"exec \\\"$M/.toolchains/uv/bin/uv\\\" \\\"\\$@\\\"\" \"$M/usr/bin/uv\" && [ -x \"$M/usr/bin/uvx\" ] && [ ! -L \"$M/usr/bin/uvx\" ] && grep -Fqx \"exec \\\"$M/.toolchains/uv/bin/uvx\\\" \\\"\\$@\\\"\" \"$M/usr/bin/uvx\" && [ -x \"$M/usr/bin/python3\" ] && [ ! -L \"$M/usr/bin/python3\" ] && grep -Fq \"exec \\\"$M/.toolchains/uv/python/\" \"$M/usr/bin/python3\""]` |
| `setting` | `component[4].detect.args` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/cargo/bin/cargo-oxide\" ] && [ -x \"$META_ROOT/usr/bin/cargo-oxide\" ] && [ ! -L \"$META_ROOT/usr/bin/cargo-oxide\" ] && grep -Fqx \"exec \\\"$M/.toolchains/cargo/bin/cargo-oxide\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/cargo-oxide\""]` |
| `setting` | `component[4].fix.script` | `manifest/base.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
export CARGO_HOME="$M/.toolchains/cargo"
export RUSTUP_HOME="$M/.toolchains/rustup"
mkdir -p "$CARGO_HOME" "$RUSTUP_HOME" "$META_ROOT/.toolchains/cargo/bin"
if [ ! -x "$CARGO_HOME/bin/rustup" ]; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --default-toolchain nightly
fi
export PATH="$CARGO_HOME/bin:$META_ROOT/.toolchains/cargo/bin:$PATH"
rustup self update
# Keep the meta default on the LATEST nightly (owner directive: always latest toolchain).
rustup update nightly
rustup default nightly
for tool in cargo cargo-clippy cargo-fmt cargo-miri clippy-driver rls rust-analyzer rust-gdb rust-gdbgui rust-lldb rustc rustdoc rustfmt rustup; do
  ln -sfn "$CARGO_HOME/bin/rustup" "$META_ROOT/.toolchains/cargo/bin/$tool"
done
` |
| `setting` | `component[4].fix.script` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
DEST="$M/.toolchains/uv/bin"
install -d -m 755 "$DEST" "$META_ROOT/usr/bin"
export PATH="$DEST:$PATH"
uv self update || curl -LsSf https://astral.sh/uv/install.sh | env UV_INSTALL_DIR="$DEST" sh
envctl_frontdoor "$DEST/uv" "$META_ROOT/usr/bin/uv"
envctl_frontdoor "$DEST/uvx" "$META_ROOT/usr/bin/uvx"
export UV_PYTHON_INSTALL_DIR="$M/.toolchains/uv/python"
"$DEST/uv" python install 3.14 --preview || "$DEST/uv" python install 3.13
# Select the python3 INTERPRETER (the canonical bin/python3), never a *-config script or the bare
# versioned binary; the old \`*/bin/python3*\` -type f glob dropped the python3 symlink and let
# \`python3.14-config\` sort last, silently linking python3 to a config script. Verify the pick imports
# its stdlib before linking, so a broken/partial install can never leave python3 pointing at a
# non-working interpreter. Link into the meta-local prefix ($META_ROOT), never $HOME (meta-local policy).
py="$(find "$UV_PYTHON_INSTALL_DIR" -path '*/bin/python3' | sort -V | tail -n1)"
[ -n "$py" ] && "$py" -c 'import encodings' 2>/dev/null && envctl_frontdoor "$py" "$META_ROOT/usr/bin/python3"

` |
| `setting` | `component[4].fix.script` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
M="${META_ROOT:?META_ROOT required}"; export CARGO_HOME="$M/.toolchains/cargo" RUSTUP_HOME="$M/.toolchains/rustup" PATH="$CARGO_HOME/bin:$META_ROOT/.toolchains/cargo/bin:$PATH"; cargo +nightly-2026-04-03 install --force --git https://github.com/NVlabs/cuda-oxide.git cargo-oxide; install -d -m 755 "$META_ROOT/usr/bin"; envctl_frontdoor "$CARGO_HOME/bin/cargo-oxide" "$META_ROOT/usr/bin/cargo-oxide"
` |
| `setting` | `component[4].fix.script` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `export PATH="$META_ROOT/.nix-profile/bin:$PATH"; yzx desktop install` |
| `setting` | `component[4].id` | `manifest/base.toml` | `scope=component source_kind=manifest` | `rustup` |
| `setting` | `component[4].install.script` | `manifest/base.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
export CARGO_HOME="$M/.toolchains/cargo"
export RUSTUP_HOME="$M/.toolchains/rustup"
mkdir -p "$CARGO_HOME" "$RUSTUP_HOME" "$META_ROOT/.toolchains/cargo/bin"
# Owner standing directive: the workstation/meta Rust default is ALWAYS nightly (latest).
# A repo with its own rust-toolchain.toml (e.g. envctl pinned 1.96.0) still overrides this for
# its own builds — the global default does not change a repo's pinned channel.
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --default-toolchain nightly
export PATH="$CARGO_HOME/bin:$META_ROOT/.toolchains/cargo/bin:$PATH"
rustup default nightly
for tool in cargo cargo-clippy cargo-fmt cargo-miri clippy-driver rls rust-analyzer rust-gdb rust-gdbgui rust-lldb rustc rustdoc rustfmt rustup; do
  ln -sfn "$CARGO_HOME/bin/rustup" "$META_ROOT/.toolchains/cargo/bin/$tool"
done
` |
| `setting` | `component[4].install.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
ROOT="$M/envctl"; source "$ROOT/assets/scripts/envctl-gh-fetch.sh"
DEST="$M/.toolchains/zellij"
TAG="$(curl -fsSLI -o /dev/null -w '%{url_effective}' https://github.com/zellij-org/zellij/releases/latest | sed 's#.*/tag/##')"
URL="https://github.com/zellij-org/zellij/releases/download/${TAG}/zellij-x86_64-unknown-linux-musl.tar.gz"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
envctl_gh_curl "$URL" -o "$tmp/zellij.tgz"
install -d -m 755 "$DEST"
tar -xzf "$tmp/zellij.tgz" -C "$DEST"
install -d -m 755 "$META_ROOT/usr/bin"
envctl_frontdoor "$DEST/zellij" "$META_ROOT/usr/bin/zellij"


` |
| `setting` | `component[4].install.script` | `manifest/components.d/portability-links.toml` | `scope=component source_kind=manifest` | `echo "meta-tool-links retired: install binaries into META_ROOT/usr/bin, META_ROOT/opt, or legacy META_ROOT/.toolchains"
` |
| `setting` | `component[4].install.script` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
DEST="$M/.toolchains/uv/bin"
install -d -m 755 "$DEST" "$META_ROOT/usr/bin"
curl -LsSf https://astral.sh/uv/install.sh | env UV_INSTALL_DIR="$DEST" sh
envctl_frontdoor "$DEST/uv" "$META_ROOT/usr/bin/uv"
envctl_frontdoor "$DEST/uvx" "$META_ROOT/usr/bin/uvx"
export UV_PYTHON_INSTALL_DIR="$M/.toolchains/uv/python"
"$DEST/uv" python install 3.14 --preview || "$DEST/uv" python install 3.13
# Select the python3 INTERPRETER (the canonical bin/python3), never a *-config script or the bare
# versioned binary; the old \`*/bin/python3*\` -type f glob dropped the python3 symlink and let
# \`python3.14-config\` sort last, silently linking python3 to a config script. Verify the pick imports
# its stdlib before linking, so a broken/partial install can never leave python3 pointing at a
# non-working interpreter. Link into the meta-local prefix ($META_ROOT), never $HOME (meta-local policy).
py="$(find "$UV_PYTHON_INSTALL_DIR" -path '*/bin/python3' | sort -V | tail -n1)"
[ -n "$py" ] && "$py" -c 'import encodings' 2>/dev/null && envctl_frontdoor "$py" "$META_ROOT/usr/bin/python3"

` |
| `setting` | `component[4].install.script` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
M="${META_ROOT:?META_ROOT required}"; export CARGO_HOME="$M/.toolchains/cargo" RUSTUP_HOME="$M/.toolchains/rustup" PATH="$CARGO_HOME/bin:$META_ROOT/.toolchains/cargo/bin:$PATH"; cargo +nightly-2026-04-03 install --git https://github.com/NVlabs/cuda-oxide.git cargo-oxide; install -d -m 755 "$META_ROOT/usr/bin"; envctl_frontdoor "$CARGO_HOME/bin/cargo-oxide" "$META_ROOT/usr/bin/cargo-oxide"
` |
| `setting` | `component[4].install.script` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `. /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh 2>/dev/null; export PATH="$META_ROOT/.nix-profile/bin:$PATH"; yzx desktop install` |
| `setting` | `component[4].name` | `manifest/base.toml` | `scope=component source_kind=manifest` | `rustup toolchain` |
| `setting` | `component[4].name` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `uv (Python toolchain)` |
| `setting` | `component[4].name` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `cuda-oxide (cargo-oxide)` |
| `setting` | `component[4].remove.args` | `manifest/base.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export CARGO_HOME=\"$M/.toolchains/cargo\" RUSTUP_HOME=\"$M/.toolchains/rustup\" PATH=\"$CARGO_HOME/bin:$META_ROOT/.toolchains/cargo/bin:$PATH\"; rustup self uninstall -y"]` |
| `setting` | `component[4].remove.args` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; for b in uv uvx; do t=\"$M/usr/bin/$b\"; src=\"$M/.toolchains/uv/bin/$b\"; { [ -L \"$t\" ] && [ \"$(readlink -f \"$t\" 2>/dev/null)\" = \"$(readlink -f \"$src\" 2>/dev/null)\" ]; } || { [ -f \"$t\" ] && grep -Fqx \"exec \\\"$src\\\" \\\"\\$@\\\"\" \"$t\"; } && rm -f \"$t\" || true; done; t=\"$M/usr/bin/python3\"; { [ -L \"$t\" ] && readlink -f \"$t\" 2>/dev/null | grep -q \"$M/.toolchains/uv/python/\"; } || { [ -f \"$t\" ] && grep -Fq \"exec \\\"$M/.toolchains/uv/python/\" \"$t\"; } && rm -f \"$t\" || true; rm -rf \"$M/.toolchains/uv\""]` |
| `setting` | `component[4].remove.args` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export CARGO_HOME=\"$M/.toolchains/cargo\" RUSTUP_HOME=\"$M/.toolchains/rustup\" PATH=\"$CARGO_HOME/bin:$PATH\"; cargo uninstall cargo-oxide || true; t=\"$M/usr/bin/cargo-oxide\"; src=\"$CARGO_HOME/bin/cargo-oxide\"; { [ -L \"$t\" ] && [ \"$(readlink -f \"$t\" 2>/dev/null)\" = \"$src\" ]; } || { [ -f \"$t\" ] && grep -Fqx \"exec \\\"$src\\\" \\\"\\$@\\\"\" \"$t\"; } && rm -f \"$t\" || true"]` |
| `setting` | `component[4].remove.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `set -u
M="${META_ROOT:?META_ROOT required}"
t="$META_ROOT/usr/bin/zellij"; [ -L "$t" ] && readlink "$t" | grep -q "$M/.toolchains/zellij" && rm -f "$t"
rm -rf "$M/.toolchains/zellij"
` |
| `setting` | `component[4].remove.script` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `export PATH="$META_ROOT/.nix-profile/bin:$PATH"; yzx desktop uninstall 2>/dev/null || rm -f "$META_ROOT/.local/share/applications/"*[yY]azelix*.desktop` |
| `setting` | `component[4].requires` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `["rust-nightly-cuda-oxide","cuda-toolkit","llvm-clang","nvidia-open"]` |
| `setting` | `component[4].verify.args` | `manifest/base.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export CARGO_HOME=\"$M/.toolchains/cargo\" RUSTUP_HOME=\"$M/.toolchains/rustup\" PATH=\"$CARGO_HOME/bin:$META_ROOT/.toolchains/cargo/bin:$PATH\"; rustup --version && rustc --version && cargo --version"]` |
| `setting` | `component[4].verify.args` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export PATH=\"$M/usr/bin:$PATH\"; [ -x \"$M/usr/bin/uv\" ] && [ ! -L \"$M/usr/bin/uv\" ] && grep -Fqx \"exec \\\"$M/.toolchains/uv/bin/uv\\\" \\\"\\$@\\\"\" \"$M/usr/bin/uv\" && [ -x \"$M/usr/bin/python3\" ] && [ ! -L \"$M/usr/bin/python3\" ] && grep -Fq \"exec \\\"$M/.toolchains/uv/python/\" \"$M/usr/bin/python3\" && uv --version && python3 --version"]` |
| `setting` | `component[4].verify.args` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export CARGO_HOME=\"$M/.toolchains/cargo\" RUSTUP_HOME=\"$M/.toolchains/rustup\" PATH=\"$META_ROOT/usr/bin:$CARGO_HOME/bin:$PATH\"; [ -x \"$META_ROOT/usr/bin/cargo-oxide\" ] && [ ! -L \"$META_ROOT/usr/bin/cargo-oxide\" ] && grep -Fqx \"exec \\\"$CARGO_HOME/bin/cargo-oxide\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/cargo-oxide\" && cargo-oxide --version"]` |
| `setting` | `component[5].description` | `manifest/base.toml` | `scope=component source_kind=manifest` | `cargo-installed CLI; first run: rtk gain ; rtk init -g.` |
| `setting` | `component[5].description` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `mise static binary into .toolchains/mise/bin + $META_ROOT/usr/bin/mise. MISE_DATA_DIR already meta. Repo: jdx/mise.` |
| `setting` | `component[5].description` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `Meta-owned mlua-backed Lua 5.4 interpreter (lua-mlua) built vendored into .toolchains/mlua — rust-native, zero system-depth. The scriptable substrate for the Rust+mlua automation layer.` |
| `setting` | `component[5].detect.args` | `manifest/base.toml` | `scope=component source_kind=manifest` | `["-lc","command -v rtk || [ -x \"$META_ROOT/usr/bin/rtk\" ] || [ -x \"$META_ROOT/.toolchains/cargo/bin/rtk\" ]"]` |
| `setting` | `component[5].detect.args` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/mise/bin/mise\" ] && [ -x \"$META_ROOT/usr/bin/mise\" ] && [ ! -L \"$META_ROOT/usr/bin/mise\" ] && grep -Fqx \"exec \\\"$M/.toolchains/mise/bin/mise\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/mise\""]` |
| `setting` | `component[5].detect.args` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/mlua/target/release/lua-mlua\" ]"]` |
| `setting` | `component[5].fix.script` | `manifest/base.toml` | `scope=component source_kind=manifest` | `export PATH="$META_ROOT/.toolchains/cargo/bin:$PATH"; cargo install --force --git https://github.com/rtk-ai/rtk` |
| `setting` | `component[5].fix.script` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
export CARGO_HOME="$M/.toolchains/cargo"
export PATH="$CARGO_HOME/bin:$META_ROOT/.toolchains/cargo/bin:$PATH"
cargo build --release --manifest-path "$M/.toolchains/mlua/src/Cargo.toml" --target-dir "$M/.toolchains/mlua/target"
envctl_frontdoor "$M/.toolchains/mlua/target/release/lua-mlua" "$META_ROOT/usr/bin/lua-mlua"

` |
| `setting` | `component[5].install.script` | `manifest/base.toml` | `scope=component source_kind=manifest` | `export PATH="$META_ROOT/.toolchains/cargo/bin:$PATH"; cargo install --git https://github.com/rtk-ai/rtk` |
| `setting` | `component[5].install.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
ROOT="$M/envctl"; source "$ROOT/assets/scripts/envctl-gh-fetch.sh"
DEST="$M/.toolchains/mise"
TAG="$(curl -fsSLI -o /dev/null -w '%{url_effective}' https://github.com/jdx/mise/releases/latest | sed 's#.*/tag/##')"
URL="https://github.com/jdx/mise/releases/download/${TAG}/mise-${TAG}-linux-x64"
install -d -m 755 "$DEST/bin"
envctl_gh_curl "$URL" -o "$DEST/bin/mise"
chmod +x "$DEST/bin/mise"
install -d -m 755 "$META_ROOT/usr/bin"
envctl_frontdoor "$DEST/bin/mise" "$META_ROOT/usr/bin/mise"


` |
| `setting` | `component[5].install.script` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
DEST="$M/.toolchains/mlua"; SRC="$DEST/src"
install -d -m 755 "$SRC/src"
cat > "$SRC/Cargo.toml" <<'TOML'
[package]
name = "lua_mlua"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "lua-mlua"
path = "src/main.rs"

[dependencies]
mlua = { version = "0.10", features = ["lua54", "vendored"] }

[profile.release]
strip = true
opt-level = "z"

# Standalone: keep this out of the parent meta Cargo workspace (it lives under $META_ROOT).
[workspace]
TOML
cat > "$SRC/src/main.rs" <<'RS'
//! Minimal mlua-backed Lua 5.4 interpreter (vendored, zero system-depth).
//! Usage: lua-mlua -e "<code>" | lua-mlua <file.lua> | echo "<code>" | lua-mlua
use std::io::Read;
fn main() -> mlua::Result<()> {
    let lua = mlua::Lua::new();
    let args: Vec<String> = std::env::args().skip(1).collect();
    // Name the chunk so Lua errors point at the user's source, not the loader call site.
    // Lua convention: \`@path\` renders as the file path; \`=label\` renders verbatim.
    let (code, name) = if args.first().map(|s| s == "-e").unwrap_or(false) {
        (args.get(1).cloned().unwrap_or_default(), "=(command line)".to_string())
    } else if let Some(path) = args.first() {
        (std::fs::read_to_string(path)?, format!("@{path}"))
    } else {
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        (s, "=stdin".to_string())
    };
    lua.load(&code).set_name(name).exec()
}
RS
export CARGO_HOME="$M/.toolchains/cargo"
export PATH="$CARGO_HOME/bin:$META_ROOT/.toolchains/cargo/bin:$PATH"
cargo build --release --manifest-path "$SRC/Cargo.toml" --target-dir "$DEST/target"
install -d -m 755 "$META_ROOT/usr/bin"
envctl_frontdoor "$DEST/target/release/lua-mlua" "$META_ROOT/usr/bin/lua-mlua"

` |
| `setting` | `component[5].name` | `manifest/base.toml` | `scope=component source_kind=manifest` | `rtk (Rust Token Killer)` |
| `setting` | `component[5].name` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `mlua (Lua-in-Rust, vendored Lua 5.4)` |
| `setting` | `component[5].remove.args` | `manifest/base.toml` | `scope=component source_kind=manifest` | `["-lc","export PATH=\"$META_ROOT/.toolchains/cargo/bin:$PATH\"; cargo uninstall rtk"]` |
| `setting` | `component[5].remove.args` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; t=\"$M/usr/bin/lua-mlua\"; src=\"$M/.toolchains/mlua/target/release/lua-mlua\"; { [ -L \"$t\" ] && [ \"$(readlink -f \"$t\" 2>/dev/null)\" = \"$(readlink -f \"$src\" 2>/dev/null)\" ]; } || { [ -f \"$t\" ] && grep -Fqx \"exec \\\"$src\\\" \\\"\\$@\\\"\" \"$t\"; } && rm -f \"$t\" || true; rm -rf \"$M/.toolchains/mlua\"; true"]` |
| `setting` | `component[5].remove.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `set -u
M="${META_ROOT:?META_ROOT required}"
t="$META_ROOT/usr/bin/mise"; [ -L "$t" ] && readlink "$t" | grep -q "$M/.toolchains/mise" && rm -f "$t"
rm -rf "$M/.toolchains/mise"
` |
| `setting` | `component[5].requires` | `manifest/base.toml` | `scope=component source_kind=manifest` | `["rustup"]` |
| `setting` | `component[5].requires` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `["rustup","llvm-clang"]` |
| `setting` | `component[5].verify.args` | `manifest/base.toml` | `scope=component source_kind=manifest` | `["-lc","export PATH=\"$META_ROOT/.toolchains/cargo/bin:$PATH\"; rtk --version"]` |
| `setting` | `component[5].verify.args` | `manifest/dev-tools.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; \"$M/.toolchains/mlua/target/release/lua-mlua\" -e 'assert(6*7==42); print(\"mlua ok\")'"]` |
| `setting` | `component[5].wiring.shell_rc[0].content` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `if [ -e /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh ]; then
  . /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh
fi
export PATH="$META_ROOT/.nix-profile/bin:$PATH"
if [[ $- == *i* ]] && [[ -z "${ZELLIJ:-}" ]] && [[ -z "${YAZELIX_ACTIVE:-}" ]] && [[ "${TERM:-dumb}" != "dumb" ]] && command -v yzx >/dev/null 2>&1; then
  export YAZELIX_ACTIVE=1
  # Bring the meta /usr mirror onto PATH/LD_LIBRARY_PATH BEFORE re-exec'ing into
  # zellij/nushell, so the yazelix panes inherit usr/bin regardless of where the
  # standalone \`meta toolchain path\` block lands in this file (it trails this one).
  command -v envctl >/dev/null 2>&1 && eval "$(envctl env --toolchains)"
  yzx enter
fi
` |
| `setting` | `component[6].description` | `manifest/ai-clis.toml` | `scope=component source_kind=manifest` | `Meta: claude + codex + gemini + kimi + devin (Codex Rust release/toolchain; Gemini via Bun).` |
| `setting` | `component[6].description` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `ollama LLM runner from upstream zstd release tarball into .toolchains/ollama (bin/ollama + lib/ollama GPU runners) + $META_ROOT/usr/bin/ollama wrapper that forces OLLAMA_MODELS=$META_ROOT/var/lib/ollama/models. Replaces the root /usr/local/bin/ollama on PATH. Repo: ollama/ollama.` |
| `setting` | `component[6].detect.args` | `manifest/ai-clis.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; export PATH=\"$META_ROOT/usr/bin:$M/.toolchains/.bun/bin:$PATH\"; for c in claude codex gemini kimi devin; do command -v $c >/dev/null || exit 1; done"]` |
| `setting` | `component[6].detect.args` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; w=\"$META_ROOT/usr/bin/ollama\"; models=\"$M/var/lib/ollama/models\"; [ -x \"$M/.toolchains/ollama/bin/ollama\" ] && [ -d \"$models\" ] && [ -x \"$w\" ] && [ ! -L \"$w\" ] && grep -F 'export OLLAMA_MODELS=\"$M/var/lib/ollama/models\"' \"$w\" >/dev/null && grep -F 'exec \"$M/.toolchains/ollama/bin/ollama\" \"$@\"' \"$w\" >/dev/null"]` |
| `setting` | `component[6].install.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
ROOT="$M/envctl"; source "$ROOT/assets/scripts/envctl-gh-fetch.sh"
DEST="$M/.toolchains/ollama"
MODEL_STORE="$M/var/lib/ollama/models"
WRAPPER="$META_ROOT/usr/bin/ollama"

migrate_ollama_models() {
  # Non-destructive one-time adoption: only copy when the meta store is still empty,
  # and never delete or move a legacy/root daemon store behind envctl's back.
  if [ -n "$(find "$MODEL_STORE" -mindepth 1 -maxdepth 1 -print -quit 2>/dev/null)" ]; then
    return 0
  fi

  for src in \
    "$M/.ollama/models" \
    "${ENVCTL_REAL_HOME:-}/.ollama/models" \
    "/usr/share/ollama/.ollama/models" \
    "/var/lib/ollama/.ollama/models" \
    "/root/.ollama/models"
  do
    [ -n "$src" ] || continue
    [ "$src" = "$MODEL_STORE" ] && continue
    [ -d "$src" ] || continue
    if [ -r "$src" ] && [ -x "$src" ]; then
      cp -a "$src/." "$MODEL_STORE/"
      echo "ollama: copied existing model blobs from $src into $MODEL_STORE (source preserved)"
      return 0
    fi
    echo "ollama: model store exists but is not readable by envctl: $src (source preserved; rerun with access if needed)" >&2
  done
}

write_ollama_wrapper() {
  cat >"$WRAPPER" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
M="${META_ROOT:-}"
if [ -z "$M" ]; then
  script="$(readlink -f "${BASH_SOURCE[0]}")"
  M="$(cd "$(dirname "$script")/../.." && pwd -P)"
fi
export META_ROOT="$M"
export OLLAMA_MODELS="$M/var/lib/ollama/models"
export OLLAMA_LIBRARY_PATH="$M/.toolchains/ollama/lib/ollama${OLLAMA_LIBRARY_PATH:+:$OLLAMA_LIBRARY_PATH}"
exec "$M/.toolchains/ollama/bin/ollama" "$@"
EOF
  chmod 755 "$WRAPPER"
}

TAG="$(curl -fsSLI -o /dev/null -w '%{url_effective}' https://github.com/ollama/ollama/releases/latest | sed 's#.*/tag/##')"
URL="https://github.com/ollama/ollama/releases/download/${TAG}/ollama-linux-amd64.tar.zst"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
envctl_gh_curl "$URL" -o "$tmp/ollama.tar.zst"
rm -rf "$DEST"
install -d -m 755 "$DEST" "$MODEL_STORE" "$META_ROOT/usr/bin"
tar --zstd -xf "$tmp/ollama.tar.zst" -C "$DEST"
migrate_ollama_models
write_ollama_wrapper
` |
| `setting` | `component[6].remove.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `set -u
M="${META_ROOT:?META_ROOT required}"
t="$META_ROOT/usr/bin/ollama"
if [ -f "$t" ] && grep -q "$M/.toolchains/ollama/bin/ollama" "$t" 2>/dev/null; then
  rm -f "$t"
elif [ -L "$t" ] && readlink "$t" | grep -q "$M/.toolchains/ollama"; then
  rm -f "$t"
fi
rm -rf "$M/.toolchains/ollama"
echo "ollama: preserved model blobs at $M/var/lib/ollama/models" >&2
` |
| `setting` | `component[6].verify.args` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `["-lc","set -euo pipefail; M=\"${META_ROOT:?META_ROOT required}\"; w=\"$META_ROOT/usr/bin/ollama\"; [ -d \"$M/var/lib/ollama/models\" ]; grep -F 'export OLLAMA_MODELS=\"$M/var/lib/ollama/models\"' \"$w\" >/dev/null; grep -F 'exec \"$M/.toolchains/ollama/bin/ollama\" \"$@\"' \"$w\" >/dev/null; PATH=\"$META_ROOT/usr/bin:$PATH\" env -u OLLAMA_MODELS ollama -v"]` |
| `setting` | `component[7].description` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `LLVM/clang-21 from upstream prebuilt release tarball into .toolchains/llvm + regular $META_ROOT/usr/bin frontdoor wrappers for the binaries whose runtime libs resolve on this host (clang/clang++/llc/llvm-* always; lld/ld.lld only where libxml2.so.2 exists). Replaces the apt clang/llvm. Pins latest 21.x (the /latest redirect now points at 22.x). Repo: llvm/llvm-project.` |
| `setting` | `component[7].detect.args` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/llvm/bin/clang\" ] && [ -x \"$M/usr/bin/clang\" ] && [ ! -L \"$M/usr/bin/clang\" ] && grep -Fqx \"exec \\\"$M/.toolchains/llvm/bin/clang\\\" \\\"\\$@\\\"\" \"$M/usr/bin/clang\""]` |
| `setting` | `component[7].install.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
ROOT="$M/envctl"; source "$ROOT/assets/scripts/envctl-gh-fetch.sh"
DEST="$M/.toolchains/llvm"
# Authenticated release listing via the shared resolver (gh tier = 5000/hr, or the vault-sealed
# mint token; gh is the meta-owned Epic-H component TASK-0057, guaranteed on PATH) — the
# /releases/latest redirect points at 22.x, so we must LIST releases to pin the latest 21.x.
# The resolver's gh path preserves \`--jq\`; we keep our OWN unauthenticated grep-based fallback
# for when gh is absent/unauthed (60/hr, may 403 when the quota is exhausted), because the
# curl-side cannot honour gh's --jq. TASK-0068 (last api.github.com rate-limit liability).
if _envctl_gh_authed; then
  TAG="$(envctl_gh_api 'repos/llvm/llvm-project/releases?per_page=100' \
    --jq '.[].tag_name | select(test("^llvmorg-21\\.[0-9]+\\.[0-9]+$"))' | sort -V | tail -1)"
else
  TAG="$(envctl_gh_curl 'https://api.github.com/repos/llvm/llvm-project/releases?per_page=100' \
    | grep -oE '"tag_name": *"llvmorg-21\.[0-9]+\.[0-9]+"' \
    | grep -oE 'llvmorg-21\.[0-9]+\.[0-9]+' | sort -V | tail -1)"
fi
[ -n "$TAG" ] || { echo "llvm: could not resolve latest 21.x tag" >&2; exit 1; }
VER="${TAG#llvmorg-}"
URL="https://github.com/llvm/llvm-project/releases/download/${TAG}/LLVM-${VER}-Linux-X64.tar.xz"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
envctl_gh_curl "$URL" -o "$tmp/llvm.tar.xz"
rm -rf "$DEST"; install -d -m 755 "$DEST"
tar -xJf "$tmp/llvm.tar.xz" -C "$DEST" --strip-components=1
install -d -m 755 "$META_ROOT/usr/bin"
# only expose regular frontdoor wrappers for binaries whose runtime libs resolve on this host
# (the prebuilt lld links libxml2.so.2, absent on libxml2.so.16 boxes); the box's strategic linker is wild
# (TASK-0054), and apt lld remains as fallback until apt removal. The \`--version\` probe
# self-prunes any tarball tool with an unsatisfied shared lib, not just lld.
for b in clang clang++ clang-cpp clang-21 llvm-config llc llvm-ar llvm-nm llvm-objcopy llvm-objdump lld ld.lld; do
  src="$DEST/bin/$b"; dst="$META_ROOT/usr/bin/$b"
  if [ -e "$src" ] && "$src" --version >/dev/null 2>&1; then
    envctl_frontdoor "$src" "$dst"
  else
    # prune only our prior exposure for a now-unexposed binary (e.g. a prior install that
    # exposed lld before libxml2.so.2 went away, or a re-install). Accept both the new
    # regular wrapper and the legacy managed symlink; never remove a foreign/apt binary.
    if { [ -L "$dst" ] && readlink "$dst" | grep -q "$DEST"; } || { [ -f "$dst" ] && grep -Fqx "exec \"$src\" \"\$@\"" "$dst"; }; then
      rm -f "$dst"
    fi
  fi
done


` |
| `setting` | `component[7].remove.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `set -u
M="${META_ROOT:?META_ROOT required}"
managed_frontdoor(){ front="$1"; private="$2"; if [ -L "$front" ] && [ "$(readlink -f "$front" 2>/dev/null || true)" = "$(readlink -f "$private" 2>/dev/null || true)" ]; then return 0; fi; [ -f "$front" ] && grep -Fqx "exec \"$private\" \"\$@\"" "$front"; }
for b in clang clang++ clang-cpp clang-21 llvm-config llc llvm-ar llvm-nm llvm-objcopy llvm-objdump lld ld.lld; do
  t="$META_ROOT/usr/bin/$b"; src="$M/.toolchains/llvm/bin/$b"; if managed_frontdoor "$t" "$src"; then rm -f "$t"; fi
done
rm -rf "$M/.toolchains/llvm"
` |
| `setting` | `component[8].description` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `Prebuilt libgccjit.so for the rustc_codegen_gcc backend, from the rust-lang/gcc release pinned by rustc_codegen_gcc's libgccjit.version, into .toolchains/libgccjit/lib. Runtime .so (no CLI binary, no $META_ROOT/usr/bin frontdoor); exposed via the GCC_PATH env seam. Replaces a system GCC/libgccjit build. Repo: rust-lang/gcc.` |
| `setting` | `component[8].detect.args` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -f \"$M/.toolchains/libgccjit/lib/libgccjit.so\" ]"]` |
| `setting` | `component[8].detect.args` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/cuda/bin/nvcc\" ] && command -v nvidia-smi >/dev/null && [ -x \"$M/.toolchains/cargo/bin/cargo-oxide\" ] && [ -x \"$META_ROOT/usr/bin/cargo-oxide\" ] && [ ! -L \"$META_ROOT/usr/bin/cargo-oxide\" ] && grep -Fqx \"exec \\\"$M/.toolchains/cargo/bin/cargo-oxide\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/cargo-oxide\""]` |
| `setting` | `component[8].detect.args` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `["-lc","command -v nix >/dev/null && bash -lc 'command -v yzx' >/dev/null 2>&1"]` |
| `setting` | `component[8].id` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `group-nix-yazelix` |
| `setting` | `component[8].install.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
ROOT="$M/envctl"; source "$ROOT/assets/scripts/envctl-gh-fetch.sh"
DEST="$M/.toolchains/libgccjit"
COMMIT="$(envctl_gh_curl https://raw.githubusercontent.com/rust-lang/rustc_codegen_gcc/master/libgccjit.version | tr -d '[:space:]')"
[ -n "$COMMIT" ] || { echo "libgccjit: could not resolve commit from libgccjit.version" >&2; exit 1; }
URL="https://github.com/rust-lang/gcc/releases/download/master-${COMMIT}/libgccjit.so"
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
envctl_gh_curl "$URL" -o "$tmp/libgccjit.so"
rm -rf "$DEST"; install -d -m 755 "$DEST/lib"
install -m 644 "$tmp/libgccjit.so" "$DEST/lib/libgccjit.so"
ln -sfn "$DEST/lib/libgccjit.so" "$DEST/lib/libgccjit.so.0"
` |
| `setting` | `component[8].name` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `Full Nix + yazelix stack` |
| `setting` | `component[8].remove.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `set -u
M="${META_ROOT:?META_ROOT required}"
rm -rf "$M/.toolchains/libgccjit"
` |
| `setting` | `component[8].requires` | `manifest/gpu.toml` | `scope=component source_kind=manifest` | `["nvidia-cuda-repo","cuda-toolkit","nvidia-open","llvm-clang","rust-nightly-cuda-oxide","cuda-oxide","nvidia-container-toolkit","pytorch-venv","gpu-verify-scripts"]` |
| `setting` | `component[8].requires` | `manifest/nix-yazelix.toml` | `scope=component source_kind=manifest` | `["nix","nix-yazelix-cache","home-manager","yazelix","yazelix-desktop","yazelix-shell","ghostty-default-terminal","yazelix-config"]` |
| `setting` | `component[8].verify.args` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -f \"$M/.toolchains/libgccjit/lib/libgccjit.so\" ] && file \"$M/.toolchains/libgccjit/lib/libgccjit.so\" | grep -q 'shared object'"]` |
| `setting` | `component[9].description` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `Bwrap-backed isolated nix: the DavHau/nix-portable static binary into .toolchains/nix-portable/bin + $META_ROOT/usr/bin/nix-portable. Provides nix without a host root /nix store (home-dir store, logical /nix/store preserved so the binary cache works). Needs bwrap at runtime (NP_RUNTIME=bwrap). Additive — never touches host /nix; destructive /nix migration deferred to supervised TASK-0067. Repo: DavHau/nix-portable.` |
| `setting` | `component[9].detect.args` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/nix-portable/bin/nix-portable\" ] && [ -x \"$META_ROOT/usr/bin/nix-portable\" ] && [ ! -L \"$META_ROOT/usr/bin/nix-portable\" ] && grep -Fqx \"exec \\\"$M/.toolchains/nix-portable/bin/nix-portable\\\" \\\"\\$@\\\"\" \"$META_ROOT/usr/bin/nix-portable\""]` |
| `setting` | `component[9].id` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `nix-portable` |
| `setting` | `component[9].install.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `envctl_frontdoor() { src="$1"; dst="$2"; printf '%s\n' '#!/bin/sh' "exec \"$src\" \"\$@\"" > "$dst"; chmod 755 "$dst"; }
set -euo pipefail
M="${META_ROOT:?META_ROOT required}"
ROOT="$M/envctl"; source "$ROOT/assets/scripts/envctl-gh-fetch.sh"
DEST="$M/.toolchains/nix-portable"
rm -rf "$DEST"; install -d -m 755 "$DEST/bin"
# Authenticated fetch via the shared resolver (gh tier = 5000/hr or the vault-sealed mint token;
# gh is a meta-owned Epic-H component TASK-0057) — avoids the 60/hr unauth GitHub API 403 (the
# box's unauth quota is exhausted). When gh is authed the resolver uses \`gh release download\`;
# otherwise we keep this component's OWN /releases/latest redirect + curl fallback (with a
# last-known-good tag), now bearer-tokened via envctl_gh_curl when a token is available.
if _envctl_gh_authed; then
  envctl_gh_release_download --repo DavHau/nix-portable --pattern 'nix-portable-x86_64' --output "$DEST/bin/nix-portable"
else
  TAG="$(curl -fsSLI -o /dev/null -w '%{url_effective}' 'https://github.com/DavHau/nix-portable/releases/latest' 2>/dev/null | grep -oE 'v[0-9]+$')"; [ -n "$TAG" ] || TAG=v012
  envctl_gh_curl "https://github.com/DavHau/nix-portable/releases/download/${TAG}/nix-portable-x86_64" -o "$DEST/bin/nix-portable"
fi
chmod +x "$DEST/bin/nix-portable"
install -d -m 755 "$META_ROOT/usr/bin"
envctl_frontdoor "$DEST/bin/nix-portable" "$META_ROOT/usr/bin/nix-portable"


# NO mutation/network: the binary forwards all args to nix and has no native --version
# (first real run bwrap-bootstraps a store). nix-portable is a self-extracting POLYGLOT —
# a \`#!/usr/bin/env bash\` wrapper around an embedded ELF — so \`file\` reports "Bourne-Again
# shell script, ASCII text executable", NOT "ELF". Verify therefore checks file-exists +
# executable + \`file | grep -qi 'executable'\` (matches both the "ASCII text executable"
# polyglot today and a plain "ELF ... executable" if upstream ever repackages — future-proof).
` |
| `setting` | `component[9].name` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `nix-portable (meta-owned)` |
| `setting` | `component[9].remove.script` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `set -u
M="${META_ROOT:?META_ROOT required}"
t="$META_ROOT/usr/bin/nix-portable"; [ -L "$t" ] && readlink "$t" | grep -q "$M/.toolchains/nix-portable" && rm -f "$t"
rm -rf "$M/.toolchains/nix-portable"
` |
| `setting` | `component[9].verify.args` | `manifest/components.d/epic-h-toolchains.toml` | `scope=component source_kind=manifest` | `["-lc","M=\"${META_ROOT:?META_ROOT required}\"; [ -x \"$M/.toolchains/nix-portable/bin/nix-portable\" ] && file \"$M/.toolchains/nix-portable/bin/nix-portable\" | grep -qi 'executable'"]` |
| `setting` | `components.cargo-nextest.content_hash` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `9932be9784fb68b2` |
| `setting` | `components.cargo-nextest.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `["rustup"]` |
| `setting` | `components.cargo-nextest.resolved` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `` |
| `setting` | `components.codex-cli.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `["bun","rustup","wild-linker"]` |
| `setting` | `components.cognitum-seed-trust.content_hash` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `eee1272e0344aa41` |
| `setting` | `components.cognitum-seed-trust.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `[]` |
| `setting` | `components.cognitum-seed-trust.resolved` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `` |
| `setting` | `components.cuda-oxide.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `["cuda-toolkit","llvm-clang","nvidia-open","rust-nightly-cuda-oxide"]` |
| `setting` | `components.cuda-toolkit.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `["rustup"]` |
| `setting` | `components.desktop-app.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `["rustup"]` |
| `setting` | `components.env-ctl.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `["rustup","sqld"]` |
| `setting` | `components.grit.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `["libssl-dev","rustup"]` |
| `setting` | `components.group-gpu-stack.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `["cuda-oxide","cuda-toolkit","gpu-verify-scripts","llvm-clang","nvidia-container-toolkit","nvidia-cuda-repo","nvidia-open","pytorch-venv","rust-nightly-cuda-oxide"]` |
| `setting` | `components.group-nix-yazelix.content_hash` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `c50c7448927d3eae` |
| `setting` | `components.group-nix-yazelix.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `["ghostty-default-terminal","home-manager","nix","nix-yazelix-cache","yazelix","yazelix-config","yazelix-desktop","yazelix-shell"]` |
| `setting` | `components.group-nix-yazelix.resolved` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `` |
| `setting` | `components.handoff-hf.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `["rustup"]` |
| `setting` | `components.home-manager.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `["nix"]` |
| `setting` | `components.kache.content_hash` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `42b0fa78f8673a95` |
| `setting` | `components.kache.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `[]` |
| `setting` | `components.kache.resolved` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `` |
| `setting` | `components.meta-env-plugin.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `["rustup"]` |
| `setting` | `components.mlua.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `["llvm-clang","rustup"]` |
| `setting` | `components.nix-portable.content_hash` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `f10c1333504cb8e2` |
| `setting` | `components.nix-portable.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `[]` |
| `setting` | `components.nix-portable.resolved` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `` |
| `setting` | `components.nix-yazelix-cache.content_hash` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `f986c70d2091b3d0` |
| `setting` | `components.nix-yazelix-cache.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `["nix"]` |
| `setting` | `components.nix-yazelix-cache.resolved` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `` |
| `setting` | `components.nix.content_hash` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `0c05525ce0453c9c` |
| `setting` | `components.nix.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `[]` |
| `setting` | `components.nix.resolved` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `` |
| `setting` | `components.prompt_hub.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `["libssl-dev","llvm-clang","rustup"]` |
| `setting` | `components.rtk.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `["rustup"]` |
| `setting` | `components.rust-nightly-cuda-oxide.content_hash` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `0c3d325cb9ba7223` |
| `setting` | `components.rust-nightly-cuda-oxide.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `["rustup"]` |
| `setting` | `components.rust-nightly-cuda-oxide.resolved` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `` |
| `setting` | `components.rustup.content_hash` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `afcd7e2271c29994` |
| `setting` | `components.rustup.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `[]` |
| `setting` | `components.rustup.resolved` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `` |
| `setting` | `components.rusty-idd.content_hash` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `23066f0aed770628` |
| `setting` | `components.rusty-idd.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `["rustup"]` |
| `setting` | `components.rusty-idd.resolved` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `` |
| `setting` | `components.wild-linker.content_hash` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `829ca753878d9ff8` |
| `setting` | `components.wild-linker.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `[]` |
| `setting` | `components.wild-linker.resolved` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `` |
| `setting` | `components.yazelix.requires` | `manifest/envctl.lock` | `scope=lock source_kind=envctl_lock` | `["ghostty","home-manager","nix","nix-yazelix-cache"]` |
| `setting` | `mcpServers.context7.args` | `.mcp.json` | `scope=agent_runtime source_kind=mcp_config` | `["-lc","ROOT=\"${META_ROOT:-/home/drdave/Desktop/meta}\"; export META_ROOT=\"$ROOT\"; export PATH=\"$ROOT/usr/bin:$ROOT/.toolchains/.bun/bin:$ROOT/.toolchains/node/bin:$ROOT/.local/bin:$PATH\"; exec \"$ROOT/usr/bin/bunx\" @upstash/context7-mcp@latest"]` |
| `setting` | `mcpServers.github.args` | `.mcp.json` | `scope=agent_runtime source_kind=mcp_config` | `["-lc","ROOT=\"${META_ROOT:-/home/drdave/Desktop/meta}\"; export META_ROOT=\"$ROOT\"; export PATH=\"$ROOT/usr/bin:$ROOT/.toolchains/.bun/bin:$ROOT/.toolchains/node/bin:$ROOT/.local/bin:$PATH\"; exec \"$ROOT/usr/bin/bunx\" @modelcontextprotocol/server-github"]` |
| `setting` | `mcpServers.n8n-mcp.args` | `.mcp.json` | `scope=agent_runtime source_kind=mcp_config` | `["-lc","ROOT=\"${META_ROOT:-/home/drdave/Desktop/meta}\"; export META_ROOT=\"$ROOT\"; export PATH=\"$ROOT/usr/bin:$ROOT/.toolchains/.bun/bin:$ROOT/.toolchains/node/bin:$ROOT/.local/bin:$PATH\"; export N8N_API_URL=\"${N8N_API_URL:-http://localhost:5678}\"; if [ -z \"${N8N_API_KEY:-}\" ] && [ -x \"$ROOT/usr/bin/secretctl\" ]; then N8N_API_KEY=\"$(\"$ROOT/usr/bin/secretctl\" secret get n8n-api-key --reveal --apply 2>/dev/null || true)\"; export N8N_API_KEY; fi; exec \"$ROOT/usr/bin/bunx\" n8n-mcp"]` |
| `setting` | `mcpServers.playwright.args` | `.mcp.json` | `scope=agent_runtime source_kind=mcp_config` | `["-lc","ROOT=\"${META_ROOT:-/home/drdave/Desktop/meta}\"; export META_ROOT=\"$ROOT\"; export PATH=\"$ROOT/usr/bin:$ROOT/.toolchains/.bun/bin:$ROOT/.toolchains/node/bin:$ROOT/.local/bin:$PATH\"; exec \"$ROOT/usr/bin/bunx\" @playwright/mcp@latest --extension"]` |
| `setting` | `mcpServers.sequential-thinking.args` | `.mcp.json` | `scope=agent_runtime source_kind=mcp_config` | `["-lc","ROOT=\"${META_ROOT:-/home/drdave/Desktop/meta}\"; export META_ROOT=\"$ROOT\"; export PATH=\"$ROOT/usr/bin:$ROOT/.toolchains/.bun/bin:$ROOT/.toolchains/node/bin:$ROOT/.local/bin:$PATH\"; exec \"$ROOT/usr/bin/bunx\" @modelcontextprotocol/server-sequential-thinking"]` |
| `setting` | `objective` | `.handoff/tasks/TASK-0012.task.json` | `scope=handoff source_kind=handoff_task` | `New pure-Rust crate crates/agent-env (6-key+extends model, multi-host resolver, SHA-256, lock)` |
| `setting` | `objective` | `.handoff/tasks/TASK-0025.task.json` | `scope=handoff source_kind=handoff_task` | `CI required checks on develop (rustfmt/clippy/test/gates) so auto-merge fails closed` |
| `setting` | `objective` | `.handoff/tasks/TASK-0033.task.json` | `scope=handoff source_kind=handoff_task` | `VPS Profile B (BLOCKED owner-gated): F7 install gate + F8/OI-SM-2 authorizer + OI-SM-3 trusted-time` |
| `setting` | `objective` | `.handoff/tasks/TASK-0034.task.json` | `scope=handoff source_kind=handoff_task` | `Hardening tail: F10 tonic pin + cargo-audit CI, F11 MSRV check, F18 audit-fsync` |
| `setting` | `objective` | `.handoff/tasks/TASK-0053.task.json` | `scope=handoff source_kind=handoff_task` | `# Overview

Capture and route the verified meta GitHub transport and automation doctrine into envctl so envctl can implement the missing credential/merge-gate pieces without relying on stale assumptions or raw GitHub API output.

Deep research started from \`meta/.kb/AGENTS.md\` and loaded the meta KB/context. The proof is source-grounded: \`.meta.yaml\`, live git remotes, \`.github_org\` architecture docs/TODO, \`handoff\` ADR/source, \`flexnetos_github_app\` source, and live \`gh\`/SSH checks.

## Goals

- Make envctl aware that local \`git\` over SSH is the repository source of truth for FlexNetOS repos.
- Keep \`gh\` CLI/API as GitHub workflow orchestration, but require re-query/cross-checks against git refs, PR state, and required checks before trusting mutations.
- Route the missing envctl-owned GitHub credential work into the existing envctl handoff loop, especially scoped GitHub App token mint/injection and policy-drift token provisioning.
- Preserve the fail-closed model: agents do not hold broad merge tokens, do not native-APPROVE their own PRs, and do not force-merge red checks.

## Acceptance Criteria

- [ ] envctl backlog/task docs include the GitHub transport doctrine: SSH git is repo truth; \`gh\` is orchestration; raw API/connector output is advisory until re-queried.
- [ ] envctl exposes/validates the scoped GitHub App token path needed by downstream consumers: \`secretctl mint-github --installation-id 140063898 --output json\` and related enroll/revoke flows remain byte-stable.
- [ ] envctl has a concrete owner path for \`POLICY_DRIFT_TOKEN\` / app-minted equivalent so \`.github\` policy drift can read branch protection, rulesets, environments, and repo settings in strict mode.
- [ ] Any envctl implementation keeps tokens broker-only/scoped/short-lived and never logs secrets.
- [ ] Integration proof cross-checks \`flexnetos_github_app\` consumer expectations, especially merge-gate check-run writer expectations.
- [ ] Verification uses SSH-backed git refs plus \`gh\` re-query; no raw API mutation is treated as success without read-back.
- [ ] Handoff continuity is exported/committed using the current redb-backed ledger plus deterministic JSONL export; do not describe this as SQLite.

## Context / Proof

- \`meta/.kb/AGENTS.md\` requires KB/context-first operation and says the document is the plan.
- \`.meta.yaml\` currently configures 66/66 project repos as \`git@github.com:FlexNetOS/...\` SSH URLs.
- Live sample origins for \`meta\`, \`.github_org\`, \`envctl\`, \`meta-ruvector\`, \`rusty-idd\`, \`weave\`, \`handoff\`, and \`flexnetos_github_app\` are SSH.
- \`git ls-remote --symref origin HEAD\` from meta succeeds over SSH.
- \`gh auth status\` is logged in, but \`gh config get git_protocol\` reports \`https\`, so \`gh\` must not be treated as the git transport source of truth.
- \`.github_org/TODO.md\` records that default \`GITHUB_TOKEN\` cannot read branch protection, rulesets, or repo settings; strict policy drift needs a provisioned token from envctl.
- \`.kb/store/documents/tasks/github-local-model-pivot.md\` records cloud-token burn from automatic Claude review flows and the requirement to move GitHub automation to local model / opt-in review.
- \`.kb/store/documents/incidents/release-please-token-unavailable.md\` records that \`GITHUB_TOKEN\`-created PRs do not trigger CI, so release PRs cannot pass required checks/auto-merge until the proper org secret/token path is granted.
- \`.github_org/architecture/map/01-meta-control-plane.md\` records that \`gh\` mutations can silently succeed and must be re-queried; it also records GitHub auto-merge/API edge cases.
- \`.github_org/architecture/plan/2026-06-17-deep-review-upgrade-plan.md\` records a concrete policy-applier hazard: \`gh repo view\` resolving from the wrong CWD can mutate the wrong repo unless owner/repo is asserted.
- \`flexnetos_github_app/crates/app-core/src/merge_gate.rs\` says the App should post a verdict as a required GitHub check-run and arm native auto-merge only after green; it must never be a native bot APPROVE, and the current \`UnwiredMergeGate\` fails closed.
- \`handoff\` source/ADRs record the out-of-band review verdict model: judgment is recorded in handoff/weave state and enforced via required check/merge gate, not by bot approving the PR.

## Envctl Scope

Primary envctl areas:

- \`.handoff/loop/backlog.md\` and relevant task cards for GitHub App mint/enroll/revoke/token provisioning.
- \`crates/secretd\`, \`crates/secretctl\`, \`crates/secrets-engine\` GitHub App provider mint path.
- Any envctl agent/environment injection surfaces that provide short-lived GitHub tokens to \`gh\`/workflow automation.

Consumer cross-checks:

- \`../flexnetos_github_app/crates/app-core/src/mint.rs\`
- \`../flexnetos_github_app/crates/app-core/src/merge_gate.rs\`
- \`.github_org\` policy drift scripts and workflows.

## Notes

This is not a request to avoid the GitHub API entirely. It is a requirement to use it through controlled \`gh\`/App paths with explicit owner/repo selection, least privilege, read-back verification, and SSH git as the repository truth.
` |
| `setting` | `objective` | `.handoff/tasks/TASK-0078.task.json` | `scope=handoff source_kind=handoff_task` | `Implement ADR-0002: upgrade envctl migration/adoption from audit/bootstrap into a typed, parity-proven meta layout engine. Envctl must scan/classify/plan/adopt/verify/activate/quarantine/purge component-scoped migrations into $META_ROOT system-shaped paths (.local/bin, .local/lib, .local/share, .local/state, .local/cache, .local/tmp, component toolchain roots) without manual path surgery, downgrades, or blind deletion. Broad migration/purge remains blocked until this task's v2 planner, evidence ledger, verification gates, activation proof, protected-path rules, and CI no-new-debt ratchet are implemented.` |
| `setting` | `skills../agent-skills::agent-env-config.description` | `agent-env.lock` | `scope=lock source_kind=agent_env_lock` | `"The CORRECT conventions and agent-environment configuration for the envctl Rust workspace — supersedes the broken ECC-auto-generated skill/instincts that assert JavaScript conventions. Use whenever writing or reviewing envctl code, naming files/types, writing tests, composing commits, or configuring the .claude/.codex agent setup (skills, MCP servers, multi-agent roles). Triggers: 'what conventions', 'how do I name this', 'write a test', 'commit message', 'configure the agents', 'MCP setup', 'is camelCase right'."` |
| `setting` | `skills../agent-skills::env-toolchain-install.description` | `agent-env.lock` | `scope=lock source_kind=agent_env_lock` | `"How to install and configure the developer toolchain the way envctl does it — declaratively, idempotently, with detect→install→verify→fix→remove lifecycle hooks per component. Use whenever installing, repairing, or configuring environment tooling (Rust, bun/node, CUDA/GPU stack, ai-clis, nix-yazelix, boot-repair, the secretd daemon) or authoring a new component. Triggers: 'install the toolchain', 'set up the environment', 'add a component', 'why is X not on PATH', 'repair the environment', 'the install isn't idempotent'."` |
| `setting` | `skills../agent-skills::env-toolchain-install.destination` | `agent-env.lock` | `scope=lock source_kind=agent_env_lock` | `.claude/skills/env-toolchain-install` |
| `setting` | `skills../agent-skills::env-toolchain-install.hash` | `agent-env.lock` | `scope=lock source_kind=agent_env_lock` | `6fb44c38b51c3f0f4f4c134ac4f33470e69f3efb8be5ee20a59859c42f081a97` |
| `setting` | `skills../agent-skills::env-toolchain-install.scope` | `agent-env.lock` | `scope=lock source_kind=agent_env_lock` | `project` |
| `setting` | `skills../agent-skills::env-toolchain-install.skill` | `agent-env.lock` | `scope=lock source_kind=agent_env_lock` | `env-toolchain-install` |
| `setting` | `skills../agent-skills::env-toolchain-install.source` | `agent-env.lock` | `scope=lock source_kind=agent_env_lock` | `./agent-skills` |
| `setting` | `skills../agent-skills::env-toolchain-install.source_revision` | `agent-env.lock` | `scope=lock source_kind=agent_env_lock` | `local` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0001.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0002.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0003.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0004.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0005.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0006.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0007.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0008.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0009.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0010.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0011.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0012.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0013.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0014.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0015.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0016.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0017.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0018.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0019.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0020.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0021.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0022.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0023.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0024.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0025.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0026.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0027.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0028.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0029.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0030.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0031-PR2.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0031-PR2C.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0031.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0032.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0033.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0034.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0035.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0036.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0037.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0038.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0039.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0041.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0042.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0043.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0044.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0045.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0046.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0047.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0048.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0049.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0050.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0051.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0052.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo build -p envctl-engine -p envctl","bash ci/gates/p7.sh"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0053.task.json` | `scope=handoff source_kind=handoff_task` | `["cargo test -p envctl-secretctl","cargo test -p envctl-secretd --features provider-github","cargo test -p envctl-secrets-engine --features provider-github","bash ci/gates/no-c.sh","cargo clippy --workspace -- -D warnings"]` |
| `setting` | `test_commands` | `.handoff/tasks/TASK-0078.task.json` | `scope=handoff source_kind=handoff_task` | `["hf doctor --json","hf gitignore --check","bash ci/gates/p7.sh","cargo test -p envctl-engine migration","cargo test -p envctl --test cli_contract migration","bash ci/gates/migration-debt.sh","cargo fmt --all --check","cargo clippy --workspace -- -D warnings"]` |
| `setting` | `title` | `.handoff/tasks/TASK-0012.task.json` | `scope=handoff source_kind=handoff_task` | `New pure-Rust crate crates/agent-env (6-key+extends model, multi-host resolver, SHA-256, lock)` |
| `setting` | `title` | `.handoff/tasks/TASK-0025.task.json` | `scope=handoff source_kind=handoff_task` | `CI required checks on develop (rustfmt/clippy/test/gates) so auto-merge fails closed` |
| `setting` | `title` | `.handoff/tasks/TASK-0033.task.json` | `scope=handoff source_kind=handoff_task` | `VPS Profile B (BLOCKED owner-gated): F7 install gate + F8/OI-SM-2 authorizer + OI-SM-3 trusted-time` |
| `setting` | `title` | `.handoff/tasks/TASK-0034.task.json` | `scope=handoff source_kind=handoff_task` | `Hardening tail: F10 tonic pin + cargo-audit CI, F11 MSRV check, F18 audit-fsync` |
