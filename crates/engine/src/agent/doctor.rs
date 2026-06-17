//! `Engine::agent_doctor` (TASK-0019, Item 1) — read-only local diagnostics for the agent-env
//! installation, ported from kasetto v3.2.0 `src/commands/doctor.rs`.
//!
//! Engine-first: this assembles the typed [`AgentDoctorReport`] (version, lock file, scope,
//! skills, install path, last sync, failures, MCP/command inventory, command-directory
//! writability checks, and the update-check block) and emits ONE `Event::AgentDoctored`. It is
//! non-printing and never writes. Both front-ends render the identical report — the CLI as the
//! grouped human view (or `--json`), the GUI as a Doctor sub-tab.

use std::path::Path;

use envctl_agent_env::config_path::default_config_path;
use envctl_agent_env::extend::load_config_any;
use envctl_agent_env::fsops::scope_root;
use envctl_agent_env::{
    all_command_global_targets, all_command_project_targets, command_global_targets,
    command_project_targets, dirs_home, lock, resolve_dest, Agent, Scope,
};

use crate::agent::report::{AgentCommandDirCheck, AgentDoctorReport, AgentUpdateCheck};
use crate::agent::{agent_lock_path, AgentScope};
use crate::event::{Event, EventSink};
use crate::Engine;

/// Options for `Engine::agent_doctor`.
#[derive(Clone, Debug, Default)]
pub struct AgentDoctorSpec {
    /// `--scope` override; `None` → the default scope (`Global`), config-less (kasetto parity).
    pub scope_override: Option<AgentScope>,
}

impl Engine {
    /// Run read-only agent-env diagnostics. Emits one `Event::AgentDoctored` and returns the
    /// typed report. Never writes.
    pub fn agent_doctor(
        &self,
        spec: AgentDoctorSpec,
        sink: &EventSink,
    ) -> anyhow::Result<AgentDoctorReport> {
        // Config-OPTIONAL, mirroring kasetto `doctor::run` and `Engine::agent_list`: doctor is a
        // read-only diagnostic that must run with NO `agent-env.yaml` present. The scope resolves
        // from the override else the default scope WITHOUT loading a config (kasetto:
        // `resolve_scope(scope_override, None)` → `Global` default); version/skills/mcps/commands
        // all derive from the LOCK, never from a required config.
        let scope: Scope = spec.scope_override.map(Into::into).unwrap_or(Scope::Global);
        let project_root = std::env::current_dir().unwrap_or_default();
        let lock_file = agent_lock_path(scope, &project_root)?;
        let lock = lock::load(&lock_file)?;
        let runtime =
            envctl_agent_env::runtime::load_runtime_state(scope, &project_root).unwrap_or_default();

        let version = env!("CARGO_PKG_VERSION").to_string();
        let state = lock.state();
        let root = scope_root(scope, &project_root)?;
        let root = &root;

        // Distinct parent directories of every installed skill (the active install paths).
        let mut install_paths: Vec<String> = state
            .skills
            .values()
            .map(|entry| {
                let p = resolve_dest(&entry.destination, root);
                p.parent().unwrap_or(&p).to_string_lossy().to_string()
            })
            .collect();
        install_paths.sort();
        install_paths.dedup();
        let installation_path = if install_paths.is_empty() {
            "none".to_string()
        } else if install_paths.len() == 1 {
            install_paths.remove(0)
        } else {
            install_paths.join(", ")
        };

        let mut skills: Vec<String> = state.skills.values().map(|e| e.skill.clone()).collect();
        skills.sort();

        let failures = runtime.load_latest_failures();
        let last_sync = runtime.last_run.clone();

        let managed_mcps = lock.list_installed_mcps();
        let managed_commands = lock.list_installed_commands();
        let command_dirs = collect_command_dirs(scope, &project_root);

        let scope_label = match scope {
            Scope::Global => "global".to_string(),
            Scope::Project => "project".to_string(),
        };

        let update_check = build_update_check(&version);

        let report = AgentDoctorReport {
            version,
            lock_file: lock_file.to_string_lossy().to_string(),
            scope: scope_label,
            skills,
            installation_path,
            last_sync,
            failures,
            mcps: managed_mcps,
            commands: managed_commands,
            command_dirs,
            update_check,
        };

        sink.emit(Event::AgentDoctored {
            report: report.clone(),
        });
        Ok(report)
    }
}

