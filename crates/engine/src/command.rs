//! The GUI worker API. `EngineCommand` (App→worker) and `EngineEvent` (worker→App,
//! the same `Event` vocabulary the CLI sees) plus `run_event_loop`, which the GUI
//! spawns on ONE thread. Every parameter is `Send + 'static`, so the
//! `std::thread::spawn` closure in the GUI satisfies its bounds.
//!
//! Live streaming: a forwarder thread relays engine events to the UI as they
//! arrive. Telemetry (Phase 5): a DEDICATED sampler thread emits `Event::Telemetry`
//! on a cadence the GUI controls via `TelemetryControl` (backoff when off-Dashboard
//! / unfocused), so a long `engine.run` never starves telemetry.
use crate::{
    agent::{
        AgentAddSpec, AgentCleanSpec, AgentDoctorSpec, AgentListSpec, AgentLockSpec,
        AgentRemoveSpec, AgentSyncSpec,
    },
    component::Phase,
    dashboard::DashboardSpec,
    model::{AddRepoSpec, RunPlan},
    Engine, Event, EventSink,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

#[derive(Clone, Debug)]
// One command is sent across the worker channel per user action, so the inter-variant
// size delta is irrelevant; boxing would add indirection for no real gain. Keep value-typed.
#[allow(clippy::large_enum_variant)]
pub enum EngineCommand {
    Detect,
    Install {
        targets: Vec<String>,
        dry_run: bool,
    },
    Fix {
        targets: Vec<String>,
        dry_run: bool,
    },
    Remove {
        targets: Vec<String>,
        dry_run: bool,
    },
    AddRepo {
        spec: AddRepoSpec,
        dry_run: bool,
    },
    /// Read-only: render the mission-control dashboard from `.meta.yaml`.
    Dashboard {
        start: PathBuf,
        meta_file: Option<PathBuf>,
        spec: DashboardSpec,
    },
    /// Fail-closed deploy of the rendered dashboard (dry-run unless `dry_run=false`).
    DeployDashboard {
        start: PathBuf,
        meta_file: Option<PathBuf>,
        spec: DashboardSpec,
        dry_run: bool,
        force: bool,
    },
    /// An agent-asset verb (sync/add/remove/lock/list/clean). The engine owns the fail-closed
    /// preview-vs-apply policy via each `Agent*Spec`'s `apply` flag; the GUI only builds the Spec.
    Agent {
        spec: AgentCommandSpec,
    },
    /// TASK-0028 (Architecture B): drive the installed `secretctl` binary as a subprocess for the
    /// GUI secrets verbs (mint-github / relay-mint / revoke). `verb` is the stable result label;
    /// `argv` is the IDENTICAL `secretctl` clap surface the CLI drives (built by the GUI as pure
    /// strings); `stdin` is an OPTIONAL secret buffer (`Zeroizing` — the revoke `--token -` token)
    /// piped to the child's stdin, never to argv. The engine spawns, captures stdout/stderr/exit,
    /// emits `Event::SecretsResult`, and holds no token after the child exits.
    Secrets {
        verb: String,
        argv: Vec<String>,
        stdin: Option<zeroize::Zeroizing<Vec<u8>>>,
    },
    SampleTelemetry,
    Shutdown,
}

/// The six agent-asset verbs, each wrapping its typed spec. The GUI builds one of these from
/// its form state (the identical `Agent*Spec` the CLI builds from clap) and sends it through
/// `EngineCommand::Agent`; `run_event_loop` dispatches to the matching `Engine::agent_*` method.
/// This keeps the agent business logic in the engine — the GUI only constructs the Spec + drains
/// the resulting events (parity with the CLI's `run_agent`).
#[derive(Clone, Debug)]
pub enum AgentCommandSpec {
    Sync(AgentSyncSpec),
    Add(AgentAddSpec),
    Remove(AgentRemoveSpec),
    Lock(AgentLockSpec),
    List(AgentListSpec),
    Clean(AgentCleanSpec),
    /// Read-only diagnostics (TASK-0019, Item 1) — the GUI Doctor sub-tab drives the identical
    /// `Engine::agent_doctor` the CLI's `agent doctor` does.
    Doctor(AgentDoctorSpec),
}

pub type EngineEvent = Event;

/// Shared control for the telemetry sampler thread. The GUI sets `cadence_ms`
/// (e.g. 1000 on Dashboard-focused, 10000 elsewhere) and calls `wake()` to sample
/// now. `Clone` is cheap (all Arc).
#[derive(Clone)]
pub struct TelemetryControl {
    alive: Arc<AtomicBool>,
    cadence_ms: Arc<AtomicU64>,
    wake: Arc<Condvar>,
    // audit fix (minor): the mutex now guards a generation counter, bumped under
    // lock on every set_cadence/sample_now so a notify landing in the gap before
    // the sampler waits is never lost (the sampler waits on `gen == last_seen`).
    gen: Arc<Mutex<u64>>,
}
impl Default for TelemetryControl {
    fn default() -> Self {
        TelemetryControl {
            alive: Arc::new(AtomicBool::new(true)),
            cadence_ms: Arc::new(AtomicU64::new(1000)),
            wake: Arc::new(Condvar::new()),
            gen: Arc::new(Mutex::new(0)),
        }
    }
}
impl TelemetryControl {
    pub fn new() -> Self {
        Self::default()
    }
    /// Set the sampling cadence (ms, clamped to >=250) and wake the sampler.
    pub fn set_cadence(&self, ms: u64) {
        self.cadence_ms.store(ms.max(250), Ordering::Relaxed);
        self.bump_and_notify();
    }
    pub fn sample_now(&self) {
        self.bump_and_notify();
    }
    /// Bump the generation under the lock, THEN notify. Holding the lock across
    /// the bump means a sampler about to wait observes the new generation (and
    /// skips the wait) instead of losing the notification in the gap.
    fn bump_and_notify(&self) {
        {
            let mut g = self.gen.lock().unwrap();
            *g = g.wrapping_add(1);
        }
        self.wake.notify_all();
    }
    fn stop(&self) {
        self.alive.store(false, Ordering::Relaxed);
        self.bump_and_notify();
    }
}

fn spawn_sampler(sink: EventSink, ctrl: TelemetryControl) {
    std::thread::spawn(move || {
        let mut last_seen = *ctrl.gen.lock().unwrap();
        while ctrl.alive.load(Ordering::Relaxed) {
            sink.emit(Event::Telemetry(crate::telemetry::sample()));
            let cadence = Duration::from_millis(ctrl.cadence_ms.load(Ordering::Relaxed).max(250));
            // audit fix (minor): wait on a predicate (generation unchanged) so a
            // set_cadence/sample_now/stop notify that lands before we wait is not
            // lost — the predicate is already true and we wake immediately.
            let guard = ctrl.gen.lock().unwrap();
            let (guard, _) = ctrl
                .wake
                .wait_timeout_while(guard, cadence, |g| *g == last_seen)
                .unwrap();
            last_seen = *guard;
            drop(guard);
            // audit fix (minor): re-check alive right after the wait so a stopped
            // sampler exits promptly instead of doing one more sample/emit during
            // teardown.
            if !ctrl.alive.load(Ordering::Relaxed) {
                break;
            }
        }
    });
}

pub fn run_event_loop(
    engine: Engine,
    cmd_rx: Receiver<EngineCommand>,
    evt_tx: Sender<EngineEvent>,
    ctrl: TelemetryControl,
    mut repaint: Box<dyn FnMut() + Send + 'static>,
) {
    let (sink, ev_rx) = EventSink::channel();

    // Forwarder: relay every engine event to the UI the instant it's emitted.
    std::thread::spawn(move || {
        while let Ok(ev) = ev_rx.recv() {
            if evt_tx.send(ev).is_err() {
                break;
            }
            repaint();
        }
    });

    // Dedicated telemetry sampler (cadence controlled by the GUI).
    spawn_sampler(sink.clone(), ctrl.clone());

    while let Ok(cmd) = cmd_rx.recv() {
        match cmd {
            EngineCommand::Shutdown => {
                ctrl.stop();
                break;
            }
            EngineCommand::Detect => {
                if let Err(e) = engine.detect(&sink) {
                    emit_setup_error(&sink, "detect", &e);
                }
            }
            EngineCommand::Install { targets, dry_run } => {
                if let Err(e) = engine.run(RunPlan::new(Phase::Install, targets, dry_run), &sink) {
                    emit_setup_error(&sink, "install", &e);
                }
            }
            EngineCommand::Fix { targets, dry_run } => {
                if let Err(e) = engine.run(RunPlan::new(Phase::Fix, targets, dry_run), &sink) {
                    emit_setup_error(&sink, "fix", &e);
                }
            }
            EngineCommand::Remove { targets, dry_run } => {
                if let Err(e) = engine.run(RunPlan::new(Phase::Remove, targets, dry_run), &sink) {
                    emit_setup_error(&sink, "remove", &e);
                }
            }
            EngineCommand::AddRepo { spec, dry_run } => {
                if let Err(e) = engine.add_repo(spec, dry_run, &sink) {
                    emit_setup_error(&sink, "add-repo", &e);
                }
            }
            EngineCommand::Dashboard {
                start,
                meta_file,
                spec,
            } => {
                if let Err(e) = engine.dashboard(start, meta_file, spec, &sink) {
                    emit_setup_error(&sink, "dashboard", &e);
                }
            }
            EngineCommand::DeployDashboard {
                start,
                meta_file,
                spec,
                dry_run,
                force,
            } => {
                if let Err(e) =
                    engine.deploy_dashboard(start, meta_file, spec, dry_run, force, &sink)
                {
                    emit_setup_error(&sink, "dashboard-deploy", &e);
                }
            }
            EngineCommand::Agent { spec } => {
                // Dispatch to the matching Engine::agent_* method, mirroring the CLI's `run_agent`
                // arms. The typed return is dropped here — the GUI consumes the run via the
                // events the verb emits (AgentRunStarted/Action/RunFinished/LockChecked plus the
                // new AgentListed/AgentEdited). A setup-time Err becomes a GuardRefused.
                let result = match spec {
                    AgentCommandSpec::Sync(s) => engine.agent_sync(s, &sink).map(|_| ()),
                    AgentCommandSpec::Add(s) => engine.agent_add(s, &sink).map(|_| ()),
                    AgentCommandSpec::Remove(s) => engine.agent_remove(s, &sink).map(|_| ()),
                    AgentCommandSpec::Lock(s) => engine.agent_lock(s, &sink).map(|_| ()),
                    AgentCommandSpec::List(s) => engine.agent_list(s, &sink).map(|_| ()),
                    AgentCommandSpec::Clean(s) => engine.agent_clean(s, &sink).map(|_| ()),
                    AgentCommandSpec::Doctor(s) => engine.agent_doctor(s, &sink).map(|_| ()),
                };
                if let Err(e) = result {
                    emit_setup_error(&sink, "agent", &e);
                }
            }
            EngineCommand::Secrets { verb, argv, stdin } => {
                // Engine-owned subprocess seam: spawn `secretctl`, pipe the (optional) Zeroizing
                // secret stdin, capture stdout/stderr/exit, emit a single SecretsResult. The engine
                // parses nothing secret and holds no token after the child exits. Fail-closed: a
                // missing binary emits an error result (never panics — handled inside run_secretctl).
                crate::secrets::run_secretctl(verb, argv, stdin, &sink);
            }
            EngineCommand::SampleTelemetry => {
                ctrl.sample_now(); // wake the sampler; it no longer samples on this thread
            }
        }
    }
}

// audit fix (minor): setup-time failures (invalid slug, duplicate id, validation)
// return Err BEFORE the engine emits any event, so the GUI would otherwise see
// nothing. Surface them as a GuardRefused, which the GUI already renders as a
// stderr "REFUSED" log line.
fn emit_setup_error(sink: &EventSink, op: &str, err: &anyhow::Error) {
    sink.emit(Event::GuardRefused {
        component: op.to_string(),
        reason: format!("{err:#}"),
    });
}
