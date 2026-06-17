//! Self-update CORE (TASK-0019, Item 4a) — the non-printing decision/data logic for
//! `envctl self update`, ported from kasetto v3.2.0 `src/commands/self_update.rs`.
//!
//! Engine-first split: this module owns the *decisions and data* — fetch the latest
//! GitHub release, decide whether it is newer (`is_newer`), compute the host target triple
//! (`current_target`), and verify a downloaded asset against `checksums.txt`
//! (`verify_checksum`). It NEVER prints and NEVER mutates the filesystem. The CLI half
//! (`crates/cli/src/self_update.rs`) owns the download/extract/atomic-replace + progress
//! rendering, because replacing the running binary is a front-end (terminal) concern.
//!
//! No new C: the HTTP client is reused from `envctl_agent_env::source::http_client()`
//! (reqwest → rustls → ring, blocking). `sha2` is the same pure-Rust crate the trust
//! boundary already links.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use envctl_agent_env::source::http_client;

/// The GitHub repo self-update + the update-notifier resolve releases from. Retargeted
/// from kasetto's `pivoshenko/kasetto` to envctl's home.
pub const GITHUB_REPO: &str = "FlexNetOS/envctl";

/// A GitHub release (the subset self-update needs).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelfUpdateRelease {
    pub tag_name: String,
    pub assets: Vec<SelfUpdateAsset>,
}

/// One downloadable asset on a [`SelfUpdateRelease`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelfUpdateAsset {
    pub name: String,
    pub browser_download_url: String,
}

/// The `up_to_date` / `update_available` decision for `self update` + the cached notice.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelfUpdateCheck {
    pub current: String,
    pub latest: String,
    /// `"up_to_date"` | `"update_available"`.
    pub status: String,
}

/// Fetch the latest GitHub release for [`GITHUB_REPO`]. Blocking; reuses the agent-env
/// shared HTTP client (reqwest → rustls → ring). Non-printing.
pub fn fetch_latest_release() -> anyhow::Result<SelfUpdateRelease> {
    let url = format!("https://api.github.com/repos/{GITHUB_REPO}/releases/latest");
    let text = http_client()
        .map_err(|e| anyhow::anyhow!("failed to build HTTP client: {e}"))?
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .map_err(|e| anyhow::anyhow!("failed to fetch latest release: {e}"))?
        .error_for_status()
        .map_err(|e| anyhow::anyhow!("GitHub API error: {e}"))?
        .text()
        .map_err(|e| anyhow::anyhow!("failed to read release response: {e}"))?;
    let release: SelfUpdateRelease = serde_json::from_str(&text)
        .map_err(|e| anyhow::anyhow!("failed to parse release response: {e}"))?;
    Ok(release)
}

/// The host target triple used to match a release asset (kasetto verbatim).
pub fn current_target() -> String {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    match (arch, os) {
        ("aarch64", "macos") => "aarch64-apple-darwin".to_owned(),
        ("x86_64", "macos") => "x86_64-apple-darwin".to_owned(),
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu".to_owned(),
        ("aarch64", "linux") => "aarch64-unknown-linux-gnu".to_owned(),
        _ => format!("{arch}-unknown-{os}"),
    }
}

/// `true` when `latest` is a newer (major, minor, patch) tuple than `current` (kasetto verbatim).
pub fn is_newer(current: &str, latest: &str) -> bool {
    let parse = |v: &str| -> (u64, u64, u64) {
        let parts: Vec<u64> = v.split('.').filter_map(|s| s.parse().ok()).collect();
        (
            parts.first().copied().unwrap_or(0),
            parts.get(1).copied().unwrap_or(0),
            parts.get(2).copied().unwrap_or(0),
        )
    };
    parse(latest) > parse(current)
}

/// Verify that the SHA-256 of `data` matches the expected hash for `asset_name` found in the
/// checksums text (one `<hash>  <filename>` per line). Kasetto verbatim (typed-error port).
pub fn verify_checksum(data: &[u8], asset_name: &str, checksums_text: &str) -> anyhow::Result<()> {
    let expected = checksums_text
        .lines()
        .find(|line| line.ends_with(asset_name))
        .and_then(|line| line.split_whitespace().next())
        .ok_or_else(|| anyhow::anyhow!("checksum not found for {asset_name} in checksums.txt"))?;

    let actual = format!("{:x}", Sha256::digest(data));
    if actual != expected {
        return Err(anyhow::anyhow!(
            "checksum mismatch for {asset_name}: expected {expected}, got {actual}"
        ));
    }
    Ok(())
}

