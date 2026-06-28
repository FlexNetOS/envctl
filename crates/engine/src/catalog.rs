//! Read-only catalog tables for ADR-0003.
//!
//! The catalog layer absorbs envctl's current files into normalized, queryable
//! rows. This first slice is intentionally in-memory and non-mutating: TOML,
//! YAML, JSON, Rust registries, and `.handoff` exports remain the accepted inputs
//! while later slices add diff/render/sync/DB-first behavior.

use crate::component::{Component, Hook, Phase};
use crate::layout::{LayoutKind, MetaLayout};
use crate::lock::{self, LockFile};
use crate::model::Registry;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Input roots for a read-only catalog import.
#[derive(Clone, Debug)]
pub struct CatalogScanSpec {
    pub repo_root: PathBuf,
    pub manifest_dir: PathBuf,
}

/// Stable table names exposed by `envctl catalog table <name>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogTableName {
    Components,
    ComponentHooks,
    Paths,
    Settings,
    EnvVars,
    AgentAssets,
    Registries,
    ConfigFiles,
    MigrationEvidence,
    ObservedFacts,
}

impl CatalogTableName {
    pub const fn canonical_name(self) -> &'static str {
        match self {
            CatalogTableName::Components => "components",
            CatalogTableName::ComponentHooks => "component_hooks",
            CatalogTableName::Paths => "paths",
            CatalogTableName::Settings => "settings",
            CatalogTableName::EnvVars => "env_vars",
            CatalogTableName::AgentAssets => "agent_assets",
            CatalogTableName::Registries => "registries",
            CatalogTableName::ConfigFiles => "config_files",
            CatalogTableName::MigrationEvidence => "migration_evidence",
            CatalogTableName::ObservedFacts => "observed_facts",
        }
    }

    pub const fn all() -> &'static [CatalogTableName] {
        &[
            CatalogTableName::Components,
            CatalogTableName::ComponentHooks,
            CatalogTableName::Paths,
            CatalogTableName::Settings,
            CatalogTableName::EnvVars,
            CatalogTableName::AgentAssets,
            CatalogTableName::Registries,
            CatalogTableName::ConfigFiles,
            CatalogTableName::MigrationEvidence,
            CatalogTableName::ObservedFacts,
        ]
    }
}

impl fmt::Display for CatalogTableName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.canonical_name())
    }
}

impl FromStr for CatalogTableName {
    type Err = String;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let normalized = raw.trim().replace('-', "_");
        match normalized.as_str() {
            "components" => Ok(CatalogTableName::Components),
            "component_hooks" => Ok(CatalogTableName::ComponentHooks),
            "paths" => Ok(CatalogTableName::Paths),
            "settings" => Ok(CatalogTableName::Settings),
            "env_vars" => Ok(CatalogTableName::EnvVars),
            "agent_assets" => Ok(CatalogTableName::AgentAssets),
            "registries" => Ok(CatalogTableName::Registries),
            "config_files" => Ok(CatalogTableName::ConfigFiles),
            "migration_evidence" | "migration_candidates" => {
                Ok(CatalogTableName::MigrationEvidence)
            }
            "observed_facts" => Ok(CatalogTableName::ObservedFacts),
            _ => Err(format!(
                "unknown catalog table '{raw}' (expected one of: {})",
                CatalogTableName::all()
                    .iter()
                    .map(|t| t.canonical_name())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }
}

/// One normalized read-only catalog snapshot.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CatalogSnapshot {
    pub repo_root: String,
    pub manifest_dir: String,
    pub generated_by: String,
    pub components: Vec<ComponentRow>,
    pub component_hooks: Vec<ComponentHookRow>,
    pub paths: Vec<PathRow>,
    pub settings: Vec<SettingRow>,
    pub env_vars: Vec<EnvVarRow>,
    pub agent_assets: Vec<AgentAssetRow>,
    pub registries: Vec<RegistryRow>,
    pub config_files: Vec<ConfigFileRow>,
    pub migration_evidence: Vec<MigrationEvidenceRow>,
    pub observed_facts: Vec<ObservedFactRow>,
}

impl CatalogSnapshot {
    /// Return a table as a JSON array while preserving the typed rows internally.
    pub fn table_value(&self, table: CatalogTableName) -> serde_json::Value {
        match table {
            CatalogTableName::Components => serde_json::to_value(&self.components),
            CatalogTableName::ComponentHooks => serde_json::to_value(&self.component_hooks),
            CatalogTableName::Paths => serde_json::to_value(&self.paths),
            CatalogTableName::Settings => serde_json::to_value(&self.settings),
            CatalogTableName::EnvVars => serde_json::to_value(&self.env_vars),
            CatalogTableName::AgentAssets => serde_json::to_value(&self.agent_assets),
            CatalogTableName::Registries => serde_json::to_value(&self.registries),
            CatalogTableName::ConfigFiles => serde_json::to_value(&self.config_files),
            CatalogTableName::MigrationEvidence => serde_json::to_value(&self.migration_evidence),
            CatalogTableName::ObservedFacts => serde_json::to_value(&self.observed_facts),
        }
        .unwrap_or_else(|_| serde_json::Value::Array(Vec::new()))
    }

