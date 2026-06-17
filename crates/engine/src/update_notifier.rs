//! Background "new version available" notice — engine half (TASK-0019, Item 6).
//!
//! Ported from kasetto v3.2.0 `src/update_notifier.rs`. This module owns the *data*: a
//! detached background thread fetches the latest GitHub release and writes a small cache
//! file; the next run reads the cache and (if a newer version exists) the CLI prints a
//! single line at the end of the command.
//!
//! Engine-first: everything here is non-printing — it returns handles / `Option`s / typed
//! data. The CLI (`crates/cli/src/main.rs`) owns the TTY check + the rendered notice line.
//!
//! Best-effort: any failure (offline, rate-limit, IO) is silent. No new C: reuses the
//! self-update core's `fetch_latest_release` (agent-env HTTP client → rustls → ring).

use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::self_update::{fetch_latest_release, is_newer};
use envctl_agent_env::dirs::dirs_agent_env_cache;

const CACHE_FILE: &str = "update-check.json";
/// Refresh the cache at most once per 24h (matches npm/brew defaults).
const TTL_SECS: u64 = 24 * 60 * 60;

/// The cached "latest version" + when it was checked (the update-check cache file body).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateCacheEntry {
    pub checked_at: u64,
    pub latest_version: String,
}

/// Handle for a pending background check; the CLI can wait briefly so the cache is
/// populated before the notice is rendered. Detached threads die when the main thread
/// exits, so without this fast commands never persist results.
pub struct UpdateCheckHandle {
    rx: mpsc::Receiver<()>,
}

fn cache_path() -> Option<PathBuf> {
    // Test / scripted override (kasetto KASETTO_CACHE_DIR → envctl ENVCTL_CACHE_DIR).
    if let Ok(dir) = std::env::var("ENVCTL_CACHE_DIR") {
        if !dir.is_empty() {
            return Some(PathBuf::from(dir).join(CACHE_FILE));
        }
    }
    dirs_agent_env_cache().ok().map(|d| d.join(CACHE_FILE))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn read_cache(path: &std::path::Path) -> Option<UpdateCacheEntry> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

fn write_cache(path: &std::path::Path, entry: &UpdateCacheEntry) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    let text = serde_json::to_string(entry)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    fs::write(&tmp, text)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

fn cache_is_fresh(entry: &UpdateCacheEntry, now: u64) -> bool {
    now.saturating_sub(entry.checked_at) < TTL_SECS
}

/// Spawn a background thread to refresh the update-check cache.
///
/// Returns `None` when the cache is fresh (no work to do) or when the cache path can't be
/// resolved. Otherwise returns a handle so the CLI can optionally [`wait_for_check`] before
/// rendering the notice. Non-printing.
pub fn spawn_background_check() -> Option<UpdateCheckHandle> {
    let path = cache_path()?;
    let now = now_secs();
    if let Some(entry) = read_cache(&path) {
        if cache_is_fresh(&entry, now) {
            return None;
        }
    }

    let (tx, rx) = mpsc::channel();
    // Best-effort: this detached thread only refreshes a cosmetic update-check cache, so any
    // failure here is isolated to the thread and never aborts the real command.
    std::thread::spawn(move || {
        if let Ok(release) = fetch_latest_release() {
            let entry = UpdateCacheEntry {
                checked_at: now_secs(),
                latest_version: release.tag_name.trim_start_matches('v').to_string(),
            };
            let _ = write_cache(&path, &entry);
        }
        let _ = tx.send(());
    });
    Some(UpdateCheckHandle { rx })
}

/// Block up to `timeout` waiting for the background check to finish.
///
/// Detached threads are killed when `main` returns, so fast commands need this to give the
/// HTTP request a chance to complete and persist its result. On timeout we silently move on.
pub fn wait_for_check(handle: Option<UpdateCheckHandle>, timeout: Duration) {
    if let Some(h) = handle {
        let _ = h.rx.recv_timeout(timeout);
    }
}

/// Read the cached "latest version" entry for diagnostic display (e.g. `agent doctor`).
pub fn read_cached_entry() -> Option<UpdateCacheEntry> {
    let path = cache_path()?;
    read_cache(&path)
}

/// Current Unix timestamp; exposed so callers can compute cache age.
pub fn now_unix_secs() -> u64 {
    now_secs()
}

/// If the cache reports a newer version than `current`, return `(current, latest)`. The CLI
/// uses this to decide whether to render the end-of-run notice (gated by TTY + suppress).
/// Non-printing.
pub fn available_update(current: &str) -> Option<(String, String)> {
    let path = cache_path()?;
    let entry = read_cache(&path)?;
    if is_newer(current, &entry.latest_version) {
        Some((current.to_string(), entry.latest_version))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Serialize the env-poking tests so parallel threads don't observe each other's
    /// `ENVCTL_CACHE_DIR` override.
    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        use std::sync::{Mutex, OnceLock};
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    #[test]
    fn cache_round_trip() {
        let dir = std::env::temp_dir().join(format!("envctl-notifier-rt-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(CACHE_FILE);
        let entry = UpdateCacheEntry {
            checked_at: 1_700_000_000,
            latest_version: "9.9.9".into(),
        };
        write_cache(&path, &entry).unwrap();
        let back = read_cache(&path).unwrap();
        assert_eq!(back.checked_at, 1_700_000_000);
        assert_eq!(back.latest_version, "9.9.9");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn fresh_cache_within_ttl() {
        let entry = UpdateCacheEntry {
            checked_at: 1_000_000,
            latest_version: "1.0.0".into(),
        };
        // TTL boundary: still fresh at TTL-1, stale at exactly TTL (kasetto verbatim).
        assert!(cache_is_fresh(&entry, 1_000_000 + TTL_SECS - 1));
        assert!(!cache_is_fresh(&entry, 1_000_000 + TTL_SECS));
    }

    #[test]
    fn missing_cache_returns_none() {
        let dir =
            std::env::temp_dir().join(format!("envctl-notifier-missing-{}", std::process::id()));
        let path = dir.join(CACHE_FILE);
        assert!(read_cache(&path).is_none());
    }

    #[test]
    fn available_update_uses_env_override_and_compares() {
        let _g = env_lock();
        let dir = std::env::temp_dir().join(format!("envctl-notifier-av-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        std::env::set_var("ENVCTL_CACHE_DIR", &dir);

        // No cache file yet → no available update.
        assert!(available_update("1.0.0").is_none());

        // Cache says a newer version is out → reported.
        write_cache(
            &dir.join(CACHE_FILE),
            &UpdateCacheEntry {
                checked_at: now_secs(),
                latest_version: "2.0.0".into(),
            },
        )
        .unwrap();
        assert_eq!(
            available_update("1.0.0"),
            Some(("1.0.0".into(), "2.0.0".into()))
        );
        // Same/older current → no notice.
        assert!(available_update("2.0.0").is_none());
        assert!(available_update("3.0.0").is_none());

        std::env::remove_var("ENVCTL_CACHE_DIR");
        let _ = fs::remove_dir_all(&dir);
    }
}
