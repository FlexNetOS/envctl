//! Integration tests for the engine agent-env subsystem (TASK-0013). All hermetic: local
//! `source:` paths (the committed `fixtures/agent/pack`), tempdir project roots, NO network.
//!
//! Isolation: a per-process temp `HOME` (with XDG_* unset → derived from HOME) keeps the
//! agent-env data/cache (the global lock + runtime memo) inside the test sandbox. Each test
//! uses a distinct project tempdir, so the per-project lock + runtime never collide. The two
//! cwd-dependent verbs (`list`, `clean`) serialize through `CWD_LOCK`.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use envctl_agent_env::{
    command_asset_id, mcp_asset_id, skill_key, AgentLockEntry, AgentLockFile, Scope,
};
use envctl_engine::event::{Event, EventSink};
use envctl_engine::{
    AgentCleanSpec, AgentListKind, AgentListSpec, AgentLockMode, AgentLockSpec, AgentRemoveSpec,
    AgentScope, AgentSectionSel, AgentSyncSpec, Engine,
};

// ---------------------------------------------------------------------------------------
// Sandbox helpers
// ---------------------------------------------------------------------------------------

/// One shared per-process temp HOME so the agent-env global data/cache dirs resolve inside the
/// sandbox. XDG_* are cleared so they derive from HOME.
fn sandbox_home() -> &'static Path {
    static HOME: OnceLock<PathBuf> = OnceLock::new();
    HOME.get_or_init(|| {
        let base = unique_dir("envctl-agent-it-home");
        std::env::set_var("HOME", &base);
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::remove_var("XDG_DATA_HOME");
        std::env::remove_var("XDG_CACHE_HOME");
        std::env::remove_var("ENVCTL_AGENT_CONFIG");
        base
    })
}

/// Serializes the cwd-dependent verbs (list/clean read `std::env::current_dir`).
fn cwd_lock() -> &'static Mutex<()> {
    static L: OnceLock<Mutex<()>> = OnceLock::new();
    L.get_or_init(|| Mutex::new(()))
}

fn unique_dir(prefix: &str) -> PathBuf {
    static N: OnceLock<Mutex<u64>> = OnceLock::new();
    let mut n = N.get_or_init(|| Mutex::new(0)).lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    *n += 1;
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let d = std::env::temp_dir().join(format!("{prefix}-{nanos}-{}", *n));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The committed local skill/MCP pack fixture.
fn pack_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/agent/pack")
}

/// The repo's live agent-skill/MCP pack declared by agent-env.yaml.
fn repo_agent_skills_dir() -> PathBuf {
    std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../agent-skills"))
        .expect("repo agent-skills fixture")
}

/// The committed local command pack fixture.
fn cmdpack_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/agent/cmdpack")
}

/// Build a project tempdir with an `agent-env.yaml` and return (engine, project_dir, cfg_path).
fn project_with_config(yaml: &str) -> (Engine, PathBuf, String) {
    sandbox_home();
    let project = unique_dir("envctl-agent-it-proj");
    let cfg_path = project.join("agent-env.yaml");
    std::fs::write(&cfg_path, yaml).unwrap();
    // The agent verbs are manifest-independent; a detached engine is enough.
    let engine = Engine::detached();
    (engine, project, cfg_path.to_string_lossy().to_string())
}

/// A claude-code, project-scope config whose skills+mcps come from the local fixture pack.
fn full_config(pack: &Path) -> String {
    format!(
        "agent: claude-code\nscope: project\nskills:\n  - source: {pack}\n    skills: \"*\"\nmcps:\n  - source: {pack}\n    mcps: \"*\"\n",
        pack = pack.display()
    )
}

/// Drain a sink's events into a Vec for assertions.
fn drain(rx: std::sync::mpsc::Receiver<Event>) -> Vec<Event> {
    let mut out = Vec::new();
    while let Ok(ev) = rx.try_recv() {
        out.push(ev);
    }
    out
}

fn sink() -> (EventSink, std::sync::mpsc::Receiver<Event>) {
    EventSink::channel()
}

fn local_asset_source() -> PathBuf {
    let source = unique_dir("envctl-agent-it-local-assets");
    copy_tree(&pack_dir(), &source);
    copy_tree(&cmdpack_dir(), &source);
    source
}

fn local_asset_config(source: &Path) -> String {
    format!(
        "agent: claude-code\nscope: project\nskills:\n  - source: {source}\n    skills:\n      - alpha\ncommands:\n  - source: {source}\n    commands:\n      - foo\nmcps:\n  - source: {source}\n    mcps:\n      - servers\n",
        source = source.display()
    )
}

fn write_agent_lock(engine: &Engine, config: &str) {
    let (s, _rx) = sink();
    let report = engine
        .agent_lock(
            AgentLockSpec {
                config_path: Some(config.to_string()),
                scope_override: None,
                check: false,
                upgrade_only: Vec::new(),
                lock_mode: AgentLockMode::Plain,
            },
            &s,
        )
        .expect("write local agent lock");
    assert!(report.saved);
    assert!(report.drift.is_empty());
}

// ---------------------------------------------------------------------------------------
// 1. sync preview (no writes, would_install) vs apply (installs + lock)
// ---------------------------------------------------------------------------------------

