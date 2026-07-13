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
use envctl_agent_env::driver::audit_lock_zero_network;
use envctl_agent_env::extend::load_config_any_zero_network;
use envctl_agent_env::{
    all_command_global_targets, all_command_project_targets, command_global_targets,
    command_project_targets, dirs_home, lock, Agent, Scope,
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
        // Scope resolution is config-optional, but health is proof- and presence-sensitive.
        let scope: Scope = spec.scope_override.map(Into::into).unwrap_or(Scope::Global);
        let project_root = std::env::current_dir().unwrap_or_default();
        let lock_file = agent_lock_path(scope, &project_root)?;
        let mut proof_issues = Vec::new();
        let lock_present = match std::fs::symlink_metadata(&lock_file) {
            Ok(_) => true,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
            Err(error) => {
                proof_issues.push(format!("lock presence cannot be inspected: {error}"));
                false
            }
        };
        let (lock, lock_readable) = match lock::load(&lock_file) {
            Ok(lock) if lock_present => (lock, true),
            Ok(lock) => {
                proof_issues.push(format!("lock file is missing: {}", lock_file.display()));
                (lock, false)
            }
            Err(error) => {
                proof_issues.push(format!("lock is unreadable or unsafe: {error}"));
                (envctl_agent_env::AgentLockFile::default(), false)
            }
        };
        if lock_readable && lock.version != envctl_agent_env::lock::LOCK_VERSION {
            proof_issues.push(format!(
                "lock schema v{} is stale; v{} ownership proofs are required",
                lock.version,
                envctl_agent_env::lock::LOCK_VERSION
            ));
        }
        if lock_readable {
            audit_config_to_lock(scope, &lock, &mut proof_issues);
        }
        let (runtime, runtime_readable) =
            match envctl_agent_env::runtime::load_runtime_state(scope, &project_root) {
                Ok(runtime) => (runtime, true),
                Err(error) => {
                    proof_issues.push(format!("runtime ownership state is unreadable: {error}"));
                    (envctl_agent_env::runtime::RuntimeState::default(), false)
                }
            };
        if runtime
            .latest_report
            .as_deref()
            .is_some_and(|report| serde_json::from_str::<serde_json::Value>(report).is_err())
        {
            proof_issues.push("runtime latest report is malformed".to_string());
        }

        let version = env!("CARGO_PKG_VERSION").to_string();
        let inventory = if lock_readable {
            match envctl_agent_env::inspect_installed_inventory(
                &lock,
                &runtime,
                scope,
                &project_root,
                false,
            ) {
                Ok(inventory) => inventory,
                Err(error) => {
                    proof_issues.push(format!("ownership inventory is invalid: {error}"));
                    envctl_agent_env::InstalledInventory::default()
                }
            }
        } else {
            envctl_agent_env::InstalledInventory::default()
        };
        proof_issues.extend(inventory.issues.clone());
        proof_issues.sort();
        proof_issues.dedup();

        let installation_path = match inventory.install_paths.as_slice() {
            [] => "none".to_string(),
            [path] => path.clone(),
            paths => paths.join(", "),
        };
        let install_paths_writable = inventory
            .install_paths
            .iter()
            .all(|path| is_writable(Path::new(path)));
        let mut skills = inventory
            .skills
            .iter()
            .map(|skill| skill.skill.clone())
            .collect::<Vec<_>>();
        skills.sort();
        skills.dedup();
        let mut managed_mcps = inventory
            .mcps
            .iter()
            .map(|asset| asset.name.clone())
            .collect::<Vec<_>>();
        managed_mcps.sort();
        managed_mcps.dedup();
        let mut managed_commands = inventory
            .commands
            .iter()
            .map(|asset| asset.name.clone())
            .collect::<Vec<_>>();
        managed_commands.sort();
        managed_commands.dedup();

        let failures = runtime.load_latest_failures();
        let last_sync = runtime.last_run.clone();
        let command_dirs = collect_command_dirs(scope, &project_root);
        let command_dirs_writable = command_dirs.iter().all(|check| check.writable);
        let healthy = lock_present
            && lock_readable
            && runtime_readable
            && install_paths_writable
            && command_dirs_writable
            && failures.is_empty()
            && proof_issues.is_empty();

        let scope_label = match scope {
            Scope::Global => "global".to_string(),
            Scope::Project => "project".to_string(),
        };
        let update_check = build_update_check(&version);

        let report = AgentDoctorReport {
            version,
            lock_file: lock_file.to_string_lossy().to_string(),
            lock_present,
            lock_readable,
            runtime_readable,
            install_paths_writable,
            healthy,
            scope: scope_label,
            skills,
            installation_path,
            last_sync,
            failures,
            mcps: managed_mcps,
            commands: managed_commands,
            command_dirs,
            proof_issues,
            update_check,
        };

        sink.emit(Event::AgentDoctored {
            report: report.clone(),
        });
        Ok(report)
    }
}