/// Compare the running binary's version against the latest release and return the
/// up-to-date / update-available decision. Non-printing; the data the CLI renders.
pub fn plan_self_update(release: &SelfUpdateRelease, current_version: &str) -> SelfUpdateCheck {
    let latest = release.tag_name.trim_start_matches('v').to_string();
    let status = if is_newer(current_version, &latest) {
        "update_available"
    } else {
        "up_to_date"
    };
    SelfUpdateCheck {
        current: current_version.to_string(),
        latest,
        status: status.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- is_newer golden vectors (kasetto src/commands/self_update.rs verbatim) ---
    #[test]
    fn is_newer_detects_patch_bump() {
        assert!(is_newer("1.0.0", "1.0.1"));
    }

    #[test]
    fn is_newer_detects_minor_bump() {
        assert!(is_newer("1.0.0", "1.1.0"));
    }

    #[test]
    fn is_newer_detects_major_bump() {
        assert!(is_newer("1.0.0", "2.0.0"));
    }

    #[test]
    fn is_newer_returns_false_for_same_version() {
        assert!(!is_newer("1.0.0", "1.0.0"));
    }

    #[test]
    fn is_newer_returns_false_for_older_version() {
        assert!(!is_newer("2.0.0", "1.0.0"));
    }

    #[test]
    fn current_target_returns_nonempty_string() {
        let target = current_target();
        assert!(!target.is_empty());
    }

    // --- verify_checksum golden vectors (kasetto verbatim; asset names retargeted to envctl) ---
    #[test]
    fn verify_checksum_passes_on_match() {
        let data = b"hello world";
        let hash = format!("{:x}", Sha256::digest(data));
        let checksums = format!("{hash}  envctl-aarch64-apple-darwin.tar.gz\n");
        verify_checksum(data, "envctl-aarch64-apple-darwin.tar.gz", &checksums).unwrap();
    }

    #[test]
    fn verify_checksum_fails_on_mismatch() {
        let data = b"hello world";
        let checksums = "0000000000000000000000000000000000000000000000000000000000000000  envctl-aarch64-apple-darwin.tar.gz\n";
        let result = verify_checksum(data, "envctl-aarch64-apple-darwin.tar.gz", checksums);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("checksum mismatch"));
    }

    #[test]
    fn verify_checksum_fails_when_asset_not_in_checksums() {
        let data = b"hello world";
        let checksums = "abcdef1234567890  envctl-x86_64-unknown-linux-gnu.tar.gz\n";
        let result = verify_checksum(data, "envctl-aarch64-apple-darwin.tar.gz", checksums);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("checksum not found"));
    }

    #[test]
    fn verify_checksum_handles_multiple_entries() {
        let data = b"binary content";
        let hash = format!("{:x}", Sha256::digest(data));
        let checksums = format!(
            "aaaa  envctl-x86_64-unknown-linux-gnu.tar.gz\n{hash}  envctl-aarch64-apple-darwin.tar.gz\nbbbb  envctl-x86_64-apple-darwin.tar.gz\n"
        );
        verify_checksum(data, "envctl-aarch64-apple-darwin.tar.gz", &checksums).unwrap();
    }

    #[test]
    fn plan_self_update_maps_status() {
        let mk = |tag: &str| SelfUpdateRelease {
            tag_name: tag.to_string(),
            assets: vec![],
        };
        // newer release → update_available; tag's leading `v` is trimmed.
        let c = plan_self_update(&mk("v2.0.0"), "1.0.0");
        assert_eq!(c.status, "update_available");
        assert_eq!(c.latest, "2.0.0");
        assert_eq!(c.current, "1.0.0");
        // same version → up_to_date.
        let c = plan_self_update(&mk("1.0.0"), "1.0.0");
        assert_eq!(c.status, "up_to_date");
    }
}
