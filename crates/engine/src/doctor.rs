//! Engine-owned whole-environment diagnostics.
//!
//! This module is deliberately read-only. In particular, writability is proven
//! with filesystem metadata plus `access(2)`; doctor never creates a directory,
//! probe file, cache entry, or cleanup mutation. Root resolution is explicit and
//! workspace-aware and has no historical `~/Desktop/meta` fallback.

use crate::lock::{self, LockDriftKind, LockFile};
use crate::model::MetaBoundaryReport;
use crate::runtime::LastRun;
use crate::{Engine, Event, EventSink, LayoutKind, MetaLayout};
use loop_lib::{build_command, SpawnSpec};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

/// Severity of one doctor observation and of the aggregate report.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    #[default]
    Ok,
    Warning,
    Error,
}

/// Aggregate counts. Only `errors > 0` makes the CLI exit non-zero; warnings
/// remain observable without turning optional host capabilities into failures.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Summary {
    pub ok: usize,
    pub warnings: usize,
    pub errors: usize,
}

impl Summary {
    fn record(&mut self, status: Status) {
        match status {
            Status::Ok => self.ok += 1,
            Status::Warning => self.warnings += 1,
            Status::Error => self.errors += 1,
        }
    }

    fn status(&self) -> Status {
        if self.errors > 0 {
            Status::Error
        } else if self.warnings > 0 {
            Status::Warning
        } else {
            Status::Ok
        }
    }
}

/// What metadata/access checks prove about a requested directory.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PathState {
    Writable,
    /// The path does not exist, but its nearest existing directory is writable.
    Creatable,
    ReadOnly,
    Missing,
    NotDirectory,
    Inaccessible,
}

/// One directory observation. `path` + `writable` retain the historical doctor
/// JSON fields; the remaining fields are additive typed context.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathCheck {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    pub path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    pub state: PathState,
    pub writable: bool,
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// One PATH/toolchain observation. `tool` + `version` retain the historical
/// machine contract; status makes absence explicit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCheck {
    pub tool: String,
    pub version: Option<String>,
    pub status: Status,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestLockStatus {
    Clean,
    Drifted,
    Missing,
    Corrupt,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestLockDrift {
    pub component: String,
    pub drift: LockDriftKind,
}

/// Typed read-only result of comparing the current manifest with `envctl.lock`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestLockReport {
    pub status: ManifestLockStatus,
    pub path: PathBuf,
    pub locked: bool,
    pub drift: Vec<ManifestLockDrift>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl ManifestLockReport {
    fn doctor_status(&self) -> Status {
        if self.status == ManifestLockStatus::Clean {
            Status::Ok
        } else {
            Status::Error
        }
    }
}

/// Inputs to the shared doctor surface.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DoctorSpec {
    /// Highest-priority root. Managed-worktree paths normalize to the owner
    /// workspace before checks are generated.
    pub meta_root: Option<PathBuf>,
    /// Start directory for upward `.meta.yaml` discovery. Defaults to cwd.
    pub start: Option<PathBuf>,
    /// Run bounded `--version` and `sudo -n true` probes. Tests may disable
    /// command probes while still exercising every filesystem decision.
    pub probe_commands: bool,
}

impl Default for DoctorSpec {
    fn default() -> Self {
        Self {
            meta_root: None,
            start: None,
            probe_commands: true,
        }
    }
}

/// The single parity contract rendered by CLI and GUI.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DoctorReport {
    pub status: Status,
    pub summary: Summary,
    pub meta_root: Option<PathBuf>,
    pub root_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_error: Option<String>,
    pub writable: Vec<PathCheck>,
    pub layout_registry: Vec<PathCheck>,
    pub tools: Vec<ToolCheck>,
    pub sudo_cached: bool,
    pub uefi: bool,
    pub secure_boot: Option<String>,
    pub nvidia_driver_loaded: bool,
    pub run_log: PathBuf,
    pub run_log_exists: bool,
    pub meta_boundary: MetaBoundaryReport,
    pub last_run: Option<LastRun>,
    pub manifest_lock: ManifestLockReport,
}

