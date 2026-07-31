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
    Alternative, AptRepo, CdiSpec, DesktopEntry, NixConfLine, ResetGates, ShellRcBlock, Wiring,
};
use std::path::Path;
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
}
