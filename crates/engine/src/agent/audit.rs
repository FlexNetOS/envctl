//! Strict, zero-network `Engine::agent_audit` for fleet policy gates.
//!
//! `agent doctor` deliberately remains config-optional and diagnostic-only. This module is the
//! complementary fail-closed gate: it resolves one config, reads only its resolved lock and
//! native destinations, validates exact config-to-lock asset identity, recomputes every installed
//! skill target hash, and records command/MCP ownership and native-target presence conflicts. It
//! never fetches, writes, or prints.

use std::collections::{BTreeMap, BTreeSet};

use envctl_agent_env::command::destination_path;
use envctl_agent_env::config::{CommandsField, McpsField, SkillsField};
use envctl_agent_env::fsops::{
    resolve_command_targets, resolve_dest, resolve_mcp_settings_targets,
};
use envctl_agent_env::hash::hash_dir;
use envctl_agent_env::lock::{self, LOCK_VERSION};
use envctl_agent_env::mcp::servers_present_in_settings;
use envctl_agent_env::sync::{
    command_asset_id, desired_command_names, desired_mcp_file_names, desired_skill_names,
    mcp_asset_id, skill_key,
};

use crate::agent::report::{
    AgentAuditIssue, AgentAuditReport, AgentCommandAudit, AgentCommandTargetAudit, AgentMcpAudit,
    AgentMcpTargetAudit, AgentSkillAudit, AgentSkillTargetAudit, AgentVerb,
};
use crate::agent::{AgentAuditSpec, AgentCtx};
use crate::event::{Event, EventSink};
use crate::Engine;

/// A config-declared source and its exact expected lock revision, grouped by managed kind.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ExpectedSource {
    kind: &'static str,
    source: String,
    revision: String,
}

impl Engine {
    /// Audit one resolved agent-env config without mutation or network access.
    ///
    /// A non-empty [`AgentAuditReport::issues`] is an unhealthy fleet-policy result; the CLI
    /// maps that typed result to exit 1 while callers can inspect the full evidence.
    pub fn agent_audit(
        &self,
        spec: AgentAuditSpec,
        sink: &EventSink,
    ) -> anyhow::Result<AgentAuditReport> {
        let ctx = AgentCtx::resolve(spec.config_path.as_deref(), spec.scope_override)?;
        sink.emit(Event::AgentRunStarted {
            verb: AgentVerb::Audit,
            scope: ctx.scope.into(),
            dry_run: true,
            lock_mode: "locked".into(),
        });

        let lock_present = ctx.lock_file.is_file();
        let lock = lock::inspect(&ctx.lock_file)?;
        let mut issues = Vec::new();
        if !lock_present {
            issue(
                &mut issues,
                "lock_missing",
                "agent-env.lock",
                format!(
                    "expected locked project state at {}",
                    ctx.lock_file.display()
                ),
            );
        }

        validate_config_lock(&ctx.cfg, &lock, &mut issues);
        let skills = audit_skills(&ctx, &lock, &mut issues);
        let commands = audit_commands(&ctx, &lock, &mut issues)?;
        let mcps = audit_mcps(&ctx, &lock, &mut issues)?;

        issues.sort_by(|a, b| (&a.kind, &a.id, &a.detail).cmp(&(&b.kind, &b.id, &b.detail)));
        let report = AgentAuditReport {
            config: ctx.cfg_label,
            lock_file: ctx.lock_file.to_string_lossy().to_string(),
            scope: ctx.scope.into(),
            lock_present,
            lock_version: lock.version,
            skills,
            commands,
            mcps,
            issues,
        };
        sink.emit(Event::AgentAudited {
            report: report.clone(),
        });
        Ok(report)
    }
}

fn expected_sources(cfg: &envctl_agent_env::Config) -> BTreeSet<ExpectedSource> {
    let mut expected = BTreeSet::new();
    for source in &cfg.skills {
        expected.insert(ExpectedSource {
            kind: "skill",
            source: source.source.clone(),
            revision: source.expected_revision(),
        });
    }
    for source in &cfg.mcps {
        expected.insert(ExpectedSource {
            kind: "mcp",
            source: source.source.clone(),
            revision: source.as_source_spec().expected_revision(),
        });
    }
    for source in &cfg.commands {
        expected.insert(ExpectedSource {
            kind: "command",
            source: source.source.clone(),
            revision: source.as_source_spec().expected_revision(),
        });
    }
    expected
}

