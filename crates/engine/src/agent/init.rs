//! `Engine::agent_init` (C-13) — create a commented starter `agent-env.yaml` in the
//! current directory or in the global agent-env config dir.
//!
//! Fail-closed: refuses to overwrite an existing file unless `spec.force` is true.
//! The engine is non-printing; it emits `Event::AgentRunStarted` + `Event::AgentInitFinished`
//! and returns the typed outcome.

use std::fs;
use std::path::PathBuf;

use envctl_agent_env::config_path::{DEFAULT_CONFIG_FILENAME, DEFAULT_GLOBAL_CONFIG_FILENAME};
use envctl_agent_env::dirs::dirs_agent_env_config;

use crate::agent::report::{AgentInitOutcome, AgentVerb};
use crate::agent::{AgentInitSpec, AgentScope};
use crate::event::{Event, EventSink};
use crate::Engine;

const TEMPLATE: &str = r#"# envctl agent-env — declarative skills / MCPs / commands
#
# Scope: global (default) or project (install into current project)
# scope: project
#
# Target agent (see docs/KASETTO-FEATURES.md for supported presets)
# agent: claude-code
#
# Or set a custom destination directory
# destination: ~/.claude/skills

# skills:
#   - source: https://github.com/example/skill-pack
#     skills: "*"
#   - source: https://github.com/example/skill-pack
#     ref: v2.0            # pin to a git tag, commit SHA, or any ref
#     skills: "*"
#   - source: https://github.com/example/skill-pack
#     branch: develop       # track a specific branch
#     skills: "*"

# mcps:
#   - source: https://github.com/example/mcp-pack
#     mcps: "*"
#   - source: https://github.com/example/monorepo
#     ref: v1.0
#     mcps:
#       - github         # → mcps/github.json
#       - linear         # → mcps/linear.json
#   - source: https://github.com/example/other
#     mcps:
#       - name: my-server
#         path: tools    # → tools/my-server.json

# commands:
#   - source: https://github.com/example/commands
#     commands: "*"
#   - source: https://github.com/example/commands
#     ref: v1.0
#     sub-dir: commands
#     commands:
#       - review-pr
#       - name: deploy
#         path: ops
"#;

impl Engine {
    /// Create a starter agent-env config file. Fail-closed on existing files unless `--force`.
    pub fn agent_init(
        &self,
        spec: AgentInitSpec,
        sink: &EventSink,
    ) -> anyhow::Result<AgentInitOutcome> {
        let path = init_config_path(spec.global)?;
        let overwritten = path.exists();
        if overwritten && !spec.force {
            anyhow::bail!(
                "{} already exists — pass --force to overwrite",
                path.display()
            );
        }

        sink.emit(Event::AgentRunStarted {
            verb: AgentVerb::Init,
            scope: AgentScope::Global,
            dry_run: false,
            lock_mode: "plain".into(),
        });

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| anyhow::anyhow!("create {}: {e}", parent.display()))?;
        }
        fs::write(&path, TEMPLATE).map_err(|e| anyhow::anyhow!("write {}: {e}", path.display()))?;

        let outcome = AgentInitOutcome {
            path: path.to_string_lossy().to_string(),
            overwritten,
        };
        sink.emit(Event::AgentInitFinished {
            outcome: outcome.clone(),
        });
        Ok(outcome)
    }
}

fn init_config_path(global: bool) -> anyhow::Result<PathBuf> {
    if global {
        let dir = dirs_agent_env_config()
            .map_err(|e| anyhow::anyhow!("resolve agent-env config dir: {e}"))?;
        Ok(dir.join(DEFAULT_GLOBAL_CONFIG_FILENAME))
    } else {
        Ok(PathBuf::from(DEFAULT_CONFIG_FILENAME))
    }
}

#[cfg(test)]
mod tests {
    use super::init_config_path;

    #[test]
    fn init_path_defaults_to_local_config() {
        let path = init_config_path(false).expect("local path");
        assert_eq!(path, std::path::PathBuf::from("agent-env.yaml"));
    }

    #[test]
    fn init_path_global_uses_agent_env_config_dir() {
        let path = init_config_path(true).expect("global path");
        assert!(path.ends_with("agent-env/agent-env.yaml"));
    }
}
