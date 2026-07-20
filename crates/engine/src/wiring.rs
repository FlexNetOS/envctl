//! Idempotent `apply()`/`revert()` of declarative `Wiring`.
//!
//! Discipline (ubuntu-boot-repair.sh gold standard):
//!   * dry-run is handled by the caller (the executor skips us on dry_run);
//!   * EVERY edit backs up before clobber (timestamped `.bak.<epoch>`);
//!   * we excise ONLY the lines/files/alternatives the engine itself owns —
//!     foreign edits (e.g. wasmer's own ~/.bashrc PATH block) are DETECTED AND
//!     REPORTED, never blind-excised;
//!   * system-scope edits go through `sudo` (the run pre-warms it);
//!   * apply order is keyring-before-list / write-before-enable; revert order is
//!     list-before-keyring / disable-before-remove / restart-after-edit;
//!   * `data_paths` are NEVER touched without `--purge` + a fail-closed UUID
//!     re-verify, and even then are renamed to trash, never `rm -rf`.
//!
//! apply()/revert() return a `WiringReport` (advisory notes + per-kind failures)
//! so the executor can surface what happened without aborting the run.
use crate::error::RunContext;
use crate::layout::MetaLayout;
use crate::model::{
    Alternative, AptRepo, CdiSpec, DesktopEntry, NixConfLine, ResetGates, ShellRcBlock,
    SystemdUnit, Wiring,
};
#[cfg(unix)]
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Clone, Debug, Default)]
pub struct WiringReport {
    pub notes: Vec<String>,
    pub failures: Vec<(String, String)>,
}
impl WiringReport {
    fn note(&mut self, s: impl Into<String>) {
        self.notes.push(s.into());
    }
    fn fail(&mut self, kind: &str, e: impl std::fmt::Display) {
        self.failures.push((kind.into(), e.to_string()));
    }
}

const NIX_CONF: &str = "/etc/nix/nix.custom.conf";
const SOURCES_D: &str = "/etc/apt/sources.list.d";

// ---------------------------------------------------------------- apply -------

pub fn apply(w: &Wiring) -> WiringReport {
    let mut rep = WiringReport::default();

    if let Some(blk) = path_export_block(w) {
        if let Err(e) = apply_shell_rc(&blk) {
            rep.fail("path_entries", e);
        }
    }
    for blk in &w.shell_rc {
        if let Err(e) = apply_shell_rc(blk) {
            rep.fail("shell_rc", e);
        }
    }
    for d in &w.desktop_entries {
        if let Err(e) = apply_desktop(d) {
            rep.fail("desktop_entry", e);
        }
    }
    for u in &w.systemd_user {
        if let Err(e) = apply_systemd(u) {
            rep.fail("systemd_user", e);
        }
    }
    // keyring-before-list; one debounced `apt-get update` after the loop.
    let mut apt_dirty = false;
    for r in &w.apt_repos {
        match apply_apt_repo(r, &mut rep) {
            Ok(changed) => apt_dirty |= changed && r.apt_update,
            Err(e) => rep.fail("apt_repo", e),
        }
    }
    if apt_dirty {
        let _ = sudo(&["apt-get", "update", "-y"]);
    }
    // nix lines, then restart the daemon ONCE iff anything actually changed.
    let mut touched_nix = false;
    for l in &w.nix_conf_lines {
        match apply_nix_line(l) {
            Ok(changed) => touched_nix |= changed,
            Err(e) => rep.fail("nix_conf_line", e),
        }
    }
    if touched_nix {
        restart_nix_daemon(&mut rep);
    }
    for c in &w.cdi_specs {
        if let Err(e) = apply_cdi(c) {
            rep.fail("cdi_spec", e);
        }
    }
    for a in &w.alternatives {
        if let Err(e) = apply_alternative(a, &mut rep) {
            rep.fail("alternative", e);
        }
    }
    rep
}

// --------------------------------------------------------------- revert -------

pub fn revert(w: &Wiring, gates: &ResetGates, ctx: &RunContext) -> WiringReport {
    let mut rep = WiringReport::default();

    if let Some(blk) = path_export_block(w) {
        if let Err(e) = revert_shell_rc(&blk, &mut rep) {
            rep.fail("path_entries", e);
        }
    }
    for blk in &w.shell_rc {
        if let Err(e) = revert_shell_rc(blk, &mut rep) {
            rep.fail("shell_rc", e);
        }
    }
    for d in &w.desktop_entries {
        if let Err(e) = revert_desktop(d) {
            rep.fail("desktop_entry", e);
        }
    }
    for u in &w.systemd_user {
        if let Err(e) = revert_systemd(u) {
            rep.fail("systemd_user", e);
        }
    }
    // ORDER IS LOAD-BEARING: .list FIRST, then keyring; stop-on-failure per repo.
    let mut apt_dirty = false;
    for r in &w.apt_repos {
        match revert_apt_repo(r, &mut rep) {
            Ok(changed) => apt_dirty |= changed && r.apt_update,
            Err(e) => rep.fail("apt_repo", e),
        }
    }
    if apt_dirty {
        let _ = sudo(&["apt-get", "update", "-y"]);
    }
    // owned nix lines, THEN one daemon restart.
    let mut touched_nix = false;
    for l in &w.nix_conf_lines {
        match revert_nix_line(l, &mut rep) {
            Ok(changed) => touched_nix |= changed,
            Err(e) => rep.fail("nix_conf_line", e),
        }
    }
    if touched_nix {
        restart_nix_daemon(&mut rep);
    }
    for c in &w.cdi_specs {
        if let Err(e) = revert_cdi(c, &mut rep) {
            rep.fail("cdi_spec", e);
        }
    }
    for a in &w.alternatives {
        if let Err(e) = revert_alternative(a, &mut rep) {
            rep.fail("alternative", e);
        }
    }

    // config_paths: removed (recoverably) unless --keep-config.
    for cp in &w.config_paths {
        let p = expand_tilde(&cp.path);
        if gates.keep_config {
            rep.note(format!("kept config {} (--keep-config)", cp.path));
            continue;
        }
        if Path::new(&p).exists() {
            let trash = format!("{p}.bak.{}", now_epoch());
            match std::fs::rename(&p, &trash) {
                Ok(()) => rep.note(format!("removed config {} -> {trash}", cp.path)),
                Err(e) => rep.fail("config_path", e),
            }
        }
    }
    // data_paths: NEVER touched without --purge; with --purge, fail-closed UUID
    // re-verify, then rename-to-trash (recoverable), never rm -rf.
    for dp in &w.data_paths {
        if !gates.purge {
            rep.note(format!(
                "left user data intact (would purge with --purge): {}",
                dp.path
            ));
            continue;
        }
        let Some(uuid) = dp.uuid.as_deref() else {
            rep.fail(
                "data_path",
                format!("cannot purge {}: no uuid declared", dp.path),
            );
            continue;
        };
        if let Some(reason) = crate::guard::verify_path_uuid(&dp.path, uuid, ctx) {
            rep.fail(
                "data_path",
                format!("refused purge of {}: {reason}", dp.path),
            );
            continue;
        }
        let p = expand_tilde(&dp.path);
        // AUDIT-FIX: the UUID check resolves symlinks, but rename operates on the
        // link itself — refuse a symlink so we never purge via a redirected path.
        if std::fs::symlink_metadata(&p)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
        {
            rep.fail(
                "data_path",
                format!("refusing to purge {}: it is a symlink", dp.path),
            );
            continue;
        }
        let trash = format!("{p}.envctl-trash.{}", now_epoch());
        match std::fs::rename(&p, &trash) {
            Ok(()) => rep.note(format!("purged {} -> {trash}", dp.path)),
            Err(e) => rep.fail("data_path", e),
        }
    }
    rep
}

// ============================== shell-rc (+ PATH) =============================

fn markers(marker: &str) -> (String, String) {
    (
        format!("# >>> BEGIN {marker} (added by envctl) >>>"),
        format!("# <<< END {marker} <<<"),
    )
}

fn expand_tilde(p: &str) -> String {
    MetaLayout::from_env_or_default().expand_meta_path(p)
}

/// PATH entries realized as ONE owned, marker'd export block so reset can excise
/// them cleanly. Marker "envctl PATH" is engine-private.
fn path_export_block(w: &Wiring) -> Option<ShellRcBlock> {
    if w.path_entries.is_empty() {
        return None;
    }
    let mut content = String::new();
    for dir in &w.path_entries {
        content.push_str(&format!(
            "case \":$PATH:\" in *\":{dir}:\"*) ;; *) export PATH=\"{dir}:$PATH\";; esac\n"
        ));
    }
    Some(ShellRcBlock {
        file: "$META_ROOT/.bashrc".into(),
        marker: "envctl PATH".into(),
        content,
    })
}

fn apply_shell_rc(blk: &ShellRcBlock) -> std::io::Result<()> {
    let file = expand_tilde(&blk.file);
    let (begin, end) = markers(&blk.marker);
    let existing = std::fs::read_to_string(&file).unwrap_or_default();
    if existing.contains(&begin) {
        return Ok(());
    }
    if Path::new(&file).exists() {
        let _ = std::fs::copy(&file, format!("{file}.bak.{}", now_epoch()));
    }
    let block = format!("\n{begin}\n{}\n{end}\n", blk.content.trim_end());
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file)?;
    f.write_all(block.as_bytes())
}

fn revert_shell_rc(blk: &ShellRcBlock, rep: &mut WiringReport) -> std::io::Result<()> {
    let file = expand_tilde(&blk.file);
    let (begin, end) = markers(&blk.marker);
    let Ok(text) = std::fs::read_to_string(&file) else {
        return Ok(());
    };
    if !text.contains(&begin) {
        // We never wrote this block. If a FOREIGN PATH edit exists, report it but
        // NEVER touch it (e.g. wasmer's own installer block).
        if blk.marker.contains("PATH") && foreign_path_line(&text) {
            rep.note(format!(
                "left a foreign PATH edit in {file} intact (not envctl-owned)"
            ));
        }
        return Ok(());
    }
    // AUDIT-FIX (blocker): only excise a properly PAIRED BEGIN..END. If the END
    // marker is missing after BEGIN (truncated/edited/crash-mid-write), do NOT
    // delete to EOF — leave the file untouched and report the failure.
    let bi = text.find(&begin).unwrap();
    if !text[bi..].contains(&end) {
        rep.fail(
            "shell_rc",
            format!(
                "unterminated envctl block '{}' in {file} — left untouched (excise it by hand)",
                blk.marker
            ),
        );
        return Ok(());
    }
    // AUDIT-FIX (minor): excise exactly the FIRST paired BEGIN..END span by byte
    // index instead of a stateful line scan, which could delete user content
    // between two BEGIN markers (or a foreign line containing the marker
    // substring) or leave a half-block. If a second BEGIN marker is present we
    // can't tell which span is ours, so bail and leave the file untouched.
    let end_marker_idx = bi + text[bi..].find(&end).unwrap();
    let mut span_end = end_marker_idx + end.len();
    // also swallow the marker line's trailing newline so we don't leave a blank.
    if text[span_end..].starts_with('\n') {
        span_end += 1;
    }
    if text[span_end..].contains(&begin) {
        rep.fail(
            "shell_rc",
            format!(
                "multiple envctl blocks '{}' in {file} — left untouched (excise by hand)",
                blk.marker
            ),
        );
        return Ok(());
    }
    let _ = std::fs::copy(&file, format!("{file}.bak.{}", now_epoch()));
    let mut out = String::with_capacity(text.len());
    out.push_str(&text[..bi]);
    out.push_str(&text[span_end..]);
    std::fs::write(&file, out)?;
    rep.note(format!(
        "excised envctl-owned block '{}' from {file}",
        blk.marker
    ));
    Ok(())
}

