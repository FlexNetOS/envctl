//! Envctl-namespaced paths. In managed shells, config lives under
//! `$META_ROOT/.config/env-ctl`, durable data/state under
//! `$META_ROOT/var/xdg-{data,state}/env-ctl`, and runtime sockets beneath the
//! Yazelix-owned `$XDG_RUNTIME_DIR`.
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Paths {
    pub config: PathBuf,
    pub data: PathBuf,
    pub state: PathBuf,
    pub runtime: PathBuf,
}

impl Paths {
    /// Resolve from the environment (explicit XDG base-dir vars, else `META_ROOT`, else `HOME`).
    pub fn resolve() -> anyhow::Result<Paths> {
        let fallback_root = std::env::var_os("META_ROOT")
            .filter(|v| !v.is_empty())
            .or_else(|| std::env::var_os("HOME").filter(|v| !v.is_empty()))
            .map(PathBuf::from)
            .ok_or_else(|| anyhow::anyhow!("neither META_ROOT nor HOME is set"))?;
        let base = |var: &str, default: PathBuf| -> PathBuf {
            std::env::var_os(var)
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
                .unwrap_or(default)
        };
        let config = base("XDG_CONFIG_HOME", fallback_root.join(".config")).join("env-ctl");
        let data = base("XDG_DATA_HOME", fallback_root.join("var/xdg-data")).join("env-ctl");
        let state = base("XDG_STATE_HOME", fallback_root.join("var/xdg-state")).join("env-ctl");
        let runtime = base(
            "XDG_RUNTIME_DIR",
            fallback_root.join("var/lib/yazelix/runtime/xdg"),
        )
        .join("env-ctl");
        Ok(Paths {
            config,
            data,
            state,
            runtime,
        })
    }

    /// Explicit roots (for tests / a sandboxed instance).
    pub fn under(root: PathBuf) -> Paths {
        Paths {
            config: root.join("config"),
            data: root.join("data"),
            state: root.join("state"),
            runtime: root.join("run"),
        }
    }

    pub fn vault_db(&self) -> PathBuf {
        self.data.join("vault.db")
    }
    pub fn control_socket(&self) -> PathBuf {
        self.runtime.join("secretd.sock")
    }
    /// The daemon's runtime config file (`$META_ROOT/.config/env-ctl/secretd.toml`): store-backend selection
    /// and libSQL connection params (OI-1 (a), Phase 1). Optional — absent => in-memory defaults.
    pub fn config_file(&self) -> PathBuf {
        self.config.join("secretd.toml")
    }
    pub fn log_file(&self) -> PathBuf {
        self.state.join("env-ctl.log")
    }
    /// The remote relay EDGE's publicly-trusted server-cert directory
    /// (`$META_ROOT/.config/env-ctl/relay-tls/`, holding `cert.pem` + `key.pem`). The Phase-8 / F2 edge
    /// (`secretd::edge`) loads its rustls `ServerConfig` ONLY from here — never from the MITM-CA path
    /// (FS-S18 / FS-S25). Mirrors [`config_file`](Self::config_file): a sibling under `config`, not a
    /// new XDG root. The directory is operator-provisioned (ACME / operator-supplied cert); a missing
    /// directory makes the edge fail closed at startup.
    pub fn relay_tls_dir(&self) -> PathBuf {
        self.config.join("relay-tls")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::sync::{Mutex, MutexGuard, OnceLock};

    fn env_lock() -> MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn restore_var(key: &str, value: Option<OsString>) {
        match value {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }

    fn clear_path_env() -> Vec<(&'static str, Option<OsString>)> {
        let keys = [
            "HOME",
            "META_ROOT",
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
            "XDG_STATE_HOME",
            "XDG_RUNTIME_DIR",
        ];
        keys.into_iter()
            .map(|key| {
                let old = std::env::var_os(key);
                std::env::remove_var(key);
                (key, old)
            })
            .collect()
    }

    fn restore_path_env(old: Vec<(&'static str, Option<OsString>)>) {
        for (key, value) in old {
            restore_var(key, value);
        }
    }

    #[test]
    fn relay_tls_dir_is_under_config_sibling_of_secretd_toml() {
        let p = Paths::under(PathBuf::from("/tmp/envctl-test"));
        // relay-tls/ lives directly under the config root, exactly like secretd.toml — and is NOT
        // any data/state/runtime root, and NOT the MITM-CA location (which lives under `data`).
        assert_eq!(p.relay_tls_dir(), p.config.join("relay-tls"));
        assert_eq!(
            p.relay_tls_dir().parent(),
            p.config_file().parent(),
            "relay-tls/ and secretd.toml must share the config dir"
        );
        assert!(p.relay_tls_dir().starts_with(&p.config));
        assert!(!p.relay_tls_dir().starts_with(&p.data));
    }

    #[test]
    fn relay_tls_dir_resolves_under_env_ctl_config() {
        let p = Paths::under(PathBuf::from("/x"));
        assert!(p.relay_tls_dir().ends_with("config/relay-tls"));
    }

    #[test]
    fn resolve_defaults_to_meta_root_when_xdg_is_unset() {
        let _g = env_lock();
        let old = clear_path_env();
        std::env::set_var("HOME", "/home/real-user");
        std::env::set_var("META_ROOT", "/home/real-user/Desktop/meta");

        let p = Paths::resolve().expect("paths resolve from META_ROOT");
        assert_eq!(
            p.config,
            PathBuf::from("/home/real-user/Desktop/meta/.config/env-ctl")
        );
        assert_eq!(
            p.data,
            PathBuf::from("/home/real-user/Desktop/meta/var/xdg-data/env-ctl")
        );
        assert_eq!(
            p.state,
            PathBuf::from("/home/real-user/Desktop/meta/var/xdg-state/env-ctl")
        );
        assert_eq!(
            p.runtime,
            PathBuf::from("/home/real-user/Desktop/meta/var/lib/yazelix/runtime/xdg/env-ctl")
        );

        restore_path_env(old);
    }

    #[test]
    fn resolve_honors_explicit_xdg_overrides_before_meta_root() {
        let _g = env_lock();
        let old = clear_path_env();
        std::env::set_var("HOME", "/home/real-user");
        std::env::set_var("META_ROOT", "/home/real-user/Desktop/meta");
        std::env::set_var("XDG_CONFIG_HOME", "/custom/config");
        std::env::set_var("XDG_DATA_HOME", "/custom/data");
        std::env::set_var("XDG_STATE_HOME", "/custom/state");
        std::env::set_var("XDG_RUNTIME_DIR", "/sandbox/yazelix/xdg");

        let p = Paths::resolve().expect("paths resolve from explicit XDG roots");
        assert_eq!(p.config, PathBuf::from("/custom/config/env-ctl"));
        assert_eq!(p.data, PathBuf::from("/custom/data/env-ctl"));
        assert_eq!(p.state, PathBuf::from("/custom/state/env-ctl"));
        assert_eq!(p.runtime, PathBuf::from("/sandbox/yazelix/xdg/env-ctl"));

        restore_path_env(old);
    }
}
