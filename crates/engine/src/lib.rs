//! envctl engine: the single shared library. No printing, no UI, no clap.
//!
//! Both the CLI (`envctl`) and the GUI (`envctl-gui`) drive the box through the
//! *identical* `Engine` API below, so the two front-ends can never diverge.
pub mod addrepo; // Phase 4: the staged build-from-source pipeline + confined AI agent
pub mod agent; // agent-env subsystem: the 6 agent-asset verbs over envctl-agent-env
pub mod catalog; // ADR-0003: catalog tables plus read-only diff/render projections
pub mod command;
pub mod component; // Component, Hook, Guard, Phase, HookRunner
pub mod dashboard; // meta mission-control: read .meta.yaml -> render zellij KDL layout
pub mod detect; // EnvReport assembly: PCI floor / proc-backed driver state / nvidia-smi / sysinfo / which probes
pub mod detect_build; // Phase 4: build-system detector table -> BuildPlan
pub mod drift; // pure diff(EnvReport, Registry) -> Vec<DriftItem>
pub mod error; // EngineError, RunContext, run_phase
pub mod event; // Event, EventSink, Stream, Telemetry, GpuSample
pub mod executor; // Engine::run(plan) best-effort loop + RunContext resolve + add_repo
pub mod graph; // graph intelligence over the component dependency DAG
pub mod guard; // fail-closed UuidResolves/NotLiveDevice/NotMounted/PathExists/HookSucceeds
pub mod hub_registry; // read-only federation over *_hub/registry.json
pub mod install; // Phase 4: regular frontdoor wrappers into meta usr/bin (refuse-unmanaged) + wire-in
pub mod layout; // meta-hosted FHS/XDG path resolver (usr/etc/var/opt/run/tmp + XDG roots)
pub mod lock; // envctl.lock — content-hashed manifest-of-record + CI gate
pub mod migration; // adoption engine: scan/plan/apply/verify/purge into meta .local topology
pub mod model; // Registry, OpResult, OpStatus, EnvReport, Wiring, RunPlan, RunSummary, AddRepoSpec
pub mod peer; // add-repo PEER path: meta-native .meta.yaml/.gitignore registration (vs component)
pub mod register; // Phase 4: synthesize the components.d drop-in (provenance + rebuild)
pub mod runner; // ProcessRunner (real) + DryRunRunner impls of HookRunner
pub mod runtime; // machine-local last-run state (XDG cache), out of the lock
pub mod secrets; // TASK-0028: engine-owned `secretctl` subprocess seam for the GUI secrets verbs
pub mod self_uninstall; // `self uninstall` — destructive, fail-closed, dry-run-by-default removal
pub mod self_update; // `self update` CORE: fetch_latest_release / is_newer / verify_checksum
pub mod telemetry; // sample() -> Telemetry (hard-timeout nvidia-smi CSV + sysinfo)
pub mod update_notifier; // end-of-run "new version available" cache + check (CLI renders)
pub mod wiring; // apply()/revert() for Wiring (shell_rc backup-then-excise) // EngineCommand / EngineEvent + run_event_loop (GUI worker API)

