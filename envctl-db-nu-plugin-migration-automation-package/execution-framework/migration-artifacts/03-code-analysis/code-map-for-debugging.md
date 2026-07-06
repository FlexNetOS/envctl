# Code Map For Debugging

Task: `ART-113_DEBUG_CODE_MAP`
Generated at: `2026-07-04T23:21:07+00:00`
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
| `WORKLOG.md` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.handoff/loop/merge-ledger.md` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.handoff/loop/merge-ledger.md` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/brain-in-the-fish/crates/cli/src/main.rs` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/cluaiz/Inference-engine/engines/src/bin/legacy/ignition.rs` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/cluaiz/cmd/src/main.rs` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/inferrs/candle-core/src/tensor.rs` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/inferrs/inferrs-benchmark/src/main.rs` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/algotune/run_benchmark.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/algotune/task_adapter.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/arc_benchmark/post_evolution_eval.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/attention_optimization/evaluator.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/attention_optimization/legacy/prev_sim__works_evaluator.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/attention_optimization/scripts/to_real_mlir.sh` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/k_module_problem/iterative_agent.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/lm_eval/lm-eval.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/mlx_metal_kernel_opt/evaluator.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/mlx_metal_kernel_opt/qwen3_benchmark_suite.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/mlx_metal_kernel_opt/run_benchmarks.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/r_robust_regression/evaluator.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/rust_adaptive_sort/evaluator.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/symbolic_regression/data_api.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/examples/symbolic_regression/eval.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/scripts/visualizer.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/tests/test_api.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/tests/test_artifacts_integration.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/tests/test_cascade_validation.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/tests/test_database.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/.worktrees/external-sources/openevolve/tests/test_iteration_counting.py` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |
| `src/teri/.worktrees/issue-86-source-wires/scripts/gitkb-connect-service-edges.js` | 10 | control_flow, entry_points, errors, external_calls, logs, metrics_alerts, runbooks |

## Entry Points

| file | line | signal | evidence |
|---|---:|---|---|
| `LOCAL_WORKAROUNDS.md` | 540 | `desktop-exec` | Exec="/home/flexnetos/.nix-profile/bin/yzx" desktop launch |
| `LOCAL_WORKAROUNDS.md` | 549 | `desktop-exec` | Exec=/usr/bin/env YAZELIX_LAYOUT_OVERRIDE="/home/flexnetos/FlexNetO....kdl" "/home/flexnetos/.nix-profile/bin/yzx" desktop launch |
| `LOCAL_WORKAROUNDS.md` | 787 | `desktop-exec` | Exec="/home/flexnetos/.nix-profile/bin/yzx" desktop launch |
| `LOCAL_WORKAROUNDS.md` | 928 | `desktop-exec` | Exec=/usr/bin/env YAZELIX_LAYOUT_OVERRIDE="/home/flexnetos/FlexNetO....kdl" "/home/flexnetos/.nix-profile/bin/yzx" desktop launch |
| `WORKLOG.md` | 798 | `desktop-exec` | Exec="/home/flexnetos/.nix-profile/bin/yzx" desktop launch |
| `WORKLOG.md` | 801 | `desktop-exec` | Exec=/usr/bin/env YAZELIX_LAYOUT_OVERRIDE="/home/flexnetos/FlexNetO....kdl" "/home/flexnetos/.nix-profile/bin/yzx" desktop launch |
| `WORKLOG.md` | 1114 | `desktop-exec` | Exec="/home/flexnetos/.nix-profile/bin/yzx" desktop launch |
| `WORKLOG.md` | 1152 | `desktop-exec` | Exec="/home/flexnetos/.nix-profile/bin/yzx" desktop launch |
| `src/teri/tests/social_world_e2e_test.rs` | 9 | `shell-entry` | #![cfg(feature = "sqlite")] |
| `src/teri/src/main.rs` | 61 | `rust-main` | #[tokio::main] |
| `src/teri/src/main.rs` | 62 | `rust-main` | async fn main() -> Result<()> { |
| `src/teri/src/i18n/locales/en.json` | 93 | `package-script` | "buildCompleteDesc": "Graph build is complete. Proceed to the next step for simulation environment setup.", |
| `src/teri/src/i18n/locales/en.json` | 266 | `package-script` | "toolQuickSearchDesc": "GraphRAG-based instant query interface with optimized indexing for fast extraction of node attributes and discrete facts", |
| `src/teri/src/i18n/locales/en.json` | 423 | `package-script` | "readingNodeData": "Reading node data...", |
| `src/teri/src/i18n/locales/en.json` | 627 | `package-script` | "fetchingAllNodes": "Fetching all nodes for graph {graphId}...", |
| `src/teri/src/i18n/locales/en.json` | 628 | `package-script` | "fetchedNodes": "Fetched {count} nodes", |
| `src/teri/src/i18n/locales/en.json` | 631 | `package-script` | "fetchingNodeDetail": "Fetching node detail: {uuid}...", |
| `src/teri/src/i18n/locales/en.json` | 632 | `package-script` | "fetchNodeDetailOp": "Fetch node detail (uuid={uuid}...)", |
| `src/teri/src/i18n/locales/en.json` | 633 | `package-script` | "fetchNodeDetailFailed": "Failed to fetch node detail: {error}", |
| `src/teri/src/i18n/locales/en.json` | 634 | `package-script` | "fetchingNodeEdges": "Fetching edges for node {uuid}...", |
| `src/teri/src/i18n/locales/en.json` | 635 | `package-script` | "foundNodeEdges": "Found {count} edges related to node", |
| `src/teri/src/i18n/locales/en.json` | 636 | `package-script` | "fetchNodeEdgesFailed": "Failed to fetch node edges: {error}", |
| `src/teri/frontend/package.json` | 6 | `package-script` | "scripts": { |
| `src/teri/frontend/package.json` | 7 | `package-script` | "dev": "vite --host", |
| ... |  |  | 136 more entries in JSON artifact |

## Control Flow Candidates

| file | line | signal | evidence |
|---|---:|---|---|
| `WORKLOG.md` | 1065 | `workflow` | snapshots and Beads state before edits, blocks task completion without live |
| `WORKLOG.md` | 2068 | `workflow` | `implements`, `depends_on`, `blocks`/`blocked_by`, and `parent_of`/`child_of`. |
| `.kb/AGENTS.md` | 811 | `js-function` | - **Before changing a function signature**: Run `kb_callers <symbol>` to find all call sites that will need updates |
| `.kb/AGENTS.md` | 838 | `js-function` | \| `kb_callees` \| Find what a function calls \| `kb_callees --symbol login` \| |
| `.kb/skills/kb-start/SKILL.md` | 57 | `workflow` | - **Dependencies**: What this blocks, what blocks this (`kb_graph`) |
| `.kb/skills/code-intelligence/SKILL.md` | 12 | `js-function` | - Finding callers of a function or method |
| `.kb/skills/code-intelligence/SKILL.md` | 13 | `js-function` | - Finding what a function calls (callees) |
| `.kb/skills/code-intelligence/SKILL.md` | 50 | `js-function` | \| `grep` for function callers \| `git-kb code callers "<symbol>" --json` — actual call sites from call graph \| |
| `.kb/skills/refactor-safety/SKILL.md` | 12 | `js-function` | - Before changing a function signature |
| `.kb/skills/refactor-safety/SKILL.md` | 31 | `js-function` | 3. **Get callees (what this function depends on):** |
| `.kb/skills/refactor-safety/SKILL.md` | 60 | `js-function` | - Signature: `async function symbolName(param: Type): Promise<Result>` |
| `.kb/skills/kb-review/SKILL.md` | 33 | `js-function` | - Look for test function names that match |
| `.kb/skills/kb-review/SKILL.md` | 69 | `js-function` | \| 1 \| [criterion text] \| DONE \| [file/function that satisfies it] \| |
| `.kb/skills/kb-close/SKILL.md` | 50 | `workflow` | Use `kb_graph` to see if this task has children or blocks other tasks: |
| `.kb/skills/kb-close/SKILL.md` | 53 | `workflow` | - If it blocks other tasks, note: "Closing this will unblock: [list]." |
| `.kb/skills/understand/SKILL.md` | 40 | `js-function` | When the argument contains `::` or looks like a function name: |
| `.kb/skills/kb-create/SKILL.md` | 6 | `sql-change` | Create a new GitKB document based on the user's input. |
| `.kb/skills/kb-create/SKILL.md` | 84 | `sql-change` | Create a concise, descriptive title from the user's description. Use title case. Keep it under 80 characters. |
| `.kb/skills/kb-create/SKILL.md` | 210 | `workflow` | - If this task blocks or is blocked by another, mention it in the body with `[[wikilinks]]` |
| `.kb/skills/explore/SKILL.md` | 58 | `js-function` | export async function login(credentials: Credentials): Promise<Token> |
| `.kb/skills/explore/SKILL.md` | 64 | `js-function` | export function validate(token: Token): boolean |
| `.kb/skills/kb-commit/SKILL.md` | 23 | `workflow` | If the diff shows changes to fields that are graph-derived (`blocks`, `children`, `references`), warn: |
| `.kb/skills/kb-commit/SKILL.md` | 24 | `workflow` | > "The field `blocks` is graph-derived — it's computed from other documents' `blocked_by` fields. Committing it may cause unexpected behavior. Consider removing it from the frontmatter." |
| `src/teri/FEATURE-PARITY.md` | 254 | `workflow` | unit tests (debounce / one-job / budget-cap / persist-reload continuity / per-domain isolation / |
| ... |  |  | 136 more entries in JSON artifact |

## External Calls

| file | line | signal | evidence |
|---|---:|---|---|
| `LOCAL_WORKAROUNDS.md` | 19 | `tool-call` | under `.codex/hooks.json` and `.codex/hooks/`, plus `features.hooks = true` in |
| `LOCAL_WORKAROUNDS.md` | 25 | `tool-call` | /home/flexnetos/FlexNetO....tar.gz |
| `LOCAL_WORKAROUNDS.md` | 32 | `tool-call` | /home/flexnetos/.codex/config.toml -> features.hooks = false |
| `LOCAL_WORKAROUNDS.md` | 33 | `tool-call` | /home/flexnetos/FlexNetOS/.codex/config.toml -> features.hooks = false |
| `LOCAL_WORKAROUNDS.md` | 34 | `tool-call` | /home/flexnetos/FlexNetOS/.codex/hooks.json -> removed |
| `LOCAL_WORKAROUNDS.md` | 35 | `tool-call` | /home/flexnetos/FlexNetOS/.codex/hooks.safe.json -> removed |
| `LOCAL_WORKAROUNDS.md` | 36 | `tool-call` | /home/flexnetos/FlexNetOS/.codex/hooks/ -> removed |
| `LOCAL_WORKAROUNDS.md` | 57 | `tool-call` | /home/flexnetos/FlexNetO....tar.gz |
| `LOCAL_WORKAROUNDS.md` | 73 | `tool-call` | command_codex_usage_command "... codex --display both --periods 5h,week ..." |
| `LOCAL_WORKAROUNDS.md` | 103 | `tool-call` | /home/flexnetos/.codex/config.toml/.codex |
| `LOCAL_WORKAROUNDS.md` | 114 | `tool-call` | /home/flexnetos/FlexNetO....patch |
| `LOCAL_WORKAROUNDS.md` | 122 | `mcp` | ## Codex GitKB MCP Route Dedup - 2026-07-03 |
| `LOCAL_WORKAROUNDS.md` | 124 | `mcp` | The active Codex config had two GitKB MCP server registrations: |
| `LOCAL_WORKAROUNDS.md` | 127 | `mcp` | gitkb -> /home/flexnetos/FlexNetO... |
| `LOCAL_WORKAROUNDS.md` | 128 | `mcp` | gitkb-yazelix -> /home/flexnetos/FlexNetO... |
| `LOCAL_WORKAROUNDS.md` | 136 | `tool-call` | /home/flexnetos/FlexNetO....tar.gz |
| `LOCAL_WORKAROUNDS.md` | 143 | `mcp` | Codex MCP: gitkb only |
| `LOCAL_WORKAROUNDS.md` | 144 | `mcp` | Wrapper: /home/flexnetos/FlexNetO... |
| `LOCAL_WORKAROUNDS.md` | 149 | `mcp` | manual use, but it should not be registered as a second top-level Codex MCP |
| `LOCAL_WORKAROUNDS.md` | 150 | `mcp` | server unless a future design gives it a non-overlapping tool namespace. |
| `LOCAL_WORKAROUNDS.md` | 154 | `tool-call` | The clean shell did not resolve an `envctl` command from the workspace |
| `LOCAL_WORKAROUNDS.md` | 155 | `tool-call` | frontdoor path. Built the envctl release binary from source, then exposed it |
| `LOCAL_WORKAROUNDS.md` | 159 | `tool-call` | /home/flexnetos/FlexNetO... -> /home/flexnetos/FlexNetO... |
| `LOCAL_WORKAROUNDS.md` | 165 | `tool-call` | env -i HOME=/home/flexnetos USER=flexnetos LOGNAME=flexnetos PATH=/home/flexnetos/FlexNetO...:/home/flexnetos/.local/bin:/home/flexnetos/.nix-profile/bin:/run/current-system/sw/bin:/usr/bin:/bin envctl --version -> envct |
| ... |  |  | 136 more entries in JSON artifact |

## Errors, Retries, Timeouts

| file | line | signal | evidence |
|---|---:|---|---|
| `LOCAL_WORKAROUNDS.md` | 1 | `validation` | # Local Workarounds and Runtime Proof |
| `LOCAL_WORKAROUNDS.md` | 68 | `validation` | Proof after regeneration: |
| `LOCAL_WORKAROUNDS.md` | 89 | `validation` | Final clean-environment proof with the same layout override reports generated |
| `LOCAL_WORKAROUNDS.md` | 148 | `validation` | The Yazelix wrapper remains available on disk for explicit local proof or |
| `LOCAL_WORKAROUNDS.md` | 162 | `validation` | Clean environment proof: |
| `LOCAL_WORKAROUNDS.md` | 197 | `validation` | Clean environment proof: |
| `LOCAL_WORKAROUNDS.md` | 230 | `validation` | Clean environment proof: |
| `LOCAL_WORKAROUNDS.md` | 238 | `validation` | Fresh desktop proof: |
| `LOCAL_WORKAROUNDS.md` | 280 | `validation` | Current proof: |
| `LOCAL_WORKAROUNDS.md` | 329 | `validation` | Fresh FlexNetOS Agent desktop proof: |
| `LOCAL_WORKAROUNDS.md` | 352 | `validation` | Clipboard tool proof: |
| `LOCAL_WORKAROUNDS.md` | 357 | `validation` | round-tripped flexnetos-yazelix-clipbo... |
| `LOCAL_WORKAROUNDS.md` | 360 | `validation` | This replaced the host clipboard contents with the proof string. Older Mars/Rio |
| `LOCAL_WORKAROUNDS.md` | 374 | `validation` | Current proof: |
| `LOCAL_WORKAROUNDS.md` | 384 | `validation` | The Codex stop gate denied shutdown after Meta GitKB verification failed. Before |
| `LOCAL_WORKAROUNDS.md` | 419 | `validation` | Current proof: |
| `LOCAL_WORKAROUNDS.md` | 476 | `validation` | Current proof: |
| `LOCAL_WORKAROUNDS.md` | 508 | `validation` | Clipboard proof: |
| `LOCAL_WORKAROUNDS.md` | 516 | `validation` | Desktop proof: |
| `LOCAL_WORKAROUNDS.md` | 553 | `validation` | Both desktop files validate with `desktop-file-validate`, and |
| `LOCAL_WORKAROUNDS.md` | 562 | `validation` | Custom launcher smoke proof: |
| `LOCAL_WORKAROUNDS.md` | 586 | `validation` | Meta org destructive operations: archive-first and compressed-evidence required |
| `LOCAL_WORKAROUNDS.md` | 615 | `validation` | Current proof: |
| `LOCAL_WORKAROUNDS.md` | 656 | `validation` | Proof on 2026-07-01: Codex MCP lists `gitkb` and `gitkb-yazelix`; both |
| ... |  |  | 136 more entries in JSON artifact |

## Logs And Audit Signals

| file | line | signal | evidence |
|---|---:|---|---|
| `LOCAL_WORKAROUNDS.md` | 6 | `audit-log` | These entries are evidence and operational guardrails, not substitutes for repo |
| `LOCAL_WORKAROUNDS.md` | 133 | `audit-log` | home config, workspace mirror, and local evidence files before mutation: |
| `LOCAL_WORKAROUNDS.md` | 242 | `nu-log` | latest launch log: /home/flexnetos/.local/share/yazelix/logs....log |
| `LOCAL_WORKAROUNDS.md` | 248 | `audit-log` | not evidence of a broken package, missing bundled `rg`, or zsh-fork resource |
| `LOCAL_WORKAROUNDS.md` | 333 | `nu-log` | latest fixed launch log: /home/flexnetos/.local/share/yazelix/logs....log |
| `LOCAL_WORKAROUNDS.md` | 337 | `nu-log` | The launch log and `/proc/1175394/environ` both show: |
| `LOCAL_WORKAROUNDS.md` | 519 | `nu-log` | latest launch log: /home/flexnetos/.local/share/yazelix/logs....log |
| `LOCAL_WORKAROUNDS.md` | 533 | `nu-log` | /home/flexnetos/.nix-profile/bin/yzx desktop install --print-path |
| `LOCAL_WORKAROUNDS.md` | 566 | `nu-log` | latest launch log: /home/flexnetos/.local/share/yazelix/logs....log |
| `LOCAL_WORKAROUNDS.md` | 585 | `audit-log` | full access: allowed for repair, upgrade, and evidence collection |
| `LOCAL_WORKAROUNDS.md` | 586 | `audit-log` | Meta org destructive operations: archive-first and compressed-evidence required |
| `LOCAL_WORKAROUNDS.md` | 596 | `nu-log` | /home/flexnetos/FlexNetO... |
| `LOCAL_WORKAROUNDS.md` | 720 | `nu-log` | /home/flexnetos/FlexNetO....log |
| `LOCAL_WORKAROUNDS.md` | 724 | `nu-log` | That log proves no active Zellij sessions, no Zellij sockets, no matching |
| `LOCAL_WORKAROUNDS.md` | 767 | `nu-log` | /home/flexnetos/FlexNetO....log |
| `LOCAL_WORKAROUNDS.md` | 771 | `audit-log` | This is host-local runtime glue and evidence, not a Yazelix product fix. Do not |
| `LOCAL_WORKAROUNDS.md` | 792 | `nu-log` | GNOME/Wayland host and writes a non-empty Mars launch log. |
| `LOCAL_WORKAROUNDS.md` | 797 | `nu-log` | /home/flexnetos/FlexNetO....log |
| `LOCAL_WORKAROUNDS.md` | 802 | `nu-log` | /home/flexnetos/FlexNetO....log |
| `LOCAL_WORKAROUNDS.md` | 804 | `nu-log` | latest successful log: /home/flexnetos/.local/share/yazelix/logs....log |
| `LOCAL_WORKAROUNDS.md` | 805 | `nu-log` | latest successful log size: 603 |
| `LOCAL_WORKAROUNDS.md` | 812 | `nu-log` | /home/flexnetos/FlexNetO....log |
| `LOCAL_WORKAROUNDS.md` | 816 | `nu-log` | That log proves: |
| `LOCAL_WORKAROUNDS.md` | 819 | `nu-log` | yzx desktop install --print-path -> /home/flexnetos/.local/share/applications....yazelix.Yazelix.Mars.desktop |
| ... |  |  | 136 more entries in JSON artifact |

## Metrics And Alerts

| file | line | signal | evidence |
|---|---:|---|---|
| `LOCAL_WORKAROUNDS.md` | 75 | `status` | /home/flexnetos/.nix-profile/bin/yzx doctor -> all checks passed |
| `LOCAL_WORKAROUNDS.md` | 86 | `status` | /home/flexnetos/.nix-profile/bin/yzx doctor --fix |
| `LOCAL_WORKAROUNDS.md` | 90 | `status` | runtime state up to date, `yzx doctor` all passed, and no `--display quota` |
| `LOCAL_WORKAROUNDS.md` | 202 | `status` | codex doctor --all -> 18 ok, 1 notes, 0 warn, 0 fail |
| `LOCAL_WORKAROUNDS.md` | 234 | `status` | codex doctor --all -> 17 ok, 2 notes, 1 warn, 0 fail; bundled /nix/store/gh9cc.../codex-path/rg detected |
| `LOCAL_WORKAROUNDS.md` | 247 | `status` | The only remaining Codex doctor warning is the stale app-server socket. It is |
| `LOCAL_WORKAROUNDS.md` | 277 | `alerts` | codex plugin add slack@meta-plugins-codex --json |
| `LOCAL_WORKAROUNDS.md` | 285 | `status` | codex doctor --all -> 18 ok, 0 warn, 0 fail |
| `LOCAL_WORKAROUNDS.md` | 326 | `status` | clean-env yzx doctor -> all checks passed |
| `LOCAL_WORKAROUNDS.md` | 424 | `status` | PATH=/home/flexnetos/FlexNetO...:$PATH /home/flexnetos/FlexNetO... doctor --json -> 16 repos discovered |
| `LOCAL_WORKAROUNDS.md` | 428 | `status` | bash /home/flexnetos/FlexNetOS/.codex/hooks/flexnetos-co....sh -> passed |
| `LOCAL_WORKAROUNDS.md` | 482 | `status` | bash /home/flexnetos/FlexNetOS/.codex/hooks/flexnetos-co....sh -> passed |
| `LOCAL_WORKAROUNDS.md` | 524 | `status` | path ahead of the profile in `PATH`; clean-environment `yzx doctor` removed the |
| `LOCAL_WORKAROUNDS.md` | 555 | `status` | Clean-environment `yzx doctor` now reports: |
| `LOCAL_WORKAROUNDS.md` | 619 | `status` | bash .codex/hooks/flexnetos-co....sh -> FlexNetOS Codex hook doctor passed |
| `LOCAL_WORKAROUNDS.md` | 667 | `status` | src/meta git-kb doctor -> 11 checks passed, no issues found |
| `LOCAL_WORKAROUNDS.md` | 679 | `status` | Known caveat: `git-kb daemon start --background` reaches readiness but does not |
| `LOCAL_WORKAROUNDS.md` | 727 | `status` | health, and `meta exec -- git-kb verify/status` passing across all 16 projects. |
| `LOCAL_WORKAROUNDS.md` | 806 | `status` | doctor result: all checks passed |
| `LOCAL_WORKAROUNDS.md` | 843 | `status` | With `YAZELIX_SKIP_STABLE_WRAP...=1`, the source-built doctor |
| `LOCAL_WORKAROUNDS.md` | 1055 | `status` | `get_active_tab_session_s...` response, and `yzx doctor` passing. The stale |
| `LOCAL_WORKAROUNDS.md` | 1123 | `status` | /home/flexnetos/FlexNetO....txt |
| `LOCAL_WORKAROUNDS.md` | 1127 | `status` | `0.143.0-alpha.35`. `codex doctor --all` reports `18 ok`, `0 warn`, and |
| `LOCAL_WORKAROUNDS.md` | 1141 | `status` | finds no matches in active or workspace Codex config, and `codex doctor --all` |
| ... |  |  | 136 more entries in JSON artifact |

## Runbooks And Operator Evidence

| file | line | signal | evidence |
|---|---:|---|---|
| `LOCAL_WORKAROUNDS.md` | 1 | `verification` | # Local Workarounds and Runtime Proof |
| `LOCAL_WORKAROUNDS.md` | 6 | `verification` | These entries are evidence and operational guardrails, not substitutes for repo |
| `LOCAL_WORKAROUNDS.md` | 68 | `verification` | Proof after regeneration: |
| `LOCAL_WORKAROUNDS.md` | 75 | `verification` | /home/flexnetos/.nix-profile/bin/yzx doctor -> all checks passed |
| `LOCAL_WORKAROUNDS.md` | 86 | `verification` | /home/flexnetos/.nix-profile/bin/yzx doctor --fix |
| `LOCAL_WORKAROUNDS.md` | 89 | `verification` | Final clean-environment proof with the same layout override reports generated |
| `LOCAL_WORKAROUNDS.md` | 90 | `verification` | runtime state up to date, `yzx doctor` all passed, and no `--display quota` |
| `LOCAL_WORKAROUNDS.md` | 98 | `verification` | ## Codex Linux Sandbox File-Root Metadata Patch Evidence - 2026-07-03 |
| `LOCAL_WORKAROUNDS.md` | 111 | `verification` | Evidence patch: |
| `LOCAL_WORKAROUNDS.md` | 133 | `verification` | home config, workspace mirror, and local evidence files before mutation: |
| `LOCAL_WORKAROUNDS.md` | 148 | `verification` | The Yazelix wrapper remains available on disk for explicit local proof or |
| `LOCAL_WORKAROUNDS.md` | 162 | `verification` | Clean environment proof: |
| `LOCAL_WORKAROUNDS.md` | 197 | `verification` | Clean environment proof: |
| `LOCAL_WORKAROUNDS.md` | 202 | `verification` | codex doctor --all -> 18 ok, 1 notes, 0 warn, 0 fail |
| `LOCAL_WORKAROUNDS.md` | 230 | `verification` | Clean environment proof: |
| `LOCAL_WORKAROUNDS.md` | 234 | `verification` | codex doctor --all -> 17 ok, 2 notes, 1 warn, 0 fail; bundled /nix/store/gh9cc.../codex-path/rg detected |
| `LOCAL_WORKAROUNDS.md` | 235 | `verification` | codex exec --ephemeral --skip-git-repo-check -s read-only -> CLEAN_ENV_CODEX_EXEC_OK |
| `LOCAL_WORKAROUNDS.md` | 238 | `verification` | Fresh desktop proof: |
| `LOCAL_WORKAROUNDS.md` | 247 | `verification` | The only remaining Codex doctor warning is the stale app-server socket. It is |
| `LOCAL_WORKAROUNDS.md` | 248 | `verification` | not evidence of a broken package, missing bundled `rg`, or zsh-fork resource |
| `LOCAL_WORKAROUNDS.md` | 280 | `verification` | Current proof: |
| `LOCAL_WORKAROUNDS.md` | 285 | `verification` | codex doctor --all -> 18 ok, 0 warn, 0 fail |
| `LOCAL_WORKAROUNDS.md` | 311 | `runbook` | Mars desktop launch config handoff was fixed in source. This is installed |
| `LOCAL_WORKAROUNDS.md` | 326 | `verification` | clean-env yzx doctor -> all checks passed |
| ... |  |  | 136 more entries in JSON artifact |

## Scan Limits

- Files scanned: `2500`
- Max files: `2500`
- Max file bytes: `600000`
- Truncated: `True`
- Skipped: `{"max_files_reached": 1, "too_large": 11, "unsupported_suffix": 607}`

## Validation

- Artifact registry persisted path and content hash for the canonical markdown and task JSON artifacts.
- Blocked path policy excluded `.env`, `secrets`, `private_keys`, `*.pem`, and `*.key` paths.
- Proof record links this markdown, the JSON artifact, the generation report, and the execution log.
