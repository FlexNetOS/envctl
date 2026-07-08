//! db — agent-first query / symbol-mapping / refactor / deploy surface over the
//! envctl catalog (GH FlexNetOS/envctl#414).
//!
//! REQ-050 scaffold: this module owns the shared error type, the multi-root
//! target model, and the [`Db`] façade. The heavier concerns live in sibling
//! modules and are wired in as REQ-051..REQ-061 land:
//!   - [`crate::db_index`]   — scalable file index (DbFileRow)          [REQ-052]
//!   - [`crate::db_symbols`] — symbol + occurrence index               [REQ-053]
//!   - [`crate::db_query`]   — deterministic query surface + presets    [REQ-054]
//!   - [`crate::db_refactor`]— root-alias refactor planner (plan/apply) [REQ-055]
//!   - [`crate::db_deploy`]  — safe hook/deploy automation              [REQ-056]
//!
//! Engine doctrine (mirrors [`crate::migration_db`]): non-printing, sync, no
//! clap. Every function returns typed values; the CLI/GUI render. Mutating
//! surfaces default to dry-run/plan and require explicit apply + confirmation.

use serde::{Deserialize, Serialize};

/// Errors from the db query/refactor/deploy layer. Kept stable and string-carried
/// at the edges so the API does not leak backend-specific error types.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("io error: {0}")]
    Io(String),
    #[error("index error: {0}")]
    Index(String),
    #[error("query error: {0}")]
    Query(String),
    #[error("unknown root: {0}")]
    UnknownRoot(String),
    #[error("refactor blocked: {0}")]
    RefactorBlocked(String),
    #[error("deploy blocked: {0}")]
    DeployBlocked(String),
    /// A seam that is scaffolded (REQ-050) but not yet implemented by its
    /// owning task. Fail-closed: callers get a typed error, never a panic.
    #[error("not implemented yet: {0}")]
    NotImplemented(&'static str),
}

pub type Result<T> = std::result::Result<T, DbError>;

impl From<std::io::Error> for DbError {
    fn from(e: std::io::Error) -> Self {
        DbError::Io(e.to_string())
    }
}

/// How much authority the db layer has over a given file. Fail-closed default
/// is [`MutablePolicy::ReadOnly`]; destructive policies are opt-in per row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MutablePolicy {
    #[default]
    ReadOnly,
    RenderOnly,
    OwnedApply,
    GuardedApply,
    Never,
}

/// The kind of environment root envctl models. `META_ROOT` stays a first-class
/// observed root; `LIFE_OS_ROOT` is the canonical new release-target spelling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvRootKind {
    MetaRoot,
    LifeOsRoot,
    RepoRoot,
    ToolchainRoot,
    XdgDataRoot,
    XdgStateRoot,
    XdgCacheRoot,
    LegacyRoot,
    Custom(String),
}

impl EnvRootKind {
    /// The canonical environment-variable spelling generated artifacts should
    /// use for this kind. `LIFEOS_ROOT` is accepted on input but normalizes to
    /// the canonical `LIFE_OS_ROOT` here (REQ-051 rule).
    pub fn canonical_var(&self) -> &str {
        match self {
            EnvRootKind::MetaRoot => "META_ROOT",
            EnvRootKind::LifeOsRoot => "LIFE_OS_ROOT",
            EnvRootKind::RepoRoot => "REPO_ROOT",
            EnvRootKind::ToolchainRoot => "TOOLCHAIN_ROOT",
            EnvRootKind::XdgDataRoot => "XDG_DATA_HOME",
            EnvRootKind::XdgStateRoot => "XDG_STATE_HOME",
            EnvRootKind::XdgCacheRoot => "XDG_CACHE_HOME",
            EnvRootKind::LegacyRoot => "TOOLCHAINS_LEGACY",
            EnvRootKind::Custom(name) => name,
        }
    }

    /// Classify a root variable name (any accepted spelling) into a kind.
    /// Accepts the `LIFEOS_ROOT` alias for `LifeOsRoot`.
    pub fn from_var(name: &str) -> EnvRootKind {
        match name
            .trim()
            .trim_start_matches('$')
            .trim_matches(|c| c == '{' || c == '}')
        {
            "META_ROOT" => EnvRootKind::MetaRoot,
            "LIFE_OS_ROOT" | "LIFEOS_ROOT" => EnvRootKind::LifeOsRoot,
            "REPO_ROOT" => EnvRootKind::RepoRoot,
            "TOOLCHAIN_ROOT" => EnvRootKind::ToolchainRoot,
            "XDG_DATA_HOME" => EnvRootKind::XdgDataRoot,
            "XDG_STATE_HOME" => EnvRootKind::XdgStateRoot,
            "XDG_CACHE_HOME" => EnvRootKind::XdgCacheRoot,
            other => EnvRootKind::Custom(other.to_string()),
        }
    }
}

