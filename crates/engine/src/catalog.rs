//! Read-only catalog tables for ADR-0003.
//!
//! The catalog layer absorbs envctl's current files into normalized, queryable
//! rows. This first slice is intentionally in-memory and non-mutating: TOML,
//! YAML, JSON, Rust registries, and `.handoff` exports remain the accepted inputs
//! while later slices add DB-first behavior. Diff/render/import/sync remain
//! no-apply surfaces: they report drift and write generated projections only to
//! an explicit output directory outside the repo. The catalog lock path is the
//! first explicit apply surface and only updates `manifest/envctl.lock` when a
//! caller opts in.

use crate::component::{Component, Hook, Phase};
use crate::layout::{LayoutKind, MetaLayout};
use crate::lock::{self, LockFile};
use crate::model::Registry;
use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Write as _};
use std::io::ErrorKind;
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
    NixComponents,
    ComponentHooks,
    Paths,
    Settings,
    EnvVars,
    AgentAssets,
    Registries,
    ConfigFiles,
    CodedbFileImports,
    MigrationEvidence,
    ObservedFacts,
}

impl CatalogTableName {
    pub const fn canonical_name(self) -> &'static str {
        match self {
            CatalogTableName::Components => "components",
            CatalogTableName::NixComponents => "nix_components",
            CatalogTableName::ComponentHooks => "component_hooks",
            CatalogTableName::Paths => "paths",
            CatalogTableName::Settings => "settings",
            CatalogTableName::EnvVars => "env_vars",
            CatalogTableName::AgentAssets => "agent_assets",
            CatalogTableName::Registries => "registries",
            CatalogTableName::ConfigFiles => "config_files",
            CatalogTableName::CodedbFileImports => "codedb_file_imports",
            CatalogTableName::MigrationEvidence => "migration_evidence",
            CatalogTableName::ObservedFacts => "observed_facts",
        }
    }

    pub const fn all() -> &'static [CatalogTableName] {
        &[
            CatalogTableName::Components,
            CatalogTableName::NixComponents,
            CatalogTableName::ComponentHooks,
            CatalogTableName::Paths,
            CatalogTableName::Settings,
            CatalogTableName::EnvVars,
            CatalogTableName::AgentAssets,
            CatalogTableName::Registries,
            CatalogTableName::ConfigFiles,
            CatalogTableName::CodedbFileImports,
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
            "nix_components" => Ok(CatalogTableName::NixComponents),
            "component_hooks" => Ok(CatalogTableName::ComponentHooks),
            "paths" => Ok(CatalogTableName::Paths),
            "settings" => Ok(CatalogTableName::Settings),
            "env_vars" => Ok(CatalogTableName::EnvVars),
            "agent_assets" => Ok(CatalogTableName::AgentAssets),
            "registries" => Ok(CatalogTableName::Registries),
            "config_files" => Ok(CatalogTableName::ConfigFiles),
            "codedb_file_imports" | "envctl_yazelix_file_import" => {
                Ok(CatalogTableName::CodedbFileImports)
            }
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
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogSnapshot {
    pub repo_root: String,
    pub manifest_dir: String,
    pub generated_by: String,
    pub components: Vec<ComponentRow>,
    pub nix_components: Vec<NixComponentRow>,
    pub component_hooks: Vec<ComponentHookRow>,
    pub paths: Vec<PathRow>,
    pub settings: Vec<SettingRow>,
    pub env_vars: Vec<EnvVarRow>,
    pub agent_assets: Vec<AgentAssetRow>,
    pub registries: Vec<RegistryRow>,
    pub config_files: Vec<ConfigFileRow>,
    pub codedb_file_imports: Vec<CodedbFileImportRow>,
    pub migration_evidence: Vec<MigrationEvidenceRow>,
    pub observed_facts: Vec<ObservedFactRow>,
}

impl CatalogSnapshot {
    /// Return a table as a JSON array while preserving the typed rows internally.
    pub fn table_value(&self, table: CatalogTableName) -> serde_json::Value {
        match table {
            CatalogTableName::Components => serde_json::to_value(&self.components),
            CatalogTableName::NixComponents => serde_json::to_value(&self.nix_components),
            CatalogTableName::ComponentHooks => serde_json::to_value(&self.component_hooks),
            CatalogTableName::Paths => serde_json::to_value(&self.paths),
            CatalogTableName::Settings => serde_json::to_value(&self.settings),
            CatalogTableName::EnvVars => serde_json::to_value(&self.env_vars),
            CatalogTableName::AgentAssets => serde_json::to_value(&self.agent_assets),
            CatalogTableName::Registries => serde_json::to_value(&self.registries),
            CatalogTableName::ConfigFiles => serde_json::to_value(&self.config_files),
            CatalogTableName::CodedbFileImports => serde_json::to_value(&self.codedb_file_imports),
            CatalogTableName::MigrationEvidence => serde_json::to_value(&self.migration_evidence),
            CatalogTableName::ObservedFacts => serde_json::to_value(&self.observed_facts),
        }
        .unwrap_or_else(|_| serde_json::Value::Array(Vec::new()))
    }

    pub fn table_count(&self, table: CatalogTableName) -> usize {
        match table {
            CatalogTableName::Components => self.components.len(),
            CatalogTableName::NixComponents => self.nix_components.len(),
            CatalogTableName::ComponentHooks => self.component_hooks.len(),
            CatalogTableName::Paths => self.paths.len(),
            CatalogTableName::Settings => self.settings.len(),
            CatalogTableName::EnvVars => self.env_vars.len(),
            CatalogTableName::AgentAssets => self.agent_assets.len(),
            CatalogTableName::Registries => self.registries.len(),
            CatalogTableName::ConfigFiles => self.config_files.len(),
            CatalogTableName::CodedbFileImports => self.codedb_file_imports.len(),
            CatalogTableName::MigrationEvidence => self.migration_evidence.len(),
            CatalogTableName::ObservedFacts => self.observed_facts.len(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogTableSummaryRow {
    pub table: String,
    pub rows: usize,
    pub columns: Vec<String>,
    pub purpose: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogFacetCount {
    pub key: String,
    pub count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogToolchainSignalRow {
    pub signal_kind: String,
    pub key: String,
    pub value: String,
    pub source: String,
    pub detail: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogAnalyzeSummary {
    pub tables: usize,
    pub rows: usize,
    pub config_files: usize,
    pub env_vars: usize,
    pub toolchain_signals: usize,
    pub codedb_imports: usize,
    pub mutating: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogAnalyzeReport {
    pub repo_root: String,
    pub manifest_dir: String,
    pub generated_by: String,
    pub summary: CatalogAnalyzeSummary,
    pub table_inventory: Vec<CatalogTableSummaryRow>,
    pub config_formats: Vec<CatalogFacetCount>,
    pub config_file_kinds: Vec<CatalogFacetCount>,
    pub env_scopes: Vec<CatalogFacetCount>,
    pub env_producers: Vec<CatalogFacetCount>,
    pub env_sensitive: Vec<CatalogFacetCount>,
    pub path_artifact_kinds: Vec<CatalogFacetCount>,
    pub path_verification_statuses: Vec<CatalogFacetCount>,
    pub codedb_file_kinds: Vec<CatalogFacetCount>,
    pub codedb_parser_hints: Vec<CatalogFacetCount>,
    pub toolchain_signals: Vec<CatalogToolchainSignalRow>,
}

/// Read-only drift report over the current catalog import.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogDiffReport {
    pub repo_root: String,
    pub manifest_dir: String,
    pub generated_by: String,
    pub summary: CatalogDiffSummary,
    pub drift: Vec<CatalogDriftRow>,
    pub snapshot: CatalogSnapshot,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogDiffSummary {
    pub config_files: usize,
    pub components: usize,
    pub lock_drifts: usize,
    pub parse_errors: usize,
    pub read_errors: usize,
    pub missing_files: usize,
    pub registry_drifts: usize,
    pub drift_count: usize,
    pub mutating: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogDriftRow {
    pub drift_id: String,
    pub subject_kind: String,
    pub subject_id: String,
    pub drift_kind: String,
    pub source: String,
    pub desired: Option<String>,
    pub observed: Option<String>,
    pub severity: String,
    pub mutating: bool,
    pub verifier_status: String,
    pub details: String,
}

/// Render request for generated catalog projections.
#[derive(Clone, Debug)]
pub struct CatalogRenderSpec {
    pub repo_root: PathBuf,
    pub manifest_dir: PathBuf,
    pub out_dir: PathBuf,
    pub target_root: Option<PathBuf>,
}

/// Report for deterministic render projections written outside the repo.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogRenderReport {
    pub repo_root: String,
    pub manifest_dir: String,
    pub out_dir: String,
    pub target_root: Option<String>,
    pub generated_by: String,
    pub summary: CatalogRenderSummary,
    pub files: Vec<CatalogRenderedFile>,
    pub config_files: Vec<ConfigFileRow>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogRenderSummary {
    pub generated_files: usize,
    pub generated_config_rows: usize,
    pub bytes: usize,
    pub source_tables: Vec<String>,
    pub mutating_repo: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogRenderedFile {
    pub path: String,
    pub config_id: String,
    pub format: String,
    pub source_table: String,
    pub row_count: usize,
    pub bytes: usize,
    pub sha256: String,
    pub generated: bool,
    pub manual_edits_allowed: bool,
    pub provenance: String,
}

/// Report for `envctl catalog import`: files -> normalized rows, no writes.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogImportReport {
    pub repo_root: String,
    pub manifest_dir: String,
    pub generated_by: String,
    pub summary: CatalogImportSummary,
    pub snapshot: CatalogSnapshot,
}

pub fn table_inventory(snapshot: &CatalogSnapshot) -> Vec<CatalogTableSummaryRow> {
    CatalogTableName::all()
        .iter()
        .map(|table| CatalogTableSummaryRow {
            table: table.canonical_name().to_string(),
            rows: snapshot.table_count(*table),
            columns: table_columns(snapshot, *table),
            purpose: table_purpose(*table).to_string(),
        })
        .collect()
}

pub fn analyze_snapshot(snapshot: &CatalogSnapshot) -> CatalogAnalyzeReport {
    let table_inventory = table_inventory(snapshot);
    let config_formats = facet_counts(snapshot.config_files.iter().map(|row| row.format.as_str()));
    let config_file_kinds = facet_counts(
        snapshot
            .config_files
            .iter()
            .map(|row| row.file_kind.as_str()),
    );
    let env_scopes = facet_counts(snapshot.env_vars.iter().map(|row| row.scope.as_str()));
    let env_producers = facet_counts(snapshot.env_vars.iter().map(|row| row.producer.as_str()));
    let env_sensitive = vec![
        CatalogFacetCount {
            key: "sensitive".to_string(),
            count: snapshot.env_vars.iter().filter(|row| row.sensitive).count(),
        },
        CatalogFacetCount {
            key: "non_sensitive".to_string(),
            count: snapshot
                .env_vars
                .iter()
                .filter(|row| !row.sensitive)
                .count(),
        },
    ];
    let path_artifact_kinds =
        facet_counts(snapshot.paths.iter().map(|row| row.artifact_kind.as_str()));
    let path_verification_statuses = facet_counts(
        snapshot
            .paths
            .iter()
            .map(|row| row.verification_status.as_str()),
    );
    let codedb_file_kinds = facet_counts(
        snapshot
            .codedb_file_imports
            .iter()
            .map(|row| row.file_kind.as_str()),
    );
    let codedb_parser_hints = facet_counts(
        snapshot
            .codedb_file_imports
            .iter()
            .map(|row| row.parser_hint.as_str()),
    );
    let toolchain_signals = toolchain_signals(snapshot);
    let summary = CatalogAnalyzeSummary {
        tables: table_inventory.len(),
        rows: catalog_total_rows(snapshot),
        config_files: snapshot.config_files.len(),
        env_vars: snapshot.env_vars.len(),
        toolchain_signals: toolchain_signals.len(),
        codedb_imports: snapshot.codedb_file_imports.len(),
        mutating: false,
    };

    CatalogAnalyzeReport {
        repo_root: snapshot.repo_root.clone(),
        manifest_dir: snapshot.manifest_dir.clone(),
        generated_by: "envctl catalog analyze".to_string(),
        summary,
        table_inventory,
        config_formats,
        config_file_kinds,
        env_scopes,
        env_producers,
        env_sensitive,
        path_artifact_kinds,
        path_verification_statuses,
        codedb_file_kinds,
        codedb_parser_hints,
        toolchain_signals,
    }
}

pub fn analyze_current(
    spec: CatalogScanSpec,
    registry: &Registry,
) -> anyhow::Result<CatalogAnalyzeReport> {
    let snapshot = scan(spec, registry)?;
    Ok(analyze_snapshot(&snapshot))
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogImportSummary {
    pub tables: usize,
    pub rows: usize,
    pub components: usize,
    pub config_files: usize,
    pub settings: usize,
    pub env_vars: usize,
    pub mutating: bool,
}

/// Bidirectional reconcile request. `apply` is intentionally refused until the
/// verifier-gated row mutation path lands; preview + optional out-of-repo render
/// are the safe ADR-0003 stepping stones.
#[derive(Clone, Debug)]
pub struct CatalogSyncSpec {
    pub repo_root: PathBuf,
    pub manifest_dir: PathBuf,
    pub render_out_dir: Option<PathBuf>,
    pub apply: bool,
}

/// Preview report for `envctl catalog sync`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogSyncReport {
    pub repo_root: String,
    pub manifest_dir: String,
    pub generated_by: String,
    pub summary: CatalogSyncSummary,
    pub planned_actions: Vec<CatalogSyncAction>,
    pub diff: CatalogDiffReport,
    pub render: Option<CatalogRenderReport>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogSyncSummary {
    pub drift_count: usize,
    pub planned_actions: usize,
    pub rendered_files: usize,
    pub applied: bool,
    pub mutating: bool,
    pub verifier_status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogSyncAction {
    pub action_id: String,
    pub action_kind: String,
    pub subject_kind: String,
    pub subject_id: String,
    pub source: String,
    pub target: Option<String>,
    pub reason: String,
    pub apply_required: bool,
    pub verifier_status: String,
    pub mutating: bool,
}

/// Catalog-native lock request. `apply=false` is a read-only check; `apply=true`
/// writes only `manifest/envctl.lock` after regenerating it from the current
/// manifest registry.
#[derive(Clone, Debug)]
pub struct CatalogLockSpec {
    pub repo_root: PathBuf,
    pub manifest_dir: PathBuf,
    pub apply: bool,
}

/// Report for `envctl catalog lock`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogLockReport {
    pub repo_root: String,
    pub manifest_dir: String,
    pub generated_by: String,
    pub lock_path: String,
    pub before_sha256: Option<String>,
    pub after_sha256: Option<String>,
    pub summary: CatalogLockSummary,
    pub drift: Vec<CatalogDriftRow>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogLockSummary {
    pub components: usize,
    pub before_drifts: usize,
    pub after_drifts: usize,
    pub applied: bool,
    pub mutating: bool,
    pub lock_written: bool,
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
pub struct NixComponentRow {
    pub component_id: String,
    pub name: String,
    pub source_file: String,
    pub nix_surface: String,
    pub owner_component: Option<String>,
    pub profile_entry: Option<String>,
    pub original_url: Option<String>,
    pub profile_url: Option<String>,
    pub store_paths: Vec<String>,
    pub frontdoor_paths: Vec<String>,
    pub requires: Vec<String>,
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
    pub owner_record_id: Option<String>,
    pub artifact_kind: String,
    pub resolved_path: Option<String>,
    pub link_target_id: Option<String>,
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
pub struct CodedbFileImportRow {
    pub table: String,
    pub target_id: String,
    pub logical_owner: String,
    pub absolute_path: String,
    pub normalized_path: String,
    pub source_of_truth_class: String,
    pub file_kind: String,
    pub parser_hint: String,
    pub content_hash: Option<String>,
    pub byte_length: u64,
    pub blob_ref: Option<String>,
    pub import_safety_policy: String,
    pub reproduction_policy: String,
    pub import_mode: String,
    pub import_status: String,
    pub skip_reason: String,
    pub structured_table: String,
    pub structured_status: String,
    pub structured_row_count: usize,
    pub structured_rows: Vec<CodedbStructuredFileRow>,
    pub last_observed: String,
    pub provenance: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodedbStructuredFileRow {
    pub row_index: usize,
    pub row_kind: String,
    pub format: String,
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct YazelixFileInventoryRow {
    #[serde(default)]
    target_id: String,
    #[serde(default)]
    absolute_path: String,
    #[serde(default, alias = "normalized_path")]
    normalized_logical_path: String,
    #[serde(default, alias = "logical_owner")]
    owner: String,
    #[serde(default)]
    source_of_truth_class: String,
    #[serde(default)]
    file_kind: String,
    #[serde(default)]
    parser_hint: String,
    #[serde(default)]
    reproduction_policy: String,
    #[serde(default, alias = "import_safety_policy")]
    safety_policy: String,
    #[serde(default)]
    import_mode: String,
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
        &mut snapshot.nix_components,
        &mut snapshot.component_hooks,
        &mut snapshot.env_vars,
    );
    ingest_live_nix_profile(
        &repo_root,
        &mut snapshot.nix_components,
        &mut snapshot.paths,
    );
    ingest_envctl_home_frontdoors(&repo_root, &mut snapshot.paths);
    ingest_layout_paths(&repo_root, &mut snapshot.paths, &mut snapshot.env_vars);
    ingest_env_schema_vars(&repo_root, &mut snapshot.env_vars);
    ingest_agent_files(&repo_root, &mut snapshot.agent_assets);
    ingest_codedb_file_imports(&repo_root, &observed_at, &mut snapshot.codedb_file_imports)?;
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
    snapshot
        .nix_components
        .sort_by_key(|row| row.resolved_order);
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
    snapshot.codedb_file_imports.sort_by(|a, b| {
        a.source_of_truth_class
            .cmp(&b.source_of_truth_class)
            .then(a.normalized_path.cmp(&b.normalized_path))
            .then(a.target_id.cmp(&b.target_id))
    });
    snapshot
        .observed_facts
        .sort_by(|a, b| a.fact_id.cmp(&b.fact_id).then(a.source.cmp(&b.source)));

    Ok(snapshot)
}

/// Compare current files/catalog/lock state without mutating the repo.
pub fn diff(spec: CatalogScanSpec, registry: &Registry) -> anyhow::Result<CatalogDiffReport> {
    let repo_root = spec.repo_root.clone();
    let manifest_dir = spec.manifest_dir.clone();
    let snapshot = scan(spec.clone(), registry)?;
    let mut drift = Vec::new();

    for row in &snapshot.config_files {
        if !row.exists {
            push_drift(
                &mut drift,
                DriftInput {
                    subject_kind: "config_file",
                    subject_id: &row.path,
                    drift_kind: "missing_file",
                    source: &row.path,
                    desired: Some("exists".to_string()),
                    observed: Some("missing".to_string()),
                    severity: "error",
                    verifier_status: "catalog_file_probe",
                    details: "configured catalog source was not present on disk",
                },
            );
        }
        if row.read_status != "ok" {
            push_drift(
                &mut drift,
                DriftInput {
                    subject_kind: "config_file",
                    subject_id: &row.path,
                    drift_kind: "read_error",
                    source: &row.path,
                    desired: Some("readable".to_string()),
                    observed: Some(row.read_status.clone()),
                    severity: "error",
                    verifier_status: "catalog_file_probe",
                    details: "catalog source could not be read",
                },
            );
        }
        if row.parse_status.starts_with("error:") {
            push_drift(
                &mut drift,
                DriftInput {
                    subject_kind: "config_file",
                    subject_id: &row.path,
                    drift_kind: "parse_error",
                    source: &row.path,
                    desired: Some("parse ok".to_string()),
                    observed: Some(row.parse_status.clone()),
                    severity: "error",
                    verifier_status: "catalog_parser",
                    details: "catalog source parser rejected the current file",
                },
            );
        }
    }

    let lock_path = lock::lock_path(&manifest_dir);
    match LockFile::load(&manifest_dir) {
        Ok(lock_file) => {
            let generated = lock::generate(registry);
            for (component_id, kind) in lock::diff(registry, &lock_file) {
                let current = generated.components.get(&component_id);
                let locked = lock_file.components.get(&component_id);
                let (desired, observed, details) = match kind {
                    lock::LockDriftKind::Added => (
                        current.map(|entry| entry.content_hash.clone()),
                        None,
                        "component exists in manifest but is absent from envctl.lock",
                    ),
                    lock::LockDriftKind::Removed => (
                        None,
                        locked.map(|entry| entry.content_hash.clone()),
                        "component exists in envctl.lock but is absent from manifest",
                    ),
                    lock::LockDriftKind::Changed => (
                        current.map(|entry| entry.content_hash.clone()),
                        locked.map(|entry| entry.content_hash.clone()),
                        "component manifest hash differs from envctl.lock",
                    ),
                };
                push_drift(
                    &mut drift,
                    DriftInput {
                        subject_kind: "component",
                        subject_id: &component_id,
                        drift_kind: &format!("lock_{}", lock_drift_kind_name(kind)),
                        source: &repo_relative(&repo_root, &lock_path),
                        desired,
                        observed,
                        severity: "error",
                        verifier_status: "envctl_lock_diff",
                        details,
                    },
                );
            }
        }
        Err(err) => {
            push_drift(
                &mut drift,
                DriftInput {
                    subject_kind: "lock_file",
                    subject_id: &repo_relative(&repo_root, &lock_path),
                    drift_kind: "lock_read_error",
                    source: &repo_relative(&repo_root, &lock_path),
                    desired: Some("lock parse ok".to_string()),
                    observed: Some(err.to_string()),
                    severity: "error",
                    verifier_status: "envctl_lock_diff",
                    details: "envctl.lock could not be loaded for drift comparison",
                },
            );
        }
    }

    for row in &snapshot.registries {
        if row.drift_status != "unknown" && row.drift_status != "ok" {
            push_drift(
                &mut drift,
                DriftInput {
                    subject_kind: "registry",
                    subject_id: &row.entry_id,
                    drift_kind: &row.drift_status,
                    source: &row.source_file,
                    desired: Some("linked".to_string()),
                    observed: row.component_id.clone(),
                    severity: "warn",
                    verifier_status: "catalog_registry_probe",
                    details: "registry row is not cleanly linked to a known component",
                },
            );
        }
    }
    for fact in &snapshot.observed_facts {
        if fact.status != "ok" {
            push_drift(
                &mut drift,
                DriftInput {
                    subject_kind: "observed_fact",
                    subject_id: &fact.fact_id,
                    drift_kind: "observed_fact_not_ok",
                    source: &fact.source,
                    desired: Some("ok".to_string()),
                    observed: Some(fact.status.clone()),
                    severity: "warn",
                    verifier_status: &fact.verifier,
                    details: "observed state did not match the desired catalog expectation",
                },
            );
        }
    }

    drift.sort_by(|a, b| {
        a.subject_kind
            .cmp(&b.subject_kind)
            .then(a.subject_id.cmp(&b.subject_id))
            .then(a.drift_kind.cmp(&b.drift_kind))
            .then(a.source.cmp(&b.source))
    });
    renumber_drift_ids(&mut drift);

    let summary = CatalogDiffSummary {
        config_files: snapshot.config_files.len(),
        components: snapshot.components.len(),
        lock_drifts: drift
            .iter()
            .filter(|row| row.drift_kind.starts_with("lock_"))
            .count(),
        parse_errors: drift
            .iter()
            .filter(|row| row.drift_kind == "parse_error")
            .count(),
        read_errors: drift
            .iter()
            .filter(|row| row.drift_kind == "read_error")
            .count(),
        missing_files: drift
            .iter()
            .filter(|row| row.drift_kind == "missing_file")
            .count(),
        registry_drifts: drift
            .iter()
            .filter(|row| row.subject_kind == "registry")
            .count(),
        drift_count: drift.len(),
        mutating: false,
    };

    Ok(CatalogDiffReport {
        repo_root: snapshot.repo_root.clone(),
        manifest_dir: snapshot.manifest_dir.clone(),
        generated_by: "envctl catalog diff".to_string(),
        summary,
        drift,
        snapshot,
    })
}

/// Render deterministic catalog projections into an explicit output directory.
///
/// This intentionally refuses output paths inside the source repo. ADR-0003's
/// first render slice must be comparable without applying anything to the live
/// system.
pub fn render(spec: CatalogRenderSpec, registry: &Registry) -> anyhow::Result<CatalogRenderReport> {
    let repo_root = absolute_existing_path(&spec.repo_root)?;
    let manifest_dir = absolute_optional_path(&spec.manifest_dir)?;
    let planned_out_dir = absolute_target_path(&spec.out_dir)?;
    let target_root = spec
        .target_root
        .as_ref()
        .map(|path| absolute_target_path(path.as_path()))
        .transpose()?;
    if planned_out_dir == repo_root || planned_out_dir.starts_with(&repo_root) {
        bail!(
            "catalog render output must be outside repo root (out={}, repo={})",
            planned_out_dir.display(),
            repo_root.display()
        );
    }

    std::fs::create_dir_all(&planned_out_dir).with_context(|| {
        format!(
            "creating catalog render output directory {}",
            planned_out_dir.display()
        )
    })?;
    let out_dir = planned_out_dir.canonicalize().with_context(|| {
        format!(
            "canonicalizing catalog render output directory {}",
            planned_out_dir.display()
        )
    })?;

    let snapshot = stable_snapshot_for_render(
        scan(
            CatalogScanSpec {
                repo_root: repo_root.clone(),
                manifest_dir: manifest_dir.clone(),
            },
            registry,
        )?,
        target_root.as_deref(),
    );

    let mut projections = render_projections(&snapshot)?;
    let mut generated_config_rows = projections
        .iter()
        .map(|projection| {
            generated_config_file_row(projection, Some(sha256_hex(&projection.bytes)))
        })
        .collect::<Vec<_>>();
    generated_config_rows.push(ConfigFileRow {
        config_id: config_id("catalog/rendered-config-files.json"),
        path: "catalog/rendered-config-files.json".to_string(),
        file_kind: "generated_projection".to_string(),
        format: "json".to_string(),
        owner_component: None,
        source_role: "generated_projection:config_files".to_string(),
        generated: true,
        manual_override: false,
        lock_hash: None,
        exists: true,
        read_status: "ok".to_string(),
        parse_status: "ok".to_string(),
        drift_status: "rendered".to_string(),
    });
    generated_config_rows.sort_by(|a, b| a.path.cmp(&b.path));
    projections.push(RenderProjection::new(
        "catalog/rendered-config-files.json",
        "config_files",
        generated_config_rows.len(),
        false,
        json_with_trailing_newline(&generated_config_rows)?,
        "Generated by envctl catalog render. Source table: config_files. Manual edits allowed: no.",
    ));
    projections.sort_by(|a, b| a.path.cmp(&b.path));

    let mut rendered_files = Vec::new();
    for projection in &projections {
        write_projection(&out_dir, projection)?;
        rendered_files.push(CatalogRenderedFile {
            path: projection.path.clone(),
            config_id: config_id(&projection.path),
            format: infer_format(&projection.path).to_string(),
            source_table: projection.source_table.clone(),
            row_count: projection.row_count,
            bytes: projection.bytes.len(),
            sha256: sha256_hex(&projection.bytes),
            generated: true,
            manual_edits_allowed: projection.manual_edits_allowed,
            provenance: projection.provenance.clone(),
        });
    }

    let source_tables = CatalogTableName::all()
        .iter()
        .map(|table| table.canonical_name().to_string())
        .collect::<Vec<_>>();
    let summary = CatalogRenderSummary {
        generated_files: rendered_files.len(),
        generated_config_rows: generated_config_rows.len(),
        bytes: rendered_files.iter().map(|row| row.bytes).sum(),
        source_tables,
        mutating_repo: false,
    };

    Ok(CatalogRenderReport {
        repo_root: repo_root.display().to_string(),
        manifest_dir: manifest_dir.display().to_string(),
        out_dir: out_dir.display().to_string(),
        target_root: target_root.map(|path| path.display().to_string()),
        generated_by: "envctl catalog render".to_string(),
        summary,
        files: rendered_files,
        config_files: generated_config_rows,
    })
}

/// Import current files into normalized catalog tables without writing anything.
pub fn import_current(
    spec: CatalogScanSpec,
    registry: &Registry,
) -> anyhow::Result<CatalogImportReport> {
    let snapshot = scan(spec, registry)?;
    let summary = CatalogImportSummary {
        tables: CatalogTableName::all().len(),
        rows: catalog_total_rows(&snapshot),
        components: snapshot.components.len(),
        config_files: snapshot.config_files.len(),
        settings: snapshot.settings.len(),
        env_vars: snapshot.env_vars.len(),
        mutating: false,
    };

    Ok(CatalogImportReport {
        repo_root: snapshot.repo_root.clone(),
        manifest_dir: snapshot.manifest_dir.clone(),
        generated_by: "envctl catalog import".to_string(),
        summary,
        snapshot,
    })
}

fn table_columns(snapshot: &CatalogSnapshot, table: CatalogTableName) -> Vec<String> {
    let value = snapshot.table_value(table);
    let mut columns = BTreeSet::new();
    if let Some(rows) = value.as_array() {
        for row in rows {
            if let Some(object) = row.as_object() {
                columns.extend(object.keys().cloned());
            }
        }
    }
    columns.into_iter().collect()
}

fn table_purpose(table: CatalogTableName) -> &'static str {
    match table {
        CatalogTableName::Components => "component registry rows and lifecycle intent",
        CatalogTableName::NixComponents => "nix-native inventory rows and frontdoor ownership",
        CatalogTableName::ComponentHooks => "detect/install/fix/reset hook wiring",
        CatalogTableName::Paths => "canonical, legacy, and bridged filesystem targets",
        CatalogTableName::Settings => "normalized config/settings key-value rows",
        CatalogTableName::EnvVars => "environment variables with producer and scope metadata",
        CatalogTableName::AgentAssets => "skills, agents, hooks, and lock-tracked assets",
        CatalogTableName::Registries => "hub and MCP registry entries",
        CatalogTableName::ConfigFiles => "source and generated config file inventory",
        CatalogTableName::CodedbFileImports => {
            "blob/structured import rows for file-backed code DB coverage"
        }
        CatalogTableName::MigrationEvidence => "adoption and purge-safety evidence",
        CatalogTableName::ObservedFacts => "runtime observations and verifier-produced facts",
    }
}

fn facet_counts<'a>(values: impl Iterator<Item = &'a str>) -> Vec<CatalogFacetCount> {
    let mut counts = BTreeMap::<String, usize>::new();
    for value in values {
        let key = if value.trim().is_empty() {
            "unknown".to_string()
        } else {
            value.to_string()
        };
        *counts.entry(key).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(key, count)| CatalogFacetCount { key, count })
        .collect()
}

fn toolchain_signals(snapshot: &CatalogSnapshot) -> Vec<CatalogToolchainSignalRow> {
    let mut rows = Vec::new();

    for row in &snapshot.env_vars {
        if let Some(value) = row
            .effective_value
            .as_deref()
            .or(row.value.as_deref())
            .or(row.default_value.as_deref())
        {
            if is_toolchain_token(&row.var_name) || is_toolchain_token(value) {
                rows.push(CatalogToolchainSignalRow {
                    signal_kind: "env_var".to_string(),
                    key: row.var_name.clone(),
                    value: value.to_string(),
                    source: row.source.clone(),
                    detail: format!(
                        "producer={} scope={} sensitive={}",
                        row.producer, row.scope, row.sensitive
                    ),
                });
            }
        }
    }

    for row in &snapshot.settings {
        if is_toolchain_token(&row.setting_key) || is_toolchain_token(&row.value) {
            rows.push(CatalogToolchainSignalRow {
                signal_kind: "setting".to_string(),
                key: row.setting_key.clone(),
                value: row.value.clone(),
                source: row.source_file.clone(),
                detail: format!("scope={} source_kind={}", row.scope, row.source_kind),
            });
        }
    }

    for row in &snapshot.paths {
        if is_toolchain_token(&row.path)
            || is_toolchain_token(&row.path_kind)
            || is_toolchain_token(&row.artifact_kind)
            || is_toolchain_token(&row.source)
        {
            rows.push(CatalogToolchainSignalRow {
                signal_kind: "path".to_string(),
                key: row.path_kind.clone(),
                value: row.path.clone(),
                source: row.source.clone(),
                detail: format!(
                    "artifact_kind={} canonical={} legacy={} bridge={} verification={}",
                    row.artifact_kind,
                    row.canonical,
                    row.legacy,
                    row.bridge,
                    row.verification_status
                ),
            });
        }
    }

    for row in &snapshot.codedb_file_imports {
        if is_toolchain_token(&row.normalized_path)
            || is_toolchain_token(&row.file_kind)
            || is_toolchain_token(&row.parser_hint)
        {
            rows.push(CatalogToolchainSignalRow {
                signal_kind: "codedb_import".to_string(),
                key: row.target_id.clone(),
                value: row.normalized_path.clone(),
                source: row.provenance.clone(),
                detail: format!(
                    "file_kind={} parser_hint={} structured_rows={}",
                    row.file_kind, row.parser_hint, row.structured_row_count
                ),
            });
        }
    }

    rows.sort_by(|a, b| {
        (&a.signal_kind, &a.key, &a.source, &a.value).cmp(&(
            &b.signal_kind,
            &b.key,
            &b.source,
            &b.value,
        ))
    });
    rows.dedup_by(|a, b| {
        a.signal_kind == b.signal_kind
            && a.key == b.key
            && a.value == b.value
            && a.source == b.source
            && a.detail == b.detail
    });
    rows
}

fn is_toolchain_token(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "cargo",
        "rust",
        "rustup",
        "toolchain",
        "linker",
        "wild",
        "kache",
        "sccache",
        "nix",
        "felix",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

/// Plan a bidirectional catalog sync without applying repository mutations.
///
/// The apply path is deliberately fail-closed until the verifier-gated row edit
/// engine lands. For now `sync` is the round-trip safety report that combines
/// import, diff, and optional out-of-repo render evidence.
pub fn sync(spec: CatalogSyncSpec, registry: &Registry) -> anyhow::Result<CatalogSyncReport> {
    if spec.apply {
        bail!(
            "catalog sync --apply requires verifier-gated row edit/apply support; \
             preview with `envctl catalog sync` and use `envctl catalog lock --apply` \
             for lock-only acceptance"
        );
    }

    let repo_root = spec.repo_root.clone();
    let manifest_dir = spec.manifest_dir.clone();
    let diff_report = diff(
        CatalogScanSpec {
            repo_root: repo_root.clone(),
            manifest_dir: manifest_dir.clone(),
        },
        registry,
    )?;

    let mut planned_actions = sync_actions_from_drift(&diff_report.drift);
    let render_report = if let Some(out_dir) = spec.render_out_dir {
        let report = render(
            CatalogRenderSpec {
                repo_root,
                manifest_dir,
                out_dir,
                target_root: None,
            },
            registry,
        )?;
        planned_actions.push(CatalogSyncAction {
            action_id: String::new(),
            action_kind: "review_render_projection".to_string(),
            subject_kind: "render".to_string(),
            subject_id: report.out_dir.clone(),
            source: "catalog_tables".to_string(),
            target: Some(report.out_dir.clone()),
            reason: "rendered generated-file projections for human/CI review".to_string(),
            apply_required: false,
            verifier_status: "catalog_render".to_string(),
            mutating: false,
        });
        Some(report)
    } else {
        None
    };
    renumber_sync_actions(&mut planned_actions);

    let summary = CatalogSyncSummary {
        drift_count: diff_report.summary.drift_count,
        planned_actions: planned_actions.len(),
        rendered_files: render_report
            .as_ref()
            .map(|report| report.summary.generated_files)
            .unwrap_or(0),
        applied: false,
        mutating: false,
        verifier_status: "preview_only".to_string(),
    };

    Ok(CatalogSyncReport {
        repo_root: diff_report.repo_root.clone(),
        manifest_dir: diff_report.manifest_dir.clone(),
        generated_by: "envctl catalog sync".to_string(),
        summary,
        planned_actions,
        diff: diff_report,
        render: render_report,
    })
}

/// Check or update the catalog lock projection (`manifest/envctl.lock`).
pub fn lock(spec: CatalogLockSpec, registry: &Registry) -> anyhow::Result<CatalogLockReport> {
    let repo_root = absolute_existing_path(&spec.repo_root)?;
    let manifest_dir = absolute_existing_path(&spec.manifest_dir)?;
    let lock_path = lock::lock_path(&manifest_dir);
    let lock_rel = repo_relative(&repo_root, &lock_path);
    let before_sha256 = file_hash_optional(&lock_path)?;

    let before_lock = match LockFile::load(&manifest_dir) {
        Ok(lock_file) => lock_file,
        Err(err) => {
            let mut drift = Vec::new();
            push_drift(
                &mut drift,
                DriftInput {
                    subject_kind: "lock_file",
                    subject_id: &lock_rel,
                    drift_kind: "lock_read_error",
                    source: &lock_rel,
                    desired: Some("lock parse ok".to_string()),
                    observed: Some(err.to_string()),
                    severity: "error",
                    verifier_status: "envctl_lock_diff",
                    details: "envctl.lock could not be loaded; catalog lock will not overwrite unreadable lock files",
                },
            );
            renumber_drift_ids(&mut drift);
            let summary = CatalogLockSummary {
                components: registry.len(),
                before_drifts: drift.len(),
                after_drifts: drift.len(),
                applied: false,
                mutating: false,
                lock_written: false,
            };
            return Ok(CatalogLockReport {
                repo_root: repo_root.display().to_string(),
                manifest_dir: manifest_dir.display().to_string(),
                generated_by: "envctl catalog lock".to_string(),
                lock_path: lock_rel,
                before_sha256: before_sha256.clone(),
                after_sha256: before_sha256,
                summary,
                drift,
            });
        }
    };

    let before_drift = lock_drift_rows(&repo_root, &manifest_dir, registry, &before_lock);
    let mut lock_written = false;
    if spec.apply && !before_drift.is_empty() {
        let mut generated = lock::generate(registry);
        generated.save(&manifest_dir)?;
        lock_written = true;
    }

    let after_sha256 = file_hash_optional(&lock_path)?;
    let after_drifts = match LockFile::load(&manifest_dir) {
        Ok(after_lock) => lock::diff(registry, &after_lock).len(),
        Err(_) => before_drift.len(),
    };
    let summary = CatalogLockSummary {
        components: registry.len(),
        before_drifts: before_drift.len(),
        after_drifts,
        applied: lock_written,
        mutating: lock_written,
        lock_written,
    };

    Ok(CatalogLockReport {
        repo_root: repo_root.display().to_string(),
        manifest_dir: manifest_dir.display().to_string(),
        generated_by: "envctl catalog lock".to_string(),
        lock_path: lock_rel,
        before_sha256,
        after_sha256,
        summary,
        drift: before_drift,
    })
}

struct DriftInput<'a> {
    subject_kind: &'a str,
    subject_id: &'a str,
    drift_kind: &'a str,
    source: &'a str,
    desired: Option<String>,
    observed: Option<String>,
    severity: &'a str,
    verifier_status: &'a str,
    details: &'a str,
}

fn push_drift(drift: &mut Vec<CatalogDriftRow>, input: DriftInput<'_>) {
    drift.push(CatalogDriftRow {
        drift_id: String::new(),
        subject_kind: input.subject_kind.to_string(),
        subject_id: input.subject_id.to_string(),
        drift_kind: input.drift_kind.to_string(),
        source: input.source.to_string(),
        desired: input.desired,
        observed: input.observed,
        severity: input.severity.to_string(),
        mutating: false,
        verifier_status: input.verifier_status.to_string(),
        details: input.details.to_string(),
    });
}

fn renumber_drift_ids(drift: &mut [CatalogDriftRow]) {
    for (idx, row) in drift.iter_mut().enumerate() {
        row.drift_id = format!("drift.{:04}", idx + 1);
    }
}

fn lock_drift_kind_name(kind: lock::LockDriftKind) -> &'static str {
    match kind {
        lock::LockDriftKind::Added => "added",
        lock::LockDriftKind::Removed => "removed",
        lock::LockDriftKind::Changed => "changed",
    }
}

fn lock_drift_rows(
    repo_root: &Path,
    manifest_dir: &Path,
    registry: &Registry,
    lock_file: &LockFile,
) -> Vec<CatalogDriftRow> {
    let lock_path = lock::lock_path(manifest_dir);
    let lock_rel = repo_relative(repo_root, &lock_path);
    let generated = lock::generate(registry);
    let mut drift = Vec::new();
    for (component_id, kind) in lock::diff(registry, lock_file) {
        let current = generated.components.get(&component_id);
        let locked = lock_file.components.get(&component_id);
        let (desired, observed, details) = match kind {
            lock::LockDriftKind::Added => (
                current.map(|entry| entry.content_hash.clone()),
                None,
                "component exists in manifest but is absent from envctl.lock",
            ),
            lock::LockDriftKind::Removed => (
                None,
                locked.map(|entry| entry.content_hash.clone()),
                "component exists in envctl.lock but is absent from manifest",
            ),
            lock::LockDriftKind::Changed => (
                current.map(|entry| entry.content_hash.clone()),
                locked.map(|entry| entry.content_hash.clone()),
                "component manifest hash differs from envctl.lock",
            ),
        };
        push_drift(
            &mut drift,
            DriftInput {
                subject_kind: "component",
                subject_id: &component_id,
                drift_kind: &format!("lock_{}", lock_drift_kind_name(kind)),
                source: &lock_rel,
                desired,
                observed,
                severity: "error",
                verifier_status: "envctl_lock_diff",
                details,
            },
        );
    }
    drift.sort_by(|a, b| {
        a.subject_kind
            .cmp(&b.subject_kind)
            .then(a.subject_id.cmp(&b.subject_id))
            .then(a.drift_kind.cmp(&b.drift_kind))
            .then(a.source.cmp(&b.source))
    });
    renumber_drift_ids(&mut drift);
    drift
}

fn sync_actions_from_drift(drift: &[CatalogDriftRow]) -> Vec<CatalogSyncAction> {
    let mut actions = drift
        .iter()
        .map(|row| {
            let (action_kind, target, reason) = if row.drift_kind.starts_with("lock_") {
                (
                    "catalog_lock_apply",
                    Some(row.source.clone()),
                    "accept current manifest hashes into envctl.lock with `envctl catalog lock --apply`",
                )
            } else if row.subject_kind == "config_file" {
                (
                    "manual_import_reconcile",
                    Some(row.source.clone()),
                    "fix or accept source-file change through catalog import before rendering",
                )
            } else if row.subject_kind == "registry" {
                (
                    "registry_link_verify",
                    Some(row.source.clone()),
                    "link registry entry to a known component or document the unresolved row",
                )
            } else if row.subject_kind == "observed_fact" {
                (
                    "verifier_follow_up",
                    None,
                    "re-run verifier or reconcile desired state against observed fact",
                )
            } else {
                (
                    "catalog_review",
                    Some(row.source.clone()),
                    "review catalog drift before any apply step",
                )
            };
            CatalogSyncAction {
                action_id: String::new(),
                action_kind: action_kind.to_string(),
                subject_kind: row.subject_kind.clone(),
                subject_id: row.subject_id.clone(),
                source: row.source.clone(),
                target,
                reason: reason.to_string(),
                apply_required: true,
                verifier_status: row.verifier_status.clone(),
                mutating: false,
            }
        })
        .collect::<Vec<_>>();
    actions.sort_by(|a, b| {
        a.action_kind
            .cmp(&b.action_kind)
            .then(a.subject_kind.cmp(&b.subject_kind))
            .then(a.subject_id.cmp(&b.subject_id))
            .then(a.source.cmp(&b.source))
    });
    actions
}

fn renumber_sync_actions(actions: &mut [CatalogSyncAction]) {
    for (idx, row) in actions.iter_mut().enumerate() {
        row.action_id = format!("sync.{:04}", idx + 1);
    }
}

#[derive(Clone, Debug)]
struct RenderProjection {
    path: String,
    source_table: String,
    row_count: usize,
    manual_edits_allowed: bool,
    bytes: Vec<u8>,
    provenance: String,
}

impl RenderProjection {
    fn new(
        path: impl Into<String>,
        source_table: impl Into<String>,
        row_count: usize,
        manual_edits_allowed: bool,
        bytes: impl Into<Vec<u8>>,
        provenance: impl Into<String>,
    ) -> Self {
        RenderProjection {
            path: path.into(),
            source_table: source_table.into(),
            row_count,
            manual_edits_allowed,
            bytes: bytes.into(),
            provenance: provenance.into(),
        }
    }
}

fn render_projections(snapshot: &CatalogSnapshot) -> anyhow::Result<Vec<RenderProjection>> {
    let mut projections = vec![
        RenderProjection::new(
            "catalog/scan.json",
            "all",
            catalog_total_rows(snapshot),
            false,
            json_with_trailing_newline(snapshot)?,
            "Generated by envctl catalog render. Source table: all. Manual edits allowed: no.",
        ),
        RenderProjection::new(
            "manifest/components.catalog.toml",
            "components",
            snapshot.components.len(),
            false,
            render_components_toml(snapshot).into_bytes(),
            "Generated by envctl catalog render. Source table: components. Manual edits allowed: no.",
        ),
        RenderProjection::new(
            "agent-env.yaml",
            "agent_assets+env_vars",
            snapshot.agent_assets.len() + snapshot.env_vars.len(),
            false,
            render_agent_env_yaml(snapshot)?,
            "Generated by envctl catalog render. Source tables: agent_assets, env_vars. Manual edits allowed: no.",
        ),
        RenderProjection::new(
            "agent-env.lock",
            "agent_assets",
            snapshot.agent_assets.len(),
            false,
            render_agent_env_lock_yaml(snapshot)?,
            "Generated by envctl catalog render. Source table: agent_assets. Manual edits allowed: no.",
        ),
        RenderProjection::new(
            ".codex/config.toml",
            "settings+registries+agent_assets",
            snapshot.settings.len() + snapshot.registries.len() + snapshot.agent_assets.len(),
            false,
            render_codex_config_toml(snapshot).into_bytes(),
            "Generated by envctl catalog render. Source tables: settings, registries, agent_assets. Manual edits allowed: no.",
        ),
        RenderProjection::new(
            ".mcp.json",
            "registries",
            snapshot
                .registries
                .iter()
                .filter(|row| row.registry_kind == "mcp")
                .count(),
            false,
            render_mcp_json(snapshot)?,
            "Generated by envctl catalog render. Source table: registries. Manual edits allowed: no.",
        ),
        RenderProjection::new(
            "shell/env.catalog.sh",
            "env_vars",
            snapshot.env_vars.len(),
            false,
            render_env_shell(snapshot).into_bytes(),
            "Generated by envctl catalog render. Source table: env_vars. Manual edits allowed: no.",
        ),
        RenderProjection::new(
            "dashboard/mission-control.catalog.kdl",
            "paths+settings",
            snapshot.paths.len() + snapshot.settings.len(),
            false,
            render_dashboard_kdl(snapshot).into_bytes(),
            "Generated by envctl catalog render. Source tables: paths, settings. Manual edits allowed: no.",
        ),
        RenderProjection::new(
            "systemd/user/envctl-catalog-check.service",
            "observed_facts",
            snapshot.observed_facts.len(),
            false,
            render_systemd_unit(snapshot).into_bytes(),
            "Generated by envctl catalog render. Source table: observed_facts. Manual edits allowed: no.",
        ),
    ];

    for table in CatalogTableName::all() {
        let value = snapshot.table_value(*table);
        projections.push(RenderProjection::new(
            format!("catalog/tables/{}.json", table.canonical_name()),
            table.canonical_name(),
            snapshot.table_count(*table),
            false,
            json_with_trailing_newline(&value)?,
            format!(
                "Generated by envctl catalog render. Source table: {}. Manual edits allowed: no.",
                table.canonical_name()
            ),
        ));
        projections.push(RenderProjection::new(
            format!("catalog/tables/{}.tsv", table.canonical_name()),
            table.canonical_name(),
            snapshot.table_count(*table),
            false,
            table_tsv(table.canonical_name(), &value)?.into_bytes(),
            format!(
                "Generated by envctl catalog render. Source table: {}. Manual edits allowed: no.",
                table.canonical_name()
            ),
        ));
    }
    Ok(projections)
}

fn stable_snapshot_for_render(
    mut snapshot: CatalogSnapshot,
    target_root: Option<&Path>,
) -> CatalogSnapshot {
    snapshot.generated_by = "envctl catalog render".to_string();
    for row in &mut snapshot.observed_facts {
        row.observed_at = "catalog_render".to_string();
    }
    if let Some(target_root) = target_root {
        retarget_layout_rows(target_root, &mut snapshot.paths, &mut snapshot.env_vars);
    }
    snapshot
}

fn retarget_layout_rows(target_root: &Path, paths: &mut [PathRow], env_vars: &mut [EnvVarRow]) {
    let layout = MetaLayout::from_meta_root(target_root);
    let path_map = layout
        .entries()
        .into_iter()
        .map(|entry| (entry.key.to_string(), entry.path.display().to_string()))
        .collect::<BTreeMap<_, _>>();
    for row in paths
        .iter_mut()
        .filter(|row| row.source == "crates/engine/src/layout.rs")
    {
        if let Some(path) = path_map.get(&row.path_id) {
            row.path = path.clone();
            row.verification_status = layout_path_verification_status(Path::new(path));
        }
    }

    let env_map = layout
        .env_exports()
        .into_iter()
        .map(|(var, value)| (var.to_string(), value.display().to_string()))
        .collect::<BTreeMap<_, _>>();
    for row in env_vars.iter_mut().filter(|row| {
        row.source == "crates/engine/src/layout.rs"
            && row.producer == "layout"
            && row.scope == "layout"
    }) {
        if let Some(value) = env_map.get(&row.var_name) {
            row.value = Some(value.clone());
            row.effective_value = Some(value.clone());
        }
    }
}

fn catalog_total_rows(snapshot: &CatalogSnapshot) -> usize {
    CatalogTableName::all()
        .iter()
        .map(|table| snapshot.table_count(*table))
        .sum()
}

fn generated_config_file_row(
    projection: &RenderProjection,
    lock_hash: Option<String>,
) -> ConfigFileRow {
    ConfigFileRow {
        config_id: config_id(&projection.path),
        path: projection.path.clone(),
        file_kind: "generated_projection".to_string(),
        format: infer_format(&projection.path).to_string(),
        owner_component: None,
        source_role: format!("generated_projection:{}", projection.source_table),
        generated: true,
        manual_override: projection.manual_edits_allowed,
        lock_hash,
        exists: true,
        read_status: "ok".to_string(),
        parse_status: "ok".to_string(),
        drift_status: "rendered".to_string(),
    }
}

fn write_projection(out_dir: &Path, projection: &RenderProjection) -> anyhow::Result<()> {
    let relative = safe_relative_render_path(&projection.path)?;
    let path = out_dir.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating render parent {}", parent.display()))?;
    }
    std::fs::write(&path, &projection.bytes)
        .with_context(|| format!("writing catalog render projection {}", path.display()))
}

fn safe_relative_render_path(path: &str) -> anyhow::Result<PathBuf> {
    let relative = PathBuf::from(path);
    if relative.is_absolute() {
        bail!("catalog render projection path must be relative: {path}");
    }
    if relative
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        bail!("catalog render projection path may not contain '..': {path}");
    }
    Ok(relative)
}

fn absolute_existing_path(path: &Path) -> anyhow::Result<PathBuf> {
    path.canonicalize()
        .with_context(|| format!("canonicalizing {}", path.display()))
}

fn absolute_optional_path(path: &Path) -> anyhow::Result<PathBuf> {
    if path.exists() {
        absolute_existing_path(path)
    } else if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()
            .context("resolving current directory for optional catalog path")?
            .join(path))
    }
}

fn absolute_target_path(path: &Path) -> anyhow::Result<PathBuf> {
    if path.exists() {
        return absolute_existing_path(path);
    }
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .context("resolving current directory for catalog render output")?
            .join(path)
    };
    let parent = absolute.parent().with_context(|| {
        format!(
            "catalog render output path has no parent directory: {}",
            absolute.display()
        )
    })?;
    let parent = parent.canonicalize().with_context(|| {
        format!(
            "canonicalizing parent output directory {}",
            parent.display()
        )
    })?;
    let file_name = absolute.file_name().with_context(|| {
        format!(
            "catalog render output path has no final directory name: {}",
            absolute.display()
        )
    })?;
    Ok(parent.join(file_name))
}

fn json_with_trailing_newline<T: Serialize>(value: &T) -> anyhow::Result<Vec<u8>> {
    let mut text = serde_json::to_string_pretty(value)?;
    text.push('\n');
    Ok(text.into_bytes())
}

fn table_tsv(table_name: &str, value: &serde_json::Value) -> anyhow::Result<String> {
    let rows = value
        .as_array()
        .with_context(|| format!("catalog table {table_name} did not serialize as an array"))?;
    let mut out = String::new();
    writeln!(
        &mut out,
        "# Generated by envctl catalog render. Source table: {table_name}. Manual edits allowed: no."
    )?;
    let mut columns = BTreeSet::new();
    for row in rows {
        if let Some(object) = row.as_object() {
            columns.extend(object.keys().cloned());
        }
    }
    let columns = columns.into_iter().collect::<Vec<_>>();
    writeln!(&mut out, "{}", columns.join("\t"))?;
    for row in rows {
        let object = row
            .as_object()
            .with_context(|| format!("catalog table {table_name} contained a non-object row"))?;
        let cells = columns
            .iter()
            .map(|column| object.get(column).map(catalog_cell).unwrap_or_default())
            .collect::<Vec<_>>();
        writeln!(&mut out, "{}", cells.join("\t"))?;
    }
    Ok(out)
}

fn catalog_cell(value: &serde_json::Value) -> String {
    let cell = match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            serde_json::to_string(value).unwrap_or_default()
        }
    };
    cell.replace(['\n', '\t'], " ")
}

fn render_components_toml(snapshot: &CatalogSnapshot) -> String {
    let mut out = String::new();
    out.push_str("# Generated by envctl catalog render.\n");
    out.push_str("# Source table: components, component_hooks.\n");
    out.push_str("# Manual edits allowed: no.\n\n");
    let mut hooks_by_component: BTreeMap<&str, Vec<&ComponentHookRow>> = BTreeMap::new();
    for hook in &snapshot.component_hooks {
        hooks_by_component
            .entry(hook.component_id.as_str())
            .or_default()
            .push(hook);
    }
    for component in &snapshot.components {
        out.push_str("[[component]]\n");
        write_toml_string(&mut out, "id", &component.component_id);
        write_toml_string(&mut out, "name", &component.name);
        write_toml_string(&mut out, "description", &component.description);
        write_toml_string_array(&mut out, "requires", &component.requires);
        let _ = writeln!(&mut out, "gpu_required = {}", component.gpu_required);
        let _ = writeln!(&mut out, "destructive = {}", component.destructive);
        write_toml_string(&mut out, "status", &component.status);
        write_toml_string(&mut out, "lock_hash", &component.lock_hash);
        let _ = writeln!(&mut out, "resolved_order = {}", component.resolved_order);
        if let Some(hooks) = hooks_by_component.get(component.component_id.as_str()) {
            for hook in hooks {
                out.push_str("\n[[component.hook]]\n");
                write_toml_string(&mut out, "phase", &hook.phase);
                write_toml_string(&mut out, "hook_kind", &hook.hook_kind);
                if let Some(command) = &hook.command {
                    write_toml_string(&mut out, "command", command);
                }
                if let Some(script) = &hook.script {
                    write_toml_string(&mut out, "script", script);
                }
                if let Some(path) = &hook.path {
                    write_toml_string(&mut out, "path", path);
                }
                write_toml_string_array(&mut out, "args", &hook.args);
                let _ = writeln!(&mut out, "needs_sudo = {}", hook.needs_sudo);
                let _ = writeln!(&mut out, "login_shell = {}", hook.login_shell);
            }
        }
        out.push('\n');
    }
    out
}

fn render_agent_env_yaml(snapshot: &CatalogSnapshot) -> anyhow::Result<Vec<u8>> {
    let value = serde_json::json!({
        "generated_by": "envctl catalog render",
        "source_tables": ["agent_assets", "env_vars"],
        "manual_edits_allowed": false,
        "agent_assets": snapshot.agent_assets,
        "env_vars": snapshot.env_vars,
    });
    yaml_with_header(&value)
}

fn render_agent_env_lock_yaml(snapshot: &CatalogSnapshot) -> anyhow::Result<Vec<u8>> {
    let locked_assets = snapshot
        .agent_assets
        .iter()
        .map(|row| {
            serde_json::json!({
                "asset_kind": row.asset_kind,
                "name": row.name,
                "source": row.source,
                "destination": row.destination,
                "hash": row.hash,
                "source_revision": row.source_revision,
                "lock_status": row.lock_status,
                "drift_status": row.drift_status,
            })
        })
        .collect::<Vec<_>>();
    let value = serde_json::json!({
        "generated_by": "envctl catalog render",
        "source_table": "agent_assets",
        "manual_edits_allowed": false,
        "assets": locked_assets,
    });
    yaml_with_header(&value)
}

fn yaml_with_header(value: &serde_json::Value) -> anyhow::Result<Vec<u8>> {
    let mut text = String::new();
    text.push_str("# Generated by envctl catalog render.\n");
    text.push_str("# Manual edits allowed: no.\n");
    text.push_str(&serde_yaml::to_string(value)?);
    Ok(text.into_bytes())
}

fn render_codex_config_toml(snapshot: &CatalogSnapshot) -> String {
    let mut out = String::new();
    out.push_str("# Generated by envctl catalog render.\n");
    out.push_str("# Source tables: settings, registries, agent_assets.\n");
    out.push_str("# Manual edits allowed: no.\n\n");
    out.push_str("[catalog]\n");
    write_toml_string(&mut out, "generated_by", "envctl catalog render");
    write_toml_string_array(
        &mut out,
        "source_tables",
        &[
            "settings".to_string(),
            "registries".to_string(),
            "agent_assets".to_string(),
        ],
    );
    out.push_str("manual_edits_allowed = false\n\n");
    for server in rendered_mcp_servers(snapshot) {
        let table_key =
            serde_json::to_string(&server.name).unwrap_or_else(|_| "\"unknown\"".to_string());
        let _ = writeln!(out, "[mcp_servers.{table_key}]");
        if let Some(command) = server.command.as_deref() {
            write_toml_string(&mut out, "command", command);
        }
        if !server.args.is_empty() {
            write_toml_string_array(&mut out, "args", &server.args);
        }
        if let Some(url) = server.url.as_deref() {
            write_toml_string(&mut out, "url", url);
        }
        if !server.env.is_empty() {
            out.push_str("[mcp_servers.");
            out.push_str(&table_key);
            out.push_str(".env]\n");
            for (key, value) in &server.env {
                write_toml_string(&mut out, key, value);
            }
        }
        out.push('\n');
    }
    out
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct RenderedMcpServer {
    name: String,
    command: Option<String>,
    args: Vec<String>,
    url: Option<String>,
    env: BTreeMap<String, String>,
}

fn rendered_mcp_servers(snapshot: &CatalogSnapshot) -> Vec<RenderedMcpServer> {
    let mut servers = BTreeMap::<String, RenderedMcpServer>::new();
    ingest_mcp_json_servers(snapshot, &mut servers);
    ingest_codex_toml_servers(snapshot, &mut servers);

    for registry in snapshot
        .registries
        .iter()
        .filter(|row| row.registry_kind == "mcp")
    {
        servers
            .entry(registry.name.clone())
            .or_insert_with(|| RenderedMcpServer {
                name: registry.name.clone(),
                ..RenderedMcpServer::default()
            });
    }

    servers.into_values().collect()
}

fn ingest_mcp_json_servers(
    snapshot: &CatalogSnapshot,
    servers: &mut BTreeMap<String, RenderedMcpServer>,
) {
    let path = Path::new(&snapshot.repo_root).join(".mcp.json");
    let Ok(Some(value)) = parse_config_to_json(&path, "json") else {
        return;
    };
    let Some(entries) = value.get("mcpServers").and_then(|value| value.as_object()) else {
        return;
    };

    for (name, value) in entries {
        let mut server = RenderedMcpServer {
            name: name.clone(),
            ..RenderedMcpServer::default()
        };
        if let Some(command) = value.get("command").and_then(|value| value.as_str()) {
            server.command = Some(command.to_string());
        }
        if let Some(args) = value.get("args").and_then(|value| value.as_array()) {
            server.args = args
                .iter()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect();
        }
        if let Some(url) = value.get("url").and_then(|value| value.as_str()) {
            server.url = Some(url.to_string());
        }
        if let Some(env) = value.get("env").and_then(|value| value.as_object()) {
            server.env = env
                .iter()
                .filter_map(|(key, value)| {
                    value.as_str().map(|value| (key.clone(), value.to_string()))
                })
                .collect();
        }
        servers.insert(name.clone(), server);
    }
}

fn ingest_codex_toml_servers(
    snapshot: &CatalogSnapshot,
    servers: &mut BTreeMap<String, RenderedMcpServer>,
) {
    let path = Path::new(&snapshot.repo_root).join(".codex/config.toml");
    let Ok(Some(value)) = parse_config_to_json(&path, "toml") else {
        return;
    };
    let Some(entries) = value.get("mcp_servers").and_then(|value| value.as_object()) else {
        return;
    };

    for (name, value) in entries {
        let server = servers
            .entry(name.clone())
            .or_insert_with(|| RenderedMcpServer {
                name: name.clone(),
                ..RenderedMcpServer::default()
            });
        if server.command.is_none() {
            if let Some(command) = value.get("command").and_then(|value| value.as_str()) {
                server.command = Some(command.to_string());
            }
        }
        if server.args.is_empty() {
            if let Some(args) = value.get("args").and_then(|value| value.as_array()) {
                server.args = args
                    .iter()
                    .filter_map(|value| value.as_str().map(str::to_string))
                    .collect();
            }
        }
        if server.url.is_none() {
            if let Some(url) = value.get("url").and_then(|value| value.as_str()) {
                server.url = Some(url.to_string());
            }
        }
        if server.env.is_empty() {
            if let Some(env) = value.get("env").and_then(|value| value.as_object()) {
                server.env = env
                    .iter()
                    .filter_map(|(key, value)| {
                        value.as_str().map(|value| (key.clone(), value.to_string()))
                    })
                    .collect();
            }
        }
    }
}

fn render_mcp_json(snapshot: &CatalogSnapshot) -> anyhow::Result<Vec<u8>> {
    let mcp_servers = snapshot
        .registries
        .iter()
        .filter(|row| row.registry_kind == "mcp")
        .map(|row| {
            (
                row.name.clone(),
                serde_json::json!({
                    "entry_id": row.entry_id,
                    "component_id": row.component_id,
                    "source_file": row.source_file,
                    "status": row.status,
                    "tier": row.tier,
                    "drift_status": row.drift_status,
                }),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let value = serde_json::json!({
        "generated_by": "envctl catalog render",
        "source_table": "registries",
        "manual_edits_allowed": false,
        "mcpServers": mcp_servers,
    });
    json_with_trailing_newline(&value)
}

fn render_env_shell(snapshot: &CatalogSnapshot) -> String {
    let mut out = String::new();
    out.push_str("# Generated by envctl catalog render.\n");
    out.push_str("# Source table: env_vars.\n");
    out.push_str("# Manual edits allowed: no.\n");
    out.push_str("# Sensitive values are omitted/redacted.\n\n");
    let mut emitted = BTreeSet::new();
    for row in &snapshot.env_vars {
        if row.sensitive || !emitted.insert(row.var_name.clone()) {
            continue;
        }
        let value = row
            .effective_value
            .as_ref()
            .or(row.value.as_ref())
            .or(row.default_value.as_ref());
        if let Some(value) = value {
            let _ = writeln!(
                &mut out,
                "export {}={}",
                shell_identifier(&row.var_name),
                shell_quote(value)
            );
        }
    }
    out
}

fn render_dashboard_kdl(snapshot: &CatalogSnapshot) -> String {
    let mut out = String::new();
    out.push_str("// Generated by envctl catalog render.\n");
    out.push_str("// Source tables: paths, settings.\n");
    out.push_str("// Manual edits allowed: no.\n\n");
    out.push_str("layout {\n");
    out.push_str("  tab name=\"catalog\" {\n");
    let _ = writeln!(
        &mut out,
        "    pane command=\"envctl\" args=\"catalog\" \"scan\" \"--json\" // paths={} settings={}",
        snapshot.paths.len(),
        snapshot.settings.len()
    );
    out.push_str("  }\n");
    out.push_str("}\n");
    out
}

fn render_systemd_unit(snapshot: &CatalogSnapshot) -> String {
    let mut out = String::new();
    out.push_str("# Generated by envctl catalog render.\n");
    out.push_str("# Source table: observed_facts.\n");
    out.push_str("# Manual edits allowed: no.\n\n");
    out.push_str("[Unit]\n");
    out.push_str("Description=envctl catalog drift check\n\n");
    out.push_str("[Service]\n");
    out.push_str("Type=oneshot\n");
    out.push_str("ExecStart=envctl catalog diff --json\n");
    let _ = writeln!(
        &mut out,
        "Environment=ENVCTL_CATALOG_OBSERVED_FACTS={}",
        snapshot.observed_facts.len()
    );
    out.push('\n');
    out.push_str("[Install]\n");
    out.push_str("WantedBy=default.target\n");
    out
}

fn write_toml_string(out: &mut String, key: &str, value: &str) {
    let _ = writeln!(
        out,
        "{key} = {}",
        serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
    );
}

fn write_toml_string_array(out: &mut String, key: &str, values: &[String]) {
    let rendered = values
        .iter()
        .map(|value| serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string()))
        .collect::<Vec<_>>()
        .join(", ");
    let _ = writeln!(out, "{key} = [{rendered}]");
}

fn shell_identifier(name: &str) -> String {
    name.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn ingest_components(
    registry: &Registry,
    component_sources: &BTreeMap<String, String>,
    lock: &LockFile,
    components: &mut Vec<ComponentRow>,
    nix_components: &mut Vec<NixComponentRow>,
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
        let component_row = ComponentRow {
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
        };
        if let Some(nix_surface) = nix_component_surface(&component_row) {
            nix_components.push(NixComponentRow {
                component_id: component_row.component_id.clone(),
                name: component_row.name.clone(),
                source_file: component_row.source_file.clone(),
                nix_surface: nix_surface.to_string(),
                owner_component: None,
                profile_entry: None,
                original_url: None,
                profile_url: None,
                store_paths: Vec::new(),
                frontdoor_paths: Vec::new(),
                requires: component_row.requires.clone(),
                status: component_row.status.clone(),
                lock_hash: component_row.lock_hash.clone(),
                resolved_order: component_row.resolved_order,
            });
        }
        components.push(component_row);
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

#[derive(Clone, Debug, Default, Deserialize)]
struct NixProfileList {
    #[serde(default)]
    elements: BTreeMap<String, NixProfileElement>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct NixProfileElement {
    #[serde(default)]
    active: bool,
    #[serde(rename = "attrPath", default)]
    attr_path: Option<String>,
    #[serde(rename = "originalUrl", default)]
    original_url: Option<String>,
    #[serde(rename = "storePaths", default)]
    store_paths: Vec<String>,
    #[serde(default)]
    url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NixProfileFrontdoor {
    path: String,
    resolved_path: String,
    store_path: String,
    path_kind: String,
    artifact_kind: String,
}

fn ingest_live_nix_profile(
    repo_root: &Path,
    nix_components: &mut Vec<NixComponentRow>,
    paths: &mut Vec<PathRow>,
) {
    let Some(home_dir) = catalog_home_dir() else {
        return;
    };
    if !repo_root.starts_with(&home_dir) {
        return;
    }
    let Some(profile) = load_nix_profile(&home_dir) else {
        return;
    };
    let base_order = nix_components.len();
    ingest_nix_profile_snapshot(&home_dir, &profile, nix_components, paths, base_order);
}

fn catalog_home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn load_nix_profile(home_dir: &Path) -> Option<NixProfileList> {
    let profile_root = home_dir.join(".nix-profile");
    if !profile_root.exists() {
        return None;
    }
    let output = std::process::Command::new("nix")
        .args(["profile", "list", "--json"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    serde_json::from_slice(&output.stdout).ok()
}

fn ingest_nix_profile_snapshot(
    home_dir: &Path,
    profile: &NixProfileList,
    nix_components: &mut Vec<NixComponentRow>,
    paths: &mut Vec<PathRow>,
    base_order: usize,
) {
    let profile_root = home_dir.join(".nix-profile");
    let entries = profile
        .elements
        .iter()
        .map(|(name, element)| (name.as_str(), element))
        .collect::<Vec<_>>();
    let frontdoors_by_entry = collect_nix_profile_frontdoors(&profile_root, &entries);
    for (idx, (entry_name, element)) in entries.into_iter().enumerate() {
        let owner_component = nix_profile_owner_component(entry_name, element);
        let row_id = format!("profile:{entry_name}");
        let mut store_paths = element.store_paths.clone();
        store_paths.sort();
        let mut frontdoor_paths = frontdoors_by_entry
            .get(entry_name)
            .cloned()
            .unwrap_or_default();
        frontdoor_paths.sort_by(|a, b| {
            a.path
                .cmp(&b.path)
                .then(a.resolved_path.cmp(&b.resolved_path))
        });
        let lock_hash = nix_profile_lock_hash(entry_name, element, &store_paths, &frontdoor_paths);
        nix_components.push(NixComponentRow {
            component_id: row_id.clone(),
            name: entry_name.to_string(),
            source_file: "nix profile list --json".to_string(),
            nix_surface: "nix_profile".to_string(),
            owner_component: owner_component.clone(),
            profile_entry: Some(entry_name.to_string()),
            original_url: element.original_url.clone(),
            profile_url: element.url.clone(),
            store_paths: store_paths.clone(),
            frontdoor_paths: frontdoor_paths.iter().map(|fd| fd.path.clone()).collect(),
            requires: Vec::new(),
            status: if element.active {
                "active".to_string()
            } else {
                "inactive".to_string()
            },
            lock_hash,
            resolved_order: base_order + idx + 1,
        });
        for store_path in &store_paths {
            let store_id = nix_store_path_id(entry_name, store_path);
            paths.push(PathRow {
                path_id: store_id.clone(),
                path: store_path.clone(),
                path_kind: "nix_store_path".to_string(),
                owner_component: owner_component.clone(),
                owner_record_id: Some(row_id.clone()),
                artifact_kind: "nix_store_object".to_string(),
                resolved_path: None,
                link_target_id: None,
                canonical: true,
                legacy: false,
                bridge: false,
                protected: true,
                source: "nix profile list --json".to_string(),
                verification_status: if element.active {
                    "listed_active".to_string()
                } else {
                    "listed_inactive".to_string()
                },
            });
        }
        for frontdoor in &frontdoor_paths {
            let target_id = nix_store_path_id(entry_name, &frontdoor.store_path);
            paths.push(PathRow {
                path_id: nix_frontdoor_path_id(entry_name, &frontdoor.path),
                path: frontdoor.path.clone(),
                path_kind: frontdoor.path_kind.clone(),
                owner_component: owner_component.clone(),
                owner_record_id: Some(row_id.clone()),
                artifact_kind: frontdoor.artifact_kind.clone(),
                resolved_path: Some(frontdoor.resolved_path.clone()),
                link_target_id: Some(target_id),
                canonical: false,
                legacy: false,
                bridge: true,
                protected: false,
                source: "nix profile list --json".to_string(),
                verification_status: "resolved".to_string(),
            });
        }
    }
}

fn collect_nix_profile_frontdoors(
    profile_root: &Path,
    entries: &[(&str, &NixProfileElement)],
) -> BTreeMap<String, Vec<NixProfileFrontdoor>> {
    let mut by_entry: BTreeMap<String, Vec<NixProfileFrontdoor>> = BTreeMap::new();
    let mut store_index = Vec::new();
    for (entry_name, element) in entries {
        for store_path in &element.store_paths {
            store_index.push((store_path.clone(), (*entry_name).to_string()));
        }
    }
    store_index.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then(a.0.cmp(&b.0)));

    for (relative_dir, path_kind, artifact_kind) in [
        (
            "bin",
            "nix_profile_frontdoor_bin",
            "nix_profile_frontdoor_symlink",
        ),
        (
            "share/applications",
            "nix_profile_frontdoor_desktop",
            "nix_profile_frontdoor_desktop_entry",
        ),
    ] {
        let dir = profile_root.join(relative_dir);
        let Ok(read_dir) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in read_dir.flatten() {
            let path = entry.path();
            let Ok(metadata) = std::fs::symlink_metadata(&path) else {
                continue;
            };
            if !metadata.file_type().is_symlink() {
                continue;
            }
            let Ok(resolved_path) = path.canonicalize() else {
                continue;
            };
            let resolved = resolved_path.display().to_string();
            let Some((store_path, owner_entry)) = store_index
                .iter()
                .find(|(store_path, _)| resolved.starts_with(store_path))
                .cloned()
            else {
                continue;
            };
            by_entry
                .entry(owner_entry)
                .or_default()
                .push(NixProfileFrontdoor {
                    path: path.display().to_string(),
                    resolved_path: resolved,
                    store_path,
                    path_kind: path_kind.to_string(),
                    artifact_kind: artifact_kind.to_string(),
                });
        }
    }
    by_entry
}

fn nix_profile_owner_component(entry_name: &str, element: &NixProfileElement) -> Option<String> {
    let mut candidates = Vec::new();
    candidates.push(entry_name.to_string());
    if let Some(attr_path) = &element.attr_path {
        candidates.push(attr_path.clone());
    }
    if let Some(original_url) = &element.original_url {
        candidates.push(original_url.clone());
    }
    if let Some(url) = &element.url {
        candidates.push(url.clone());
    }
    if candidates
        .iter()
        .any(|candidate| candidate.contains("yazelix"))
    {
        Some("yazelix".to_string())
    } else {
        None
    }
}

fn nix_profile_lock_hash(
    entry_name: &str,
    element: &NixProfileElement,
    store_paths: &[String],
    frontdoors: &[NixProfileFrontdoor],
) -> String {
    let mut out = String::new();
    out.push_str(entry_name);
    out.push('\n');
    if let Some(attr_path) = &element.attr_path {
        out.push_str(attr_path);
    }
    out.push('\n');
    if let Some(original_url) = &element.original_url {
        out.push_str(original_url);
    }
    out.push('\n');
    if let Some(url) = &element.url {
        out.push_str(url);
    }
    out.push('\n');
    for store_path in store_paths {
        out.push_str(store_path);
        out.push('\n');
    }
    for frontdoor in frontdoors {
        out.push_str(&frontdoor.path);
        out.push('\n');
        out.push_str(&frontdoor.resolved_path);
        out.push('\n');
    }
    sha256_hex(out.as_bytes())
}

fn nix_store_path_id(entry_name: &str, store_path: &str) -> String {
    format!(
        "nix_store:{}:{}",
        sanitize_catalog_id(entry_name),
        sha256_hex(store_path.as_bytes())
    )
}

fn nix_frontdoor_path_id(entry_name: &str, frontdoor_path: &str) -> String {
    format!(
        "nix_frontdoor:{}:{}",
        sanitize_catalog_id(entry_name),
        sha256_hex(frontdoor_path.as_bytes())
    )
}

fn sanitize_catalog_id(raw: &str) -> String {
    raw.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn nix_component_surface(row: &ComponentRow) -> Option<&'static str> {
    if row.source_file.starts_with("manifest/nix") {
        Some("nix_manifest")
    } else if row.component_id == "nix-portable" {
        Some("nix_portable_toolchain")
    } else {
        None
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

fn ingest_envctl_home_frontdoors(repo_root: &Path, paths: &mut Vec<PathRow>) {
    if !repo_root.join("home").is_dir() {
        return;
    }

    let meta_root = catalog_meta_root(repo_root);
    for source_rel in collect_envctl_home_managed_relpaths(repo_root) {
        let Some(runtime_rel) = envctl_home_runtime_relative_path(&source_rel) else {
            continue;
        };
        let Some(owner_component) = envctl_home_owner_component(&source_rel) else {
            continue;
        };
        let source_abs = repo_root.join(&source_rel);
        let source_id = envctl_home_source_path_id(&source_rel);
        let source_display = source_abs.display().to_string();
        paths.push(PathRow {
            path_id: source_id.clone(),
            path: source_display.clone(),
            path_kind: "envctl_home_source".to_string(),
            owner_component: Some(owner_component.to_string()),
            owner_record_id: Some(owner_component.to_string()),
            artifact_kind: "envctl_home_source_file".to_string(),
            resolved_path: None,
            link_target_id: None,
            canonical: true,
            legacy: false,
            bridge: false,
            protected: true,
            source: source_rel.clone(),
            verification_status: "tracked".to_string(),
        });

        let runtime_abs = meta_root.join(&runtime_rel);
        paths.push(PathRow {
            path_id: envctl_home_frontdoor_path_id(&runtime_rel),
            path: runtime_abs.display().to_string(),
            path_kind: envctl_home_frontdoor_kind(&source_rel).to_string(),
            owner_component: Some(owner_component.to_string()),
            owner_record_id: Some(owner_component.to_string()),
            artifact_kind: envctl_home_frontdoor_artifact(&source_rel).to_string(),
            resolved_path: Some(source_display),
            link_target_id: Some(source_id),
            canonical: true,
            legacy: false,
            bridge: true,
            protected: true,
            source: "manifest/components.d/portability-links.toml".to_string(),
            verification_status: "declared".to_string(),
        });
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
            owner_record_id: None,
            artifact_kind: entry.purpose.to_string(),
            resolved_path: None,
            link_target_id: None,
            canonical,
            legacy,
            bridge: legacy || entry.key.contains("bridge") || entry.key.contains("legacy"),
            protected: is_protected_layout_key(entry.key),
            source: "crates/engine/src/layout.rs".to_string(),
            verification_status: layout_path_verification_status(&entry.path),
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

fn layout_path_verification_status(path: &Path) -> String {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            let file_type = metadata.file_type();
            if file_type.is_symlink() {
                "symlink_exists".to_string()
            } else if file_type.is_dir() {
                "dir_exists".to_string()
            } else if file_type.is_file() {
                "file_exists".to_string()
            } else {
                "special_exists".to_string()
            }
        }
        Err(err) if err.kind() == ErrorKind::NotFound => "missing".to_string(),
        Err(err) if err.kind() == ErrorKind::PermissionDenied => "inaccessible".to_string(),
        Err(_) => "error".to_string(),
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

fn ingest_codedb_file_imports(
    repo_root: &Path,
    observed_at: &str,
    rows: &mut Vec<CodedbFileImportRow>,
) -> anyhow::Result<()> {
    let inventory_path = repo_root.join("docs/generated/yazelix_file_target_inventory.json");
    if !inventory_path.is_file() {
        return Ok(());
    }

    let bytes = std::fs::read(&inventory_path).with_context(|| {
        format!(
            "reading Yazelix file inventory {}",
            inventory_path.display()
        )
    })?;
    let inventory: Vec<YazelixFileInventoryRow> =
        serde_json::from_slice(&bytes).with_context(|| {
            format!(
                "parsing Yazelix file inventory {}",
                inventory_path.display()
            )
        })?;

    for item in inventory {
        let target = PathBuf::from(&item.absolute_path);
        let file_bytes = if item.import_mode == "content_blob" {
            std::fs::read(&target).ok()
        } else {
            None
        };
        let metadata_len = std::fs::metadata(&target)
            .ok()
            .filter(|meta| meta.is_file())
            .map(|meta| meta.len())
            .unwrap_or(0);
        let content_hash = file_bytes.as_deref().map(sha256_hex);
        let byte_length = file_bytes
            .as_ref()
            .map(|bytes| bytes.len() as u64)
            .unwrap_or(metadata_len);
        let blob_ref = content_hash.as_ref().map(|hash| format!("sha256:{hash}"));
        let content_ready = blob_ref.is_some();
        let import_status = if item.import_mode == "content_blob" {
            if content_ready {
                "blob_metadata_ready"
            } else {
                "metadata_only"
            }
        } else {
            "metadata_only"
        };
        let skip_reason = if content_ready {
            String::new()
        } else if item.import_mode == "content_blob" {
            "content_blob target is not readable as a regular file".to_string()
        } else if item.safety_policy.is_empty() {
            "metadata_only import mode".to_string()
        } else {
            item.safety_policy.clone()
        };
        let structured_rows = if content_ready {
            structured_file_rows(&target, &item.parser_hint)
        } else {
            Vec::new()
        };
        let structured_status = if !structured_rows.is_empty() {
            "structured_rows_ready"
        } else if item.import_mode == "metadata_only" {
            "metadata_only"
        } else {
            "unstructured_blob"
        };

        rows.push(CodedbFileImportRow {
            table: "envctl_yazelix_file_import".to_string(),
            target_id: item.target_id,
            logical_owner: item.owner,
            absolute_path: item.absolute_path,
            normalized_path: item.normalized_logical_path,
            source_of_truth_class: item.source_of_truth_class,
            file_kind: item.file_kind,
            parser_hint: item.parser_hint,
            content_hash,
            byte_length,
            blob_ref,
            import_safety_policy: item.safety_policy,
            reproduction_policy: item.reproduction_policy,
            import_mode: item.import_mode,
            import_status: import_status.to_string(),
            skip_reason,
            structured_table: "envctl_yazelix_file_structured_rows".to_string(),
            structured_status: structured_status.to_string(),
            structured_row_count: structured_rows.len(),
            structured_rows,
            last_observed: observed_at.to_string(),
            provenance: repo_relative(repo_root, &inventory_path),
        });
    }

    Ok(())
}

fn structured_file_rows(path: &Path, parser_hint: &str) -> Vec<CodedbStructuredFileRow> {
    if matches!(
        parser_hint,
        "json" | "jsonc" | "toml" | "yaml" | "yml" | "nu" | "kdl"
    ) {
        if let Ok(Some(value)) = parse_config_to_json(path, parser_hint) {
            let mut flattened = Vec::new();
            flatten_json(None, &value, &mut flattened);
            return flattened
                .into_iter()
                .enumerate()
                .map(|(idx, (key, value))| CodedbStructuredFileRow {
                    row_index: idx,
                    row_kind: "structured_value".to_string(),
                    format: parser_hint.to_string(),
                    key,
                    value,
                })
                .collect();
        }
    }

    if !matches!(
        parser_hint,
        "nix"
            | "lua"
            | "markdown"
            | "desktop"
            | "service"
            | "shell"
            | "conf"
            | "terminal_conf"
            | "plain_config"
    ) {
        return Vec::new();
    }
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .enumerate()
        .filter_map(|(idx, line)| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            let (row_kind, key, value) = text_line_parts(trimmed);
            Some(CodedbStructuredFileRow {
                row_index: idx,
                row_kind: row_kind.to_string(),
                format: parser_hint.to_string(),
                key,
                value,
            })
        })
        .collect()
}

fn text_line_parts(line: &str) -> (&'static str, String, String) {
    if line.starts_with('#') || line.starts_with("//") || line.starts_with("--") {
        return ("comment", String::new(), line.to_string());
    }
    for delimiter in ["=", ":", " "] {
        if let Some((key, value)) = line.split_once(delimiter) {
            let key = key.trim().to_string();
            if !key.is_empty() {
                return ("entry", key, value.trim().to_string());
            }
        }
    }
    ("line", String::new(), line.to_string())
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
    collect_envctl_home_files(repo_root, &mut paths);
    collect_yazelix_config_files(repo_root, &mut paths);
    paths.sort();
    paths
}

fn collect_envctl_home_files(repo_root: &Path, out: &mut Vec<PathBuf>) {
    for rel in collect_envctl_home_managed_relpaths(repo_root) {
        push_if_present(out, repo_root.join(rel));
    }
}

fn collect_yazelix_config_files(repo_root: &Path, out: &mut Vec<PathBuf>) {
    let has_yazelix_config_contract = repo_root
        .join("config_metadata/main_config_contract.toml")
        .is_file()
        || repo_root.join("settings_default.jsonc").is_file();
    if !has_yazelix_config_contract {
        return;
    }

    for rel in [
        "settings_default.jsonc",
        "release_metadata.toml",
        "flake.nix",
        "flake.lock",
    ] {
        push_if_present(out, repo_root.join(rel));
    }

    for (rel, exts) in [
        ("config_metadata", &["toml", "json"][..]),
        (
            "configs",
            &[
                "toml", "json", "jsonc", "kdl", "yml", "yaml", "conf", "lua", "scm",
            ][..],
        ),
        ("nushell", &["nu", "toml", "md"][..]),
        ("home_manager", &["nix", "md"][..]),
        ("packaging", &["nix", "toml"][..]),
        ("rust_core/yazelix_zellij_config_pack", &["toml", "kdl"][..]),
    ] {
        collect_matching_files(&repo_root.join(rel), exts, out);
    }
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
        "jsonc" => Ok(Some(serde_json::from_str(&jsonc_to_json(&text))?)),
        "nushell" => Ok(Some(nushell_reproduction_metadata(&text))),
        "nix" | "lua" | "terminal_conf" | "markdown" => {
            Ok(Some(text_reproduction_metadata(format, &text)))
        }
        "yaml" => {
            let value: serde_yaml::Value = serde_yaml::from_str(&text)?;
            Ok(Some(serde_json::to_value(value)?))
        }
        "json" => Ok(Some(serde_json::from_str(&text)?)),
        "kdl" => Ok(Some(kdl_reproduction_metadata(&text))),
        _ => Ok(None),
    }
}

fn nushell_reproduction_metadata(text: &str) -> serde_json::Value {
    let lines: Vec<&str> = text.lines().collect();
    let source_lines: Vec<String> = trimmed_lines_with_prefix(&lines, "source ");
    let use_lines: Vec<String> = trimmed_lines_with_prefix(&lines, "use ");
    let plugin_use_lines: Vec<String> = lines
        .iter()
        .map(|line| line.trim())
        .filter(|line| line.starts_with("plugin use ") || line.starts_with("plugin add "))
        .map(str::to_string)
        .collect();
    let env_assignment_count = lines
        .iter()
        .map(|line| line.trim())
        .filter(|line| {
            line.starts_with("$env.")
                || line.starts_with("load-env ")
                || line.starts_with("with-env ")
        })
        .count();

    serde_json::json!({
        "format": "nushell",
        "byte_count": text.len(),
        "line_count": lines.len(),
        "nonempty_line_count": lines.iter().filter(|line| !line.trim().is_empty()).count(),
        "sha256": sha256_hex(text.as_bytes()),
        "reproduction_policy": "source_bytes_required",
        "structured_metadata": true,
        "source_lines": source_lines,
        "use_lines": use_lines,
        "plugin_use_lines": plugin_use_lines,
        "def_count": lines.iter().filter(|line| line.trim_start().starts_with("def ")).count(),
        "alias_count": lines.iter().filter(|line| line.trim_start().starts_with("alias ")).count(),
        "env_assignment_count": env_assignment_count,
        "references_yazelix_init": text.contains("yazelix_init"),
        "references_yazelix_extern": text.contains("extern") || text.contains("completions"),
    })
}

fn kdl_reproduction_metadata(text: &str) -> serde_json::Value {
    let lines: Vec<&str> = text.lines().collect();
    let node_names = kdl_node_names(&lines);

    serde_json::json!({
        "format": "kdl",
        "byte_count": text.len(),
        "line_count": lines.len(),
        "nonempty_line_count": lines.iter().filter(|line| !line.trim().is_empty()).count(),
        "sha256": sha256_hex(text.as_bytes()),
        "reproduction_policy": "source_bytes_required",
        "structured_metadata": true,
        "node_names": node_names.iter().cloned().collect::<Vec<_>>(),
        "layout_count": node_names.iter().filter(|name| name.as_str() == "layout").count(),
        "tab_count": node_names.iter().filter(|name| name.as_str() == "tab").count(),
        "pane_count": node_names.iter().filter(|name| name.as_str() == "pane").count(),
        "plugin_count": node_names.iter().filter(|name| name.as_str() == "plugin").count(),
        "has_layout_node": node_names.contains("layout"),
    })
}

fn text_reproduction_metadata(format: &str, text: &str) -> serde_json::Value {
    let lines: Vec<&str> = text.lines().collect();
    let comment_line_count = lines
        .iter()
        .map(|line| line.trim_start())
        .filter(|line| line.starts_with('#') || line.starts_with("//") || line.starts_with("--"))
        .count();

    serde_json::json!({
        "format": format,
        "byte_count": text.len(),
        "line_count": lines.len(),
        "nonempty_line_count": lines.iter().filter(|line| !line.trim().is_empty()).count(),
        "comment_line_count": comment_line_count,
        "sha256": sha256_hex(text.as_bytes()),
        "reproduction_policy": "source_bytes_required",
        "structured_metadata": true,
        "top_level_tokens": top_level_tokens(&lines),
    })
}

fn top_level_tokens(lines: &[&str]) -> Vec<String> {
    let mut tokens = BTreeSet::new();
    for line in lines {
        let trimmed = line.trim_start();
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with("//")
            || trimmed.starts_with("--")
        {
            continue;
        }
        if let Some(token) = trimmed
            .split(|ch: char| ch.is_whitespace() || ch == '=' || ch == '{' || ch == '(')
            .next()
            .filter(|token| !token.is_empty())
        {
            tokens.insert(token.to_string());
        }
    }
    tokens.into_iter().collect()
}

fn trimmed_lines_with_prefix(lines: &[&str], prefix: &str) -> Vec<String> {
    lines
        .iter()
        .map(|line| line.trim())
        .filter(|line| line.starts_with(prefix))
        .map(str::to_string)
        .collect()
}

fn kdl_node_names(lines: &[&str]) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for line in lines {
        let trimmed = line.trim_start();
        if trimmed.is_empty()
            || trimmed.starts_with("//")
            || trimmed.starts_with('#')
            || trimmed.starts_with('}')
        {
            continue;
        }
        if let Some(name) = trimmed
            .split(|ch: char| ch.is_whitespace() || ch == '{' || ch == '(' || ch == ';')
            .next()
            .filter(|name| !name.is_empty())
        {
            names.insert(name.to_string());
        }
    }
    names
}

fn jsonc_to_json(input: &str) -> String {
    let mut without_comments = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if in_string {
            without_comments.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            without_comments.push(ch);
            continue;
        }

        if ch == '/' {
            match chars.peek().copied() {
                Some('/') => {
                    chars.next();
                    for next in chars.by_ref() {
                        if next == '\n' {
                            without_comments.push('\n');
                            break;
                        }
                    }
                    continue;
                }
                Some('*') => {
                    chars.next();
                    let mut previous = '\0';
                    for next in chars.by_ref() {
                        if previous == '*' && next == '/' {
                            break;
                        }
                        previous = next;
                    }
                    continue;
                }
                _ => {}
            }
        }

        without_comments.push(ch);
    }

    remove_json_trailing_commas(&without_comments)
}

fn remove_json_trailing_commas(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if in_string {
            output.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            output.push(ch);
            continue;
        }

        if ch == ',' {
            let mut lookahead = chars.clone();
            while matches!(lookahead.peek(), Some(next) if next.is_whitespace()) {
                lookahead.next();
            }
            if matches!(lookahead.peek(), Some('}' | ']')) {
                continue;
            }
        }

        output.push(ch);
    }

    output
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
    if rel == "flake.lock" {
        "json"
    } else if rel.ends_with(".toml") || rel.ends_with(".lock") && rel.contains("envctl") {
        "toml"
    } else if rel.ends_with(".yaml") || rel.ends_with(".yml") || rel.ends_with("agent-env.lock") {
        "yaml"
    } else if rel.ends_with(".jsonc") {
        "jsonc"
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
    } else if rel.ends_with(".nix") {
        "nix"
    } else if rel.ends_with(".lua") {
        "lua"
    } else if rel.ends_with(".conf") {
        "terminal_conf"
    } else if rel.ends_with(".nu") {
        "nushell"
    } else if rel.ends_with(".sh") || rel.ends_with(".bash") {
        "shell"
    } else {
        "unknown"
    }
}

fn infer_file_kind(manifest_dir: &Path, path: &Path, rel: &str) -> &'static str {
    if path.starts_with(manifest_dir) && rel.ends_with(".toml") {
        "manifest"
    } else if rel.starts_with("home/.claude/") {
        "envctl_home_claude_config"
    } else if rel == "home/.gitconfig" {
        "envctl_home_git_config"
    } else if rel.starts_with("home/.config/ghostty/") {
        "envctl_home_ghostty_config"
    } else if rel.starts_with("home/.config/kasetto/") {
        "envctl_home_kasetto_config"
    } else if rel == "home/.config/nushell/meta-usr-path.nu" {
        "envctl_home_nushell_path"
    } else if rel.starts_with("home/.config/nushell/") {
        "envctl_home_nushell_config"
    } else if rel.starts_with("home/.config/rtk/") {
        "envctl_home_rtk_config"
    } else if rel.starts_with("home/.config/systemd/user/") {
        "envctl_home_systemd_user_unit"
    } else if rel.starts_with("home/.config/yazelix/") {
        "envctl_home_yazelix_config"
    } else if rel == "settings_default.jsonc" {
        "yazelix_settings_default"
    } else if rel.starts_with("config_metadata/") {
        "yazelix_config_metadata"
    } else if rel.starts_with("configs/") {
        "yazelix_runtime_config"
    } else if rel.starts_with("nushell/") {
        "yazelix_nushell_config"
    } else if rel.starts_with("home_manager/") {
        "yazelix_home_manager_config"
    } else if rel.starts_with("packaging/") || rel == "flake.nix" || rel == "flake.lock" {
        "yazelix_packaging_config"
    } else if rel.starts_with("rust_core/yazelix_zellij_config_pack/") {
        "yazelix_zellij_config_pack"
    } else if rel == "release_metadata.toml" {
        "yazelix_release_metadata"
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
    } else if let Some(owner) = envctl_home_owner_component(rel) {
        Some(owner.to_string())
    } else if rel.starts_with("manifest/")
        && rel.ends_with(".toml")
        && rel != "manifest/envctl.lock"
    {
        rel.rsplit_once('/')
            .map(|(_, name)| name.trim_end_matches(".toml").to_string())
    } else if rel == "settings_default.jsonc"
        || rel == "release_metadata.toml"
        || rel == "flake.nix"
        || rel == "flake.lock"
        || rel.starts_with("config_metadata/")
        || rel.starts_with("configs/")
        || rel.starts_with("nushell/")
        || rel.starts_with("home_manager/")
        || rel.starts_with("packaging/")
        || rel.starts_with("rust_core/yazelix_zellij_config_pack/")
    {
        Some("yazelix".to_string())
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
        "envctl_home_claude_config" => "agent_runtime",
        "envctl_home_git_config" => "git",
        "envctl_home_ghostty_config" => "terminal",
        "envctl_home_kasetto_config" => "agent_env",
        "envctl_home_nushell_config" | "envctl_home_nushell_path" => "shell",
        "envctl_home_rtk_config" => "tooling",
        "envctl_home_systemd_user_unit" => "systemd",
        "envctl_home_yazelix_config" => "yazelix",
        "secretd_config" | "secrets_env_schema" | "secrets_proto" => "secrets",
        "yazelix_settings_default"
        | "yazelix_config_metadata"
        | "yazelix_runtime_config"
        | "yazelix_nushell_config"
        | "yazelix_home_manager_config"
        | "yazelix_packaging_config"
        | "yazelix_zellij_config_pack"
        | "yazelix_release_metadata" => "yazelix",
        "handoff_task" | "handoff_ledger_export" | "handoff_report" => "handoff",
        _ => "workspace",
    }
}

fn setting_precedence(file_kind: &str) -> u32 {
    match file_kind {
        "codex_config" | "mcp_config" => 80,
        "agent_env" => 70,
        "manifest" => 60,
        "envctl_home_claude_config" => 88,
        "envctl_home_git_config" => 86,
        "envctl_home_ghostty_config" => 84,
        "envctl_home_kasetto_config" => 83,
        "envctl_home_nushell_config" => 83,
        "envctl_home_rtk_config" => 84,
        "envctl_home_systemd_user_unit" => 81,
        "envctl_home_yazelix_config" => 88,
        "envctl_home_nushell_path" => 82,
        "secretd_config" => 55,
        "secrets_env_schema" | "secrets_proto" => 45,
        "yazelix_settings_default" | "yazelix_config_metadata" => 90,
        "yazelix_runtime_config" | "yazelix_nushell_config" => 85,
        "yazelix_home_manager_config" | "yazelix_packaging_config" => 75,
        "yazelix_zellij_config_pack" => 70,
        "yazelix_release_metadata" => 65,
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

fn collect_envctl_home_managed_relpaths(repo_root: &Path) -> Vec<String> {
    let mut paths = Vec::new();
    walk_files(&repo_root.join("home"), &mut paths, &mut |path| {
        let rel = repo_relative(repo_root, path);
        envctl_home_runtime_relative_path(&rel).is_some()
    });
    paths
        .into_iter()
        .map(|path| repo_relative(repo_root, &path))
        .collect()
}

fn envctl_home_runtime_relative_path(rel: &str) -> Option<String> {
    envctl_home_owner_component(rel)
        .and_then(|_| rel.strip_prefix("home/").map(ToString::to_string))
}

fn envctl_home_owner_component(rel: &str) -> Option<&'static str> {
    if rel == "home/.gitconfig"
        || rel.starts_with("home/.config/ghostty/")
        || rel.starts_with("home/.config/kasetto/")
        || rel.starts_with("home/.config/nushell/")
        || rel.starts_with("home/.config/systemd/user/")
        || rel.starts_with("home/.config/yazelix/")
    {
        Some("home-config-links")
    } else if rel.starts_with("home/.config/rtk/") {
        Some("rtk-config-links")
    } else if rel.starts_with("home/.claude/") {
        Some("claude-global-links")
    } else {
        None
    }
}

fn envctl_home_frontdoor_kind(rel: &str) -> &'static str {
    if rel.starts_with("home/.config/yazelix/") {
        "envctl_home_yazelix_frontdoor"
    } else if rel == "home/.config/nushell/meta-usr-path.nu" {
        "envctl_home_nushell_frontdoor"
    } else if rel.starts_with("home/.config/rtk/") {
        "envctl_home_rtk_frontdoor"
    } else if rel.starts_with("home/.claude/") {
        "envctl_home_claude_frontdoor"
    } else {
        "envctl_home_frontdoor"
    }
}

fn envctl_home_frontdoor_artifact(rel: &str) -> &'static str {
    if rel.starts_with("home/.config/yazelix/configs/zellij/layouts/") {
        "envctl_managed_runtime_layout"
    } else if rel == "home/.config/nushell/meta-usr-path.nu" {
        "envctl_managed_shell_overlay"
    } else if rel.starts_with("home/.config/systemd/user/") {
        "envctl_managed_systemd_user_unit"
    } else if rel.starts_with("home/.claude/") {
        "envctl_managed_agent_config"
    } else if rel == "home/.gitconfig" {
        "envctl_managed_git_config"
    } else {
        "envctl_managed_runtime_config"
    }
}

fn catalog_meta_root(repo_root: &Path) -> PathBuf {
    if let Some(root) = std::env::var_os("META_ROOT").filter(|value| !value.is_empty()) {
        return PathBuf::from(root);
    }
    if repo_root
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
        == Some("src")
    {
        return repo_root
            .parent()
            .and_then(|parent| parent.parent())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| repo_root.to_path_buf());
    }
    repo_root.to_path_buf()
}

fn envctl_home_source_path_id(source_rel: &str) -> String {
    format!("envctl_home_source:{}", config_id(source_rel))
}

fn envctl_home_frontdoor_path_id(runtime_rel: &str) -> String {
    format!("envctl_home_frontdoor:{}", config_id(runtime_rel))
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

fn file_hash_optional(path: &Path) -> anyhow::Result<Option<String>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(sha256_hex(&bytes))),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).with_context(|| format!("reading {}", path.display())),
    }
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
            "nix-components".parse::<CatalogTableName>().unwrap(),
            CatalogTableName::NixComponents
        );
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
    fn jsonc_parser_keeps_strings_and_removes_comments_and_trailing_commas() {
        let value: serde_json::Value = serde_json::from_str(&jsonc_to_json(
            r#"
// leading comment
{
  "url": "https://example.test/a//b",
  "items": [
    "/* literal */",
  ],
}
"#,
        ))
        .unwrap();

        assert_eq!(value["url"], "https://example.test/a//b");
        assert_eq!(value["items"][0], "/* literal */");
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
        assert!(snapshot.nix_components.is_empty());
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

    #[test]
    fn scan_imports_yazelix_config_files_without_manifest() {
        let root = fixture_root();
        write_yazelix_fixture(&root);
        let before = std::fs::read(root.join("settings_default.jsonc")).unwrap();

        let snapshot = scan(
            CatalogScanSpec {
                repo_root: root.clone(),
                manifest_dir: root.join("missing-manifest"),
            },
            &Registry::empty(),
        )
        .unwrap();

        assert_eq!(
            std::fs::read(root.join("settings_default.jsonc")).unwrap(),
            before
        );
        assert_eq!(snapshot.components.len(), 0);
        assert!(snapshot.config_files.iter().any(|row| {
            row.path == "settings_default.jsonc"
                && row.file_kind == "yazelix_settings_default"
                && row.owner_component.as_deref() == Some("yazelix")
        }));
        assert!(snapshot.config_files.iter().any(|row| {
            row.path == "config_metadata/main_config_contract.toml"
                && row.file_kind == "yazelix_config_metadata"
                && row.parse_status == "ok"
        }));
        assert!(snapshot.config_files.iter().any(|row| {
            row.path == "configs/zellij/layouts/flexnetos_agent_workspace.kdl"
                && row.file_kind == "yazelix_runtime_config"
                && row.format == "kdl"
                && row.parse_status == "ok"
        }));
        assert!(snapshot.config_files.iter().any(|row| {
            row.path == "nushell/config/config.nu"
                && row.file_kind == "yazelix_nushell_config"
                && row.format == "nushell"
                && row.parse_status == "ok"
        }));
        assert!(snapshot
            .settings
            .iter()
            .any(|row| row.source_file == "settings_default.jsonc"
                && row.scope == "yazelix"
                && row.precedence == 90));
        assert!(snapshot.settings.iter().any(|row| {
            row.source_file == "nushell/config/config.nu"
                && row.setting_key == "source_lines"
                && row.value.contains("yazelix_init.nu")
        }));
        assert!(snapshot.settings.iter().any(|row| {
            row.source_file == "configs/zellij/layouts/flexnetos_agent_workspace.kdl"
                && row.setting_key == "node_names"
                && row.value.contains("layout")
                && row.value.contains("pane")
        }));
        for (source_file, format) in [
            ("home_manager/module.nix", "nix"),
            ("packaging/mk_runtime_tree.nix", "nix"),
            (
                "configs/terminal_emulators/kitty/kitty.conf",
                "terminal_conf",
            ),
            ("configs/yazi/plugins/sidebar-status.yazi/main.lua", "lua"),
            ("home_manager/README.md", "markdown"),
        ] {
            assert!(
                snapshot.config_files.iter().any(|row| {
                    row.path == source_file && row.format == format && row.parse_status == "ok"
                }),
                "missing parsed config row for {source_file}"
            );
            assert!(
                snapshot.settings.iter().any(|row| {
                    row.source_file == source_file
                        && row.setting_key == "reproduction_policy"
                        && row.value == "source_bytes_required"
                }),
                "missing reproduction row for {source_file}"
            );
        }
        assert!(snapshot.codedb_file_imports.iter().any(|row| {
            row.target_id == "repo_settings_default"
                && row.import_status == "blob_metadata_ready"
                && row
                    .content_hash
                    .as_deref()
                    .is_some_and(|hash| hash.len() == 64)
                && row
                    .blob_ref
                    .as_deref()
                    .is_some_and(|blob| blob.starts_with("sha256:"))
                && row.byte_length == r#"{"debug_mode":false}"#.len() as u64
                && row.last_observed.contains('T')
                && row.structured_status == "structured_rows_ready"
                && row.structured_row_count >= 1
                && row
                    .structured_rows
                    .iter()
                    .any(|structured| structured.key == "debug_mode" && structured.value == "false")
        }));
        assert!(snapshot.codedb_file_imports.iter().any(|row| {
            row.target_id == "nix_store_runtime"
                && row.source_of_truth_class == "nix_store_package_output"
                && row.import_status == "metadata_only"
                && row.content_hash.is_none()
                && row.blob_ref.is_none()
                && row.structured_status == "metadata_only"
                && row.structured_row_count == 0
                && row.skip_reason == "nix_store_metadata_only"
        }));
    }

    #[test]
    fn render_imports_yazelix_config_files_without_manifest() {
        let root = fixture_root();
        write_yazelix_fixture(&root);
        let out = fixture_root();

        let report = render(
            CatalogRenderSpec {
                repo_root: root,
                manifest_dir: out.join("missing-manifest"),
                out_dir: out.clone(),
                target_root: None,
            },
            &Registry::empty(),
        )
        .unwrap();

        assert!(!report.summary.mutating_repo);
        let config_files_path = out.join("catalog/tables/config_files.json");
        assert!(config_files_path.is_file());
        assert!(out.join("dashboard/mission-control.catalog.kdl").is_file());
        let rows: Vec<ConfigFileRow> =
            serde_json::from_slice(&std::fs::read(config_files_path).unwrap()).unwrap();
        assert!(rows.iter().any(|row| row.path == "settings_default.jsonc"
            && row.file_kind == "yazelix_settings_default"));
        assert!(out
            .join("catalog/tables/codedb_file_imports.json")
            .is_file());
    }

    #[test]
    fn diff_reports_parse_and_lock_drift_without_writes() {
        let root = fixture_root();
        write_fixture(&root);
        let bad_mcp = root.join(".mcp.json");
        std::fs::write(&bad_mcp, "{not json").unwrap();
        let before = std::fs::read(&bad_mcp).unwrap();
        let manifest_dir = root.join("manifest");
        let registry = Registry::load(&manifest_dir).unwrap();

        let report = diff(
            CatalogScanSpec {
                repo_root: root.clone(),
                manifest_dir,
            },
            &registry,
        )
        .unwrap();

        let after = std::fs::read(&bad_mcp).unwrap();
        assert_eq!(before, after, "catalog diff must not rewrite source files");
        assert!(!report.summary.mutating);
        assert!(report.summary.parse_errors >= 1, "{report:#?}");
        assert!(report.drift.iter().any(|row| {
            row.drift_kind == "parse_error" && row.subject_id == ".mcp.json" && !row.mutating
        }));
        assert!(report.summary.lock_drifts > 0, "{report:#?}");
        assert!(report
            .drift
            .iter()
            .any(|row| row.drift_kind.starts_with("lock_")));
    }

    #[test]
    fn render_writes_deterministic_projection_files_outside_repo() {
        let root = fixture_root();
        write_fixture(&root);
        let manifest_dir = root.join("manifest");
        let registry = Registry::load(&manifest_dir).unwrap();
        let out_a = fixture_root();
        let out_b = fixture_root();

        let report_a = render(
            CatalogRenderSpec {
                repo_root: root.clone(),
                manifest_dir: manifest_dir.clone(),
                out_dir: out_a.clone(),
                target_root: None,
            },
            &registry,
        )
        .unwrap();
        let report_b = render(
            CatalogRenderSpec {
                repo_root: root.clone(),
                manifest_dir,
                out_dir: out_b.clone(),
                target_root: None,
            },
            &registry,
        )
        .unwrap();

        assert!(!report_a.summary.mutating_repo);
        assert_eq!(
            report_a.summary.generated_files,
            report_b.summary.generated_files
        );
        for rel in [
            "catalog/scan.json",
            "manifest/components.catalog.toml",
            "agent-env.yaml",
            "agent-env.lock",
            ".codex/config.toml",
            ".mcp.json",
            "shell/env.catalog.sh",
            "dashboard/mission-control.catalog.kdl",
            "systemd/user/envctl-catalog-check.service",
            "catalog/rendered-config-files.json",
        ] {
            assert!(out_a.join(rel).is_file(), "missing rendered file {rel}");
            assert!(
                report_a.files.iter().any(|row| row.path == rel),
                "missing rendered report row {rel}"
            );
            assert!(
                report_a.config_files.iter().any(|row| row.path == rel),
                "missing generated config_files row {rel}"
            );
        }
        assert_eq!(
            render_tree_hashes(&out_a),
            render_tree_hashes(&out_b),
            "catalog render must be deterministic across output dirs"
        );
    }

    #[test]
    fn render_refuses_output_inside_repo() {
        let root = fixture_root();
        write_fixture(&root);
        let manifest_dir = root.join("manifest");
        let registry = Registry::load(&manifest_dir).unwrap();

        let err = render(
            CatalogRenderSpec {
                repo_root: root.clone(),
                manifest_dir,
                out_dir: root.join("rendered"),
                target_root: None,
            },
            &registry,
        )
        .unwrap_err();

        assert!(
            err.to_string().contains("outside repo root"),
            "unexpected error: {err:#}"
        );
    }

    #[test]
    fn import_report_wraps_scan_without_writes() {
        let root = fixture_root();
        write_fixture(&root);
        let agent_env = root.join("agent-env.yaml");
        let before = std::fs::read(&agent_env).unwrap();
        let manifest_dir = root.join("manifest");
        let registry = Registry::load(&manifest_dir).unwrap();

        let report = import_current(
            CatalogScanSpec {
                repo_root: root.clone(),
                manifest_dir,
            },
            &registry,
        )
        .unwrap();

        assert_eq!(std::fs::read(&agent_env).unwrap(), before);
        assert_eq!(report.generated_by, "envctl catalog import");
        assert!(!report.summary.mutating);
        assert_eq!(report.summary.tables, CatalogTableName::all().len());
        assert_eq!(report.summary.components, 2);
        assert!(report.summary.rows >= report.summary.components);
    }

    #[test]
    fn scan_projects_nix_components_into_dedicated_table() {
        let root = fixture_root();
        let manifest_dir = root.join("manifest");
        std::fs::create_dir_all(&manifest_dir).unwrap();
        std::fs::create_dir_all(manifest_dir.join("components.d")).unwrap();

        std::fs::write(
            manifest_dir.join("nix-yazelix.toml"),
            r#"
[[component]]
id = "nix"
name = "Nix"

[[component]]
id = "home-manager"
name = "home-manager"
requires = ["nix"]
"#,
        )
        .unwrap();
        std::fs::write(
            manifest_dir.join("components.d/epic-h-toolchains.toml"),
            r#"
[[component]]
id = "nix-portable"
name = "nix-portable"
"#,
        )
        .unwrap();
        std::fs::write(manifest_dir.join("envctl.lock"), "").unwrap();

        let registry = Registry::load(&manifest_dir).unwrap();
        let snapshot = scan(
            CatalogScanSpec {
                repo_root: root,
                manifest_dir,
            },
            &registry,
        )
        .unwrap();

        assert_eq!(
            snapshot.nix_components.len(),
            3,
            "{:#?}",
            snapshot.nix_components
        );
        assert!(snapshot.nix_components.iter().any(|row| {
            row.component_id == "nix"
                && row.source_file == "manifest/nix-yazelix.toml"
                && row.nix_surface == "nix_manifest"
        }));
        assert!(snapshot.nix_components.iter().any(|row| {
            row.component_id == "home-manager" && row.nix_surface == "nix_manifest"
        }));
        assert!(snapshot.nix_components.iter().any(|row| {
            row.component_id == "nix-portable"
                && row.source_file == "manifest/components.d/epic-h-toolchains.toml"
                && row.nix_surface == "nix_portable_toolchain"
        }));
        let table_value = snapshot.table_value(CatalogTableName::NixComponents);
        assert_eq!(table_value.as_array().unwrap().len(), 3);
        assert!(snapshot
            .observed_facts
            .iter()
            .any(|row| row.fact_id == "catalog.table_count.nix_components"));
    }

    #[test]
    fn render_can_retarget_layout_paths_and_env_exports() {
        let root = fixture_root();
        write_fixture(&root);
        let manifest_dir = root.join("manifest");
        let registry = Registry::load(&manifest_dir).unwrap();
        let out = fixture_root();
        let target_root = fixture_root().join("meta-root");
        std::fs::create_dir_all(&target_root).unwrap();

        let report = render(
            CatalogRenderSpec {
                repo_root: root,
                manifest_dir,
                out_dir: out.clone(),
                target_root: Some(target_root.clone()),
            },
            &registry,
        )
        .unwrap();

        assert_eq!(report.target_root, Some(target_root.display().to_string()));

        let paths: Vec<PathRow> =
            serde_json::from_slice(&std::fs::read(out.join("catalog/tables/paths.json")).unwrap())
                .unwrap();
        let env_vars: Vec<EnvVarRow> = serde_json::from_slice(
            &std::fs::read(out.join("catalog/tables/env_vars.json")).unwrap(),
        )
        .unwrap();

        let usr = paths
            .iter()
            .find(|row| row.path_id == "usr")
            .expect("usr path row");
        assert_eq!(usr.path, target_root.join("usr").display().to_string());

        let expected_usr = target_root.join("usr").display().to_string();
        let envctl_usr = env_vars
            .iter()
            .find(|row| row.var_name == "ENVCTL_USR" && row.producer == "layout")
            .expect("layout env var row");
        assert_eq!(envctl_usr.value.as_deref(), Some(expected_usr.as_str()));
        assert_eq!(
            envctl_usr.effective_value.as_deref(),
            Some(expected_usr.as_str())
        );
    }

    #[test]
    fn ingest_nix_profile_adds_frontdoor_and_store_provenance_rows() {
        let home = fixture_root();
        let profile_root = home.join(".nix-profile");
        std::fs::create_dir_all(profile_root.join("bin")).unwrap();
        std::fs::create_dir_all(profile_root.join("share/applications")).unwrap();

        let store_root = home.join("fake-store/yazelix-flexnetos-foundation");
        std::fs::create_dir_all(store_root.join("bin")).unwrap();
        std::fs::create_dir_all(store_root.join("share/applications")).unwrap();
        std::fs::write(store_root.join("bin/yzx"), "#!/bin/sh\n").unwrap();
        std::fs::write(
            store_root.join("share/applications/com.yazelix.Yazelix.Mars.desktop"),
            "[Desktop Entry]\nName=Yazelix\n",
        )
        .unwrap();

        std::os::unix::fs::symlink(store_root.join("bin/yzx"), profile_root.join("bin/yzx"))
            .unwrap();
        std::os::unix::fs::symlink(
            store_root.join("share/applications/com.yazelix.Yazelix.Mars.desktop"),
            profile_root.join("share/applications/com.yazelix.Yazelix.Mars.desktop"),
        )
        .unwrap();

        let mut profile = NixProfileList::default();
        profile.elements.insert(
            "yazelix_flexnetos_foundation".to_string(),
            NixProfileElement {
                active: true,
                attr_path: Some("packages.x86_64-linux.yazelix_flexnetos_foundation".to_string()),
                original_url: Some("path:/home/flexnetos/FlexNetOS/src/yazelix".to_string()),
                store_paths: vec![store_root.display().to_string()],
                url: Some("path:/home/flexnetos/FlexNetOS/src/yazelix".to_string()),
            },
        );

        let mut nix_components = Vec::new();
        let mut paths = Vec::new();
        ingest_nix_profile_snapshot(&home, &profile, &mut nix_components, &mut paths, 0);

        assert!(nix_components.iter().any(|row| {
            row.component_id == "profile:yazelix_flexnetos_foundation"
                && row.nix_surface == "nix_profile"
                && row.owner_component.as_deref() == Some("yazelix")
                && row.frontdoor_paths
                    == vec![
                        profile_root.join("bin/yzx").display().to_string(),
                        profile_root
                            .join("share/applications/com.yazelix.Yazelix.Mars.desktop")
                            .display()
                            .to_string(),
                    ]
                && row.store_paths == vec![store_root.display().to_string()]
        }));

        let store_path_id = nix_store_path_id(
            "yazelix_flexnetos_foundation",
            &store_root.display().to_string(),
        );
        assert!(paths.iter().any(|row| {
            row.path_id == store_path_id
                && row.path_kind == "nix_store_path"
                && row.owner_component.as_deref() == Some("yazelix")
                && row.owner_record_id.as_deref() == Some("profile:yazelix_flexnetos_foundation")
                && row.artifact_kind == "nix_store_object"
                && row.canonical
                && row.protected
        }));
        assert!(paths.iter().any(|row| {
            row.path == profile_root.join("bin/yzx").display().to_string()
                && row.path_kind == "nix_profile_frontdoor_bin"
                && row.resolved_path.as_deref()
                    == Some(&store_root.join("bin/yzx").display().to_string())
                && row.link_target_id.as_deref() == Some(store_path_id.as_str())
                && row.owner_component.as_deref() == Some("yazelix")
                && row.verification_status == "resolved"
        }));
        assert!(paths.iter().any(|row| {
            row.path
                == profile_root
                    .join("share/applications/com.yazelix.Yazelix.Mars.desktop")
                    .display()
                    .to_string()
                && row.path_kind == "nix_profile_frontdoor_desktop"
                && row.resolved_path.as_deref()
                    == Some(
                        &store_root
                            .join("share/applications/com.yazelix.Yazelix.Mars.desktop")
                            .display()
                            .to_string(),
                    )
                && row.link_target_id.as_deref() == Some(store_path_id.as_str())
                && row.owner_component.as_deref() == Some("yazelix")
        }));
    }

    #[test]
    fn scan_catalogs_envctl_home_managed_sources_and_frontdoors() {
        // scan() reads the process-global META_ROOT (via catalog_meta_root) and
        // HOME; hold test_env_lock so a concurrent env-mutating test cannot make
        // the frontdoor rows resolve against a foreign root. Fixes the parallel
        // flake at the runtime_abs assertion below.
        let _env = crate::test_env_lock();
        let root = fixture_root();
        std::fs::create_dir_all(root.join("manifest/components.d")).unwrap();
        std::fs::write(
            root.join("manifest/components.d/portability-links.toml"),
            "[[component]]\nid = \"home-config-links\"\nname = \"home-config-links\"\n\n[[component]]\nid = \"rtk-config-links\"\nname = \"rtk-config-links\"\n\n[[component]]\nid = \"claude-global-links\"\nname = \"claude-global-links\"\n",
        )
        .unwrap();
        write_envctl_home_fixture(&root);

        let snapshot = scan(
            CatalogScanSpec {
                repo_root: root.clone(),
                manifest_dir: root.join("manifest"),
            },
            &Registry::empty(),
        )
        .unwrap();

        let source_rel = "home/.config/yazelix/settings.jsonc";
        let source_abs = root.join(source_rel).display().to_string();
        let runtime_abs = root
            .join(".config/yazelix/settings.jsonc")
            .display()
            .to_string();
        let source_id = envctl_home_source_path_id(source_rel);

        assert!(snapshot.config_files.iter().any(|row| {
            row.path == source_rel
                && row.file_kind == "envctl_home_yazelix_config"
                && row.owner_component.as_deref() == Some("home-config-links")
                && row.parse_status == "ok"
        }));
        assert!(snapshot.config_files.iter().any(|row| {
            row.path == "home/.config/nushell/meta-usr-path.nu"
                && row.file_kind == "envctl_home_nushell_path"
                && row.owner_component.as_deref() == Some("home-config-links")
        }));
        assert!(snapshot.config_files.iter().any(|row| {
            row.path == "home/.claude/settings.json"
                && row.file_kind == "envctl_home_claude_config"
                && row.owner_component.as_deref() == Some("claude-global-links")
        }));
        assert!(snapshot.config_files.iter().any(|row| {
            row.path == "home/.config/rtk/config.toml"
                && row.file_kind == "envctl_home_rtk_config"
                && row.owner_component.as_deref() == Some("rtk-config-links")
        }));
        assert!(snapshot.settings.iter().any(|row| {
            row.source_file == source_rel
                && row.scope == "yazelix"
                && row.precedence == 88
                && row.setting_key == "default_shell"
                && row.value == "nu"
        }));
        assert!(snapshot.settings.iter().any(|row| {
            row.source_file == "home/.config/nushell/meta-usr-path.nu"
                && row.scope == "shell"
                && row.setting_key == "source_lines"
                && row.value.contains("meta-usr-path")
        }));
        assert!(snapshot.paths.iter().any(|row| {
            row.path_id == source_id
                && row.path == source_abs
                && row.path_kind == "envctl_home_source"
                && row.owner_component.as_deref() == Some("home-config-links")
                && row.verification_status == "tracked"
        }));
        assert!(snapshot.paths.iter().any(|row| {
            row.path == runtime_abs
                && row.path_kind == "envctl_home_yazelix_frontdoor"
                && row.owner_component.as_deref() == Some("home-config-links")
                && row.resolved_path.as_deref() == Some(source_abs.as_str())
                && row.link_target_id.as_deref() == Some(source_id.as_str())
                && row.bridge
                && row.protected
                && row.verification_status == "declared"
        }));
        assert!(snapshot.paths.iter().any(|row| {
            row.path == root.join(".claude/settings.json").display().to_string()
                && row.path_kind == "envctl_home_claude_frontdoor"
                && row.owner_component.as_deref() == Some("claude-global-links")
                && row.artifact_kind == "envctl_managed_agent_config"
        }));
        assert!(snapshot.paths.iter().any(|row| {
            row.path == root.join(".config/rtk/config.toml").display().to_string()
                && row.path_kind == "envctl_home_rtk_frontdoor"
                && row.owner_component.as_deref() == Some("rtk-config-links")
                && row.artifact_kind == "envctl_managed_runtime_config"
        }));
        assert!(snapshot.paths.iter().any(|row| {
            row.path
                == root
                    .join(".config/systemd/user/env-ctl.service")
                    .display()
                    .to_string()
                && row.path_kind == "envctl_home_frontdoor"
                && row.owner_component.as_deref() == Some("home-config-links")
                && row.artifact_kind == "envctl_managed_systemd_user_unit"
        }));
    }

    #[test]
    fn sync_preview_reports_actions_without_mutation() {
        let root = fixture_root();
        write_fixture(&root);
        let lock_path = root.join("manifest/envctl.lock");
        let before = std::fs::read(&lock_path).unwrap();
        let manifest_dir = root.join("manifest");
        let registry = Registry::load(&manifest_dir).unwrap();
        let out = fixture_root();

        let report = sync(
            CatalogSyncSpec {
                repo_root: root.clone(),
                manifest_dir,
                render_out_dir: Some(out.clone()),
                apply: false,
            },
            &registry,
        )
        .unwrap();

        assert_eq!(std::fs::read(&lock_path).unwrap(), before);
        assert!(!report.summary.mutating);
        assert!(!report.summary.applied);
        assert_eq!(report.summary.verifier_status, "preview_only");
        assert!(report.summary.planned_actions > 0, "{report:#?}");
        assert!(report
            .planned_actions
            .iter()
            .any(|row| row.action_kind == "catalog_lock_apply"));
        assert!(report
            .planned_actions
            .iter()
            .any(|row| row.action_kind == "review_render_projection"));
        assert!(report
            .render
            .as_ref()
            .is_some_and(|render| render.summary.generated_files > 0));
        assert!(out.join("catalog/scan.json").is_file());
    }

    #[test]
    fn sync_apply_refuses_without_verifier_gate() {
        let root = fixture_root();
        write_fixture(&root);
        let manifest_dir = root.join("manifest");
        let registry = Registry::load(&manifest_dir).unwrap();

        let err = sync(
            CatalogSyncSpec {
                repo_root: root,
                manifest_dir,
                render_out_dir: None,
                apply: true,
            },
            &registry,
        )
        .unwrap_err();

        assert!(
            err.to_string().contains("verifier-gated"),
            "unexpected error: {err:#}"
        );
    }

    #[test]
    fn lock_check_reports_drift_without_writes() {
        let root = fixture_root();
        write_fixture(&root);
        let lock_path = root.join("manifest/envctl.lock");
        let before = std::fs::read(&lock_path).unwrap();
        let manifest_dir = root.join("manifest");
        let registry = Registry::load(&manifest_dir).unwrap();

        let report = lock(
            CatalogLockSpec {
                repo_root: root.clone(),
                manifest_dir,
                apply: false,
            },
            &registry,
        )
        .unwrap();

        assert_eq!(std::fs::read(&lock_path).unwrap(), before);
        assert!(!report.summary.mutating);
        assert!(!report.summary.applied);
        assert!(!report.summary.lock_written);
        assert!(report.summary.before_drifts > 0, "{report:#?}");
        assert_eq!(report.summary.before_drifts, report.summary.after_drifts);
        assert_eq!(report.before_sha256, report.after_sha256);
    }

    #[test]
    fn lock_apply_updates_envctl_lock() {
        let root = fixture_root();
        write_fixture(&root);
        let lock_path = root.join("manifest/envctl.lock");
        let before = std::fs::read_to_string(&lock_path).unwrap();
        let manifest_dir = root.join("manifest");
        let registry = Registry::load(&manifest_dir).unwrap();

        let report = lock(
            CatalogLockSpec {
                repo_root: root.clone(),
                manifest_dir: manifest_dir.clone(),
                apply: true,
            },
            &registry,
        )
        .unwrap();

        let after = std::fs::read_to_string(&lock_path).unwrap();
        assert_ne!(before, after);
        assert!(report.summary.mutating);
        assert!(report.summary.applied);
        assert!(report.summary.lock_written);
        assert!(report.summary.before_drifts > 0, "{report:#?}");
        assert_eq!(report.summary.after_drifts, 0);
        assert_ne!(report.before_sha256, report.after_sha256);
        let lock_file = LockFile::load(&manifest_dir).unwrap();
        assert!(lock::diff(&registry, &lock_file).is_empty());
    }

    fn render_tree_hashes(root: &Path) -> std::collections::BTreeMap<String, String> {
        fn visit(base: &Path, dir: &Path, out: &mut std::collections::BTreeMap<String, String>) {
            let mut entries = std::fs::read_dir(dir)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect::<Vec<_>>();
            entries.sort();
            for path in entries {
                if path.is_dir() {
                    visit(base, &path, out);
                } else {
                    let rel = path
                        .strip_prefix(base)
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    let bytes = std::fs::read(&path).unwrap();
                    out.insert(rel, sha256_hex(&bytes));
                }
            }
        }

        let mut out = std::collections::BTreeMap::new();
        visit(root, root, &mut out);
        out
    }

    fn fixture_root() -> PathBuf {
        static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let seq = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("envctl-catalog-test-{id}-{seq}"));
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
            r#"{"mcpServers":{"github":{"command":"github"},"memory":{"command":"bash","args":["-lc","ROOT=\"${META_ROOT:-/fixture}\"; exec \"$ROOT/envctl/assets/scripts/envctl-mcp-memory-server\""],"env":{"META_ROOT":"/fixture"}}}}"#,
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

    fn write_envctl_home_fixture(root: &Path) {
        std::fs::create_dir_all(root.join("home/.claude/commands")).unwrap();
        std::fs::create_dir_all(root.join("home/.config/yazelix/configs/zellij/layouts")).unwrap();
        std::fs::create_dir_all(root.join("home/.config/yazelix/helix/steel_plugins")).unwrap();
        std::fs::create_dir_all(root.join("home/.config/ghostty")).unwrap();
        std::fs::create_dir_all(root.join("home/.config/kasetto")).unwrap();
        std::fs::create_dir_all(root.join("home/.config/nushell")).unwrap();
        std::fs::create_dir_all(root.join("home/.config/rtk")).unwrap();
        std::fs::create_dir_all(root.join("home/.config/systemd/user")).unwrap();
        std::fs::write(root.join("home/.gitconfig"), "[user]\nname = Envctl\n").unwrap();
        std::fs::write(
            root.join("home/.claude/settings.json"),
            "{\n  \"model\": \"opus\"\n}\n",
        )
        .unwrap();
        std::fs::write(root.join("home/.claude/CLAUDE.md"), "# claude\n").unwrap();
        std::fs::write(
            root.join("home/.claude/commands/remember.md"),
            "# remember\n",
        )
        .unwrap();
        std::fs::write(
            root.join("home/.config/ghostty/config.ghostty"),
            "theme = dusk\n",
        )
        .unwrap();
        std::fs::write(
            root.join("home/.config/kasetto/kasetto.yaml"),
            "agent: codex\n",
        )
        .unwrap();
        std::fs::write(
            root.join("home/.config/yazelix/settings.jsonc"),
            "{\n  \"default_shell\": \"nu\"\n}\n",
        )
        .unwrap();
        std::fs::write(
            root.join("home/.config/yazelix/shell_bash.sh"),
            "export YAZELIX_ACTIVE=1\n",
        )
        .unwrap();
        std::fs::write(
            root.join("home/.config/yazelix/shell_nu.nu"),
            "source ./meta-usr-path.nu\n",
        )
        .unwrap();
        std::fs::write(
            root.join("home/.config/yazelix/terminal_ghostty.conf"),
            "theme = dawn\n",
        )
        .unwrap();
        std::fs::write(
            root.join("home/.config/yazelix/tombi.toml"),
            "[ui]\nlayout = \"stack\"\n",
        )
        .unwrap();
        std::fs::write(
            root.join("home/.config/yazelix/helix/steel_plugins/README.md"),
            "# steel plugins\n",
        )
        .unwrap();
        std::fs::write(
            root.join("home/.config/yazelix/configs/zellij/layouts/mission-control.kdl"),
            "layout {\n    pane\n}\n",
        )
        .unwrap();
        std::fs::write(
            root.join("home/.config/nushell/meta-usr-path.nu"),
            "source ./meta-usr-path.nu\n",
        )
        .unwrap();
        std::fs::write(
            root.join("home/.config/nushell/config.nu"),
            "source ./meta-usr-path.nu\n",
        )
        .unwrap();
        std::fs::write(
            root.join("home/.config/rtk/config.toml"),
            "[display]\nmode = \"compact\"\n",
        )
        .unwrap();
        std::fs::write(
            root.join("home/.config/systemd/user/env-ctl.service"),
            "[Unit]\nDescription=envctl\n",
        )
        .unwrap();
    }

    fn write_yazelix_fixture(root: &Path) {
        std::fs::create_dir_all(root.join("config_metadata")).unwrap();
        std::fs::create_dir_all(root.join("configs/zellij/layouts")).unwrap();
        std::fs::create_dir_all(root.join("configs/helix")).unwrap();
        std::fs::create_dir_all(root.join("configs/terminal_emulators/kitty")).unwrap();
        std::fs::create_dir_all(root.join("configs/yazi/plugins/sidebar-status.yazi")).unwrap();
        std::fs::create_dir_all(root.join("nushell/config")).unwrap();
        std::fs::create_dir_all(root.join("home_manager")).unwrap();
        std::fs::create_dir_all(root.join("packaging")).unwrap();
        std::fs::create_dir_all(root.join("docs/generated")).unwrap();
        std::fs::create_dir_all(root.join("rust_core/yazelix_zellij_config_pack/layouts")).unwrap();

        std::fs::write(
            root.join("settings_default.jsonc"),
            r#"{"debug_mode":false}"#,
        )
        .unwrap();
        std::fs::write(
            root.join("config_metadata/main_config_contract.toml"),
            r#"
[[field]]
key = "debug_mode"
default = false
"#,
        )
        .unwrap();
        std::fs::write(
            root.join("config_metadata/yazelix_settings.schema.json"),
            r#"{"type":"object"}"#,
        )
        .unwrap();
        std::fs::write(
            root.join("configs/zellij/layouts/flexnetos_agent_workspace.kdl"),
            r#"layout {
    tab name="agent" {
        pane command="nu"
    }
}
"#,
        )
        .unwrap();
        std::fs::write(
            root.join("configs/helix/yazelix_config.toml"),
            "theme = \"zed\"\n",
        )
        .unwrap();
        std::fs::write(
            root.join("configs/terminal_emulators/kitty/kitty.conf"),
            "font_family JetBrainsMono Nerd Font\n",
        )
        .unwrap();
        std::fs::write(
            root.join("configs/yazi/plugins/sidebar-status.yazi/main.lua"),
            "return { setup = function() end }\n",
        )
        .unwrap();
        std::fs::write(
            root.join("nushell/config/config.nu"),
            r#"source /opt/yazelix/share/initializers/nushell/yazelix_init.nu
use /opt/yazelix/share/completions/yazelix_extern.nu *
alias yzx = yazelix
def yzx_ready [] { true }
$env.config.show_banner = false
"#,
        )
        .unwrap();
        std::fs::write(
            root.join("home_manager/module.nix"),
            "{ config, ... }: {}\n",
        )
        .unwrap();
        std::fs::write(root.join("home_manager/README.md"), "# Home Manager\n").unwrap();
        std::fs::write(root.join("packaging/mk_runtime_tree.nix"), "{ }:\n").unwrap();
        std::fs::write(
            root.join("rust_core/yazelix_zellij_config_pack/layouts/yzx_side.kdl"),
            "layout {}\n",
        )
        .unwrap();
        std::fs::write(
            root.join("docs/generated/yazelix_file_target_inventory.json"),
            format!(
                r#"[
  {{
    "target_id": "repo_settings_default",
    "absolute_path": "{}",
    "normalized_logical_path": "repo_source:settings_default.jsonc",
    "owner": "yazelix",
    "source_of_truth_class": "repo_source",
    "exists": true,
    "file_kind": "regular_file",
    "parser_hint": "jsonc",
    "mutability": "source_controlled",
    "reproduction_policy": "git_checkout",
    "safety_policy": "source_content_import_allowed",
    "import_mode": "content_blob"
  }},
  {{
    "target_id": "nix_store_runtime",
    "absolute_path": "/nix/store/example-yazelix-runtime",
    "normalized_logical_path": "nix_store:/nix/store/example-yazelix-runtime",
    "owner": "nix",
    "source_of_truth_class": "nix_store_package_output",
    "exists": true,
    "file_kind": "package_output",
    "parser_hint": "nix_store_path",
    "mutability": "immutable_store",
    "reproduction_policy": "nix_realise",
    "safety_policy": "nix_store_metadata_only",
    "import_mode": "metadata_only"
  }},
  {{
    "target_id": "local_state",
    "absolute_path": "{}/.local/share/yazelix/state.json",
    "normalized_logical_path": "real_home_runtime_state:.local/share/yazelix/state.json",
    "owner": "user",
    "source_of_truth_class": "real_home_runtime_state",
    "exists": true,
    "file_kind": "regular_file",
    "parser_hint": "json",
    "mutability": "user_state",
    "reproduction_policy": "runtime_observed",
    "safety_policy": "real_home_metadata_only",
    "import_mode": "metadata_only"
  }}
]
"#,
                root.join("settings_default.jsonc").display(),
                root.display()
            ),
        )
        .unwrap();
    }
}
