//! auto-detect: build an `EnvReport` read-only. NEVER writes.
//!
//! GPU detection is layered so it works on the documented first-boot reality
//! (software-rendered, no driver yet):
//!   Tier 0  PCI floor — scan /sys/bus/pci/devices for vendor 0x10de + display
//!           class 0x03xx. Authoritative count, works with NO driver.
//!   Tier 1  /proc/driver/nvidia/version — driver_loaded + version.
//!   Tier 2  nvidia-smi / nvcc — names, driver/CUDA versions (enrichment only).
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
use std::path::{Path, PathBuf};
use std::process::Command;

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
    report.driver_version = nvidia_smi_driver_version();
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
    if let Ok(out) = Command::new("modinfo")
        .args(["-F", "license", "nvidia"])
        .output()
    {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).to_lowercase();
            return s.contains("mit") || s.contains("gpl");
        }
    }
    false
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
    let out = Command::new(cmd).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).to_string();
    if s.trim().is_empty() {
        None
    } else {
        Some(s)
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
    let expected_root = meta_root.display().to_string();
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
                    if !resolved.starts_with(meta_root) {
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
    if resolved.starts_with(meta_root) {
        return;
    }
    let kind = if md.file_type().is_symlink() {
        MetaBoundaryViolationKind::ForeignSymlinkTarget
    } else {
        file_kind
    };
    push_violation(tool, path, &resolved, expected_root, kind, seen, violations);
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
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn meta_boundary_accepts_meta_local_bin_symlink_into_meta_root() {
        let root = temp_root("accepts-meta-local-bin-symlink");
        let meta = root.join("meta");
        let local_bin = meta.join(".local/bin");
        let cargo_bin = meta.join(".toolchains/cargo/bin");
        std::fs::create_dir_all(meta.join("target/release")).unwrap();
        std::fs::create_dir_all(&local_bin).unwrap();
        std::fs::create_dir_all(&cargo_bin).unwrap();
        let meta_bin = meta.join("target/release/meta");
        std::fs::write(&meta_bin, b"#!/bin/sh\n").unwrap();
        symlink(&meta_bin, local_bin.join("meta")).unwrap();

        let report = meta_boundary_report_for(&meta, &local_bin, &cargo_bin, false);

        assert!(report.ok(), "{:?}", report.violations);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn meta_boundary_accepts_meta_local_bin_real_file_inside_meta_root() {
        let root = temp_root("accepts-meta-local-bin-real-file");
        let meta = root.join("meta");
        let local_bin = meta.join(".local/bin");
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
        let local_bin = meta.join(".local/bin");
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

    fn temp_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("envctl-{name}-{}-{nanos}", std::process::id()))
    }
}