#[test]
fn sync_preview_writes_nothing_then_apply_installs() {
    let (engine, project, cfg) = project_with_config(&full_config(&pack_dir()));

    // Preview: zero writes.
    let (s, rx) = sink();
    let report = engine
        .agent_sync(
            AgentSyncSpec {
                config_path: Some(cfg.clone()),
                apply: false,
                ..Default::default()
            },
            &s,
        )
        .expect("preview sync");
    assert!(report.dry_run);
    assert!(report.summary.installed >= 2, "alpha+beta would install");
    assert!(report.summary.failed == 0);
    let events = drain(rx);
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::AgentRunStarted { .. })));
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::AgentRunFinished { .. })));
    // No skills dir, no lock on disk.
    assert!(
        !project.join(".claude/skills/alpha").exists(),
        "preview wrote nothing"
    );
    assert!(
        !project.join("agent-env.lock").exists(),
        "preview wrote no lock"
    );

    // Apply: installs + lock.
    let (s2, _rx2) = sink();
    let applied = engine
        .agent_sync(
            AgentSyncSpec {
                config_path: Some(cfg.clone()),
                apply: true,
                ..Default::default()
            },
            &s2,
        )
        .expect("apply sync");
    assert!(!applied.dry_run);
    assert_eq!(applied.summary.failed, 0);
    assert!(project.join(".claude/skills/alpha/SKILL.md").is_file());
    assert!(project.join(".claude/skills/beta/SKILL.md").is_file());
    assert!(
        project.join("agent-env.lock").is_file(),
        "apply wrote the lock"
    );
}

#[test]
fn future_lock_schema_refuses_plain_and_update_before_any_mutation() {
    for lock_mode in [
        AgentLockMode::Plain,
        AgentLockMode::Update { only: Vec::new() },
    ] {
        let source = unique_dir("envctl-agent-it-future-lock-source");
        std::fs::create_dir_all(source.join("alpha")).unwrap();
        std::fs::write(source.join("alpha/SKILL.md"), "# original\n").unwrap();
        let yaml = format!(
            "agent: claude-code\nscope: project\nskills:\n  - source: {source}\n    skills: [alpha]\n",
            source = source.display()
        );
        let (engine, project, cfg) = project_with_config(&yaml);

        let (seed_sink, _seed_rx) = sink();
        let seeded = engine
            .agent_sync(
                AgentSyncSpec {
                    config_path: Some(cfg.clone()),
                    apply: true,
                    ..Default::default()
                },
                &seed_sink,
            )
            .expect("seed current lock and ownership state");
        assert_eq!(seeded.summary.failed, 0, "{:#?}", seeded.actions);

        let lock_path = project.join("agent-env.lock");
        let output_path = project.join(".claude/skills/alpha/SKILL.md");
        let runtime_path =
            envctl_agent_env::runtime::runtime_state_path(Scope::Project, &project).unwrap();
        std::fs::write(source.join("alpha/SKILL.md"), "# replacement\n").unwrap();

        let current_lock = std::fs::read_to_string(&lock_path).unwrap();
        let future_lock = current_lock.replacen(
            &format!("version: {}", envctl_agent_env::LOCK_VERSION),
            &format!("version: {}", envctl_agent_env::LOCK_VERSION + 1),
            1,
        ) + "future_schema_field: preserve-verbatim\n";
        assert_ne!(future_lock, current_lock, "fixture must advance the schema");
        std::fs::write(&lock_path, future_lock.as_bytes()).unwrap();

        let lock_before = std::fs::read(&lock_path).unwrap();
        let output_before = std::fs::read(&output_path).unwrap();
        let runtime_before = std::fs::read(&runtime_path).unwrap();
        let (attempt_sink, _attempt_rx) = sink();
        let error = engine
            .agent_sync(
                AgentSyncSpec {
                    config_path: Some(cfg),
                    apply: true,
                    lock_mode,
                    ..Default::default()
                },
                &attempt_sink,
            )
            .expect_err("a future lock schema must be rejected, never migrated backward");

        let message = format!("{error:#}");
        assert!(message.contains("newer than supported"), "{message}");
        assert_eq!(
            std::fs::read(&lock_path).unwrap(),
            lock_before,
            "future lock bytes must remain untouched"
        );
        assert_eq!(
            std::fs::read(&output_path).unwrap(),
            output_before,
            "installed output must remain untouched"
        );
        assert_eq!(
            std::fs::read(&runtime_path).unwrap(),
            runtime_before,
            "runtime ownership/report state must remain untouched"
        );
    }
}

// ---------------------------------------------------------------------------------------
// 2. MCP never-clobber: pre-existing broker/repowire/weave survive a sync
// ---------------------------------------------------------------------------------------

#[test]
fn mcp_sync_is_additive_never_clobbers_existing_servers() {
    let (engine, project, cfg) = project_with_config(&full_config(&pack_dir()));

    // Seed a pre-existing .mcp.json with three global servers NOT tracked by the agent lock.
    let pre = serde_json::json!({
        "mcpServers": {
            "broker": { "command": "broker" },
            "repowire": { "command": "repowire" },
            "weave": { "command": "weave" }
        }
    });
    std::fs::write(
        project.join(".mcp.json"),
        serde_json::to_string_pretty(&pre).unwrap(),
    )
    .unwrap();

    let (s, _rx) = sink();
    let report = engine
        .agent_sync(
            AgentSyncSpec {
                config_path: Some(cfg),
                apply: true,
                ..Default::default()
            },
            &s,
        )
        .expect("apply sync");
    assert_eq!(report.summary.failed, 0);

    let merged: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(project.join(".mcp.json")).unwrap()).unwrap();
    let servers = merged["mcpServers"].as_object().unwrap();
    // 3 pre-existing + 2 from the fixture pack (github, context7) = 5 present.
    for name in ["broker", "repowire", "weave", "github", "context7"] {
        assert!(
            servers.contains_key(name),
            "{name} must be present after sync"
        );
    }
}

