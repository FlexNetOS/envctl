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
    "rtk",
    "git-kb",
    "gitnexus",
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

#[derive(Debug, Default)]
struct ActiveProfileProvenance {
    targets: BTreeSet<(String, PathBuf)>,
}

impl ActiveProfileProvenance {
    /// Load the active real-home Nix profile only when the ownership chain has
    /// the Yazelix shape: `~/.nix-profile` -> the XDG profile selector -> the
    /// current numbered generation. A direct store link is deliberately not
    /// accepted because it bypasses the profile-owned frontdoor.
    fn from_home(home: &Path) -> Option<Self> {
        Self::from_home_with_store_root(home, Path::new("/nix/store"))
    }

    fn from_home_with_store_root(home: &Path, store_root: &Path) -> Option<Self> {
        let frontdoor = home.join(".nix-profile");
        let profiles = home.join(".local/state/nix/profiles");
        let selector = profiles.join("profile");
        let frontdoor_target = symlink_target(&frontdoor)?;
        if frontdoor_target != selector {
            return None;
        }

        let generation_link = symlink_target(&selector)?;
        let generation_name = generation_link.file_name()?.to_str()?;
        let generation_number = generation_name
            .strip_prefix("profile-")?
            .strip_suffix("-link")?;
        if generation_link.parent() != Some(profiles.as_path())
            || generation_number.is_empty()
            || !generation_number.chars().all(|ch| ch.is_ascii_digit())
        {
            return None;
        }

        let store_root = std::fs::canonicalize(store_root).ok()?;
        let generation = std::fs::canonicalize(&generation_link).ok()?;
        let generation_store_name = generation.file_name()?.to_str()?;
        if generation.parent() != Some(store_root.as_path())
            || !generation_store_name.ends_with("-profile")
            || generation_store_name == "-profile"
            || std::fs::canonicalize(&frontdoor).ok()? != generation
        {
            return None;
        }

        let mut targets = BTreeSet::new();
        for tool in LOCAL_META_TOOLS.iter().chain(CARGO_META_TOOLS.iter()) {
            for dir in ["bin", "toolbin"] {
                let entry = frontdoor.join(dir).join(tool);
                if let Ok(target) = std::fs::canonicalize(entry) {
                    if target.starts_with(&store_root) && target != store_root {
                        targets.insert(((*tool).to_string(), target));
                    }
                }
            }
        }
        Some(Self { targets })
    }

    fn from_env() -> Option<Self> {
        let home = std::env::var_os("ENVCTL_REAL_HOME")
            .filter(|value| !value.is_empty())
            .or_else(|| std::env::var_os("HOME"))?;
        Self::from_home(Path::new(&home))
    }

    fn owns(&self, tool: &str, resolved: &Path) -> bool {
        self.targets
            .contains(&(tool.to_string(), canonical_or_self(resolved.to_path_buf())))
    }
}