fn validate_config_lock(
    cfg: &envctl_agent_env::Config,
    lock: &envctl_agent_env::lock::AgentLockFile,
    issues: &mut Vec<AgentAuditIssue>,
) {
    if lock.version != LOCK_VERSION {
        issue(
            issues,
            "unsupported_lock_version",
            "agent-env.lock",
            format!(
                "expected lock schema version {LOCK_VERSION}, found {}",
                lock.version
            ),
        );
    }

    let mut expected_ids = BTreeSet::new();
    for source in &cfg.skills {
        if matches!(&source.skills, SkillsField::Wildcard(value) if value != "*") {
            issue(
                issues,
                "invalid_config_selector",
                format!("skill:{}", source.source),
                "skills wildcard must be exactly `*`".into(),
            );
        }
        let names = desired_skill_names(source, lock);
        if names.is_empty() {
            issue(
                issues,
                "config_source_selects_no_assets",
                format!("skill:{}", source.source),
                "the configured skill source resolves to no locked skills".into(),
            );
        }
        for name in names {
            expected_ids.insert(("skill", skill_key(&source.source, &name)));
        }
    }
    for source in &cfg.commands {
        if matches!(&source.commands, CommandsField::Wildcard(value) if value != "*") {
            issue(
                issues,
                "invalid_config_selector",
                format!("command:{}", source.source),
                "commands wildcard must be exactly `*`".into(),
            );
        }
        let names = desired_command_names(source, lock);
        if names.is_empty() {
            issue(
                issues,
                "config_source_selects_no_assets",
                format!("command:{}", source.source),
                "the configured command source resolves to no locked commands".into(),
            );
        }
        for name in names {
            expected_ids.insert(("command", command_asset_id(&source.source, &name)));
        }
    }
    for source in &cfg.mcps {
        if matches!(&source.mcps, McpsField::Wildcard(value) if value != "*") {
            issue(
                issues,
                "invalid_config_selector",
                format!("mcp:{}", source.source),
                "MCP wildcard must be exactly `*`".into(),
            );
        }
        let names = desired_mcp_file_names(source, lock);
        if names.is_empty() {
            issue(
                issues,
                "config_source_selects_no_assets",
                format!("mcp:{}", source.source),
                "the configured MCP source resolves to no locked MCP files".into(),
            );
        }
        for name in names {
            expected_ids.insert(("mcp", mcp_asset_id(&source.source, &name)));
        }
    }

    for (kind, id) in &expected_ids {
        let present = match *kind {
            "skill" => lock.skills.contains_key(id),
            "command" | "mcp" => lock.assets.get(id).is_some_and(|asset| asset.kind == *kind),
            _ => false,
        };
        if !present {
            issue(
                issues,
                "config_asset_missing_from_lock",
                id,
                format!("the config-selected {kind} has no canonical lock entry"),
            );
        }
    }

    let expected = expected_sources(cfg);
    let mut locked: BTreeMap<(&str, String), BTreeSet<String>> = BTreeMap::new();
    for (id, entry) in &lock.skills {
        let canonical_id = skill_key(&entry.source, &entry.skill);
        if *id != canonical_id {
            issue(
                issues,
                "lock_id_mismatch",
                id,
                format!("canonical skill lock id is `{canonical_id}`"),
            );
        }
        if !expected_ids.contains(&("skill", canonical_id)) {
            issue(
                issues,
                "lock_asset_not_in_config",
                id,
                "the locked skill is not selected by the config".into(),
            );
        }
        if entry.source.is_empty() || entry.skill.is_empty() || entry.hash.is_empty() {
            issue(
                issues,
                "invalid_lock_entry",
                id,
                "skill lock entries require non-empty source, skill, and hash fields".into(),
            );
        }
        locked
            .entry(("skill", entry.source.clone()))
            .or_default()
            .insert(entry.source_revision.clone());
    }
    for (id, asset) in &lock.assets {
        match asset.kind.as_str() {
            "mcp" | "command" => {
                let kind = if asset.kind == "mcp" {
                    "mcp"
                } else {
                    "command"
                };
                let canonical_id = if kind == "mcp" {
                    mcp_asset_id(&asset.source, &asset.name)
                } else {
                    command_asset_id(&asset.source, &asset.name)
                };
                if *id != canonical_id {
                    issue(
                        issues,
                        "lock_id_mismatch",
                        id,
                        format!("canonical {} lock id is `{canonical_id}`", asset.kind),
                    );
                }
                if !expected_ids.contains(&(kind, canonical_id)) {
                    issue(
                        issues,
                        "lock_asset_not_in_config",
                        id,
                        format!("the locked {} is not selected by the config", asset.kind),
                    );
                }
                if asset.source.is_empty()
                    || asset.name.is_empty()
                    || asset.hash.is_empty()
                    || asset.destination.is_empty()
                {
                    issue(
                        issues,
                        "invalid_lock_entry",
                        id,
                        "managed asset lock entries require non-empty source, name, hash, and destination fields"
                            .into(),
                    );
                }
                locked
                    .entry((kind, asset.source.clone()))
                    .or_default()
                    .insert(asset.source_revision.clone());
            }
            other => issue(
                issues,
                "unknown_lock_asset_kind",
                &asset.name,
                format!("lock records unsupported managed asset kind `{other}`"),
            ),
        }
    }

    for source in &expected {
        let Some(revisions) = locked.get(&(source.kind, source.source.clone())) else {
            issue(
                issues,
                "config_source_missing_from_lock",
                format!("{}:{}", source.kind, source.source),
                "the config source has no corresponding locked managed asset".into(),
            );
            continue;
        };
        if !revisions.contains(&source.revision) {
            issue(
                issues,
                "lock_revision_mismatch",
                format!("{}:{}", source.kind, source.source),
                format!(
                    "config expects `{}`, lock records {}",
                    source.revision,
                    revisions.iter().cloned().collect::<Vec<_>>().join(", ")
                ),
            );
        }
    }

    for (kind, source) in locked.keys() {
        if !expected
            .iter()
            .any(|expected| expected.kind == *kind && expected.source == *source)
        {
            issue(
                issues,
                "lock_source_not_in_config",
                format!("{kind}:{source}"),
                "the lock claims a managed asset no longer declared by the config".into(),
            );
        }
    }
}

