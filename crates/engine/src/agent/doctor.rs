//! `Engine::agent_doctor` (TASK-0019, Item 1) — read-only local diagnostics for the agent-env
//! installation, ported from kasetto v3.2.0 `src/commands/doctor.rs`.
//!
//! Engine-first: this assembles the typed [`AgentDoctorReport`] (version, lock file, scope,
//! skills, install path, last sync, failures, MCP/command inventory, command-directory
//! writability checks, and the update-check block) and emits ONE `Event::AgentDoctored`. It is
//! non-printing and never writes. Both front-ends render the identical report — the CLI as the
//! grouped human view (or `--json`), the GUI as a Doctor sub-tab.

use std::path::Path;

use envctl_agent_env::{
    all_command_global_targets, all_command_project_targets, command_global_targets,
    command_project_targets, dirs_home, lock, resolve_dest, Agent, Scope,
};

use crate::agent::report::{AgentCommandDirCheck, AgentDoctorReport, AgentUpdateCheck};
use crate::agent::{AgentCtx, AgentScope};
use crate::event::{Event, EventSink};
use crate::Engine;

/// Options for `Engine::agent_doctor`.
#[derive(Clone, Debug, Default)]
pub struct AgentDoctorSpec {
    /// `--scope` override; `None` → resolved from the config.
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
        let ctx = AgentCtx::resolve(None, spec.scope_override)?;
        let scope = ctx.scope;
        let lock_file = &ctx.lock_file;
        let lock = lock::load(lock_file)?;
        let runtime =
            envctl_agent_env::runtime::load_runtime_state(scope, &ctx.cfg_dir).unwrap_or_default();

        let version = env!("CARGO_PKG_VERSION").to_string();
        let state = lock.state();
        let root = &ctx.scope_root;

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
        let command_dirs = collect_command_dirs(scope, &ctx.cfg, &ctx.cfg_dir);

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
/// every supported agent when no config agents are configured (kasetto verbatim semantics).
fn collect_command_dirs(
    scope: Scope,
    cfg: &envctl_agent_env::Config,
    project_root: &Path,
) -> Vec<AgentCommandDirCheck> {
    let agents: Vec<Agent> = cfg.agents();
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
    fn build_update_check_unknown_without_cache() {
        // Point the cache at an empty dir → no entry → "unknown".
        let dir = std::env::temp_dir().join(format!("envctl-doctor-uc-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // Use the same env override the notifier reads.
        std::env::set_var("ENVCTL_CACHE_DIR", &dir);
        let uc = build_update_check("1.0.0");
        assert_eq!(uc.status, "unknown");
        assert!(uc.latest_version.is_none());
        std::env::remove_var("ENVCTL_CACHE_DIR");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