/// Conservative heuristic: a surviving `export PATH=...:$PATH` line outside any
/// envctl marker block => a foreign PATH edit worth reporting (never excised).
fn foreign_path_line(text: &str) -> bool {
    let mut in_block = false;
    for line in text.lines() {
        let t = line.trim_start();
        // anchored marker matching (audit fix) — a stray substring won't fool us.
        if t.starts_with("# >>> BEGIN ") && t.contains("(added by envctl)") {
            in_block = true;
        } else if t.starts_with("# <<< END ") {
            in_block = false;
        } else if !in_block
            && (t.starts_with("export PATH=") || t.starts_with("PATH="))
            && (t.contains(":$PATH") || t.contains(":${PATH}"))
        {
            return true;
        }
    }
    false
}

// ============================== desktop entries ==============================

fn xdg_autostart(filename: &str) -> String {
    let base = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", home()));
    format!("{base}/autostart/{filename}")
}

fn apply_desktop(d: &DesktopEntry) -> std::io::Result<()> {
    let path = xdg_autostart(&d.filename);
    if let Some(parent) = Path::new(&path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    if Path::new(&path).exists() {
        if std::fs::read_to_string(&path).unwrap_or_default() == d.content {
            return Ok(());
        }
        let _ = std::fs::copy(&path, format!("{path}.bak.{}", now_epoch()));
    }
    std::fs::write(&path, &d.content)
}

fn revert_desktop(d: &DesktopEntry) -> std::io::Result<()> {
    let path = xdg_autostart(&d.filename);
    if !Path::new(&path).exists() {
        return Ok(()); // already gone (e.g. one_shot self-disabled)
    }
    let _ = std::fs::copy(&path, format!("{path}.bak.{}", now_epoch()));
    std::fs::remove_file(&path)
}

// ============================== systemd --user ==============================

pub(crate) fn systemd_user_present(u: &SystemdUnit) -> bool {
    let Ok(layout) = MetaLayout::from_env_required() else {
        return false;
    };
    let Ok(content) = render_systemd_user_content(&layout, &u.content) else {
        return false;
    };
    let Ok(paths) = systemd_unit_paths(&layout, u) else {
        return false;
    };
    systemd_user_present_at(&paths.canonical, &content)
        && owned_systemd_bridge_present(&paths.bridge, &paths.canonical)
        && systemd_unit_is_discovered(u, &paths).unwrap_or(false)
}

fn systemd_user_present_at(path: &Path, expected: &str) -> bool {
    std::fs::read_to_string(path)
        .map(|actual| actual == expected)
        .unwrap_or(false)
}

fn apply_systemd(u: &SystemdUnit) -> std::io::Result<()> {
    let layout = MetaLayout::from_env_required()?;
    let content = render_systemd_user_content(&layout, &u.content)?;
    let paths = systemd_unit_paths(&layout, u)?;

    // Validate both ownership surfaces before the first mkdir/write/systemctl
    // mutation. In particular, never let an existing bridge redirect a write
    // into a tracked home-tree projection or another unit owner.
    ensure_regular_or_absent(&paths.canonical)?;
    ensure_owned_systemd_bridge_or_absent(&paths.bridge, &paths.canonical)?;
    let canonical_before = match std::fs::read(&paths.canonical) {
        Ok(content) => Some(content),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    let bridge_was_present = std::fs::symlink_metadata(&paths.bridge).is_ok();
    let runtime_before =
        snapshot_systemd_runtime_state(u, &paths, canonical_before.is_some(), bridge_was_present)?;

    let canonical_dir = paths.canonical.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "systemd user unit path has no parent",
        )
    })?;
    let bridge_dir = paths.bridge.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "systemd user bridge path has no parent",
        )
    })?;
    std::fs::create_dir_all(canonical_dir)?;
    std::fs::create_dir_all(bridge_dir)?;

    let changed = canonical_before.as_deref() != Some(content.as_bytes());
    let manager_may_have_changed = std::cell::Cell::new(false);
    let result = (|| {
        if changed {
            if canonical_before.is_some() {
                let backup =
                    PathBuf::from(format!("{}.bak.{}", paths.canonical.display(), now_epoch()));
                std::fs::copy(&paths.canonical, backup)?;
            }
            write_systemd_unit_atomically(&paths.canonical, content.as_bytes())?;
        }
        let bridge_created = create_owned_systemd_bridge(&paths.bridge, &paths.canonical)?;
        if changed || bridge_created {
            // A failed reload may still have partially changed manager state.
            manager_may_have_changed.set(true);
            run_systemctl(&["--user", "daemon-reload"])?;
        }
        systemd_unit_is_discovered(u, &paths)?;
        if changed && runtime_before.is_some_and(|state| state.active) {
            manager_may_have_changed.set(true);
            run_systemctl(&["--user", "restart", &u.name])?;
        }
        if u.enable {
            manager_may_have_changed.set(true);
            run_systemctl(&["--user", "enable", "--now", &u.name])?;
        }
        Ok(())
    })();

    match result {
        Ok(()) => Ok(()),
        Err(error) => Err(systemd_rollback_error(
            error,
            restore_systemd_transaction(
                u,
                &paths,
                canonical_before.as_deref(),
                bridge_was_present,
                runtime_before,
                Some(content.as_bytes()),
                manager_may_have_changed.get(),
            ),
        )),
    }
}

#[derive(Debug)]
struct SystemdUserPaths {
    canonical: PathBuf,
    bridge: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SystemdRuntimeState {
    enabled: bool,
    active: bool,
}

fn systemd_unit_paths(layout: &MetaLayout, u: &SystemdUnit) -> std::io::Result<SystemdUserPaths> {
    let name = Path::new(&u.name);
    let mut components = name.components();
    if u.name.is_empty()
        || !matches!(components.next(), Some(std::path::Component::Normal(_)))
        || components.next().is_some()
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "systemd user unit name '{}' must be a single path component",
                u.name
            ),
        ));
    }

    let canonical = layout.systemd_user_dir().join(name);
    let real_config = real_user_xdg_config_home(layout)?;
    let bridge = real_config.join("systemd/user").join(name);
    if bridge == canonical {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "the active systemd user search path must be distinct from META_ROOT; set ENVCTL_REAL_HOME or ENVCTL_REAL_XDG_CONFIG_HOME",
        ));
    }
    Ok(SystemdUserPaths { canonical, bridge })
}

fn real_user_xdg_config_home(layout: &MetaLayout) -> std::io::Result<PathBuf> {
    let explicit = std::env::var_os("ENVCTL_REAL_XDG_CONFIG_HOME").filter(|path| !path.is_empty());
    let ambient = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .filter(|path| path != &layout.xdg_config_home());
    let config = if let Some(path) = explicit.map(PathBuf::from).or(ambient) {
        path
    } else {
        let home = std::env::var_os("ENVCTL_REAL_HOME")
            .filter(|path| !path.is_empty())
            .or_else(|| std::env::var_os("HOME").filter(|path| !path.is_empty()))
            .map(PathBuf::from)
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "ENVCTL_REAL_HOME or HOME is required for the active systemd user bridge",
                )
            })?;
        home.join(".config")
    };
    if !config.is_absolute() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "the active systemd user XDG config path must be absolute",
        ));
    }
    Ok(config)
}

fn ensure_regular_or_absent(path: &Path) -> std::io::Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "refusing systemd user unit symlink projection: {}",
                path.display()
            ),
        )),
        Ok(metadata) if !metadata.is_file() => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "refusing non-file systemd user unit target: {}",
                path.display()
            ),
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn ensure_owned_systemd_bridge_or_absent(bridge: &Path, canonical: &Path) -> std::io::Result<()> {
    match std::fs::symlink_metadata(bridge) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            let target = std::fs::read_link(bridge)?;
            if target == canonical {
                Ok(())
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "refusing foreign systemd user bridge {} -> {} (expected {})",
                        bridge.display(),
                        target.display(),
                        canonical.display()
                    ),
                ))
            }
        }
        Ok(_) => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "refusing non-symlink systemd user bridge: {}",
                bridge.display()
            ),
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn owned_systemd_bridge_present(bridge: &Path, canonical: &Path) -> bool {
    ensure_owned_systemd_bridge_or_absent(bridge, canonical).is_ok()
        && std::fs::symlink_metadata(bridge)
            .map(|metadata| metadata.file_type().is_symlink())
            .unwrap_or(false)
}

#[cfg(unix)]
fn create_owned_systemd_bridge(bridge: &Path, canonical: &Path) -> std::io::Result<bool> {
    if std::fs::symlink_metadata(bridge).is_ok() {
        ensure_owned_systemd_bridge_or_absent(bridge, canonical)?;
        return Ok(false);
    }
    match std::os::unix::fs::symlink(canonical, bridge) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            ensure_owned_systemd_bridge_or_absent(bridge, canonical)?;
            Ok(false)
        }
        Err(error) => Err(error),
    }
}

fn systemd_unit_is_discovered(u: &SystemdUnit, paths: &SystemdUserPaths) -> std::io::Result<bool> {
    let output = trusted_systemctl_command()
        .args([
            "--user",
            "show",
            "--property=FragmentPath",
            "--value",
            &u.name,
        ])
        .output()?;
    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "systemctl --user show FragmentPath {} exited with status {}",
            u.name, output.status
        )));
    }
    let fragment = std::str::from_utf8(&output.stdout)
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "systemctl returned a non-UTF-8 FragmentPath",
            )
        })?
        .trim();
    let fragment = Path::new(fragment);
    if fragment == paths.canonical || fragment == paths.bridge {
        Ok(true)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "systemd user manager resolved {} from {} instead of the envctl-owned {} bridge to {}",
                u.name,
                fragment.display(),
                paths.bridge.display(),
                paths.canonical.display()
            ),
        ))
    }
}

