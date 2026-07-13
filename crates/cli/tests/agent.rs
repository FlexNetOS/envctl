//! Integration tests for `envctl agent {sync,add,remove,lock,list,clean}`. Drives the
//! real `envctl` binary against a hermetic temp project (its own cwd, config, and
//! XDG dirs) and asserts: (1) every mutating verb's dry-run (no `--apply`) writes
//! NOTHING — config + `agent-env.lock` + destination dir are byte-identical before/after
//! (the fail-closed invariant); (2) the `--json` shape of `list` and `lock --check`;
//! (3) the exit-code contract (`list` ⇒ 0; the `--ref`/`--branch` conflict ⇒ engine bail
//! ⇒ nonzero).
//!
//! Hermetic: the binary loads a manifest dir at startup even for agent verbs, so each
//! test points `ENVCTL_MANIFEST_DIR` at an empty temp dir (the agent path never reads
//! the component registry). `XDG_DATA_HOME`/`XDG_CONFIG_HOME` redirect the global lock
//! + config off the real `~`, and the project root is the spawned process's cwd.
use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_envctl")
}

/// A hermetic temp project: a `project/` cwd holding `agent-env.yaml`, an empty
/// `manifest/` dir, and isolated XDG roots — all under one unique temp dir.
struct Fixture {
    root: PathBuf,
    project: PathBuf,
    manifest: PathBuf,
    xdg_data: PathBuf,
    xdg_config: PathBuf,
    xdg_cache: PathBuf,
    /// The config-declared destination dir (must NOT be created by a dry-run).
    dest: PathBuf,
}

impl Fixture {
    fn new() -> Fixture {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        // Disambiguate concurrent tests within the same process by counter too.
        use std::sync::atomic::{AtomicU64, Ordering};
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let seq = SEQ.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("envctl-agent-it-{nanos}-{seq}"));
        let project = root.join("project");
        let manifest = root.join("manifest");
        let xdg_data = root.join("xdg-data");
        let xdg_config = root.join("xdg-config");
        let xdg_cache = root.join("xdg-cache");
        let dest = project.join("dest");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::create_dir_all(&manifest).unwrap();
        std::fs::write(
            project.join("agent-env.yaml"),
            format!(
                "destination: {}\nscope: project\nskills: []\nmcps: []\ncommands: []\n",
                dest.display()
            ),
        )
        .unwrap();
        Fixture {
            root,
            project,
            manifest,
            xdg_data,
            xdg_config,
            xdg_cache,
            dest,
        }
    }

    /// A command rooted in the project cwd with the hermetic env applied.
    fn cmd(&self) -> Command {
        let mut c = Command::new(bin());
        c.current_dir(&self.project)
            .env("ENVCTL_MANIFEST_DIR", &self.manifest)
            .env("XDG_DATA_HOME", &self.xdg_data)
            .env("XDG_CONFIG_HOME", &self.xdg_config)
            .env("XDG_CACHE_HOME", &self.xdg_cache);
        c
    }

    fn config_path(&self) -> PathBuf {
        self.project.join("agent-env.yaml")
    }

    fn lock_path(&self) -> PathBuf {
        self.project.join("agent-env.lock")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).ok();
    }
}

/// A snapshot of the on-disk state that a dry-run must not mutate.
fn snapshot(fx: &Fixture) -> (Option<String>, bool, bool) {
    let config = std::fs::read_to_string(fx.config_path()).ok();
    let lock_exists = fx.lock_path().exists();
    let dest_exists = fx.dest.exists();
    (config, lock_exists, dest_exists)
}