/// Scope the COMMAND DIRECTORIES check to the agents the config actually wires; fall back to
/// every supported agent when there is NO config or no agents configured (kasetto verbatim
/// semantics — "what does envctl know how to write to?"). Config-OPTIONAL: any load error or an
/// empty agent set takes the all-targets debugging view rather than erroring.
fn collect_command_dirs(scope: Scope, project_root: &Path) -> Vec<AgentCommandDirCheck> {
    let agents: Vec<Agent> = match load_config_any(&default_config_path()) {
        Ok((cfg, _, _)) => cfg.agents(),
        Err(_) => Vec::new(),
    };
    let targets = match scope {
        Scope::Project => {
            if agents.is_empty() {
                all_command_project_targets(project_root)
            } else {
                command_project_targets(project_root, &agents)
            }
        }
        Scope::Global => match dirs_home() {
            Ok(home) => {
                if agents.is_empty() {
                    all_command_global_targets(&home)
                } else {
                    command_global_targets(&home, &agents)
                }
            }
            Err(_) => return Vec::new(),
        },
    };
    targets
        .into_iter()
        .map(|t| AgentCommandDirCheck {
            writable: is_writable(&t.path),
            path: t.path.to_string_lossy().to_string(),
        })
        .collect()
}

/// Walk up to the first existing ancestor, then probe its write permissions (kasetto verbatim).
fn is_writable(path: &Path) -> bool {
    let mut probe = path.to_path_buf();
    loop {
        if probe.exists() {
            break;
        }
        let Some(parent) = probe.parent().map(Path::to_path_buf) else {
            return false;
        };
        if parent == probe {
            return false;
        }
        probe = parent;
    }
    match std::fs::metadata(&probe) {
        Ok(meta) => !meta.permissions().readonly(),
        Err(_) => false,
    }
}

/// Derive the update-check block from the update-notifier cache (kasetto `build_update_check`).
fn build_update_check(current_version: &str) -> AgentUpdateCheck {
    let Some(entry) = crate::update_notifier::read_cached_entry() else {
        return AgentUpdateCheck {
            status: "unknown".to_string(),
            latest_version: None,
            checked_at: None,
            age_seconds: None,
        };
    };
    let age = crate::update_notifier::now_unix_secs().saturating_sub(entry.checked_at);
    let status = if crate::self_update::is_newer(current_version, &entry.latest_version) {
        "update_available"
    } else {
        "up_to_date"
    };
    AgentUpdateCheck {
        status: status.to_string(),
        latest_version: Some(entry.latest_version),
        checked_at: Some(entry.checked_at),
        age_seconds: Some(age),
    }
}