fn systemd_unit_load_state(u: &SystemdUnit) -> std::io::Result<String> {
    let output = trusted_systemctl_command()
        .args(["--user", "show", "--property=LoadState", "--value", &u.name])
        .output()?;
    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "systemctl --user show LoadState {} exited with status {}",
            u.name, output.status
        )));
    }
    std::str::from_utf8(&output.stdout)
        .map(str::trim)
        .map(str::to_owned)
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "systemctl returned a non-UTF-8 LoadState",
            )
        })
}

fn snapshot_systemd_runtime_state(
    u: &SystemdUnit,
    paths: &SystemdUserPaths,
    canonical_present: bool,
    bridge_present: bool,
) -> std::io::Result<Option<SystemdRuntimeState>> {
    match systemd_unit_load_state(u)?.as_str() {
        "not-found" if !bridge_present => Ok(None),
        "not-found" => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "refusing to mutate {} because its envctl-owned discovery bridge exists but systemd reports LoadState=not-found",
                u.name
            ),
        )),
        "loaded" if canonical_present && bridge_present => {
            systemd_unit_is_discovered(u, paths)?;
            Ok(Some(SystemdRuntimeState {
                // Snapshot enablement before activity so rollback can recreate
                // the persistent manager state before starting the unit.
                enabled: systemd_unit_is_enabled(u)?,
                active: systemd_unit_is_active(u)?,
            }))
        }
        "loaded" => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "refusing to mutate {} because systemd reports it loaded without both envctl-owned unit artifacts",
                u.name
            ),
        )),
        state => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "refusing to mutate {} while systemd reports LoadState={state}",
                u.name
            ),
        )),
    }
}

fn systemd_unit_is_enabled(u: &SystemdUnit) -> std::io::Result<bool> {
    let output = trusted_systemctl_command()
        .args(["--user", "is-enabled", &u.name])
        .output()?;
    let state = std::str::from_utf8(&output.stdout)
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "systemctl returned a non-UTF-8 is-enabled state",
            )
        })?
        .trim();
    match state {
        "enabled" | "enabled-runtime" => Ok(true),
        // The explicit envctl discovery bridge may make systemd report a
        // linked state even though no target wants the unit at login.
        "disabled" | "linked" | "linked-runtime" | "static" | "indirect" | "generated"
        | "transient" | "alias" => Ok(false),
        _ => Err(std::io::Error::other(format!(
            "systemctl --user is-enabled {} exited with status {} and reported {:?}",
            u.name, output.status, state
        ))),
    }
}

fn systemd_unit_is_active(u: &SystemdUnit) -> std::io::Result<bool> {
    let output = trusted_systemctl_command()
        .args(["--user", "is-active", &u.name])
        .output()?;
    let state = std::str::from_utf8(&output.stdout)
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "systemctl returned a non-UTF-8 is-active state",
            )
        })?
        .trim();
    match state {
        "active" if output.status.success() => Ok(true),
        "inactive" | "failed" if !output.status.success() => Ok(false),
        _ => Err(std::io::Error::other(format!(
            "systemctl --user is-active {} exited with status {} and reported {:?}",
            u.name, output.status, state
        ))),
    }
}

fn write_systemd_unit_atomically(path: &Path, content: &[u8]) -> std::io::Result<()> {
    use std::io::Write;

    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "systemd user unit path has no parent",
        )
    })?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "systemd user unit name is not valid UTF-8",
            )
        })?;
    let temporary = parent.join(format!(
        ".{name}.envctl-tmp-{}-{}",
        std::process::id(),
        now_epoch()
    ));
    let result = (|| {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(content)?;
        file.sync_all()?;
        std::fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

fn run_systemctl(args: &[&str]) -> std::io::Result<()> {
    let status = trusted_systemctl_command().args(args).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "systemctl {} exited with status {}",
            args.join(" "),
            status
        )))
    }
}

fn trusted_systemctl_command() -> Command {
    #[cfg(test)]
    let program = std::env::var_os("ENVCTL_TEST_SYSTEMCTL_BIN")
        .unwrap_or_else(|| std::ffi::OsString::from("/usr/bin/systemctl"));
    #[cfg(not(test))]
    let program = std::ffi::OsString::from("/usr/bin/systemctl");

    let mut command = Command::new(program);
    // systemctl is a service-ownership control-plane boundary. Do not let caller PATH,
    // LD_PRELOAD, SYSTEMD_UNIT_PATH, pager/editor controls, or exported shell state influence it.
    // Derive the session-bus coordinates from the current effective UID instead of accepting a
    // caller-selected bus. The test-only binary override is compiled out of shipping builds.
    command.env_clear().env("PATH", "/usr/bin:/bin");
    if let Some(runtime_dir) = trusted_user_runtime_dir() {
        command.env("XDG_RUNTIME_DIR", &runtime_dir).env(
            "DBUS_SESSION_BUS_ADDRESS",
            format!("unix:path={}/bus", runtime_dir.display()),
        );
    }
    for key in ["LANG", "LC_ALL", "LC_CTYPE"] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    command
        .env("SYSTEMD_PAGER", "/usr/bin/cat")
        .env("SYSTEMD_COLORS", "0");
    command
}

#[cfg(unix)]
fn trusted_user_runtime_dir() -> Option<PathBuf> {
    let uid = rustix::process::geteuid().as_raw();
    let runtime_dir = PathBuf::from(format!("/run/user/{uid}"));
    let runtime_metadata = std::fs::symlink_metadata(&runtime_dir).ok()?;
    if !runtime_metadata.file_type().is_dir() || runtime_metadata.uid() != uid {
        return None;
    }
    let bus_metadata = std::fs::symlink_metadata(runtime_dir.join("bus")).ok()?;
    if !bus_metadata.file_type().is_socket() || bus_metadata.uid() != uid {
        return None;
    }
    Some(runtime_dir)
}

#[cfg(not(unix))]
fn trusted_user_runtime_dir() -> Option<PathBuf> {
    None
}

fn render_systemd_user_content(layout: &MetaLayout, content: &str) -> std::io::Result<String> {
    let meta_root = layout.meta_root();
    let root = meta_root.to_str().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "META_ROOT is not valid UTF-8 for a systemd unit",
        )
    })?;
    if !meta_root.is_absolute()
        || root.is_empty()
        || root
            .chars()
            .any(|ch| ch.is_control() || matches!(ch, '"' | '\\' | '$'))
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "META_ROOT is not an absolute path safe for a systemd unit",
        ));
    }

    // `${META_ROOT}` is an explicit manifest-template token here, not a shell
    // expansion: systemd does not expand shell variables in ExecStart. Escape
    // '%' so a literal path segment cannot be interpreted as a systemd specifier.
    let systemd_root = root.replace('%', "%%");
    Ok(content.replace("${META_ROOT}", &systemd_root))
}

fn revert_systemd(u: &SystemdUnit) -> std::io::Result<()> {
    let layout = MetaLayout::from_env_required()?;
    // Validate the META_ROOT token and both ownership surfaces before stopping,
    // unlinking, or deleting anything. A relative root or foreign bridge must
    // leave the running unit and every file untouched.
    let expected_content = render_systemd_user_content(&layout, &u.content)?;
    let paths = systemd_unit_paths(&layout, u)?;
    ensure_regular_or_absent(&paths.canonical)?;
    ensure_owned_systemd_bridge_or_absent(&paths.bridge, &paths.canonical)?;
    let canonical_before = match std::fs::read(&paths.canonical) {
        Ok(content) => Some(content),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    if canonical_before
        .as_deref()
        .is_some_and(|content| content != expected_content.as_bytes())
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "refusing to remove foreign-modified canonical systemd user unit: {}",
                paths.canonical.display()
            ),
        ));
    }
    let bridge_was_present = std::fs::symlink_metadata(&paths.bridge).is_ok();
    if canonical_before.is_none() && !bridge_was_present {
        return Ok(());
    }
    let runtime_before =
        snapshot_systemd_runtime_state(u, &paths, canonical_before.is_some(), bridge_was_present)?;

    if runtime_before.is_some() {
        // Snapshot is-enabled/is-active above before the first manager
        // mutation. Even a failed systemctl command may have partially
        // disabled or stopped the service, so restore from the same snapshot.
        if let Err(error) = run_systemctl(&["--user", "disable", "--now", &u.name]) {
            return Err(systemd_rollback_error(
                error,
                restore_systemd_transaction(
                    u,
                    &paths,
                    canonical_before.as_deref(),
                    bridge_was_present,
                    runtime_before,
                    None,
                    false,
                ),
            ));
        }
    }
    let result = (|| {
        match std::fs::symlink_metadata(&paths.bridge) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                // systemctl disable normally removes both its enable links and
                // the explicit unit-file bridge. Remove only the still-verified
                // bridge if the manager left it behind.
                ensure_owned_systemd_bridge_or_absent(&paths.bridge, &paths.canonical)?;
                std::fs::remove_file(&paths.bridge)?;
            }
            Ok(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "systemctl left a foreign systemd user bridge at {}",
                        paths.bridge.display()
                    ),
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        if canonical_before.is_some() {
            let backup =
                PathBuf::from(format!("{}.bak.{}", paths.canonical.display(), now_epoch()));
            std::fs::copy(&paths.canonical, backup)?;
            std::fs::remove_file(&paths.canonical)?;
        }
        run_systemctl(&["--user", "daemon-reload"])
    })();

    match result {
        Ok(()) => Ok(()),
        Err(error) => Err(systemd_rollback_error(
            error,
            restore_systemd_transaction(
                u,
                &paths,
                canonical_before.as_deref(),
                bridge_was_present,
                runtime_before,
                None,
                false,
            ),
        )),
    }
}

fn restore_systemd_ownership(
    paths: &SystemdUserPaths,
    canonical_content: Option<&[u8]>,
    bridge_was_present: bool,
    transaction_content: Option<&[u8]>,
) -> std::io::Result<()> {
    let current_content = match std::fs::symlink_metadata(&paths.canonical) {
        Ok(metadata) if metadata.is_file() => Some(std::fs::read(&paths.canonical)?),
        Ok(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "refusing foreign canonical unit during rollback: {}",
                    paths.canonical.display()
                ),
            ));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    if current_content.as_deref().is_some_and(|content| {
        Some(content) != canonical_content && Some(content) != transaction_content
    }) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "refusing to overwrite a changed canonical unit during rollback: {}",
                paths.canonical.display()
            ),
        ));
    }
    match (canonical_content, current_content.as_deref()) {
        (Some(expected), Some(current)) if expected != current => {
            write_systemd_unit_atomically(&paths.canonical, expected)?;
        }
        (Some(expected), None) => write_systemd_unit_atomically(&paths.canonical, expected)?,
        (None, Some(_)) => std::fs::remove_file(&paths.canonical)?,
        _ => {}
    }

    ensure_owned_systemd_bridge_or_absent(&paths.bridge, &paths.canonical)?;
    let bridge_is_present = std::fs::symlink_metadata(&paths.bridge).is_ok();
    match (bridge_was_present, bridge_is_present) {
        (true, false) => {
            create_owned_systemd_bridge(&paths.bridge, &paths.canonical)?;
        }
        (false, true) => std::fs::remove_file(&paths.bridge)?,
        _ => {}
    }
    Ok(())
}

