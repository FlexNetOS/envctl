//! `envctl-gui` — native egui/eframe dashboard over the shared engine.
//!
//! ONE worker thread runs `run_event_loop`. The spawn closure captures only
//! `Send + 'static` values — an owned `Engine` clone, the mpsc endpoints, and a
//! `Box<dyn FnMut() + Send + 'static>` repaint hook built from a cloned
//! `egui::Context` (Arc-backed). `update()` drains events non-blocking via
//! `try_recv`, so the UI thread never blocks on engine work. This file is the
//! explicit proof the worker-closure bounds hold.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod theme;

use eframe::egui::{self, Color32, RichText};
use egui_extras::{Column, TableBuilder};
use envctl_engine::{
    run_event_loop, AddRepoSpec, AgentAddSpec, AgentCleanSpec, AgentCommandSpec, AgentDoctorReport,
    AgentDoctorSpec, AgentEditOutcome, AgentList, AgentListKind, AgentListSpec, AgentLockDriftItem,
    AgentLockMode, AgentLockSpec, AgentRemoveSpec, AgentReport, AgentScope, AgentSectionSel,
    AgentSyncSpec, BuildStrategy, ComponentState, DashboardPlan, DashboardSpec, DriftItem,
    DriftKind, Engine, EngineCommand, EngineEvent, Event, OpStatus, Refactor, RefactorGoal,
    RenameRule, Severity, Stream, Telemetry, TelemetryControl, Zeroizing,
};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::mpsc::{channel, Receiver, Sender};

fn main() -> eframe::Result<()> {
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1040.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "envctl",
        opts,
        Box::new(|cc| Ok(Box::new(EnvctlApp::new(cc)))),
    )
}

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Dashboard,
    Components,
    Graph,
    AddRepo,
    Agent,
    Secrets,
    Mesh,
    Logs,
    Settings,
}

impl Screen {
    fn label(self) -> &'static str {
        match self {
            Screen::Dashboard => "Dashboard",
            Screen::Components => "Components",
            Screen::Graph => "Graph",
            Screen::AddRepo => "Add Repo",
            Screen::Agent => "Agent",
            Screen::Secrets => "Secrets",
            Screen::Mesh => "Mesh",
            Screen::Logs => "Logs",
            Screen::Settings => "Settings",
        }
    }
}

/// The active verb sub-tab on the Secrets screen (TASK-0028). Each maps to a `secretctl`
/// invocation the GUI drives via the engine subprocess seam (Architecture B).
#[derive(Clone, Copy, PartialEq)]
enum SecretsVerbTab {
    MintGithub,
    RelayMint,
    Revoke,
}

impl SecretsVerbTab {
    fn label(self) -> &'static str {
        match self {
            SecretsVerbTab::MintGithub => "Mint GitHub",
            SecretsVerbTab::RelayMint => "Relay mint",
            SecretsVerbTab::Revoke => "Revoke",
        }
    }
}

/// METADATA-ONLY relay-mint result (TASK-0028). `secretctl relay mint --json` emits
/// `{bearer, token_id, expires_at, native}`; the GUI parses it and keeps ONLY these three
/// non-secret fields — the `bearer` (secret-class) is NEVER stored or rendered.
#[derive(Clone, Debug, PartialEq)]
struct RelayMintMeta {
    token_id: String,
    expires_at: String,
    native: bool,
}

/// The active verb sub-tab on the Agent screen (one of the six agent-asset verbs).
#[derive(Clone, Copy, PartialEq)]
enum AgentVerbTab {
    Sync,
    Add,
    Remove,
    Lock,
    List,
    Clean,
    Doctor,
}

impl AgentVerbTab {
    fn label(self) -> &'static str {
        match self {
            AgentVerbTab::Sync => "Sync",
            AgentVerbTab::Add => "Add",
            AgentVerbTab::Remove => "Remove",
            AgentVerbTab::Lock => "Lock",
            AgentVerbTab::List => "List",
            AgentVerbTab::Clean => "Clean",
            AgentVerbTab::Doctor => "Doctor",
        }
    }
}

/// The scope selector shared by every agent verb form (`--scope`). `Default` = no override
/// (engine resolves from the config); `Global`/`Project` map to an `AgentScope` override.
#[derive(Clone, Copy, PartialEq)]
enum AgentScopeSel {
    Default,
    Global,
    Project,
}

impl AgentScopeSel {
    fn label(self) -> &'static str {
        match self {
            AgentScopeSel::Default => "default (from config)",
            AgentScopeSel::Global => "global",
            AgentScopeSel::Project => "project",
        }
    }

    /// Map to the engine `scope_override` (blank/default → `None`), exactly as the CLI does
    /// via `scope.map(AgentScope::from)`.
    fn to_override(self) -> Option<AgentScope> {
        match self {
            AgentScopeSel::Default => None,
            AgentScopeSel::Global => Some(AgentScope::Global),
            AgentScopeSel::Project => Some(AgentScope::Project),
        }
    }
}

/// The `list --kind` selector.
#[derive(Clone, Copy, PartialEq)]
enum AgentListKindSel {
    All,
    Skills,
    Mcps,
    Commands,
}

impl AgentListKindSel {
    fn label(self) -> &'static str {
        match self {
            AgentListKindSel::All => "all",
            AgentListKindSel::Skills => "skills",
            AgentListKindSel::Mcps => "mcps",
            AgentListKindSel::Commands => "commands",
        }
    }

    fn to_kind(self) -> AgentListKind {
        match self {
            AgentListKindSel::All => AgentListKind::All,
            AgentListKindSel::Skills => AgentListKind::Skills,
            AgentListKindSel::Mcps => AgentListKind::Mcps,
            AgentListKindSel::Commands => AgentListKind::Commands,
        }
    }
}

/// One captured log line, with its originating stream so the console can color
/// stderr distinctly. (Owned `String` + `Copy` enum: trivially `Send`.)
struct LogLine {
    stream: Stream,
    text: String,
}

/// Recent GPU utilization for sparklines, keyed by GPU index. Owned `VecDeque`s
/// of plain numbers: `Send`, lives entirely on the UI thread.
const SPARK_LEN: usize = 60;

struct EnvctlApp {
    cmd_tx: Sender<EngineCommand>,
    evt_rx: Receiver<EngineEvent>,
    screen: Screen,
    header: String,
    components: Vec<ComponentState>,
    drift: Vec<DriftItem>,
    busy: HashSet<String>,
    log: VecDeque<LogLine>,
    log_cap: usize,
    telemetry: Option<Telemetry>,
    util_history: HashMap<u32, VecDeque<f32>>,
    dry_run_default: bool,
    filter: String,
    tel: TelemetryControl,
    // read-only engine clone for on-thread graph queries
    geng: Engine,
    graph_focus: String,
    // GPU summary (from the last EnvReport) for the DriverNotActive card
    gpu_present: bool,
    driver_loaded: bool,
    software_rendered: bool,
    gpu_count: usize,
    // add-repo form
    add_url: String,
    add_id: String,
    add_build: String,
    add_strategy: String,
    add_ref: String,
    add_bins: String,
    add_renames: String,
    add_patch: String,
    add_ai_goal: String,
    add_ai_instruction: String,
    add_build_flag: bool,
    // meta mission-control dashboard parity
    dash_plan: Option<DashboardPlan>,
    dash_panes_per_tab: usize,
    dash_status: String,
    // agent-env panel — the active verb sub-tab + shared form inputs
    agent_verb: AgentVerbTab,
    agent_config: String,
    agent_scope: AgentScopeSel,
    agent_source: String,
    agent_skills: String,
    agent_mcps: String,
    agent_commands: String,
    agent_git_ref: String,
    agent_branch: String,
    agent_sub_dir: String,
    agent_apply: bool,
    agent_no_sync: bool,
    agent_no_verify: bool,
    agent_locked: bool,
    agent_update_on: bool,
    agent_update: String,
    agent_lock_check: bool,
    agent_upgrade_pkg: String,
    agent_list_kind: AgentListKindSel,
    // agent-env panel — result holders (filled from the worker events)
    agent_list: Option<AgentList>,
    agent_list_stale: bool,
    agent_last_edit: Option<AgentEditOutcome>,
    agent_last_report: Option<AgentReport>,
    agent_lock_drift: Option<Vec<AgentLockDriftItem>>,
    agent_last_doctor: Option<AgentDoctorReport>,
    agent_status: String,
    // ── Secrets panel (TASK-0028) — the active verb sub-tab + per-verb form inputs ──────────
    secrets_verb: SecretsVerbTab,
    // mint-github form
    sec_install_id: String,
    sec_repo_ids: String,
    sec_perms: String,
    sec_ttl_secs: String,
    // relay-mint form
    sec_relay_name: String,
    sec_relay_ttl: String,
    sec_relay_mode: String,
    sec_relay_provider: String,
    sec_relay_repos: String,
    sec_relay_perms: String,
    // revoke form — `sec_revoke_token` is TRANSIENT secret input, cleared on dispatch, never persisted
    sec_revoke_token: String,
    sec_revoke_install_id: String,
    sec_revoke_apply: bool,
    // results (METADATA ONLY — no secret bytes)
    sec_status: String,
    sec_mint_expires: Option<i64>,
    sec_mint_has_token: bool,
    // token to copy ONCE: held only between a mint SecretsResult landing and the next copy-affordance
    // render that consumes it via ui.output().copied_text, then immediately cleared. Never persisted,
    // never to push_log / self.log.
    sec_mint_copy_once: Option<String>,
    sec_relay_result: Option<RelayMintMeta>,
    sec_revoke_result: Option<(bool, bool)>, // (revoked, dry_run)
}

