# Code Map For Debugging

Task: `ART-113_DEBUG_CODE_MAP`
Generated at: `2026-07-28T11:30:41+00:00`
Target root: `/home/flexnetos/FlexNetOS`

## Scope

This is a static debugging map for the target filesystem. It records entry points, control-flow candidates, external calls, error and retry paths, logging, metrics or alert signals, and runbook references. Dynamic dispatch and generated runtime behavior are marked as static candidates by design.

## Scan Summary

| signal | count |
|---|---:|
| entry point count | 160 |
| control flow count | 160 |
| external call count | 160 |
| error path count | 160 |
| log signal count | 160 |
| metrics alert count | 160 |
| runbook signal count | 160 |
| hotspot count | 80 |

## Hotspots

| file | score | categories |
|---|---:|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.codex/skills/agent-env-codex/scripts/check-yazelix-contract.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.handoff/loop/backlog.md` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-skills/agent-env-codex/scripts/check-yazelix-contract.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/ci/gates/enable.sh` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/ci/gates/kdf-feature-off.sh` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/ci/gates/meta-local-policy.sh` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/cli/src/main.rs` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/cli/src/migration_cmd.rs` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/gui/src/main.rs` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secretctl/src/main.rs` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secretctl/src/sqld_auth.rs` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secretd/src/main.rs` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/src/lib.rs` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/docs/generated/toolchain-signal-inventory.md` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/migration-artifacts/03-code-analysis/code-map-for-debugging.md` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/migration-artifacts/art-104_toolchain_tree/toolchain-dependency-tree.json` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/migration-artifacts/art-109_data_lineage/data-lineage-map.json` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/migration-artifacts/art-111_event_map/event-message-contract-map.json` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/migration-artifacts/art-113_debug_code_map/debug-code-map.json` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/migration-artifacts/art-113_debug_code_map/debug-code-map.md` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/migration-artifacts/art-115_config_inventory/config_inventory.json` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/generate_art102_repository_map.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/generate_art104_toolchain_tree.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/generate_art106_runtime_dep_map.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/generate_art109_data_lineage.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/generate_art110_api_catalog.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/generate_art111_event_map.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/generate_art113_debug_code_map.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/generate_art118_observability.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/envctl-db-nu-plugin-migration-automation-package/execution-framework/scripts/generate_art120_wave_plan.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |

## Entry Points

| file | line | signal | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/commits/019f2a56-e524-74a3-a034-3a3cc2473e3f.json` | 2 | `package-script` | "commit": {"author":"drdave <flexnetos@de-flex.net>","changes":[{"change_type":"modified","content_hash":"bdb9105f2aeea09f400ad2bc...","doc_type":"incident","document_id":"019f2a52-3abb-79a3-b941-...","hunks":[{"lines":[ |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/commits/019f2d99-0ab9-7940-9e86-7aa369ce278b.json` | 2 | `package-script` | "commit": {"author":"drdave <flexnetos@de-flex.net>","changes":[{"change_type":"modified","content_hash":"5f7c405cf62c29d8e6a6b0f0...","doc_type":"task","document_id":"019f2adb-59da-76c2-9164-...","hunks":[{"context":"## |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/packaging/envctl-gui.desktop` | 6 | `desktop-exec` | Exec=envctl-gui |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/packaging/install-desktop.sh` | 1 | `shell-entry` | #!/usr/bin/env bash |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/examples/seed_factor_probe.rs` | 17 | `rust-main` | fn main() { |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/examples/seed_factor_probe.rs` | 48 | `rust-main` | fn main() { |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secrets-engine/src/lib.rs` | 16 | `shell-entry` | #![allow(dead_code)] // Some scaffold fields/bodies are placeholders until later phases. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/tests/ruvector_codec_compat.rs` | 1 | `shell-entry` | #![cfg(feature = "ruvector-pg")] |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/install.rs` | 13 | `shell-entry` | #![cfg(unix)] |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/db_index.rs` | 383 | `rust-main` | fs::write(root.join("main.rs"), b"fn main() {}\n").unwrap(); |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/engine/src/db_index.rs` | 411 | `rust-main` | assert_eq!(main.byte_len, 13); // "fn main() {}\n" |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secretd/tests/edge_hardening_e2e.rs` | 18 | `shell-entry` | #![cfg(feature = "relay-edge")] |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secretd/tests/edge_e2e.rs` | 14 | `shell-entry` | #![cfg(feature = "relay-edge")] |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secretd/tests/edge_stream_e2e.rs` | 21 | `shell-entry` | #![cfg(feature = "relay-edge")] |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secretd/tests/profile_b_e2e.rs` | 15 | `shell-entry` | #![cfg(feature = "relay-edge")] |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secretd/tests/mitm_e2e.rs` | 16 | `shell-entry` | #![cfg(feature = "mitm-ca")] |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secretd/tests/native_mint_e2e.rs` | 15 | `shell-entry` | #![cfg(feature = "provider-github")] |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secretd/src/main.rs` | 35 | `cli-parser` | use clap::Parser; |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secretd/src/main.rs` | 66 | `rust-main` | fn main() -> anyhow::Result<()> { |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secretd/src/conv.rs` | 12 | `shell-entry` | #![allow(dead_code)] |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/secretd/src/conv.rs` | 16 | `shell-entry` | #![allow(clippy::result_large_err)] |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/cli/tests/cli_contract.rs` | 727 | `rust-main` | std::fs::write(local_repo.join("src/main.rs"), "fn main() {}\n").unwrap(); |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/cli/src/main.rs` | 17 | `cli-parser` | /// subcommand. Uses `clap::builder::styling` (built into clap 4.x — no new dep). |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/crates/cli/src/main.rs` | 18 | `cli-parser` | fn clap_styles() -> clap::builder::Styles { |
| ... |  |  | 136 more entries in JSON artifact |

## Control Flow Candidates

| file | line | signal | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 20 | `workflow` | \| 2026-06-18 \| feature-forge \| A retirement/absorption is not DONE until the on-disk artifacts the new code RESOLVES are migrated AND any claimed enforcement actually runs — a documented-but-unwired gate is worse than no |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 23 | `workflow` | \| 2026-06-18 \| forge-loop \| When a self-cleaning mechanism exists for ONE consistency surface, check every PARALLEL surface — a fix on the remote leaves the local mirror stale. Sync all of them, FF-only + clean-only (nev |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 29 | `workflow` | \| 2026-06-23 \| forge-loop \| Don't dodge an infra limit — AUTHENTICATE with the infra the owner already built. When automation hits a quota/permission wall (GitHub API 403), the fix is to use the available credential, not |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 146 | `workflow` | - `reset` excises **only envctl-owned** wiring blocks; foreign edits are reported. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/README.md` | 68 | `workflow` | complete command contract, all query presets, symbol-mapping workflow, and current CLI examples |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/README.md` | 110 | `workflow` | Add-Repo form · Live Logs · Settings. The engine runs on a worker thread; the UI never blocks. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 277 | `workflow` | \| 2026-06-18 \| U2/U5 — complete the kasetto integration: migrate config filenames + wire the (claimed-but-absent) drift gate (TASK-0040) \| agent-env.yaml + agent-env.lock (git mv from kasetto.yaml/kasetto.lock; header re |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 279 | `workflow` | \| 2026-06-18 \| Reaper polish: FF-sync the protected trunk branches (close the /verify finding) \| scripts/reap-worktrees.sh (new step 1b); skills/forge-loop ("Worktree hygiene" note) \| `/verify` found the local main check |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 355 | `workflow` | rtk gh run list # Compact workflow runs (82%) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/tests/fixtures/nushell/env-config.nu` | 7 | `nu-def` | def --wrapped git [...rest] { ^rtk git ...$rest } |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/AGENTS.md` | 813 | `js-function` | - **Before changing a function signature**: Run `kb_callers <symbol>` to find all call sites that will need updates |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/AGENTS.md` | 840 | `js-function` | \| `kb_callees` \| Find what a function calls \| `kb_callees --symbol login` \| |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/tasks/envctl-codex-mcp-runtime-import.md` | 14 | `workflow` | This task captures the envctl-side upgrade path: use the `nu_plugin` `/nu_plugin:import` workflow to import the relevant Codex/MCP/config files into the envctl catalog tables, so envctl can own the declarative rows and l |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/tasks/envctl-codex-mcp-runtime-import.md` | 19 | `workflow` | - Use the `nu_plugin` import workflow as the frontdoor for bringing those files into CodeDB/envctl tables. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/tasks/envctl-codex-mcp-runtime-import.md` | 40 | `workflow` | - [ ] A documented envctl task exists for importing Codex/MCP/runtime-config files through the `nu_plugin` import workflow. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/tasks/envctl-codex-mcp-runtime-import.md` | 43 | `workflow` | - [ ] The task references the existing CodeDB/Nu plugin inventory work so this import slice lands in the established workflow instead of becoming a parallel one-off. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/tasks/import-host-artifacts-yazelix-mission-control-20260703.md` | 13 | `workflow` | On 2026-07-03 an interactive session with Claude Code set up ccboard (FlorianBruniaux/ccboard) as a Mission Control tab and popup inside the yazelix flexnetos foundation runtime. The changes on the yazelix source side la |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/tasks/import-host-artifacts-yazelix-mission-control-20260703.md` | 56 | `workflow` | - Bring all host-side artifacts above into envctl catalog rows via the nu_plugin import workflow, so the same files can be reproduced from envctl declarative sources on a fresh install. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/tasks/import-host-artifacts-yazelix-mission-control-20260703.md` | 83 | `workflow` | - Files enumerated but not yet imported. Next envctl session should run the nu_plugin import workflow against the eight paths listed above, starting with the two brand-new files (~/.local/bin/flexnetos-rebu... and ~/.loc |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/documents/context/immutable/project-brief.md` | 14 | `workflow` | envctl is secondary — a subordinate agent whose single job is to **own and converge the meta |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/store/commits/019f2d99-0ab9-7940-9e86-7aa369ce278b.json` | 2 | `workflow` | "commit": {"author":"drdave <flexnetos@de-flex.net>","changes":[{"change_type":"modified","content_hash":"5f7c405cf62c29d8e6a6b0f0...","doc_type":"task","document_id":"019f2adb-59da-76c2-9164-...","hunks":[{"context":"## |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/kb-start/SKILL.md` | 57 | `workflow` | - **Dependencies**: What this blocks, what blocks this (`kb_graph`) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/code-intelligence/SKILL.md` | 12 | `js-function` | - Finding callers of a function or method |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/.kb/skills/code-intelligence/SKILL.md` | 13 | `js-function` | - Finding what a function calls (callees) |
| ... |  |  | 136 more entries in JSON artifact |

## External Calls

| file | line | signal | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-env.yaml` | 1 | `tool-call` | # agent-env.yaml — agent environment for the envctl workspace. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-env.yaml` | 3 | `tool-call` | # absorbed engine is now `envctl agent`. The absorbed CLI resolves this filename by default.) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-env.yaml` | 4 | `mcp` | # Provisions a curated, locked agent toolkit (skills + MCP baseline) into the Claude Code and |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-env.yaml` | 6 | `tool-call` | # ECC-auto-generated .claude/.codex files. Lock-of-record: agent-env.lock (was kasetto.lock). |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-env.yaml` | 9 | `tool-call` | # envctl agent lock --check --locked --color never # read-only, zero-network |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-env.yaml` | 10 | `tool-call` | # envctl agent sync --json --color never # preview; writes nothing |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-env.yaml` | 11 | `tool-call` | # envctl agent sync --apply --color never # only after review/approval |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-env.yaml` | 13 | `tool-call` | # envctl agent lock --check --locked # zero-network; exits 1 on drift |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-env.yaml` | 16 | `tool-call` | # /home/flexnetos/.codex/config.toml is the active runtime config. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-env.yaml` | 17 | `tool-call` | # /home/flexnetos/.codex/RULES.md, /home/flexnetos/.codex/RTK.md, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-env.yaml` | 18 | `tool-call` | # /home/flexnetos/.codex/AGENTS.rtk.md, /home/flexnetos/AGENTS.md, and |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-env.yaml` | 20 | `tool-call` | # their envctl-owned source copies live under home/.codex/ and home/. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-env.yaml` | 21 | `tool-call` | # /home/flexnetos/lifeos/.codex and /home/flexnetos/FlexNetOS/.codex are |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-env.yaml` | 33 | `tool-call` | - codex |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-env.yaml` | 36 | `tool-call` | # Curated, envctl-grounded dev skills (correct Rust conventions). |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-env.yaml` | 41 | `mcp` | # Yazelix is the normative runtime model. Only MCP entries that do not launch |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 15 | `tool-call` | \| 2026-06-17 \| feature-forge \| Sync-engine→async-daemon I/O has a fixed envctl idiom (off-reactor `block_on` via captured `Handle` inside `spawn_blocking`; reuse the frozen `build_upstream_client`, add no dep; key-free e |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 16 | `tool-call` | \| 2026-06-17 \| feature-forge \| The guardian must classify clippy findings by BOTH axis (gate form vs `--all-targets`) and origin (touched vs untouched file): `--all-targets`-only in an untouched file = inherited red (NOT |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 19 | `tool-call` | \| 2026-06-18 \| forge-loop \| TICK-ON-MERGED, not tick-on-armed: "guardian PASS + `gh pr merge --auto`" only ARMS a merge (a required check can still block it). Terminal Done must require `gh pr view <N>` == MERGED; armed- |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 25 | `tool-call` | \| 2026-06-18 \| forge-loop \| The squash-robust merge ORACLE is the GitHub merged-PR head-ref, not the local `[gone]`/ancestor heuristics. After a squash merge the tip is never an ancestor of master, and a deleted upstream |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 26 | `tool-call` | \| 2026-06-18 \| forge-loop \| Irreversible EXTERNAL mutations (remote-branch deletes, security-advisory dismissals, anything that leaves the repo and can't be `git reset`) are a correct human wall — gate them on explicit o |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 27 | `tool-call` | \| 2026-06-23 \| forge-loop \| Write-side complement of TICK-ON-MERGED: once `gh pr merge --auto` is armed, treat the PR branch as potentially-already-merged. A fast PR can merge (and origin can auto-delete the head, `delet |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 29 | `tool-call` | \| 2026-06-23 \| forge-loop \| Don't dodge an infra limit — AUTHENTICATE with the infra the owner already built. When automation hits a quota/permission wall (GitHub API 403), the fix is to use the available credential, not |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 31 | `process-exec` | \| 2026-06-23 \| feature-forge \| Vendor self-extracting installers (makeself: CUDA `.run`, many `.sh` bundles) try to spawn a terminal/xterm when stdout is NOT a TTY — under automation they die `exec: -title` (exit 127) wi |
| ... |  |  | 136 more entries in JSON artifact |

## Errors, Retries, Timeouts

| file | line | signal | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 16 | `validation` | \| 2026-06-17 \| feature-forge \| The guardian must classify clippy findings by BOTH axis (gate form vs `--all-targets`) and origin (touched vs untouched file): `--all-targets`-only in an untouched file = inherited red (NOT |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 19 | `validation` | \| 2026-06-18 \| forge-loop \| TICK-ON-MERGED, not tick-on-armed: "guardian PASS + `gh pr merge --auto`" only ARMS a merge (a required check can still block it). Terminal Done must require `gh pr view <N>` == MERGED; armed- |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 24 | `validation` | \| 2026-06-18 \| forge-loop \| Destructive/irreversible automation must mirror the product's own invariants: dry-run-by-default, never `--force`, protect-and-skip (skip dirty, protect master/develop/current). This is also w |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 25 | `validation` | \| 2026-06-18 \| forge-loop \| The squash-robust merge ORACLE is the GitHub merged-PR head-ref, not the local `[gone]`/ancestor heuristics. After a squash merge the tip is never an ancestor of master, and a deleted upstream |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 29 | `retry-timeout` | \| 2026-06-23 \| forge-loop \| Don't dodge an infra limit — AUTHENTICATE with the infra the owner already built. When automation hits a quota/permission wall (GitHub API 403), the fix is to use the available credential, not |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 31 | `validation` | \| 2026-06-23 \| feature-forge \| Vendor self-extracting installers (makeself: CUDA `.run`, many `.sh` bundles) try to spawn a terminal/xterm when stdout is NOT a TTY — under automation they die `exec: -title` (exit 127) wi |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 34 | `validation` | \| 2026-06-23 \| forge-loop \| When a subagent dies mid-cycle (weekly model limit / terminal API error), the orchestrator must finish the cycle deterministically rather than abandon it — the agent's edits persist in the wor |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 36 | `validation` | \| 2026-06-27 \| plan-loop \| Phantom-workspace / sibling-escape environment hazard: a stray `Cargo.toml` in an ancestor dir makes Cargo walk UP and silently absorb the target into a foreign virtual workspace, breaking stan |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 40 | `exception` | \| 2026-07-09 \| feature-forge \| Re-pin a RED-authored acceptance pattern against the REAL surface at GREEN: a test written before its surface exists asserts a GUESSED oracle that can be both false-RED (never-matchable) AN |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 43 | `validation` | \| 2026-07-09 \| forge-loop \| Hook-enforcement verify-at-pick must account for SESSION-SNAPSHOT semantics: Claude Code binds the hook set at session start, so a hook wired mid-session does NOT fire in the running session a |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 1 | `validation` | # envctl — HANDOFF / verification guide |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 28 | `validation` | guard failed *open* on btrfs; `--rename foo=git` could shadow system `git`; the GUI's |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 124 | `validation` | `kasetto` binary is required. Quick verification: |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 142 | `retry-timeout` | - Guards fail closed (`UuidResolves`/`NotLiveDevice`/`NotMounted` via blkid/findmnt; |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 147 | `validation` | - A half-failed run is `Incomplete` → CLI exits nonzero (never green on failure). |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/README.md` | 33 | `validation` | # Rust is required (latest nightly is the dev default; exact Rust 1.89.0 is the MSRV lane): |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 35 | `retry-timeout` | (meta's shared builder) while keeping its own supervision (setsid reaping, per-phase timeout, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 260 | `validation` | \| 2026-06-04 \| Add rtk-proxy + baseline-stash guidance \| skills/rust-feature-impl...; skills/feature-forge \| Smoke test: rtk summarizes cargo/git output (corrupts fmt/clippy diagnostics); floating `stable`=1.96 causes pr |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 273 | `validation` | \| 2026-06-17 \| G2 retro: encode the instincts that made the run clean (evolution-steward, 5 low-risk APPLY) \| skills/feature-forge (Phase 0 verify-triggering-claim step; Phase 1.5 independent-modules routing); agents/fea |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 275 | `validation` | \| 2026-06-18 \| Worktree/branch reaper — keep worktrees ↔ branches ↔ origin consistent \| scripts/reap-worktrees.sh (NEW); skills/session-relay-wra... (new step 5b); skills/session-relay-resume (new step 4b); skills/forge- |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 277 | `validation` | \| 2026-06-18 \| U2/U5 — complete the kasetto integration: migrate config filenames + wire the (claimed-but-absent) drift gate (TASK-0040) \| agent-env.yaml + agent-env.lock (git mv from kasetto.yaml/kasetto.lock; header re |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 282 | `validation` | \| 2026-06-18 \| Epic G deep audit + Tier-1 hardening (TASK-0041/0042/0043) \| `.handoff/loop/backlog.md` (Epic G plan, TASK-0041..0052, Tier-2/3 decisions LOCKED); ci/gates/loop-state.sh + scripts/tests/test-loop-....sh (N |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 283 | `validation` | \| 2026-06-18 \| Epic G Tier-2/3 complete: TASK-0044 (hf-kernel cards) + TASK-0052 (harness_hub package) \| TASK-0044: `.handoff/tasks/TASK-*.task.json` (53 fleet-scoped cards via handoff-kernel-engineer) + `skills/forge-lo |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/AGENTS.md` | 55 | `validation` | <command> ...` when exact stdout/stderr is required. Direct raw shell execution |
| ... |  |  | 136 more entries in JSON artifact |

## Logs And Audit Signals

| file | line | signal | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 1 | `audit-log` | # Harness lessons ledger (Feature Forge + ejected harnesses) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 8 | `audit-log` | \| date \| harness \| lesson (class, generalized) \| evidence (cycle/finding) \| recurrence \| routed-to \| status \| |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 11 | `audit-log` | \| date \| harness \| lesson (class, generalized) \| evidence \| recurrence \| routed-to \| status \| |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 22 | `audit-log` | \| 2026-06-18 \| feature-forge \| Prevent drift at PICK-time, not just at wrap-up: a frozen-consumer-contract check that runs only as a reactive end-of-session sweep fires a cycle too late. Grep for a frozen CLI/RPC/JSON co |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 20 | `nu-log` | regression-tested (see `git log`). A content-hashed **`envctl.lock`** (CI gate) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 155 | `audit-log` | see `.handoff/loop/rust-port/p....md` for the 102/115 parity-verified |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/README.md` | 6 | `nu-log` | system-shaped prefix (`$META_ROOT/{usr/bin,usr/lib,usr/share,etc,var/lib,var/cache,var/log,var/tmp,opt} plus XDG meta-home roots`), with |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/README.md` | 83 | `nu-log` | \| durable envctl state/logs \| `$META_ROOT/var/lib/envctl` / `$META_ROOT/var/log/envctl` \| |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 97 | `audit-log` | bash ci/gates/p7.sh # .handoff Tier-A p7-conformance: schema tags + ledger residency (ADR-0004 §3) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 250 | `audit-log` | > evidence-backed planning cycle and `/harness:plan-loop` for the continuous Ralph loop). The envctl |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 262 | `audit-log` | \| 2026-06-05 \| Correct relay signal model after full smoke \| skills/session-relay \| Smoke test: `CronCreate{durable}` is session-only here (not persisted), and a self-identity weave message is invisible to the successor' |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 265 | `audit-log` | \| 2026-06-05 \| Add component-research/audit phase (auto-append upgrades to backlog) \| skills/env-install-loop; skills/auto-provision (+scripts/ralph-provision.sh) \| Generalize the manual pytorch deep-dive (shallow gate,  |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 266 | `audit-log` | \| 2026-06-05 \| Add A2 cross-repo parallel build (default-OFF, scale auto-trigger) \| skills/{feature-forge,forge-loop,session-relay}; agents/{rust-implementer,continuity-steward} \| Cross-repo parallelism via the three-own |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 269 | `audit-log` | \| 2026-06-12 \| Migrate harness durable state `_workspace/`→`.handoff/loop/`; add kasetto-absorption capability + handoff-sync skill + hf-aware continuity \| skills/{forge-loop,feature-forge,session-relay,env-install-loop, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 270 | `nu-log` | \| 2026-06-13 \| Add `handoff-kernel-engineer` agent (Epic A) + seed loop_state to schema + reconcile backlog \| agents/handoff-kernel-en....md; skills/feature-forge (crew table + Epic-A Build routing); .handoff/loop/{loop_ |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 271 | `audit-log` | \| 2026-06-13 \| Wire continuity auto-hooks (dormant until hf) + fix broken `.kb` hook \| .claude/settings.json (NEW project layer); .claude/hooks/hf-checkpoint.sh (NEW); .handoff/loop/backlog.md; (meta repo) .claude/settin |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 272 | `audit-log` | \| 2026-06-13 \| Eject `rust-port-merge` harness for the kasetto absorption (Epic C / TASK-0012) \| .claude/skills/{rust-port,rust-port-inventory,rust-port-translate,rust-port-parity,rust-port-merge,cross-repo-reference,icm |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 273 | `audit-log` | \| 2026-06-17 \| G2 retro: encode the instincts that made the run clean (evolution-steward, 5 low-risk APPLY) \| skills/feature-forge (Phase 0 verify-triggering-claim step; Phase 1.5 independent-modules routing); agents/fea |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 275 | `nu-log` | \| 2026-06-18 \| Worktree/branch reaper — keep worktrees ↔ branches ↔ origin consistent \| scripts/reap-worktrees.sh (NEW); skills/session-relay-wra... (new step 5b); skills/session-relay-resume (new step 4b); skills/forge- |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 280 | `audit-log` | \| 2026-06-18 \| Retro audit (evolution-steward): recurrence fix + 2 new lessons + irreversible-remote-delete discipline \| LESSONS.md (frozen-contract recurrence 1→2; +2 rows); scripts/reap-worktrees.sh (`[gone]`≠merged CA |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 282 | `audit-log` | \| 2026-06-18 \| Epic G deep audit + Tier-1 hardening (TASK-0041/0042/0043) \| `.handoff/loop/backlog.md` (Epic G plan, TASK-0041..0052, Tier-2/3 decisions LOCKED); ci/gates/loop-state.sh + scripts/tests/test-loop-....sh (N |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 283 | `audit-log` | \| 2026-06-18 \| Epic G Tier-2/3 complete: TASK-0044 (hf-kernel cards) + TASK-0052 (harness_hub package) \| TASK-0044: `.handoff/tasks/TASK-*.task.json` (53 fleet-scoped cards via handoff-kernel-engineer) + `skills/forge-lo |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 336 | `nu-log` | rtk git log # Compact log (works with all git flags) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 381 | `nu-log` | rtk log <file> # Deduplicated logs with counts |
| ... |  |  | 136 more entries in JSON artifact |

## Metrics And Alerts

| file | line | signal | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-env.yaml` | 26 | `status` | # The only active user runtime is the Yazelix-managed Nushell route. Agent sync/lock/doctor |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 17 | `status` | Verbs: `auto-detect · install · reset · auto-fix · add-repo · graph · lock · doctor`. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 21 | `status` | and a **`doctor`** diagnostic were adopted from the kasetto catalog. |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 38 | `metrics` | Phase 4+5 add-repo+telemetry · Phase 3 reset/auto-fix · GUI theme · PRD · |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 83 | `status` | # 3c. doctor — read-only health (writability, toolchains, sudo, UEFI, GPU) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 84 | `status` | cargo run -p envctl -- doctor |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 85 | `status` | cargo run -p envctl -- doctor --json |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/README.md` | 26 | `status` | \| `doctor` \| read-only health: writability, toolchains, sudo, UEFI/Secure-Boot, GPU, last-op \| — \| |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/README.md` | 109 | `metrics` | Dashboard (live GPU/CPU/mem telemetry) · Components grid (install/fix per row) · |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 17 | `status` | `graph`, `lock`, `doctor` (see `README.md`). |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 29 | `status` | (`crates/cli/src/bin/meta-env.rs`, native `meta_plugin_protocol::run_plugin`): `meta env doctor`, |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 103 | `metrics` | bash ci/gates/loop-state.sh # forge-loop counter integrity: ints, cadence>=1, cycles_total monotonic & >= last_wrapup (TASK-0041) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 217 | `status` | `doctor` is green** ("install everything", "set up the box", "loop until installed"), use |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 218 | `status` | **`env-install-loop`** (the same loop+relay continuity, driving envctl's `doctor`/`install`/ |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 227 | `status` | drift/lock/doctor → `env-stabilize`; conventions → `agent-env-config`.) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 263 | `status` | \| 2026-06-05 \| Add env-install-loop (whole-box provisioning loop) \| skills/env-install-loop; agents/continuity-steward; skills/session-relay \| First-class loop to drive the workstation to fully-installed/healthy/... via  |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 268 | `alerts` | \| 2026-06-12 \| Dashboard panes default to shell; require human opt-in for Claude \| assets/scripts/envctl-da...; assets/scripts/envctl-op...; manifest/dashboard.toml \| Prevent auto-spawn of idle Claude sessions in every z |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 270 | `metrics` | \| 2026-06-13 \| Add `handoff-kernel-engineer` agent (Epic A) + seed loop_state to schema + reconcile backlog \| agents/handoff-kernel-en....md; skills/feature-forge (crew table + Epic-A Build routing); .handoff/loop/{loop_ |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 272 | `status` | \| 2026-06-13 \| Eject `rust-port-merge` harness for the kasetto absorption (Epic C / TASK-0012) \| .claude/skills/{rust-port,rust-port-inventory,rust-port-translate,rust-port-parity,rust-port-merge,cross-repo-reference,icm |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 282 | `metrics` | \| 2026-06-18 \| Epic G deep audit + Tier-1 hardening (TASK-0041/0042/0043) \| `.handoff/loop/backlog.md` (Epic G plan, TASK-0041..0052, Tier-2/3 decisions LOCKED); ci/gates/loop-state.sh + scripts/tests/test-loop-....sh (N |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/CLAUDE.md` | 317 | `metrics` | rtk next build # Next.js build with route metrics (87%) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/AGENTS.md` | 18 | `status` | `graph`, `lock`, `doctor` (see `README.md`). |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/AGENTS.md` | 87 | `metrics` | bash ci/gates/loop-state.sh # forge-loop counter integrity: ints, cadence>=1, cycles_total monotonic & >= last_wrapup (TASK-0041) |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/AGENTS.md` | 266 | `status` | `doctor` is green** ("install everything", "set up the box", "loop until installed"), use |
| ... |  |  | 136 more entries in JSON artifact |

## Runbooks And Operator Evidence

| file | line | signal | evidence |
|---|---:|---|---|
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-env.yaml` | 9 | `verification` | # envctl agent lock --check --locked --color never # read-only, zero-network |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-env.yaml` | 13 | `verification` | # envctl agent lock --check --locked # zero-network; exits 1 on drift |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/agent-env.yaml` | 26 | `verification` | # The only active user runtime is the Yazelix-managed Nushell route. Agent sync/lock/doctor |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 8 | `verification` | \| date \| harness \| lesson (class, generalized) \| evidence (cycle/finding) \| recurrence \| routed-to \| status \| |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 11 | `verification` | \| date \| harness \| lesson (class, generalized) \| evidence \| recurrence \| routed-to \| status \| |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 13 | `runbook` | \| 2026-06-17 \| feature-forge \| Verify a triggering claim that asserts concrete code state against HEAD before designing — cross-session relay/handoff claims go stale; a plan built on a false premise wastes a cycle (no-fa |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 16 | `verification` | \| 2026-06-17 \| feature-forge \| The guardian must classify clippy findings by BOTH axis (gate form vs `--all-targets`) and origin (touched vs untouched file): `--all-targets`-only in an untouched file = inherited red (NOT |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 19 | `verification` | \| 2026-06-18 \| forge-loop \| TICK-ON-MERGED, not tick-on-armed: "guardian PASS + `gh pr merge --auto`" only ARMS a merge (a required check can still block it). Terminal Done must require `gh pr view <N>` == MERGED; armed- |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 20 | `verification` | \| 2026-06-18 \| feature-forge \| A retirement/absorption is not DONE until the on-disk artifacts the new code RESOLVES are migrated AND any claimed enforcement actually runs — a documented-but-unwired gate is worse than no |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 21 | `runbook` | \| 2026-06-18 \| forge-loop \| High-churn shared state committed in feature branches needs a merge DRIVER, not git's default 3-way — append-heavy files (loop_state/backlog) silently CONCATENATE across non-overlapping region |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 22 | `verification` | \| 2026-06-18 \| feature-forge \| Prevent drift at PICK-time, not just at wrap-up: a frozen-consumer-contract check that runs only as a reactive end-of-session sweep fires a cycle too late. Grep for a frozen CLI/RPC/JSON co |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 23 | `verification` | \| 2026-06-18 \| forge-loop \| When a self-cleaning mechanism exists for ONE consistency surface, check every PARALLEL surface — a fix on the remote leaves the local mirror stale. Sync all of them, FF-only + clean-only (nev |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 25 | `verification` | \| 2026-06-18 \| forge-loop \| The squash-robust merge ORACLE is the GitHub merged-PR head-ref, not the local `[gone]`/ancestor heuristics. After a squash merge the tip is never an ancestor of master, and a deleted upstream |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 26 | `runbook` | \| 2026-06-18 \| forge-loop \| Irreversible EXTERNAL mutations (remote-branch deletes, security-advisory dismissals, anything that leaves the repo and can't be `git reset`) are a correct human wall — gate them on explicit o |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 28 | `verification` | \| 2026-06-22 \| feature-forge \| RECURRENCE-CONFIRM (no new upgrade): the verify-against-HEAD-befo... class (rows 2026-06-17 Phase-0 step 4 + 2026-06-18 Phase-0 step 5) generalizes beyond a *suspect stale relay claim* to a |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 31 | `verification` | \| 2026-06-23 \| feature-forge \| Vendor self-extracting installers (makeself: CUDA `.run`, many `.sh` bundles) try to spawn a terminal/xterm when stdout is NOT a TTY — under automation they die `exec: -title` (exit 127) wi |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 34 | `runbook` | \| 2026-06-23 \| forge-loop \| When a subagent dies mid-cycle (weekly model limit / terminal API error), the orchestrator must finish the cycle deterministically rather than abandon it — the agent's edits persist in the wor |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 36 | `runbook` | \| 2026-06-27 \| plan-loop \| Phantom-workspace / sibling-escape environment hazard: a stray `Cargo.toml` in an ancestor dir makes Cargo walk UP and silently absorb the target into a foreign virtual workspace, breaking stan |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 38 | `runbook` | \| 2026-06-27 \| plan-loop \| Weave as in-cycle cross-loop coordination (NEW capability): planning loops can use a weave A2A round-trip DURING a cycle to get another loop/session to verify a plan, fold the corrections, and  |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 39 | `verification` | \| 2026-06-28 \| forge-loop \| Owner-supervised migrations should climb a review ladder in separate PRs — status/precondition, validation, scaffold, explicit writer, and only then any live movement. Do not combine manifest  |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 40 | `verification` | \| 2026-07-09 \| feature-forge \| Re-pin a RED-authored acceptance pattern against the REAL surface at GREEN: a test written before its surface exists asserts a GUESSED oracle that can be both false-RED (never-matchable) AN |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 41 | `verification` | \| 2026-07-09 \| feature-forge \| Similarity/ranking test fixtures need DECISIVE angular margins (≥45°), not near-ties: ANN/HNSW float precision reorders near-equal cosine scores nondeterministically, so a ~6°-apart fixture |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/LESSONS.md` | 43 | `verification` | \| 2026-07-09 \| forge-loop \| Hook-enforcement verify-at-pick must account for SESSION-SNAPSHOT semantics: Claude Code binds the hook set at session start, so a hook wired mid-session does NOT fire in the running session a |
| `src/flexnetos_runner/_work/actions-runner-02-work/envctl/envctl/HANDOFF.md` | 1 | `runbook` | # envctl — HANDOFF / verification guide |
| ... |  |  | 136 more entries in JSON artifact |

## Scan Limits

- Files scanned: `2500`
- Max files: `2500`
- Max file bytes: `600000`
- Truncated: `True`
- Skipped: `{"max_files_reached": 1, "too_large": 10, "unsupported_suffix": 337}`

## Validation

- Artifact registry persisted path and content hash for the canonical markdown and task JSON artifacts.
- Blocked path policy excluded `.env`, `secrets`, `private_keys`, `*.pem`, and `*.key` paths.
- Proof record links this markdown, the JSON artifact, the generation report, and the execution log.