/// Human-readable relative age (kasetto `format_age`). Lives in the engine so the CLI render is a
/// thin caller; the boundaries are the parity-pinned vectors.
pub fn format_age(secs: u64) -> String {
    if secs < 60 {
        format!("{secs}s ago")
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86_400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86_400)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_writable_walks_up_to_existing_ancestor() {
        // A deep nonexistent path under a writable existing tmp dir resolves to writable.
        let base =
            std::env::temp_dir().join(format!("envctl-doctor-writable-{}", std::process::id()));
        std::fs::create_dir_all(&base).unwrap();
        let deep = base.join("a/b/c/does/not/exist");
        // The deep nonexistent path walks up to the (writable) existing `base`.
        assert!(is_writable(&deep));

        // Make `base` read-only and re-probe: the walk-up now lands on a read-only ancestor.
        // (Skipped when running as root, where the write bit is advisory and `.readonly()`
        // reports false regardless — the same faithful-port behavior as kasetto.)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&base).unwrap().permissions();
            perms.set_mode(0o555);
            std::fs::set_permissions(&base, perms).unwrap();
            if !rustix::process::geteuid().is_root() {
                assert!(
                    !is_writable(&base),
                    "a read-only ancestor must report not-writable"
                );
            }
        }
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn format_age_boundaries() {
        // kasetto golden boundaries: s / m / h / d.
        assert_eq!(format_age(0), "0s ago");
        assert_eq!(format_age(59), "59s ago");
        assert_eq!(format_age(60), "1m ago");
        assert_eq!(format_age(3599), "59m ago");
        assert_eq!(format_age(3600), "1h ago");
        assert_eq!(format_age(86_399), "23h ago");
        assert_eq!(format_age(86_400), "1d ago");
    }

    #[test]
    fn doctor_report_json_round_trips_full_field_set() {
        // Parity field-set vector: the report serializes with kasetto's DoctorOutput field names
        // and round-trips through serde unchanged (the GUI receives it over the event channel,
        // so it must Deserialize too).
        use crate::agent::report::{AgentCommandDirCheck, AgentDoctorReport, AgentUpdateCheck};
        use envctl_agent_env::report::SyncFailure;
        let report = AgentDoctorReport {
            version: "1.2.3".into(),
            lock_file: "/x/agent-env.lock".into(),
            scope: "global".into(),
            skills: vec!["edit".into(), "find".into()],
            installation_path: "/home/u/.claude/skills".into(),
            last_sync: Some("1700000000".into()),
            failures: vec![SyncFailure {
                name: "broken".into(),
                source: "https://example.com/x".into(),
                reason: "missing".into(),
            }],
            mcps: vec!["github".into()],
            commands: vec!["review".into()],
            command_dirs: vec![AgentCommandDirCheck {
                path: "/home/u/.claude/commands".into(),
                writable: true,
            }],
            update_check: AgentUpdateCheck {
                status: "update_available".into(),
                latest_version: Some("2.0.0".into()),
                checked_at: Some(1_700_000_000),
                age_seconds: Some(42),
            },
        };
        let json = serde_json::to_string(&report).unwrap();
        // The kasetto field names are all present.
        for key in [
            "version",
            "lock_file",
            "scope",
            "skills",
            "installation_path",
            "last_sync",
            "failures",
            "mcps",
            "commands",
            "command_dirs",
            "update_check",
        ] {
            assert!(json.contains(key), "missing field `{key}` in {json}");
        }
        let back: AgentDoctorReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back.version, "1.2.3");
        assert_eq!(back.skills, vec!["edit".to_string(), "find".to_string()]);
        assert_eq!(back.command_dirs.len(), 1);
        assert!(back.command_dirs[0].writable);
        assert_eq!(back.update_check.status, "update_available");
        assert_eq!(back.failures[0].reason, "missing");
    }

    #[test]
    fn doctor_runs_config_less() {
        // No `agent-env.yaml` anywhere → `agent doctor` must still return Ok with an empty
        // (nothing-installed) report and the all-targets command-dir fallback, exactly like
        // kasetto and like `envctl agent list`. This is the no-downgrade regression guard.
        use crate::event::EventSink;
        use crate::Engine;

        // Mutates process-global HOME/XDG + cwd — serialize against every other env-touching
        // test (incl. agent::init's global-path reader) so parallel `cargo test` can't observe
        // a half-applied env. Held for the whole test; env restored to its prior values at the end.
        let _env = crate::test_env_lock();
        let prev_home = std::env::var_os("HOME");
        let prev_xdg_config = std::env::var_os("XDG_CONFIG_HOME");
        let prev_xdg_data = std::env::var_os("XDG_DATA_HOME");

        // Isolate HOME/XDG + cwd so no real config or lock is in scope; point everything at a
        // throwaway tmp tree with NO agent-env.yaml.
        let base = std::env::temp_dir().join(format!("envctl-doctor-cl-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let home = base.join("home");
        let xdg_config = base.join("config");
        let xdg_data = base.join("data");
        let cwd = base.join("cwd");
        for d in [&home, &xdg_config, &xdg_data, &cwd] {
            std::fs::create_dir_all(d).unwrap();
        }
        let prev_cwd = std::env::current_dir().ok();
        std::env::set_var("HOME", &home);
        std::env::set_var("XDG_CONFIG_HOME", &xdg_config);
        std::env::set_var("XDG_DATA_HOME", &xdg_data);
        std::env::set_current_dir(&cwd).unwrap();

        let engine = Engine::detached();
        let sink = EventSink::null();
        // Default scope (Global): MUST NOT error on a missing config.
        let report = engine
            .agent_doctor(AgentDoctorSpec::default(), &sink)
            .expect("config-less doctor must return Ok");

        // Nothing installed: skills/mcps/commands empty, install path "none".
        assert!(report.skills.is_empty(), "no config → no installed skills");
        assert!(report.mcps.is_empty(), "no config → no managed mcps");
        assert!(
            report.commands.is_empty(),
            "no config → no managed commands"
        );
        assert_eq!(report.installation_path, "none");
        assert_eq!(report.scope, "global");
        // The command-dir check falls back to the all-targets debugging view (non-empty: every
        // supported agent's global command dir), NOT an empty/errored set.
        assert!(
            !report.command_dirs.is_empty(),
            "config-less doctor must use the all-targets command-dir fallback"
        );

        // Restore environment to its prior state for the rest of the test process (never leave
        // HOME/XDG unset — agent::init's reader asserts an env-derived path).
        if let Some(p) = prev_cwd {
            let _ = std::env::set_current_dir(p);
        }
        restore_var("HOME", prev_home);
        restore_var("XDG_CONFIG_HOME", prev_xdg_config);
        restore_var("XDG_DATA_HOME", prev_xdg_data);
        let _ = std::fs::remove_dir_all(&base);
    }

    /// Restore an env var to a saved value (or remove it if it was unset).
    #[cfg(test)]
    fn restore_var(key: &str, prev: Option<std::ffi::OsString>) {
        match prev {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }

    #[test]
    fn build_update_check_unknown_without_cache() {
        // Mutates the process-global ENVCTL_CACHE_DIR the notifier reads — serialize against the
        // notifier's own env-poking tests via the shared lock; restore the prior value after.
        let _env = crate::test_env_lock();
        let prev = std::env::var_os("ENVCTL_CACHE_DIR");
        // Point the cache at an empty dir → no entry → "unknown".
        let dir = std::env::temp_dir().join(format!("envctl-doctor-uc-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // Use the same env override the notifier reads.
        std::env::set_var("ENVCTL_CACHE_DIR", &dir);
        let uc = build_update_check("1.0.0");
        assert_eq!(uc.status, "unknown");
        assert!(uc.latest_version.is_none());
        restore_var("ENVCTL_CACHE_DIR", prev);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
