//! Meta-hosted filesystem layout resolver.
//!
//! `envctl` is the path-defining tool for the meta workspace: installs should
//! resolve through a single registry/layout surface and land under `$META_ROOT`
//! in a system-shaped tree (`.local/bin`, `.local/lib`, `.local/share`, ...),
//! not through hand-maintained host-global paths.  The legacy `.toolchains/`
//! tree is kept as a compatibility prefix while existing component manifests
//! are migrated.
use std::path::{Path, PathBuf};

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

    /// Stable environment variables exported by `envctl env --toolchains`.
    pub fn env_exports(&self) -> Vec<(&'static str, PathBuf)> {
        vec![
            ("ENVCTL_LOCAL", self.local()),
            ("ENVCTL_BIN_DIR", self.bin()),
            ("ENVCTL_LIB_DIR", self.lib()),
            ("ENVCTL_SHARE_DIR", self.share()),
            ("ENVCTL_STATE_DIR", self.state()),
            ("ENVCTL_CACHE_DIR", self.cache()),
            ("ENVCTL_TMP_DIR", self.tmp()),
            ("ENVCTL_OPT_DIR", self.opt()),
            ("ENVCTL_REPO_STORE", self.repo_store()),
            ("ENVCTL_LEGACY_TOOLCHAINS", self.legacy_toolchains()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::MetaLayout;
    use std::path::Path;

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
        assert_eq!(
            l.component_prefix("ripgrep"),
            Path::new("/m/.local/opt/ripgrep")
        );
        assert_eq!(l.legacy_toolchains(), Path::new("/m/.toolchains"));
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
    }
}