fn audit_skills(
    ctx: &AgentCtx,
    lock: &envctl_agent_env::lock::AgentLockFile,
    issues: &mut Vec<AgentAuditIssue>,
) -> Vec<AgentSkillAudit> {
    let mut skills = Vec::new();
    for (id, entry) in &lock.skills {
        let locked_destination = resolve_dest(&entry.destination, &ctx.scope_root);
        let expected_destinations = ctx
            .destinations
            .iter()
            .map(|destination| destination.join(&entry.skill))
            .collect::<Vec<_>>();
        if !expected_destinations.contains(&locked_destination) {
            issue(
                issues,
                "skill_lock_destination_mismatch",
                id,
                format!(
                    "lock destination {} is not one of the configured native targets",
                    locked_destination.display()
                ),
            );
        }
        let targets = expected_destinations
            .into_iter()
            .map(|destination| {
                let actual_hash = if destination.is_dir() {
                    match hash_dir(&destination) {
                        Ok(hash) => Some(hash),
                        Err(error) => {
                            issue(
                                issues,
                                "skill_hash_unreadable",
                                id,
                                format!("could not hash {}: {error}", destination.display()),
                            );
                            None
                        }
                    }
                } else {
                    issue(
                        issues,
                        "skill_destination_missing",
                        id,
                        format!(
                            "expected installed skill directory {}",
                            destination.display()
                        ),
                    );
                    None
                };
                let matches_lock = actual_hash.as_deref() == Some(entry.hash.as_str());
                if actual_hash.is_some() && !matches_lock {
                    issue(
                        issues,
                        "skill_hash_mismatch",
                        id,
                        format!(
                            "target={}, lock={}, installed={}",
                            destination.display(),
                            entry.hash,
                            actual_hash.as_deref().unwrap_or_default()
                        ),
                    );
                }
                AgentSkillTargetAudit {
                    path: destination.to_string_lossy().to_string(),
                    actual_hash,
                    matches_lock,
                }
            })
            .collect();
        if let Some(scope) = entry.scope {
            if scope != ctx.scope {
                issue(
                    issues,
                    "skill_scope_mismatch",
                    id,
                    "lock entry scope differs from the resolved config scope".into(),
                );
            }
        }
        skills.push(AgentSkillAudit {
            id: id.clone(),
            source: entry.source.clone(),
            expected_hash: entry.hash.clone(),
            targets,
        });
    }
    skills.sort_by(|a, b| a.id.cmp(&b.id));
    skills
}

