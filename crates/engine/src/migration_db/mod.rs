//! migration_db — the database-backed, replayable migration automation engine.
//!
//! Repo-native implementation of the `envctl-db-nu-plugin-migration-automation-package`
//! contract (sql/001–003 + DATABASE_FEATURE_SPEC + AGENT_CONTROL_PROTOCOL): the 15 DDL
//! entities, both state machines, the hash-chained append-only event ledger, the approval
//! gate, the query views, and replay verification.
//!
//! Storage is redb (pure Rust, the fleet ledger store family — handoff ADR-0017, CodeDB),
//! NOT SQLite: the no-C trust boundary holds (`ci/gates/no-c.sh`). Entities are stored as
//! canonical JSON values keyed by id, with secondary index tables standing in for the SQL
//! indexes; the SQL views are the read functions in [`views`]. Constraints (CHECK enums,
//! UNIQUE, FKs) are enforced in code at the API layer — see [`api`].
//!
//! Engine doctrine: non-printing, sync, no clap. Every function returns typed values;
//! the CLI renders.

pub mod api;
pub mod machine;
pub mod model;
pub mod replay;
pub mod store;
pub mod views;

#[cfg(test)]
mod tests;

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub use api::{ApprovalDecision, OperationSpec, RunBundle, RunSpec, TargetSpec, ValidationSpec};
pub use model::*;
pub use replay::{
    FileHashCheck, NonDeterministicOperation, ReplayCheck, ReplayMode, ReplayOperationPlan,
    ReplayReport, ReplayRequest, RequiredApproval,
};
pub use views::{ApprovalRow, ReplayReadinessRow, RunStatusRow, ScorecardRow, TimelineRow};

/// Errors from the migration database. redb errors are carried as strings so the
/// API stays stable across redb major versions.
#[derive(Debug, thiserror::Error)]
pub enum MigrationDbError {
    #[error("store error: {0}")]
    Store(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("illegal transition: {kind} {from} -> {to}")]
    IllegalTransition {
        kind: &'static str,
        from: String,
        to: String,
    },
    #[error("approval required: {0}")]
    ApprovalRequired(String),
    #[error("json error: {0}")]
    Json(String),
    #[error("io error: {0}")]
    Io(String),
}

pub type Result<T> = std::result::Result<T, MigrationDbError>;

impl From<serde_json::Error> for MigrationDbError {
    fn from(e: serde_json::Error) -> Self {
        MigrationDbError::Json(e.to_string())
    }
}

impl From<std::io::Error> for MigrationDbError {
    fn from(e: std::io::Error) -> Self {
        MigrationDbError::Io(e.to_string())
    }
}

/// Open handle over the migration automation store.
pub struct MigrationDb {
    pub(crate) db: redb::Database,
    /// Path the store was opened at (reported by views / doctor surfaces).
    pub path: PathBuf,
}

impl MigrationDb {
    /// Open (creating if absent) the migration store at `path`. All tables are
    /// created up front so read transactions never race table existence.
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let db =
            redb::Database::create(path).map_err(|e| MigrationDbError::Store(e.to_string()))?;
        let this = Self {
            db,
            path: path.to_path_buf(),
        };
        store::create_all_tables(&this)?;
        Ok(this)
    }

    /// Resolve the default store path: `$ENVCTL_MIGRATION_DB`, else
    /// `$META_ROOT/var/envctl/migration.redb`, else `./.envctl/migration.redb`.
    pub fn default_path() -> PathBuf {
        if let Ok(p) = std::env::var("ENVCTL_MIGRATION_DB") {
            return PathBuf::from(p);
        }
        if let Ok(root) = std::env::var("META_ROOT") {
            return Path::new(&root).join("var/envctl/migration.redb");
        }
        PathBuf::from(".envctl/migration.redb")
    }
}

/// Hex-encoded SHA-256 (the estate's parity hash — comparable with `sha256sum`).
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let out = h.finalize();
    let mut s = String::with_capacity(64);
    for b in out {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Canonical JSON: serde_json's default map is sorted (BTreeMap), so a
/// to_string round-trip through `Value` is key-order deterministic.
pub fn canonical_json(v: &serde_json::Value) -> String {
    serde_json::to_string(v).unwrap_or_default()
}

/// UTC timestamp in the DDL's `strftime('%Y-%m-%dT%H:%M:%fZ')` shape.
pub fn now_utc() -> String {
    chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string()
}
