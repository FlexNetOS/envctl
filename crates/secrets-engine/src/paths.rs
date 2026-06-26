//! XDG, env-ctl-namespaced paths. In envctl-managed shells, config/data/state live under
//! `$META_ROOT/.config/env-ctl`, `$META_ROOT/.local/share/env-ctl`, and
//! `$META_ROOT/.local/state/env-ctl`; runtime socket remains under `$XDG_RUNTIME_DIR`.
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Paths {
    pub config: PathBuf,
    pub data: PathBuf,
    pub state: PathBuf,
    pub runtime: PathBuf,
}

impl Paths {
    /// Resolve from the environment (`HOME` + the XDG base-dir vars).
    pub fn resolve() -> anyhow::Result<Paths> {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| anyhow::anyhow!("HOME is not set"))?;
        let base = |var: &str, default: PathBuf| -> PathBuf {
            std::env::var_os(var).map(PathBuf::from).unwrap_or(default)
        };
        let config = base("XDG_CONFIG_HOME", home.join(".config")).join("env-ctl");
        let data = base("XDG_DATA_HOME", home.join(".local/share")).join("env-ctl");
        let state = base("XDG_STATE_HOME", home.join(".local/state")).join("env-ctl");
        let runtime = match std::env::var_os("XDG_RUNTIME_DIR") {
            Some(r) => PathBuf::from(r).join("env-ctl"),
            None => state.clone(),
        };
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
}
