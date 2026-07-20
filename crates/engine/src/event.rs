//! Progress/log streaming. The engine NEVER prints; it emits `Event`s into an
//! `EventSink` (a newtype over `mpsc::Sender<Event>`). All payloads are
//! `Send + 'static`, so events cross the GUI worker→UI channel unchanged, and the
//! CLI drains the same vocabulary. (`EventSink::channel()`, not `new()`, keeps
//! clippy's `new_ret_no_self` happy — it returns a channel pair, not `Self`.)
use crate::agent::report::{
    AgentAuditReport, AgentDoctorReport, AgentEditOutcome, AgentInitOutcome, AgentList,
    AgentLockDriftItem, AgentReport, AgentVerb,
};
use crate::agent::AgentScope;
use crate::component::Phase;
use crate::dashboard::{DashboardPlan, DeployOutcome};
use crate::doctor::DoctorReport;
use crate::model::{EnvReport, OpResult, RunSummary};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    RunStarted {
        phase: Phase,
        total: usize,
        dry_run: bool,
    },
    StepStarted {
        component: String,
        phase: Phase,
        index: usize,
        total: usize,
    },
    Log {
        component: String,
        stream: Stream,
        line: String,
    },
    StepFinished {
        result: OpResult,
    },
    Telemetry(Telemetry),
    /// The read-only inventory, emitted at the end of auto-detect (drives the
    /// GUI Components grid + Dashboard).
    Report {
        report: EnvReport,
    },
    /// The typed whole-environment diagnostic report. Emitted exactly once by
    /// `Engine::doctor`; both front-ends consume this same payload.
    Doctored {
        report: DoctorReport,
    },
    GuardRefused {
        component: String,
        reason: String,
    },
    RunFinished {
        summary: RunSummary,
    },
    /// The rendered mission-control dashboard plan (read-only render result).
    /// Drives the CLI stdout/JSON output + the GUI dashboard preview.
    Dashboard {
        plan: DashboardPlan,
    },
    /// The outcome of a dashboard deploy (dry-run preview or applied write).
    DashboardDeployed {
        outcome: DeployOutcome,
    },
    /// Deterministic catalog projections rendered outside the repo root.
    CatalogRendered {
        report: Box<crate::catalog::CatalogRenderReport>,
    },
    /// The migration/adoption report over the meta-hosted install topology. This
    /// is emitted by every `migrate` verb so the CLI and GUI consume one shared
    /// engine surface for scan/plan/apply/verify/purge.
    MigrationReported {
        report: Box<crate::migration::MigrationReport>,
    },
    /// An agent-asset verb run started (sync/add/remove/lock/list/clean). `dry_run`
    /// reflects preview-vs-apply; `lock_mode` is the resolved mode label.
    AgentRunStarted {
        verb: AgentVerb,
        scope: AgentScope,
        dry_run: bool,
        lock_mode: String,
    },
    /// One per-asset action from an agent-asset verb (live tree for GUI/CLI): the
    /// source it came from, the asset name, the outcome status, and any error detail.
    AgentAction {
        source: Option<String>,
        asset: Option<String>,
        status: String,
        error: Option<String>,
    },
    /// An agent-asset verb run finished — carries the full typed report.
    AgentRunFinished {
        report: AgentReport,
    },
    /// The drift result of `agent_lock --check` (empty = lock is up to date).
    AgentLockChecked {
        drift: Vec<AgentLockDriftItem>,
    },
    /// The read-only inventory from `agent_list`, emitted just before the typed return.
    /// `list` emits only `AgentRunStarted` otherwise, so its rows live only in the return
    /// value (CLI prints via `render_agent_list`). The GUI worker→UI channel is event-only,
    /// so this carries the list to the GUI without changing the CLI's typed-return render.
    AgentListed {
        list: AgentList,
    },
    /// The edit outcome from `agent_add` / `agent_remove`, emitted at the tail (after the
    /// optional follow-up sync). The preview `would_add`/`would_remove` items are in NO other
    /// event; this transports them to the GUI. (CLI keeps its typed-return render, unchanged.)
    AgentEdited {
        outcome: AgentEditOutcome,
    },
    /// The outcome of `agent init`: the config file path that was created and whether an
    /// existing file was overwritten. Emitted so the GUI can update its status even though
    /// `init` has no per-action tree.
    AgentInitFinished {
        outcome: AgentInitOutcome,
    },
    /// The read-only diagnostic report from `agent_doctor`. Emitted once per run; carries the
    /// full report so the GUI Doctor tab renders the identical data the CLI prints.
    AgentDoctored {
        report: AgentDoctorReport,
    },
    /// The strict read-only config → lock → installed-assets audit used by Meta fleet policy.
    AgentAudited {
        report: AgentAuditReport,
    },
    /// The outcome of `self uninstall`: a preview (dry-run, zero writes) or applied teardown.
    /// Emitted once; the destructive removal counts + any fail-closed refusal reason.
    SelfUninstall {
        outcome: crate::self_uninstall::SelfUninstallOutcome,
    },
    /// TASK-0028: the result of an engine-driven `secretctl` subprocess (the GUI secrets verbs
    /// mint-github / relay-mint / revoke under Architecture B). `verb` is the stable label the GUI
    /// keys off; `json_stdout` is the child's stdout (parsed by the GUI for METADATA only — the
    /// mint token is held transiently and never persisted); `stderr` is surfaced verbatim in a
    /// fail-closed DANGER card; `code` is the child exit status (`None` ⇒ not spawned / not found).
    /// The engine parses nothing secret and holds no token after the child exits.
    SecretsResult {
        verb: String,
        json_stdout: String,
        stderr: String,
        code: Option<i32>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stream {
    Stdout,
    Stderr,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Telemetry {
    pub at_ms: u128,
    pub gpus: Vec<GpuSample>,
    pub load_avg: Option<f32>,
    pub mem_used_mb: Option<u64>,
    pub mem_total_mb: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GpuSample {
    pub index: u32,
    pub name: String,
    pub util_pct: u32,
    pub mem_used_mb: u64,
    pub mem_total_mb: u64,
    pub temp_c: u32,
    pub power_w: Option<u32>,
}

/// Send-able sink: `Sender<Event>` is `Send`, so an `EventSink` moves into the
/// worker thread.
#[derive(Clone)]
pub struct EventSink(Sender<Event>);

impl EventSink {
    /// Construct a sink + its receiving end. (Named `channel`, not `new`, on
    /// purpose: it returns a pair, not `Self`.)
    pub fn channel() -> (EventSink, Receiver<Event>) {
        let (tx, rx) = std::sync::mpsc::channel();
        (EventSink(tx), rx)
    }

    /// A sink whose receiver is dropped: every `emit` is silently discarded.
    /// Used where events aren't consumed (e.g. guard predicate hooks).
    pub fn null() -> EventSink {
        let (tx, _rx) = std::sync::mpsc::channel();
        EventSink(tx)
    }

    pub fn emit(&self, ev: Event) {
        let _ = self.0.send(ev);
    }

    /// Time a closure and stamp `duration_ms` on its result. (The caller — the
    /// executor — owns `StepFinished` emission, so every component emits exactly
    /// one, hook or not.)
    pub fn timed<F: FnOnce() -> OpResult>(&self, f: F) -> OpResult {
        let t = Instant::now();
        let mut r = f();
        r.duration_ms = t.elapsed().as_millis();
        r
    }
}