#[test]
fn live_agent_skills_mcp_pack_preserves_mesh_servers_and_enforces_yazelix_mirror() {
    let pack = repo_agent_skills_dir();
    let yaml = format!(
        "agent:\n  - claude-code\n  - codex\nscope: project\nmcps:\n  - source: {pack}\n    mcps:\n      - exa\n",
        pack = pack.display()
    );
    let (engine, project, cfg) = project_with_config(&yaml);

    let mesh_json = serde_json::json!({
        "mcpServers": {
            "broker": { "command": "broker", "env": { "TOKEN": "real-broker-token" } },
            "repowire": { "command": "repowire" },
            "weave": { "url": "https://weave.local" }
        }
    });
    std::fs::write(
        project.join(".mcp.json"),
        serde_json::to_string_pretty(&mesh_json).unwrap(),
    )
    .unwrap();

    std::fs::create_dir_all(project.join(".codex")).unwrap();
    std::fs::write(
        project.join(".codex/config.toml"),
        r#"[mcp_servers.broker]
command = "broker"
[mcp_servers.broker.env]
TOKEN = "real-broker-token"
[mcp_servers.repowire]
command = "repowire"
[mcp_servers.weave]
url = "https://weave.local"
"#,
    )
    .unwrap();

    let (s, _rx) = sink();
    let report = engine
        .agent_sync(
            AgentSyncSpec {
                config_path: Some(cfg),
                apply: true,
                ..Default::default()
            },
            &s,
        )
        .expect("apply sync");
    assert_eq!(report.summary.failed, 0);

    let expected = ["broker", "repowire", "weave", "exa"];
    let retired = [
        "github",
        "context7",
        "memory",
        "playwright",
        "sequential-thinking",
        "n8n-mcp",
    ];

    let claude: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(project.join(".mcp.json")).unwrap()).unwrap();
    let claude_servers = claude["mcpServers"].as_object().unwrap();
    for name in expected {
        assert!(
            claude_servers.contains_key(name),
            "Claude MCP config must contain {name}"
        );
    }
    for name in retired {
        assert!(
            !claude_servers.contains_key(name),
            "Claude MCP config must not restore retired MCP {name}"
        );
    }
    assert!(
        claude_servers["exa"].get("url").is_some(),
        "Claude exa MCP must remain URL-only"
    );
    assert!(
        claude_servers["exa"].get("command").is_none(),
        "Claude exa MCP must not use a local launcher"
    );
    assert_eq!(
        claude_servers["broker"]["env"]["TOKEN"], "real-broker-token",
        "Claude merge must not overwrite the existing broker secret"
    );

    let codex: toml::Value = std::fs::read_to_string(project.join(".codex/config.toml"))
        .unwrap()
        .parse()
        .unwrap();
    let codex_servers = codex["mcp_servers"].as_table().unwrap();
    for name in expected {
        assert!(
            codex_servers.contains_key(name),
            "Codex MCP config must contain {name}"
        );
    }
    for name in retired {
        assert!(
            !codex_servers.contains_key(name),
            "Codex MCP config must not restore retired MCP {name}"
        );
    }
    assert!(
        codex_servers["exa"].get("url").is_some(),
        "Codex exa MCP must remain URL-only"
    );
    assert!(
        codex_servers["exa"].get("command").is_none(),
        "Codex exa MCP must not use a local launcher"
    );
    assert_eq!(
        codex_servers["broker"]["env"]["TOKEN"].as_str().unwrap(),
        "real-broker-token",
        "Codex merge must not overwrite the existing broker secret"
    );
}

// ---------------------------------------------------------------------------------------
// 3. lock --check drift (mutate content -> drift; clean -> empty)
// ---------------------------------------------------------------------------------------

#[test]
fn lock_check_reports_drift_then_clean() {
    let (engine, project, cfg) = project_with_config(&full_config(&pack_dir()));

    // Write the lock first.
    let (s, _rx) = sink();
    engine
        .agent_lock(
            AgentLockSpec {
                config_path: Some(cfg.clone()),
                scope_override: None,
                check: false,
                upgrade_only: Vec::new(),
                lock_mode: AgentLockMode::Plain,
            },
            &s,
        )
        .expect("lock write");
    assert!(project.join("agent-env.lock").is_file());

    // Clean check: no drift.
    let (s2, _rx2) = sink();
    let clean = engine
        .agent_lock(
            AgentLockSpec {
                config_path: Some(cfg.clone()),
                scope_override: None,
                check: true,
                upgrade_only: Vec::new(),
                lock_mode: AgentLockMode::Plain,
            },
            &s2,
        )
        .expect("lock check clean");
    assert!(clean.check && !clean.saved);
    assert!(clean.drift.is_empty(), "no drift right after writing");

    // Mutate a skill's content -> the re-resolved hash differs -> drift.
    let copied = pack_dir(); // mutate a COPY so the committed fixture stays pristine.
    let mutated_pack = project.join("pack");
    copy_tree(&copied, &mutated_pack);
    std::fs::write(
        mutated_pack.join("alpha/SKILL.md"),
        "---\nname: alpha\n---\nMUTATED\n",
    )
    .unwrap();
    let cfg2_path = project.join("agent-env-2.yaml");
    std::fs::write(&cfg2_path, full_config(&mutated_pack)).unwrap();

    let (s3, _rx3) = sink();
    let drifted = engine
        .agent_lock(
            AgentLockSpec {
                config_path: Some(cfg2_path.to_string_lossy().to_string()),
                scope_override: None,
                check: true,
                upgrade_only: Vec::new(),
                lock_mode: AgentLockMode::Plain,
            },
            &s3,
        )
        .expect("lock check drifted");
    assert!(!drifted.drift.is_empty(), "mutated content drifts");
    assert!(drifted.drift.iter().any(|d| d.id.contains("alpha")));
}