impl EnvctlApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply(&cc.egui_ctx);

        let (cmd_tx, cmd_rx) = channel::<EngineCommand>();
        let (evt_tx, evt_rx) = channel::<EngineEvent>();
        let ctx = cc.egui_ctx.clone(); // Arc-backed: Send + Sync + 'static

        // THE worker spawn. Every captured value is Send + 'static => the closure
        // is Send + 'static => std::thread::spawn accepts it.
        let engine = Engine::load_default().expect("manifest load");
        let geng = engine.clone(); // read-only clone for graph queries on the UI thread
        let tel = TelemetryControl::new();
        let tel_worker = tel.clone();
        std::thread::spawn(move || {
            let repaint: Box<dyn FnMut() + Send + 'static> =
                Box::new(move || ctx.request_repaint());
            run_event_loop(engine, cmd_rx, evt_tx, tel_worker, repaint);
        });

        let app = Self {
            cmd_tx,
            evt_rx,
            screen: Screen::Dashboard,
            header: "scanning…".into(),
            components: Vec::new(),
            drift: Vec::new(),
            busy: HashSet::new(),
            log: VecDeque::new(),
            log_cap: 8000,
            telemetry: None,
            util_history: HashMap::new(),
            dry_run_default: true,
            filter: String::new(),
            tel,
            geng,
            graph_focus: String::new(),
            gpu_present: false,
            driver_loaded: false,
            software_rendered: false,
            gpu_count: 0,
            add_url: String::new(),
            add_id: String::new(),
            add_build: String::new(),
            add_strategy: "as-is".into(),
            add_ref: String::new(),
            add_bins: String::new(),
            add_renames: String::new(),
            add_patch: String::new(),
            add_ai_goal: "port-to-rust".into(),
            add_ai_instruction: String::new(),
            add_build_flag: false,
            dash_plan: None,
            dash_panes_per_tab: 6,
            dash_status: String::new(),
            agent_verb: AgentVerbTab::Sync,
            agent_config: String::new(),
            agent_scope: AgentScopeSel::Default,
            agent_source: String::new(),
            agent_skills: String::new(),
            agent_mcps: String::new(),
            agent_commands: String::new(),
            agent_git_ref: String::new(),
            agent_branch: String::new(),
            agent_sub_dir: String::new(),
            agent_apply: false,
            agent_no_sync: false,
            agent_no_verify: false,
            agent_locked: false,
            agent_update_on: false,
            agent_update: String::new(),
            agent_lock_check: true,
            agent_upgrade_pkg: String::new(),
            agent_list_kind: AgentListKindSel::All,
            agent_list: None,
            agent_list_stale: false,
            agent_last_edit: None,
            agent_last_report: None,
            agent_lock_drift: None,
            agent_last_doctor: None,
            agent_status: String::new(),
            secrets_verb: SecretsVerbTab::MintGithub,
            sec_install_id: String::new(),
            sec_repo_ids: String::new(),
            sec_perms: String::new(),
            sec_ttl_secs: "3600".into(),
            sec_relay_name: String::new(),
            sec_relay_ttl: String::new(),
            sec_relay_mode: "base-url".into(),
            sec_relay_provider: "generic".into(),
            sec_relay_repos: String::new(),
            sec_relay_perms: String::new(),
            sec_revoke_token: String::new(),
            sec_revoke_install_id: String::new(),
            sec_revoke_apply: false,
            sec_status: String::new(),
            sec_mint_expires: None,
            sec_mint_has_token: false,
            sec_mint_copy_once: None,
            sec_relay_result: None,
            sec_revoke_result: None,
        };
        let _ = app.cmd_tx.send(EngineCommand::Detect);
        let _ = app.cmd_tx.send(EngineCommand::SampleTelemetry);
        app
    }

    fn drain(&mut self) {
        while let Ok(ev) = self.evt_rx.try_recv() {
            match ev {
                Event::Report { report } => {
                    let detected = report.components.iter().filter(|c| c.detected).count();
                    self.header = format!(
                        "{} GPU(s) · driver {} · {}/{} present · {} drift",
                        report.gpu_count,
                        if report.driver_loaded {
                            "loaded"
                        } else {
                            "not loaded"
                        },
                        detected,
                        report.components.len(),
                        report.drift.len()
                    );
                    // read Copy fields BEFORE moving report.components (partial-move guard)
                    self.gpu_present = report.gpu_present;
                    self.driver_loaded = report.driver_loaded;
                    self.software_rendered = report.software_rendered;
                    self.gpu_count = report.gpu_count;
                    self.components = report.components;
                    self.drift = report.drift;
                }
                Event::Log {
                    component,
                    stream,
                    line,
                } => self.push_log(stream, format!("[{component}] {line}")),
                Event::Telemetry(t) => {
                    for g in &t.gpus {
                        let buf = self.util_history.entry(g.index).or_default();
                        if buf.len() >= SPARK_LEN {
                            buf.pop_front();
                        }
                        buf.push_back(g.util_pct as f32);
                    }
                    // audit fix (minor): drop history for GPU indices no longer present
                    // so stale sparklines don't linger or reappear.
                    let live: HashSet<u32> = t.gpus.iter().map(|g| g.index).collect();
                    self.util_history.retain(|k, _| live.contains(k));
                    self.telemetry = Some(t);
                }
                Event::GuardRefused { component, reason } => {
                    self.push_log(Stream::Stderr, format!("⛔ REFUSED {component}: {reason}"))
                }
                Event::StepFinished { result } => {
                    self.busy.remove(&result.component);
                    if result.status != OpStatus::NoHook {
                        let stream = if matches!(
                            result.status,
                            OpStatus::Failed | OpStatus::Refused | OpStatus::Incomplete
                        ) {
                            Stream::Stderr
                        } else {
                            Stream::Stdout
                        };
                        self.push_log(
                            stream,
                            format!(
                                "{} {:?} -> {:?}",
                                result.component, result.phase, result.status
                            ),
                        );
                    }
                }
                Event::RunFinished { .. } => {
                    let _ = self.cmd_tx.send(EngineCommand::Detect); // refresh after a run
                }
                Event::Dashboard { plan } => {
                    self.dash_status = format!("rendered {} tabs", plan.tabs.len());
                    self.dash_plan = Some(plan);
                }
                Event::DashboardDeployed { outcome } => {
                    self.dash_status = if outcome.applied {
                        format!("deployed -> {}", outcome.target.display())
                    } else {
                        format!("dry-run: would write {}", outcome.target.display())
                    };
                    for note in &outcome.notes {
                        self.push_log(Stream::Stdout, format!("[dashboard] {note}"));
                    }
                }
                Event::AgentRunFinished { report } => {
                    let s = &report.summary;
                    self.agent_status = format!(
                        "{} · installed {} · updated {} · removed {} · unchanged {} · failed {}",
                        if report.dry_run { "preview" } else { "applied" },
                        s.installed,
                        s.updated,
                        s.removed,
                        s.unchanged,
                        s.failed
                    );
                    self.agent_last_report = Some(report);
                    // sync/clean changed the lock or on-disk state; any cached list is stale.
                    self.agent_list_stale = true;
                }
                Event::AgentLockChecked { drift } => {
                    self.agent_status = if drift.is_empty() {
                        "lock is up to date".into()
                    } else {
                        format!("{} drift change(s)", drift.len())
                    };
                    self.agent_lock_drift = Some(drift);
                }
                Event::AgentListed { list } => {
                    self.agent_status = format!(
                        "{} skill(s) · {} mcp(s) · {} command(s)",
                        list.skills.len(),
                        list.mcps.len(),
                        list.commands.len()
                    );
                    self.agent_list = Some(list);
                    self.agent_list_stale = false;
                }
                Event::AgentEdited { outcome } => {
                    self.agent_status = format!("{} · {}", outcome.action, outcome.source);
                    self.agent_last_edit = Some(outcome);
                    // add/remove changed the config (and possibly synced) — cached list is stale.
                    self.agent_list_stale = true;
                }
                Event::AgentDoctored { report } => {
                    self.agent_status = format!(
                        "envctl {} · {} · {} skill(s) · {} mcp(s) · {} command(s) · {} failure(s)",
                        report.version,
                        report.scope,
                        report.skills.len(),
                        report.mcps.len(),
                        report.commands.len(),
                        report.failures.len(),
                    );
                    self.agent_last_doctor = Some(report);
                }
                Event::AgentAction {
                    source,
                    asset,
                    status,
                    error,
                } => {
                    let mut line = String::from("[agent]");
                    if let Some(s) = &source {
                        line.push_str(&format!(" {s}"));
                    }
                    if let Some(a) = &asset {
                        line.push_str(&format!("/{a}"));
                    }
                    line.push_str(&format!(" -> {status}"));
                    if let Some(e) = &error {
                        line.push_str(&format!(" ({e})"));
                    }
                    let stream = if error.is_some() {
                        Stream::Stderr
                    } else {
                        Stream::Stdout
                    };
                    self.push_log(stream, line);
                }
                Event::SecretsResult {
                    verb,
                    json_stdout,
                    stderr,
                    code,
                } => self.handle_secrets_result(&verb, &json_stdout, &stderr, code),
                _ => {}
            }
        }
    }

    /// Route a `secretctl` subprocess result (TASK-0028) into METADATA-ONLY GUI state. Secret
    /// hygiene is enforced HERE: mint stdout (the frozen `{token,expires_at_unix}`) never goes
    /// through `push_log` — the token is held transiently for a single copy-once affordance and
    /// only `expires_at_unix` + a `has_token` bool persist; the relay `bearer` is dropped (only
    /// `{token_id,expires_at,native}` kept). On a non-zero exit / not-found, only `stderr` (never
    /// secret) surfaces in a fail-closed DANGER status — success is never synthesized.
    fn handle_secrets_result(
        &mut self,
        verb: &str,
        json_stdout: &str,
        stderr: &str,
        code: Option<i32>,
    ) {
        // Fail-closed: anything but a clean exit ⇒ surface stderr (never secret) and stop. We do NOT
        // parse stdout on a failure, so no partial/half-success metadata is ever shown.
        if code != Some(0) {
            let why = if stderr.trim().is_empty() {
                match code {
                    Some(c) => format!("secretctl exited with code {c}"),
                    None => "secretctl did not run (not installed / failed to spawn)".to_string(),
                }
            } else {
                stderr.trim().to_string()
            };
            self.sec_status = format!("⛔ {verb} failed: {why}");
            // Surface the diagnostic stderr to the log (stderr is non-secret by the daemon contract).
            self.push_log(Stream::Stderr, format!("[secrets/{verb}] {why}"));
            return;
        }

        // The daemon's `--json` outputs are flat, compact objects. We extract ONLY the named
        // metadata fields with tiny pure-Rust scanners (NO serde_json dep — Architecture B keeps
        // the GUI dep set frozen). For mint-github, the `token` value is read out and moved into
        // the transient copy-once holder; it never flows through push_log.
        match verb {
            "mint-github" => {
                // Frozen contract `{token, expires_at_unix}`. Keep only the expiry + has_token; the
                // token is held transiently for the copy-once affordance, NEVER logged.
                let token = json_string_field(json_stdout, "token");
                let expires = json_number_field(json_stdout, "expires_at_unix");
                match (token, expires) {
                    (Some(tok), Some(exp)) => {
                        self.sec_mint_expires = Some(exp);
                        self.sec_mint_has_token = true;
                        self.sec_mint_copy_once = Some(tok);
                        self.sec_status =
                            format!("minted installation token (expires_at_unix {exp})");
                    }
                    _ => {
                        self.sec_status =
                            "⛔ mint-github: malformed JSON (missing token/expires_at_unix)".into();
                    }
                }
            }
            "relay-mint" => {
                // `{bearer, token_id, expires_at, native}`. DROP `bearer` (secret-class); keep only
                // the three non-secret metadata fields. `bearer` is NEVER extracted.
                let token_id = json_string_field(json_stdout, "token_id").unwrap_or_default();
                let expires_at = json_string_field(json_stdout, "expires_at").unwrap_or_default();
                let native = json_bool_field(json_stdout, "native").unwrap_or(false);
                self.sec_relay_result = Some(RelayMintMeta {
                    token_id: token_id.clone(),
                    expires_at,
                    native,
                });
                self.sec_status = format!("minted relay bearer (token {token_id})");
            }
            "revoke" => {
                // `{"revoked":bool,"dry_run":bool}`.
                let revoked = json_bool_field(json_stdout, "revoked").unwrap_or(false);
                let dry_run = json_bool_field(json_stdout, "dry_run").unwrap_or(true);
                self.sec_revoke_result = Some((revoked, dry_run));
                self.sec_status = if dry_run {
                    "dry-run: would revoke (no egress)".into()
                } else if revoked {
                    "revoked installation token".into()
                } else {
                    "apply: nothing was revoked".into()
                };
            }
            other => {
                self.sec_status = format!("unknown secrets verb: {other}");
            }
        }
    }

    fn push_log(&mut self, stream: Stream, text: String) {
        if self.log.len() >= self.log_cap {
            self.log.pop_front();
        }
        self.log.push_back(LogLine { stream, text });
    }

    fn dispatch(&mut self, cmd: EngineCommand, busy_id: Option<String>) {
        if let Some(id) = busy_id {
            self.busy.insert(id);
        }
        let _ = self.cmd_tx.send(cmd);
    }

    /// The worst drift severity recorded for a component, if any.
    fn drift_for<'a>(&'a self, id: &str) -> Option<&'a DriftItem> {
        self.drift
            .iter()
            .filter(|d| d.component == id)
            .min_by_key(|d| match d.severity {
                Severity::High => 0,
                Severity::Medium => 1,
                Severity::Low => 2,
            })
    }
}

impl eframe::App for EnvctlApp {
    fn update(&mut self, ctx: &egui::Context, _f: &mut eframe::Frame) {
        self.drain();

        egui::TopBottomPanel::top("nav")
            .frame(
                egui::Frame::none()
                    .fill(theme::PANEL)
                    .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                    .stroke(egui::Stroke::new(1.0, theme::BORDER)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("◆ envctl")
                            .size(18.0)
                            .strong()
                            .color(theme::ACCENT),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("GPU control center")
                            .size(11.0)
                            .color(theme::TEXT_FAINT),
                    );
                    ui.add_space(14.0);

                    for s in [
                        Screen::Dashboard,
                        Screen::Components,
                        Screen::Graph,
                        Screen::AddRepo,
                        Screen::Agent,
                        Screen::Secrets,
                        Screen::Mesh,
                        Screen::Logs,
                        Screen::Settings,
                    ] {
                        self.nav_tab(ui, s);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(&self.header)
                                .size(12.0)
                                .color(theme::TEXT_MUTED),
                        );
                    });
                });
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(theme::BG)
                    .inner_margin(egui::Margin::same(16.0)),
            )
            .show(ctx, |ui| match self.screen {
                Screen::Dashboard => self.dashboard(ui),
                Screen::Components => self.components_screen(ui),
                Screen::Graph => self.graph_screen(ui),
                Screen::AddRepo => self.add_repo_screen(ui),
                Screen::Agent => self.agent_screen(ui),
                Screen::Secrets => self.secrets_screen(ui),
                Screen::Mesh => self.mesh_screen(ui),
                Screen::Logs => self.logs_screen(ui),
                Screen::Settings => self.settings_screen(ui),
            });

        // The dedicated sampler thread emits Telemetry on its own cadence; the GUI
        // just sets the cadence (fast on Dashboard, slow elsewhere) + repaints.
        if self.screen == Screen::Dashboard {
            let cadence = if ctx.input(|i| i.focused) { 1000 } else { 3000 };
            self.tel.set_cadence(cadence);
            ctx.request_repaint_after(std::time::Duration::from_millis(cadence));
        } else {
            self.tel.set_cadence(10000);
        }
    }

    // Audit fix: on window close, tell the worker loop to shut down so it calls
    // ctrl.stop(); otherwise the telemetry sampler thread leaks and keeps
    // spawning nvidia-smi forever.
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let _ = self.cmd_tx.send(EngineCommand::Shutdown);
    }
}

impl EnvctlApp {
    fn nav_tab(&mut self, ui: &mut egui::Ui, s: Screen) {
        let active = self.screen == s;
        let text = if active {
            RichText::new(s.label()).color(theme::ACCENT_TEXT).strong()
        } else {
            RichText::new(s.label()).color(theme::TEXT_MUTED)
        };
        let btn = egui::Button::new(text)
            .fill(if active {
                theme::ACCENT
            } else {
                Color32::TRANSPARENT
            })
            .stroke(egui::Stroke::NONE)
            .rounding(egui::Rounding::same(7.0));
        if ui.add(btn).clicked() {
            self.screen = s;
        }
    }

