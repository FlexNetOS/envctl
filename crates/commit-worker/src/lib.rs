//! ARCHBP-042: envctl-only durable commit and bidirectional reconciliation.
//!
//! envctl is the exclusive production PostgreSQL/RuVector committer. This
//! worker drains the ordered, sequence-keyed export contracts that the
//! CodeDB seams land in staging (`codedb_outbox_export`), commits them
//! durably into envctl-owned authoritative tables inside one transaction
//! per batch — attaching the exact commit identity (transaction id, WAL
//! LSN, generation) and a chained witness digest — and advances the
//! reconciliation cursor atomically with the commit, so acknowledgement
//! can never precede durability. Committed state projects back
//! deterministically through the redb owner's versioned UDS protocol.
//! Grants deny every non-envctl role at the database, not by convention.

use postgres::{Client, NoTls};
use serde::Serialize;
use sha3::{
    digest::{ExtendableOutput, Update, XofReader},
    Shake256,
};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

pub mod activation;
mod embedding;
pub mod gates;

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
/// Blueprint-mandated domain separator for canonical witness records.
const WITNESS_DOMAIN: &[u8] = b"lifeos-witness-v1";
/// SHAKE256 witness size in bytes, matching the canonical PostgreSQL schema.
const WITNESS_BYTES: usize = 32;

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

fn internal(message: impl std::fmt::Display) -> CommitError {
    CommitError::new(message.to_string())
}