fn assert_unchanged(fx: &Fixture, before: &(Option<String>, bool, bool), verb: &str) {
    let after = snapshot(fx);
    assert_eq!(before.0, after.0, "{verb} dry-run mutated the config");
    assert_eq!(
        before.1, after.1,
        "{verb} dry-run created/removed agent-env.lock"
    );
    assert_eq!(
        before.2, after.2,
        "{verb} dry-run created/removed the destination dir"
    );
    // Belt-and-suspenders: the dry-run must never materialize the destination.
    assert!(
        !fx.dest.exists(),
        "{verb} dry-run created the destination dir {}",
        fx.dest.display()
    );
    assert!(
        !fx.xdg_cache.exists(),
        "{verb} dry-run created cosmetic runtime/cache state"
    );
}

// --------------------------------------------------------------------------------------
// Per-verb dry-run = zero writes (the fail-closed invariant).
// --------------------------------------------------------------------------------------

#[test]
fn sync_dry_run_writes_nothing() {
    let fx = Fixture::new();
    let before = snapshot(&fx);
    let out = fx.cmd().args(["agent", "sync"]).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_unchanged(&fx, &before, "sync");
}

#[test]
fn add_dry_run_writes_nothing() {
    let fx = Fixture::new();
    let before = snapshot(&fx);
    // `add` with a local path but NO --apply: preview only (records "would_add").
    let out = fx
        .cmd()
        .args(["agent", "add", "./some-source", "--no-sync"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_unchanged(&fx, &before, "add");
}

#[test]
fn clean_dry_run_writes_nothing() {
    let fx = Fixture::new();
    let before = snapshot(&fx);
    let out = fx.cmd().args(["agent", "clean"]).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_unchanged(&fx, &before, "clean");
}

// --------------------------------------------------------------------------------------
// `--json` shape.
// --------------------------------------------------------------------------------------

#[test]
fn list_json_has_agent_list_shape() {
    let fx = Fixture::new();
    let out = fx.cmd().args(["agent", "list", "--json"]).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    // AgentList: { skills: [], mcps: [], commands: [], merged_scopes: bool }
    assert!(v["skills"].is_array(), "json: {v}");
    assert!(v["mcps"].is_array(), "json: {v}");
    assert!(v["commands"].is_array(), "json: {v}");
    assert!(v["merged_scopes"].is_boolean(), "json: {v}");
    // No --scope override -> the two scopes are merged.
    assert_eq!(v["merged_scopes"], true);
}

#[test]
fn lock_check_json_has_outcome_shape() {
    let fx = Fixture::new();
    let out = fx
        .cmd()
        .args(["agent", "lock", "--check", "--json"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    // AgentLockOutcome: { check, saved, skills, sources, drift: [] }
    assert_eq!(v["check"], true);
    assert_eq!(v["saved"], false);
    assert!(v["drift"].is_array(), "json: {v}");
    // `--check` must not write the lock.
    assert!(
        !fx.lock_path().exists(),
        "lock --check wrote agent-env.lock"
    );
}

#[test]
fn lock_check_locked_json_reports_local_drift_and_exits_nonzero_without_writing() {
    let fx = Fixture::new();
    let source = fx.project.join("skills-source");
    std::fs::create_dir_all(source.join("alpha")).unwrap();
    std::fs::write(
        source.join("alpha/SKILL.md"),
        "---\nname: alpha\n---\noriginal\n",
    )
    .unwrap();
    std::fs::write(
        fx.config_path(),
        format!(
            "destination: {}\nscope: project\nskills:\n  - source: {}\n    skills:\n      - alpha\n",
            fx.dest.display(),
            source.display()
        ),
    )
    .unwrap();

    let seeded = fx.cmd().args(["agent", "lock"]).output().unwrap();
    assert!(
        seeded.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&seeded.stderr)
    );
    let lock_before = std::fs::read_to_string(fx.lock_path()).unwrap();
    std::fs::write(
        source.join("alpha/SKILL.md"),
        "---\nname: alpha\n---\nchanged\n",
    )
    .unwrap();

    let checked = fx
        .cmd()
        .args(["agent", "lock", "--check", "--locked", "--json"])
        .output()
        .unwrap();
    assert_eq!(checked.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&checked.stdout).unwrap();
    assert_eq!(value["check"], true);
    assert_eq!(value["saved"], false);
    assert!(value["drift"].as_array().is_some_and(|drift| {
        drift.iter().any(|item| {
            item["status"] == "updated"
                && item["id"].as_str().is_some_and(|id| id.contains("alpha"))
        })
    }));
    assert_eq!(
        std::fs::read_to_string(fx.lock_path()).unwrap(),
        lock_before
    );
}

#[test]
fn sync_locked_installs_validated_local_skills_commands_and_mcps() {
    let fx = Fixture::new();
    let source = fx.project.join("local-assets");
    std::fs::create_dir_all(source.join("alpha")).unwrap();
    std::fs::create_dir_all(source.join("commands")).unwrap();
    std::fs::create_dir_all(source.join("mcps")).unwrap();
    std::fs::write(
        source.join("alpha/SKILL.md"),
        "---\nname: alpha\n---\n# Alpha\n",
    )
    .unwrap();
    std::fs::write(source.join("commands/review.md"), "Review $ARGUMENTS\n").unwrap();
    std::fs::write(
        source.join("mcps/servers.json"),
        r#"{"mcpServers":{"local-server":{"command":"local-server"}}}"#,
    )
    .unwrap();
    std::fs::write(
        fx.config_path(),
        format!(
            "agent: claude-code\nscope: project\nskills:\n  - source: {source}\n    skills:\n      - alpha\ncommands:\n  - source: {source}\n    commands:\n      - review\nmcps:\n  - source: {source}\n    mcps:\n      - servers\n",
            source = source.display()
        ),
    )
    .unwrap();

    let locked = fx.cmd().args(["agent", "lock"]).output().unwrap();
    assert!(
        locked.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&locked.stderr)
    );
    assert!(!fx.project.join(".claude").exists());
    assert!(!fx.project.join(".mcp.json").exists());

    let unproven = fx
        .cmd()
        .args(["agent", "sync", "--locked", "--apply", "--json"])
        .output()
        .unwrap();
    assert_eq!(unproven.status.code(), Some(1));
    assert!(!fx.project.join(".claude").exists());

    let bootstrap = fx
        .cmd()
        .args(["agent", "sync", "--apply", "--json"])
        .output()
        .unwrap();
    assert!(
        bootstrap.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&bootstrap.stdout),
        String::from_utf8_lossy(&bootstrap.stderr)
    );
    let lock_before = std::fs::read(fx.lock_path()).unwrap();
    std::fs::remove_dir_all(fx.project.join(".claude")).unwrap();
    std::fs::remove_file(fx.project.join(".mcp.json")).unwrap();

    let synced = fx
        .cmd()
        .args(["agent", "sync", "--locked", "--apply", "--json"])
        .output()
        .unwrap();
    assert!(
        synced.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&synced.stdout),
        String::from_utf8_lossy(&synced.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&synced.stdout).unwrap();
    assert_eq!(report["summary"]["failed"], 0);
    assert!(fx.project.join(".claude/skills/alpha/SKILL.md").is_file());
    assert!(fx.project.join(".claude/commands/review.md").is_file());
    let mcp: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(fx.project.join(".mcp.json")).unwrap())
            .unwrap();
    assert!(mcp["mcpServers"]["local-server"].is_object());
    assert_eq!(
        std::fs::read(fx.lock_path()).unwrap(),
        lock_before,
        "--locked sync must not rewrite the lock"
    );
}