fn restore_systemd_transaction(
    u: &SystemdUnit,
    paths: &SystemdUserPaths,
    canonical_content: Option<&[u8]>,
    bridge_was_present: bool,
    runtime_before: Option<SystemdRuntimeState>,
    transaction_content: Option<&[u8]>,
    cleanup_new_runtime: bool,
) -> std::io::Result<()> {
    let active_content_changed =
        transaction_content.is_some_and(|transaction| Some(transaction) != canonical_content);
    if runtime_before.is_none() && cleanup_new_runtime {
        // A failed first install may have partially started or enabled a unit.
        // It is still discoverable through the transaction bridge here, so
        // stop/disable it before restoring an originally-not-loaded state.
        run_systemctl(&["--user", "disable", "--now", &u.name])?;
    }
    restore_systemd_ownership(
        paths,
        canonical_content,
        bridge_was_present,
        transaction_content,
    )?;
    run_systemctl(&["--user", "daemon-reload"])?;

    let Some(runtime) = runtime_before else {
        return Ok(());
    };
    if runtime.enabled {
        run_systemctl(&["--user", "enable", &u.name])?;
    } else {
        run_systemctl(&["--user", "disable", &u.name])?;
        // systemctl disable removes envctl's discovery link along with wants
        // links. Recreate only the pre-existing, already-validated bridge.
        restore_systemd_ownership(
            paths,
            canonical_content,
            bridge_was_present,
            transaction_content,
        )?;
        run_systemctl(&["--user", "daemon-reload"])?;
    }
    if runtime.active && active_content_changed {
        // `start` is a no-op for an already-active unit. A later apply step
        // can fail after the changed unit restarted successfully, so force a
        // stop/start after restoring the old bytes and reloading the manager.
        run_systemctl(&["--user", "stop", &u.name])?;
        run_systemctl(&["--user", "start", &u.name])?;
    } else if runtime.active {
        run_systemctl(&["--user", "start", &u.name])?;
    } else {
        run_systemctl(&["--user", "stop", &u.name])?;
    }
    Ok(())
}

fn systemd_rollback_error(
    error: std::io::Error,
    restore_result: std::io::Result<()>,
) -> std::io::Error {
    match restore_result {
        Ok(()) => std::io::Error::new(
            error.kind(),
            format!("{error}; canonical unit, discovery bridge, and prior runtime state restored"),
        ),
        Err(restore_error) => std::io::Error::other(format!(
            "{error}; additionally failed to restore the prior systemd state: {restore_error}"
        )),
    }
}

// ================================ apt repos =================================

/// AUDIT-FIX (minor): a manifest-supplied `list_file` is interpolated into
/// `{SOURCES_D}/{list_file}` for sudo tee/cp/rm. Reject anything that is not a
/// single, plain path component so a `../` or absolute value can't escape
/// /etc/apt/sources.list.d.
fn check_list_file(list_file: &str) -> anyhow::Result<()> {
    if list_file.is_empty() || list_file.contains('/') || list_file == "." || list_file == ".." {
        anyhow::bail!("invalid apt list_file '{list_file}': must be a single path component");
    }
    Ok(())
}

/// Returns Ok(true) if it wrote anything (keyring or .list).
fn apply_apt_repo(r: &AptRepo, rep: &mut WiringReport) -> anyhow::Result<bool> {
    check_list_file(&r.list_file)?;
    let mut changed = false;
    if !Path::new(&r.keyring_path).exists() {
        if let Some(parent) = Path::new(&r.keyring_path).parent() {
            let p = parent.to_string_lossy().into_owned();
            sudo(&["install", "-dm", "755", &p])?;
        }
        if let Err(e) = fetch_apt_keyring(r) {
            // AUDIT-FIX: drop the partial keyring so the next run retries the fetch.
            let _ = sudo(&["rm", "-f", &r.keyring_path]);
            return Err(e);
        }
        sudo(&["chmod", "go+r", &r.keyring_path])?;
        rep.note(format!("wrote keyring {}", r.keyring_path));
        changed = true;
    }
    let list_path = format!("{SOURCES_D}/{}", r.list_file);
    let want = format!("{}\n", r.list_line.trim_end());
    if std::fs::read_to_string(&list_path).unwrap_or_default() != want {
        if Path::new(&list_path).exists() {
            sudo(&[
                "cp",
                &list_path,
                &format!("{list_path}.bak.{}", now_epoch()),
            ])?;
        }
        write_sudo(&list_path, &want)?;
        rep.note(format!("wrote apt source {list_path}"));
        changed = true;
    }
    Ok(changed)
}

/// Returns Ok(true) if it removed anything. Stops at the first failure BEFORE
/// touching the keyring (the only broken state is key-gone+list-present).
fn revert_apt_repo(r: &AptRepo, rep: &mut WiringReport) -> anyhow::Result<bool> {
    check_list_file(&r.list_file)?; // audit fix (minor): keep rm/cp confined to SOURCES_D
    let mut changed = false;
    let list_path = format!("{SOURCES_D}/{}", r.list_file);
    if Path::new(&list_path).exists() {
        sudo(&[
            "cp",
            &list_path,
            &format!("{list_path}.bak.{}", now_epoch()),
        ])?;
        sudo(&["rm", "-f", &list_path])?;
        rep.note(format!("removed apt source {list_path}"));
        changed = true;
    }
    if Path::new(&r.keyring_path).exists() {
        sudo(&[
            "cp",
            &r.keyring_path,
            &format!("{}.bak.{}", r.keyring_path, now_epoch()),
        ])?;
        sudo(&["rm", "-f", &r.keyring_path])?;
        rep.note(format!("removed keyring {}", r.keyring_path));
        changed = true;
    }
    Ok(changed)
}

// =============================== nix conf lines =============================

/// Returns Ok(true) if it appended the line (was absent).
fn apply_nix_line(l: &NixConfLine) -> anyhow::Result<bool> {
    sudo(&["install", "-dm", "755", "/etc/nix"])?;
    sudo(&["touch", NIX_CONF])?;
    let cur = read_sudo(NIX_CONF).unwrap_or_default();
    if cur.lines().any(|ln| ln == l.line) {
        return Ok(false);
    }
    sudo(&["cp", NIX_CONF, &format!("{NIX_CONF}.bak.{}", now_epoch())])?;
    let mut wanted = cur;
    if !wanted.is_empty() && !wanted.ends_with('\n') {
        wanted.push('\n');
    }
    wanted.push_str(&l.line);
    wanted.push('\n');
    write_sudo(NIX_CONF, &wanted)?;
    Ok(true)
}

/// Returns Ok(true) if it removed an owned line.
fn revert_nix_line(l: &NixConfLine, rep: &mut WiringReport) -> anyhow::Result<bool> {
    let Some(cur) = read_sudo(NIX_CONF) else {
        return Ok(false);
    };
    if !cur.lines().any(|ln| ln == l.line) {
        return Ok(false);
    }
    sudo(&["cp", NIX_CONF, &format!("{NIX_CONF}.bak.{}", now_epoch())])?;
    let kept: String = cur
        .lines()
        .filter(|ln| *ln != l.line)
        .map(|ln| format!("{ln}\n"))
        .collect();
    write_sudo(NIX_CONF, &kept)?;
    rep.note(format!("removed nix.custom.conf line: {}", l.line));
    Ok(true)
}

fn restart_nix_daemon(rep: &mut WiringReport) {
    match sudo(&["systemctl", "restart", "nix-daemon"]) {
        Ok(()) => rep.note("restarted nix-daemon (nix.custom.conf changed)"),
        Err(error) => rep.fail("nix_conf_line", error),
    }
}

// ================================ cdi spec ==================================

fn apply_cdi(c: &CdiSpec) -> anyhow::Result<()> {
    if Path::new(&c.output).exists() {
        return Ok(());
    }
    if let Some(parent) = Path::new(&c.output).parent() {
        let p = parent.to_string_lossy().into_owned();
        sudo(&["install", "-dm", "755", &p])?;
    }
    let mut argv: Vec<String> = vec!["nvidia-ctk".into()];
    argv.extend(c.generate_args.iter().cloned());
    argv.push(format!("--output={}", c.output));
    let refs: Vec<&str> = argv.iter().map(|s| s.as_str()).collect();
    let _ = sudo(&refs); // wizard guards this with `|| true`
    Ok(())
}

fn revert_cdi(c: &CdiSpec, rep: &mut WiringReport) -> anyhow::Result<()> {
    if Path::new(&c.output).exists() {
        sudo(&[
            "cp",
            &c.output,
            &format!("{}.bak.{}", c.output, now_epoch()),
        ])?;
        sudo(&["rm", "-f", &c.output])?;
        rep.note(format!("removed CDI spec {}", c.output));
    }
    Ok(())
}

// ============================== alternatives ================================

fn apply_alternative(a: &Alternative, rep: &mut WiringReport) -> anyhow::Result<()> {
    let Some(target) = resolve_target(&a.target) else {
        rep.note(format!(
            "alternative '{}': target '{}' not found; skipped",
            a.name, a.target
        ));
        return Ok(());
    };
    let prio = a.priority.to_string();
    // AUDIT-FIX: surface sudo failures instead of asserting success.
    if let Err(e) = sudo(&[
        "update-alternatives",
        "--install",
        &a.link,
        &a.name,
        &target,
        &prio,
    ]) {
        rep.fail("alternative", e);
        return Ok(());
    }
    if let Err(e) = sudo(&["update-alternatives", "--set", &a.name, &target]) {
        rep.fail("alternative", e);
        return Ok(());
    }
    rep.note(format!("set alternative {} -> {target}", a.name));
    Ok(())
}

