//! Meta-hosted filesystem layout resolver.
//!
//! `envctl` is the path-defining tool for the meta workspace: installs should
//! resolve through a single registry/layout surface and land under `$META_ROOT`
//! in a system-shaped tree (`.local/bin`, `.local/lib`, `.local/share`, ...),
//! not through hand-maintained host-global paths.  The legacy `.toolchains/`
//! tree is kept as a compatibility prefix while existing component manifests
//! are migrated.
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

    /// Meta's XDG-shaped local prefix: all envctl-owned exposure and state live
    /// below this tree.
    pub fn local(&self) -> PathBuf {
        self.meta_root.join(".local")
    }

    pub fn bin(&self) -> PathBuf {
        self.local().join("bin")
    }

    pub fn lib(&self) -> PathBuf {
        self.local().join("lib")
    }

    pub fn share(&self) -> PathBuf {
        self.local().join("share")
    }

    pub fn state(&self) -> PathBuf {
        self.local().join("state")
    }

    pub fn cache(&self) -> PathBuf {
        self.local().join("cache")
    }

    pub fn tmp(&self) -> PathBuf {
        self.local().join("tmp")
    }

    pub fn opt(&self) -> PathBuf {
        self.local().join("opt")
    }

    pub fn envctl_share(&self) -> PathBuf {
        self.share().join("envctl")
    }

    pub fn envctl_lib(&self) -> PathBuf {
        self.lib().join("envctl")
    }

    pub fn secrets_libexec(&self) -> PathBuf {
        self.envctl_lib().join("secrets/bin")
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
        self.envctl_share().join("repos")
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
    /// hand.  `.toolchains` is intentionally labeled compatibility-only: envctl
    /// can still expose it to older manifests, but new materialization happens
    /// through the canonical `.local` tree.
    pub fn entries(&self) -> Vec<LayoutEntry> {
        vec![
            LayoutEntry {
                key: "local",
                path: self.local(),
                kind: LayoutKind::Canonical,
                purpose: "meta-local prefix for envctl-managed installs",
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
                key: "share",
                path: self.share(),
                kind: LayoutKind::Canonical,
                purpose: "architecture-independent shared data",
            },
            LayoutEntry {
                key: "state",
                path: self.state(),
                kind: LayoutKind::Canonical,
                purpose: "meta-local persistent state",
            },
            LayoutEntry {
                key: "cache",
                path: self.cache(),
                kind: LayoutKind::Canonical,
                purpose: "meta-local cache data",
            },
            LayoutEntry {
                key: "tmp",
                path: self.tmp(),
                kind: LayoutKind::Canonical,
                purpose: "meta-local temporary workspace",
            },
            LayoutEntry {
                key: "opt",
                path: self.opt(),
                kind: LayoutKind::Canonical,
                purpose: "component prefixes under .local/opt/<component>",
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
                purpose: "envctl private library and libexec root",
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
                key: "legacy_toolchains",
                path: self.legacy_toolchains(),
                kind: LayoutKind::LegacyCompatibility,
                purpose: "compatibility prefix for manifests not yet migrated to .local",
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

    /// Create the canonical meta-local directory tree.
    ///
    /// This deliberately skips compatibility-only paths such as `.toolchains`:
    /// those may continue to exist on old machines, but envctl no longer treats
    /// them as the target organization for new installs.
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
            ("ENVCTL_LOCAL", self.local()),
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
    fn resolves_system_shaped_tree_inside_meta() {
        let l = MetaLayout::from_meta_root("/m");
        assert_eq!(l.local(), Path::new("/m/.local"));
        assert_eq!(l.bin(), Path::new("/m/.local/bin"));
        assert_eq!(l.lib(), Path::new("/m/.local/lib"));
        assert_eq!(l.share(), Path::new("/m/.local/share"));
        assert_eq!(l.state(), Path::new("/m/.local/state"));
        assert_eq!(l.cache(), Path::new("/m/.local/cache"));
        assert_eq!(l.tmp(), Path::new("/m/.local/tmp"));
        assert_eq!(l.opt(), Path::new("/m/.local/opt"));
        assert_eq!(l.repo_store(), Path::new("/m/.local/share/envctl/repos"));
        assert_eq!(l.envctl_lib(), Path::new("/m/.local/lib/envctl"));
        assert_eq!(
            l.secrets_libexec(),
            Path::new("/m/.local/lib/envctl/secrets/bin")
        );
        assert_eq!(
            l.seed_ca(),
            Path::new("/m/.local/share/envctl/secrets/ca/cognitum-ca.crt")
        );
        assert_eq!(
            l.component_prefix("ripgrep"),
            Path::new("/m/.local/opt/ripgrep")
        );
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
    fn exports_registry_path_variables() {
        let l = MetaLayout::from_meta_root("/meta");
        let exports = l.env_exports();
        assert!(exports
            .iter()
            .any(|(k, v)| *k == "ENVCTL_BIN_DIR" && v == Path::new("/meta/.local/bin")));
        assert!(exports
            .iter()
            .any(|(k, v)| *k == "ENVCTL_REPO_STORE"
                && v == Path::new("/meta/.local/share/envctl/repos")));
        assert!(exports.iter().any(|(k, v)| *k == "ENVCTL_SECRETS_BIN_DIR"
            && v == Path::new("/meta/.local/lib/envctl/secrets/bin")));
        assert!(exports.iter().any(|(k, v)| *k == "ENVCTL_SEED_CA"
            && v == Path::new("/meta/.local/share/envctl/secrets/ca/cognitum-ca.crt")));
    }

    #[test]
    fn registry_marks_toolchains_as_legacy_compatibility() {
        let l = MetaLayout::from_meta_root("/meta");
        let entries = l.entries();
        let legacy = entries
            .iter()
            .find(|entry| entry.key == "legacy_toolchains")
            .expect("legacy toolchains entry");
        assert_eq!(legacy.path, Path::new("/meta/.toolchains"));
        assert_eq!(legacy.kind, LayoutKind::LegacyCompatibility);
        assert!(!legacy.is_canonical());

        let legacy_secrets = entries
            .iter()
            .find(|entry| entry.key == "legacy_secrets_bin")
            .expect("legacy secrets bin entry");
        assert_eq!(
            legacy_secrets.path,
            Path::new("/meta/.toolchains/secrets/bin")
        );
        assert_eq!(legacy_secrets.kind, LayoutKind::LegacyCompatibility);

        let repo_store = entries
            .iter()
            .find(|entry| entry.key == "repo_store")
            .expect("repo store entry");
        assert_eq!(repo_store.kind, LayoutKind::Canonical);
        assert_eq!(
            repo_store.path,
            Path::new("/meta/.local/share/envctl/repos")
        );
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
            root.join(".local/opt/ripgrep")
        );
        assert!(l.ensure_component_prefix("../evil").is_err());
        assert!(l.ensure_component_prefix("nested/tool").is_err());
        assert!(l.ensure_component_prefix("").is_err());

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn expand_meta_path_retargets_home_tokens_to_meta_root() {
        let l = MetaLayout::from_meta_root("/meta");

        assert_eq!(
            l.expand_meta_path("$META_ROOT/.local/bin"),
            "/meta/.local/bin"
        );
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