/// Normalize a root variable spelling to its canonical form. `LIFEOS_ROOT`
/// becomes `LIFE_OS_ROOT`; unknown names pass through unchanged. This is the
/// single normalization seam refactor/deploy planners resolve token forms
/// against (REQ-051 rule; no blind `$META_ROOT` string rewrite).
pub fn normalize_root_var(name: &str) -> String {
    let kind = EnvRootKind::from_var(name);
    match kind {
        EnvRootKind::Custom(_) => name
            .trim()
            .trim_start_matches('$')
            .trim_matches(|c| c == '{' || c == '}')
            .to_string(),
        other => other.canonical_var().to_string(),
    }
}

/// The token forms an env root can appear as in files: `$VAR`, `${VAR}`, and
/// the bare name. Used by the refactor planner to find occurrences.
pub fn token_forms(var: &str) -> Vec<String> {
    let v = normalize_root_var(var);
    vec![format!("${v}"), format!("${{{v}}}"), v]
}

/// The role a root plays in a migration — the model holds current, declared,
/// and release-target roots simultaneously (no blind `$META_ROOT` rewrite).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvRootRole {
    ObservedCurrent,
    DeclaredCurrent,
    ReleaseTarget,
    MigrationSource,
    MigrationTarget,
    LegacyCompat,
}

/// One row in the multi-root target model. Populated by REQ-051.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvRootRow {
    pub root_id: String,
    pub kind: EnvRootKind,
    pub role: EnvRootRole,
    /// e.g. `["META_ROOT"]`, `["LIFE_OS_ROOT", "LIFEOS_ROOT"]` (normalize to the
    /// first spelling in generated artifacts).
    pub var_names: Vec<String>,
    pub absolute_path: Option<String>,
    /// `$META_ROOT`, `${META_ROOT}`, literal path, etc.
    pub token_forms: Vec<String>,
    pub source: String,
    pub precedence: u32,
    pub active: bool,
    pub target_profile: Option<String>,
    pub verifier_status: String,
}

/// The db façade. Holds the resolved index root(s) and offers the query /
/// symbol / refactor / deploy entry points that CLI and GUI share (REQ-059).
///
/// REQ-050 provides construction + the `roots` seam; later tasks fill in the
/// index-backed methods.
#[derive(Debug, Clone, Default)]
pub struct Db {
    roots: Vec<EnvRootRow>,
}

impl Db {
    /// Construct an empty db façade. REQ-051 adds `from_catalog` /
    /// `with_observed_roots` constructors that seed the multi-root model.
    pub fn new() -> Self {
        Self::default()
    }

    /// The current multi-root model rows (the `envctl db roots --json` surface).
    pub fn roots(&self) -> &[EnvRootRow] {
        &self.roots
    }

    /// Register a root row into the model. REQ-051 wires this from the catalog
    /// scan + declared/release-target profiles.
    pub fn push_root(&mut self, row: EnvRootRow) {
        self.roots.push(row);
    }

    /// Build the multi-root model holding the current observed root and a
    /// release-target root at the same time — the core #414 requirement:
    /// envctl keeps operating `$META_ROOT` while generating/validating a
    /// different `$LIFE_OS_ROOT` release layout. `observed`/`release` are the
    /// resolved absolute paths (if known).
    ///
    /// `META_ROOT` is recorded as [`EnvRootRole::ObservedCurrent`] and
    /// `LIFE_OS_ROOT` as [`EnvRootRole::ReleaseTarget`] under the given profile
    /// (e.g. `lifeos-release`). Any `LIFEOS_ROOT` spelling is normalized.
    pub fn from_profiles(
        observed: Option<String>,
        release: Option<String>,
        release_profile: &str,
    ) -> Self {
        let mut db = Self::new();
        db.push_root(EnvRootRow {
            root_id: "root-meta".into(),
            kind: EnvRootKind::MetaRoot,
            role: EnvRootRole::ObservedCurrent,
            var_names: vec![EnvRootKind::MetaRoot.canonical_var().to_string()],
            absolute_path: observed,
            token_forms: token_forms("META_ROOT"),
            source: "observed-env".into(),
            precedence: 100,
            active: true,
            target_profile: Some("current".into()),
            verifier_status: "observed".into(),
        });
        db.push_root(EnvRootRow {
            root_id: "root-lifeos".into(),
            kind: EnvRootKind::LifeOsRoot,
            role: EnvRootRole::ReleaseTarget,
            var_names: vec![EnvRootKind::LifeOsRoot.canonical_var().to_string()],
            absolute_path: release,
            token_forms: token_forms("LIFE_OS_ROOT"),
            source: "release-profile".into(),
            precedence: 90,
            active: true,
            target_profile: Some(release_profile.to_string()),
            verifier_status: "declared".into(),
        });
        db
    }