pub use agent::{
    AgentAddSpec, AgentCleanSpec, AgentCommandDirCheck, AgentDoctorReport, AgentDoctorSpec,
    AgentEditItem, AgentEditOutcome, AgentInitOutcome, AgentInitSpec, AgentList, AgentListKind,
    AgentListSpec, AgentLockDriftItem, AgentLockMode, AgentLockOutcome, AgentLockSpec,
    AgentRemoveSpec, AgentReport, AgentScope, AgentSectionSel, AgentSyncSpec, AgentUpdateCheck,
    AgentVerb,
};
pub use catalog::{
    CatalogDiffReport, CatalogDiffSummary, CatalogDriftRow, CatalogImportReport,
    CatalogImportSummary, CatalogLockReport, CatalogLockSpec, CatalogLockSummary,
    CatalogRenderReport, CatalogRenderSpec, CatalogRenderSummary, CatalogRenderedFile,
    CatalogScanSpec, CatalogSnapshot, CatalogSyncAction, CatalogSyncReport, CatalogSyncSpec,
    CatalogSyncSummary, CatalogTableName,
};
pub use command::{
    run_event_loop, AgentCommandSpec, EngineCommand, EngineEvent, MigrationCommandSpec,
    TelemetryControl,
};
pub use component::{Component, Guard, Hook, HookRunner, Phase};
pub use dashboard::{
    DashboardPane, DashboardPlan, DashboardSpec, DashboardTab, DeployOutcome, MetaRepo,
    MetaWorkspace,
};
pub use drift::DriftSummary;
pub use error::{EngineError, RunContext};
pub use event::{Event, EventSink, GpuSample, Stream, Telemetry};
pub use hub_registry::{
    HubRegistryDrift, HubRegistryEntryView, HubRegistryReport, HubRegistrySource, HubRegistryStatus,
};
pub use layout::{LayoutEntry, LayoutKind, MetaLayout};
pub use migration::{
    MigrationAction, MigrationItem, MigrationKind, MigrationLayoutEntry, MigrationLayoutKind,
    MigrationOwner, MigrationReport, MigrationRisk, MigrationScope, MigrationSpec, MigrationStatus,
    MigrationSummary, MigrationVerb,
};
pub use model::{
    AddRepoMode, AddRepoSpec, AiAgent, BuildStrategy, BuildSystem, ComponentState, DataPath,
    DesktopEntry, DriftItem, DriftKind, EnvReport, MetaBoundaryReport, MetaBoundaryViolation,
    MetaBoundaryViolationKind, OpResult, OpStatus, Refactor, RefactorGoal, Registry, RenameRule,
    ResetGates, RunPlan, RunSummary, Severity, ShellRcBlock, SystemdUnit, ToolState, Wiring,
};
pub use runner::{DryRunRunner, ProcessRunner};
pub use self_uninstall::{SelfUninstallOutcome, SelfUninstallSpec};
pub use self_update::{
    current_target, fetch_latest_release, is_newer, plan_self_update, verify_checksum,
    SelfUpdateAsset, SelfUpdateCheck, SelfUpdateRelease, GITHUB_REPO,
};
// Re-export `Zeroizing` so front-ends (the GUI) can build the secret stdin buffer for
// `EngineCommand::Secrets` WITHOUT taking a direct `zeroize` dependency (Architecture B keeps the
// GUI dep set frozen). The engine owns the zeroize dep; the GUI uses it through this path.
pub use zeroize::Zeroizing;

use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Top-level engine handle: owns the Registry, manifest dir, and a HookRunner.
/// Cheaply cloneable (Arc inside) and `Send + Sync + 'static` so it can be moved
/// into the GUI worker-thread closure.
#[derive(Clone)]
pub struct Engine {
    inner: Arc<EngineInner>,
}

struct EngineInner {
    registry: Registry,
    manifest_dir: PathBuf,
    // dyn-dispatched; `trait HookRunner: Send + Sync` makes Box<dyn HookRunner>
    // carry Send+Sync automatically, which is what keeps Engine Send+Sync.
    runner: Box<dyn HookRunner>,
}

impl Engine {
    /// Load a manifest dir into an Engine backed by the real ProcessRunner.
    pub fn load(manifest_dir: PathBuf) -> anyhow::Result<Engine> {
        let registry = Registry::load(&manifest_dir)?;
        Ok(Engine {
            inner: Arc::new(EngineInner {
                registry,
                manifest_dir,
                runner: Box::new(ProcessRunner),
            }),
        })
    }

