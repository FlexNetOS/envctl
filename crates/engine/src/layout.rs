//! Meta-hosted filesystem layout resolver.
//!
//! `envctl` is the path-defining tool for the meta workspace: installs should
//! resolve through a single registry/layout surface and land under `$META_ROOT`
//! in a standard FHS-shaped tree (`usr/bin`, `usr/libexec`, `usr/lib`,
//! `usr/share`, `etc`, `var/lib`, `var/cache`, `var/log`, `run`, `tmp`, `opt`)
//! plus meta-home XDG surfaces (`.config`, `.local/share`, `.local/state`,
//! `.cache`) for tools that require HOME semantics.  The real user home is
//! compatibility-only; `.local/bin` and `.toolchains/` are legacy bridge
//! prefixes while existing component manifests are migrated.
use std::io;
use std::path::{Component, Path, PathBuf};

/// Whether a layout path is part of envctl's canonical target topology or a
/// compatibility surface kept for older manifests while they migrate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutKind {
    Canonical,
    LegacyCompatibility,
}

/// One named path in envctl's meta-hosted install registry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayoutEntry {
    pub key: &'static str,
    pub path: PathBuf,
    pub kind: LayoutKind,
    pub purpose: &'static str,
}

impl LayoutEntry {
    pub fn is_canonical(&self) -> bool {
        self.kind == LayoutKind::Canonical
    }
}

/// The canonical install topology for a single meta workspace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetaLayout {
    meta_root: PathBuf,
}

impl MetaLayout {
    pub fn from_meta_root(root: impl Into<PathBuf>) -> Self {
        Self {
            meta_root: root.into(),
        }
    }