#[test]
fn locked_check_reports_local_command_and_mcp_content_drift() {
    let source = unique_dir("envctl-agent-it-lock-assets");
    copy_tree(&pack_dir(), &source);
    copy_tree(&cmdpack_dir(), &source);
    let yaml = format!(
        "agent: claude-code\nscope: project\nmcps:\n  - source: {source}\n    mcps: \"*\"\ncommands:\n  - source: {source}\n    commands: \"*\"\n",
        source = source.display()
    );
    let (engine, project, cfg) = project_with_config(&yaml);
    let (s, _rx) = sink();
    engine
        .agent_sync(
            AgentSyncSpec {
                config_path: Some(cfg.clone()),
                apply: true,
                ..Default::default()
            },
            &s,
        )
        .expect("seed command/MCP lock");
    let lock_path = project.join("agent-env.lock");
    let lock_before = std::fs::read_to_string(&lock_path).unwrap();

    std::fs::write(source.join("commands/foo.md"), "# changed local command\n").unwrap();
    std::fs::write(
        source.join("mcps/servers.json"),
        r#"{"mcpServers":{"replacement":{"command":"replacement"}}}"#,
    )
    .unwrap();

    let (s2, _rx2) = sink();
    let checked = engine
        .agent_lock(
            AgentLockSpec {
                config_path: Some(cfg),
                scope_override: None,
                check: true,
                upgrade_only: Vec::new(),
                lock_mode: AgentLockMode::Locked,
            },
            &s2,
        )
        .expect("locked local asset audit");
    assert!(
        checked
            .drift
            .iter()
            .any(|d| { d.status == "updated" && d.id.starts_with("command::") }),
        "command drift: {:?}",
        checked.drift
    );
    assert!(
        checked
            .drift
            .iter()
            .any(|d| { d.status == "updated" && d.id.starts_with("mcp::") }),
        "MCP drift: {:?}",
        checked.drift
    );
    assert_eq!(std::fs::read_to_string(&lock_path).unwrap(), lock_before);
}

// ---------------------------------------------------------------------------------------
// 4. --locked zero-network (unlocked source -> locked_error + failed, no fetch)
// ---------------------------------------------------------------------------------------

#[test]
fn locked_mode_fails_closed_without_lock_then_passes_when_locked() {
    let (engine, project, cfg) = project_with_config(&full_config(&pack_dir()));

    // No lock yet: --locked must fail-closed (locked_error) without installing anything.
    let (s, _rx) = sink();
    let report = engine
        .agent_sync(
            AgentSyncSpec {
                config_path: Some(cfg.clone()),
                apply: true,
                lock_mode: AgentLockMode::Locked,
                ..Default::default()
            },
            &s,
        )
        .expect("locked sync (no lock)");
    assert!(
        report.summary.failed > 0,
        "unlocked source under --locked fails closed"
    );
    assert!(
        report.summary.installed == 0,
        "no install under failing --locked"
    );
    assert!(
        report.actions.iter().any(|a| a.status == "locked_error"),
        "a locked_error is recorded"
    );
    assert!(
        !project.join(".claude/skills/alpha").exists(),
        "no fetch/install happened"
    );

    // Now write the lock + install plainly, then --locked is satisfied (unchanged, no fetch).
    let (s2, _rx2) = sink();
    engine
        .agent_sync(
            AgentSyncSpec {
                config_path: Some(cfg.clone()),
                apply: true,
                lock_mode: AgentLockMode::Plain,
                ..Default::default()
            },
            &s2,
        )
        .unwrap();

    let (s3, _rx3) = sink();
    let locked_ok = engine
        .agent_sync(
            AgentSyncSpec {
                config_path: Some(cfg),
                apply: true,
                lock_mode: AgentLockMode::Locked,
                ..Default::default()
            },
            &s3,
        )
        .expect("locked sync (satisfied)");
    assert_eq!(
        locked_ok.summary.failed, 0,
        "satisfied lock passes --locked"
    );
    assert!(locked_ok.summary.unchanged >= 2);
}

#[test]
fn locked_sync_materializes_hash_locked_local_skills_commands_and_mcps() {
    let source = local_asset_source();
    let (engine, project, cfg) = project_with_config(&local_asset_config(&source));
    write_agent_lock(&engine, &cfg);

    let lock_path = project.join("agent-env.lock");
    assert!(!project.join(".claude").exists());
    assert!(!project.join(".mcp.json").exists());

    // A desired-only lock is not ownership evidence. Locked mode must not turn an arbitrary
    // checked-in lock into permission to install, even when the local source bytes match.
    let (s, _rx) = sink();
    let unproven = engine
        .agent_sync(
            AgentSyncSpec {
                config_path: Some(cfg.clone()),
                apply: true,
                lock_mode: AgentLockMode::Locked,
                ..Default::default()
            },
            &s,
        )
        .expect("locked local sync");
    assert_eq!(unproven.summary.failed, 1);
    assert!(!project.join(".claude").exists());

    // Plain apply creates portable installed-output proofs. Once proven, deleting the disposable
    // outputs and re-running locked can restore them zero-network from the hash-locked local input.
    let (plain_sink, _plain_rx) = sink();
    engine
        .agent_sync(
            AgentSyncSpec {
                config_path: Some(cfg.clone()),
                apply: true,
                lock_mode: AgentLockMode::Plain,
                ..Default::default()
            },
            &plain_sink,
        )
        .expect("plain ownership bootstrap");
    let lock_before = std::fs::read(&lock_path).unwrap();
    std::fs::remove_dir_all(project.join(".claude")).unwrap();
    std::fs::remove_file(project.join(".mcp.json")).unwrap();

    let (proven_sink, _proven_rx) = sink();
    let report = engine
        .agent_sync(
            AgentSyncSpec {
                config_path: Some(cfg),
                apply: true,
                lock_mode: AgentLockMode::Locked,
                ..Default::default()
            },
            &proven_sink,
        )
        .expect("proven locked local sync");

    assert_eq!(report.summary.failed, 0, "actions: {:?}", report.actions);
    assert!(project.join(".claude/skills/alpha/SKILL.md").is_file());
    assert!(project.join(".claude/commands/foo.md").is_file());
    let mcp: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(project.join(".mcp.json")).unwrap()).unwrap();
    assert!(mcp["mcpServers"]["github"].is_object());
    assert!(mcp["mcpServers"]["context7"].is_object());
    assert_eq!(
        std::fs::read(&lock_path).unwrap(),
        lock_before,
        "--locked must not rewrite the validated lock"
    );
}

