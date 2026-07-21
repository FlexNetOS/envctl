//! ARCHBP-042: envctl-only durable commit and bidirectional reconciliation.
//!
//! envctl is the exclusive production PostgreSQL/RuVector committer. This
//! worker drains the ordered, sequence-keyed export contracts that the
//! CodeDB seams land in staging (`codedb_outbox_export`,
//! `codedb_raw_objects`), commits them durably into envctl-owned
//! authoritative tables inside one transaction per batch — attaching the
//! exact commit identity (transaction id, WAL LSN, generation) and a
//! chained witness digest — and advances the reconciliation cursor
//! atomically with the commit, so acknowledgement can never precede
//! durability. Committed state projects back deterministically through the
//! redb owner's versioned UDS protocol. Grants deny every non-envctl role.

use serde::Serialize;
use std::path::Path;

/// Authoritative schema owned by envctl.
pub const AUTHORITATIVE_SCHEMA: &str = "lifeos_runtime";
/// Authoritative committed-record table (sequence-keyed, append-only).
pub const COMMITTED_TABLE: &str = "lifeos_runtime.envctl_committed_records";
/// Single-row reconciliation cursor advanced atomically with each commit.
pub const CURSOR_TABLE: &str = "lifeos_runtime.envctl_reconciliation_cursor";
/// The staging export contract this worker drains (from ARCHBP-002).
pub const STAGING_TABLE: &str = "codedb_outbox_export";
/// The exclusive committer role every grant converges on.
pub const COMMITTER_ROLE: &str = "lifeos_envctl";
/// Versioned redb-owner UDS protocol used for the return projection.
pub const OWNER_PROTOCOL_VERSION: &str = "flexnetos.redb-owner.v0";
/// Version stamped on the return projection keys.
pub const RETURN_PROJECTION_VERSION: &str = "envctl.return-projection.v0";

#[derive(Debug)]
pub struct CommitError(String);

impl CommitError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl std::fmt::Display for CommitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for CommitError {}

/// One durably committed record with its exact commit identity.
#[derive(Debug, Clone, Serialize)]
pub struct CommittedRecord {
    pub seq: i64,
    pub blob_sha256: String,
    pub contract_version: String,
    pub commit_txid: String,
    pub commit_lsn: String,
    pub generation: i64,
    pub witness: String,
}

/// The reconciliation cursor state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReconciliationCursor {
    pub acknowledged_seq: i64,
    pub generation: i64,
    pub last_witness: String,
}

/// Receipt of one drain run.
#[derive(Debug, Clone, Serialize)]
pub struct DrainReceipt {
    pub committed: Vec<i64>,
    pub skipped_existing: Vec<i64>,
    pub generation: i64,
    pub acknowledged_seq: i64,
}

/// Apply the envctl-exclusive role and grant policy: the authoritative
/// schema is writable only by [`COMMITTER_ROLE`]; every other role is
/// denied at the database, not by convention.
pub fn apply_role_and_grant_policy(admin_conn: &str) -> Result<(), CommitError> {
    let _ = admin_conn;
    Err(CommitError::new("apply_role_and_grant_policy is not implemented"))
}

/// Drain staging rows beyond the cursor in sequence order and commit them
/// durably. The cursor advance shares the commit transaction, so
/// acknowledgement is atomic with durability — never before it.
/// `fail_before_commit` is a test failpoint aborting the transaction after
/// all writes but before COMMIT.
pub fn drain_and_commit(
    conn: &str,
    max_batch: usize,
    fail_before_commit: bool,
) -> Result<DrainReceipt, CommitError> {
    let _ = (conn, max_batch, fail_before_commit);
    Err(CommitError::new("drain_and_commit is not implemented"))
}

/// Read back every committed record ordered by sequence.
pub fn committed_records(conn: &str) -> Result<Vec<CommittedRecord>, CommitError> {
    let _ = conn;
    Err(CommitError::new("committed_records is not implemented"))
}

/// Read the reconciliation cursor (zeroed if no commit has happened).
pub fn reconciliation_cursor(conn: &str) -> Result<ReconciliationCursor, CommitError> {
    let _ = conn;
    Err(CommitError::new("reconciliation_cursor is not implemented"))
}

/// Recompute and verify the witness chain over the committed records.
pub fn verify_witness_chain(records: &[CommittedRecord]) -> Result<(), CommitError> {
    let _ = records;
    Err(CommitError::new("verify_witness_chain is not implemented"))
}

/// Project database-controlled state back through the redb owner's
/// versioned UDS protocol. Deterministic: identical committed state yields
/// identical projected keys and values.
pub fn return_projection(
    conn: &str,
    owner_root: &Path,
) -> Result<Vec<(String, String)>, CommitError> {
    let _ = (conn, owner_root);
    Err(CommitError::new("return_projection is not implemented"))
}