    /// Default manifest dir: `$ENVCTL_MANIFEST_DIR`, else `./manifest`.
    pub fn load_default() -> anyhow::Result<Engine> {
        let dir = std::env::var("ENVCTL_MANIFEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("manifest"));
        Engine::load(dir)
    }

    /// An Engine with NO manifest loaded (empty registry, real runner). For
    /// manifest-independent verbs (e.g. `dashboard`) that read `.meta.yaml`, never
    /// the component registry — so they work from any cwd without a `manifest/` dir.
    /// The manifest dir is still recorded (default resolution) for any path that
    /// needs it, but the registry is empty.
    pub fn detached() -> Engine {
        let manifest_dir = std::env::var("ENVCTL_MANIFEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("manifest"));
        Engine {
            inner: Arc::new(EngineInner {
                registry: Registry::empty(),
                manifest_dir,
                runner: Box::new(ProcessRunner),
            }),
        }
    }

    /// Construct an Engine with a custom HookRunner (used by tests: DryRunRunner).
    pub fn with_runner(
        manifest_dir: PathBuf,
        runner: Box<dyn HookRunner>,
    ) -> anyhow::Result<Engine> {
        let registry = Registry::load(&manifest_dir)?;
        Ok(Engine {
            inner: Arc::new(EngineInner {
                registry,
                manifest_dir,
                runner,
            }),
        })
    }

    pub fn registry(&self) -> &Registry {
        &self.inner.registry
    }

    /// Read-only federation over the workspace's `*_hub/registry.json` files.
    pub fn hub_registry(&self) -> anyhow::Result<HubRegistryReport> {
        let root = workspace_root_for_manifest_dir(&self.inner.manifest_dir);
        hub_registry::load(&root, &self.inner.registry)
    }

    /// Read-only ADR-0003 catalog import: current files -> normalized in-memory tables.
    pub fn catalog_scan(&self) -> anyhow::Result<CatalogSnapshot> {
        let root = workspace_root_for_manifest_dir(&self.inner.manifest_dir);
        catalog::scan(
            catalog::CatalogScanSpec {
                repo_root: root,
                manifest_dir: self.inner.manifest_dir.clone(),
            },
            &self.inner.registry,
        )
    }

    /// ADR-0003 explicit import report: current files -> normalized rows, no writes.
    pub fn catalog_import(&self) -> anyhow::Result<CatalogImportReport> {
        let root = workspace_root_for_manifest_dir(&self.inner.manifest_dir);
        catalog::import_current(
            catalog::CatalogScanSpec {
                repo_root: root,
                manifest_dir: self.inner.manifest_dir.clone(),
            },
            &self.inner.registry,
        )
    }

    /// Read-only ADR-0003 catalog diff: file/catalog/lock drift without mutation.
    pub fn catalog_diff(&self) -> anyhow::Result<CatalogDiffReport> {
        let root = workspace_root_for_manifest_dir(&self.inner.manifest_dir);
        catalog::diff(
            catalog::CatalogScanSpec {
                repo_root: root,
                manifest_dir: self.inner.manifest_dir.clone(),
            },
            &self.inner.registry,
        )
    }

    /// ADR-0003 bidirectional-sync preview: import + diff + optional render evidence.
    pub fn catalog_sync(
        &self,
        render_out_dir: Option<&Path>,
        apply: bool,
    ) -> anyhow::Result<CatalogSyncReport> {
        let root = workspace_root_for_manifest_dir(&self.inner.manifest_dir);
        catalog::sync(
            catalog::CatalogSyncSpec {
                repo_root: root,
                manifest_dir: self.inner.manifest_dir.clone(),
                render_out_dir: render_out_dir.map(Path::to_path_buf),
                apply,
            },
            &self.inner.registry,
        )
    }

    /// ADR-0003 catalog-native lock check/update for `manifest/envctl.lock`.
    pub fn catalog_lock(&self, apply: bool) -> anyhow::Result<CatalogLockReport> {
        let root = workspace_root_for_manifest_dir(&self.inner.manifest_dir);
        catalog::lock(
            catalog::CatalogLockSpec {
                repo_root: root,
                manifest_dir: self.inner.manifest_dir.clone(),
                apply,
            },
            &self.inner.registry,
        )
    }

    /// Render deterministic ADR-0003 catalog projections into an explicit output dir.
    pub fn catalog_render(&self, out_dir: impl AsRef<Path>) -> anyhow::Result<CatalogRenderReport> {
        let root = workspace_root_for_manifest_dir(&self.inner.manifest_dir);
        catalog::render(
            catalog::CatalogRenderSpec {
                repo_root: root,
                manifest_dir: self.inner.manifest_dir.clone(),
                out_dir: out_dir.as_ref().to_path_buf(),
            },
            &self.inner.registry,
        )
    }

    /// The manifest directory (where `envctl.lock` + `components.d/` live).
    pub fn manifest_dir(&self) -> &std::path::Path {
        &self.inner.manifest_dir
    }

    /// THE shared mutating entrypoint (install/reset/auto-fix). Best-effort:
    /// `Err` only for setup-time problems; `Ok(summary)` where `!summary.ok()`
    /// means some components failed or were refused. Emits Events into `sink`.
    pub fn run(&self, plan: RunPlan, sink: &EventSink) -> anyhow::Result<RunSummary> {
        let phase = plan.phase;
        let dry = plan.dry_run;
        let summary = executor::run(&self.inner.registry, self.inner.runner.as_ref(), plan, sink)?;
        if !dry {
            // best-effort machine-local last-run record (out of the committed lock)
            crate::runtime::record_run(&self.inner.manifest_dir, phase, &summary);
        }
        Ok(summary)
    }

    /// Read-only auto-detect. Never writes. Used identically by `envctl
    /// auto-detect` and the GUI status grid. Emits a final `Event::Report`.
    pub fn detect(&self, sink: &EventSink) -> anyhow::Result<EnvReport> {
        detect::run(&self.inner.registry, self.inner.runner.as_ref(), sink)
    }

    /// add-repo: synthesize a build-from-source Component, persist a drop-in
    /// under `<manifest_dir>/components.d/<id>.toml` (atomic + backed up), then
    /// (unless dry_run) install it.
    /// Interactive handoff: clone + drop the user into an agent session in the
    /// clone (for cherry-pick / port-to-rust). Blocks on the real terminal; runs
    /// on the caller's (main) thread, NOT the GUI worker. Never as root.
    pub fn connect_repo(&self, spec: &AddRepoSpec) -> anyhow::Result<()> {
        crate::addrepo::connect_agent(spec)
    }

    pub fn add_repo(
        &self,
        spec: AddRepoSpec,
        dry_run: bool,
        sink: &EventSink,
    ) -> anyhow::Result<RunSummary> {
        executor::add_repo(
            &self.inner.manifest_dir,
            &self.inner.registry,
            self.inner.runner.as_ref(),
            spec,
            dry_run,
            sink,
        )
    }

    /// Read-only: locate + read `.meta.yaml` (walking up from `start`, or the
    /// override) and render the zellij dashboard layout. Never writes. Emits a
    /// final `Event::Dashboard` with the plan. Used identically by `envctl
    /// dashboard` and the GUI parity action.
    pub fn dashboard(
        &self,
        start: PathBuf,
        meta_file: Option<PathBuf>,
        spec: DashboardSpec,
        sink: &EventSink,
    ) -> anyhow::Result<DashboardPlan> {
        let file = dashboard::locate_meta_file(&start, meta_file.as_deref())?;
        let workspace = dashboard::read_workspace(&file)?;
        let plan = dashboard::render(&workspace, &spec);
        sink.emit(Event::Dashboard { plan: plan.clone() });
        Ok(plan)
    }

    /// Fail-closed deploy of the rendered dashboard layout to the yazelix zellij
    /// layouts dir. DRY-RUN by default; the write only happens with `dry_run =
    /// false`. Refuses to clobber a non-envctl file without `force`. Emits the
    /// `Event::Dashboard` plan plus a `DashboardDeployed` outcome.
    pub fn deploy_dashboard(
        &self,
        start: PathBuf,
        meta_file: Option<PathBuf>,
        spec: DashboardSpec,
        dry_run: bool,
        force: bool,
        sink: &EventSink,
    ) -> anyhow::Result<DeployOutcome> {
        let file = dashboard::locate_meta_file(&start, meta_file.as_deref())?;
        let workspace = dashboard::read_workspace(&file)?;
        let plan = dashboard::render(&workspace, &spec);
        sink.emit(Event::Dashboard { plan: plan.clone() });
        let outcome = dashboard::deploy(&plan, dry_run, force)?;
        sink.emit(Event::DashboardDeployed {
            outcome: outcome.clone(),
        });
        Ok(outcome)
    }

    /// Read-only migration/adoption scan: inventory canonical meta FHS/XDG dirs,
    /// legacy manifest tokens, preserved agent assets, and protected meta shared
    /// substrates such as `loop_lib`.
    pub fn migrate_scan(
        &self,
        spec: MigrationSpec,
        sink: &EventSink,
    ) -> anyhow::Result<MigrationReport> {
        let report = migration::scan(&self.inner.registry, &self.inner.manifest_dir, &spec);
        migration::emit_report(&report, sink);
        Ok(report)
    }

    /// Read-only migration plan. Same inventory as scan, labeled as the plan
    /// surface so front-ends can show the next action without mutating state.
    pub fn migrate_plan(
        &self,
        spec: MigrationSpec,
        sink: &EventSink,
    ) -> anyhow::Result<MigrationReport> {
        let report = migration::plan(&self.inner.registry, &self.inner.manifest_dir, &spec);
        migration::emit_report(&report, sink);
        Ok(report)
    }

    /// Apply the safe subset of the migration plan. Preview unless `apply=true`;
    /// the first implementation only materializes canonical meta FHS/XDG
    /// directories and writes the migration ledger.
    pub fn migrate_apply(
        &self,
        spec: MigrationSpec,
        apply: bool,
        sink: &EventSink,
    ) -> anyhow::Result<MigrationReport> {
        let report =
            migration::apply(&self.inner.registry, &self.inner.manifest_dir, &spec, apply)?;
        migration::emit_report(&report, sink);
        Ok(report)
    }

    /// Verify that the migration plan is fully resolved. Non-mutating; callers
    /// may choose to exit non-zero when `report.ok()` is false.
    pub fn migrate_verify(
        &self,
        spec: MigrationSpec,
        sink: &EventSink,
    ) -> anyhow::Result<MigrationReport> {
        let report = migration::verify(&self.inner.registry, &self.inner.manifest_dir, &spec);
        migration::emit_report(&report, sink);
        Ok(report)
    }

    /// Strict-upgrade purge surface. Preview unless `apply=true` and `confirmed=true`;
    /// the engine refuses to delete any legacy path that has not first been proven
    /// adopted into a canonical replacement and ledgered.
    pub fn migrate_purge(
        &self,
        spec: MigrationSpec,
        apply: bool,
        confirmed: bool,
        sink: &EventSink,
    ) -> anyhow::Result<MigrationReport> {
        let report = migration::purge(
            &self.inner.registry,
            &self.inner.manifest_dir,
            &spec,
            apply,
            confirmed,
        )?;
        migration::emit_report(&report, sink);
        Ok(report)
    }
}

fn workspace_root_for_manifest_dir(manifest_dir: &std::path::Path) -> std::path::PathBuf {
    match manifest_dir.parent() {
        Some(parent) if parent.as_os_str().is_empty() => {
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        }
        Some(parent) if parent.is_absolute() => parent.to_path_buf(),
        Some(parent) => std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join(parent),
        None => std::env::current_dir().unwrap_or_else(|_| manifest_dir.to_path_buf()),
    }
}

/// Process-wide lock serializing tests that MUTATE environment variables
/// (`HOME` / `XDG_*` / `ENVCTL_CACHE_DIR`) against each other AND against tests
/// that READ env-derived paths (e.g. `agent::init`'s global-path resolution).
/// Without it, parallel `cargo test` lets one test's `set_var`/`remove_var` race
/// another's env read — the cause of the `init_path_global` CI flake. Resilient
/// to poisoning so one panicking test can't cascade-fail the rest.
#[cfg(test)]
pub(crate) fn test_env_lock() -> std::sync::MutexGuard<'static, ()> {
    use std::sync::{Mutex, OnceLock};
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}