#[test]
fn locked_local_source_drift_is_atomic_for_every_asset_kind() {
    for kind in ["skill", "command", "mcp"] {
        let source = local_asset_source();
        let (engine, project, cfg) = project_with_config(&local_asset_config(&source));
        write_agent_lock(&engine, &cfg);
        let lock_path = project.join("agent-env.lock");
        let lock_before = std::fs::read(&lock_path).unwrap();

        match kind {
            "skill" => std::fs::write(source.join("alpha/SKILL.md"), "# drifted skill\n").unwrap(),
            "command" => {
                std::fs::write(source.join("commands/foo.md"), "# drifted command\n").unwrap()
            }
            "mcp" => std::fs::write(
                source.join("mcps/servers.json"),
                r#"{"mcpServers":{"drifted":{"command":"false"}}}"#,
            )
            .unwrap(),
            _ => unreachable!(),
        }

        let (s, _rx) = sink();
        let report = engine
            .agent_sync(
                AgentSyncSpec {
                    config_path: Some(cfg),
                    apply: true,
                    lock_mode: AgentLockMode::Locked,
                    ..Default::default()
                },
                &s,
            )
            .expect("locked drift refusal");

        assert!(report.summary.failed > 0, "{kind}: {:#?}", report.actions);
        assert!(
            report.actions.iter().any(|action| {
                action.status == "locked_error"
                    && action
                        .error
                        .as_deref()
                        .is_some_and(|error| error.contains("lock drift"))
            }),
            "{kind}: {:#?}",
            report.actions
        );
        assert!(
            !project.join(".claude").exists() && !project.join(".mcp.json").exists(),
            "{kind}: a failed preflight must not create or mutate destinations"
        );
        assert_eq!(
            std::fs::read(&lock_path).unwrap(),
            lock_before,
            "{kind}: a failed preflight must not rewrite the lock"
        );
    }
}

#[test]
fn locked_local_metadata_drift_is_atomic_and_fail_closed() {
    for field in ["identity", "selector", "destination", "revision", "scope"] {
        let source = local_asset_source();
        let source_name = source.to_string_lossy().to_string();
        let (engine, project, cfg) = project_with_config(&local_asset_config(&source));
        write_agent_lock(&engine, &cfg);
        let lock_path = project.join("agent-env.lock");
        let mut lock = envctl_agent_env::lock::load(&lock_path).unwrap();

        let skill_id = skill_key(&source_name, "alpha");
        let command_id = command_asset_id(&source_name, "foo");
        let mcp_id = mcp_asset_id(&source_name, "servers.json");
        assert!(lock.source_selectors.contains_key(&skill_id));
        assert!(lock.source_selectors.contains_key(&command_id));
        assert!(lock.source_selectors.contains_key(&mcp_id));

        match field {
            "identity" => lock.skills.get_mut(&skill_id).unwrap().source = "other-source".into(),
            "selector" => {
                lock.source_selectors
                    .insert(command_id, "v1|kind=command|selection=other".into());
            }
            "destination" => {
                lock.assets.get_mut(&mcp_id).unwrap().destination = "other-server".into();
            }
            "revision" => {
                lock.assets.get_mut(&command_id).unwrap().source_revision = "branch:main".into();
            }
            "scope" => lock.skills.get_mut(&skill_id).unwrap().scope = None,
            _ => unreachable!(),
        }
        envctl_agent_env::lock::save(&mut lock, &lock_path).unwrap();
        let lock_before = std::fs::read(&lock_path).unwrap();

        let (s, _rx) = sink();
        let report = engine
            .agent_sync(
                AgentSyncSpec {
                    config_path: Some(cfg),
                    apply: true,
                    lock_mode: AgentLockMode::Locked,
                    ..Default::default()
                },
                &s,
            )
            .expect("locked metadata refusal");

        assert!(report.summary.failed > 0, "{field}: {:#?}", report.actions);
        assert!(
            report.actions.iter().any(|action| {
                action.status == "locked_error"
                    && action
                        .error
                        .as_deref()
                        .is_some_and(|error| error.contains("lock drift"))
            }),
            "{field}: {:#?}",
            report.actions
        );
        assert!(
            !project.join(".claude").exists() && !project.join(".mcp.json").exists(),
            "{field}: a failed metadata preflight must not mutate destinations"
        );
        assert_eq!(
            std::fs::read(&lock_path).unwrap(),
            lock_before,
            "{field}: a failed metadata preflight must not rewrite the lock"
        );
    }
}

