//! auto-detect: build an `EnvReport` read-only. NEVER writes.
//!
//! GPU detection is layered so it works on the documented first-boot reality
//! (software-rendered, no driver yet):
//!   Tier 0  PCI floor — scan /sys/bus/pci/devices for vendor 0x10de + display
//!           class 0x03xx. Authoritative count, works with NO driver.
//!   Tier 1  /proc/driver/nvidia/version — driver_loaded + version.
//!   Tier 2  modinfo/nvidia-smi/nvcc — open-module + names + CUDA/driver enrichment.
//! `software_rendered = pci_sees_nvidia && !driver_loaded` → the GUI shows a
//! "reboot to load nvidia-open" hint instead of a false "no GPU".
use crate::component::{HookRunner, Phase};
use crate::event::{Event, EventSink};
use crate::layout::MetaLayout;
use crate::model::{
    ComponentState, EnvReport, MetaBoundaryReport, MetaBoundaryViolation,
    MetaBoundaryViolationKind, OpStatus, Registry, ToolState,
};
use std::collections::BTreeSet;
use std::io::Read;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const PROBE_TIMEOUT: Duration = Duration::from_secs(10);

/// Tools probed for version (curated to ones that print-and-exit; avoids hangs).
const PROBE_TOOLS: &[&str] = &[
    "cargo",
    "rustc",
    "bun",
    "node",
    "nvcc",
    "nvidia-smi",
    "nix",
    "gh",
    "uv",
    "wasmer",
    "podman",
    "python3",
    "git",
    "curl",
];

pub fn run(reg: &Registry, runner: &dyn HookRunner, sink: &EventSink) -> anyhow::Result<EnvReport> {
    let mut report = EnvReport {
        generated_at: chrono::Utc::now().to_rfc3339(),
        ..Default::default()
    };

    // ---- host (sysinfo) ----
    let sys = sysinfo::System::new_all();
    report.kernel = sysinfo::System::kernel_version();
    report.os = sysinfo::System::long_os_version();
    report.cpu_model = sys.cpus().first().map(|c| c.brand().trim().to_string());
    report.cpu_threads = sys.cpus().len();
    report.mem_total_mb = sys.total_memory() / 1024 / 1024;

    // ---- GPU: Tier 0 PCI floor (driver-independent) ----
    report.gpu_count = pci_nvidia_count();
    report.gpu_present = report.gpu_count > 0;

    // ---- GPU: Tier 1 driver state ----
    report.driver_loaded = Path::new("/proc/driver/nvidia/version").exists();
    report.open_kernel_module = nvidia_open_module();
    report.software_rendered = report.gpu_present && !report.driver_loaded;

    // ---- GPU: Tier 2 enrichment (names/versions; best-effort) ----
    report.gpus = nvidia_smi_names();
    if report.gpus.is_empty() && report.gpu_present {
        report.gpus = lspci_nvidia_names();
    }
    report.driver_version = proc_nvidia_driver_version().or_else(nvidia_smi_driver_version);
    report.cuda_version = nvcc_cuda_version();

    // ---- installed tool versions (which + --version) ----
    for t in PROBE_TOOLS {
        let path = which::which(t).ok().map(|p| p.display().to_string());
        let version = path.as_ref().and_then(|_| tool_version(t));
        if path.is_some() {
            report.tools.push(ToolState {
                name: (*t).to_string(),
                path,
                version,
            });
        }
    }

    // ---- per-component detect (+ verify if detected) + wiring presence ----
    for comp in reg.ordered() {
        let mut st = ComponentState {
            id: comp.id.clone(),
            name: comp.name.clone(),
            detected: false,
            healthy: None,
            wiring_present: wiring_present(comp),
            note: String::new(),
        };
        if comp.gpu_required && !report.gpu_present {
            st.note = "skipped: no GPU".into();
            report.components.push(st);
            continue;
        }
        if let Some(h) = comp.detect.as_ref() {
            st.detected =
                runner.run(&comp.id, Phase::Detect, h, false, sink).status == OpStatus::Ok;
        }
        if st.detected {
            if let Some(h) = comp.verify.as_ref() {
                st.healthy = Some(
                    runner.run(&comp.id, Phase::Verify, h, false, sink).status == OpStatus::Ok,
                );
            }
        }
        report.components.push(st);
    }

    report.meta_boundary = meta_boundary_report();

    // Drift = diff(detected, desired) with suggested verbs.
    let drift = crate::drift::compute(&report, reg);
    report.drift = drift;

    sink.emit(Event::Report {
        report: report.clone(),
    });
    Ok(report)
}

