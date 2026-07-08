//! db_ops — the single shared db entry point CLI and GUI both drive (REQ-059).
//!
//! The db query / refactor / deploy behaviour lives *here in the engine*, not in
//! `crates/cli/src/main.rs` and not in the GUI. Both front-ends construct a
//! [`DbSnapshot`] from a scope and call the identical methods below; neither owns
//! any db orchestration of its own. This is what keeps the two surfaces from
//! diverging (the same guarantee [`crate::Engine`] gives the component surface),
//! and it is what the workspace `db_parity` test pins.
//!
//! A snapshot scans the file index and builds the symbol index once, then serves
//! query/refactor/deploy off that shared in-memory state. Mutating surfaces still
//! return fail-closed *plans* ([`crate::db_refactor`] / [`crate::db_deploy`]); a
//! front-end must call the module-level `apply` with confirm+approval to mutate.

use crate::db::{Db, EnvRootRow, Result};
use crate::db_deploy::{self, DeployPlan, DeploySpec};
use crate::db_index::{FileIndex, ScanScope};
use crate::db_query::{self, QueryResult, QuerySpec};
use crate::db_refactor::{self, Approval, RefactorPlan, RootAliasSpec};
use crate::db_symbols::SymbolIndex;

/// An immutable, indexed snapshot of a scope. Built once, queried many times.
#[derive(Debug, Clone)]
pub struct DbSnapshot {
    scope: ScanScope,
    files: FileIndex,
    symbols: SymbolIndex,
}

impl DbSnapshot {
    /// Scan `scope` and build the file + symbol indexes. The single constructor
    /// both CLI and GUI use — no front-end rebuilds indexes on its own.
    pub fn open(scope: ScanScope) -> Result<Self> {
        let files = FileIndex::scan(&scope)?;
        let symbols = SymbolIndex::build(&files)?;
        Ok(Self {
            scope,
            files,
            symbols,
        })
    }

    /// The scope this snapshot was built from.
    pub fn scope(&self) -> &ScanScope {
        &self.scope
    }

    /// The file index (the `envctl db files --json` surface).
    pub fn files(&self) -> &FileIndex {
        &self.files
    }

    /// The symbol/occurrence index (`envctl db symbols --json`).
    pub fn symbols(&self) -> &SymbolIndex {
        &self.symbols
    }

    /// Run a deterministic query/preset against the snapshot.
    pub fn query(&self, spec: &QuerySpec) -> Result<QueryResult> {
        db_query::evaluate(spec, &self.files, &self.symbols)
    }

    /// Build a fail-closed root-alias refactor plan (never mutates).
    pub fn refactor_plan(&self, spec: &RootAliasSpec) -> Result<RefactorPlan> {
        db_refactor::plan(spec, &self.files, &self.symbols)
    }

    /// Render `plan`'s safe changes into the NEW tree at `spec.render_out`.
    /// Originals are never touched (the safe half of the mutating surface).
    /// Requires `spec.render_out` to be set. Returns the paths written.
    pub fn refactor_render(
        &self,
        plan: &RefactorPlan,
        spec: &RootAliasSpec,
    ) -> Result<Vec<String>> {
        db_refactor::render(plan, spec, &self.files)
    }

    /// Apply `plan`'s safe changes IN PLACE — the destructive path. Fail-closed:
    /// the engine refuses unless `confirm == true` AND `approval` is approved
    /// (R3 / human_approval_required). Returns the paths mutated.
    pub fn refactor_apply(
        &self,
        plan: &RefactorPlan,
        spec: &RootAliasSpec,
        confirm: bool,
        approval: Option<&Approval>,
    ) -> Result<Vec<String>> {
        db_refactor::apply(plan, spec, &self.files, confirm, approval)
    }

    /// Build a fail-closed deploy plan (never mutates).
    pub fn deploy_plan(&self, spec: &DeploySpec) -> Result<DeployPlan> {
        db_deploy::plan(spec, &self.files)
    }
}

/// Build the multi-root model (the `envctl db roots --json` surface) from the
/// observed + release-target roots. Front-end-agnostic; both CLI and GUI call it.
pub fn roots(observed: Option<String>, release: Option<String>, profile: &str) -> Vec<EnvRootRow> {
    Db::from_profiles(observed, release, profile)
        .roots()
        .to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_query::{QueryPreset, QuerySpec};
    use std::fs;

    fn tmp() -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("envctl-db-ops-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn snapshot_serves_query_and_plans_off_one_index() {
        let root = tmp();
        fs::write(root.join("cli.rs"), b"const R: &str = \"x\";\n").unwrap();
        fs::write(root.join("wrapper.sh"), b"cd $META_ROOT/bin\n").unwrap();

        let snap = DbSnapshot::open(ScanScope {
            root: root.display().to_string(),
            ..Default::default()
        })
        .unwrap();

        // Query preset works off the shared snapshot.
        let q = QuerySpec {
            table: None,
            filters: vec![],
            preset: Some(QueryPreset::RootMeta),
            target_profile: None,
            explain: false,
        };
        assert_eq!(snap.query(&q).unwrap().row_count, 1);

        // Refactor plan is fail-closed Plan mode.
        let rp = snap
            .refactor_plan(&RootAliasSpec {
                from: "META_ROOT".into(),
                to: "LIFE_OS_ROOT".into(),
                target_profile: None,
                scope: None,
                render_out: None,
            })
            .unwrap();
        assert_eq!(rp.files_touched, 1);
        assert!(!rp.approved);

        // roots surface holds both observed + release simultaneously.
        let rs = roots(Some("/o".into()), Some("/r".into()), "lifeos-release");
        assert_eq!(rs.len(), 2);

        let _ = fs::remove_dir_all(&root);
    }
}