fn revert_alternative(a: &Alternative, rep: &mut WiringReport) -> anyhow::Result<()> {
    let Some(target) = resolve_target(&a.target) else {
        // AUDIT-FIX: the target no longer resolves (e.g. binary uninstalled), but
        // the engine-owned alternative slot may still be installed. Fall back to
        // --remove-all so we don't silently leave it behind.
        if let Err(e) = sudo(&["update-alternatives", "--remove-all", &a.name]) {
            rep.fail("alternative", e);
            return Ok(());
        }
        rep.note(format!(
            "removed alternative {} (target '{}' unresolved; used --remove-all)",
            a.name, a.target
        ));
        return Ok(());
    };
    if let Err(e) = sudo(&["update-alternatives", "--remove", &a.name, &target]) {
        rep.fail("alternative", e);
        return Ok(());
    }
    rep.note(format!("removed alternative {} -> {target}", a.name));
    Ok(())
}

fn resolve_target(t: &str) -> Option<String> {
    if t.starts_with('/') && Path::new(t).exists() {
        return Some(t.to_string());
    }
    which::which(t)
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
}

// ================================ helpers ===================================

fn home() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "/root".into())
}

fn sudo(argv: &[&str]) -> anyhow::Result<()> {
    let mut command = sudo_invocation(argv)?;
    let st = command.status()?;
    if st.success() {
        Ok(())
    } else {
        anyhow::bail!("sudo {} exited {:?}", argv.join(" "), st.code())
    }
}

fn trusted_host_tool(name: &str) -> anyhow::Result<&'static str> {
    match name {
        "apt-get" => Ok("/usr/bin/apt-get"),
        "cat" => Ok("/usr/bin/cat"),
        "chmod" => Ok("/usr/bin/chmod"),
        "cp" => Ok("/usr/bin/cp"),
        "gpg" => Ok("/usr/bin/gpg"),
        "install" => Ok("/usr/bin/install"),
        "nvidia-ctk" => Ok("/usr/bin/nvidia-ctk"),
        "rm" => Ok("/usr/bin/rm"),
        "systemctl" => Ok("/usr/bin/systemctl"),
        "tee" => Ok("/usr/bin/tee"),
        "touch" => Ok("/usr/bin/touch"),
        "update-alternatives" => Ok("/usr/bin/update-alternatives"),
        other => anyhow::bail!("refusing untrusted sudo tool '{other}'"),
    }
}

fn trusted_command(program: &str) -> Command {
    let mut command = Command::new(program);
    command.env_clear().env("PATH", "/usr/bin:/bin");
    for key in ["LANG", "LC_ALL", "LC_CTYPE"] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    command
}

fn trusted_sudo_command() -> Command {
    trusted_command("/usr/bin/sudo")
}

fn sudo_invocation(argv: &[&str]) -> anyhow::Result<Command> {
    let Some((tool, args)) = argv.split_first() else {
        anyhow::bail!("sudo invocation must name a tool");
    };
    let mut command = trusted_sudo_command();
    command
        .arg("-n")
        .arg("--")
        .arg(trusted_host_tool(tool)?)
        .args(args);
    Ok(command)
}

fn fetch_apt_keyring(repo: &AptRepo) -> anyhow::Result<()> {
    let mut curl = trusted_command("/usr/bin/curl");
    curl.args([
        "--disable",
        "--fail",
        "--silent",
        "--show-error",
        "--location",
        "--proto",
        "=https",
        "--tlsv1.2",
        &repo.keyring_url,
    ])
    .stdout(Stdio::piped());
    let mut curl_child = curl.spawn()?;
    let stdout = curl_child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("curl keyring stdout was not piped"))?;

    let sink_tool = if repo.dearmor { "gpg" } else { "tee" };
    let mut sink = sudo_invocation(&[sink_tool])?;
    if repo.dearmor {
        sink.args([
            "--no-options",
            "--batch",
            "--yes",
            "--dearmor",
            "--output",
            &repo.keyring_path,
        ]);
    } else {
        sink.arg(&repo.keyring_path).stdout(Stdio::null());
    }
    let sink_status = match sink.stdin(Stdio::from(stdout)).status() {
        Ok(status) => status,
        Err(error) => {
            let _ = curl_child.kill();
            let _ = curl_child.wait();
            return Err(error.into());
        }
    };
    let curl_status = curl_child.wait()?;
    if !curl_status.success() || !sink_status.success() {
        anyhow::bail!("keyring fetch failed: curl={curl_status}, {sink_tool}={sink_status}");
    }
    Ok(())
}

fn read_sudo(path: &str) -> Option<String> {
    let out = sudo_invocation(&["cat", path]).ok()?.output().ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        None
    }
}

fn write_sudo(path: &str, body: &str) -> anyhow::Result<()> {
    use std::io::Write;
    let mut child = sudo_invocation(&["tee", path])?
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .spawn()?;
    child.stdin.as_mut().unwrap().write_all(body.as_bytes())?;
    let st = child.wait()?;
    if st.success() {
        Ok(())
    } else {
        anyhow::bail!("sudo tee {path} failed")
    }
}

/// Nanosecond stamp for backup/trash names — collision-proof at sub-second
/// resolution (two edits to the same file in the same second won't clobber a
/// prior backup) (audit fix).
fn now_epoch() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    fn systemd_test_dir(label: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "envctl-systemd-{label}-{}-{}",
            std::process::id(),
            now_epoch()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[cfg(unix)]
    fn install_fake_systemctl(root: &Path, fail_stage: Option<&str>) -> std::path::PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let bin = root.join("bin");
        let fail_file = root.join("fail-stage");
        std::fs::create_dir_all(&bin).unwrap();
        if let Some(stage) = fail_stage {
            std::fs::write(&fail_file, stage).unwrap();
        }
        let script = r#"#!/bin/sh
set -eu
state='__STATE__'
log="$state/systemctl.log"
fail_file="$state/fail-stage"
fail_once="$state/fail-once"
test_home_file="$state/systemctl.test-home"
[ -f "$test_home_file" ] || exit 30
test_home="$(cat "$test_home_file")"
active="$state/systemctl.active"
enabled="$state/systemctl.enabled"
running_content="$state/systemctl.running-content"
record_running_content() {
  unit="$1"
  bridge="$test_home/.config/systemd/user/$unit"
  [ -L "$bridge" ] && [ -f "$(readlink "$bridge")" ] || return 31
  cp "$(readlink "$bridge")" "$running_content"
}
printf '%s\n' "$*" >> "$log"
if [ -s "$fail_file" ]; then
  fail="$(cat "$fail_file")"
  case "$*" in
    *"$fail"*)
      # Model a command that mutates manager state before returning failure;
      # rollback tests must repair partial effects, not just a clean no-op.
      case "$*" in
        '--user restart '*) rm -f "$active" "$running_content" ;;
        '--user enable --now '*)
          unit="$4"
          : > "$enabled"
          : > "$active"
          record_running_content "$unit"
          ;;
        '--user disable --now '*)
          unit="$4"
          rm -f "$test_home/.config/systemd/user/$unit" "$enabled" "$active" "$running_content"
          ;;
      esac
      if [ -e "$fail_once" ]; then
        rm -f "$fail_once" "$fail_file"
      fi
      exit 23
      ;;
  esac
fi
case "$*" in
  '--user show --property=FragmentPath --value '*)
    unit="$5"
    bridge="$test_home/.config/systemd/user/$unit"
    [ -L "$bridge" ] || exit 4
    target="$(readlink "$bridge")"
    [ -f "$target" ] || exit 4
    printf '%s\n' "$bridge"
    ;;
  '--user show --property=LoadState --value '*)
    unit="$5"
    bridge="$test_home/.config/systemd/user/$unit"
    if [ -L "$bridge" ] && [ -f "$(readlink "$bridge")" ]; then
      printf 'loaded\n'
    else
      printf 'not-found\n'
    fi
    ;;
  '--user show --property=ActiveState --value '*)
    if [ -f "$active" ]; then printf 'active\n'; else printf 'inactive\n'; fi
    ;;
  '--user show --property=UnitFileState --value '*)
    if [ -f "$enabled" ]; then printf 'enabled\n'; else printf 'disabled\n'; fi
    ;;
  '--user is-active '*)
    if [ -f "$active" ]; then
      printf 'active\n'
    else
      printf 'inactive\n'
      exit 3
    fi
    ;;
  '--user is-enabled '*)
    if [ -f "$enabled" ]; then
      printf 'enabled\n'
    else
      printf 'disabled\n'
      exit 1
    fi
    ;;
  '--user enable --now '*)
    unit="$4"
    bridge="$test_home/.config/systemd/user/$unit"
    [ -L "$bridge" ] && [ -f "$(readlink "$bridge")" ] || exit 31
    : > "$enabled"
    : > "$active"
    record_running_content "$unit"
    ;;
  '--user enable '*)
    unit="$3"
    bridge="$test_home/.config/systemd/user/$unit"
    [ -L "$bridge" ] && [ -f "$(readlink "$bridge")" ] || exit 31
    : > "$enabled"
    ;;
  '--user disable --now '*)
    unit="$4"
    rm -f "$test_home/.config/systemd/user/$unit" "$enabled" "$active" "$running_content"
    ;;
  '--user disable '*)
    unit="$3"
    rm -f "$test_home/.config/systemd/user/$unit" "$enabled"
    ;;
  '--user restart '*)
    unit="$3"
    bridge="$test_home/.config/systemd/user/$unit"
    [ -L "$bridge" ] && [ -f "$(readlink "$bridge")" ] || exit 31
    : > "$active"
    record_running_content "$unit"
    ;;
  '--user start '*)
    unit="$3"
    bridge="$test_home/.config/systemd/user/$unit"
    [ -L "$bridge" ] && [ -f "$(readlink "$bridge")" ] || exit 31
    if [ ! -f "$active" ]; then
      : > "$active"
      record_running_content "$unit"
    fi
    ;;
  '--user stop '*)
    rm -f "$active" "$running_content"
    ;;