    pub fn table_count(&self, table: CatalogTableName) -> usize {
        match table {
            CatalogTableName::Components => self.components.len(),
            CatalogTableName::ComponentHooks => self.component_hooks.len(),
            CatalogTableName::Paths => self.paths.len(),
            CatalogTableName::Settings => self.settings.len(),
            CatalogTableName::EnvVars => self.env_vars.len(),
            CatalogTableName::AgentAssets => self.agent_assets.len(),
            CatalogTableName::Registries => self.registries.len(),
            CatalogTableName::ConfigFiles => self.config_files.len(),
            CatalogTableName::MigrationEvidence => self.migration_evidence.len(),
            CatalogTableName::ObservedFacts => self.observed_facts.len(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComponentRow {
    pub component_id: String,
    pub name: String,
    pub source_file: String,
    pub description: String,
    pub requires: Vec<String>,
    pub gpu_required: bool,
    pub destructive: bool,
    pub has_detect: bool,
    pub has_install: bool,
    pub has_verify: bool,
    pub has_fix: bool,
    pub has_remove: bool,
    pub status: String,
    pub lock_hash: String,
    pub resolved_order: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComponentHookRow {
    pub component_id: String,
    pub phase: String,
    pub hook_kind: String,
    pub command: Option<String>,
    pub script: Option<String>,
    pub path: Option<String>,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub needs_sudo: bool,
    pub login_shell: bool,
    pub source_file: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PathRow {
    pub path_id: String,
    pub path: String,
    pub path_kind: String,
    pub owner_component: Option<String>,
    pub artifact_kind: String,
    pub canonical: bool,
    pub legacy: bool,
    pub bridge: bool,
    pub protected: bool,
    pub source: String,
    pub verification_status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SettingRow {
    pub setting_key: String,
    pub value: String,
    pub source_file: String,
    pub source_kind: String,
    pub owner_component: Option<String>,
    pub scope: String,
    pub precedence: u32,
    pub sensitive: bool,
    pub generated: bool,
    pub manual_override: bool,
    pub override_reason: Option<String>,
    pub override_owner: Option<String>,
    pub override_timestamp: Option<String>,
    pub expires_at: Option<String>,
    pub review_required: bool,
    pub generated_conflict_policy: String,
    pub drift_status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnvVarRow {
    pub var_name: String,
    pub value: Option<String>,
    pub producer: String,
    pub consumer: Option<String>,
    pub scope: String,
    pub sensitive: bool,
    pub default_value: Option<String>,
    pub effective_value: Option<String>,
    pub source: String,
    pub generated_by: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentAssetRow {
    pub asset_kind: String,
    pub name: String,
    pub source: String,
    pub destination: Option<String>,
    pub hash: String,
    pub source_revision: Option<String>,
    pub lock_status: String,
    pub drift_status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryRow {
    pub registry_kind: String,
    pub entry_id: String,
    pub name: String,
    pub component_id: Option<String>,
    pub status: String,
    pub tier: Option<String>,
    pub source_file: String,
    pub drift_status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigFileRow {
    pub config_id: String,
    pub path: String,
    pub file_kind: String,
    pub format: String,
    pub owner_component: Option<String>,
    pub source_role: String,
    pub generated: bool,
    pub manual_override: bool,
    pub lock_hash: Option<String>,
    pub exists: bool,
    pub read_status: String,
    pub parse_status: String,
    pub drift_status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MigrationEvidenceRow {
    pub component_id: Option<String>,
    pub artifact_kind: String,
    pub legacy_path: Option<String>,
    pub canonical_path: Option<String>,
    pub before_checksum: Option<String>,
    pub after_checksum: Option<String>,
    pub before_version: Option<String>,
    pub after_version: Option<String>,
    pub verifier: Option<String>,
    pub verifier_status: String,
    pub activation_status: String,
    pub quarantine_path: Option<String>,
    pub rollback_plan: Option<String>,
    pub purge_eligible: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObservedFactRow {
    pub fact_id: String,
    pub subject_kind: String,
    pub subject_id: String,
    pub fact_kind: String,
    pub value: String,
    pub source: String,
    pub observed_at: String,
    pub verifier: String,
    pub status: String,
}

/// Read the current repo files into normalized catalog rows. This function is
/// deliberately read-only: it never writes locks, renders files, or mutates
/// state.
pub fn scan(spec: CatalogScanSpec, registry: &Registry) -> anyhow::Result<CatalogSnapshot> {
    let repo_root = spec.repo_root;
    let manifest_dir = spec.manifest_dir;
    let observed_at = chrono::Utc::now().to_rfc3339();
    let component_sources = component_source_files(&repo_root, &manifest_dir)?;
    let lock = LockFile::load(&manifest_dir).unwrap_or_default();

    let mut snapshot = CatalogSnapshot {
        repo_root: repo_root.display().to_string(),
        manifest_dir: manifest_dir.display().to_string(),
        generated_by: "envctl catalog scan".to_string(),
        ..CatalogSnapshot::default()
    };

    let sources = discover_control_plane_files(&repo_root, &manifest_dir);
    let mut seen_config_paths = BTreeSet::new();
    for path in sources {
        if !seen_config_paths.insert(path.clone()) {
            continue;
        }
        let row = config_file_row(&repo_root, &manifest_dir, &path);
        record_file_observation(&mut snapshot.observed_facts, &row, &observed_at);
        if row.exists && row.read_status == "ok" {
            ingest_config_settings(&repo_root, &path, &row, &mut snapshot.settings);
            ingest_agent_assets_from_config(&repo_root, &path, &row, &mut snapshot.agent_assets);
            ingest_registries_from_file(
                &repo_root,
                &path,
                &row,
                registry,
                &mut snapshot.registries,
            );
        }
        snapshot.config_files.push(row);
    }

    ingest_components(
        registry,
        &component_sources,
        &lock,
        &mut snapshot.components,
        &mut snapshot.component_hooks,
        &mut snapshot.env_vars,
    );
    ingest_layout_paths(&repo_root, &mut snapshot.paths, &mut snapshot.env_vars);
    ingest_env_schema_vars(&repo_root, &mut snapshot.env_vars);
    ingest_agent_files(&repo_root, &mut snapshot.agent_assets);
    snapshot.observed_facts.push(ObservedFactRow {
        fact_id: "catalog.table_count.components".to_string(),
        subject_kind: "catalog_table".to_string(),
        subject_id: "components".to_string(),
        fact_kind: "row_count".to_string(),
        value: snapshot.components.len().to_string(),
        source: "envctl catalog scan".to_string(),
        observed_at: observed_at.clone(),
        verifier: "catalog_import".to_string(),
        status: "ok".to_string(),
    });
    for table in CatalogTableName::all() {
        if *table == CatalogTableName::Components {
            continue;
        }
        snapshot.observed_facts.push(ObservedFactRow {
            fact_id: format!("catalog.table_count.{}", table.canonical_name()),
            subject_kind: "catalog_table".to_string(),
            subject_id: table.canonical_name().to_string(),
            fact_kind: "row_count".to_string(),
            value: snapshot.table_count(*table).to_string(),
            source: "envctl catalog scan".to_string(),
            observed_at: observed_at.clone(),
            verifier: "catalog_import".to_string(),
            status: "ok".to_string(),
        });
    }

    snapshot.components.sort_by_key(|row| row.resolved_order);
    snapshot.component_hooks.sort_by(|a, b| {
        a.component_id
            .cmp(&b.component_id)
            .then(phase_rank(&a.phase).cmp(&phase_rank(&b.phase)))
    });
    snapshot.paths.sort_by(|a, b| a.path_id.cmp(&b.path_id));
    snapshot.settings.sort_by(|a, b| {
        a.source_file
            .cmp(&b.source_file)
            .then(a.setting_key.cmp(&b.setting_key))
    });
    snapshot.env_vars.sort_by(|a, b| {
        a.var_name
            .cmp(&b.var_name)
            .then(a.source.cmp(&b.source))
            .then(a.producer.cmp(&b.producer))
    });
    snapshot.agent_assets.sort_by(|a, b| {
        a.asset_kind
            .cmp(&b.asset_kind)
            .then(a.name.cmp(&b.name))
            .then(a.source.cmp(&b.source))
    });
    snapshot.registries.sort_by(|a, b| {
        a.registry_kind
            .cmp(&b.registry_kind)
            .then(a.entry_id.cmp(&b.entry_id))
            .then(a.source_file.cmp(&b.source_file))
    });
    snapshot.config_files.sort_by(|a, b| a.path.cmp(&b.path));
    snapshot
        .observed_facts
        .sort_by(|a, b| a.fact_id.cmp(&b.fact_id).then(a.source.cmp(&b.source)));

    Ok(snapshot)
}

fn ingest_components(
    registry: &Registry,
    component_sources: &BTreeMap<String, String>,
    lock: &LockFile,
    components: &mut Vec<ComponentRow>,
    hooks: &mut Vec<ComponentHookRow>,
    env_vars: &mut Vec<EnvVarRow>,
) {
    for (idx, component) in registry.ordered().enumerate() {
        let source_file = component_sources
            .get(&component.id)
            .cloned()
            .unwrap_or_else(|| "manifest/**/*.toml".to_string());
        let lock_hash = lock
            .components
            .get(&component.id)
            .map(|entry| entry.content_hash.clone())
            .filter(|hash| !hash.is_empty())
            .unwrap_or_else(|| lock::component_hash(component));
        components.push(ComponentRow {
            component_id: component.id.clone(),
            name: component.name.clone(),
            source_file: source_file.clone(),
            description: component.description.clone(),
            requires: component.requires.clone(),
            gpu_required: component.gpu_required,
            destructive: component.destructive,
            has_detect: component.detect.is_some(),
            has_install: component.install.is_some(),
            has_verify: component.verify.is_some(),
            has_fix: component.fix.is_some(),
            has_remove: component.remove.is_some(),
            status: "declared".to_string(),
            lock_hash,
            resolved_order: idx + 1,
        });
        for phase in [
            Phase::Detect,
            Phase::Install,
            Phase::Verify,
            Phase::Fix,
            Phase::Remove,
        ] {
            if let Some(hook) = component.hook(phase) {
                let row = hook_row(component, phase, hook, source_file.clone());
                for (name, value) in &row.env {
                    let sensitive = is_sensitive_key(name);
                    let redacted = if sensitive {
                        Some("<redacted>".to_string())
                    } else {
                        Some(value.clone())
                    };
                    env_vars.push(EnvVarRow {
                        var_name: name.clone(),
                        value: redacted.clone(),
                        producer: component.id.clone(),
                        consumer: Some(format!("component.{}.{}", component.id, row.phase)),
                        scope: "hook".to_string(),
                        sensitive,
                        default_value: None,
                        effective_value: redacted,
                        source: source_file.clone(),
                        generated_by: None,
                    });
                }
                hooks.push(row);
            }
        }
    }
}

fn hook_row(
    component: &Component,
    phase: Phase,
    hook: &Hook,
    source_file: String,
) -> ComponentHookRow {
    let phase = phase_name(phase).to_string();
    match hook {
        Hook::Command {
            command,
            args,
            env,
            needs_sudo,
        } => ComponentHookRow {
            component_id: component.id.clone(),
            phase,
            hook_kind: "command".to_string(),
            command: Some(command.clone()),
            script: None,
            path: None,
            args: args.clone(),
            env: redacted_env(env),
            needs_sudo: *needs_sudo,
            login_shell: false,
            source_file,
        },
        Hook::Script {
            script,
            path,
            env,
            needs_sudo,
            login_shell,
        } => ComponentHookRow {
            component_id: component.id.clone(),
            phase,
            hook_kind: "script".to_string(),
            command: None,
            script: if script.is_empty() {
                None
            } else {
                Some(script.clone())
            },
            path: path.clone(),
            args: Vec::new(),
            env: redacted_env(env),
            needs_sudo: *needs_sudo,
            login_shell: *login_shell,
            source_file,
        },
        Hook::ShippedScript {
            path,
            args,
            needs_sudo,
        } => ComponentHookRow {
            component_id: component.id.clone(),
            phase,
            hook_kind: "shipped_script".to_string(),
            command: None,
            script: None,
            path: Some(path.clone()),
            args: args.clone(),
            env: BTreeMap::new(),
            needs_sudo: *needs_sudo,
            login_shell: false,
            source_file,
        },
    }
}

fn ingest_layout_paths(repo_root: &Path, paths: &mut Vec<PathRow>, env_vars: &mut Vec<EnvVarRow>) {
    let layout = MetaLayout::from_meta_root(repo_root);
    for entry in layout.entries() {
        let canonical = entry.kind == LayoutKind::Canonical;
        let legacy = !canonical;
        paths.push(PathRow {
            path_id: entry.key.to_string(),
            path: entry.path.display().to_string(),
            path_kind: layout_path_kind(entry.key, entry.purpose).to_string(),
            owner_component: None,
            artifact_kind: entry.purpose.to_string(),
            canonical,
            legacy,
            bridge: legacy || entry.key.contains("bridge") || entry.key.contains("legacy"),
            protected: is_protected_layout_key(entry.key),
            source: "crates/engine/src/layout.rs".to_string(),
            verification_status: "not_checked".to_string(),
        });
    }

    for (var, value) in layout.env_exports() {
        env_vars.push(EnvVarRow {
            var_name: var.to_string(),
            value: Some(value.display().to_string()),
            producer: "layout".to_string(),
            consumer: Some("shell".to_string()),
            scope: "layout".to_string(),
            sensitive: false,
            default_value: None,
            effective_value: Some(value.display().to_string()),
            source: "crates/engine/src/layout.rs".to_string(),
            generated_by: Some("envctl env".to_string()),
        });
    }
}

fn ingest_env_schema_vars(repo_root: &Path, env_vars: &mut Vec<EnvVarRow>) {
    let mut schema_files = Vec::new();
    collect_env_schema_files(repo_root, &mut schema_files);
    schema_files.sort();

    let mut seen = BTreeSet::new();
    for file in schema_files {
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        let source = repo_relative(repo_root, &file);
        for token in env_schema_tokens(&text) {
            if !seen.insert((source.clone(), token.clone())) {
                continue;
            }
            env_vars.push(EnvVarRow {
                var_name: token.clone(),
                value: None,
                producer: "secrets_env_schema".to_string(),
                consumer: Some(env_schema_consumer(&source).to_string()),
                scope: "schema".to_string(),
                sensitive: is_sensitive_key(&token),
                default_value: None,
                effective_value: None,
                source: source.clone(),
                generated_by: None,
            });
        }
    }
}

fn ingest_config_settings(
    repo_root: &Path,
    path: &Path,
    row: &ConfigFileRow,
    settings: &mut Vec<SettingRow>,
) {
    let Some(value) = parse_config_to_json(path, &row.format).ok().flatten() else {
        return;
    };
    let source_file = repo_relative(repo_root, path);
    let mut flattened = Vec::new();
    flatten_json(None, &value, &mut flattened);
    for (key, value) in flattened {
        let sensitive = is_sensitive_key(&key);
        settings.push(SettingRow {
            setting_key: key,
            value: if sensitive {
                "<redacted>".to_string()
            } else {
                value
            },
            source_file: source_file.clone(),
            source_kind: row.file_kind.clone(),
            owner_component: row.owner_component.clone(),
            scope: setting_scope(&row.file_kind).to_string(),
            precedence: setting_precedence(&row.file_kind),
            sensitive,
            generated: row.generated,
            manual_override: false,
            override_reason: None,
            override_owner: None,
            override_timestamp: None,
            expires_at: None,
            review_required: false,
            generated_conflict_policy: "import_preserve".to_string(),
            drift_status: "unknown".to_string(),
        });
    }
}

fn ingest_agent_assets_from_config(
    repo_root: &Path,
    path: &Path,
    row: &ConfigFileRow,
    assets: &mut Vec<AgentAssetRow>,
) {
    let rel = repo_relative(repo_root, path);
    if rel == ".mcp.json" {
        if let Ok(Some(value)) = parse_config_to_json(path, &row.format) {
            if let Some(servers) = value.get("mcpServers").and_then(|v| v.as_object()) {
                for name in servers.keys() {
                    assets.push(AgentAssetRow {
                        asset_kind: "mcp".to_string(),
                        name: name.clone(),
                        source: rel.clone(),
                        destination: Some(".mcp.json".to_string()),
                        hash: file_hash(path).unwrap_or_default(),
                        source_revision: None,
                        lock_status: "not_checked".to_string(),
                        drift_status: "unknown".to_string(),
                    });
                }
            }
        }
    }
    if rel == ".codex/config.toml" {
        if let Ok(Some(value)) = parse_config_to_json(path, &row.format) {
            if let Some(servers) = value.get("mcp_servers").and_then(|v| v.as_object()) {
                for name in servers.keys() {
                    assets.push(AgentAssetRow {
                        asset_kind: "mcp".to_string(),
                        name: name.clone(),
                        source: rel.clone(),
                        destination: Some(".codex/config.toml".to_string()),
                        hash: file_hash(path).unwrap_or_default(),
                        source_revision: None,
                        lock_status: "not_checked".to_string(),
                        drift_status: "unknown".to_string(),
                    });
                }
            }
        }
    }
}

fn ingest_agent_files(repo_root: &Path, assets: &mut Vec<AgentAssetRow>) {
    for (asset_kind, dir, filename) in [
        ("skill", ".agents/skills", "SKILL.md"),
        ("skill", ".Codex/skills", "SKILL.md"),
        ("agent", ".Codex/agents", ""),
        ("hook", ".Codex/hooks", ""),
    ] {
        let root = repo_root.join(dir);
        if !root.exists() {
            continue;
        }
        if filename.is_empty() {
            for file in direct_files(&root) {
                let Some(name) = file.file_stem().and_then(|s| s.to_str()) else {
                    continue;
                };
                assets.push(AgentAssetRow {
                    asset_kind: asset_kind.to_string(),
                    name: name.to_string(),
                    source: repo_relative(repo_root, &file),
                    destination: Some(repo_relative(repo_root, &file)),
                    hash: file_hash(&file).unwrap_or_default(),
                    source_revision: None,
                    lock_status: "not_checked".to_string(),
                    drift_status: "unknown".to_string(),
                });
            }
        } else {
            for child in direct_dirs(&root) {
                let file = child.join(filename);
                if !file.is_file() {
                    continue;
                }
                let Some(name) = child.file_name().and_then(|s| s.to_str()) else {
                    continue;
                };
                assets.push(AgentAssetRow {
                    asset_kind: asset_kind.to_string(),
                    name: name.to_string(),
                    source: repo_relative(repo_root, &file),
                    destination: Some(repo_relative(repo_root, &file)),
                    hash: file_hash(&file).unwrap_or_default(),
                    source_revision: None,
                    lock_status: "not_checked".to_string(),
                    drift_status: "unknown".to_string(),
                });
            }
        }
    }
}

fn ingest_registries_from_file(
    repo_root: &Path,
    path: &Path,
    row: &ConfigFileRow,
    registry: &Registry,
    registries: &mut Vec<RegistryRow>,
) {
    let rel = repo_relative(repo_root, path);
    if rel.ends_with("registry.json") {
        if let Ok(Some(value)) = parse_config_to_json(path, &row.format) {
            let registry_kind = if rel.contains("_hub/") {
                "hub"
            } else {
                "registry"
            };
            let entries = registry_entries_from_json(&value);
            for entry in entries {
                let drift_status = entry
                    .component_id
                    .as_ref()
                    .map(|id| {
                        if registry.get(id).is_some() {
                            "unknown"
                        } else {
                            "missing_component"
                        }
                    })
                    .unwrap_or("unknown")
                    .to_string();
                registries.push(RegistryRow {
                    registry_kind: registry_kind.to_string(),
                    entry_id: entry.entry_id,
                    name: entry.name,
                    component_id: entry.component_id,
                    status: entry.status,
                    tier: entry.tier,
                    source_file: rel.clone(),
                    drift_status,
                });
            }
        }
    }

    if rel == ".mcp.json" {
        if let Ok(Some(value)) = parse_config_to_json(path, &row.format) {
            if let Some(servers) = value.get("mcpServers").and_then(|v| v.as_object()) {
                for name in servers.keys() {
                    registries.push(RegistryRow {
                        registry_kind: "mcp".to_string(),
                        entry_id: name.clone(),
                        name: name.clone(),
                        component_id: None,
                        status: "declared".to_string(),
                        tier: None,
                        source_file: rel.clone(),
                        drift_status: "unknown".to_string(),
                    });
                }
            }
        }
    }
    if rel == ".codex/config.toml" {
        if let Ok(Some(value)) = parse_config_to_json(path, &row.format) {
            if let Some(servers) = value.get("mcp_servers").and_then(|v| v.as_object()) {
                for name in servers.keys() {
                    registries.push(RegistryRow {
                        registry_kind: "mcp".to_string(),
                        entry_id: name.clone(),
                        name: name.clone(),
                        component_id: None,
                        status: "declared".to_string(),
                        tier: None,
                        source_file: rel.clone(),
                        drift_status: "unknown".to_string(),
                    });
                }
            }
        }
    }
}

#[derive(Debug)]
struct ParsedRegistryEntry {
    entry_id: String,
    name: String,
    component_id: Option<String>,
    status: String,
    tier: Option<String>,
}

fn registry_entries_from_json(value: &serde_json::Value) -> Vec<ParsedRegistryEntry> {
    let mut out = Vec::new();
    if let Some(entries) = value.get("entries") {
        match entries {
            serde_json::Value::Array(items) => {
                for item in items {
                    if let Some(obj) = item.as_object() {
                        let entry_id = obj
                            .get("id")
                            .or_else(|| obj.get("entry_id"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();
                        let name = obj
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&entry_id)
                            .to_string();
                        let component_id = obj
                            .get("component")
                            .or_else(|| obj.get("component_id"))
                            .and_then(|v| v.as_str())
                            .map(ToOwned::to_owned);
                        let status = obj
                            .get("status")
                            .and_then(json_scalar_string)
                            .unwrap_or_else(|| "declared".to_string());
                        let tier = obj.get("tier").and_then(json_scalar_string);
                        out.push(ParsedRegistryEntry {
                            entry_id,
                            name,
                            component_id,
                            status,
                            tier,
                        });
                    }
                }
            }
            serde_json::Value::Object(map) => {
                for (key, item) in map {
                    let obj = item.as_object();
                    let name = obj
                        .and_then(|o| o.get("name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or(key)
                        .to_string();
                    let component_id = obj
                        .and_then(|o| o.get("component").or_else(|| o.get("component_id")))
                        .and_then(|v| v.as_str())
                        .map(ToOwned::to_owned);
                    let status = obj
                        .and_then(|o| o.get("status"))
                        .and_then(json_scalar_string)
                        .unwrap_or_else(|| "declared".to_string());
                    let tier = obj.and_then(|o| o.get("tier")).and_then(json_scalar_string);
                    out.push(ParsedRegistryEntry {
                        entry_id: key.clone(),
                        name,
                        component_id,
                        status,
                        tier,
                    });
                }
            }
            _ => {}
        }
    }
    out
}

fn config_file_row(repo_root: &Path, manifest_dir: &Path, path: &Path) -> ConfigFileRow {
    let rel = repo_relative(repo_root, path);
    let exists = path.is_file();
    let format = infer_format(&rel).to_string();
    let file_kind = infer_file_kind(manifest_dir, path, &rel).to_string();
    let (read_status, lock_hash, parse_status) = if exists {
        match std::fs::read(path) {
            Ok(bytes) => {
                let hash = sha256_hex(&bytes);
                let parse_status = parse_status(path, &format);
                ("ok".to_string(), Some(hash), parse_status)
            }
            Err(err) => (format!("error:{err}"), None, "not_parsed".to_string()),
        }
    } else {
        ("missing".to_string(), None, "missing".to_string())
    };
    ConfigFileRow {
        config_id: config_id(&rel),
        path: rel.clone(),
        file_kind: file_kind.clone(),
        format,
        owner_component: owner_component_from_path(&rel),
        source_role: "import_source".to_string(),
        generated: rel.ends_with(".lock") || rel.ends_with("envctl.lock"),
        manual_override: false,
        lock_hash,
        exists,
        read_status,
        parse_status,
        drift_status: "unknown".to_string(),
    }
}

fn record_file_observation(
    facts: &mut Vec<ObservedFactRow>,
    row: &ConfigFileRow,
    observed_at: &str,
) {
    facts.push(ObservedFactRow {
        fact_id: format!("config_file.{}.exists", row.config_id),
        subject_kind: "config_file".to_string(),
        subject_id: row.config_id.clone(),
        fact_kind: "exists".to_string(),
        value: row.exists.to_string(),
        source: row.path.clone(),
        observed_at: observed_at.to_string(),
        verifier: "catalog_import".to_string(),
        status: if row.exists { "ok" } else { "missing" }.to_string(),
    });
    facts.push(ObservedFactRow {
        fact_id: format!("config_file.{}.parse_status", row.config_id),
        subject_kind: "config_file".to_string(),
        subject_id: row.config_id.clone(),
        fact_kind: "parse_status".to_string(),
        value: row.parse_status.clone(),
        source: row.path.clone(),
        observed_at: observed_at.to_string(),
        verifier: "catalog_import".to_string(),
        status: if row.parse_status == "ok" || row.parse_status == "not_parsed" {
            "ok"
        } else {
            "error"
        }
        .to_string(),
    });
}

fn discover_control_plane_files(repo_root: &Path, manifest_dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    collect_manifest_files(manifest_dir, &mut paths);
    push_if_present(&mut paths, manifest_dir.join(lock::LOCK_FILENAME));
    for rel in [
        "agent-env.yaml",
        "agent-env.lock",
        ".codex/config.toml",
        ".mcp.json",
        "crates/engine/src/layout.rs",
    ] {
        push_if_present(&mut paths, repo_root.join(rel));
    }
    collect_named_files(repo_root, "registry.json", &mut paths);
    collect_named_files(repo_root, "secretd.toml", &mut paths);
    collect_env_schema_files(repo_root, &mut paths);
    collect_matching_files(&repo_root.join(".handoff/tasks"), &["json"], &mut paths);
    collect_matching_files(&repo_root.join(".handoff"), &["jsonl"], &mut paths);
    collect_matching_files(&repo_root.join(".handoff/loop"), &["md"], &mut paths);
    collect_matching_files(&repo_root.join(".handoff/decisions"), &["md"], &mut paths);
    paths.sort();
    paths
}

fn collect_manifest_files(manifest_dir: &Path, out: &mut Vec<PathBuf>) {
    collect_matching_files(manifest_dir, &["toml"], out);
}

fn collect_named_files(root: &Path, name: &str, out: &mut Vec<PathBuf>) {
    walk_files(root, out, &mut |path| {
        path.file_name().and_then(|s| s.to_str()) == Some(name)
    });
}

fn collect_matching_files(root: &Path, exts: &[&str], out: &mut Vec<PathBuf>) {
    walk_files(root, out, &mut |path| {
        path.extension()
            .and_then(|s| s.to_str())
            .map(|ext| {
                exts.iter()
                    .any(|expected| ext.eq_ignore_ascii_case(expected))
            })
            .unwrap_or(false)
    });
}

fn collect_env_schema_files(repo_root: &Path, out: &mut Vec<PathBuf>) {
    for rel in [
        "crates/secretd/src",
        "crates/secrets-engine/src",
        "crates/secrets-proto/proto",
    ] {
        walk_files(&repo_root.join(rel), out, &mut |path| {
            let is_schema_surface = path
                .extension()
                .and_then(|s| s.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("rs") || ext.eq_ignore_ascii_case("proto"))
                .unwrap_or(false);
            if !is_schema_surface {
                return false;
            }
            std::fs::read_to_string(path)
                .map(|text| env_schema_tokens(&text).next().is_some())
                .unwrap_or(false)
        });
    }
}

fn walk_files(root: &Path, out: &mut Vec<PathBuf>, pred: &mut dyn FnMut(&Path) -> bool) {
    if !root.exists() {
        return;
    }
    if root.is_file() {
        if pred(root) {
            out.push(root.to_path_buf());
        }
        return;
    }
    let Ok(read_dir) = std::fs::read_dir(root) else {
        return;
    };
    let mut entries: Vec<PathBuf> = read_dir.flatten().map(|entry| entry.path()).collect();
    entries.sort();
    for path in entries {
        if should_skip_path(&path) {
            continue;
        }
        if path.is_dir() {
            walk_files(&path, out, pred);
        } else if pred(&path) {
            out.push(path);
        }
    }
}

fn should_skip_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|name| matches!(name, ".git" | "target" | ".worktrees" | "node_modules"))
        .unwrap_or(false)
}

fn push_if_present(out: &mut Vec<PathBuf>, path: PathBuf) {
    if path.exists() {
        out.push(path);
    }
}

fn component_source_files(
    repo_root: &Path,
    manifest_dir: &Path,
) -> anyhow::Result<BTreeMap<String, String>> {
    let mut sources = BTreeMap::new();
    let mut files = Vec::new();
    collect_manifest_files(manifest_dir, &mut files);
    files.sort();
    for file in files {
        let text = std::fs::read_to_string(&file)
            .with_context(|| format!("reading manifest {}", file.display()))?;
        let value: toml::Value = toml::from_str(&text)
            .with_context(|| format!("parsing manifest {}", file.display()))?;
        if let Some(components) = value.get("component").and_then(|v| v.as_array()) {
            for component in components {
                if let Some(id) = component.get("id").and_then(|v| v.as_str()) {
                    sources.insert(id.to_string(), repo_relative(repo_root, &file));
                }
            }
        }
    }
    Ok(sources)
}

fn parse_config_to_json(path: &Path, format: &str) -> anyhow::Result<Option<serde_json::Value>> {
    let text = std::fs::read_to_string(path)?;
    match format {
        "toml" => {
            let value: toml::Value = toml::from_str(&text)?;
            Ok(Some(serde_json::to_value(value)?))
        }
        "yaml" => {
            let value: serde_yaml::Value = serde_yaml::from_str(&text)?;
            Ok(Some(serde_json::to_value(value)?))
        }
        "json" => Ok(Some(serde_json::from_str(&text)?)),
        _ => Ok(None),
    }
}

fn parse_status(path: &Path, format: &str) -> String {
    match parse_config_to_json(path, format) {
        Ok(Some(_)) => "ok".to_string(),
        Ok(None) => "not_parsed".to_string(),
        Err(err) => format!("error:{err}"),
    }
}

fn flatten_json(
    prefix: Option<String>,
    value: &serde_json::Value,
    out: &mut Vec<(String, String)>,
) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                let next = prefix
                    .as_ref()
                    .map(|p| format!("{p}.{key}"))
                    .unwrap_or_else(|| key.clone());
                flatten_json(Some(next), child, out);
            }
        }
        serde_json::Value::Array(items) => {
            if items
                .iter()
                .all(|item| !item.is_object() && !item.is_array())
            {
                let key = prefix.unwrap_or_else(|| "value".to_string());
                out.push((key, json_value_string(value)));
            } else {
                for (idx, child) in items.iter().enumerate() {
                    let next = prefix
                        .as_ref()
                        .map(|p| format!("{p}[{idx}]"))
                        .unwrap_or_else(|| format!("[{idx}]"));
                    flatten_json(Some(next), child, out);
                }
            }
        }
        _ => {
            let key = prefix.unwrap_or_else(|| "value".to_string());
            out.push((key, json_value_string(value)));
        }
    }
}

fn json_value_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        _ => serde_json::to_string(value).unwrap_or_default(),
    }
}

fn json_scalar_string(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::Null => None,
        serde_json::Value::Bool(b) => Some(b.to_string()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::String(s) => Some(s.clone()),
        _ => None,
    }
}

fn redacted_env(env: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    env.iter()
        .map(|(key, value)| {
            (
                key.clone(),
                if is_sensitive_key(key) {
                    "<redacted>".to_string()
                } else {
                    value.clone()
                },
            )
        })
        .collect()
}

fn is_sensitive_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    [
        "secret",
        "token",
        "password",
        "passwd",
        "private_key",
        "credential",
        "apikey",
        "api_key",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn env_schema_tokens(text: &str) -> impl Iterator<Item = String> {
    let mut out = BTreeSet::new();
    let mut token = String::new();

    for ch in text.chars() {
        if ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_' {
            token.push(ch);
        } else if !token.is_empty() {
            if is_env_schema_token(&token) {
                out.insert(token.clone());
            }
            token.clear();
        }
    }
    if !token.is_empty() && is_env_schema_token(&token) {
        out.insert(token);
    }

    out.into_iter()
}

fn is_env_schema_token(token: &str) -> bool {
    if matches!(
        token,
        "HOME"
            | "PATH"
            | "RUST_LOG"
            | "META_ROOT"
            | "HTTPS_PROXY"
            | "HTTP_PROXY"
            | "NO_PROXY"
            | "SSL_CERT_FILE"
            | "SSL_CERT_DIR"
            | "REQUESTS_CA_BUNDLE"
            | "NODE_EXTRA_CA_CERTS"
    ) {
        return true;
    }
    if token.len() < 2 || !token.contains('_') || token.contains("_TEST_") {
        return false;
    }
    if token.starts_with("ENVCTL_")
        || token.starts_with("SECRETD_")
        || token.starts_with("ANTHROPIC_")
        || token.starts_with("OPENAI_")
        || token.starts_with("LLM_")
        || token.starts_with("XDG_")
    {
        return true;
    }
    [
        "_API_KEY",
        "_BASE_URL",
        "_TOKEN",
        "_TOKEN_FILE",
        "_CA",
        "_CA_PATH",
        "_PUBKEY",
        "_CONTEXT",
        "_PROXY",
        "_URL",
        "_BACKEND",
        "_CONFIG",
    ]
    .iter()
    .any(|suffix| token.ends_with(suffix))
}

fn env_schema_consumer(source: &str) -> &'static str {
    if source.starts_with("crates/secretd/") {
        "secretd"
    } else if source.starts_with("crates/secrets-engine/") {
        "secrets-engine"
    } else if source.starts_with("crates/secrets-proto/") {
        "secrets-proto"
    } else {
        "envctl"
    }
}

fn infer_format(rel: &str) -> &'static str {
    if rel.ends_with(".toml") || rel.ends_with(".lock") && rel.contains("envctl") {
        "toml"
    } else if rel.ends_with(".yaml") || rel.ends_with(".yml") || rel.ends_with("agent-env.lock") {
        "yaml"
    } else if rel.ends_with(".json") {
        "json"
    } else if rel.ends_with(".jsonl") {
        "jsonl"
    } else if rel.ends_with(".rs") {
        "rust"
    } else if rel.ends_with(".proto") {
        "proto"
    } else if rel.ends_with(".md") {
        "markdown"
    } else if rel.ends_with(".kdl") {
        "kdl"
    } else if rel.ends_with(".sh") || rel.ends_with(".bash") {
        "shell"
    } else {
        "unknown"
    }
}

fn infer_file_kind(manifest_dir: &Path, path: &Path, rel: &str) -> &'static str {
    if path.starts_with(manifest_dir) && rel.ends_with(".toml") {
        "manifest"
    } else if rel == "manifest/envctl.lock" {
        "envctl_lock"
    } else if rel == "agent-env.yaml" {
        "agent_env"
    } else if rel == "agent-env.lock" {
        "agent_env_lock"
    } else if rel == ".codex/config.toml" {
        "codex_config"
    } else if rel == ".mcp.json" {
        "mcp_config"
    } else if rel.ends_with("registry.json") {
        if rel.contains("_hub/") {
            "hub_registry"
        } else {
            "registry"
        }
    } else if rel == "crates/engine/src/layout.rs" {
        "path_registry"
    } else if rel.ends_with("secretd.toml") {
        "secretd_config"
    } else if rel.starts_with("crates/secrets-proto/proto/") && rel.ends_with(".proto") {
        "secrets_proto"
    } else if (rel.starts_with("crates/secretd/src/")
        || rel.starts_with("crates/secrets-engine/src/"))
        && rel.ends_with(".rs")
    {
        "secrets_env_schema"
    } else if rel.starts_with(".handoff/tasks/") {
        "handoff_task"
    } else if rel.starts_with(".handoff/") && rel.ends_with(".jsonl") {
        "handoff_ledger_export"
    } else if rel.starts_with(".handoff/") {
        "handoff_report"
    } else {
        "config"
    }
}

fn owner_component_from_path(rel: &str) -> Option<String> {
    if rel.starts_with("manifest/components.d/") && rel.ends_with(".toml") {
        rel.rsplit_once('/')
            .map(|(_, name)| name.trim_end_matches(".toml").to_string())
    } else if rel.starts_with("manifest/")
        && rel.ends_with(".toml")
        && rel != "manifest/envctl.lock"
    {
        rel.rsplit_once('/')
            .map(|(_, name)| name.trim_end_matches(".toml").to_string())
    } else if rel.contains("secretd") {
        Some("secretd".to_string())
    } else {
        None
    }
}

fn setting_scope(file_kind: &str) -> &'static str {
    match file_kind {
        "manifest" => "component",
        "envctl_lock" | "agent_env_lock" => "lock",
        "agent_env" => "agent_env",
        "codex_config" | "mcp_config" => "agent_runtime",
        "secretd_config" | "secrets_env_schema" | "secrets_proto" => "secrets",
        "handoff_task" | "handoff_ledger_export" | "handoff_report" => "handoff",
        _ => "workspace",
    }
}

fn setting_precedence(file_kind: &str) -> u32 {
    match file_kind {
        "codex_config" | "mcp_config" => 80,
        "agent_env" => 70,
        "manifest" => 60,
        "secretd_config" => 55,
        "secrets_env_schema" | "secrets_proto" => 45,
        "envctl_lock" | "agent_env_lock" => 50,
        "handoff_task" | "handoff_ledger_export" | "handoff_report" => 20,
        _ => 10,
    }
}

fn config_id(rel: &str) -> String {
    rel.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

fn phase_name(phase: Phase) -> &'static str {
    match phase {
        Phase::Detect => "detect",
        Phase::Install => "install",
        Phase::Verify => "verify",
        Phase::Fix => "fix",
        Phase::Remove => "remove",
    }
}

fn phase_rank(phase: &str) -> u8 {
    match phase {
        "detect" => 0,
        "install" => 1,
        "verify" => 2,
        "fix" => 3,
        "remove" => 4,
        _ => 9,
    }
}

fn layout_path_kind(key: &str, purpose: &str) -> &'static str {
    if key.contains("bin") || purpose.contains("executable") || purpose.contains("binaries") {
        "binary"
    } else if key.contains("lib") {
        "library"
    } else if key.contains("cache") {
        "cache"
    } else if key.contains("log") {
        "log"
    } else if key.contains("tmp") {
        "tmp"
    } else if key.contains("etc") || purpose.contains("configuration") {
        "config"
    } else if key.contains("state") || key.contains("var") || purpose.contains("persistent") {
        "state"
    } else if key.contains("toolchain") || key.contains("legacy") {
        "toolchain_root"
    } else {
        "path"
    }
}

fn is_protected_layout_key(key: &str) -> bool {
    key.contains("secrets")
        || key.contains("repo_store")
        || key == "state"
        || key == "var_lib_envctl"
}

fn repo_relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .trim_start_matches('/')
        .to_string()
}

fn file_hash(path: &Path) -> anyhow::Result<String> {
    let bytes = std::fs::read(path)?;
    Ok(sha256_hex(&bytes))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hash = hasher.finalize();
    hash.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn direct_dirs(root: &Path) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(root)
        .ok()
        .into_iter()
        .flat_map(|rd| rd.flatten())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    dirs.sort();
    dirs
}

fn direct_files(root: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(root)
        .ok()
        .into_iter()
        .flat_map(|rd| rd.flatten())
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect();
    files.sort();
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Registry;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn table_name_aliases_accept_kebab_case() {
        assert_eq!(
            "component-hooks".parse::<CatalogTableName>().unwrap(),
            CatalogTableName::ComponentHooks
        );
        assert_eq!(
            "env-vars".parse::<CatalogTableName>().unwrap(),
            CatalogTableName::EnvVars
        );
        assert_eq!(
            "migration-candidates".parse::<CatalogTableName>().unwrap(),
            CatalogTableName::MigrationEvidence
        );
        assert!("bogus".parse::<CatalogTableName>().is_err());
    }

    #[test]
    fn scan_normalizes_current_file_shapes_without_writes() {
        let root = fixture_root();
        write_fixture(&root);
        let manifest_dir = root.join("manifest");
        let registry = Registry::load(&manifest_dir).unwrap();

        let snapshot = scan(
            CatalogScanSpec {
                repo_root: root.clone(),
                manifest_dir,
            },
            &registry,
        )
        .unwrap();

        assert_eq!(snapshot.components.len(), 2, "{:#?}", snapshot.components);
        assert!(snapshot
            .components
            .iter()
            .any(|row| row.component_id == "base" && row.has_detect));
        assert!(snapshot
            .component_hooks
            .iter()
            .any(|row| row.component_id == "extra" && row.phase == "install"));
        assert!(snapshot.paths.iter().any(|row| row.path_id == "bin"));
        assert!(snapshot
            .config_files
            .iter()
            .any(|row| row.path == "agent-env.yaml" && row.file_kind == "agent_env"));
        assert!(snapshot
            .config_files
            .iter()
            .any(|row| row.path == "mcp_hub/registry.json" && row.file_kind == "hub_registry"));
        assert!(snapshot
            .config_files
            .iter()
            .any(|row| row.path == "crates/secrets-engine/src/seam.rs"
                && row.file_kind == "secrets_env_schema"));
        assert!(snapshot.settings.iter().any(
            |row| row.setting_key == "components[0].source" || row.setting_key == "sources[0]"
        ));
        assert!(snapshot.env_vars.iter().any(|row| {
            row.var_name == "API_TOKEN"
                && row.value.as_deref() == Some("<redacted>")
                && row.sensitive
        }));
        assert!(snapshot.env_vars.iter().any(|row| {
            row.var_name == "ENVCTL_SEED_TOKEN"
                && row.producer == "secrets_env_schema"
                && row.consumer.as_deref() == Some("secrets-engine")
                && row.scope == "schema"
                && row.sensitive
        }));
        assert!(snapshot
            .agent_assets
            .iter()
            .any(|row| row.asset_kind == "skill" && row.name == "demo"));
        assert!(snapshot
            .registries
            .iter()
            .any(|row| row.registry_kind == "hub" && row.entry_id == "demo-tool"));
        assert!(snapshot
            .observed_facts
            .iter()
            .any(|row| row.fact_id == "catalog.table_count.components"));

        let env_vars = snapshot.table_value(CatalogTableName::EnvVars);
        assert!(env_vars
            .as_array()
            .unwrap()
            .iter()
            .any(|row| { row.get("var_name").and_then(|v| v.as_str()) == Some("API_TOKEN") }));
    }

    fn fixture_root() -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("envctl-catalog-test-{id}"));
        let _ = std::fs::remove_dir_all(&root);
        root
    }