// ---- GPU helpers ----

/// Count PCI functions with NVIDIA vendor 0x10de and a display class 0x03xx.
/// `pub(crate)` so the executor can resolve GPU presence for `RunContext`.
pub(crate) fn pci_nvidia_count() -> usize {
    let mut n = 0;
    if let Ok(rd) = std::fs::read_dir("/sys/bus/pci/devices") {
        for e in rd.flatten() {
            let vendor = std::fs::read_to_string(e.path().join("vendor")).unwrap_or_default();
            let class = std::fs::read_to_string(e.path().join("class")).unwrap_or_default();
            if vendor.trim() == "0x10de" && class.trim().starts_with("0x03") {
                n += 1;
            }
        }
    }
    n
}

fn nvidia_open_module() -> bool {
    // The open kernel modules report a free license; the proprietary one does not.
    run_capture("modinfo", &["-F", "license", "nvidia"])
        .map(|s| s.to_lowercase())
        .is_some_and(|s| s.contains("mit") || s.contains("gpl"))
}

fn nvidia_smi_names() -> Vec<String> {
    run_capture("nvidia-smi", &["--query-gpu=name", "--format=csv,noheader"])
        .map(|s| {
            s.lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn nvidia_smi_driver_version() -> Option<String> {
    run_capture(
        "nvidia-smi",
        &["--query-gpu=driver_version", "--format=csv,noheader"],
    )
    .and_then(|s| s.lines().next().map(|l| l.trim().to_string()))
    .filter(|s| !s.is_empty())
}

fn proc_nvidia_driver_version() -> Option<String> {
    let text = std::fs::read_to_string("/proc/driver/nvidia/version").ok()?;
    proc_nvidia_driver_version_from_str(&text)
}

fn proc_nvidia_driver_version_from_str(text: &str) -> Option<String> {
    text.lines()
        .flat_map(|line| line.split_whitespace())
        .map(|token| token.trim_matches(|c: char| !c.is_ascii_digit() && c != '.'))
        .find(|token| is_nvidia_version_token(token))
        .map(|token| token.to_string())
}

fn is_nvidia_version_token(token: &str) -> bool {
    let mut parts = token.split('.');
    let (Some(major), Some(minor), Some(patch), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    [major, minor, patch]
        .into_iter()
        .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
}

fn lspci_nvidia_names() -> Vec<String> {
    run_capture(
        "bash",
        &["-lc", "lspci | grep -iE 'vga|3d' | grep -i nvidia"],
    )
    .map(|s| {
        s.lines()
            .filter_map(|l| l.split_once(": ").map(|(_, n)| n.trim().to_string()))
            .collect()
    })
    .unwrap_or_default()
}

fn nvcc_cuda_version() -> Option<String> {
    let out = run_capture("nvcc", &["--version"])?;
    // line: "Cuda compilation tools, release 13.3, V13.3.xx"
    out.split("release ")
        .nth(1)
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
}

// ---- tool/version helpers ----

fn tool_version(tool: &str) -> Option<String> {
    let out = run_capture(tool, &["--version"]).or_else(|| run_capture(tool, &["-V"]))?;
    out.lines().next().map(|l| l.trim().to_string())
}

fn run_capture(cmd: &str, args: &[&str]) -> Option<String> {
    run_capture_timeout(cmd, args, PROBE_TIMEOUT)
}

pub(crate) fn run_capture_timeout(cmd: &str, args: &[&str], timeout: Duration) -> Option<String> {
    let mut command = Command::new(cmd);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    unsafe {
        command.pre_exec(|| {
            let _ = rustix::process::setsid();
            Ok(())
        });
    }
    let mut child = command.spawn().ok()?;
    let pid = child.id();
    let stdout = child.stdout.take()?;
    let stderr = child.stderr.take()?;
    let stdout_reader = std::thread::spawn(move || read_pipe(stdout));
    let stderr_reader = std::thread::spawn(move || read_pipe(stderr));

    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let out = stdout_reader.join().ok().unwrap_or_default();
                let err = stderr_reader.join().ok().unwrap_or_default();
                if !status.success() {
                    return None;
                }
                let text = if !out.trim().is_empty() { out } else { err };
                return if text.trim().is_empty() {
                    None
                } else {
                    Some(text)
                };
            }
            Ok(None) => {
                if Instant::now() >= deadline {
                    kill_group(pid);
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = stdout_reader.join();
                    let _ = stderr_reader.join();
                    return None;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(_) => {
                kill_group(pid);
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return None;
            }
        }
    }
}

fn read_pipe<R: Read>(mut pipe: R) -> String {
    let mut text = String::new();
    let _ = pipe.read_to_string(&mut text);
    text
}

fn kill_group(pid: u32) {
    if let Some(p) = rustix::process::Pid::from_raw(pid as i32) {
        // If the child never became a group leader (for example, if pre_exec
        // failed before setsid() took effect), fall back to killing the child
        // PID directly so a wedged probe cannot linger.
        if rustix::process::kill_process_group(p, rustix::process::Signal::Kill).is_err() {
            let _ = rustix::process::kill_process(p, rustix::process::Signal::Kill);
        }
    }
}

// ---- meta/FlexNetOS boundary helpers ----

const LOCAL_META_TOOLS: &[&str] = &[
    "meta",
    "meta-git",
    "meta-mcp",
    "meta-project",
    "loop",
    "lane",
    "vox",
    "icm",
    "grit",
    "envctl",
    "envctl-gui",
    "meta-dashboard",
];

const CARGO_META_TOOLS: &[&str] = &["weave", "grit", "secretctl", "secretd"];

fn meta_boundary_report() -> MetaBoundaryReport {
    let layout = MetaLayout::from_env_or_default();
    let local_bin = layout.bin();
    let cargo_bin = layout.legacy_toolchains().join("cargo/bin");
    let Some(meta_root) = resolve_meta_root() else {
        return MetaBoundaryReport {
            meta_root: None,
            local_bin: local_bin.display().to_string(),
            cargo_bin: cargo_bin.display().to_string(),
            violations: Vec::new(),
        };
    };
    meta_boundary_report_for(&meta_root, &local_bin, &cargo_bin, true)
}

fn resolve_meta_root() -> Option<PathBuf> {
    if let Ok(root) = std::env::var("META_ROOT") {
        if !root.trim().is_empty() {
            return Some(canonical_or_self(PathBuf::from(root)));
        }
    }
    let cwd = std::env::current_dir().ok()?;
    let meta_file = crate::dashboard::locate_meta_file(&cwd, None).ok()?;
    meta_file
        .parent()
        .map(|p| normalize_meta_root(canonical_or_self(p.to_path_buf())))
}

fn normalize_meta_root(root: PathBuf) -> PathBuf {
    let mut before_worktrees = PathBuf::new();
    for component in root.components() {
        if component.as_os_str() == ".worktrees" {
            return before_worktrees;
        }
        before_worktrees.push(component.as_os_str());
    }
    root
}

fn meta_boundary_report_for(
    meta_root: &Path,
    local_bin: &Path,
    cargo_bin: &Path,
    scan_path: bool,
) -> MetaBoundaryReport {
    let active_profile = active_nix_profile_root();
    let expected_root = active_profile
        .as_ref()
        .map(|profile| {
            format!(
                "META_ROOT {} or exact active Nix profile frontdoors {}/{{bin,toolbin}} and their current canonical exposure paths",
                meta_root.display(),
                profile.display()
            )
        })
        .unwrap_or_else(|| meta_root.display().to_string());
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut violations = Vec::new();

    for tool in LOCAL_META_TOOLS {
        inspect_bin_entry(
            tool,
            &local_bin.join(tool),
            meta_root,
            MetaBoundaryViolationKind::ForeignLocalBinFile,
            &expected_root,
            &mut seen,
            &mut violations,
        );
    }
    for tool in CARGO_META_TOOLS {
        inspect_bin_entry(
            tool,
            &cargo_bin.join(tool),
            meta_root,
            MetaBoundaryViolationKind::ForeignCargoBinFile,
            &expected_root,
            &mut seen,
            &mut violations,
        );
    }
    if scan_path {
        for tool in LOCAL_META_TOOLS.iter().chain(CARGO_META_TOOLS.iter()) {
            if let Ok(paths) = which::which_all(tool) {
                for path in paths {
                    let resolved = canonical_or_self(path.clone());
                    let profile_owned = active_profile.as_ref().is_some_and(|profile| {
                        active_profile_owns_tool_path(tool, &path, profile, Path::new("/nix/store"))
                    });
                    if !resolved.starts_with(meta_root) && !profile_owned {
                        push_violation(
                            tool,
                            &path,
                            &resolved,
                            &expected_root,
                            MetaBoundaryViolationKind::ForeignPathEntry,
                            &mut seen,
                            &mut violations,
                        );
                    }
                }
            }
        }
    }

    MetaBoundaryReport {
        meta_root: Some(expected_root),
        local_bin: local_bin.display().to_string(),
        cargo_bin: cargo_bin.display().to_string(),
        violations,
    }
}

fn active_nix_profile_root() -> Option<PathBuf> {
    std::env::var_os("ENVCTL_REAL_HOME")
        .filter(|home| !home.is_empty())
        .or_else(|| std::env::var_os("HOME").filter(|home| !home.is_empty()))
        .map(PathBuf::from)
        .map(|home| home.join(".nix-profile"))
}

/// Prove that `candidate` is an exact command exposure owned by the active
/// Yazelix Nix profile.  A profile path is accepted only when it is the exact
/// `bin/<tool>` or `toolbin/<tool>` exposure. A direct Nix-store path is
/// accepted only when it is the command path inside the canonical *current*
/// profile exposure directory. Comparing only the ultimate binary target is
/// insufficient: an old foundation generation can point at the same package.
/// This deliberately rejects user-bin, second-profile, old-generation, and
/// raw-package paths even when they ultimately reach the same store object.
fn active_profile_owns_tool_path(
    tool: &str,
    candidate: &Path,
    profile_root: &Path,
    store_root: &Path,
) -> bool {
    if tool.is_empty() || tool.contains('/') || tool.contains('\\') {
        return false;
    }

    let Ok(candidate_target) = std::fs::canonicalize(candidate) else {
        return false;
    };

    [profile_root.join("bin"), profile_root.join("toolbin")]
        .iter()
        .any(|profile_dir| {
            let lexical_frontdoor = profile_dir.join(tool);
            let Ok(current_store_dir) = std::fs::canonicalize(profile_dir) else {
                return false;
            };
            let current_store_frontdoor = current_store_dir.join(tool);
            if !current_store_frontdoor.starts_with(store_root) {
                return false;
            }
            let Ok(exposure_target) = std::fs::canonicalize(&current_store_frontdoor) else {
                return false;
            };
            exposure_target.starts_with(store_root)
                && candidate_target == exposure_target
                && (candidate == lexical_frontdoor || candidate == current_store_frontdoor)
        })
}

fn inspect_bin_entry(
    tool: &str,
    path: &Path,
    meta_root: &Path,
    file_kind: MetaBoundaryViolationKind,
    expected_root: &str,
    seen: &mut BTreeSet<(String, String)>,
    violations: &mut Vec<MetaBoundaryViolation>,
) {
    let Ok(md) = std::fs::symlink_metadata(path) else {
        return;
    };
    let resolved = canonical_or_self(path.to_path_buf());
    if md.file_type().is_symlink() {
        let kind = if resolved.starts_with(meta_root)
            && matches!(file_kind, MetaBoundaryViolationKind::ForeignLocalBinFile)
        {
            MetaBoundaryViolationKind::MetaFrontdoorSymlink
        } else if resolved.starts_with(meta_root) {
            return;
        } else {
            MetaBoundaryViolationKind::ForeignSymlinkTarget
        };
        push_violation(tool, path, &resolved, expected_root, kind, seen, violations);
        return;
    }
    if resolved.starts_with(meta_root) {
        return;
    }
    push_violation(
        tool,
        path,
        &resolved,
        expected_root,
        file_kind,
        seen,
        violations,
    );
}

fn push_violation(
    tool: &str,
    path: &Path,
    resolved: &Path,
    expected_root: &str,
    kind: MetaBoundaryViolationKind,
    seen: &mut BTreeSet<(String, String)>,
    violations: &mut Vec<MetaBoundaryViolation>,
) {
    let path_s = path.display().to_string();
    if !seen.insert((tool.to_string(), path_s.clone())) {
        return;
    }
    violations.push(MetaBoundaryViolation {
        tool: tool.to_string(),
        path: path_s,
        resolved_path: resolved.display().to_string(),
        expected_root: expected_root.to_string(),
        kind,
    });
}

fn canonical_or_self(path: PathBuf) -> PathBuf {
    std::fs::canonicalize(&path).unwrap_or(path)
}

/// True iff every wiring footprint this component owns is present on disk.
/// audit fix (minor #13): previously only shell_rc was inspected and an empty
/// shell_rc returned false outright, so a component wired only via path_entries/
/// apt_repos/nix_conf_lines/cdi_specs/alternatives reported wiring_present=false.
/// Now each declared family is conservatively probed (mirrors executor::
/// wiring_present); a component that declares no wiring at all reports true.
fn wiring_present(comp: &crate::component::Component) -> bool {
    let w = &comp.wiring;
    let layout = crate::layout::MetaLayout::from_env_or_default();

    let shell_rc_ok = w.shell_rc.iter().all(|blk| {
        let file = layout.expand_meta_path(&blk.file);
        // Suffix-agnostic: the wizard writes the same blocks as
        // "BEGIN <marker> (added by yazelix-setup.sh)"; envctl writes
        // "(added by envctl)". Match the marker regardless of who wrote it.
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
    // (mirrors wiring.rs apply targets: sources.list.d/<list_file>, NIX_CONF
    // line, cdi output file, alternative link).
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    #[test]
    fn run_capture_timeout_returns_output() {
        let out = run_capture_timeout("sh", &["-lc", "printf 'hello\\n'"], Duration::from_secs(1));
        assert_eq!(out.as_deref(), Some("hello\n"));
    }

    #[test]
    fn run_capture_timeout_times_out() {
        let started = Instant::now();
        let out = run_capture_timeout(
            "sh",
            &["-lc", "sleep 1; printf 'late\\n'"],
            Duration::from_millis(150),
        );
        assert!(out.is_none(), "expected timeout, got {out:?}");
        assert!(
            started.elapsed() < Duration::from_secs(1),
            "timeout helper should return promptly"
        );
    }

    #[test]
    fn run_capture_timeout_drains_noisy_output() {
        let out = run_capture_timeout(
            "sh",
            &[
                "-lc",
                "i=0; while [ \"$i\" -lt 20000 ]; do printf 'line%05d\\n' \"$i\"; i=$((i + 1)); done",
            ],
            Duration::from_secs(2),
        )
        .expect("noisy command should complete without filling the pipe");

        assert!(out.contains("line00000"), "missing first line");
        assert!(out.contains("line19999"), "missing final line");
    }

    #[test]
    fn proc_nvidia_driver_version_from_str_parses_version_token() {
        let sample =
            "NVRM version: NVIDIA UNIX Open Kernel Module for x86_64  610.54.03  Release Build";

        let parsed = proc_nvidia_driver_version_from_str(sample);

        assert_eq!(parsed.as_deref(), Some("610.54.03"));
    }

    #[test]
    fn proc_nvidia_driver_version_from_str_rejects_non_version_tokens() {
        let sample = "NVRM version: not-a-version";

        let parsed = proc_nvidia_driver_version_from_str(sample);

        assert_eq!(parsed, None);
    }

    #[test]
    fn meta_boundary_refuses_meta_usr_bin_symlink_into_meta_root() {
        let root = temp_root("refuses-meta-usr-bin-symlink");
        let meta = root.join("meta");
        let local_bin = meta.join("usr/bin");
        let cargo_bin = meta.join(".toolchains/cargo/bin");
        std::fs::create_dir_all(meta.join("target/release")).unwrap();
        std::fs::create_dir_all(&local_bin).unwrap();
        std::fs::create_dir_all(&cargo_bin).unwrap();
        let meta_bin = meta.join("target/release/meta");
        std::fs::write(&meta_bin, b"#!/bin/sh\n").unwrap();
        symlink(&meta_bin, local_bin.join("meta")).unwrap();

        let report = meta_boundary_report_for(&meta, &local_bin, &cargo_bin, false);

        assert_eq!(report.violations.len(), 1);
        assert_eq!(
            report.violations[0].kind,
            MetaBoundaryViolationKind::MetaFrontdoorSymlink
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn meta_boundary_accepts_meta_usr_bin_real_file_inside_meta_root() {
        let root = temp_root("accepts-meta-usr-bin-real-file");
        let meta = root.join("meta");
        let local_bin = meta.join("usr/bin");
        let cargo_bin = meta.join(".toolchains/cargo/bin");
        std::fs::create_dir_all(&meta).unwrap();
        std::fs::create_dir_all(&local_bin).unwrap();
        std::fs::create_dir_all(&cargo_bin).unwrap();
        std::fs::write(local_bin.join("meta"), b"meta-hosted frontdoor").unwrap();

        let report = meta_boundary_report_for(&meta, &local_bin, &cargo_bin, false);

        assert!(report.ok(), "{:?}", report.violations);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn meta_boundary_refuses_symlink_target_outside_meta_root() {
        let root = temp_root("refuses-foreign-symlink");
        let meta = root.join("meta");
        let outside = root.join("outside");
        let local_bin = meta.join("usr/bin");
        let cargo_bin = meta.join(".toolchains/cargo/bin");
        std::fs::create_dir_all(&meta).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::create_dir_all(&local_bin).unwrap();
        std::fs::create_dir_all(&cargo_bin).unwrap();
        let foreign = outside.join("meta");
        std::fs::write(&foreign, b"foreign").unwrap();
        symlink(&foreign, local_bin.join("meta")).unwrap();

        let report = meta_boundary_report_for(&meta, &local_bin, &cargo_bin, false);

        assert_eq!(report.violations.len(), 1);
        assert_eq!(
            report.violations[0].kind,
            MetaBoundaryViolationKind::ForeignSymlinkTarget
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn meta_boundary_normalizes_meta_managed_worktree_root() {
        let root = PathBuf::from("/home/user/Desktop/meta/.worktrees/task-0007");

        let normalized = normalize_meta_root(root);

        assert_eq!(normalized, PathBuf::from("/home/user/Desktop/meta"));
    }

    #[test]
    fn active_profile_ownership_accepts_only_lexical_and_current_store_frontdoors() {
        let root = temp_root("active-profile-ownership");
        let profile = root.join("home/.nix-profile");
        let store = root.join("nix/store");
        let raw_target = store.join("meta-package/bin/meta");
        let current_store_bin = store.join("current-foundation/toolbin/meta");
        let stale_store_bin = store.join("old-foundation/toolbin/meta");
        let profile_bin = profile.join("toolbin/meta");
        std::fs::create_dir_all(raw_target.parent().unwrap()).unwrap();
        std::fs::create_dir_all(current_store_bin.parent().unwrap()).unwrap();
        std::fs::create_dir_all(stale_store_bin.parent().unwrap()).unwrap();
        std::fs::create_dir_all(&profile).unwrap();
        std::fs::write(&raw_target, b"profile-owned meta").unwrap();
        symlink(&raw_target, &current_store_bin).unwrap();
        symlink(&raw_target, &stale_store_bin).unwrap();
        symlink(current_store_bin.parent().unwrap(), profile.join("toolbin")).unwrap();

        assert!(active_profile_owns_tool_path(
            "meta",
            &profile_bin,
            &profile,
            &store,
        ));
        assert!(active_profile_owns_tool_path(
            "meta",
            &current_store_bin,
            &profile,
            &store,
        ));
        assert!(!active_profile_owns_tool_path(
            "meta",
            &stale_store_bin,
            &profile,
            &store,
        ));
        assert!(!active_profile_owns_tool_path(
            "meta",
            &raw_target,
            &profile,
            &store,
        ));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn active_profile_ownership_refuses_user_bin_and_second_profile_shadows() {
        let root = temp_root("profile-shadow-refusal");
        let profile = root.join("home/.nix-profile");
        let store = root.join("nix/store");
        let store_bin = store.join("profile-generation/bin/meta");
        let profile_bin = profile.join("bin/meta");
        let user_shadow = root.join("home/.local/bin/meta");
        let second_profile = root.join("home/.local/state/nix/profiles/second/bin/meta");
        std::fs::create_dir_all(store_bin.parent().unwrap()).unwrap();
        std::fs::create_dir_all(profile_bin.parent().unwrap()).unwrap();
        std::fs::create_dir_all(user_shadow.parent().unwrap()).unwrap();
        std::fs::create_dir_all(second_profile.parent().unwrap()).unwrap();
        std::fs::write(&store_bin, b"profile-owned meta").unwrap();
        symlink(&store_bin, &profile_bin).unwrap();
        symlink(&store_bin, &user_shadow).unwrap();
        symlink(&store_bin, &second_profile).unwrap();

        assert!(!active_profile_owns_tool_path(
            "meta",
            &user_shadow,
            &profile,
            &store,
        ));
        assert!(!active_profile_owns_tool_path(
            "meta",
            &second_profile,
            &profile,
            &store,
        ));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn active_profile_ownership_refuses_stale_store_and_missing_profile_proof() {
        let root = temp_root("profile-proof-refusal");
        let profile = root.join("home/.nix-profile");
        let missing_profile = root.join("missing/.nix-profile");
        let store = root.join("nix/store");
        let active_store_bin = store.join("active-generation/bin/meta");
        let stale_store_bin = store.join("stale-generation/bin/meta");
        let profile_bin = profile.join("bin/meta");
        std::fs::create_dir_all(active_store_bin.parent().unwrap()).unwrap();
        std::fs::create_dir_all(stale_store_bin.parent().unwrap()).unwrap();
        std::fs::create_dir_all(profile_bin.parent().unwrap()).unwrap();
        std::fs::write(&active_store_bin, b"active").unwrap();
        std::fs::write(&stale_store_bin, b"stale").unwrap();
        symlink(&active_store_bin, &profile_bin).unwrap();

        assert!(!active_profile_owns_tool_path(
            "meta",
            &stale_store_bin,
            &profile,
            &store,
        ));
        assert!(!active_profile_owns_tool_path(
            "meta",
            &active_store_bin,
            &missing_profile,
            &store,
        ));

        let _ = std::fs::remove_dir_all(root);
    }

    fn temp_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("envctl-{name}-{}-{nanos}", std::process::id()))
    }
}
