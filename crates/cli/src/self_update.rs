//! `envctl self update` — CLI half (TASK-0019, Item 4b).
//!
//! The engine owns the *decisions* (`envctl_engine::self_update`: fetch the release, decide
//! `is_newer`, compute the target triple, verify the checksum). This module owns the
//! *front-end side effects* a running, self-replacing binary needs: download the matching
//! release asset, extract it, and atomically replace the current executable (with a `.old`
//! backup + restore-on-failure). Replacing the running binary is a terminal concern — there is
//! no GUI analog — so it lives in the CLI, not the engine.
//!
//! No new C: the blocking HTTP client is reused from `envctl_agent_env::source::http_client`
//! (reqwest → rustls → ring); `tar` + `flate2` (rust_backend) + `sha2` are the same pure-Rust
//! deps the trust boundary already links.

use std::fs;
use std::path::Path;

use envctl_agent_env::source::http_client;
use envctl_engine::{current_target, fetch_latest_release, is_newer, verify_checksum};

/// The known envctl binary names that may appear inside a release archive.
const ARCHIVE_BINARIES: &[&str] = &["envctl", "envctl-gui"];

/// Run `envctl self update`. Checks for a newer release; if one exists, downloads the matching
/// asset, verifies its checksum (when `checksums.txt` is published), and atomically replaces the
/// running binary. Prints progress (human) or a final JSON summary (`--json`).
pub fn run(as_json: bool) -> anyhow::Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");

    let release = fetch_latest_release()?;
    let latest_version = release.tag_name.trim_start_matches('v');

    if !is_newer(current_version, latest_version) {
        if as_json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "current_version": current_version,
                    "latest_version": latest_version,
                    "status": "up_to_date",
                }))?
            );
        } else {
            println!("Audited envctl {current_version} (already latest)");
        }
        return Ok(());
    }

    if !as_json {
        println!("Update available  {current_version} -> {latest_version}");
    }

    let target = current_target();
    let asset = release
        .assets
        .iter()
        .find(|a| a.name.contains(&target))
        .ok_or_else(|| anyhow::anyhow!("no release asset found for target: {target}"))?;

    let current_exe = std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("failed to locate current executable: {e}"))?;

    let checksums_asset = release.assets.iter().find(|a| a.name == "checksums.txt");

    // Phase 1: download the archive bytes.
    let body = download_archive(&asset.browser_download_url)?;
    if !as_json {
        println!("Downloaded envctl {latest_version}");
    }

    // Phase 2: verify checksum (only when the release publishes checksums.txt).
    if let Some(cs) = checksums_asset {
        let checksums_text = http_client()
            .map_err(|e| anyhow::anyhow!("failed to build HTTP client: {e}"))?
            .get(&cs.browser_download_url)
            .send()
            .and_then(|r| r.error_for_status())
            .and_then(|r| r.text())
            .map_err(|e| anyhow::anyhow!("failed to download checksums.txt: {e}"))?;
        verify_checksum(&body, &asset.name, &checksums_text)?;
        if !as_json {
            println!("Signature verified");
        }
    }

    // Phase 3: install (atomic replace + restore-on-failure).
    install_from_archive(&body, &current_exe)?;
    if !as_json {
        println!("Installed to {}", current_exe.display());
    }

    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "current_version": current_version,
                "latest_version": latest_version,
                "status": "updated",
            }))?
        );
    } else {
        println!("Updated envctl {current_version} -> {latest_version}");
        println!("  Run envctl --version to confirm");
    }
    Ok(())
}

/// Download the release archive bytes from `url`.
fn download_archive(url: &str) -> anyhow::Result<Vec<u8>> {
    let body = http_client()
        .map_err(|e| anyhow::anyhow!("failed to build HTTP client: {e}"))?
        .get(url)
        .send()?
        .error_for_status()
        .map_err(|e| anyhow::anyhow!("failed to download update: {e}"))?
        .bytes()?;
    Ok(body.to_vec())
}

/// Extract the archive into a tmp dir, replace `exe_path` with the new binary, and back up the
/// old one (restored on failure). Ports kasetto `install_from_archive`; the in-archive binary
/// names are retargeted to envctl (`envctl` / `envctl-gui`), with the tar-slip `..` guard kept.
fn install_from_archive(body: &[u8], exe_path: &Path) -> anyhow::Result<()> {
    let gz = flate2::read::GzDecoder::new(body);
    let mut archive = tar::Archive::new(gz);

    let tmp_dir = std::env::temp_dir().join(format!("envctl-update-{}", std::process::id()));
    fs::create_dir_all(&tmp_dir)?;

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        if path.to_string_lossy().contains("..") {
            let _ = fs::remove_dir_all(&tmp_dir);
            return Err(anyhow::anyhow!("unsafe archive path"));
        }
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if ARCHIVE_BINARIES.contains(&file_name.as_str()) {
            let target = tmp_dir.join(&file_name);
            entry.unpack(&target)?;
        }
    }

    // Replace whichever binary matches the running executable's stem (envctl or envctl-gui).
    let exe_stem = exe_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "envctl".to_string());
    let new_binary = tmp_dir.join(&exe_stem);
    let new_binary = if new_binary.exists() {
        new_binary
    } else {
        tmp_dir.join("envctl")
    };
    if !new_binary.exists() {
        let _ = fs::remove_dir_all(&tmp_dir);
        return Err(anyhow::anyhow!(
            "envctl binary not found in release archive"
        ));
    }

    let backup = exe_path.with_extension("old");
    fs::rename(exe_path, &backup)
        .map_err(|e| anyhow::anyhow!("failed to back up current binary: {e}"))?;

    match fs::copy(&new_binary, exe_path) {
        Ok(_) => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(exe_path, fs::Permissions::from_mode(0o755))?;
            }
            let _ = fs::remove_file(&backup);
        }
        Err(e) => {
            let _ = fs::rename(&backup, exe_path);
            let _ = fs::remove_dir_all(&tmp_dir);
            return Err(anyhow::anyhow!("failed to replace binary: {e}"));
        }
    }

    let _ = fs::remove_dir_all(&tmp_dir);
    Ok(())
}
