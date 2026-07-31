//! The best-effort run loop. Maps a `RunPlan` to an ordered target list (topo
//! forward for install/fix, reverse for remove), runs each phase, accumulates a
//! `RunSummary`, and emits `RunStarted`/`StepStarted`/`StepFinished`/`RunFinished`
//! so the CLI and GUI render identically. Also `add_repo()` — the hardened
//! drop-in writer.
//!
//! Best-effort = one component failing never aborts the run (mirrors the wizard's
//! `run()`), but a failed component's dependents are `SkippedBlocked`, and an
//! `install` whose `detect` already passes is `Skipped` (idempotent).
use crate::component::{Component, HookRunner, Phase};
use crate::error::{run_phase, RunContext};
use crate::event::{Event, EventSink, Stream};
use crate::layout::MetaLayout;
use crate::model::{
    AddRepoMode, AddRepoSpec, ComponentAvailability, OpResult, OpStatus, Registry, ResetGates,
    RunPlan, RunSummary, Wiring,
};
use std::collections::HashSet;
use std::path::Path;

/// Resolve run-wide identities ONCE (no TOCTOU). GPU presence via the PCI floor
/// (driver-independent); live-root UUID via findmnt+blkid for the guard engine.
fn resolve_context() -> RunContext {
    RunContext {
        gpu_present: crate::detect::pci_nvidia_count() > 0,
        live_root_uuid: crate::guard::resolve_live_root_uuid(),
    }
}

pub fn run(
    reg: &Registry,
    runner: &dyn HookRunner,
    plan: RunPlan,
    sink: &EventSink,
) -> anyhow::Result<RunSummary> {
    let ctx = resolve_context();

    // Forward order, or reverse for Remove (tear down dependents first).
    let mut order: Vec<&Component> = if plan.targets.is_empty() {
        reg.ordered().collect()
    } else if plan.phase == Phase::Remove {
        // AUDIT-FIX (blocker): Remove must NEVER expand a target to its
        // prerequisites. `closure(id)` is id + transitive PREREQS — correct for
        // install (deps must exist first) but catastrophic for remove: it would
        // make `reset claude-code-cli` also uninstall shared `bun`. The base set
        // is exactly the named targets; `--cascade` folds in reverse-DEPENDENTS
        // (never prerequisites) in the gate block below. Validate each id exists.
        let mut set: HashSet<&str> = HashSet::new();
        for id in &plan.targets {
            if reg.get(id).is_none() {
                anyhow::bail!("unknown component '{id}'");
            }
            set.insert(id.as_str());
        }
        reg.ordered()
            .filter(|c| set.contains(c.id.as_str()))
            .collect()
    } else {
        let mut v: Vec<&Component> = Vec::new();
        for id in &plan.targets {
            for c in reg.closure(id)? {
                if !v.iter().any(|x| x.id == c.id) {
                    v.push(c);
                }
            }
        }
        v
    };
    if plan.phase == Phase::Remove {
        order.reverse();
    }

    let mut summary = RunSummary::default();

    // ---- Reset gates (Phase::Remove only), evaluated ONCE under frozen ctx ----
    if plan.phase == Phase::Remove {
        // (1) Untargeted whole-roster reset requires --all AND --confirm.
        if plan.targets.is_empty() && !(plan.gates.all && plan.gates.confirm) {
            let reason = "refusing whole-roster reset: pass --all --confirm".to_string();
            sink.emit(Event::GuardRefused {
                component: "<reset>".into(),
                reason: reason.clone(),
            });
            summary.refused.push("<reset>".into());
            finish(
                sink,
                &mut summary,
                mkres_id(
                    "<reset>",
                    Phase::Remove,
                    OpStatus::Refused,
                    &reason,
                    plan.dry_run,
                ),
            );
            sink.emit(Event::RunFinished {
                summary: summary.clone(),
            });
            return Ok(summary);
        }
        // (2)+(3) Reverse-dependent refusal / cascade fold (explicit targets only).
        if !plan.targets.is_empty() {
            let target_set: HashSet<String> = order.iter().map(|c| c.id.clone()).collect();
            let mut fold: HashSet<String> = HashSet::new();
            let mut refuse: HashSet<String> = HashSet::new();
            for tid in &plan.targets {
                for rdep in reg.reverse_dependents(tid) {
                    // audit fix (minor): the reverse-dependent's Detect hook is run here
                    // purely as a liveness PROBE to decide refuse/cascade — detect hooks
                    // must therefore be side-effect-free (idempotent, read-only).
                    let live = run_phase(sink, rdep, Phase::Detect, runner, false, &ctx).status
                        == OpStatus::Ok;
                    if live && !target_set.contains(&rdep.id) {
                        if plan.gates.cascade {
                            fold.insert(rdep.id.clone());
                        } else {
                            refuse.insert(tid.clone());
                        }
                    }
                }
            }
            for tid in &refuse {
                let reason = format!("refusing remove of {tid}: a live reverse-dependent is not in the set (use --cascade)");
                sink.emit(Event::GuardRefused {
                    component: tid.clone(),
                    reason: reason.clone(),
                });
                summary.refused.push(tid.clone());
                finish(
                    sink,
                    &mut summary,
                    mkres_id(tid, Phase::Remove, OpStatus::Refused, &reason, plan.dry_run),
                );
            }
            // Folding extra components beyond the named targets needs --confirm on --apply.
            if !fold.is_empty() && !plan.gates.confirm && !plan.dry_run {
                let list: Vec<String> = {
                    let mut v: Vec<String> = fold.into_iter().collect();
                    v.sort();
                    v
                };
                let reason = format!(
                    "refusing cascade: would also remove {} — pass --confirm",
                    list.join(", ")
                );
                sink.emit(Event::GuardRefused {
                    component: "<cascade>".into(),
                    reason: reason.clone(),
                });
                summary.refused.push("<cascade>".into());
                finish(
                    sink,
                    &mut summary,
                    mkres_id(
                        "<cascade>",
                        Phase::Remove,
                        OpStatus::Refused,
                        &reason,
                        plan.dry_run,
                    ),
                );
                sink.emit(Event::RunFinished {
                    summary: summary.clone(),
                });
                return Ok(summary);
            }
            // Rebuild the removal set = (surviving named targets) ∪ (folded
            // reverse-dependents). NO closure: a target's prerequisites are never
            // auto-removed (blocker [8]), and a refused target is dropped so the
            // live reverse-dependent it protects survives (blocker [0] / FOCUS #0).
            if !refuse.is_empty() || !fold.is_empty() {
                let mut keep: HashSet<String> = plan
                    .targets
                    .iter()
                    .filter(|t| !refuse.contains(*t))
                    .cloned()
                    .collect();
                keep.extend(fold.iter().cloned());
                order = reg.ordered().filter(|c| keep.contains(&c.id)).collect();
                order.reverse();
            }
        }
    }

    // Pre-warm sudo (+ keepalive) if this run will need it; dropped at fn end.
    let _sudo = prewarm_sudo(&order, plan.phase, plan.dry_run, sink);

    let total = order.len();
    sink.emit(Event::RunStarted {
        phase: plan.phase,
        total,
        dry_run: plan.dry_run,
    });

    let mut failed_ids: HashSet<String> = HashSet::new();

    for (i, comp) in order.iter().enumerate() {
        let install_post_verify =
            plan.phase == Phase::Install && comp.refresh_on_verify_failure && comp.verify.is_some();
        sink.emit(Event::StepStarted {
            component: comp.id.clone(),
            phase: plan.phase,
            index: i,
            total,
        });

        // Dependency gate (forward phases): a dependency that already failed
        // this run blocks its dependents instead of running them on rubble.
        if matches!(plan.phase, Phase::Install | Phase::Fix)
            && comp.requires.iter().any(|d| failed_ids.contains(d))
        {
            let res = mkres(
                comp,
                plan.phase,
                OpStatus::SkippedBlocked,
                "dependency failed",
                plan.dry_run,
            )
            .with_availability(ComponentAvailability::Unavailable);
            summary.skipped_blocked.push(comp.id.clone());
            failed_ids.insert(comp.id.clone());
            finish(sink, &mut summary, res);
            continue;
        }

        // Idempotent install: skip-if-already-detected (never re-run curl|bash),
        // but still reconcile its declarative wiring (idempotent) so an already-
        // installed tool with a missing PATH/rc block gets fixed.
        if plan.phase == Phase::Install && !plan.dry_run {
            // run_phase (not runner.run) so the probe gets catch_unwind + gpu/guard
            // treatment, consistent with every other phase (audit fix).
            let detected = comp.detect.is_some()
                && run_phase(sink, comp, Phase::Detect, runner, false, &ctx).status == OpStatus::Ok;
            let refresh_required = detected
                && comp.refresh_on_verify_failure
                && comp.verify.is_some()
                && run_phase(sink, comp, Phase::Verify, runner, false, &ctx).status != OpStatus::Ok;
            if detected && !refresh_required {
                {
                    let mut res = mkres(
                        comp,
                        plan.phase,
                        OpStatus::Skipped,
                        "already present",
                        false,
                    )
                    .with_availability(ComponentAvailability::Healthy);
                    apply_wiring(comp, sink, &mut res, &mut summary);
                    if install_post_verify {
                        if let Some(d) = reverify_install_healthy(comp, runner, sink, &ctx) {
                            res = d;
                            summary.incomplete.push(comp.id.clone());
                            failed_ids.insert(comp.id.clone());
                        }
                    }
                    if res.status == OpStatus::Incomplete {
                        failed_ids.insert(comp.id.clone());
                    }
                    finish(sink, &mut summary, res);
                    continue;
                }
            }
        }

        // Auto-fix triage (Phase::Fix): act ONLY on broken/partial components.
        if plan.phase == Phase::Fix && !plan.dry_run {
            if comp.detect.is_some()
                && run_phase(sink, comp, Phase::Detect, runner, false, &ctx).status != OpStatus::Ok
            {
                failed_ids.insert(comp.id.clone());
                summary.skipped_blocked.push(comp.id.clone());
                finish(
                    sink,
                    &mut summary,
                    mkres(
                        comp,
                        Phase::Fix,
                        OpStatus::SkippedBlocked,
                        "not installed; use install",
                        false,
                    )
                    .with_availability(ComponentAvailability::Unavailable),
                );
                continue;
            }
            let healthy = comp.verify.is_none()
                || run_phase(sink, comp, Phase::Verify, runner, false, &ctx).status == OpStatus::Ok;
            if healthy && wiring_present(comp) {
                finish(
                    sink,
                    &mut summary,
                    mkres(
                        comp,
                        Phase::Fix,
                        OpStatus::Skipped,
                        "already healthy",
                        false,
                    )
                    .with_availability(ComponentAvailability::Healthy),
                );
                continue;
            }
            // A system-scope fix (apt/nix/cdi/alt) is destructive infra — gate it.
            if has_system_scope(&comp.wiring) && !plan.gates.confirm {
                let reason = "system-scope fix needs --confirm".to_string();
                sink.emit(Event::GuardRefused {
                    component: comp.id.clone(),
                    reason: reason.clone(),
                });
                summary.refused.push(comp.id.clone());
                failed_ids.insert(comp.id.clone());
                finish(
                    sink,
                    &mut summary,
                    mkres(comp, Phase::Fix, OpStatus::Refused, &reason, false),
                );
                continue;
            }
        }

        let mut res = run_phase(sink, comp, plan.phase, runner, plan.dry_run, &ctx);
        match res.status {
            OpStatus::Failed => {
                summary.failed.push(comp.id.clone());
                failed_ids.insert(comp.id.clone());
            }
            OpStatus::Refused => {
                summary.refused.push(comp.id.clone());
                failed_ids.insert(comp.id.clone());
            }
            OpStatus::SkippedBlocked => {
                summary.skipped_blocked.push(comp.id.clone());
                failed_ids.insert(comp.id.clone());
            }
            OpStatus::Skipped if res.availability == Some(ComponentAvailability::Unavailable) => {
                summary.skipped_blocked.push(comp.id.clone());
                failed_ids.insert(comp.id.clone());
            }
            _ => {}
        }

        // Wiring + post-action re-verify (frozen ctx; never on dry-run; only when
        // the hook actually acted: Ok | NoHook).
        if !plan.dry_run && matches!(res.status, OpStatus::Ok | OpStatus::NoHook) {
            match plan.phase {
                Phase::Install => {
                    let install_hook_absent = res.status == OpStatus::NoHook;
                    apply_wiring(comp, sink, &mut res, &mut summary);
                    // A detected component reached Install only because its opt-in Verify failed.
                    // Prove the refresh and its wiring/start effects before reporting success.
                    if res.status != OpStatus::Incomplete && install_hook_absent {
                        if let Some(d) = reverify_install_present(comp, runner, sink, &ctx) {
                            res = d;
                            summary.incomplete.push(comp.id.clone());
                        } else if comp.detect.is_some() {
                            res.availability = Some(ComponentAvailability::Healthy);
                        }
                    } else if install_post_verify {
                        if let Some(d) = reverify_install_healthy(comp, runner, sink, &ctx) {
                            res = d;
                            summary.incomplete.push(comp.id.clone());
                        }
                    }
                }
                Phase::Remove => {
                    revert_wiring(comp, &plan.gates, &ctx, sink, &mut res, &mut summary);
                    // reset must leave the component ABSENT.
                    if let Some(d) = reverify_absent(comp, runner, sink, &ctx) {
                        res = d;
                        summary.incomplete.push(comp.id.clone());
                    }
                }
                Phase::Fix => {
                    apply_wiring(comp, sink, &mut res, &mut summary);
                    // auto-fix must leave the component HEALTHY.
                    if let Some(d) = reverify_healthy(comp, runner, sink, &ctx) {
                        res = d;
                        summary.incomplete.push(comp.id.clone());
                    }
                }
                _ => {}
            }
        }

        if matches!(plan.phase, Phase::Install | Phase::Fix) && res.status == OpStatus::Incomplete {
            failed_ids.insert(comp.id.clone());
        }

        finish(sink, &mut summary, res);
    }

    // Dedup the roster vecs — a component can be pushed onto a roster twice
    // (e.g. wiring-fail + reverify-fail both mark incomplete) (audit fix).
    for v in [
        &mut summary.failed,
        &mut summary.refused,
        &mut summary.skipped_blocked,
        &mut summary.incomplete,
    ] {
        v.sort();
        v.dedup();
    }

    sink.emit(Event::RunFinished {
        summary: summary.clone(),
    });
    Ok(summary)
}