impl DoctorReport {
    pub fn healthy(&self) -> bool {
        self.status != Status::Error
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RootSource {
    Explicit,
    Environment,
    MetaFile,
    ManagedWorktree,
}

impl RootSource {
    fn label(self) -> &'static str {
        match self {
            RootSource::Explicit => "explicit",
            RootSource::Environment => "meta_root_env",
            RootSource::MetaFile => "meta_yaml",
            RootSource::ManagedWorktree => "managed_worktree",
        }
    }
}

#[derive(Debug)]
struct ResolvedRoot {
    path: PathBuf,
    source: RootSource,
}

pub(crate) fn run(
    engine: &Engine,
    spec: &DoctorSpec,
    sink: &EventSink,
) -> anyhow::Result<DoctorReport> {
    let manifest_lock = engine.manifest_lock_check();
    let resolved = resolve_meta_root(spec);
    let report = match resolved {
        Ok(root) => build_report(engine, spec, root, manifest_lock),
        Err(root_error) => {
            let tools = probe_tools(spec.probe_commands);
            let sudo_cached = spec.probe_commands && command_success("sudo", &["-n", "true"]);
            let uefi = Path::new("/sys/firmware/efi").is_dir();
            let secure_boot = read_secure_boot();
            let nvidia_driver_loaded = Path::new("/proc/driver/nvidia/version").is_file();
            let mut summary = Summary::default();
            summary.record(Status::Error);
            for tool in &tools {
                summary.record(tool.status);
            }
            summary.record(if sudo_cached {
                Status::Ok
            } else {
                Status::Warning
            });
            summary.record(if uefi { Status::Ok } else { Status::Warning });
            summary.record(if secure_boot.is_some() {
                Status::Ok
            } else {
                Status::Warning
            });
            summary.record(if nvidia_driver_loaded {
                Status::Ok
            } else {
                Status::Warning
            });
            summary.record(Status::Warning); // no root-derived run-log path
            summary.record(manifest_lock.doctor_status());
            DoctorReport {
                status: Status::Error,
                summary,
                meta_root: None,
                root_source: None,
                root_error: Some(root_error),
                writable: Vec::new(),
                layout_registry: Vec::new(),
                tools,
                sudo_cached,
                uefi,
                secure_boot,
                nvidia_driver_loaded,
                run_log: PathBuf::new(),
                run_log_exists: false,
                meta_boundary: MetaBoundaryReport::default(),
                last_run: crate::runtime::load(engine.manifest_dir()).last_run,
                manifest_lock,
            }
        }
    };
    sink.emit(Event::Doctored {
        report: report.clone(),
    });
    Ok(report)
}

fn build_report(
    engine: &Engine,
    spec: &DoctorSpec,
    root: ResolvedRoot,
    manifest_lock: ManifestLockReport,
) -> DoctorReport {
    let layout = MetaLayout::from_meta_root(&root.path);
    let entries = layout.entries();
    let layout_registry: Vec<PathCheck> = entries
        .iter()
        .map(|entry| {
            path_check(
                entry.path.clone(),
                Some(entry.key.to_string()),
                Some(match entry.kind {
                    LayoutKind::Canonical => "canonical",
                    LayoutKind::LegacyCompatibility => "legacy_compatibility",
                }),
                Some(entry.purpose.to_string()),
                false,
            )
        })
        .collect();

    let mut writable: Vec<PathCheck> = entries
        .iter()
        .filter(|entry| entry.is_canonical())
        .map(|entry| {
            path_check(
                entry.path.clone(),
                Some(entry.key.to_string()),
                Some("canonical"),
                Some(entry.purpose.to_string()),
                true,
            )
        })
        .collect();
    writable.push(path_check(
        PathBuf::from("/etc"),
        Some("host_etc".to_string()),
        Some("host_integration"),
        Some("optional privileged host integration".to_string()),
        false,
    ));

    let tools = probe_tools(spec.probe_commands);
    let sudo_cached = spec.probe_commands && command_success("sudo", &["-n", "true"]);
    let uefi = Path::new("/sys/firmware/efi").is_dir();
    let secure_boot = read_secure_boot();
    let nvidia_driver_loaded = Path::new("/proc/driver/nvidia/version").is_file();
    let run_log = layout.state().join("envctl.log");
    let run_log_exists = run_log.is_file();
    let meta_boundary = crate::detect::meta_boundary_report_for_root(&root.path);
    let last_run = crate::runtime::load(engine.manifest_dir()).last_run;

    let mut summary = Summary::default();
    summary.record(Status::Ok); // root resolution
    for check in &writable {
        summary.record(check.status);
    }
    for tool in &tools {
        summary.record(tool.status);
    }
    summary.record(if sudo_cached {
        Status::Ok
    } else {
        Status::Warning
    });
    summary.record(if uefi { Status::Ok } else { Status::Warning });
    summary.record(if secure_boot.is_some() {
        Status::Ok
    } else {
        Status::Warning
    });
    summary.record(if nvidia_driver_loaded {
        Status::Ok
    } else {
        Status::Warning
    });
    summary.record(if run_log_exists {
        Status::Ok
    } else {
        Status::Warning
    });
    summary.record(if meta_boundary.ok() {
        Status::Ok
    } else {
        Status::Error
    });
    summary.record(manifest_lock.doctor_status());

    DoctorReport {
        status: summary.status(),
        summary,
        meta_root: Some(root.path),
        root_source: Some(root.source.label().to_string()),
        root_error: None,
        writable,
        layout_registry,
        tools,
        sudo_cached,
        uefi,
        secure_boot,
        nvidia_driver_loaded,
        run_log,
        run_log_exists,
        meta_boundary,
        last_run,
        manifest_lock,
    }
}

pub(crate) fn manifest_lock_check(manifest_dir: &Path) -> ManifestLockReport {
    let path = lock::lock_path(manifest_dir);
    if !path.is_file() {
        return ManifestLockReport {
            status: ManifestLockStatus::Missing,
            path,
            locked: false,
            drift: Vec::new(),
            detail: Some("envctl.lock is missing".to_string()),
        };
    }

    let loaded = match LockFile::load(manifest_dir) {
        Ok(lock) => lock,
        Err(error) => {
            return ManifestLockReport {
                status: ManifestLockStatus::Corrupt,
                path,
                locked: false,
                drift: Vec::new(),
                detail: Some(error.to_string()),
            };
        }
    };
    let registry = match crate::model::Registry::load(manifest_dir) {
        Ok(registry) => registry,
        Err(error) => {
            return ManifestLockReport {
                status: ManifestLockStatus::Corrupt,
                path,
                locked: false,
                drift: Vec::new(),
                detail: Some(format!("cannot load current manifest: {error}")),
            };
        }
    };
    let drift = lock::diff(&registry, &loaded)
        .into_iter()
        .map(|(component, drift)| ManifestLockDrift { component, drift })
        .collect::<Vec<_>>();
    let locked = drift.is_empty();
    ManifestLockReport {
        status: if locked {
            ManifestLockStatus::Clean
        } else {
            ManifestLockStatus::Drifted
        },
        path,
        locked,
        drift,
        detail: None,
    }
}

fn resolve_meta_root(spec: &DoctorSpec) -> Result<ResolvedRoot, String> {
    if let Some(root) = spec.meta_root.as_ref() {
        return validate_root(root, RootSource::Explicit);
    }
    if let Some(root) = std::env::var_os("META_ROOT").filter(|value| !value.is_empty()) {
        return validate_root(Path::new(&root), RootSource::Environment);
    }

    let start = match spec.start.clone() {
        Some(start) => absolutize(start)?,
        None => std::env::current_dir()
            .map_err(|error| format!("cannot resolve current directory: {error}"))?,
    };
    let start_dir = if start.is_file() {
        start
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| format!("start path has no parent: {}", start.display()))?
    } else {
        start.clone()
    };
    let start_dir = canonical_or_self(start_dir);
    let managed_root = verified_managed_meta_root(&start_dir)?;