// --------------------------------------------------------------------------------------
// Exit-code contract.
// --------------------------------------------------------------------------------------

#[test]
fn list_exits_zero() {
    let fx = Fixture::new();
    let out = fx.cmd().args(["agent", "list"]).output().unwrap();
    assert!(out.status.success(), "agent list must exit 0");
}

#[test]
fn doctor_missing_project_lock_reports_unhealthy_and_exits_nonzero() {
    let fx = Fixture::new();
    let out = fx
        .cmd()
        .args(["agent", "doctor", "--scope", "project", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["lock_present"], false);
    assert_eq!(report["lock_readable"], false);
    assert_eq!(report["healthy"], false);
    assert!(report["proof_issues"].as_array().is_some_and(|issues| {
        issues
            .iter()
            .any(|issue| issue.as_str().is_some_and(|text| text.contains("missing")))
    }));
}

#[test]
fn doctor_rejects_config_to_empty_lock_drift_without_writes() {
    let fx = Fixture::new();
    let source = fx.project.join("doctor-empty-lock-source");
    std::fs::create_dir_all(source.join("alpha")).unwrap();
    std::fs::write(source.join("alpha/SKILL.md"), "# declared but unlocked\n").unwrap();
    std::fs::write(
        fx.config_path(),
        format!(
            "scope: project\nagent: claude-code\nskills:\n  - source: {}\n    skills: [alpha]\n",
            source.display()
        ),
    )
    .unwrap();
    std::fs::write(
        fx.lock_path(),
        format!(
            "version: {}\nskills: {{}}\nassets: {{}}\n",
            envctl_agent_env::LOCK_VERSION
        ),
    )
    .unwrap();

    let config_before = std::fs::read(fx.config_path()).unwrap();
    let lock_before = std::fs::read(fx.lock_path()).unwrap();
    let source_before = std::fs::read(source.join("alpha/SKILL.md")).unwrap();
    let out = fx
        .cmd()
        // Point the cosmetic notifier directly at this sentinel. Doctor must not create it:
        // its read-only/machine contract forbids starting the notifier in the first place.
        .env("ENVCTL_CACHE_DIR", &fx.xdg_cache)
        .args(["agent", "doctor", "--scope", "project", "--json"])
        .output()
        .unwrap();

    assert_eq!(
        out.status.code(),
        Some(1),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["lock_present"], true);
    assert_eq!(report["lock_readable"], true);
    assert_eq!(report["healthy"], false);
    assert!(report["proof_issues"].as_array().is_some_and(|issues| {
        issues.iter().any(|issue| {
            issue
                .as_str()
                .is_some_and(|text| text.contains("config-to-lock drift") && text.contains("alpha"))
        })
    }));
    assert_eq!(std::fs::read(fx.config_path()).unwrap(), config_before);
    assert_eq!(std::fs::read(fx.lock_path()).unwrap(), lock_before);
    assert_eq!(
        std::fs::read(source.join("alpha/SKILL.md")).unwrap(),
        source_before
    );
    assert!(
        !fx.dest.exists(),
        "doctor must not materialize destinations"
    );
    assert!(
        !fx.xdg_cache.exists(),
        "doctor must not create the update cache or other runtime/cache state"
    );
}

#[cfg(unix)]
#[test]
fn doctor_audits_only_a_config_owned_by_the_selected_scope() {
    use std::os::unix::fs::PermissionsExt;

    let fx = Fixture::new();
    let source = fx.project.join("doctor-scope-source");
    std::fs::create_dir_all(source.join("alpha")).unwrap();
    std::fs::write(
        source.join("alpha/SKILL.md"),
        "# project-only declaration\n",
    )
    .unwrap();
    std::fs::write(
        fx.config_path(),
        format!(
            "scope: project\nagent: claude-code\nskills:\n  - source: {}\n    skills: [alpha]\n",
            source.display()
        ),
    )
    .unwrap();

    // Make both selected lock authorities readable empty v3 locks. The project config must
    // drift against the project lock, but it is unrelated to the default global lock.
    let global_lock_dir = fx.xdg_data.join("agent-env");
    std::fs::create_dir_all(&global_lock_dir).unwrap();
    std::fs::set_permissions(&fx.xdg_data, std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::set_permissions(&global_lock_dir, std::fs::Permissions::from_mode(0o700)).unwrap();
    let empty_lock = format!(
        "version: {}\nskills: {{}}\nassets: {{}}\n",
        envctl_agent_env::LOCK_VERSION
    );
    let global_lock = global_lock_dir.join("agent-env.lock");
    std::fs::write(&global_lock, &empty_lock).unwrap();
    std::fs::set_permissions(&global_lock, std::fs::Permissions::from_mode(0o600)).unwrap();
    std::fs::write(fx.lock_path(), &empty_lock).unwrap();
    std::fs::set_permissions(fx.lock_path(), std::fs::Permissions::from_mode(0o600)).unwrap();

    let global = fx
        .cmd()
        .args(["agent", "doctor", "--json"])
        .output()
        .unwrap();
    let global_report: serde_json::Value = serde_json::from_slice(&global.stdout).unwrap();
    assert_eq!(global_report["scope"], "global");
    assert_eq!(global_report["lock_present"], true);
    assert_eq!(global_report["lock_readable"], true);
    assert!(global_report["proof_issues"]
        .as_array()
        .is_some_and(|issues| {
            issues.iter().all(|issue| {
                !issue
                    .as_str()
                    .is_some_and(|text| text.contains("config-to-lock"))
            })
        }));

    let project = fx
        .cmd()
        .args(["agent", "doctor", "--scope", "project", "--json"])
        .output()
        .unwrap();
    assert_eq!(project.status.code(), Some(1));
    let project_report: serde_json::Value = serde_json::from_slice(&project.stdout).unwrap();
    assert_eq!(project_report["scope"], "project");
    assert!(project_report["proof_issues"]
        .as_array()
        .is_some_and(|issues| {
            issues.iter().any(|issue| {
                issue.as_str().is_some_and(|text| {
                    text.contains("config-to-lock drift") && text.contains("alpha")
                })
            })
        }));

    // The root's explicit project scope wins over an extended parent. Default-global doctor must
    // classify it as unrelated before attempting to resolve the remote project-only parent;
    // explicit project doctor must retain the zero-network, fail-closed audit.
    std::fs::write(
        fx.config_path(),
        "extends: https://network-sentinel.invalid/base.yaml\nscope: project\n",
    )
    .unwrap();
    let global = fx
        .cmd()
        .args(["agent", "doctor", "--json"])
        .output()
        .unwrap();
    let global_report: serde_json::Value = serde_json::from_slice(&global.stdout).unwrap();
    assert_eq!(global_report["scope"], "global");
    assert!(global_report["proof_issues"]
        .as_array()
        .is_some_and(|issues| {
            issues.iter().all(|issue| {
                !issue
                    .as_str()
                    .is_some_and(|text| text.contains("config-to-lock"))
            })
        }));

    let project = fx
        .cmd()
        .args(["agent", "doctor", "--scope", "project", "--json"])
        .output()
        .unwrap();
    assert_eq!(project.status.code(), Some(1));
    let project_report: serde_json::Value = serde_json::from_slice(&project.stdout).unwrap();
    assert!(project_report["proof_issues"]
        .as_array()
        .is_some_and(|issues| {
            issues.iter().any(|issue| {
                issue.as_str().is_some_and(|text| {
                    text.contains("config-to-lock audit failed")
                        && text.contains("forbids remote config fetch")
                })
            })
        }));
}

#[test]
fn doctor_reports_healthy_after_proven_project_sync() {
    let fx = Fixture::new();
    let source = fx.project.join("doctor-skill-source");
    std::fs::create_dir_all(source.join("alpha")).unwrap();
    std::fs::write(
        source.join("alpha/SKILL.md"),
        "---\nname: alpha\ndescription: doctor proof\n---\n# Alpha\n",
    )
    .unwrap();
    std::fs::write(
        fx.config_path(),
        format!(
            "scope: project\nagent: claude-code\nskills:\n  - source: {}\n    skills: [alpha]\n",
            source.display()
        ),
    )
    .unwrap();
    let sync = fx
        .cmd()
        .args(["agent", "sync", "--apply", "--json"])
        .output()
        .unwrap();
    assert!(
        sync.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&sync.stdout),
        String::from_utf8_lossy(&sync.stderr)
    );

    let doctor = fx
        .cmd()
        .args(["agent", "doctor", "--scope", "project", "--json"])
        .output()
        .unwrap();
    assert!(
        doctor.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&doctor.stdout),
        String::from_utf8_lossy(&doctor.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&doctor.stdout).unwrap();
    assert_eq!(report["healthy"], true);
    assert_eq!(report["lock_present"], true);
    assert_eq!(report["lock_readable"], true);
    assert_eq!(report["proof_issues"], serde_json::json!([]));
    assert_eq!(report["skills"], serde_json::json!(["alpha"]));

    std::fs::write(
        fx.project.join(".claude/skills/alpha/SKILL.md"),
        "# drifted after install\n",
    )
    .unwrap();
    let drifted = fx
        .cmd()
        .args(["agent", "doctor", "--scope", "project", "--json"])
        .output()
        .unwrap();
    assert_eq!(drifted.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&drifted.stdout).unwrap();
    assert_eq!(report["healthy"], false);
    assert_eq!(report["skills"], serde_json::json!([]));
    assert!(report["proof_issues"].as_array().is_some_and(|issues| {
        issues.iter().any(|issue| {
            issue
                .as_str()
                .is_some_and(|text| text.contains("drifted from its ownership proof"))
        })
    }));
}

#[test]
fn add_ref_and_branch_conflict_exits_nonzero() {
    // The engine bail (`--ref and --branch are mutually exclusive`) must propagate
    // as a nonzero exit through the worker-join `?` path.
    let fx = Fixture::new();
    let out = fx
        .cmd()
        .args(["agent", "add", "src", "--ref", "a", "--branch", "b"])
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "conflicting --ref/--branch must exit nonzero; stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// `agent --help` lists all eight verbs (surface smoke).
#[test]
fn help_lists_the_eight_verbs() {
    let out = Command::new(bin())
        .args(["agent", "--help"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let help = String::from_utf8(out.stdout).unwrap();
    for verb in [
        "sync", "add", "remove", "lock", "list", "clean", "init", "doctor",
    ] {
        assert!(
            help.contains(verb),
            "agent --help missing `{verb}`:\n{help}"
        );
    }
}

#[test]
fn lock_help_describes_v3_ownership_bootstrap_instead_of_immediate_locked_install() {
    let out = Command::new(bin())
        .args(["agent", "lock", "--help"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let help = String::from_utf8(out.stdout).unwrap();
    assert!(
        help.contains("sync --apply"),
        "missing bootstrap step:\n{help}"
    );
    assert!(
        help.contains("portable proofs"),
        "missing clone contract:\n{help}"
    );
    assert!(
        help.contains("sync --locked --apply"),
        "missing clone command:\n{help}"
    );
    assert!(
        !help.contains("immediately usable"),
        "stale desired-equals-ownership contract:\n{help}"
    );
}

/// Sanity: the fixture's dest dir genuinely does not pre-exist, so the
/// `assert_unchanged` dest check is meaningful (not vacuously satisfied).
#[test]
fn fixture_dest_absent_until_apply() {
    let fx = Fixture::new();
    assert!(!fx.dest.exists());
    assert!(Path::new(&fx.config_path()).exists());
}

/// Regression (found by /verify): `agent list` in HUMAN mode must render the inventory,
/// not just a header. The inventory lives in the returned `AgentList` (the engine emits
/// no per-item events for `list`), so the non-`--json` path must print the returned rows.
/// Before the fix, `agent list` showed only `==> agent List …` and no installed assets.
#[test]
fn list_human_output_renders_installed_inventory() {
    let fx = Fixture::new();
    // A local skill pack with one skill.
    let pack = fx.root.join("pack");
    std::fs::create_dir_all(pack.join("alpha")).unwrap();
    std::fs::write(
        pack.join("alpha/SKILL.md"),
        "---\nname: alpha\ndescription: Alpha skill\n---\n# Alpha\n",
    )
    .unwrap();
    // Point the config at the pack with a claude-code preset (installs to <cwd>/.claude/skills).
    std::fs::write(
        fx.config_path(),
        format!(
            "scope: project\nagent: claude-code\nskills:\n  - source: {}\n    skills: [\"alpha\"]\n",
            pack.display()
        ),
    )
    .unwrap();
    // Install it.
    let sync = fx
        .cmd()
        .args(["agent", "sync", "--apply"])
        .output()
        .unwrap();
    assert!(
        sync.status.success(),
        "sync --apply failed: {}",
        String::from_utf8_lossy(&sync.stderr)
    );
    // HUMAN list MUST show the skill row + a count header — the bug was a header-only render.
    let out = fx.cmd().args(["agent", "list"]).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("skills (1)"),
        "human `list` missing the skills inventory header:\n{stdout}"
    );
    assert!(
        stdout.contains("alpha"),
        "human `list` did not render the installed skill:\n{stdout}"
    );
    // --json still carries the same row (no regression).
    let j = fx.cmd().args(["agent", "list", "--json"]).output().unwrap();
    let v: serde_json::Value = serde_json::from_slice(&j.stdout).unwrap();
    assert_eq!(v["skills"].as_array().unwrap().len(), 1);
}

/// Regression (found by /verify): an `add`/`remove` PREVIEW (no `--apply`) has no per-item
/// events, so the human view must still render the edit-outcome summary line — before the
/// fix it printed only a header.
#[test]
fn remove_preview_renders_outcome_summary() {
    let fx = Fixture::new();
    let pack = fx.root.join("pack");
    std::fs::create_dir_all(pack.join("alpha")).unwrap();
    std::fs::write(
        pack.join("alpha/SKILL.md"),
        "---\nname: alpha\n---\n# Alpha\n",
    )
    .unwrap();
    std::fs::write(
        fx.config_path(),
        format!(
            "scope: project\nagent: claude-code\nskills:\n  - source: {}\n    skills: [\"alpha\"]\n",
            pack.display()
        ),
    )
    .unwrap();
    let before = snapshot(&fx);
    let out = fx
        .cmd()
        .args(["agent", "remove", &pack.display().to_string()])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("would_remove"),
        "human remove preview missing the would_remove summary:\n{stdout}"
    );
    // Still fail-closed: a preview writes nothing.
    assert_unchanged(&fx, &before, "remove");
}

/// C-13 — `agent init` creates a commented starter config.
#[test]
fn init_creates_agent_env_yaml() {
    let fx = Fixture::new();
    // The standard fixture already has a config; remove it so `init` starts from scratch.
    std::fs::remove_file(fx.config_path()).unwrap();
    let out = fx.cmd().args(["agent", "init"]).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let written = std::fs::read_to_string(fx.config_path()).unwrap();
    assert!(written.contains("# envctl agent-env"));
    assert!(written.contains("skills:"));
}

/// C-13 — `agent init` refuses to overwrite without `--force`.
#[test]
fn init_refuses_overwrite_without_force() {
    let fx = Fixture::new();
    std::fs::remove_file(fx.config_path()).unwrap();
    assert!(fx
        .cmd()
        .args(["agent", "init"])
        .output()
        .unwrap()
        .status
        .success());
    let out = fx.cmd().args(["agent", "init"]).output().unwrap();
    assert!(
        !out.status.success(),
        "init must fail when config exists and --force is absent"
    );
    let out2 = fx
        .cmd()
        .args(["agent", "init", "--force"])
        .output()
        .unwrap();
    assert!(out2.status.success(), "init --force must succeed");
}