#[test]
fn locked_remote_missing_destination_refuses_without_network_input() {
    let source = "https://network-sentinel.invalid/org/skills";
    let (engine, project, cfg) = project_with_config(&format!(
        "destination: ./dest\nscope: project\nskills:\n  - source: {source}\n    ref: deadbeef\n    skills:\n      - alpha\n"
    ));
    let lock_path = project.join("agent-env.lock");
    let id = skill_key(source, "alpha");
    let mut lock = AgentLockFile::default();
    lock.skills.insert(
        id.clone(),
        AgentLockEntry {
            destination: "dest/alpha".into(),
            hash: "locked-remote-content-hash".into(),
            skill: "alpha".into(),
            description: "remote alpha".into(),
            source: source.into(),
            source_revision: "ref:deadbeef".into(),
            scope: Some(Scope::Project),
        },
    );
    lock.set_source_selector(
        &id,
        Some(
            "v2|base=38:v1|kind=skill|sub-dir=-|selection=name|scope=project|targets=1|10:dest/alpha"
                .into(),
        ),
    );
    envctl_agent_env::lock::save(&mut lock, &lock_path).unwrap();
    let lock_before = std::fs::read(&lock_path).unwrap();

    let (s, _rx) = sink();
    let report = engine
        .agent_sync(
            AgentSyncSpec {
                config_path: Some(cfg),
                apply: true,
                lock_mode: AgentLockMode::Locked,
                ..Default::default()
            },
            &s,
        )
        .expect("remote locked refusal");

    assert_eq!(report.summary.failed, 1, "{:#?}", report.actions);
    assert!(report.actions.iter().any(|action| {
        action.status == "locked_error"
            && action
                .error
                .as_deref()
                .is_some_and(|error| error.contains("no verified source input"))
    }));
    assert!(
        !project.join("dest").exists(),
        "remote locked refusal must happen before destination mutation"
    );
    assert_eq!(std::fs::read(&lock_path).unwrap(), lock_before);
}

#[test]
fn locked_sync_rejects_remote_extends_before_fetch() {
    let yaml = "extends: https://network-sentinel.invalid/base.yaml\nagent: claude-code\nscope: project\nskills: []\n";
    let (engine, _project, cfg) = project_with_config(yaml);
    let (s, _rx) = sink();
    let err = engine
        .agent_sync(
            AgentSyncSpec {
                config_path: Some(cfg),
                apply: false,
                lock_mode: AgentLockMode::Locked,
                ..Default::default()
            },
            &s,
        )
        .expect_err("locked sync must reject remote extends locally");
    let chain = format!("{err:#}");
    assert!(
        chain.contains("--locked forbids remote config fetch"),
        "must refuse before HTTP: {chain}"
    );
}

// ---------------------------------------------------------------------------------------
// 5. remove + sync-after prune
// ---------------------------------------------------------------------------------------

#[test]
fn remove_then_sync_after_prunes_skill() {
    // Skills-only config so we can drop a named skill list.
    let pack = pack_dir();
    let yaml = format!(
        "agent: claude-code\nscope: project\nskills:\n  - source: {p}\n    skills:\n      - alpha\n      - beta\n",
        p = pack.display()
    );
    let (engine, project, cfg) = project_with_config(&yaml);

    // Install both.
    let (s, _rx) = sink();
    engine
        .agent_sync(
            AgentSyncSpec {
                config_path: Some(cfg.clone()),
                apply: true,
                ..Default::default()
            },
            &s,
        )
        .unwrap();
    assert!(project.join(".claude/skills/alpha/SKILL.md").is_file());
    assert!(project.join(".claude/skills/beta/SKILL.md").is_file());

    // Remove `beta` from the skills list, with sync-after (apply).
    let (s2, _rx2) = sink();
    let outcome = engine
        .agent_remove(
            AgentRemoveSpec {
                source: pack.display().to_string(),
                section: AgentSectionSel {
                    skills: vec!["beta".into()],
                    ..Default::default()
                },
                git_ref: None,
                branch: None,
                sub_dir: None,
                config_path: Some(cfg.clone()),
                scope_override: None,
                apply: true,
                no_sync: false,
                lock_mode: AgentLockMode::Plain,
            },
            &s2,
        )
        .expect("remove beta");
    assert_eq!(outcome.action, "removed");
    assert!(outcome.sync.is_some(), "sync-after ran");
    // alpha kept, beta pruned.
    assert!(project.join(".claude/skills/alpha/SKILL.md").is_file());
    assert!(
        !project.join(".claude/skills/beta").exists(),
        "beta pruned by sync-after"
    );
}

// ---------------------------------------------------------------------------------------
// 6. clean preview vs apply (untracked MCP survives)
// ---------------------------------------------------------------------------------------

#[test]
fn clean_preview_keeps_then_apply_removes_tracked_only() {
    let _guard = cwd_lock().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let (engine, project, cfg) = project_with_config(&full_config(&pack_dir()));

    // Seed an untracked global server alongside what the sync will add.
    std::fs::write(
        project.join(".mcp.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "mcpServers": { "weave": { "command": "weave" } }
        }))
        .unwrap(),
    )
    .unwrap();

    let (s, _rx) = sink();
    engine
        .agent_sync(
            AgentSyncSpec {
                config_path: Some(cfg),
                apply: true,
                ..Default::default()
            },
            &s,
        )
        .unwrap();
    assert!(project.join(".claude/skills/alpha").exists());

    // clean + list are cwd-based.
    let prev_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(&project).unwrap();

    // list (read-only) sees the installed skills + the synced MCP servers.
    let (sl, _rxl) = sink();
    let listed = engine
        .agent_list(
            AgentListSpec {
                scope_override: Some(AgentScope::Project),
                kind: AgentListKind::All,
            },
            &sl,
        )
        .expect("list");
    assert!(listed.skills.iter().any(|s| s.skill == "alpha"));
    assert!(listed.mcps.iter().any(|m| m.name == "github"));

    // list filtered to skills only drops the MCP rows.
    let (sl2, _rxl2) = sink();
    let skills_only = engine
        .agent_list(
            AgentListSpec {
                scope_override: Some(AgentScope::Project),
                kind: AgentListKind::Skills,
            },
            &sl2,
        )
        .expect("list skills");
    assert!(skills_only.mcps.is_empty(), "skills-only list has no mcps");

    // Preview: nothing removed.
    let (s2, _rx2) = sink();
    let preview = engine
        .agent_clean(
            AgentCleanSpec {
                config_path: None,
                scope_override: Some(AgentScope::Project),
                apply: false,
            },
            &s2,
        )
        .expect("clean preview");
    assert!(preview.dry_run);
    assert!(preview.summary.removed >= 1);
    assert!(
        project.join(".claude/skills/alpha").exists(),
        "preview removed nothing"
    );

    // Apply: tracked assets removed, untracked `weave` survives.
    let (s3, _rx3) = sink();
    let applied = engine
        .agent_clean(
            AgentCleanSpec {
                config_path: None,
                scope_override: Some(AgentScope::Project),
                apply: true,
            },
            &s3,
        )
        .expect("clean apply");
    assert!(!applied.dry_run);
    assert!(
        !project.join(".claude/skills/alpha").exists(),
        "tracked skill removed"
    );

    let servers: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(project.join(".mcp.json")).unwrap()).unwrap();
    let map = servers["mcpServers"].as_object().unwrap();
    assert!(
        map.contains_key("weave"),
        "untracked global MCP survives clean"
    );
    assert!(!map.contains_key("github"), "tracked MCP removed by clean");

    std::env::set_current_dir(prev_cwd).unwrap();
}

