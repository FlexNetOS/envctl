//! Migration/adoption engine for moving an existing meta checkout into envctl's
//! canonical `$META_ROOT` FHS/XDG topology (`usr`, `etc`, `var`, `opt`, `run`, `tmp`, plus meta-home XDG roots).
//!
//! This module is deliberately conservative:
//! - scan/plan/verify are read-only;
//! - apply is a preview unless the caller passes `apply = true`;
//! - purge never deletes an arbitrary legacy path. A legacy path must first be
//!   proven adopted into a canonical replacement and then appear as a typed
//!   purge candidate. The first implementation only reports and refuses unsafe
//!   purges, so it upgrades the workflow without downgrading existing installs.
use crate::event::{Event, EventSink};
use crate::layout::{LayoutKind, MetaLayout};
use crate::model::Registry;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MigrationSpec {
    #[serde(default)]
    pub scopes: Vec<MigrationScope>,
    #[serde(default)]
    pub components: Vec<String>,
}

impl Default for MigrationSpec {
    fn default() -> Self {
        Self {
            scopes: vec![MigrationScope::All],
            components: Vec::new(),
        }
    }
}

impl MigrationSpec {
    pub fn wants(&self, scope: MigrationScope) -> bool {
        self.scopes.is_empty()
            || self.scopes.contains(&MigrationScope::All)
            || self.scopes.contains(&scope)
    }