/// Compare a present config's declared state with the lock without resolving any remote input.
/// A genuinely config-less doctor keeps kasetto's inventory-only behavior; a selected remote
/// config or remote `extends` is present but unauditable offline and therefore fails closed.
fn audit_config_to_lock(
    scope: Scope,
    current: &envctl_agent_env::AgentLockFile,
    proof_issues: &mut Vec<String>,
) {
    let config_path = default_config_path();
    let config_is_remote =
        config_path.starts_with("http://") || config_path.starts_with("https://");
    if !config_is_remote {
        match std::fs::symlink_metadata(&config_path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
            _ => {}
        }
        // A local overlay's explicit scalar `scope` wins over every extended parent. Inspect
        // only that scalar before resolving `extends`: an unrelated project config must not make
        // the default global doctor fail merely because its project-only parent is remote. If the
        // root scope is absent, invalid, or unreadable we cannot prove the config is unrelated and
        // retain the fail-closed full zero-network load below.
        match local_root_declared_scope(&config_path) {
            Ok(Some(config_scope)) if config_scope != scope => return,
            Ok(_) => {}
            Err(error) => {
                proof_issues.push(format!("config-to-lock audit failed: {error}"));
                return;
            }
        }
    }

    let (cfg, cfg_dir, _) = match load_config_any_zero_network(&config_path) {
        Ok(loaded) => loaded,
        Err(error) => {
            proof_issues.push(format!("config-to-lock audit failed: {error}"));
            return;
        }
    };
    // `doctor` defaults to the global installation even when invoked from a project that has
    // its own project-scoped config.  That local config describes a different lock authority;
    // comparing it with the global lock would manufacture drift in an otherwise unrelated
    // global installation.  An explicit project doctor still selects this config because the
    // two resolved scopes match.
    if cfg.resolved_scope() != scope {
        return;
    }
    let expected = match audit_lock_zero_network(&cfg, &cfg_dir, scope, current) {
        Ok(expected) => expected,
        Err(error) => {
            proof_issues.push(format!("config-to-lock audit failed: {error}"));
            return;
        }
    };
    proof_issues.extend(current.lock_check(&expected).into_iter().map(|drift| {
        format!(
            "config-to-lock drift: {} {}",
            drift.status.label(),
            drift.id
        )
    }));
}

/// Read only an explicit scalar `scope` from a local root config, without resolving `extends`.
/// `None` means the effective scope still depends on the fully merged config.
fn local_root_declared_scope(config_path: &str) -> Result<Option<Scope>, String> {
    let path = Path::new(config_path);
    let bytes = envctl_agent_env::read_config_optional(path)
        .map_err(|error| format!("cannot inspect local config scope: {error}"))?
        .ok_or_else(|| {
            format!(
                "config disappeared while inspecting scope: {}",
                path.display()
            )
        })?;
    let value: serde_yaml::Value = serde_yaml::from_slice(&bytes)
        .map_err(|error| format!("failed to parse config {}: {error}", path.display()))?;
    let Some(mapping) = value.as_mapping() else {
        return Ok(None);
    };
    let Some(value) = mapping.get("scope") else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    serde_yaml::from_value::<Scope>(value.clone())
        .map(Some)
        .map_err(|error| format!("invalid config scope in {}: {error}", path.display()))
}

