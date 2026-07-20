//! XDG / home base-directory resolution — ported verbatim from kasetto v3.2.0
//! `src/fsops/dirs.rs` (ledger XC-03).
//!
//! The agent-NATIVE base-directory rules (`HOME`, the three `XDG_*_HOME` overrides with
//! their home-relative config/data/cache fallbacks) are kept byte-for-byte. Only the
//! kasetto-SELF-NAMED per-product subdirectory helpers are renamed to an envctl-neutral
//! `dirs_agent_env_{config,data,cache}` (appending `agent-env` instead of `kasetto`), so the
//! absorbed crate carries no foreign product identity. The XDG bases they build on are
//! unchanged. kasetto's `err(...)` channel maps onto [`crate::AgentEnvError::Message`].

use std::path::PathBuf;

use crate::{err, Result};

#[cfg(test)]
pub(crate) fn test_env_lock() -> std::sync::MutexGuard<'static, ()> {
    use std::sync::{Mutex, OnceLock};
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The user's home directory from `$HOME` (errors when unset).
pub fn dirs_home() -> Result<PathBuf> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| err("HOME is not set"))
}

/// [XDG Base Directory](https://specifications.freedesktop.org/basedir-spec/latest/) config home:
/// `XDG_CONFIG_HOME`, or `$HOME/.config` when unset or empty.
pub fn dirs_xdg_config_home() -> Result<PathBuf> {
    match std::env::var("XDG_CONFIG_HOME") {
        Ok(p) if !p.is_empty() => Ok(PathBuf::from(p)),
        _ => Ok(dirs_home()?.join(".config")),
    }
}

/// Per-user agent-env configuration directory: `$XDG_CONFIG_HOME/agent-env`.
///
/// (kasetto names this `kasetto`; renamed to the envctl-neutral `agent-env` — the XDG base is
/// agent-native and verbatim, only the product-self-named leaf differs.)
pub fn dirs_agent_env_config() -> Result<PathBuf> {
    Ok(dirs_xdg_config_home()?.join("agent-env"))
}

/// [XDG Base Directory](https://specifications.freedesktop.org/basedir-spec/latest/) data home:
/// `XDG_DATA_HOME`, or the XDG data fallback under `$HOME` when unset or empty.
pub fn dirs_xdg_data_home() -> Result<PathBuf> {
    match std::env::var("XDG_DATA_HOME") {
        Ok(p) if !p.is_empty() => Ok(PathBuf::from(p)),
        _ => Ok(dirs_home()?.join(".local/share")),
    }
}

/// Per-user agent-env data directory (lock file, etc.): `$XDG_DATA_HOME/agent-env`.
pub fn dirs_agent_env_data() -> Result<PathBuf> {
    Ok(dirs_xdg_data_home()?.join("agent-env"))
}

/// [XDG Base Directory](https://specifications.freedesktop.org/basedir-spec/latest/) cache home:
/// `XDG_CACHE_HOME`, or `$HOME/.cache` when unset or empty.
pub fn dirs_xdg_cache_home() -> Result<PathBuf> {
    match std::env::var("XDG_CACHE_HOME") {
        Ok(p) if !p.is_empty() => Ok(PathBuf::from(p)),
        _ => Ok(dirs_home()?.join(".cache")),
    }
}

/// Per-user agent-env cache directory (update-check cache, etc.): `$XDG_CACHE_HOME/agent-env`.
pub fn dirs_agent_env_cache() -> Result<PathBuf> {
    Ok(dirs_xdg_cache_home()?.join("agent-env"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_reads_env() {
        let _g = test_env_lock();
        let previous = std::env::var_os("HOME");
        std::env::set_var("HOME", "/home/tester");
        assert_eq!(dirs_home().unwrap(), PathBuf::from("/home/tester"));
        match previous {
            Some(value) => std::env::set_var("HOME", value),
            None => std::env::remove_var("HOME"),
        }
    }

    #[test]
    fn xdg_config_honors_override_only_when_non_empty() {
        let _g = test_env_lock();
        let previous_home = std::env::var_os("HOME");
        let previous_config = std::env::var_os("XDG_CONFIG_HOME");
        std::env::set_var("HOME", "/home/tester");

        std::env::set_var("XDG_CONFIG_HOME", "/custom/cfg");
        assert_eq!(
            dirs_xdg_config_home().unwrap(),
            PathBuf::from("/custom/cfg")
        );
        assert_eq!(
            dirs_agent_env_config().unwrap(),
            PathBuf::from("/custom/cfg/agent-env")
        );

        // Empty override falls back to `$HOME/.config`.
        std::env::set_var("XDG_CONFIG_HOME", "");
        assert_eq!(
            dirs_xdg_config_home().unwrap(),
            PathBuf::from("/home/tester/.config")
        );
        std::env::remove_var("XDG_CONFIG_HOME");
        assert_eq!(
            dirs_xdg_config_home().unwrap(),
            PathBuf::from("/home/tester/.config")
        );
        match previous_home {
            Some(value) => std::env::set_var("HOME", value),
            None => std::env::remove_var("HOME"),
        }
        match previous_config {
            Some(value) => std::env::set_var("XDG_CONFIG_HOME", value),
            None => std::env::remove_var("XDG_CONFIG_HOME"),
        }
    }

    #[test]
    fn xdg_data_and_cache_fallbacks() {
        let _g = test_env_lock();
        let previous_home = std::env::var_os("HOME");
        let previous_data = std::env::var_os("XDG_DATA_HOME");
        let previous_cache = std::env::var_os("XDG_CACHE_HOME");
        std::env::set_var("HOME", "/home/tester");
        std::env::remove_var("XDG_DATA_HOME");
        std::env::remove_var("XDG_CACHE_HOME");
        assert_eq!(
            dirs_xdg_data_home().unwrap(),
            PathBuf::from("/home/tester/.local/share")
        );
        assert_eq!(
            dirs_agent_env_data().unwrap(),
            PathBuf::from("/home/tester/.local/share/agent-env")
        );
        assert_eq!(
            dirs_xdg_cache_home().unwrap(),
            PathBuf::from("/home/tester/.cache")
        );
        assert_eq!(
            dirs_agent_env_cache().unwrap(),
            PathBuf::from("/home/tester/.cache/agent-env")
        );
        match previous_home {
            Some(value) => std::env::set_var("HOME", value),
            None => std::env::remove_var("HOME"),
        }
        match previous_data {
            Some(value) => std::env::set_var("XDG_DATA_HOME", value),
            None => std::env::remove_var("XDG_DATA_HOME"),
        }
        match previous_cache {
            Some(value) => std::env::set_var("XDG_CACHE_HOME", value),
            None => std::env::remove_var("XDG_CACHE_HOME"),
        }
    }
}