fn audit_commands(
    ctx: &AgentCtx,
    lock: &envctl_agent_env::lock::AgentLockFile,
    issues: &mut Vec<AgentAuditIssue>,
) -> anyhow::Result<Vec<AgentCommandAudit>> {
    let native_targets = resolve_command_targets(&ctx.cfg, ctx.scope, &ctx.cfg_dir)?;
    let mut owners: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for asset in lock.assets.values().filter(|asset| asset.kind == "command") {
        owners
            .entry(asset.name.clone())
            .or_default()
            .insert(asset.source.clone());
    }

    let mut commands = Vec::new();
    for (name, owner_set) in owners {
        let targets = native_targets
            .iter()
            .map(|target| {
                let path = destination_path(target, &name);
                AgentCommandTargetAudit {
                    present: path.is_file(),
                    path: path.to_string_lossy().to_string(),
                }
            })
            .collect::<Vec<_>>();
        let conflict = owner_set.len() > 1;
        if conflict {
            issue(
                issues,
                "command_ownership_conflict",
                &name,
                format!(
                    "multiple locked sources claim this command: {}",
                    owner_set.iter().cloned().collect::<Vec<_>>().join(", ")
                ),
            );
        }
        if targets.is_empty() || targets.iter().any(|target| !target.present) {
            issue(
                issues,
                "managed_command_missing",
                &name,
                if targets.is_empty() {
                    "the config has no native command target for this managed command".into()
                } else {
                    "the managed command is absent from one or more configured native targets"
                        .into()
                },
            );
        }
        commands.push(AgentCommandAudit {
            name,
            owners: owner_set.into_iter().collect(),
            conflict,
            targets,
        });
    }
    Ok(commands)
}

fn audit_mcps(
    ctx: &AgentCtx,
    lock: &envctl_agent_env::lock::AgentLockFile,
    issues: &mut Vec<AgentAuditIssue>,
) -> anyhow::Result<Vec<AgentMcpAudit>> {
    let targets = resolve_mcp_settings_targets(&ctx.cfg, ctx.scope, &ctx.cfg_dir)?;
    let mut owners: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for asset in lock.assets.values().filter(|asset| asset.kind == "mcp") {
        for name in asset
            .destination
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            owners
                .entry(name.to_string())
                .or_default()
                .insert(asset.source.clone());
        }
    }

    let mut mcps = Vec::new();
    for (name, owner_set) in owners {
        let targets = targets
            .iter()
            .map(|target| AgentMcpTargetAudit {
                path: target.path.to_string_lossy().to_string(),
                present: servers_present_in_settings(std::slice::from_ref(&name), target),
            })
            .collect::<Vec<_>>();
        let conflict = owner_set.len() > 1;
        if conflict {
            issue(
                issues,
                "mcp_ownership_conflict",
                &name,
                format!(
                    "multiple locked sources claim this server: {}",
                    owner_set.iter().cloned().collect::<Vec<_>>().join(", ")
                ),
            );
        }
        if targets.is_empty() || targets.iter().any(|target| !target.present) {
            issue(
                issues,
                "managed_mcp_missing",
                &name,
                if targets.is_empty() {
                    "the config has no MCP target for this managed server".into()
                } else {
                    "the managed server is absent from one or more configured native targets".into()
                },
            );
        }
        mcps.push(AgentMcpAudit {
            name,
            owners: owner_set.into_iter().collect(),
            conflict,
            targets,
        });
    }
    Ok(mcps)
}