/// One durably committed record with its exact commit identity.
#[derive(Debug, Clone, Serialize)]
pub struct CommittedRecord {
    pub seq: i64,
    pub blob_sha256: String,
    pub contract_version: String,
    pub job_json: String,
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

/// Connect over a Unix socket only. TCP without the envctl TLS policy is
/// refused fail-closed: production credentials never travel unprotected.
fn connect(conn: &str) -> Result<Client, CommitError> {
    let has_unix_host = conn.split_whitespace().any(|part| {
        part.strip_prefix("host=")
            .is_some_and(|h| h.starts_with('/'))
    });
    if !has_unix_host {
        return Err(CommitError::new(
            "the commit worker requires an explicit Unix-socket host; \
             TCP requires the envctl TLS policy",
        ));
    }
    Client::connect(conn, NoTls)
        .map_err(|_| CommitError::new("PostgreSQL connection failed; connection details redacted"))
}

fn absorb_framed(hasher: &mut Shake256, value: &[u8]) {
    hasher.update(&(value.len() as u64).to_be_bytes());
    hasher.update(value);
}

fn witness_link(previous: &str, seq: i64, blob_sha256: &str, job_json: &str) -> String {
    let mut hasher = Shake256::default();
    absorb_framed(&mut hasher, WITNESS_DOMAIN);
    absorb_framed(&mut hasher, previous.as_bytes());
    absorb_framed(&mut hasher, &seq.to_be_bytes());
    absorb_framed(&mut hasher, blob_sha256.as_bytes());
    absorb_framed(&mut hasher, job_json.as_bytes());

    let mut output = [0_u8; WITNESS_BYTES];
    hasher.finalize_xof().read(&mut output);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut witness = String::with_capacity(WITNESS_BYTES * 2);
    for byte in output {
        witness.push(HEX[usize::from(byte >> 4)] as char);
        witness.push(HEX[usize::from(byte & 0x0f)] as char);
    }
    witness
}

/// Apply the envctl-exclusive role and grant policy: the authoritative
/// schema is writable only by [`COMMITTER_ROLE`]; every other role is
/// denied at the database, not by convention.
pub fn apply_role_and_grant_policy(admin_conn: &str) -> Result<(), CommitError> {
    let mut client = connect(admin_conn)?;
    client
        .batch_execute(&format!(
            "CREATE SCHEMA IF NOT EXISTS {AUTHORITATIVE_SCHEMA};\
             DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname='{COMMITTER_ROLE}')\
               THEN EXECUTE 'CREATE ROLE {COMMITTER_ROLE} NOLOGIN'; END IF; END $$;\
             CREATE TABLE IF NOT EXISTS {COMMITTED_TABLE} (\
                 seq BIGINT PRIMARY KEY,\
                 blob_sha256 TEXT NOT NULL,\
                 contract_version TEXT NOT NULL,\
                 job JSONB NOT NULL,\
                 commit_txid TEXT NOT NULL,\
                 commit_lsn TEXT NOT NULL,\
                 generation BIGINT NOT NULL,\
                 witness TEXT NOT NULL,\
                 committed_at TIMESTAMPTZ NOT NULL DEFAULT now()\
             );\
             CREATE TABLE IF NOT EXISTS {CURSOR_TABLE} (\
                 id BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (id),\
                 acknowledged_seq BIGINT NOT NULL,\
                 generation BIGINT NOT NULL,\
                 last_witness TEXT NOT NULL\
             );\
             INSERT INTO {CURSOR_TABLE} (id, acknowledged_seq, generation, last_witness)\
               VALUES (TRUE, 0, 0, '') ON CONFLICT (id) DO NOTHING;\
             REVOKE ALL ON SCHEMA {AUTHORITATIVE_SCHEMA} FROM PUBLIC;\
             REVOKE ALL ON {COMMITTED_TABLE} FROM PUBLIC;\
             REVOKE ALL ON {CURSOR_TABLE} FROM PUBLIC;\
             GRANT USAGE ON SCHEMA {AUTHORITATIVE_SCHEMA} TO {COMMITTER_ROLE};\
             GRANT SELECT, INSERT ON {COMMITTED_TABLE} TO {COMMITTER_ROLE};\
             GRANT SELECT, UPDATE ON {CURSOR_TABLE} TO {COMMITTER_ROLE};"
        ))
        .map_err(|e| internal(format!("applying role and grant policy: {e}")))
}

/// Drain staging rows beyond the cursor in sequence order and commit them
/// durably under the committer role. The cursor advance shares the commit
/// transaction, so acknowledgement is atomic with durability — never
/// before it. `fail_before_commit` is a test failpoint aborting the
/// transaction after all writes but before COMMIT.
pub fn drain_and_commit(
    conn: &str,
    max_batch: usize,
    fail_before_commit: bool,
) -> Result<DrainReceipt, CommitError> {
    if max_batch == 0 {
        return Err(CommitError::new("max_batch must be at least 1"));
    }
    let mut client = connect(conn)?;
    // Converge the committer's read grant on the staging contract; envctl
    // owns its own access, never the staging table itself.
    client
        .batch_execute(&format!(
            "DO $$ BEGIN IF to_regclass('{STAGING_TABLE}') IS NOT NULL THEN \
               EXECUTE 'GRANT SELECT ON {STAGING_TABLE} TO {COMMITTER_ROLE}';\
             END IF; END $$;"
        ))
        .map_err(|e| internal(format!("staging grant: {e} ({:?})", e.as_db_error())))?;

    let mut committed = Vec::new();
    let mut skipped_existing = Vec::new();
    let mut final_generation;
    let mut final_ack;
    loop {
        let mut tx = client.transaction().map_err(internal)?;
        // The whole batch executes AS the committer role: the grants are
        // exercised, not merely declared.
        tx.batch_execute(&format!("SET LOCAL ROLE {COMMITTER_ROLE}"))
            .map_err(internal)?;
        let cursor_row = tx
            .query_one(
                &format!("SELECT acknowledged_seq, generation, last_witness FROM {CURSOR_TABLE}"),
                &[],
            )
            .map_err(internal)?;
        let acknowledged: i64 = cursor_row.get(0);
        let generation: i64 = cursor_row.get(1);
        let mut previous_witness: String = cursor_row.get(2);
        final_generation = generation;
        final_ack = acknowledged;

        let staging = tx
            .query(
                &format!(
                    "SELECT seq, contract_version, blob_sha256, job::text FROM {STAGING_TABLE} \
                     WHERE seq > $1 ORDER BY seq LIMIT $2"
                ),
                &[&acknowledged, &(max_batch as i64)],
            )
            .map_err(|e| internal(format!("reading staging contract: {e}")))?;
        if staging.is_empty() {
            tx.rollback().map_err(internal)?;
            break;
        }
        let identity = tx
            .query_one(
                "SELECT pg_current_xact_id()::text, pg_current_wal_lsn()::text",
                &[],
            )
            .map_err(internal)?;
        let commit_txid: String = identity.get(0);
        let commit_lsn: String = identity.get(1);
        let batch_generation = generation + 1;
        let mut batch_last_seq = acknowledged;
        let mut batch_committed = Vec::new();
        let mut batch_skipped = Vec::new();
        for row in &staging {
            let seq: i64 = row.get(0);
            let contract_version: String = row.get(1);
            let blob_sha256: String = row.get(2);
            let staged_job_json: String = row.get(3);
            let job_json = embedding::enrich_job(&staged_job_json)?;
            let witness = witness_link(&previous_witness, seq, &blob_sha256, &job_json);
            let affected = tx
                .execute(
                    &format!(
                        "INSERT INTO {COMMITTED_TABLE} \
                         (seq, blob_sha256, contract_version, job, commit_txid, commit_lsn, \
                          generation, witness) \
                         VALUES ($1, $2, $3, $4::text::jsonb, $5, $6, $7, $8) \
                         ON CONFLICT (seq) DO NOTHING"
                    ),
                    &[
                        &seq,
                        &blob_sha256,
                        &contract_version,
                        &job_json,
                        &commit_txid,
                        &commit_lsn,
                        &batch_generation,
                        &witness,
                    ],
                )
                .map_err(|e| internal(format!("committing seq {seq}: {e}")))?;
            if affected == 1 {
                batch_committed.push(seq);
            } else {
                batch_skipped.push(seq);
            }
            previous_witness = witness;
            batch_last_seq = seq;
        }
        tx.execute(
            &format!(
                "UPDATE {CURSOR_TABLE} SET acknowledged_seq=$1, generation=$2, last_witness=$3"
            ),
            &[&batch_last_seq, &batch_generation, &previous_witness],
        )
        .map_err(internal)?;
        if fail_before_commit {
            drop(tx);
            return Err(CommitError::new(
                "injected failure before COMMIT: nothing durable, nothing acknowledged",
            ));
        }
        tx.commit().map_err(internal)?;
        committed.extend(batch_committed);
        skipped_existing.extend(batch_skipped);
    }
    Ok(DrainReceipt {
        committed,
        skipped_existing,
        generation: final_generation,
        acknowledged_seq: final_ack,
    })
}

/// Read back every committed record ordered by sequence.
pub fn committed_records(conn: &str) -> Result<Vec<CommittedRecord>, CommitError> {
    let mut client = connect(conn)?;
    let rows = client
        .query(
            &format!(
                "SELECT seq, blob_sha256, contract_version, job::text, commit_txid, \
                 commit_lsn, generation, witness FROM {COMMITTED_TABLE} ORDER BY seq"
            ),
            &[],
        )
        .map_err(internal)?;
    Ok(rows
        .into_iter()
        .map(|row| CommittedRecord {
            seq: row.get(0),
            blob_sha256: row.get(1),
            contract_version: row.get(2),
            job_json: row.get(3),
            commit_txid: row.get(4),
            commit_lsn: row.get(5),
            generation: row.get(6),
            witness: row.get(7),
        })
        .collect())
}

/// Read the reconciliation cursor (zeroed if no commit has happened).
pub fn reconciliation_cursor(conn: &str) -> Result<ReconciliationCursor, CommitError> {
    let mut client = connect(conn)?;
    let row = client
        .query_one(
            &format!("SELECT acknowledged_seq, generation, last_witness FROM {CURSOR_TABLE}"),
            &[],
        )
        .map_err(internal)?;
    Ok(ReconciliationCursor {
        acknowledged_seq: row.get(0),
        generation: row.get(1),
        last_witness: row.get(2),
    })
}

/// Recompute and verify the witness chain over the committed records.
pub fn verify_witness_chain(records: &[CommittedRecord]) -> Result<(), CommitError> {
    let mut previous = String::new();
    for record in records {
        let expected = witness_link(&previous, record.seq, &record.blob_sha256, &record.job_json);
        if expected != record.witness {
            return Err(CommitError::new(format!(
                "witness chain breaks at seq {}: expected {expected}, stored {}",
                record.seq, record.witness
            )));
        }
        previous = record.witness.clone();
    }
    Ok(())
}

/// Project database-controlled state back through the redb owner's
/// versioned UDS protocol. Deterministic: identical committed state yields
/// identical projected keys and values, in a fixed order.
pub fn return_projection(
    conn: &str,
    owner_root: &Path,
) -> Result<Vec<(String, String)>, CommitError> {
    let cursor = reconciliation_cursor(conn)?;
    let mut client = connect(conn)?;
    let count: i64 = client
        .query_one(&format!("SELECT count(*) FROM {COMMITTED_TABLE}"), &[])
        .map_err(internal)?
        .get(0);
    let pairs = vec![
        (
            "envctl/return-projection/version".to_string(),
            RETURN_PROJECTION_VERSION.to_string(),
        ),
        (
            "envctl/return-projection/acknowledged_seq".to_string(),
            cursor.acknowledged_seq.to_string(),
        ),
        (
            "envctl/return-projection/generation".to_string(),
            cursor.generation.to_string(),
        ),
        (
            "envctl/return-projection/last_witness".to_string(),
            cursor.last_witness.clone(),
        ),
        (
            "envctl/return-projection/committed_count".to_string(),
            count.to_string(),
        ),
    ];

    let token = std::fs::read_to_string(owner_root.join("owner.token"))
        .map_err(|e| internal(format!("owner token: {e}")))?
        .trim()
        .to_string();
    let stream = UnixStream::connect(owner_root.join("owner.sock"))
        .map_err(|e| internal(format!("owner socket: {e}")))?;
    let mut reader = BufReader::new(stream.try_clone().map_err(internal)?);
    let mut writer = stream;
    for (key, value) in &pairs {
        let request = serde_json::json!({
            "protocol_version": OWNER_PROTOCOL_VERSION,
            "token": token,
            "op": "put",
            "key": key,
            "value": value,
        });
        writeln!(writer, "{request}").map_err(internal)?;
        let mut response = String::new();
        reader.read_line(&mut response).map_err(internal)?;
        let response: serde_json::Value = serde_json::from_str(response.trim())
            .map_err(|e| internal(format!("owner response: {e}")))?;
        if response["ok"] != serde_json::Value::Bool(true) {
            return Err(CommitError::new(format!(
                "owner rejected the projection of {key}: {}",
                response["error"]
            )));
        }
    }
    Ok(pairs)
}

#[cfg(test)]
mod tests {
    use super::witness_link;

    #[test]
    fn shake256_witness_matches_independent_known_vector() {
        let witness = witness_link(
            "",
            42,
            "abababababababababababababababababababababababababababababababab",
            r#"{"model_name":"m","seq":42}"#,
        );
        assert_eq!(
            witness,
            "89391fba07cf80ee211febfbbdb6f8ae6f268a34a5c4690defa59ecc4a9ea252"
        );
    }

    #[test]
    fn witness_framing_prevents_field_boundary_ambiguity() {
        assert_ne!(
            witness_link("a", 1, "bc", "d"),
            witness_link("ab", 1, "c", "d")
        );
    }
}