fn symlink_target(path: &Path) -> Option<PathBuf> {
    let target = std::fs::read_link(path).ok()?;
    if target.is_absolute() {
        Some(target)
    } else {
        Some(path.parent()?.join(target))
    }
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
    let expected_root = meta_root.display().to_string();
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut violations = Vec::new();
    let active_profile = scan_path.then(ActiveProfileProvenance::from_env).flatten();

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
                    let profile_owned = active_profile
                        .as_ref()
                        .is_some_and(|profile| profile.owns(tool, &resolved));
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

    // A unit at the right filename is still drifted when its rendered body
    // targets a retired META_ROOT.
    let systemd_ok = w
        .systemd_user
        .iter()
        .all(crate::wiring::systemd_user_present);

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

    shell_rc_ok && path_ok && systemd_ok && apt_ok && nix_ok && cdi_ok && alt_ok
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
    fn active_profile_provenance_accepts_only_current_generation_targets() {
        let root = temp_root("active-profile-current-generation");
        let home = root.join("home");
        let profiles = home.join(".local/state/nix/profiles");
        let current_generation = root.join("nix/store/current-profile");
        let current_package = root.join("nix/store/current-meta");
        let stale_package = root.join("nix/store/stale-meta");
        std::fs::create_dir_all(&profiles).unwrap();
        std::fs::create_dir_all(current_generation.join("bin")).unwrap();
        std::fs::create_dir_all(current_generation.join("toolbin")).unwrap();
        std::fs::create_dir_all(current_package.join("bin")).unwrap();
        std::fs::create_dir_all(stale_package.join("bin")).unwrap();
        std::fs::write(current_package.join("bin/meta"), b"current").unwrap();
        std::fs::write(stale_package.join("bin/meta"), b"stale").unwrap();
        symlink(
            current_package.join("bin/meta"),
            current_generation.join("bin/meta"),
        )
        .unwrap();
        symlink(
            current_package.join("bin/meta"),
            current_generation.join("toolbin/meta"),
        )
        .unwrap();
        symlink(&current_generation, profiles.join("profile-2-link")).unwrap();
        symlink("profile-2-link", profiles.join("profile")).unwrap();
        symlink(profiles.join("profile"), home.join(".nix-profile")).unwrap();

        let provenance =
            ActiveProfileProvenance::from_home_with_store_root(&home, &root.join("nix/store"))
                .unwrap();

        assert!(provenance.owns("meta", &current_package.join("bin/meta")));
        assert!(!provenance.owns("meta", &stale_package.join("bin/meta")));
        assert!(!provenance.owns("other", &current_package.join("bin/meta")));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn active_profile_provenance_rejects_direct_store_frontdoor() {
        let root = temp_root("active-profile-direct-store");
        let home = root.join("home");
        let generation = root.join("nix/store/profile");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(&generation).unwrap();
        symlink(&generation, home.join(".nix-profile")).unwrap();

        assert!(ActiveProfileProvenance::from_home(&home).is_none());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn active_profile_provenance_rejects_non_numeric_or_non_store_generations() {
        let root = temp_root("active-profile-invalid-generation");
        let home = root.join("home");
        let profiles = home.join(".local/state/nix/profiles");
        let store = root.join("nix/store");
        let outside = root.join("foreign/current-profile");
        std::fs::create_dir_all(&profiles).unwrap();
        std::fs::create_dir_all(&store).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        symlink(&outside, profiles.join("profile-current-link")).unwrap();
        symlink("profile-current-link", profiles.join("profile")).unwrap();
        symlink(profiles.join("profile"), home.join(".nix-profile")).unwrap();

        assert!(
            ActiveProfileProvenance::from_home_with_store_root(&home, &store).is_none(),
            "a named-but-nonnumeric generation must not establish profile provenance"
        );

        std::fs::remove_file(profiles.join("profile")).unwrap();
        std::fs::remove_file(profiles.join("profile-current-link")).unwrap();
        symlink(&outside, profiles.join("profile-9-link")).unwrap();
        symlink("profile-9-link", profiles.join("profile")).unwrap();
        assert!(
            ActiveProfileProvenance::from_home_with_store_root(&home, &store).is_none(),
            "a generation outside the Nix store must not establish profile provenance"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn active_profile_provenance_does_not_accept_a_tool_escaping_the_store() {
        let root = temp_root("active-profile-tool-store-escape");
        let home = root.join("home");
        let profiles = home.join(".local/state/nix/profiles");
        let store = root.join("nix/store");
        let generation = store.join("abc-profile");
        let foreign_tool = root.join("foreign/meta");
        std::fs::create_dir_all(generation.join("bin")).unwrap();
        std::fs::create_dir_all(foreign_tool.parent().unwrap()).unwrap();
        std::fs::create_dir_all(&profiles).unwrap();
        std::fs::write(&foreign_tool, b"foreign").unwrap();
        symlink(&foreign_tool, generation.join("bin/meta")).unwrap();
        symlink(&generation, profiles.join("profile-3-link")).unwrap();
        symlink("profile-3-link", profiles.join("profile")).unwrap();
        symlink(profiles.join("profile"), home.join(".nix-profile")).unwrap();

        let provenance = ActiveProfileProvenance::from_home_with_store_root(&home, &store).unwrap();
        assert!(!provenance.owns("meta", &foreign_tool));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn meta_boundary_normalizes_meta_managed_worktree_root() {
        let root = PathBuf::from("/home/user/Desktop/meta/.worktrees/task-0007");

        let normalized = normalize_meta_root(root);

        assert_eq!(normalized, PathBuf::from("/home/user/Desktop/meta"));
    }

    fn temp_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("envctl-{name}-{}-{nanos}", std::process::id()))
    }
}