    let mut roots = BTreeSet::new();
    for ancestor in start_dir.ancestors() {
        if ancestor.join(".meta.yaml").is_file() {
            roots.insert(canonical_or_self(ancestor.to_path_buf()));
        }
    }
    match roots.len() {
        1 => {
            let root = roots.into_iter().next().expect("one root");
            let source = if managed_root.as_ref() == Some(&root) {
                RootSource::ManagedWorktree
            } else {
                RootSource::MetaFile
            };
            validate_root(&root, source)
        }
        n if n > 1 => Err(format!(
            "ambiguous meta roots discovered above {}: {}",
            start_dir.display(),
            roots
                .iter()
                .map(|root| root.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )),
        _ => Err(format!(
            "no meta root: set META_ROOT, pass --root, or run below a .meta.yaml (start: {})",
            start_dir.display()
        )),
    }
}

fn validate_root(path: &Path, source: RootSource) -> Result<ResolvedRoot, String> {
    let path = absolutize(path.to_path_buf())?;
    let path = canonical_or_self(path);
    let path = verified_managed_meta_root(&path)?.unwrap_or(path);
    match std::fs::metadata(&path) {
        Ok(metadata) if metadata.is_dir() => Ok(ResolvedRoot { path, source }),
        Ok(_) => Err(format!("meta root is not a directory: {}", path.display())),
        Err(error) => Err(format!(
            "cannot access meta root {}: {error}",
            path.display()
        )),
    }
}

fn absolutize(path: PathBuf) -> Result<PathBuf, String> {
    if path.is_absolute() {
        Ok(path)
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .map_err(|error| format!("cannot resolve current directory: {error}"))
    }
}

/// Resolve a path below `<meta>/.worktrees/<slug>/<repo>` to its owning Meta
/// root only when the filesystem proves that topology. The lexical component
/// alone is not evidence: require the canonical `.meta.yaml`, a declared repo
/// id, and live linked-worktree gitdir metadata for the checkout.
fn verified_managed_meta_root(path: &Path) -> Result<Option<PathBuf>, String> {
    let mut prefix = PathBuf::new();
    let mut components = path.components();
    while let Some(component) = components.next() {
        if component.as_os_str() == ".worktrees" {
            let slug = match components.next() {
                Some(std::path::Component::Normal(slug)) if !slug.is_empty() => slug,
                _ => {
                    return Err(format!(
                        "unverified managed-worktree path {}: missing worktree-set slug",
                        path.display()
                    ));
                }
            };
            let repo = match components.next() {
                Some(std::path::Component::Normal(repo)) if !repo.is_empty() => repo,
                _ => {
                    return Err(format!(
                        "unverified managed-worktree path {}: missing repository identity",
                        path.display()
                    ));
                }
            };
            let Some(repo_id) = repo.to_str() else {
                return Err(format!(
                    "unverified managed-worktree path {}: repository identity is not UTF-8",
                    path.display()
                ));
            };
            let root = canonical_or_self(prefix);
            let meta_file = root.join(".meta.yaml");
            let workspace = crate::dashboard::read_workspace(&meta_file).map_err(|error| {
                format!(
                    "unverified managed-worktree path {}: cannot prove owning Meta root from {}: {error}",
                    path.display(),
                    meta_file.display()
                )
            })?;
            if !workspace
                .repos
                .iter()
                .any(|declared| declared.id == repo_id)
            {
                return Err(format!(
                    "unverified managed-worktree path {}: repository {repo_id} is not declared in {}",
                    path.display(),
                    meta_file.display()
                ));
            }

            let checkout = root.join(".worktrees").join(slug).join(repo);
            let git_file = checkout.join(".git");
            let git_metadata = std::fs::read_to_string(&git_file).map_err(|error| {
                format!(
                    "unverified managed-worktree path {}: cannot read linked-worktree metadata {}: {error}",
                    path.display(),
                    git_file.display()
                )
            })?;
            let gitdir = git_metadata
                .lines()
                .next()
                .and_then(|line| line.strip_prefix("gitdir: "))
                .filter(|gitdir| !gitdir.trim().is_empty())
                .ok_or_else(|| {
                    format!(
                        "unverified managed-worktree path {}: malformed linked-worktree metadata {}",
                        path.display(),
                        git_file.display()
                    )
                })?;
            let gitdir = PathBuf::from(gitdir);
            let gitdir = if gitdir.is_absolute() {
                gitdir
            } else {
                checkout.join(gitdir)
            };
            if !gitdir.is_dir() {
                return Err(format!(
                    "unverified managed-worktree path {}: linked gitdir does not exist: {}",
                    path.display(),
                    gitdir.display()
                ));
            }
            return Ok(Some(root));
        }
        prefix.push(component.as_os_str());
    }
    Ok(None)
}

fn canonical_or_self(path: PathBuf) -> PathBuf {
    std::fs::canonicalize(&path).unwrap_or(path)
}

fn path_check(
    path: PathBuf,
    key: Option<String>,
    kind: Option<&str>,
    purpose: Option<String>,
    required: bool,
) -> PathCheck {
    let state = path_state(&path);
    let writable = matches!(state, PathState::Writable | PathState::Creatable);
    let status = if writable {
        Status::Ok
    } else if required {
        Status::Error
    } else {
        Status::Warning
    };
    let detail = match state {
        PathState::Writable => None,
        PathState::Creatable => Some("nearest existing parent is writable".to_string()),
        PathState::ReadOnly => Some("write access was not proven".to_string()),
        PathState::Missing => Some("path and writable parent were not found".to_string()),
        PathState::NotDirectory => Some("path exists but is not a directory".to_string()),
        PathState::Inaccessible => Some("path metadata is inaccessible".to_string()),
    };
    PathCheck {
        key,
        path,
        kind: kind.map(str::to_string),
        purpose,
        state,
        writable,
        status,
        detail,
    }
}

fn path_state(path: &Path) -> PathState {
    match std::fs::metadata(path) {
        Ok(metadata) if !metadata.is_dir() => PathState::NotDirectory,
        Ok(_) if write_access(path) => PathState::Writable,
        Ok(_) => PathState::ReadOnly,
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            PathState::Inaccessible
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let mut ancestor = path.parent();
            while let Some(candidate) = ancestor {
                match std::fs::metadata(candidate) {
                    Ok(metadata) if metadata.is_dir() => {
                        return if write_access(candidate) {
                            PathState::Creatable
                        } else {
                            PathState::Missing
                        };
                    }
                    Ok(_) => return PathState::NotDirectory,
                    Err(parent_error)
                        if parent_error.kind() == std::io::ErrorKind::PermissionDenied =>
                    {
                        return PathState::Inaccessible;
                    }
                    Err(_) => ancestor = candidate.parent(),
                }
            }
            PathState::Missing
        }
        Err(_) => PathState::Inaccessible,
    }
}