fn mkres(comp: &Component, phase: Phase, status: OpStatus, msg: &str, dry_run: bool) -> OpResult {
    OpResult {
        component: comp.id.clone(),
        phase,
        status,
        availability: None,
        exit_code: None,
        duration_ms: 0,
        message: msg.into(),
        dry_run,
    }
}

fn finish(sink: &EventSink, summary: &mut RunSummary, res: OpResult) {
    sink.emit(Event::StepFinished {
        result: res.clone(),
    });
    summary.results.push(res);
}

fn mkres_id(id: &str, phase: Phase, status: OpStatus, msg: &str, dry_run: bool) -> OpResult {
    OpResult {
        component: id.into(),
        phase,
        status,
        availability: None,
        exit_code: None,
        duration_ms: 0,
        message: msg.into(),
        dry_run,
    }
}

fn wiring_empty(w: &Wiring) -> bool {
    w.path_entries.is_empty()
        && w.shell_rc.is_empty()
        && w.desktop_entries.is_empty()
        && w.apt_repos.is_empty()
        && w.nix_conf_lines.is_empty()
        && w.cdi_specs.is_empty()
        && w.alternatives.is_empty()
        && w.data_paths.is_empty()
        && w.config_paths.is_empty()
}

fn has_system_scope(w: &Wiring) -> bool {
    !w.apt_repos.is_empty()
        || !w.nix_conf_lines.is_empty()
        || !w.cdi_specs.is_empty()
        || !w.alternatives.is_empty()
}