    /// Resolve the canonical meta layout from an explicitly exported
    /// `META_ROOT`. Mutating ownership-sensitive surfaces use this constructor
    /// so a missing environment contract cannot silently target a historical
    /// checkout derived from `HOME`.
    pub fn from_env_required() -> io::Result<Self> {
        std::env::var_os("META_ROOT")
            .filter(|root| !root.is_empty())
            .map(Self::from_meta_root)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "META_ROOT is required for this envctl-owned path",
                )
            })
    }

    /// Resolve from `META_ROOT`, falling back to the historical local checkout
    /// convention. This is intentionally non-canonicalizing: envctl must be able
    /// to render paths for not-yet-created directories and worktree-relative
    /// fixtures without touching the filesystem.
    pub fn from_env_or_default() -> Self {
        if let Some(root) = std::env::var_os("META_ROOT").filter(|s| !s.is_empty()) {
            return Self::from_meta_root(root);
        }
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/root"));
        Self::from_meta_root(home.join("Desktop/meta"))
    }

    pub fn meta_root(&self) -> &Path {
        &self.meta_root
    }

    /// Expand the path tokens that envctl manifests are allowed to use for
    /// meta-hosted paths.
    ///
    /// Managed hooks run with `HOME=$META_ROOT`, so a legacy leading `~/`,
    /// `$HOME/`, or `${HOME}/` is deliberately resolved to the meta checkout
    /// here too.  The real user home remains available to hook bodies only as
    /// `ENVCTL_REAL_HOME` for explicit host-integration bridges.
    pub fn expand_meta_path(&self, p: &str) -> String {
        let root = self.meta_root.display();
        match p {
            "$META_ROOT" | "${META_ROOT}" | "$HOME" | "${HOME}" | "~" => root.to_string(),
            _ => {
                for prefix in ["$META_ROOT/", "${META_ROOT}/", "$HOME/", "${HOME}/", "~/"] {
                    if let Some(rest) = p.strip_prefix(prefix) {
                        return self.meta_root.join(rest).display().to_string();
                    }
                }
                p.to_string()
            }
        }
    }

    /// Meta-home local prefix. This remains available for XDG data/state and
    /// compatibility bridges, but envctl frontdoors and component prefixes do
    /// not canonically install here.
    pub fn local(&self) -> PathBuf {
        self.meta_root.join(".local")
    }

    pub fn usr(&self) -> PathBuf {
        self.meta_root.join("usr")
    }

    pub fn usr_bin(&self) -> PathBuf {
        self.usr().join("bin")
    }

    pub fn usr_lib(&self) -> PathBuf {
        self.usr().join("lib")
    }

    pub fn usr_libexec(&self) -> PathBuf {
        self.usr().join("libexec")
    }

    pub fn usr_share(&self) -> PathBuf {
        self.usr().join("share")
    }

    pub fn usr_sbin(&self) -> PathBuf {
        self.usr().join("sbin")
    }

    pub fn usr_lib64(&self) -> PathBuf {
        self.usr().join("lib64")
    }

    pub fn usr_include(&self) -> PathBuf {
        self.usr().join("include")
    }

    pub fn usr_src(&self) -> PathBuf {
        self.usr().join("src")
    }

    pub fn usr_games(&self) -> PathBuf {
        self.usr().join("games")
    }

    /// Pre-formatted/installed manual pages under the canonical share tree.
    pub fn usr_share_man(&self) -> PathBuf {
        self.usr_share().join("man")
    }

    /// Meta-hosted `/usr/local` prefix: locally-built installs that mirror the
    /// host `/usr/local` convention while staying meta-resident.
    pub fn usr_local(&self) -> PathBuf {
        self.usr().join("local")
    }

    pub fn usr_local_bin(&self) -> PathBuf {
        self.usr_local().join("bin")
    }

    pub fn usr_local_sbin(&self) -> PathBuf {
        self.usr_local().join("sbin")
    }

    pub fn usr_local_lib(&self) -> PathBuf {
        self.usr_local().join("lib")
    }

    pub fn usr_local_lib64(&self) -> PathBuf {
        self.usr_local().join("lib64")
    }

    pub fn usr_local_include(&self) -> PathBuf {
        self.usr_local().join("include")
    }

    pub fn usr_local_share(&self) -> PathBuf {
        self.usr_local().join("share")
    }

    pub fn etc(&self) -> PathBuf {
        self.meta_root.join("etc")
    }

    pub fn etc_envctl(&self) -> PathBuf {
        self.etc().join("envctl")
    }

    pub fn var(&self) -> PathBuf {
        self.meta_root.join("var")
    }

    pub fn var_lib(&self) -> PathBuf {
        self.var().join("lib")
    }

    pub fn var_lib_envctl(&self) -> PathBuf {
        self.var_lib().join("envctl")
    }

    /// Meta-owned Ollama model blob store.
    ///
    /// Ollama model layers are persistent data, not binaries, so they live under
    /// the canonical meta `var/lib` tree while the runner binary remains in the
    /// legacy `.toolchains/ollama` compatibility prefix until shimmy+ruvllm
    /// proves the replacement path.
    pub fn ollama_models(&self) -> PathBuf {
        self.var_lib().join("ollama/models")
    }

    pub fn var_cache(&self) -> PathBuf {
        self.var().join("cache")
    }

    pub fn var_cache_envctl(&self) -> PathBuf {
        self.var_cache().join("envctl")
    }

    pub fn var_log(&self) -> PathBuf {
        self.var().join("log")
    }

    pub fn var_log_envctl(&self) -> PathBuf {
        self.var_log().join("envctl")
    }

    pub fn var_tmp(&self) -> PathBuf {
        self.var().join("tmp")
    }

    pub fn run(&self) -> PathBuf {
        self.meta_root.join("run")
    }

    pub fn tmp_root(&self) -> PathBuf {
        self.meta_root.join("tmp")
    }

    pub fn opt_root(&self) -> PathBuf {
        self.meta_root.join("opt")
    }

    pub fn xdg_config_home(&self) -> PathBuf {
        self.meta_root.join(".config")
    }

    /// The one active systemd-user unit directory owned by envctl.
    ///
    /// It is deliberately derived from the meta XDG config root, never from
    /// the invoking user's `HOME` or ambient `XDG_CONFIG_HOME`.
    pub fn systemd_user_dir(&self) -> PathBuf {
        self.xdg_config_home().join("systemd/user")
    }

    pub fn xdg_data_home(&self) -> PathBuf {
        self.local().join("share")
    }

    pub fn xdg_state_home(&self) -> PathBuf {
        self.local().join("state")
    }

    pub fn xdg_cache_home(&self) -> PathBuf {
        self.meta_root.join(".cache")
    }

    pub fn local_bin(&self) -> PathBuf {
        self.local().join("bin")
    }

    pub fn local_lib(&self) -> PathBuf {
        self.local().join("lib")
    }

    pub fn local_cache(&self) -> PathBuf {
        self.local().join("cache")
    }

    pub fn local_tmp(&self) -> PathBuf {
        self.local().join("tmp")
    }

    pub fn local_opt(&self) -> PathBuf {
        self.local().join("opt")
    }

    /// Canonical executable frontdoor tree.
    pub fn bin(&self) -> PathBuf {
        self.usr_bin()
    }

    pub fn lib(&self) -> PathBuf {
        self.usr_lib()
    }

    pub fn share(&self) -> PathBuf {
        self.usr_share()
    }

    /// Envctl-owned persistent state root.
    pub fn state(&self) -> PathBuf {
        self.var_lib_envctl()
    }

    pub fn cache(&self) -> PathBuf {
        self.var_cache_envctl()
    }

    pub fn tmp(&self) -> PathBuf {
        self.var_tmp()
    }

    pub fn opt(&self) -> PathBuf {
        self.opt_root()
    }

    pub fn envctl_share(&self) -> PathBuf {
        self.share().join("envctl")
    }

    pub fn envctl_lib(&self) -> PathBuf {
        self.lib().join("envctl")
    }

    pub fn secrets_libexec(&self) -> PathBuf {
        self.usr_libexec().join("envctl/secrets/bin")
    }

    pub fn secrets_share(&self) -> PathBuf {
        self.envctl_share().join("secrets")
    }

    pub fn secrets_ca_dir(&self) -> PathBuf {
        self.secrets_share().join("ca")
    }

    pub fn seed_ca(&self) -> PathBuf {
        self.secrets_ca_dir().join("cognitum-ca.crt")
    }

    pub fn repo_store(&self) -> PathBuf {
        self.var_lib_envctl().join("repos")
    }

    pub fn component_prefix(&self, id: &str) -> PathBuf {
        self.opt().join(id)
    }

    /// Compatibility-only prefix for component manifests that still install
    /// manager-specific tool trees under `.toolchains`.
    pub fn legacy_toolchains(&self) -> PathBuf {
        self.meta_root.join(".toolchains")
    }

    pub fn legacy_secrets_bin(&self) -> PathBuf {
        self.legacy_toolchains().join("secrets/bin")
    }

    pub fn legacy_seed_ca(&self) -> PathBuf {
        self.legacy_toolchains().join("secrets/ca/cognitum-ca.crt")
    }

    /// The central path registry for envctl-owned installs and state.
    ///
    /// Callers should consume this registry instead of re-deriving path lists by
    /// hand.  `.local/bin`, `.local/lib`, `.local/cache`, `.local/tmp`,
    /// `.local/opt`, and `.toolchains` are intentionally labeled
    /// compatibility-only: envctl can still expose them to older manifests, but
    /// new materialization happens through the canonical FHS/XDG tree.
    pub fn entries(&self) -> Vec<LayoutEntry> {
        vec![
            LayoutEntry {
                key: "usr",
                path: self.usr(),
                kind: LayoutKind::Canonical,
                purpose: "meta usr prefix for envctl-managed installs",
            },
            LayoutEntry {
                key: "bin",
                path: self.bin(),
                kind: LayoutKind::Canonical,
                purpose: "executable frontdoor tree exposed on PATH",
            },
            LayoutEntry {
                key: "lib",
                path: self.lib(),
                kind: LayoutKind::Canonical,
                purpose: "shared libraries and native support files",
            },
            LayoutEntry {
                key: "libexec",
                path: self.usr_libexec(),
                kind: LayoutKind::Canonical,
                purpose: "private executables not directly exposed on PATH",
            },
            LayoutEntry {
                key: "share",
                path: self.share(),
                kind: LayoutKind::Canonical,
                purpose: "architecture-independent shared data",
            },
            LayoutEntry {
                key: "usr_sbin",
                path: self.usr_sbin(),
                kind: LayoutKind::Canonical,
                purpose: "system administration binaries exposed on PATH",
            },
            LayoutEntry {
                key: "usr_lib64",
                path: self.usr_lib64(),
                kind: LayoutKind::Canonical,
                purpose: "64-bit shared libraries on the library search path",
            },
            LayoutEntry {
                key: "usr_include",
                path: self.usr_include(),
                kind: LayoutKind::Canonical,
                purpose: "C/C++ headers on the compiler include path",
            },
            LayoutEntry {
                key: "usr_src",
                path: self.usr_src(),
                kind: LayoutKind::Canonical,
                purpose: "source trees for meta-managed builds",
            },
            LayoutEntry {
                key: "usr_games",
                path: self.usr_games(),
                kind: LayoutKind::Canonical,
                purpose: "FHS games prefix mirror",
            },
            LayoutEntry {
                key: "usr_share_man",
                path: self.usr_share_man(),
                kind: LayoutKind::Canonical,
                purpose: "installed manual pages on MANPATH",
            },
            LayoutEntry {
                key: "usr_local",
                path: self.usr_local(),
                kind: LayoutKind::Canonical,
                purpose: "meta-hosted /usr/local prefix for locally-built installs",
            },
            LayoutEntry {
                key: "usr_local_bin",
                path: self.usr_local_bin(),
                kind: LayoutKind::Canonical,
                purpose: "locally-built executables on PATH",
            },
            LayoutEntry {
                key: "usr_local_sbin",
                path: self.usr_local_sbin(),
                kind: LayoutKind::Canonical,
                purpose: "locally-built admin binaries on PATH",
            },
            LayoutEntry {
                key: "usr_local_lib",
                path: self.usr_local_lib(),
                kind: LayoutKind::Canonical,
                purpose: "locally-built shared libraries on the library search path",
            },
            LayoutEntry {
                key: "usr_local_lib64",
                path: self.usr_local_lib64(),
                kind: LayoutKind::Canonical,
                purpose: "locally-built 64-bit shared libraries on the library search path",
            },
            LayoutEntry {
                key: "usr_local_include",
                path: self.usr_local_include(),
                kind: LayoutKind::Canonical,
                purpose: "locally-built headers on the compiler include path",
            },
            LayoutEntry {
                key: "usr_local_share",
                path: self.usr_local_share(),
                kind: LayoutKind::Canonical,
                purpose: "locally-built architecture-independent shared data",
            },
            LayoutEntry {
                key: "etc",
                path: self.etc(),
                kind: LayoutKind::Canonical,
                purpose: "meta-hosted configuration root",
            },
            LayoutEntry {
                key: "etc_envctl",
                path: self.etc_envctl(),
                kind: LayoutKind::Canonical,
                purpose: "envctl configuration root",
            },
            LayoutEntry {
                key: "var",
                path: self.var(),
                kind: LayoutKind::Canonical,
                purpose: "meta-hosted variable data root",
            },
            LayoutEntry {
                key: "var_lib",
                path: self.var_lib(),
                kind: LayoutKind::Canonical,
                purpose: "persistent variable data root",
            },
            LayoutEntry {
                key: "state",
                path: self.state(),
                kind: LayoutKind::Canonical,
                purpose: "envctl persistent state root",
            },
            LayoutEntry {
                key: "var_cache",
                path: self.var_cache(),
                kind: LayoutKind::Canonical,
                purpose: "cache variable data root",
            },
            LayoutEntry {
                key: "cache",
                path: self.cache(),
                kind: LayoutKind::Canonical,
                purpose: "envctl cache data root",
            },
            LayoutEntry {
                key: "var_log",
                path: self.var_log(),
                kind: LayoutKind::Canonical,
                purpose: "meta-hosted log root",
            },
            LayoutEntry {
                key: "var_log_envctl",
                path: self.var_log_envctl(),
                kind: LayoutKind::Canonical,
                purpose: "envctl log root",
            },
            LayoutEntry {
                key: "tmp",
                path: self.tmp(),
                kind: LayoutKind::Canonical,
                purpose: "meta-hosted temporary workspace",
            },
            LayoutEntry {
                key: "run",
                path: self.run(),
                kind: LayoutKind::Canonical,
                purpose: "runtime files for meta-managed daemons",
            },
            LayoutEntry {
                key: "tmp_root",
                path: self.tmp_root(),
                kind: LayoutKind::Canonical,
                purpose: "short-lived scratch files under meta",
            },
            LayoutEntry {
                key: "opt",
                path: self.opt(),
                kind: LayoutKind::Canonical,
                purpose: "component prefixes under opt/<component>",
            },
            LayoutEntry {
                key: "xdg_config_home",
                path: self.xdg_config_home(),
                kind: LayoutKind::Canonical,
                purpose: "meta-home XDG config root",
            },
            LayoutEntry {
                key: "systemd_user_dir",
                path: self.systemd_user_dir(),
                kind: LayoutKind::Canonical,
                purpose: "authoritative envctl-owned systemd user unit directory",
            },
            LayoutEntry {
                key: "xdg_data_home",
                path: self.xdg_data_home(),
                kind: LayoutKind::Canonical,
                purpose: "meta-home XDG data root",
            },
            LayoutEntry {
                key: "xdg_state_home",
                path: self.xdg_state_home(),
                kind: LayoutKind::Canonical,
                purpose: "meta-home XDG state root",
            },
            LayoutEntry {
                key: "xdg_cache_home",
                path: self.xdg_cache_home(),
                kind: LayoutKind::Canonical,
                purpose: "meta-home XDG cache root",
            },
            LayoutEntry {
                key: "envctl_share",
                path: self.envctl_share(),
                kind: LayoutKind::Canonical,
                purpose: "envctl shared data root",
            },
            LayoutEntry {
                key: "envctl_lib",
                path: self.envctl_lib(),
                kind: LayoutKind::Canonical,
                purpose: "envctl private library root",
            },
            LayoutEntry {
                key: "secrets_libexec",
                path: self.secrets_libexec(),
                kind: LayoutKind::Canonical,
                purpose: "private installed secretd/secretctl binaries",
            },
            LayoutEntry {
                key: "secrets_share",
                path: self.secrets_share(),
                kind: LayoutKind::Canonical,
                purpose: "shared secrets-stack data such as trust roots",
            },
            LayoutEntry {
                key: "secrets_ca_dir",
                path: self.secrets_ca_dir(),
                kind: LayoutKind::Canonical,
                purpose: "pinned Cognitum Seed CA directory",
            },
            LayoutEntry {
                key: "repo_store",
                path: self.repo_store(),
                kind: LayoutKind::Canonical,
                purpose: "0700 source/build repo store for envctl add-repo",
            },
            LayoutEntry {
                key: "local",
                path: self.local(),
                kind: LayoutKind::LegacyCompatibility,
                purpose: "compatibility root for XDG data/state parent and host bridge",
            },
            LayoutEntry {
                key: "local_bin",
                path: self.local_bin(),
                kind: LayoutKind::LegacyCompatibility,
                purpose: "compatibility executable bridge for older PATH consumers",
            },
            LayoutEntry {
                key: "local_lib",
                path: self.local_lib(),
                kind: LayoutKind::LegacyCompatibility,
                purpose: "compatibility library prefix for old manifests",
            },
            LayoutEntry {
                key: "local_cache",
                path: self.local_cache(),
                kind: LayoutKind::LegacyCompatibility,
                purpose: "compatibility cache prefix for old manifests",
            },
            LayoutEntry {
                key: "local_tmp",
                path: self.local_tmp(),
                kind: LayoutKind::LegacyCompatibility,
                purpose: "compatibility temporary prefix for old manifests",
            },
            LayoutEntry {
                key: "local_opt",
                path: self.local_opt(),
                kind: LayoutKind::LegacyCompatibility,
                purpose: "compatibility component prefix for old manifests",
            },
            LayoutEntry {
                key: "legacy_toolchains",
                path: self.legacy_toolchains(),
                kind: LayoutKind::LegacyCompatibility,
                purpose: "compatibility prefix for manifests not yet migrated to the FHS layout",
            },
            LayoutEntry {
                key: "legacy_secrets_bin",
                path: self.legacy_secrets_bin(),
                kind: LayoutKind::LegacyCompatibility,
                purpose: "compatibility secretd/secretctl binary prefix",
            },
        ]
    }

    /// Canonical directories that envctl is allowed to materialize.
    pub fn canonical_dirs(&self) -> Vec<PathBuf> {
        self.entries()
            .into_iter()
            .filter(LayoutEntry::is_canonical)
            .map(|entry| entry.path)
            .collect()
    }

    /// Create the canonical meta-owned directory tree.
    ///
    /// This deliberately skips compatibility-only paths such as `.local/bin`
    /// and `.toolchains`: those may continue to exist on old machines, but
    /// envctl no longer treats them as the target organization for new installs.
    pub fn ensure_dirs(&self) -> io::Result<()> {
        for dir in self.canonical_dirs() {
            std::fs::create_dir_all(dir)?;
        }
        set_private_dir_permissions(&self.repo_store())?;
        Ok(())
    }

    /// Create and return the canonical prefix for one component.
    pub fn ensure_component_prefix(&self, id: &str) -> io::Result<PathBuf> {
        if !is_safe_component_id(id) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("component id '{id}' must be a single path component"),
            ));
        }
        let prefix = self.component_prefix(id);
        std::fs::create_dir_all(&prefix)?;
        Ok(prefix)
    }

    /// Stable environment variables exported by `envctl env --toolchains`.
    pub fn env_exports(&self) -> Vec<(&'static str, PathBuf)> {
        vec![
            ("ENVCTL_META_ROOT", self.meta_root.clone()),
            ("ENVCTL_LOCAL", self.local()),
            ("ENVCTL_LOCAL_BIN", self.local_bin()),
            ("ENVCTL_USR", self.usr()),
            ("ENVCTL_USR_BIN", self.usr_bin()),
            ("ENVCTL_USR_LIB", self.usr_lib()),
            ("ENVCTL_USR_LIBEXEC", self.usr_libexec()),
            ("ENVCTL_USR_SHARE", self.usr_share()),
            ("ENVCTL_USR_SBIN", self.usr_sbin()),
            ("ENVCTL_USR_LIB64", self.usr_lib64()),
            ("ENVCTL_USR_INCLUDE", self.usr_include()),
            ("ENVCTL_USR_SRC", self.usr_src()),
            ("ENVCTL_USR_GAMES", self.usr_games()),
            ("ENVCTL_USR_SHARE_MAN", self.usr_share_man()),
            ("ENVCTL_USR_LOCAL", self.usr_local()),
            ("ENVCTL_USR_LOCAL_BIN", self.usr_local_bin()),
            ("ENVCTL_USR_LOCAL_SBIN", self.usr_local_sbin()),
            ("ENVCTL_USR_LOCAL_LIB", self.usr_local_lib()),
            ("ENVCTL_USR_LOCAL_LIB64", self.usr_local_lib64()),
            ("ENVCTL_USR_LOCAL_INCLUDE", self.usr_local_include()),
            ("ENVCTL_USR_LOCAL_SHARE", self.usr_local_share()),
            ("ENVCTL_ETC", self.etc()),
            ("ENVCTL_ETC_DIR", self.etc_envctl()),
            ("ENVCTL_VAR", self.var()),
            ("ENVCTL_VAR_LIB", self.var_lib()),
            ("ENVCTL_VAR_CACHE", self.var_cache()),
            ("ENVCTL_VAR_LOG", self.var_log()),
            ("ENVCTL_RUN_DIR", self.run()),
            ("ENVCTL_BIN_DIR", self.bin()),
            ("ENVCTL_LIB_DIR", self.lib()),
            ("ENVCTL_SHARE_DIR", self.share()),
            ("ENVCTL_ENVCTL_SHARE_DIR", self.envctl_share()),
            ("ENVCTL_ENVCTL_LIB_DIR", self.envctl_lib()),
            ("ENVCTL_SECRETS_BIN_DIR", self.secrets_libexec()),
            ("ENVCTL_SEED_CA", self.seed_ca()),
            ("ENVCTL_STATE_DIR", self.state()),
            ("ENVCTL_CACHE_DIR", self.cache()),
            ("ENVCTL_TMP_DIR", self.tmp()),
            ("ENVCTL_OPT_DIR", self.opt()),
            ("ENVCTL_REPO_STORE", self.repo_store()),
            ("ENVCTL_XDG_CONFIG_HOME", self.xdg_config_home()),
            ("ENVCTL_SYSTEMD_USER_DIR", self.systemd_user_dir()),
            ("ENVCTL_XDG_DATA_HOME", self.xdg_data_home()),
            ("ENVCTL_XDG_STATE_HOME", self.xdg_state_home()),
            ("ENVCTL_XDG_CACHE_HOME", self.xdg_cache_home()),
            ("ENVCTL_LEGACY_TOOLCHAINS", self.legacy_toolchains()),
        ]
    }
}