// ---------------------------------------------------------------------------------------
// 7. M-22 fallback (config_path: None -> scope from default-config file)
// ---------------------------------------------------------------------------------------

#[test]
fn m22_fallback_resolves_default_config_from_cwd() {
    let _guard = cwd_lock().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let (engine, project, _cfg) = project_with_config(&full_config(&pack_dir()));

    // No explicit config_path -> default_config_path() resolves the local `agent-env.yaml`.
    let prev_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(&project).unwrap();

    let (s, _rx) = sink();
    let report = engine
        .agent_sync(
            AgentSyncSpec {
                config_path: None,
                apply: false,
                ..Default::default()
            },
            &s,
        )
        .expect("m22 fallback sync");
    // The default config was found and its project scope resolved.
    assert_eq!(report.scope, AgentScope::Project);
    assert!(
        report.summary.installed >= 2,
        "skills discovered via default config"
    );

    std::env::set_current_dir(prev_cwd).unwrap();
}

// ---------------------------------------------------------------------------------------
// 8. never-prune-on-failure (good + failing source -> good assets kept)
// ---------------------------------------------------------------------------------------

#[test]
fn never_prune_when_a_source_fails() {
    // Install a good skill first, then add a failing (nonexistent) source and re-sync:
    // the failing source bumps `failed`, so the good locked skill must NOT be pruned.
    let pack = pack_dir();
    let good_only = format!(
        "agent: claude-code\nscope: project\nskills:\n  - source: {p}\n    skills:\n      - alpha\n",
        p = pack.display()
    );
    let (engine, project, cfg) = project_with_config(&good_only);

    let (s, _rx) = sink();
    engine
        .agent_sync(
            AgentSyncSpec {
                config_path: Some(cfg),
                apply: true,
                ..Default::default()
            },
            &s,
        )
        .unwrap();
    assert!(project.join(".claude/skills/alpha/SKILL.md").is_file());

    // Now a config that keeps alpha but adds a source that ERRORS at materialize time
    // (a nonexistent `sub-dir` of the real pack -> source_error -> summary.failed++,
    // distinct from a `broken` skill which would NOT trip the never-prune guard).
    let with_broken = format!(
        "agent: claude-code\nscope: project\nskills:\n  - source: {p}\n    skills:\n      - alpha\n  - source: {p}\n    sub-dir: no-such-subdir\n    skills:\n      - ghost\n",
        p = pack.display()
    );
    let cfg2 = project.join("agent-env-2.yaml");
    std::fs::write(&cfg2, with_broken).unwrap();

    let (s2, _rx2) = sink();
    let report = engine
        .agent_sync(
            AgentSyncSpec {
                config_path: Some(cfg2.to_string_lossy().to_string()),
                apply: true,
                ..Default::default()
            },
            &s2,
        )
        .expect("sync with broken source");
    assert!(report.summary.failed > 0, "broken source records a failure");
    assert_eq!(report.summary.removed, 0, "never prune when failed > 0");
    assert!(
        project.join(".claude/skills/alpha/SKILL.md").is_file(),
        "good locked skill survives a failing sibling source"
    );
}

// ---------------------------------------------------------------------------------------
// 9. cross-phase never-prune-on-failure: skills failure must not prune commands/MCPs
// ---------------------------------------------------------------------------------------

#[test]
fn skills_failure_does_not_prune_installed_commands() {
    let cmd_pack = cmdpack_dir();
    let skill_pack = pack_dir();

    // Install a command.
    let cmd_yaml = format!(
        "agent: claude-code\nscope: project\ncommands:\n  - source: {p}\n    commands: \"*\"\n",
        p = cmd_pack.display()
    );
    let (engine, project, cfg) = project_with_config(&cmd_yaml);
    let (s, _rx) = sink();
    engine
        .agent_sync(
            AgentSyncSpec {
                config_path: Some(cfg.clone()),
                apply: true,
                ..Default::default()
            },
            &s,
        )
        .unwrap();
    assert!(
        project.join(".claude/commands/foo.md").is_file(),
        "command installed"
    );

    // Re-sync with a failing skills source and NO commands config.
    let failing_yaml = format!(
        "agent: claude-code\nscope: project\nskills:\n  - source: {p}\n    sub-dir: no-such-subdir\n    skills:\n      - ghost\n",
        p = skill_pack.display()
    );
    let cfg2 = project.join("agent-env-failing.yaml");
    std::fs::write(&cfg2, failing_yaml).unwrap();

    let (s2, _rx2) = sink();
    let report = engine
        .agent_sync(
            AgentSyncSpec {
                config_path: Some(cfg2.to_string_lossy().to_string()),
                apply: true,
                ..Default::default()
            },
            &s2,
        )
        .expect("sync with failing skill");
    assert!(report.summary.failed > 0, "failing skill records a failure");
    assert!(
        project.join(".claude/commands/foo.md").is_file(),
        "installed command is NOT pruned when an earlier phase fails"
    );
}