    fn write_fixture(root: &Path) {
        std::fs::create_dir_all(root.join("manifest/components.d")).unwrap();
        std::fs::create_dir_all(root.join(".codex")).unwrap();
        std::fs::create_dir_all(root.join(".agents/skills/demo")).unwrap();
        std::fs::create_dir_all(root.join(".Codex/agents")).unwrap();
        std::fs::create_dir_all(root.join(".Codex/hooks")).unwrap();
        std::fs::create_dir_all(root.join("mcp_hub")).unwrap();
        std::fs::create_dir_all(root.join(".handoff/tasks")).unwrap();
        std::fs::create_dir_all(root.join(".handoff/loop")).unwrap();
        std::fs::create_dir_all(root.join("crates/engine/src")).unwrap();
        std::fs::create_dir_all(root.join("crates/secrets-engine/src")).unwrap();

        std::fs::write(
            root.join("manifest/base.toml"),
            r#"
[[component]]
id = "base"
name = "Base"
description = "base component"

[component.detect]
kind = "command"
command = "true"
args = ["--version"]

[component.detect.env]
API_TOKEN = "super-secret"
"#,
        )
        .unwrap();
        std::fs::write(
            root.join("manifest/components.d/extra.toml"),
            r#"
[[component]]
id = "extra"
name = "Extra"
requires = ["base"]

[component.install]
kind = "script"
script = "echo install"
login_shell = false
"#,
        )
        .unwrap();
        std::fs::write(
            root.join("manifest/envctl.lock"),
            r#"
version = 1

[components.base]
content_hash = "locked-base"
requires = []
resolved = ""
"#,
        )
        .unwrap();
        std::fs::write(
            root.join("agent-env.yaml"),
            r#"
sources:
  - ./agent-skills
components:
  - source: ./agent-skills/demo
"#,
        )
        .unwrap();
        std::fs::write(root.join("agent-env.lock"), "assets: []\n").unwrap();
        std::fs::write(
            root.join(".codex/config.toml"),
            r#"
[mcp_servers.context7]
command = "context7"
"#,
        )
        .unwrap();
        std::fs::write(
            root.join(".mcp.json"),
            r#"{"mcpServers":{"github":{"command":"github"}}}"#,
        )
        .unwrap();
        std::fs::write(
            root.join("mcp_hub/registry.json"),
            r#"{"schema":"hub.registry.v1","entries":[{"id":"demo-tool","name":"Demo Tool","component":"base","status":"stable","tier":1}]}"#,
        )
        .unwrap();
        std::fs::write(root.join(".agents/skills/demo/SKILL.md"), "# demo\n").unwrap();
        std::fs::write(root.join(".Codex/agents/demo.md"), "# agent\n").unwrap();
        std::fs::write(root.join(".Codex/hooks/demo.sh"), "#!/bin/sh\n").unwrap();
        std::fs::write(
            root.join(".handoff/tasks/TASK-1.task.json"),
            r#"{"id":"TASK-1","status":"open"}"#,
        )
        .unwrap();
        std::fs::write(root.join(".handoff/loop/report.md"), "# report\n").unwrap();
        std::fs::write(
            root.join("crates/engine/src/layout.rs"),
            "// layout registry\n",
        )
        .unwrap();
        std::fs::write(
            root.join("crates/secrets-engine/src/seam.rs"),
            r#"
const ENVCTL_SEED_TOKEN: &str = "ENVCTL_SEED_TOKEN";
const ENVCTL_SEED_TOKEN_FILE: &str = "ENVCTL_SEED_TOKEN_FILE";
"#,
        )
        .unwrap();
    }
}