/// True iff every wiring footprint this component owns is present on disk
/// (matches detect.rs::wiring_present; suffix-agnostic so wizard-written blocks
/// count). AUDIT-FIX (#4): previously only shell_rc was inspected, so a
/// component whose only footprint is system-scope wiring (path_entries/apt_repos/
/// nix_conf_lines/cdi_specs/alternatives) always reported present — its absence
/// was undetectable. Now each owned footprint is conservatively probed.
fn wiring_present(comp: &Component) -> bool {
    let w = &comp.wiring;
    let layout = MetaLayout::from_env_or_default();

    let shell_rc_ok = w.shell_rc.iter().all(|blk| {
        let file = layout.expand_meta_path(&blk.file);
        // Suffix-agnostic: wizard-written blocks and envctl-written blocks both
        // satisfy the same marker. Keep this in sync with detect.rs.
        std::fs::read_to_string(&file)
            .map(|t| t.contains(&format!("BEGIN {}", blk.marker)))
            .unwrap_or(false)
    });

    // path_entries are realized into the engine-owned "envctl PATH" block in
    // $META_ROOT/.bashrc (see wiring::path_block); probe for that marker.
    let path_ok = w.path_entries.is_empty() || {
        std::fs::read_to_string(layout.meta_root().join(".bashrc"))
            .map(|t| t.contains("BEGIN envctl PATH"))
            .unwrap_or(false)
    };

    // System-scope footprints: each is present iff its on-disk target exists
    // (mirrors wiring.rs apply targets: SOURCES_D/<list_file>, NIX_CONF line,
    // cdi output file, alternative link).
    let apt_ok = w.apt_repos.iter().all(|r| {
        std::path::Path::new(&format!("/etc/apt/sources.list.d/{}", r.list_file)).exists()
    });
    let nix_ok = w.nix_conf_lines.is_empty() || {
        std::fs::read_to_string("/etc/nix/nix.custom.conf")
            .map(|t| w.nix_conf_lines.iter().all(|l| t.contains(&l.line)))
            .unwrap_or(false)
    };
    let cdi_ok = w
        .cdi_specs
        .iter()
        .all(|c| std::path::Path::new(&c.output).exists());
    let alt_ok = w
        .alternatives
        .iter()
        .all(|a| std::path::Path::new(&a.link).exists());

    shell_rc_ok && path_ok && apt_ok && nix_ok && cdi_ok && alt_ok
}

fn emit_wiring(comp: &Component, sink: &EventSink, rep: &crate::wiring::WiringReport, verb: &str) {
    for n in &rep.notes {
        sink.emit(Event::Log {
            component: comp.id.clone(),
            stream: Stream::Stdout,
            line: n.clone(),
        });
    }
    for (kind, e) in &rep.failures {
        sink.emit(Event::Log {
            component: comp.id.clone(),
            stream: Stream::Stderr,
            line: format!("wiring {verb} ({kind}) failed: {e}"),
        });
    }
    if rep.notes.is_empty() && rep.failures.is_empty() {
        sink.emit(Event::Log {
            component: comp.id.clone(),
            stream: Stream::Stdout,
            line: format!("wiring {verb}"),
        });
    }
}

fn apply_wiring(comp: &Component, sink: &EventSink, res: &mut OpResult, summary: &mut RunSummary) {
    if wiring_empty(&comp.wiring) {
        return;
    }
    let rep = crate::wiring::apply(&comp.wiring);
    emit_wiring(comp, sink, &rep, "applied");
    if !rep.failures.is_empty()
        && matches!(
            res.status,
            OpStatus::Ok | OpStatus::NoHook | OpStatus::Skipped
        )
    {
        res.status = OpStatus::Incomplete;
        res.availability = Some(ComponentAvailability::Unavailable);
        res.message = "wiring apply reported failures (see log)".into();
        summary.incomplete.push(comp.id.clone());
    }
}

fn revert_wiring(
    comp: &Component,
    gates: &ResetGates,
    ctx: &RunContext,
    sink: &EventSink,
    res: &mut OpResult,
    summary: &mut RunSummary,
) {
    if wiring_empty(&comp.wiring) {
        return;
    }
    let rep = crate::wiring::revert(&comp.wiring, gates, ctx);
    emit_wiring(comp, sink, &rep, "reverted");
    if !rep.failures.is_empty() && matches!(res.status, OpStatus::Ok | OpStatus::NoHook) {
        res.status = OpStatus::Incomplete;
        res.availability = Some(ComponentAvailability::Unavailable);
        res.message = "wiring revert reported failures (see log)".into();
        summary.incomplete.push(comp.id.clone());
    }
}

/// reset/Remove postcondition: detect must now FAIL (absent). No detect hook =>
/// unverifiable => satisfied (None).
fn reverify_absent(
    comp: &Component,
    runner: &dyn HookRunner,
    sink: &EventSink,
    ctx: &RunContext,
) -> Option<OpResult> {
    comp.detect.as_ref()?;
    if run_phase(sink, comp, Phase::Detect, runner, false, ctx).status == OpStatus::Ok {
        Some(mkres(
            comp,
            Phase::Remove,
            OpStatus::Incomplete,
            "removed, but still detected (orphaned/partial remove) — re-run reset or inspect",
            false,
        ))
    } else {
        None
    }
}

/// auto-fix/Fix postcondition: verify must now SUCCEED (healthy). No verify hook
/// => unverifiable => satisfied (None).
fn reverify_healthy(
    comp: &Component,
    runner: &dyn HookRunner,
    sink: &EventSink,
    ctx: &RunContext,
) -> Option<OpResult> {
    comp.verify.as_ref()?;
    if run_phase(sink, comp, Phase::Verify, runner, false, ctx).status == OpStatus::Ok {
        None
    } else {
        Some(mkres(
            comp,
            Phase::Fix,
            OpStatus::Incomplete,
            "fix ran, but verify still fails — review log / escalate",
            false,
        ))
    }
}

/// refresh-on-install postcondition: the opt-in repair must leave the component HEALTHY after
/// wiring has been applied. No verify hook is impossible for the caller, but remains fail-safe.
fn reverify_install_healthy(
    comp: &Component,
    runner: &dyn HookRunner,
    sink: &EventSink,
    ctx: &RunContext,
) -> Option<OpResult> {
    comp.verify.as_ref()?;
    if run_phase(sink, comp, Phase::Verify, runner, false, ctx).status == OpStatus::Ok {
        None
    } else {
        Some(
            mkres(
                comp,
                Phase::Install,
                OpStatus::Incomplete,
                "refresh install ran, but verify still fails — review log / escalate",
                false,
            )
            .with_availability(ComponentAvailability::Unavailable),
        )
    }
}

/// An absent component with no Install hook may still be a wiring-owned aggregate, but wiring is
/// not proof that its Detect contract became true. Re-detect after wiring; an unproved prerequisite
/// is unavailable and must block its dependency closure.
fn reverify_install_present(
    comp: &Component,
    runner: &dyn HookRunner,
    sink: &EventSink,
    ctx: &RunContext,
) -> Option<OpResult> {
    comp.detect.as_ref()?;
    if run_phase(sink, comp, Phase::Detect, runner, false, ctx).status == OpStatus::Ok {
        None
    } else {
        Some(
            mkres(
                comp,
                Phase::Install,
                OpStatus::Incomplete,
                "no install hook acted and the component is still unavailable after wiring",
                false,
            )
            .with_availability(ComponentAvailability::Unavailable),
        )
    }
}