    // ── Dashboard ───────────────────────────────────────────────────────────
    fn dashboard(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let t = self.telemetry.clone();

            // DriverNotActive: GPUs present but the kernel driver isn't loaded.
            if self.gpu_present && (!self.driver_loaded || self.software_rendered) {
                theme::card().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.colored_label(
                        theme::WARN,
                        format!("⟳  {} NVIDIA GPU(s) present but the driver is not loaded — install nvidia-open and REBOOT to light them up.", self.gpu_count),
                    );
                });
                ui.add_space(8.0);
            }

            ui.label(theme::section("SYSTEM"));
            ui.add_space(4.0);
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                match &t {
                    Some(t) => {
                        ui.horizontal(|ui| {
                            ui.allocate_ui_with_layout(
                                egui::vec2(ui.available_width() * 0.55, 0.0),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    if let (Some(used), Some(total)) =
                                        (t.mem_used_mb, t.mem_total_mb)
                                    {
                                        let frac = used as f32 / total.max(1) as f32;
                                        ui.label(
                                            RichText::new(format!(
                                                "Memory   {used} / {total} MiB"
                                            ))
                                            .color(theme::TEXT),
                                        );
                                        ui.add(
                                            egui::ProgressBar::new(frac)
                                                .fill(theme::load_color(frac))
                                                .desired_height(10.0)
                                                .rounding(egui::Rounding::same(5.0)),
                                        );
                                    } else {
                                        ui.colored_label(theme::TEXT_FAINT, "memory: n/a");
                                    }
                                },
                            );
                            ui.add_space(16.0);
                            ui.vertical(|ui| {
                                let la = t.load_avg.unwrap_or(0.0);
                                ui.label(RichText::new("Load avg (1m)").color(theme::TEXT_MUTED));
                                ui.label(
                                    RichText::new(format!("{la:.2}"))
                                        .size(20.0)
                                        .strong()
                                        .color(theme::TEXT),
                                );
                            });
                        });
                    }
                    None => {
                        ui.horizontal(|ui| {
                            ui.add(egui::Spinner::new().color(theme::ACCENT));
                            ui.colored_label(theme::TEXT_MUTED, "sampling system telemetry…");
                        });
                    }
                }
            });

            ui.add_space(12.0);
            ui.label(theme::section("GPUs"));
            ui.add_space(4.0);

            match &t {
                Some(t) if !t.gpus.is_empty() => {
                    for g in &t.gpus {
                        self.gpu_card(ui, g);
                    }
                }
                Some(_) => {
                    theme::card().show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.colored_label(theme::WARN, "⚠");
                            ui.colored_label(
                                theme::TEXT_MUTED,
                                "No live GPU telemetry — driver inactive. Install/REBOOT nvidia-open.",
                            );
                        });
                    });
                }
                None => {
                    theme::card().show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.add(egui::Spinner::new().color(theme::ACCENT));
                            ui.colored_label(theme::TEXT_MUTED, "sampling GPUs…");
                        });
                    });
                }
            }
        });
    }

    fn gpu_card(&self, ui: &mut egui::Ui, g: &envctl_engine::GpuSample) {
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("GPU {}", g.index))
                        .strong()
                        .color(theme::ACCENT),
                );
                ui.label(RichText::new(&g.name).color(theme::TEXT).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // sparkline of recent utilization
                    if let Some(hist) = self.util_history.get(&g.index) {
                        self.sparkline(ui, hist);
                    }
                });
            });
            ui.add_space(8.0);

            // Utilization
            let util = g.util_pct as f32 / 100.0;
            ui.horizontal(|ui| {
                ui.label(RichText::new("Util").color(theme::TEXT_MUTED).size(12.0));
                ui.add(
                    egui::ProgressBar::new(util)
                        .text(RichText::new(format!("{}%", g.util_pct)).color(theme::TEXT))
                        .fill(theme::load_color(util))
                        .desired_height(14.0)
                        .rounding(egui::Rounding::same(6.0)),
                );
            });
            ui.add_space(4.0);

            // VRAM
            let vram = g.mem_used_mb as f32 / g.mem_total_mb.max(1) as f32;
            ui.horizontal(|ui| {
                ui.label(RichText::new("VRAM").color(theme::TEXT_MUTED).size(12.0));
                ui.add(
                    egui::ProgressBar::new(vram)
                        .text(
                            RichText::new(format!("{} / {} MiB", g.mem_used_mb, g.mem_total_mb))
                                .color(theme::TEXT),
                        )
                        .fill(theme::load_color(vram))
                        .desired_height(14.0)
                        .rounding(egui::Rounding::same(6.0)),
                );
            });
            ui.add_space(8.0);

            // temp + power stat chips
            ui.horizontal(|ui| {
                let temp_col = theme::load_color(g.temp_c as f32 / 90.0);
                stat_chip(ui, "TEMP", &format!("{}°C", g.temp_c), temp_col);
                match g.power_w {
                    Some(p) => stat_chip(ui, "POWER", &format!("{p} W"), theme::INFO),
                    None => stat_chip(ui, "POWER", "n/a", theme::TEXT_FAINT),
                }
            });
        });
    }

    /// Paint a small utilization sparkline from a 0..=100 history buffer.
    fn sparkline(&self, ui: &mut egui::Ui, hist: &VecDeque<f32>) {
        let (rect, _resp) = ui.allocate_exact_size(egui::vec2(120.0, 28.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, egui::Rounding::same(5.0), theme::BG);

        if hist.len() < 2 {
            return;
        }
        let n = hist.len();
        let pad = 3.0;
        let w = rect.width() - pad * 2.0;
        let h = rect.height() - pad * 2.0;
        let pts: Vec<egui::Pos2> = hist
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let x = rect.left() + pad + (i as f32 / (n - 1) as f32) * w;
                let y = rect.top() + pad + (1.0 - (v / 100.0).clamp(0.0, 1.0)) * h;
                egui::pos2(x, y)
            })
            .collect();
        let last = hist.back().copied().unwrap_or(0.0) / 100.0;
        let col = theme::load_color(last);
        painter.add(egui::Shape::line(pts.clone(), egui::Stroke::new(1.6, col)));
        if let Some(p) = pts.last() {
            painter.circle_filled(*p, 2.2, col);
        }
    }

    // ── Components ────────────────────────────────────────────────────────────
    fn components_screen(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Components").heading());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let any_missing = self.components.iter().any(|c| !c.detected);
                let install =
                    egui::Button::new(RichText::new("Install all missing").color(if any_missing {
                        theme::ACCENT_TEXT
                    } else {
                        theme::TEXT_FAINT
                    }))
                    .fill(if any_missing {
                        theme::ACCENT
                    } else {
                        theme::SURFACE
                    });
                if ui.add_enabled(any_missing, install).clicked() {
                    let missing: Vec<String> = self
                        .components
                        .iter()
                        .filter(|c| !c.detected)
                        .map(|c| c.id.clone())
                        .collect();
                    for id in &missing {
                        self.busy.insert(id.clone());
                    }
                    self.dispatch(
                        EngineCommand::Install {
                            targets: missing,
                            dry_run: false,
                        },
                        None,
                    );
                }
            });
        });

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("🔍").color(theme::TEXT_MUTED));
            ui.add(
                egui::TextEdit::singleline(&mut self.filter)
                    .hint_text("filter components…")
                    .desired_width(260.0),
            );
            if !self.filter.is_empty() && ui.button("✕").clicked() {
                self.filter.clear();
            }
        });
        ui.add_space(8.0);

        let needle = self.filter.trim().to_lowercase();
        let rows: Vec<ComponentState> = self
            .components
            .iter()
            .filter(|c| {
                needle.is_empty()
                    || c.id.to_lowercase().contains(&needle)
                    || c.name.to_lowercase().contains(&needle)
            })
            .cloned()
            .collect();

        if rows.is_empty() {
            ui.colored_label(theme::TEXT_FAINT, "no matching components");
            return;
        }

        // Snapshot the per-row presentation so we don't borrow self in the table closure.
        struct RowView {
            id: String,
            name: String,
            detected: bool,
            busy: bool,
            dot: Color32,
            status_text: String,
            health: String,
            health_col: Color32,
        }
        let views: Vec<RowView> = rows
            .iter()
            .map(|c| {
                let d = self.drift_for(&c.id);
                let (dot, status_text) = pill_for(c, d);
                let (health_col, health) = health_label(c, d);
                RowView {
                    id: c.id.clone(),
                    name: c.name.clone(),
                    detected: c.detected,
                    busy: self.busy.contains(&c.id),
                    dot,
                    status_text,
                    health,
                    health_col,
                }
            })
            .collect();

        let mut to_install: Option<String> = None;
        let mut to_fix: Option<String> = None;

        TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::exact(120.0)) // status pill
            .column(Column::exact(150.0)) // id
            .column(Column::remainder().at_least(140.0)) // name
            .column(Column::exact(180.0)) // health/drift
            .column(Column::exact(150.0)) // actions
            .header(24.0, |mut h| {
                for title in ["STATUS", "ID", "NAME", "HEALTH", ""] {
                    h.col(|ui| {
                        ui.label(
                            RichText::new(title)
                                .size(11.0)
                                .strong()
                                .color(theme::TEXT_MUTED),
                        );
                    });
                }
            })
            .body(|mut body| {
                for v in &views {
                    body.row(34.0, |mut row| {
                        row.col(|ui| {
                            ui.label(RichText::new("●").color(v.dot));
                            ui.label(RichText::new(&v.status_text).size(12.0).color(v.dot));
                        });
                        row.col(|ui| {
                            ui.label(RichText::new(&v.id).monospace().color(theme::TEXT));
                        });
                        row.col(|ui| {
                            ui.label(RichText::new(&v.name).color(theme::TEXT_MUTED));
                        });
                        row.col(|ui| {
                            ui.label(RichText::new(&v.health).size(12.0).color(v.health_col));
                        });
                        row.col(|ui| {
                            if v.busy {
                                ui.add(egui::Spinner::new().color(theme::ACCENT));
                                ui.colored_label(theme::TEXT_FAINT, "working…");
                            } else if !v.detected {
                                let b = egui::Button::new(
                                    RichText::new("Install").color(theme::ACCENT_TEXT),
                                )
                                .fill(theme::ACCENT);
                                if ui.add(b).clicked() {
                                    to_install = Some(v.id.clone());
                                }
                            } else {
                                if ui.button("Fix").clicked() {
                                    to_fix = Some(v.id.clone());
                                }
                            }
                        });
                    });
                }
            });

        if let Some(id) = to_install {
            self.dispatch(
                EngineCommand::Install {
                    targets: vec![id.clone()],
                    dry_run: false,
                },
                Some(id),
            );
        }
        if let Some(id) = to_fix {
            // Audit fix: dry_run_default==true means "dry-run by default", so the
            // checked box must map directly to dry_run (was inverted, running Fix
            // for real by default and defeating the only GUI safety guard).
            self.dispatch(
                EngineCommand::Fix {
                    targets: vec![id.clone()],
                    dry_run: self.dry_run_default,
                },
                Some(id),
            );
        }
    }

    // ── Graph ─────────────────────────────────────────────────────────────────
    fn graph_screen(&mut self, ui: &mut egui::Ui) {
        use envctl_engine::graph;
        // Gather everything OWNED up front so the combo can borrow &mut self.graph_focus
        // without aliasing the registry borrow (immediate-mode: 1-frame lag is fine).
        let g = graph::analyze(self.geng.registry());
        let ids: Vec<String> = self.geng.registry().ids().cloned().collect();
        let focus = self.graph_focus.clone();
        let im = if focus.is_empty() {
            None
        } else {
            graph::impact(self.geng.registry(), &focus)
        };
        let paths = if focus.is_empty() {
            Vec::new()
        } else {
            graph::dependency_paths(self.geng.registry(), &focus)
        };

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label(theme::section("DEPENDENCY GRAPH"));
            ui.add_space(4.0);
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(format!(
                    "{} components · {} edges · {} groups",
                    g.nodes,
                    g.edges,
                    g.groups.len()
                ));
                ui.label(format!(
                    "{} roots · {} leaves · {} orphans",
                    g.roots.len(),
                    g.leaves.len(),
                    g.orphans.len()
                ));
                if let Some((id, n)) = &g.max_dependents {
                    ui.colored_label(theme::TEXT_MUTED, format!("most depended-on: {id} ({n})"));
                }
                ui.add_space(6.0);
                ui.label(RichText::new("critical path").color(theme::ACCENT_TEXT));
                ui.monospace(g.critical_path.join("  →  "));
            });

            ui.add_space(10.0);
            ui.label(theme::section("IMPACT — pick a component"));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let sel = if focus.is_empty() {
                    "(select)".to_string()
                } else {
                    focus.clone()
                };
                egui::ComboBox::from_id_salt("graph_focus")
                    .selected_text(sel)
                    .width(280.0)
                    .show_ui(ui, |ui| {
                        for id in &ids {
                            ui.selectable_value(&mut self.graph_focus, id.clone(), id.as_str());
                        }
                    });
            });

            if let Some(im) = &im {
                ui.add_space(6.0);
                theme::card().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        RichText::new(format!("install {}", im.component)).color(theme::HEALTHY),
                    );
                    ui.monospace(format!(
                        "pulls in ({}): {}",
                        im.install_closure.len(),
                        im.install_closure.join("  →  ")
                    ));
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(format!("reset {} --cascade", im.component))
                            .color(theme::WARN),
                    );
                    ui.monospace(format!(
                        "also removes ({}): {}",
                        im.cascade_removes.len(),
                        if im.cascade_removes.is_empty() {
                            "(none)".into()
                        } else {
                            im.cascade_removes.join(", ")
                        }
                    ));
                    ui.add_space(6.0);
                    ui.label(RichText::new("why it's needed (root → it)").color(theme::TEXT_MUTED));
                    for p in &paths {
                        ui.monospace(format!("  {}", p.join("  →  ")));
                    }
                });
            }
        });
    }

    // ── Add Repo ──────────────────────────────────────────────────────────────
    fn add_repo_screen(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Add a repo as a managed component").heading());
        ui.add_space(10.0);

        theme::inset().show(ui, |ui| {
            ui.set_max_width(620.0);
            egui::Grid::new("addrepo")
                .num_columns(2)
                .spacing([14.0, 12.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Git URL").color(theme::TEXT_MUTED));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.add_url)
                            .hint_text("https://github.com/owner/repo")
                            .desired_width(380.0),
                    );
                    ui.end_row();

                    ui.label(RichText::new("ID").color(theme::TEXT_MUTED));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.add_id)
                            .hint_text("short-unique-id")
                            .desired_width(380.0),
                    );
                    ui.end_row();

                    ui.label(RichText::new("Ref").color(theme::TEXT_MUTED));
                    ui.add(egui::TextEdit::singleline(&mut self.add_ref).hint_text("branch/tag/sha (optional)").desired_width(380.0));
                    ui.end_row();

                    ui.label(RichText::new("Build cmd").color(theme::TEXT_MUTED));
                    ui.add(egui::TextEdit::singleline(&mut self.add_build).hint_text("(blank = auto-detect)").desired_width(380.0));
                    ui.end_row();

                    ui.label(RichText::new("Strategy").color(theme::TEXT_MUTED));
                    egui::ComboBox::from_id_salt("strategy")
                        .selected_text(&self.add_strategy)
                        .show_ui(ui, |ui| {
                            for s in ["as-is", "cherry-pick", "rename", "refactor"] {
                                ui.selectable_value(&mut self.add_strategy, s.to_string(), s);
                            }
                        });
                    ui.end_row();
                });

            // strategy-specific fields
            ui.add_space(8.0);
            match self.add_strategy.as_str() {
                "cherry-pick" => {
                    ui.label(RichText::new("Bins (comma-separated file-stems)").color(theme::TEXT_MUTED));
                    ui.add(egui::TextEdit::singleline(&mut self.add_bins).hint_text("rg, foo").desired_width(420.0));
                }
                "rename" => {
                    ui.label(RichText::new("Renames (old=new, comma-separated)").color(theme::TEXT_MUTED));
                    ui.add(egui::TextEdit::singleline(&mut self.add_renames).hint_text("rg=rgx").desired_width(420.0));
                }
                "refactor" => {
                    ui.label(RichText::new("Patch cmd (leave blank for AI refactor)").color(theme::TEXT_MUTED));
                    ui.add(egui::TextEdit::singleline(&mut self.add_patch).desired_width(420.0));
                    if self.add_patch.trim().is_empty() {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("AI goal").color(theme::TEXT_MUTED));
                            egui::ComboBox::from_id_salt("ai_goal")
                                .selected_text(&self.add_ai_goal)
                                .show_ui(ui, |ui| {
                                    for g in ["port-to-rust", "cherry-pick-to-crate", "rename-for-synergy", "custom"] {
                                        ui.selectable_value(&mut self.add_ai_goal, g.to_string(), g);
                                    }
                                });
                        });
                        ui.add(egui::TextEdit::singleline(&mut self.add_ai_instruction).hint_text("extra instruction (optional)").desired_width(420.0));
                        ui.colored_label(theme::WARN, "envctl invokes the agent NON-INTERACTIVELY in the clone; it never auto-commits or pushes.");
                    }
                }
                _ => {}
            }

            ui.add_space(10.0);
            ui.checkbox(&mut self.add_build_flag, "Build now (run the upstream build / AI agent + install) — off = preview only");

            ui.add_space(12.0);
            let ready = !self.add_url.trim().is_empty() && !self.add_id.trim().is_empty();
            ui.horizontal(|ui| {
                if ui.add_enabled(ready, egui::Button::new("Validate (dry-run)")).clicked() {
                    self.dispatch(self.add_repo_cmd(true), None);
                    self.screen = Screen::Logs;
                }
                let label = if self.add_build_flag { "Build + Register" } else { "Register (preview)" };
                let reg = egui::Button::new(RichText::new(label).color(if ready { theme::ACCENT_TEXT } else { theme::TEXT_FAINT }))
                    .fill(if ready { theme::ACCENT } else { theme::SURFACE });
                if ui.add_enabled(ready, reg).clicked() {
                    self.dispatch(self.add_repo_cmd(false), None);
                    self.screen = Screen::Logs;
                }
            });
        });

        ui.add_space(10.0);
        ui.colored_label(
            theme::TEXT_FAINT,
            "Acquire + detect + preview by default. 'Build now' clones, builds from source, installs into $META_ROOT/usr/bin, and registers a managed drop-in.",
        );
    }

    fn add_repo_cmd(&self, dry_run: bool) -> EngineCommand {
        let opt = |s: &str| {
            let t = s.trim();
            if t.is_empty() {
                None
            } else {
                Some(t.to_string())
            }
        };
        let strategy = match self.add_strategy.as_str() {
            "cherry-pick" => BuildStrategy::CherryPick {
                bins: split_csv(&self.add_bins),
            },
            "rename" => BuildStrategy::Rename {
                renames: split_csv(&self.add_renames)
                    .into_iter()
                    .filter_map(|p| {
                        p.split_once('=').map(|(a, b)| RenameRule {
                            from: a.trim().into(),
                            to: b.trim().into(),
                        })
                    })
                    .collect(),
            },
            "refactor" => BuildStrategy::Refactor {
                refactor: if let Some(cmd) = opt(&self.add_patch) {
                    Refactor::Patch { command: cmd }
                } else {
                    Refactor::Ai {
                        agent: None,
                        goal: match self.add_ai_goal.as_str() {
                            "port-to-rust" => RefactorGoal::PortToRust,
                            "cherry-pick-to-crate" => RefactorGoal::CherryPickToCrate,
                            "rename-for-synergy" => RefactorGoal::RenameForSynergy,
                            _ => RefactorGoal::Custom,
                        },
                        instruction: opt(&self.add_ai_instruction),
                    }
                },
            },
            _ => BuildStrategy::AsIs,
        };
        EngineCommand::AddRepo {
            spec: AddRepoSpec {
                id: self.add_id.trim().to_string(),
                git_url: self.add_url.trim().to_string(),
                git_ref: opt(&self.add_ref),
                build_cmd: self.add_build.trim().to_string(),
                strategy,
                allow_build: self.add_build_flag,
                ..Default::default()
            },
            dry_run,
        }
    }

    // ── Agent (agent-env) ───────────────────────────────────────────────────────
    // PURE state→Spec builders: NO egui types, unit-testable. Each mirrors the CLI
    // `run_agent` arm field-for-field (cli/src/main.rs:943-1060). Blank string → None,
    // CSV → split_csv → Vec, apply default false (fail-closed).

    /// The shared `--locked` / `--update [names]` → `AgentLockMode` mapping, derived from the
    /// form toggles exactly as the CLI passes `AgentLockMode::from_flags(locked, update)`.
    fn agent_lock_mode(&self) -> AgentLockMode {
        let update = if self.agent_update_on {
            Some(split_csv(&self.agent_update))
        } else {
            None
        };
        AgentLockMode::from_flags(self.agent_locked, update)
    }

    fn agent_sync_spec(&self) -> AgentSyncSpec {
        AgentSyncSpec {
            config_path: opt_str(&self.agent_config),
            scope_override: self.agent_scope.to_override(),
            apply: self.agent_apply,
            lock_mode: self.agent_lock_mode(),
        }
    }

    fn agent_section(&self) -> AgentSectionSel {
        AgentSectionSel {
            skills: split_csv(&self.agent_skills),
            mcps: split_csv(&self.agent_mcps),
            commands: split_csv(&self.agent_commands),
        }
    }

    fn agent_add_spec(&self) -> AgentAddSpec {
        AgentAddSpec {
            source: self.agent_source.trim().to_string(),
            section: self.agent_section(),
            git_ref: opt_str(&self.agent_git_ref),
            branch: opt_str(&self.agent_branch),
            sub_dir: opt_str(&self.agent_sub_dir),
            config_path: opt_str(&self.agent_config),
            scope_override: self.agent_scope.to_override(),
            apply: self.agent_apply,
            no_sync: self.agent_no_sync,
            no_verify: self.agent_no_verify,
            lock_mode: self.agent_lock_mode(),
        }
    }

    fn agent_remove_spec(&self) -> AgentRemoveSpec {
        AgentRemoveSpec {
            source: self.agent_source.trim().to_string(),
            section: self.agent_section(),
            git_ref: opt_str(&self.agent_git_ref),
            branch: opt_str(&self.agent_branch),
            sub_dir: opt_str(&self.agent_sub_dir),
            config_path: opt_str(&self.agent_config),
            scope_override: self.agent_scope.to_override(),
            apply: self.agent_apply,
            no_sync: self.agent_no_sync,
            lock_mode: self.agent_lock_mode(),
        }
    }

    fn agent_lock_spec(&self) -> AgentLockSpec {
        AgentLockSpec {
            config_path: opt_str(&self.agent_config),
            scope_override: self.agent_scope.to_override(),
            check: self.agent_lock_check,
            upgrade_only: split_csv(&self.agent_upgrade_pkg),
            // CLI passes `from_flags(locked, None)` for lock (no --update on lock).
            lock_mode: AgentLockMode::from_flags(self.agent_locked, None),
        }
    }

    fn agent_list_spec(&self) -> AgentListSpec {
        AgentListSpec {
            scope_override: self.agent_scope.to_override(),
            kind: self.agent_list_kind.to_kind(),
        }
    }

    fn agent_clean_spec(&self) -> AgentCleanSpec {
        AgentCleanSpec {
            config_path: opt_str(&self.agent_config),
            scope_override: self.agent_scope.to_override(),
            apply: self.agent_apply,
        }
    }

    fn agent_doctor_spec(&self) -> AgentDoctorSpec {
        AgentDoctorSpec {
            scope_override: self.agent_scope.to_override(),
        }
    }

    /// Wrap the active verb's spec in `EngineCommand::Agent { spec }` — the single command the
    /// worker dispatches to the matching `Engine::agent_*` method.
    fn agent_command(&self) -> EngineCommand {
        let spec = match self.agent_verb {
            AgentVerbTab::Sync => AgentCommandSpec::Sync(self.agent_sync_spec()),
            AgentVerbTab::Add => AgentCommandSpec::Add(self.agent_add_spec()),
            AgentVerbTab::Remove => AgentCommandSpec::Remove(self.agent_remove_spec()),
            AgentVerbTab::Lock => AgentCommandSpec::Lock(self.agent_lock_spec()),
            AgentVerbTab::List => AgentCommandSpec::List(self.agent_list_spec()),
            AgentVerbTab::Clean => AgentCommandSpec::Clean(self.agent_clean_spec()),
            AgentVerbTab::Doctor => AgentCommandSpec::Doctor(self.agent_doctor_spec()),
        };
        EngineCommand::Agent { spec }
    }

    fn agent_screen(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Agent-env — manage skills / MCPs / commands").heading());
        ui.add_space(8.0);

        // verb sub-tabs
        ui.horizontal(|ui| {
            for v in [
                AgentVerbTab::Sync,
                AgentVerbTab::Add,
                AgentVerbTab::Remove,
                AgentVerbTab::Lock,
                AgentVerbTab::List,
                AgentVerbTab::Clean,
                AgentVerbTab::Doctor,
            ] {
                let active = self.agent_verb == v;
                let text = if active {
                    RichText::new(v.label()).color(theme::ACCENT_TEXT).strong()
                } else {
                    RichText::new(v.label()).color(theme::TEXT_MUTED)
                };
                let btn = egui::Button::new(text)
                    .fill(if active {
                        theme::ACCENT
                    } else {
                        Color32::TRANSPARENT
                    })
                    .rounding(egui::Rounding::same(7.0));
                if ui.add(btn).clicked() {
                    self.agent_verb = v;
                }
            }
        });
        ui.add_space(10.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            theme::inset().show(ui, |ui| {
                ui.set_max_width(640.0);
                self.agent_scope_row(ui);

                match self.agent_verb {
                    AgentVerbTab::Sync => {
                        self.agent_config_row(ui);
                        self.agent_lock_mode_row(ui);
                        ui.checkbox(&mut self.agent_apply, "Apply (write) — off = preview");
                    }
                    AgentVerbTab::Add => {
                        self.agent_source_rows(ui);
                        self.agent_section_rows(ui);
                        self.agent_config_row(ui);
                        self.agent_lock_mode_row(ui);
                        ui.checkbox(&mut self.agent_no_sync, "No sync after edit (--no-sync)");
                        ui.checkbox(
                            &mut self.agent_no_verify,
                            "No verify on add (--no-verify)",
                        );
                        ui.checkbox(&mut self.agent_apply, "Apply (write) — off = preview");
                    }
                    AgentVerbTab::Remove => {
                        self.agent_source_rows(ui);
                        self.agent_section_rows(ui);
                        self.agent_config_row(ui);
                        self.agent_lock_mode_row(ui);
                        ui.checkbox(&mut self.agent_no_sync, "No sync after edit (--no-sync)");
                        ui.checkbox(&mut self.agent_apply, "Apply (write) — off = preview");
                    }
                    AgentVerbTab::Lock => {
                        self.agent_config_row(ui);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Upgrade packages (CSV)").color(theme::TEXT_MUTED));
                            ui.add(
                                egui::TextEdit::singleline(&mut self.agent_upgrade_pkg)
                                    .hint_text("name,name (optional)")
                                    .desired_width(360.0),
                            );
                        });
                        ui.checkbox(&mut self.agent_locked, "Zero-network audit (--locked)");
                        ui.checkbox(
                            &mut self.agent_lock_check,
                            "Check only (--check) — off = rewrite the lock",
                        );
                    }
                    AgentVerbTab::List => {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Kind").color(theme::TEXT_MUTED));
                            egui::ComboBox::from_id_salt("agent_list_kind")
                                .selected_text(self.agent_list_kind.label())
                                .show_ui(ui, |ui| {
                                    for k in [
                                        AgentListKindSel::All,
                                        AgentListKindSel::Skills,
                                        AgentListKindSel::Mcps,
                                        AgentListKindSel::Commands,
                                    ] {
                                        ui.selectable_value(
                                            &mut self.agent_list_kind,
                                            k,
                                            k.label(),
                                        );
                                    }
                                });
                        });
                    }
                    AgentVerbTab::Clean => {
                        ui.colored_label(
                            theme::WARN,
                            "⚠ Clean removes managed assets not present in the config (the lock's orphans). Preview first; Apply writes.",
                        );
                        ui.checkbox(&mut self.agent_apply, "Apply (write) — off = preview");
                    }
                    AgentVerbTab::Doctor => {
                        ui.colored_label(
                            theme::TEXT_MUTED,
                            "Read-only diagnostics: version, lock, scope, inventory, command-dir writability, and the update check.",
                        );
                    }
                }

                ui.add_space(12.0);
                self.agent_action_button(ui);
            });

            ui.add_space(10.0);
            if !self.agent_status.is_empty() {
                ui.colored_label(theme::TEXT_MUTED, &self.agent_status);
                ui.add_space(8.0);
            }
            self.agent_results(ui);
        });
    }

    fn agent_scope_row(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Scope").color(theme::TEXT_MUTED));
            egui::ComboBox::from_id_salt("agent_scope")
                .selected_text(self.agent_scope.label())
                .show_ui(ui, |ui| {
                    for s in [
                        AgentScopeSel::Default,
                        AgentScopeSel::Global,
                        AgentScopeSel::Project,
                    ] {
                        ui.selectable_value(&mut self.agent_scope, s, s.label());
                    }
                });
        });
    }

    fn agent_config_row(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Config path").color(theme::TEXT_MUTED));
            ui.add(
                egui::TextEdit::singleline(&mut self.agent_config)
                    .hint_text("(blank = default resolution)")
                    .desired_width(360.0),
            );
        });
    }

    fn agent_source_rows(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Source").color(theme::TEXT_MUTED));
            ui.add(
                egui::TextEdit::singleline(&mut self.agent_source)
                    .hint_text("repo url / path / id")
                    .desired_width(360.0),
            );
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("Ref").color(theme::TEXT_MUTED));
            ui.add(
                egui::TextEdit::singleline(&mut self.agent_git_ref)
                    .hint_text("ref (optional)")
                    .desired_width(150.0),
            );
            ui.label(RichText::new("Branch").color(theme::TEXT_MUTED));
            ui.add(
                egui::TextEdit::singleline(&mut self.agent_branch)
                    .hint_text("branch (optional)")
                    .desired_width(150.0),
            );
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("Sub-dir").color(theme::TEXT_MUTED));
            ui.add(
                egui::TextEdit::singleline(&mut self.agent_sub_dir)
                    .hint_text("sub-dir (optional)")
                    .desired_width(360.0),
            );
        });
    }

    fn agent_section_rows(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Skills (CSV)").color(theme::TEXT_MUTED));
            ui.add(egui::TextEdit::singleline(&mut self.agent_skills).desired_width(360.0));
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("MCPs (CSV)").color(theme::TEXT_MUTED));
            ui.add(egui::TextEdit::singleline(&mut self.agent_mcps).desired_width(360.0));
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("Commands (CSV)").color(theme::TEXT_MUTED));
            ui.add(egui::TextEdit::singleline(&mut self.agent_commands).desired_width(360.0));
        });
    }

    fn agent_lock_mode_row(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.agent_locked, "Locked (--locked, zero-network)");
            ui.checkbox(&mut self.agent_update_on, "Update (--update)");
            if self.agent_update_on {
                ui.add(
                    egui::TextEdit::singleline(&mut self.agent_update)
                        .hint_text("names CSV (blank = all)")
                        .desired_width(220.0),
                );
            }
        });
    }

    /// The single action button. For the mutating verbs it carries the apply/preview state in
    /// the Spec (`agent_apply`), so a fresh, fail-closed default (apply=false) previews. Always
    /// dispatches to the worker (`dispatch` → mpsc send) so the UI thread never blocks.
    fn agent_action_button(&mut self, ui: &mut egui::Ui) {
        let lock_rewrites = matches!(self.agent_verb, AgentVerbTab::Lock) && !self.agent_lock_check;
        let mutating = matches!(
            self.agent_verb,
            AgentVerbTab::Sync | AgentVerbTab::Add | AgentVerbTab::Remove | AgentVerbTab::Clean
        ) || lock_rewrites;
        let label = if matches!(self.agent_verb, AgentVerbTab::Lock) {
            if self.agent_lock_check {
                format!("Check {}", self.agent_verb.label())
            } else {
                format!("Rewrite {}", self.agent_verb.label())
            }
        } else if mutating {
            if self.agent_apply {
                format!("Apply {}", self.agent_verb.label())
            } else {
                format!("Preview {}", self.agent_verb.label())
            }
        } else {
            format!("Run {}", self.agent_verb.label())
        };
        let fill = if mutating && (self.agent_apply || lock_rewrites) {
            theme::WARN
        } else {
            theme::ACCENT
        };
        let btn = egui::Button::new(RichText::new(label).color(theme::ACCENT_TEXT)).fill(fill);
        if ui.add(btn).clicked() {
            self.dispatch(self.agent_command(), Some("agent".into()));
        }
    }

    fn agent_results(&mut self, ui: &mut egui::Ui) {
        if self.agent_list_stale {
            ui.colored_label(theme::WARN, "⚠ agent list is stale — run List to refresh");
            ui.add_space(4.0);
        }
        if let Some(list) = self.agent_list.clone() {
            self.agent_list_tables(ui, &list);
        }
        if let Some(edit) = self.agent_last_edit.clone() {
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new(format!("edit: {} · {}", edit.action, edit.source)).strong(),
                );
                for it in &edit.items {
                    ui.colored_label(
                        theme::TEXT_MUTED,
                        format!("  {} / {}", it.section, it.target),
                    );
                }
            });
            ui.add_space(8.0);
        }
        if let Some(report) = self.agent_last_report.clone() {
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                let s = &report.summary;
                ui.label(RichText::new("run summary").strong());
                ui.colored_label(
                    theme::TEXT_MUTED,
                    format!(
                        "installed {} · updated {} · removed {} · unchanged {} · failed {}",
                        s.installed, s.updated, s.removed, s.unchanged, s.failed
                    ),
                );
            });
            ui.add_space(8.0);
        }
        if let Some(drift) = self.agent_lock_drift.clone() {
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new("lock drift").strong());
                if drift.is_empty() {
                    ui.colored_label(theme::TEXT_FAINT, "  no drift");
                }
                for d in &drift {
                    ui.colored_label(theme::TEXT_MUTED, format!("  {} {}", d.status, d.id));
                }
            });
        }
        if let Some(doctor) = self.agent_last_doctor.clone() {
            self.agent_doctor_tables(ui, &doctor);
        }
    }

    /// Render the `agent doctor` report (parity with the CLI's grouped view): Environment,
    /// Inventory, Checks, Command directories, Failures. Driven by the identical
    /// `Engine::agent_doctor` the CLI calls (the report arrives via `Event::AgentDoctored`).
    fn agent_doctor_tables(&self, ui: &mut egui::Ui, d: &AgentDoctorReport) {
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new(format!(
                    "doctor — envctl {} ({})",
                    d.version,
                    if d.failures.is_empty() {
                        "✓ healthy"
                    } else {
                        "✗ issues"
                    }
                ))
                .strong(),
            );

            ui.add_space(6.0);
            ui.label(
                RichText::new("Environment")
                    .color(theme::ACCENT_TEXT)
                    .strong(),
            );
            let update_text = match d.update_check.status.as_str() {
                "update_available" => format!(
                    "{} available",
                    d.update_check.latest_version.as_deref().unwrap_or("?")
                ),
                "up_to_date" => "up-to-date".to_string(),
                _ => "not yet checked".to_string(),
            };
            for (k, v) in [
                ("Scope", d.scope.clone()),
                ("Lock file", d.lock_file.clone()),
                ("Install path", d.installation_path.clone()),
                (
                    "Last sync",
                    d.last_sync.clone().unwrap_or_else(|| "none".into()),
                ),
                ("Updates", update_text),
            ] {
                ui.colored_label(theme::TEXT_MUTED, format!("  {k}: {v}"));
            }

            ui.add_space(6.0);
            ui.label(
                RichText::new("Inventory")
                    .color(theme::ACCENT_TEXT)
                    .strong(),
            );
            ui.colored_label(
                theme::TEXT_MUTED,
                format!(
                    "  Skills {} · MCP servers {} · Commands {}",
                    d.skills.len(),
                    d.mcps.len(),
                    d.commands.len()
                ),
            );

            ui.add_space(6.0);
            ui.label(RichText::new("Checks").color(theme::ACCENT_TEXT).strong());
            let dirs_writable = d.command_dirs.iter().filter(|c| c.writable).count();
            let dirs_total = d.command_dirs.len();
            let check = |ui: &mut egui::Ui, ok: bool, label: &str| {
                let (glyph, color) = if ok {
                    ("✓", theme::TEXT_MUTED)
                } else {
                    ("✗", theme::WARN)
                };
                ui.colored_label(color, format!("  {glyph} {label}"));
            };
            check(ui, !d.lock_file.is_empty(), "Lock file readable");
            check(ui, d.failures.is_empty(), "No failed skills");
            check(
                ui,
                dirs_writable == dirs_total,
                &format!("{dirs_writable} of {dirs_total} command directories writable"),
            );

            if !d.command_dirs.is_empty() {
                ui.add_space(6.0);
                ui.label(
                    RichText::new("Command directories")
                        .color(theme::ACCENT_TEXT)
                        .strong(),
                );
                for c in &d.command_dirs {
                    let (glyph, color) = if c.writable {
                        ("✓", theme::TEXT_MUTED)
                    } else {
                        ("✗", theme::WARN)
                    };
                    ui.colored_label(color, format!("  {glyph} {}", c.path));
                }
            }

            if !d.failures.is_empty() {
                ui.add_space(6.0);
                ui.label(RichText::new("Failures").color(theme::WARN).strong());
                for f in &d.failures {
                    ui.colored_label(
                        theme::WARN,
                        format!("  ! {} {} {}", f.name, f.reason, f.source),
                    );
                }
            }
        });
    }

    fn agent_list_tables(&self, ui: &mut egui::Ui, list: &AgentList) {
        if !list.skills.is_empty() {
            ui.label(RichText::new("skills").strong());
            TableBuilder::new(ui)
                .id_salt("agent_skills_tbl")
                .striped(true)
                .column(Column::auto().at_least(120.0))
                .column(Column::auto().at_least(120.0))
                .column(Column::auto().at_least(80.0))
                .column(Column::remainder())
                .column(Column::auto().at_least(80.0))
                .header(20.0, |mut h| {
                    for t in ["name", "skill", "scope", "source", "updated"] {
                        h.col(|ui| {
                            ui.label(RichText::new(t).color(theme::TEXT_FAINT));
                        });
                    }
                })
                .body(|mut body| {
                    for sk in &list.skills {
                        body.row(18.0, |mut row| {
                            row.col(|ui| {
                                ui.label(&sk.name);
                            });
                            row.col(|ui| {
                                ui.label(&sk.skill);
                            });
                            row.col(|ui| {
                                ui.label(format!("{:?}", sk.scope));
                            });
                            row.col(|ui| {
                                ui.label(&sk.source);
                            });
                            row.col(|ui| {
                                ui.label(&sk.updated_ago);
                            });
                        });
                    }
                });
            ui.add_space(8.0);
        }
        for (title, rows, salt) in [
            ("mcps", &list.mcps, "agent_mcps_tbl"),
            ("commands", &list.commands, "agent_commands_tbl"),
        ] {
            if rows.is_empty() {
                continue;
            }
            ui.label(RichText::new(title).strong());
            TableBuilder::new(ui)
                .id_salt(salt)
                .striped(true)
                .column(Column::auto().at_least(140.0))
                .column(Column::auto().at_least(80.0))
                .column(Column::remainder())
                .header(20.0, |mut h| {
                    for t in ["name", "scope", "source"] {
                        h.col(|ui| {
                            ui.label(RichText::new(t).color(theme::TEXT_FAINT));
                        });
                    }
                })
                .body(|mut body| {
                    for r in rows {
                        body.row(18.0, |mut row| {
                            row.col(|ui| {
                                ui.label(&r.name);
                            });
                            row.col(|ui| {
                                ui.label(format!("{:?}", r.scope));
                            });
                            row.col(|ui| {
                                ui.label(&r.source);
                            });
                        });
                    }
                });
            ui.add_space(8.0);
        }
    }

    // ── Secrets (TASK-0028) ─────────────────────────────────────────────────────
    // PURE state→argv builders: NO egui types, unit-testable. Each builds the IDENTICAL
    // `secretctl` clap surface the CLI drives (Architecture B) — the GUI has ZERO mint/revoke
    // logic, so CLI↔GUI cannot diverge. Blank optional fields → flags omitted (mirrors the
    // CLI/consumer builders); the revoke token rides on stdin (`--token -`), never argv.

    /// `secretctl mint-github --installation-id <id> --ttl-secs <ttl> --output json` + conditional
    /// `--repository-ids <csv>` / `--permissions <csv>`. Mirrors the FROZEN consumer argv
    /// (`secretctl/src/main.rs::consumer_build_argv`): scopes are comma-joined and omitted when blank.
    fn mint_github_argv(&self) -> Vec<String> {
        let mut argv = vec![
            "mint-github".to_string(),
            "--installation-id".to_string(),
            self.sec_install_id.trim().to_string(),
            "--ttl-secs".to_string(),
            self.sec_ttl_secs.trim().to_string(),
            "--output".to_string(),
            "json".to_string(),
        ];
        let repo_ids = split_csv(&self.sec_repo_ids);
        if !repo_ids.is_empty() {
            argv.push("--repository-ids".to_string());
            argv.push(repo_ids.join(","));
        }
        let perms = split_csv(&self.sec_perms);
        if !perms.is_empty() {
            argv.push("--permissions".to_string());
            argv.push(perms.join(","));
        }
        argv
    }

    /// `secretctl relay mint <name> [--ttl <s>] [--mode <m>] [--provider <p>] (--repo <r>)*
    /// (--perm <p>)* --json`. Repeated `--repo`/`--perm` per CSV token (mirrors clap's `Vec`).
    /// Does NOT inject the native `checks:write` default — that stays CLI-side (`mint_req_for_relay_mint`).
    fn relay_mint_argv(&self) -> Vec<String> {
        let mut argv = vec![
            "relay".to_string(),
            "mint".to_string(),
            self.sec_relay_name.trim().to_string(),
        ];
        if let Some(ttl) = opt_str(&self.sec_relay_ttl) {
            argv.push("--ttl".to_string());
            argv.push(ttl);
        }
        if let Some(mode) = opt_str(&self.sec_relay_mode) {
            argv.push("--mode".to_string());
            argv.push(mode);
        }
        if let Some(provider) = opt_str(&self.sec_relay_provider) {
            argv.push("--provider".to_string());
            argv.push(provider);
        }
        for repo in split_csv(&self.sec_relay_repos) {
            argv.push("--repo".to_string());
            argv.push(repo);
        }
        for perm in split_csv(&self.sec_relay_perms) {
            argv.push("--perm".to_string());
            argv.push(perm);
        }
        argv.push("--json".to_string());
        argv
    }

    /// `secretctl github-app revoke-token --token - [--installation-id <id>] [--apply] --json`.
    /// `--token -` is ALWAYS present (the token rides on stdin, never argv). `--apply` is omitted
    /// by default (fail-closed dry-run); present only when `sec_revoke_apply` is on.
    fn revoke_argv(&self) -> Vec<String> {
        let mut argv = vec![
            "github-app".to_string(),
            "revoke-token".to_string(),
            "--token".to_string(),
            "-".to_string(),
        ];
        if let Some(id) = opt_str(&self.sec_revoke_install_id) {
            argv.push("--installation-id".to_string());
            argv.push(id);
        }
        if self.sec_revoke_apply {
            argv.push("--apply".to_string());
        }
        argv.push("--json".to_string());
        argv
    }

    /// True when the active verb's required form fields are present and well-typed, so the dispatch
    /// button can be `add_enabled`-gated (no invocation on an invalid form).
    fn secrets_form_ready(&self) -> bool {
        match self.secrets_verb {
            SecretsVerbTab::MintGithub => {
                self.sec_install_id.trim().parse::<u64>().is_ok()
                    && self.sec_ttl_secs.trim().parse::<i64>().is_ok()
            }
            SecretsVerbTab::RelayMint => !self.sec_relay_name.trim().is_empty(),
            SecretsVerbTab::Revoke => !self.sec_revoke_token.trim().is_empty(),
        }
    }

    fn secrets_screen(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("Secrets — mint / relay-mint / revoke (drives secretctl)").heading(),
        );
        ui.add_space(8.0);

        // verb sub-tabs
        ui.horizontal(|ui| {
            for v in [
                SecretsVerbTab::MintGithub,
                SecretsVerbTab::RelayMint,
                SecretsVerbTab::Revoke,
            ] {
                let active = self.secrets_verb == v;
                let text = if active {
                    RichText::new(v.label()).color(theme::ACCENT_TEXT).strong()
                } else {
                    RichText::new(v.label()).color(theme::TEXT_MUTED)
                };
                let btn = egui::Button::new(text)
                    .fill(if active {
                        theme::ACCENT
                    } else {
                        Color32::TRANSPARENT
                    })
                    .rounding(egui::Rounding::same(7.0));
                if ui.add(btn).clicked() {
                    self.secrets_verb = v;
                }
            }
        });
        ui.add_space(10.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            theme::inset().show(ui, |ui| {
                ui.set_max_width(640.0);
                match self.secrets_verb {
                    SecretsVerbTab::MintGithub => self.secrets_mint_form(ui),
                    SecretsVerbTab::RelayMint => self.secrets_relay_form(ui),
                    SecretsVerbTab::Revoke => self.secrets_revoke_form(ui),
                }
                ui.add_space(12.0);
                self.secrets_action_button(ui);
            });

            ui.add_space(10.0);
            if !self.sec_status.is_empty() {
                let color = if self.sec_status.starts_with('⛔') {
                    theme::DANGER
                } else {
                    theme::TEXT_MUTED
                };
                ui.colored_label(color, &self.sec_status);
                ui.add_space(8.0);
            }
            self.secrets_results(ui);
        });
    }

    fn secrets_mint_form(&mut self, ui: &mut egui::Ui) {
        ui.colored_label(
            theme::TEXT_MUTED,
            "Mint a short-lived GitHub App installation token via the daemon. The token is shown ONCE to copy, never persisted.",
        );
        ui.horizontal(|ui| {
            ui.label(RichText::new("Installation id (u64)").color(theme::TEXT_MUTED));
            ui.add(
                egui::TextEdit::singleline(&mut self.sec_install_id)
                    .hint_text("e.g. 4044997")
                    .desired_width(200.0),
            );
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("TTL secs (i64)").color(theme::TEXT_MUTED));
            ui.add(
                egui::TextEdit::singleline(&mut self.sec_ttl_secs)
                    .hint_text("3600")
                    .desired_width(120.0),
            );
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("Repository ids (CSV)").color(theme::TEXT_MUTED));
            ui.add(
                egui::TextEdit::singleline(&mut self.sec_repo_ids)
                    .hint_text("10,20 (blank = installation default)")
                    .desired_width(300.0),
            );
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("Permissions (CSV name:access)").color(theme::TEXT_MUTED));
            ui.add(
                egui::TextEdit::singleline(&mut self.sec_perms)
                    .hint_text("checks:write,contents:read (blank = default)")
                    .desired_width(320.0),
            );
        });
    }

    fn secrets_relay_form(&mut self, ui: &mut egui::Ui) {
        ui.colored_label(
            theme::TEXT_MUTED,
            "Mint a <=24h peer-bound relay bearer under a policy. The bearer itself is never shown — only its metadata.",
        );
        ui.horizontal(|ui| {
            ui.label(RichText::new("Policy name").color(theme::TEXT_MUTED));
            ui.add(
                egui::TextEdit::singleline(&mut self.sec_relay_name)
                    .hint_text("relay policy name")
                    .desired_width(300.0),
            );
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("TTL secs").color(theme::TEXT_MUTED));
            ui.add(
                egui::TextEdit::singleline(&mut self.sec_relay_ttl)
                    .hint_text("(blank = policy default)")
                    .desired_width(150.0),
            );
            ui.label(RichText::new("Mode").color(theme::TEXT_MUTED));
            egui::ComboBox::from_id_salt("sec_relay_mode")
                .selected_text(self.sec_relay_mode.clone())
                .show_ui(ui, |ui| {
                    for m in ["base-url", "proxy", "native"] {
                        ui.selectable_value(&mut self.sec_relay_mode, m.to_string(), m);
                    }
                });
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("Provider").color(theme::TEXT_MUTED));
            egui::ComboBox::from_id_salt("sec_relay_provider")
                .selected_text(self.sec_relay_provider.clone())
                .show_ui(ui, |ui| {
                    for p in ["anthropic", "openai", "github", "generic"] {
                        ui.selectable_value(&mut self.sec_relay_provider, p.to_string(), p);
                    }
                });
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("Repos (CSV, native)").color(theme::TEXT_MUTED));
            ui.add(egui::TextEdit::singleline(&mut self.sec_relay_repos).desired_width(300.0));
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("Perms (CSV, native)").color(theme::TEXT_MUTED));
            ui.add(egui::TextEdit::singleline(&mut self.sec_relay_perms).desired_width(300.0));
        });
    }

    fn secrets_revoke_form(&mut self, ui: &mut egui::Ui) {
        ui.colored_label(
            theme::WARN,
            "⚠ Revoke a leaked GitHub installation token. The token is fed over stdin (never the command line). Preview (dry-run) first; Apply performs the revoke.",
        );
        ui.horizontal(|ui| {
            ui.label(RichText::new("Token (stdin)").color(theme::TEXT_MUTED));
            ui.add(
                egui::TextEdit::singleline(&mut self.sec_revoke_token)
                    .password(true)
                    .hint_text("ghs_… (held transiently, cleared on dispatch)")
                    .desired_width(360.0),
            );
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("Installation id (optional)").color(theme::TEXT_MUTED));
            ui.add(
                egui::TextEdit::singleline(&mut self.sec_revoke_install_id)
                    .hint_text("u64 (optional)")
                    .desired_width(160.0),
            );
        });
        ui.checkbox(
            &mut self.sec_revoke_apply,
            "Apply (revoke) — off = dry-run preview (no egress)",
        );
    }

    /// The single action button. Builds the verb's argv (pure strings) + the optional Zeroizing
    /// stdin (revoke token only), clears the transient token field, and dispatches the engine
    /// subprocess command. Disabled until the form is valid (no invocation on bad input). Fill is
    /// WARN when the destructive apply affordance is armed, else ACCENT.
    fn secrets_action_button(&mut self, ui: &mut egui::Ui) {
        let ready = self.secrets_form_ready();
        let (label, fill) = match self.secrets_verb {
            SecretsVerbTab::MintGithub => ("Mint".to_string(), theme::ACCENT),
            SecretsVerbTab::RelayMint => ("Mint relay".to_string(), theme::ACCENT),
            SecretsVerbTab::Revoke => {
                if self.sec_revoke_apply {
                    ("Revoke (apply)".to_string(), theme::WARN)
                } else {
                    ("Revoke (dry-run)".to_string(), theme::ACCENT)
                }
            }
        };
        let btn = egui::Button::new(RichText::new(label).color(theme::ACCENT_TEXT)).fill(fill);
        if ui.add_enabled(ready, btn).clicked() {
            let cmd = self.build_secrets_command();
            self.dispatch(cmd, Some("secrets".into()));
        }
    }

    /// Build the `EngineCommand::Secrets` for the active verb. For revoke, moves the transient
    /// token into a `Zeroizing<Vec<u8>>` stdin buffer and CLEARS `sec_revoke_token` (never persisted,
    /// never argv). Mutating method (clears the field), so it is split out of the pure argv builders.
    fn build_secrets_command(&mut self) -> EngineCommand {
        match self.secrets_verb {
            SecretsVerbTab::MintGithub => EngineCommand::Secrets {
                verb: "mint-github".to_string(),
                argv: self.mint_github_argv(),
                stdin: None,
            },
            SecretsVerbTab::RelayMint => EngineCommand::Secrets {
                verb: "relay-mint".to_string(),
                argv: self.relay_mint_argv(),
                stdin: None,
            },
            SecretsVerbTab::Revoke => {
                let argv = self.revoke_argv();
                // Move the token into a Zeroizing buffer for stdin, then clear the field.
                let stdin = Zeroizing::new(self.sec_revoke_token.trim().as_bytes().to_vec());
                self.sec_revoke_token.clear();
                EngineCommand::Secrets {
                    verb: "revoke".to_string(),
                    argv,
                    stdin: Some(stdin),
                }
            }
        }
    }

    /// METADATA-ONLY result cards. The mint token (if just minted) gets a SINGLE copy-once button
    /// that moves it to the clipboard and immediately drops it from state — it is never re-rendered
    /// and never logged.
    fn secrets_results(&mut self, ui: &mut egui::Ui) {
        // Mint result: expiry + a copy-once affordance for the transient token (then it's dropped).
        if self.sec_mint_has_token {
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new("GitHub installation token").strong());
                if let Some(exp) = self.sec_mint_expires {
                    ui.colored_label(theme::TEXT_MUTED, format!("expires_at_unix: {exp}"));
                }
                if self.sec_mint_copy_once.is_some() {
                    if ui.button("Copy token (once)").clicked() {
                        // take() so the token is moved out and dropped after this frame — copy once.
                        if let Some(tok) = self.sec_mint_copy_once.take() {
                            ui.output_mut(|o| o.copied_text = tok);
                            self.sec_status =
                                "token copied to clipboard (cleared from memory)".into();
                        }
                    }
                    ui.colored_label(
                        theme::TEXT_FAINT,
                        "The token is held transiently and dropped after one copy.",
                    );
                } else {
                    ui.colored_label(
                        theme::TEXT_FAINT,
                        "token already copied (no longer in memory)",
                    );
                }
            });
            ui.add_space(8.0);
        }

        if let Some(meta) = self.sec_relay_result.clone() {
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new("Relay bearer (metadata only)").strong());
                ui.colored_label(theme::TEXT_MUTED, format!("token_id: {}", meta.token_id));
                ui.colored_label(
                    theme::TEXT_MUTED,
                    format!("expires_at: {}", meta.expires_at),
                );
                ui.colored_label(
                    theme::TEXT_MUTED,
                    format!("native GitHub mint: {}", meta.native),
                );
            });
            ui.add_space(8.0);
        }

        if let Some((revoked, dry_run)) = self.sec_revoke_result {
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new("Revoke result").strong());
                if dry_run {
                    ui.colored_label(theme::WARN, "dry-run: would revoke (no egress)");
                } else {
                    ui.colored_label(
                        if revoked {
                            theme::HEALTHY
                        } else {
                            theme::DANGER
                        },
                        format!("revoked: {revoked}"),
                    );
                }
            });
            ui.add_space(8.0);
        }
    }

    // ── Logs ──────────────────────────────────────────────────────────────────
    fn logs_screen(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Console").heading());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Clear").clicked() {
                    self.log.clear();
                }
                ui.colored_label(theme::TEXT_FAINT, format!("{} lines", self.log.len()));
            });
        });
        ui.add_space(6.0);

        egui::Frame::none()
            .fill(theme::BG)
            .stroke(egui::Stroke::new(1.0, theme::BORDER))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if self.log.is_empty() {
                            ui.colored_label(theme::TEXT_FAINT, "no output yet");
                        }
                        for l in &self.log {
                            ui.label(
                                RichText::new(&l.text)
                                    .monospace()
                                    .size(12.5)
                                    .color(log_color(l)),
                            );
                        }
                    });
            });
    }

    // ── Settings ──────────────────────────────────────────────────────────────
    /// meta mission-control dashboard parity: render the zellij layout from
    /// `.meta.yaml` (read-only) and deploy it (gated by the dry-run toggle, like
    /// the other mutations). Drives the IDENTICAL Engine API the CLI uses via
    /// EngineCommand::Dashboard / DeployDashboard.
    fn mesh_screen(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Mesh — meta mission-control").heading());
        ui.add_space(6.0);
        ui.colored_label(
            theme::TEXT_FAINT,
            "Render a zellij dashboard layout from .meta.yaml (tabs by tag, pane-per-repo).",
        );
        ui.add_space(10.0);

        let start = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let spec = DashboardSpec {
            panes_per_tab: self.dash_panes_per_tab.max(1),
            ..DashboardSpec::default()
        };

        ui.horizontal(|ui| {
            ui.label("Panes per tab:");
            ui.add(egui::DragValue::new(&mut self.dash_panes_per_tab).range(1..=24));
        });
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Button::new(RichText::new("Render").color(theme::ACCENT_TEXT))
                        .fill(theme::ACCENT),
                )
                .clicked()
            {
                let _ = self.cmd_tx.send(EngineCommand::Dashboard {
                    start: start.clone(),
                    meta_file: None,
                    spec: spec.clone(),
                });
            }
            // Deploy is a mutation: dry-run unless the dry-run-default toggle is OFF
            // (mirrors how Fix gates its --apply in this GUI).
            let dry_run = self.dry_run_default;
            let label = if dry_run {
                "Deploy (dry-run)"
            } else {
                "Deploy (apply)"
            };
            if ui.button(label).clicked() {
                let _ = self.cmd_tx.send(EngineCommand::DeployDashboard {
                    start: start.clone(),
                    meta_file: None,
                    spec: spec.clone(),
                    dry_run,
                    force: false,
                });
            }
        });

        if !self.dash_status.is_empty() {
            ui.add_space(8.0);
            ui.colored_label(theme::TEXT_MUTED, &self.dash_status);
        }

        if let Some(plan) = &self.dash_plan {
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
            ui.label(RichText::new(format!("{} ({} tabs)", plan.name, plan.tabs.len())).strong());
            ui.colored_label(
                theme::TEXT_FAINT,
                format!("target: {}", plan.target.display()),
            );
            ui.add_space(8.0);
            egui::ScrollArea::vertical()
                .max_height(420.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut plan.kdl.as_str())
                            .code_editor()
                            .desired_width(f32::INFINITY),
                    );
                });
        }
    }

    fn settings_screen(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Settings").heading());
        ui.add_space(10.0);
        theme::inset().show(ui, |ui| {
            ui.set_max_width(560.0);
            ui.checkbox(
                &mut self.dry_run_default,
                "Destructive ops dry-run by default",
            );
            ui.colored_label(
                theme::TEXT_FAINT,
                "When on, Fix runs in dry-run mode unless explicitly forced.",
            );
            ui.add_space(14.0);
            ui.separator();
            ui.add_space(14.0);
            if ui
                .add(
                    egui::Button::new(RichText::new("Re-detect").color(theme::ACCENT_TEXT))
                        .fill(theme::ACCENT),
                )
                .clicked()
            {
                let _ = self.cmd_tx.send(EngineCommand::Detect);
            }
            ui.colored_label(
                theme::TEXT_FAINT,
                "Re-scan the environment and refresh drift.",
            );
        });
    }
}

