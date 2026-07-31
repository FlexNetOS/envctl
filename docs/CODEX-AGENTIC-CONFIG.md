# Codex agentic configuration audit — 2026-06-26

This note records the project Codex configuration pass that updated `.codex/config.toml`,
`.codex/hooks.json`, `.codex/AGENTS.md`, and the three baseline custom-agent TOML files.
The retired lifecycle bundle remains archive-only. The active generated hook surface is
the profile-owned RTK `PreToolUse` hook for Bash only.

## Official-doc research used

Sources were restricted to current OpenAI Codex documentation fetched on 2026-06-26:

- Codex configuration layers: project `.codex/config.toml` is loaded for trusted projects and
  overlays user/system config; project config cannot set provider/auth redirection keys. Source:
  <https://developers.openai.com/codex/config-basic> and
  <https://developers.openai.com/codex/config-advanced>.
- Current sample config keys include `model_context_window`,
  `model_auto_compact_token_limit`, `tool_output_token_limit`, `project_doc_max_bytes`,
  `[agents].max_threads`, `[agents].job_max_runtime_seconds`, `[features]`, `[history]`,
  `[memories]`, `[sandbox_workspace_write]`, and `[tui].status_line`. Source:
  <https://developers.openai.com/codex/config-reference> and
  <https://developers.openai.com/codex/config-sample>.
- Recommended Codex model as of the fetched manual is `gpt-5.5`; reasoning effort supports
  `minimal|low|medium|high|xhigh`, and plan mode can use its own reasoning effort. Source:
  <https://developers.openai.com/codex/models> and
  <https://developers.openai.com/codex/config-sample>.
- Subagent workflows are enabled by default, use `.codex/agents/*.toml` for project-scoped
  custom agents, and expose `agents.max_threads`, `agents.max_depth`, and
  `agents.job_max_runtime_seconds`. Source: <https://developers.openai.com/codex/subagents>.
- Hooks are a stable lifecycle extension; project-local hooks load only in trusted projects.
  Source: <https://developers.openai.com/codex/hooks>.
- Memories are off by default; enabling `[features].memories` plus `[memories]` lets Codex use and
  generate local memory while AGENTS.md remains the mandatory team guidance surface. Source:
  <https://developers.openai.com/codex/memories>.
- MCP configuration belongs in config.toml; CLI and IDE share the same MCP config. Source:
  <https://developers.openai.com/codex/mcp>.
- Current June 2026 changelog items also reinforce that Codex remote/background workflows depend on
  fresh app versions and explicit host/connection setup; these are operational, not repo-local TOML
  keys. Source: <https://developers.openai.com/codex/changelog>.

## Findings and changes

| Area | Prior state | Change | Why |
| --- | --- | --- | --- |
| Context compaction | No explicit context window or compaction trigger. | Set `model_context_window = 128000` and `model_auto_compact_token_limit = 64000`. | Forces earlier compaction before long harness runs exhaust context. |
| Instruction truncation | `AGENTS.md` is about 37 KiB, above Codex default 32 KiB project-doc cap. | Set `project_doc_max_bytes = 65536`. | Prevents losing repo hard rules from first-turn instructions. |
| Tool-output bloat | No explicit per-tool stored-output cap. | Set `tool_output_token_limit = 12000`. | Keeps large command output from consuming the context window. |
| Model/reasoning | No repo default model/reasoning. | Set `model = "gpt-5.5"`, `model_reasoning_effort = "high"`, `plan_mode_reasoning_effort = "xhigh"`. | Aligns envctl with the current official Codex model guidance for complex coding/planning. |
| Background/subagents | Only 6 threads and no worker timeout. | Set `max_threads = 12`, `max_depth = 1`, `job_max_runtime_seconds = 3600`. | Supports broad fan-out without recursive runaway. |
| Hooks | Legacy lifecycle hooks used absolute paths and mixed responsibilities. | Generate only `/home/flexnetos/.nix-profile/bin/rtk hook claude` for `PreToolUse` with matcher `Bash`, and enable `[features].hooks`. | Keeps shell-command rewriting profile-owned while preventing legacy lifecycle hooks from returning. |
| Memories/history | Memory not explicitly enabled in project config; history size default. | Enable memories and set history cap to 100 MiB. | Improves cross-thread recovery while keeping team rules in AGENTS.md. |
| Subagent schema | Baseline three custom agent files lacked explicit `name`/`description`. | Add required standalone custom-agent fields. | Matches current `.codex/agents/*.toml` schema. |
| Project auth/provider | `model_provider = "openai"` was initially considered. | Deliberately omitted. | Current docs and `codex --strict-config doctor` confirm provider/auth keys are ignored in project config. |

## Validation

- `python3`/`tomllib` parsed `.codex/config.toml` and the edited agent TOML files.
- `codex --strict-config --cd . doctor` loaded the project config with no config warnings after
  removing the unsupported project-local `model_provider` key.

## Operational notes

- Keep credentials, provider redirects, and private auth in `/home/flexnetos/meta/var/lib/codex/config.toml`; project config
  should define repo-local behavior only.
- Background app-server / remote-control host setup is outside this repo-local config. Use the Codex
  app/remote-connection setup on the host when continuous app/mobile control is desired.
- Do not raise `agents.max_depth` above 1 unless a specific workflow needs recursive delegation;
  OpenAI docs warn that deeper nesting increases token usage, latency, and predictability risk.