    /// Look up a root row by any of its variable spellings (alias-aware).
    pub fn root_by_var(&self, var: &str) -> Option<&EnvRootRow> {
        let kind = EnvRootKind::from_var(var);
        self.roots.iter().find(|r| r.kind == kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{db_deploy, db_index, db_query, db_refactor, db_symbols};

    #[test]
    fn scaffold_seams_are_wired_and_fail_closed() {
        // Façade + multi-root model.
        let mut db = Db::new();
        assert!(db.roots().is_empty());
        db.push_root(EnvRootRow {
            root_id: "meta".into(),
            kind: EnvRootKind::MetaRoot,
            role: EnvRootRole::ObservedCurrent,
            var_names: vec!["META_ROOT".into()],
            absolute_path: None,
            token_forms: vec!["$META_ROOT".into(), "${META_ROOT}".into()],
            source: "test".into(),
            precedence: 0,
            active: true,
            target_profile: None,
            verifier_status: "unverified".into(),
        });
        assert_eq!(db.roots().len(), 1);
        assert_eq!(db.roots()[0].kind, EnvRootKind::MetaRoot);

        // Index seams return empty, well-formed values (not panics).
        let files = db_index::FileIndex::scan(&db_index::ScanScope::default()).unwrap();
        let symbols = db_symbols::SymbolIndex::build(&files).unwrap();
        assert!(files.files().is_empty());
        assert!(symbols.symbols().is_empty());

        // Query seam returns an empty, well-formed result.
        let q = db_query::QuerySpec {
            table: Some(db_query::QueryTable::Roots),
            filters: vec![],
            preset: None,
            target_profile: None,
            explain: true,
        };
        let res = db_query::evaluate(&q, &files, &symbols).unwrap();
        assert_eq!(res.row_count, 0);

        // Mutating seams default to the fail-closed Plan mode.
        let rplan = db_refactor::plan(
            &db_refactor::RootAliasSpec {
                from: "META_ROOT".into(),
                to: "LIFE_OS_ROOT".into(),
                target_profile: Some("lifeos-release".into()),
                scope: None,
                render_out: None,
            },
            &files,
            &symbols,
        )
        .unwrap();
        assert_eq!(rplan.mode, db_refactor::ApplyMode::Plan);
        assert!(!rplan.approved);

        let dplan = db_deploy::plan(
            &db_deploy::DeploySpec {
                kind: "hooks".into(),
                target: "$LIFE_OS_ROOT".into(),
                stage_dir: None,
            },
            &files,
        )
        .unwrap();
        assert!(!dplan.approved);
    }

    #[test]
    fn multi_root_model_holds_observed_and_release_simultaneously() {
        let db = Db::from_profiles(
            Some("/home/u/meta".into()),
            Some("/home/u/lifeos".into()),
            "lifeos-release",
        );
        assert_eq!(db.roots().len(), 2);

        let meta = db.root_by_var("META_ROOT").expect("meta root");
        assert_eq!(meta.role, EnvRootRole::ObservedCurrent);
        assert_eq!(meta.absolute_path.as_deref(), Some("/home/u/meta"));

        let lifeos = db.root_by_var("LIFE_OS_ROOT").expect("lifeos root");
        assert_eq!(lifeos.role, EnvRootRole::ReleaseTarget);
        assert_eq!(lifeos.target_profile.as_deref(), Some("lifeos-release"));

        // Alias-aware: LIFEOS_ROOT resolves to the same LifeOsRoot row.
        assert_eq!(
            db.root_by_var("LIFEOS_ROOT").map(|r| &r.root_id),
            Some(&"root-lifeos".to_string())
        );
    }

    #[test]
    fn root_var_normalization_and_token_forms() {
        assert_eq!(normalize_root_var("LIFEOS_ROOT"), "LIFE_OS_ROOT");
        assert_eq!(normalize_root_var("${LIFEOS_ROOT}"), "LIFE_OS_ROOT");
        assert_eq!(normalize_root_var("$META_ROOT"), "META_ROOT");
        assert_eq!(normalize_root_var("MY_CUSTOM"), "MY_CUSTOM");
        assert_eq!(
            token_forms("LIFEOS_ROOT"),
            vec!["$LIFE_OS_ROOT", "${LIFE_OS_ROOT}", "LIFE_OS_ROOT"]
        );
    }
}