// ── small free helpers (no &self borrow) ──────────────────────────────────────

/// A compact "LABEL value" stat chip on a faint surface.
fn stat_chip(ui: &mut egui::Ui, label: &str, value: &str, value_col: Color32) {
    egui::Frame::none()
        .fill(theme::BG)
        .rounding(egui::Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(10.0, 5.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(label).size(10.0).color(theme::TEXT_FAINT));
                ui.label(RichText::new(value).strong().color(value_col));
            });
        });
}

/// Status pill color + short text for a component, factoring in drift severity.
fn pill_for(c: &ComponentState, drift: Option<&DriftItem>) -> (Color32, String) {
    if !c.detected {
        let col = drift.map(|d| sev_color(d.severity)).unwrap_or(theme::WARN);
        return (col, "missing".into());
    }
    if let Some(d) = drift {
        return (sev_color(d.severity), "drift".into());
    }
    match c.healthy {
        Some(false) => (theme::DANGER, "unhealthy".into()),
        Some(true) => (theme::HEALTHY, "healthy".into()),
        None => (theme::HEALTHY, "present".into()),
    }
}

/// A human label for the health/drift cell.
fn health_label(c: &ComponentState, drift: Option<&DriftItem>) -> (Color32, String) {
    // returns (text, color) swapped intentionally? -> keep (color, text)
    if let Some(d) = drift {
        let kind = match d.kind {
            DriftKind::Missing => "missing",
            DriftKind::Unhealthy => "unhealthy",
            DriftKind::WiringMissing => "wiring missing",
            DriftKind::DriverInactive => "driver inactive",
            DriftKind::BoundaryViolation => "boundary violation",
        };
        return (
            sev_color(d.severity),
            format!("{kind} · {}", d.suggested_verb),
        );
    }
    if !c.detected {
        return (theme::WARN, "not installed".into());
    }
    match c.healthy {
        Some(false) => (theme::DANGER, "verify failed".into()),
        Some(true) => (theme::HEALTHY, "ok".into()),
        None => (theme::TEXT_MUTED, "—".into()),
    }
}