fn issue(
    issues: &mut Vec<AgentAuditIssue>,
    kind: impl Into<String>,
    id: impl Into<String>,
    detail: String,
) {
    issues.push(AgentAuditIssue {
        kind: kind.into(),
        id: id.into(),
        detail,
    });
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use envctl_agent_env::hash::hash_dir;
    use envctl_agent_env::lock::{load, save, AgentLockEntry, AgentLockFile, AssetEntry};
    use envctl_agent_env::Scope;

    use super::*;
    use crate::event::EventSink;

    fn fixture_root(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "envctl-agent-audit-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn write_fixture(root: &std::path::Path, second_mcp_owner: bool) -> PathBuf {
        let skill = root.join(".claude/skills/example");
        fs::create_dir_all(&skill).unwrap();
        fs::write(skill.join("SKILL.md"), "# Example\n").unwrap();
        fs::write(
            root.join(".mcp.json"),
            r#"{"mcpServers":{"exa":{"type":"http","url":"https://example.invalid"}}}"#,
        )
        .unwrap();
        let config = root.join("agent-env.yaml");
        let second = if second_mcp_owner {
            "  - source: ./other\n    mcps: [exa]\n"
        } else {
            ""
        };
        fs::write(
            &config,
            format!(
                "scope: project\nagent: claude-code\nskills:\n  - source: ./pack\n    skills: [example]\nmcps:\n  - source: ./pack\n    mcps: [exa]\n{second}"
            ),
        )
        .unwrap();

        let mut lock = AgentLockFile::default();
        lock.skills.insert(
            skill_key("./pack", "example"),
            AgentLockEntry {
                destination: ".claude/skills/example".into(),
                hash: hash_dir(&skill).unwrap(),
                skill: "example".into(),
                description: String::new(),
                source: "./pack".into(),
                source_revision: "local".into(),
                scope: Some(Scope::Project),
            },
        );
        lock.assets.insert(
            mcp_asset_id("./pack", "exa.json"),
            AssetEntry {
                kind: "mcp".into(),
                name: "exa.json".into(),
                hash: "source-pack-hash".into(),
                source: "./pack".into(),
                destination: "exa".into(),
                source_revision: "local".into(),
            },
        );
        if second_mcp_owner {
            lock.assets.insert(
                mcp_asset_id("./other", "exa.json"),
                AssetEntry {
                    kind: "mcp".into(),
                    name: "exa.json".into(),
                    hash: "other-source-pack-hash".into(),
                    source: "./other".into(),
                    destination: "exa".into(),
                    source_revision: "local".into(),
                },
            );
        }
        save(&mut lock, &root.join("agent-env.lock")).unwrap();
        config
    }

    fn run_audit(config: &std::path::Path) -> AgentAuditReport {
        Engine::detached()
            .agent_audit(
                AgentAuditSpec {
                    config_path: Some(config.to_string_lossy().to_string()),
                    scope_override: None,
                },
                &EventSink::null(),
            )
            .unwrap()
    }

    #[test]
    fn audit_confirms_locked_skill_hash_and_mcp_owner() {
        let root = fixture_root("healthy");
        let config = write_fixture(&root, false);
        let report = run_audit(&config);
        assert!(
            report.is_healthy(),
            "unexpected issues: {:?}",
            report.issues
        );
        assert_eq!(report.skills.len(), 1);
        assert!(report.skills[0]
            .targets
            .iter()
            .all(|target| target.matches_lock));
        assert!(report.commands.is_empty());
        assert_eq!(report.mcps[0].owners, vec!["./pack"]);
        assert!(report.mcps[0].targets.iter().all(|target| target.present));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn audit_fails_closed_on_installed_skill_hash_drift() {
        let root = fixture_root("hash-drift");
        let config = write_fixture(&root, false);
        fs::write(root.join(".claude/skills/example/SKILL.md"), "# Changed\n").unwrap();
        let report = run_audit(&config);
        assert!(!report.is_healthy());
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.kind == "skill_hash_mismatch"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn audit_reports_multiple_mcp_owners_as_a_conflict() {
        let root = fixture_root("mcp-conflict");
        let config = write_fixture(&root, true);
        let report = run_audit(&config);
        assert!(!report.is_healthy());
        assert_eq!(report.mcps[0].owners, vec!["./other", "./pack"]);
        assert!(report.mcps[0].conflict);
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.kind == "mcp_ownership_conflict"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn audit_fails_when_one_explicit_config_asset_is_missing_from_lock() {
        let root = fixture_root("missing-explicit-asset");
        let config = write_fixture(&root, false);
        fs::write(
            &config,
            "scope: project\nagent: claude-code\nskills:\n  - source: ./pack\n    skills: [example, missing]\nmcps:\n  - source: ./pack\n    mcps: [exa]\n",
        )
        .unwrap();
        let report = run_audit(&config);
        assert!(report.issues.iter().any(|issue| {
            issue.kind == "config_asset_missing_from_lock"
                && issue.id == skill_key("./pack", "missing")
        }));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn audit_fails_when_lock_contains_an_unselected_asset() {
        let root = fixture_root("unselected-lock-asset");
        let config = write_fixture(&root, false);
        let lock_path = root.join("agent-env.lock");
        let mut lock = load(&lock_path).unwrap();
        lock.skills.insert(
            skill_key("./pack", "extra"),
            AgentLockEntry {
                destination: ".claude/skills/extra".into(),
                hash: "unselected-hash".into(),
                skill: "extra".into(),
                description: String::new(),
                source: "./pack".into(),
                source_revision: "local".into(),
                scope: Some(Scope::Project),
            },
        );
        save(&mut lock, &lock_path).unwrap();
        let report = run_audit(&config);
        assert!(report.issues.iter().any(|issue| {
            issue.kind == "lock_asset_not_in_config" && issue.id == skill_key("./pack", "extra")
        }));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn audit_reports_command_targets_and_missing_command_files() {
        let root = fixture_root("command-target");
        fs::create_dir_all(root.join(".claude/commands")).unwrap();
        fs::write(root.join(".claude/commands/review.md"), "# Review\n").unwrap();
        let config = root.join("agent-env.yaml");
        fs::write(
            &config,
            "scope: project\nagent: claude-code\ncommands:\n  - source: ./pack\n    commands: [review]\n",
        )
        .unwrap();
        let mut lock = AgentLockFile::default();
        lock.assets.insert(
            command_asset_id("./pack", "review"),
            AssetEntry {
                kind: "command".into(),
                name: "review".into(),
                hash: "source-command-hash".into(),
                source: "./pack".into(),
                destination: ".claude/commands/review.md".into(),
                source_revision: "local".into(),
            },
        );
        save(&mut lock, &root.join("agent-env.lock")).unwrap();

        let healthy = run_audit(&config);
        assert!(healthy.is_healthy(), "issues: {:?}", healthy.issues);
        assert_eq!(healthy.commands.len(), 1);
        assert!(healthy.commands[0]
            .targets
            .iter()
            .all(|target| target.present));

        fs::remove_file(root.join(".claude/commands/review.md")).unwrap();
        let missing = run_audit(&config);
        assert!(missing
            .issues
            .iter()
            .any(|issue| issue.kind == "managed_command_missing"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn audit_rejects_an_unsupported_lock_schema() {
        let root = fixture_root("lock-version");
        let config = write_fixture(&root, false);
        let lock_path = root.join("agent-env.lock");
        let text = fs::read_to_string(&lock_path).unwrap();
        fs::write(
            &lock_path,
            text.replacen(
                &format!("version: {LOCK_VERSION}"),
                &format!("version: {}", LOCK_VERSION + 1),
                1,
            ),
        )
        .unwrap();
        let report = run_audit(&config);
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.kind == "unsupported_lock_version"));
        let _ = fs::remove_dir_all(root);
    }
}
