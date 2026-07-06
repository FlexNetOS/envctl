
# Multi-Agent Task Graph Columns

The task graph supports multi-agent and background helper execution through these fields:

| Field | Purpose |
|---|---|
| `owner_lane` | Logical execution lane such as planning, repo A, repo B, filesystem, verification, release, history. |
| `owner_agent` | Named agent responsible for the task. |
| `helper_id` | Background helper or shell helper identity. |
| `model_tag` | Routing tag such as `codex-5.4` or `gpt-5.3-spark`. |
| `agent_runtime` | Execution runtime, for example Codex CLI, shell, Python, or Nushell plugin. |
| `shell_mode` | Whether the helper can write, read-only, or requires approval. |
| `parallel_group` | Group of tasks that can run together. |
| `can_run_parallel` | Boolean lane flag. |
| `max_parallel` | Max concurrent tasks for a group or lane. |
| `heartbeat_file` | Progress heartbeat path. |
| `logs_uri` | Log path for audit and debugging. |
| `proof_uri` | Required proof record path. |

## Lane examples

- `lane_a_planning`
- `lane_b_repo_a`
- `lane_c_repo_b`
- `lane_d_filesystem`
- `lane_e_verification`
- `lane_f_release`
- `lane_g_history_scan`