fn sev_color(sev: Severity) -> Color32 {
    match sev {
        Severity::High => theme::DANGER,
        Severity::Medium => theme::WARN,
        Severity::Low => theme::INFO,
    }
}

/// Color a console line by stream then by a cheap level heuristic.
fn log_color(l: &LogLine) -> Color32 {
    if l.stream == Stream::Stderr {
        return theme::DANGER;
    }
    let lower = l.text.to_lowercase();
    if lower.contains("refused") || lower.contains("error") || lower.contains("fail") {
        theme::DANGER
    } else if lower.contains("warn") || lower.contains("skip") {
        theme::WARN
    } else if lower.contains("-> ok") || lower.contains("ok\"") || lower.contains("done") {
        theme::HEALTHY
    } else {
        theme::TEXT
    }
}

/// Split a comma/whitespace-separated list into trimmed non-empty tokens.
fn split_csv(s: &str) -> Vec<String> {
    s.split([',', ' ', '\n'])
        .map(|x| x.trim())
        .filter(|x| !x.is_empty())
        .map(|x| x.to_string())
        .collect()
}

/// Trim a form string to an `Option<String>`: blank → `None` (the agent specs' `config_path`
/// /`git_ref`/… mirror the CLI's blank → `None` mapping).
fn opt_str(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

// ── Minimal JSON field scanners (TASK-0028) ───────────────────────────────────
// Architecture B keeps the GUI dep set FROZEN (no serde_json). The daemon's `--json` outputs are
// flat, compact, well-formed objects (`secretctl/src/{main,render}.rs` build them via serde_json,
// so keys are unique and values are properly escaped). These scanners extract a single named field
// from such an object. They are deliberately tiny and self-contained — they do NOT parse arbitrary
// JSON, only pull the metadata fields the secrets verbs emit.

/// Locate the value slice immediately after `"<key>":` (whitespace-skipped). Returns the byte index
/// of the first value char, or `None` if the key is absent.
fn json_value_start(json: &str, key: &str) -> Option<usize> {
    let needle = format!("\"{key}\"");
    let mut from = 0usize;
    while let Some(rel) = json[from..].find(&needle) {
        let kpos = from + rel;
        let after = kpos + needle.len();
        // Skip whitespace, require a colon, skip whitespace again → value start.
        let rest = &json[after..];
        let trimmed = rest.trim_start();
        if let Some(colon) = trimmed.strip_prefix(':') {
            let val = colon.trim_start();
            return Some(json.len() - val.len());
        }
        from = after; // a "<key>" that wasn't a key:value (e.g. inside a string) → keep scanning
    }
    None
}

/// Extract a JSON *string* field, un-escaping `\"` and `\\` (sufficient for the token/id/timestamp
/// values the daemon emits). Returns `None` if the field is absent or not a string.
fn json_string_field(json: &str, key: &str) -> Option<String> {
    let start = json_value_start(json, key)?;
    let bytes = json.as_bytes();
    if bytes.get(start) != Some(&b'"') {
        return None;
    }
    let mut out = String::new();
    let mut i = start + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if i + 1 < bytes.len() => {
                let c = bytes[i + 1];
                match c {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'/' => out.push('/'),
                    b'n' => out.push('\n'),
                    b't' => out.push('\t'),
                    b'r' => out.push('\r'),
                    other => {
                        // unknown escape → keep both chars verbatim (lossless fallback)
                        out.push('\\');
                        out.push(other as char);
                    }
                }
                i += 2;
            }
            b'"' => return Some(out),
            b => {
                out.push(b as char);
                i += 1;
            }
        }
    }
    None // unterminated string
}