/// Scope the COMMAND DIRECTORIES check to the agents the config actually wires; fall back to
/// every supported agent when there is NO config or no agents configured (kasetto verbatim
/// semantics — "what does envctl know how to write to?"). Config-OPTIONAL: any load error or an
/// empty agent set takes the all-targets debugging view rather than erroring.
fn collect_command_dirs(scope: Scope, project_root: &Path) -> Vec<AgentCommandDirCheck> {
    let agents: Vec<Agent> = match load_config_any_zero_network(&default_config_path()) {
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

/// Walk up to the first existing ancestor without following symlinks, require a safe authority
/// chain, then evaluate the effective user's write+search permission bits for that directory.
fn is_writable(path: &Path) -> bool {
    if !envctl_agent_env::managed_path_authority_is_safe(path) {
        return false;
    }
    let mut probe = path.to_path_buf();
    let metadata = loop {
        match std::fs::symlink_metadata(&probe) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    return false;
                }
                break metadata;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let Some(parent) = probe.parent().map(Path::to_path_buf) else {
                    return false;
                };
                if parent == probe {
                    return false;
                }
                probe = parent;
            }
            Err(_) => return false,
        }
    };

    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};

        let uid = rustix::process::geteuid().as_raw();
        if uid == 0 {
            return true;
        }
        let gid = rustix::process::getegid().as_raw();
        let supplementary_groups = std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|status| {
                status.lines().find_map(|line| {
                    line.strip_prefix("Groups:").map(|groups| {
                        groups
                            .split_whitespace()
                            .filter_map(|group| group.parse::<u32>().ok())
                            .collect::<Vec<_>>()
                    })
                })
            })
            .unwrap_or_default();
        let mode = metadata.permissions().mode();
        let required = if metadata.uid() == uid {
            0o300
        } else if metadata.gid() == gid || supplementary_groups.contains(&metadata.gid()) {
            0o030
        } else {
            0o003
        };
        mode & required == required
    }

    #[cfg(not(unix))]
    {
        !metadata.permissions().readonly()
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

    #[cfg(unix)]
    #[test]
    fn is_writable_rejects_symlink_and_cross_user_writable_authority() {
        use std::os::unix::fs::{symlink, PermissionsExt};

        let base = std::env::temp_dir().join(format!(
            "envctl-doctor-writable-authority-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        let target = base.join("target");
        std::fs::create_dir_all(&target).unwrap();
        symlink(&target, base.join("link")).unwrap();
        assert!(!is_writable(&base.join("link/child")));

        let mut permissions = std::fs::metadata(&base).unwrap().permissions();
        permissions.set_mode(0o777);
        std::fs::set_permissions(&base, permissions).unwrap();
        assert!(
            !is_writable(&base.join("new/child")),
            "an other-writable authority must fail closed even when it is writable"
        );

        let mut permissions = std::fs::metadata(&base).unwrap().permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(&base, permissions).unwrap();
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
            lock_present: true,
            lock_readable: true,
            runtime_readable: true,
            install_paths_writable: true,
            healthy: false,
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
            proof_issues: vec!["proof drift".into()],
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
            "lock_present",
            "lock_readable",
            "runtime_readable",
            "install_paths_writable",
            "healthy",
            "scope",
            "skills",
            "installation_path",
            "last_sync",
            "failures",
            "mcps",
            "commands",
            "command_dirs",
            "proof_issues",
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
        assert!(!back.healthy);
        assert_eq!(back.proof_issues, vec!["proof drift".to_string()]);
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
        assert!(!report.lock_present);
        assert!(!report.lock_readable);
        assert!(!report.healthy);
        assert!(
            report
                .proof_issues
                .iter()
                .any(|issue| issue.contains("lock file is missing")),
            "missing authority must be a typed health issue: {:?}",
            report.proof_issues
        );
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
