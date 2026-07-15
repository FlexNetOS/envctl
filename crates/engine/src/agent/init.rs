//! `Engine::agent_init` (C-13) — create a commented starter `agent-env.yaml` in the
//! current directory or in the global agent-env config dir.
//!
//! Fail-closed: refuses to overwrite an existing file unless `spec.force` is true.
//! The engine is non-printing; it emits `Event::AgentRunStarted` + `Event::AgentInitFinished`
//! and returns the typed outcome.

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
        sink.emit(Event::AgentRunStarted {
            verb: AgentVerb::Init,
            scope: AgentScope::Global,
            dry_run: false,
            lock_mode: "plain".into(),
        });

        let overwritten =
            envctl_agent_env::initialize_config_atomic(&path, TEMPLATE.as_bytes(), spec.force)
                .map_err(|e| {
                    if !spec.force && e.to_string().contains("already exists") {
                        anyhow::anyhow!(
                            "{} already exists — pass --force to overwrite",
                            path.display()
                        )
                    } else {
                        anyhow::anyhow!("write {}: {e}", path.display())
                    }
                })?;

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

    #[cfg(unix)]
    fn private_root(name: &str) -> std::path::PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "envctl-agent-init-{name}-{}-{stamp}",
            std::process::id()
        ));
        std::fs::create_dir(&root).unwrap();
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
        root
    }

    #[test]
    fn init_path_defaults_to_local_config() {
        let path = init_config_path(false).expect("local path");
        assert_eq!(path, std::path::PathBuf::from("agent-env.yaml"));
    }

    #[test]
    fn init_path_global_uses_agent_env_config_dir() {
        // Reads $XDG_CONFIG_HOME/$HOME — serialize against env-mutating tests
        // (e.g. doctor::doctor_runs_config_less) so a concurrent remove_var can't
        // make the global-path resolution observe an unset env.
        let _env = crate::test_env_lock();
        let path = init_config_path(true).expect("global path");
        assert!(path.ends_with("agent-env/agent-env.yaml"));
    }

    #[cfg(unix)]
    #[test]
    fn init_force_refuses_symlink_leaf_without_touching_target() {
        use std::os::unix::fs::symlink;

        let _env = crate::test_env_lock();
        let root = private_root("symlink-leaf");
        let victim = root.join("victim");
        let config = root.join("agent-env.yaml");
        std::fs::write(&victim, b"do-not-touch\n").unwrap();
        symlink(&victim, &config).unwrap();

        let previous_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&root).unwrap();
        let result = crate::Engine::detached().agent_init(
            crate::agent::AgentInitSpec {
                global: false,
                force: true,
            },
            &crate::event::EventSink::null(),
        );
        std::env::set_current_dir(previous_dir).unwrap();

        assert!(
            result.is_err(),
            "--force must never follow a config symlink"
        );
        assert_eq!(std::fs::read(&victim).unwrap(), b"do-not-touch\n");
        assert!(std::fs::symlink_metadata(&config)
            .unwrap()
            .file_type()
            .is_symlink());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn global_init_refuses_symlinked_intermediate_parent() {
        use std::os::unix::fs::symlink;

        let _env = crate::test_env_lock();
        let root = private_root("symlink-parent");
        let victim_dir = root.join("victim-dir");
        let xdg_link = root.join("xdg-link");
        std::fs::create_dir(&victim_dir).unwrap();
        symlink(&victim_dir, &xdg_link).unwrap();
        let previous_xdg = std::env::var_os("XDG_CONFIG_HOME");
        std::env::set_var("XDG_CONFIG_HOME", &xdg_link);

        let result = crate::Engine::detached().agent_init(
            crate::agent::AgentInitSpec {
                global: true,
                force: true,
            },
            &crate::event::EventSink::null(),
        );

        match previous_xdg {
            Some(value) => std::env::set_var("XDG_CONFIG_HOME", value),
            None => std::env::remove_var("XDG_CONFIG_HOME"),
        }
        assert!(
            result.is_err(),
            "init must reject a symlinked authority chain"
        );
        assert!(!victim_dir.join("agent-env/agent-env.yaml").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn global_init_creates_private_parents_independent_of_umask() {
        use std::os::unix::fs::PermissionsExt;

        struct UmaskRestore(rustix::fs::Mode);
        impl Drop for UmaskRestore {
            fn drop(&mut self) {
                rustix::process::umask(self.0);
            }
        }

        let _env = crate::test_env_lock();
        let root = private_root("private-parents");
        let xdg = root.join("nested/config");
        let previous_xdg = std::env::var_os("XDG_CONFIG_HOME");
        std::env::set_var("XDG_CONFIG_HOME", &xdg);
        let previous_umask = rustix::process::umask(rustix::fs::Mode::empty());
        let restore_umask = UmaskRestore(previous_umask);

        let result = crate::Engine::detached().agent_init(
            crate::agent::AgentInitSpec {
                global: true,
                force: false,
            },
            &crate::event::EventSink::null(),
        );

        drop(restore_umask);
        match previous_xdg {
            Some(value) => std::env::set_var("XDG_CONFIG_HOME", value),
            None => std::env::remove_var("XDG_CONFIG_HOME"),
        }
        let outcome = result.expect("secure global init");
        for directory in [root.join("nested"), xdg.clone(), xdg.join("agent-env")] {
            assert_eq!(
                std::fs::metadata(directory).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
        assert_eq!(
            std::fs::metadata(outcome.path)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o644
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