    fn component_filter(&self) -> Option<BTreeSet<&str>> {
        if self.components.is_empty() {
            None
        } else {
            Some(self.components.iter().map(String::as_str).collect())
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationScope {
    All,
    Layout,
    ComponentRegistry,
    AgentAssets,
    MetaSubstrates,
    LegacyPaths,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationVerb {
    Scan,
    Plan,
    Apply,
    Verify,
    Purge,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MigrationReport {
    pub schema: String,
    pub verb: MigrationVerb,
    pub meta_root: String,
    pub project_root: String,
    pub manifest_dir: String,
    pub ledger_path: String,
    pub archive_root: String,
    pub layout: Vec<MigrationLayoutEntry>,
    pub items: Vec<MigrationItem>,
    pub summary: MigrationSummary,
}

impl MigrationReport {
    pub fn ok(&self) -> bool {
        self.summary.refused == 0
            && self.summary.errors == 0
            && (!matches!(self.verb, MigrationVerb::Verify) || self.summary.needs_migration == 0)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MigrationLayoutEntry {
    pub key: String,
    pub path: String,
    pub kind: MigrationLayoutKind,
    pub purpose: String,
    pub exists: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationLayoutKind {
    Canonical,
    LegacyCompatibility,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MigrationItem {
    pub id: String,
    pub owner: MigrationOwner,
    pub kind: MigrationKind,
    pub status: MigrationStatus,
    pub action: MigrationAction,
    pub risk: MigrationRisk,
    pub subject: String,
    pub detail: String,
    pub source: Option<String>,
    pub component: Option<String>,
    pub canonical: Option<String>,
    pub legacy: Option<String>,
    pub protected: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationOwner {
    LayoutRegistry,
    ComponentRegistry,
    AgentAssets,
    MetaSubstrate,
    LegacyPath,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationKind {
    CanonicalDirectory,
    ManifestLegacyToken,
    UserGlobalPath,
    SystemGlobalPath,
    AgentConfig,
    HarnessException,
    MetaSharedLibrary,
    LegacyCompatibilityRoot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationStatus {
    Current,
    NeedsMigration,
    MissingCanonical,
    Materialized,
    Preserved,
    Protected,
    ReportOnly,
    Refused,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationAction {
    None,
    MaterializeCanonicalDir,
    UpdateManifestToCanonicalLayout,
    AdoptIntoMetaLocal,
    PreserveConfig,
    ProtectSubstrate,
    ReportOnly,
    RefusePurge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationRisk {
    Low,
    Medium,
    High,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MigrationSummary {
    pub total: usize,
    pub current: usize,
    pub needs_migration: usize,
    pub missing_canonical: usize,
    pub materialized: usize,
    pub preserved: usize,
    pub protected: usize,
    pub report_only: usize,
    pub refused: usize,
    pub errors: usize,
    pub highest_risk: Option<MigrationRisk>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MigrationLedgerEntry {
    pub schema: String,
    pub at: String,
    pub verb: MigrationVerb,
    pub meta_root: String,
    pub summary: MigrationSummary,
}

pub fn scan(registry: &Registry, manifest_dir: &Path, spec: &MigrationSpec) -> MigrationReport {
    let roots = MigrationRoots::resolve(manifest_dir);
    let layout = MetaLayout::from_meta_root(&roots.meta_root);
    let mut items = Vec::new();

    if spec.wants(MigrationScope::Layout) {
        collect_layout_items(&layout, &mut items);
    }
    if spec.wants(MigrationScope::ComponentRegistry) || spec.wants(MigrationScope::LegacyPaths) {
        collect_manifest_items(registry, manifest_dir, spec, &layout, &mut items);
    }
    if spec.wants(MigrationScope::AgentAssets) {
        collect_agent_asset_items(&roots.project_root, &mut items);
    }
    if spec.wants(MigrationScope::MetaSubstrates) {
        collect_meta_substrate_items(&roots.project_root, &mut items);
    }

    let mut report = report_from_parts(MigrationVerb::Scan, roots, layout_entries(&layout), items);
    report.summary = summarize(&report.items);
    report
}

pub fn plan(registry: &Registry, manifest_dir: &Path, spec: &MigrationSpec) -> MigrationReport {
    let mut report = scan(registry, manifest_dir, spec);
    report.verb = MigrationVerb::Plan;
    report
}

pub fn apply(
    registry: &Registry,
    manifest_dir: &Path,
    spec: &MigrationSpec,
    apply: bool,
) -> anyhow::Result<MigrationReport> {
    let roots = MigrationRoots::resolve(manifest_dir);
    let layout = MetaLayout::from_meta_root(&roots.meta_root);
    if apply {
        layout.ensure_dirs()?;
    }
    let mut report = scan(registry, manifest_dir, spec);
    report.verb = MigrationVerb::Apply;
    if apply {
        for item in &mut report.items {
            if item.action == MigrationAction::MaterializeCanonicalDir {
                item.status = MigrationStatus::Materialized;
            }
        }
        report.summary = summarize(&report.items);
        append_ledger(&report)?;
    }
    Ok(report)
}

pub fn verify(registry: &Registry, manifest_dir: &Path, spec: &MigrationSpec) -> MigrationReport {
    let mut report = scan(registry, manifest_dir, spec);
    report.verb = MigrationVerb::Verify;
    report
}

pub fn purge(
    registry: &Registry,
    manifest_dir: &Path,
    spec: &MigrationSpec,
    apply: bool,
    confirmed: bool,
) -> anyhow::Result<MigrationReport> {
    let mut report = scan(registry, manifest_dir, spec);
    report.verb = MigrationVerb::Purge;
    let mut refused = MigrationItem {
        id: "purge-refused".to_string(),
        owner: MigrationOwner::LegacyPath,
        kind: MigrationKind::LegacyCompatibilityRoot,
        status: MigrationStatus::Refused,
        action: MigrationAction::RefusePurge,
        risk: MigrationRisk::High,
        subject: "legacy path purge".to_string(),
        detail: if apply && confirmed {
            "no legacy path has a verified canonical replacement in the migration ledger; refusing to delete"
                .to_string()
        } else {
            "purge is dry-run by default and requires --apply --confirm plus verified adopted candidates"
                .to_string()
        },
        source: None,
        component: None,
        canonical: None,
        legacy: Some(
            MetaLayout::from_meta_root(&MigrationRoots::resolve(manifest_dir).meta_root)
                .legacy_toolchains()
                .display()
                .to_string(),
        ),
        protected: true,
    };
    if !apply || !confirmed {
        refused.status = MigrationStatus::ReportOnly;
        refused.action = MigrationAction::ReportOnly;
    }
    report.items.push(refused);
    report.summary = summarize(&report.items);
    if apply && confirmed {
        append_ledger(&report)?;
    }
    Ok(report)
}

pub fn emit_report(report: &MigrationReport, sink: &EventSink) {
    sink.emit(Event::MigrationReported {
        report: Box::new(report.clone()),
    });
}

fn collect_layout_items(layout: &MetaLayout, items: &mut Vec<MigrationItem>) {
    for entry in layout.entries() {
        let exists = entry.path.exists();
        match entry.kind {
            LayoutKind::Canonical => items.push(MigrationItem {
                id: format!("layout:{}", entry.key),
                owner: MigrationOwner::LayoutRegistry,
                kind: MigrationKind::CanonicalDirectory,
                status: if exists {
                    MigrationStatus::Current
                } else {
                    MigrationStatus::MissingCanonical
                },
                action: if exists {
                    MigrationAction::None
                } else {
                    MigrationAction::MaterializeCanonicalDir
                },
                risk: MigrationRisk::Low,
                subject: entry.key.to_string(),
                detail: entry.purpose.to_string(),
                source: None,
                component: None,
                canonical: Some(entry.path.display().to_string()),
                legacy: None,
                protected: false,
            }),
            LayoutKind::LegacyCompatibility => items.push(MigrationItem {
                id: format!("layout:{}", entry.key),
                owner: MigrationOwner::LayoutRegistry,
                kind: MigrationKind::LegacyCompatibilityRoot,
                status: MigrationStatus::Protected,
                action: MigrationAction::ReportOnly,
                risk: MigrationRisk::Low,
                subject: entry.key.to_string(),
                detail: "compatibility root is tracked but never treated as the canonical install target"
                    .to_string(),
                source: None,
                component: None,
                canonical: None,
                legacy: Some(entry.path.display().to_string()),
                protected: true,
            }),
        }
    }
}

fn collect_manifest_items(
    registry: &Registry,
    manifest_dir: &Path,
    spec: &MigrationSpec,
    layout: &MetaLayout,
    items: &mut Vec<MigrationItem>,
) {
    let component_filter = spec.component_filter();
    let registry_ids: BTreeSet<&str> = registry.ordered().map(|c| c.id.as_str()).collect();
    for path in manifest_files(manifest_dir) {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let mut current_component: Option<String> = None;
        for (idx, line) in text.lines().enumerate() {
            if let Some(id) = parse_id_line(line) {
                current_component = Some(id.to_string());
            }
            let component = current_component
                .as_ref()
                .filter(|id| registry_ids.contains(id.as_str()))
                .cloned();
            if let Some(filter) = &component_filter {
                if !component
                    .as_deref()
                    .map(|id| filter.contains(id))
                    .unwrap_or(false)
                {
                    continue;
                }
            }
            for hit in classify_line(line) {
                items.push(MigrationItem {
                    id: format!(
                        "manifest:{}:{}:{}",
                        path.file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("manifest"),
                        idx + 1,
                        hit.id
                    ),
                    owner: MigrationOwner::ComponentRegistry,
                    kind: hit.kind,
                    status: MigrationStatus::NeedsMigration,
                    action: hit.action,
                    risk: hit.risk,
                    subject: hit.subject.to_string(),
                    detail: hit.detail.to_string(),
                    source: Some(format!("{}:{}", path.display(), idx + 1)),
                    component: component.clone(),
                    canonical: Some(layout.meta_root().display().to_string()),
                    legacy: Some(line.trim().to_string()),
                    protected: false,
                });
            }
        }
    }
}

fn collect_agent_asset_items(project_root: &Path, items: &mut Vec<MigrationItem>) {
    let assets = [
        ("agent-env.yaml", "agent-env declarative config"),
        ("agent-env.lock", "agent-env resolved lock"),
        (".codex/config.toml", "Codex MCP baseline"),
        (".mcp.json", "MCP baseline"),
        (".Codex/skills", "hand-authored/ejected harness skills"),
        (".Codex/agents", "hand-authored/ejected harness agents"),
        (".agents/skills", "Codex-visible ejected skills mirror"),
    ];
    for (rel, detail) in assets {
        let path = project_root.join(rel);
        if path.exists() {
            let harness_exception = rel.starts_with(".Codex") || rel.starts_with(".agents");
            items.push(MigrationItem {
                id: format!("agent-assets:{rel}"),
                owner: MigrationOwner::AgentAssets,
                kind: if harness_exception {
                    MigrationKind::HarnessException
                } else {
                    MigrationKind::AgentConfig
                },
                status: if harness_exception {
                    MigrationStatus::Protected
                } else {
                    MigrationStatus::Preserved
                },
                action: MigrationAction::PreserveConfig,
                risk: MigrationRisk::Low,
                subject: rel.to_string(),
                detail: format!("{detail}; preserve/adopt in place, never rebuild blindly"),
                source: Some(path.display().to_string()),
                component: None,
                canonical: Some(path.display().to_string()),
                legacy: None,
                protected: true,
            });
        }
    }
}

fn collect_meta_substrate_items(project_root: &Path, items: &mut Vec<MigrationItem>) {
    for rel in ["Cargo.toml", "crates/engine/Cargo.toml"] {
        let path = project_root.join(rel);
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        if text.contains("loop_lib") {
            items.push(MigrationItem {
                id: format!("meta-substrate:{rel}:loop_lib"),
                owner: MigrationOwner::MetaSubstrate,
                kind: MigrationKind::MetaSharedLibrary,
                status: MigrationStatus::Protected,
                action: MigrationAction::ProtectSubstrate,
                risk: MigrationRisk::Low,
                subject: "loop_lib".to_string(),
                detail: "meta shared command-construction substrate; envctl may upgrade/build on it but must not remove or downgrade it"
                    .to_string(),
                source: Some(path.display().to_string()),
                component: None,
                canonical: Some("meta/loop_lib".to_string()),
                legacy: None,
                protected: true,
            });
        }
    }
}

struct LineHit {
    id: &'static str,
    kind: MigrationKind,
    action: MigrationAction,
    risk: MigrationRisk,
    subject: &'static str,
    detail: &'static str,
}

fn classify_line(line: &str) -> Vec<LineHit> {
    let trimmed = line.trim();
    if trimmed.starts_with('#') {
        return Vec::new();
    }
    let mut hits = Vec::new();
    if trimmed.contains(".toolchains") || trimmed.contains("ENVCTL_LEGACY_TOOLCHAINS") {
        hits.push(LineHit {
            id: "toolchains",
            kind: MigrationKind::ManifestLegacyToken,
            action: MigrationAction::UpdateManifestToCanonicalLayout,
            risk: MigrationRisk::Medium,
            subject: ".toolchains reference",
            detail: "manifest still points at the compatibility prefix; update hook/wiring to MetaLayout usr/var/opt/XDG paths",
        });
    }
    if trimmed.contains("~/.local")
        || trimmed.contains("$HOME/.local")
        || trimmed.contains("${HOME}/.local")
        || trimmed.contains("%h/.local")
    {
        hits.push(LineHit {
            id: "home-local",
            kind: MigrationKind::UserGlobalPath,
            action: MigrationAction::AdoptIntoMetaLocal,
            risk: MigrationRisk::Medium,
            subject: "user-global .local reference",
            detail: "the real-home .local tree carries Yazelix-owned Nix profile state; envctl-owned payloads belong under META_ROOT usr/var/opt or XDG roots",
        });
    }
    if trimmed.contains("/usr/local") || trimmed.contains("/opt/") {
        hits.push(LineHit {
            id: "system-global",
            kind: MigrationKind::SystemGlobalPath,
            action: MigrationAction::ReportOnly,
            risk: MigrationRisk::High,
            subject: "system/global path reference",
            detail: "system-depth path must be component-owned and explicitly justified before migration/purge",
        });
    }
    hits
}

fn parse_id_line(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix("id")?.trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    rest.split('"').next()
}

fn manifest_files(manifest_dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for dir in [
        manifest_dir.to_path_buf(),
        manifest_dir.join("components.d"),
    ] {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for entry in rd.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    files.push(path);
                }
            }
        }
    }
    files.sort();
    files
}

fn layout_entries(layout: &MetaLayout) -> Vec<MigrationLayoutEntry> {
    layout
        .entries()
        .into_iter()
        .map(|entry| MigrationLayoutEntry {
            key: entry.key.to_string(),
            exists: entry.path.exists(),
            path: entry.path.display().to_string(),
            kind: match entry.kind {
                LayoutKind::Canonical => MigrationLayoutKind::Canonical,
                LayoutKind::LegacyCompatibility => MigrationLayoutKind::LegacyCompatibility,
            },
            purpose: entry.purpose.to_string(),
        })
        .collect()
}

fn report_from_parts(
    verb: MigrationVerb,
    roots: MigrationRoots,
    layout: Vec<MigrationLayoutEntry>,
    items: Vec<MigrationItem>,
) -> MigrationReport {
    let ledger_path = ledger_path(&roots.meta_root);
    let archive_root = archive_root(&roots.meta_root);
    let summary = summarize(&items);
    MigrationReport {
        schema: "envctl.migration.report.v1".to_string(),
        verb,
        meta_root: roots.meta_root.display().to_string(),
        project_root: roots.project_root.display().to_string(),
        manifest_dir: roots.manifest_dir.display().to_string(),
        ledger_path: ledger_path.display().to_string(),
        archive_root: archive_root.display().to_string(),
        layout,
        items,
        summary,
    }
}

fn summarize(items: &[MigrationItem]) -> MigrationSummary {
    let mut s = MigrationSummary {
        total: items.len(),
        ..MigrationSummary::default()
    };
    for item in items {
        match item.status {
            MigrationStatus::Current => s.current += 1,
            MigrationStatus::NeedsMigration => s.needs_migration += 1,
            MigrationStatus::MissingCanonical => s.missing_canonical += 1,
            MigrationStatus::Materialized => s.materialized += 1,
            MigrationStatus::Preserved => s.preserved += 1,
            MigrationStatus::Protected => s.protected += 1,
            MigrationStatus::ReportOnly => s.report_only += 1,
            MigrationStatus::Refused => s.refused += 1,
        }
        s.highest_risk = Some(match s.highest_risk {
            Some(risk) => risk.max(item.risk),
            None => item.risk,
        });
    }
    s
}

#[derive(Clone, Debug)]
struct MigrationRoots {
    meta_root: PathBuf,
    project_root: PathBuf,
    manifest_dir: PathBuf,
}

impl MigrationRoots {
    fn resolve(manifest_dir: &Path) -> Self {
        let manifest_dir = absolutize(manifest_dir);
        let project_root = manifest_dir
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let meta_root = std::env::var_os("META_ROOT")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| find_up_marker(&project_root, ".meta.yaml"))
            .or_else(|| find_up_marker(&std::env::current_dir().ok()?, ".meta.yaml"))
            .unwrap_or_else(|| project_root.clone());
        Self {
            meta_root,
            project_root,
            manifest_dir,
        }
    }
}

fn absolutize(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

fn find_up_marker(start: &Path, marker: &str) -> Option<PathBuf> {
    let mut cur = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };
    loop {
        if cur.join(marker).exists() {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}

fn ledger_path(meta_root: &Path) -> PathBuf {
    meta_root.join("var/lib/envctl/migrations/ledger.jsonl")
}

fn archive_root(meta_root: &Path) -> PathBuf {
    meta_root.join("var/lib/envctl/legacy-archives")
}

fn append_ledger(report: &MigrationReport) -> anyhow::Result<()> {
    let path = PathBuf::from(&report.ledger_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let entry = MigrationLedgerEntry {
        schema: "envctl.migration.ledger.v1".to_string(),
        at: chrono::Utc::now().to_rfc3339(),
        verb: report.verb,
        meta_root: report.meta_root.clone(),
        summary: report.summary.clone(),
    };
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{}", serde_json::to_string(&entry)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DryRunRunner, Engine};

    #[test]
    fn scan_reports_layout_and_protects_loop_lib() {
        let root = tempdir("migration-scan");
        let manifest = root.join("manifest");
        std::fs::create_dir_all(&manifest).unwrap();
        let legacy_home_local = ["~", ".local/bin/foo"].join("/");
        let legacy_usr_local = ["/usr/local/bin", "foo"].join("/");
        std::fs::write(
            manifest.join("base.toml"),
            format!(
                r#"
[[component]]
id = "legacy"
name = "Legacy"
[component.install]
kind = "script"
script = "mkdir -p $META_ROOT/.toolchains/legacy && ln -s {legacy_home_local} {legacy_usr_local}"
"#
            ),
        )
        .unwrap();
        std::fs::create_dir_all(root.join("crates/engine")).unwrap();
        std::fs::write(
            root.join("crates/engine/Cargo.toml"),
            "loop_lib = { path = \"../../../loop_lib\" }\n",
        )
        .unwrap();
        let _guard = crate::test_env_lock();
        std::env::set_var("META_ROOT", &root);
        let engine = Engine::with_runner(manifest.clone(), Box::new(DryRunRunner)).unwrap();

        let report = scan(engine.registry(), &manifest, &MigrationSpec::default());

        assert!(report
            .items
            .iter()
            .any(|i| i.action == MigrationAction::MaterializeCanonicalDir));
        assert!(report.items.iter().any(|i| {
            i.kind == MigrationKind::ManifestLegacyToken
                && i.status == MigrationStatus::NeedsMigration
        }));
        assert!(report.items.iter().any(|i| {
            i.kind == MigrationKind::MetaSharedLibrary
                && i.status == MigrationStatus::Protected
                && i.subject == "loop_lib"
        }));
        std::env::remove_var("META_ROOT");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn apply_materializes_only_canonical_dirs_and_writes_ledger() {
        let root = tempdir("migration-apply");
        let manifest = root.join("manifest");
        std::fs::create_dir_all(&manifest).unwrap();
        std::fs::write(
            manifest.join("base.toml"),
            r#"
[[component]]
id = "stub"
name = "Stub"
"#,
        )
        .unwrap();
        let _guard = crate::test_env_lock();
        std::env::set_var("META_ROOT", &root);
        let engine = Engine::with_runner(manifest.clone(), Box::new(DryRunRunner)).unwrap();

        let report = apply(
            engine.registry(),
            &manifest,
            &MigrationSpec::default(),
            true,
        )
        .unwrap();

        assert_eq!(report.verb, MigrationVerb::Apply);
        assert!(root.join("usr/bin").is_dir());
        assert!(root.join("usr/lib").is_dir());
        assert!(root.join("usr/libexec").is_dir());
        assert!(root.join("usr/share").is_dir());
        assert!(root.join("etc/envctl").is_dir());
        assert!(root.join("var/lib/envctl").is_dir());
        assert!(root.join("var/cache/envctl").is_dir());
        assert!(root.join("opt").is_dir());
        assert!(root.join(".config").is_dir());
        assert!(root.join(".local/share").is_dir());
        assert!(root.join(".local/state").is_dir());
        assert!(root.join(".cache").is_dir());
        assert!(!root.join(".local/bin").exists());
        assert!(!root.join(".toolchains").exists());
        assert!(PathBuf::from(&report.ledger_path).is_file());
        std::env::remove_var("META_ROOT");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn classify_line_flags_all_real_home_local_spellings() {
        for spelling in [
            "~/.local/bin/foo",
            "$HOME/.local/bin/foo",
            "${HOME}/.local/bin/foo",
            "%h/.local/bin/foo",
        ] {
            assert!(
                classify_line(spelling)
                    .iter()
                    .any(|hit| hit.id == "home-local"),
                "{spelling} should be migration debt"
            );
        }

        assert!(
            classify_line(
                "$ENVCTL_REAL_HOME/.nix-profile -> $ENVCTL_REAL_HOME/.local/state/nix/profile"
            )
            .is_empty(),
            "the Yazelix Nix profile policy must not be classified as a per-tool install"
        );
    }

    fn tempdir(label: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir =
            std::env::temp_dir().join(format!("envctl-{label}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