/// Pre-warm sudo once (so streamed, TTY-less hooks don't prompt) and keep the
/// credential fresh for the duration of the run. Returns a guard that stops the
/// keepalive on drop. No-op unless the run actually needs sudo.
struct SudoKeepalive {
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}
impl Drop for SudoKeepalive {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

fn needs_sudo(order: &[&Component], phase: Phase) -> bool {
    order.iter().any(|c| {
        // System-scope wiring (apt/nix/cdi/alt) runs sudo during apply/revert.
        has_system_scope(&c.wiring)
            || match c.hook(phase) {
                Some(crate::component::Hook::Command { needs_sudo, .. }) => *needs_sudo,
                Some(crate::component::Hook::ShippedScript { needs_sudo, .. }) => *needs_sudo,
                Some(crate::component::Hook::Script {
                    needs_sudo, script, ..
                }) => *needs_sudo || script.contains("sudo "),
                None => false,
            }
    })
}

fn prewarm_sudo(
    order: &[&Component],
    phase: Phase,
    dry_run: bool,
    sink: &EventSink,
) -> Option<SudoKeepalive> {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    if dry_run || !matches!(phase, Phase::Install | Phase::Fix | Phase::Remove) {
        return None;
    }
    if !needs_sudo(order, phase) {
        return None;
    }
    // `sudo -v` inherits this process's stdio: from a real terminal it prompts
    // once; with no TTY it fails fast (and we warn) rather than hanging later.
    let ok = trusted_sudo_command()
        .arg("-v")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !ok {
        sink.emit(Event::Log {
            component: "sudo".into(),
            stream: Stream::Stderr,
            line: "could not pre-authorize sudo (no TTY / not a sudoer?) — privileged steps will fail fast. Run from a real terminal."
                .into(),
        });
        return None;
    }
    sink.emit(Event::Log {
        component: "sudo".into(),
        stream: Stream::Stdout,
        line: "sudo pre-authorized; keepalive running for this run".into(),
    });
    let stop = Arc::new(AtomicBool::new(false));
    let stop2 = stop.clone();
    let handle = std::thread::spawn(move || {
        while !stop2.load(Ordering::Relaxed) {
            let _ = trusted_sudo_command()
                .arg("-n")
                .arg("--")
                .arg("/usr/bin/true")
                .status();
            for _ in 0..50 {
                if stop2.load(Ordering::Relaxed) {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    });
    Some(SudoKeepalive {
        stop,
        handle: Some(handle),
    })
}

fn trusted_sudo_command() -> std::process::Command {
    let mut command = std::process::Command::new("/usr/bin/sudo");
    command.env_clear().env("PATH", "/usr/bin:/bin");
    for key in ["TERM", "LANG", "LC_ALL", "LC_CTYPE"] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    command
}

// ---------------------------------------------------------------------------
// add-repo (hardened drop-in writer). Phase-0 scope: validate + register a
// component drop-in atomically (with backup); the build itself runs on the next
// explicit `envctl install <id>`. The full 9-stage build pipeline is Phase 4.
// ---------------------------------------------------------------------------
pub fn add_repo(
    manifest_dir: &Path,
    reg: &Registry,
    _runner: &dyn HookRunner,
    spec: AddRepoSpec,
    dry_run: bool,
    sink: &EventSink,
) -> anyhow::Result<RunSummary> {
    let id = spec.id.trim().to_string();
    validate_add_repo_spec(&spec)?; // shared gate: id slug + ''' + leading-dash guard

    // ROUTE: peer (meta-native, .meta.yaml) vs component (build-from-source drop-in).
    // `Auto` (default) sends owned/FlexNetOS remotes down the peer path — the
    // meta-repo model — and everything else down the legacy component path. This
    // is what un-drifts add-repo from the child-repo policy.
    if resolve_peer(&spec) {
        return crate::peer::register_peer(&spec, dry_run, sink);
    }

    if reg.get(&id).is_some() {
        anyhow::bail!(
            "component id '{id}' already exists — refusing to shadow it (pick another --id)"
        );
    }

    // Run the staged pipeline (acquire → [transform] → detect → build → locate →
    // shape). It refuses as root, gates real work behind spec.allow_build, and
    // streams every stage. Returns the partial summary + outcome.
    let preview = dry_run || !spec.allow_build;
    let repos_root = repos_root(preview)?;
    let (mut summary, outcome) = crate::addrepo::run_pipeline(&spec, &repos_root, dry_run, sink)?;
    let Some(outcome) = outcome else {
        return Ok(summary); // pipeline short-circuited (root-refusal / a stage failed)
    };

    let bsys = crate::detect_build::system_tag(outcome.build_plan.system).to_string();
    let installed: Vec<String> = outcome
        .installs
        .iter()
        .map(|(n, _)| local_bin_target(&spec, n))
        .collect();
    let rspec = build_register_spec(&id, &spec, &outcome, &bsys, &installed);

    // PREVIEW path: no --build (or --dry-run) → show the drop-in, write nothing.
    if !spec.allow_build || dry_run {
        let toml = crate::register::synth_dropin(&rspec);
        sink.emit(Event::Log {
            component: id.clone(),
            stream: Stream::Stdout,
            line: format!("[preview] would register components.d/{id}.toml:\n{toml}"),
        });
        return Ok(summary);
    }

    // Re-check id-collision against a FRESH registry (close the long-pipeline
    // TOCTOU) BEFORE installing — so a concurrent registration can't leave
    // orphaned meta usr/bin frontdoors + a PATH block behind on the bail path (audit fix).
    if let Ok(fresh) = Registry::load(manifest_dir) {
        if fresh.get(&id).is_some() {
            anyhow::bail!(
                "component id '{id}' was registered concurrently — refusing to overwrite"
            );
        }
    }

    // INSTALL + WIRE-IN (regular frontdoor, refuse-shadow, refuse-unmanaged-unless-force).
    let iplan = build_install_plan(&id, &spec, &outcome);
    let ireport = crate::install::install_and_wire(&iplan, spec.force, false, sink);
    // AUDIT-FIX (#24): a half-installed add-repo must NOT persist a drop-in. If
    // install_and_wire reported failures — or produced no installed paths when
    // installs were expected — the symlinks/targets we'd record never landed, so
    // writing components.d/<id>.toml would create permanent drift (and a later
    // `reset <id>` would try to unwire links that never existed). Bail BEFORE
    // write_dropin so a failed install leaves nothing registered.
    let installs_expected = !iplan.artifacts.is_empty();
    if !ireport.failures.is_empty() || (installs_expected && ireport.installed_paths.is_empty()) {
        summary.failed.push(format!("{id}/install"));
        sink.emit(Event::Log {
            component: id.clone(),
            stream: Stream::Stderr,
            line: format!("install failed for '{id}' — not registering a drop-in (no half-installed component persisted)"),
        });
        sink.emit(Event::RunFinished {
            summary: summary.clone(),
        });
        return Ok(summary);
    }

    let final_targets = if ireport.installed_paths.is_empty() {
        installed.clone()
    } else {
        ireport.installed_paths.clone()
    };
    let rspec = RegisterSpec {
        installed_targets: final_targets,
        ..rspec
    };
    let toml = crate::register::synth_dropin(&rspec);
    write_dropin(manifest_dir, &id, &toml, sink)?;

    sink.emit(Event::Log {
        component: id.clone(),
        stream: Stream::Stdout,
        line: format!("registered '{id}' (build-from-source). Manage with: envctl auto-detect / install {id} / reset {id} --apply"),
    });
    sink.emit(Event::RunFinished {
        summary: summary.clone(),
    });
    Ok(summary)
}

use crate::install::{ArtifactPlan, InstallPlan};
use crate::model::{BuildStrategy, Refactor};
use crate::register::RegisterSpec;

/// The SINGLE add-repo gate, shared by every entry point (`executor::add_repo`
/// AND `addrepo::connect_agent`). Validates the id slug (no `/`, `..`, leading
/// dash, ≤64 chars) and every user-supplied string (leading-dash option
/// injection, `'''` manifest break, ref shape). Call this BEFORE any path join
/// or git invocation. (AUDIT-FIX blocker: the `--connect` path used to skip both
/// of these, allowing `--id ../../etc/x` traversal and git option-injection.)
/// Decide whether this add-repo goes down the PEER path. `Auto` (default) routes
/// owned/FlexNetOS remotes to peer; an explicit `--mode peer|component` overrides.
/// A local-only working tree (no remote URL) can never be a peer, so `Auto`/`Peer`
/// fall back to component there (and `peer::plan_peer` bails with a clear message
/// if `Peer` was forced).
pub(crate) fn resolve_peer(spec: &AddRepoSpec) -> bool {
    match spec.mode {
        AddRepoMode::Component => false,
        AddRepoMode::Peer => true,
        AddRepoMode::Auto => {
            !spec.git_url.trim().is_empty() && crate::peer::is_owned_remote(&spec.git_url)
        }
    }
}

pub(crate) fn validate_add_repo_spec(spec: &AddRepoSpec) -> anyhow::Result<()> {
    let id = spec.id.trim();
    if !is_valid_slug(id) {
        anyhow::bail!(
            "invalid component id '{id}': start [a-z0-9], then [a-z0-9._-] (no spaces/slashes/..)"
        );
    }
    validate_spec_strings(spec)
}

pub(crate) fn validate_spec_strings(spec: &AddRepoSpec) -> anyhow::Result<()> {
    let mut strs: Vec<(&str, String)> = vec![
        ("git_url", spec.git_url.clone()),
        ("build_cmd", spec.build_cmd.clone()),
    ];
    if let Some(r) = &spec.git_ref {
        strs.push(("git_ref", r.clone()));
    }
    // audit fix (minor): verify_cmd is a user string too — guard it for '''/charset
    // so the register docstring's "every user string is guarded" claim holds.
    if let Some(v) = &spec.verify_cmd {
        strs.push(("verify_cmd", v.clone()));
    }
    for g in &spec.artifacts {
        strs.push(("artifact", g.clone()));
    }
    match &spec.strategy {
        BuildStrategy::Refactor {
            refactor: Refactor::Patch { command },
        } => strs.push(("patch_cmd", command.clone())),
        BuildStrategy::Refactor {
            refactor:
                Refactor::Ai {
                    instruction: Some(i),
                    ..
                },
        } => strs.push(("ai_instruction", i.clone())),
        BuildStrategy::Rename { renames } => {
            for r in renames {
                if !is_valid_slug(&r.to) {
                    anyhow::bail!("--rename target '{}' is not a valid install name", r.to);
                }
                strs.push(("rename_from", r.from.clone()));
            }
        }
        BuildStrategy::CherryPick { bins } => {
            for b in bins {
                strs.push(("bin", b.clone()));
            }
        }
        _ => {}
    }
    for (label, val) in strs {
        if val.contains("'''") {
            anyhow::bail!("{label} may not contain ''' (would break the generated manifest)");
        }
    }
    // AUDIT-FIX (major): reject leading-dash values — git treats them as options
    // (e.g. git_url `--upload-pack=…` => arbitrary exec) — and validate the ref shape.
    if spec.git_url.starts_with('-') {
        anyhow::bail!("git_url may not start with '-'");
    }
    if let Some(lp) = &spec.local_path {
        if lp.to_string_lossy().starts_with('-') {
            anyhow::bail!("--local path may not start with '-'");
        }
    }
    if let Some(r) = &spec.git_ref {
        if r.starts_with('-')
            || !r
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/'))
        {
            anyhow::bail!("invalid --git-ref '{r}' (use [A-Za-z0-9._/-], no leading '-')");
        }
    }
    Ok(())
}

fn repos_root(preview: bool) -> std::io::Result<std::path::PathBuf> {
    let layout = crate::layout::MetaLayout::from_env_or_default();
    if !preview {
        layout.ensure_dirs()?;
    }
    Ok(layout.repo_store())
}

fn local_bin_target(_spec: &AddRepoSpec, name: &str) -> String {
    crate::layout::MetaLayout::from_env_or_default()
        .bin()
        .join(name)
        .display()
        .to_string()
}

fn build_install_plan(
    id: &str,
    _spec: &AddRepoSpec,
    outcome: &crate::addrepo::PipelineOutcome,
) -> InstallPlan {
    let artifacts = outcome
        .installs
        .iter()
        .map(|(name, src)| {
            let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            ArtifactPlan {
                source: src.clone(),
                install_name: name.clone(),
                renamed: name != stem,
            }
        })
        .collect();
    InstallPlan {
        id: id.into(),
        slug: id.into(),
        artifacts,
        extra_path_entries: vec![],
    }
}

fn build_register_spec(
    id: &str,
    spec: &AddRepoSpec,
    outcome: &crate::addrepo::PipelineOutcome,
    build_system: &str,
    installed: &[String],
) -> RegisterSpec {
    let strategy_tag = match &spec.strategy {
        BuildStrategy::AsIs => "as-is",
        BuildStrategy::CherryPick { .. } => "cherry-pick",
        BuildStrategy::Rename { .. } => "rename",
        BuildStrategy::Refactor {
            refactor: Refactor::Patch { .. },
        } => "refactor:patch",
        BuildStrategy::Refactor {
            refactor: Refactor::Ai { .. },
        } => "refactor:ai",
    }
    .to_string();
    let transform = match &spec.strategy {
        BuildStrategy::Refactor {
            refactor: Refactor::Patch { command },
        } => Some(command.clone()),
        BuildStrategy::Refactor {
            refactor: Refactor::Ai {
                goal, instruction, ..
            },
        } => Some(format!(
            "ai goal={goal:?} {}",
            instruction.clone().unwrap_or_default()
        )),
        _ => None,
    };
    let relinks: Vec<(String, String)> = outcome
        .installs
        .iter()
        .map(|(name, src)| {
            let rel = src
                .strip_prefix(&outcome.clone_dir)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| src.to_string_lossy().into_owned());
            (name.clone(), rel)
        })
        .collect();
    let primary = outcome.installs.first().map(|(n, _)| n.clone());
    RegisterSpec {
        id: id.into(),
        slug: id.into(),
        display_name: format!("{id} (add-repo)"),
        source: spec.git_url.clone(),
        git_ref: spec.git_ref.clone(),
        resolved_sha: outcome.resolved_sha.clone().unwrap_or_default(),
        strategy_tag,
        build_system: build_system.into(),
        build_cmd: outcome.build_plan.build_cmd.clone(),
        transform,
        primary_bin: primary,
        verify_cmd: spec.verify_cmd.clone(),
        relinks,
        installed_targets: installed.to_vec(),
    }
}

fn write_dropin(
    manifest_dir: &Path,
    id: &str,
    toml_text: &str,
    sink: &EventSink,
) -> anyhow::Result<()> {
    let dir = manifest_dir.join("components.d");
    std::fs::create_dir_all(&dir)?;
    let target = dir.join(format!("{id}.toml"));
    if target.exists() {
        // audit fix (minor): nanosecond epoch + uniqueness loop so two backups taken
        // within the same instant don't clobber each other (matches install.rs).
        let mut bak = dir.join(format!("{id}.toml.bak.{}", now_epoch()));
        let mut n = 0u32;
        while bak.symlink_metadata().is_ok() {
            n += 1;
            bak = dir.join(format!("{id}.toml.bak.{}.{n}", now_epoch()));
        }
        std::fs::copy(&target, &bak)?;
        sink.emit(Event::Log {
            component: id.into(),
            stream: Stream::Stdout,
            line: format!("backed up existing drop-in -> {}", bak.display()),
        });
    }
    let tmp = dir.join(format!(".{id}.toml.tmp"));
    std::fs::write(&tmp, toml_text)?;
    std::fs::rename(&tmp, &target)?;
    Ok(())
}

pub(crate) fn is_valid_slug(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphanumeric() => {}
        _ => return false,
    }
    s.len() <= 64
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

// audit fix (minor): nanosecond resolution so two same-second backups produce
// distinct `.bak.<n>` suffixes instead of colliding (matches install.rs/wiring.rs).
fn now_epoch() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sudo_prewarm_uses_an_absolute_scrubbed_entrypoint() {
        let command = trusted_sudo_command();
        assert_eq!(command.get_program(), "/usr/bin/sudo");
        assert_eq!(
            command
                .get_envs()
                .find(|(key, _)| *key == std::ffi::OsStr::new("PATH"))
                .and_then(|(_, value)| value),
            Some(std::ffi::OsStr::new("/usr/bin:/bin"))
        );
        assert!(command
            .get_envs()
            .all(|(key, _)| key != std::ffi::OsStr::new("LD_PRELOAD")));
    }
    use crate::component::{Hook, HookRunner};
    use crate::event::EventSink;
    use crate::model::{AddRepoSpec, ShellRcBlock, Wiring};
    use std::os::unix::fs::PermissionsExt;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct RefreshRunner {
        detected: AtomicBool,
        current: AtomicBool,
        repair_succeeds: AtomicBool,
        calls: Mutex<Vec<Phase>>,
    }

    impl RefreshRunner {
        fn new() -> Self {
            Self {
                detected: AtomicBool::new(true),
                current: AtomicBool::new(false),
                repair_succeeds: AtomicBool::new(true),
                calls: Mutex::new(Vec::new()),
            }
        }

        fn take_calls(&self) -> Vec<Phase> {
            std::mem::take(
                &mut *self
                    .calls
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            )
        }
    }

    impl HookRunner for RefreshRunner {
        fn run(
            &self,
            comp: &str,
            phase: Phase,
            _hook: &Hook,
            dry_run: bool,
            _sink: &EventSink,
        ) -> OpResult {
            self.calls
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(phase);
            let status = match phase {
                Phase::Detect if self.detected.load(Ordering::SeqCst) => OpStatus::Ok,
                Phase::Detect => OpStatus::Failed,
                Phase::Verify if self.current.load(Ordering::SeqCst) => OpStatus::Ok,
                Phase::Verify => OpStatus::Failed,
                Phase::Install | Phase::Fix => {
                    if self.repair_succeeds.load(Ordering::SeqCst) {
                        self.current.store(true, Ordering::SeqCst);
                    }
                    OpStatus::Ok
                }
                Phase::Remove => OpStatus::Ok,
            };
            OpResult {
                component: comp.to_string(),
                phase,
                status,
                availability: None,
                exit_code: None,
                duration_ms: 0,
                message: String::new(),
                dry_run,
            }
        }
    }

    #[test]
    fn verify_refresh_opt_in_repairs_present_drift_on_install_and_fix() {
        let root = temp_root("executor-verify-refresh");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("refresh.toml"),
            r#"
[[component]]
id = "sqld-fixture"
name = "sqld fixture"
refresh_on_verify_failure = true

[component.detect]
kind = "command"
command = "true"

[component.install]
kind = "command"
command = "true"

[component.verify]
kind = "command"
command = "true"

[component.fix]
kind = "command"
command = "true"
"#,
        )
        .unwrap();
        let registry = Registry::load(&root).unwrap();
        assert!(
            registry
                .get("sqld-fixture")
                .unwrap()
                .refresh_on_verify_failure
        );
        let runner = RefreshRunner::new();
        let sink = EventSink::null();

        let install = super::run(
            &registry,
            &runner,
            RunPlan::new(Phase::Install, vec!["sqld-fixture".into()], false),
            &sink,
        )
        .unwrap();
        assert!(install.ok());
        assert_eq!(
            runner.take_calls(),
            vec![Phase::Detect, Phase::Verify, Phase::Install, Phase::Verify]
        );

        runner.current.store(false, Ordering::SeqCst);
        let fix = super::run(
            &registry,
            &runner,
            RunPlan::new(Phase::Fix, vec!["sqld-fixture".into()], false),
            &sink,
        )
        .unwrap();
        assert!(fix.ok());
        assert_eq!(
            runner.take_calls(),
            vec![Phase::Detect, Phase::Verify, Phase::Fix, Phase::Verify]
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn verify_refresh_opt_in_proves_first_install_and_healthy_skip() {
        let root = temp_root("executor-verify-refresh-convergence");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("refresh.toml"),
            r#"
[[component]]
id = "sqld-fixture"
name = "sqld fixture"
refresh_on_verify_failure = true

[component.detect]
kind = "command"
command = "true"

[component.install]
kind = "command"
command = "true"

[component.verify]
kind = "command"
command = "true"
"#,
        )
        .unwrap();
        let registry = Registry::load(&root).unwrap();
        let runner = RefreshRunner::new();
        let sink = EventSink::null();

        runner.detected.store(false, Ordering::SeqCst);
        let first = super::run(
            &registry,
            &runner,
            RunPlan::new(Phase::Install, vec!["sqld-fixture".into()], false),
            &sink,
        )
        .unwrap();
        assert!(first.ok());
        assert_eq!(
            runner.take_calls(),
            vec![Phase::Detect, Phase::Install, Phase::Verify]
        );

        runner.detected.store(true, Ordering::SeqCst);
        runner.current.store(true, Ordering::SeqCst);
        let healthy = super::run(
            &registry,
            &runner,
            RunPlan::new(Phase::Install, vec!["sqld-fixture".into()], false),
            &sink,
        )
        .unwrap();
        assert!(healthy.ok());
        assert_eq!(
            runner.take_calls(),
            vec![Phase::Detect, Phase::Verify, Phase::Verify]
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn verify_refresh_install_is_incomplete_when_postcondition_stays_broken() {
        let root = temp_root("executor-verify-refresh-postcondition");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("refresh.toml"),
            r#"
[[component]]
id = "forged-helper-fixture"
name = "forged helper fixture"
refresh_on_verify_failure = true

[component.detect]
kind = "command"
command = "true"

[component.install]
kind = "command"
command = "true"

[component.verify]
kind = "command"
command = "false"
"#,
        )
        .unwrap();
        let registry = Registry::load(&root).unwrap();
        let runner = RefreshRunner::new();
        runner.repair_succeeds.store(false, Ordering::SeqCst);
        let summary = super::run(
            &registry,
            &runner,
            RunPlan::new(Phase::Install, vec!["forged-helper-fixture".into()], false),
            &EventSink::null(),
        )
        .unwrap();

        assert!(!summary.ok());
        assert_eq!(summary.incomplete, vec!["forged-helper-fixture"]);
        assert_eq!(
            runner.take_calls(),
            vec![Phase::Detect, Phase::Verify, Phase::Install, Phase::Verify]
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn failed_install_postcondition_blocks_dependents() {
        struct Runner {
            calls: Mutex<Vec<(String, Phase)>>,
        }
        impl HookRunner for Runner {
            fn run(
                &self,
                comp: &str,
                phase: Phase,
                _hook: &Hook,
                dry_run: bool,
                _sink: &EventSink,
            ) -> OpResult {
                self.calls
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push((comp.to_owned(), phase));
                let status = match (comp, phase) {
                    ("sqld-fixture", Phase::Detect | Phase::Verify) => OpStatus::Failed,
                    _ => OpStatus::Ok,
                };
                OpResult {
                    component: comp.to_owned(),
                    phase,
                    status,
                    availability: None,
                    exit_code: None,
                    duration_ms: 0,
                    message: String::new(),
                    dry_run,
                }
            }
        }

        let root = temp_root("executor-install-postcondition-dependency");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("refresh.toml"),
            r#"
[[component]]
id = "sqld-fixture"
name = "sqld fixture"
refresh_on_verify_failure = true
[component.detect]
kind = "command"
command = "true"
[component.install]
kind = "command"
command = "true"
[component.verify]
kind = "command"
command = "false"

[[component]]
id = "dependent"
name = "dependent"
requires = ["sqld-fixture"]
[component.detect]
kind = "command"
command = "false"
[component.install]
kind = "command"
command = "true"
"#,
        )
        .unwrap();
        let registry = Registry::load(&root).unwrap();
        let runner = Runner {
            calls: Mutex::new(Vec::new()),
        };
        let summary = super::run(
            &registry,
            &runner,
            RunPlan::new(
                Phase::Install,
                vec!["sqld-fixture".into(), "dependent".into()],
                false,
            ),
            &EventSink::null(),
        )
        .unwrap();
        assert!(!summary.ok());
        assert_eq!(summary.incomplete, vec!["sqld-fixture"]);
        assert_eq!(summary.skipped_blocked, vec!["dependent"]);
        assert_eq!(
            *runner
                .calls
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            vec![
                ("sqld-fixture".into(), Phase::Detect),
                ("sqld-fixture".into(), Phase::Install),
                ("sqld-fixture".into(), Phase::Verify),
            ]
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn refused_dependency_blocks_the_full_transitive_chain() {
        struct Runner {
            calls: Mutex<Vec<(String, Phase)>>,
        }
        impl HookRunner for Runner {
            fn run(
                &self,
                comp: &str,
                phase: Phase,
                _hook: &Hook,
                dry_run: bool,
                _sink: &EventSink,
            ) -> OpResult {
                self.calls
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push((comp.to_owned(), phase));
                let status = match (comp, phase) {
                    ("a", Phase::Detect) => OpStatus::Failed,
                    ("a", Phase::Install) => OpStatus::Refused,
                    _ => OpStatus::Ok,
                };
                OpResult {
                    component: comp.to_owned(),
                    phase,
                    status,
                    availability: None,
                    exit_code: None,
                    duration_ms: 0,
                    message: String::new(),
                    dry_run,
                }
            }
        }

        let root = temp_root("executor-refused-transitive");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("chain.toml"),
            r#"
[[component]]
id = "a"
name = "a"
[component.detect]
kind = "command"
command = "false"
[component.install]
kind = "command"
command = "false"

[[component]]
id = "b"
name = "b"
requires = ["a"]
[component.install]
kind = "command"
command = "false"

[[component]]
id = "c"
name = "c"
requires = ["b"]
[component.install]
kind = "command"
command = "false"
"#,
        )
        .unwrap();
        let registry = Registry::load(&root).unwrap();
        let runner = Runner {
            calls: Mutex::new(Vec::new()),
        };
        let summary = super::run(
            &registry,
            &runner,
            RunPlan::new(Phase::Install, vec!["c".into()], false),
            &EventSink::null(),
        )
        .unwrap();

        assert!(!summary.ok());
        assert_eq!(summary.refused, vec!["a"]);
        assert_eq!(summary.skipped_blocked, vec!["b", "c"]);
        assert_eq!(
            *runner
                .calls
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            vec![("a".into(), Phase::Detect), ("a".into(), Phase::Install)]
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn fix_missing_dependency_is_explicitly_unavailable_and_non_green() {
        struct Runner;
        impl HookRunner for Runner {
            fn run(
                &self,
                comp: &str,
                phase: Phase,
                _hook: &Hook,
                dry_run: bool,
                _sink: &EventSink,
            ) -> OpResult {
                OpResult {
                    component: comp.to_owned(),
                    phase,
                    status: OpStatus::Failed,
                    availability: None,
                    exit_code: Some(1),
                    duration_ms: 0,
                    message: String::new(),
                    dry_run,
                }
            }
        }

        let root = temp_root("executor-fix-unavailable");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("chain.toml"),
            r#"
[[component]]
id = "a"
name = "a"
[component.detect]
kind = "command"
command = "false"
[component.fix]
kind = "command"
command = "false"

[[component]]
id = "b"
name = "b"
requires = ["a"]
[component.fix]
kind = "command"
command = "false"

[[component]]
id = "c"
name = "c"
requires = ["b"]
[component.fix]
kind = "command"
command = "false"
"#,
        )
        .unwrap();
        let registry = Registry::load(&root).unwrap();
        let summary = super::run(
            &registry,
            &Runner,
            RunPlan::new(Phase::Fix, vec!["c".into()], false),
            &EventSink::null(),
        )
        .unwrap();

        assert!(!summary.ok());
        assert_eq!(summary.skipped_blocked, vec!["a", "b", "c"]);
        assert_eq!(summary.results[0].status, OpStatus::SkippedBlocked);
        assert_eq!(
            summary.results[0].availability,
            Some(ComponentAvailability::Unavailable)
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn healthy_install_skip_keeps_dependents_available() {
        struct Runner {
            calls: Mutex<Vec<(String, Phase)>>,
        }
        impl HookRunner for Runner {
            fn run(
                &self,
                comp: &str,
                phase: Phase,
                _hook: &Hook,
                dry_run: bool,
                _sink: &EventSink,
            ) -> OpResult {
                self.calls
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push((comp.to_owned(), phase));
                let status = match (comp, phase) {
                    ("healthy", Phase::Detect) => OpStatus::Ok,
                    ("dependent", Phase::Detect) => OpStatus::Failed,
                    ("dependent", Phase::Install) => OpStatus::Ok,
                    _ => OpStatus::Failed,
                };
                OpResult {
                    component: comp.to_owned(),
                    phase,
                    status,
                    availability: None,
                    exit_code: None,
                    duration_ms: 0,
                    message: String::new(),
                    dry_run,
                }
            }
        }

        let root = temp_root("executor-healthy-skip");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("healthy.toml"),
            r#"
[[component]]
id = "healthy"
name = "healthy"
[component.detect]
kind = "command"
command = "true"
[component.install]
kind = "command"
command = "true"

[[component]]
id = "dependent"
name = "dependent"
requires = ["healthy"]
[component.detect]
kind = "command"
command = "false"
[component.install]
kind = "command"
command = "true"
"#,
        )
        .unwrap();
        let registry = Registry::load(&root).unwrap();
        let runner = Runner {
            calls: Mutex::new(Vec::new()),
        };
        let summary = super::run(
            &registry,
            &runner,
            RunPlan::new(Phase::Install, vec!["dependent".into()], false),
            &EventSink::null(),
        )
        .unwrap();

        assert!(summary.ok());
        assert!(summary.skipped_blocked.is_empty());
        assert_eq!(summary.results[0].status, OpStatus::Skipped);
        assert_eq!(
            summary.results[0].availability,
            Some(ComponentAvailability::Healthy)
        );
        assert_eq!(summary.results[1].status, OpStatus::Ok);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn detected_component_with_failed_wiring_blocks_its_dependent() {
        let _env = crate::test_env_lock();
        struct Runner {
            calls: Mutex<Vec<(String, Phase)>>,
        }
        impl HookRunner for Runner {
            fn run(
                &self,
                comp: &str,
                phase: Phase,
                _hook: &Hook,
                dry_run: bool,
                _sink: &EventSink,
            ) -> OpResult {
                self.calls
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push((comp.to_owned(), phase));
                OpResult {
                    component: comp.to_owned(),
                    phase,
                    status: if (comp, phase) == ("healthy", Phase::Detect) {
                        OpStatus::Ok
                    } else {
                        OpStatus::Failed
                    },
                    availability: None,
                    exit_code: None,
                    duration_ms: 0,
                    message: String::new(),
                    dry_run,
                }
            }
        }

        let root = temp_root("executor-wiring-failure-blocks");
        let meta = root.join("meta");
        let manifest = root.join("manifest");
        std::fs::create_dir_all(&meta).unwrap();
        std::fs::create_dir_all(&manifest).unwrap();
        // OpenOptions::append cannot open a directory, deterministically forcing a wiring report
        // failure without relying on host permissions.
        std::fs::create_dir(meta.join(".bashrc")).unwrap();
        std::fs::write(
            manifest.join("wiring.toml"),
            r#"
[[component]]
id = "healthy"
name = "healthy"
[component.detect]
kind = "command"
command = "true"
[component.install]
kind = "command"
command = "true"
[[component.wiring.shell_rc]]
file = "$META_ROOT/.bashrc"
marker = "fixture"
content = "export FIXTURE=1"

[[component]]
id = "dependent"
name = "dependent"
requires = ["healthy"]
[component.install]
kind = "command"
command = "false"
"#,
        )
        .unwrap();
        let old_meta = std::env::var_os("META_ROOT");
        std::env::set_var("META_ROOT", &meta);
        let registry = Registry::load(&manifest).unwrap();
        let runner = Runner {
            calls: Mutex::new(Vec::new()),
        };
        let summary = super::run(
            &registry,
            &runner,
            RunPlan::new(Phase::Install, vec!["dependent".into()], false),
            &EventSink::null(),
        )
        .unwrap();

        assert!(!summary.ok());
        assert_eq!(summary.incomplete, vec!["healthy"]);
        assert_eq!(summary.skipped_blocked, vec!["dependent"]);
        assert_eq!(summary.results[0].status, OpStatus::Incomplete);
        assert_eq!(
            summary.results[0].availability,
            Some(ComponentAvailability::Unavailable)
        );
        assert_eq!(
            *runner
                .calls
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            vec![("healthy".into(), Phase::Detect)]
        );
        restore_env("META_ROOT", old_meta);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn no_install_hook_cannot_make_an_undetected_prerequisite_available() {
        struct Runner;
        impl HookRunner for Runner {
            fn run(
                &self,
                comp: &str,
                phase: Phase,
                _hook: &Hook,
                dry_run: bool,
                _sink: &EventSink,
            ) -> OpResult {
                OpResult {
                    component: comp.to_owned(),
                    phase,
                    status: OpStatus::Failed,
                    availability: None,
                    exit_code: Some(1),
                    duration_ms: 0,
                    message: String::new(),
                    dry_run,
                }
            }
        }

        let root = temp_root("executor-no-install-hook");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("no-hook.toml"),
            r#"
[[component]]
id = "a"
name = "a"
[component.detect]
kind = "command"
command = "false"

[[component]]
id = "b"
name = "b"
requires = ["a"]
[component.install]
kind = "command"
command = "false"
"#,
        )
        .unwrap();
        let registry = Registry::load(&root).unwrap();
        let summary = super::run(
            &registry,
            &Runner,
            RunPlan::new(Phase::Install, vec!["b".into()], false),
            &EventSink::null(),
        )
        .unwrap();

        assert!(!summary.ok());
        assert_eq!(summary.incomplete, vec!["a"]);
        assert_eq!(summary.skipped_blocked, vec!["b"]);
        assert_eq!(summary.results[0].status, OpStatus::Incomplete);
        assert_eq!(
            summary.results[0].availability,
            Some(ComponentAvailability::Unavailable)
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn engine_process_runner_repairs_forged_generation_and_blocks_on_failed_final_verify() {
        let _env = crate::test_env_lock();
        let root = temp_root("engine-process-runner-refresh");
        let meta = root.join("meta");
        let home = root.join("home");
        let manifest = root.join("manifest");
        let state = meta.join("state/current");
        let fake_bin = home.join(".nix-profile/toolbin");
        std::fs::create_dir_all(&state).unwrap();
        std::fs::create_dir_all(&manifest).unwrap();
        std::fs::create_dir_all(&fake_bin).unwrap();
        std::fs::create_dir_all(home.join(".nix-profile/bin")).unwrap();
        std::fs::create_dir_all(meta.join("var/lib/envctl")).unwrap();
        for command in ["cat", "mkdir", "mv", "rm", "touch"] {
            std::os::unix::fs::symlink(format!("/usr/bin/{command}"), fake_bin.join(command))
                .unwrap();
        }
        for leaf in ["secretctl", "secretctl.sha256", "secretctl.source.sha256"] {
            std::fs::write(state.join(leaf), format!("forged:{leaf}\n")).unwrap();
        }
        std::fs::write(meta.join("removable.bin"), "payload\n").unwrap();
        std::fs::write(meta.join("unit.loaded"), "loaded\n").unwrap();
        let systemctl = fake_bin.join("systemctl");
        std::fs::write(
            &systemctl,
            format!(
                r#"#!/bin/sh
set -eu
state='{}'
case " $* " in
  *" show --property=LoadState --value removable.service "*)
    if [ -f "$state" ]; then echo loaded; else echo not-found; fi ;;
  *" show --property=ActiveState --value removable.service "*)
    if [ -f "$state" ]; then echo active; else echo inactive; fi ;;
  *" disable --now removable.service "*) rm -f "$state" ;;
  *) exit 2 ;;
esac
"#,
                meta.join("unit.loaded").display()
            ),
        )
        .unwrap();
        std::fs::set_permissions(&systemctl, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::write(
            manifest.join("fixture.toml"),
            r#"
[[component]]
id = "sqld-generation"
name = "sqld generation fixture"
refresh_on_verify_failure = true
[component.detect]
kind = "command"
command = "bash"
args = ["-lc", "test -f \"$META_ROOT/state/current/secretctl\""]
[component.install]
kind = "script"
script = '''
set -euo pipefail
stage="$META_ROOT/state/.next"
rm -rf "$stage"
mkdir -p "$stage"
generation=trusted
[ "$(/usr/bin/cat "$META_ROOT/repair-success")" = 1 ] || generation=forged
for leaf in secretctl secretctl.sha256 secretctl.source.sha256; do
  printf '%s:%s\n' "$generation" "$leaf" >"$stage/$leaf"
done
rm -rf "$META_ROOT/state/current"
mv "$stage" "$META_ROOT/state/current"
'''
[component.verify]
kind = "script"
script = '''
set -euo pipefail
for leaf in secretctl secretctl.sha256 secretctl.source.sha256; do
  [ "$(cat "$META_ROOT/state/current/$leaf")" = "trusted:$leaf" ]
done
'''

[[component]]
id = "dependent"
name = "dependent"
requires = ["sqld-generation"]
[component.detect]
kind = "command"
command = "bash"
args = ["-lc", "test -f \"$META_ROOT/dependent.ready\""]
[component.install]
kind = "command"
command = "bash"
args = ["-lc", "touch \"$META_ROOT/dependent.ready\""]

[[component]]
id = "removable"
name = "removable"
[component.detect]
kind = "command"
command = "bash"
args = ["-lc", "test -f \"$META_ROOT/removable.bin\""]
[component.remove]
kind = "script"
script = '''
set -euo pipefail
load_state="$(systemctl --user show --property=LoadState --value removable.service)"
active_state="$(systemctl --user show --property=ActiveState --value removable.service)"
if [ "$load_state" = not-found ]; then
  [ "$active_state" = inactive ]
else
  systemctl --user disable --now removable.service
fi
rm -f "$META_ROOT/removable.bin"
'''
"#,
        )
        .unwrap();

        let old_meta = std::env::var_os("META_ROOT");
        let old_home = std::env::var_os("HOME");
        std::env::set_var("META_ROOT", &meta);
        std::env::set_var("HOME", &home);
        std::fs::write(meta.join("repair-success"), "1\n").unwrap();
        let engine = crate::Engine::load(manifest.clone()).unwrap();
        let repaired = engine
            .run(
                RunPlan::new(Phase::Install, vec!["dependent".into()], false),
                &EventSink::null(),
            )
            .unwrap();
        assert!(repaired.ok());
        assert!(meta.join("dependent.ready").is_file());
        assert_eq!(
            std::fs::read_to_string(state.join("secretctl")).unwrap(),
            "trusted:secretctl\n"
        );

        std::fs::remove_file(meta.join("dependent.ready")).unwrap();
        for leaf in ["secretctl", "secretctl.sha256", "secretctl.source.sha256"] {
            std::fs::write(state.join(leaf), format!("forged:{leaf}\n")).unwrap();
        }
        std::fs::write(meta.join("repair-success"), "0\n").unwrap();
        let blocked = engine
            .run(
                RunPlan::new(Phase::Install, vec!["dependent".into()], false),
                &EventSink::null(),
            )
            .unwrap();
        assert!(!blocked.ok());
        assert_eq!(blocked.incomplete, vec!["sqld-generation"]);
        assert_eq!(blocked.skipped_blocked, vec!["dependent"]);
        assert!(!meta.join("dependent.ready").exists());

        let first_remove = engine
            .run(
                RunPlan::new(Phase::Remove, vec!["removable".into()], false),
                &EventSink::null(),
            )
            .unwrap();
        assert!(first_remove.ok());
        let second_remove = engine
            .run(
                RunPlan::new(Phase::Remove, vec!["removable".into()], false),
                &EventSink::null(),
            )
            .unwrap();
        assert!(second_remove.ok());

        restore_env("META_ROOT", old_meta);
        restore_env("HOME", old_home);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn resolve_peer_routes_by_owner_and_honors_mode_override() {
        let owned = |mode| AddRepoSpec {
            git_url: "https://github.com/FlexNetOS/beads_rust".into(),
            mode,
            ..Default::default()
        };
        let foreign = |mode| AddRepoSpec {
            git_url: "https://github.com/someone/tool".into(),
            mode,
            ..Default::default()
        };
        // Auto: owned → peer, foreign → component.
        assert!(resolve_peer(&owned(AddRepoMode::Auto)));
        assert!(!resolve_peer(&foreign(AddRepoMode::Auto)));
        // Explicit override wins either way.
        assert!(resolve_peer(&foreign(AddRepoMode::Peer)));
        assert!(!resolve_peer(&owned(AddRepoMode::Component)));
        // Local-only (no remote) never auto-routes to peer.
        assert!(!resolve_peer(&AddRepoSpec {
            git_url: String::new(),
            local_path: Some("/tmp/x".into()),
            mode: AddRepoMode::Auto,
            ..Default::default()
        }));
    }

    #[test]
    fn wiring_present_probes_meta_root_bashrc_not_os_home() {
        // Mutates the process-global META_ROOT/HOME; serialize against every other
        // test that reads or writes env (e.g. catalog's scan()) — see test_env_lock.
        let _env = crate::test_env_lock();
        let root = temp_root("executor-wiring-meta-root");
        let meta = root.join("meta");
        let home = root.join("home");
        std::fs::create_dir_all(&meta).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(
            meta.join(".bashrc"),
            "# >>> BEGIN envctl PATH (added by envctl) >>>\n# <<< END envctl PATH <<<\n# >>> BEGIN meta toolchain path (added by envctl) >>>\neval \"$(envctl env --toolchains)\"\n# <<< END meta toolchain path <<<\n",
        )
        .unwrap();
        std::fs::write(home.join(".bashrc"), "").unwrap();

        let old_meta = std::env::var_os("META_ROOT");
        let old_home = std::env::var_os("HOME");
        std::env::set_var("META_ROOT", &meta);
        std::env::set_var("HOME", &home);

        let comp = Component {
            id: "bun".into(),
            name: "bun".into(),
            description: String::new(),
            requires: Vec::new(),
            gpu_required: false,
            destructive: false,
            refresh_on_verify_failure: false,
            detect: None,
            install: None,
            verify: None,
            fix: None,
            remove: None,
            wiring: Wiring {
                path_entries: vec!["$META_ROOT/usr/bin".into()],
                shell_rc: vec![ShellRcBlock {
                    file: "$META_ROOT/.bashrc".into(),
                    marker: "meta toolchain path".into(),
                    content: "eval \"$(envctl env --toolchains)\"".into(),
                }],
                ..Default::default()
            },
            guards: Vec::new(),
        };

        assert!(wiring_present(&comp));

        restore_env("META_ROOT", old_meta);
        restore_env("HOME", old_home);
        let _ = std::fs::remove_dir_all(root);
    }

    fn temp_root(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("envctl-{name}-{}-{nanos}", std::process::id()))
    }

    fn restore_env(key: &str, value: Option<std::ffi::OsString>) {
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}