fn write_access(path: &Path) -> bool {
    rustix::fs::access(
        path,
        rustix::fs::Access::WRITE_OK | rustix::fs::Access::EXEC_OK,
    )
    .is_ok()
}

fn probe_tools(run_versions: bool) -> Vec<ToolCheck> {
    const TOOLS: &[&str] = &[
        "git",
        "cargo",
        "rustc",
        "claude",
        "nix",
        "podman",
        "nvidia-smi",
        "gh",
        "uv",
        "bun",
    ];
    TOOLS
        .iter()
        .map(|tool| {
            let path = which::which(tool).ok();
            let version = path.as_ref().map(|path| {
                if run_versions {
                    command_version(path).unwrap_or_else(|| path.display().to_string())
                } else {
                    path.display().to_string()
                }
            });
            ToolCheck {
                tool: (*tool).to_string(),
                status: if version.is_some() {
                    Status::Ok
                } else {
                    Status::Warning
                },
                version,
            }
        })
        .collect()
}

fn command_version(path: &Path) -> Option<String> {
    command_output(path, &["--version"], Duration::from_secs(2)).and_then(|output| {
        output.status.success().then(|| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_string)
        })?
    })
}

fn command_success(program: &str, args: &[&str]) -> bool {
    command_output(Path::new(program), args, Duration::from_secs(2))
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn command_output(
    program: &Path,
    args: &[&str],
    timeout: Duration,
) -> Option<std::process::Output> {
    let program = program.to_string_lossy().into_owned();
    let args = args
        .iter()
        .map(|arg| (*arg).to_string())
        .collect::<Vec<_>>();
    let mut command = build_command(&SpawnSpec {
        program: &program,
        args: &args,
        current_dir: None,
        env: &[],
        // Version/sudo probes are read-only diagnostics. Do not let loader,
        // shell, or exported-function state from the caller influence them.
        clear_env: true,
    });
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return child.wait_with_output().ok(),
            Ok(None) if started.elapsed() < timeout => {
                std::thread::sleep(Duration::from_millis(25));
            }
            Ok(None) | Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
}

fn read_secure_boot() -> Option<String> {
    let entries = std::fs::read_dir("/sys/firmware/efi/efivars").ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        if name.to_string_lossy().starts_with("SecureBoot-") {
            let bytes = std::fs::read(entry.path()).ok()?;
            return bytes.get(4).map(|value| value.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DryRunRunner, Registry};
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Fixture {
        root: PathBuf,
        manifest: PathBuf,
    }

    impl Fixture {
        fn new(label: &str) -> Self {
            static SEQ: AtomicU64 = AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "envctl-engine-doctor-{label}-{}-{}",
                std::process::id(),
                SEQ.fetch_add(1, Ordering::Relaxed)
            ));
            let manifest = root.join("manifest");
            std::fs::create_dir_all(&manifest).unwrap();
            std::fs::write(
                manifest.join("base.toml"),
                r#"
[[component]]
id = "stub"
name = "Stub"
[component.detect]
kind = "command"
command = "true"
"#,
            )
            .unwrap();
            let registry = Registry::load(&manifest).unwrap();
            let mut lock = lock::generate(&registry);
            lock.save(&manifest).unwrap();
            Self { root, manifest }
        }

        fn engine(&self) -> Engine {
            Engine::with_runner(self.manifest.clone(), Box::new(DryRunRunner)).unwrap()
        }

        fn spec(&self) -> DoctorSpec {
            DoctorSpec {
                meta_root: Some(self.root.clone()),
                start: None,
                probe_commands: false,
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.root).ok();
        }
    }

    fn tree_snapshot(root: &Path) -> Vec<PathBuf> {
        fn walk(root: &Path, current: &Path, out: &mut Vec<PathBuf>) {
            let Ok(entries) = std::fs::read_dir(current) else {
                return;
            };
            let mut entries = entries.flatten().collect::<Vec<_>>();
            entries.sort_by_key(|entry| entry.file_name());
            for entry in entries {
                let path = entry.path();
                out.push(path.strip_prefix(root).unwrap().to_path_buf());
                if path.is_dir() {
                    walk(root, &path, out);
                }
            }
        }
        let mut out = Vec::new();
        walk(root, root, &mut out);
        out
    }

    #[test]
    fn summary_aggregates_warning_and_error_without_downgrading_error() {
        let mut summary = Summary::default();
        summary.record(Status::Ok);
        summary.record(Status::Warning);
        assert_eq!(summary.status(), Status::Warning);
        summary.record(Status::Error);
        assert_eq!(summary.status(), Status::Error);
        assert_eq!(summary.ok, 1);
        assert_eq!(summary.warnings, 1);
        assert_eq!(summary.errors, 1);
    }

    #[cfg(unix)]
    #[test]
    fn write_only_directory_is_not_reported_writable_without_search_access() {
        use std::os::unix::fs::PermissionsExt;

        let fixture = Fixture::new("write-without-search");
        let directory = fixture.root.join("write-only");
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o200)).unwrap();

        assert_eq!(path_state(&directory), PathState::ReadOnly);

        std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700)).unwrap();
    }

    #[test]
    fn doctor_is_non_mutating_and_emits_exactly_one_doctored_event() {
        let fixture = Fixture::new("non-mutating");
        let before = tree_snapshot(&fixture.root);
        let (sink, rx) = EventSink::channel();
        let report = fixture.engine().doctor(&fixture.spec(), &sink).unwrap();
        drop(sink);
        let events = rx.into_iter().collect::<Vec<_>>();
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, Event::Doctored { .. }))
                .count(),
            1
        );
        assert_eq!(before, tree_snapshot(&fixture.root));
        assert_eq!(report.manifest_lock.status, ManifestLockStatus::Clean);
    }

    #[test]
    fn explicit_root_wins_over_environment() {
        let _guard = crate::test_env_lock();
        let fixture = Fixture::new("explicit");
        let other = fixture.root.join("other");
        std::fs::create_dir_all(&other).unwrap();
        std::env::set_var("META_ROOT", &other);
        let resolved = resolve_meta_root(&fixture.spec()).unwrap();
        std::env::remove_var("META_ROOT");
        assert_eq!(resolved.source, RootSource::Explicit);
        assert_eq!(resolved.path, fixture.root.canonicalize().unwrap());
    }

    #[test]
    fn environment_root_wins_over_upward_meta_file() {
        let _guard = crate::test_env_lock();
        let fixture = Fixture::new("environment");
        let env_root = fixture.root.join("env-root");
        let nested = fixture.root.join("repo/nested");
        std::fs::create_dir_all(&env_root).unwrap();
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(fixture.root.join(".meta.yaml"), "projects: {}\n").unwrap();
        std::env::set_var("META_ROOT", &env_root);
        let resolved = resolve_meta_root(&DoctorSpec {
            meta_root: None,
            start: Some(nested),
            probe_commands: false,
        })
        .unwrap();
        std::env::remove_var("META_ROOT");
        assert_eq!(resolved.source, RootSource::Environment);
        assert_eq!(resolved.path, env_root.canonicalize().unwrap());
    }

    #[test]
    fn upward_meta_file_resolves_workspace_root() {
        let _guard = crate::test_env_lock();
        std::env::remove_var("META_ROOT");
        let fixture = Fixture::new("meta-file");
        let nested = fixture.root.join("repo/nested");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(fixture.root.join(".meta.yaml"), "projects: {}\n").unwrap();
        let resolved = resolve_meta_root(&DoctorSpec {
            meta_root: None,
            start: Some(nested),
            probe_commands: false,
        })
        .unwrap();
        assert_eq!(resolved.source, RootSource::MetaFile);
        assert_eq!(resolved.path, fixture.root.canonicalize().unwrap());
    }

    #[test]
    fn managed_worktree_normalizes_to_owner_root_without_fallback() {
        let _guard = crate::test_env_lock();
        std::env::remove_var("META_ROOT");
        let fixture = Fixture::new("worktree");
        let nested = fixture.root.join(".worktrees/slug/envctl");
        let gitdir = fixture.root.join(".git/worktrees/envctl-test");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::create_dir_all(&gitdir).unwrap();
        std::fs::write(
            fixture.root.join(".meta.yaml"),
            "projects:\n  envctl:\n    repo: test://envctl\n",
        )
        .unwrap();
        std::fs::write(
            nested.join(".git"),
            format!("gitdir: {}\n", gitdir.display()),
        )
        .unwrap();
        let resolved = resolve_meta_root(&DoctorSpec {
            meta_root: None,
            start: Some(nested),
            probe_commands: false,
        })
        .unwrap();
        assert_eq!(resolved.source, RootSource::ManagedWorktree);
        assert_eq!(resolved.path, fixture.root.canonicalize().unwrap());
    }

    #[test]
    fn lexical_dot_worktrees_path_without_meta_identity_is_refused() {
        let _guard = crate::test_env_lock();
        std::env::remove_var("META_ROOT");
        let fixture = Fixture::new("fake-worktree");
        let nested = fixture.root.join(".worktrees/fake/envctl");
        std::fs::create_dir_all(&nested).unwrap();

        let error = resolve_meta_root(&DoctorSpec {
            meta_root: None,
            start: Some(nested),
            probe_commands: false,
        })
        .unwrap_err();

        assert!(
            error.contains("unverified managed-worktree path"),
            "{error}"
        );
        assert!(error.contains(".meta.yaml"), "{error}");
    }

    #[test]
    fn managed_worktree_requires_declared_repo_and_live_linked_gitdir() {
        let fixture = Fixture::new("worktree-identity");
        let checkout = fixture.root.join(".worktrees/slug/envctl");
        let gitdir = fixture.root.join(".git/worktrees/envctl-test");
        std::fs::create_dir_all(&checkout).unwrap();
        std::fs::create_dir_all(&gitdir).unwrap();
        std::fs::write(
            fixture.root.join(".meta.yaml"),
            "projects:\n  different-repo:\n    repo: test://different\n",
        )
        .unwrap();
        std::fs::write(
            checkout.join(".git"),
            format!("gitdir: {}\n", gitdir.display()),
        )
        .unwrap();

        let undeclared = verified_managed_meta_root(&checkout).unwrap_err();
        assert!(undeclared.contains("repository envctl is not declared"));

        std::fs::write(
            fixture.root.join(".meta.yaml"),
            "projects:\n  envctl:\n    repo: test://envctl\n",
        )
        .unwrap();
        std::fs::remove_file(checkout.join(".git")).unwrap();
        let unlinked = verified_managed_meta_root(&checkout).unwrap_err();
        assert!(unlinked.contains("cannot read linked-worktree metadata"));
    }

    #[test]
    fn missing_root_is_an_error_and_never_invents_desktop_meta() {
        let _guard = crate::test_env_lock();
        std::env::remove_var("META_ROOT");
        let fixture = Fixture::new("missing-root");
        let start = fixture.root.join("ordinary/repo");
        std::fs::create_dir_all(&start).unwrap();
        let error = resolve_meta_root(&DoctorSpec {
            meta_root: None,
            start: Some(start),
            probe_commands: false,
        })
        .unwrap_err();
        assert!(error.contains("no meta root"));
        assert!(!error.contains("Desktop/meta"));
    }

    #[test]
    fn multiple_distinct_upward_meta_files_are_refused_as_ambiguous() {
        let _guard = crate::test_env_lock();
        std::env::remove_var("META_ROOT");
        let fixture = Fixture::new("ambiguous");
        let nested_root = fixture.root.join("nested");
        let start = nested_root.join("repo");
        std::fs::create_dir_all(&start).unwrap();
        std::fs::write(fixture.root.join(".meta.yaml"), "projects: {}\n").unwrap();
        std::fs::write(nested_root.join(".meta.yaml"), "projects: {}\n").unwrap();
        let error = resolve_meta_root(&DoctorSpec {
            meta_root: None,
            start: Some(start),
            probe_commands: false,
        })
        .unwrap_err();
        assert!(error.contains("ambiguous meta roots"));
    }

    #[test]
    fn manifest_lock_check_reports_clean_drift_missing_and_corrupt() {
        let fixture = Fixture::new("lock-states");
        let engine = fixture.engine();
        assert_eq!(
            engine.manifest_lock_check().status,
            ManifestLockStatus::Clean
        );

        std::fs::write(
            fixture.manifest.join("extra.toml"),
            r#"
[[component]]
id = "extra"
name = "Extra"
[component.detect]
kind = "command"
command = "true"
"#,
        )
        .unwrap();
        assert_eq!(
            engine.manifest_lock_check().status,
            ManifestLockStatus::Drifted
        );

        std::fs::remove_file(fixture.manifest.join(lock::LOCK_FILENAME)).unwrap();
        assert_eq!(
            engine.manifest_lock_check().status,
            ManifestLockStatus::Missing
        );
        std::fs::write(fixture.manifest.join(lock::LOCK_FILENAME), "not = [valid").unwrap();
        assert_eq!(
            engine.manifest_lock_check().status,
            ManifestLockStatus::Corrupt
        );
    }

    #[test]
    fn manifest_lock_check_rejects_tampered_requires_resolved_and_version() {
        let fixture = Fixture::new("lock-semantic-tampering");
        let engine = fixture.engine();
        let original = LockFile::load(&fixture.manifest).unwrap();
        let lock_path = fixture.manifest.join(lock::LOCK_FILENAME);
        let write_lock = |lock_file: &LockFile| {
            std::fs::write(&lock_path, toml::to_string_pretty(lock_file).unwrap()).unwrap();
        };

        let mut tampered = original.clone();
        tampered
            .components
            .get_mut("stub")
            .unwrap()
            .requires
            .push("ghost-dependency".to_string());
        write_lock(&tampered);
        let report = engine.manifest_lock_check();
        assert_eq!(report.status, ManifestLockStatus::Drifted);
        assert!(report
            .drift
            .iter()
            .any(|entry| { entry.component == "stub" && entry.drift == LockDriftKind::Changed }));

        let mut tampered = original.clone();
        tampered.components.get_mut("stub").unwrap().resolved = "deadbeef".to_string();
        write_lock(&tampered);
        let report = engine.manifest_lock_check();
        assert_eq!(report.status, ManifestLockStatus::Drifted);
        assert!(report
            .drift
            .iter()
            .any(|entry| { entry.component == "stub" && entry.drift == LockDriftKind::Changed }));

        let mut tampered = original;
        tampered.version = lock::LOCK_VERSION + 1;
        write_lock(&tampered);
        let report = engine.manifest_lock_check();
        assert_eq!(report.status, ManifestLockStatus::Corrupt);
        assert!(report
            .detail
            .as_deref()
            .is_some_and(|detail| detail.contains("unsupported envctl.lock version")));
    }
}