esac
exit 0
"#
        .replace("__STATE__", &root.display().to_string());
        let path = bin.join("systemctl");
        std::fs::write(&path, script).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        bin
    }

    #[cfg(unix)]
    fn run_systemd_test_child(
        test_name: &str,
        mode: &str,
        meta_root: Option<&Path>,
        home: &Path,
        fake_bin: &Path,
        current_dir: Option<&Path>,
    ) {
        std::fs::write(
            fake_bin.parent().unwrap().join("systemctl.test-home"),
            home.as_os_str().as_encoded_bytes(),
        )
        .unwrap();
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([test_name, "--exact", "--nocapture"])
            .env("ENVCTL_SYSTEMD_TEST_CHILD", mode)
            .env("HOME", home)
            .env_remove("XDG_CONFIG_HOME")
            .env_remove("ENVCTL_REAL_HOME")
            .env_remove("ENVCTL_REAL_XDG_CONFIG_HOME")
            .env("ENVCTL_TEST_SYSTEMCTL_BIN", fake_bin.join("systemctl"));
        if let Some(root) = meta_root {
            command.env("META_ROOT", root);
        } else {
            command.env_remove("META_ROOT");
        }
        if let Some(dir) = current_dir {
            command.current_dir(dir);
        }

        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "child {mode} failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn demo_systemd_unit(enable: bool) -> SystemdUnit {
        SystemdUnit {
            name: "demo.service".into(),
            content:
                "Environment=\"META_ROOT=${META_ROOT}\"\nExecStart=\"${META_ROOT}/usr/bin/demo\"\n"
                    .into(),
            enable,
        }
    }

    fn changed_demo_systemd_unit(enable: bool) -> SystemdUnit {
        SystemdUnit {
            name: "demo.service".into(),
            content: "Environment=\"META_ROOT=${META_ROOT}\"\nExecStart=\"${META_ROOT}/usr/bin/demo\" --changed\n"
                .into(),
            enable,
        }
    }

    #[cfg(unix)]
    #[test]
    fn systemctl_control_plane_uses_an_absolute_scrubbed_entrypoint() {
        let _lock = crate::test_env_lock();
        let prior_path = std::env::var_os("PATH");
        let prior_preload = std::env::var_os("LD_PRELOAD");
        let prior_bus = std::env::var_os("DBUS_SESSION_BUS_ADDRESS");
        let prior_runtime = std::env::var_os("XDG_RUNTIME_DIR");
        let prior_override = std::env::var_os("ENVCTL_TEST_SYSTEMCTL_BIN");
        std::env::set_var("PATH", "/must/not/use");
        std::env::set_var("LD_PRELOAD", "/must/not/load.so");
        std::env::set_var("DBUS_SESSION_BUS_ADDRESS", "unix:path=/must/not/use/bus");
        std::env::set_var("XDG_RUNTIME_DIR", "/must/not/use");
        std::env::remove_var("ENVCTL_TEST_SYSTEMCTL_BIN");

        let command = trusted_systemctl_command();

        match prior_path {
            Some(value) => std::env::set_var("PATH", value),
            None => std::env::remove_var("PATH"),
        }
        match prior_preload {
            Some(value) => std::env::set_var("LD_PRELOAD", value),
            None => std::env::remove_var("LD_PRELOAD"),
        }
        match prior_bus {
            Some(value) => std::env::set_var("DBUS_SESSION_BUS_ADDRESS", value),
            None => std::env::remove_var("DBUS_SESSION_BUS_ADDRESS"),
        }
        match prior_runtime {
            Some(value) => std::env::set_var("XDG_RUNTIME_DIR", value),
            None => std::env::remove_var("XDG_RUNTIME_DIR"),
        }
        match prior_override {
            Some(value) => std::env::set_var("ENVCTL_TEST_SYSTEMCTL_BIN", value),
            None => std::env::remove_var("ENVCTL_TEST_SYSTEMCTL_BIN"),
        }

        assert_eq!(command.get_program(), "/usr/bin/systemctl");
        assert!(command
            .get_envs()
            .all(|(key, _)| key != std::ffi::OsStr::new("LD_PRELOAD")));
        assert_eq!(
            command
                .get_envs()
                .find(|(key, _)| *key == std::ffi::OsStr::new("PATH"))
                .and_then(|(_, value)| value),
            Some(std::ffi::OsStr::new("/usr/bin:/bin"))
        );
        let expected_runtime = trusted_user_runtime_dir();
        let actual_runtime = command
            .get_envs()
            .find(|(key, _)| *key == std::ffi::OsStr::new("XDG_RUNTIME_DIR"))
            .and_then(|(_, value)| value)
            .map(PathBuf::from);
        assert_eq!(actual_runtime, expected_runtime);
        let expected_bus = expected_runtime
            .as_ref()
            .map(|path| std::ffi::OsString::from(format!("unix:path={}/bus", path.display())));
        let actual_bus = command
            .get_envs()
            .find(|(key, _)| *key == std::ffi::OsStr::new("DBUS_SESSION_BUS_ADDRESS"))
            .and_then(|(_, value)| value)
            .map(std::ffi::OsString::from);
        assert_eq!(actual_bus, expected_bus);
    }

    #[test]
    fn privileged_wiring_uses_absolute_allowlisted_tools_and_a_scrubbed_environment() {
        let command = sudo_invocation(&["cp", "/from", "/to"]).unwrap();
        assert_eq!(command.get_program(), "/usr/bin/sudo");
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            ["-n", "--", "/usr/bin/cp", "/from", "/to"].map(std::ffi::OsStr::new)
        );
        assert_eq!(
            command
                .get_envs()
                .find(|(key, _)| *key == std::ffi::OsStr::new("PATH"))
                .and_then(|(_, value)| value),
            Some(std::ffi::OsStr::new("/usr/bin:/bin"))
        );
        assert!(sudo_invocation(&["caller-shadow"]).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn systemd_user_apply_is_meta_owned_and_second_pass_does_not_rewrite() {
        const MODE: &str = "meta-owned-idempotent";
        const TEST: &str =
            "wiring::tests::systemd_user_apply_is_meta_owned_and_second_pass_does_not_rewrite";
        if std::env::var("ENVCTL_SYSTEMD_TEST_CHILD").as_deref() == Ok(MODE) {
            let root = std::path::PathBuf::from(std::env::var_os("META_ROOT").unwrap());
            let home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
            let unit = demo_systemd_unit(true);

            apply_systemd(&unit).unwrap();
            apply_systemd(&unit).unwrap();

            let unit_path = root.join(".config/systemd/user/demo.service");
            assert_eq!(
                std::fs::read_to_string(&unit_path).unwrap(),
                format!(
                    "Environment=\"META_ROOT={}\"\nExecStart=\"{}/usr/bin/demo\"\n",
                    root.display(),
                    root.display()
                )
            );
            let bridge = home.join(".config/systemd/user/demo.service");
            assert_eq!(
                std::fs::read_link(&bridge).unwrap(),
                unit_path,
                "the only real-home artifact must be the engine-owned discovery bridge"
            );
            let backups = std::fs::read_dir(unit_path.parent().unwrap())
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_name().to_string_lossy().contains(".bak."))
                .count();
            assert_eq!(
                backups, 0,
                "an unchanged second pass must not create a backup"
            );

            let log = std::fs::read_to_string(root.join("systemctl.log")).unwrap();
            assert_eq!(log.matches("--user daemon-reload").count(), 1);
            assert_eq!(
                log.matches("--user show --property=FragmentPath --value demo.service")
                    .count(),
                3
            );
            assert_eq!(log.matches("--user is-enabled demo.service").count(), 1);
            assert_eq!(log.matches("--user is-active demo.service").count(), 1);
            assert_eq!(log.matches("--user enable --now demo.service").count(), 2);
            return;
        }

        let root = systemd_test_dir("meta-root");
        let home = systemd_test_dir("real-home");
        let fake_bin = install_fake_systemctl(&root, None);
        run_systemd_test_child(TEST, MODE, Some(&root), &home, &fake_bin, None);
        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_dir_all(home).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn systemd_user_apply_restarts_only_when_active_content_changes() {
        const MODE: &str = "restart-changed-active";
        const TEST: &str =
            "wiring::tests::systemd_user_apply_restarts_only_when_active_content_changes";
        if std::env::var("ENVCTL_SYSTEMD_TEST_CHILD").as_deref() == Ok(MODE) {
            let root = std::path::PathBuf::from(std::env::var_os("META_ROOT").unwrap());
            let unit = demo_systemd_unit(true);
            apply_systemd(&unit).unwrap();
            apply_systemd(&unit).unwrap();
            let unchanged_log = std::fs::read_to_string(root.join("systemctl.log")).unwrap();
            assert!(!unchanged_log.contains("--user restart demo.service"));

            apply_systemd(&changed_demo_systemd_unit(true)).unwrap();
            let changed_log = std::fs::read_to_string(root.join("systemctl.log")).unwrap();
            assert_eq!(
                changed_log.matches("--user restart demo.service").count(),
                1
            );
            assert!(root.join("systemctl.active").exists());
            assert!(root.join("systemctl.enabled").exists());
            return;
        }

        let root = systemd_test_dir("restart-changed-active-meta");
        let home = systemd_test_dir("restart-changed-active-home");
        let fake_bin = install_fake_systemctl(&root, None);
        run_systemd_test_child(TEST, MODE, Some(&root), &home, &fake_bin, None);
        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_dir_all(home).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn systemd_user_apply_restart_failure_restores_content_and_runtime_state() {
        const MODE: &str = "restart-failure-rollback";
        const TEST: &str =
            "wiring::tests::systemd_user_apply_restart_failure_restores_content_and_runtime_state";
        if std::env::var("ENVCTL_SYSTEMD_TEST_CHILD").as_deref() == Ok(MODE) {
            let root = std::path::PathBuf::from(std::env::var_os("META_ROOT").unwrap());
            let home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
            let canonical = root.join(".config/systemd/user/demo.service");
            let bridge = home.join(".config/systemd/user/demo.service");
            let original = demo_systemd_unit(true);
            apply_systemd(&original).unwrap();
            let original_content = std::fs::read(&canonical).unwrap();
            std::fs::write(root.join("fail-stage"), "restart demo.service").unwrap();

            let err = apply_systemd(&changed_demo_systemd_unit(true)).unwrap_err();
            assert_eq!(err.kind(), std::io::ErrorKind::Other);
            assert!(err.to_string().contains("restart"));
            assert_eq!(std::fs::read(&canonical).unwrap(), original_content);
            assert_eq!(std::fs::read_link(&bridge).unwrap(), canonical);
            assert!(root.join("systemctl.active").exists());
            assert!(root.join("systemctl.enabled").exists());
            let log = std::fs::read_to_string(root.join("systemctl.log")).unwrap();
            assert!(log.contains("--user enable demo.service"));
            assert!(log.contains("--user start demo.service"));
            return;
        }

        let root = systemd_test_dir("restart-failure-rollback-meta");
        let home = systemd_test_dir("restart-failure-rollback-home");
        let fake_bin = install_fake_systemctl(&root, None);
        run_systemd_test_child(TEST, MODE, Some(&root), &home, &fake_bin, None);
        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_dir_all(home).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn systemd_user_apply_late_failure_reloads_restored_active_content() {
        const MODE: &str = "late-enable-failure-runtime-rollback";
        const TEST: &str =
            "wiring::tests::systemd_user_apply_late_failure_reloads_restored_active_content";
        if std::env::var("ENVCTL_SYSTEMD_TEST_CHILD").as_deref() == Ok(MODE) {
            let root = std::path::PathBuf::from(std::env::var_os("META_ROOT").unwrap());
            let home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
            let canonical = root.join(".config/systemd/user/demo.service");
            let bridge = home.join(".config/systemd/user/demo.service");
            let running_content = root.join("systemctl.running-content");
            let original = demo_systemd_unit(true);
            apply_systemd(&original).unwrap();
            let original_content = std::fs::read(&canonical).unwrap();
            assert_eq!(std::fs::read(&running_content).unwrap(), original_content);
            std::fs::write(root.join("fail-stage"), "enable --now demo.service").unwrap();

            let err = apply_systemd(&changed_demo_systemd_unit(true)).unwrap_err();
            assert_eq!(err.kind(), std::io::ErrorKind::Other);
            assert!(err.to_string().contains("enable --now"));
            assert_eq!(std::fs::read(&canonical).unwrap(), original_content);
            assert_eq!(std::fs::read_link(&bridge).unwrap(), canonical);
            assert!(root.join("systemctl.active").exists());
            assert!(root.join("systemctl.enabled").exists());
            assert_eq!(
                std::fs::read(&running_content).unwrap(),
                original_content,
                "rollback must reload the restored bytes into the active process"
            );

            let log = std::fs::read_to_string(root.join("systemctl.log")).unwrap();
            assert_eq!(log.matches("--user restart demo.service").count(), 1);
            assert!(log.contains("--user stop demo.service"));
            assert!(log.contains("--user start demo.service"));
            return;
        }

        let root = systemd_test_dir("late-enable-failure-runtime-rollback-meta");
        let home = systemd_test_dir("late-enable-failure-runtime-rollback-home");
        let fake_bin = install_fake_systemctl(&root, None);
        run_systemd_test_child(TEST, MODE, Some(&root), &home, &fake_bin, None);
        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_dir_all(home).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn systemd_user_apply_refuses_when_meta_root_is_absent() {
        const MODE: &str = "missing-meta-root";
        const TEST: &str = "wiring::tests::systemd_user_apply_refuses_when_meta_root_is_absent";
        if std::env::var("ENVCTL_SYSTEMD_TEST_CHILD").as_deref() == Ok(MODE) {
            let home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
            let err = apply_systemd(&demo_systemd_unit(true)).unwrap_err();
            assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
            assert!(!home.join(".config/systemd/user/demo.service").exists());
            assert!(!home.join("Desktop/meta").exists());
            return;
        }

        let root = systemd_test_dir("missing-meta-fixture");
        let home = systemd_test_dir("missing-meta-home");
        let fake_bin = install_fake_systemctl(&root, None);
        run_systemd_test_child(TEST, MODE, None, &home, &fake_bin, None);
        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_dir_all(home).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn systemd_user_apply_refuses_a_relative_meta_root_without_mutation() {
        const MODE: &str = "relative-meta-root";
        const TEST: &str =
            "wiring::tests::systemd_user_apply_refuses_a_relative_meta_root_without_mutation";
        if std::env::var("ENVCTL_SYSTEMD_TEST_CHILD").as_deref() == Ok(MODE) {
            let home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
            let err = apply_systemd(&demo_systemd_unit(true)).unwrap_err();
            assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
            assert!(!Path::new("relative-meta").exists());
            assert!(!home.join(".config/systemd/user/demo.service").exists());
            return;
        }

        let fixture = systemd_test_dir("relative-root-fixture");
        let home = systemd_test_dir("relative-root-home");
        let fake_bin = install_fake_systemctl(&fixture, None);
        run_systemd_test_child(
            TEST,
            MODE,
            Some(Path::new("relative-meta")),
            &home,
            &fake_bin,
            Some(&fixture),
        );
        assert!(!fixture.join("relative-meta").exists());
        assert!(!fixture.join("systemctl.log").exists());
        std::fs::remove_dir_all(fixture).unwrap();
        std::fs::remove_dir_all(home).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn systemd_user_apply_refuses_meta_root_dollar_expansion_without_mutation() {
        const MODE: &str = "meta-root-dollar-expansion";
        const TEST: &str =
            "wiring::tests::systemd_user_apply_refuses_meta_root_dollar_expansion_without_mutation";
        if std::env::var("ENVCTL_SYSTEMD_TEST_CHILD").as_deref() == Ok(MODE) {
            let root = std::path::PathBuf::from(std::env::var_os("META_ROOT").unwrap());
            let home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
            assert!(root.is_absolute());
            assert!(root.to_string_lossy().contains("${HOME}"));

            let err = apply_systemd(&demo_systemd_unit(false)).unwrap_err();
            assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
            assert!(err.to_string().contains("safe for a systemd unit"));
            assert!(!root.join(".config/systemd/user/demo.service").exists());
            assert!(!home.join(".config/systemd/user/demo.service").exists());
            assert!(!root.parent().unwrap().join("systemctl.log").exists());
            return;
        }

        let fixture = systemd_test_dir("meta-root-dollar-expansion-fixture");
        let root = fixture.join("${HOME}");
        let home = systemd_test_dir("meta-root-dollar-expansion-home");
        let fake_bin = install_fake_systemctl(&fixture, None);
        run_systemd_test_child(TEST, MODE, Some(&root), &home, &fake_bin, None);
        assert!(!root.exists());
        std::fs::remove_dir_all(fixture).unwrap();
        std::fs::remove_dir_all(home).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn systemd_user_apply_refuses_a_symlink_projection() {
        const MODE: &str = "symlink-projection";
        const TEST: &str = "wiring::tests::systemd_user_apply_refuses_a_symlink_projection";
        if std::env::var("ENVCTL_SYSTEMD_TEST_CHILD").as_deref() == Ok(MODE) {
            let root = std::path::PathBuf::from(std::env::var_os("META_ROOT").unwrap());
            let unit_dir = root.join(".config/systemd/user");
            let tracked_projection = root.join("tracked-home-projection.service");
            std::fs::create_dir_all(&unit_dir).unwrap();
            std::fs::write(
                &tracked_projection,
                "tracked projection must remain unchanged\n",
            )
            .unwrap();
            std::os::unix::fs::symlink(&tracked_projection, unit_dir.join("demo.service")).unwrap();

            let err = apply_systemd(&demo_systemd_unit(false)).unwrap_err();
            assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
            assert_eq!(
                std::fs::read_to_string(&tracked_projection).unwrap(),
                "tracked projection must remain unchanged\n"
            );
            assert!(!root.join("systemctl.log").exists());
            return;
        }

        let root = systemd_test_dir("symlink-meta");
        let home = systemd_test_dir("symlink-home");
        let fake_bin = install_fake_systemctl(&root, None);
        run_systemd_test_child(TEST, MODE, Some(&root), &home, &fake_bin, None);
        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_dir_all(home).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn systemd_user_apply_refuses_a_foreign_discovery_bridge() {
        const MODE: &str = "foreign-bridge";
        const TEST: &str = "wiring::tests::systemd_user_apply_refuses_a_foreign_discovery_bridge";
        if std::env::var("ENVCTL_SYSTEMD_TEST_CHILD").as_deref() == Ok(MODE) {
            let root = std::path::PathBuf::from(std::env::var_os("META_ROOT").unwrap());
            let home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
            let tracked_projection = root.join("tracked-home-projection.service");
            let bridge = home.join(".config/systemd/user/demo.service");
            std::fs::create_dir_all(bridge.parent().unwrap()).unwrap();
            std::fs::write(&tracked_projection, "foreign tracked unit\n").unwrap();
            std::os::unix::fs::symlink(&tracked_projection, &bridge).unwrap();

            let err = apply_systemd(&demo_systemd_unit(false)).unwrap_err();
            assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
            assert_eq!(std::fs::read_link(&bridge).unwrap(), tracked_projection);
            assert!(!root.join(".config/systemd/user/demo.service").exists());
            assert!(!root.join("systemctl.log").exists());
            return;
        }

        let root = systemd_test_dir("foreign-bridge-meta");
        let home = systemd_test_dir("foreign-bridge-home");
        let fake_bin = install_fake_systemctl(&root, None);
        run_systemd_test_child(TEST, MODE, Some(&root), &home, &fake_bin, None);
        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_dir_all(home).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn systemd_user_apply_propagates_systemctl_failures() {
        const TEST: &str = "wiring::tests::systemd_user_apply_propagates_systemctl_failures";
        if let Ok(mode) = std::env::var("ENVCTL_SYSTEMD_TEST_CHILD") {
            if mode == "reload-failure" || mode == "enable-failure" {
                let err = apply_systemd(&demo_systemd_unit(true)).unwrap_err();
                assert_eq!(err.kind(), std::io::ErrorKind::Other);
                let root = std::path::PathBuf::from(std::env::var_os("META_ROOT").unwrap());
                let log = std::fs::read_to_string(root.join("systemctl.log")).unwrap();
                if mode == "reload-failure" {
                    assert!(!log.contains("enable --now"));
                } else {
                    assert!(log.contains("daemon-reload"));
                    assert!(log.contains("enable --now demo.service"));
                    assert!(!root.join("systemctl.active").exists());
                    assert!(!root.join("systemctl.enabled").exists());
                    assert!(!root.join(".config/systemd/user/demo.service").exists());
                    let home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
                    assert!(!home.join(".config/systemd/user/demo.service").exists());
                }
                return;
            }
        }

        for (mode, fail_stage) in [
            ("reload-failure", "daemon-reload"),
            ("enable-failure", "enable --now"),
        ] {
            let root = systemd_test_dir(mode);
            let home = systemd_test_dir(&format!("{mode}-home"));
            let fake_bin = install_fake_systemctl(&root, Some(fail_stage));
            run_systemd_test_child(TEST, mode, Some(&root), &home, &fake_bin, None);
            std::fs::remove_dir_all(root).unwrap();
            std::fs::remove_dir_all(home).unwrap();
        }
    }

    #[cfg(unix)]
    #[test]
    fn systemd_user_revert_propagates_systemctl_failures() {
        const TEST: &str = "wiring::tests::systemd_user_revert_propagates_systemctl_failures";
        if let Ok(mode) = std::env::var("ENVCTL_SYSTEMD_TEST_CHILD") {
            if mode == "disable-failure" || mode == "revert-reload-failure" {
                let root = std::path::PathBuf::from(std::env::var_os("META_ROOT").unwrap());
                let home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
                let canonical = root.join(".config/systemd/user/demo.service");
                let bridge = home.join(".config/systemd/user/demo.service");
                let unit = demo_systemd_unit(true);
                apply_systemd(&unit).unwrap();
                std::fs::write(
                    root.join("fail-stage"),
                    if mode == "disable-failure" {
                        "disable --now"
                    } else {
                        "daemon-reload"
                    },
                )
                .unwrap();
                if mode == "revert-reload-failure" {
                    std::fs::write(root.join("fail-once"), "").unwrap();
                }

                let err = revert_systemd(&unit).unwrap_err();
                assert_eq!(err.kind(), std::io::ErrorKind::Other);
                let log = std::fs::read_to_string(root.join("systemctl.log")).unwrap();
                assert!(log.contains("--user disable --now demo.service"));
                if mode == "disable-failure" {
                    assert!(canonical.exists());
                    assert_eq!(std::fs::read_link(bridge).unwrap(), canonical);
                } else {
                    assert!(log.matches("--user daemon-reload").count() >= 2);
                    assert!(canonical.exists());
                    assert_eq!(std::fs::read_link(bridge).unwrap(), canonical);
                    assert!(root.join("systemctl.active").exists());
                    assert!(root.join("systemctl.enabled").exists());
                    assert!(log.contains("--user enable demo.service"));
                    assert!(log.contains("--user start demo.service"));
                    assert!(err.to_string().contains("runtime state restored"));
                }
                return;
            }
        }

        for mode in ["disable-failure", "revert-reload-failure"] {
            let root = systemd_test_dir(mode);
            let home = systemd_test_dir(&format!("{mode}-home"));
            let fake_bin = install_fake_systemctl(&root, None);
            run_systemd_test_child(TEST, mode, Some(&root), &home, &fake_bin, None);
            std::fs::remove_dir_all(root).unwrap();
            std::fs::remove_dir_all(home).unwrap();
        }
    }

    #[cfg(unix)]
    #[test]
    fn systemd_user_revert_reload_failure_restores_active_disabled_state() {
        const MODE: &str = "revert-active-disabled-rollback";
        const TEST: &str =
            "wiring::tests::systemd_user_revert_reload_failure_restores_active_disabled_state";
        if std::env::var("ENVCTL_SYSTEMD_TEST_CHILD").as_deref() == Ok(MODE) {
            let root = std::path::PathBuf::from(std::env::var_os("META_ROOT").unwrap());
            let home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
            let unit = demo_systemd_unit(false);
            let canonical = root.join(".config/systemd/user/demo.service");
            let bridge = home.join(".config/systemd/user/demo.service");
            apply_systemd(&unit).unwrap();
            run_systemctl(&["--user", "start", "demo.service"]).unwrap();
            assert!(root.join("systemctl.active").exists());
            assert!(!root.join("systemctl.enabled").exists());
            std::fs::write(root.join("fail-stage"), "daemon-reload").unwrap();
            std::fs::write(root.join("fail-once"), "").unwrap();

            let err = revert_systemd(&unit).unwrap_err();
            assert_eq!(err.kind(), std::io::ErrorKind::Other);
            assert_eq!(std::fs::read_link(&bridge).unwrap(), canonical);
            assert!(canonical.exists());
            assert!(root.join("systemctl.active").exists());
            assert!(!root.join("systemctl.enabled").exists());

            let log = std::fs::read_to_string(root.join("systemctl.log")).unwrap();
            assert!(log.contains("--user is-enabled demo.service"));
            assert!(log.contains("--user is-active demo.service"));
            assert!(log.contains("--user disable demo.service"));
            assert!(log.contains("--user start demo.service"));
            assert!(!log.contains("--user enable demo.service"));
            assert!(err.to_string().contains("runtime state restored"));
            return;
        }

        let root = systemd_test_dir("revert-active-disabled-rollback-meta");
        let home = systemd_test_dir("revert-active-disabled-rollback-home");
        let fake_bin = install_fake_systemctl(&root, None);
        run_systemd_test_child(TEST, MODE, Some(&root), &home, &fake_bin, None);
        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_dir_all(home).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn systemd_user_revert_refuses_foreign_modified_canonical_content() {
        const MODE: &str = "foreign-canonical-revert";
        const TEST: &str =
            "wiring::tests::systemd_user_revert_refuses_foreign_modified_canonical_content";
        if std::env::var("ENVCTL_SYSTEMD_TEST_CHILD").as_deref() == Ok(MODE) {
            let root = std::path::PathBuf::from(std::env::var_os("META_ROOT").unwrap());
            let home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
            let unit = demo_systemd_unit(false);
            let canonical = root.join(".config/systemd/user/demo.service");
            let bridge = home.join(".config/systemd/user/demo.service");
            apply_systemd(&unit).unwrap();
            let log_before = std::fs::read_to_string(root.join("systemctl.log")).unwrap();
            std::fs::write(&canonical, "foreign operator edit\n").unwrap();

            let err = revert_systemd(&unit).unwrap_err();
            assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
            assert!(err.to_string().contains("foreign-modified canonical"));
            assert_eq!(
                std::fs::read_to_string(&canonical).unwrap(),
                "foreign operator edit\n"
            );
            assert_eq!(std::fs::read_link(&bridge).unwrap(), canonical);
            assert_eq!(
                std::fs::read_to_string(root.join("systemctl.log")).unwrap(),
                log_before,
                "ownership refusal must happen before systemctl mutation"
            );
            return;
        }

        let root = systemd_test_dir("foreign-canonical-revert-meta");
        let home = systemd_test_dir("foreign-canonical-revert-home");
        let fake_bin = install_fake_systemctl(&root, None);
        run_systemd_test_child(TEST, MODE, Some(&root), &home, &fake_bin, None);
        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_dir_all(home).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn systemd_user_revert_succeeds_after_remove_hook_predisables_unit() {
        const MODE: &str = "remove-hook-predisabled";
        const TEST: &str =
            "wiring::tests::systemd_user_revert_succeeds_after_remove_hook_predisables_unit";
        if std::env::var("ENVCTL_SYSTEMD_TEST_CHILD").as_deref() == Ok(MODE) {
            let root = std::path::PathBuf::from(std::env::var_os("META_ROOT").unwrap());
            let home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
            let unit = demo_systemd_unit(false);
            let canonical = root.join(".config/systemd/user/demo.service");
            let bridge = home.join(".config/systemd/user/demo.service");

            apply_systemd(&unit).unwrap();
            // Component remove hooks must stop before deleting their payload.
            // systemctl disable is documented to remove the discovery link;
            // generic revert immediately follows and must tolerate that state.
            run_systemctl(&["--user", "disable", "--now", "demo.service"]).unwrap();
            assert!(canonical.exists());
            assert!(!bridge.exists());

            revert_systemd(&unit).unwrap();
            assert!(!canonical.exists());
            assert!(!bridge.exists());
            let log = std::fs::read_to_string(root.join("systemctl.log")).unwrap();
            assert_eq!(log.matches("--user disable --now demo.service").count(), 1);
            assert!(log.contains("--user show --property=LoadState --value demo.service"));
            return;
        }

        let root = systemd_test_dir("remove-hook-predisabled-meta");
        let home = systemd_test_dir("remove-hook-predisabled-home");
        let fake_bin = install_fake_systemctl(&root, None);
        run_systemd_test_child(TEST, MODE, Some(&root), &home, &fake_bin, None);
        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_dir_all(home).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn systemd_user_presence_requires_the_owned_bridge_and_manager_discovery() {
        const MODE: &str = "presence-bridge-discovery";
        const TEST: &str =
            "wiring::tests::systemd_user_presence_requires_the_owned_bridge_and_manager_discovery";
        if std::env::var("ENVCTL_SYSTEMD_TEST_CHILD").as_deref() == Ok(MODE) {
            let root = std::path::PathBuf::from(std::env::var_os("META_ROOT").unwrap());
            let home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
            let unit = demo_systemd_unit(false);
            let canonical = root.join(".config/systemd/user/demo.service");
            let bridge = home.join(".config/systemd/user/demo.service");
            apply_systemd(&unit).unwrap();
            assert!(systemd_user_present(&unit));

            std::fs::remove_file(&bridge).unwrap();
            assert!(!systemd_user_present(&unit));
            std::os::unix::fs::symlink(&canonical, &bridge).unwrap();

            std::fs::write(root.join("fail-stage"), "show --property=FragmentPath").unwrap();
            assert!(!systemd_user_present(&unit));
            std::fs::remove_file(root.join("fail-stage")).unwrap();
            assert!(systemd_user_present(&unit));
            return;
        }

        let root = systemd_test_dir("presence-meta");
        let home = systemd_test_dir("presence-home");
        let fake_bin = install_fake_systemctl(&root, None);
        run_systemd_test_child(TEST, MODE, Some(&root), &home, &fake_bin, None);
        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn systemd_user_content_expands_the_explicit_meta_root_token() {
        let layout = MetaLayout::from_meta_root("/meta root/100%");
        let rendered = render_systemd_user_content(
            &layout,
            "Environment=\"META_ROOT=${META_ROOT}\"\nExecStart=\"${META_ROOT}/usr/bin/demo\"\n",
        )
        .unwrap();

        assert_eq!(
            rendered,
            "Environment=\"META_ROOT=/meta root/100%%\"\nExecStart=\"/meta root/100%%/usr/bin/demo\"\n"
        );
    }

    #[test]
    fn systemd_user_content_rejects_control_characters_in_meta_root() {
        let layout = MetaLayout::from_meta_root("/meta\nInjected=bad");
        let err = render_systemd_user_content(&layout, "ExecStart=${META_ROOT}/demo").unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn systemd_user_content_rejects_unit_syntax_characters_in_meta_root() {
        for root in ["/meta/\"quoted", "/meta/\\escaped"] {
            let layout = MetaLayout::from_meta_root(root);
            let err = render_systemd_user_content(&layout, "ExecStart=\"${META_ROOT}/demo\"")
                .unwrap_err();

            assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        }
    }

    #[test]
    fn systemd_user_content_rejects_a_relative_meta_root() {
        let layout = MetaLayout::from_meta_root("relative/meta");
        let err = render_systemd_user_content(&layout, "ExecStart=${META_ROOT}/demo").unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn systemd_user_presence_rejects_a_stale_meta_root() {
        let root = std::env::temp_dir().join(format!("envctl-systemd-presence-{}", now_epoch()));
        std::fs::create_dir_all(&root).unwrap();
        let unit_path = root.join("demo.service");
        let layout = MetaLayout::from_meta_root("/current/meta");
        let expected =
            render_systemd_user_content(&layout, "ExecStart=\"${META_ROOT}/usr/bin/demo\"\n")
                .unwrap();

        std::fs::write(&unit_path, "ExecStart=\"/retired/meta/usr/bin/demo\"\n").unwrap();
        assert!(!systemd_user_present_at(&unit_path, &expected));

        std::fs::write(&unit_path, "ExecStart=\"/current/meta/usr/bin/demo\"\n").unwrap();
        assert!(systemd_user_present_at(&unit_path, &expected));

        std::fs::remove_dir_all(root).unwrap();
    }
}