/// Extract a JSON *integer* field (i64). Returns `None` if absent or not a bare number.
fn json_number_field(json: &str, key: &str) -> Option<i64> {
    let start = json_value_start(json, key)?;
    let rest = &json[start..];
    let end = rest
        .find(|c: char| !(c.is_ascii_digit() || c == '-' || c == '+'))
        .unwrap_or(rest.len());
    rest[..end].trim().parse::<i64>().ok()
}

/// Extract a JSON *bool* field. Returns `None` if absent or not `true`/`false`.
fn json_bool_field(json: &str, key: &str) -> Option<bool> {
    let start = json_value_start(json, key)?;
    let rest = json[start..].trim_start();
    if rest.starts_with("true") {
        Some(true)
    } else if rest.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

#[cfg(test)]
mod agent_spec_tests {
    use super::*;
    use envctl_engine::AgentLockMode;

    /// A pure, window-free `EnvctlApp` carrying just the agent form fields. The worker channel
    /// and engine clone aren't needed by the `*_spec` builders, so a fresh test app constructs
    /// dummy channels and a default engine, then the test mutates only the agent fields.
    fn test_app() -> EnvctlApp {
        let (cmd_tx, _cmd_rx) = channel::<EngineCommand>();
        let (_evt_tx, evt_rx) = channel::<EngineEvent>();
        // detached() needs no manifest dir, so the test runs from any cwd; the
        // `*_spec` builders touch neither this engine clone nor the channels.
        let geng = Engine::detached();
        // run_event_loop is never started; the builders touch no channel/engine.
        EnvctlApp {
            cmd_tx,
            evt_rx,
            screen: Screen::Agent,
            header: String::new(),
            components: Vec::new(),
            drift: Vec::new(),
            busy: HashSet::new(),
            log: VecDeque::new(),
            log_cap: 8000,
            telemetry: None,
            util_history: HashMap::new(),
            dry_run_default: true,
            filter: String::new(),
            tel: TelemetryControl::new(),
            geng,
            graph_focus: String::new(),
            gpu_present: false,
            driver_loaded: false,
            software_rendered: false,
            gpu_count: 0,
            add_url: String::new(),
            add_id: String::new(),
            add_build: String::new(),
            add_strategy: "as-is".into(),
            add_ref: String::new(),
            add_bins: String::new(),
            add_renames: String::new(),
            add_patch: String::new(),
            add_ai_goal: "port-to-rust".into(),
            add_ai_instruction: String::new(),
            add_build_flag: false,
            dash_plan: None,
            dash_panes_per_tab: 6,
            dash_status: String::new(),
            agent_verb: AgentVerbTab::Sync,
            agent_config: String::new(),
            agent_scope: AgentScopeSel::Default,
            agent_source: String::new(),
            agent_skills: String::new(),
            agent_mcps: String::new(),
            agent_commands: String::new(),
            agent_git_ref: String::new(),
            agent_branch: String::new(),
            agent_sub_dir: String::new(),
            agent_apply: false,
            agent_no_sync: false,
            agent_no_verify: false,
            agent_locked: false,
            agent_update_on: false,
            agent_update: String::new(),
            agent_lock_check: true,
            agent_upgrade_pkg: String::new(),
            agent_list_kind: AgentListKindSel::All,
            agent_list: None,
            agent_list_stale: false,
            agent_last_edit: None,
            agent_last_report: None,
            agent_lock_drift: None,
            agent_last_doctor: None,
            agent_status: String::new(),
            secrets_verb: SecretsVerbTab::MintGithub,
            sec_install_id: String::new(),
            sec_repo_ids: String::new(),
            sec_perms: String::new(),
            sec_ttl_secs: "3600".into(),
            sec_relay_name: String::new(),
            sec_relay_ttl: String::new(),
            sec_relay_mode: "base-url".into(),
            sec_relay_provider: "generic".into(),
            sec_relay_repos: String::new(),
            sec_relay_perms: String::new(),
            sec_revoke_token: String::new(),
            sec_revoke_install_id: String::new(),
            sec_revoke_apply: false,
            sec_status: String::new(),
            sec_mint_expires: None,
            sec_mint_has_token: false,
            sec_mint_copy_once: None,
            sec_relay_result: None,
            sec_revoke_result: None,
        }
    }

    #[test]
    fn sync_blank_config_is_none_apply_defaults_false() {
        let app = test_app();
        let spec = app.agent_sync_spec();
        assert_eq!(spec.config_path, None, "blank config → None");
        assert_eq!(spec.scope_override, None, "default scope → no override");
        assert!(!spec.apply, "apply defaults false (fail-closed)");
        assert!(matches!(spec.lock_mode, AgentLockMode::Plain));
    }

    #[test]
    fn sync_config_and_apply_and_scope_map() {
        let mut app = test_app();
        app.agent_config = "  /tmp/cfg.toml ".into();
        app.agent_apply = true;
        app.agent_scope = AgentScopeSel::Project;
        let spec = app.agent_sync_spec();
        assert_eq!(spec.config_path.as_deref(), Some("/tmp/cfg.toml"));
        assert!(spec.apply);
        assert_eq!(spec.scope_override, Some(AgentScope::Project));
    }

    #[test]
    fn lock_mode_locked_wins() {
        let mut app = test_app();
        app.agent_locked = true;
        app.agent_update_on = true;
        app.agent_update = "a,b".into();
        assert!(matches!(app.agent_lock_mode(), AgentLockMode::Locked));
    }

    #[test]
    fn lock_mode_update_csv_splits() {
        let mut app = test_app();
        app.agent_update_on = true;
        app.agent_update = "a, b".into();
        match app.agent_lock_mode() {
            AgentLockMode::Update { only } => assert_eq!(only, vec!["a", "b"]),
            other => panic!("expected Update, got {other:?}"),
        }
    }

    #[test]
    fn lock_mode_update_off_is_plain() {
        let mut app = test_app();
        app.agent_update_on = false;
        app.agent_update = "a,b".into(); // ignored while toggle off
        assert!(matches!(app.agent_lock_mode(), AgentLockMode::Plain));
    }

    #[test]
    fn add_section_csv_and_fields_map() {
        let mut app = test_app();
        app.agent_source = " repo ".into();
        app.agent_skills = "a,b".into();
        app.agent_mcps = "m1".into();
        app.agent_commands = "".into();
        app.agent_git_ref = "v1".into();
        app.agent_branch = "".into();
        app.agent_sub_dir = "sub".into();
        app.agent_no_sync = true;
        app.agent_no_verify = true;
        let spec = app.agent_add_spec();
        assert_eq!(spec.source, "repo");
        assert_eq!(spec.section.skills, vec!["a", "b"]);
        assert_eq!(spec.section.mcps, vec!["m1"]);
        assert!(spec.section.commands.is_empty());
        assert_eq!(spec.git_ref.as_deref(), Some("v1"));
        assert_eq!(spec.branch, None);
        assert_eq!(spec.sub_dir.as_deref(), Some("sub"));
        assert!(spec.no_sync);
        assert!(spec.no_verify);
        assert!(!spec.apply, "apply defaults false");
    }

    #[test]
    fn remove_maps_without_no_verify() {
        let mut app = test_app();
        app.agent_source = "repo".into();
        app.agent_commands = "c1,c2".into();
        app.agent_no_sync = true;
        let spec = app.agent_remove_spec();
        assert_eq!(spec.source, "repo");
        assert_eq!(spec.section.commands, vec!["c1", "c2"]);
        assert!(spec.no_sync);
        assert!(!spec.apply);
    }

    #[test]
    fn lock_check_and_upgrade_map() {
        let mut app = test_app();
        app.agent_lock_check = true;
        app.agent_upgrade_pkg = "p1, p2".into();
        app.agent_locked = true;
        let spec = app.agent_lock_spec();
        assert!(spec.check);
        assert_eq!(spec.upgrade_only, vec!["p1", "p2"]);
        // lock ignores --update; --locked makes the audit zero-network.
        assert!(matches!(spec.lock_mode, AgentLockMode::Locked));
    }

    #[test]
    fn list_kind_maps() {
        let mut app = test_app();
        app.agent_list_kind = AgentListKindSel::Mcps;
        app.agent_scope = AgentScopeSel::Global;
        let spec = app.agent_list_spec();
        assert_eq!(spec.kind, AgentListKind::Mcps);
        assert_eq!(spec.scope_override, Some(AgentScope::Global));
    }

    #[test]
    fn clean_apply_defaults_false() {
        let app = test_app();
        let spec = app.agent_clean_spec();
        assert!(!spec.apply, "clean apply defaults false (fail-closed)");
        assert_eq!(spec.scope_override, None);
    }

    #[test]
    fn agent_command_wraps_active_verb() {
        let mut app = test_app();
        app.agent_verb = AgentVerbTab::Clean;
        match app.agent_command() {
            EngineCommand::Agent {
                spec: AgentCommandSpec::Clean(_),
            } => {}
            _ => panic!("expected EngineCommand::Agent(Clean)"),
        }
    }

    // ===== TASK-0028: Secrets (mint-github / relay-mint / revoke) ============================
    //
    // The argv-parity tests use VERBATIM REPLICATION of the `secretctl` arg-struct shapes (copied
    // from `secretctl/src/{cli,main}.rs`) — NO `envctl-secretctl` dev-dep, so the GUI dev graph never
    // pulls tonic/tokio. If the GUI's argv builder diverges from the frozen secretctl surface, the
    // replicated assertions below fail.

    /// VERBATIM replica of `secretctl`'s `MintGithubArgs` parse semantics (`cli.rs:101-120`):
    /// `--installation-id`(u64), `--ttl-secs`(i64), `--output`(String, must be "json"),
    /// `--repository-ids`(comma-delimited Vec), `--permissions`(comma-delimited Vec). Parsing the
    /// GUI's argv through this replica proves the GUI drives the IDENTICAL clap surface the CLI does.
    #[derive(Default, Debug, PartialEq)]
    struct MintGithubArgsReplica {
        installation_id: u64,
        ttl_secs: i64,
        output: String,
        repository_ids: Vec<String>,
        permissions: Vec<String>,
    }
    fn parse_mint_github_replica(argv: &[String]) -> MintGithubArgsReplica {
        assert_eq!(argv.first().map(String::as_str), Some("mint-github"));
        let mut a = MintGithubArgsReplica::default();
        let mut i = 1;
        while i < argv.len() {
            match argv[i].as_str() {
                "--installation-id" => {
                    a.installation_id = argv[i + 1].parse().expect("u64 installation-id");
                    i += 2;
                }
                "--ttl-secs" => {
                    a.ttl_secs = argv[i + 1].parse().expect("i64 ttl-secs");
                    i += 2;
                }
                "--output" => {
                    a.output = argv[i + 1].clone();
                    i += 2;
                }
                "--repository-ids" => {
                    a.repository_ids = argv[i + 1].split(',').map(str::to_string).collect();
                    i += 2;
                }
                "--permissions" => {
                    a.permissions = argv[i + 1].split(',').map(str::to_string).collect();
                    i += 2;
                }
                other => panic!("unexpected mint-github arg: {other}"),
            }
        }
        a
    }

    // ---- Test 1: argv parity (anti-divergence) ----
    #[test]
    fn mint_github_argv_round_trips_through_replica() {
        let mut app = test_app();
        app.secrets_verb = SecretsVerbTab::MintGithub;
        app.sec_install_id = "99".into();
        app.sec_ttl_secs = "3600".into();
        app.sec_repo_ids = "10,20".into();
        app.sec_perms = "checks:write,contents:read".into();
        let parsed = parse_mint_github_replica(&app.mint_github_argv());
        assert_eq!(parsed.installation_id, 99);
        assert_eq!(parsed.ttl_secs, 3600);
        assert_eq!(
            parsed.output, "json",
            "fixed --output json (frozen contract)"
        );
        assert_eq!(parsed.repository_ids, vec!["10", "20"]);
        assert_eq!(parsed.permissions, vec!["checks:write", "contents:read"]);
    }

    // ---- Test 2: blank optional scopes omitted ----
    #[test]
    fn mint_github_argv_omits_blank_optional_scopes() {
        let mut app = test_app();
        app.sec_install_id = "4044997".into();
        app.sec_ttl_secs = "600".into();
        app.sec_repo_ids = "".into();
        app.sec_perms = "  ".into();
        let argv = app.mint_github_argv();
        assert!(
            !argv.iter().any(|a| a == "--repository-ids"),
            "blank repo-ids ⇒ flag omitted"
        );
        assert!(
            !argv.iter().any(|a| a == "--permissions"),
            "blank perms ⇒ flag omitted"
        );
        let parsed = parse_mint_github_replica(&argv);
        assert_eq!(parsed.installation_id, 4_044_997);
        assert!(parsed.repository_ids.is_empty());
        assert!(parsed.permissions.is_empty());
    }

    // ---- Test 3: relay mode/provider/repos/perms mapping; no GUI-side checks:write default ----
    #[test]
    fn relay_mint_argv_maps_mode_provider_repos_perms() {
        let mut app = test_app();
        app.secrets_verb = SecretsVerbTab::RelayMint;
        app.sec_relay_name = "mypolicy".into();
        app.sec_relay_ttl = "1800".into();
        app.sec_relay_mode = "native".into();
        app.sec_relay_provider = "github".into();
        app.sec_relay_repos = "org/a,org/b".into();
        app.sec_relay_perms = "".into(); // GUI must NOT inject checks:write — that is CLI-side
        let argv = app.relay_mint_argv();
        // positional name first after `relay mint`
        assert_eq!(&argv[0], "relay");
        assert_eq!(&argv[1], "mint");
        assert_eq!(&argv[2], "mypolicy");
        // optional scalars present
        assert!(argv.windows(2).any(|w| w == ["--ttl", "1800"]));
        assert!(argv.windows(2).any(|w| w == ["--mode", "native"]));
        assert!(argv.windows(2).any(|w| w == ["--provider", "github"]));
        // repeated --repo per CSV token
        assert!(argv.windows(2).any(|w| w == ["--repo", "org/a"]));
        assert!(argv.windows(2).any(|w| w == ["--repo", "org/b"]));
        // NO --perm at all (blank) — the native checks:write default is applied by secretctl, not here
        assert!(
            !argv.iter().any(|a| a == "--perm"),
            "GUI must not inject the native checks:write default"
        );
        // and the structured json flag is always present
        assert_eq!(argv.last().map(String::as_str), Some("--json"));
    }

    #[test]
    fn relay_mint_argv_omits_blank_optionals() {
        let mut app = test_app();
        app.sec_relay_name = "p".into();
        app.sec_relay_ttl = "".into();
        // mode/provider default to non-empty ("base-url"/"generic") so they ARE present.
        app.sec_relay_repos = "".into();
        app.sec_relay_perms = "".into();
        let argv = app.relay_mint_argv();
        assert!(!argv.iter().any(|a| a == "--ttl"), "blank ttl omitted");
        assert!(!argv.iter().any(|a| a == "--repo"));
        assert!(!argv.iter().any(|a| a == "--perm"));
        // defaults present
        assert!(argv.windows(2).any(|w| w == ["--mode", "base-url"]));
        assert!(argv.windows(2).any(|w| w == ["--provider", "generic"]));
    }

    // ---- Test 4: revoke defaults dry-run + token via stdin (never argv) ----
    #[test]
    fn revoke_argv_defaults_dry_run_uses_stdin_token() {
        let mut app = test_app();
        app.secrets_verb = SecretsVerbTab::Revoke;
        app.sec_revoke_token = "ghs_supersecret".into();
        app.sec_revoke_install_id = "42".into();
        // apply defaults false
        let argv = app.revoke_argv();
        assert_eq!(&argv[0], "github-app");
        assert_eq!(&argv[1], "revoke-token");
        // `--token -` always present; the literal token NEVER appears in argv.
        assert!(
            argv.windows(2).any(|w| w == ["--token", "-"]),
            "stdin token sentinel"
        );
        assert!(
            !argv.iter().any(|a| a.contains("ghs_supersecret")),
            "the secret token must NEVER appear in argv"
        );
        assert!(argv.windows(2).any(|w| w == ["--installation-id", "42"]));
        assert!(
            !argv.iter().any(|a| a == "--apply"),
            "no --apply by default (fail-closed dry-run)"
        );
        assert_eq!(argv.last().map(String::as_str), Some("--json"));

        // With apply on, --apply appears.
        app.sec_revoke_apply = true;
        let argv = app.revoke_argv();
        assert!(
            argv.iter().any(|a| a == "--apply"),
            "explicit apply adds --apply"
        );
    }

    // ---- Test 5: apply toggle defaults false ----
    #[test]
    fn revoke_apply_toggle_defaults_false() {
        let app = test_app();
        assert!(
            !app.sec_revoke_apply,
            "revoke apply defaults false (fail-closed)"
        );
    }

    // ---- Test 6: no-persist — secret stdin moves into Zeroizing; token field cleared on dispatch ----
    #[test]
    fn revoke_dispatch_moves_token_to_stdin_and_clears_field() {
        let mut app = test_app();
        app.secrets_verb = SecretsVerbTab::Revoke;
        app.sec_revoke_token = "ghs_leaked".into();
        let cmd = app.build_secrets_command();
        // field cleared (never persisted)
        assert!(
            app.sec_revoke_token.is_empty(),
            "the transient token field is cleared after dispatch"
        );
        match cmd {
            EngineCommand::Secrets { verb, argv, stdin } => {
                assert_eq!(verb, "revoke");
                // token rides on stdin, never argv
                assert!(!argv.iter().any(|a| a.contains("ghs_leaked")));
                let buf = stdin.expect("revoke supplies a stdin buffer");
                assert_eq!(&buf[..], b"ghs_leaked");
            }
            _ => panic!("expected EngineCommand::Secrets"),
        }
    }

    #[test]
    fn mint_and_relay_dispatch_have_no_stdin() {
        let mut app = test_app();
        app.secrets_verb = SecretsVerbTab::MintGithub;
        app.sec_install_id = "1".into();
        match app.build_secrets_command() {
            EngineCommand::Secrets { verb, stdin, .. } => {
                assert_eq!(verb, "mint-github");
                assert!(stdin.is_none(), "mint has no stdin");
            }
            _ => panic!("expected Secrets"),
        }
        app.secrets_verb = SecretsVerbTab::RelayMint;
        app.sec_relay_name = "p".into();
        match app.build_secrets_command() {
            EngineCommand::Secrets { verb, stdin, .. } => {
                assert_eq!(verb, "relay-mint");
                assert!(stdin.is_none(), "relay has no stdin");
            }
            _ => panic!("expected Secrets"),
        }
    }

    // ---- Test 7: JSON metadata parse — only metadata retained (no bearer, no mint token logged) ----
    #[test]
    fn handle_revoke_result_keeps_only_metadata() {
        let mut app = test_app();
        app.handle_secrets_result("revoke", r#"{"revoked":true,"dry_run":false}"#, "", Some(0));
        assert_eq!(app.sec_revoke_result, Some((true, false)));
    }

    #[test]
    fn handle_relay_result_drops_bearer() {
        let mut app = test_app();
        // bearer present in stdout — must NOT be retained anywhere.
        let stdout = r#"{"bearer":"SECRET_BEARER","token_id":"tok-123","expires_at":"2026-06-18T00:00:00Z","native":true}"#;
        app.handle_secrets_result("relay-mint", stdout, "", Some(0));
        let meta = app.sec_relay_result.clone().expect("relay metadata parsed");
        assert_eq!(meta.token_id, "tok-123");
        assert_eq!(meta.expires_at, "2026-06-18T00:00:00Z");
        assert!(meta.native);
        // The bearer must never appear in status or anywhere in the metadata struct (the struct has
        // no bearer field; assert the value didn't leak into the status line either).
        assert!(!app.sec_status.contains("SECRET_BEARER"));
    }

    #[test]
    fn handle_mint_result_holds_token_transiently_not_logged() {
        let mut app = test_app();
        let stdout = r#"{"token":"ghs_MINTED","expires_at_unix":1750000000}"#;
        app.handle_secrets_result("mint-github", stdout, "", Some(0));
        assert_eq!(app.sec_mint_expires, Some(1_750_000_000));
        assert!(app.sec_mint_has_token);
        // token held transiently for the copy-once affordance...
        assert_eq!(app.sec_mint_copy_once.as_deref(), Some("ghs_MINTED"));
        // ...but it must NEVER be in the log or the status line.
        assert!(!app.sec_status.contains("ghs_MINTED"));
        assert!(
            !app.log.iter().any(|l| l.text.contains("ghs_MINTED")),
            "mint stdout must NEVER flow through push_log"
        );
    }

    // ---- Test 8: degrade — secretctl-absent / non-zero exit renders the explanatory DANGER state ----
    #[test]
    fn handle_failure_renders_danger_no_success_card() {
        let mut app = test_app();
        // code=None ⇒ binary not found / failed to spawn (the engine's fail-closed not-found path).
        app.handle_secrets_result("revoke", "", "secretctl not installed", None);
        assert!(
            app.sec_status.starts_with('⛔'),
            "fail-closed DANGER status"
        );
        assert!(app.sec_status.contains("not installed"));
        // No success metadata is synthesized.
        assert!(app.sec_revoke_result.is_none());
        assert!(!app.sec_mint_has_token);
    }

    #[test]
    fn handle_nonzero_exit_surfaces_stderr_not_stdout() {
        let mut app = test_app();
        // A non-zero exit with secret-free stderr: stdout (even if it had a token) must be ignored.
        app.handle_secrets_result(
            "mint-github",
            r#"{"token":"ghs_SHOULD_BE_IGNORED","expires_at_unix":1}"#,
            "vault is locked (is secretd running?)",
            Some(1),
        );
        assert!(app.sec_status.starts_with('⛔'));
        assert!(app.sec_status.contains("vault is locked"));
        assert!(!app.sec_mint_has_token, "no success on a non-zero exit");
        assert!(app.sec_mint_copy_once.is_none());
        assert!(
            !app.sec_status.contains("ghs_SHOULD_BE_IGNORED"),
            "stdout token never parsed/shown on failure"
        );
    }

    // ---- JSON scanner unit tests (the no-serde-json extractors) ----
    #[test]
    fn json_scanners_extract_named_fields() {
        let j = r#"{"token":"a\"b","expires_at_unix":1750000000,"native":true,"dry_run":false}"#;
        assert_eq!(json_string_field(j, "token").as_deref(), Some("a\"b"));
        assert_eq!(json_number_field(j, "expires_at_unix"), Some(1_750_000_000));
        assert_eq!(json_bool_field(j, "native"), Some(true));
        assert_eq!(json_bool_field(j, "dry_run"), Some(false));
        assert_eq!(json_string_field(j, "absent"), None);
        assert_eq!(json_number_field(j, "absent"), None);
        assert_eq!(json_bool_field(j, "absent"), None);
    }
}