#[test]
fn skills_failure_does_not_prune_installed_mcps() {
    let skill_pack = pack_dir();

    // Install an MCP.
    let mcp_yaml = format!(
        "agent: claude-code\nscope: project\nmcps:\n  - source: {p}\n    mcps: \"*\"\n",
        p = skill_pack.display()
    );
    let (engine, project, cfg) = project_with_config(&mcp_yaml);
    let (s, _rx) = sink();
    engine
        .agent_sync(
            AgentSyncSpec {
                config_path: Some(cfg.clone()),
                apply: true,
                ..Default::default()
            },
            &s,
        )
        .unwrap();
    let mcp_json = project.join(".mcp.json");
    assert!(mcp_json.is_file(), "mcp settings file created");
    let before: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&mcp_json).unwrap()).unwrap();
    assert!(
        before["mcpServers"]
            .as_object()
            .unwrap()
            .contains_key("github"),
        "github mcp installed"
    );

    // Re-sync with a failing skills source, no MCP config, and no configured agent so the
    // MCP settings target list is empty (the orphaned-MCP fallback path).
    let failing_yaml = format!(
        "destination: .claude\nscope: project\nskills:\n  - source: {p}\n    sub-dir: no-such-subdir\n    skills:\n      - ghost\n",
        p = skill_pack.display()
    );
    let cfg2 = project.join("agent-env-failing.yaml");
    std::fs::write(&cfg2, failing_yaml).unwrap();

    let (s2, _rx2) = sink();
    let report = engine
        .agent_sync(
            AgentSyncSpec {
                config_path: Some(cfg2.to_string_lossy().to_string()),
                apply: true,
                ..Default::default()
            },
            &s2,
        )
        .expect("sync with failing skill and empty mcp targets");
    assert!(report.summary.failed > 0, "failing skill records a failure");
    let after: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&mcp_json).unwrap()).unwrap();
    assert!(
        after["mcpServers"]
            .as_object()
            .unwrap()
            .contains_key("github"),
        "installed MCP is NOT pruned when an earlier phase fails"
    );
}

// ---------------------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------------------

fn copy_tree(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_tree(&from, &to);
        } else {
            std::fs::copy(&from, &to).unwrap();
        }
    }
}

// ---------------------------------------------------------------------------------------
// Transport events (TASK-0014b): `list` and the `add`/`remove` edit outcome have no other
// event carrier — the GUI worker→UI channel is event-only, so the engine MUST emit them or
// the GUI panel renders nothing (the exact regression /verify caught on the CLI list view).
// ---------------------------------------------------------------------------------------

#[test]
fn agent_list_emits_agent_listed_event() {
    // `list` is cwd-based, so this test mutates the process-global cwd — it MUST serialize
    // against the other cwd-mutating tests (clean/m22) via the same lock, or under
    // `cargo test --workspace` parallelism it races and corrupts their cwd-based scope
    // resolution mid-call (the CI failure that blocked #93: `clean ... removed >= 1`).
    let _guard = cwd_lock().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let (engine, project, cfg) = project_with_config(&full_config(&pack_dir()));
    let (s, _rx) = sink();
    engine
        .agent_sync(
            AgentSyncSpec {
                config_path: Some(cfg),
                apply: true,
                ..Default::default()
            },
            &s,
        )
        .unwrap();

    // `list` is cwd-based — set cwd to the project for the call, then restore.
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&project).unwrap();
    let (sl, rxl) = sink();
    let _ = engine
        .agent_list(
            AgentListSpec {
                scope_override: Some(AgentScope::Project),
                kind: AgentListKind::All,
            },
            &sl,
        )
        .expect("list");
    std::env::set_current_dir(prev).unwrap();

    let evs = drain(rxl);
    let listed = evs.iter().find_map(|e| match e {
        Event::AgentListed { list } => Some(list),
        _ => None,
    });
    let list = listed.expect("agent_list must emit Event::AgentListed (GUI transport)");
    assert!(
        list.skills.iter().any(|s| s.skill == "alpha"),
        "AgentListed payload must carry the inventory the typed return has"
    );
}

#[test]
fn agent_remove_preview_emits_agent_edited_event() {
    let (engine, _project, cfg) = project_with_config(&full_config(&pack_dir()));
    let (s, rx) = sink();
    // Preview remove (apply:false) of the configured source — records `would_remove`, no writes.
    let _ = engine
        .agent_remove(
            AgentRemoveSpec {
                source: pack_dir().display().to_string(),
                section: AgentSectionSel::default(),
                git_ref: None,
                branch: None,
                sub_dir: None,
                config_path: Some(cfg),
                scope_override: Some(AgentScope::Project),
                apply: false,
                no_sync: false,
                lock_mode: AgentLockMode::Plain,
            },
            &s,
        )
        .expect("remove preview");
    let evs = drain(rx);
    let edited = evs.iter().find_map(|e| match e {
        Event::AgentEdited { outcome } => Some(outcome),
        _ => None,
    });
    let outcome = edited.expect("agent_remove must emit Event::AgentEdited (GUI transport)");
    assert!(
        outcome.action.contains("remove"),
        "AgentEdited carries the would_remove/removed outcome, got action={}",
        outcome.action
    );
}