fn is_safe_component_id(id: &str) -> bool {
    if id.is_empty() || id.contains('/') || id.contains('\\') {
        return false;
    }
    let mut comps = Path::new(id).components();
    matches!(
        (comps.next(), comps.next()),
        (Some(Component::Normal(_)), None)
    )
}

#[cfg(unix)]
fn set_private_dir_permissions(dir: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn set_private_dir_permissions(_dir: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{LayoutKind, MetaLayout};
    use std::path::{Path, PathBuf};

    #[test]
    fn resolves_standard_tree_inside_meta() {
        let l = MetaLayout::from_meta_root("/m");
        assert_eq!(l.local(), Path::new("/m/.local"));
        assert_eq!(l.bin(), Path::new("/m/usr/bin"));
        assert_eq!(l.lib(), Path::new("/m/usr/lib"));
        assert_eq!(l.share(), Path::new("/m/usr/share"));
        assert_eq!(l.state(), Path::new("/m/var/lib/envctl"));
        assert_eq!(l.cache(), Path::new("/m/var/cache/envctl"));
        assert_eq!(l.tmp(), Path::new("/m/var/tmp"));
        assert_eq!(l.opt(), Path::new("/m/opt"));
        assert_eq!(l.xdg_config_home(), Path::new("/m/.config"));
        assert_eq!(l.systemd_user_dir(), Path::new("/m/.config/systemd/user"));
        assert_eq!(l.xdg_data_home(), Path::new("/m/.local/share"));
        assert_eq!(l.xdg_state_home(), Path::new("/m/.local/state"));
        assert_eq!(l.xdg_cache_home(), Path::new("/m/.cache"));
        assert_eq!(l.repo_store(), Path::new("/m/var/lib/envctl/repos"));
        assert_eq!(l.ollama_models(), Path::new("/m/var/lib/ollama/models"));
        assert_eq!(l.envctl_lib(), Path::new("/m/usr/lib/envctl"));
        assert_eq!(
            l.secrets_libexec(),
            Path::new("/m/usr/libexec/envctl/secrets/bin")
        );
        assert_eq!(
            l.seed_ca(),
            Path::new("/m/usr/share/envctl/secrets/ca/cognitum-ca.crt")
        );
        assert_eq!(l.component_prefix("ripgrep"), Path::new("/m/opt/ripgrep"));
        assert_eq!(l.local_bin(), Path::new("/m/.local/bin"));
        assert_eq!(l.legacy_toolchains(), Path::new("/m/.toolchains"));
        assert_eq!(
            l.legacy_secrets_bin(),
            Path::new("/m/.toolchains/secrets/bin")
        );
        assert_eq!(
            l.legacy_seed_ca(),
            Path::new("/m/.toolchains/secrets/ca/cognitum-ca.crt")
        );
    }

    #[test]
    fn usr_mirror_completes_the_fhs_skeleton() {
        let l = MetaLayout::from_meta_root("/m");
        // The full /usr mirror: every standard subdir resolves under meta's usr.
        assert_eq!(l.usr_sbin(), Path::new("/m/usr/sbin"));
        assert_eq!(l.usr_lib64(), Path::new("/m/usr/lib64"));
        assert_eq!(l.usr_include(), Path::new("/m/usr/include"));
        assert_eq!(l.usr_src(), Path::new("/m/usr/src"));
        assert_eq!(l.usr_games(), Path::new("/m/usr/games"));
        assert_eq!(l.usr_share_man(), Path::new("/m/usr/share/man"));
        assert_eq!(l.usr_local(), Path::new("/m/usr/local"));
        assert_eq!(l.usr_local_bin(), Path::new("/m/usr/local/bin"));
        assert_eq!(l.usr_local_sbin(), Path::new("/m/usr/local/sbin"));
        assert_eq!(l.usr_local_lib(), Path::new("/m/usr/local/lib"));
        assert_eq!(l.usr_local_lib64(), Path::new("/m/usr/local/lib64"));
        assert_eq!(l.usr_local_include(), Path::new("/m/usr/local/include"));
        assert_eq!(l.usr_local_share(), Path::new("/m/usr/local/share"));

        // Every new mirror dir is Canonical, so ensure_dirs() materializes it.
        let entries = l.entries();
        for key in [
            "usr_sbin",
            "usr_lib64",
            "usr_include",
            "usr_src",
            "usr_games",
            "usr_share_man",
            "usr_local",
            "usr_local_bin",
            "usr_local_sbin",
            "usr_local_lib",
            "usr_local_lib64",
            "usr_local_include",
            "usr_local_share",
        ] {
            let e = entries
                .iter()
                .find(|entry| entry.key == key)
                .unwrap_or_else(|| panic!("missing layout entry: {key}"));
            assert_eq!(e.kind, LayoutKind::Canonical, "{key} must be canonical");
        }
    }

    #[test]
    fn exports_registry_path_variables() {
        let l = MetaLayout::from_meta_root("/meta");
        let exports = l.env_exports();
        assert!(exports
            .iter()
            .any(|(k, v)| *k == "ENVCTL_META_ROOT" && v == Path::new("/meta")));
        assert!(exports
            .iter()
            .any(|(k, v)| *k == "ENVCTL_BIN_DIR" && v == Path::new("/meta/usr/bin")));
        assert!(exports
            .iter()
            .any(|(k, v)| *k == "ENVCTL_LOCAL_BIN" && v == Path::new("/meta/.local/bin")));
        assert!(exports.iter().any(
            |(k, v)| *k == "ENVCTL_REPO_STORE" && v == Path::new("/meta/var/lib/envctl/repos")
        ));
        assert!(exports.iter().any(|(k, v)| *k == "ENVCTL_SECRETS_BIN_DIR"
            && v == Path::new("/meta/usr/libexec/envctl/secrets/bin")));
        assert!(exports.iter().any(|(k, v)| *k == "ENVCTL_SEED_CA"
            && v == Path::new("/meta/usr/share/envctl/secrets/ca/cognitum-ca.crt")));
        assert!(exports
            .iter()
            .any(|(k, v)| *k == "ENVCTL_XDG_CONFIG_HOME" && v == Path::new("/meta/.config")));
        assert!(exports.iter().any(|(k, v)| *k == "ENVCTL_SYSTEMD_USER_DIR"
            && v == Path::new("/meta/.config/systemd/user")));
    }

    #[test]
    fn registry_marks_legacy_prefixes_as_compatibility() {
        let l = MetaLayout::from_meta_root("/meta");
        let entries = l.entries();
        let legacy = entries
            .iter()
            .find(|entry| entry.key == "legacy_toolchains")
            .expect("legacy toolchains entry");
        assert_eq!(legacy.path, Path::new("/meta/.toolchains"));
        assert_eq!(legacy.kind, LayoutKind::LegacyCompatibility);
        assert!(!legacy.is_canonical());

        let local_bin = entries
            .iter()
            .find(|entry| entry.key == "local_bin")
            .expect("local bin entry");
        assert_eq!(local_bin.path, Path::new("/meta/.local/bin"));
        assert_eq!(local_bin.kind, LayoutKind::LegacyCompatibility);

        let repo_store = entries
            .iter()
            .find(|entry| entry.key == "repo_store")
            .expect("repo store entry");
        assert_eq!(repo_store.kind, LayoutKind::Canonical);
        assert_eq!(repo_store.path, Path::new("/meta/var/lib/envctl/repos"));
    }

    #[test]
    fn ensure_dirs_materializes_canonical_tree_only() {
        let root = tempdir("layout-materialize");
        let l = MetaLayout::from_meta_root(&root);

        l.ensure_dirs().unwrap();

        for dir in l.canonical_dirs() {
            assert!(dir.is_dir(), "canonical dir missing: {}", dir.display());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                l.repo_store().metadata().unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
        assert!(
            !l.local_bin().exists(),
            "compatibility .local/bin must not be materialized as canonical layout"
        );
        assert!(
            !l.legacy_toolchains().exists(),
            "compatibility .toolchains must not be materialized as canonical layout"
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn component_prefix_materializer_rejects_escaped_ids() {
        let root = tempdir("layout-component-prefix");
        let l = MetaLayout::from_meta_root(&root);

        assert_eq!(
            l.ensure_component_prefix("ripgrep").unwrap(),
            root.join("opt/ripgrep")
        );
        assert!(l.ensure_component_prefix("../evil").is_err());
        assert!(l.ensure_component_prefix("nested/tool").is_err());
        assert!(l.ensure_component_prefix("").is_err());

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn expand_meta_path_retargets_home_tokens_to_meta_root() {
        let l = MetaLayout::from_meta_root("/meta");

        assert_eq!(l.expand_meta_path("$META_ROOT/usr/bin"), "/meta/usr/bin");
        assert_eq!(
            l.expand_meta_path("${META_ROOT}/envctl/assets/scripts/demo.sh"),
            "/meta/envctl/assets/scripts/demo.sh"
        );
        assert_eq!(
            l.expand_meta_path("$HOME/.config/env-ctl"),
            "/meta/.config/env-ctl"
        );
        let tilde_local = ["~", ".local/share/env-ctl"].join("/");
        assert_eq!(
            l.expand_meta_path(&tilde_local),
            "/meta/.local/share/env-ctl"
        );
        assert_eq!(
            l.expand_meta_path("/etc/systemd/system/demo.service"),
            "/etc/systemd/system/demo.service"
        );
    }

    fn tempdir(label: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir =
            std::env::temp_dir().join(format!("envctl-{label}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
